//! CPU-side mesh data: procedural primitives and OBJ / glTF importers.

use crate::assets::Assets;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

/// Interleaved CPU/GPU vertex layout: position, normal and UV, all 32-bit floats.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    /// Local-space position in metres.
    pub pos: [f32; 3],
    /// Local-space unit surface normal.
    pub normal: [f32; 3],
    /// Texture coordinates, conventionally in 0..1 but tiling is supported.
    pub uv: [f32; 2],
}

/// A decoded base-colour image that came with a model (glTF/GLB).
#[derive(Clone)]
pub struct Albedo {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Tightly packed row-major RGBA8 bytes (width * height * 4).
    pub rgba: Vec<u8>,
}

/// A run of triangles that share one imported material (one glTF primitive group).
#[derive(Clone)]
pub struct MeshPart {
    /// Offset into `MeshData::indices` of the first index of the run.
    pub first_index: u32,
    /// Number of indices in the run (a multiple of three).
    pub index_count: u32,
    /// Base-colour image of the material, with the material's colour factor already applied.
    pub albedo: Option<std::sync::Arc<Albedo>>,
    /// Tangent-space normal map of the material (OpenGL convention), if it has one.
    pub normal: Option<std::sync::Arc<Albedo>>,
    /// Occlusion (R) / roughness (G) / metallic (B) image repacked from the material, if it has one.
    pub orm: Option<std::sync::Arc<Albedo>>,
}

/// Indexed CPU triangle mesh with optional imported material information.
/// Indices refer into vertices; call validate before uploading externally generated data.
#[derive(Clone)]
pub struct MeshData {
    /// Interleaved triangle vertices.
    pub vertices: Vec<Vertex>,
    /// Triangle-list indices; length must be divisible by three.
    pub indices: Vec<u32>,
    /// Base-colour texture of the model's first textured material, if it had one.
    pub albedo: Option<std::sync::Arc<Albedo>>,
    /// Base-colour factor of the model's first material (white if none).
    pub base_color: [f32; 4],
    /// Material runs for models with several materials (empty = the whole mesh uses `albedo`).
    pub parts: Vec<MeshPart>,
    /// Asset-relative source files referenced by an imported mesh (buffers/textures).
    /// Render caches watch these dependencies as well as the model file itself.
    pub dependencies: Vec<String>,
}

impl Default for MeshData {
    fn default() -> Self {
        Self { vertices: Vec::new(), indices: Vec::new(), albedo: None, base_color: [1.0; 4], parts: Vec::new(), dependencies: Vec::new() }
    }
}

/// Prefix for built-in unit primitive mesh keys.
pub const PRIMITIVE_PREFIX: &str = "primitive:";
/// Parameters of a procedural mesh; stored in the `Shape` component and encoded in the mesh key.
#[derive(Clone, Debug, PartialEq)]
pub struct ProcParams {
    /// Lowercase primitive family accepted by build.
    pub kind: String,
    /// Full local extents used by cubes, planes and rocks.
    pub size: Vec3,
    /// Primary radius in local metres.
    pub radius: f32,
    /// Top radius (cylinder / cone) or tube radius (torus).
    pub radius2: f32,
    /// Full shape height in local metres.
    pub height: f32,
    /// Azimuth segments; build clamps ordinary shapes to 3..256.
    pub segments: u32,
    /// Vertical/tube subdivisions; build clamps to 2..128.
    pub rings: u32,
    /// Deterministic random seed used by rock generation.
    pub seed: u32,
}

/// Prefix for parameterized procedural geometry keys.
pub const PROC_PREFIX: &str = "proc:";

impl ProcParams {
    /// Encodes this descriptor as a procedural mesh cache key.
    pub fn key(&self) -> String {
        format!(
            "{PROC_PREFIX}{}:{},{},{},{},{},{},{},{},{}",
            self.kind, self.size.x, self.size.y, self.size.z, self.radius, self.radius2, self.height, self.segments, self.rings, self.seed
        )
    }

    /// Parses a procedural mesh key, returning None for malformed parameters.
    pub fn parse(key: &str) -> Option<ProcParams> {
        let (kind, rest) = key.strip_prefix(PROC_PREFIX)?.split_once(':')?;
        let n: Vec<&str> = rest.split(',').collect();
        if n.len() != 9 {
            return None;
        }
        let real = |i: usize| n[i].parse::<f32>().ok().filter(|v| v.is_finite());
        Some(ProcParams {
            kind: kind.to_string(),
            size: Vec3::new(real(0)?, real(1)?, real(2)?),
            radius: real(3)?,
            radius2: real(4)?,
            height: real(5)?,
            segments: n[6].parse().ok()?,
            rings: n[7].parse().ok()?,
            seed: n[8].parse().ok()?,
        })
    }

    /// Builds a supported shape with bounded tessellation, or None for unknown families.
    pub fn build(&self) -> Option<MeshData> {
        if !self.size.is_finite() || !self.radius.is_finite() || !self.radius2.is_finite() || !self.height.is_finite() {
            return None;
        }
        let seg = self.segments.clamp(3, 256);
        let rings = self.rings.clamp(2, 128);
        let (r, h) = (self.radius.max(0.001), self.height.max(0.001));
        Some(match self.kind.as_str() {
            "cube" => scaled(cube(), self.size),
            "plane" => scaled(plane(rings.min(128)), Vec3::new(self.size.x, 1.0, self.size.z)),
            "sphere" => lathe(&arc_rows(0.0, 0.0, PI, rings, r), seg),
            "capsule" => {
                let straight = (h * 0.5 - r).max(0.0);
                let mut rows = arc_rows(straight, 0.0, PI / 2.0, rings.div_ceil(2), r);
                rows.extend(arc_rows(-straight, PI / 2.0, PI, rings.div_ceil(2), r));
                lathe(&rows, seg)
            }
            "cylinder" | "cone" => {
                let rt = if self.kind == "cone" { 0.0 } else { self.radius2.max(0.0) };
                let (nr, ny) = (h, r - rt);
                let len = (nr * nr + ny * ny).sqrt();
                let (nr, ny) = (nr / len, ny / len);
                let hh = h * 0.5;
                let mut rows = vec![(0.0, hh, 0.0, 1.0)];
                if rt > 0.0 {
                    rows.push((rt, hh, 0.0, 1.0));
                }
                rows.extend([(rt, hh, nr, ny), (r, -hh, nr, ny), (r, -hh, 0.0, -1.0), (0.0, -hh, 0.0, -1.0)]);
                lathe(&rows, seg)
            }
            "torus" => torus(r, self.radius2.max(0.001), seg, rings),
            "rock" => rock(self.size, self.radius2.clamp(0.0, 1.5), self.segments.clamp(1, 7), self.seed),
            _ => return None,
        })
    }
}

/// A detailed boulder: an icosphere displaced by layered ridged / billow / fine noise, flattened at
/// the bottom so it sits on the ground. `subdivisions` 5 is about 20 000 triangles.
fn rock(size: Vec3, ruggedness: f32, subdivisions: u32, seed: u32) -> MeshData {
    use crate::noise::{Noise, NoiseKind};
    let t = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let mut verts: Vec<Vec3> = [
        (-1.0, t, 0.0),
        (1.0, t, 0.0),
        (-1.0, -t, 0.0),
        (1.0, -t, 0.0),
        (0.0, -1.0, t),
        (0.0, 1.0, t),
        (0.0, -1.0, -t),
        (0.0, 1.0, -t),
        (t, 0.0, -1.0),
        (t, 0.0, 1.0),
        (-t, 0.0, -1.0),
        (-t, 0.0, 1.0),
    ]
    .iter()
    .map(|&(x, y, z)| Vec3::new(x, y, z).normalize())
    .collect();
    let mut faces: Vec<[u32; 3]> = vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];
    for _ in 0..subdivisions {
        let mut cache: HashMap<(u32, u32), u32> = HashMap::new();
        let mut mid = |a: u32, b: u32, verts: &mut Vec<Vec3>| -> u32 {
            let key = (a.min(b), a.max(b));
            *cache.entry(key).or_insert_with(|| {
                verts.push(((verts[a as usize] + verts[b as usize]) * 0.5).normalize());
                verts.len() as u32 - 1
            })
        };
        let mut next = Vec::with_capacity(faces.len() * 4);
        for [a, b, c] in faces {
            let (ab, bc, ca) = (mid(a, b, &mut verts), mid(b, c, &mut verts), mid(c, a, &mut verts));
            next.extend([[a, ab, ca], [b, bc, ab], [c, ca, bc], [ab, bc, ca]]);
        }
        faces = next;
    }
    let seed = seed.wrapping_mul(2654435761);
    let ridged = Noise::new(NoiseKind::Ridged).with_seed(seed).with_octaves(4);
    let billow = Noise::new(NoiseKind::Billow).with_seed(seed ^ 0x5bd1).with_octaves(3);
    let fine = Noise::new(NoiseKind::Perlin).with_seed(seed ^ 0x9e37).with_octaves(5);
    let lumps = Noise::new(NoiseKind::Perlin).with_seed(seed ^ 0x1234).with_octaves(2);
    let mut m = MeshData::default();
    for v in &verts {
        let d = *v;
        let displacement = 0.16 * lumps.sample3(d.x * 1.1, d.y * 1.1, d.z * 1.1)
            + ruggedness
                * (0.22 * ridged.sample3(d.x * 1.8, d.y * 1.8, d.z * 1.8)
                    + 0.10 * billow.sample3(d.x * 3.6, d.y * 3.6, d.z * 3.6)
                    + 0.035 * fine.sample3(d.x * 11.0, d.y * 11.0, d.z * 11.0));
        let mut p = d * (1.0 + displacement) * size * 0.5;
        // Flat-ish underside so rocks rest on terrain.
        let floor = -size.y * 0.32;
        if p.y < floor {
            p.y = floor + (p.y - floor) * 0.15;
        }
        // Box-projected UVs (the renderer can triplanar-map instead).
        let a = d.abs();
        let uv = if a.y >= a.x && a.y >= a.z {
            [p.x * 0.5 + 0.5, p.z * 0.5 + 0.5]
        } else if a.x >= a.z {
            [p.z * 0.5 + 0.5, p.y * 0.5 + 0.5]
        } else {
            [p.x * 0.5 + 0.5, p.y * 0.5 + 0.5]
        };
        m.vertices.push(Vertex { pos: p.into(), normal: d.into(), uv });
    }
    for [a, b, c] in faces {
        m.indices.extend([a, b, c]);
    }
    compute_normals(&mut m);
    m
}

fn scaled(mut m: MeshData, s: Vec3) -> MeshData {
    for v in &mut m.vertices {
        v.pos = (Vec3::from(v.pos) * s).into();
    }
    m
}

fn torus(major: f32, minor: f32, segments: u32, rings: u32) -> MeshData {
    let mut m = MeshData::default();
    for i in 0..=segments {
        let u = i as f32 / segments as f32 * TAU;
        for j in 0..=rings {
            let v = j as f32 / rings as f32 * TAU;
            let n = Vec3::new(v.cos() * u.cos(), v.sin(), v.cos() * u.sin());
            let c = Vec3::new(u.cos(), 0.0, u.sin()) * major;
            m.vertices.push(Vertex {
                pos: (c + n * minor).into(),
                normal: n.into(),
                uv: [i as f32 / segments as f32, j as f32 / rings as f32],
            });
        }
    }
    let stride = rings + 1;
    for i in 0..segments {
        for j in 0..rings {
            let a = i * stride + j;
            let b = a + stride;
            m.indices.extend([a, a + 1, b, a + 1, b + 1, b]);
        }
    }
    m
}

/// Lowercase names accepted by MeshData::primitive and primitive mesh keys.
pub const PRIMITIVES: &[&str] = &["cube", "sphere", "plane", "cylinder", "capsule", "cone"];

impl MeshData {
    /// Number of complete indexed triangles. Use [`Self::validate`] to reject a trailing index.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Whether there are no drawable triangles (vertices alone do not draw anything).
    pub fn is_empty(&self) -> bool {
        self.indices.len() < 3 || self.vertices.is_empty()
    }

    /// Checks triangle topology, finite vertex attributes and embedded image dimensions.
    /// Zero-length normals and degenerate triangles are allowed (common at primitive poles).
    pub fn validate(&self) -> Result<(), String> {
        self.validate_geometry()?;
        for (i, vertex) in self.vertices.iter().enumerate() {
            if !Vec3::from(vertex.normal).is_finite() || vertex.uv.iter().any(|v| !v.is_finite()) {
                return Err(format!("vertex {i} has non-finite normal or UV"));
            }
        }
        if self.base_color.iter().any(|v| !v.is_finite()) {
            return Err("base colour is non-finite".into());
        }
        if let Some(image) = &self.albedo {
            let expected = (image.width as usize).checked_mul(image.height as usize).and_then(|n| n.checked_mul(4));
            if image.width == 0 || image.height == 0 || expected != Some(image.rgba.len()) {
                return Err("embedded albedo dimensions do not match RGBA bytes".into());
            }
        }
        Ok(())
    }

    fn validate_geometry(&self) -> Result<(), String> {
        if !self.indices.len().is_multiple_of(3) {
            return Err("triangle index count is not divisible by three".into());
        }
        if let Some(&index) = self.indices.iter().find(|&&i| i as usize >= self.vertices.len()) {
            return Err(format!("mesh index {index} is outside {} vertices", self.vertices.len()));
        }
        if let Some((i, _)) = self.vertices.iter().enumerate().find(|(_, v)| !Vec3::from(v.pos).is_finite()) {
            return Err(format!("vertex {i} position is non-finite"));
        }
        Ok(())
    }

    /// Recomputes area-weighted smooth vertex normals from indexed triangles.
    /// Checks topology/positions before mutation; isolated vertices receive zero normals.
    pub fn recalculate_normals(&mut self) -> Result<(), String> {
        self.validate_geometry()?;
        compute_normals(self);
        Ok(())
    }

    /// Copies and transforms a mesh by a finite nonsingular affine matrix.
    /// Normals use the inverse transpose; mirrored transforms reverse triangle winding
    /// so the front-face convention stays consistent. Texture pixels remain shared.
    pub fn transformed(&self, matrix: Mat4) -> Result<Self, String> {
        self.validate()?;
        if !matrix.is_finite() || matrix.determinant() == 0.0 || !matrix.row(3).abs_diff_eq(glam::Vec4::W, 1e-6) {
            return Err("mesh transform must be finite, affine and nonsingular".into());
        }
        let normals = matrix.inverse().transpose();
        let mut result = self.clone();
        for vertex in &mut result.vertices {
            vertex.pos = matrix.transform_point3(Vec3::from(vertex.pos)).into();
            vertex.normal = normals.transform_vector3(Vec3::from(vertex.normal)).normalize_or_zero().into();
        }
        if matrix.determinant() < 0.0 {
            for triangle in result.indices.as_chunks_mut::<3>().0.iter_mut() {
                triangle.swap(1, 2);
            }
        }
        result.validate()?;
        Ok(result)
    }

    /// The material runs of this mesh; a mesh without explicit parts is one run with its own albedo.
    pub fn effective_parts(&self) -> Vec<MeshPart> {
        if self.parts.is_empty() {
            vec![MeshPart { first_index: 0, index_count: self.indices.len() as u32, albedo: self.albedo.clone(), normal: None, orm: None }]
        } else {
            self.parts.clone()
        }
    }

    /// Appends another valid mesh, offsetting indices, while retaining this mesh's material.
    /// Checks both inputs and index overflow before mutation. Meshes with different
    /// textures should be drawn separately instead of merged through this operation.
    pub fn append(&mut self, other: &Self) -> Result<(), String> {
        self.validate()?;
        other.validate()?;
        let total = self.vertices.len().checked_add(other.vertices.len()).ok_or("vertex count overflow")?;
        if total > u32::MAX as usize {
            return Err("merged mesh exceeds u32 index space".into());
        }
        let offset = self.vertices.len() as u32;
        let index_offset = self.indices.len() as u32;
        if !self.parts.is_empty() || !other.parts.is_empty() {
            let mut parts = self.effective_parts();
            parts.extend(other.effective_parts().into_iter().map(|p| MeshPart { first_index: p.first_index + index_offset, ..p }));
            self.parts = merge_parts(parts);
        }
        self.vertices.extend_from_slice(&other.vertices);
        self.indices.extend(other.indices.iter().map(|i| i + offset));
        self.dependencies.extend(other.dependencies.iter().cloned());
        self.dependencies.sort();
        self.dependencies.dedup();
        Ok(())
    }

    /// Sum of triangle areas in square local units, including duplicated/double-sided faces.
    /// Invalid topology or non-finite positions return an error.
    pub fn surface_area(&self) -> Result<f32, String> {
        self.validate_geometry()?;
        Ok(self
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|t| {
                let a = Vec3::from(self.vertices[t[0] as usize].pos);
                let b = Vec3::from(self.vertices[t[1] as usize].pos);
                let c = Vec3::from(self.vertices[t[2] as usize].pos);
                (b - a).cross(c - a).length() * 0.5
            })
            .sum())
    }

    /// Axis-aligned local bounds (minimum, maximum). Empty meshes return two zero vectors.
    pub fn bounds(&self) -> (Vec3, Vec3) {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for v in &self.vertices {
            min = min.min(Vec3::from(v.pos));
            max = max.max(Vec3::from(v.pos));
        }
        if self.vertices.is_empty() {
            (Vec3::ZERO, Vec3::ZERO)
        } else {
            (min, max)
        }
    }

    /// Builds a built-in unit primitive by lowercase name, or None for an unknown name.
    pub fn primitive(name: &str) -> Option<MeshData> {
        Some(match name {
            "cube" => cube(),
            "plane" => plane(32),
            "sphere" => sphere(),
            "cylinder" => lathe(
                &[
                    (0.0, 0.5, 0.0, 1.0),
                    (0.5, 0.5, 0.0, 1.0),
                    (0.5, 0.5, 1.0, 0.0),
                    (0.5, -0.5, 1.0, 0.0),
                    (0.5, -0.5, 0.0, -1.0),
                    (0.0, -0.5, 0.0, -1.0),
                ],
                32,
            ),
            "cone" => {
                let (nr, ny) = (0.894_427_2, 0.447_213_6);
                lathe(&[(0.0, 0.5, nr, ny), (0.5, -0.5, nr, ny), (0.5, -0.5, 0.0, -1.0), (0.0, -0.5, 0.0, -1.0)], 32)
            }
            "capsule" => capsule(),
            _ => return None,
        })
    }

    /// Loads `primitive:<name>` or a model file from the asset source.
    pub fn load(assets: &Assets, path: &str) -> Result<MeshData, String> {
        if path.starts_with(PROC_PREFIX) {
            return ProcParams::parse(path).and_then(|p| p.build()).ok_or_else(|| format!("bad procedural mesh '{path}'"));
        }
        if let Some(name) = path.strip_prefix(PRIMITIVE_PREFIX) {
            return Self::primitive(name).ok_or_else(|| format!("unknown primitive '{name}'"));
        }
        let bytes = assets.read(path).map_err(|e| format!("{path}: {e}"))?;
        let lower = path.to_ascii_lowercase();
        let mesh = if lower.ends_with(".obj") {
            parse_obj(&String::from_utf8_lossy(&bytes))
        } else if lower.ends_with(".fbx") {
            parse_fbx(assets, path, &bytes)
        } else if lower.ends_with(".gltf") || lower.ends_with(".glb") {
            parse_gltf(assets, path, &bytes)
        } else {
            Err("unsupported model format (use .obj, .fbx, .gltf or .glb)".into())
        };
        mesh.and_then(|mesh| {
            mesh.validate()?;
            Ok(mesh)
        })
        .map_err(|e| format!("{path}: {e}"))
    }
}

/// Joins neighbouring runs that use the same image.
fn merge_parts(parts: Vec<MeshPart>) -> Vec<MeshPart> {
    let mut out: Vec<MeshPart> = Vec::new();
    for p in parts {
        let same = |a: &Option<std::sync::Arc<Albedo>>, b: &Option<std::sync::Arc<Albedo>>| match (a, b) {
            (Some(x), Some(y)) => std::sync::Arc::ptr_eq(x, y),
            (None, None) => true,
            _ => false,
        };
        match out.last_mut() {
            Some(last) if last.first_index + last.index_count == p.first_index && same(&last.albedo, &p.albedo) && same(&last.normal, &p.normal) && same(&last.orm, &p.orm) => last.index_count += p.index_count,
            _ => out.push(p),
        }
    }
    out
}

/// The image of a glTF material: its base-colour texture (or a flat colour) times the colour factor.
fn material_albedo(images: &[Option<std::sync::Arc<Albedo>>], pbr: &gltf::material::PbrMetallicRoughness) -> Option<std::sync::Arc<Albedo>> {
    let factor = pbr.base_color_factor();
    let white = factor.iter().take(3).all(|v| (v - 1.0).abs() < 0.01);
    let texture = pbr.base_color_texture().and_then(|info| images.get(info.texture().source().index()).cloned().flatten());
    let to_byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    match texture {
        Some(t) if white => Some(t),
        Some(t) => {
            let mut copy = (*t).clone();
            for px in copy.rgba.chunks_exact_mut(4) {
                for c in 0..3 {
                    px[c] = (px[c] as f32 * factor[c]).clamp(0.0, 255.0) as u8;
                }
            }
            Some(std::sync::Arc::new(copy))
        }
        None if white => None,
        None => Some(std::sync::Arc::new(Albedo { width: 1, height: 1, rgba: vec![to_byte(factor[0]), to_byte(factor[1]), to_byte(factor[2]), 255] })),
    }
}

/// Normal and ORM images of a glTF material. glTF packs roughness in G and metallic in B of one
/// image; occlusion (R) is kept only when it comes from the same image, and metallic is cleared
/// because several exporters write the roughness into both channels.
fn material_maps(images: &[Option<std::sync::Arc<Albedo>>], material: &gltf::Material) -> (Option<std::sync::Arc<Albedo>>, Option<std::sync::Arc<Albedo>>) {
    let image = |index: usize| images.get(index).cloned().flatten();
    let normal = material.normal_texture().and_then(|t| image(t.texture().source().index()));
    let orm = material.pbr_metallic_roughness().metallic_roughness_texture().and_then(|info| {
        let source = info.texture().source().index();
        let keep_occlusion = material.occlusion_texture().is_some_and(|o| o.texture().source().index() == source);
        image(source).map(|img| {
            let mut copy = (*img).clone();
            for px in copy.rgba.chunks_exact_mut(4) {
                if !keep_occlusion {
                    px[0] = 255;
                }
                px[2] = 0;
            }
            std::sync::Arc::new(copy)
        })
    });
    (normal, orm)
}

fn quad_indices(base: u32) -> [u32; 6] {
    [base, base + 1, base + 2, base, base + 2, base + 3]
}

fn cube() -> MeshData {
    let (x, y, z) = (Vec3::X, Vec3::Y, Vec3::Z);
    // (u, v, normal) with u x v = normal, so the quad is counter-clockwise seen from outside.
    let faces = [(x, y, z), (y, z, x), (z, x, y), (y, x, -z), (z, y, -x), (x, z, -y)];
    let mut m = MeshData::default();
    for (u, v, n) in faces {
        let base = m.vertices.len() as u32;
        for (su, sv, uv) in [(-1.0, -1.0, [0.0, 1.0]), (1.0, -1.0, [1.0, 1.0]), (1.0, 1.0, [1.0, 0.0]), (-1.0, 1.0, [0.0, 0.0])] {
            let p = n * 0.5 + u * (0.5 * su) + v * (0.5 * sv);
            m.vertices.push(Vertex { pos: p.into(), normal: n.into(), uv });
        }
        m.indices.extend(quad_indices(base));
    }
    m
}

/// 1x1 plane facing +Y, subdivided so vertex shaders can displace it.
fn plane(n: u32) -> MeshData {
    let mut m = MeshData::default();
    for iz in 0..=n {
        for ix in 0..=n {
            let (u, v) = (ix as f32 / n as f32, iz as f32 / n as f32);
            m.vertices.push(Vertex { pos: [u - 0.5, 0.0, v - 0.5], normal: [0.0, 1.0, 0.0], uv: [u, v] });
        }
    }
    for iz in 0..n {
        for ix in 0..n {
            let a = iz * (n + 1) + ix;
            let b = a + n + 1;
            m.indices.extend([a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    m
}

/// Surface of revolution around Y. Each row is `(radius, y, normal_radial, normal_y)`, top to bottom.
fn lathe(rows: &[(f32, f32, f32, f32)], segments: u32) -> MeshData {
    let mut m = MeshData::default();
    let stride = segments + 1;
    for (i, &(r, y, nr, ny)) in rows.iter().enumerate() {
        for j in 0..=segments {
            let t = j as f32 / segments as f32;
            let (s, c) = (t * TAU).sin_cos();
            m.vertices.push(Vertex { pos: [r * c, y, r * s], normal: [nr * c, ny, nr * s], uv: [t, i as f32 / (rows.len() - 1) as f32] });
        }
    }
    for i in 0..rows.len() as u32 - 1 {
        for j in 0..segments {
            let a = i * stride + j;
            let b = a + stride;
            m.indices.extend([a, a + 1, b, a + 1, b + 1, b]);
        }
    }
    m
}

fn arc_rows(center_y: f32, from: f32, to: f32, steps: u32, radius: f32) -> Vec<(f32, f32, f32, f32)> {
    (0..=steps)
        .map(|k| {
            let a = from + (to - from) * k as f32 / steps as f32;
            (radius * a.sin(), center_y + radius * a.cos(), a.sin(), a.cos())
        })
        .collect()
}

fn sphere() -> MeshData {
    lathe(&arc_rows(0.0, 0.0, PI, 16, 0.5), 32)
}

/// Radius 0.5, total height 2 (matches the capsule collider defaults).
fn capsule() -> MeshData {
    let mut rows = arc_rows(0.5, 0.0, PI / 2.0, 8, 0.5);
    rows.extend(arc_rows(-0.5, PI / 2.0, PI, 8, 0.5));
    lathe(&rows, 32)
}

fn parse_obj(text: &str) -> Result<MeshData, String> {
    let (mut pos, mut uvs, mut norms) = (Vec::<[f32; 3]>::new(), Vec::<[f32; 2]>::new(), Vec::<[f32; 3]>::new());
    let mut m = MeshData::default();
    let mut dedupe: HashMap<(usize, usize, usize), u32> = HashMap::new();
    let mut has_normals = true;
    let nums = |s: std::str::SplitWhitespace| -> Vec<f32> { s.filter_map(|t| t.parse().ok()).collect() };
    for line in text.lines() {
        let mut it = line.split_whitespace();
        match it.next() {
            Some("v") => {
                let n = nums(it);
                if n.len() >= 3 {
                    pos.push([n[0], n[1], n[2]]);
                }
            }
            Some("vt") => {
                let n = nums(it);
                if n.len() >= 2 {
                    uvs.push([n[0], 1.0 - n[1]]);
                }
            }
            Some("vn") => {
                let n = nums(it);
                if n.len() >= 3 {
                    norms.push([n[0], n[1], n[2]]);
                }
            }
            Some("f") => {
                let mut face = Vec::new();
                for tok in it {
                    let mut parts = tok.split('/');
                    let mut idx = |len: usize| -> usize {
                        match parts.next().and_then(|s| s.parse::<i64>().ok()) {
                            Some(i) if i > 0 => i as usize,
                            Some(i) if i < 0 => (len as i64 + i + 1).max(0) as usize,
                            _ => 0,
                        }
                    };
                    let key = (idx(pos.len()), idx(uvs.len()), idx(norms.len()));
                    if key.0 == 0 || key.0 > pos.len() {
                        return Err(format!("bad face index in '{line}'"));
                    }
                    let id = *dedupe.entry(key).or_insert_with(|| {
                        has_normals &= key.2 != 0 && key.2 <= norms.len();
                        m.vertices.push(Vertex {
                            pos: pos[key.0 - 1],
                            normal: if key.2 != 0 && key.2 <= norms.len() { norms[key.2 - 1] } else { [0.0; 3] },
                            uv: if key.1 != 0 && key.1 <= uvs.len() { uvs[key.1 - 1] } else { [0.0; 2] },
                        });
                        m.vertices.len() as u32 - 1
                    });
                    face.push(id);
                }
                for k in 1..face.len().saturating_sub(1) {
                    m.indices.extend([face[0], face[k], face[k + 1]]);
                }
            }
            _ => {}
        }
    }
    if m.indices.is_empty() {
        return Err("no faces".into());
    }
    if !has_normals {
        compute_normals(&mut m);
    }
    Ok(m)
}

/// FBX through ufbx: every mesh node is triangulated, transformed to world space (y up, metres)
/// and merged. The first base-colour / diffuse texture that can be found next to the file becomes
/// the model's albedo.
fn parse_fbx(assets: &Assets, path: &str, bytes: &[u8]) -> Result<MeshData, String> {
    use ufbx::{CoordinateAxes, CoordinateAxis};
    let opts = ufbx::LoadOpts {
        target_axes: CoordinateAxes { right: CoordinateAxis::PositiveX, up: CoordinateAxis::PositiveY, front: CoordinateAxis::PositiveZ },
        target_unit_meters: 1.0,
        generate_missing_normals: true,
        ..Default::default()
    };
    let scene = ufbx::load_memory(bytes, opts).map_err(|e| format!("{e:?}"))?;
    let dir = path.rsplit_once('/').map_or("", |(d, _)| d);
    let mut m = MeshData::default();
    let mut tri = Vec::new();
    for node in scene.nodes.iter() {
        let Some(mesh) = &node.mesh else { continue };
        let g = &node.geometry_to_world;
        let world = Mat4::from_cols_array(&[
            g.m00 as f32,
            g.m10 as f32,
            g.m20 as f32,
            0.0,
            g.m01 as f32,
            g.m11 as f32,
            g.m21 as f32,
            0.0,
            g.m02 as f32,
            g.m12 as f32,
            g.m22 as f32,
            0.0,
            g.m03 as f32,
            g.m13 as f32,
            g.m23 as f32,
            1.0,
        ]);
        let normal_m = world.inverse().transpose();
        // One image per material: the texture the file names, or one guessed from the material name.
        let parent = dir.rsplit_once('/').map_or("", |(d, _)| d);
        let join = |d: &str, f: &str| if d.is_empty() { f.to_string() } else { format!("{d}/{f}") };
        let stem = path.rsplit('/').next().unwrap_or(path).rsplit_once('.').map_or(path, |(s, _)| s).to_string();
        let mut material_images: Vec<Option<std::sync::Arc<Albedo>>> = Vec::new();
        for mat in mesh.materials.iter() {
            let tex = mat.pbr.base_color.texture.as_ref().or(mat.fbx.diffuse_color.texture.as_ref());
            let mut image = None;
            if let Some(t) = tex {
                let name = t.filename.rsplit(['/', '\\']).next().unwrap_or("").to_string();
                // Next to the model, or in a sibling `textures` folder (common for exported packs).
                let candidates = [join(dir, &name), join(&join(dir, "textures"), &name), join(&join(parent, "textures"), &name), join(&join(parent, "Textures"), &name)];
                // Missing candidates are dependencies too: creating a missing texture
                // later must invalidate a model cached with a fallback material.
                m.dependencies.extend(candidates.iter().cloned());
                image = candidates.iter().find_map(|c| load_albedo(assets, c));
            }
            if image.is_none() {
                image = guess_texture(assets, dir, parent, &stem, &mat.element.name.to_string());
            }
            material_images.push(image);
        }
        if m.albedo.is_none() {
            m.albedo = material_images.iter().flatten().next().cloned();
        }
        let base = m.vertices.len() as u32;
        for i in 0..mesh.num_indices {
            let p = mesh.vertex_position[i];
            let n = if mesh.vertex_normal.exists { mesh.vertex_normal[i] } else { ufbx::Vec3 { x: 0.0, y: 1.0, z: 0.0 } };
            let uv = if mesh.vertex_uv.exists {
                let uv = mesh.vertex_uv[i];
                [uv.x as f32, 1.0 - uv.y as f32]
            } else {
                [0.0; 2]
            };
            m.vertices.push(Vertex {
                pos: world.transform_point3(Vec3::new(p.x as f32, p.y as f32, p.z as f32)).into(),
                normal: normal_m.transform_vector3(Vec3::new(n.x as f32, n.y as f32, n.z as f32)).normalize_or_zero().into(),
                uv,
            });
        }
        // Triangles grouped by material so each group can be drawn with its own image.
        let groups = mesh.materials.len().max(1);
        let mut per_material: Vec<Vec<u32>> = vec![Vec::new(); groups];
        for (face_index, face) in mesh.faces.iter().enumerate() {
            ufbx::triangulate_face_vec(&mut tri, mesh, *face);
            let slot = if mesh.face_material.is_empty() { 0 } else { (mesh.face_material[face_index] as usize).min(groups - 1) };
            per_material[slot].extend(tri.iter().map(|&i| base + i));
        }
        for (slot, list) in per_material.into_iter().enumerate() {
            if list.is_empty() {
                continue;
            }
            let first_index = m.indices.len() as u32;
            let index_count = list.len() as u32;
            m.indices.extend(list);
            let albedo = material_images.get(slot).cloned().flatten();
            m.parts.push(MeshPart { first_index, index_count, albedo, normal: None, orm: None });
        }
    }
    if m.indices.is_empty() {
        return Err("no triangles".into());
    }
    m.parts = merge_parts(std::mem::take(&mut m.parts));
    if m.parts.len() <= 1 {
        m.parts.clear();
    }
    Ok(m)
}

fn load_albedo(assets: &Assets, path: &str) -> Option<std::sync::Arc<Albedo>> {
    let img = image::load_from_memory(&assets.read(path).ok()?).ok()?;
    let rgba = img.to_rgba8();
    Some(std::sync::Arc::new(Albedo { width: rgba.width(), height: rgba.height(), rgba: rgba.into_raw() }))
}

/// Finds a texture for a material that names none: `Trunk`/`Bark` and `Leaf`/`Leaves` materials look
/// for an image with that word in its name, preferring the one that also carries the model family
/// ("STOak3" -> "oak"), e.g. `STOakLeafTexture.png`.
fn guess_texture(assets: &Assets, dir: &str, parent: &str, stem: &str, material: &str) -> Option<std::sync::Arc<Albedo>> {
    let material = material.to_ascii_lowercase();
    let kind = if material.contains("trunk") || material.contains("bark") {
        "trunk"
    } else if material.contains("leaf") || material.contains("leaves") {
        "leaf"
    } else {
        return None;
    };
    let family: String = stem.chars().filter(|c| c.is_ascii_alphabetic()).collect::<String>().to_ascii_lowercase();
    let sub = |d: &str| if d.is_empty() { "textures".to_string() } else { format!("{}/textures", d.to_ascii_lowercase()) };
    let roots = [dir.to_ascii_lowercase(), sub(dir), sub(parent)];
    let mut best: Option<(i32, String)> = None;
    for file in assets.list() {
        let lower = file.to_ascii_lowercase();
        if !(lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg")) {
            continue;
        }
        let (folder, name) = lower.rsplit_once('/').map_or(("", lower.as_str()), |(f, n)| (f, n));
        if !roots.iter().any(|r| r.as_str() == folder) || !name.contains(kind) {
            continue;
        }
        let score = if name.contains(&family) { 2 } else { 0 } - (name.contains("alt") as i32);
        if best.as_ref().is_none_or(|(s, _)| score > *s) {
            best = Some((score, file));
        }
    }
    best.and_then(|(_, file)| load_albedo(assets, &file))
}

fn compute_normals(m: &mut MeshData) {
    for v in &mut m.vertices {
        v.normal = [0.0; 3];
    }
    for tri in m.indices.as_chunks::<3>().0.iter() {
        let p = |i: u32| Vec3::from(m.vertices[i as usize].pos);
        let n = (p(tri[1]) - p(tri[0])).cross(p(tri[2]) - p(tri[0]));
        for &i in tri {
            let v = &mut m.vertices[i as usize];
            v.normal = (Vec3::from(v.normal) + n).into();
        }
    }
    for v in &mut m.vertices {
        v.normal = Vec3::from(v.normal).normalize_or_zero().into();
    }
}

fn to_albedo(img: &gltf::image::Data) -> Option<Albedo> {
    use gltf::image::Format;
    let n = (img.width * img.height) as usize;
    let rgba = match img.format {
        Format::R8G8B8A8 => img.pixels.clone(),
        Format::R8G8B8 => img.pixels.as_chunks::<3>().0.iter().flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
        Format::R8 => img.pixels.iter().flat_map(|&v| [v, v, v, 255]).collect(),
        Format::R8G8 => img.pixels.as_chunks::<2>().0.iter().flat_map(|p| [p[0], p[0], p[0], p[1]]).collect(),
        _ => return None,
    };
    (rgba.len() == n * 4).then_some(Albedo { width: img.width, height: img.height, rgba })
}

fn decode_uri_path(uri: &str) -> Result<String, String> {
    let mut decoded = Vec::with_capacity(uri.len());
    let mut bytes = uri.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let high = bytes.next().and_then(|b| (b as char).to_digit(16)).ok_or("invalid URI percent escape")?;
            let low = bytes.next().and_then(|b| (b as char).to_digit(16)).ok_or("invalid URI percent escape")?;
            decoded.push(((high << 4) | low) as u8);
        } else {
            decoded.push(byte);
        }
    }
    String::from_utf8(decoded).map_err(|e| format!("asset URI is not UTF-8: {e}"))
}

/// glTF documents that reference files (`Textures/colormap.png`) cannot go through `import_slice`;
/// those buffers and images are read through `Assets`, relative to the model's folder.
struct ImportedGltf {
    document: gltf::Document,
    buffers: Vec<gltf::buffer::Data>,
    images: Vec<Option<std::sync::Arc<Albedo>>>,
    dependencies: Vec<String>,
}

fn load_external(assets: &Assets, path: &str, bytes: &[u8]) -> Result<ImportedGltf, String> {
    let gltf::Gltf { document, blob } = gltf::Gltf::from_slice(bytes).map_err(|e| e.to_string())?;
    let dir = path.rsplit_once('/').map_or("", |(d, _)| d);
    let dependencies = std::cell::RefCell::new(Vec::new());
    let read = |uri: &str| -> Result<Vec<u8>, String> {
        if uri.starts_with("data:") {
            return gltf::buffer::Data::from_source(gltf::buffer::Source::Uri(uri), None).map(|data| data.0).map_err(|e| e.to_string());
        }
        let decoded = decode_uri_path(uri)?;
        let full = if dir.is_empty() { decoded } else { format!("{dir}/{decoded}") };
        let full = crate::assets::normalize_path(&full).map_err(|e| e.to_string())?;
        dependencies.borrow_mut().push(full.clone());
        assets.read(&full).map_err(|e| format!("{full}: {e}"))
    };
    let mut buffers = Vec::new();
    for b in document.buffers() {
        let data = match b.source() {
            gltf::buffer::Source::Bin => blob.clone().ok_or("GLB without a binary chunk")?,
            gltf::buffer::Source::Uri(uri) => read(uri)?,
        };
        if data.len() < b.length() {
            return Err(format!("glTF buffer {} is shorter than its declared byteLength", b.index()));
        }
        buffers.push(gltf::buffer::Data(data));
    }
    let mut albedos = Vec::new();
    for img in document.images() {
        let encoded = match img.source() {
            gltf::image::Source::Uri { uri, .. } => read(uri).ok(),
            gltf::image::Source::View { view, .. } => buffers
                .get(view.buffer().index())
                .and_then(|buffer| view.offset().checked_add(view.length()).and_then(|end| buffer.0.get(view.offset()..end)))
                .map(<[u8]>::to_vec),
        };
        albedos.push(encoded.and_then(|e| image::load_from_memory(&e).ok()).map(|i| {
            let rgba = i.to_rgba8();
            std::sync::Arc::new(Albedo { width: rgba.width(), height: rgba.height(), rgba: rgba.into_raw() })
        }));
    }
    Ok(ImportedGltf { document, buffers, images: albedos, dependencies: dependencies.into_inner() })
}

fn parse_gltf(assets: &Assets, path: &str, bytes: &[u8]) -> Result<MeshData, String> {
    let ImportedGltf { document: doc, buffers, images, dependencies } = match gltf::import_slice(bytes) {
        Ok((document, buffers, images)) => ImportedGltf {
            document,
            buffers,
            images: images.iter().map(|i| to_albedo(i).map(std::sync::Arc::new)).collect(),
            dependencies: Vec::new(),
        },
        Err(_) => load_external(assets, path, bytes)?,
    };
    let scene = doc.default_scene().or_else(|| doc.scenes().next()).ok_or("no scene")?;
    let mut mesh = MeshData { dependencies, ..Default::default() };
    // Explicit enter/exit frames avoid stack overflow and detect cycles while
    // allowing the same node to be instanced under separate parent paths.
    enum Work<'a> {
        Enter(gltf::Node<'a>, Mat4),
        Exit(usize),
    }
    let mut stack: Vec<_> = scene.nodes().collect::<Vec<_>>().into_iter().rev().map(|node| Work::Enter(node, Mat4::IDENTITY)).collect();
    let mut active = std::collections::HashSet::new();
    while let Some(work) = stack.pop() {
        let (node, parent) = match work {
            Work::Exit(index) => {
                active.remove(&index);
                continue;
            }
            Work::Enter(node, parent) => (node, parent),
        };
        if !active.insert(node.index()) {
            return Err(format!("cyclic glTF node hierarchy at node {}", node.index()));
        }
        let world = parent * Mat4::from_cols_array_2d(&node.transform().matrix());
        if !world.is_finite() {
            return Err(format!("non-finite glTF node transform {}", node.index()));
        }
        let normal_matrix = if world.determinant() == 0.0 { Mat4::IDENTITY } else { world.inverse().transpose() };
        if let Some(source) = node.mesh() {
            for primitive in source.primitives() {
                use gltf::mesh::Mode;
                if !matches!(primitive.mode(), Mode::Triangles | Mode::TriangleStrip | Mode::TriangleFan) {
                    continue;
                }
                let pbr = primitive.material().pbr_metallic_roughness();
                if mesh.albedo.is_none() && mesh.indices.is_empty() {
                    mesh.base_color = pbr.base_color_factor();
                }
                if mesh.albedo.is_none() {
                    if let Some(info) = pbr.base_color_texture() {
                        mesh.albedo = images.get(info.texture().source().index()).cloned().flatten();
                    }
                }
                let part_albedo = material_albedo(&images, &pbr);
                let (part_normal, part_orm) = material_maps(&images, &primitive.material());
                let first_index = mesh.indices.len() as u32;
                let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|data| data.0.as_slice()));
                let Some(positions) = reader.read_positions() else {
                    continue;
                };
                let base = u32::try_from(mesh.vertices.len()).map_err(|_| "glTF exceeds u32 vertex space")?;
                let normals: Vec<_> = reader.read_normals().map(|values| values.collect()).unwrap_or_default();
                let uvs: Vec<_> = reader.read_tex_coords(0).map(|values| values.into_f32().collect()).unwrap_or_default();
                for (index, position) in positions.enumerate() {
                    mesh.vertices.push(Vertex {
                        pos: world.transform_point3(Vec3::from(position)).into(),
                        normal: normals
                            .get(index)
                            .map_or([0.0; 3], |&normal| normal_matrix.transform_vector3(Vec3::from(normal)).normalize_or_zero().into()),
                        uv: uvs.get(index).copied().unwrap_or([0.0; 2]),
                    });
                }
                let count = mesh.vertices.len() - base as usize;
                let indices: Vec<u32> =
                    reader.read_indices().map(|indices| indices.into_u32().collect()).unwrap_or_else(|| (0..count as u32).collect());
                if indices.iter().any(|&index| index as usize >= count) {
                    return Err("glTF primitive index exceeds its POSITION count".into());
                }
                let mut triangles = Vec::new();
                match primitive.mode() {
                    Mode::Triangles => {
                        if !indices.len().is_multiple_of(3) {
                            return Err("glTF triangle index count is not divisible by three".into());
                        }
                        triangles.extend(indices.as_chunks::<3>().0.iter().map(|tri| [tri[0], tri[1], tri[2]]));
                    }
                    Mode::TriangleStrip => {
                        for (index, tri) in indices.windows(3).enumerate() {
                            triangles.push(if index % 2 == 0 { [tri[0], tri[1], tri[2]] } else { [tri[1], tri[0], tri[2]] });
                        }
                    }
                    Mode::TriangleFan => {
                        for tri in indices.get(1..).unwrap_or_default().windows(2) {
                            triangles.push([indices[0], tri[0], tri[1]]);
                        }
                    }
                    _ => unreachable!(),
                }
                for mut triangle in triangles {
                    if triangle[0] == triangle[1] || triangle[1] == triangle[2] || triangle[0] == triangle[2] {
                        continue;
                    }
                    if world.determinant() < 0.0 {
                        triangle.swap(1, 2);
                    }
                    for index in triangle {
                        mesh.indices.push(base.checked_add(index).ok_or("glTF index overflow")?);
                    }
                }
                let index_count = mesh.indices.len() as u32 - first_index;
                if index_count > 0 {
                    mesh.parts.push(MeshPart { first_index, index_count, albedo: part_albedo, normal: part_normal, orm: part_orm });
                }
            }
        }
        stack.push(Work::Exit(node.index()));
        for child in node.children().collect::<Vec<_>>().into_iter().rev() {
            stack.push(Work::Enter(child, world));
        }
    }
    if mesh.indices.is_empty() {
        return Err("no triangles".into());
    }
    mesh.parts = merge_parts(std::mem::take(&mut mesh.parts));
    // A single material needs no runs; the renderer treats the mesh as one piece.
    if mesh.parts.len() <= 1 && mesh.parts.iter().all(|p| p.normal.is_none() && p.orm.is_none()) {
        mesh.parts.clear();
    }
    mesh.validate_geometry()?;
    if mesh.vertices.iter().any(|vertex| vertex.normal == [0.0; 3]) {
        // Recompute consistently when any primitive lacks normals; mixing valid
        // and zero normals would otherwise leave unlit patches in merged models.
        compute_normals(&mut mesh);
    }
    mesh.validate()?;
    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitives_face_outward() {
        for name in PRIMITIVES {
            let m = MeshData::primitive(name).unwrap();
            let mut checked = 0;
            for tri in m.indices.as_chunks::<3>().0.iter() {
                let v = |i: u32| m.vertices[i as usize];
                let (a, b, c) = (v(tri[0]), v(tri[1]), v(tri[2]));
                let geo = (Vec3::from(b.pos) - Vec3::from(a.pos)).cross(Vec3::from(c.pos) - Vec3::from(a.pos));
                if geo.length() < 1e-6 {
                    continue; // degenerate triangle at a pole
                }
                let n = Vec3::from(a.normal) + Vec3::from(b.normal) + Vec3::from(c.normal);
                assert!(geo.dot(n) > 0.0, "{name}: triangle winds against its normals");
                checked += 1;
            }
            assert!(checked > 0, "{name}: no triangles");
        }
    }

    #[test]
    fn procedural_shapes_face_outward() {
        for kind in ["cube", "plane", "sphere", "capsule", "cylinder", "cone", "torus", "rock"] {
            let p = ProcParams {
                kind: kind.into(),
                size: Vec3::new(2.0, 1.0, 3.0),
                radius: 0.7,
                radius2: 0.25,
                height: 2.0,
                segments: 20,
                rings: 10,
                seed: 3,
            };
            assert_eq!(ProcParams::parse(&p.key()), Some(p.clone()), "{kind} key roundtrip");
            let m = p.build().unwrap();
            let mut checked = 0;
            for tri in m.indices.as_chunks::<3>().0.iter() {
                let v = |i: u32| m.vertices[i as usize];
                let (a, b, c) = (v(tri[0]), v(tri[1]), v(tri[2]));
                let geo = (Vec3::from(b.pos) - Vec3::from(a.pos)).cross(Vec3::from(c.pos) - Vec3::from(a.pos));
                if geo.length() < 1e-6 {
                    continue;
                }
                let n = Vec3::from(a.normal) + Vec3::from(b.normal) + Vec3::from(c.normal);
                assert!(geo.dot(n) > 0.0, "{kind}: wrong winding");
                checked += 1;
            }
            assert!(checked > 0, "{kind}");
        }
    }

    #[test]
    fn obj_quad_is_triangulated() {
        let m = parse_obj("v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n").unwrap();
        assert_eq!(m.indices.len(), 6);
        assert!(m.vertices[0].normal[2] > 0.99);
    }
}
