//! D1i-3: LookAt(rotation角度) / Follow(position+offset) の意味論ゴールデン(S16)。
//! concept: `rotation(t)=look_at(self.center, target.center)`。PlusX/PlusY を固定。
//! 本ファイルのアサーション更新は禁止(新variant+新ファイルのみ)。

use std::collections::BTreeMap;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::param_eval::{eval_doc_param, eval_look_at_rotation, eval_rotation};
use motolii_doc::{
    resolve_document_spaces, Affine2D, Clip, ClipSource, DocParam, Document, ItemEnvelope, LayerId,
    LookAtAxis, ParamEvalError, ResolvedLayerParams, Track, TrackItem, Transform2D,
    RECT_LAYER_SOURCE,
};
use motolii_eval::{DataTracks, Value};

fn approx_angle(got: f64, want: f64) {
    let d = (got - want).abs();
    assert!(d < 1e-12, "got {got} want {want} (Δ={d})");
}

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

fn rotation_of(m: Affine2D) -> f64 {
    m.m[3].atan2(m.m[0])
}

#[test]
fn look_at_plus_x_plus_y_angles() {
    let target = LayerId::from_raw(1);
    let mut resolved = ResolvedLayerParams::default();
    let self_pos = [0.0, 0.0];
    let at = |resolved: &ResolvedLayerParams, axis| {
        eval_look_at_rotation(self_pos, target, axis, resolved).unwrap()
    };

    // self(0,0) → target(1,1): atan2(1,1)=π/4
    resolved.insert_position(target, [1.0, 1.0]);
    approx_angle(at(&resolved, LookAtAxis::PlusX), FRAC_PI_4);
    approx_angle(at(&resolved, LookAtAxis::PlusY), FRAC_PI_4 - FRAC_PI_2);

    // +X 方向(1,0): PlusX→0、PlusY→-π/2
    resolved.insert_position(target, [1.0, 0.0]);
    approx_angle(at(&resolved, LookAtAxis::PlusX), 0.0);
    approx_angle(at(&resolved, LookAtAxis::PlusY), -FRAC_PI_2);

    // +Y 方向(0,1)
    resolved.insert_position(target, [0.0, 1.0]);
    approx_angle(at(&resolved, LookAtAxis::PlusX), FRAC_PI_2);
    approx_angle(at(&resolved, LookAtAxis::PlusY), 0.0);
}

#[test]
fn eval_rotation_dispatches_look_at_axes() {
    let target = LayerId::from_raw(2);
    let mut resolved = ResolvedLayerParams::default();
    resolved.insert_position(target, [1.0, 0.0]);
    let tracks = DataTracks::new();
    let self_pos = [0.0, 0.0];

    approx_angle(
        eval_rotation(
            &DocParam::LookAt {
                target,
                axis: LookAtAxis::PlusX,
            },
            self_pos,
            RationalTime::ZERO,
            &tracks,
            &resolved,
        )
        .unwrap(),
        0.0,
    );
    approx_angle(
        eval_rotation(
            &DocParam::LookAt {
                target,
                axis: LookAtAxis::PlusY,
            },
            self_pos,
            RationalTime::ZERO,
            &tracks,
            &resolved,
        )
        .unwrap(),
        -FRAC_PI_2,
    );
}

#[test]
fn look_at_unresolved_via_eval_look_at_rotation() {
    // 未解決は eval_look_at_rotation 経由で UnresolvedLookAt。
    // eval_doc_param(LookAt) は LookAtRequiresSelfPosition であり、ここでは期待しない。
    let err = eval_look_at_rotation(
        [0.0, 0.0],
        LayerId::from_raw(99),
        LookAtAxis::PlusY,
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

fn rect_clip(layer: LayerId, xform: Transform2D) -> Clip {
    Clip {
        envelope: ItemEnvelope {
            transform: xform,
            ..ItemEnvelope::new(layer)
        },
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::Plugin {
            plugin_id: RECT_LAYER_SOURCE.into(),
            effect_version: 1,
            params: BTreeMap::from([
                ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                ("size".into(), DocParam::const_vec2([0.1, 0.1])),
                ("color".into(), DocParam::const_color([1.0, 1.0, 1.0, 1.0])),
            ]),
            extra: Default::default(),
        },
    }
}

/// 文書順非依存の薄い固定。group/parent の詳細 E2E は `d3_lookat_resolve`。
#[test]
fn look_at_angle_independent_of_document_item_order() {
    let tracks = DataTracks::new();
    let build = |target_first: bool| {
        let mut doc = Document::new_v1();
        doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
        let target = doc.layers.allocate("target").unwrap();
        let looker = doc.layers.allocate("looker").unwrap();
        let tid = doc.track_ids.allocate("V1").unwrap();

        let mut target_xf = Transform2D::identity();
        target_xf.position = DocParam::const_vec2([1.0, 1.0]);
        let mut looker_xf = Transform2D::identity();
        looker_xf.position = DocParam::const_vec2([0.0, 0.0]);
        looker_xf.rotation = DocParam::LookAt {
            target,
            axis: LookAtAxis::PlusX,
        };

        let items = if target_first {
            vec![
                TrackItem::Clip(rect_clip(target, target_xf)),
                TrackItem::Clip(rect_clip(looker, looker_xf)),
            ]
        } else {
            vec![
                TrackItem::Clip(rect_clip(looker, looker_xf)),
                TrackItem::Clip(rect_clip(target, target_xf)),
            ]
        };
        doc.tracks.push(Track { id: tid, items });
        (doc, looker)
    };

    let (doc_a, looker) = build(true);
    let (doc_b, looker_b) = build(false);
    assert_eq!(looker, looker_b);

    let (_, worlds_a) = resolve_document_spaces(&doc_a, RationalTime::ZERO, &tracks).unwrap();
    let (_, worlds_b) = resolve_document_spaces(&doc_b, RationalTime::ZERO, &tracks).unwrap();
    let ra = rotation_of(worlds_a[&looker.get()]);
    let rb = rotation_of(worlds_b[&looker.get()]);
    approx_angle(ra, FRAC_PI_4);
    approx_angle(rb, FRAC_PI_4);
    approx_angle(ra, rb);
}
