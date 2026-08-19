//! Timeline の**構造操作**(2026-08-19)— fold 開閉・rename・lock・group/ungroup・
//! duplicate。egui 版 `timeline_editor/mod.rs` の対応するテスト
//! (`group_closed_hides_children_but_keeps_the_group_row` 等)と同じ問いを、
//! 駆動器だけ `iced_test::Simulator` に替えて訊く。
//!
//! `drive_timeline.rs` と同じ駆動の型(1フレーム = 1 Simulator、押す/引きずる/
//! 離すは別フレーム)を、fixture のロード・座標計算ごと複製する
//! (test binary は互いに import できないので、共有できるのは `tests/common/`
//! だけ — そこに Timeline 専用の座標計算まで足すと無関係なテストまで巻き込むので、
//! ここに閉じる)。

mod common;

use std::collections::HashMap;
use std::path::PathBuf;

use common::drain;
use iced::keyboard::{Key, Location, Modifiers};
use iced::mouse;
use iced::Point;
use motolii_doc::{Document, LayerId, TrackItem};
use motolii_shell_iced::timeline::semantics::{
    initial_view, row_fold_arrow_x, row_indent, row_lock_button_x, row_params_toggle_x,
    row_own_lock, PaneGeometry, ROW_H,
};
use motolii_shell_iced::{view, Message, Outcome, ScriptedPrompts, Shell};
use motolii_ui::timeline_editor::{lab_fixture, TimelineView};

// ---------------------------------------------------------------------------
// 支度(drive_timeline.rs と同じ型。test binary 間で共有できないので複製)
// ---------------------------------------------------------------------------

fn fixture_project(tag: &str) -> (PathBuf, HashMap<LayerId, String>) {
    let (document, names) = lab_fixture();
    let dir = motolii_testkit::tmp_dir(&format!("iced_timeline_structure_{tag}"));
    let path = dir.join("project.json");
    let mut session =
        motolii_doc::ProjectSession::acquire(&path, &motolii_doc::ResourceLimits::production())
            .expect("acquire temp project");
    session
        .save_document(&document, &motolii_doc::SaveOptions::default())
        .expect("save temp project");
    drop(session);
    (path, names)
}

fn layer_named(names: &HashMap<LayerId, String>, want: &str) -> LayerId {
    *names
        .iter()
        .find(|(_, name)| name.as_str() == want)
        .map(|(layer, _)| layer)
        .expect("fixture layer")
}

fn seated_shell(path: &PathBuf) -> Shell {
    let mut shell = Shell::new(ScriptedPrompts {
        open_project_path: Some(path.clone()),
        ..ScriptedPrompts::default()
    });
    let pressed = common::press(
        iced_test::simulator(view(&shell)),
        motolii_shell_iced::view::OPEN_PROJECT,
    );
    drain(&mut shell, pressed);
    assert!(shell.is_seated(), "fixture project へ座れていない");
    shell
}

fn timeline_bounds(shell: &Shell) -> iced::Rectangle {
    let mut probe = iced_test::simulator(view(shell));
    let target: iced_test::selector::Target = probe
        .find(motolii_shell_iced::view::TIMELINE_PANE_ID)
        .expect("Timeline pane container が座席上に立っていない");
    target.bounds()
}

fn geometry(shell: &Shell) -> PaneGeometry {
    let bounds = timeline_bounds(shell);
    PaneGeometry {
        width: bounds.width,
        height: bounds.height,
        wave_h: 0.0,
    }
}

fn fresh_view() -> TimelineView {
    initial_view(16.0)
}

fn cursor_moved(at: Point) -> iced::event::Event {
    iced::event::Event::Mouse(mouse::Event::CursorMoved { position: at })
}

fn pressed() -> iced::event::Event {
    iced::event::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
}

fn pointer_step(
    shell: &mut Shell,
    at: Point,
    events: impl IntoIterator<Item = iced::event::Event>,
) -> Outcome {
    let origin = timeline_bounds(shell);
    let at = Point::new(at.x + origin.x, at.y + origin.y);
    let mut ui = iced_test::simulator(view(shell));
    ui.point_at(at);
    let events: Vec<iced::event::Event> =
        std::iter::once(cursor_moved(at)).chain(events).collect();
    let _ = ui.simulate(events);
    let messages: Vec<Message> = ui.into_messages().collect();
    drain(shell, messages)
}

fn key_step(shell: &mut Shell, events: impl IntoIterator<Item = iced::event::Event>) -> Outcome {
    let messages = common::feed(iced_test::simulator(view(shell)), events);
    drain(shell, messages)
}

fn drag_gesture(shell: &mut Shell, from: Point, to: Point) {
    pointer_step(shell, from, [pressed()]);
    pointer_step(shell, to, std::iter::empty());
    pointer_step(
        shell,
        to,
        [iced::event::Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        ))],
    );
}

/// 修飾つきの1キー(押して離す)。`common::command_key` は Shift を持てないので
/// ここで組む。
fn key_with_modifiers(character: char, modifiers: Modifiers) -> Vec<iced::event::Event> {
    let key = Key::Character(character.to_string().into());
    vec![
        iced::event::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key.clone(),
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: Location::Standard,
            modifiers,
            repeat: false,
            text: None,
        }),
        iced::event::Event::Keyboard(iced::keyboard::Event::KeyReleased {
            key: key.clone(),
            modified_key: key,
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: Location::Standard,
            modifiers,
        }),
    ]
}

fn named_key(named: iced::keyboard::key::Named) -> Vec<iced::event::Event> {
    let key = Key::Named(named);
    vec![
        iced::event::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key.clone(),
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: Location::Standard,
            modifiers: Modifiers::empty(),
            repeat: false,
            text: None,
        }),
        iced::event::Event::Keyboard(iced::keyboard::Event::KeyReleased {
            key: key.clone(),
            modified_key: key,
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: Location::Standard,
            modifiers: Modifiers::empty(),
        }),
    ]
}

fn intents_of_kind(shell: &Shell, kind: &str) -> usize {
    shell
        .intents()
        .iter()
        .filter(|event| {
            serde_json::to_string(&event.intent)
                .expect("serializable")
                .contains(&format!(r#""kind":"{kind}""#))
        })
        .count()
}

fn is_group(document: &Document, layer: LayerId) -> bool {
    matches!(
        motolii_doc::find_item_location(document, layer).map(|(_, _, item)| item),
        Some(TrackItem::Group(_))
    )
}

/// いま見えている行数。`rows()` は純関数なので、殻が持つ document + fold から
/// テスト側でそのまま作れる(製品コードに新しいアクセサを増やさない)。
fn visible_rows_len(shell: &Shell) -> usize {
    let document = shell.timeline_snapshot().expect("seated");
    motolii_ui::timeline_rows::rows(&document, &shell.timeline_pane().fold).len()
}

// ---------------------------------------------------------------------------
// fold: children / params(Document には入らない — intent は1件も飛ばない)
// ---------------------------------------------------------------------------

/// **子の開閉矢印は行を増減させるが、intent を1つも出さない。**
/// fixture: 行0 = Title scene(Group、子3枚)。
#[test]
fn the_fold_arrow_opens_and_closes_children_without_an_intent() {
    let (path, names) = fixture_project("fold_children");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let view0 = fresh_view();
    let intents_before = shell.intents().len();

    assert_eq!(visible_rows_len(&shell), 3, "畳んだ既定は3行");

    let (ax0, ax1) = row_fold_arrow_x(row_indent(0));
    let at = Point::new((ax0 + ax1) / 2.0, geometry.row_top(0, 0.0) + ROW_H / 2.0);
    pointer_step(&mut shell, at, [pressed()]);

    assert_eq!(
        visible_rows_len(&shell),
        6,
        "開くと Group の子3枚が増えて6行になる"
    );
    assert_eq!(
        shell.intents().len(),
        intents_before,
        "畳み開閉は Document に入らないので journal は伸びない"
    );

    // もう一度押すと畳まれて3行へ戻る。
    pointer_step(&mut shell, at, [pressed()]);
    assert_eq!(visible_rows_len(&shell), 3, "畳み直すと3行へ戻る");

    let _ = names;
    let _ = view0;
}

/// **param 行の開閉(◇/◆)は子の開閉と独立の軸。** Title scene は position と
/// opacity の2つにキーがあるので、開くと2行増える。
#[test]
fn the_params_toggle_opens_keyed_rows_independently_of_children() {
    let (path, _names) = fixture_project("fold_params");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let intents_before = shell.intents().len();

    let (px0, px1) = row_params_toggle_x();
    let at = Point::new((px0 + px1) / 2.0, geometry.row_top(0, 0.0) + ROW_H / 2.0);
    pointer_step(&mut shell, at, [pressed()]);

    assert_eq!(
        visible_rows_len(&shell),
        5,
        "position + opacity の2行が増える(子は閉じたまま)"
    );
    assert_eq!(shell.intents().len(), intents_before, "param 開閉も intent にならない");
}

// ---------------------------------------------------------------------------
// rename: ダブルクリック → 編集 → Enter確定 / Esc取消
// ---------------------------------------------------------------------------

/// **ダブルクリック→文字入力→Enter = 確定。** 1件の `rename_layer` intent が飛び、
/// Document の表示名が変わる。
#[test]
fn double_click_then_enter_commits_a_rename() {
    let (path, names) = fixture_project("rename_commit");
    let background = layer_named(&names, "Background");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let intents_before = shell.intents().len();

    let y = geometry.row_top(1, 0.0) + ROW_H / 2.0;
    let rail_x = 10.0;
    let at = Point::new(rail_x, y);
    // **二重クリックの検出は canvas ローカル state(`TimelineCanvasState`)が
    // 持つ**。この駆動器(`iced_test::Simulator`)は呼ぶたびに窓を作り直すので、
    // 別々の `pointer_step` にまたがる canvas state は本物の窓とは違って
    // 保たれない — 同じ1回の `simulate` へ2回分の押下をまとめて渡す。
    pointer_step(&mut shell, at, [pressed(), pressed()]);

    assert!(
        shell.timeline_pane().renaming.is_some(),
        "二重クリックで rename が始まらない"
    );

    // 既存バッファを Backspace で全部消してから打ち直す。
    for _ in 0.."Background".len() {
        key_step(&mut shell, named_key(iced::keyboard::key::Named::Backspace));
    }
    for ch in "Renamed".chars() {
        key_step(&mut shell, key_with_modifiers(ch, Modifiers::empty()));
    }
    key_step(&mut shell, named_key(iced::keyboard::key::Named::Enter));

    assert!(shell.timeline_pane().renaming.is_none(), "確定したら編集状態を抜ける");
    let document = shell.timeline_snapshot().expect("seated");
    let name = document.layers.display_name(background).unwrap_or("?");
    assert_eq!(name, "Renamed", "Document の表示名が書き換わらない: {name}");
    assert_eq!(intents_of_kind(&shell, "rename_layer"), 1, "1確定 = 1 intent");
    let _ = intents_before;
}

/// **Esc = 取消。** 編集中の Esc は `RenameCancelled` に化ける — 進行中の
/// move/trim ジェスチャの Esc(`GestureCancelled`)とは別腕(別レーン担当)。
#[test]
fn escape_while_renaming_cancels_without_touching_the_document() {
    let (path, names) = fixture_project("rename_cancel");
    let background = layer_named(&names, "Background");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let revision_before = shell.revision();

    let y = geometry.row_top(1, 0.0) + ROW_H / 2.0;
    let at = Point::new(10.0, y);
    pointer_step(&mut shell, at, [pressed(), pressed()]);
    assert!(shell.timeline_pane().renaming.is_some());

    key_step(&mut shell, key_with_modifiers('x', Modifiers::empty()));
    key_step(&mut shell, named_key(iced::keyboard::key::Named::Escape));

    assert!(shell.timeline_pane().renaming.is_none(), "Esc で編集状態を抜ける");
    assert_eq!(shell.revision(), revision_before, "Esc で Document は無傷");
    let document = shell.timeline_snapshot().expect("seated");
    assert_eq!(
        document.layers.display_name(background).unwrap_or("?"),
        "Background",
        "取消したら元の名前のまま"
    );
}

// ---------------------------------------------------------------------------
// lock: L ボタン(明示値)。own lock を反転し、掛かった行は掴めない。
// ---------------------------------------------------------------------------

#[test]
fn the_lock_button_sets_an_explicit_value_and_blocks_the_move_gesture() {
    let (path, names) = fixture_project("lock_button");
    let background = layer_named(&names, "Background");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let view0 = fresh_view();

    assert!(
        !row_own_lock(&shell.timeline_snapshot().expect("seated"), background),
        "fixture は最初ロックしていない"
    );

    let (lx0, lx1) = row_lock_button_x();
    let lock_at = Point::new(
        (lx0 + lx1) / 2.0,
        geometry.row_top(1, 0.0) + ROW_H / 2.0,
    );
    pointer_step(&mut shell, lock_at, [pressed()]);

    let document = shell.timeline_snapshot().expect("seated");
    assert!(row_own_lock(&document, background), "L を押すとロックが掛かる");
    assert_eq!(intents_of_kind(&shell, "set_layer_lock"), 1);

    // ロック中は drag しても動かない。
    let before = motolii_doc::find_item_location(&document, background)
        .map(|(_, _, item)| match item {
            TrackItem::Clip(c) => c.start,
            _ => panic!("Background は clip のはず"),
        })
        .expect("location");
    let y = geometry.row_top(1, 0.0) + ROW_H / 2.0;
    drag_gesture(
        &mut shell,
        Point::new(geometry.time_to_x(view0, 2.0), y),
        Point::new(geometry.time_to_x(view0, 5.0), y),
    );
    let after_document = shell.timeline_snapshot().expect("seated");
    let after = motolii_doc::find_item_location(&after_document, background)
        .map(|(_, _, item)| match item {
            TrackItem::Clip(c) => c.start,
            _ => panic!("Background は clip のはず"),
        })
        .expect("location");
    assert_eq!(before, after, "ロック中は掴んでも動かない");

    // もう一度押すと外れる。
    pointer_step(&mut shell, lock_at, [pressed()]);
    let unlocked = shell.timeline_snapshot().expect("seated");
    assert!(!row_own_lock(&unlocked, background), "もう一度押すと外れる");
    assert_eq!(intents_of_kind(&shell, "set_layer_lock"), 2);
}

// ---------------------------------------------------------------------------
// Group / Ungroup(Cmd+G / Cmd+Shift+G)
// ---------------------------------------------------------------------------

/// **Cmd+G で選択をまとめる。** Background + audio(fixture の2つの兄弟)を
/// まとめると新しい Group が1つでき、選択がそれに変わる。
#[test]
fn cmd_g_groups_the_selection_into_one_new_group() {
    let (path, names) = fixture_project("group_cmd_g");
    let background = layer_named(&names, "Background");
    let audio = layer_named(&names, "starter-tone.wav");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);

    // Background を選び、Cmd+クリックで audio を足す。**modifiers はその状態を
    // 反映した Message が drain された後で初めて `pane.modifiers` へ載る**
    // (`drive_timeline.rs` の Cmd+click と同じ手順 — Cmd を立てる押下と、
    // それを読む click を同じ `simulate` バッチに混ぜると、click 側は
    // まだ古い modifiers を読んでしまう)。
    let y1 = geometry.row_top(1, 0.0) + ROW_H / 2.0;
    let y2 = geometry.row_top(2, 0.0) + ROW_H / 2.0;
    pointer_step(&mut shell, Point::new(10.0, y1), [pressed()]);
    key_step(
        &mut shell,
        [iced::event::Event::Keyboard(
            iced::keyboard::Event::ModifiersChanged(Modifiers::COMMAND),
        )],
    );
    pointer_step(&mut shell, Point::new(10.0, y2), [pressed()]);
    assert_eq!(
        shell.timeline_selection().len(),
        2,
        "Cmd+クリックで2つ選ばれていない: {:?}",
        shell.timeline_selection()
    );

    key_step(&mut shell, key_with_modifiers('g', Modifiers::COMMAND));

    assert_eq!(intents_of_kind(&shell, "group_layers"), 1);
    let selection = shell.timeline_selection();
    assert_eq!(selection.len(), 1, "まとめた後は新しい Group 1つだけを選ぶ");
    let document = shell.timeline_snapshot().expect("seated");
    assert!(is_group(&document, selection[0]), "選ばれているのは Group");
    assert!(
        shell.timeline_pane().fold.children_are_open(selection[0]),
        "まとめたら中が見えている状態にする(egui 版と同じ手触り)"
    );
    let _ = (background, audio);
}

/// **Cmd+Shift+G で Group を解く。** fixture の Title scene(子3枚)を解くと、
/// 3枚とも Title scene が居た場所(トップレベル)へ展開される。
#[test]
fn cmd_shift_g_ungroups_the_selected_group() {
    let (path, names) = fixture_project("ungroup_cmd_shift_g");
    let group = layer_named(&names, "Title scene");
    let shared_left = layer_named(&names, "Shared left");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);

    // x=10 は Title scene 行では fold 矢印の当たり(`row_fold_arrow_x(row_indent(0))`
    // = 8..22)に入ってしまう(このfixtureの行0だけ has_children — 他の行の
    // テストで x=10 を使っているのは children を持たない行だから安全だった)。
    // 矢印の外・名前の中を押す。
    let y0 = geometry.row_top(0, 0.0) + ROW_H / 2.0;
    pointer_step(&mut shell, Point::new(100.0, y0), [pressed()]);
    assert_eq!(shell.timeline_selection(), vec![group]);

    key_step(&mut shell, key_with_modifiers('G', Modifiers::COMMAND | Modifiers::SHIFT));

    assert_eq!(intents_of_kind(&shell, "ungroup_layer"), 1);
    let document = shell.timeline_snapshot().expect("seated");
    // 子だった layer が、いまはトップレベルの track item として見つかる。
    let (parent, _, _) =
        motolii_doc::find_item_location(&document, shared_left).expect("still in the document");
    assert!(
        matches!(parent, motolii_doc::ParentLocator::Track(_)),
        "解いた子はトップレベルへ出る: {parent:?}"
    );
    // Group 自体は残る(D2 に消す口が無い — RETURN 参照)が、空になっている。
    let (_, _, group_item) =
        motolii_doc::find_item_location(&document, group).expect("group still lives");
    match group_item {
        TrackItem::Group(g) => assert!(g.children.is_empty(), "子を出したら空になる"),
        other => panic!("Group のはず: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Duplicate(Cmd+D)— capsule の想定に反して D2 の口が実在したので実装
// ---------------------------------------------------------------------------

#[test]
fn cmd_d_duplicates_the_selected_layer() {
    let (path, names) = fixture_project("duplicate_cmd_d");
    let background = layer_named(&names, "Background");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);

    let before_document = shell.timeline_snapshot().expect("seated");
    let layer_count_before = before_document.layers.len();

    let y1 = geometry.row_top(1, 0.0) + ROW_H / 2.0;
    pointer_step(&mut shell, Point::new(10.0, y1), [pressed()]);
    assert_eq!(shell.timeline_selection(), vec![background]);

    key_step(&mut shell, key_with_modifiers('d', Modifiers::COMMAND));

    assert_eq!(intents_of_kind(&shell, "duplicate_selection"), 1);
    let document = shell.timeline_snapshot().expect("seated");
    assert_eq!(
        document.layers.len(),
        layer_count_before + 1,
        "複製で層が1つ増える"
    );
    let selection = shell.timeline_selection();
    assert_eq!(selection.len(), 1, "複製後は増えたほうを選ぶ");
    assert_ne!(selection[0], background, "選択が複製されたほうへ移る");
}
