//! 保存と読込 — **形式は上流の `.rrd`**。自前形式を発明していない。

use motolii_store::{
    Composition, Document, Fps, Interp, Intent, Keyframe, KeyframeTrack, LayerId, LayerMeta,
    LayerSource, LayerTiming, PropertyId, RationalTime, Value, property,
};

fn tmp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("motolii-persist-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

fn authored() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 1920,
        height: 1080,
        fps: Fps::try_new(30000, 1001).unwrap(),
        duration_frames: 900,
    }))
    .unwrap();

    for i in 1..=3u64 {
        let layer = LayerId(i);
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: t(0),
            value: Value::Vec2([0.0, 0.0]),
            interp: Interp::Bezier {
                x1: 0.42,
                y1: 0.0,
                x2: 0.58,
                y2: 1.0,
            },
        });
        track.insert(Keyframe {
            t: t(60),
            value: Value::Vec2([500.0, 0.0]),
            interp: Interp::Linear,
        });

        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Solid {
                        rgba: [10 * i as u8, 20, 30, 255],
                        width: 640,
                        height: 360,
                    },
                    order: i as i16,
                    timing: LayerTiming {
                        start: i as i64 * 30,
                        duration: 120,
                        source_in: 5,
                    },
                },
            },
            Intent::SetTrack {
                layer,
                property: PropertyId::new(property::POSITION).unwrap(),
                track,
            },
        ])
        .unwrap();
    }
    doc
}

#[test]
fn save_and_load_round_trips_the_whole_document() {
    let doc = authored();
    let path = tmp("roundtrip.motolii");
    doc.save(&path).expect("保存できない");

    let loaded = Document::load(&path).expect("読み込めない");

    assert_eq!(
        loaded.view().composition().unwrap(),
        doc.view().composition().unwrap(),
        "comp 設定が往復しない"
    );
    assert_eq!(loaded.view().layers(), doc.view().layers(), "layer が往復しない");

    for layer in doc.view().layers() {
        assert_eq!(
            loaded.view().meta(layer).unwrap(),
            doc.view().meta(layer).unwrap(),
            "layer {layer:?} の素材・重ね順・配置が往復しない"
        );
        assert_eq!(
            loaded.view().properties(layer),
            doc.view().properties(layer),
            "property の一覧が往復しない"
        );
        for property in doc.view().properties(layer) {
            assert_eq!(
                loaded.view().track(layer, &property).unwrap(),
                doc.view().track(layer, &property).unwrap(),
                "track が往復しない({property:?})"
            );
        }
        // 解決まで一致すること(bezier イージングも含めて)。
        for frame in [30, 45, 60, 90] {
            assert_eq!(
                loaded.view().resolve(layer, t(frame)).unwrap(),
                doc.view().resolve(layer, t(frame)).unwrap(),
                "frame {frame} の解決が往復しない"
            );
        }
    }
}

/// **保存で履歴を畳む**。畳まないと project file が編集回数に比例して伸びる。
#[test]
fn saving_folds_the_edit_history() {
    let mut doc = authored();
    // たくさん編集する
    for i in 0..200 {
        doc.apply(Intent::SetTiming {
            layer: LayerId(1),
            timing: LayerTiming {
                start: i,
                duration: 120,
                source_in: 5,
            },
        })
        .unwrap();
    }
    assert!(doc.edit_head() > 200);

    let path = tmp("folded.motolii");
    doc.save(&path).unwrap();
    let loaded = Document::load(&path).unwrap();

    assert_eq!(
        loaded.view().meta(LayerId(1)).unwrap().unwrap().timing.start,
        199,
        "最後の状態が保存されていない"
    );
    assert!(
        !loaded.can_undo(),
        "開いた直後に戻せる = 履歴が畳まれていないか、底が立っていない"
    );

    let size = std::fs::metadata(&path).unwrap().len();
    println!("保存サイズ: {size} bytes(編集 {}回のあと)", doc.edit_head());
    assert!(
        size < 256 * 1024,
        "200編集した後の保存が {size} bytes。履歴が畳まれていない"
    );
}
