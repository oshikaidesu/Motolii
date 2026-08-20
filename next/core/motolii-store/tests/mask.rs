//! マスクの意味 — layer 内をパスで切り抜く。
//!
//! matte(他 layer の alpha で抜く)とは別物で、こちらは **layer が自分で持つパス**。
//!
//! ここで固定するのは4つ:
//! - `None` はモードではない(消すことは「重ね方」ではない)
//! - マスクは**同一性**を持つ。並べ替えても消しても、形状トラックが別のマスクへ付き直さない
//! - 不透明度は**比**(1.0 基準)であってパーセントではない
//! - 形状が無いマスクは**黙って飛ばさず**理由を返す(裁定37)

use motolii_store::{
    Composition, Document, Fps, Interp, Intent, Keyframe, KeyframeTrack, LayerId, LayerMeta,
    LayerSource, LayerTiming, Mask, MaskId, MaskMode, Path, PathVertex, PropertyId, RationalTime,
    Value,
};

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

/// 頂点1つだけの目印パス。**どのマスクの形状か**を x 座標で見分ける。
fn marker_path(x: f64) -> Path {
    Path {
        vertices: vec![PathVertex {
            point: [x, 0.0],
            in_tangent: [0.0, 0.0],
            out_tangent: [0.0, 0.0],
        }],
        closed: true,
    }
}

fn still(value: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value,
        interp: Interp::Hold,
    });
    track
}

/// comp と layer を1つ置いた Document。
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

/// マスクを1つ足す(静止した形状つき)。**追加と形状は1操作 = 1 undo**。
fn add_mask(doc: &mut Document, layer: LayerId, mask: Mask, x: f64) {
    let mut masks = doc.view().masks(layer).unwrap();
    masks.push(mask);
    doc.apply_all([
        Intent::SetMasks { layer, masks },
        Intent::SetTrack {
            layer,
            property: PropertyId::mask_shape(mask.id),
            track: still(Value::Path(marker_path(x))),
        },
    ])
    .unwrap();
}

/// **`None` はモードではない。** マスクを消すことと、重ね方を選ぶことは別の操作である。
#[test]
fn none_is_not_a_mask_mode() {
    for (name, mode) in [
        ("\"Add\"", MaskMode::Add),
        ("\"Subtract\"", MaskMode::Subtract),
        ("\"Intersect\"", MaskMode::Intersect),
        ("\"Lighten\"", MaskMode::Lighten),
        ("\"Darken\"", MaskMode::Darken),
        ("\"Difference\"", MaskMode::Difference),
    ] {
        let back: MaskMode = serde_json::from_str(name).expect("6値は読めるはず");
        assert_eq!(back, mode);
        assert_eq!(serde_json::to_string(&mode).unwrap(), name);
    }

    assert!(
        serde_json::from_str::<MaskMode>("\"None\"").is_err(),
        "`None` を列挙へ混ぜると「消す」が「重ね方」として保存できてしまう"
    );
}

/// **同一性の試験**(裁定65 と同型)。
///
/// マスクを添字で持つと、真ん中を1つ消した瞬間に3枚目の形状が2枚目へ付き直す。
/// 「並べ替え・削除が結果を黙って変える」形なので、`MaskId` で縛る。
#[test]
fn masks_keep_their_shape_when_a_sibling_is_removed() {
    let (mut doc, layer) = doc_with_layer();
    for i in 0..3u32 {
        add_mask(
            &mut doc,
            layer,
            Mask {
                id: MaskId(i),
                mode: MaskMode::Add,
                inverted: false,
            },
            i as f64,
        );
    }

    // 真ん中を消す。**形状トラックは残す**(undo で戻るべきなので墓標と同じ考え方)。
    let kept: Vec<Mask> = doc
        .view()
        .masks(layer)
        .unwrap()
        .into_iter()
        .filter(|m| m.id != MaskId(1))
        .collect();
    doc.apply(Intent::SetMasks {
        layer,
        masks: kept,
    })
    .unwrap();

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    let xs: Vec<f64> = resolved
        .masks
        .iter()
        .map(|m| m.shape.vertices[0].point[0])
        .collect();
    assert_eq!(
        xs,
        vec![0.0, 2.0],
        "添字で持つと 3枚目の形状が 2枚目へ付き直す"
    );
}

/// スタックの順序は**明示された列の順**。暗黙の隣接参照を作らない(裁定66)。
#[test]
fn the_stack_order_is_the_declared_order() {
    let (mut doc, layer) = doc_with_layer();
    for i in 0..3u32 {
        add_mask(
            &mut doc,
            layer,
            Mask {
                id: MaskId(i),
                mode: MaskMode::Add,
                inverted: false,
            },
            i as f64,
        );
    }

    let mut reordered = doc.view().masks(layer).unwrap();
    reordered.reverse();
    doc.apply(Intent::SetMasks {
        layer,
        masks: reordered,
    })
    .unwrap();

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    let xs: Vec<f64> = resolved
        .masks
        .iter()
        .map(|m| m.shape.vertices[0].point[0])
        .collect();
    assert_eq!(xs, vec![2.0, 1.0, 0.0], "並べ替えが解決結果へ出ていない");
}

/// 同じ id のマスクを2枚置けると、形状トラックの持ち主が決まらない。
#[test]
fn duplicate_mask_ids_are_rejected() {
    let (mut doc, layer) = doc_with_layer();
    let masks = vec![
        Mask {
            id: MaskId(0),
            mode: MaskMode::Add,
            inverted: false,
        },
        Mask {
            id: MaskId(0),
            mode: MaskMode::Subtract,
            inverted: false,
        },
    ];
    assert!(
        doc.apply(Intent::SetMasks { layer, masks }).is_err(),
        "同じ id が2枚通ると、形状トラックがどちらの物か決まらない"
    );
}

/// **不透明度は比**(1.0 基準)。Lottie のパーセントは採らない(裁定58/65)。
#[test]
fn mask_opacity_is_a_ratio_not_a_percentage() {
    let (mut doc, layer) = doc_with_layer();
    add_mask(
        &mut doc,
        layer,
        Mask {
            id: MaskId(0),
            mode: MaskMode::Add,
            inverted: false,
        },
        0.0,
    );

    // キーを打っていない property は静止値(裁定20)。
    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert_eq!(resolved.masks[0].opacity, 1.0, "既定は不透明");

    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::mask_opacity(MaskId(0)),
        track: still(Value::F64(0.25)),
    })
    .unwrap();
    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert_eq!(
        resolved.masks[0].opacity, 0.25,
        "0.25 が 25% として解釈されている"
    );

    // 範囲外は layer の不透明度と同じく畳む。
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::mask_opacity(MaskId(0)),
        track: still(Value::F64(100.0)),
    })
    .unwrap();
    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert_eq!(resolved.masks[0].opacity, 1.0);
}

/// **反転は Subtract ではない。** 両方が別々に保存できること。
#[test]
fn inverted_is_not_the_same_as_subtract() {
    let (mut doc, layer) = doc_with_layer();
    add_mask(
        &mut doc,
        layer,
        Mask {
            id: MaskId(0),
            mode: MaskMode::Add,
            inverted: true,
        },
        0.0,
    );
    add_mask(
        &mut doc,
        layer,
        Mask {
            id: MaskId(1),
            mode: MaskMode::Subtract,
            inverted: false,
        },
        1.0,
    );

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert_eq!(resolved.masks[0].mode, MaskMode::Add);
    assert!(resolved.masks[0].inverted);
    assert_eq!(resolved.masks[1].mode, MaskMode::Subtract);
    assert!(!resolved.masks[1].inverted);
}

/// 形状は **`Value::Path` のトラック**なので、キーを打てば動く。
#[test]
fn the_mask_shape_interpolates_between_keys() {
    let (mut doc, layer) = doc_with_layer();
    add_mask(
        &mut doc,
        layer,
        Mask {
            id: MaskId(0),
            mode: MaskMode::Add,
            inverted: false,
        },
        0.0,
    );

    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value: Value::Path(marker_path(0.0)),
        interp: Interp::Linear,
    });
    track.insert(Keyframe {
        t: t(30),
        value: Value::Path(marker_path(100.0)),
        interp: Interp::Linear,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::mask_shape(MaskId(0)),
        track,
    })
    .unwrap();

    let mid = doc.view().resolve(layer, t(15)).unwrap().expect("居る");
    assert!(
        (mid.masks[0].shape.vertices[0].point[0] - 50.0).abs() < 1e-9,
        "形状が補間されていない: {:?}",
        mid.masks[0].shape
    );
}

/// 形状の無いマスクは**壊れた Document**。既定値へ静かに落とすと
/// 利用者には「マスクが勝手に消えた」としか見えない(裁定37)。
#[test]
fn a_mask_without_a_shape_is_an_error_not_a_silent_skip() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetMasks {
        layer,
        masks: vec![Mask {
            id: MaskId(0),
            mode: MaskMode::Add,
            inverted: false,
        }],
    })
    .unwrap();

    assert!(
        doc.view().resolve(layer, t(0)).is_err(),
        "形状の無いマスクを黙って飛ばしている"
    );
}

/// マスクの編集も `edit` timeline 上の普通の刻み。自前の履歴機構を持たない。
#[test]
fn editing_masks_is_undoable_like_any_other_edit() {
    let (mut doc, layer) = doc_with_layer();
    add_mask(
        &mut doc,
        layer,
        Mask {
            id: MaskId(0),
            mode: MaskMode::Add,
            inverted: false,
        },
        0.0,
    );
    assert_eq!(doc.view().masks(layer).unwrap().len(), 1);

    assert!(doc.undo());
    assert!(
        doc.view().masks(layer).unwrap().is_empty(),
        "マスクの追加が undo で戻らない"
    );
    assert!(doc.redo());
    assert_eq!(doc.view().masks(layer).unwrap()[0].mode, MaskMode::Add);
}

/// 保存は履歴を畳む(裁定56)ので、**畳む口が知らない component は消える**。
/// マスクは `meta` の中に無いぶん、ここで固定しておかないと黙って落ちる。
#[test]
fn masks_survive_a_save_and_load_round_trip() {
    let (mut doc, layer) = doc_with_layer();
    add_mask(
        &mut doc,
        layer,
        Mask {
            id: MaskId(5),
            mode: MaskMode::Intersect,
            inverted: true,
        },
        3.0,
    );

    let dir = std::env::temp_dir().join(format!("motolii-mask-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("masks.rrd");
    doc.save(&file).unwrap();
    let loaded = Document::load(&file).unwrap();

    let masks = loaded.view().masks(layer).unwrap();
    assert_eq!(
        masks,
        vec![Mask {
            id: MaskId(5),
            mode: MaskMode::Intersect,
            inverted: true,
        }],
        "保存で畳む時にマスクが落ちている"
    );
    let resolved = loaded.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert_eq!(resolved.masks[0].shape.vertices[0].point[0], 3.0);
}

/// property の一覧は **store に聞く**(裁定57)。マスクの形状トラックもそこに並ぶ。
/// 一方 `masks` は layer 自身の component なので property ではない(裁定35)。
#[test]
fn mask_tracks_appear_in_the_property_list_but_the_mask_list_does_not() {
    let (mut doc, layer) = doc_with_layer();
    add_mask(
        &mut doc,
        layer,
        Mask {
            id: MaskId(0),
            mode: MaskMode::Add,
            inverted: false,
        },
        0.0,
    );

    let names: Vec<String> = doc
        .view()
        .properties(layer)
        .iter()
        .map(|p| p.name().to_owned())
        .collect();
    assert!(
        names.contains(&"mask.0.shape".to_owned()),
        "マスクの形状が property 一覧に出ていない: {names:?}"
    );
    assert!(
        !names.iter().any(|n| n == "masks"),
        "layer 自身の component が property に混ざっている: {names:?}"
    );
    assert!(
        PropertyId::new("masks").is_err(),
        "`masks` を property 名に使えるとマスク一覧を上書きできる"
    );
}
