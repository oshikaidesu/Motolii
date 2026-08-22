//! `modulator` 発注単位(裁定213) — 「接続子は加算・ゲートはキーフレーム」。
//!
//! ここで固定するのは:
//! - 値 = `PropertySource::base` の評価値 + Σ(`modulators` の寄与)(可換・順序非依存・
//!   負の寄与も対象——減算という別モードは作らない、負を足すのが減算)
//! - `Intent::SetPropertyModulators`(新設)は **`base` を読んで保つ** — `SetTrack`/
//!   `SetPropertySlot`/`SetPropertyLink` の「丸ごと置き換え」とは違う口
//! - 変調できる型の境界は `motolii_eval::Value::add` が決める: `F64`/`Vec2`/`Color`/
//!   `Path`(頂点数一致時)は加算できる、`Bool`/`Enum`/`LayerId` は単一 source が勝つ
//! - **成分軸は既存の `PropertyId` split(`position.x`/`position.y`、裁定61)を
//!   再利用する** — 新しい「どの成分か」の指し方は作らない
//! - `Path` の **instance 軸(頂点単位)は今は 1 に固定**(全頂点一括の和のみ)
//! - スケールが 0 を跨いでも(反転)`world_affine`/`local_transform` は panic しない
//!   (`.inverse()` を呼ばない前向き合成のみなので無害 — `AGENTS.md` の
//!   `Mat2::inverse()` 自己アサート地雷とは別の経路)
//! - 負の `time_offset` は許可する。source_t が最初のキーフレームより前になっても
//!   既存の Hold 拡張がそのまま適用される

use motolii_store::{
    property, Composition, Document, Fps, Interp, Intent, Keyframe, KeyframeTrack, LayerId,
    LayerMeta, LayerSource, LayerTiming, PropertyId, PropertyLink, PropertySource, RationalTime,
    Slot, SlotId, Value,
};

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

fn still(value: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value,
        interp: Interp::Hold,
        spatial: None,
    });
    track
}

fn identity_link(source_layer: LayerId, source_property: PropertyId) -> PropertyLink {
    PropertyLink {
        source_layer,
        source_property,
        time_offset: RationalTime::ZERO,
        plugin_id: "motolii.link.identity".to_owned(),
        params: Vec::new(),
    }
}

fn doc_with_two_layers() -> (Document, LayerId, LayerId) {
    let mut doc = Document::new();
    let (a, b) = (LayerId(1), LayerId(2));
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    for layer in [a, b] {
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Solid {
                        rgba: [255, 0, 0, 255],
                        width: 64,
                        height: 64,
                    },
                    order: layer.0 as i16,
                    timing: LayerTiming::place(0, None, 300),
                },
            },
        ])
        .unwrap();
    }
    (doc, a, b)
}

// ---------------------------------------------------------------------------
// 基本の和 — base + modulator
// ---------------------------------------------------------------------------

/// **本題**: base(普通の track)の上に modulator(別 layer の値)を加算する。
#[test]
fn a_modulator_adds_onto_an_existing_track_base() {
    let (mut doc, a, b) = doc_with_two_layers();
    let noise = PropertyId::new("noise").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: noise.clone(),
        track: still(Value::F64(0.2)),
    })
    .unwrap();

    let opacity = PropertyId::new(property::OPACITY).unwrap();
    doc.apply(Intent::SetTrack {
        layer: b,
        property: opacity.clone(),
        track: still(Value::F64(0.5)),
    })
    .unwrap();
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: opacity.clone(),
        modulators: vec![identity_link(a, noise)],
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(b, &opacity, t(0)).unwrap(),
        Some(Value::F64(0.7)),
        "0.5 (base) + 0.2 (modulator) = 0.7 のはず"
    );
    // **base は保たれている** — `SetPropertyModulators` は `SetPropertyLink` と
    // 違って丸ごと置き換えない。
    assert!(matches!(
        doc.view().property_source(b, &opacity).unwrap(),
        Some(PropertySource {
            base: Some(motolii_store::PropertyBase::Track(_)),
            ..
        })
    ));
}

/// **負の modulator は減算になる** — 別モードを作らず、負を足すだけ。
#[test]
fn a_negative_modulator_subtracts() {
    let (mut doc, a, b) = doc_with_two_layers();
    let drag = PropertyId::new("drag").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: drag.clone(),
        track: still(Value::F64(-3.0)),
    })
    .unwrap();

    let x = PropertyId::new("x").unwrap();
    doc.apply(Intent::SetTrack {
        layer: b,
        property: x.clone(),
        track: still(Value::F64(10.0)),
    })
    .unwrap();
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: x.clone(),
        modulators: vec![identity_link(a, drag)],
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(b, &x, t(0)).unwrap(),
        Some(Value::F64(7.0)),
        "10 + (-3) = 7 のはず"
    );
}

/// **複数 modulator は可換・順序非依存**——列を逆順にしても同じ結果。
#[test]
fn multiple_modulators_sum_and_the_order_does_not_matter() {
    let (mut doc, a, b) = doc_with_two_layers();
    let m1 = PropertyId::new("m1").unwrap();
    let m2 = PropertyId::new("m2").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: m1.clone(),
        track: still(Value::F64(3.0)),
    })
    .unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: m2.clone(),
        track: still(Value::F64(4.0)),
    })
    .unwrap();

    let target = PropertyId::new("target").unwrap();
    doc.apply(Intent::SetTrack {
        layer: b,
        property: target.clone(),
        track: still(Value::F64(0.0)),
    })
    .unwrap();
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: target.clone(),
        modulators: vec![identity_link(a, m1.clone()), identity_link(a, m2.clone())],
    })
    .unwrap();
    let forward = doc.view().value_at(b, &target, t(0)).unwrap();

    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: target.clone(),
        modulators: vec![identity_link(a, m2), identity_link(a, m1)],
    })
    .unwrap();
    let reversed = doc.view().value_at(b, &target, t(0)).unwrap();

    assert_eq!(forward, Some(Value::F64(7.0)));
    assert_eq!(forward, reversed, "modulator の列の順序で結果が変わっている");
}

/// **base が無い(=`None`)場合、modulator の和だけが値になる** — 旧
/// `PropertySource::Link` の「置き換え」を加算が包含することの直接固定
/// (`PropertySource::link_only` と全く同じ値になる)。
#[test]
fn a_modulator_with_no_base_is_the_value_by_itself() {
    let (mut doc, a, b) = doc_with_two_layers();
    let source = PropertyId::new("source").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: source.clone(),
        track: still(Value::F64(42.0)),
    })
    .unwrap();

    let target = PropertyId::new("target").unwrap();
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: target.clone(),
        modulators: vec![identity_link(a, source)],
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(b, &target, t(0)).unwrap(),
        Some(Value::F64(42.0))
    );
}

/// 参照先に値が無い(track 未設定)modulator は寄与しない——base だけの値になる
/// (裁定20「キーを打っていない property は静止値」の応用、ぶら下がった参照と同じ扱い)。
#[test]
fn a_modulator_pointing_at_a_valueless_source_contributes_nothing() {
    let (mut doc, a, b) = doc_with_two_layers();
    let ghost = PropertyId::new("ghost").unwrap();

    let opacity = PropertyId::new(property::OPACITY).unwrap();
    doc.apply(Intent::SetTrack {
        layer: b,
        property: opacity.clone(),
        track: still(Value::F64(0.5)),
    })
    .unwrap();
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: opacity.clone(),
        modulators: vec![identity_link(a, ghost)],
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(b, &opacity, t(0)).unwrap(),
        Some(Value::F64(0.5)),
        "値の無い modulator が base を巻き込んで壊している"
    );
}

// ---------------------------------------------------------------------------
// 型の境界 — Value::add が決める(motolii-eval)
// ---------------------------------------------------------------------------

/// `Vec2`/`Color` は成分ごとの和(`Color` はアルファも含む)。
#[test]
fn vec2_and_color_modulators_sum_component_wise_including_alpha() {
    let (mut doc, a, b) = doc_with_two_layers();
    let offset = PropertyId::new("offset").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: offset.clone(),
        track: still(Value::Vec2([1.0, -2.0])),
    })
    .unwrap();
    let anchor = PropertyId::new(property::ANCHOR).unwrap();
    doc.apply(Intent::SetTrack {
        layer: b,
        property: anchor.clone(),
        track: still(Value::Vec2([10.0, 10.0])),
    })
    .unwrap();
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: anchor.clone(),
        modulators: vec![identity_link(a, offset)],
    })
    .unwrap();
    assert_eq!(
        doc.view().value_at(b, &anchor, t(0)).unwrap(),
        Some(Value::Vec2([11.0, 8.0]))
    );

    let tint = PropertyId::new("tint").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: tint.clone(),
        track: still(Value::Color([0.25, 0.25, 0.25, 0.25])),
    })
    .unwrap();
    let fill = PropertyId::new("fill").unwrap();
    doc.apply(Intent::SetTrack {
        layer: b,
        property: fill.clone(),
        track: still(Value::Color([0.25, 0.25, 0.25, 0.25])),
    })
    .unwrap();
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: fill.clone(),
        modulators: vec![identity_link(a, tint)],
    })
    .unwrap();
    assert_eq!(
        doc.view().value_at(b, &fill, t(0)).unwrap(),
        Some(Value::Color([0.5, 0.5, 0.5, 0.5])),
        "アルファも他の3成分と同じく和に含まれるはず(裁定213 の判断)"
    );
}

/// **`Bool`/`Enum`/`LayerId` は単一 source が勝つ**(補間が Hold の型は加算も
/// 無意味) — modulator が型不一致で寄与できず、base の値がそのまま残る。
#[test]
fn bool_enum_and_layer_id_properties_ignore_modulators() {
    let (mut doc, a, b) = doc_with_two_layers();
    let flip = PropertyId::new("flip").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: flip.clone(),
        track: still(Value::Bool(false)),
    })
    .unwrap();

    let visible = PropertyId::new("visible").unwrap();
    doc.apply(Intent::SetTrack {
        layer: b,
        property: visible.clone(),
        track: still(Value::Bool(true)),
    })
    .unwrap();
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: visible.clone(),
        modulators: vec![identity_link(a, flip)],
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(b, &visible, t(0)).unwrap(),
        Some(Value::Bool(true)),
        "Bool は加算できないので base(単一 source)が勝つはず"
    );
}

/// `Path` は頂点数(と `closed`)が一致する時だけ、頂点ごとの和(instance 軸は
/// 今は 1 に固定 — 全頂点を一括で足すだけで、特定の1頂点だけを狙う指定は無い)。
#[test]
fn path_modulators_sum_only_when_vertex_counts_match() {
    use motolii_eval::{Path, PathVertex};

    fn vertex(x: f64, y: f64) -> PathVertex {
        PathVertex {
            point: [x, y],
            in_tangent: [0.0, 0.0],
            out_tangent: [0.0, 0.0],
        }
    }

    let (mut doc, a, b) = doc_with_two_layers();
    let wiggle = PropertyId::new("wiggle").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: wiggle.clone(),
        track: still(Value::Path(Path {
            vertices: vec![vertex(1.0, 1.0), vertex(-1.0, -1.0)],
            closed: true,
        })),
    })
    .unwrap();

    let shape = PropertyId::new("shape").unwrap();
    doc.apply(Intent::SetTrack {
        layer: b,
        property: shape.clone(),
        track: still(Value::Path(Path {
            vertices: vec![vertex(0.0, 0.0), vertex(10.0, 10.0)],
            closed: true,
        })),
    })
    .unwrap();
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: shape.clone(),
        modulators: vec![identity_link(a, wiggle.clone())],
    })
    .unwrap();

    let Some(Value::Path(summed)) = doc.view().value_at(b, &shape, t(0)).unwrap() else {
        panic!("path が返らない");
    };
    assert_eq!(summed.vertices[0].point, [1.0, 1.0]);
    assert_eq!(summed.vertices[1].point, [9.0, 9.0]);

    // 頂点数が違う modulator は近似せず寄与ゼロ(base のまま)。
    let mismatched = PropertyId::new("mismatched").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: mismatched.clone(),
        track: still(Value::Path(Path {
            vertices: vec![vertex(5.0, 5.0)],
            closed: true,
        })),
    })
    .unwrap();
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: shape.clone(),
        modulators: vec![identity_link(a, mismatched)],
    })
    .unwrap();
    let Some(Value::Path(unaffected)) = doc.view().value_at(b, &shape, t(0)).unwrap() else {
        panic!("path が返らない");
    };
    assert_eq!(
        unaffected.vertices[0].point, [0.0, 0.0],
        "頂点数不一致の modulator が近似で寄与してしまっている"
    );
}

// ---------------------------------------------------------------------------
// 成分軸 — 既存の PropertyId split(position.x/position.y、裁定61)を再利用する
// ---------------------------------------------------------------------------

/// **成分だけを狙った変調**は、新しい「どの成分か」の指し方を発明せず、
/// 既存の split-position(`position.x`/`position.y`)へ modulator を付けるだけで
/// 表現できる——`resolve_position` が split を合成に使う既存の経路なので、
/// Y だけを LFO で揺らしつつ X はそのまま、という編集がここだけで成立する。
#[test]
fn a_modulator_can_target_a_single_component_via_the_existing_split_position_properties() {
    let (mut doc, a, b) = doc_with_two_layers();
    let wobble = PropertyId::new("wobble").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: wobble.clone(),
        track: still(Value::F64(5.0)),
    })
    .unwrap();

    let position_x = PropertyId::new(property::POSITION_X).unwrap();
    let position_y = PropertyId::new(property::POSITION_Y).unwrap();
    doc.apply(Intent::SetTrack {
        layer: b,
        property: position_x.clone(),
        track: still(Value::F64(100.0)),
    })
    .unwrap();
    doc.apply(Intent::SetTrack {
        layer: b,
        property: position_y.clone(),
        track: still(Value::F64(200.0)),
    })
    .unwrap();
    // Y だけに modulator を付ける — X は無傷のまま。
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: position_y.clone(),
        modulators: vec![identity_link(a, wobble)],
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(b, &position_x, t(0)).unwrap(),
        Some(Value::F64(100.0)),
        "X は modulator を付けていないので無傷のはず"
    );
    assert_eq!(
        doc.view().value_at(b, &position_y, t(0)).unwrap(),
        Some(Value::F64(205.0)),
        "Y だけ 200+5=205 になるはず"
    );

    // 実際に comp 空間へ合成される transform でも Y だけ動いていることを確認
    // (`resolve_position` が split を優先度どおりに合成する既存経路)。
    let resolved = doc.view().resolve(b, t(0)).unwrap().expect("居る");
    assert_eq!(resolved.placement.transform.translation.x, 100.0);
    assert_eq!(resolved.placement.transform.translation.y, 205.0);
}

// ---------------------------------------------------------------------------
// スケールが 0 を跨ぐ(反転) — world_affine は panic しない
// ---------------------------------------------------------------------------

/// scale が modulator で 0 を跨いでも(意味のある反転の表現)、この crate の
/// 合成(`resolve`/`world_affine`/`local_transform`)は前向きの行列合成だけで
/// `.inverse()` を一度も呼ばないので panic しない——`Mat2::inverse()` の自己
/// アサート地雷(`AGENTS.md`)は逆行列を取る側(`ui/motolii-stage-pane` の
/// gizmo/hit-test、write-set 外)の話であってここには無い、ことの直接固定。
#[test]
fn a_scale_modulator_crossing_zero_does_not_panic_the_forward_composition() {
    let (mut doc, a, b) = doc_with_two_layers();
    let invert = PropertyId::new("invert").unwrap();
    // t=0 で -2.0(scale 2.0 を打ち消して反転)、ちょうど 0 を跨ぐ入力。
    doc.apply(Intent::SetTrack {
        layer: a,
        property: invert.clone(),
        track: still(Value::Vec2([-2.0, -2.0])),
    })
    .unwrap();

    let scale = PropertyId::new(property::SCALE).unwrap();
    doc.apply(Intent::SetTrack {
        layer: b,
        property: scale.clone(),
        track: still(Value::Vec2([2.0, 2.0])),
    })
    .unwrap();
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: scale.clone(),
        modulators: vec![identity_link(a, invert)],
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(b, &scale, t(0)).unwrap(),
        Some(Value::Vec2([0.0, 0.0])),
        "2.0 + (-2.0) = 0.0 のはず(退化した scale)"
    );
    // panic せずに resolve できること自体がこのテストの主張。
    let resolved = doc.view().resolve(b, t(0)).unwrap().expect("居る");
    assert_eq!(resolved.placement.transform.matrix2.determinant(), 0.0);
}

// ---------------------------------------------------------------------------
// 負の time_offset(先読みの逆) — 既存の Hold 拡張がそのまま適用される
// ---------------------------------------------------------------------------

/// 負の `time_offset` は許可する。source_t が最初のキーフレームより前になっても、
/// `KeyframeTrack::eval` の既存の「先頭キーフレームより前は Hold」がそのまま
/// 適用される——新しい特別扱いは要らない。
#[test]
fn a_negative_time_offset_holds_at_the_first_keyframe_before_it() {
    let (mut doc, a, b) = doc_with_two_layers();
    let x = PropertyId::new("x").unwrap();
    let mut ramp = KeyframeTrack::new();
    ramp.insert(Keyframe {
        t: t(10),
        value: Value::F64(100.0),
        interp: Interp::Linear,
        spatial: None,
    });
    ramp.insert(Keyframe {
        t: t(20),
        value: Value::F64(200.0),
        interp: Interp::Linear,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer: a,
        property: x.clone(),
        track: ramp,
    })
    .unwrap();

    let y = PropertyId::new("y").unwrap();
    let minus_20_frames =
        RationalTime::try_from_frame(-20, Fps::try_new(30, 1).unwrap()).unwrap();
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: y.clone(),
        modulators: vec![PropertyLink {
            time_offset: minus_20_frames,
            ..identity_link(a, x)
        }],
    })
    .unwrap();

    // t(5) + (-20 frames) = frame -15、最初のキー(frame 10)より前 → Hold で 100.0。
    assert_eq!(
        doc.view().value_at(b, &y, t(5)).unwrap(),
        Some(Value::F64(100.0)),
        "負の time_offset で comp 開始前を読んでも先頭キーフレームで Hold するはず"
    );
}

// ---------------------------------------------------------------------------
// 循環・柵・base の保持・保存/読込
// ---------------------------------------------------------------------------

/// **循環は modulator 1本ごとに拒否する**——`Intent::SetPropertyLink` と同じ柵
/// (`validate_no_link_cycle`)を `SetPropertyModulators` にも掛ける。
#[test]
fn set_property_modulators_rejects_a_cycle() {
    let (mut doc, a, _b) = doc_with_two_layers();
    let prop = PropertyId::new("self_ref").unwrap();
    let result = doc.apply(Intent::SetPropertyModulators {
        layer: a,
        property: prop.clone(),
        modulators: vec![identity_link(a, prop)],
    });
    assert!(result.is_err(), "自己参照の modulator が通ってしまっている");
}

/// locked な layer への `SetPropertyModulators` は他の層変更 Intent と同じく拒む。
#[test]
fn locked_layer_rejects_set_property_modulators() {
    let (mut doc, a, b) = doc_with_two_layers();
    let source = PropertyId::new("source").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: source.clone(),
        track: still(Value::F64(1.0)),
    })
    .unwrap();
    doc.apply(Intent::SetAttrs {
        layer: b,
        patch: motolii_store::LayerAttrsPatch {
            locked: Some(true),
            ..Default::default()
        },
    })
    .unwrap();

    let target = PropertyId::new("target").unwrap();
    let result = doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: target,
        modulators: vec![identity_link(a, source)],
    });
    assert!(result.is_err(), "locked 層への SetPropertyModulators は拒まれるはず");
}

/// 空の `Vec` を渡せば modulator を全部外せる(専用の「解除」variant は要らない、
/// `SetTrack`/`SetPropertySlot` と同じ流儀)。
#[test]
fn passing_an_empty_vec_clears_all_modulators() {
    let (mut doc, a, b) = doc_with_two_layers();
    let bump = PropertyId::new("bump").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: bump.clone(),
        track: still(Value::F64(1.0)),
    })
    .unwrap();

    let opacity = PropertyId::new(property::OPACITY).unwrap();
    doc.apply(Intent::SetTrack {
        layer: b,
        property: opacity.clone(),
        track: still(Value::F64(0.5)),
    })
    .unwrap();
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: opacity.clone(),
        modulators: vec![identity_link(a, bump)],
    })
    .unwrap();
    assert_eq!(
        doc.view().value_at(b, &opacity, t(0)).unwrap(),
        Some(Value::F64(1.5)),
        "0.5 (base) + 1.0 (modulator) = 1.5 のはず"
    );

    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: opacity.clone(),
        modulators: Vec::new(),
    })
    .unwrap();
    assert_eq!(
        doc.view().value_at(b, &opacity, t(0)).unwrap(),
        Some(Value::F64(0.5)),
        "空の Vec で modulator が外れていない"
    );
}

/// カメラの property も modulator を持てる(`SetCameraTrack`/`SetCameraPropertySlot`
/// と entity を分けているのと同じ形の `Intent::SetCameraPropertyModulators`)。
#[test]
fn a_camera_property_can_also_be_modulated() {
    let (mut doc, a, _b) = doc_with_two_layers();
    let drift = PropertyId::new("drift").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: drift.clone(),
        track: still(Value::F64(0.5)),
    })
    .unwrap();

    let zoom = PropertyId::camera(property::CAMERA_ZOOM).unwrap();
    doc.apply(Intent::SetCameraTrack {
        property: zoom.clone(),
        track: still(Value::F64(1.0)),
    })
    .unwrap();
    doc.apply(Intent::SetCameraPropertyModulators {
        property: zoom.clone(),
        modulators: vec![identity_link(a, drift)],
    })
    .unwrap();

    assert_eq!(
        doc.view().camera_value_at(&zoom, t(0)).unwrap(),
        Some(Value::F64(1.5))
    );
}

/// base(`Slot` 参照)+ modulator の組み合わせも成立する——`base` は `Track`/`Slot`
/// のどちらでもよい、という設計どおり。
#[test]
fn a_modulator_can_sit_on_top_of_a_slot_base() {
    let (mut doc, a, b) = doc_with_two_layers();
    doc.apply(Intent::SetSlots {
        slots: vec![Slot {
            id: SlotId("brand_radius".to_owned()),
            track: still(Value::F64(10.0)),
        }],
    })
    .unwrap();

    let bump = PropertyId::new("bump").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: bump.clone(),
        track: still(Value::F64(2.0)),
    })
    .unwrap();

    let radius = PropertyId::new("radius").unwrap();
    doc.apply(Intent::SetPropertySlot {
        layer: b,
        property: radius.clone(),
        slot: SlotId("brand_radius".to_owned()),
    })
    .unwrap();
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: radius.clone(),
        modulators: vec![identity_link(a, bump)],
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(b, &radius, t(0)).unwrap(),
        Some(Value::F64(12.0)),
        "スロット由来の base(10.0) + modulator(2.0) = 12.0 のはず"
    );
}

/// base + modulators は保存/読込を往復する(裁定56 の畳む履歴でも消えない)。
#[test]
fn base_and_modulators_survive_a_save_and_load_round_trip() {
    let (mut doc, a, b) = doc_with_two_layers();
    let bump = PropertyId::new("bump").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: bump.clone(),
        track: still(Value::F64(3.0)),
    })
    .unwrap();

    let opacity = PropertyId::new(property::OPACITY).unwrap();
    doc.apply(Intent::SetTrack {
        layer: b,
        property: opacity.clone(),
        track: still(Value::F64(0.4)),
    })
    .unwrap();
    doc.apply(Intent::SetPropertyModulators {
        layer: b,
        property: opacity.clone(),
        modulators: vec![identity_link(a, bump)],
    })
    .unwrap();

    let dir = std::env::temp_dir().join(format!("motolii-modulator-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("modulator.rrd");
    doc.save(&file).unwrap();
    let loaded = Document::load(&file).unwrap();

    assert_eq!(
        loaded.view().value_at(b, &opacity, t(0)).unwrap(),
        Some(Value::F64(3.4)),
        "base(0.4) + modulator(3.0) が保存/読込で消えている"
    );
}
