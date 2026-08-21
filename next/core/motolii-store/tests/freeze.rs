//! freeze/unfreeze 意図動詞(裁定119、G1 に続く「意図優先の原則」の第1切片)。
//!
//! 仕様源: `docs/reviews/2026-08-20-group-layer-semantics-decision.md` §4
//! (「フリーズ(可逆・性能)とフラット化(確定)は別の2操作」)・
//! `docs/reviews/2026-08-22-map-audit-rulings.md` queue 1(「freeze/unfreeze 動詞」)・
//! `docs/reviews/2026-08-22-intent-first-grouping-decision.md`(G1、同じ層の意図動詞)。
//!
//! ここで固定するのは:
//! - (a) freeze→unfreeze 往復で Document バイト不変(serde 往復・save/load 込み) —
//!   凍結は導出キャッシュの許可証であって Document データではない(裁定119 OUTCOME)
//! - (b) 凍結中の部分木(子孫)への編集 Intent は理由つき拒否。**凍結された Group
//!   自身**への編集(位置・改名等)は拒まない — 拒むのは「中身」だけ。
//!   グループ外の layer への編集は今まで通り通る
//! - (c) 凍結中の Group 自体: **Ungroup は拒否**(焼き込みが中身の track を書き換え
//!   うるので「中身への編集」の一種)・**削除(RemoveLayer)は許可**
//!   (tombstone なので可逆 — 解除して作り直すのと同じコストではない、undo で
//!   即座に戻せる)。論証はこの節の各テスト内コメントに書く
//! - (d) journal replay 決定論: 同じ Intent 列を2つの Document に順に適用すると、
//!   同じ観測可能な状態になる(Freeze/Unfreeze も他の Intent と同じ決定論を持つ)

use motolii_store::{
    property, Composition, Document, Fps, Intent, Interp, Keyframe, KeyframeTrack, LayerAttrs,
    LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime,
    StoreError, Value,
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
        background: Composition::default_background(),
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

/// `place` は Null 素材(group_ungroup.rs と同じ形)。`place_solid` は resolve が
/// 実際に絵を返す層が要る時用(solo_locked.rs と同じ形)。
fn place(doc: &mut Document, layer: LayerId, parent: Option<LayerId>) {
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Null,
                order: layer.0 as i16,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .unwrap();
    if let Some(parent) = parent {
        doc.apply(Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch {
                parent: Some(Some(parent)),
                ..Default::default()
            },
        })
        .unwrap();
    }
}

fn place_solid(doc: &mut Document, layer: LayerId, parent: Option<LayerId>) {
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: solid([255, 0, 0, 255]),
                order: layer.0 as i16,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .unwrap();
    if let Some(parent) = parent {
        doc.apply(Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch {
                parent: Some(Some(parent)),
                ..Default::default()
            },
        })
        .unwrap();
    }
}

fn freeze(doc: &mut Document, group: LayerId) -> Result<(), StoreError> {
    doc.apply(Intent::Freeze { group })
}

fn unfreeze(doc: &mut Document, group: LayerId) -> Result<(), StoreError> {
    doc.apply(Intent::Unfreeze { group })
}

fn attrs_or_default(doc: &Document, layer: LayerId) -> LayerAttrs {
    doc.view().attrs(layer).unwrap().unwrap_or_default()
}

/// 与えた layer 列の `meta`+`attrs` を JSON へ落として連結したもの。**store の
/// 物理的な framing(RowId/StoreId — 呼ぶたびに乱数)ではなく、Document の意味を
/// 運ぶ値だけ**を比べるための正準表現(`serde 往復含む` の oracle にはこれで足りる
/// — `LayerAttrs`/`LayerMeta` の JSON はどちらも値そのものしか持たない)。
fn semantic_snapshot(doc: &Document, layers: &[LayerId]) -> String {
    let mut parts = Vec::new();
    for &layer in layers {
        let meta = doc.view().meta(layer).unwrap();
        // `attrs()` の `None`(まだ一度も `SetAttrs` で書かれていない)と
        // `Some(LayerAttrs::default())` は**読み手側では同義**(裁定37、
        // `StoreView::attrs` の doc 参照) — freeze は最初の書き込みで `attrs`
        // component を作る副作用を持つので(read-modify-write が土台に
        // `LayerAttrs::default()` を使う)、`None → Some(default)` という
        // JSON 上の見た目の違いを「意味が変わった」と誤検出しないよう、ここでは
        // 常に defaulted な値で比べる。
        let attrs = attrs_or_default(doc, layer);
        parts.push(format!(
            "{}:{}:{}",
            layer.0,
            serde_json::to_string(&meta).unwrap(),
            serde_json::to_string(&attrs).unwrap()
        ));
    }
    parts.join("|")
}

// ---------------------------------------------------------------------------
// (a) freeze→unfreeze 往復で Document バイト不変
// ---------------------------------------------------------------------------

/// frozen 以外の全フィールドは触らない(read-modify-write)。往復後、group の
/// `attrs` JSON はフリーズ前と完全に同じ文字列に戻る。
#[test]
fn freeze_then_unfreeze_round_trips_the_groups_attrs_byte_for_byte() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    let group = doc.group_layers(&[a]).unwrap().unwrap();
    // frozen 以外にも何か非既定値を持たせて、read-modify-write が他フィールドを
    // 巻き込んでいないことも一緒に確かめる。
    doc.apply(Intent::SetAttrs {
        layer: group,
        patch: LayerAttrsPatch {
            name: Some("my group".to_owned()),
            label_color: Some(Some(2)),
            ..Default::default()
        },
    })
    .unwrap();

    let before = serde_json::to_string(&attrs_or_default(&doc, group)).unwrap();
    assert!(!attrs_or_default(&doc, group).frozen, "既定は未凍結のはず");

    freeze(&mut doc, group).expect("freeze は通るはず");
    assert!(attrs_or_default(&doc, group).frozen, "freeze が frozen を立てていない");
    unfreeze(&mut doc, group).expect("unfreeze は通るはず");

    let after = serde_json::to_string(&attrs_or_default(&doc, group)).unwrap();
    assert_eq!(
        before, after,
        "freeze→unfreeze 往復で attrs の JSON がバイト不変ではない"
    );
}

/// 往復後の絵(resolve)は完全に不変 — frozen は resolve のどの分岐にも入らない
/// (裁定119「Document の意味は1bitも変わらない」)。
#[test]
fn freeze_then_unfreeze_does_not_change_the_resolved_picture() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    doc.apply(Intent::SetTrack {
        layer: a,
        property: PropertyId::new(property::POSITION).unwrap(),
        track: still(Value::Vec2([12.0, -7.0])),
    })
    .unwrap();
    let group = doc.group_layers(&[a]).unwrap().unwrap();

    let before = doc.view().resolve(a, t(0)).unwrap().unwrap().placement.transform;

    freeze(&mut doc, group).unwrap();
    unfreeze(&mut doc, group).unwrap();

    let after = doc.view().resolve(a, t(0)).unwrap().unwrap().placement.transform;
    assert_eq!(before, after, "freeze→unfreeze 往復で絵が変わった");
}

/// serde 往復(save/load を経た実ファイル)込みでも意味が保たれる。フリーズ前の
/// Document と、フリーズ→アンフリーズしてから save/load した Document とで、
/// 全 layer の `meta`+`attrs` の正準表現(JSON、`semantic_snapshot`)が一致する。
#[test]
fn freeze_then_unfreeze_survives_a_save_and_load_round_trip() {
    let mut doc = doc_with_comp(300);
    let (a, b) = (LayerId(1), LayerId(2));
    place(&mut doc, a, None);
    place(&mut doc, b, None);
    let group = doc.group_layers(&[a, b]).unwrap().unwrap();
    let layers = [a, b, group];

    let before = semantic_snapshot(&doc, &layers);

    freeze(&mut doc, group).unwrap();
    unfreeze(&mut doc, group).unwrap();

    let dir = std::env::temp_dir().join(format!("motolii-freeze-fence-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("freeze_roundtrip.motolii");
    doc.save(&path).expect("保存できない");
    let loaded = Document::load(&path).expect("読み込めない");

    let after = semantic_snapshot(&loaded, &layers);
    assert_eq!(
        before, after,
        "freeze→unfreeze→save→load を経ると Document の意味が変わった"
    );
}

/// 既に frozen な Group への `Freeze` は冪等(再度 true を書くだけ、Err にならない)。
#[test]
fn freezing_an_already_frozen_group_is_idempotent() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    let group = doc.group_layers(&[a]).unwrap().unwrap();

    freeze(&mut doc, group).unwrap();
    freeze(&mut doc, group).expect("2回目の freeze も通るはず(冪等)");
    assert!(attrs_or_default(&doc, group).frozen);
}

/// 一度も frozen になっていない Group への `Unfreeze` も冪等(Err にならない)。
#[test]
fn unfreezing_a_never_frozen_group_is_a_no_op_ok() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    let group = doc.group_layers(&[a]).unwrap().unwrap();

    unfreeze(&mut doc, group).expect("凍結していない Group への unfreeze も通るはず");
    assert!(!attrs_or_default(&doc, group).frozen);
}

// ---------------------------------------------------------------------------
// (b) 凍結中の部分木への編集は拒否・グループ外/Group 自身は通る
// ---------------------------------------------------------------------------

/// `Intent::SetAttrs`(代表)。子への hidden 変更が拒まれ、理由(凍結)を含む。
#[test]
fn freezing_a_group_rejects_set_attrs_on_its_child() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    let group = doc.group_layers(&[a]).unwrap().unwrap();
    freeze(&mut doc, group).unwrap();

    let result = doc.apply(Intent::SetAttrs {
        layer: a,
        patch: LayerAttrsPatch {
            hidden: Some(true),
            ..Default::default()
        },
    });
    let err = result.expect_err("凍結中の子への SetAttrs は拒まれるはず");
    assert!(
        err.to_string().contains("凍結") || err.to_string().to_lowercase().contains("frozen"),
        "エラーは理由(frozen/凍結)を含むはず: {err}"
    );
    assert!(!attrs_or_default(&doc, a).hidden, "拒否されたのに hidden が変わっている");
}

/// `Intent::SetShapes`(発注書が名指しした代表その2)。
#[test]
fn freezing_a_group_rejects_set_shapes_on_its_child() {
    let mut doc = doc_with_comp(300);
    let shape_child = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(shape_child),
        Intent::SetMeta {
            layer: shape_child,
            meta: LayerMeta {
                source: LayerSource::Shape,
                order: 0,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .unwrap();
    let group = doc.group_layers(&[shape_child]).unwrap().unwrap();
    freeze(&mut doc, group).unwrap();

    let result = doc.apply(Intent::SetShapes {
        layer: shape_child,
        shapes: Vec::new(),
    });
    assert!(result.is_err(), "凍結中の子への SetShapes は拒まれるはず");
}

/// `Intent::SetTrack`(キーフレーム自体の変更も「中身」)。
#[test]
fn freezing_a_group_rejects_set_track_on_its_child() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    let group = doc.group_layers(&[a]).unwrap().unwrap();
    freeze(&mut doc, group).unwrap();

    let result = doc.apply(Intent::SetTrack {
        layer: a,
        property: PropertyId::new(property::POSITION).unwrap(),
        track: still(Value::Vec2([1.0, 1.0])),
    });
    assert!(result.is_err(), "凍結中の子への SetTrack は拒まれるはず");
}

/// `Intent::RemoveLayer`(子の削除も「中身」の編集 — グループ自体の削除とは別、
/// (c) 節参照)。
#[test]
fn freezing_a_group_rejects_remove_layer_of_its_child() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    let group = doc.group_layers(&[a]).unwrap().unwrap();
    freeze(&mut doc, group).unwrap();

    let result = doc.apply(Intent::RemoveLayer(a));
    assert!(result.is_err(), "凍結中の子への RemoveLayer は拒まれるはず");
    assert!(doc.view().has_layer(a), "拒否されたのに子が消えている");
}

/// 部分木は**入れ子越しに伝播する** — 孫(内側 Group の子)への編集も拒まれる。
#[test]
fn freeze_blocks_edits_to_grandchildren_through_a_nested_group() {
    let mut doc = doc_with_comp(300);
    let grandchild = LayerId(1);
    place(&mut doc, grandchild, None);
    let inner = doc.group_layers(&[grandchild]).unwrap().unwrap();
    let outer = doc.group_layers(&[inner]).unwrap().unwrap();
    freeze(&mut doc, outer).unwrap();

    let result = doc.apply(Intent::SetAttrs {
        layer: grandchild,
        patch: LayerAttrsPatch {
            hidden: Some(true),
            ..Default::default()
        },
    });
    assert!(
        result.is_err(),
        "外側 Group を凍結すると、内側 Group 越しの孫への編集も拒まれるはず"
    );
}

/// **凍結された Group 自身**への編集(名前を変える)は拒まない — 拒むのは「中身」
/// だけ(裁定119 §4「凍結中の中身への編集」、`LayerAttrs::frozen` の doc 参照)。
#[test]
fn freezing_does_not_block_editing_the_frozen_groups_own_attrs() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    let group = doc.group_layers(&[a]).unwrap().unwrap();
    freeze(&mut doc, group).unwrap();

    doc.apply(Intent::SetAttrs {
        layer: group,
        patch: LayerAttrsPatch {
            name: Some("still movable".to_owned()),
            ..Default::default()
        },
    })
    .expect("凍結された Group 自身の attrs 変更は拒まれないはず");
    assert_eq!(attrs_or_default(&doc, group).name, "still movable");
}

/// 凍結された Group 自身の位置(track)を動かすことも拒まれない — フリーズは
/// 「持ち手」としての Group の配置には触らない。
#[test]
fn freezing_does_not_block_moving_the_frozen_group_itself() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    let group = doc.group_layers(&[a]).unwrap().unwrap();
    freeze(&mut doc, group).unwrap();

    doc.apply(Intent::SetTrack {
        layer: group,
        property: PropertyId::new(property::POSITION).unwrap(),
        track: still(Value::Vec2([50.0, 50.0])),
    })
    .expect("凍結された Group 自身を動かすのは拒まれないはず");
}

/// **内側 Group の自己編集免除は、内側 Group が別の凍結された祖先の中に居るときは
/// 効かない** — 免除は「自分が frozen かどうか」ではなく「祖先に frozen が居るか」
/// で判定するので、外側が凍結されていれば内側 Group 自身の attrs も「外側の中身」
/// として拒まれる。
#[test]
fn freezing_the_outer_group_also_blocks_editing_the_inner_groups_own_attrs() {
    let mut doc = doc_with_comp(300);
    let leaf = LayerId(1);
    place(&mut doc, leaf, None);
    let inner = doc.group_layers(&[leaf]).unwrap().unwrap();
    let outer = doc.group_layers(&[inner]).unwrap().unwrap();
    freeze(&mut doc, outer).unwrap();

    let result = doc.apply(Intent::SetAttrs {
        layer: inner,
        patch: LayerAttrsPatch {
            name: Some("nope".to_owned()),
            ..Default::default()
        },
    });
    assert!(
        result.is_err(),
        "外側が凍結されていれば、内側 Group 自身への編集も拒まれるはず"
    );
}

/// グループ外の layer への編集は今まで通り通る(発注書の oracle そのもの)。
#[test]
fn freezing_a_group_does_not_affect_layers_outside_it() {
    let mut doc = doc_with_comp(300);
    let (a, outside) = (LayerId(1), LayerId(2));
    place(&mut doc, a, None);
    place(&mut doc, outside, None);
    let group = doc.group_layers(&[a]).unwrap().unwrap();
    freeze(&mut doc, group).unwrap();

    doc.apply(Intent::SetAttrs {
        layer: outside,
        patch: LayerAttrsPatch {
            hidden: Some(true),
            ..Default::default()
        },
    })
    .expect("グループ外の layer への編集は拒まれないはず");
    assert!(attrs_or_default(&doc, outside).hidden);
}

/// unfreeze すれば、以前拒まれていた子への編集が再び通る(黙って自動解凍しない
/// が、明示的に解凍すれば普通に戻る)。
#[test]
fn unfreezing_allows_editing_the_previously_frozen_child_again() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    let group = doc.group_layers(&[a]).unwrap().unwrap();
    freeze(&mut doc, group).unwrap();
    assert!(doc
        .apply(Intent::SetAttrs {
            layer: a,
            patch: LayerAttrsPatch {
                hidden: Some(true),
                ..Default::default()
            },
        })
        .is_err());

    unfreeze(&mut doc, group).unwrap();
    doc.apply(Intent::SetAttrs {
        layer: a,
        patch: LayerAttrsPatch {
            hidden: Some(true),
            ..Default::default()
        },
    })
    .expect("unfreeze 後は子への編集が通るはず");
    assert!(attrs_or_default(&doc, a).hidden);
}

/// 凍結中の Group(またはその部分木)を、まだ外に居る layer の新しい親にする
/// (= 新しい子を迎え入れる)ことも拒まれる — 部分木の中身を増やす編集の一種。
#[test]
fn reparenting_a_layer_into_a_frozen_group_is_rejected() {
    let mut doc = doc_with_comp(300);
    let (a, outsider) = (LayerId(1), LayerId(2));
    place(&mut doc, a, None);
    place(&mut doc, outsider, None);
    let group = doc.group_layers(&[a]).unwrap().unwrap();
    freeze(&mut doc, group).unwrap();

    let result = doc.apply(Intent::SetAttrs {
        layer: outsider,
        patch: LayerAttrsPatch {
            parent: Some(Some(group)),
            ..Default::default()
        },
    });
    assert!(
        result.is_err(),
        "凍結中の Group を新しい親にする(= 新しい子を迎える)のは拒まれるはず"
    );
    assert_eq!(
        attrs_or_default(&doc, outsider).parent,
        None,
        "拒否されたのに parent が変わっている"
    );
}

/// `Freeze`/`Unfreeze` 自体も locked な Group には効かない(他の層変更 Intent と
/// 同じ柵 `check_not_locked` を通る)。
#[test]
fn freeze_rejects_a_locked_group() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    let group = doc.group_layers(&[a]).unwrap().unwrap();
    doc.apply(Intent::SetAttrs {
        layer: group,
        patch: LayerAttrsPatch {
            locked: Some(true),
            ..Default::default()
        },
    })
    .unwrap();

    let result = freeze(&mut doc, group);
    let err = result.expect_err("locked な Group への freeze は拒まれるはず");
    assert!(err.to_string().contains("locked"));
}

/// `Freeze`/`Unfreeze` は `LayerSource::Group` でない layer には効かない(型では
/// 防げないので実行時に理由つき拒否)。
#[test]
fn freeze_rejects_a_non_group_layer() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);

    let result = freeze(&mut doc, a);
    assert!(result.is_err(), "Group でない layer への freeze は拒まれるはず");
}

/// 存在しない layer への freeze も理由つき拒否(パニックしない)。
#[test]
fn freeze_rejects_a_nonexistent_layer() {
    let mut doc = doc_with_comp(300);
    let result = freeze(&mut doc, LayerId(999));
    assert!(result.is_err(), "存在しない layer への freeze は拒まれるはず");
}

/// 祖先(外側の Group)が凍結中なら、内側の(まだ凍結されていない)Group 自身の
/// freeze/unfreeze も拒まれる — 凍結状態を動かすこと自体が「中身への編集」。
#[test]
fn freezing_a_group_whose_ancestor_is_already_frozen_is_rejected() {
    let mut doc = doc_with_comp(300);
    let leaf = LayerId(1);
    place(&mut doc, leaf, None);
    let inner = doc.group_layers(&[leaf]).unwrap().unwrap();
    let outer = doc.group_layers(&[inner]).unwrap().unwrap();
    freeze(&mut doc, outer).unwrap();

    let result = freeze(&mut doc, inner);
    assert!(
        result.is_err(),
        "外側が凍結中なら、内側 Group 自身の freeze も拒まれるはず"
    );
}

// ---------------------------------------------------------------------------
// (c) 凍結中の Group 自体: Ungroup は拒否・削除(RemoveLayer)は許可
// ---------------------------------------------------------------------------

/// **論証**: `Document::ungroup_layers` は Group の変換を子ローカルへ焼き込んで
/// から(`bake_child_local`)子の parent を書き換える。焼き込みは子の
/// position/rotation/scale/skew の **track を書き換えうる**(`BakedChildTransform`)
/// ので、これは「凍結中の中身への編集」そのものである。よって Ungroup は
/// **frozen な Group に対して理由つき拒否**する。黙って skip はしない
/// (`meta.source != Group` の「対象が違う」場合と区別する — こちらは「対象は
/// 正しいが今は無理」なので理由が要る、裁定119「黙って自動解凍しない」)。
#[test]
fn ungrouping_a_frozen_group_is_rejected() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    let group = doc.group_layers(&[a]).unwrap().unwrap();
    freeze(&mut doc, group).unwrap();

    let result = doc.ungroup_layers(&[group]);
    let err = result.expect_err("凍結中の Group への ungroup は拒まれるはず");
    assert!(
        err.to_string().contains("凍結") || err.to_string().to_lowercase().contains("frozen"),
        "エラーは理由(frozen/凍結)を含むはず: {err}"
    );

    // 実際に何も変わっていない(原子性/副作用ゼロ)ことも確かめる。
    assert!(doc.view().has_layer(group), "拒否されたのに Group が消えている");
    assert_eq!(
        attrs_or_default(&doc, a).parent,
        Some(group),
        "拒否されたのに子の parent が変わっている"
    );
}

/// unfreeze すれば ungroup は普通に通る(解凍すれば元の動詞がそのまま使える)。
#[test]
fn unfreezing_then_ungrouping_succeeds() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    let group = doc.group_layers(&[a]).unwrap().unwrap();
    freeze(&mut doc, group).unwrap();
    unfreeze(&mut doc, group).unwrap();

    let released = doc.ungroup_layers(&[group]).expect("unfreeze 後の ungroup は通るはず");
    assert_eq!(released, vec![a]);
}

/// **論証**: 削除(`Intent::RemoveLayer`)は tombstone を立てるだけで、子の
/// parent には一切触れない(`Document::write` の `RemoveLayer` 腕を参照 — present
/// フラグを false にするだけ)。Group 自身の中身(子の track)を一切書き換えない
/// ので「中身への編集」ではなく、しかも tombstone は undo で完全に可逆
/// (`docs/reviews/2026-08-20-group-layer-semantics-decision.md` の削除=最も破壊的
/// だが undo で戻る、という既存原則そのまま)。よって凍結中でも削除は**許可**する
/// — Ungroup と非対称なのは意図的(裁定119レーン仕様の推奨どおり)。
#[test]
fn deleting_a_frozen_group_itself_is_allowed() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    let group = doc.group_layers(&[a]).unwrap().unwrap();
    freeze(&mut doc, group).unwrap();

    doc.apply(Intent::RemoveLayer(group))
        .expect("凍結中の Group 自身への RemoveLayer は許可されるはず");
    assert!(!doc.view().has_layer(group), "削除したのに Group が残っている");

    // undo で完全に戻る(可逆性の直接確認)。
    assert!(doc.undo());
    assert!(doc.view().has_layer(group), "undo で Group が戻っていない");
    assert!(attrs_or_default(&doc, group).frozen, "undo で frozen 状態も戻るはず");
}

// ---------------------------------------------------------------------------
// (d) journal replay 決定論
// ---------------------------------------------------------------------------

/// 同じ Intent 列(Group 化・freeze・グループ外の編集・unfreeze・子の編集)を
/// 2つの独立な Document に順に適用すると、観測可能な状態(全 layer の
/// `meta`+`attrs` の正準表現)が一致する。Freeze/Unfreeze が journal replay の
/// 決定論を壊していないことの直接証拠。
#[test]
fn freeze_and_unfreeze_replay_deterministically() {
    fn build() -> (Document, LayerId, LayerId, LayerId) {
        let mut doc = doc_with_comp(300);
        let (a, b) = (LayerId(1), LayerId(2));
        place_solid(&mut doc, a, None);
        place_solid(&mut doc, b, None);
        let group = doc.group_layers(&[a]).unwrap().unwrap();
        doc.apply(Intent::Freeze { group }).unwrap();
        doc.apply(Intent::SetAttrs {
            layer: b,
            patch: LayerAttrsPatch {
                hidden: Some(true),
                ..Default::default()
            },
        })
        .unwrap();
        doc.apply(Intent::Unfreeze { group }).unwrap();
        doc.apply(Intent::SetAttrs {
            layer: a,
            patch: LayerAttrsPatch {
                name: Some("after unfreeze".to_owned()),
                ..Default::default()
            },
        })
        .unwrap();
        (doc, a, b, group)
    }

    let (doc1, a1, b1, group1) = build();
    let (doc2, a2, b2, group2) = build();
    assert_eq!(a1, a2);
    assert_eq!(b1, b2);
    assert_eq!(group1, group2);

    let layers = [a1, b1, group1];
    assert_eq!(
        semantic_snapshot(&doc1, &layers),
        semantic_snapshot(&doc2, &layers),
        "同じ Intent 列を replay しても Freeze/Unfreeze を含む終状態が一致しない"
    );
}
