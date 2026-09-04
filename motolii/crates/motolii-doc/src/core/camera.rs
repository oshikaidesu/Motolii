
use crate::doc::core::CompSpec;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedCamera {
    pub center: [f32; 2],
    pub zoom: f32,
    pub roll_degrees: f32,
}

impl Default for ResolvedCamera {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0],
            zoom: 1.0,
            roll_degrees: 0.0,
        }
    }
}

pub const CAMERA_BASE_VERTICAL_FOV_DEGREES: f32 = 55.0;

pub const NEAR_PLANE: f32 = 0.01;

#[derive(Clone, Copy, Debug)]
pub struct CameraProjection {
    pub eye: glam::Vec3,
    pub rotation: glam::Quat,
    pub vertical_fov_radians: f32,
    pub aspect_ratio: f32,
    pub near_plane_distance: f32,
}

fn base_distance(comp: CompSpec) -> f32 {
    let half_fov = (CAMERA_BASE_VERTICAL_FOV_DEGREES * 0.5).to_radians();
    (comp.height as f32 * 0.5) / half_fov.tan()
}

pub fn distance_from_camera(comp: CompSpec, z: f32) -> f32 {
    base_distance(comp) + z
}

fn safe_axis_angle(axis: glam::Vec3, radians: f32) -> glam::Quat {
    let radians = if radians.is_finite() { radians } else { 0.0 };
    glam::Quat::from_axis_angle(axis, radians)
}

pub fn camera_projection(comp: CompSpec, camera: ResolvedCamera) -> CameraProjection {
    let distance = base_distance(comp);
    let zoom = camera.zoom.max(1e-3);
    let half_base_fov = (CAMERA_BASE_VERTICAL_FOV_DEGREES * 0.5).to_radians();
    let vertical_fov_radians = 2.0 * (half_base_fov.tan() / zoom).atan();

    let eye = glam::vec3(
        comp.width as f32 * 0.5 + camera.center[0],
        comp.height as f32 * 0.5 + camera.center[1],
        -distance,
    );

    let base = glam::Quat::from_axis_angle(glam::Vec3::X, std::f32::consts::PI);
    let roll = safe_axis_angle(glam::Vec3::Z, -camera.roll_degrees.to_radians());
    let rotation = roll * base;

    CameraProjection {
        eye,
        rotation,
        vertical_fov_radians,
        aspect_ratio: comp.width as f32 / comp.height as f32,
        near_plane_distance: NEAR_PLANE,
    }
}

impl CameraProjection {
    pub fn view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_quat(self.rotation) * glam::Mat4::from_translation(-self.eye)
    }

    pub fn projection_matrix(&self) -> glam::Mat4 {
        glam::Mat4::perspective_infinite_reverse_rh(
            self.vertical_fov_radians,
            self.aspect_ratio,
            self.near_plane_distance,
        )
    }
}

pub fn camera_screen_from_world_at_z(
    comp: CompSpec,
    camera: ResolvedCamera,
    z: f32,
) -> glam::Affine2 {
    let projection = camera_projection(comp, camera);
    let clip_from_world = projection.projection_matrix() * projection.view_matrix();

    let to_pixel = |world_xy: glam::Vec2| -> glam::Vec2 {
        let clip = clip_from_world * glam::Vec4::new(world_xy.x, world_xy.y, z, 1.0);
        let ndc = glam::vec2(clip.x / clip.w, clip.y / clip.w);
        glam::vec2(
            (ndc.x + 1.0) * 0.5 * comp.width as f32,
            (1.0 - ndc.y) * 0.5 * comp.height as f32,
        )
    };

    let origin = to_pixel(glam::Vec2::ZERO);
    let x_axis = to_pixel(glam::Vec2::X) - origin;
    let y_axis = to_pixel(glam::Vec2::Y) - origin;
    glam::Affine2::from_cols(x_axis, y_axis, origin)
}

pub fn camera_screen_from_world_z0(comp: CompSpec, camera: ResolvedCamera) -> glam::Affine2 {
    camera_screen_from_world_at_z(comp, camera, 0.0)
}

/// Authored world geometry enters the same Rerun view for every projection state.
/// glam's rotation-arc contract maps +Z onto the center ray; rotating about the
/// center keeps 2.5D rigid. 2D cancels the camera in view space, including focal
/// scaling, while retaining each vertex's depth relative to the layer center.
pub fn layer_projection_transform(
    comp: CompSpec,
    camera: ResolvedCamera,
    mode: crate::doc::store::LayerProjection,
    center: glam::Vec3,
) -> glam::Affine3A {
    use crate::doc::store::LayerProjection;
    use glam::{Affine3A, Vec3};
    let projection = camera_projection(comp, camera);
    match mode {
        LayerProjection::ThreeD => Affine3A::IDENTITY,
        LayerProjection::TwoPointFiveD => {
            let Some(ray) = (center - projection.eye).try_normalize() else { return Affine3A::IDENTITY };
            let rotation = glam::Quat::from_rotation_arc(Vec3::Z, ray);
            Affine3A::from_rotation_translation(rotation, center - rotation * center)
        }
        LayerProjection::TwoD => {
            let baseline = camera_projection(comp, ResolvedCamera::default());
            let focal = (projection.vertical_fov_radians * 0.5).tan()
                / (baseline.vertical_fov_radians * 0.5).tan();
            Affine3A::from_mat4(projection.view_matrix()).inverse()
                * Affine3A::from_scale(glam::vec3(focal, focal, 1.0))
                * Affine3A::from_mat4(baseline.view_matrix())
                * Affine3A::from_translation(-Vec3::Z * center.z)
        }
    }
}

#[cfg(test)]
mod projection_tests {
    use super::*;
    use crate::doc::store::LayerProjection;

    fn pixel(comp: CompSpec, camera: ResolvedCamera, p: glam::Vec3) -> glam::Vec2 {
        let c = camera_projection(comp, camera);
        let q = c.projection_matrix() * c.view_matrix() * p.extend(1.0);
        glam::vec2((q.x / q.w + 1.0) * comp.width as f32 / 2.0, (1.0 - q.y / q.w) * comp.height as f32 / 2.0)
    }

    #[test]
    fn two_d_cancels_camera_for_nonplanar_vertices_and_preserves_legacy_z0() {
        let comp = CompSpec { width: 1920, height: 1080 };
        let camera = ResolvedCamera { center: [210.0, -130.0], zoom: 2.7, roll_degrees: 47.0 };
        let center = glam::vec3(430.0, 760.0, 300.0);
        let transform = layer_projection_transform(comp, camera, LayerProjection::TwoD, center);
        for offset in [glam::vec3(-90.0, -40.0, -60.0), glam::vec3(60.0, 30.0, 80.0), glam::Vec3::ZERO] {
            let p = center + offset;
            let actual = pixel(comp, camera, transform.transform_point3(p));
            let expected = pixel(comp, ResolvedCamera::default(), p - glam::Vec3::Z * center.z);
            assert!(actual.distance(expected) < 0.003, "{actual:?} != {expected:?}");
        }
        let legacy = camera_screen_from_world_z0(comp, camera).inverse();
        let plane = layer_projection_transform(comp, camera, LayerProjection::TwoD, glam::Vec3::ZERO);
        let p = glam::vec2(200.0, 100.0);
        assert!(plane.transform_point3(p.extend(0.0)).truncate().distance(legacy.transform_point2(p)) < 0.02);
    }

    #[test]
    fn two_point_five_d_preserves_pose_relative_to_ray_and_rigid_volume() {
        let comp = CompSpec { width: 1920, height: 1080 };
        let camera = ResolvedCamera { center: [90.0, -30.0], zoom: 1.8, roll_degrees: 33.0 };
        let eye = camera_projection(comp, camera).eye;
        let normal = glam::Quat::from_rotation_x(0.6) * glam::Vec3::Z;
        let tetrahedron = [glam::Vec3::ZERO, glam::Vec3::X * 80.0, glam::Vec3::Y * 60.0, glam::Vec3::Z * 40.0];
        for center in [glam::vec3(250.0, 200.0, 0.0), glam::vec3(1700.0, 900.0, 300.0)] {
            let transform = layer_projection_transform(comp, camera, LayerProjection::TwoPointFiveD, center);
            assert!(transform.transform_point3(center).distance(center) < 0.001);
            let ray = (center - eye).normalize();
            assert!((transform.transform_vector3(normal).dot(ray) - normal.z).abs() < 0.0001);
            for a in tetrahedron { for b in tetrahedron {
                assert!((transform.transform_point3(center + a).distance(transform.transform_point3(center + b)) - a.distance(b)).abs() < 0.002);
            } }
            assert!((transform.matrix3.determinant() - 1.0).abs() < 0.0001);
        }
    }

    #[test]
    fn widened_user_view_keeps_two_d_on_the_same_output_pixels() {
        let comp = CompSpec { width: 1920, height: 1080 };
        let camera = ResolvedCamera { center: [150.0, 90.0], zoom: 1.6, roll_degrees: 24.0 };
        let wide = ResolvedCamera { zoom: camera.zoom / 3.0, ..camera };
        let center = glam::vec3(100.0, 850.0, 200.0);
        let transformed = layer_projection_transform(comp, camera, LayerProjection::TwoD, center).transform_point3(center);
        let mid = glam::vec2(960.0, 540.0);
        assert!(pixel(comp, camera, transformed).distance(mid + (pixel(comp, wide, transformed) - mid) * 3.0) < 0.003);
    }
}
