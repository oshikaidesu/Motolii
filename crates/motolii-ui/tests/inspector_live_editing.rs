//! Inspector を**生きた面**にする検収。
//!
//! 見るのは4つだけ:
//! 1. Timeline の選択が Inspector の read-model になる(選択が変われば内容が変わり、
//!    非選択なら空になる)
//! 2. 数値の直打ちが Document へ届き、**Undo 1回**で戻る
//! 3. ドラッグ中の連続変更が **1 gesture = 1 Undo** に畳まれる
//! 4. ◇ が playhead へキーを打ち、既にキーがある時刻なら**値を更新**する
//!
//! 再生中の適用が再生を止めないことは、`playing` を直接置ける
//! `timeline_editor` の unit 側で見る(`tapping_m_while_playing_...` と同じ形)。
//!
//! ここは **writer 経由**であることも同時に見る — Document を書くのはエディタが
//! 抱える1つの writer だけで、Inspector は投影と要求しか持たない(single writer)。

use std::sync::Arc;

use motolii_doc::{DocParam, DocValue, Document, DocumentWriter, LayerId};
use motolii_ui::inspector_panel::{
    project_inspector_live_model, InspectorEditParam, InspectorKeyState, InspectorPanel,
};
use motolii_ui::timeline_editor::{lab_fixture, TimelineEditor};
use motolii_ui::timeline_rows::ParamRef;

fn seated_editor() -> (TimelineEditor, std::collections::HashMap<LayerId, String>) {
    let (document, names) = lab_fixture();
    let catalog =
        Arc::new(motolii_plugin::reference::reference_catalog().expect("reference catalog"));
    let writer = DocumentWriter::new(document, catalog).expect("writer");
    (TimelineEditor::new(writer), names)
}

fn layer_named(names: &std::collections::HashMap<LayerId, String>, want: &str) -> LayerId {
    *names
        .iter()
        .find(|(_, name)| name.as_str() == want)
        .map(|(layer, _)| layer)
        .expect("fixture layer")
}

fn reference_catalog() -> motolii_plugin::PluginCatalog {
    motolii_plugin::reference::reference_catalog().expect("reference catalog")
}

/// envelope の Position(const)を読む。書けたことの判定に使う。
fn const_position(document: &Document, layer: LayerId) -> [f64; 2] {
    match &find_envelope(document, layer).transform.position {
        DocParam::Const(DocValue::Vec2(v)) => *v,
        other => panic!("position is not a const Vec2: {other:?}"),
    }
}

fn find_envelope(document: &Document, layer: LayerId) -> &motolii_doc::ItemEnvelope {
    fn walk(
        items: &[motolii_doc::TrackItem],
        layer: LayerId,
    ) -> Option<&motolii_doc::ItemEnvelope> {
        for item in items {
            let envelope = match item {
                motolii_doc::TrackItem::Clip(clip) => &clip.envelope,
                motolii_doc::TrackItem::Group(group) => &group.envelope,
            };
            if envelope.layer_id == layer {
                return Some(envelope);
            }
            if let motolii_doc::TrackItem::Group(group) = item {
                if let Some(found) = walk(&group.children, layer) {
                    return Some(found);
                }
            }
        }
        None
    }
    document
        .tracks
        .iter()
        .find_map(|track| walk(&track.items, layer))
        .expect("layer has a track item")
}

// ---- 1. 選択 → read-model ----

/// 選択を変えると Inspector の target が変わり、選択を外すと**空状態**になる。
/// fixture へ落ちない — 座席があるあいだ Inspector が映すのは live の選択だけ。
#[test]
fn the_selection_drives_the_inspector_and_an_empty_selection_empties_it() {
    let (mut editor, names) = seated_editor();
    let background = layer_named(&names, "Background");
    let title = layer_named(&names, "Title scene");
    let mut panel = InspectorPanel::placeholder("seed");

    editor.select_layer(background);
    assert_eq!(editor.selected_layers(), [background]);
    panel.seat_live(
        editor.document(),
        editor.selected_layers(),
        editor.playhead_time().expect("playhead time"),
    );
    let model = panel.read_model().expect("selected layer projects");
    assert_eq!(model.target.layer_name, "Background");

    editor.select_layer(title);
    panel.seat_live(
        editor.document(),
        editor.selected_layers(),
        editor.playhead_time().expect("playhead time"),
    );
    assert_eq!(
        panel
            .read_model()
            .expect("second selection projects")
            .target
            .layer_name,
        "Title scene",
        "選択が変われば Inspector の中身も変わる"
    );

    editor.clear_selection();
    panel.seat_live(
        editor.document(),
        editor.selected_layers(),
        editor.playhead_time().expect("playhead time"),
    );
    assert!(
        panel.read_model().is_none(),
        "非選択は fixture ではなく空状態"
    );
    assert!(
        panel.empty_note().contains("No selection"),
        "空状態は理由を出す(黙って空にしない): {:?}",
        panel.empty_note()
    );
}

/// 複数選択は「N items」の要約に落ちる(1つに絞るまで property は出さない)。
#[test]
fn a_multi_selection_summarises_instead_of_editing() {
    let (mut editor, names) = seated_editor();
    let background = layer_named(&names, "Background");
    let title = layer_named(&names, "Title scene");
    let mut panel = InspectorPanel::placeholder("seed");

    editor.select_layer(background);
    editor.add_to_selection(title);
    assert_eq!(editor.selected_layers().len(), 2);

    panel.seat_live(
        editor.document(),
        editor.selected_layers(),
        editor.playhead_time().expect("playhead time"),
    );
    assert!(panel.read_model().is_none());
    assert!(
        panel.empty_note().contains('2'),
        "N items 要約: {:?}",
        panel.empty_note()
    );
}

// ---- 2. 数値直打ち ----

/// Position X の直打ちが Document へ届き、**Undo 1回**で元へ戻る。
#[test]
fn typing_a_position_reaches_the_document_and_one_undo_puts_it_back() {
    let (mut editor, names) = seated_editor();
    let background = layer_named(&names, "Background");
    let before = const_position(editor.document(), background);
    let undo_before = editor.undo_len();

    editor.begin_param_edit(background, ParamRef::Position);
    editor.set_param_component(background, ParamRef::Position, 0, 0.25);
    editor.end_param_edit();

    assert_eq!(
        const_position(editor.document(), background),
        [0.25, before[1]],
        "直打ちが Document へ届く"
    );
    assert_eq!(
        editor.undo_len(),
        undo_before + 1,
        "1回の確定 = 1 gesture = 1 Undo"
    );

    editor.undo_gesture();
    assert_eq!(
        const_position(editor.document(), background),
        before,
        "Undo 1回で元へ戻る"
    );
}

/// Opacity(スカラ)も同じ経路で書ける。
#[test]
fn typing_an_opacity_reaches_the_document() {
    let (mut editor, names) = seated_editor();
    let background = layer_named(&names, "Background");

    editor.begin_param_edit(background, ParamRef::Opacity);
    editor.set_param_component(background, ParamRef::Opacity, 0, 0.5);
    editor.end_param_edit();

    match &find_envelope(editor.document(), background).opacity {
        DocParam::Const(DocValue::F64(v)) => assert!((v - 0.5).abs() < 1e-9, "opacity = {v}"),
        other => panic!("opacity is not a const f64: {other:?}"),
    }
}

// ---- 3. ドラッグの畳み ----

/// ドラッグ中の連続変更は **1 gesture** に畳まれ、Undo 1回で掴む前へ戻る。
#[test]
fn a_drag_folds_every_step_into_one_undo() {
    let (mut editor, names) = seated_editor();
    let background = layer_named(&names, "Background");
    let before = const_position(editor.document(), background);
    let undo_before = editor.undo_len();

    editor.begin_param_edit(background, ParamRef::Position);
    for step in 1..=8 {
        editor.set_param_component(background, ParamRef::Position, 0, step as f64 * 0.05);
    }
    editor.end_param_edit();

    assert!(
        (const_position(editor.document(), background)[0] - 0.4).abs() < 1e-9,
        "最後の値が残る"
    );
    assert_eq!(
        editor.undo_len(),
        undo_before + 1,
        "8回動かしても Undo は1段(1ドラッグ = 1 gesture)"
    );
    editor.undo_gesture();
    assert_eq!(
        const_position(editor.document(), background),
        before,
        "Undo 1回で掴む前へ戻る"
    );
}

// ---- 4. ◇ キー打鍵 ----

/// ◇ が playhead へキーを打ち、同じ時刻でもう一度押すと**値を更新**する
/// (キーは増えない)。read-model の ◇/◆ もそれに従う。
#[test]
fn the_diamond_keys_at_the_playhead_and_updates_an_existing_key() {
    let (mut editor, names) = seated_editor();
    let background = layer_named(&names, "Background");
    let catalog = reference_catalog();
    editor.select_layer(background);
    editor.set_playhead_seconds(2.0);
    let at = editor.playhead_time().expect("playhead time");

    // 打つ前は Const = ◇
    let model = project_inspector_live_model(editor.document(), &catalog, background.get(), at)
        .expect("live model");
    let row = model
        .editable
        .iter()
        .find(|row| row.param == InspectorEditParam::Position)
        .expect("Position row is editable");
    assert_eq!(row.key_state, InspectorKeyState::Unkeyed);

    // ◇ を押す → playhead にキーが1つ
    let undo_before = editor.undo_len();
    editor.key_param_at_playhead(background, ParamRef::Position, &[0.0, 0.0]);
    assert_eq!(editor.undo_len(), undo_before + 1, "1回の打鍵 = 1 gesture");
    let keys = position_keys(editor.document(), background);
    assert_eq!(keys.len(), 1, "playhead にキーが1つ");

    let model = project_inspector_live_model(editor.document(), &catalog, background.get(), at)
        .expect("live model after keying");
    let row = model
        .editable
        .iter()
        .find(|row| row.param == InspectorEditParam::Position)
        .expect("Position row");
    assert_eq!(
        row.key_state,
        InspectorKeyState::KeyedAtPlayhead,
        "playhead にキーがある行は ◆"
    );

    // 同じ時刻でもう一度 ◇ → キーは増えず、値が更新される
    editor.key_param_at_playhead(background, ParamRef::Position, &[0.75, -0.25]);
    let keys = position_keys(editor.document(), background);
    assert_eq!(keys.len(), 1, "同じ時刻ではキーが増えない");
    assert_eq!(
        keys[0].1,
        DocValue::Vec2([0.75, -0.25]),
        "既にキーがある時刻は値を更新する"
    );

    // playhead を動かして押せば2本目が立つ
    editor.set_playhead_seconds(4.0);
    editor.key_param_at_playhead(background, ParamRef::Position, &[1.0, 0.0]);
    assert_eq!(
        position_keys(editor.document(), background).len(),
        2,
        "別の時刻なら新しいキーが立つ"
    );
}

/// キーを持つ param の直打ちは、playhead のキーの値を書き換える(Const へ潰さない)。
#[test]
fn typing_on_an_animated_param_writes_the_key_at_the_playhead() {
    let (mut editor, names) = seated_editor();
    let title = layer_named(&names, "Title scene");
    // fixture の "Title scene" は 2.8s / 10.1s に position キーを持つ。
    editor.set_playhead_seconds(2.8);

    editor.begin_param_edit(title, ParamRef::Position);
    editor.set_param_component(title, ParamRef::Position, 1, 0.5);
    editor.end_param_edit();

    let keys = position_keys(editor.document(), title);
    assert_eq!(keys.len(), 2, "キーの本数は変わらない");
    let at_playhead = keys
        .iter()
        .find(|(t, _)| (t.as_seconds_f64() - 2.8).abs() < 1e-6)
        .expect("key at the playhead");
    assert_eq!(
        at_playhead.1,
        DocValue::Vec2([0.0, 0.5]),
        "playhead のキーの値が変わる"
    );
    assert!(
        matches!(
            find_envelope(editor.document(), title).transform.position,
            DocParam::Keyframes(_)
        ),
        "アニメーション済みの param を Const へ潰さない"
    );
}

fn position_keys(
    document: &Document,
    layer: LayerId,
) -> Vec<(motolii_core::RationalTime, DocValue)> {
    match &find_envelope(document, layer).transform.position {
        DocParam::Keyframes(track) => track
            .keys()
            .iter()
            .map(|key| (key.t, key.value.clone()))
            .collect(),
        _ => Vec::new(),
    }
}
