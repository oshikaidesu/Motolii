//! D1i-3: Transform合成の意味論ゴールデン(S16)。
//! `M = T(position) · R(rotation) · S(scale) · T(−anchor)` と親左合成を固定する。
//! 本ファイルのアサーション更新は禁止(新variant+新ファイルのみ)。

use motolii_core::RationalTime;
use motolii_doc::{
    compose_local, compose_transform, resolve_transform, Affine2D, DocParam, LayerId,
    ParamEvalError, ResolvedLayerParams, Transform2D,
};
use motolii_eval::DataTracks;

fn approx_pt(got: [f64; 2], want: [f64; 2]) {
    assert!(
        (got[0] - want[0]).abs() < 1e-9 && (got[1] - want[1]).abs() < 1e-9,
        "expected {want:?}, got {got:?}"
    );
}

fn approx_m(got: Affine2D, want: [f64; 6]) {
    for (i, (a, b)) in got.m.iter().zip(want.iter()).enumerate() {
        assert!(
            (a - b).abs() < 1e-9,
            "m[{i}]: expected {b}, got {a} (full {got:?} vs {want:?})"
        );
    }
}

#[test]
fn compose_local_matches_spec_trs_anchor_order() {
    // anchor(1,0) → scale(2,1) → rot 90° → pos(3,4)
    let m = compose_local(
        [3.0, 4.0],
        [1.0, 0.0],
        [2.0, 1.0],
        std::f64::consts::FRAC_PI_2,
    );
    // 点(1,0)=anchor は原点へ行きスケール後も原点、回転後も原点、位置へ → (3,4)
    approx_pt(m.transform_point(1.0, 0.0), [3.0, 4.0]);
    // 点(2,0): 相対(1,0) → scale(2,0) → rot90 → (0,2) → +(3,4)=(3,6)
    approx_pt(m.transform_point(2.0, 0.0), [3.0, 6.0]);
    // 列ベクトル同次上2行を数値固定(解釈変更の検出用)
    // T(3,4)·R(π/2)·S(2,1)·T(−1,0) → [0,-1,3; 2,0,2]
    approx_m(m, [0.0, -1.0, 3.0, 2.0, 0.0, 2.0]);
}

#[test]
fn parent_left_multiplies_child_local() {
    let parent = compose_local([10.0, 0.0], [0.0, 0.0], [1.0, 1.0], 0.0);
    let child = compose_local([1.0, 2.0], [0.0, 0.0], [1.0, 1.0], 0.0);
    let world = compose_transform(parent, child);
    approx_pt(world.transform_point(0.0, 0.0), [11.0, 2.0]);
    // M_world = M_parent · M_local(左が親)
    approx_m(world, [1.0, 0.0, 11.0, 0.0, 1.0, 2.0]);
}

#[test]
fn parent_rotation_scales_child_translation() {
    // 親が90°回転するとき、子の並進(1,0)は親空間で(0,1)へ写る。
    let parent = compose_local(
        [0.0, 0.0],
        [0.0, 0.0],
        [1.0, 1.0],
        std::f64::consts::FRAC_PI_2,
    );
    let child = compose_local([1.0, 0.0], [0.0, 0.0], [1.0, 1.0], 0.0);
    let world = compose_transform(parent, child);
    approx_pt(world.transform_point(0.0, 0.0), [0.0, 1.0]);
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
    let world = resolve_transform(&child, RationalTime::ZERO, &tracks, &resolved, &lookup).unwrap();
    approx_pt(world.transform_point(0.0, 0.0), [11.0, 2.0]);
}

#[test]
fn resolve_transform_rejects_parent_cycle_with_typed_error() {
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
    let err = resolve_transform(&xa, RationalTime::ZERO, &tracks, &resolved, &lookup).unwrap_err();
    assert!(matches!(err, ParamEvalError::ParentCycle { .. }));
}

#[test]
fn identity_local_is_approx_identity_matrix() {
    let m = compose_local([0.0, 0.0], [0.0, 0.0], [1.0, 1.0], 0.0);
    assert!(m.is_approx_identity());
}
