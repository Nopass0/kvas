//! Camera projection, viewport conversion and picking helpers.
use glam::{Mat4, Vec2, Vec3, Vec4};
use kvas_engine::components::Camera;
use kvas_engine::scene::Scene;
use kvas_engine::spatial::{Frustum, Ray};

/// Right-handed camera matrices using wgpu's 0..1 clip-depth range.
#[derive(Clone, Copy)]
pub struct View {
    /// World-to-camera matrix.
    pub view: Mat4,
    /// Camera-to-clip projection matrix.
    pub proj: Mat4,
    /// World-space camera position (used by lighting/shaders).
    pub eye: Vec3,
}

impl View {
    /// Perspective camera from a rigid camera-to-world transform (no scale).
    /// FOV is vertical degrees; near/far are metres. The caller must supply
    /// finite valid clipping/FOV values; use [`Self::try_perspective`] for checked input.
    pub fn new(camera_to_world: Mat4, fov_deg: f32, aspect: f32, near: f32, far: f32) -> View {
        View {
            view: camera_to_world.inverse(),
            proj: glam::camera::rh::proj::directx::perspective(fov_deg.to_radians(), aspect.max(0.01), near, far),
            eye: camera_to_world.w_axis.truncate(),
        }
    }

    /// Combined world-to-clip matrix, `projection * view`.
    pub fn view_proj(&self) -> Mat4 {
        self.proj * self.view
    }

    /// World-space ray through a point in normalized device coordinates (x right, y up, -1..1).
    pub fn ray(&self, ndc: Vec2) -> (Vec3, Vec3) {
        let inv = self.view_proj().inverse();
        let near = inv * Vec4::new(ndc.x, ndc.y, 0.0, 1.0);
        let far = inv * Vec4::new(ndc.x, ndc.y, 1.0, 1.0);
        let (near, far) = (near.truncate() / near.w, far.truncate() / far.w);
        (near, (far - near).normalize_or_zero())
    }

    /// Projects a world point to XY NDC, allowing offscreen coordinates.
    /// Perspective points behind the camera return `None`; use [`Self::project_visible`]
    /// to additionally enforce side and near/far clipping for either projection type.
    pub fn project(&self, p: Vec3) -> Option<Vec2> {
        let c = self.view_proj() * p.extend(1.0);
        (c.is_finite() && c.w > 1e-4).then(|| Vec2::new(c.x / c.w, c.y / c.w))
    }

    /// Validates perspective parameters before constructing a camera.
    /// Rejects non-finite/singular transforms, invalid FOV/aspect and clipping ranges.
    pub fn try_perspective(camera_to_world: Mat4, fov_deg: f32, aspect: f32, near: f32, far: f32) -> Option<Self> {
        if !camera_to_world.is_finite()
            || camera_to_world.determinant() == 0.0
            || !fov_deg.is_finite()
            || !(0.0..180.0).contains(&fov_deg)
            || fov_deg == 0.0
            || !aspect.is_finite()
            || aspect <= 0.0
            || !near.is_finite()
            || !far.is_finite()
            || near <= 0.0
            || far <= near
        {
            return None;
        }
        let view = Self::new(camera_to_world, fov_deg, aspect, near, far);
        view.view_proj().is_finite().then_some(view)
    }

    /// Projects only points inside all six clip planes to NDC (XYZ, depth 0..1).
    pub fn project_visible(&self, point: Vec3) -> Option<Vec3> {
        let clip = self.view_proj() * point.extend(1.0);
        if !clip.is_finite() || clip.w <= 0.0 {
            return None;
        }
        let ndc = clip.truncate() / clip.w;
        (ndc.x.abs() <= 1.0 && ndc.y.abs() <= 1.0 && (0.0..=1.0).contains(&ndc.z)).then_some(ndc)
    }

    /// Converts a visible world point to pixel coordinates with origin at viewport top-left.
    pub fn world_to_screen(&self, point: Vec3, viewport: Vec2) -> Option<Vec2> {
        if !viewport.is_finite() || viewport.min_element() <= 0.0 {
            return None;
        }
        let ndc = self.project_visible(point)?;
        Some(Vec2::new(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * viewport)
    }

    /// World-space picking ray through viewport-local pixels (top-left origin).
    /// Offscreen pixels are permitted. Invalid viewport/projection data returns `None`.
    /// The origin is on the near plane, supporting both orthographic and perspective views.
    pub fn screen_ray(&self, pixel: Vec2, viewport: Vec2) -> Option<Ray> {
        if !pixel.is_finite()
            || !viewport.is_finite()
            || viewport.min_element() <= 0.0
            || !self.view_proj().is_finite()
            || self.view_proj().determinant() == 0.0
        {
            return None;
        }
        let ndc = Vec2::new(pixel.x / viewport.x * 2.0 - 1.0, 1.0 - pixel.y / viewport.y * 2.0);
        let (origin, direction) = self.ray(ndc);
        Ray::new(origin, direction)
    }

    /// Extracts normalized clipping planes for CPU visibility queries.
    pub fn frustum(&self) -> Option<Frustum> {
        Frustum::from_view_projection(self.view_proj())
    }
}

impl View {
    /// Orthographic camera with a full vertical view `height` in metres.
    /// The camera transform must be rigid and clipping parameters finite with far > near.
    pub fn orthographic(camera_to_world: Mat4, height: f32, aspect: f32, near: f32, far: f32) -> View {
        let half_h = height * 0.5;
        let half_w = half_h * aspect.max(0.01);
        View {
            view: camera_to_world.inverse(),
            proj: glam::camera::rh::proj::directx::orthographic(-half_w, half_w, -half_h, half_h, near, far),
            eye: camera_to_world.w_axis.truncate(),
        }
    }

    /// Validates an orthographic camera before constructing it.
    /// Height/aspect must be positive and finite; near/far must be finite with far > near.
    /// Negative near planes are allowed for editor-style parallel cameras.
    pub fn try_orthographic(camera_to_world: Mat4, height: f32, aspect: f32, near: f32, far: f32) -> Option<Self> {
        if !camera_to_world.is_finite()
            || camera_to_world.determinant() == 0.0
            || !height.is_finite()
            || height <= 0.0
            || !aspect.is_finite()
            || aspect <= 0.0
            || !near.is_finite()
            || !far.is_finite()
            || far <= near
        {
            return None;
        }
        let view = Self::orthographic(camera_to_world, height, aspect, near, far);
        view.view_proj().is_finite().then_some(view)
    }
}

/// View from the first valid primary camera, then any valid non-primary camera.
/// Invalid/singular configurations are skipped. World poses are evaluated directly,
/// so newly spawned cameras do not require a cached GlobalTransform first.
pub fn game_view(scene: &Scene, aspect: f32) -> Option<View> {
    let world = &scene.world;
    world
        .iter::<Camera>()
        .filter(|(_, camera)| camera.primary)
        .chain(world.iter::<Camera>().filter(|(_, camera)| !camera.primary))
        .find_map(|(entity, camera)| {
            if !world.has::<kvas_engine::components::Transform>(entity) {
                return None;
            }
            let matrix = kvas_engine::hierarchy::global_matrix(world, entity);
            if !matrix.is_finite() || matrix.determinant() == 0.0 {
                return None;
            }
            let (_, rotation, position) = matrix.to_scale_rotation_translation();
            if !rotation.is_finite() || rotation.length_squared() == 0.0 {
                return None;
            }
            let rigid = Mat4::from_rotation_translation(rotation.normalize(), position);
            if camera.orthographic {
                View::try_orthographic(rigid, camera.ortho_size, aspect, camera.near, camera.far)
            } else {
                View::try_perspective(rigid, camera.vertical_fov(), aspect, camera.near, camera.far)
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn picking_roundtrips_perspective_and_orthographic_pixels() {
        for view in [View::new(Mat4::IDENTITY, 60.0, 2.0, 0.1, 100.0), View::orthographic(Mat4::IDENTITY, 4.0, 2.0, 0.1, 100.0)] {
            let viewport = Vec2::new(800.0, 400.0);
            let point = Vec3::new(0.2, 0.1, -5.0);
            let pixel = view.world_to_screen(point, viewport).unwrap();
            let ray = view.screen_ray(pixel, viewport).unwrap();
            let distance = ray.plane_distance(point, Vec3::Z).unwrap();
            assert!((ray.point_at(distance) - point).length() < 1e-4);
            assert!(view.project_visible(Vec3::new(0.0, 0.0, 5.0)).is_none());
            assert!(view.world_to_screen(point, Vec2::ZERO).is_none());
            assert!(view.frustum().unwrap().contains_point(point));
        }
    }
    #[test]
    fn checked_projection_rejects_invalid_configuration() {
        assert!(View::try_perspective(Mat4::ZERO, 60.0, 1.0, 0.1, 10.0).is_none());
        for fov in [0.0, 180.0, -1.0, f32::NAN] {
            assert!(View::try_perspective(Mat4::IDENTITY, fov, 1.0, 0.1, 10.0).is_none());
        }
        let view = View::try_perspective(Mat4::IDENTITY, 60.0, 1.0, 0.1, 10.0).unwrap();
        assert!(view.project(Vec3::splat(f32::NAN)).is_none());
        let big = View::orthographic(Mat4::IDENTITY, 1000.0, 1.0, 0.1, 1_000_000.0);
        assert!(big.screen_ray(Vec2::splat(50.0), Vec2::splat(100.0)).is_some());
    }
}
