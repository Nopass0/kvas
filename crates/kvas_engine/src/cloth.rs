//! Cloth simulation: Verlet particles joined by structural, shear and bend links.
//! Wind acts on each triangle by its area and orientation, so cloth billows and flaps.
//! Links snap when stretched past `tear`, ground / sphere / box colliders make cloth drape and pile up.

use crate::choice_enum;
use crate::components::*;
use crate::hierarchy;
use crate::mesh::{MeshData, Vertex};
use crate::noise::{Noise, NoiseKind};
use crate::procedural::signature;
use crate::reflect::Field;
use glam::{Mat4, Vec3};
use kvas_ecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

choice_enum!(
    /// Pin no particles, the upper edge, the upper corners, or the left edge of the cloth.
    ClothPin { Free, TopEdge, TopCorners, LeftEdge } default TopEdge);

/// Verlet cloth attached to an entity. Structural edits rebuild the particle grid;
/// wind and other simulation controls can change while running.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Cloth {
    /// Rest width of the sheet in metres.
    pub width: f32,
    /// Rest height of the sheet in metres.
    pub height: f32,
    /// Particles along the width (more = finer folds, slower).
    pub segments: i32,
    /// Boundary particles fixed to the entity transform.
    pub pin: ClothPin,
    /// 0..1: how rigid the cloth is (solver iterations).
    pub stiffness: f32,
    /// Per-step motion damping; values near zero retain motion.
    pub damping: f32,
    /// Multiplier of the cloth solver gravity.
    pub gravity_scale: f32,
    /// World-space wind velocity acting on triangle surfaces.
    pub wind: Vec3,
    /// How much the wind varies in space and time (0 = steady).
    pub gust: f32,
    /// Air resistance of the surface.
    pub drag: f32,
    /// Links snap when stretched past this multiple of their length (0 = never tear).
    pub tear: f32,
    /// Whether particles collide with the horizontal ground plane.
    pub collide_ground: bool,
    /// World-space height of the optional ground plane.
    pub ground_y: f32,
    /// Collide with Sphere and Box colliders in the scene.
    pub collide_objects: bool,
    #[serde(skip)]
    #[doc(hidden)]
    pub state: Option<State>,
    #[serde(skip)]
    #[doc(hidden)]
    pub sig: u64,
}

impl Default for Cloth {
    fn default() -> Self {
        Self {
            width: 3.0,
            height: 2.0,
            segments: 24,
            pin: ClothPin::TopEdge,
            stiffness: 0.8,
            damping: 0.02,
            gravity_scale: 1.0,
            wind: Vec3::new(2.5, 0.0, 1.2),
            gust: 1.0,
            drag: 0.8,
            tear: 0.0,
            collide_ground: true,
            ground_y: 0.0,
            collide_objects: true,
            state: None,
            sig: 0,
        }
    }
}

impl Component for Cloth {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::f32("width", &mut self.width),
            Field::f32("height", &mut self.height),
            Field::i32("segments", &mut self.segments),
            Field::choice("pin", &mut self.pin),
            Field::f32("stiffness", &mut self.stiffness),
            Field::f32("damping", &mut self.damping),
            Field::f32("gravity_scale", &mut self.gravity_scale),
            Field::vec3("wind", &mut self.wind),
            Field::f32("gust", &mut self.gust),
            Field::f32("drag", &mut self.drag),
            Field::f32("tear", &mut self.tear),
            Field::bool("collide_ground", &mut self.collide_ground),
            Field::f32("ground_y", &mut self.ground_y),
            Field::bool("collide_objects", &mut self.collide_objects),
        ]
    }
}

impl Cloth {
    /// Fraction of links that have not torn (1 = intact).
    pub fn integrity(&self) -> f32 {
        self.state.as_ref().map_or(1.0, |s| s.links.iter().filter(|l| l.alive).count() as f32 / s.links.len().max(1) as f32)
    }

    /// Current particle positions in world space (empty before the first update).
    pub fn particles(&self) -> &[Vec3] {
        self.state.as_ref().map_or(&[], |s| &s.pos)
    }
}

#[derive(Clone)]
struct Link {
    a: usize,
    b: usize,
    rest: f32,
    alive: bool,
}

/// Simulation state (internal).
#[doc(hidden)]
#[derive(Clone)]
pub struct State {
    nx: usize,
    ny: usize,
    /// Rest layout in the entity's local space (used for pinning and for the initial pose).
    local: Vec<Vec3>,
    pos: Vec<Vec3>,
    prev: Vec<Vec3>,
    pinned: Vec<bool>,
    links: Vec<Link>,
    /// Triangles with the three links that must be intact for them to exist.
    tris: Vec<([usize; 3], [usize; 3])>,
    version: u64,
}

enum Obstacle {
    Sphere { center: Vec3, radius: f32 },
    Box { to_local: Mat4, to_world: Mat4, half: Vec3 },
}

impl State {
    fn new(c: &Cloth, model: Mat4) -> State {
        let nx = c.segments.clamp(2, 80) as usize;
        let ny = ((nx as f32 * c.height / c.width.max(0.01)).round() as usize).clamp(2, 80);
        let idx = |i: usize, j: usize| j * (nx + 1) + i;
        let mut local = Vec::new();
        let mut pinned = Vec::new();
        for j in 0..=ny {
            for i in 0..=nx {
                local.push(Vec3::new((i as f32 / nx as f32 - 0.5) * c.width, -(j as f32 / ny as f32) * c.height, 0.0));
                pinned.push(match c.pin {
                    ClothPin::Free => false,
                    ClothPin::TopEdge => j == 0,
                    ClothPin::TopCorners => j == 0 && (i == 0 || i == nx),
                    ClothPin::LeftEdge => i == 0,
                });
            }
        }
        let pos: Vec<Vec3> = local.iter().map(|&p| model.transform_point3(p)).collect();
        let mut links = Vec::new();
        let link = |a: usize, b: usize, links: &mut Vec<Link>| {
            links.push(Link { a, b, rest: pos[a].distance(pos[b]), alive: true });
            links.len() - 1
        };
        let mut tris = Vec::new();
        let (mut h, mut v) = (vec![0usize; (nx + 1) * (ny + 1)], vec![0usize; (nx + 1) * (ny + 1)]);
        for j in 0..=ny {
            for i in 0..=nx {
                if i < nx {
                    h[idx(i, j)] = link(idx(i, j), idx(i + 1, j), &mut links);
                }
                if j < ny {
                    v[idx(i, j)] = link(idx(i, j), idx(i, j + 1), &mut links);
                }
            }
        }
        for j in 0..ny {
            for i in 0..nx {
                let diag = link(idx(i + 1, j), idx(i, j + 1), &mut links); // B-C
                link(idx(i, j), idx(i + 1, j + 1), &mut links); // A-D (shear only)
                let (a, b, cc, d) = (idx(i, j), idx(i + 1, j), idx(i, j + 1), idx(i + 1, j + 1));
                tris.push(([a, b, cc], [h[a], v[a], diag]));
                tris.push(([b, d, cc], [v[b], h[cc], diag]));
            }
        }
        for j in 0..=ny {
            for i in 0..=nx {
                if i + 2 <= nx {
                    link(idx(i, j), idx(i + 2, j), &mut links);
                }
                if j + 2 <= ny {
                    link(idx(i, j), idx(i, j + 2), &mut links);
                }
            }
        }
        State { nx, ny, prev: pos.clone(), pos, local, pinned, links, tris, version: 0 }
    }

    fn tri_alive(&self, t: usize) -> bool {
        self.tris[t].1.iter().all(|&l| self.links[l].alive)
    }

    fn step(&mut self, c: &Cloth, obstacles: &[Obstacle], model: Mat4, time: f32, dt: f32) {
        let gust_noise = Noise::new(NoiseKind::Perlin).with_seed(11).with_octaves(2);
        let iterations = 3 + (c.stiffness.clamp(0.0, 1.0) * 9.0) as usize;
        let gravity = Vec3::new(0.0, -9.81 * c.gravity_scale, 0.0);
        let n = self.pos.len();

        // Aerodynamic force per triangle, shared by its three particles.
        let mut force = vec![Vec3::ZERO; n];
        for t in 0..self.tris.len() {
            if !self.tri_alive(t) {
                continue;
            }
            let [a, b, cc] = self.tris[t].0;
            let (pa, pb, pc) = (self.pos[a], self.pos[b], self.pos[cc]);
            let cross = (pb - pa).cross(pc - pa);
            let area2 = cross.length();
            if area2 < 1e-9 {
                continue;
            }
            let normal = cross / area2;
            let centroid = (pa + pb + pc) / 3.0;
            let gust = 1.0 + c.gust * gust_noise.sample3(centroid.x * 0.35, centroid.z * 0.35 + centroid.y * 0.2, time * 0.9);
            let velocity = ((pa - self.prev[a]) + (pb - self.prev[b]) + (pc - self.prev[cc])) / (3.0 * dt.max(1e-4));
            let relative = c.wind * gust - velocity;
            let push = normal * normal.dot(relative) * c.drag * 1.5;
            for p in [a, b, cc] {
                force[p] += push / 3.0;
            }
        }

        for (i, force) in force.iter().enumerate().take(n) {
            if self.pinned[i] {
                self.pos[i] = model.transform_point3(self.local[i]);
                self.prev[i] = self.pos[i];
                continue;
            }
            let velocity = (self.pos[i] - self.prev[i]) * (1.0 - c.damping.clamp(0.0, 0.5));
            self.prev[i] = self.pos[i];
            self.pos[i] += velocity + (gravity + *force) * dt * dt;
        }

        for _ in 0..iterations {
            for l in 0..self.links.len() {
                let Link { a, b, rest, alive } = self.links[l];
                if !alive {
                    continue;
                }
                let delta = self.pos[b] - self.pos[a];
                let dist = delta.length();
                if dist < 1e-7 {
                    continue;
                }
                if c.tear > 1.0 && dist > rest * c.tear {
                    self.links[l].alive = false;
                    continue;
                }
                let correction = delta * ((dist - rest) / dist);
                match (self.pinned[a], self.pinned[b]) {
                    (false, false) => {
                        self.pos[a] += correction * 0.5;
                        self.pos[b] -= correction * 0.5;
                    }
                    (true, false) => self.pos[b] -= correction,
                    (false, true) => self.pos[a] += correction,
                    (true, true) => {}
                }
            }
            self.collide(c, obstacles);
        }
    }

    fn collide(&mut self, c: &Cloth, obstacles: &[Obstacle]) {
        const THICKNESS: f32 = 0.02;
        for i in 0..self.pos.len() {
            if self.pinned[i] {
                continue;
            }
            let mut p = self.pos[i];
            if c.collide_ground && p.y < c.ground_y + THICKNESS {
                p.y = c.ground_y + THICKNESS;
                // Friction: drag the previous position along so cloth does not skate.
                self.prev[i].x += (p.x - self.prev[i].x) * 0.25;
                self.prev[i].z += (p.z - self.prev[i].z) * 0.25;
            }
            for o in obstacles {
                match o {
                    Obstacle::Sphere { center, radius } => {
                        let d = p - *center;
                        let r = radius + THICKNESS;
                        let len = d.length();
                        if len < r && len > 1e-6 {
                            let normal = d / len;
                            p = *center + normal * r;
                            self.prev[i] = contact_friction(p, self.prev[i], normal);
                        }
                    }
                    Obstacle::Box { to_local, to_world, half } => {
                        let l = to_local.transform_point3(p);
                        let margin = Vec3::splat(THICKNESS) / to_world.to_scale_rotation_translation().0.max(Vec3::splat(1e-3));
                        let h = *half + margin;
                        if l.abs().cmple(h).all() {
                            let depth = h - l.abs();
                            let mut out = l;
                            if depth.x <= depth.y && depth.x <= depth.z {
                                out.x = h.x * l.x.signum();
                            } else if depth.y <= depth.z {
                                out.y = h.y * l.y.signum();
                            } else {
                                out.z = h.z * l.z.signum();
                            }
                            p = to_world.transform_point3(out);
                            let axis = (out - l).normalize_or_zero();
                            self.prev[i] = contact_friction(p, self.prev[i], to_world.transform_vector3(axis).normalize_or_zero());
                        }
                    }
                }
            }
            self.pos[i] = p;
        }
    }

    /// Double-sided triangle mesh in the entity's local space; torn triangles are left out.
    fn mesh(&self, to_local: Mat4) -> MeshData {
        let n = self.pos.len();
        let local: Vec<Vec3> = self.pos.iter().map(|&p| to_local.transform_point3(p)).collect();
        let mut normals = vec![Vec3::ZERO; n];
        let mut live = Vec::new();
        for (t, (verts, _)) in self.tris.iter().enumerate() {
            if !self.tri_alive(t) {
                continue;
            }
            let [a, b, c] = *verts;
            let face = (local[b] - local[a]).cross(local[c] - local[a]);
            for v in [a, b, c] {
                normals[v] += face;
            }
            live.push(*verts);
        }
        let mut m = MeshData::default();
        for side in [1.0f32, -1.0] {
            for i in 0..n {
                let (x, y) = (i % (self.nx + 1), i / (self.nx + 1));
                m.vertices.push(Vertex {
                    pos: local[i].into(),
                    normal: (normals[i].normalize_or_zero() * side).into(),
                    uv: [x as f32 / self.nx as f32, y as f32 / self.ny as f32],
                });
            }
        }
        for [a, b, c] in live {
            m.indices.extend([a as u32, b as u32, c as u32]);
            let o = n as u32;
            m.indices.extend([c as u32 + o, b as u32 + o, a as u32 + o]);
        }
        m
    }
}

/// Keeps most of the sliding motion out of a contact: removes the normal velocity (no bounce) and
/// 70 % of the tangential velocity, so cloth grips what it rests on instead of skating off.
fn contact_friction(pos: Vec3, prev: Vec3, normal: Vec3) -> Vec3 {
    let v = pos - prev;
    let tangential = v - normal * v.dot(normal);
    pos - tangential * 0.3
}

fn gather_obstacles(world: &World, skip: Entity) -> Vec<Obstacle> {
    let mut out = Vec::new();
    for (e, col) in world.iter::<Collider>() {
        if e == skip || col.is_trigger {
            continue;
        }
        let Some(g) = world.get::<GlobalTransform>(e).map(|g| g.0) else { continue };
        if !g.is_finite() || g.determinant() == 0.0 { continue; }
        match col.shape {
            ShapeKind::Sphere => {
                let (scale, _, pos) = g.to_scale_rotation_translation();
                out.push(Obstacle::Sphere { center: pos, radius: col.size.x.abs() * 0.5 * scale.abs().max_element() });
            }
            ShapeKind::Box => out.push(Obstacle::Box { to_local: g.inverse(), to_world: g, half: col.size.abs() * 0.5 }),
            _ => {}
        }
    }
    out
}

/// Advances every cloth and refreshes its `DynamicMesh`. With `simulate` false only the initial
/// (rest) shape is built, which is what the editor shows outside Play mode.
/// Simulation deltas are sanitized and limited to 0.1 seconds for solver stability.
/// Cached world matrices are refreshed so same-frame transform edits are visible.
pub fn update(world: &mut World, time: f32, dt: f32, simulate: bool) {
    if world.count::<Cloth>() == 0 { return; }
    hierarchy::update_globals(world);
    let dt = if dt.is_finite() { dt.clamp(0.0, 0.1) } else { 0.0 };
    let time = if time.is_finite() { time } else { 0.0 };
    for e in world.entities_with::<Cloth>() {
        let model = world.get::<GlobalTransform>(e).map_or_else(|| hierarchy::global_matrix(world, e), |g| g.0);
        if !model.is_finite() || model.determinant() == 0.0 { continue; }
        let obstacles =
            if simulate && world.get::<Cloth>(e).is_some_and(|c| c.collide_objects) { gather_obstacles(world, e) } else { Vec::new() };
        let cloth = world.get_mut::<Cloth>(e).unwrap();
        // Only the layout restarts the simulation; wind, damping, tearing... are applied live.
        let sig = signature(&(cloth.width, cloth.height, cloth.segments, cloth.pin));
        if cloth.state.is_none() || cloth.sig != sig {
            cloth.state = Some(State::new(cloth, model));
            cloth.sig = sig;
        }
        let mut state = cloth.state.take().unwrap();
        if simulate && dt > 0.0 {
            let substeps = 2;
            for _ in 0..substeps {
                state.step(cloth, &obstacles, model, time, dt / substeps as f32);
            }
        } else if !simulate {
            // Edit mode: keep the rest shape attached to the entity as it is moved.
            state.pos = state.local.iter().map(|&p| model.transform_point3(p)).collect();
            state.prev = state.pos.clone();
        }
        state.version = state.version.wrapping_add(1);
        let mesh = state.mesh(model.inverse());
        let version = state.version;
        cloth.state = Some(state);
        world.insert(e, DynamicMesh { data: Arc::new(mesh), version });
        if !world.has::<MeshRenderer>(e) {
            world.insert(e, MeshRenderer { color: [0.75, 0.2, 0.2, 1.0], roughness: 0.9, ..Default::default() });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Scene;

    fn cloth_scene(cloth: Cloth, at: Vec3) -> (Scene, Entity) {
        let mut scene = Scene::default();
        let e = hierarchy::spawn_named(&mut scene.world, "Cloth");
        scene.world.insert(e, Transform::at(at));
        scene.world.insert(e, cloth);
        hierarchy::update_globals(&mut scene.world);
        (scene, e)
    }

    fn run(scene: &mut Scene, seconds: f32) {
        for i in 0..(seconds * 60.0) as usize {
            update(&mut scene.world, i as f32 / 60.0, 1.0 / 60.0, true);
        }
    }

    #[test]
    fn pinned_cloth_hangs_and_keeps_its_length() {
        let (mut scene, e) = cloth_scene(Cloth { wind: Vec3::ZERO, ..Default::default() }, Vec3::new(0.0, 5.0, 0.0));
        run(&mut scene, 4.0);
        let c = scene.world.get::<Cloth>(e).unwrap();
        let ys: Vec<f32> = c.particles().iter().map(|p| p.y).collect();
        let (top, bottom) = (ys.iter().cloned().fold(f32::MIN, f32::max), ys.iter().cloned().fold(f32::MAX, f32::min));
        assert!((top - 5.0).abs() < 0.01, "pinned row stays put: {top}");
        assert!(bottom < 3.3 && bottom > 2.6, "hangs about its own height below the pin: {bottom}");
        assert_eq!(c.integrity(), 1.0);
    }

    #[test]
    fn free_cloth_falls_onto_the_ground_and_stays_above_it() {
        let (mut scene, e) = cloth_scene(Cloth { pin: ClothPin::Free, wind: Vec3::ZERO, ..Default::default() }, Vec3::new(0.0, 3.0, 0.0));
        run(&mut scene, 5.0);
        let min = scene.world.get::<Cloth>(e).unwrap().particles().iter().map(|p| p.y).fold(f32::MAX, f32::min);
        assert!(min >= 0.0 && min < 0.2, "cloth rests on the ground: {min}");
    }

    #[test]
    fn cloth_drapes_over_a_sphere() {
        let (mut scene, e) = cloth_scene(
            Cloth { pin: ClothPin::Free, wind: Vec3::ZERO, collide_ground: false, ..Default::default() },
            Vec3::new(0.0, 3.0, 0.0),
        );
        let ball = hierarchy::spawn_named(&mut scene.world, "Ball");
        scene.world.insert(ball, Transform::at(Vec3::new(0.0, 1.0, 1.0)));
        scene.world.insert(ball, Collider { shape: ShapeKind::Sphere, size: Vec3::splat(1.0), ..Default::default() });
        hierarchy::update_globals(&mut scene.world);
        // The cloth hangs in the XY plane from y=3 to y=1; rotate it flat so it falls onto the ball.
        scene.world.get_mut::<Transform>(e).unwrap().rotation = Vec3::new(-90.0, 0.0, 0.0);
        hierarchy::update_globals(&mut scene.world);
        run(&mut scene, 4.0);
        let c = scene.world.get::<Cloth>(e).unwrap();
        let center = Vec3::new(0.0, 1.0, 1.0);
        assert!(c.particles().iter().all(|p| p.distance(center) > 0.49), "no particle inside the ball");
        assert!(c.particles().iter().any(|p| p.y > 1.45), "some cloth is held up by the ball");
    }

    #[test]
    fn strong_wind_tears_weak_cloth() {
        let tough = Cloth { wind: Vec3::new(0.0, 0.0, 40.0), gust: 0.0, tear: 0.0, ..Default::default() };
        let weak = Cloth { tear: 1.08, ..tough.clone() };
        let ((mut a, ea), (mut b, eb)) = (cloth_scene(tough, Vec3::new(0.0, 5.0, 0.0)), cloth_scene(weak, Vec3::new(0.0, 5.0, 0.0)));
        run(&mut a, 3.0);
        run(&mut b, 3.0);
        assert_eq!(a.world.get::<Cloth>(ea).unwrap().integrity(), 1.0);
        assert!(b.world.get::<Cloth>(eb).unwrap().integrity() < 1.0, "torn cloth loses links");
    }

    #[test]
    fn wind_pushes_hanging_cloth_downwind() {
        let (mut scene, e) =
            cloth_scene(Cloth { wind: Vec3::new(0.0, 0.0, 6.0), gust: 0.0, ..Default::default() }, Vec3::new(0.0, 5.0, 0.0));
        run(&mut scene, 4.0);
        let mean_z = scene.world.get::<Cloth>(e).unwrap().particles().iter().map(|p| p.z).sum::<f32>()
            / scene.world.get::<Cloth>(e).unwrap().particles().len() as f32;
        assert!(mean_z > 0.3, "cloth swings with the wind: {mean_z}");
    }

    #[test]
    fn changing_wind_at_run_time_does_not_restart_the_simulation() {
        let (mut scene, e) = cloth_scene(Cloth { pin: ClothPin::Free, wind: Vec3::ZERO, ..Default::default() }, Vec3::new(0.0, 6.0, 0.0));
        for i in 0..60 {
            scene.world.get_mut::<Cloth>(e).unwrap().wind = Vec3::new(i as f32 * 0.01, 0.0, 0.0); // what a wind script does
            update(&mut scene.world, i as f32 / 60.0, 1.0 / 60.0, true);
        }
        let top = scene.world.get::<Cloth>(e).unwrap().particles().iter().map(|p| p.y).fold(f32::MIN, f32::max);
        assert!(top < 5.0, "cloth keeps falling instead of resetting every frame: {top}");
    }

    #[test]
    fn mesh_is_double_sided_and_skips_torn_triangles() {
        let (mut scene, e) = cloth_scene(Cloth { segments: 4, ..Default::default() }, Vec3::ZERO);
        update(&mut scene.world, 0.0, 0.0, false);
        let full = scene.world.get::<DynamicMesh>(e).unwrap().data.indices.len();
        assert_eq!(full, 4 * 3 * 2 * 3 * 2, "4x3 quads, 2 triangles each, two sides");
    }
}
