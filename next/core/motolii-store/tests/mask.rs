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
        spatial: None,
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
        background: [0.0, 0.0, 0.0, 1.0],
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
///
/// **2026-08-22(裁定206 に続く壁7の恒久修正)**: 以前はここで `SetMasks`(一覧に
/// push)→`SetTrack`(shape)の2 intent を `apply_all` で束ねていた。その順序
/// (一覧を先に更新)は `Intent::SetMasks` 自身が「新しく現れる id は shape が
/// 既に読めることを要求する」よう変わったため、今この順序のままだと
/// `SetMasks` が `Err` を返す——`Intent::AddMask` 1本(一覧への追加と shape の
/// 初期値を同じ `write()` 呼び出しで束ねる、順序に頼らない原子操作)に置き換えた。
fn add_mask(doc: &mut Document, layer: LayerId, mask: Mask, x: f64) {
    doc.apply(Intent::AddMask {
        layer,
        mask,
        shape: still(Value::Path(marker_path(x))),
    })
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
        spatial: None,
    });
    track.insert(Keyframe {
        t: t(30),
        value: Value::Path(marker_path(100.0)),
        interp: Interp::Linear,
        spatial: None,
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

/// **2026-08-22(壁7の恒久修正)以前の挙動だった**: 以前は `SetMasks` に新規 id を
/// 素通しさせ、`resolve()` の時になって初めて `Err` を返していた(既定値へ静かに
/// 落とすと利用者には「マスクが勝手に消えた」としか見えない、裁定37)。今は
/// `SetMasks` 自身が書き込み時にこの状態を作れなくする(`AddMask` が導入された
/// 理由そのもの)——`resolved_masks` 自体の型検査(パスでない値が入っている等)は
/// 従来どおり resolve 時のまま(ここが変わったのは「shape が無い」の一点だけ)。
#[test]
fn set_masks_rejects_a_new_mask_without_a_shape_at_write_time() {
    let (mut doc, layer) = doc_with_layer();
    let result = doc.apply(Intent::SetMasks {
        layer,
        masks: vec![Mask {
            id: MaskId(0),
            mode: MaskMode::Add,
            inverted: false,
        }],
    });

    assert!(
        result.is_err(),
        "shape の無い新規マスクが SetMasks を素通りしている"
    );
    // 拒まれた書き込みは何も残さない——マスクは1枚も置かれていないはず。
    assert!(doc.view().masks(layer).unwrap().is_empty());
}

/// `AddMask` は「一覧への追加」と「shape の初期値」を1つの `write()` 呼び出しで
/// 束ねるので、shape の無いマスクが生まれる中間状態が物理的に存在しない——
/// 追加した直後から常に resolve が通る。
#[test]
fn add_mask_never_produces_a_shapeless_mask() {
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
    assert!(
        doc.view().resolve(layer, t(0)).is_ok(),
        "AddMask で足したマスクは shape を伴うので resolve が通るはず"
    );
}

/// `SetMasks` で新規追加する場合でも、**同じ `apply_all` の中で先に shape の
/// `SetTrack` を書けば通る**(§ `Intent::SetMasks` の doc 参照 — 各 intent は
/// 同じ edit 刻みで逐次 ingest されるので、後続 intent の検査は先行 intent の
/// 結果を見る)。`AddMask` の方が1回で済むが、`apply_all` での手動束ねも壊れて
/// いないことを固定する。
#[test]
fn set_masks_accepts_a_new_mask_when_the_shape_is_written_first_in_the_same_batch() {
    let (mut doc, layer) = doc_with_layer();
    let mask = Mask {
        id: MaskId(0),
        mode: MaskMode::Add,
        inverted: false,
    };
    doc.apply_all([
        Intent::SetTrack {
            layer,
            property: PropertyId::mask_shape(mask.id),
            track: still(Value::Path(marker_path(0.0))),
        },
        Intent::SetMasks {
            layer,
            masks: vec![mask],
        },
    ])
    .unwrap();

    assert!(doc.view().resolve(layer, t(0)).is_ok());
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

/// 膨張は shape/opacity と同じ **property track**(B02、2026-08-22、Lottie
/// `helpers/mask/x`)。キーを打っていない = 既定 0(無効)。`Mask` struct には
/// フィールドを足していないので、静的な `Mask` ではなく解決済み property を
/// 直接読んで確かめる(`ResolvedMask` へは resolve 時に既定0として運ばれる)。
#[test]
fn mask_expansion_defaults_to_zero_until_a_track_is_written() {
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

    // キーを一度も打っていない = track が無い = 既定 0(無効)。
    assert_eq!(
        doc.view()
            .value_at(layer, &PropertyId::mask_expansion(MaskId(0)), t(0))
            .unwrap(),
        None,
        "expansion の track を書く前から値が生えている"
    );

    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::mask_expansion(MaskId(0)),
        track: still(Value::F64(-5.0)),
    })
    .unwrap();

    assert_eq!(
        doc.view()
            .value_at(layer, &PropertyId::mask_expansion(MaskId(0)), t(0))
            .unwrap(),
        Some(Value::F64(-5.0)),
        "expansion は負(内側への収縮)も持てる — Lottie/AE と同じ符号"
    );
}

/// 膨張は shape と同じ `KeyframeTrack` — キーを打てば動く(AE 実機のタイムライン
/// 挙動と同じ、Mask Opacity/Shape とここが並ぶことが B02 の主張)。
#[test]
fn mask_expansion_interpolates_between_keys_like_any_other_track() {
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
        value: Value::F64(0.0),
        interp: Interp::Linear,
        spatial: None,
    });
    track.insert(Keyframe {
        t: t(30),
        value: Value::F64(100.0),
        interp: Interp::Linear,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::mask_expansion(MaskId(0)),
        track,
    })
    .unwrap();

    let mid = doc
        .view()
        .value_at(layer, &PropertyId::mask_expansion(MaskId(0)), t(15))
        .unwrap()
        .expect("キーの間なので値が居るはず");
    assert_eq!(mid, Value::F64(50.0), "膨張が補間されていない");
}

/// 保存は履歴を畳む(裁定56)。expansion も他の property track と同じ形
/// (`Layer:mask.{id}.expansion` component)なので、`flattened`/`track_json_components`
/// は component 名を列挙していない(`persist.rs` 参照)— **この試験を書かないと
/// 退行が黙って通る**。
#[test]
fn mask_expansion_survives_a_save_and_load_round_trip() {
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
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::mask_expansion(MaskId(0)),
        track: still(Value::F64(3.5)),
    })
    .unwrap();

    let dir = std::env::temp_dir().join(format!("motolii-mask-expansion-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("mask-expansion.rrd");
    doc.save(&file).unwrap();
    let loaded = Document::load(&file).unwrap();

    assert_eq!(
        loaded
            .view()
            .value_at(layer, &PropertyId::mask_expansion(MaskId(0)), t(0))
            .unwrap(),
        Some(Value::F64(3.5)),
        "保存で畳む時に expansion が落ちている"
    );
}

/// **旧 bit の読み込み**: expansion の track を一度も書いていない旧保存を読み戻しても、
/// `Mask` struct 自体は形が変わっていないので普通に読める(`Mask` にフィールドを
/// 足していない — shape/opacity と同じ形、B02)。
#[test]
fn documents_saved_before_expansion_existed_still_load() {
    let (mut doc, layer) = doc_with_layer();
    add_mask(
        &mut doc,
        layer,
        Mask {
            id: MaskId(7),
            mode: MaskMode::Intersect,
            inverted: true,
        },
        4.0,
    );
    // expansion の track を意図的に書かない — 旧保存の再現。

    let dir = std::env::temp_dir().join(format!("motolii-mask-old-bit-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("mask-old-bit.rrd");
    doc.save(&file).unwrap();
    let loaded = Document::load(&file).unwrap();

    let masks = loaded.view().masks(layer).unwrap();
    assert_eq!(
        masks,
        vec![Mask {
            id: MaskId(7),
            mode: MaskMode::Intersect,
            inverted: true,
        }],
        "旧保存(expansion なし)の Mask が変わってしまっている"
    );
    assert_eq!(
        loaded
            .view()
            .value_at(layer, &PropertyId::mask_expansion(MaskId(7)), t(0))
            .unwrap(),
        None,
        "旧保存に無い track が読み込みで生えてはいけない(既定は resolve 側が 0 として扱う)"
    );
    // resolve された expansion も既定0で保持される。
    let resolved = loaded.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert_eq!(resolved.masks[0].shape.vertices[0].point[0], 4.0);
    assert_eq!(resolved.masks[0].expansion, 0.0);
}

/// マスクの expansion 編集も `edit` timeline 上の普通の刻み。
#[test]
fn editing_mask_expansion_is_undoable_like_any_other_edit() {
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

    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::mask_expansion(MaskId(0)),
        track: still(Value::F64(20.0)),
    })
    .unwrap();
    assert_eq!(
        doc.view()
            .value_at(layer, &PropertyId::mask_expansion(MaskId(0)), t(0))
            .unwrap(),
        Some(Value::F64(20.0))
    );

    assert!(doc.undo());
    assert_eq!(
        doc.view()
            .value_at(layer, &PropertyId::mask_expansion(MaskId(0)), t(0))
            .unwrap(),
        None,
        "expansion の設定が undo で戻らない"
    );
    assert!(doc.redo());
    assert_eq!(
        doc.view()
            .value_at(layer, &PropertyId::mask_expansion(MaskId(0)), t(0))
            .unwrap(),
        Some(Value::F64(20.0)),
        "expansion の設定が redo で戻らない"
    );
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
