//! Built-in components. Every one is plain serde data plus an optional `fields()` listing for the inspector.

use crate::choice_enum;
use crate::reflect::{AssetKind, Field};
use glam::{EulerRot, Mat4, Quat, Vec3};
use kvas_ecs::{Entity, World};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Editor-visible component contract: identity, serde persistence and editable fields.
/// Built-ins are registered in both [`register_all`] and [`crate::registry::Registry`].
/// Script libraries may access built-ins but must not register custom unloadable storage.
pub trait Component: kvas_ecs::Component + Serialize + DeserializeOwned + Default + Clone {
    /// Whether the user may remove it in the inspector.
    const REMOVABLE: bool = true;
    /// Mutable field descriptors used by the Inspector and script property serialization.
    fn fields(&mut self) -> Vec<Field<'_>> {
        Vec::new()
    }
}

/// Runtime link to the parent entity. Saved in scenes as an index, not as a component.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Parent(pub Entity);

/// World-space matrix, recomputed every frame from `Transform` and `Parent`.
#[derive(Clone, Copy, Debug)]
pub struct GlobalTransform(pub Mat4);

/// Human-readable entity label. Names are not required to be unique.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Name(pub String);

impl Component for Name {
    const REMOVABLE: bool = false;
}

/// Local translation, Euler rotation and scale relative to the parent.
///
/// Units: metres, degrees and dimensionless scale. Rotations use Y-X-Z order,
/// +Y is up and -Z is forward. Scale defaults to one. Shear is not representable.
///
/// ```
/// use kvas_engine::components::Transform;
/// use glam::Vec3;
/// let mut pose = Transform::at(Vec3::new(0.0, 1.0, 0.0));
/// assert!(pose.look_at(Vec3::new(0.0, 1.0, -1.0), Vec3::Y));
/// assert!((pose.forward() - Vec3::NEG_Z).length() < 1e-5);
/// ```
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Transform {
    /// Translation in parent-local metres.
    pub position: Vec3,
    /// Euler angles in degrees.
    pub rotation: Vec3,
    /// Per-axis local scale, one for an unscaled object.
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self { position: Vec3::ZERO, rotation: Vec3::ZERO, scale: Vec3::ONE }
    }
}

impl Transform {
    /// Identity orientation/scale positioned at a parent-local point.
    pub fn at(position: Vec3) -> Self {
        Self { position, ..Default::default() }
    }

    /// Converts stored Euler degrees to a quaternion using Y-X-Z rotation order.
    pub fn quat(&self) -> Quat {
        let r = self.rotation * (std::f32::consts::PI / 180.0);
        Quat::from_euler(EulerRot::YXZ, r.y, r.x, r.z)
    }

    /// Stores a unit quaternion as Euler degrees; equivalent angles may wrap.
    /// The caller must supply a finite normalized quaternion.
    pub fn set_quat(&mut self, q: Quat) {
        let (y, x, z) = q.to_euler(EulerRot::YXZ);
        self.rotation = Vec3::new(x, y, z) * (180.0 / std::f32::consts::PI);
    }

    /// Local TRS matrix for column-vector multiplication (`translation * rotation * scale`).
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.quat(), self.position)
    }

    /// Decomposes a finite, non-singular TRS matrix.
    /// Shear cannot be retained. Use [`Self::try_from_matrix`] for unchecked input.
    pub fn from_matrix(m: Mat4) -> Self {
        let (scale, rot, position) = m.to_scale_rotation_translation();
        let mut t = Self { position, rotation: Vec3::ZERO, scale };
        t.set_quat(rot);
        t
    }

    /// Local -Z rotated into parent space, ignoring scale.
    pub fn forward(&self) -> Vec3 {
        self.quat() * Vec3::NEG_Z
    }
    /// Local +X rotated into parent space, ignoring scale.
    pub fn right(&self) -> Vec3 {
        self.quat() * Vec3::X
    }
    /// Local +Y rotated into parent space, ignoring scale.
    pub fn up(&self) -> Vec3 {
        self.quat() * Vec3::Y
    }

    /// Checks that every stored scalar is finite; zero scale remains valid data.
    pub fn is_finite(&self) -> bool {
        self.position.is_finite() && self.rotation.is_finite() && self.scale.is_finite()
    }

    /// Decomposes a finite non-singular matrix, returning `None` for invalid data.
    /// Perspective and shear matrices are rejected by checking the reconstructed TRS.
    pub fn try_from_matrix(matrix: Mat4) -> Option<Self> {
        if !matrix.is_finite() || matrix.determinant() == 0.0 {
            return None;
        }
        let pose = Self::from_matrix(matrix);
        if !pose.is_finite() || !pose.matrix().abs_diff_eq(matrix, 1e-4) {
            return None;
        }
        Some(pose)
    }

    /// Transforms a local point into parent space, including translation and scale.
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.matrix().transform_point3(point)
    }

    /// Rotates a direction into parent space, ignoring translation and scale.
    pub fn transform_direction(&self, direction: Vec3) -> Vec3 {
        self.quat() * direction
    }

    /// Converts a parent-space point to local space, or `None` for a singular/invalid pose.
    pub fn inverse_transform_point(&self, point: Vec3) -> Option<Vec3> {
        let matrix = self.matrix();
        if !point.is_finite() || !matrix.is_finite() || matrix.determinant() == 0.0 {
            return None;
        }
        let local = matrix.inverse().transform_point3(point);
        local.is_finite().then_some(local)
    }

    /// Orients local -Z toward a parent-space target while keeping position/scale.
    /// Returns `false` without mutation for coincident points, non-finite data,
    /// a zero up vector or an up vector parallel to the viewing direction.
    pub fn look_at(&mut self, target: Vec3, up: Vec3) -> bool {
        if !self.position.is_finite() || !target.is_finite() || !up.is_finite() {
            return false;
        }
        let Some(forward) = (target - self.position).try_normalize() else {
            return false;
        };
        let Some(right) = forward.cross(up).try_normalize() else {
            return false;
        };
        let up = right.cross(forward);
        self.set_quat(Quat::from_mat3(&glam::Mat3::from_cols(right, up, -forward)).normalize());
        true
    }

    /// Interpolates translation/scale linearly and orientation along the shortest
    /// quaternion arc. `t` is clamped to `[0, 1]`; non-finite `t` selects `self`.
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        let t = if t.is_finite() { t.clamp(0.0, 1.0) } else { 0.0 };
        let mut pose =
            Self { position: self.position.lerp(other.position, t), scale: self.scale.lerp(other.scale, t), rotation: Vec3::ZERO };
        pose.set_quat(self.quat().slerp(other.quat(), t));
        pose
    }
}

impl Component for Transform {
    const REMOVABLE: bool = false;
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::vec3("position", &mut self.position),
            Field::vec3("rotation", &mut self.rotation),
            Field::vec3("scale", &mut self.scale),
        ]
    }
}

/// Static/procedural mesh appearance consumed by the renderer.
/// Texture/shader paths are asset-relative or built-in procedural keys.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MeshRenderer {
    /// `primitive:<name>` or a model path (.obj / .gltf / .glb).
    pub mesh: String,
    /// Base RGBA tint. Alpha below one enables the renderer's transparency path.
    pub color: [f32; 4],
    /// Surface roughness; zero is smooth and one is fully rough.
    pub roughness: f32,
    /// Emission multiplier added independently of incoming light.
    pub emissive: f32,
    /// 0 = plastic-like, 1 = metal (reflections take the surface colour, no diffuse).
    pub metallic: f32,
    /// Mirror strength; fades in at grazing angles (Fresnel). 1 with a low colour alpha gives glass.
    pub reflectivity: f32,
    /// Whether this mesh contributes to the directional shadow map.
    pub cast_shadows: bool,
    /// Free parameters passed to custom shaders as `object.user0` / `object.user1`.
    pub user: [f32; 8],
    /// How many times the texture repeats across the mesh UVs.
    pub tiling: f32,
    /// Base-colour texture asset or procedural key; empty uses a white fallback.
    pub texture: String,
    /// Tangent-space normal map (OpenGL convention: green up).
    pub normal_map: String,
    /// Packed occlusion (R) / roughness (G) / metallic (B) texture, or an `orm:` key composed from
    /// separate maps. Any normal or ORM map switches the material to PBR shading.
    pub orm_map: String,
    /// Tangent-space normal-map intensity; one uses the map unchanged.
    pub normal_strength: f32,
    /// Triplanar mapping scale in metres per texture repeat (0 = use the mesh UVs). Good for rocks.
    pub triplanar: f32,
    /// Custom WGSL file; empty means the built-in lit shader.
    pub shader: String,
}

impl Default for MeshRenderer {
    fn default() -> Self {
        Self {
            mesh: "primitive:cube".into(),
            color: [0.8, 0.8, 0.8, 1.0],
            roughness: 0.5,
            emissive: 0.0,
            metallic: 0.0,
            reflectivity: 0.0,
            cast_shadows: true,
            user: [0.0; 8],
            tiling: 1.0,
            texture: String::new(),
            normal_map: String::new(),
            orm_map: String::new(),
            normal_strength: 1.0,
            triplanar: 0.0,
            shader: String::new(),
        }
    }
}

impl Component for MeshRenderer {
    fn fields(&mut self) -> Vec<Field<'_>> {
        let [p1, p2, p3, p4, ..] = &mut self.user;
        vec![
            Field::asset("mesh", &mut self.mesh, AssetKind::Mesh),
            Field::color("color", &mut self.color),
            Field::f32("roughness", &mut self.roughness),
            Field::f32("emissive", &mut self.emissive),
            Field::f32("metallic", &mut self.metallic),
            Field::f32("reflectivity", &mut self.reflectivity),
            Field::bool("cast_shadows", &mut self.cast_shadows),
            Field::f32("tiling", &mut self.tiling),
            Field::f32("shader param 1", p1),
            Field::f32("shader param 2", p2),
            Field::f32("shader param 3", p3),
            Field::f32("shader param 4", p4),
            Field::asset("texture", &mut self.texture, AssetKind::Texture),
            Field::asset("normal_map", &mut self.normal_map, AssetKind::Texture),
            Field::asset("orm_map", &mut self.orm_map, AssetKind::Texture),
            Field::f32("normal_strength", &mut self.normal_strength),
            Field::f32("triplanar", &mut self.triplanar),
            Field::asset("shader", &mut self.shader, AssetKind::Shader),
        ]
    }
}

choice_enum!(
    /// Procedural mesh families. Cube/plane use size, round shapes use radii, and Rock uses seed.
    ShapeType { Cube, Sphere, Plane, Cylinder, Capsule, Cone, Torus, Rock } default Cube);

/// Parametric mesh. When an entity has both `Shape` and `MeshRenderer`, the shape is what is drawn.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Shape {
    /// Procedural shape family.
    pub kind: ShapeType,
    /// Cube: width/height/depth. Plane: width (x) and depth (z).
    pub size: Vec3,
    /// Sphere/cylinder/cone/capsule radius; torus: distance from the centre to the tube.
    pub radius: f32,
    /// Cylinder: top radius (0 = cone). Torus: tube radius.
    pub radius2: f32,
    /// Full shape height in local metres.
    pub height: f32,
    /// Segments around the axis (more = smoother).
    pub segments: i32,
    /// Rings along the surface (sphere/capsule/torus tube) or plane subdivisions.
    pub rings: i32,
    /// Random seed (rocks).
    pub seed: i32,
}

impl Default for Shape {
    fn default() -> Self {
        Self { kind: ShapeType::Cube, size: Vec3::ONE, radius: 0.5, radius2: 0.5, height: 1.0, segments: 32, rings: 16, seed: 1 }
    }
}

impl Shape {
    /// Creates default parameters for a shape family (including a zero top radius for cones).
    pub fn of(kind: ShapeType) -> Self {
        let base = Self { kind, ..Default::default() };
        match kind {
            ShapeType::Capsule => Self { height: 2.0, ..base },
            ShapeType::Torus => Self { radius: 0.5, radius2: 0.18, segments: 48, rings: 24, ..base },
            ShapeType::Plane => Self { rings: 16, ..base },
            ShapeType::Rock => Self { size: Vec3::new(1.6, 1.0, 1.3), radius2: 1.0, segments: 5, ..base },
            _ => base,
        }
    }

    /// Encodes sanitized geometry parameters as a stable procedural mesh cache key.
    pub fn key(&self) -> String {
        use crate::reflect::Choice;
        crate::mesh::ProcParams {
            kind: ShapeType::OPTIONS[self.kind.to_index()].to_lowercase(),
            size: self.size,
            radius: self.radius,
            radius2: self.radius2,
            height: self.height,
            segments: self.segments.max(3) as u32,
            rings: self.rings.max(2) as u32,
            seed: self.seed as u32,
        }
        .key()
    }
}

impl Component for Shape {
    fn fields(&mut self) -> Vec<Field<'_>> {
        let kind = self.kind;
        let mut f = vec![Field::choice("kind", &mut self.kind)];
        match kind {
            ShapeType::Cube => f.push(Field::vec3("size", &mut self.size)),
            ShapeType::Plane => {
                f.push(Field::vec3("size", &mut self.size));
                f.push(Field::i32("subdivisions", &mut self.rings));
            }
            ShapeType::Sphere => {
                f.push(Field::f32("radius", &mut self.radius));
                f.push(Field::i32("segments", &mut self.segments));
                f.push(Field::i32("rings", &mut self.rings));
            }
            ShapeType::Capsule => {
                f.push(Field::f32("radius", &mut self.radius));
                f.push(Field::f32("height", &mut self.height));
                f.push(Field::i32("segments", &mut self.segments));
                f.push(Field::i32("rings", &mut self.rings));
            }
            ShapeType::Cylinder => {
                f.push(Field::f32("radius", &mut self.radius));
                f.push(Field::f32("top radius", &mut self.radius2));
                f.push(Field::f32("height", &mut self.height));
                f.push(Field::i32("segments", &mut self.segments));
            }
            ShapeType::Cone => {
                f.push(Field::f32("radius", &mut self.radius));
                f.push(Field::f32("height", &mut self.height));
                f.push(Field::i32("segments", &mut self.segments));
            }
            ShapeType::Rock => {
                f.push(Field::vec3("size", &mut self.size));
                f.push(Field::f32("ruggedness", &mut self.radius2));
                f.push(Field::i32("detail", &mut self.segments));
                f.push(Field::i32("seed", &mut self.seed));
            }
            ShapeType::Torus => {
                f.push(Field::f32("radius", &mut self.radius));
                f.push(Field::f32("tube radius", &mut self.radius2));
                f.push(Field::i32("segments", &mut self.segments));
                f.push(Field::i32("tube rings", &mut self.rings));
            }
        }
        f
    }
}

/// Marks entities created by systems (e.g. grass chunks). They are hidden from the hierarchy,
/// never saved in scenes and not pickable.
#[derive(Clone, Copy, Default)]
pub struct Transient;

/// Marks a chunk entity created for a grass patch.
#[derive(Clone, Copy, Default)]
pub struct GrassChunk;

/// Geometry produced at run time by a system (cloth, smoke, grass...). Not saved in scenes.
/// Bump `version` whenever `data` changes so the renderer re-uploads it.
#[derive(Clone, Default)]
pub struct DynamicMesh {
    /// Shared immutable CPU geometry; clone the Arc to inspect without copying vertices.
    pub data: std::sync::Arc<crate::mesh::MeshData>,
    /// Change counter used to invalidate GPU buffers when geometry is rebuilt.
    pub version: u64,
}

/// The mesh key an entity is drawn with: dynamic geometry, else its `Shape`, else `MeshRenderer.mesh`.
pub fn mesh_key(world: &World, e: Entity) -> Option<String> {
    let mr = world.get::<MeshRenderer>(e)?;
    if world.has::<DynamicMesh>(e) {
        return Some(format!("dyn:{}", e.to_bits()));
    }
    Some(world.get::<Shape>(e).map_or_else(|| mr.mesh.clone(), Shape::key))
}

/// Perspective or orthographic scene camera. The first primary camera is selected for gameplay.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Camera {
    /// Vertical perspective field of view in degrees. Ignored for orthographic cameras.
    pub fov: f32,
    /// Near clipping distance in metres; must be positive for perspective projection.
    pub near: f32,
    /// Far clipping distance in metres; must exceed near.
    pub far: f32,
    /// The first primary camera is used to render the game.
    pub primary: bool,
    /// Orthographic projection (2D games). `ortho_size` is the visible height in world units.
    pub orthographic: bool,
    /// Full vertical orthographic view height in metres.
    pub ortho_size: f32,
    /// Lens focal length in millimetres on a 24 mm tall sensor; above 0 it replaces `fov`
    /// (24 mm is a wide shot, 50 mm natural, 135 mm a telephoto).
    pub focal_length: f32,
    /// Depth of field: blur what is not at the focus distance.
    pub dof: bool,
    /// Aperture as an f-number: 1.4 gives strong blur, 8 a deep focus, 22 almost none.
    pub aperture: f32,
    /// Distance in metres that is in focus (set live by autofocus).
    pub focus_distance: f32,
    /// Focus on whatever is in the middle of the screen (needs colliders to hit).
    pub autofocus: bool,
    /// How fast autofocus racks to a new target.
    pub focus_speed: f32,
    /// Exposure compensation in stops (+1 doubles the light).
    pub exposure_comp: f32,
    /// Cinematic bars: target picture ratio such as 2.39 or 1.85 (0 = full screen).
    pub letterbox: f32,
    /// Camera shaders of this camera only, applied after the scene's own effects while it is the
    /// active camera (security cam static, a character's blurred view...).
    pub effects: Vec<crate::fx::PostEffect>,
}

impl Default for Camera {
    fn default() -> Self {
        Self { fov: 60.0, near: 0.1, far: 500.0, primary: true, orthographic: false, ortho_size: 10.0, focal_length: 0.0, dof: false, aperture: 2.8, focus_distance: 8.0, autofocus: true, focus_speed: 6.0, exposure_comp: 0.0, letterbox: 0.0, effects: Vec::new() }
    }
}

/// The camera the game looks through: the first `primary` camera, else the first camera at all.
pub fn active_camera(world: &World) -> Option<Entity> {
    world.iter::<Camera>().find(|(_, c)| c.primary).or_else(|| world.iter::<Camera>().next()).map(|(e, _)| e)
}

/// Makes `camera` the only primary camera (the one the game looks through).
pub fn set_active_camera(world: &mut World, camera: Entity) -> bool {
    if !world.has::<Camera>(camera) {
        return false;
    }
    let all: Vec<Entity> = world.iter::<Camera>().map(|(e, _)| e).collect();
    for e in all {
        if let Some(c) = world.get_mut::<Camera>(e) {
            c.primary = e == camera;
        }
    }
    true
}

impl Camera {
    /// Vertical field of view in degrees: from the focal length when one is set, else `fov`.
    pub fn vertical_fov(&self) -> f32 {
        if self.focal_length > 1.0 { (2.0 * (12.0 / self.focal_length).atan()).to_degrees().clamp(2.0, 170.0) } else { self.fov }
    }
}

impl Component for Camera {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::f32("fov", &mut self.fov),
            Field::f32("near", &mut self.near),
            Field::f32("far", &mut self.far),
            Field::bool("primary", &mut self.primary),
            Field::bool("orthographic", &mut self.orthographic),
            Field::f32("ortho_size", &mut self.ortho_size),
            Field::f32("focal_length", &mut self.focal_length),
            Field::bool("dof", &mut self.dof),
            Field::f32("aperture", &mut self.aperture),
            Field::f32("focus_distance", &mut self.focus_distance),
            Field::bool("autofocus", &mut self.autofocus),
            Field::f32("focus_speed", &mut self.focus_speed),
            Field::f32("exposure_comp", &mut self.exposure_comp),
            Field::f32("letterbox", &mut self.letterbox),
        ]
    }
}

choice_enum!(
    /// Directional lights illuminate globally; point/spot lights have a finite range.
    LightKind { Directional, Point, Spot } default Directional);

/// Directional, point or spot light with optional directional shadow casting.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Light {
    /// Directional sun, spherical point source or spot cone.
    pub kind: LightKind,
    /// RGBA light tint; RGB channels affect illumination.
    pub color: [f32; 4],
    /// Linear light strength multiplier.
    pub intensity: f32,
    /// Local light influence radius in metres (point and spot).
    pub range: f32,
    /// Spot only: outer cone angle in degrees.
    pub angle: f32,
    /// Spot only: 0 = hard cone edge, 1 = fully soft.
    pub softness: f32,
    /// Directional only: cast shadows.
    pub shadows: bool,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            kind: LightKind::Directional,
            color: [1.0, 0.96, 0.9, 1.0],
            intensity: 1.0,
            range: 10.0,
            angle: 35.0,
            softness: 0.25,
            shadows: true,
        }
    }
}

impl Component for Light {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::choice("kind", &mut self.kind),
            Field::color("color", &mut self.color),
            Field::f32("intensity", &mut self.intensity),
            Field::f32("range", &mut self.range),
            Field::f32("angle", &mut self.angle),
            Field::f32("softness", &mut self.softness),
            Field::bool("shadows", &mut self.shadows),
        ]
    }
}

choice_enum!(
    /// Dynamic bodies respond to forces, Fixed bodies stay still, Kinematic bodies follow transforms.
    BodyKind { Dynamic, Fixed, Kinematic } default Dynamic);

/// Physics body configuration and per-step motion requests.
/// Dynamic results are written back into Transform and velocity by Physics::step.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RigidBody {
    /// Dynamic, fixed or position-driven kinematic body.
    pub kind: BodyKind,
    /// Body mass in kilograms; physics clamps it to at least 0.001.
    pub mass: f32,
    /// Multiplier for scene gravity; zero disables gravity for this body.
    pub gravity_scale: f32,
    /// Linear velocity damping coefficient.
    pub linear_damping: f32,
    /// Angular velocity damping coefficient.
    pub angular_damping: f32,
    /// Locks all rotational axes, useful for upright player capsules.
    pub lock_rotation: bool,
    /// Continuous collision detection for fast dynamic bodies (helps prevent tunnelling).
    pub ccd: bool,
    /// Current linear velocity (read-only, mirrored from the simulation).
    #[serde(skip)]
    pub velocity: Vec3,
    /// Scripts write these; the physics step consumes them.
    #[serde(skip)]
    pub impulse: Vec3,
    #[serde(skip)]
    /// One-shot world-space linear velocity request, consumed on the next dynamic step.
    pub set_velocity: Option<Vec3>,
    #[serde(skip)]
    /// One-shot world-space angular velocity request in radians/second.
    pub set_angular_velocity: Option<Vec3>,
    /// One-shot request to move the simulated body to the current Transform pose.
    /// Use after editing a dynamic body's transform; ordinary physics normally owns its pose.
    #[serde(skip)]
    pub teleport: bool,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            kind: BodyKind::Dynamic,
            mass: 1.0,
            gravity_scale: 1.0,
            linear_damping: 0.05,
            angular_damping: 0.05,
            lock_rotation: false,
            ccd: false,
            velocity: Vec3::ZERO,
            impulse: Vec3::ZERO,
            set_velocity: None,
            set_angular_velocity: None,
            teleport: false,
        }
    }
}

impl Component for RigidBody {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::choice("kind", &mut self.kind),
            Field::f32("mass", &mut self.mass),
            Field::f32("gravity_scale", &mut self.gravity_scale),
            Field::f32("linear_damping", &mut self.linear_damping),
            Field::f32("angular_damping", &mut self.angular_damping),
            Field::bool("lock_rotation", &mut self.lock_rotation),
            Field::bool("ccd", &mut self.ccd),
        ]
    }
}

choice_enum!(
    /// Collider shapes. Primitives use full extents; `Mesh` is the exact triangles of the entity's
    /// model (for static level geometry), `Hull` its convex hull (for moving props).
    ShapeKind { Box, Sphere, Capsule, Cylinder, Mesh, Hull } default Box);

/// Collision shape. `size` is the full extent: box = (w, h, d); sphere = diameter in x;
/// capsule/cylinder = diameter in x and total height in y; mesh/hull = a multiplier on the model
/// (`MeshRenderer.mesh`, a box of `size` if it has none). Scaled by the entity's world scale.
/// Without a RigidBody on the entity or an ancestor it is static geometry.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Collider {
    /// Primitive collision shape family.
    pub shape: ShapeKind,
    /// Full local extents/diameters in metres, multiplied by world scale.
    pub size: Vec3,
    /// Surface friction coefficient; typical values lie in 0..1.
    pub friction: f32,
    /// Bounciness; zero is inelastic, one is ideally elastic.
    pub restitution: f32,
    /// Sensor-only volume: reports overlap without applying collision forces.
    pub is_trigger: bool,
    /// Collision-layer membership bits. Defaults to every layer (`u32::MAX`).
    pub memberships: u32,
    /// Layers this collider accepts. Interaction requires both colliders' masks to agree.
    pub filter: u32,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: ShapeKind::Box,
            size: Vec3::ONE,
            friction: 0.6,
            restitution: 0.1,
            is_trigger: false,
            memberships: u32::MAX,
            filter: u32::MAX,
        }
    }
}

impl Component for Collider {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::choice("shape", &mut self.shape),
            Field::vec3("size", &mut self.size),
            Field::f32("friction", &mut self.friction),
            Field::f32("restitution", &mut self.restitution),
            Field::bool("is_trigger", &mut self.is_trigger),
            Field::u32("memberships", &mut self.memberships),
            Field::u32("filter", &mut self.filter),
        ]
    }
}

choice_enum!(
    /// Fixed weld, freely rotating ball, limited hinge, or damped spring constraint.
    JointKind { Fixed, Ball, Hinge, Spring } default Fixed);

/// Connects this entity's RigidBody to another body (by entity name) or to the world if `target` is empty.
/// The joint is attached where the bodies currently are, so place objects first.
/// Fixed: weld at the current pose. Ball/Hinge: pivot at `anchor` (local to this entity).
/// Spring: `anchor` is where the other end is attached, relative to this entity.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Joint {
    /// Constraint family connecting this body to a named target or world anchor.
    pub kind: JointKind,
    /// Target body entity name; empty connects to a fixed world anchor.
    pub target: String,
    /// Pivot (Ball/Hinge) or far end (Spring), in this entity's local space.
    pub anchor: Vec3,
    /// Hinge axis in local space.
    pub axis: Vec3,
    /// Whether hinge angle bounds are applied.
    pub use_limits: bool,
    /// Minimum hinge angle in degrees.
    pub limit_min: f32,
    /// Maximum hinge angle in degrees.
    pub limit_max: f32,
    /// Spring equilibrium length in metres.
    pub rest_length: f32,
    /// Spring stiffness coefficient.
    pub stiffness: f32,
    /// Spring damping coefficient.
    pub damping: f32,
}

impl Default for Joint {
    fn default() -> Self {
        Self {
            kind: JointKind::Fixed,
            target: String::new(),
            anchor: Vec3::ZERO,
            axis: Vec3::Y,
            use_limits: false,
            limit_min: -45.0,
            limit_max: 45.0,
            rest_length: 1.0,
            stiffness: 50.0,
            damping: 5.0,
        }
    }
}

impl Component for Joint {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::choice("kind", &mut self.kind),
            Field::entity("target", &mut self.target),
            Field::vec3("anchor", &mut self.anchor),
            Field::vec3("axis", &mut self.axis),
            Field::bool("use_limits", &mut self.use_limits),
            Field::f32("limit_min", &mut self.limit_min),
            Field::f32("limit_max", &mut self.limit_max),
            Field::f32("rest_length", &mut self.rest_length),
            Field::f32("stiffness", &mut self.stiffness),
            Field::f32("damping", &mut self.damping),
        ]
    }
}

/// One attached native script type and its saved editable properties.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ScriptSlot {
    /// Exact exported Rust script type name.
    pub name: String,
    /// Saved field values keyed by the names returned from Script::fields.
    #[serde(default)]
    pub props: serde_json::Map<String, serde_json::Value>,
}

/// Rust scripts attached to an entity (several allowed). Property values live in `props`.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Scripts(pub Vec<ScriptSlot>);

impl Component for Scripts {}

macro_rules! name_components {
    ($($t:ident),+ $(,)?) => {
        $(impl kvas_ecs::Component for $t { const NAME: &'static str = stringify!($t); })+

        /// Creates the storage of every built-in component. Must run on every new `World`.
        pub fn register_all(world: &mut World) {
            $(world.register::<$t>();)+
        }
    };
}

pub use crate::audio::AudioSource;
pub use crate::gui::{UiElement, UiKind};
pub use crate::mesh_edit::EditableMesh;
pub use crate::neural::NeuralAgent;
pub use crate::animation::TransformAnimation;
pub use crate::cloth::Cloth;
pub use crate::gameplay::{Lifetime, Tags, Visibility};
pub use crate::grass::Grass;
pub use crate::tree::{Tree, TreePart};
pub use crate::decal::Decal;
pub use crate::interaction::{Slider, Switch};
pub use crate::meshmod::{MeshModifiers, ModifierStateCell};
pub use crate::paint::TexturePaint;
pub use crate::particles::ParticleEmitter;
pub use crate::skeletal::{Animator, AnimatorState};
pub use crate::modifiers::{Bob, ColorCycle, Pulse, Spin, Wobble};
pub use crate::tween::MoveTween;
pub use crate::voxel::{VoxelClouds, VoxelSmoke};
pub use crate::water::VoxelWater;

name_components!(
    UiElement,
    EditableMesh,
    NeuralAgent,
    TransformAnimation,
    ParticleEmitter,
    Animator,
    AnimatorState,
    TexturePaint,
    MeshModifiers,
    ModifierStateCell,
    AudioSource,
    MoveTween,
    Tags,
    Lifetime,
    Visibility,
    Switch,
    Slider,
    Transient,
    GrassChunk,
    Tree,
    TreePart,
    Decal,
    DynamicMesh,
    Cloth,
    Grass,
    VoxelClouds,
    VoxelSmoke,
    VoxelWater,
    Bob,
    Spin,
    Pulse,
    Wobble,
    ColorCycle,
    Parent,
    GlobalTransform,
    Name,
    Transform,
    MeshRenderer,
    Shape,
    Camera,
    Light,
    RigidBody,
    Collider,
    Joint,
    Scripts
);
