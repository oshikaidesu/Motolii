//! Document の契約 — undo/redo が「時間の移動」だけで成立することを固定する。

use std::time::Instant;

use motolii_store::{
    Document, Interp, Intent, Keyframe, KeyframeTrack, LayerId, PropertyId, RationalTime, Value,
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
