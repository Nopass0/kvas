//! Skeletal animation: skinned models with bones and clips, played by the [`Animator`] component.
//!
//! * **Import.** FBX (ufbx) and glTF/GLB skins are read with their skeleton, per-vertex bone
//!   weights (up to four) and every animation clip. Clips are sampled at 30 frames per second
//!   into model-space bone poses; walking clips are made "in place" (the forward drift of the
//!   hips is removed, the sway stays) because game code moves the object itself.
//! * **Humanoid clips without animation files.** A rig with recognisable bone names (hips or
//!   pelvis, spine, chest, neck, head, upper arm, forearm, thigh, shin, foot — `.L/.R`,
//!   `_left/_right` and similar) gets procedural clips: `Idle`, `Walk`, `Run`, `Talk`, `Wave`,
//!   `Sit`, `Wipe` (wiping a counter) and `Look` (glancing around). Clips from the file keep
//!   their names; the procedural ones are added only when no clip of that name exists.
//! * **Playback.** [`Animator`] picks the clip, speed and looping; changing `clip` cross-fades
//!   over `fade` seconds. The pose is skinned on the CPU into a [`DynamicMesh`], so any shader,
//!   texture and shadow work as for a static model (a few thousand vertices cost microseconds).
//!
//! ```no_run
//! # use kvas_engine::prelude::*;
//! # fn f(ctx: &mut Ctx, monster: Entity) {
//! // In a script: walk while moving, idle otherwise.
//! if let Some(a) = ctx.get_mut_of::<Animator>(monster) {
//!     a.clip = "Walk".into();
//! }
//! # }
//! ```

use crate::assets::Assets;
use crate::components::{DynamicMesh, MeshRenderer};
use crate::mesh::MeshData;
use glam::{Mat4, Quat, Vec3};
use kvas_ecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Translation, rotation and scale of one bone in model space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trs {
    /// Position.
    pub t: Vec3,
    /// Orientation.
    pub r: Quat,
    /// Scale per axis.
    pub s: Vec3,
}

impl Default for Trs {
    fn default() -> Self {
        Self { t: Vec3::ZERO, r: Quat::IDENTITY, s: Vec3::ONE }
    }
}

impl Trs {
    /// Splits a matrix into translation, rotation and scale.
    pub fn from_mat(m: Mat4) -> Self {
        let (s, r, t) = m.to_scale_rotation_translation();
        Self { t, r: r.normalize(), s }
    }
    /// The matrix of this transform.
    pub fn mat(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.s, self.r, self.t)
    }
    /// Blend towards `o` by `k` (0..1), shortest-arc for the rotation.
    pub fn lerp(&self, o: &Trs, k: f32) -> Trs {
        Trs { t: self.t.lerp(o.t, k), r: self.r.slerp(o.r, k).normalize(), s: self.s.lerp(o.s, k) }
    }
}

/// A node of the skeleton (a bone, or another node the bones hang from).
#[derive(Clone, Debug)]
pub struct SkelNode {
    /// Bone or node name from the file.
    pub name: String,
    /// Index of the parent node in [`SkinnedModel::nodes`].
    pub parent: Option<usize>,
    /// Model-space transform in the rest (bind) pose.
    pub rest: Mat4,
}

/// What a vertex weight refers to: a skeleton node and the matrix from the mesh's bind space
/// to that node.
#[derive(Clone, Debug)]
pub struct Joint {
    /// Index in [`SkinnedModel::nodes`].
    pub node: usize,
    /// From the mesh's bind space to the node's space in the bind pose.
    pub inverse_bind: Mat4,
}

/// An animation sampled at a fixed rate: `frames[f][node]` is a model-space pose.
#[derive(Clone, Debug)]
pub struct Clip {
    /// Clip name (`Walk`, `Idle_Watchful`...).
    pub name: String,
    /// Length in seconds.
    pub duration: f32,
    /// Sampling rate of `frames`.
    pub fps: f32,
    /// `frames[frame][node]`: model-space pose of each node.
    pub frames: Vec<Vec<Trs>>,
}

/// A skinned model: its bind-pose mesh, the weights, the skeleton and the clips.
#[derive(Clone)]
pub struct SkinnedModel {
    /// The mesh in its rest pose (model space), with textures and material runs.
    pub mesh: MeshData,
    /// Per vertex: position and normal in the mesh's bind space (what joints transform).
    pub bind: Vec<(Vec3, Vec3)>,
    /// Per vertex: up to four joints and their weights (summing to one).
    pub joints: Vec<[u16; 4]>,
    /// Per vertex: the weights of `joints`.
    pub weights: Vec<[f32; 4]>,
    /// The skeleton (parents before children for FBX).
    pub nodes: Vec<SkelNode>,
    /// What `joints` refer to.
    pub skin: Vec<Joint>,
    /// Clips from the file and the procedural humanoid ones.
    pub clips: Vec<Clip>,
    /// The humanoid parts, when the bones have recognisable names (IK, ragdolls).
    pub humanoid: Option<crate::humanoid::HumanoidBones>,
    /// `children[i]`: nodes whose parent is `i`.
    pub children: Vec<Vec<usize>>,
}

fn norm_name(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase()
}

impl SkinnedModel {
    /// Clip by name: exact, then case-insensitive, then the part after `|` (`Rig|Walk` = `Walk`).
    pub fn clip(&self, name: &str) -> Option<&Clip> {
        let n = norm_name(name);
        self.clips
            .iter()
            .find(|c| c.name == name)
            .or_else(|| self.clips.iter().find(|c| norm_name(&c.name) == n))
            .or_else(|| self.clips.iter().find(|c| norm_name(c.name.rsplit('|').next().unwrap_or("")) == n))
    }

    /// The rest pose.
    pub fn rest_pose(&self) -> Vec<Trs> {
        self.nodes.iter().map(|n| Trs::from_mat(n.rest)).collect()
    }

    /// Pose of `clip` at `time` seconds (wrapped when looping, held at the end otherwise).
    pub fn sample(&self, clip: &Clip, time: f32, looping: bool, out: &mut Vec<Trs>) {
        out.clear();
        let n = clip.frames.len();
        if n == 0 {
            out.extend(self.rest_pose());
            return;
        }
        let t = if looping && clip.duration > 0.0 { time.rem_euclid(clip.duration) } else { time.clamp(0.0, clip.duration) };
        let f = t * clip.fps;
        let a = (f.floor() as usize).min(n - 1);
        let b = if a + 1 < n { a + 1 } else if looping { 0 } else { a };
        let k = (f - f.floor()).clamp(0.0, 1.0);
        out.extend(clip.frames[a].iter().zip(&clip.frames[b]).map(|(x, y)| x.lerp(y, k)));
    }

    /// Deforms the vertices of `out` (a copy of `self.mesh`) into `pose`.
    pub fn skin_into(&self, pose: &[Trs], out: &mut MeshData) {
        let mats: Vec<Mat4> = self.skin.iter().map(|j| pose.get(j.node).map_or(Mat4::IDENTITY, Trs::mat) * j.inverse_bind).collect();
        for (i, v) in out.vertices.iter_mut().enumerate() {
            let (Some(&(p, n)), Some(js), Some(ws)) = (self.bind.get(i), self.joints.get(i), self.weights.get(i)) else { continue };
            let (mut pos, mut nrm) = (Vec3::ZERO, Vec3::ZERO);
            for k in 0..4 {
                let w = ws[k];
                if w <= 0.0 {
                    continue;
                }
                let m = mats.get(js[k] as usize).copied().unwrap_or(Mat4::IDENTITY);
                pos += m.transform_point3(p) * w;
                nrm += m.transform_vector3(n) * w;
            }
            v.pos = pos.into();
            v.normal = nrm.normalize_or_zero().into();
        }
    }

    fn node_named(&self, names: &[&str]) -> Option<usize> {
        names.iter().find_map(|want| self.nodes.iter().position(|n| norm_name(&n.name) == *want))
    }

    /// Makes clips play in place: removes the hips' straight-line drift over the clip.
    fn in_place(&mut self) {
        let Some(hips) = self.node_named(&["hips", "pelvis", "hip", "torso"]) else { return };
        for clip in &mut self.clips {
            let (Some(first), Some(last)) = (clip.frames.first(), clip.frames.last()) else { continue };
            let drift = (last[hips].t - first[hips].t) * Vec3::new(1.0, 0.0, 1.0);
            if drift.length() < 0.05 {
                continue;
            }
            let n = (clip.frames.len() - 1).max(1) as f32;
            for (f, frame) in clip.frames.iter_mut().enumerate() {
                let off = drift * (f as f32 / n);
                for trs in frame.iter_mut() {
                    trs.t -= off;
                }
            }
        }
    }
}

fn top4(mut list: Vec<(u16, f32)>) -> ([u16; 4], [f32; 4]) {
    list.sort_by(|a, b| b.1.total_cmp(&a.1));
    list.truncate(4);
    let sum: f32 = list.iter().map(|w| w.1).sum();
    let (mut j, mut w) = ([0u16; 4], [0f32; 4]);
    for (i, (jj, ww)) in list.iter().enumerate() {
        j[i] = *jj;
        w[i] = if sum > 0.0 { ww / sum } else { 0.0 };
    }
    (j, w)
}

// ---------------------------------------------------------------- FBX

fn fbx_mat(g: &ufbx::Matrix) -> Mat4 {
    Mat4::from_cols_array(&[
        g.m00 as f32, g.m10 as f32, g.m20 as f32, 0.0, g.m01 as f32, g.m11 as f32, g.m21 as f32, 0.0, g.m02 as f32, g.m12 as f32, g.m22 as f32, 0.0, g.m03 as f32,
        g.m13 as f32, g.m23 as f32, 1.0,
    ])
}

fn load_fbx(assets: &Assets, path: &str, bytes: &[u8]) -> Result<SkinnedModel, String> {
    use ufbx::{CoordinateAxes, CoordinateAxis};
    // The same options as the static loader, so the vertices line up one to one.
    let opts = ufbx::LoadOpts {
        target_axes: CoordinateAxes { right: CoordinateAxis::PositiveX, up: CoordinateAxis::PositiveY, front: CoordinateAxis::PositiveZ },
        target_unit_meters: 1.0,
        generate_missing_normals: true,
        ..Default::default()
    };
    let scene = ufbx::load_memory(bytes, opts).map_err(|e| format!("{e:?}"))?;
    let mesh = MeshData::load(assets, path)?;
    // Skeleton: every node a weight refers to, with all its ancestors.
    let mut index_of: HashMap<u32, usize> = HashMap::new();
    let mut nodes: Vec<SkelNode> = Vec::new();
    fn add(node: &ufbx::Node, index_of: &mut HashMap<u32, usize>, nodes: &mut Vec<SkelNode>) -> usize {
        if let Some(&i) = index_of.get(&node.element.typed_id) {
            return i;
        }
        let parent = node.parent.as_ref().map(|p| add(p, index_of, nodes));
        nodes.push(SkelNode { name: node.element.name.to_string(), parent, rest: fbx_mat(&node.node_to_world) });
        index_of.insert(node.element.typed_id, nodes.len() - 1);
        nodes.len() - 1
    }
    let mut skin = Vec::new();
    let (mut bind, mut joints, mut weights) = (Vec::new(), Vec::new(), Vec::new());
    for node in scene.nodes.iter() {
        let Some(m) = &node.mesh else { continue };
        let deformer = m.skin_deformers.first();
        // Which joint each skin cluster is; an unskinned mesh follows its own node.
        let cluster_joint: Vec<u16> = match deformer {
            Some(d) => d
                .clusters
                .iter()
                .map(|c| {
                    let bone = c.bone_node.as_ref().map(|b| add(b, &mut index_of, &mut nodes)).unwrap_or_else(|| add(node, &mut index_of, &mut nodes));
                    // The rest pose of a bone is where the skin was bound, not the file's
                    // current pose (they differ when the file was saved mid-animation).
                    nodes[bone].rest = fbx_mat(&c.bind_to_world);
                    skin.push(Joint { node: bone, inverse_bind: fbx_mat(&c.geometry_to_bone) });
                    (skin.len() - 1) as u16
                })
                .collect(),
            None => {
                let own = add(node, &mut index_of, &mut nodes);
                skin.push(Joint { node: own, inverse_bind: fbx_mat(&node.geometry_to_node) });
                vec![(skin.len() - 1) as u16]
            }
        };
        for i in 0..m.num_indices {
            let p = m.vertex_position[i];
            let n = if m.vertex_normal.exists { m.vertex_normal[i] } else { ufbx::Vec3 { x: 0.0, y: 1.0, z: 0.0 } };
            bind.push((Vec3::new(p.x as f32, p.y as f32, p.z as f32), Vec3::new(n.x as f32, n.y as f32, n.z as f32)));
            let (j, w) = match deformer {
                Some(d) => {
                    let vi = m.vertex_indices[i] as usize;
                    let list: Vec<(u16, f32)> = d
                        .vertices
                        .get(vi)
                        .map(|sv| {
                            (sv.weight_begin..sv.weight_begin + sv.num_weights)
                                .filter_map(|k| d.weights.get(k as usize))
                                .filter_map(|w| Some((*cluster_joint.get(w.cluster_index as usize)?, w.weight as f32)))
                                .collect()
                        })
                        .unwrap_or_default();
                    if list.is_empty() {
                        ([0; 4], [0.0; 4])
                    } else {
                        top4(list)
                    }
                }
                None => ([cluster_joint[0], 0, 0, 0], [1.0, 0.0, 0.0, 0.0]),
            };
            joints.push(j);
            weights.push(w);
        }
    }
    if bind.len() != mesh.vertices.len() {
        return Err(format!("{path}: skinned vertex count {} differs from the mesh ({})", bind.len(), mesh.vertices.len()));
    }
    // Clips: every node's model-space pose, 30 times a second.
    let mut clips = Vec::new();
    for stack in scene.anim_stacks.iter() {
        let duration = (stack.time_end - stack.time_begin).max(0.0) as f32;
        let fps = 30.0;
        let count = (duration * fps).ceil() as usize + 1;
        let mut frames = Vec::with_capacity(count);
        for f in 0..count {
            let t = stack.time_begin + (f as f64 / fps as f64).min(duration as f64);
            let eval = ufbx::evaluate_scene(&scene, &stack.anim, t, ufbx::EvaluateOpts::default()).map_err(|e| format!("{e:?}"))?;
            let mut pose = vec![Trs::default(); nodes.len()];
            for (&typed, &i) in &index_of {
                if let Some(n) = eval.nodes.get(typed as usize) {
                    pose[i] = Trs::from_mat(fbx_mat(&n.node_to_world));
                }
            }
            frames.push(pose);
        }
        let name = stack.element.name.to_string();
        clips.push(Clip { name: name.rsplit('|').next().unwrap_or(&name).to_string(), duration, fps, frames });
    }
    Ok(SkinnedModel { mesh, bind, joints, weights, nodes, skin, clips, humanoid: None, children: Vec::new() })
}

// ---------------------------------------------------------------- glTF

fn load_gltf(assets: &Assets, path: &str, bytes: &[u8]) -> Result<SkinnedModel, String> {
    let crate::mesh::ImportedGltf { document: doc, buffers, images, .. } = crate::mesh::import_gltf(assets, path, bytes)?;
    let get = |b: gltf::Buffer| buffers.get(b.index()).map(|d| d.0.as_slice());
    // Rest pose of every node (local), parents, model-space rest.
    let count = doc.nodes().len();
    let mut parent = vec![None; count];
    for n in doc.nodes() {
        for c in n.children() {
            parent[c.index()] = Some(n.index());
        }
    }
    let local_rest: Vec<(Vec3, Quat, Vec3)> = doc
        .nodes()
        .map(|n| {
            let (t, r, s) = n.transform().decomposed();
            (Vec3::from(t), Quat::from_array(r).normalize(), Vec3::from(s))
        })
        .collect();
    let world_of = |local: &[(Vec3, Quat, Vec3)]| -> Vec<Mat4> {
        let mut world: Vec<Option<Mat4>> = vec![None; count];
        fn resolve(i: usize, parent: &[Option<usize>], local: &[(Vec3, Quat, Vec3)], world: &mut Vec<Option<Mat4>>, depth: usize) -> Mat4 {
            if let Some(m) = world[i] {
                return m;
            }
            let (t, r, s) = local[i];
            let own = Mat4::from_scale_rotation_translation(s, r, t);
            let m = match parent[i] {
                Some(p) if depth < 64 => resolve(p, parent, local, world, depth + 1) * own,
                _ => own,
            };
            world[i] = Some(m);
            m
        }
        (0..count).map(|i| resolve(i, &parent, local, &mut world, 0)).collect()
    };
    let rest_world = world_of(&local_rest);
    let nodes: Vec<SkelNode> = doc.nodes().map(|n| SkelNode { name: n.name().unwrap_or("").to_string(), parent: parent[n.index()], rest: rest_world[n.index()] }).collect();

    let mut mesh = MeshData::default();
    let (mut bind, mut joints, mut weights, mut skin) = (Vec::new(), Vec::new(), Vec::new(), Vec::<Joint>::new());
    let mut parts = Vec::new();
    for node in doc.nodes() {
        let Some(gm) = node.mesh() else { continue };
        // Joints of this node's skin (or the node itself for a rigid mesh).
        let joint_base = skin.len() as u16;
        match node.skin() {
            Some(s) => {
                let ibms: Vec<Mat4> = s.reader(get).read_inverse_bind_matrices().map(|it| it.map(|m| Mat4::from_cols_array_2d(&m)).collect()).unwrap_or_default();
                for (k, j) in s.joints().enumerate() {
                    skin.push(Joint { node: j.index(), inverse_bind: ibms.get(k).copied().unwrap_or(Mat4::IDENTITY) });
                }
            }
            None => skin.push(Joint { node: node.index(), inverse_bind: Mat4::IDENTITY }),
        }
        let skinned = node.skin().is_some();
        for prim in gm.primitives() {
            if prim.mode() != gltf::mesh::Mode::Triangles {
                continue;
            }
            let r = prim.reader(get);
            let Some(pos) = r.read_positions() else { continue };
            let pos: Vec<Vec3> = pos.map(Vec3::from).collect();
            let nrm: Vec<Vec3> = r.read_normals().map(|n| n.map(Vec3::from).collect()).unwrap_or_else(|| vec![Vec3::Y; pos.len()]);
            let uvs: Vec<[f32; 2]> = r.read_tex_coords(0).map(|t| t.into_f32().collect()).unwrap_or_else(|| vec![[0.0; 2]; pos.len()]);
            let js: Vec<[u16; 4]> = r.read_joints(0).map(|j| j.into_u16().collect()).unwrap_or_default();
            let ws: Vec<[f32; 4]> = r.read_weights(0).map(|w| w.into_f32().collect()).unwrap_or_default();
            let base = mesh.vertices.len() as u32;
            let rest_rigid = rest_world[node.index()];
            for (i, p) in pos.iter().enumerate() {
                let n = nrm.get(i).copied().unwrap_or(Vec3::Y);
                let (j, w) = if skinned && i < js.len() && i < ws.len() {
                    let list: Vec<(u16, f32)> = (0..4).filter(|k| ws[i][*k] > 0.0).map(|k| (joint_base + js[i][k], ws[i][k])).collect();
                    top4(list)
                } else {
                    ([joint_base, 0, 0, 0], [1.0, 0.0, 0.0, 0.0])
                };
                // The rest-pose vertex in model space, for the static look and the bounds.
                let m: Mat4 = (0..4).filter(|k| w[*k] > 0.0).map(|k| {
                    let jt = &skin[j[k] as usize];
                    (rest_world[jt.node] * jt.inverse_bind) * w[k]
                }).fold(Mat4::ZERO, |a, b| a + b);
                let m = if skinned { m } else { rest_rigid };
                mesh.vertices.push(crate::mesh::Vertex { pos: m.transform_point3(*p).into(), normal: m.transform_vector3(n).normalize_or_zero().into(), uv: uvs.get(i).copied().unwrap_or([0.0; 2]) });
                bind.push((*p, n));
                joints.push(j);
                weights.push(w);
            }
            let first = mesh.indices.len() as u32;
            match r.read_indices() {
                Some(ix) => mesh.indices.extend(ix.into_u32().map(|i| base + i)),
                None => mesh.indices.extend((0..pos.len() as u32).map(|i| base + i)),
            }
            let albedo = crate::mesh::material_albedo(&images, &prim.material().pbr_metallic_roughness());
            if mesh.albedo.is_none() {
                mesh.albedo = albedo.clone();
            }
            parts.push(crate::mesh::MeshPart { first_index: first, index_count: mesh.indices.len() as u32 - first, albedo, normal: None, orm: None });
        }
    }
    if parts.len() > 1 {
        mesh.parts = parts;
    }
    if mesh.indices.is_empty() {
        return Err(format!("{path}: no triangles"));
    }
    // Clips.
    let mut clips = Vec::new();
    for anim in doc.animations() {
        let mut tracks = Vec::new();
        let mut duration = 0.0f32;
        for ch in anim.channels() {
            let r = ch.reader(get);
            let times: Vec<f32> = r.read_inputs().map(|t| t.collect()).unwrap_or_default();
            duration = duration.max(times.last().copied().unwrap_or(0.0));
            let cubic = ch.sampler().interpolation() == gltf::animation::Interpolation::CubicSpline;
            use gltf::animation::util::ReadOutputs;
            let values: Vec<[f32; 4]> = match r.read_outputs() {
                Some(ReadOutputs::Translations(v)) => v.map(|x| [x[0], x[1], x[2], 0.0]).collect(),
                Some(ReadOutputs::Scales(v)) => v.map(|x| [x[0], x[1], x[2], 0.0]).collect(),
                Some(ReadOutputs::Rotations(v)) => v.into_f32().collect(),
                _ => continue,
            };
            // Cubic splines store (in-tangent, value, out-tangent); keep the values.
            let values: Vec<[f32; 4]> = if cubic { values.chunks(3).filter_map(|c| c.get(1).copied()).collect() } else { values };
            tracks.push((ch.target().node().index(), ch.target().property(), times, values));
        }
        let fps = 30.0;
        let frames_n = (duration * fps).ceil() as usize + 1;
        let mut frames = Vec::with_capacity(frames_n);
        for f in 0..frames_n {
            let t = f as f32 / fps;
            let mut local = local_rest.clone();
            for (node, prop, times, values) in &tracks {
                if times.is_empty() || values.is_empty() {
                    continue;
                }
                let k = times.partition_point(|x| *x <= t).saturating_sub(1).min(values.len() - 1);
                let k2 = (k + 1).min(values.len() - 1).min(times.len() - 1);
                let span = times[k2] - times[k];
                let a = if span > 0.0 { ((t - times[k]) / span).clamp(0.0, 1.0) } else { 0.0 };
                let (v0, v1) = (values[k], values[k2]);
                use gltf::animation::Property;
                match prop {
                    Property::Translation => local[*node].0 = Vec3::new(v0[0], v0[1], v0[2]).lerp(Vec3::new(v1[0], v1[1], v1[2]), a),
                    Property::Scale => local[*node].2 = Vec3::new(v0[0], v0[1], v0[2]).lerp(Vec3::new(v1[0], v1[1], v1[2]), a),
                    Property::Rotation => local[*node].1 = Quat::from_array(v0).normalize().slerp(Quat::from_array(v1).normalize(), a),
                    _ => {}
                }
            }
            frames.push(world_of(&local).into_iter().map(Trs::from_mat).collect());
        }
        clips.push(Clip { name: anim.name().unwrap_or("Clip").to_string(), duration, fps, frames });
    }
    Ok(SkinnedModel { mesh, bind, joints, weights, nodes, skin, clips, humanoid: None, children: Vec::new() })
}

/// Loads a skinned model (FBX, glTF, GLB) with its clips, plus procedural humanoid clips.
pub fn load(assets: &Assets, path: &str) -> Result<SkinnedModel, String> {
    let bytes = assets.read(path).map_err(|e| format!("{path}: {e}"))?;
    let lower = path.to_ascii_lowercase();
    let mut model = if lower.ends_with(".fbx") {
        load_fbx(assets, path, &bytes)?
    } else if lower.ends_with(".glb") || lower.ends_with(".gltf") {
        load_gltf(assets, path, &bytes)?
    } else {
        return Err(format!("{path}: skinning needs an FBX or glTF model"));
    };
    model.in_place();
    crate::humanoid::add_clips(&mut model);
    model.humanoid = crate::humanoid::find_bones(&model);
    model.children = vec![Vec::new(); model.nodes.len()];
    for (i, n) in model.nodes.iter().enumerate() {
        if let Some(p) = n.parent {
            model.children[p].push(i);
        }
    }
    Ok(model)
}

/// Skinned models by path, loaded once (the asset source is the one given to
/// [`crate::meshmod::set_assets`]). `None` when the file is missing or not skinned.
pub fn model(path: &str) -> Option<Arc<SkinnedModel>> {
    static CACHE: Mutex<Option<HashMap<String, Option<Arc<SkinnedModel>>>>> = Mutex::new(None);
    let mut cache = CACHE.lock().ok()?;
    cache
        .get_or_insert_with(HashMap::new)
        .entry(path.to_string())
        .or_insert_with(|| {
            let assets = crate::meshmod::assets().unwrap_or(Assets::Dir(std::path::PathBuf::new()));
            load(&assets, path).ok().map(Arc::new)
        })
        .clone()
}

// ---------------------------------------------------------------- the component

/// Plays the clips of a skinned model (`MeshRenderer.mesh`). Changing `clip` cross-fades.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Animator {
    /// Clip to play (`Idle`, `Walk`, or a name from the file). Empty: the first clip.
    pub clip: String,
    /// Playback rate; 1 is the clip's own speed.
    pub speed: f32,
    /// Wrap at the end (walk cycles) or hold the last frame (a lunge, a fall).
    pub looping: bool,
    /// Paused when false (the pose stays).
    pub playing: bool,
    /// Seconds to blend from the previous clip into a new one.
    pub fade: f32,
    /// Clips the model offers (filled in by the engine; read only).
    #[serde(skip)]
    pub clips: Vec<String>,
    /// Seconds into the current clip (read only; set `clip` to restart).
    #[serde(skip)]
    pub time: f32,
}

impl Default for Animator {
    fn default() -> Self {
        Self { clip: String::new(), speed: 1.0, looping: true, playing: true, fade: 0.25, clips: Vec::new(), time: 0.0 }
    }
}

impl crate::components::Component for Animator {
    fn fields(&mut self) -> Vec<crate::reflect::Field<'_>> {
        use crate::reflect::Field;
        vec![Field::text("clip", &mut self.clip), Field::f32("speed", &mut self.speed), Field::bool("looping", &mut self.looping), Field::bool("playing", &mut self.playing), Field::f32("fade", &mut self.fade)]
    }
}

/// Run-time playback state next to an [`Animator`] (not saved).
#[derive(Clone, Default)]
pub struct AnimatorState {
    current: String,
    time: f32,
    previous: Option<(String, f32)>,
    fade_t: f32,
    fade_len: f32,
    wall: f32,
    version: u64,
    pose: Vec<Trs>,
    blend: Vec<Trs>,
    ik: crate::ik::IkState,
    ragdoll: crate::ragdoll::RagdollState,
}

/// Traces down for foot IK: `(from, length, the character)` → height of the hit (world space).
pub type GroundQuery<'a> = &'a dyn Fn(Vec3, f32, Entity) -> Option<f32>;

/// Advances every animator and skins its model. `simulate` is false in the editor outside Play:
/// the animation then runs on wall-clock `time` as a preview. Without a ground query the
/// feet are not planted (see [`update_with`]).
pub fn update(world: &mut World, time: f32, dt: f32, simulate: bool) {
    update_with(world, time, dt, simulate, None);
}

/// [`update`] with physics: `ground` plants the feet ([`crate::ik::CharacterIk`]); ragdolls
/// ([`crate::ragdoll::Ragdoll`]) run only when `simulate` is true. Called by the runtime after
/// the physics steps of a frame.
pub fn update_with(world: &mut World, time: f32, dt: f32, simulate: bool, ground: Option<GroundQuery>) {
    // Animators that were removed leave their skinned mesh (and ragdoll bodies) behind; drop them.
    let stale: Vec<Entity> = world.iter::<AnimatorState>().map(|(e, _)| e).filter(|e| !world.has::<Animator>(*e)).collect();
    for e in stale {
        if let Some(mut st) = world.remove::<AnimatorState>(e) {
            crate::ragdoll::deactivate(world, &mut st.ragdoll);
        }
        world.remove::<DynamicMesh>(e);
    }
    let list: Vec<(Entity, Animator, String)> = world.iter::<Animator>().filter_map(|(e, a)| Some((e, a.clone(), world.get::<MeshRenderer>(e)?.mesh.clone()))).collect();
    for (e, mut anim, mesh) in list {
        let Some(model) = model(&mesh) else { continue };
        let mut st = world.get::<AnimatorState>(e).cloned().unwrap_or_default();
        let step = if simulate { dt } else { (time - st.wall).clamp(0.0, 0.1) };
        st.wall = time;
        let wanted = if anim.clip.is_empty() { model.clips.first().map(|c| c.name.clone()).unwrap_or_default() } else { anim.clip.clone() };
        if wanted != st.current {
            if !st.current.is_empty() && anim.fade > 0.0 {
                st.previous = Some((st.current.clone(), st.time));
                st.fade_t = 0.0;
                st.fade_len = anim.fade;
            }
            st.current = wanted;
            st.time = 0.0;
        }
        let advance = if anim.playing { step * anim.speed } else { 0.0 };
        st.time += advance;
        let mut pose = std::mem::take(&mut st.pose);
        match model.clip(&st.current) {
            Some(c) => model.sample(c, st.time, anim.looping, &mut pose),
            None => pose = model.rest_pose(),
        }
        if let Some((prev, t)) = st.previous.clone() {
            st.fade_t += step;
            let k = (st.fade_t / st.fade_len.max(1e-3)).clamp(0.0, 1.0);
            let k = k * k * (3.0 - 2.0 * k);
            if k >= 1.0 {
                st.previous = None;
            } else if let Some(c) = model.clip(&prev) {
                let mut from = std::mem::take(&mut st.blend);
                model.sample(c, t + st.fade_t * anim.speed, true, &mut from);
                for (p, f) in pose.iter_mut().zip(&from) {
                    *p = f.lerp(p, k);
                }
                st.blend = from;
            }
        }
        // Ragdoll or IK on top of the clip.
        let owner_world = world.get::<crate::components::GlobalTransform>(e).map_or_else(|| crate::hierarchy::global_matrix(world, e), |g| g.0);
        let rd = world.get::<crate::ragdoll::Ragdoll>(e).cloned();
        let want_rag = simulate && model.humanoid.is_some() && rd.as_ref().is_some_and(|r| r.active);
        if want_rag && !st.ragdoll.is_active() {
            let velocity = world.get::<crate::components::RigidBody>(e).map_or(Vec3::ZERO, |r| r.velocity);
            st.ragdoll = crate::ragdoll::activate(world, e, &model, &pose, owner_world, rd.as_ref().unwrap(), velocity);
        } else if !want_rag && st.ragdoll.is_active() {
            crate::ragdoll::deactivate(world, &mut st.ragdoll);
        }
        if st.ragdoll.is_active() {
            crate::ragdoll::pose(world, &model, &st.ragdoll, owner_world, &mut pose);
        } else if let Some(ik) = world.get::<crate::ik::CharacterIk>(e).cloned() {
            let target = ik.look_point.or_else(|| {
                let t = (!ik.look_at.is_empty()).then(|| crate::hierarchy::find_by_name(world, &ik.look_at)).flatten()?;
                Some(world.get::<crate::components::GlobalTransform>(t).map_or_else(|| crate::hierarchy::global_matrix(world, t), |g| g.0).w_axis.truncate())
            });
            let probe = ground.map(|g| move |from: Vec3, len: f32| g(from, len, e));
            let probe_ref = probe.as_ref().map(|p| p as &dyn Fn(Vec3, f32) -> Option<f32>);
            crate::ik::apply(&model, &mut pose, owner_world, &ik, target, probe_ref, &mut st.ik, if simulate { dt } else { step });
        }
        let mut data = model.mesh.clone();
        model.skin_into(&pose, &mut data);
        st.pose = pose;
        st.version += 1;
        let version = st.version;
        anim.time = st.time;
        if anim.clips.len() != model.clips.len() {
            anim.clips = model.clips.iter().map(|c| c.name.clone()).collect();
        }
        if let Some(a) = world.get_mut::<Animator>(e) {
            a.time = anim.time;
            if a.clips.len() != anim.clips.len() {
                a.clips = anim.clips;
            }
        }
        world.insert(e, st);
        world.insert(e, DynamicMesh { data: Arc::new(data), version });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trs_round_trip_and_blend() {
        let m = Mat4::from_scale_rotation_translation(Vec3::splat(2.0), Quat::from_rotation_y(0.7), Vec3::new(1.0, 2.0, 3.0));
        let t = Trs::from_mat(m);
        assert!(t.mat().abs_diff_eq(m, 1e-4));
        let half = Trs::default().lerp(&t, 0.5);
        assert!((half.t - Vec3::new(0.5, 1.0, 1.5)).length() < 1e-5);
    }
}
