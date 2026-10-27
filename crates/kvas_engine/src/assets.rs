//! Asset access. Everything reads through `Assets`, so the editor (loose files) and a shipped
//! game (single .pak) share the exact same loading code.

use std::borrow::Cow;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

const MAGIC: &[u8; 8] = b"KVASPAK1";
/// Multi-volume archives: a small index file plus data volumes of bounded size.
const MAGIC2: &[u8; 8] = b"KVASPAK2";
/// Default upper size of one data volume (a single file larger than this gets a volume of its own).
pub const DEFAULT_VOLUME_BYTES: u64 = 256 * 1024 * 1024;

/// Validated in-memory archive shared by cheap [`Assets::Pak`] clones.
/// Construct through [`Assets::open_pak`] or [`Assets::from_pak_bytes`].
pub struct PakData {
    source: PakSource,
    /// Path -> (volume, offset, length).
    index: HashMap<String, (usize, usize, usize)>,
}

enum PakSource {
    /// A single-file archive held in memory (`KVASPAK1`, or built with `from_pak_bytes`).
    Memory(Vec<u8>),
    /// Data volumes read on demand from disk (`KVASPAK2`), so huge games do not need huge RAM.
    Volumes(Vec<std::sync::Mutex<std::fs::File>>),
}

/// Unified asset source with root-relative, forward-slash paths.
///
/// Paths are normalized lexically and cannot escape the root. Directory reads
/// also reject symlinks that resolve outside the canonical root. Hidden entries
/// and symlinked directories are omitted from listing/packing.
#[derive(Clone)]
pub enum Assets {
    /// Files beneath a project assets directory; read on demand.
    Dir(PathBuf),
    /// Validated archive held in shared memory; no filesystem reads after opening.
    Pak(Arc<PakData>),
}

fn not_found(path: &str) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, format!("asset not found: {path}"))
}

impl Assets {
    /// Reads and validates a `KVASPAK1` archive.
    /// Reports `InvalidData` for malformed tables, duplicate/non-canonical paths,
    /// offsets into the table, or data ranges outside the file.
    pub fn open_pak(path: &Path) -> io::Result<Assets> {
        let bytes = std::fs::read(path)?;
        if bytes.len() >= 8 && &bytes[..8] == MAGIC2 {
            return Self::open_volumes(path, &bytes);
        }
        Self::from_pak_bytes(bytes)
    }

    /// Opens a `KVASPAK2` index and its data volumes (next to the index file).
    fn open_volumes(path: &Path, bytes: &[u8]) -> io::Result<Assets> {
        let bad = |what: &str| io::Error::new(io::ErrorKind::InvalidData, format!("corrupt pak index: {what}"));
        let mut pos = 8usize;
        let mut take = |n: usize| -> io::Result<&[u8]> {
            let slice = bytes.get(pos..pos.checked_add(n).ok_or_else(|| bad("overflow"))?).ok_or_else(|| bad("truncated"))?;
            pos += n;
            Ok(slice)
        };
        let u32_of = |b: &[u8]| u32::from_le_bytes(b.try_into().unwrap());
        let u64_of = |b: &[u8]| u64::from_le_bytes(b.try_into().unwrap());
        let volume_count = u32_of(take(4)?) as usize;
        let dir = path.parent().unwrap_or(Path::new("."));
        let mut files = Vec::new();
        let mut sizes = Vec::new();
        for _ in 0..volume_count {
            let name_len = u32_of(take(4)?) as usize;
            let name = String::from_utf8(take(name_len)?.to_vec()).map_err(|_| bad("volume name"))?;
            if name.contains(['/', '\\']) || name.is_empty() {
                return Err(bad("volume name"));
            }
            let size = u64_of(take(8)?);
            let file = std::fs::File::open(dir.join(&name)).map_err(|e| io::Error::new(e.kind(), format!("{name}: {e}")))?;
            if file.metadata()?.len() != size {
                return Err(bad(&format!("{name} has the wrong size (an incomplete copy?)")));
            }
            files.push(std::sync::Mutex::new(file));
            sizes.push(size);
        }
        let count = u32_of(take(4)?) as usize;
        let mut index = HashMap::new();
        for _ in 0..count {
            let name_len = u32_of(take(4)?) as usize;
            let name = String::from_utf8(take(name_len)?.to_vec()).map_err(|_| bad("name"))?;
            if normalize_path(&name).map_err(|_| bad("path"))? != name {
                return Err(bad("path"));
            }
            let volume = u32_of(take(4)?) as usize;
            let offset = u64_of(take(8)?);
            let len = u64_of(take(8)?);
            if volume >= sizes.len() || offset.checked_add(len).is_none_or(|end| end > sizes[volume]) {
                return Err(bad("entry outside its volume"));
            }
            if index.insert(name, (volume, offset as usize, len as usize)).is_some() {
                return Err(bad("duplicate entry"));
            }
        }
        Ok(Assets::Pak(Arc::new(PakData { source: PakSource::Volumes(files), index })))
    }

    /// Opens an owned archive buffer without an intermediate disk file.
    /// Uses the same validation as [`Self::open_pak`].
    pub fn from_pak_bytes(bytes: Vec<u8>) -> io::Result<Assets> {
        let bad = || io::Error::new(io::ErrorKind::InvalidData, "corrupt pak");
        if bytes.len() < 12 || &bytes[..8] != MAGIC {
            return Err(bad());
        }
        let u32_at = |p: usize| bytes.get(p..p + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()) as usize);
        let u64_at = |p: usize| bytes.get(p..p.checked_add(8)?).and_then(|b| usize::try_from(u64::from_le_bytes(b.try_into().ok()?)).ok());
        let count = u32_at(8).ok_or_else(bad)?;
        if count > (bytes.len() - 12) / 20 {
            return Err(bad());
        }
        let mut pos = 12;
        let mut index = HashMap::new();
        for _ in 0..count {
            let name_len = u32_at(pos).ok_or_else(bad)?;
            let start = pos.checked_add(4).ok_or_else(bad)?;
            let end = start.checked_add(name_len).ok_or_else(bad)?;
            let name = bytes.get(start..end).ok_or_else(bad)?;
            let name = String::from_utf8(name.to_vec()).map_err(|_| bad())?;
            if normalize_path(&name).map_err(|_| bad())? != name {
                return Err(bad());
            }
            pos = end;
            let offset = u64_at(pos).ok_or_else(bad)?;
            let len = u64_at(pos + 8).ok_or_else(bad)?;
            pos += 16;
            if offset.checked_add(len).is_none_or(|end| end > bytes.len()) {
                return Err(bad());
            }
            if index.insert(name, (0usize, offset, len)).is_some() {
                return Err(bad());
            }
        }
        if index.values().any(|&(_, offset, _)| offset < pos) {
            return Err(bad());
        }
        Ok(Assets::Pak(Arc::new(PakData { source: PakSource::Memory(bytes), index })))
    }

    /// Reads a root-relative asset into an owned byte buffer.
    /// Errors preserve I/O kinds (`NotFound`, `InvalidInput`, etc.).
    pub fn read(&self, path: &str) -> io::Result<Vec<u8>> {
        self.read_bytes(path).map(Cow::into_owned)
    }

    /// Reads an asset, borrowing archive bytes without copying when packed.
    /// Directory assets return an owned buffer. The borrow keeps the source alive.
    pub fn read_bytes(&self, path: &str) -> io::Result<Cow<'_, [u8]>> {
        let path = normalize_path(path)?;
        match self {
            Assets::Dir(root) => std::fs::read(resolve_file(root, &path)?).map(Cow::Owned),
            Assets::Pak(p) => {
                let &(volume, off, len) = p.index.get(&path).ok_or_else(|| not_found(&path))?;
                match &p.source {
                    PakSource::Memory(bytes) => Ok(Cow::Borrowed(&bytes[off..off + len])),
                    PakSource::Volumes(files) => {
                        use std::io::{Read, Seek, SeekFrom};
                        let mut file = files[volume].lock().map_err(|_| io::Error::other("pak volume lock poisoned"))?;
                        file.seek(SeekFrom::Start(off as u64))?;
                        let mut data = vec![0u8; len];
                        file.read_exact(&mut data)?;
                        Ok(Cow::Owned(data))
                    }
                }
            }
        }
    }

    /// Reads UTF-8 text, returning `InvalidData` for invalid encoding.
    pub fn read_string(&self, path: &str) -> io::Result<String> {
        String::from_utf8(self.read(path)?).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Last modification time, used for hot reload. Always `None` inside a pak.
    pub fn modified(&self, path: &str) -> Option<SystemTime> {
        let path = normalize_path(path).ok()?;
        match self {
            Assets::Dir(root) => std::fs::metadata(resolve_file(root, &path).ok()?).and_then(|m| m.modified()).ok(),
            Assets::Pak(_) => None,
        }
    }

    /// All asset paths (forward slashes, relative to the root), sorted.
    /// I/O failures return an empty list; use [`Self::try_list`] when packing/exporting.
    pub fn list(&self) -> Vec<String> {
        self.try_list().unwrap_or_default()
    }

    /// Lists assets while preserving directory traversal errors.
    /// Hidden entries and symlinks are skipped consistently with packing.
    pub fn try_list(&self) -> io::Result<Vec<String>> {
        let mut out: Vec<String> = match self {
            Assets::Dir(root) => {
                let mut v = Vec::new();
                walk(root, root, &mut v)?;
                v
            }
            Assets::Pak(p) => p.index.keys().cloned().collect(),
        };
        out.sort();
        Ok(out)
    }

    /// Whether an asset is a readable file. Invalid paths and I/O errors yield `false`.
    pub fn exists(&self, path: &str) -> bool {
        self.byte_len(path).is_ok()
    }

    /// Asset size in bytes without reading its contents.
    pub fn byte_len(&self, path: &str) -> io::Result<u64> {
        let path = normalize_path(path)?;
        match self {
            Assets::Dir(root) => {
                let metadata = std::fs::metadata(resolve_file(root, &path)?)?;
                if !metadata.is_file() {
                    return Err(not_found(&path));
                }
                Ok(metadata.len())
            }
            Assets::Pak(p) => p.index.get(&path).map(|&(_, _, len)| len as u64).ok_or_else(|| not_found(&path)),
        }
    }

    /// Sorted paths below a directory prefix (not a partial filename).
    /// Empty prefix lists everything; invalid prefixes return an empty list.
    pub fn list_prefix(&self, prefix: &str) -> Vec<String> {
        if prefix.is_empty() {
            return self.list();
        }
        let Ok(prefix) = normalize_path(prefix) else { return Vec::new() };
        let prefix = format!("{prefix}/");
        self.list().into_iter().filter(|p| p.starts_with(&prefix)).collect()
    }
}

/// Normalizes separators and `.`/`..` segments in an asset-relative path.
/// Rejects empty/root paths, absolute paths, drive prefixes, NUL and traversal
/// above the root. The result uses forward slashes on every platform.
pub fn normalize_path(path: &str) -> io::Result<String> {
    let bad = || io::Error::new(io::ErrorKind::InvalidInput, "expected an asset-relative path within the root");
    let path = path.replace('\\', "/");
    if path.starts_with('/') || path.contains(':') || path.contains('\0') {
        return Err(bad());
    }
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    return Err(bad());
                }
            }
            _ => parts.push(part),
        }
    }
    if parts.is_empty() {
        return Err(bad());
    }
    Ok(parts.join("/"))
}

fn resolve_file(root: &Path, path: &str) -> io::Result<PathBuf> {
    let root = root.canonicalize()?;
    let file = root.join(path).canonicalize()?;
    if !file.starts_with(&root) {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "asset resolves outside root"));
    }
    Ok(file)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) -> io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            continue;
        }
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        if kind.is_dir() {
            walk(root, &path, out)?;
        } else if kind.is_file() {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    Ok(())
}

/// Packs every file under `src`, plus `extra` in-memory files, into one archive.
/// Returns the number of files.
/// Extra files override loose files with the same normalized name; repeated
/// extra names use the last value. Entries are sorted for reproducible builds.
/// The output archive itself is excluded when located inside `src`.
pub fn pack(src: &Path, out: &Path, extra: &[(&str, Vec<u8>)]) -> io::Result<usize> {
    pack_with(src, out, extra, DEFAULT_VOLUME_BYTES).map(|(count, _)| count)
}

/// Data volumes of the archive being written (`<stem>.NNN.pkd` beside it) are never packed again.
fn is_own_volume(file: &Path, output: &Path) -> bool {
    let (Some(dir), Some(stem)) = (output.parent(), output.file_stem()) else { return false };
    let name = file.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    file.parent() == Some(dir) && name.starts_with(&format!("{}.", stem.to_string_lossy())) && name.ends_with(".pkd")
}

/// Like [`pack`], with an explicit maximum volume size. Returns `(files, volumes)`.
///
/// The archive is an index file (`out`, e.g. `game.pak`) plus data volumes next to it
/// (`game.000.pkd`, `game.001.pkd`...). Files are spread over the volumes largest-first so each
/// volume ends up close to the limit and the volumes are of similar size; a file larger than the
/// limit gets a volume of its own. Data is streamed to disk file by file, so packing a multi-gigabyte
/// project does not need that much memory.
pub fn pack_with(src: &Path, out: &Path, extra: &[(&str, Vec<u8>)], max_volume: u64) -> io::Result<(usize, usize)> {
    use std::io::Write;
    let src = src.canonicalize()?;
    let assets = Assets::Dir(src.clone());
    let output_path = match out.canonicalize() {
        Ok(path) => path,
        Err(_) => {
            let absolute = std::path::absolute(out)?;
            let parent =
                absolute.parent().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "output has no parent"))?.canonicalize()?;
            parent.join(absolute.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "output has no filename"))?)
        }
    };
    let mut files: std::collections::BTreeMap<String, Option<Vec<u8>>> =
        assets.try_list()?.into_iter().filter(|f| src.join(f) != output_path && !is_own_volume(&src.join(f), &output_path)).map(|f| (f, None)).collect();
    for (name, data) in extra {
        files.insert(normalize_path(name)?, Some(data.clone()));
    }
    if files.len() > u32::MAX as usize {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "too many archive entries"));
    }
    // Sizes without reading the contents.
    let mut sized: Vec<(String, u64)> = Vec::with_capacity(files.len());
    for (name, inline) in &files {
        let size = match inline {
            Some(d) => d.len() as u64,
            None => assets.byte_len(name)?,
        };
        sized.push((name.clone(), size));
    }
    // First-fit decreasing: biggest files first, each into the first volume with room.
    let max_volume = max_volume.max(1024 * 1024);
    let mut order: Vec<&(String, u64)> = sized.iter().collect();
    order.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut volumes: Vec<(u64, Vec<&str>)> = Vec::new();
    for (name, size) in order {
        match volumes.iter_mut().find(|(used, _)| *used + *size <= max_volume) {
            Some((used, list)) => {
                *used += *size;
                list.push(name);
            }
            None => volumes.push((*size, vec![name])),
        }
    }
    if volumes.is_empty() {
        volumes.push((0, Vec::new()));
    }
    // Remove volumes of an earlier build, then write the new ones.
    let dir = output_path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let stem = output_path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "game".into());
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if n.starts_with(&format!("{stem}.")) && n.ends_with(".pkd") {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    let mut entries: Vec<(String, usize, u64, u64)> = Vec::new();
    let mut volume_info: Vec<(String, u64)> = Vec::new();
    for (index, (_, names)) in volumes.iter_mut().enumerate() {
        names.sort_unstable();
        let volume_name = format!("{stem}.{index:03}.pkd");
        let mut writer = io::BufWriter::new(std::fs::File::create(dir.join(&volume_name))?);
        let mut offset = 0u64;
        for name in names.iter() {
            let data = match &files[*name] {
                Some(d) => d.clone(),
                None => assets.read(name)?,
            };
            writer.write_all(&data)?;
            entries.push((name.to_string(), index, offset, data.len() as u64));
            offset += data.len() as u64;
        }
        writer.flush()?;
        volume_info.push((volume_name, offset));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut index = Vec::new();
    index.extend(MAGIC2);
    index.extend((volume_info.len() as u32).to_le_bytes());
    for (name, size) in &volume_info {
        index.extend((name.len() as u32).to_le_bytes());
        index.extend(name.as_bytes());
        index.extend(size.to_le_bytes());
    }
    index.extend((entries.len() as u32).to_le_bytes());
    for (name, volume, offset, len) in &entries {
        index.extend((name.len() as u32).to_le_bytes());
        index.extend(name.as_bytes());
        index.extend((*volume as u32).to_le_bytes());
        index.extend(offset.to_le_bytes());
        index.extend(len.to_le_bytes());
    }
    crate::file_io::atomic_write(out, &index)?;
    Ok((entries.len(), volume_info.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_roundtrip() {
        let dir = std::env::temp_dir().join(format!("kvas_pak_test_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("a.txt"), b"hello").unwrap();
        std::fs::write(dir.join("sub/b.bin"), [1u8, 2, 3]).unwrap();
        let pak = dir.join("out.pak");
        let n = pack(&dir, &pak, &[("project.json", b"{}".to_vec())]).unwrap();
        assert_eq!(n, 3);
        let a = Assets::open_pak(&pak).unwrap();
        assert_eq!(a.read("a.txt").unwrap(), b"hello");
        assert_eq!(a.read("project.json").unwrap(), b"{}");
        assert_eq!(a.read("sub/b.bin").unwrap(), vec![1, 2, 3]);
        assert!(a.read("missing").is_err());
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn big_projects_are_split_into_balanced_volumes() {
        let dir = std::env::temp_dir().join(format!("kvas_pak_split_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        // 6 files of 700 KiB and a 2.5 MiB one: with a 2 MiB limit they cannot share one volume.
        for i in 0..6 {
            std::fs::write(dir.join(format!("assets/f{i}.bin")), vec![i as u8; 700 * 1024]).unwrap();
        }
        std::fs::write(dir.join("assets/big.bin"), vec![9u8; 2560 * 1024]).unwrap();
        let out = dir.join("out/game.pak");
        std::fs::create_dir_all(dir.join("out")).unwrap();
        let (files, volumes) = pack_with(&dir.join("assets"), &out, &[("project.json", b"{}".to_vec())], 2 * 1024 * 1024).unwrap();
        assert_eq!(files, 8);
        assert!(volumes >= 3, "got {volumes} volumes");
        let pak = Assets::open_pak(&out).unwrap();
        assert_eq!(pak.read("big.bin").unwrap().len(), 2560 * 1024);
        assert_eq!(pak.read("f3.bin").unwrap(), vec![3u8; 700 * 1024]);
        assert_eq!(pak.read("project.json").unwrap(), b"{}");
        assert_eq!(pak.list().len(), 8);
        // Rebuilding replaces the old volumes instead of leaving stale ones behind.
        pack_with(&dir.join("assets"), &out, &[], 64 * 1024 * 1024).unwrap();
        let count = std::fs::read_dir(dir.join("out")).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().ends_with(".pkd")).count();
        assert_eq!(count, 1);
        std::fs::remove_dir_all(dir).ok();
    }
}
