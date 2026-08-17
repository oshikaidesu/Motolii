//! Browser / Inspector native pane の read-model と model 供給の検収。
//!
//! Inspector 側の意味の正本は
//! `docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs`(P1〜N7)。
//! ここでは同じ fixture(`docs/mocks-ui/fixtures/reference-document.json` +
//! `inspector-read-model-parts.json` の nodes 相当 = first-party catalog)から
//! Rust read-model が同じ意味へ落ちることを見る。
//!
//! Browser 側は `media_library` のモデルをそのまま食うことと、
//! 選択・検索・filter の状態が視覚状態の根になることを見る。

use std::fs;
use std::path::{Path, PathBuf};

use motolii_doc::{DocKeyframeTrack, DocParam, Document, TrackItem};
use motolii_plugins_firstparty::first_party_catalog;
use motolii_ui::browser_panel::{BrowserPanel, BrowserTab, BrowserView};
use motolii_ui::inspector_panel::{
    project_inspector_read_model, InspectorItemKind, InspectorPosition, InspectorReadModelError,
    INSPECTOR_READ_MODEL_REVISION,
};

fn reference_document() -> Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/mocks-ui/fixtures/reference-document.json");
    motolii_doc::load_document(&path).expect("reference document fixture loads")
}

/// decoder P1: target layer 5 は group で child_count 1、
/// effect_definitions は文書の [0, 2] を plugin params(nodes)で結んだもの、
/// position は const (0, 0)。
#[test]
fn p1_reference_target_5_matches_the_decoder_meaning() {
    let document = reference_document();
    let catalog = first_party_catalog().expect("first-party catalog");
    let model = project_inspector_read_model(&document, &catalog, 5).expect("P1 decodes");

    assert_eq!(model.fixture_revision, INSPECTOR_READ_MODEL_REVISION);
    assert_eq!(model.target.layer_id, 5);
    assert_eq!(model.target.layer_name, "Reference group");
    assert_eq!(model.target.item_kind, InspectorItemKind::Group);
    assert_eq!(model.target.child_count, Some(1));

    let ids: Vec<u64> = model
        .effect_definitions
        .iter()
        .map(|def| def.definition_id)
        .collect();
    assert_eq!(ids, [0, 2]);
    for def in &model.effect_definitions {
        assert_eq!(def.plugin_id, "core.filter.opacity");
        assert_eq!(def.params.len(), 1);
        let param = &def.params[0];
        assert_eq!(param.id, "amount");
        assert_eq!(param.value_type, motolii_plugin::ValueType::F64);
        assert_eq!(param.default, motolii_plugin::Value::F64(1.0));
        let domain = param.f64_domain.expect("amount has a unit domain");
        assert_eq!(domain.min_inclusive, Some(0.0));
        assert_eq!(domain.max_inclusive, Some(1.0));
        assert!(!domain.integer);
    }

    assert_eq!(model.position, InspectorPosition::Const { x: 0.0, y: 0.0 });
}

/// decoder P2: target layer 0 は clip、child_count は載らない。
#[test]
fn p2_target_layer_0_is_a_clip_without_child_count() {
    let document = reference_document();
    let catalog = first_party_catalog().expect("first-party catalog");
    let model = project_inspector_read_model(&document, &catalog, 0).expect("P2 decodes");
    assert_eq!(model.target.layer_name, "Shared left");
    assert_eq!(model.target.item_kind, InspectorItemKind::Clip);
    assert_eq!(model.target.child_count, None);
}

/// decoder P3: group の子 clip も直接選べる。
#[test]
fn p3_group_child_clip_resolves() {
    let document = reference_document();
    let catalog = first_party_catalog().expect("first-party catalog");
    let model = project_inspector_read_model(&document, &catalog, 6).expect("P3 decodes");
    assert_eq!(model.target.layer_name, "Group shape child");
    assert_eq!(model.target.item_kind, InspectorItemKind::Clip);
    assert_eq!(model.target.child_count, None);
}

/// decoder R7: 存在しない layer を指す target は落とす。
#[test]
fn r7_dangling_target_layer_is_rejected() {
    let document = reference_document();
    let catalog = first_party_catalog().expect("first-party catalog");
    let error = project_inspector_read_model(&document, &catalog, 99)
        .expect_err("dangling target must not decode");
    assert!(matches!(
        error,
        InspectorReadModelError::TargetLayerMissing { layer_id: 99 }
    ));
}

/// decoder R7-b: layer 台帳には居るが track item が無い layer は ambiguous。
#[test]
fn r7b_layer_without_a_track_item_is_rejected() {
    let mut document = reference_document();
    let catalog = first_party_catalog().expect("first-party catalog");
    // layer 6 の item(group の子)だけを消す。台帳側の layer 6 は残る。
    for track in &mut document.tracks {
        for item in &mut track.items {
            if let TrackItem::Group(group) = item {
                group.children.clear();
            }
        }
    }
    let error = project_inspector_read_model(&document, &catalog, 6)
        .expect_err("layer without an item must not decode");
    assert!(matches!(
        error,
        InspectorReadModelError::TargetItemMissing { layer_id: 6 }
    ));
}

/// decoder R9: plugin id が catalog で解決できなければ落とす。
#[test]
fn r9_unresolved_plugin_id_is_rejected() {
    let document = reference_document();
    let empty = motolii_plugin::PluginCatalogBuilder::new()
        .build()
        .expect("empty catalog");
    let error = project_inspector_read_model(&document, &empty, 5)
        .expect_err("unresolved plugin must not decode");
    assert!(matches!(
        error,
        InspectorReadModelError::PluginUnresolved { .. }
    ));
}

/// decoder N6: keyframe 持ちの position は値を開かず `animated` の要約だけを出す。
#[test]
fn n6_keyframed_position_projects_as_animated() {
    let mut document = reference_document();
    let catalog = first_party_catalog().expect("first-party catalog");
    for track in &mut document.tracks {
        for item in &mut track.items {
            if let TrackItem::Group(group) = item {
                group.envelope.transform.position = DocParam::Keyframes(DocKeyframeTrack::new());
            }
        }
    }
    let model = project_inspector_read_model(&document, &catalog, 5).expect("animated decodes");
    assert_eq!(model.position, InspectorPosition::Animated);
}

/// decoder N6: const / keyframes 以外の position は閉じない(要約を発明しない)。
#[test]
fn n6_unsupported_position_kinds_are_rejected() {
    let mut document = reference_document();
    let catalog = first_party_catalog().expect("first-party catalog");
    for track in &mut document.tracks {
        for item in &mut track.items {
            if let TrackItem::Group(group) = item {
                group.envelope.transform.position = DocParam::Vec2Axes {
                    x: Box::new(DocParam::const_f64(0.0)),
                    y: Box::new(DocParam::const_f64(0.0)),
                };
            }
        }
    }
    let error = project_inspector_read_model(&document, &catalog, 5)
        .expect_err("unsupported position must not decode");
    assert!(matches!(
        error,
        InspectorReadModelError::UnsupportedPosition { .. }
    ));
}

// ---- Browser ----

fn temp_media_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "motolii-native-browser-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(dir.join("b-roll")).expect("media dir");
    // 1x1 PNG(実 decode 可能)。thumbnail 生成が image kind で走ることを見る。
    let png: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8,
        0xCF, 0xC0, 0xF0, 0x1F, 0x00, 0x05, 0x00, 0x01, 0xFF, 0x89, 0x99, 0x3D, 0x1D, 0x00, 0x00,
        0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    fs::write(dir.join("still.png"), png).expect("png");
    fs::write(dir.join("hero.mp4"), b"mp4").expect("mp4");
    fs::write(dir.join("b-roll").join("cut.wav"), b"wav").expect("wav");
    dir
}

/// Browser は media_library のモデルをそのまま食う(モック配列を持たない)。
#[test]
fn browser_panel_feeds_from_the_media_library_model() {
    let dir = temp_media_dir("feed");
    let mut panel = BrowserPanel::with_root(dir.clone());

    let names: Vec<String> = panel
        .visible_cards()
        .iter()
        .map(|card| card.name.clone())
        .collect();
    assert_eq!(names, ["cut.wav", "hero.mp4", "still.png"]);
    assert_eq!(panel.result_count(), 3);

    // kind filter(sidebar LIBRARY 行 = browser-library.html:47-49 と同じ意味)。
    panel.set_kind_filter(Some("video"));
    let names: Vec<String> = panel
        .visible_cards()
        .iter()
        .map(|card| card.name.clone())
        .collect();
    assert_eq!(names, ["hero.mp4"]);
    panel.set_kind_filter(None);

    // 検索は name + tags の haystack(browser-library.html:290-294 と同じ意味)。
    panel.set_query("wav");
    assert_eq!(panel.result_count(), 1);
    panel.set_query("");

    // tag filter(filter shelf = model の派生 tag)。
    panel.set_tag_filter(Some("b-roll".to_owned()));
    let names: Vec<String> = panel
        .visible_cards()
        .iter()
        .map(|card| card.name.clone())
        .collect();
    assert_eq!(names, ["cut.wav"]);

    let _ = fs::remove_dir_all(&dir);
}

/// 選択は視覚状態の根。click 相当の select が card へ写る。
#[test]
fn browser_panel_selection_reaches_the_cards() {
    let dir = temp_media_dir("select");
    let mut panel = BrowserPanel::with_root(dir.clone());

    assert!(panel.selected_id().is_none());
    let first = panel.visible_cards()[0].id.clone();
    panel.select(&first);
    assert_eq!(panel.selected_id(), Some(first.as_str()));
    let cards = panel.visible_cards();
    assert!(cards[0].selected);
    assert!(!cards[1].selected);

    // filter で見えなくなっても選択そのものは保持(mock は selectedIds を持ち続ける)。
    panel.set_query("hero");
    assert_eq!(panel.selected_id(), Some(first.as_str()));

    let _ = fs::remove_dir_all(&dir);
}

/// thumbnail は既存 `browser_blitz::thumbnail` の縮小実体(cache path)。
/// 元画像 path をそのまま使わない(毎フレーム再合成の重さを持ち込まない)。
#[test]
fn browser_panel_uses_the_shared_thumbnail_cache() {
    let dir = temp_media_dir("thumb");
    let panel = BrowserPanel::with_root(dir.clone());

    let still = panel
        .visible_cards()
        .iter()
        .find(|card| card.name == "still.png")
        .expect("png card")
        .id
        .clone();
    let thumb = panel.thumbnail_path(&still).expect("image gets a thumbnail");
    assert!(thumb.is_file());
    assert_ne!(thumb, dir.join("still.png"));

    // 非画像 kind は縮小実体を持たない(glyph card で描く)。
    let hero = panel
        .visible_cards()
        .iter()
        .find(|card| card.name == "hero.mp4")
        .expect("mp4 card")
        .id
        .clone();
    assert!(panel.thumbnail_path(&hero).is_none());

    let _ = fs::remove_dir_all(&dir);
}

/// tab / view の状態遷移(browser-library.html の state.tab / state.view と同じ語彙)。
#[test]
fn browser_panel_tab_and_view_states_follow_the_mock_vocabulary() {
    let dir = temp_media_dir("tabs");
    let mut panel = BrowserPanel::with_root(dir.clone());

    assert_eq!(panel.tab(), BrowserTab::Media);
    assert_eq!(panel.view(), BrowserView::Grid);

    // Media 以外の tab は model を持たないので 0 件(項目を発明しない)。
    panel.set_tab(BrowserTab::Effects);
    assert_eq!(panel.result_count(), 0);
    panel.set_tab(BrowserTab::Media);
    assert_eq!(panel.result_count(), 3);

    panel.set_view(BrowserView::List);
    assert_eq!(panel.view(), BrowserView::List);

    let _ = fs::remove_dir_all(&dir);
}
