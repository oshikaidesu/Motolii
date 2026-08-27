//! effect 発注単位(10行) — `effects/effect ef`/`en`/`ty` + `effect-values/*` 7行。
//!
//! ここで固定するのは:
//! - `ty`(Type)は plugin id 文字列であって閉じた int registry ではない(裁定70)。
//!   first-party と third-party が同じ列に同居できる
//! - **`en`(Enabled)は裁定213でキーフレームへ移した** — `EffectInstance` はもう
//!   `enabled` フィールドを持たず、`PropertyId::effect_enabled` の平坦 track
//!   (`Value::Bool`、Hold 補間)で表す。無効化しても `SetEffects` の列からは
//!   消えない(「消す」と「一時的に切る」は別物のまま)
//! - param(`ef`)は Document が専用フィールドで持たず、`effect.{id}.param.{name}` と
//!   いう平坦 `PropertyId` の `KeyframeTrack` にそのまま乗る(裁定72「新機構ゼロ」)。
//!   track の有無が「この param を触っているか」を表す(裁定20 の応用)
//! - param の型語彙(scalar/Bool/Enum/color/Vec2/LayerId、effect-values の catalog)は
//!   `motolii_eval::Value` の既存/新設バリアントがそのまま担う
//! - `resolve()` は effect stack を運ぶ(裁定153 S1)。**enabled な effect だけ**が
//!   宣言順で並び、param は時刻 t で評価済み。disabled/track の無い param は現れない

use motolii_store::{
    Composition, Document, EffectId, EffectInstance, Fps, Interp, Intent, Keyframe,
    KeyframeTrack, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value,
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

fn doc_with_layer() -> (Document, LayerId) {
    let mut doc = Document::new();
    let layer = LayerId(1);
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
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
                order: 0,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .unwrap();
    (doc, layer)
}

// ---------------------------------------------------------------------------
// ty (Type) — plugin id 文字列
// ---------------------------------------------------------------------------

/// **`ty` は plugin id 文字列**(裁定70)であって閉じた int registry ではない —
/// first-party / third-party が同じ列に同居できることの最小固定(D6)。
#[test]
fn effect_type_is_a_plugin_id_string_shared_by_first_and_third_party() {
    let (mut doc, layer) = doc_with_layer();
    let effects = vec![
        EffectInstance {
            id: EffectId(0),
            plugin_id: "motolii.gaussian-blur".to_owned(),
        },
        EffectInstance {
            id: EffectId(1),
            plugin_id: "thirdparty.acme.glow".to_owned(),
        },
    ];
    doc.apply(Intent::SetEffects {
        layer,
        effects: effects.clone(),
    })
    .unwrap();
    assert_eq!(
        doc.view().effects(layer).unwrap(),
        effects,
        "first/third-party が同じ列へ乗らない"
    );
}

// ---------------------------------------------------------------------------
// en (Enabled) — 裁定213: キーフレーム可能な track へ。消さずに切る
// ---------------------------------------------------------------------------

/// **`en` は消さずに切る。** 無効化(= `effect.{id}.enabled` の track に
/// `Value::Bool(false)` を書く)しても `SetEffects` の列からは消えない ——
/// 列から取り除く(削除)とは別の操作、という既存の意図(裁定213 でも変えない)。
#[test]
fn disabling_an_effect_keeps_it_in_the_list() {
    let (mut doc, layer) = doc_with_layer();
    let effect = EffectId(0);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: effect,
            plugin_id: "motolii.gaussian-blur".to_owned(),
        }],
    })
    .unwrap();

    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::effect_enabled(effect),
        track: still(Value::Bool(false)),
    })
    .unwrap();

    let effects = doc.view().effects(layer).unwrap();
    assert_eq!(effects.len(), 1, "無効化で列から消えてしまっている");
    assert_eq!(
        doc.view().resolve(layer, t(0)).unwrap().expect("居る").effects.len(),
        0,
        "disabled な effect が resolve() に運ばれている"
    );
}

/// **キーを打っていない = 既定で有効**(`mask_opacity` の「既定 1.0」と同じ判断) —
/// effect を追加した直後は何もしなくても効いているはず、という利用者の直感。
#[test]
fn an_effect_with_no_enabled_track_defaults_to_enabled() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: EffectId(0),
            plugin_id: "motolii.gaussian-blur".to_owned(),
        }],
    })
    .unwrap();

    assert_eq!(
        doc.view().resolve(layer, t(0)).unwrap().expect("居る").effects.len(),
        1,
        "enabled track を一度も書いていない effect が既定で無効になっている"
    );
}

/// **`Value::Bool` の Hold 補間がそのまま on/off の意味になる** — 裁定213 の
/// 核心。1本の track に複数キーを打てば、comp 時刻によって effect が
/// 途中で切れたり戻ったりする(「一時的に切る」がキーフレーム可能になった)。
#[test]
fn enabling_and_disabling_can_be_keyframed_across_time() {
    let (mut doc, layer) = doc_with_layer();
    let effect = EffectId(0);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: effect,
            plugin_id: "motolii.gaussian-blur".to_owned(),
        }],
    })
    .unwrap();

    let mut enabled_track = KeyframeTrack::new();
    enabled_track.insert(Keyframe {
        t: t(0),
        value: Value::Bool(true),
        interp: Interp::Hold,
        spatial: None,
    });
    enabled_track.insert(Keyframe {
        t: t(15),
        value: Value::Bool(false),
        interp: Interp::Hold,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::effect_enabled(effect),
        track: enabled_track,
    })
    .unwrap();

    assert_eq!(
        doc.view().resolve(layer, t(0)).unwrap().expect("居る").effects.len(),
        1,
        "frame 0 では有効なはず"
    );
    assert_eq!(
        doc.view().resolve(layer, t(10)).unwrap().expect("居る").effects.len(),
        1,
        "Hold 補間なので次のキーまでは前の値(有効)を保つはず"
    );
    assert_eq!(
        doc.view().resolve(layer, t(20)).unwrap().expect("居る").effects.len(),
        0,
        "frame 15 以降は無効なはず"
    );
}

// ---------------------------------------------------------------------------
// ef (Effect Values) — param は平坦 PropertyId の KeyframeTrack
// ---------------------------------------------------------------------------

/// param は**専用フィールドを持たない** — track が無ければ、その param には
/// 単に触っていない(裁定20 の応用)。`PropertyId` の構築自体は effect の存在を検査しない
/// (`SetTrack` を投げる時点で store が普通に検証する、mask/text-range と同じ形)。
#[test]
fn a_param_with_no_track_is_simply_absent() {
    let (doc, layer) = doc_with_layer();
    let radius = PropertyId::effect_param(EffectId(0), "radius").unwrap();
    assert_eq!(radius.name(), "effect.0.param.radius");
    assert_eq!(doc.view().value_at(layer, &radius, t(0)).unwrap(), None);
}

/// スライダー/角度パラメータ(effect-values/slider, effect-values/angle)は
/// **F64 スカラー**。既存の scalar track とまったく同じ経路(新機構ゼロ、裁定72)。
#[test]
fn slider_and_angle_params_are_plain_scalar_tracks() {
    let (mut doc, layer) = doc_with_layer();
    let effect = EffectId(0);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: effect,
            plugin_id: "motolii.gaussian-blur".to_owned(),
        }],
    })
    .unwrap();

    let radius = PropertyId::effect_param(effect, "radius").unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: radius.clone(),
        track: still(Value::F64(12.5)),
    })
    .unwrap();
    assert_eq!(
        doc.view().value_at(layer, &radius, t(0)).unwrap(),
        Some(Value::F64(12.5))
    );

    let direction = PropertyId::effect_param(effect, "direction").unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: direction.clone(),
        track: still(Value::F64(90.0)),
    })
    .unwrap();
    assert_eq!(
        doc.view().value_at(layer, &direction, t(0)).unwrap(),
        Some(Value::F64(90.0)),
        "angle param(角度の意味域)が数値として乗らない"
    );
}

/// checkbox param(effect-values/checkbox)は **`Value::Bool`**。int-boolean を
/// そのまま持たない(裁定78/96 と同じ理由をここでも守る — 旧 ValueType に無かった穴)。
#[test]
fn checkbox_param_is_bool_not_int_boolean() {
    let (mut doc, layer) = doc_with_layer();
    let effect = EffectId(0);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: effect,
            plugin_id: "motolii.drop-shadow".to_owned(),
        }],
    })
    .unwrap();

    let use_layer_color = PropertyId::effect_param(effect, "use_layer_color").unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: use_layer_color.clone(),
        track: still(Value::Bool(true)),
    })
    .unwrap();
    assert_eq!(
        doc.view().value_at(layer, &use_layer_color, t(0)).unwrap(),
        Some(Value::Bool(true))
    );
}

/// color param(effect-values/color)は `Value::Color`(M2E-13 と同じ RGBA straight-alpha)。
#[test]
fn color_param_round_trips() {
    let (mut doc, layer) = doc_with_layer();
    let effect = EffectId(0);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: effect,
            plugin_id: "motolii.drop-shadow".to_owned(),
        }],
    })
    .unwrap();

    let shadow_color = PropertyId::effect_param(effect, "color").unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: shadow_color.clone(),
        track: still(Value::Color([0.0, 0.0, 0.0, 0.75])),
    })
    .unwrap();
    assert_eq!(
        doc.view().value_at(layer, &shadow_color, t(0)).unwrap(),
        Some(Value::Color([0.0, 0.0, 0.0, 0.75]))
    );
}

/// point param(effect-values/point)は `Value::Vec2`。
#[test]
fn point_param_round_trips() {
    let (mut doc, layer) = doc_with_layer();
    let effect = EffectId(0);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: effect,
            plugin_id: "thirdparty.acme.displace".to_owned(),
        }],
    })
    .unwrap();

    let center = PropertyId::effect_param(effect, "center").unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: center.clone(),
        track: still(Value::Vec2([32.0, 18.0])),
    })
    .unwrap();
    assert_eq!(
        doc.view().value_at(layer, &center, t(0)).unwrap(),
        Some(Value::Vec2([32.0, 18.0]))
    );
}

/// drop-down param(effect-values/drop-down)は `Value::Enum` — plugin が定義する
/// 選択肢表への index。**中間状態は無意味なので Hold**(0.5 番目の選択肢は存在しない、
/// `motolii_eval::Value` 側で保証済み — ここでは store 経路を通しても崩れないことを見る)。
#[test]
fn drop_down_param_is_an_enum_index_and_holds_between_keys() {
    let (mut doc, layer) = doc_with_layer();
    let effect = EffectId(0);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: effect,
            plugin_id: "motolii.blend-mode-fx".to_owned(),
        }],
    })
    .unwrap();

    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value: Value::Enum(0),
        interp: Interp::Linear,
        spatial: None,
    });
    track.insert(Keyframe {
        t: t(30),
        value: Value::Enum(3),
        interp: Interp::Linear,
        spatial: None,
    });
    let mode = PropertyId::effect_param(effect, "mode").unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: mode.clone(),
        track,
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(layer, &mode, t(15)).unwrap(),
        Some(Value::Enum(0)),
        "選択肢 index が中間で補間されてしまっている(存在しない選択肢が作れる)"
    );
    assert_eq!(
        doc.view().value_at(layer, &mode, t(30)).unwrap(),
        Some(Value::Enum(3))
    );
}

/// layer param(effect-values/layer)は **`Value::LayerId`** — Lottie の `ind`
/// (挿入削除で崩れる index)ではなく安定 id を参照する(裁定65)。
#[test]
fn layer_param_references_a_stable_layer_id_not_an_index() {
    let (mut doc, layer) = doc_with_layer();
    let target = LayerId(7);
    let effect = EffectId(0);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: effect,
            plugin_id: "thirdparty.acme.displace".to_owned(),
        }],
    })
    .unwrap();

    let map_layer = PropertyId::effect_param(effect, "map_layer").unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: map_layer.clone(),
        track: still(Value::LayerId(target.0)),
    })
    .unwrap();
    assert_eq!(
        doc.view().value_at(layer, &map_layer, t(0)).unwrap(),
        Some(Value::LayerId(7))
    );
}

// ---------------------------------------------------------------------------
// 横断: property 一覧・保存/読込・同一性検査
// ---------------------------------------------------------------------------

/// property 一覧は**store に聞く**(裁定57)。effect の param track もそこに並ぶ。
#[test]
fn effect_param_tracks_appear_in_the_property_list() {
    let (mut doc, layer) = doc_with_layer();
    let effect = EffectId(2);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: effect,
            plugin_id: "motolii.gaussian-blur".to_owned(),
        }],
    })
    .unwrap();
    let radius = PropertyId::effect_param(effect, "radius").unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: radius,
        track: still(Value::F64(4.0)),
    })
    .unwrap();

    let names: Vec<String> = doc
        .view()
        .properties(layer)
        .iter()
        .map(|p| p.name().to_owned())
        .collect();
    assert!(
        names.contains(&"effect.2.param.radius".to_owned()),
        "param track が property 一覧に出ていない: {names:?}"
    );
}

/// **保存/読込を経ても effect の列(id・plugin id)と enabled track・param track が
/// 全部残る**(裁定56 が畳む履歴の中で、新設フィールドが黙って消えないことの固定)。
#[test]
fn effects_and_their_tracks_survive_a_save_and_load_round_trip() {
    let (mut doc, layer) = doc_with_layer();
    let effect = EffectId(3);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: effect,
            plugin_id: "motolii.gaussian-blur".to_owned(),
        }],
    })
    .unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::effect_enabled(effect),
        track: still(Value::Bool(false)),
    })
    .unwrap();
    let radius = PropertyId::effect_param(effect, "radius").unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: radius.clone(),
        track: still(Value::F64(9.0)),
    })
    .unwrap();

    let dir = std::env::temp_dir().join(format!("motolii-effect-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("effect.rrd");
    doc.save(&file).unwrap();
    let loaded = Document::load(&file).unwrap();

    let effects = loaded.view().effects(layer).unwrap();
    assert_eq!(effects.len(), 1);
    assert_eq!(
        loaded.view().resolve(layer, t(0)).unwrap().expect("居る").effects.len(),
        0,
        "enabled=false が保存で消えている/戻っている"
    );
    assert_eq!(
        loaded.view().value_at(layer, &radius, t(0)).unwrap(),
        Some(Value::F64(9.0)),
        "param track が保存で消えている"
    );
}

/// 同じ id の effect が2枚あると param track の持ち主が決まらない
/// (mask と同型の検査。`enabled` が track へ移っても id の検査は変わらない)。
#[test]
fn duplicate_effect_ids_are_still_rejected() {
    let (mut doc, layer) = doc_with_layer();
    let result = doc.apply(Intent::SetEffects {
        layer,
        effects: vec![
            EffectInstance {
                id: EffectId(0),
                plugin_id: "a".to_owned(),
            },
            EffectInstance {
                id: EffectId(0),
                plugin_id: "b".to_owned(),
            },
        ],
    });
    assert!(result.is_err(), "同じ id の effect が2枚置けてしまっている");
}

// ---------------------------------------------------------------------------
// resolve() が effect stack を運ぶ(裁定153 S1)
// ---------------------------------------------------------------------------

/// **effect を積んだ layer の resolve 結果に評価済み effect 列が現れる。**
/// `resolve()` が `effects()`/param track を一度も読まなかった旧状態
/// (2026-08-21 縫い目調査)の直接の固定。
#[test]
fn resolve_carries_the_enabled_effect_stack_with_evaluated_params() {
    let (mut doc, layer) = doc_with_layer();
    let effect = EffectId(0);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: effect,
            plugin_id: "motolii.gaussian-blur".to_owned(),
        }],
    })
    .unwrap();
    let radius = PropertyId::effect_param(effect, "radius").unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: radius,
        track: still(Value::F64(12.5)),
    })
    .unwrap();

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert_eq!(
        resolved.effects.len(),
        1,
        "effect が resolve() に運ばれていない: {:?}",
        resolved.effects
    );
    assert_eq!(resolved.effects[0].plugin_id, "motolii.gaussian-blur");
    assert_eq!(
        resolved.effects[0].params,
        vec![("radius".to_owned(), Value::F64(12.5))]
    );
}

/// **disabled は現れない。** 列から消えていない(`en` は「消さずに切る」)ことは
/// `effects()` 側で既に固定済みだが、resolve() の出力側は disabled をそもそも運ばない
/// — hidden な layer が `resolve()` の外へ落ちるのと同じ形。
#[test]
fn resolve_does_not_carry_disabled_effects() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![
            EffectInstance {
                id: EffectId(0),
                plugin_id: "motolii.gaussian-blur".to_owned(),
            },
            EffectInstance {
                id: EffectId(1),
                plugin_id: "motolii.drop-shadow".to_owned(),
            },
        ],
    })
    .unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::effect_enabled(EffectId(0)),
        track: still(Value::Bool(false)),
    })
    .unwrap();

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert_eq!(
        resolved.effects.len(),
        1,
        "disabled な effect が resolve() に現れている: {:?}",
        resolved.effects
    );
    assert_eq!(resolved.effects[0].plugin_id, "motolii.drop-shadow");
}

/// **param のキーフレームが時刻で効く。** 平坦 track の既存評価経路
/// (`StoreView::value_at`)をそのまま再利用していることの固定 — 新しい評価器を
/// 書いていれば補間の挙動がここでずれる。
#[test]
fn resolved_effect_params_interpolate_with_time() {
    let (mut doc, layer) = doc_with_layer();
    let effect = EffectId(0);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: effect,
            plugin_id: "motolii.gaussian-blur".to_owned(),
        }],
    })
    .unwrap();

    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value: Value::F64(0.0),
        interp: Interp::Linear,
        spatial: None,
    });
    track.insert(Keyframe {
        t: t(30),
        value: Value::F64(30.0),
        interp: Interp::Linear,
        spatial: None,
    });
    let radius = PropertyId::effect_param(effect, "radius").unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: radius,
        track,
    })
    .unwrap();

    let at_start = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert_eq!(
        at_start.effects[0].params,
        vec![("radius".to_owned(), Value::F64(0.0))]
    );

    let mid = doc.view().resolve(layer, t(15)).unwrap().expect("居る");
    assert_eq!(
        mid.effects[0].params,
        vec![("radius".to_owned(), Value::F64(15.0))],
        "param のキーフレームが時刻で効いていない: {:?}",
        mid.effects[0].params
    );
}

/// track の無い param は effect が有効でも運ばれない — store は plugin の param
/// カタログを知らない(裁定70)ので、既定値を埋めるのは呼び手の役目。
#[test]
fn resolved_effect_omits_params_with_no_track() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: EffectId(0),
            plugin_id: "motolii.gaussian-blur".to_owned(),
        }],
    })
    .unwrap();

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert_eq!(resolved.effects.len(), 1, "enabled な effect 自体は現れるべき");
    assert!(
        resolved.effects[0].params.is_empty(),
        "触っていない param が既定値で埋まっている: {:?}",
        resolved.effects[0].params
    );
}

/// effect を持たない layer は空スタック。
#[test]
fn resolve_yields_an_empty_effect_stack_when_none_are_set() {
    let (doc, layer) = doc_with_layer();
    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert!(resolved.effects.is_empty());
}
