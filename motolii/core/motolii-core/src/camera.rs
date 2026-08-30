
use crate::CompSpec;

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

#[cfg(test)]
mod tests {
    use super::*;

    const COMP: CompSpec = CompSpec {
        width: 640,
        height: 360,
    };

    fn approx(a: glam::Vec2, b: glam::Vec2, eps: f32) {
        assert!(
            (a.x - b.x).abs() < eps && (a.y - b.y).abs() < eps,
            "{a:?} != {b:?} (eps={eps})"
        );
    }

    #[test]
    fn default_camera_matches_identity_pixel_mapping_at_z0() {
        let affine = camera_screen_from_world_z0(COMP, ResolvedCamera::default());
        for corner in [
            glam::vec2(0.0, 0.0),
            glam::vec2(COMP.width as f32, 0.0),
            glam::vec2(0.0, COMP.height as f32),
            glam::vec2(COMP.width as f32, COMP.height as f32),
            glam::vec2(320.0, 180.0),
        ] {
            approx(affine.transform_point2(corner), corner, 1e-2);
        }
    }

    #[test]
    fn zoom_scales_the_z0_plane_isotropically_around_comp_center() {
        let camera = ResolvedCamera {
            zoom: 2.0,
            ..Default::default()
        };
        let affine = camera_screen_from_world_z0(COMP, camera);
        let center = glam::vec2(COMP.width as f32 * 0.5, COMP.height as f32 * 0.5);

        approx(affine.transform_point2(center), center, 1e-2);

        let probe = center + glam::vec2(50.0, 30.0);
        let mapped = affine.transform_point2(probe);
        let expected = center + (probe - center) * 2.0;
        approx(mapped, expected, 1e-1);
    }

    #[test]
    fn roll_rotates_the_z0_plane_clockwise_around_comp_center() {
        let camera = ResolvedCamera {
            roll_degrees: 90.0,
            ..Default::default()
        };
        let affine = camera_screen_from_world_z0(COMP, camera);
        let center = glam::vec2(COMP.width as f32 * 0.5, COMP.height as f32 * 0.5);

        let right = center + glam::vec2(40.0, 0.0);
        let mapped = affine.transform_point2(right);
        let expected_down = center + glam::vec2(0.0, 40.0);
        approx(mapped, expected_down, 1e-1);
    }

    #[test]
    fn panning_the_camera_shifts_the_z0_plane_the_opposite_way() {
        let camera = ResolvedCamera {
            center: [50.0, 0.0],
            ..Default::default()
        };
        let affine = camera_screen_from_world_z0(COMP, camera);
        let comp_center = glam::vec2(COMP.width as f32 * 0.5, COMP.height as f32 * 0.5);
        let mapped = affine.transform_point2(comp_center);
        assert!(
            mapped.x < comp_center.x,
            "center を +X へ動かしたら絵は -X 側へ寄るはず: {mapped:?}"
        );
    }

    #[test]
    fn nearer_planes_move_more_than_farther_planes_under_the_same_pan() {
        let before = ResolvedCamera::default();
        let after = ResolvedCamera {
            center: [30.0, 0.0],
            ..Default::default()
        };

        let near_z = -200.0; // カメラに近い(手前)
        let far_z = 400.0; // カメラから遠い(奥)

        let probe = glam::vec2(COMP.width as f32 * 0.5, COMP.height as f32 * 0.5);

        let near_before = camera_screen_from_world_at_z(COMP, before, near_z).transform_point2(probe);
        let near_after = camera_screen_from_world_at_z(COMP, after, near_z).transform_point2(probe);
        let far_before = camera_screen_from_world_at_z(COMP, before, far_z).transform_point2(probe);
        let far_after = camera_screen_from_world_at_z(COMP, after, far_z).transform_point2(probe);

        let near_delta = (near_after - near_before).length();
        let far_delta = (far_after - far_before).length();

        assert!(
            near_delta > far_delta,
            "近い板({near_delta})が遠い板({far_delta})より大きく動くはず"
        );
    }

    #[test]
    fn distance_from_camera_is_stable_across_zoom() {
        let d = distance_from_camera(COMP, 0.0);
        assert!(d > 0.0);
        assert!(distance_from_camera(COMP, 100.0) > d);
        assert!(distance_from_camera(COMP, -100.0) < d);
    }

    #[test]
    fn non_finite_roll_degrees_does_not_panic_and_falls_back_to_no_roll() {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let camera = ResolvedCamera {
                roll_degrees: bad,
                ..Default::default()
            };
            let projection = camera_projection(COMP, camera);
            let view = projection.view_matrix(); // ここが修正前は panic していた
            assert!(
                view.is_finite(),
                "roll_degrees={bad} で view_matrix() が非有限行列を返した"
            );

            let affine = camera_screen_from_world_z0(COMP, camera);
            let default_affine = camera_screen_from_world_z0(COMP, ResolvedCamera::default());
            let probe = glam::vec2(320.0, 180.0);
            approx(
                affine.transform_point2(probe),
                default_affine.transform_point2(probe),
                1e-2,
            );
        }
    }
}
