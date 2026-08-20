//! `Document::flattened()` の柵(裁定108(a))。
//!
//! `persist.rs::flattened()` は元々、保存対象を **component 名で手で列挙**していた
//! (`meta`/`masks`/`markers`/`composition`/property track)。この形は
//! **component を1つ足しても、その事実に誰も気付けない**まま保存から黙って消える —
//! テストの網羅も同じ手列挙で書けば「揃っている」ことになってしまい、柵にならない。
//!
//! そこでこの試験は **layer-meta 束で新しく足した component(`Layer:attrs`)** を使う。
//! `flattened()` が「store に存在する component を機械的に全部運ぶ」形(裁定57)に
//! なっていれば、`attrs` の存在を1行も知らずに通る。手列挙のままなら
//! (`attrs` を運ぶ分岐が無いので)ここで確実に落ちる。

use motolii_store::{
    BlendMode, Composition, Document, Fps, Intent, LayerAttrs, LayerId, LayerMeta, LayerSource,
    LayerTiming,
};

fn doc_with_layer_and_attrs() -> (Document, LayerId) {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
    }))
    .unwrap();

    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Solid {
                    rgba: [10, 20, 30, 255],
                    width: 64,
                    height: 64,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .unwrap();

    doc.apply(Intent::SetAttrs {
        layer,
        attrs: LayerAttrs {
            hidden: false,
            parent: None,
            blend_mode: BlendMode::Multiply,
            matte: None,
            name: "circle".to_owned(),
            auto_orient: true,
        },
    })
    .unwrap();

    (doc, layer)
}

/// 柵そのもの。**この試験が今の手列挙でも通ってしまうなら柵になっていない**
/// (発注書の注意書きどおり) — `attrs` は手列挙の対象にまだ入っていない component。
#[test]
fn flattened_carries_every_component_the_store_has_not_just_the_named_ones() {
    let (doc, layer) = doc_with_layer_and_attrs();

    let flat = doc.flattened().expect("flattened() が失敗した");
    let attrs = flat
        .view()
        .attrs(layer)
        .expect("attrs の読み出しに失敗した")
        .expect("flattened() が Layer:attrs component を運んでいない(裁定108(a) の柵が機能していない)");

    assert_eq!(attrs.name, "circle", "attrs.name が flatten で消えた");
    assert_eq!(attrs.blend_mode, BlendMode::Multiply, "attrs.blend_mode が flatten で消えた");
    assert!(attrs.auto_orient, "attrs.auto_orient が flatten で消えた");
}

/// 同じ柵を save/load(実ファイル)経由でも確かめる — `flattened()` は `save()` の
/// 内部でも呼ばれるので、ファイルへ実際に書いて読み直しても消えないことを見る。
#[test]
fn save_and_load_round_trips_attrs_too() {
    let (doc, layer) = doc_with_layer_and_attrs();

    let dir = std::env::temp_dir().join(format!("motolii-flatten-fence-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("attrs_roundtrip.motolii");
    doc.save(&path).expect("保存できない");

    let loaded = Document::load(&path).expect("読み込めない");
    let attrs = loaded
        .view()
        .attrs(layer)
        .unwrap()
        .expect("保存/読込を経ると Layer:attrs が消えている");
    assert_eq!(attrs.name, "circle");
}
