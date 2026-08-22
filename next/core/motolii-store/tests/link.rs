//! `link` 発注単位(裁定206) — 型付き link、`PropertySource` の第三の枝。
//!
//! 設計は `docs/reviews/2026-08-22-persona-touchdesigner-round2.md` §1(TouchDesigner
//! CHOP Export / Blender Driver を先例とした設計案)そのもの。ここで固定するのは:
//!
//! - `value_at` が link を再帰的に解決する(§1.4)
//! - `time_offset` が評価時刻をずらす(§1.3)
//! - `plugin_id` の閉じた集合(identity/linear/remap)が正しく変換する
//! - **循環は書き込み時に拒否する**(§1.5、Blender の実行時検出より強い保証)——
//!   自己参照・多段の間接循環のどちらも拒む
//! - 旧 Document(`Track`/`Slot` のみ)が無改造で読める(移行コストゼロ、§1.6)
//! - `Intent::SetPropertyLink` は他の Intent と同じ単一 undo(`apply_all`、裁定48)
//! - 保存/読込を往復しても link が残る

use motolii_store::{
    Composition, Document, Fps, Intent, Interp, Keyframe, KeyframeTrack, LayerId, LayerMeta,
    LayerSource, LayerTiming, PropertyId, PropertyLink, PropertySource, RationalTime, Value,
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

fn ramp(from: Value, to: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value: from,
        interp: Interp::Linear,
        spatial: None,
    });
    track.insert(Keyframe {
        t: t(30),
        value: to,
        interp: Interp::Linear,
        spatial: None,
    });
    track
}

/// comp と layer を2つ置いた Document(A→B の link を試すのに要る最小構成)。
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

fn identity_link(source_layer: LayerId, source_property: PropertyId) -> PropertyLink {
    PropertyLink {
        source_layer,
        source_property,
        time_offset: RationalTime::ZERO,
        plugin_id: "motolii.link.identity".to_owned(),
        params: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// 解決値 — value_at が link を再帰的に解決する
// ---------------------------------------------------------------------------

/// **本題**: B の property を A の property へ link すると、`value_at` は A の
/// 評価済みの値をそのまま返す(TD の CHOP Export と同じ「宛先だけが参照を持つ」
/// 非対称な片方向束縛)。
#[test]
fn a_property_bound_to_a_link_resolves_the_source_layers_value() {
    let (mut doc, a, b) = doc_with_two_layers();
    let rotation = PropertyId::new("rotation").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: rotation.clone(),
        track: still(Value::F64(45.0)),
    })
    .unwrap();

    let opacity = PropertyId::new(motolii_store::property::OPACITY).unwrap();
    doc.apply(Intent::SetPropertyLink {
        layer: b,
        property: opacity.clone(),
        link: identity_link(a, rotation),
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(b, &opacity, t(0)).unwrap(),
        Some(Value::F64(45.0)),
        "link が解決されていない"
    );
}

/// ソースがアニメーションしていれば、link 先も同じ補間結果をそのまま見る
/// (スロット共有と同じ「評価は1本」の規律)。
#[test]
fn a_link_follows_the_sources_animation() {
    let (mut doc, a, b) = doc_with_two_layers();
    let x = PropertyId::new("x").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: x.clone(),
        track: ramp(Value::F64(0.0), Value::F64(30.0)),
    })
    .unwrap();

    let y = PropertyId::new("y").unwrap();
    doc.apply(Intent::SetPropertyLink {
        layer: b,
        property: y.clone(),
        link: identity_link(a, x),
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(b, &y, t(15)).unwrap(),
        Some(Value::F64(15.0))
    );
}

/// ソースにまだ値が無ければ(track 未設定)、link 先も「値が無い」(裁定20の応用 —
/// ぶら下がったスロット参照と同じ扱い)。
#[test]
fn a_link_to_an_unset_source_property_resolves_to_none() {
    let (mut doc, a, b) = doc_with_two_layers();
    let ghost = PropertyId::new("ghost").unwrap();
    let target = PropertyId::new("target").unwrap();
    doc.apply(Intent::SetPropertyLink {
        layer: b,
        property: target.clone(),
        link: identity_link(a, ghost),
    })
    .unwrap();

    assert_eq!(doc.view().value_at(b, &target, t(0)).unwrap(), None);
}

/// **`track()` は生の意味だけを返す**: link 先の property は「この property 自身の
/// track」を持たない(`Slot` と同じ非対称、`tests/slot.rs` の対になる試験)。
#[test]
fn track_returns_none_for_a_link_bound_property_even_though_value_at_resolves() {
    let (mut doc, a, b) = doc_with_two_layers();
    let source = PropertyId::new("source_prop").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: source.clone(),
        track: still(Value::F64(9.0)),
    })
    .unwrap();
    let target = PropertyId::new("target_prop").unwrap();
    doc.apply(Intent::SetPropertyLink {
        layer: b,
        property: target.clone(),
        link: identity_link(a, source),
    })
    .unwrap();

    assert_eq!(doc.view().track(b, &target).unwrap(), None);
    assert_eq!(
        doc.view().value_at(b, &target, t(0)).unwrap(),
        Some(Value::F64(9.0))
    );
    assert!(matches!(
        doc.view().property_source(b, &target).unwrap(),
        Some(PropertySource::Link(_))
    ));
}

/// `SetTrack` を再び投げれば、link から普通の track へ戻せる — 同じ component を
/// 上書きするだけなので、専用の「解除」操作を持たない(`tests/slot.rs` と同型)。
#[test]
fn set_track_after_set_property_link_switches_back_to_a_track() {
    let (mut doc, a, b) = doc_with_two_layers();
    let source = PropertyId::new("source_prop").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: source.clone(),
        track: still(Value::F64(1.0)),
    })
    .unwrap();
    let target = PropertyId::new("target_prop").unwrap();
    doc.apply(Intent::SetPropertyLink {
        layer: b,
        property: target.clone(),
        link: identity_link(a, source),
    })
    .unwrap();
    assert_eq!(
        doc.view().value_at(b, &target, t(0)).unwrap(),
        Some(Value::F64(1.0))
    );

    doc.apply(Intent::SetTrack {
        layer: b,
        property: target.clone(),
        track: still(Value::F64(99.0)),
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(b, &target, t(0)).unwrap(),
        Some(Value::F64(99.0)),
        "SetTrack の後も link のままになっている"
    );
    assert!(matches!(
        doc.view().property_source(b, &target).unwrap(),
        Some(PropertySource::Track(_))
    ));
}

// ---------------------------------------------------------------------------
// time_offset
// ---------------------------------------------------------------------------

/// `time_offset` は評価時刻をずらす——B が frame 10 に居る時、A の frame 15 を読む
/// (`time_offset = +5 frames`)。
#[test]
fn time_offset_shifts_the_evaluation_time_of_the_source() {
    let (mut doc, a, b) = doc_with_two_layers();
    let x = PropertyId::new("x").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: x.clone(),
        track: ramp(Value::F64(0.0), Value::F64(30.0)),
    })
    .unwrap();

    let y = PropertyId::new("y").unwrap();
    let five_frames = RationalTime::try_from_frame(5, Fps::try_new(30, 1).unwrap()).unwrap();
    doc.apply(Intent::SetPropertyLink {
        layer: b,
        property: y.clone(),
        link: PropertyLink {
            time_offset: five_frames,
            ..identity_link(a, x)
        },
    })
    .unwrap();

    // frame 10 + offset 5 = frame 15 → A の値は 15.0(ramp が frame 0..30 で 0..30)。
    assert_eq!(
        doc.view().value_at(b, &y, t(10)).unwrap(),
        Some(Value::F64(15.0))
    );
}

// ---------------------------------------------------------------------------
// plugin_id の閉じた集合(translate_link は slot.rs の単体テストで詳細に固定済み。
// ここでは Document を通した end-to-end 経路だけ確かめる)
// ---------------------------------------------------------------------------

#[test]
fn linear_plugin_id_scales_and_offsets_through_the_document() {
    let (mut doc, a, b) = doc_with_two_layers();
    let x = PropertyId::new("x").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: x.clone(),
        track: still(Value::F64(10.0)),
    })
    .unwrap();

    let y = PropertyId::new("y").unwrap();
    doc.apply(Intent::SetPropertyLink {
        layer: b,
        property: y.clone(),
        link: PropertyLink {
            plugin_id: "motolii.link.linear".to_owned(),
            params: vec![
                ("scale".to_owned(), Value::F64(3.0)),
                ("offset".to_owned(), Value::F64(1.0)),
            ],
            ..identity_link(a, x)
        },
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(b, &y, t(0)).unwrap(),
        Some(Value::F64(31.0)),
        "10*3+1 = 31 のはず"
    );
}

// ---------------------------------------------------------------------------
// 循環 — 書き込み時に拒否する(§1.5)
// ---------------------------------------------------------------------------

/// **自己参照**: layer 自身の同じ property を指す link は、参照鎖の長さ0で
/// 即座に自分へ戻る——最も単純な循環。
#[test]
fn a_property_cannot_link_to_itself() {
    let (mut doc, a, _b) = doc_with_two_layers();
    let prop = PropertyId::new("self_ref").unwrap();
    let result = doc.apply(Intent::SetPropertyLink {
        layer: a,
        property: prop.clone(),
        link: identity_link(a, prop),
    });
    assert!(result.is_err(), "自己参照の link が通ってしまっている");
}

/// **多段の間接循環**: A.p1 → B.p2 → A.p1 のような2段の鎖も拒む
/// (`validate_no_parent_cycle` の `(LayerId, PropertyId)` 版であることの直接証拠)。
#[test]
fn an_indirect_link_cycle_across_two_layers_is_rejected() {
    let (mut doc, a, b) = doc_with_two_layers();
    let p1 = PropertyId::new("p1").unwrap();
    let p2 = PropertyId::new("p2").unwrap();

    // まず A.p1 → B.p2 を作る(この時点では循環していない — B.p2 はまだ何も指さない)。
    doc.apply(Intent::SetPropertyLink {
        layer: a,
        property: p1.clone(),
        link: identity_link(b, p2.clone()),
    })
    .unwrap();

    // 次に B.p2 → A.p1 を作ろうとすると、鎖が A.p1 へ戻ってくるので拒まれるはず。
    let result = doc.apply(Intent::SetPropertyLink {
        layer: b,
        property: p2,
        link: identity_link(a, p1),
    });
    assert!(
        result.is_err(),
        "A→B→A の間接循環が通ってしまっている"
    );
}

/// 循環を拒んでも、**それ以外の正当な link は影響を受けない** — 拒否は
/// 書き込みそのものを止めるだけで、既存の Document は変化しない。
#[test]
fn rejecting_a_cycle_does_not_corrupt_the_document() {
    let (mut doc, a, b) = doc_with_two_layers();
    let source = PropertyId::new("source_prop").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: source.clone(),
        track: still(Value::F64(42.0)),
    })
    .unwrap();
    let target = PropertyId::new("target_prop").unwrap();
    doc.apply(Intent::SetPropertyLink {
        layer: b,
        property: target.clone(),
        link: identity_link(a, source.clone()),
    })
    .unwrap();

    // 循環になる書き込みを試みて拒まれる。
    assert!(doc
        .apply(Intent::SetPropertyLink {
            layer: a,
            property: source.clone(),
            link: identity_link(b, target.clone()),
        })
        .is_err());

    // 元の link はそのまま生きている。
    assert_eq!(
        doc.view().value_at(b, &target, t(0)).unwrap(),
        Some(Value::F64(42.0)),
        "拒否された書き込みが既存の link を壊している"
    );
}

// ---------------------------------------------------------------------------
// undo — 他の Intent と同じ単一 undo(裁定48)
// ---------------------------------------------------------------------------

#[test]
fn setting_a_link_is_undoable_like_any_other_edit() {
    let (mut doc, a, b) = doc_with_two_layers();
    let source = PropertyId::new("source_prop").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: source.clone(),
        track: still(Value::F64(1.0)),
    })
    .unwrap();
    let target = PropertyId::new("target_prop").unwrap();
    doc.apply(Intent::SetPropertyLink {
        layer: b,
        property: target.clone(),
        link: identity_link(a, source),
    })
    .unwrap();
    assert_eq!(
        doc.view().value_at(b, &target, t(0)).unwrap(),
        Some(Value::F64(1.0))
    );

    assert!(doc.undo());
    assert_eq!(
        doc.view().value_at(b, &target, t(0)).unwrap(),
        None,
        "link の設定が undo で戻らない"
    );
    assert!(doc.redo());
    assert_eq!(
        doc.view().value_at(b, &target, t(0)).unwrap(),
        Some(Value::F64(1.0)),
        "link の設定が redo で戻らない"
    );
}

// ---------------------------------------------------------------------------
// 保存/読込 — 移行コストゼロ(§1.6)・後方互換
// ---------------------------------------------------------------------------

/// **後方互換の核**: link を1つも使っていない旧 Document(`Track`/`Slot` のみ)は、
/// この機能追加の前後で無改造で読める——`#[serde(untagged)]` の3択目を足しても、
/// 既存の保存済み JSON は1バイトも変わらない。
#[test]
fn documents_without_any_link_still_load_after_adding_the_link_variant() {
    let (mut doc, a, b) = doc_with_two_layers();
    let opacity = PropertyId::new(motolii_store::property::OPACITY).unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: opacity.clone(),
        track: still(Value::F64(0.8)),
    })
    .unwrap();
    let slot_prop = PropertyId::new("fill").unwrap();
    doc.apply(Intent::SetSlots {
        slots: vec![motolii_store::Slot {
            id: motolii_store::SlotId("brand".to_owned()),
            track: still(Value::Color([0.1, 0.2, 0.3, 1.0])),
        }],
    })
    .unwrap();
    doc.apply(Intent::SetPropertySlot {
        layer: b,
        property: slot_prop.clone(),
        slot: motolii_store::SlotId("brand".to_owned()),
    })
    .unwrap();

    let dir = std::env::temp_dir().join(format!("motolii-link-back-compat-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("no-link.rrd");
    doc.save(&file).unwrap();
    let loaded = Document::load(&file).unwrap();

    assert_eq!(
        loaded.view().value_at(a, &opacity, t(0)).unwrap(),
        Some(Value::F64(0.8)),
        "Link 変種の追加で既存の Track が壊れている"
    );
    assert_eq!(
        loaded.view().value_at(b, &slot_prop, t(0)).unwrap(),
        Some(Value::Color([0.1, 0.2, 0.3, 1.0])),
        "Link 変種の追加で既存の Slot 参照が壊れている"
    );
}

/// link を実際に使った Document も保存/読込を往復する。
#[test]
fn a_link_survives_a_save_and_load_round_trip() {
    let (mut doc, a, b) = doc_with_two_layers();
    let source = PropertyId::new("source_prop").unwrap();
    doc.apply(Intent::SetTrack {
        layer: a,
        property: source.clone(),
        track: still(Value::F64(7.0)),
    })
    .unwrap();
    let target = PropertyId::new("target_prop").unwrap();
    doc.apply(Intent::SetPropertyLink {
        layer: b,
        property: target.clone(),
        link: PropertyLink {
            plugin_id: "motolii.link.linear".to_owned(),
            params: vec![("scale".to_owned(), Value::F64(2.0))],
            ..identity_link(a, source)
        },
    })
    .unwrap();

    let dir = std::env::temp_dir().join(format!("motolii-link-round-trip-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("link.rrd");
    doc.save(&file).unwrap();
    let loaded = Document::load(&file).unwrap();

    assert_eq!(
        loaded.view().value_at(b, &target, t(0)).unwrap(),
        Some(Value::F64(14.0)),
        "保存で畳む時に link が落ちている、または変換が消えている"
    );
    assert!(matches!(
        loaded.view().property_source(b, &target).unwrap(),
        Some(PropertySource::Link(_))
    ));
}

// ---------------------------------------------------------------------------
// locked/frozen — 他の層変更 Intent と同じ柵
// ---------------------------------------------------------------------------

#[test]
fn locked_layer_rejects_set_property_link() {
    let (mut doc, a, b) = doc_with_two_layers();
    let source = PropertyId::new("source_prop").unwrap();
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

    let target = PropertyId::new("target_prop").unwrap();
    let result = doc.apply(Intent::SetPropertyLink {
        layer: b,
        property: target,
        link: identity_link(a, source),
    });
    assert!(result.is_err(), "locked 層への SetPropertyLink は拒まれるはず");
}
