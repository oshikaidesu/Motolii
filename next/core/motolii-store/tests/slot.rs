//! `slot` 発注単位(4行) — `composition/animation/slots`(表)/ `helpers/slot p`
//! (スロットが差し込む値)/ `helpers/slottable-object sid`(参照識別子)/
//! `properties/property sid`(property 側の口)。
//!
//! ここで固定するのは:
//! - comp 水準の Slots 表は `Intent::SetSlots`(丸ごと差し替え、`SetMasks`/`SetMarkers`
//!   と同じ形)で書き、`view.slots()` で読む。並びは編集順(mask/effect と同じ流儀)
//! - 同じ id のスロットが2枚あると `SetSlots` が拒む(mask/effect と同型の検査)
//! - property の値の出処は `PropertySource::{Track,Slot}` の2択
//!   (`properties/property sid` の note どおり)。**新しい component は増やさない** —
//!   `SetTrack`/`SetPropertySlot` はどちらも同じ `descriptor_track(property)` の
//!   component を書く。`Track` の wire 形は今までの `KeyframeTrack` の JSON と
//!   ビット単位で同じ(`#[serde(untagged)]`)なので、この束の前後で既存の
//!   保存済み track が1つも壊れない
//! - `track()`/`camera_track()` は Slot 参照を「track 無し」として `None` を返す
//!   (生の意味だけを返す約束、評価はしない) — `value_at()`/`camera_value_at()` は
//!   スロット参照を実際に解決した値を返す。この非対称を `property_source()` で
//!   区別できる
//! - 表に無い id を指す(ぶら下がった参照)は `value_at()` が `None` を返す
//!   (裁定20「キーを打っていない property は静止値」の応用 — 参照先が無いだけで
//!   壊れてはいない)
//! - `SetTrack` を投げれば Slot 参照から普通の track へ戻せる(同じ component を
//!   上書きするだけ、専用の「解除」操作は無い)
//! - Slots 表もスロット参照も `flattened()`/`save()`/`load()` を経ても残る
//!   (裁定57「store に聞く」経路が新しい component も自動的に運ぶ)
//! - `TextDocument::slot_id`(text-1 が先に建てた口)は `slot` 束が実装した
//!   [`SlotId`] と同じ型 — 第二の差し替え機構になっていない(`tests/text.rs` 側で固定)

use motolii_store::{
    property, Composition, Document, Fps, Intent, Interp, Keyframe, KeyframeTrack, LayerId,
    LayerMeta, LayerSource, LayerTiming, PropertyId, PropertySource, RationalTime, Slot, SlotId,
    Value,
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

fn doc_with_layer() -> (Document, LayerId) {
    let mut doc = Document::new();
    let layer = LayerId(1);
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
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
// composition/animation/slots — comp 水準の Slots 表
// ---------------------------------------------------------------------------

/// 空の comp は空のスロット表を持つ(未設定 = 空、mask/marker と同じ扱い)。
#[test]
fn a_fresh_composition_has_no_slots() {
    let (doc, _layer) = doc_with_layer();
    assert_eq!(doc.view().slots().unwrap(), Vec::new());
}

/// `SetSlots` が Slots 表を丸ごと差し替える(`SetMasks`/`SetMarkers` と同じ形)。
#[test]
fn set_slots_round_trips_through_the_view() {
    let (mut doc, _layer) = doc_with_layer();
    let slots = vec![
        Slot {
            id: SlotId("primary_color".to_owned()),
            track: still(Value::Color([1.0, 0.0, 0.0, 1.0])),
        },
        Slot {
            id: SlotId("headline".to_owned()),
            track: still(Value::F64(12.0)),
        },
    ];
    doc.apply(Intent::SetSlots {
        slots: slots.clone(),
    })
    .unwrap();
    assert_eq!(doc.view().slots().unwrap(), slots);
}

/// 同じ id のスロットが2枚あると、property 側の `PropertySource::Slot` がどちらを
/// 指しているか決まらない(mask/effect と同型の検査)。
#[test]
fn duplicate_slot_ids_are_rejected_by_set_slots() {
    let (mut doc, _layer) = doc_with_layer();
    let result = doc.apply(Intent::SetSlots {
        slots: vec![
            Slot {
                id: SlotId("dup".to_owned()),
                track: still(Value::F64(1.0)),
            },
            Slot {
                id: SlotId("dup".to_owned()),
                track: still(Value::F64(2.0)),
            },
        ],
    });
    assert!(result.is_err(), "同じ id のスロットが2枚置けてしまっている");
}

// ---------------------------------------------------------------------------
// properties/property sid — PropertySource::{Track,Slot}
// ---------------------------------------------------------------------------

/// **新機構ゼロの固定**: `SetTrack` で書いた property は今までどおり `Track` として
/// 読める。`slot` 束を足す前と挙動が変わらないことの回帰試験。
#[test]
fn a_property_set_with_set_track_still_behaves_exactly_like_before() {
    let (mut doc, layer) = doc_with_layer();
    let opacity = PropertyId::new(property::OPACITY).unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: opacity.clone(),
        track: still(Value::F64(0.5)),
    })
    .unwrap();

    assert_eq!(
        doc.view().track(layer, &opacity).unwrap(),
        Some(still(Value::F64(0.5)))
    );
    assert_eq!(
        doc.view().value_at(layer, &opacity, t(0)).unwrap(),
        Some(Value::F64(0.5))
    );
    assert_eq!(
        doc.view().property_source(layer, &opacity).unwrap(),
        Some(PropertySource::Track(still(Value::F64(0.5))))
    );
}

/// **本題**: スロットへ差し替えた property は、`value_at` がスロット表の値を返す —
/// 同じ `sid` を持つ複数箇所を、comp 側のスロット1つを直すだけで揃って変えられる
/// (テンプレートの差し替え口そのもの)。
#[test]
fn a_property_bound_to_a_slot_resolves_the_slots_track() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetSlots {
        slots: vec![Slot {
            id: SlotId("brand_color".to_owned()),
            track: still(Value::Color([0.1, 0.2, 0.3, 1.0])),
        }],
    })
    .unwrap();

    let fill = PropertyId::new("fill").unwrap();
    doc.apply(Intent::SetPropertySlot {
        layer,
        property: fill.clone(),
        slot: SlotId("brand_color".to_owned()),
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(layer, &fill, t(0)).unwrap(),
        Some(Value::Color([0.1, 0.2, 0.3, 1.0])),
        "スロット参照が解決されていない"
    );
}

/// **`track()` は生の意味だけを返す約束**: スロットへ委譲している property は
/// 「この property 自身の track」を持たないので `None`(評価済みの値が欲しければ
/// `value_at` を使う、という非対称を固定する)。
#[test]
fn track_returns_none_for_a_slot_bound_property_even_though_value_at_resolves() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetSlots {
        slots: vec![Slot {
            id: SlotId("s".to_owned()),
            track: still(Value::F64(7.0)),
        }],
    })
    .unwrap();
    let prop = PropertyId::new("radius").unwrap();
    doc.apply(Intent::SetPropertySlot {
        layer,
        property: prop.clone(),
        slot: SlotId("s".to_owned()),
    })
    .unwrap();

    assert_eq!(doc.view().track(layer, &prop).unwrap(), None);
    assert_eq!(
        doc.view().value_at(layer, &prop, t(0)).unwrap(),
        Some(Value::F64(7.0))
    );
    assert_eq!(
        doc.view().property_source(layer, &prop).unwrap(),
        Some(PropertySource::Slot(SlotId("s".to_owned()))),
        "track() の None だけでは「スロット委譲」と「未設定」が区別できない"
    );
}

/// ぶら下がった参照(表に無い id)は、キーを打っていない property と同じ「無い」
/// として扱う(裁定20 の応用) — 壊れているのではなく、まだ値が無いだけ。
#[test]
fn a_dangling_slot_reference_resolves_to_none_not_an_error() {
    let (mut doc, layer) = doc_with_layer();
    let prop = PropertyId::new("radius").unwrap();
    doc.apply(Intent::SetPropertySlot {
        layer,
        property: prop.clone(),
        slot: SlotId("does_not_exist".to_owned()),
    })
    .unwrap();

    assert_eq!(doc.view().value_at(layer, &prop, t(0)).unwrap(), None);
}

/// `SetTrack` を再び投げれば、Slot 参照から普通の track へ戻せる — 同じ component を
/// 上書きするだけなので、専用の「解除」操作を持たない設計の固定。
#[test]
fn set_track_after_set_property_slot_switches_back_to_a_track() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetSlots {
        slots: vec![Slot {
            id: SlotId("s".to_owned()),
            track: still(Value::F64(1.0)),
        }],
    })
    .unwrap();
    let prop = PropertyId::new("radius").unwrap();
    doc.apply(Intent::SetPropertySlot {
        layer,
        property: prop.clone(),
        slot: SlotId("s".to_owned()),
    })
    .unwrap();
    assert_eq!(
        doc.view().value_at(layer, &prop, t(0)).unwrap(),
        Some(Value::F64(1.0))
    );

    doc.apply(Intent::SetTrack {
        layer,
        property: prop.clone(),
        track: still(Value::F64(99.0)),
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(layer, &prop, t(0)).unwrap(),
        Some(Value::F64(99.0)),
        "SetTrack の後もスロット参照のままになっている"
    );
    assert!(matches!(
        doc.view().property_source(layer, &prop).unwrap(),
        Some(PropertySource::Track(_))
    ));
}

/// アニメーションしているスロット(2キー)を複数の property が共有する時、
/// 各 property は自前の補間を持たず、全員がスロットの補間結果をそのまま見る。
#[test]
fn an_animated_slot_is_shared_verbatim_by_every_property_that_references_it() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetSlots {
        slots: vec![Slot {
            id: SlotId("pulse".to_owned()),
            track: ramp(Value::F64(0.0), Value::F64(30.0)),
        }],
    })
    .unwrap();

    let a = PropertyId::new("a").unwrap();
    let b = PropertyId::new("b").unwrap();
    doc.apply_all([
        Intent::SetPropertySlot {
            layer,
            property: a.clone(),
            slot: SlotId("pulse".to_owned()),
        },
        Intent::SetPropertySlot {
            layer,
            property: b.clone(),
            slot: SlotId("pulse".to_owned()),
        },
    ])
    .unwrap();

    let mid = t(15);
    let va = doc.view().value_at(layer, &a, mid).unwrap();
    let vb = doc.view().value_at(layer, &b, mid).unwrap();
    assert_eq!(va, vb);
    assert_eq!(va, Some(Value::F64(15.0)));
}

// ---------------------------------------------------------------------------
// カメラの property もスロットへ委譲できる(SetCameraTrack と同じ形)
// ---------------------------------------------------------------------------

#[test]
fn a_camera_property_can_also_be_bound_to_a_slot() {
    let (mut doc, _layer) = doc_with_layer();
    doc.apply(Intent::SetSlots {
        slots: vec![Slot {
            id: SlotId("zoom_level".to_owned()),
            track: still(Value::F64(2.0)),
        }],
    })
    .unwrap();

    let zoom = PropertyId::camera(property::CAMERA_ZOOM).unwrap();
    doc.apply(Intent::SetCameraPropertySlot {
        property: zoom.clone(),
        slot: SlotId("zoom_level".to_owned()),
    })
    .unwrap();

    assert_eq!(
        doc.view().camera_value_at(&zoom, t(0)).unwrap(),
        Some(Value::F64(2.0))
    );
    assert_eq!(doc.view().camera_track(&zoom).unwrap(), None);
}

// ---------------------------------------------------------------------------
// 保存/読込・移行コストゼロ
// ---------------------------------------------------------------------------

/// Slots 表とスロット参照は `flattened()`(裁定57)を経ても残る —
/// mask/effect/text と同じ「新しい component を1つも足さずに保存へ乗る」経路。
#[test]
fn slots_and_slot_bound_properties_survive_a_save_and_load_round_trip() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetSlots {
        slots: vec![Slot {
            id: SlotId("brand".to_owned()),
            track: still(Value::Color([0.5, 0.5, 0.5, 1.0])),
        }],
    })
    .unwrap();
    let fill = PropertyId::new("fill").unwrap();
    doc.apply(Intent::SetPropertySlot {
        layer,
        property: fill.clone(),
        slot: SlotId("brand".to_owned()),
    })
    .unwrap();
    // 通常の track も同居させて、slot 導入前と壊れずに共存することを見る。
    let opacity = PropertyId::new(property::OPACITY).unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: opacity.clone(),
        track: still(Value::F64(0.8)),
    })
    .unwrap();

    let dir = std::env::temp_dir().join(format!("motolii-slot-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("slot.rrd");
    doc.save(&file).unwrap();
    let loaded = Document::load(&file).unwrap();

    assert_eq!(
        loaded
            .view()
            .slots()
            .unwrap()
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>(),
        vec![SlotId("brand".to_owned())],
        "Slots 表が保存で消えている"
    );
    assert_eq!(
        loaded.view().value_at(layer, &fill, t(0)).unwrap(),
        Some(Value::Color([0.5, 0.5, 0.5, 1.0])),
        "スロット参照が保存で消えている"
    );
    assert_eq!(
        loaded.view().value_at(layer, &opacity, t(0)).unwrap(),
        Some(Value::F64(0.8)),
        "普通の track が slot 導入後に保存で壊れている"
    );
}
