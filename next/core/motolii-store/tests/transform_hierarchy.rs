//! 裁定173 H1 — 変換階層(親が動くと子も動く)。
//!
//! ここで固定するのは:
//! - 親移動→子の最終位置の数値証明(平行移動のみ・回転+スケールを含む合成順)
//! - 壊れた parent 鎖(親が tombstone/存在しない)への安全な縮退(ローカルのみへ)
//! - `LayerSource::Group` の serde 往復(旧 JSON との混在・`parent` 無し旧 Document の不変)
//! - 保存/読込を経由しても親子合成が保たれる
//!
//! **循環 Intent の拒否**は既に `tests/layer_meta.rs::parent_chain_cannot_form_a_cycle`/
//! `parent_cannot_point_at_itself` が固定済み(このスライスで書き口を一切変えていない
//! ので、そちらの oracle が無改造のまま緑であること自体が防衛テストになる)。

use motolii_store::{
    property, Composition, Document, Fps, Intent, Interp, Keyframe, KeyframeTrack, LayerAttrs,
    LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value,
};

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
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

/// [`place`] は既定で `LayerSource::Null`。`LayerSource::Group` を使うテストは
/// これで差し替える(`SetSource` は既存 layer の素材だけを差し替える口、
/// `tests/layer_meta.rs::set_source_changes_only_the_source` と同じ経路)。
fn make_group(doc: &mut Document, layer: LayerId) {
    doc.apply(Intent::SetSource {
        layer,
        source: LayerSource::Group,
    })
    .unwrap();
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

fn set_scalar(doc: &mut Document, layer: LayerId, name: &str, value: f64) {
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::new(name).unwrap(),
        track: still(Value::F64(value)),
    })
    .unwrap();
}

fn set_vec2(doc: &mut Document, layer: LayerId, name: &str, value: [f64; 2]) {
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::new(name).unwrap(),
        track: still(Value::Vec2(value)),
    })
    .unwrap();
}

fn approx(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-4, "{a} != {b}");
}

// ---------------------------------------------------------------------------
// 数値証明1: 平行移動のみ(裁定173 H1 oracle に書かれた具体例そのもの)
// ---------------------------------------------------------------------------

/// 親 position (100,0) + 子ローカル (10,5) → 子 world (110,5)。
#[test]
fn parent_translation_moves_the_childs_final_world_position() {
    let mut doc = doc_with_comp(300);
    let (parent, child) = (LayerId(1), LayerId(2));
    place(&mut doc, parent, None);
    place(&mut doc, child, Some(parent));

    set_vec2(&mut doc, parent, property::POSITION, [100.0, 0.0]);
    set_vec2(&mut doc, child, property::POSITION, [10.0, 5.0]);

    let resolved = doc.view().resolve(child, t(0)).unwrap().expect("居る");
    let world = resolved.placement.transform.translation;
    approx(world.x, 110.0);
    approx(world.y, 5.0);
}

/// 親を動かすと(140,0)へ)子の最終位置も追従する — 「親が動くと子も動く」の
/// 名前どおりの oracle。
#[test]
fn moving_the_parent_moves_the_child_too() {
    let mut doc = doc_with_comp(300);
    let (parent, child) = (LayerId(1), LayerId(2));
    place(&mut doc, parent, None);
    place(&mut doc, child, Some(parent));

    set_vec2(&mut doc, parent, property::POSITION, [100.0, 0.0]);
    set_vec2(&mut doc, child, property::POSITION, [10.0, 5.0]);
    let before = doc.view().resolve(child, t(0)).unwrap().unwrap();
    approx(before.placement.transform.translation.x, 110.0);

    // 親を +40 動かす。
    set_vec2(&mut doc, parent, property::POSITION, [140.0, 0.0]);
    let after = doc.view().resolve(child, t(0)).unwrap().unwrap();
    approx(after.placement.transform.translation.x, 150.0);
    approx(after.placement.transform.translation.y, 5.0);
}

// ---------------------------------------------------------------------------
// 数値証明2: 回転・スケールを含む合成順は1本(手計算で独立に検算)
// ---------------------------------------------------------------------------

/// 親: position(100,0)・rotation 90°・scale(2,1)(非一様)。子ローカル: position
/// (10,5)。
///
/// 手計算: 親の local 行列は `T(100,0)·R(90°)·S(2,1)`(裁定58 の適用順、anchor=0・
/// skew=0)。子は純平行移動なので、world 変換の並進成分は
/// `parent_local.transform_point2(child_local_position)` に一致する:
/// `S(2,1)·(10,5) = (20,5)` → `R(90°)·(20,5) = (-5,20)`(cos90=0, sin90=1 の
/// 標準回転行列)→ `+ (100,0) = (95,20)`。
///
/// もし実装が合成順を取り違えていたら(例: 親を先にではなく後に掛ける、または
/// scale と rotation の順を逆にする)、この値とはズレる。
#[test]
fn rotation_and_scale_compose_in_the_documented_order() {
    let mut doc = doc_with_comp(300);
    let (parent, child) = (LayerId(1), LayerId(2));
    place(&mut doc, parent, None);
    place(&mut doc, child, Some(parent));

    set_vec2(&mut doc, parent, property::POSITION, [100.0, 0.0]);
    set_scalar(&mut doc, parent, property::ROTATION, 90.0);
    set_vec2(&mut doc, parent, property::SCALE, [2.0, 1.0]);

    set_vec2(&mut doc, child, property::POSITION, [10.0, 5.0]);

    let resolved = doc.view().resolve(child, t(0)).unwrap().expect("居る");
    let world = resolved.placement.transform.translation;
    approx(world.x, 95.0);
    approx(world.y, 20.0);
}

// ---------------------------------------------------------------------------
// 壊れた parent 鎖への安全な縮退(H2 の到達性判定と同じ意味論)
// ---------------------------------------------------------------------------

/// 親が tombstone(`RemoveLayer` で present=false)になったら、子はローカルのみへ
/// 縮退する — `projection.rs::rows` の `attrs.parent.filter(|p| present.contains(p))`
/// と同じ意味論。
#[test]
fn a_tombstoned_parent_falls_back_to_the_childs_local_transform_only() {
    let mut doc = doc_with_comp(300);
    let (parent, child) = (LayerId(1), LayerId(2));
    place(&mut doc, parent, None);
    place(&mut doc, child, Some(parent));

    set_vec2(&mut doc, parent, property::POSITION, [100.0, 0.0]);
    set_vec2(&mut doc, child, property::POSITION, [10.0, 5.0]);

    // 縮退前: 親の位置が乗って (110,5)。
    let before = doc.view().resolve(child, t(0)).unwrap().unwrap();
    approx(before.placement.transform.translation.x, 110.0);

    doc.apply(Intent::RemoveLayer(parent)).unwrap();

    // 親は tombstone(present ではない)。attrs.parent フィールド自体は残っている
    // (RemoveLayer は present だけを落とす)が、resolve は present でフィルタして
    // ローカルのみへ縮退するはず。
    let after = doc
        .view()
        .resolve(child, t(0))
        .unwrap()
        .expect("子は居るはず");
    approx(after.placement.transform.translation.x, 10.0);
    approx(after.placement.transform.translation.y, 5.0);
}

/// 存在したことのない `LayerId` を parent に指す(壊れた/旧ファイルの直接編集を
/// 想定)場合も同じ縮退。
#[test]
fn a_parent_pointing_at_a_never_created_layer_falls_back_to_local_only() {
    let mut doc = doc_with_comp(300);
    let child = LayerId(2);
    place(&mut doc, child, Some(LayerId(999)));
    set_vec2(&mut doc, child, property::POSITION, [10.0, 5.0]);

    let resolved = doc.view().resolve(child, t(0)).unwrap().expect("居る");
    approx(resolved.placement.transform.translation.x, 10.0);
    approx(resolved.placement.transform.translation.y, 5.0);
}

/// parent を1つも設定していない(既存 Document と同型の)layer は、この変更の前後で
/// 挙動が変わらない — EXACT TARGET #1「既存 Document のバイト不変」の振る舞い版。
#[test]
fn a_layer_without_any_parent_behaves_exactly_as_before() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, None);
    set_vec2(&mut doc, layer, property::POSITION, [42.0, -7.0]);
    set_scalar(&mut doc, layer, property::ROTATION, 33.0);

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    // parent が無い layer の world = local そのもの(裁定58 の正本どおり)。
    approx(resolved.placement.transform.translation.x, 42.0);
    approx(resolved.placement.transform.translation.y, -7.0);
}

// ---------------------------------------------------------------------------
// LayerSource::Group — 絵を持たない・serde 往復
// ---------------------------------------------------------------------------

#[test]
fn group_layer_resolves_but_has_no_declared_size_and_draws_nothing() {
    let mut doc = doc_with_comp(300);
    let group = LayerId(1);
    place(&mut doc, group, None);
    make_group(&mut doc, group);

    let resolved = doc
        .view()
        .resolve(group, t(0))
        .unwrap()
        .expect("居る(絵は無くても解決はする)");
    assert_eq!(resolved.source, LayerSource::Group);
    assert_eq!(resolved.declared_size, [0.0, 0.0]);
}

/// Group を親にした子の world も普通に合成される — Group は「印」だけで、変換の
/// 合成規則そのものは他の `LayerSource` と何も変わらない(裁定173: 合成は単一
/// アルゴリズム)。
#[test]
fn a_child_parented_to_a_group_composes_normally() {
    let mut doc = doc_with_comp(300);
    let (group, child) = (LayerId(1), LayerId(2));
    place(&mut doc, group, None);
    make_group(&mut doc, group);
    place(&mut doc, child, Some(group));

    set_vec2(&mut doc, group, property::POSITION, [50.0, 50.0]);
    set_vec2(&mut doc, child, property::POSITION, [1.0, 2.0]);

    let resolved = doc.view().resolve(child, t(0)).unwrap().unwrap();
    approx(resolved.placement.transform.translation.x, 51.0);
    approx(resolved.placement.transform.translation.y, 52.0);
}

/// `LayerSource` の全 variant(`Group` を含む)が JSON 往復する。外部タグ表現
/// (unit variant は素の文字列)なので、`Group` の追加は既存 variant の形を
/// 一切変えない — 混在した `Vec<LayerSource>` で確かめる。
#[test]
fn layer_source_group_round_trips_alongside_every_other_variant() {
    let sources = vec![
        LayerSource::Solid {
            rgba: [1, 2, 3, 255],
            width: 10,
            height: 20,
        },
        LayerSource::Media {
            path: "clip.mp4".to_owned(),
            fingerprint: Some("abc".to_owned()),
        },
        LayerSource::Null,
        LayerSource::Shape,
        LayerSource::Text,
        LayerSource::Group,
    ];

    let json = serde_json::to_string(&sources).unwrap();
    let back: Vec<LayerSource> = serde_json::from_str(&json).unwrap();
    assert_eq!(back, sources, "LayerSource::Group 混在の Vec が往復しない");

    // Group 単体の表現は unit variant らしく素の文字列("Group")のはず
    // (Null/Shape/Text と同じ形 — 新しい表現方式を発明していないことの確認)。
    let group_json = serde_json::to_value(&LayerSource::Group).unwrap();
    let null_json = serde_json::to_value(&LayerSource::Null).unwrap();
    assert!(
        group_json.is_string(),
        "Group の JSON 表現が文字列ではない: {group_json}"
    );
    assert_eq!(
        group_json.is_string(),
        null_json.is_string(),
        "Group と Null(既存の絵を持たない variant)で表現の形が食い違っている"
    );
}

/// `parent` キーが無い旧 `LayerAttrs` JSON(`Group`/`parent` 導入前)でも読める —
/// `attrs_without_a_label_color_field_defaults_to_unassigned`(`tests/persist.rs`)と
/// 同じ手口(JSON からキーを取り除いてから読み戻す)。`parent: Option<LayerId>` は
/// `serde(default)` を付けていないが、`Option<T>` は欠落時に自動で `None` へ落ちる
/// (serde のデフォルト挙動、実測確認済み)ので、この試験はその前提を明示的に固定する。
#[test]
fn attrs_without_a_parent_field_defaults_to_none() {
    let current = LayerAttrs {
        name: "旧ドキュメントの layer".to_owned(),
        ..Default::default()
    };
    let mut value = serde_json::to_value(&current).unwrap();
    value
        .as_object_mut()
        .expect("LayerAttrs は JSON object のはず")
        .remove("parent")
        .expect("旧形式を模すには parent キーが無いことが前提");

    let loaded: LayerAttrs =
        serde_json::from_value(value).expect("旧形式の JSON(parent 無し)を読めない");
    assert_eq!(loaded.parent, None, "parent 欠落時は None へ落ちるはず");
}

// ---------------------------------------------------------------------------
// 保存/読込を経由しても親子合成が保たれる
// ---------------------------------------------------------------------------

fn tmp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "motolii-transform-hierarchy-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn parent_composition_survives_save_and_load() {
    let mut doc = doc_with_comp(300);
    let (group, parent, child) = (LayerId(1), LayerId(2), LayerId(3));
    place(&mut doc, group, None);
    place(&mut doc, parent, Some(group));
    place(&mut doc, child, Some(parent));

    set_vec2(&mut doc, group, property::POSITION, [10.0, 0.0]);
    set_vec2(&mut doc, parent, property::POSITION, [20.0, 0.0]);
    set_vec2(&mut doc, child, property::POSITION, [1.0, 1.0]);
    make_group(&mut doc, group);

    let before = doc.view().resolve(child, t(0)).unwrap().unwrap();

    let path = tmp("parent_composition_roundtrip.motolii");
    doc.save(&path).expect("保存できない");
    let loaded = Document::load(&path).expect("読み込めない");

    let after = loaded.view().resolve(child, t(0)).unwrap().unwrap();
    assert_eq!(
        after.placement.transform, before.placement.transform,
        "保存/読込の前後で親子合成した world transform が変わった"
    );
    approx(after.placement.transform.translation.x, 31.0);
    approx(after.placement.transform.translation.y, 1.0);
    assert_eq!(
        loaded.view().meta(group).unwrap().unwrap().source,
        LayerSource::Group
    );
}
