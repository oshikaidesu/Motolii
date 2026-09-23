//! Simplex noise, 3D, on the CPU. Port of Ashima Arts / Stefan Gustavson's webgl-noise (MIT).
//! Mirrors `effects/program/noise.wgsl` so points displaced on the CPU and meshes displaced on the GPU
//! move through the same field.

/// Shared GPU source for image and geometry deformation.
pub const WGSL: &str = include_str!("effects/program/noise.wgsl");

use glam::{Vec3, Vec3Swizzles as _, Vec4, Vec4Swizzles as _};

fn mod289_3(x: Vec3) -> Vec3 {
    x - (x * (1.0 / 289.0)).floor() * 289.0
}
fn mod289_4(x: Vec4) -> Vec4 {
    x - (x * (1.0 / 289.0)).floor() * 289.0
}
fn permute(x: Vec4) -> Vec4 {
    mod289_4(((x * 34.0) + 10.0) * x)
}
fn taylor_inv_sqrt(r: Vec4) -> Vec4 {
    Vec4::splat(1.79284291400159) - 0.85373472095314 * r
}
fn step(edge: Vec3, x: Vec3) -> Vec3 {
    Vec3::new(
        if x.x < edge.x { 0.0 } else { 1.0 },
        if x.y < edge.y { 0.0 } else { 1.0 },
        if x.z < edge.z { 0.0 } else { 1.0 },
    )
}
fn step4(edge: Vec4, x: Vec4) -> Vec4 {
    Vec4::new(
        if x.x < edge.x { 0.0 } else { 1.0 },
        if x.y < edge.y { 0.0 } else { 1.0 },
        if x.z < edge.z { 0.0 } else { 1.0 },
        if x.w < edge.w { 0.0 } else { 1.0 },
    )
}

/// Simplex noise in [-1, 1].
pub fn simplex3(v: Vec3) -> f32 {
    let c = glam::vec2(1.0 / 6.0, 1.0 / 3.0);
    let d = Vec4::new(0.0, 0.5, 1.0, 2.0);

    let mut i = (v + Vec3::splat(v.dot(Vec3::splat(c.y)))).floor();
    let x0 = v - i + Vec3::splat(i.dot(Vec3::splat(c.x)));

    let g = step(x0.yzx(), x0);
    let l = Vec3::ONE - g;
    let i1 = g.min(l.zxy());
    let i2 = g.max(l.zxy());

    let x1 = x0 - i1 + Vec3::splat(c.x);
    let x2 = x0 - i2 + Vec3::splat(c.y);
    let x3 = x0 - Vec3::splat(d.y);

    i = mod289_3(i);
    let p = permute(
        permute(
            permute(Vec4::splat(i.z) + Vec4::new(0.0, i1.z, i2.z, 1.0))
                + Vec4::splat(i.y)
                + Vec4::new(0.0, i1.y, i2.y, 1.0),
        ) + Vec4::splat(i.x)
            + Vec4::new(0.0, i1.x, i2.x, 1.0),
    );

    let n_ = 0.142857142857f32;
    let ns = n_ * Vec3::new(d.w, d.y, d.z) - Vec3::new(d.x, d.z, d.x);

    let j = p - 49.0 * (p * ns.z * ns.z).floor();
    let x_ = (j * ns.z).floor();
    let y_ = (j - 7.0 * x_).floor();

    let x = x_ * ns.x + Vec4::splat(ns.y);
    let y = y_ * ns.x + Vec4::splat(ns.y);
    let h = Vec4::ONE - x.abs() - y.abs();

    let b0 = Vec4::new(x.x, x.y, y.x, y.y);
    let b1 = Vec4::new(x.z, x.w, y.z, y.w);

    let s0 = b0.floor() * 2.0 + Vec4::ONE;
    let s1 = b1.floor() * 2.0 + Vec4::ONE;
    let sh = -step4(h, Vec4::ZERO);

    let a0 = b0.xzyw() + s0.xzyw() * sh.xxyy();
    let a1 = b1.xzyw() + s1.xzyw() * sh.zzww();

    let mut p0 = Vec3::new(a0.x, a0.y, h.x);
    let mut p1 = Vec3::new(a0.z, a0.w, h.y);
    let mut p2 = Vec3::new(a1.x, a1.y, h.z);
    let mut p3 = Vec3::new(a1.z, a1.w, h.w);

    let norm = taylor_inv_sqrt(Vec4::new(p0.dot(p0), p1.dot(p1), p2.dot(p2), p3.dot(p3)));
    p0 *= norm.x;
    p1 *= norm.y;
    p2 *= norm.z;
    p3 *= norm.w;

    let mut m = (Vec4::splat(0.6) - Vec4::new(x0.dot(x0), x1.dot(x1), x2.dot(x2), x3.dot(x3)))
        .max(Vec4::ZERO);
    m *= m;
    42.0 * (m * m).dot(Vec4::new(p0.dot(x0), p1.dot(x1), p2.dot(x2), p3.dot(x3)))
}

/// Fractal sum of `octaves` simplex layers (lacunarity 2, gain 0.5), normalized to about [-1, 1].
pub fn fbm3(p: Vec3, octaves: u32) -> f32 {
    let mut sum = 0.0;
    let mut amplitude = 0.5f32;
    let mut frequency = 1.0f32;
    let mut total = 0.0f32;
    for _ in 0..octaves {
        sum += amplitude * simplex3(p * frequency);
        total += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    sum / total.max(1e-6)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplex_is_bounded_smooth_and_varied() {
        let mut max = 0.0f32;
        let mut varied = false;
        for i in 0..2000 {
            let p = Vec3::new(
                i as f32 * 0.173,
                (i as f32 * 0.311).sin() * 9.0,
                i as f32 * -0.057,
            );
            let v = simplex3(p);
            assert!(v.is_finite() && v.abs() <= 1.0, "{p:?} -> {v}");
            let near = simplex3(p + Vec3::splat(1e-3));
            assert!(
                (v - near).abs() < 0.05,
                "not smooth at {p:?}: {v} vs {near}"
            );
            max = max.max(v.abs());
            varied |= v.abs() > 0.3;
        }
        assert!(varied && max > 0.5, "max {max}");
        assert!(fbm3(Vec3::new(0.3, 0.4, 0.5), 4).abs() <= 1.0);
    }
}
