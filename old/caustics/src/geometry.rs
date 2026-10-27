//! Closed glass meshes: a tube swept along a closed 3D path, and a flattened pebble.
//! Only geometry lives here; every optical decision is in the borrowed WGSL.
use bytemuck::{Pod, Zeroable};
use glam::{Quat, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// Makes every triangle counter-clockwise when seen from outside, using the vertex normals as the
/// definition of "outside". A closed, consistently-wound mesh is what the two-sided capture needs.
fn orient_outward(m: &mut Mesh) {
    for t in m.indices.chunks_exact_mut(3) {
        let p = |i: u32| Vec3::from(m.vertices[i as usize].position);
        let n = |i: u32| Vec3::from(m.vertices[i as usize].normal);
        let face = (p(t[1]) - p(t[0])).cross(p(t[2]) - p(t[0]));
        if face.dot(n(t[0]) + n(t[1]) + n(t[2])) < 0.0 {
            t.swap(1, 2);
        }
    }
}

fn catmull_closed(points: &[Vec3], per_span: usize) -> Vec<Vec3> {
    let n = points.len() as i32;
    let at = |i: i32| points[(((i % n) + n) % n) as usize];
    let mut out = Vec::new();
    for i in 0..n {
        let (p0, p3) = (at(i), at(i + 1));
        let p1 = p0 + (at(i + 1) - at(i - 1)) * (1.0 / 6.0);
        let p2 = p3 - (at(i + 2) - at(i)) * (1.0 / 6.0);
        for s in 0..per_span {
            let t = s as f32 / per_span as f32;
            let u = 1.0 - t;
            out.push(p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t));
        }
    }
    out
}

/// Sweeps a circle along a closed path with parallel-transport frames, then spreads the leftover
/// twist over the whole loop so the last ring meets the first without a seam.
pub fn tube(path: &[Vec3], sides: usize, radius: f32) -> Mesh {
    let n = path.len();
    let tangent = |i: usize| (path[(i + 1) % n] - path[(i + n - 1) % n]).normalize();
    let mut ns = vec![Vec3::ZERO; n];
    let t0 = tangent(0);
    let helper = if t0.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
    ns[0] = (helper - t0 * helper.dot(t0)).normalize();
    for i in 1..n {
        let (ta, tb) = (tangent(i - 1), tangent(i));
        let axis = ta.cross(tb);
        let moved = if axis.length() < 1e-6 {
            ns[i - 1]
        } else {
            Quat::from_axis_angle(axis.normalize(), ta.dot(tb).clamp(-1.0, 1.0).acos()) * ns[i - 1]
        };
        ns[i] = (moved - tb * moved.dot(tb)).normalize();
    }
    // Residual twist between the transported last frame and the first one.
    let (ta, tb) = (tangent(n - 1), tangent(0));
    let axis = ta.cross(tb);
    let end = if axis.length() < 1e-6 {
        ns[n - 1]
    } else {
        Quat::from_axis_angle(axis.normalize(), ta.dot(tb).clamp(-1.0, 1.0).acos()) * ns[n - 1]
    };
    let end = (end - tb * end.dot(tb)).normalize();
    let twist = tb.dot(end.cross(ns[0])).atan2(end.dot(ns[0]));
    let mut vertices = Vec::with_capacity(n * sides);
    for i in 0..n {
        let t = tangent(i);
        let ni = Quat::from_axis_angle(t, twist * i as f32 / n as f32) * ns[i];
        let bi = t.cross(ni);
        for k in 0..sides {
            let a = k as f32 / sides as f32 * std::f32::consts::TAU;
            let radial = ni * a.cos() + bi * a.sin();
            vertices.push(Vertex { position: (path[i] + radial * radius).into(), normal: radial.into() });
        }
    }
    let mut indices = Vec::with_capacity(n * sides * 6);
    for i in 0..n {
        let i1 = (i + 1) % n;
        for k in 0..sides {
            let k1 = (k + 1) % sides;
            let (a, b, c, d) = (
                (i * sides + k) as u32,
                (i * sides + k1) as u32,
                (i1 * sides + k) as u32,
                (i1 * sides + k1) as u32,
            );
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    let mut m = Mesh { vertices, indices };
    orient_outward(&mut m);
    m
}

/// The closed 3D path used in beauty.mm (six control points), laid around a cube-sized object on a floor.
pub fn glass_loop() -> Mesh {
    let src: [[f32; 3]; 6] = [
        [-0.78, -0.22, 0.12],
        [-0.22, -0.48, 0.38],
        [0.55, -0.12, 0.22],
        [0.42, 0.46, -0.22],
        [-0.18, 0.38, -0.48],
        [-0.68, 0.08, -0.12],
    ];
    let ring: Vec<Vec3> = src
        .iter()
        .map(|q| Vec3::new((q[0] + 0.095) * 0.55 - 0.25, 0.22 + q[2] * 0.20, q[1] * 0.70))
        .collect();
    tube(&catmull_closed(&ring, 40), 64, 0.035)
}

/// A flattened ellipsoid lying on the floor, tilted about Z.
pub fn pebble(center: Vec3, radii: Vec3, tilt: f32) -> Mesh {
    let (rings, segs) = (48usize, 96usize);
    let rot = Quat::from_rotation_z(tilt);
    let mut vertices = Vec::new();
    for i in 0..=rings {
        let theta = i as f32 / rings as f32 * std::f32::consts::PI;
        for j in 0..=segs {
            let phi = j as f32 / segs as f32 * std::f32::consts::TAU;
            let unit = Vec3::new(theta.sin() * phi.cos(), theta.cos(), theta.sin() * phi.sin());
            let p = center + rot * (unit * radii);
            let n = rot * (unit / radii).normalize();
            vertices.push(Vertex { position: p.into(), normal: n.into() });
        }
    }
    let mut indices = Vec::new();
    let w = (segs + 1) as u32;
    for i in 0..rings as u32 {
        for j in 0..segs as u32 {
            let (a, b, c, d) = (i * w + j, i * w + j + 1, (i + 1) * w + j, (i + 1) * w + j + 1);
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    let mut m = Mesh { vertices, indices };
    orient_outward(&mut m);
    m
}
