//! Document の契約 — undo/redo が「時間の移動」だけで成立することを固定する。

use std::time::Instant;

use motolii_store::{
    Document, EffectId, EffectInstance, Interp, Intent, Keyframe, KeyframeTrack, LayerId,
    PropertyId, RationalTime, Value,
};

fn prop(name: &str) -> PropertyId {
    PropertyId::new(name).expect("property name")
}

fn t(num: i64) -> RationalTime {
    RationalTime::try_new(num, 30).expect("rational time")
}

fn track(points: &[(i64, f64, Interp)]) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    for (frame, value, interp) in points {
        track.insert(Keyframe {
            t: t(*frame),
            value: Value::F64(*value),
            interp: interp.clone(),
            spatial: None,
        });
    }
    track
}

#[test]
fn undo_and_redo_are_only_time_movement() {
    let mut doc = Document::new();
    let layer = LayerId(1);
    let opacity = prop("opacity");

    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: opacity.clone(),
        track: track(&[(0, 0.0, Interp::Linear), (30, 1.0, Interp::Linear)]),
    })
    .unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: opacity.clone(),
        track: track(&[(0, 0.5, Interp::Linear), (30, 1.0, Interp::Linear)]),
    })
    .unwrap();

    assert_eq!(doc.view().value_at(layer, &opacity, t(0)).unwrap(), Some(Value::F64(0.5)));

    assert!(doc.undo());
    assert_eq!(doc.view().value_at(layer, &opacity, t(0)).unwrap(), Some(Value::F64(0.0)));

    assert!(doc.undo());
    assert_eq!(doc.view().value_at(layer, &opacity, t(0)).unwrap(), None, "track を打つ前へ戻る");
    assert!(doc.view().has_layer(layer), "layer 追加はまだ生きている");

    // redo は store から何も失われていないので、時間を進めるだけで戻る
    assert!(doc.redo());
    assert_eq!(doc.view().value_at(layer, &opacity, t(0)).unwrap(), Some(Value::F64(0.0)));
    assert!(doc.redo());
    assert_eq!(doc.view().value_at(layer, &opacity, t(0)).unwrap(), Some(Value::F64(0.5)));
    assert!(!doc.redo(), "先端では redo できない");
}

#[test]
fn undo_to_empty_document() {
    let mut doc = Document::new();
    let layer = LayerId(7);

    doc.apply(Intent::AddLayer(layer)).unwrap();
    assert!(doc.view().has_layer(layer));

    assert!(doc.undo());
    assert_eq!(doc.edit_head(), 0);
    assert!(doc.view().layers().is_empty(), "edit=0 は空の Document");
    assert!(!doc.undo(), "空より前へは戻れない");
}

#[test]
fn new_edit_after_undo_drops_the_redo_space() {
    let mut doc = Document::new();
    let layer = LayerId(1);
    let opacity = prop("opacity");

    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: opacity.clone(),
        track: track(&[(0, 1.0, Interp::Linear)]),
    })
    .unwrap();

    assert!(doc.undo());
    assert!(doc.can_redo());

    // 分岐した編集。ここで redo 空間が落ちる(rerun blueprint と同じ規則)。
    doc.apply(Intent::SetTrack {
        layer,
        property: opacity.clone(),
        track: track(&[(0, 0.25, Interp::Linear)]),
    })
    .unwrap();

    assert!(!doc.can_redo(), "分岐後に redo 先が残っていてはならない");
    assert_eq!(
        doc.view().value_at(layer, &opacity, t(0)).unwrap(),
        Some(Value::F64(0.25))
    );
}

#[test]
fn remove_layer_is_a_tombstone_not_a_delete() {
    let mut doc = Document::new();
    let layer = LayerId(3);

    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::RemoveLayer(layer)).unwrap();
    assert!(!doc.view().has_layer(layer));

    // chunk を落としていないので、時間を戻すだけで復活する
    assert!(doc.undo());
    assert!(
        doc.view().has_layer(layer),
        "削除を drop で実装すると undo で戻らない"
    );
}

/// 敵対的レビュー(2026-08-20)が実証した部分コミットの再現。修正前は `write` が
/// intent ごとに即座に store へ確定していたので、`apply_all([正当, 不正])` は
/// 「正当な分だけ確定し `head` も進んだまま `Err` を返す」という部分コミットに
/// なっていた。**原子性**: `Err` の後は `view()` がバッチ前と完全に一致しなければ
/// ならない(`has_layer` も `edit_head` も)。
#[test]
fn apply_all_is_atomic_the_valid_intent_does_not_stick_when_a_later_one_fails() {
    let mut doc = Document::new();
    let layer = LayerId(1);
    let before_head = doc.edit_head();

    let result = doc.apply_all([
        Intent::AddLayer(layer),
        // 同じ id の effect が2枚 — `write` が `Err` を返す(mask と同型の検査)。
        Intent::SetEffects {
            layer,
            effects: vec![
                EffectInstance {
                    id: EffectId(1),
                    plugin_id: "a".to_owned(),
                    enabled: true,
                },
                EffectInstance {
                    id: EffectId(1),
                    plugin_id: "b".to_owned(),
                    enabled: true,
                },
            ],
        },
    ]);

    assert!(result.is_err(), "重複 effect id が通ってしまっている");
    assert!(
        !doc.view().has_layer(layer),
        "先行 intent(AddLayer)がバッチの失敗を跨いで確定してしまっている(部分コミット)"
    );
    assert_eq!(
        doc.edit_head(),
        before_head,
        "失敗した batch の後で edit_head が前進してしまっている"
    );
    assert!(!doc.can_undo(), "確定していない batch が undo 履歴に残っている");
}

/// 複数の失敗パターンで同じ原子性が成り立つことを確かめる(`SetMeta` の新規配置専用の
/// 柵、裁定108(c))— `AddLayer` + `SetMeta` の後にもう一度 `SetMeta` を同じバッチへ
/// 混ぜると2つ目が `Err` になるが、1つ目の配置も含めてバッチ全体が無かったことになる。
#[test]
fn apply_all_rolls_back_even_when_the_batch_writes_multiple_components() {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(motolii_store::Composition {
        width: 64,
        height: 64,
        fps: motolii_store::Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let after_setup_head = doc.edit_head();
    let layer = LayerId(1);

    let meta = motolii_store::LayerMeta {
        source: motolii_store::LayerSource::Solid {
            rgba: [1, 2, 3, 255],
            width: 4,
            height: 4,
        },
        order: 0,
        timing: motolii_store::LayerTiming::place(0, None, 300),
    };

    let result = doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: meta.clone(),
        },
        // 同じバッチ内でもう一度 `SetMeta` — 直前の物と合わせて「既に meta を持つ」
        // 判定に引っかかり `Err` になる。
        Intent::SetMeta { layer, meta },
    ]);

    assert!(result.is_err());
    assert!(!doc.view().has_layer(layer), "AddLayer が部分コミットしている");
    assert_eq!(doc.view().meta(layer).unwrap(), None, "SetMeta が部分コミットしている");
    assert_eq!(doc.edit_head(), after_setup_head);
}

#[test]
fn value_at_goes_through_the_ported_evaluator() {
    let mut doc = Document::new();
    let layer = LayerId(1);
    let position = prop("position.x");

    // ease-in-out。線形なら中点は 0.5 になるが、この bezier では下回る。
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: position.clone(),
        track: track(&[
            (
                0,
                0.0,
                Interp::Bezier {
                    x1: 0.42,
                    y1: 0.0,
                    x2: 0.58,
                    y2: 1.0,
                },
            ),
            (30, 1.0, Interp::Linear),
        ]),
    })
    .unwrap();

    let Some(Value::F64(mid)) = doc.view().value_at(layer, &position, t(15)).unwrap() else {
        panic!("中点が取れない");
    };
    assert!(
        (mid - 0.5).abs() < 0.05,
        "ease-in-out の中点は 0.5 付近のはず: {mid}"
    );

    let Some(Value::F64(quarter)) = doc.view().value_at(layer, &position, t(7)).unwrap() else {
        panic!("1/4 点が取れない");
    };
    assert!(
        quarter < 0.25,
        "ease-in は序盤が遅い(線形なら 0.233 付近): {quarter}"
    );
}

/// R0-1 を「実際に使う型」で再測する。JSON 符号化の代償を予算で縛る。
#[test]
fn edit_storm_with_the_real_track_type() {
    const KEYS: i64 = 300;
    const EDITS: i64 = 1000;
    const BYTES_BUDGET: u64 = 64 * 1024 * 1024;
    const QUERY_BUDGET_US: u128 = 1000;

    let mut doc = Document::new();
    let layer = LayerId(1);
    let position = prop("position.x");
    doc.apply(Intent::AddLayer(layer)).unwrap();

    let points: Vec<(i64, f64, Interp)> = (0..KEYS)
        .map(|k| (k, k as f64, Interp::Linear))
        .collect();

    let start = Instant::now();
    for edit in 0..EDITS {
        let mut points = points.clone();
        points[0].1 = edit as f64;
        doc.apply(Intent::SetTrack {
            layer,
            property: position.clone(),
            track: track(&points),
        })
        .unwrap();
    }
    let write_elapsed = start.elapsed();

    let query_start = Instant::now();
    let value = doc.view().value_at(layer, &position, t(0)).unwrap();
    let query_us = query_start.elapsed().as_micros();
    assert_eq!(value, Some(Value::F64((EDITS - 1) as f64)));

    println!(
        "store storm: {EDITS}編集 × {KEYS}打点 — chunks={} bytes={:.1}MB 書き込み={:?}({:.0}µs/編集) query={query_us}µs",
        doc.store_chunks(),
        doc.store_bytes() as f64 / 1024.0 / 1024.0,
        write_elapsed,
        write_elapsed.as_micros() as f64 / EDITS as f64,
    );

    assert!(
        doc.store_bytes() < BYTES_BUDGET,
        "store が {}MB(予算 {}MB)",
        doc.store_bytes() / 1024 / 1024,
        BYTES_BUDGET / 1024 / 1024
    );
    assert!(
        query_us < QUERY_BUDGET_US,
        "query が {query_us}µs(予算 {QUERY_BUDGET_US}µs)"
    );
}
