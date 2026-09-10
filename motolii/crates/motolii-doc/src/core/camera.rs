
use crate::doc::core::CompSpec;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedCamera {
    pub center: [f32; 2],
    pub zoom: f32,
    pub roll_degrees: f32,
    pub orbit_degrees: [f32; 2],
    pub distance_scale: f32,
    pub target_z: f32,
}

impl Default for ResolvedCamera {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0],
            zoom: 1.0,
            roll_degrees: 0.0,
            orbit_degrees: [0.0; 2],
            distance_scale: 1.0,
            target_z: 0.0,
        }
    }
}

impl ResolvedCamera {
    pub fn target(&self, comp: CompSpec) -> glam::Vec3 {
        glam::vec3(
            comp.width as f32 * 0.5 + self.center[0],
            comp.height as f32 * 0.5 + self.center[1],
            self.target_z,
        )
    }

    /// rerun `focus_entity`: look_target = the object's centre, eye = target − fwd × 1.5 × bounding-sphere radius.
    pub fn looking_at(self, comp: CompSpec, target: glam::Vec3, radius: f32) -> Self {
        let distance = (radius * 1.5).max(base_distance(comp) * 0.01);
        Self {
            center: [target.x - comp.width as f32 * 0.5, target.y - comp.height as f32 * 0.5],
            target_z: target.z,
            distance_scale: distance / base_distance(comp),
            ..self
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

    let target = camera.target(comp);
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
    match mode {
        LayerProjection::ThreeD => glam::Affine3A::IDENTITY,
        LayerProjection::TwoPointFiveD => {
            let rotation = camera_projection(comp, camera).rotation.inverse()
                * camera_projection(comp, ResolvedCamera::default()).rotation;
            glam::Affine3A::from_translation(center)
                * glam::Affine3A::from_quat(rotation)
                * glam::Affine3A::from_translation(-center)
        }
        LayerProjection::TwoD => two_d_frame(comp, camera) * two_d_center(comp, center),
    }
}

/// Camera-independent part of 2D: the default camera's view scaled to the current zoom.
fn two_d_frame(comp: CompSpec, camera: ResolvedCamera) -> glam::Affine3A {
    use glam::Affine3A;
    let projection = camera_projection(comp, camera);
    let baseline = camera_projection(comp, ResolvedCamera::default());
    let focal = (projection.vertical_fov_radians * 0.5).tan()
        / (baseline.vertical_fov_radians * 0.5).tan();
    Affine3A::from_mat4(projection.view_matrix()).inverse()
        * Affine3A::from_scale(glam::vec3(focal, focal, 1.0))
        * Affine3A::from_mat4(baseline.view_matrix())
}

/// Center-dependent part of 2D: drop the layer onto z=0 and shear its depth so the
/// frame center's vanishing point does not tilt an off-center layer.
fn two_d_center(comp: CompSpec, center: glam::Vec3) -> glam::Affine3A {
    use glam::{Affine3A, Vec3};
    let frame_center = glam::vec2(comp.width as f32 * 0.5, comp.height as f32 * 0.5);
    let displacement = (center.truncate() - frame_center) / base_distance(comp);
    Affine3A::from_mat3(glam::Mat3::from_cols(
        Vec3::X, Vec3::Y, glam::vec3(displacement.x, displacement.y, 1.0),
    )) * Affine3A::from_translation(-Vec3::Z * center.z)
}

/// World transform that draws the same picture under `to` that `world` draws under `from`.
/// `None` when the layer's center cannot be placed in front of the camera.
pub fn projection_switch_world(
    comp: CompSpec,
    camera: ResolvedCamera,
    from: crate::doc::store::LayerProjection,
    to: crate::doc::store::LayerProjection,
    world: glam::Affine3A,
    local_center: glam::Vec3,
) -> Option<glam::Affine3A> {
    use crate::doc::store::LayerProjection;
    use glam::Affine3A;
    let seen = layer_projection_transform(comp, camera, from, world.transform_point3(local_center)) * world;
    if !matches!(to, LayerProjection::TwoD) {
        // 3D and 2.5D act about the layer's center, which the switch leaves in place.
        let switched = layer_projection_transform(comp, camera, to, seen.transform_point3(local_center)).inverse() * seen;
        return switched.is_finite().then_some(switched);
    }
    // 2D draws the layer on z=0. Scale the seen geometry toward the eye until its center
    // lands there: the picture is unchanged and the shear is then exact.
    let eye = camera_projection(comp, camera).eye;
    let unframed = two_d_frame(comp, camera).inverse();
    let center = seen.transform_point3(local_center);
    let a = unframed.transform_point3(eye);
    let b = unframed.transform_vector3(center - eye);
    let k = -a.z / b.z;
    if !k.is_finite() || k <= 0.0 { return None; }
    let toward_eye = Affine3A::from_translation(eye) * Affine3A::from_scale(glam::Vec3::splat(k)) * Affine3A::from_translation(-eye);
    let flat = unframed * toward_eye * seen;
    let switched = two_d_center(comp, flat.transform_point3(local_center)).inverse() * flat;
    switched.is_finite().then_some(switched)
}

/// Screen positions of the 8 corners of a layer's local bounds, as the Stage shows them:
/// `camera` decides the layer's projection, `observer` draws the result.
pub fn projected_screen_corners(
    comp: CompSpec,
    camera: ResolvedCamera,
    observer: ResolvedCamera,
    mode: crate::doc::store::LayerProjection,
    world: glam::Affine3A,
    min: [f32; 3],
    max: [f32; 3],
) -> [glam::Vec2; 8] {
    let projection = camera_projection(comp, observer);
    let matrix = projection.projection_matrix() * projection.view_matrix();
    let center = world.transform_point3((glam::Vec3::from(min) + glam::Vec3::from(max)) * 0.5);
    let correction = layer_projection_transform(comp, camera, mode, center);
    std::array::from_fn(|i| {
        let p = glam::Vec3::from_array(std::array::from_fn(|a| if i & (1 << a) == 0 { min[a] } else { max[a] }));
        let c = matrix * correction.transform_point3(world.transform_point3(p)).extend(1.0);
        let w = if c.w.abs() < 1e-6 { 1e-6 } else { c.w };
        glam::vec2((c.x / w + 1.0) * 0.5 * comp.width as f32, (1.0 - c.y / w) * 0.5 * comp.height as f32)
    })
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
    fn two_point_five_d_keeps_its_camera_facing_angle_and_volume_wherever_it_sits() {
        let comp = CompSpec { width: 1920, height: 1080 };
        let normal = glam::Quat::from_rotation_x(0.6) * glam::Vec3::Z;
        let vertices = [glam::Vec3::X*80.0, glam::Vec3::Y*60.0, glam::Vec3::Z*40.0];
        let facing = |camera, v: glam::Vec3| camera_projection(comp, camera).rotation * v;
        for camera in [ResolvedCamera::default(), ResolvedCamera { center:[90.0,-30.0], zoom:1.8, roll_degrees:33.0, ..Default::default() },
            ResolvedCamera { orbit_degrees:[20.0,-35.0], roll_degrees:-10.0, ..Default::default() }] {
            for center in [glam::vec3(250.0,200.0,0.0),glam::vec3(1700.0,900.0,300.0)] {
                let transform=layer_projection_transform(comp,camera,LayerProjection::TwoPointFiveD,center);
                assert!(transform.transform_point3(center).distance(center) < 1e-3);
                assert!(facing(camera, transform.transform_vector3(normal)).distance(facing(ResolvedCamera::default(), normal)) < 1e-5);
                for vertex in vertices {
                    let moved = transform.transform_point3(center+vertex);
                    assert!((moved.distance(center) - vertex.length()).abs() < 1e-3, "volume kept");
                }
            }
            if camera.roll_degrees == 0.0 && camera.orbit_degrees == [0.0; 2] {
                assert_eq!(layer_projection_transform(comp,camera,LayerProjection::TwoPointFiveD,glam::vec3(700.0,100.0,50.0)), glam::Affine3A::IDENTITY);
            }
        }
    }

    #[test]
    fn two_point_five_d_camera_pan_produces_parallax_without_added_tilt() {
        let comp=CompSpec {width:1920,height:1080};
        let baseline=ResolvedCamera::default();
        let panned=ResolvedCamera {center:[100.0,0.0],..baseline};
        let projected=|camera,center:glam::Vec3,point:glam::Vec3| pixel(comp,camera,
            layer_projection_transform(comp,camera,LayerProjection::TwoPointFiveD,center).transform_point3(point));
        let near=glam::vec3(250.0,200.0,0.0);
        let far=glam::vec3(250.0,200.0,350.0);
        let near_shift=projected(panned,near,near)-projected(baseline,near,near);
        let far_shift=projected(panned,far,far)-projected(baseline,far,far);
        assert!(near_shift.x<0.0 && far_shift.x<0.0 && near_shift.x.abs()>far_shift.x.abs());
        for center in [near,far] {
            let edge=projected(panned,center,center+glam::Vec3::X*80.0)-projected(panned,center,center);
            assert!(edge.y.abs()<0.002);
            let rolled=ResolvedCamera {roll_degrees:30.0,..panned};
            let edge=projected(rolled,center,center+glam::Vec3::X*80.0)-projected(rolled,center,center);
            assert!(edge.y.abs()<0.002, "2.5D stays level when the camera rolls");
            let world=|point:glam::Vec3| pixel(comp,rolled,point);
            assert!((world(center+glam::Vec3::X*80.0)-world(center)).y.abs()>1.0, "3D rolls with the camera");
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

    #[test]
    fn projection_switch_keeps_every_screen_corner_in_both_directions() {
        let comp = CompSpec { width: 1920, height: 1080 };
        let min = [-12.0, -20.0, 0.0];
        let max = [212.0, 140.0, 0.0];
        let local_center = (glam::Vec3::from(min) + glam::Vec3::from(max)) * 0.5;
        let world = glam::Affine3A::from_scale_rotation_translation(
            glam::vec3(1.4, 0.8, 1.0),
            glam::Quat::from_rotation_x(0.5) * glam::Quat::from_rotation_y(-0.9) * glam::Quat::from_rotation_z(0.3),
            glam::vec3(1500.0, 200.0, 260.0),
        );
        let cameras = [ResolvedCamera::default(), ResolvedCamera { center: [210.0, -130.0], zoom: 1.7, roll_degrees: 47.0, ..Default::default() }];
        let modes = [LayerProjection::TwoD, LayerProjection::TwoPointFiveD, LayerProjection::ThreeD];
        for camera in cameras {
            for from in modes {
                for to in modes {
                    let switched = projection_switch_world(comp, camera, from, to, world, local_center).unwrap();
                    let before = projected_screen_corners(comp, camera, camera, from, world, min, max);
                    let after = projected_screen_corners(comp, camera, camera, to, switched, min, max);
                    for (a, b) in before.iter().zip(after) {
                        assert!(a.distance(b) < 0.01, "{from:?}->{to:?} {camera:?}: {a:?} != {b:?}");
                    }
                    if matches!(to, LayerProjection::TwoD) {
                        assert!(switched.transform_point3(local_center).z.abs() < 0.01, "2D rests on z=0");
                    } else {
                        let seen = layer_projection_transform(comp, camera, from, world.transform_point3(local_center)) * world;
                        assert!(switched.transform_point3(local_center).distance(seen.transform_point3(local_center)) < 0.01);
                    }
                }
            }
        }
    }

    /// 画面に貼る板(背景・2D 層)は、回して寄せたカメラでも出力の四隅に正確に載る。
    #[test]
    fn a_pinned_frame_lands_on_the_output_corners_under_an_orbiting_camera() {
        let comp = CompSpec { width: 1920, height: 1080 };
        let (w, h) = (comp.width as f32, comp.height as f32);
        for camera in [
            ResolvedCamera::default(),
            ResolvedCamera { orbit_degrees: [-25.0, 60.0], distance_scale: 0.5, ..Default::default() },
            ResolvedCamera { orbit_degrees: [40.0, -120.0], distance_scale: 2.0, zoom: 1.7, roll_degrees: 30.0, center: [200.0, -100.0], target_z: 300.0, ..Default::default() },
        ] {
            let pin = layer_projection_transform(comp, camera, LayerProjection::TwoD, glam::vec3(w * 0.5, h * 0.5, 0.0));
            for corner in [glam::vec3(0.0, 0.0, 0.0), glam::vec3(w, 0.0, 0.0), glam::vec3(0.0, h, 0.0), glam::vec3(w, h, 0.0)] {
                let seen = pixel(comp, camera, pin.transform_point3(corner));
                assert!(seen.distance(corner.truncate()) < 0.05, "{camera:?}: {corner:?} -> {seen:?}");
            }
        }
    }

    #[test]
    fn looking_at_keeps_the_target_under_the_centre_and_the_sphere_inside_the_frame() {
        let comp = CompSpec { width: 1280, height: 720 };
        let centre = glam::vec2(640.0, 360.0);
        for orbit in [[0.0, 0.0], [-15.0, 30.0], [60.0, -140.0]] {
            let before = ResolvedCamera { orbit_degrees: orbit, ..Default::default() };
            for (target, radius) in [(glam::vec3(900.0, 200.0, -350.0), 80.0), (glam::vec3(-40.0, 700.0, 1200.0), 900.0)] {
                let camera = before.looking_at(comp, target, radius);
                assert_eq!(camera.orbit_degrees, orbit, "focus keeps the viewing direction");
                assert!(pixel(comp, camera, target).distance(centre) < 0.01, "target sits under the centre");
                let right = camera_projection(comp, camera).rotation.inverse() * glam::Vec3::X;
                let edge = pixel(comp, camera, target + right * radius);
                assert!(edge.x > centre.x && edge.x < comp.width as f32, "sphere edge stays inside: {edge}");
            }
            assert!(before.looking_at(comp, before.target(comp), 0.0).distance_scale > 0.0);
        }
    }
}
