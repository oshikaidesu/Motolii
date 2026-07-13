//! 2D変形の合成(D3 / F-3)。
//!
//! 子ローカル→親空間: `M = T(position) · R(rotation) · S(scale) · T(−anchor)`。
//! 親参照は親の `M` を左から合成する。継承は変形のみ。

use std::collections::HashSet;
use std::ops::Mul;

use motolii_core::RationalTime;
use motolii_eval::DataTracks;

use crate::param_eval::{eval_f64, eval_vec2, ParamEvalError, ResolvedLayerParams};
use crate::schema::Transform2D;
use crate::LayerId;

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

    pub fn is_approx_identity(self) -> bool {
        const EPS: f64 = 1e-12;
        (self.m[0] - 1.0).abs() < EPS
            && self.m[1].abs() < EPS
            && self.m[2].abs() < EPS
            && self.m[3].abs() < EPS
            && (self.m[4] - 1.0).abs() < EPS
            && self.m[5].abs() < EPS
    }

    fn det2(self) -> f64 {
        self.m[0] * self.m[4] - self.m[1] * self.m[3]
    }

    /// 逆アフィン。特異なら None。
    pub fn try_invert(self) -> Option<Self> {
        let det = self.det2();
        if det.abs() < f64::EPSILON {
            return None;
        }
        let inv = 1.0 / det;
        let m = self.m;
        Some(Self {
            m: [
                m[4] * inv,
                -m[1] * inv,
                (m[1] * m[5] - m[4] * m[2]) * inv,
                -m[3] * inv,
                m[0] * inv,
                (m[3] * m[2] - m[0] * m[5]) * inv,
            ],
        })
    }

    /// 正準アフィンを UV 空間(原点左上・Y-down・[0,1]²)へ写した逆行列。
    /// `uv_src = inv_uv * uv_dst` でサンプリングする。
    pub fn to_inverse_uv_matrix(self, aspect: f64) -> Option<[f32; 6]> {
        let inv = self.try_invert()?;
        // C: UV→canonical, C_inv: canonical→UV。A = C_inv · M_inv · C
        let a = aspect;
        let m = inv.m;
        // C = [a, 0, -0.5a; 0, -1, 0.5]
        // M_inv · C
        let n0 = m[0] * a;
        let n1 = -m[1];
        let n2 = m[0] * (-0.5 * a) + m[1] * 0.5 + m[2];
        let n3 = m[3] * a;
        let n4 = -m[4];
        let n5 = m[3] * (-0.5 * a) + m[4] * 0.5 + m[5];
        // C_inv = [1/a, 0, 0.5; 0, -1, 0.5]
        let inv_a = 1.0 / a;
        Some([
            (inv_a * n0) as f32,
            (inv_a * n1) as f32,
            (inv_a * n2 + 0.5) as f32,
            (-n3) as f32,
            (-n4) as f32,
            (-n5 + 0.5) as f32,
        ])
    }
}

/// `self * other` (左が親、右が子ローカル)。
impl Mul for Affine2D {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
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
}

/// 仕様式: `M = T(position) · R(rotation) · S(scale) · T(−anchor)`。
pub fn compose_local(
    position: [f64; 2],
    anchor: [f64; 2],
    scale: [f64; 2],
    rotation: f64,
) -> Affine2D {
    Affine2D::translation(position[0], position[1])
        * Affine2D::rotation(rotation)
        * Affine2D::scale(scale[0], scale[1])
        * Affine2D::translation(-anchor[0], -anchor[1])
}

/// 親空間へ: `M_world = M_parent · M_local`。
pub fn compose_transform(parent: Affine2D, local: Affine2D) -> Affine2D {
    parent * local
}

/// `lookup(layer)` は親レイヤーの `Transform2D` を返す。validate の森検査と揃える。
pub fn resolve_transform<'doc>(
    xform: &Transform2D,
    t: RationalTime,
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
    lookup: &dyn Fn(LayerId) -> Option<&'doc Transform2D>,
) -> Result<Affine2D, ParamEvalError> {
    let mut visiting = HashSet::new();
    resolve_transform_rec(xform, t, tracks, resolved, lookup, &mut visiting)
}

fn resolve_local_only(
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

fn resolve_transform_rec<'doc>(
    xform: &Transform2D,
    t: RationalTime,
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
    lookup: &dyn Fn(LayerId) -> Option<&'doc Transform2D>,
    visiting: &mut HashSet<u64>,
) -> Result<Affine2D, ParamEvalError> {
    let local = resolve_local_only(xform, t, tracks, resolved)?;
    let Some(parent_id) = xform.parent else {
        return Ok(local);
    };
    let pid = parent_id.get();
    if !visiting.insert(pid) {
        return Err(ParamEvalError::ParentCycle { layer: pid });
    }
    let parent_xform = lookup(parent_id).ok_or(ParamEvalError::DanglingParent { parent: pid })?;
    let parent_m = resolve_transform_rec(parent_xform, t, tracks, resolved, lookup, visiting)?;
    visiting.remove(&pid);
    Ok(compose_transform(parent_m, local))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::param::DocParam;
    use crate::schema::Transform2D;
    use motolii_eval::DataTracks;

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

    #[test]
    fn resolve_transform_composes_parent_chain() {
        let parent_id = LayerId::from_raw(1);
        let parent = Transform2D {
            position: DocParam::const_vec2([10.0, 0.0]),
            ..Transform2D::identity()
        };
        let child = Transform2D {
            position: DocParam::const_vec2([1.0, 2.0]),
            parent: Some(parent_id),
            ..Transform2D::identity()
        };
        let tracks = DataTracks::new();
        let resolved = ResolvedLayerParams::default();
        let lookup = |id: LayerId| {
            if id == parent_id {
                Some(&parent)
            } else {
                None
            }
        };
        let world =
            resolve_transform(&child, RationalTime::ZERO, &tracks, &resolved, &lookup).unwrap();
        approx(world.transform_point(0.0, 0.0), [11.0, 2.0]);
    }

    #[test]
    fn resolve_transform_rejects_parent_cycle() {
        let a = LayerId::from_raw(1);
        let b = LayerId::from_raw(2);
        let xa = Transform2D {
            parent: Some(b),
            ..Transform2D::identity()
        };
        let xb = Transform2D {
            parent: Some(a),
            ..Transform2D::identity()
        };
        let tracks = DataTracks::new();
        let resolved = ResolvedLayerParams::default();
        let lookup = |id: LayerId| {
            if id == a {
                Some(&xa)
            } else if id == b {
                Some(&xb)
            } else {
                None
            }
        };
        let err =
            resolve_transform(&xa, RationalTime::ZERO, &tracks, &resolved, &lookup).unwrap_err();
        assert!(matches!(err, ParamEvalError::ParentCycle { .. }));
    }

    #[test]
    fn resolve_transform_rejects_dangling_parent() {
        let child = Transform2D {
            parent: Some(LayerId::from_raw(99)),
            ..Transform2D::identity()
        };
        let tracks = DataTracks::new();
        let resolved = ResolvedLayerParams::default();
        let lookup = |_id: LayerId| None;
        let err =
            resolve_transform(&child, RationalTime::ZERO, &tracks, &resolved, &lookup).unwrap_err();
        assert!(matches!(err, ParamEvalError::DanglingParent { parent: 99 }));
    }
}
