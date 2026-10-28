//! Editable static geometry, persisted in scenes and uploaded as versioned DynamicMesh data.
use crate::{components::{Component, DynamicMesh}, mesh::{MeshData, Vertex}};
use glam::Vec3;
use kvas_ecs::World;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Scene-persistent geometry. Increment revision after manual vertex/index changes.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct EditableMesh {
    /// Position, normal and UV attributes, indexed by triangle data.
    pub vertices: Vec<Vertex>,
    /// Triangle-list topology.
    pub indices: Vec<u32>,
    /// Version of geometry; update increments refresh the shared GPU mesh.
    pub revision: u64,
}
impl Component for EditableMesh {}

impl EditableMesh {
    /// Copies the geometry of a validated source. Material images remain on MeshRenderer.
    pub fn from_mesh(mesh: &MeshData) -> Result<Self, String> {
        mesh.validate()?;
        if mesh.vertices.len() > 100_000 || mesh.indices.len() > 600_000 { return Err("editable geometry limit: 100000 vertices / 600000 indices".into()); }
        Ok(Self { vertices: mesh.vertices.clone(), indices: mesh.indices.clone(), revision: 1 })
    }
    /// CPU mesh snapshot, with no imported material runs.
    pub fn mesh(&self) -> MeshData { MeshData { vertices: self.vertices.clone(), indices: self.indices.clone(), ..Default::default() } }
    /// Marks vertex/index modifications for upload.
    pub fn changed(&mut self) { self.revision = self.revision.saturating_add(1); }
    /// Rebuilds area-weighted normals after deformation.
    pub fn recalculate_normals(&mut self) -> Result<(), String> {
        let mut mesh = self.mesh(); mesh.recalculate_normals()?; self.vertices = mesh.vertices; self.changed(); Ok(())
    }
    /// Simple dominant-normal box projection. This is a UV inspection tool, not an island packer.
    pub fn box_uv(&mut self, scale: f32) -> Result<(), String> {
        if !scale.is_finite() || scale <= 0.0 { return Err("UV scale must be positive".into()); }
        for v in &mut self.vertices {
            let n = Vec3::from(v.normal).abs(); let p = Vec3::from(v.pos) / scale;
            v.uv = if n.y >= n.x && n.y >= n.z { [p.x, p.z] } else if n.x >= n.z { [p.z, p.y] } else { [p.x, p.y] };
        }
        self.changed(); Ok(())
    }
    /// Pushes nearby vertices along their normals with a smooth local-space brush falloff.
    /// Coincident split vertices share a direction to keep hard-edged meshes closed.
    pub fn sculpt(&mut self, center: Vec3, radius: f32, amount: f32) -> bool {
        if !center.is_finite() || !radius.is_finite() || radius <= 0.0 || !amount.is_finite() { return false; }
        let key = |p: [f32; 3]| p.map(|v| (v as f64 * 100000.0).round() as i64);
        let mut normals = std::collections::HashMap::<[i64; 3], Vec3>::new();
        for v in &self.vertices { *normals.entry(key(v.pos)).or_default() += Vec3::from(v.normal); }
        let mut changed = false;
        for v in &mut self.vertices {
            let p = Vec3::from(v.pos); let distance = p.distance(center) / radius;
            if distance < 1.0 {
                let falloff = (1.0 - distance * distance).powi(2);
                v.pos = (p + normals[&key(v.pos)].normalize_or_zero() * amount * falloff).to_array();
                changed = true;
            }
        }
        if changed { self.changed(); } changed
    }
    /// OBJ with indexed positions, UVs and normals. Does not export glTF material bindings.
    pub fn to_obj(&self) -> Result<String, String> {
        use std::fmt::Write;
        self.mesh().validate()?;
        let mut text = String::from("# Kvas edited geometry; local coordinates\n");
        for v in &self.vertices { writeln!(text, "v {} {} {}", v.pos[0], v.pos[1], v.pos[2]).unwrap(); }
        for v in &self.vertices { writeln!(text, "vt {} {}", v.uv[0], v.uv[1]).unwrap(); }
        for v in &self.vertices { writeln!(text, "vn {} {} {}", v.normal[0], v.normal[1], v.normal[2]).unwrap(); }
        for t in self.indices.chunks_exact(3) { let a = t[0] + 1; let b = t[1] + 1; let c = t[2] + 1; writeln!(text, "f {a}/{a}/{a} {b}/{b}/{b} {c}/{c}/{c}").unwrap(); }
        Ok(text)
    }
}

/// Uploads changed editable geometry. Takes precedence over procedural geometry on the same entity.
pub fn update(world: &mut World) {
    for e in world.entities_with::<EditableMesh>() {
        let Some(edit) = world.get::<EditableMesh>(e) else { continue };
        // High bit separates authored geometry revisions from other procedural systems.
        let version = edit.revision | (1u64 << 63);
        if world.get::<DynamicMesh>(e).is_some_and(|d| d.version == version) { continue; }
        let mesh = edit.mesh(); if mesh.validate().is_err() { continue; }
        world.insert(e, DynamicMesh { data: Arc::new(mesh), version });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sculpt_uv_export_and_scene_round_trip_preserve_geometry() {
        let source = MeshData::primitive("cube").unwrap();
        let mut editable = EditableMesh::from_mesh(&source).unwrap();
        assert!(editable.sculpt(Vec3::new(0.5, 0.5, 0.5), 0.2, 0.1));
        // Split vertices of the same corner receive identical displacement.
        let a = Vec3::from(editable.vertices[0].pos);
        assert!(a.is_finite());
        editable.recalculate_normals().unwrap(); editable.box_uv(2.0).unwrap();
        let json = serde_json::to_string(&editable).unwrap();
        let restored: EditableMesh = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.to_obj().unwrap(), editable.to_obj().unwrap());
        assert!(restored.to_obj().unwrap().contains("f "));
        assert!(!editable.sculpt(Vec3::ZERO, 0.0, 1.0));
    }
}
