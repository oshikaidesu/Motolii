
use crate::doc::core::CompSpec;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedCamera {
    pub center: [f32; 2],
    pub zoom: f32,
    pub roll_degrees: f32,
    pub orbit_degrees: [f32; 2],
    pub distance_scale: f32,
}

impl Default for ResolvedCamera {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0],
            zoom: 1.0,
            roll_degrees: 0.0,
            orbit_degrees: [0.0; 2],
            distance_scale: 1.0,
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

    let target = glam::vec3(
        comp.width as f32 * 0.5 + camera.center[0],
        comp.height as f32 * 0.5 + camera.center[1],
        0.0,
    );
    let orbit = glam::Quat::from_rotation_y(camera.orbit_degrees[1].to_radians()) * glam::Quat::from_rotation_x(camera.orbit_degrees[0].to_radians());
    let eye = target + orbit * glam::vec3(0.0, 0.0, -distance * camera.distance_scale.max(0.01));

    let base = glam::Quat::from_xyzw(1.0, 0.0, 0.0, 0.0);
    let roll = safe_axis_angle(glam::Vec3::Z, -camera.roll_degrees.to_radians());
    let rotation = roll * base * orbit.inverse();

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

    let origin = clip_from_world * glam::Vec4::new(0.0, 0.0, z, 1.0);
    let pixel_scale = glam::vec2(comp.width as f32 * 0.5, -(comp.height as f32) * 0.5);
    let xy = |column: glam::Vec4| glam::vec2(column.x, column.y) / origin.w * pixel_scale;
    glam::Affine2::from_cols(
        xy(clip_from_world.x_axis),
        xy(clip_from_world.y_axis),
        xy(origin) + glam::vec2(comp.width as f32 * 0.5, comp.height as f32 * 0.5),
    )
}

pub fn camera_screen_from_world_z0(comp: CompSpec, camera: ResolvedCamera) -> glam::Affine2 {
    camera_screen_from_world_at_z(comp, camera, 0.0)
}

/// 2D uses a frame-relative transform. Spatial layers retain their authored
/// world geometry; projection does not auto-orient or flatten that geometry.
pub fn layer_projection_transform(
    comp: CompSpec,
    camera: ResolvedCamera,
    mode: crate::doc::store::LayerProjection,
    center: glam::Vec3,
) -> glam::Affine3A {
    use crate::doc::store::LayerProjection;
    use glam::{Affine3A, Vec3};
    match mode {
        LayerProjection::ThreeD | LayerProjection::TwoPointFiveD => Affine3A::IDENTITY,
        LayerProjection::TwoD => {
            let projection = camera_projection(comp, camera);
            let baseline = camera_projection(comp, ResolvedCamera::default());
            let focal = (projection.vertical_fov_radians * 0.5).tan()
                / (baseline.vertical_fov_radians * 0.5).tan();
            let frame_center = glam::vec2(comp.width as f32 * 0.5, comp.height as f32 * 0.5);
            let displacement = (center.truncate() - frame_center) / base_distance(comp);
            let frame_translation = Affine3A::from_mat3(glam::Mat3::from_cols(
                Vec3::X, Vec3::Y, glam::vec3(displacement.x, displacement.y, 1.0),
            ));
            Affine3A::from_mat4(projection.view_matrix()).inverse()
                * Affine3A::from_scale(glam::vec3(focal, focal, 1.0))
                * Affine3A::from_mat4(baseline.view_matrix())
                * frame_translation
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
    fn planar_affine_matches_direct_perspective_projection_across_the_image() {
        let comp = CompSpec { width: 1920, height: 1080 };
        for camera in [
            ResolvedCamera::default(),
            ResolvedCamera { center: [210.0, -130.0], zoom: 2.7, roll_degrees: 47.0, ..Default::default() },
            ResolvedCamera { center: [-900.0, 600.0], zoom: 0.4, roll_degrees: -120.0, ..Default::default() },
        ] {
            for z in [-200.0, 0.0, 350.0] {
                let affine = camera_screen_from_world_at_z(comp, camera, z);
                let projection = camera_projection(comp, camera);
                let clip = projection.projection_matrix() * projection.view_matrix();
                assert_eq!(clip.x_axis.w, 0.0);
                assert_eq!(clip.y_axis.w, 0.0);
                for point in [glam::vec2(0.0, 0.0), glam::vec2(200.0, 100.0), glam::vec2(1920.0, 1080.0), glam::vec2(-400.0, 1500.0)] {
                    let projected = pixel(comp, camera, point.extend(z));
                    assert!(affine.transform_point2(point).distance(projected) < 0.003);
                    assert!(affine.inverse().transform_point2(projected).distance(point) < 0.003);
                }
            }
        }
    }

    #[test]
    fn two_d_cancels_camera_for_nonplanar_vertices_and_preserves_legacy_z0() {
        let comp = CompSpec { width: 1920, height: 1080 };
        let camera = ResolvedCamera { center: [210.0, -130.0], zoom: 2.7, roll_degrees: 47.0, ..Default::default() };
        let center = glam::vec3(430.0, 760.0, 300.0);
        let transform = layer_projection_transform(comp, camera, LayerProjection::TwoD, center);
        for offset in [glam::vec3(-90.0, -40.0, -60.0), glam::vec3(60.0, 30.0, 80.0), glam::Vec3::ZERO] {
            let p = center + offset;
            let actual = pixel(comp, camera, transform.transform_point3(p));
            let frame_center = glam::vec2(comp.width as f32 * 0.5, comp.height as f32 * 0.5);
            let expected = center.truncate() + pixel(comp, ResolvedCamera::default(), frame_center.extend(0.0) + offset) - frame_center;
            assert!(actual.distance(expected) < 0.003, "{actual:?} != {expected:?}");
        }
        let legacy = camera_screen_from_world_z0(comp, camera).inverse();
        let plane = layer_projection_transform(comp, camera, LayerProjection::TwoD, glam::Vec3::ZERO);
        let p = glam::vec2(200.0, 100.0);
        assert!(plane.transform_point3(p.extend(0.0)).truncate().distance(legacy.transform_point2(p)) < 0.02);
    }

    #[test]
    fn two_d_position_changes_translate_the_picture_without_changing_its_shape() {
        let comp=CompSpec {width:1920,height:1080};
        let centers=[glam::vec3(180.0,300.0,0.0),glam::vec3(1710.0,760.0,400.0)];
        let rotation=glam::Quat::from_rotation_y(1.1)*glam::Quat::from_rotation_x(0.3);
        let vertices=[glam::vec3(-80.0,-120.0,0.0),glam::vec3(80.0,-120.0,0.0),glam::vec3(80.0,120.0,0.0),glam::vec3(-80.0,120.0,0.0),glam::vec3(20.0,30.0,50.0)];
        for camera in [ResolvedCamera::default(),ResolvedCamera {center:[210.0,-130.0],zoom:2.7,roll_degrees:47.0, ..Default::default() }] {
            let transforms=centers.map(|center|layer_projection_transform(comp,camera,LayerProjection::TwoD,center));
            assert!(transforms.iter().all(|t|t.matrix3.determinant().abs()>0.0001));
            for vertex in vertices {
                let local=rotation*vertex;
                let relative=std::array::from_fn::<_,2,_>(|i|pixel(comp,camera,transforms[i].transform_point3(centers[i]+local))-centers[i].truncate());
                assert!(relative[0].distance(relative[1])<0.003,"position changed the shape: {relative:?}");
            }
        }
    }

    #[test]
    fn two_point_five_d_keeps_authored_orientation_and_volume() {
        let comp = CompSpec { width: 1920, height: 1080 };
        let normal = glam::Quat::from_rotation_x(0.6) * glam::Vec3::Z;
        let vertices = [glam::Vec3::ZERO, glam::Vec3::X*80.0, glam::Vec3::Y*60.0, glam::Vec3::Z*40.0];
        for camera in [ResolvedCamera::default(), ResolvedCamera { center:[90.0,-30.0], zoom:1.8, roll_degrees:33.0, ..Default::default() }] {
            for center in [glam::vec3(250.0,200.0,0.0),glam::vec3(1700.0,900.0,300.0)] {
                let transform=layer_projection_transform(comp,camera,LayerProjection::TwoPointFiveD,center);
                assert_eq!(transform.transform_vector3(normal),normal);
                for vertex in vertices {
                    assert_eq!(transform.transform_point3(center+vertex),center+vertex);
                }
            }
        }
    }

    #[test]
    fn two_point_five_d_camera_pan_produces_parallax_without_added_tilt() {
        let comp=CompSpec {width:1920,height:1080};
        let baseline=ResolvedCamera::default();
        let panned=ResolvedCamera {center:[100.0,0.0],..baseline};
        let projected=|camera,point:glam::Vec3| pixel(comp,camera,
            layer_projection_transform(comp,camera,LayerProjection::TwoPointFiveD,point).transform_point3(point));
        let near=glam::vec3(250.0,200.0,0.0);
        let far=glam::vec3(250.0,200.0,350.0);
        let near_shift=projected(panned,near)-projected(baseline,near);
        let far_shift=projected(panned,far)-projected(baseline,far);
        assert!(near_shift.x<0.0 && far_shift.x<0.0 && near_shift.x.abs()>far_shift.x.abs());
        for center in [near,far] {
            let edge=projected(panned,center+glam::Vec3::X*80.0)-projected(panned,center);
            assert!(edge.y.abs()<0.002);
            let rolled=ResolvedCamera {roll_degrees:30.0,..panned};
            let edge=projected(rolled,center+glam::Vec3::X*80.0)-projected(rolled,center);
            assert!(edge.y.abs()>1.0);
        }
    }

    #[test]
    fn widened_user_view_keeps_two_d_on_the_same_output_pixels() {
        let comp = CompSpec { width: 1920, height: 1080 };
        let camera = ResolvedCamera { center: [150.0, 90.0], zoom: 1.6, roll_degrees: 24.0, ..Default::default() };
        let wide = ResolvedCamera { zoom: camera.zoom / 3.0, ..camera };
        let center = glam::vec3(100.0, 850.0, 200.0);
        let transformed = layer_projection_transform(comp, camera, LayerProjection::TwoD, center).transform_point3(center);
        let mid = glam::vec2(960.0, 540.0);
        assert!(pixel(comp, camera, transformed).distance(mid + (pixel(comp, wide, transformed) - mid) * 3.0) < 0.003);
    }
}
