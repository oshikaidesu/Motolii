//! 2D変形の合成(D3 / F-3)。
//!
//! 子ローカル→親空間: `M = T(position) · R(rotation) · S(scale) · T(−anchor)`。
//! 親参照は親の `M` を左から合成する。継承は変形のみ。

use motolii_core::RationalTime;
use motolii_eval::DataTracks;

use crate::param_eval::{eval_f64, eval_vec2, ParamEvalError, ResolvedLayerParams};
use crate::schema::Transform2D;

/// 正準空間のアフィン(列ベクトル・同次3x3の上2行相当)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine2D {
    /// `[m00, m01, m02; m10, m11, m12]` で `p' = M * [x,y,1]`。
    pub m: [f64; 6],
}

impl Affine2D {
    pub const IDENTITY: Self = Self {
        m: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
    };

    pub fn translation(tx: f64, ty: f64) -> Self {
        Self {
            m: [1.0, 0.0, tx, 0.0, 1.0, ty],
        }
    }

    pub fn rotation(radians: f64) -> Self {
        let (s, c) = radians.sin_cos();
        Self {
            m: [c, -s, 0.0, s, c, 0.0],
        }
    }

    pub fn scale(sx: f64, sy: f64) -> Self {
        Self {
            m: [sx, 0.0, 0.0, 0.0, sy, 0.0],
        }
    }

    /// `self * other` (左が親、右が子ローカル)。
    pub fn mul(self, other: Self) -> Self {
        let a = self.m;
        let b = other.m;
        Self {
            m: [
                a[0] * b[0] + a[1] * b[3],
                a[0] * b[1] + a[1] * b[4],
                a[0] * b[2] + a[1] * b[5] + a[2],
                a[3] * b[0] + a[4] * b[3],
                a[3] * b[1] + a[4] * b[4],
                a[3] * b[2] + a[4] * b[5] + a[5],
            ],
        }
    }

    pub fn transform_point(self, x: f64, y: f64) -> [f64; 2] {
        let m = self.m;
        [m[0] * x + m[1] * y + m[2], m[3] * x + m[4] * y + m[5]]
    }

    /// 均一スケール近似(軸が直交する前提の列ノルム平均)。rect size 用。
    pub fn approx_scale(self) -> [f64; 2] {
        let sx = (self.m[0] * self.m[0] + self.m[3] * self.m[3]).sqrt();
        let sy = (self.m[1] * self.m[1] + self.m[4] * self.m[4]).sqrt();
        [sx, sy]
    }

    pub fn translation_of(self) -> [f64; 2] {
        [self.m[2], self.m[5]]
    }
}

/// 仕様式: `M = T(position) · R(rotation) · S(scale) · T(−anchor)`。
pub fn compose_local(
    position: [f64; 2],
    anchor: [f64; 2],
    scale: [f64; 2],
    rotation: f64,
) -> Affine2D {
    Affine2D::translation(position[0], position[1])
        .mul(Affine2D::rotation(rotation))
        .mul(Affine2D::scale(scale[0], scale[1]))
        .mul(Affine2D::translation(-anchor[0], -anchor[1]))
}

/// 親空間へ: `M_world = M_parent · M_local`。
pub fn compose_transform(parent: Affine2D, local: Affine2D) -> Affine2D {
    parent.mul(local)
}

pub fn resolve_transform(
    xform: &Transform2D,
    t: RationalTime,
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
) -> Result<Affine2D, ParamEvalError> {
    let position = eval_vec2(&xform.position, t, tracks, resolved)?;
    let anchor = eval_vec2(&xform.anchor, t, tracks, resolved)?;
    let scale = eval_vec2(&xform.scale, t, tracks, resolved)?;
    let rotation = eval_f64(&xform.rotation, t, tracks, resolved)?;
    Ok(compose_local(position, anchor, scale, rotation))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: [f64; 2], b: [f64; 2]) {
        assert!((a[0] - b[0]).abs() < 1e-9, "{a:?} vs {b:?}");
        assert!((a[1] - b[1]).abs() < 1e-9, "{a:?} vs {b:?}");
    }

    #[test]
    fn compose_local_matches_spec_order() {
        // anchor(1,0) → scale(2,1) → rot 90° → pos(3,4)
        let m = compose_local(
            [3.0, 4.0],
            [1.0, 0.0],
            [2.0, 1.0],
            std::f64::consts::FRAC_PI_2,
        );
        // 点(1,0)=anchor は原点へ行きスケール後も原点、回転後も原点、位置へ → (3,4)
        approx(m.transform_point(1.0, 0.0), [3.0, 4.0]);
        // 点(2,0): 相対(1,0) → scale(2,0) → rot90 → (0,2) → +(3,4)=(3,6)
        approx(m.transform_point(2.0, 0.0), [3.0, 6.0]);
    }

    #[test]
    fn parent_left_multiplies_child() {
        let parent = compose_local([10.0, 0.0], [0.0, 0.0], [1.0, 1.0], 0.0);
        let child = compose_local([1.0, 2.0], [0.0, 0.0], [1.0, 1.0], 0.0);
        let world = compose_transform(parent, child);
        approx(world.transform_point(0.0, 0.0), [11.0, 2.0]);
    }
}
