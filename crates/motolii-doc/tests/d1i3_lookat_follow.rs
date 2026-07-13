//! D1i-3: LookAt / Follow の意味論ゴールデン(S16)。
//! D3 `param_eval` の現行契約を固定する(軸付き回転 LookAt は未実装 — 発明しない)。
//! 本ファイルのアサーション更新は禁止(新variant+新ファイルのみ)。

use motolii_core::RationalTime;
use motolii_doc::param_eval::eval_doc_param;
use motolii_doc::{DocParam, LayerId, LookAtAxis, ParamEvalError, ResolvedLayerParams};
use motolii_eval::{DataTracks, Value};

fn approx_vec2(got: Value, want: [f64; 2]) {
    match got {
        Value::Vec2(v) => {
            assert!(
                (v[0] - want[0]).abs() < 1e-12 && (v[1] - want[1]).abs() < 1e-12,
                "expected {want:?}, got {v:?}"
            );
        }
        other => panic!("expected Vec2, got {other:?}"),
    }
}

#[test]
fn look_at_returns_resolved_target_position() {
    let target = LayerId::from_raw(7);
    let mut resolved = ResolvedLayerParams::default();
    resolved.insert_position(target, [0.25, -0.5]);
    let param = DocParam::LookAt {
        target,
        axis: LookAtAxis::PlusY,
    };
    let got = eval_doc_param(&param, RationalTime::ZERO, &DataTracks::new(), &resolved).unwrap();
    approx_vec2(got, [0.25, -0.5]);
}

#[test]
fn look_at_axis_does_not_alter_evaluated_position() {
    // v1 の LookAt は position 受け口のみ。axis はスキーマ予約で評価値に影響しない。
    let target = LayerId::from_raw(3);
    let mut resolved = ResolvedLayerParams::default();
    resolved.insert_position(target, [1.0, 2.0]);
    let tracks = DataTracks::new();
    for axis in [LookAtAxis::PlusY, LookAtAxis::PlusX] {
        let param = DocParam::LookAt { target, axis };
        let got = eval_doc_param(&param, RationalTime::ZERO, &tracks, &resolved).unwrap();
        approx_vec2(got, [1.0, 2.0]);
    }
}

#[test]
fn look_at_unresolved_is_typed_error() {
    let param = DocParam::LookAt {
        target: LayerId::from_raw(99),
        axis: LookAtAxis::PlusY,
    };
    let err = eval_doc_param(
        &param,
        RationalTime::ZERO,
        &DataTracks::new(),
        &ResolvedLayerParams::default(),
    )
    .unwrap_err();
    assert!(matches!(err, ParamEvalError::UnresolvedLookAt(99)));
}

#[test]
fn follow_adds_offset_to_resolved_target_position() {
    let target = LayerId::from_raw(4);
    let mut resolved = ResolvedLayerParams::default();
    resolved.insert_position(target, [1.0, 2.0]);
    let param = DocParam::Follow {
        target,
        offset: [0.5, -0.25],
    };
    let got = eval_doc_param(&param, RationalTime::ZERO, &DataTracks::new(), &resolved).unwrap();
    approx_vec2(got, [1.5, 1.75]);
}

#[test]
fn follow_zero_offset_matches_look_at_position() {
    let target = LayerId::from_raw(5);
    let mut resolved = ResolvedLayerParams::default();
    resolved.insert_position(target, [-0.3, 0.8]);
    let tracks = DataTracks::new();
    let look = eval_doc_param(
        &DocParam::LookAt {
            target,
            axis: LookAtAxis::PlusX,
        },
        RationalTime::ZERO,
        &tracks,
        &resolved,
    )
    .unwrap();
    let follow = eval_doc_param(
        &DocParam::Follow {
            target,
            offset: [0.0, 0.0],
        },
        RationalTime::ZERO,
        &tracks,
        &resolved,
    )
    .unwrap();
    assert_eq!(look, follow);
}

#[test]
fn follow_unresolved_is_typed_error() {
    let param = DocParam::Follow {
        target: LayerId::from_raw(42),
        offset: [1.0, 0.0],
    };
    let err = eval_doc_param(
        &param,
        RationalTime::ZERO,
        &DataTracks::new(),
        &ResolvedLayerParams::default(),
    )
    .unwrap_err();
    assert!(matches!(err, ParamEvalError::UnresolvedFollow(42)));
}
