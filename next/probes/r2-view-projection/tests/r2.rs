//! R2: `view()` 1回ぶんの投影コスト。

use std::time::Instant;

use motolii_store::{
    property, Document, Interp, Intent, Keyframe, KeyframeTrack, LayerId, LayerMeta, LayerSource,
    PropertyId, RationalTime, Value,
};
use motolii_store::LayerTiming;

/// 実寸に近い Document: 40 layer × 5 property × 30 keyframe。
fn realistic_document() -> Document {
    let mut doc = Document::new();
    let props = [
        property::POSITION,
        property::OPACITY,
    ];

    for i in 0..40u64 {
        let layer = LayerId(i);
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Solid {
                    rgba: [200, 100, 50, 255],
                    width: 1920,
                    height: 1080,
                },
                order: i as i16,
                timing: LayerTiming::place(0, None, 100_000),
            },
        })
        .unwrap();

        for name in props {
            let mut track = KeyframeTrack::new();
            for k in 0..30 {
                track.insert(Keyframe {
                    t: RationalTime::try_new(k * 10, 30).unwrap(),
                    value: Value::F64(k as f64),
                    interp: Interp::Linear,
                    spatial: None,
                });
            }
            doc.apply(Intent::SetTrack {
                layer,
                property: PropertyId::new(name).unwrap(),
                track,
            })
            .unwrap();
        }
    }
    doc
}

/// **Stage が要る投影** — 全 layer を comp 時刻で解決する。
#[test]
fn stage_projection_fits_a_frame() {
    // 16.6ms のうち投影に割ける分。ここを超えるなら front はキャッシュを持つことになり、
    // 翻訳層が戻ってくる。
    const BUDGET_US: u128 = 2_000;
    const RUNS: u128 = 200;

    let doc = realistic_document();
    let t = RationalTime::try_new(100, 30).unwrap();

    let _ = doc.view().resolved_layers(t).unwrap();
    let start = Instant::now();
    let mut count = 0usize;
    for _ in 0..RUNS {
        count += doc.view().resolved_layers(t).unwrap().len();
    }
    let per_call = start.elapsed().as_micros() / RUNS;

    println!(
        "R2 Stage 投影(40 layer × 5 property × 30 key): {per_call}µs / 予算 {BUDGET_US}µs \
         (16.6ms の {:.1}%) — layer {}枚",
        per_call as f64 / 166.0,
        count / RUNS as usize
    );
    assert!(
        per_call < BUDGET_US,
        "Stage 投影が {per_call}µs。front がキャッシュを持たざるを得なくなる"
    );
}

/// **Timeline が要る投影** — 全 layer の全 property の track を生で読む
/// (キー菱形を描くのに必要)。Stage より重い側。
#[test]
fn timeline_projection_fits_a_frame() {
    const BUDGET_US: u128 = 4_000;
    const RUNS: u128 = 100;

    let doc = realistic_document();
    let props: Vec<PropertyId> = [
        property::POSITION,
        property::OPACITY,
    ]
    .iter()
    .map(|name| PropertyId::new(name).unwrap())
    .collect();

    let read_all = |doc: &Document| {
        let view = doc.view();
        let mut keys = 0usize;
        for layer in view.layers() {
            for property in &props {
                if let Some(track) = view.track(layer, property).unwrap() {
                    keys += track.keys().len();
                }
            }
        }
        keys
    };

    let _ = read_all(&doc);
    let start = Instant::now();
    let mut keys = 0;
    for _ in 0..RUNS {
        keys = read_all(&doc);
    }
    let per_call = start.elapsed().as_micros() / RUNS;

    println!(
        "R2 Timeline 投影(track を生で全部): {per_call}µs / 予算 {BUDGET_US}µs \
         (16.6ms の {:.1}%) — key {keys}個",
        per_call as f64 / 166.0
    );
    assert!(
        per_call < BUDGET_US,
        "Timeline 投影が {per_call}µs。front がキャッシュを持たざるを得なくなる"
    );
}

/// 変化検出が**上流にある**こと。無ければ front は「前回の値」を持つことになり、
/// それが二重帳簿の入口になる。
#[test]
fn change_detection_is_upstream_and_cheap() {
    const RUNS: u128 = 10_000;

    let mut doc = realistic_document();
    let before = doc.revision();

    let start = Instant::now();
    let mut sink = 0u64;
    for _ in 0..RUNS {
        sink += u64::from(doc.revision() == before);
    }
    let per_call = start.elapsed().as_nanos() / RUNS;
    assert_eq!(sink, RUNS as u64);

    doc.apply(Intent::AddLayer(LayerId(999))).unwrap();
    assert_ne!(
        doc.revision(),
        before,
        "編集しても generation が動かないなら変化検出に使えない"
    );

    assert!(doc.undo());
    println!("R2 変化検出: revision() = {per_call}ns/回(store 世代 + edit 位置)");
    assert!(per_call < 1_000, "revision() が {per_call}ns。毎フレーム呼べない");

    // **undo/redo でも変わること**。store の世代だけを見ていた頃は undo で変わらず、
    // front が `last_edit_head` を自分で持つ入口になっていた(2026-08-20 の敵対的レビュー)。
    let after_edit = doc.revision();
    assert!(doc.undo());
    assert_ne!(
        doc.revision(),
        after_edit,
        "undo しても revision が変わらないなら front は再描画できない"
    );
    assert!(doc.redo());
    assert_eq!(doc.revision(), after_edit, "redo で元の revision へ戻る");
}
