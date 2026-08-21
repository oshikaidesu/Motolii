//! 保存と読込 — **形式は上流の `.rrd`**。自前形式を発明していない。

use motolii_store::{
    Composition, Document, Fps, Interp, Intent, Keyframe, KeyframeTrack, LayerAttrsPatch, LayerId,
    LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value, property,
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
        background: [0.0, 0.0, 0.0, 1.0],
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
            spatial: None,
        });
        track.insert(Keyframe {
            t: t(60),
            value: Value::Vec2([500.0, 0.0]),
            interp: Interp::Linear,
            spatial: None,
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
                        ..Default::default()
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

/// `Composition::background`(黒だと気分が上がらない、利用者要望)専用の保存往復。
/// 既定でない値を使い、シリアライズ漏れを検出する。
#[test]
fn composition_background_round_trips_through_save_and_load() {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 360,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: [0.2, 0.4, 0.6, 0.8],
    }))
    .unwrap();

    let path = tmp("background_roundtrip.motolii");
    doc.save(&path).expect("保存できない");
    let loaded = Document::load(&path).expect("読み込めない");

    assert_eq!(
        loaded.view().composition().unwrap().unwrap().background,
        [0.2, 0.4, 0.6, 0.8],
        "background が保存往復で消えた/変わった"
    );
}

/// 旧保存ファイル(`background` component が無い版)を模した JSON でも読める —
/// `#[serde(default = "Composition::default_background")]` の後方互換確認。
/// 実ファイルではなく JSON 直操作で確かめる: 現行の `Composition` を一度シリアライズし、
/// `background` キーだけを取り除いてから読み戻す(旧 Fps の num/den 等、他フィールドの
/// 形式をハードコードしないための遠回り)。
#[test]
fn composition_without_a_background_field_defaults_to_opaque_black() {
    let current = Composition {
        width: 640,
        height: 360,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: [0.9, 0.9, 0.9, 0.9],
    };
    let mut value = serde_json::to_value(&current).unwrap();
    value
        .as_object_mut()
        .expect("Composition は JSON object のはず")
        .remove("background")
        .expect("旧形式を模すには background キーが無いことが前提");

    let loaded: Composition = serde_json::from_value(value).expect("旧形式の JSON を読めない");
    assert_eq!(
        loaded.background,
        Composition::default_background(),
        "background 欠落時は既定(不透明黒)へ落ちるはず"
    );
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
                ..Default::default()
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

/// レイヤー差し色(`label_color`、index 保存)が保存往復で消えない。
#[test]
fn label_color_round_trips_through_save_and_load() {
    let mut doc = authored();
    doc.apply(Intent::SetAttrs {
        layer: LayerId(1),
        patch: LayerAttrsPatch {
            label_color: Some(Some(9)),
            ..Default::default()
        },
    })
    .unwrap();

    let path = tmp("label_color_roundtrip.motolii");
    doc.save(&path).expect("保存できない");
    let loaded = Document::load(&path).expect("読み込めない");

    assert_eq!(
        loaded.view().attrs(LayerId(1)).unwrap().unwrap().label_color,
        Some(9),
        "label_color が保存往復で消えた/変わった"
    );
}

/// 旧保存ファイル(`label_color` component が無い版)を模した JSON でも読める —
/// `#[serde(default)]` の後方互換確認。`composition_without_a_background_field_
/// defaults_to_opaque_black` と同じ手口(JSON からキーを取り除いてから読み戻す)。
#[test]
fn attrs_without_a_label_color_field_defaults_to_unassigned() {
    let current = motolii_store::LayerAttrs {
        name: "旧ドキュメントの layer".to_owned(),
        ..Default::default()
    };
    let mut value = serde_json::to_value(&current).unwrap();
    value
        .as_object_mut()
        .expect("LayerAttrs は JSON object のはず")
        .remove("label_color")
        .expect("旧形式を模すには label_color キーが無いことが前提");

    let loaded: motolii_store::LayerAttrs =
        serde_json::from_value(value).expect("旧形式の JSON を読めない");
    assert_eq!(loaded.label_color, None, "label_color 欠落時は未割当へ落ちるはず");
}
