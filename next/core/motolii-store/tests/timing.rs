//! layer の時間 — 配置・trim・split がすべて乗る土台。

use motolii_store::{
    Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, RationalTime,
};

fn doc_with_comp(duration_frames: i64) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames,
    }))
    .unwrap();
    doc
}

fn solid() -> LayerSource {
    LayerSource::Solid {
        rgba: [255, 0, 0, 255],
        width: 64,
        height: 64,
    }
}

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

#[test]
fn a_layer_only_exists_inside_its_placement() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: solid(),
                order: 0,
                timing: LayerTiming {
                    start: 10,
                    duration: 20,
                    source_in: 0,
                },
            },
        },
    ])
    .unwrap();

    let view = doc.view();
    assert!(view.resolve(layer, t(9)).unwrap().is_none(), "開始前に居る");
    assert!(view.resolve(layer, t(10)).unwrap().is_some(), "開始で居ない");
    assert!(view.resolve(layer, t(29)).unwrap().is_some(), "終端の1つ前で居ない");
    assert!(
        view.resolve(layer, t(30)).unwrap().is_none(),
        "終端は半開区間なので居てはいけない"
    );
}

#[test]
fn source_in_shifts_which_frame_is_used() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: solid(),
                order: 0,
                timing: LayerTiming {
                    start: 100,
                    duration: 50,
                    source_in: 7,
                },
            },
        },
    ])
    .unwrap();

    let resolved = doc.view().resolve(layer, t(100)).unwrap().expect("居る");
    assert_eq!(resolved.source_frame, 7, "素材の頭出しが効いていない");

    let later = doc.view().resolve(layer, t(110)).unwrap().expect("居る");
    assert_eq!(later.source_frame, 17, "comp の進みが素材へ写っていない");
}

/// M4 — 置いた時の尺は **min(素材, comp の残り)**。
#[test]
fn placement_clamps_to_the_shorter_of_source_and_comp() {
    // 素材が comp より長い → comp の残りで切れる
    let long_source = LayerTiming::place(280, Some(600), 300);
    assert_eq!(long_source.duration, 20, "comp の残りで切れていない");

    // 素材が短い → 素材の尺
    let short_source = LayerTiming::place(0, Some(45), 300);
    assert_eq!(short_source.duration, 45, "素材の尺を超えて置いている");

    // 尺が分からない(静止画)→ comp の残り全部
    let still = LayerTiming::place(100, None, 300);
    assert_eq!(still.duration, 200);

    // comp の外に置こうとしたら尺ゼロ(負にしない)
    let outside = LayerTiming::place(400, Some(60), 300);
    assert_eq!(outside.duration, 0);
}

/// move / trim / split はすべて `SetTiming` 1つで表せること。
#[test]
fn move_trim_and_split_are_all_one_intent() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: solid(),
                order: 0,
                timing: LayerTiming::place(0, Some(100), 300),
            },
        },
    ])
    .unwrap();

    // move: start だけ動かす
    doc.apply(Intent::SetTiming {
        layer,
        timing: LayerTiming {
            start: 50,
            duration: 100,
            source_in: 0,
        },
    })
    .unwrap();
    assert!(doc.view().resolve(layer, t(50)).unwrap().is_some());
    assert!(doc.view().resolve(layer, t(49)).unwrap().is_none());

    // trim(頭): start と source_in と duration を一緒に動かす
    doc.apply(Intent::SetTiming {
        layer,
        timing: LayerTiming {
            start: 60,
            duration: 90,
            source_in: 10,
        },
    })
    .unwrap();
    let head = doc.view().resolve(layer, t(60)).unwrap().expect("居る");
    assert_eq!(head.source_frame, 10, "頭 trim で素材の頭出しがずれていない");

    // 1操作 = 1 undo
    doc.undo();
    let restored = doc.view().resolve(layer, t(50)).unwrap().expect("居る");
    assert_eq!(restored.source_frame, 0);
}
