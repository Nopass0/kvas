//! Internal transactional file replacement shared by project, scene and archive saves.

use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

/// Writes a sibling temporary file before replacing the destination.
/// Failure leaves the existing destination intact; temporary files are removed
/// on ordinary errors. This is not a cross-process lock or a durability promise
/// for directory entries after power loss.
pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let name = path.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "destination has no filename"))?;
    let mut temp_name = name.to_os_string();
    temp_name.push(format!(".{}.{}.tmp", std::process::id(), NEXT_TEMP.fetch_add(1, Ordering::Relaxed)));
    let temp = path.with_file_name(temp_name);
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}
