//! solo / locked 束(採番用の設計判断は終了報告に列挙、DECISIONS.md には書かない)。
//!
//! ここで固定するのは:
//! - solo: comp のどこかに `solo=true` な layer が居ると、solo でない layer は
//!   resolve に落ちる(hidden と同じ経路)
//! - **hidden が solo に勝つ** — hidden かつ solo な layer は出ない
//! - solo が誰にも立っていなければ今まで通り(既定 false、既存の振る舞いを壊さない)
//! - locked: 描画には無関係(resolve/engine は読まない)。効くのは `Document::write`
//!   のほう — locked な layer への層変更 Intent は理由つき `Err` になる
//! - **`locked` 自身の解除/再ロックだけは常に通る**(自分をロックしたら二度と
//!   触れなくなる詰みを作らない)
//! - 保存往復で solo/locked が消えない

use motolii_store::{
    Composition, Document, Fps, Intent, Interp, Keyframe, KeyframeTrack, LayerAttrsPatch, LayerId,
    LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Slot, SlotId, Value,
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

fn doc_with_comp(duration_frames: i64) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    doc
}

fn solid(rgba: [u8; 4]) -> LayerSource {
    LayerSource::Solid {
        rgba,
        width: 64,
        height: 64,
    }
}

fn place(doc: &mut Document, layer: LayerId, source: LayerSource, start: i64, duration: i64) {
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source,
                order: 0,
                timing: LayerTiming {
                    start,
                    duration,
                    source_in: 0,
                    ..Default::default()
                },
            },
        },
    ])
    .unwrap();
}

fn set_solo(doc: &mut Document, layer: LayerId, solo: bool) {
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            solo: Some(solo),
            ..Default::default()
        },
    })
    .unwrap();
}

fn set_hidden(doc: &mut Document, layer: LayerId, hidden: bool) {
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            hidden: Some(hidden),
            ..Default::default()
        },
    })
    .unwrap();
}

fn set_locked(
    doc: &mut Document,
    layer: LayerId,
    locked: bool,
) -> Result<(), motolii_store::StoreError> {
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            locked: Some(locked),
            ..Default::default()
        },
    })
}

// ---------------------------------------------------------------------------
// solo
// ---------------------------------------------------------------------------

/// 誰も solo でなければ、今まで通り全員 resolve する(既定 false が振る舞いを
/// 変えないことの固定)。
#[test]
fn no_solo_resolves_everyone() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    let b = LayerId(2);
    place(&mut doc, a, solid([255, 0, 0, 255]), 0, 100);
    place(&mut doc, b, solid([0, 255, 0, 255]), 0, 100);

    assert!(doc.view().resolve(a, t(0)).unwrap().is_some());
    assert!(doc.view().resolve(b, t(0)).unwrap().is_some());
}

/// solo な layer が1つでも居ると、solo でない layer は resolve に出ない。
/// (画素/resolve テスト — 合成する前の resolve 段階で落ちることを見る)
#[test]
fn solo_layer_suppresses_non_solo_siblings() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    let b = LayerId(2);
    place(&mut doc, a, solid([255, 0, 0, 255]), 0, 100);
    place(&mut doc, b, solid([0, 255, 0, 255]), 0, 100);

    set_solo(&mut doc, a, true);

    assert!(
        doc.view().resolve(a, t(0)).unwrap().is_some(),
        "solo 自身は resolve するはず"
    );
    assert!(
        doc.view().resolve(b, t(0)).unwrap().is_none(),
        "solo でない layer は落ちるはず"
    );

    // resolved_layers 経由(engine が実際に使う経路)でも同じ結果になる。
    let layers = doc.view().resolved_layers(t(0)).unwrap();
    assert_eq!(
        layers.len(),
        1,
        "resolved_layers も solo でない layer を落とすはず"
    );
}

/// solo を複数枚に立てると、立てた分だけ resolve する(排他ではなく「solo 集合」)。
#[test]
fn multiple_solo_layers_all_resolve() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    let b = LayerId(2);
    let c = LayerId(3);
    place(&mut doc, a, solid([255, 0, 0, 255]), 0, 100);
    place(&mut doc, b, solid([0, 255, 0, 255]), 0, 100);
    place(&mut doc, c, solid([0, 0, 255, 255]), 0, 100);

    set_solo(&mut doc, a, true);
    set_solo(&mut doc, b, true);

    assert!(doc.view().resolve(a, t(0)).unwrap().is_some());
    assert!(doc.view().resolve(b, t(0)).unwrap().is_some());
    assert!(doc.view().resolve(c, t(0)).unwrap().is_none());
}

/// **hidden が solo に勝つ** — hidden かつ solo な layer は出ない
/// (隠した層は solo で復活しない)。
#[test]
fn hidden_wins_over_solo_on_the_same_layer() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, solid([255, 0, 0, 255]), 0, 100);

    set_solo(&mut doc, a, true);
    set_hidden(&mut doc, a, true);

    assert!(
        doc.view().resolve(a, t(0)).unwrap().is_none(),
        "hidden かつ solo な layer は出てはいけない"
    );
}

/// hidden かつ solo な layer が居ても、その solo フラグ自体は「comp のどこかに
/// solo が居る」を成立させる — 他の非 hidden・非 solo な layer は依然として
/// 落ちる(solo という意図は hidden に隠されても消えない)。
#[test]
fn a_hidden_solo_layer_still_activates_solo_mode_for_others() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    let b = LayerId(2);
    place(&mut doc, a, solid([255, 0, 0, 255]), 0, 100);
    place(&mut doc, b, solid([0, 255, 0, 255]), 0, 100);

    set_solo(&mut doc, a, true);
    set_hidden(&mut doc, a, true);

    assert!(
        doc.view().resolve(a, t(0)).unwrap().is_none(),
        "hidden が勝つので a 自体は出ない"
    );
    assert!(
        doc.view().resolve(b, t(0)).unwrap().is_none(),
        "a の solo が comp 全体を solo モードにするので、b(非 solo)は出ない"
    );
}

// ---------------------------------------------------------------------------
// locked — 拒否
// ---------------------------------------------------------------------------

#[test]
fn locked_layer_rejects_set_timing() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid([255, 0, 0, 255]), 0, 100);
    set_locked(&mut doc, layer, true).unwrap();

    let result = doc.apply(Intent::SetTiming {
        layer,
        timing: LayerTiming {
            start: 10,
            duration: 50,
            source_in: 0,
            ..Default::default()
        },
    });
    let err = result.expect_err("locked 層への SetTiming は拒まれるはず");
    assert!(
        err.to_string().contains("locked"),
        "エラーは理由(locked)を含むはず: {err}"
    );
}

#[test]
fn locked_layer_rejects_set_source() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid([255, 0, 0, 255]), 0, 100);
    set_locked(&mut doc, layer, true).unwrap();

    let result = doc.apply(Intent::SetSource {
        layer,
        source: solid([0, 255, 0, 255]),
    });
    assert!(result.is_err(), "locked 層への SetSource は拒まれるはず");
}

#[test]
fn locked_layer_rejects_set_order() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid([255, 0, 0, 255]), 0, 100);
    set_locked(&mut doc, layer, true).unwrap();

    let result = doc.apply(Intent::SetOrder { layer, order: 7 });
    assert!(result.is_err(), "locked 層への SetOrder は拒まれるはず");
}

/// 削除は最も破壊的な編集(supervisor 裁定): locked な layer への RemoveLayer は
/// 他の層変更 Intent と同じく理由つき Err で拒まれる(AE と同じ意味論)。
#[test]
fn locked_layer_rejects_remove_layer() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid([255, 0, 0, 255]), 0, 100);
    set_locked(&mut doc, layer, true).unwrap();

    let result = doc.apply(Intent::RemoveLayer(layer));
    let err = result.expect_err("locked 層への RemoveLayer は拒まれるはず");
    assert!(
        err.to_string().contains("locked"),
        "エラーは理由(locked)を含むはず: {err}"
    );

    // 実際に消えていないことも確かめる(present は `has_layer` が見る — `resolve`
    // 単体は present を見ないので、削除の成否の確認には使えない)。
    assert!(
        doc.view().has_layer(layer),
        "拒否されたのに layer が消えている"
    );
}

/// 解除→削除の2手は常に可能(詰みを作らない、`locking_yourself_is_not_a_dead_end`
/// と同じ形の確認を RemoveLayer で行う)。
#[test]
fn unlock_then_remove_layer_always_succeeds() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid([255, 0, 0, 255]), 0, 100);
    set_locked(&mut doc, layer, true).unwrap();

    assert!(doc.apply(Intent::RemoveLayer(layer)).is_err());

    set_locked(&mut doc, layer, false).expect("解除できないと詰む");
    doc.apply(Intent::RemoveLayer(layer))
        .expect("解除後の RemoveLayer は通るはず");

    assert!(
        !doc.view().has_layer(layer),
        "解除後に削除したのに layer がまだ present"
    );
}

#[test]
fn locked_layer_rejects_set_masks() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid([255, 0, 0, 255]), 0, 100);
    set_locked(&mut doc, layer, true).unwrap();

    let result = doc.apply(Intent::SetMasks {
        layer,
        masks: Vec::new(),
    });
    assert!(result.is_err(), "locked 層への SetMasks は拒まれるはず");
}

#[test]
fn locked_layer_rejects_set_effects() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid([255, 0, 0, 255]), 0, 100);
    set_locked(&mut doc, layer, true).unwrap();

    let result = doc.apply(Intent::SetEffects {
        layer,
        effects: Vec::new(),
    });
    assert!(result.is_err(), "locked 層への SetEffects は拒まれるはず");
}

#[test]
fn locked_layer_rejects_set_track() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid([255, 0, 0, 255]), 0, 100);
    set_locked(&mut doc, layer, true).unwrap();

    let result = doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::new("opacity_probe").unwrap(),
        track: still(Value::F64(1.0)),
    });
    assert!(result.is_err(), "locked 層への SetTrack は拒まれるはず");
}

#[test]
fn locked_layer_rejects_set_property_slot() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid([255, 0, 0, 255]), 0, 100);
    set_locked(&mut doc, layer, true).unwrap();

    doc.apply(Intent::SetSlots {
        slots: vec![Slot {
            id: SlotId("s1".to_owned()),
            track: still(Value::F64(1.0)),
        }],
    })
    .unwrap();

    let result = doc.apply(Intent::SetPropertySlot {
        layer,
        property: PropertyId::new("opacity_probe").unwrap(),
        slot: SlotId("s1".to_owned()),
    });
    assert!(
        result.is_err(),
        "locked 層への SetPropertySlot は拒まれるはず"
    );
}

/// SetAttrs で `locked` 以外のフィールドを触ろうとすると拒まれる。
#[test]
fn locked_layer_rejects_set_attrs_on_other_fields() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid([255, 0, 0, 255]), 0, 100);
    set_locked(&mut doc, layer, true).unwrap();

    let result = doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            name: Some("触れないはず".to_owned()),
            ..Default::default()
        },
    });
    let err = result.expect_err("locked 層への SetAttrs(名前変更)は拒まれるはず");
    assert!(err.to_string().contains("locked"));

    // 実際に触れていないことも確かめる。
    let attrs = doc.view().attrs(layer).unwrap().unwrap();
    assert_eq!(attrs.name, "", "拒否されたのに name が変わっている");
}

// ---------------------------------------------------------------------------
// locked — 解除だけは常に通る
// ---------------------------------------------------------------------------

#[test]
fn unlocking_a_locked_layer_always_succeeds() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid([255, 0, 0, 255]), 0, 100);
    set_locked(&mut doc, layer, true).unwrap();

    // 解除そのものは常に通る。
    set_locked(&mut doc, layer, false).expect("locked の解除は常に通るはず");

    let attrs = doc.view().attrs(layer).unwrap().unwrap();
    assert!(!attrs.locked);

    // 解除した後は普通に編集できる(詰みを作っていないことの確認)。
    doc.apply(Intent::SetOrder { layer, order: 3 })
        .expect("解除後は普通に編集できるはず");
}

/// 既に locked な layer を再度 locked にする(`locked` だけを触る patch)も通る —
/// 「locked 以外の何かに触れているか」だけが拒否条件であって、値そのものではない。
#[test]
fn relocking_an_already_locked_layer_is_not_rejected() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid([255, 0, 0, 255]), 0, 100);
    set_locked(&mut doc, layer, true).unwrap();

    set_locked(&mut doc, layer, true).expect("locked を再度 true にする patch は通るはず");
}

/// 自分をロックしたら二度と触れなくなる、という詰みを作っていないことの
/// エンドツーエンド確認(ロック→拒否→解除→成功、を1つの流れで)。
#[test]
fn locking_yourself_is_not_a_dead_end() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid([255, 0, 0, 255]), 0, 100);

    set_locked(&mut doc, layer, true).unwrap();
    assert!(doc.apply(Intent::SetOrder { layer, order: 9 }).is_err());

    set_locked(&mut doc, layer, false).expect("解除できないと詰む");
    assert!(doc.apply(Intent::SetOrder { layer, order: 9 }).is_ok());
}

// ---------------------------------------------------------------------------
// locked — 描画には無関係
// ---------------------------------------------------------------------------

/// locked は resolve に影響しない(hidden とは独立の軸)。
#[test]
fn locked_does_not_affect_resolve() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid([255, 0, 0, 255]), 0, 100);
    set_locked(&mut doc, layer, true).unwrap();

    assert!(
        doc.view().resolve(layer, t(0)).unwrap().is_some(),
        "locked は描画に無関係なので resolve は出るはず"
    );
}

// ---------------------------------------------------------------------------
// 保存往復
// ---------------------------------------------------------------------------

#[test]
fn solo_and_locked_round_trip_through_save_and_load() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid([255, 0, 0, 255]), 0, 100);
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            solo: Some(true),
            locked: Some(true),
            ..Default::default()
        },
    })
    .unwrap();

    let dir =
        std::env::temp_dir().join(format!("motolii-solo-locked-fence-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("solo_locked_roundtrip.motolii");
    doc.save(&path).expect("保存できない");

    let loaded = Document::load(&path).expect("読み込めない");
    let attrs = loaded.view().attrs(layer).unwrap().unwrap();
    assert!(attrs.solo, "solo が保存往復で消えた");
    assert!(attrs.locked, "locked が保存往復で消えた");
}
