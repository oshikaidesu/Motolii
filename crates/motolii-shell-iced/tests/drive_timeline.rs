//! Timeline の運転席(M-3)— 窓を開かずに掴んで・引きずって・離して・読む。
//!
//! egui 版の timeline oracle(`timeline_editor/mod.rs` のテストと
//! `motolii-ui/tests/blitz_shell_editor_seat.rs`)と**同じ問い**を、駆動器だけ
//! `iced_test::Simulator` に替えて訊く: 同じジェスチャが同じ Document 変化・
//! 同じ Undo 粒度・同じ選択になるか。
//!
//! ## 駆動の型
//!
//! iced の canvas はモデル(`&Shell`)を借りて立つので、1フレーム = 1 Simulator。
//! ジェスチャは「押す」「引きずる」「離す」を**別々のフレーム**として流す
//! (実窓でも押下と移動の間に必ず update → view が挟まる — 同じ段取りである)。
//!
//! ## 審判2本(replay oracle)
//!
//! 1. **intent 列 replay**(`ShellGateway::replay`)… 選択・revision・
//!    track item 数・playhead が一致する(意味の再現)
//! 2. **Message 列 replay** … 同じ Message 列を新しい殻へ流すと zoom / scroll の
//!    view 状態まで一致する(表示状態の再現。view は intent にしない決定の対)

mod common;

use std::collections::HashMap;
use std::path::PathBuf;

use common::drain;
use iced::keyboard::key::Named;
use iced::mouse;
use iced::Point;
use motolii_doc::{Document, LayerId, TrackItem};
use motolii_shell_iced::timeline::semantics::{initial_view, PaneGeometry, ROW_H};
use motolii_shell_iced::{view, Message, Outcome, ScriptedPrompts, Shell};
use motolii_ui::blitz_shell::ShellGateway;
use motolii_ui::timeline_editor::{lab_fixture, TimelineView};

// ---------------------------------------------------------------------------
// 支度: fixture project と、canvas の座標
// ---------------------------------------------------------------------------

/// lab の fixture(Group + clip 3種 + キー入り)を一時 project に保存する。
/// 経路は egui 側 `blitz_shell_editor_seat.rs` と同じ
/// (`ProjectSession::acquire` → `save_document`。lock は返す)。
fn fixture_project(tag: &str) -> (PathBuf, HashMap<LayerId, String>) {
    let (document, names) = lab_fixture();
    let dir = motolii_testkit::tmp_dir(&format!("iced_timeline_{tag}"));
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

/// Open ボタンで fixture project へ座る(dialog は台本が答える)。
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

/// Timeline pane container の実 bounds(窓座標)。4面合成(統合第2弾)で
/// Timeline canvas は窓の左上 (0,0) には立たなくなった(上段
/// Browser|Stage|Inspector の下、高さ 1:1)ので、運転席は
/// `motolii_shell_iced::view::TIMELINE_PANE_ID` で実座標を毎回訊く —
/// 決め打ちの高さ定数を持たない(ここが production の layout と食い違うと
/// 静かに違う場所を叩くだけの red にならないテストになる)。
fn timeline_bounds(shell: &Shell) -> iced::Rectangle {
    let mut probe = iced_test::simulator(view(shell));
    let target: iced_test::selector::Target = probe
        .find(motolii_shell_iced::view::TIMELINE_PANE_ID)
        .expect("Timeline pane container が座席上に立っていない");
    target.bounds()
}

/// テストは製品と**同じ幾何**(`PaneGeometry`)で座標を計算する。
/// width/height は実 bounds から([`timeline_bounds`])。
fn geometry(shell: &Shell) -> PaneGeometry {
    let bounds = timeline_bounds(shell);
    PaneGeometry {
        width: bounds.width,
        height: bounds.height,
        wave_h: 0.0, // fixture に soundtrack は無い
    }
}

/// 起動直後の view(0..16s)。fixture の composition は 16s なのでこのまま。
fn fresh_view() -> TimelineView {
    initial_view(16.0)
}

fn clip_span_seconds(document: &Document, layer: LayerId) -> (f64, f64) {
    let (_, _, item) =
        motolii_doc::find_item_location(document, layer).expect("layer lives in the document");
    match item {
        TrackItem::Clip(clip) => (
            clip.start.as_seconds_f64(),
            clip.start.as_seconds_f64() + clip.duration.as_seconds_f64(),
        ),
        other => panic!("expected a clip, found {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 駆動の道具: 1フレーム = 1 Simulator
// ---------------------------------------------------------------------------

fn cursor_moved(at: Point) -> iced::event::Event {
    iced::event::Event::Mouse(mouse::Event::CursorMoved { position: at })
}

fn pressed() -> iced::event::Event {
    iced::event::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
}

fn released() -> iced::event::Event {
    iced::event::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
}

/// ポインタを `at`(Timeline pane ローカル座標)へ置いて事象列を流す1フレーム。
/// 窓座標への変換(pane の実 bounds ぶんの平行移動)はここ1箇所でやる —
/// 呼び出し側は全部 `geometry()`(pane ローカル)のまま計算してよい。
/// 出た Message は殻へ流し込む。
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

/// キーボードだけの1フレーム。
fn key_step(shell: &mut Shell, events: impl IntoIterator<Item = iced::event::Event>) -> Outcome {
    let messages = common::feed(iced_test::simulator(view(shell)), events);
    drain(shell, messages)
}

/// 修飾なしの1キー(押して離す)。
fn tap(named: Named) -> Vec<iced::event::Event> {
    iced_test::simulator::tap_key(iced::keyboard::Key::Named(named), None).collect()
}

/// 押す → 引きずる → 離す、の完全な1ジェスチャ。
fn drag_gesture(shell: &mut Shell, from: Point, to: Point) {
    pointer_step(shell, from, [pressed()]);
    pointer_step(shell, to, std::iter::empty());
    pointer_step(shell, to, [released()]);
}

// ---------------------------------------------------------------------------
// Q1 文法: click=選択 / drag=移動 / 端drag=trim / release=確定 / Esc=復元 /
//          Delete=削除 / Cmd+Z=undo
// ---------------------------------------------------------------------------

/// **click = 選択。** bar を押した瞬間に選ばれ、intent は `select_layer` 1件。
#[test]
fn clicking_a_bar_selects_the_layer_through_the_gateway() {
    let (path, names) = fixture_project("click_select");
    let background = layer_named(&names, "Background");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let view0 = fresh_view();

    // Background は行 1(Group が 0)。本体 2.0s を押す。
    let revision_before = shell.revision();
    let at = Point::new(
        geometry.time_to_x(view0, 2.0),
        geometry.row_top(1, 0.0) + ROW_H / 2.0,
    );
    pointer_step(&mut shell, at, [pressed()]);

    assert_eq!(
        shell.timeline_selection(),
        vec![background],
        "click が選択にならない"
    );
    let kinds: Vec<String> = shell
        .intents()
        .iter()
        .map(|event| serde_json::to_string(&event.intent).expect("serializable"))
        .collect();
    assert!(
        kinds
            .last()
            .is_some_and(|line| line.contains(r#""kind":"select_layer""#)),
        "選択が intent として journal に載っていない: {kinds:?}"
    );

    pointer_step(&mut shell, at, [released()]);
    assert_eq!(
        shell.revision(),
        revision_before,
        "動かしていない release で Document が変わってはいけない"
    );
    assert!(
        !shell
            .intents()
            .iter()
            .any(|event| serde_json::to_string(&event.intent)
                .expect("serializable")
                .contains("move_clips")),
        "動かしていないのに move_clips が journal に載った"
    );
}

/// **drag = 移動、release = 確定。** +2.0s 動かすと Document の start が 2.0s 進み、
/// 1ドラッグ = 1 Undo 単位で、intent は `move_clips` 1件だけ。
#[test]
fn dragging_a_body_moves_the_clip_and_lands_one_undo_unit() {
    let (path, names) = fixture_project("drag_move");
    let background = layer_named(&names, "Background");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let view0 = fresh_view();

    let before = clip_span_seconds(&shell.timeline_snapshot().expect("seated"), background);
    assert_eq!(before.0, 0.0, "fixture の Background は 0s 始まり");
    let revision_before = shell.revision();

    let y = geometry.row_top(1, 0.0) + ROW_H / 2.0;
    drag_gesture(
        &mut shell,
        Point::new(geometry.time_to_x(view0, 2.0), y),
        Point::new(geometry.time_to_x(view0, 4.0), y),
    );

    let after = clip_span_seconds(&shell.timeline_snapshot().expect("seated"), background);
    assert!(
        (after.0 - 2.0).abs() < 1e-3,
        "move が Document へ届いていない: {after:?}"
    );
    assert!(shell.revision() > revision_before, "revision が進む");

    let moves: Vec<String> = shell
        .intents()
        .iter()
        .map(|event| serde_json::to_string(&event.intent).expect("serializable"))
        .filter(|line| line.contains("move_clips"))
        .collect();
    assert_eq!(moves.len(), 1, "1ドラッグ = 1 move intent: {moves:?}");

    // Cmd+Z 1回で掴む前へ戻る(1ドラッグ = 1 Undo 単位)。
    key_step(&mut shell, common::command_key('z'));
    let restored = clip_span_seconds(&shell.timeline_snapshot().expect("seated"), background);
    assert!(
        (restored.0 - before.0).abs() < 1e-3,
        "Cmd+Z で戻らない: {restored:?}"
    );
}

/// **端 drag = trim。** 右端を 10.0s へ引くと out 点だけが動き、start は変わらない。
#[test]
fn dragging_the_right_edge_trims_the_out_point() {
    let (path, names) = fixture_project("trim_out");
    let audio = layer_named(&names, "starter-tone.wav");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let view0 = fresh_view();

    let before = clip_span_seconds(&shell.timeline_snapshot().expect("seated"), audio);
    assert!((before.1 - 13.6).abs() < 1e-3, "fixture の audio は 13.6s 終わり");

    // audio は行 2。右端の内側 4px を掴む。
    let y = geometry.row_top(2, 0.0) + ROW_H / 2.0;
    drag_gesture(
        &mut shell,
        Point::new(geometry.time_to_x(view0, before.1 as f32) - 4.0, y),
        Point::new(geometry.time_to_x(view0, 10.0), y),
    );

    let after = clip_span_seconds(&shell.timeline_snapshot().expect("seated"), audio);
    assert!((after.0 - before.0).abs() < 1e-6, "trim out で start は動かない");
    assert!((after.1 - 10.0).abs() < 1e-3, "out 点が 10.0s に立たない: {after:?}");

    let trims: Vec<String> = shell
        .intents()
        .iter()
        .map(|event| serde_json::to_string(&event.intent).expect("serializable"))
        .filter(|line| line.contains("trim_clip"))
        .collect();
    assert_eq!(trims.len(), 1, "1トリム = 1 intent: {trims:?}");
    assert!(
        trims[0].contains(r#""edge":"out""#),
        "out 端の trim が out と名乗らない: {}",
        trims[0]
    );
}

/// **左端 drag = 入り点の trim。** 終端は動かない(D2 `TrimClipIn` の意味)。
#[test]
fn dragging_the_left_edge_trims_the_in_point() {
    let (path, names) = fixture_project("trim_in");
    let audio = layer_named(&names, "starter-tone.wav");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let view0 = fresh_view();

    let before = clip_span_seconds(&shell.timeline_snapshot().expect("seated"), audio);
    let y = geometry.row_top(2, 0.0) + ROW_H / 2.0;
    drag_gesture(
        &mut shell,
        Point::new(geometry.time_to_x(view0, before.0 as f32) + 4.0, y),
        Point::new(geometry.time_to_x(view0, 3.0), y),
    );

    let after = clip_span_seconds(&shell.timeline_snapshot().expect("seated"), audio);
    assert!((after.0 - 3.0).abs() < 1e-3, "in 点が 3.0s に立たない: {after:?}");
    assert!(
        (after.1 - before.1).abs() < 1e-3,
        "trim in で旧右端が動いた: {after:?}"
    );
}

/// **Esc = 復元。** 掴んで引きずって Esc したら Document は無傷で、
/// journal に move intent は残らない(release まで Document を触らない設計の審判)。
#[test]
fn escape_during_a_drag_restores_without_a_trace() {
    let (path, names) = fixture_project("esc_cancel");
    let background = layer_named(&names, "Background");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let view0 = fresh_view();

    let before = clip_span_seconds(&shell.timeline_snapshot().expect("seated"), background);
    let revision_before = shell.revision();
    let undo_before = shell
        .intents()
        .len();

    let y = geometry.row_top(1, 0.0) + ROW_H / 2.0;
    pointer_step(&mut shell, Point::new(geometry.time_to_x(view0, 2.0), y), [pressed()]);
    pointer_step(&mut shell, Point::new(geometry.time_to_x(view0, 6.0), y), std::iter::empty());
    key_step(&mut shell, tap(Named::Escape));
    // 離しても何も確定しない(ジェスチャは Esc で終わっている)。
    pointer_step(&mut shell, Point::new(geometry.time_to_x(view0, 6.0), y), [released()]);

    let after = clip_span_seconds(&shell.timeline_snapshot().expect("seated"), background);
    assert_eq!(after, before, "Esc で戻っていない");
    assert_eq!(shell.revision(), revision_before, "Document は無傷のはず");
    let intents_after: Vec<String> = shell
        .intents()
        .iter()
        .skip(undo_before)
        .map(|event| serde_json::to_string(&event.intent).expect("serializable"))
        .collect();
    assert!(
        !intents_after.iter().any(|line| line.contains("move_clips")),
        "Esc したのに move intent が残った: {intents_after:?}"
    );
}

/// **Delete = 削除、Cmd+Z = undo。** Group を消せば中身ごと消え、1回で戻る。
#[test]
fn delete_removes_the_selection_and_undo_brings_it_back() {
    let (path, names) = fixture_project("delete_undo");
    let background = layer_named(&names, "Background");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let view0 = fresh_view();

    assert_eq!(shell.track_item_count(), 3, "fixture は top-level 3件");

    // Background を選んで消す。
    let y = geometry.row_top(1, 0.0) + ROW_H / 2.0;
    let at = Point::new(geometry.time_to_x(view0, 2.0), y);
    pointer_step(&mut shell, at, [pressed()]);
    pointer_step(&mut shell, at, [released()]);
    assert_eq!(shell.timeline_selection(), vec![background]);

    key_step(&mut shell, tap(Named::Delete));
    assert_eq!(shell.track_item_count(), 2, "Delete が Document へ届かない");
    assert!(
        shell.timeline_selection().is_empty(),
        "消した後は選択も空(egui と同じ)"
    );

    key_step(&mut shell, common::command_key('z'));
    assert_eq!(shell.track_item_count(), 3, "Cmd+Z 1回で戻る(1 Delete = 1 Undo)");
}

/// **Cmd+click = 足し引き、複数選択は塊のまま動く。** 左へ突き抜けたら
/// 全員 0 で止まる(相対配置が壊れない — egui `commit_drag` と同じクランプ)。
#[test]
fn a_multi_selection_moves_as_a_block_and_stops_at_zero() {
    let (path, names) = fixture_project("multi_move");
    let background = layer_named(&names, "Background");
    let audio = layer_named(&names, "starter-tone.wav");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let view0 = fresh_view();

    let bg_y = geometry.row_top(1, 0.0) + ROW_H / 2.0;
    let audio_y = geometry.row_top(2, 0.0) + ROW_H / 2.0;

    // まず Background を +2.0s へ(0 始まりのままだと塊が左へ1mmも動けない)。
    drag_gesture(
        &mut shell,
        Point::new(geometry.time_to_x(view0, 2.0), bg_y),
        Point::new(geometry.time_to_x(view0, 4.0), bg_y),
    );
    let document = shell.timeline_snapshot().expect("seated");
    assert!((clip_span_seconds(&document, background).0 - 2.0).abs() < 1e-3);

    // Cmd を立てて audio を選択に足す(Background はドラッグ後も選ばれたまま)。
    key_step(
        &mut shell,
        [iced::event::Event::Keyboard(
            iced::keyboard::Event::ModifiersChanged(iced::keyboard::Modifiers::COMMAND),
        )],
    );
    let audio_at = Point::new(geometry.time_to_x(view0, 3.0), audio_y);
    pointer_step(&mut shell, audio_at, [pressed()]);
    pointer_step(&mut shell, audio_at, [released()]);
    assert_eq!(
        shell.timeline_selection(),
        vec![background, audio],
        "Cmd+click が選択に足していない"
    );
    key_step(
        &mut shell,
        [iced::event::Event::Keyboard(
            iced::keyboard::Event::ModifiersChanged(iced::keyboard::Modifiers::empty()),
        )],
    );

    // 塊ごと -5.0s 引きずる → 一番左(audio, 1.1s)が 0 に着いた所で全員止まる。
    let from = Point::new(geometry.time_to_x(view0, 6.0), audio_y);
    let to = Point::new(geometry.time_to_x(view0, 1.0), audio_y);
    drag_gesture(&mut shell, from, to);

    let document = shell.timeline_snapshot().expect("seated");
    let audio_after = clip_span_seconds(&document, audio);
    let bg_after = clip_span_seconds(&document, background);
    assert!(
        audio_after.0.abs() < 1e-3,
        "一番左の clip が 0 で止まっていない: {audio_after:?}"
    );
    assert!(
        (bg_after.0 - 0.9).abs() < 1e-3,
        "塊の相対配置が保たれていない(Background は -1.1s だけ動いて 0.9s): {bg_after:?}"
    );
}

// ---------------------------------------------------------------------------
// playhead: scrub と矢印コマ送り
// ---------------------------------------------------------------------------

/// **ルーラの click / drag = scrub。** release で `set_playhead` が1件立ち、
/// playhead はフレーム境界に乗る。
#[test]
fn scrubbing_the_ruler_seeks_the_playhead() {
    let (path, _names) = fixture_project("scrub");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let view0 = fresh_view();

    // click: 5.0s。
    let at5 = Point::new(geometry.time_to_x(view0, 5.0), geometry.ruler_top() + 10.0);
    pointer_step(&mut shell, at5, [pressed()]);
    pointer_step(&mut shell, at5, [released()]);
    assert!(
        (shell.timeline_playhead() - 5.0).abs() < 1e-3,
        "click で playhead が立たない: {}",
        shell.timeline_playhead()
    );

    // drag: 5.0 → 8.0s。
    let at8 = Point::new(geometry.time_to_x(view0, 8.0), geometry.ruler_top() + 10.0);
    pointer_step(&mut shell, at5, [pressed()]);
    pointer_step(&mut shell, at8, std::iter::empty());
    pointer_step(&mut shell, at8, [released()]);
    assert!(
        (shell.timeline_playhead() - 8.0).abs() < 1e-3,
        "drag で playhead が追わない: {}",
        shell.timeline_playhead()
    );

    let seeks: Vec<String> = shell
        .intents()
        .iter()
        .map(|event| serde_json::to_string(&event.intent).expect("serializable"))
        .filter(|line| line.contains("set_playhead"))
        .collect();
    assert_eq!(seeks.len(), 2, "scrub 2回 = intent 2件: {seeks:?}");
}

/// **← / → = コマ送り**(±1、Shift で ±10)。動くのは playhead だけ。
#[test]
fn arrow_keys_step_the_playhead_by_frames() {
    let (path, _names) = fixture_project("frame_step");
    let mut shell = seated_shell(&path);

    key_step(&mut shell, tap(Named::ArrowRight));
    let one = shell.timeline_playhead();
    assert!((one - 1.0 / 30.0).abs() < 1e-4, "→ が 1 コマでない: {one}");

    let mut shifted = Vec::new();
    let key = iced::keyboard::Key::Named(Named::ArrowRight);
    shifted.push(iced::event::Event::Keyboard(
        iced::keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key.clone(),
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers: iced::keyboard::Modifiers::SHIFT,
            repeat: false,
            text: None,
        },
    ));
    key_step(&mut shell, shifted);
    let eleven = shell.timeline_playhead();
    assert!(
        (eleven - 11.0 / 30.0).abs() < 1e-4,
        "Shift+→ が 10 コマでない: {eleven}"
    );
}

/// **Cmd+A = 全選択。** 見えている top-level object 行(Group は畳んだまま
/// 1行 — 中身の a/b/c は見えていないので選ばれない)が全部選ばれる。
/// egui 版 `self.selected = object_layers.clone()` と同じ意味(2026-08-19
/// 近道キー移植レーン)。
#[test]
fn command_a_selects_every_visible_object_row() {
    let (path, names) = fixture_project("select_all");
    let title_scene = layer_named(&names, "Title scene");
    let background = layer_named(&names, "Background");
    let audio = layer_named(&names, "starter-tone.wav");
    let mut shell = seated_shell(&path);

    assert!(
        shell.timeline_selection().is_empty(),
        "座った直後は何も選ばれていない"
    );

    key_step(&mut shell, common::command_key('a'));
    let mut selected = shell.timeline_selection();
    selected.sort_by_key(|layer| layer.get());
    let mut want = vec![title_scene, background, audio];
    want.sort_by_key(|layer| layer.get());
    assert_eq!(
        selected, want,
        "Cmd+A で見えている3行(Group 1 + clip 2)が選ばれない"
    );

    let select_all_intents = shell
        .intents()
        .iter()
        .filter(|event| {
            serde_json::to_string(&event.intent)
                .expect("serializable")
                .contains(r#""kind":"select_layer""#)
        })
        .count();
    assert_eq!(
        select_all_intents, 3,
        "3行ぶんの select_layer intent が journal に載る"
    );

    // もう一度 Cmd+A を押しても、既に選ばれている行を外してはいけない
    // (`additive: true` は Cmd+click と同じトグルなので、素朴に全行へ
    // もう一度出すと選択が消える — その回帰を防ぐ)。
    key_step(&mut shell, common::command_key('a'));
    let mut selected_again = shell.timeline_selection();
    selected_again.sort_by_key(|layer| layer.get());
    assert_eq!(
        selected_again, want,
        "2回目の Cmd+A で選択が変わってはいけない(トグルで外れる回帰)"
    );
    assert_eq!(
        shell
            .intents()
            .iter()
            .filter(|event| {
                serde_json::to_string(&event.intent)
                    .expect("serializable")
                    .contains(r#""kind":"select_layer""#)
            })
            .count(),
        select_all_intents,
        "2回目の Cmd+A で新しい select_layer intent が増えてはいけない(全部既に選ばれている)"
    );
}

// ---------------------------------------------------------------------------
// zoom / pan(view 状態 — intent にはならず、Message 列 replay で再現される)
// ---------------------------------------------------------------------------

/// **Cmd+ホイール = zoom。** カーソル下の時刻が動かない。
#[test]
fn command_wheel_zooms_around_the_cursor() {
    let (path, _names) = fixture_project("zoom");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);

    key_step(
        &mut shell,
        [iced::event::Event::Keyboard(
            iced::keyboard::Event::ModifiersChanged(iced::keyboard::Modifiers::COMMAND),
        )],
    );
    let intents_before = shell.intent_count();
    let anchor = Point::new(500.0, 300.0);
    let before = shell.timeline_pane().view;
    let anchor_t = geometry.x_to_time(before, anchor.x);
    pointer_step(
        &mut shell,
        anchor,
        [iced::event::Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
        })],
    );

    let after = shell.timeline_pane().view;
    assert!(after.span < before.span, "Cmd+ホイールで寄っていない");
    let after_t = geometry.x_to_time(after, anchor.x);
    assert!(
        (after_t - anchor_t).abs() < 1e-2,
        "カーソル下の時刻が動いた: {anchor_t} -> {after_t}"
    );

    // intent は増えない(view は Document でも選択でもない)。
    assert_eq!(
        shell.intent_count(),
        intents_before,
        "zoom が intent になっている"
    );
}

/// **Shift+ホイール = 横パン。** 窓は composition の外へ出ない。
#[test]
fn shift_wheel_pans_horizontally_inside_the_composition() {
    let (path, _names) = fixture_project("pan");
    let mut shell = seated_shell(&path);

    // まず寄る(span 16 のままでは comp 全体が見えていてパンできない)。
    key_step(
        &mut shell,
        [iced::event::Event::Keyboard(
            iced::keyboard::Event::ModifiersChanged(iced::keyboard::Modifiers::COMMAND),
        )],
    );
    pointer_step(
        &mut shell,
        Point::new(300.0, 300.0),
        [iced::event::Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: 2.0 },
        })],
    );
    key_step(
        &mut shell,
        [iced::event::Event::Keyboard(
            iced::keyboard::Event::ModifiersChanged(iced::keyboard::Modifiers::SHIFT),
        )],
    );
    let before = shell.timeline_pane().view;
    pointer_step(
        &mut shell,
        Point::new(500.0, 300.0),
        [iced::event::Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
        })],
    );
    let after = shell.timeline_pane().view;
    assert!(after.start > before.start, "Shift+ホイールでパンしていない");
    assert!(after.start + after.span <= 16.0 + 1e-3, "窓が composition を出た");
}

// ---------------------------------------------------------------------------
// replay oracle
// ---------------------------------------------------------------------------

/// **記録した intent 列は、窓の無い新しい gateway で同じ状態を再現する。**
/// 選択・revision・track item 数・playhead が一致し、Document の中身
/// (動かした clip の位置)も一致する。
#[test]
fn a_timeline_session_replays_from_its_intent_log() {
    let (path, names) = fixture_project("replay");
    let background = layer_named(&names, "Background");
    let audio = layer_named(&names, "starter-tone.wav");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let view0 = fresh_view();

    // 1セッション: 選ぶ → 動かす → trim → scrub → コマ送り。
    let bg_y = geometry.row_top(1, 0.0) + ROW_H / 2.0;
    let audio_y = geometry.row_top(2, 0.0) + ROW_H / 2.0;
    drag_gesture(
        &mut shell,
        Point::new(geometry.time_to_x(view0, 2.0), bg_y),
        Point::new(geometry.time_to_x(view0, 4.0), bg_y),
    );
    drag_gesture(
        &mut shell,
        Point::new(geometry.time_to_x(view0, 13.6) - 4.0, audio_y),
        Point::new(geometry.time_to_x(view0, 10.0), audio_y),
    );
    let ruler5 = Point::new(geometry.time_to_x(view0, 5.0), geometry.ruler_top() + 10.0);
    pointer_step(&mut shell, ruler5, [pressed()]);
    pointer_step(&mut shell, ruler5, [released()]);
    key_step(&mut shell, tap(Named::ArrowRight));

    let intents: Vec<_> = shell.intents().into_iter().map(|event| event.intent).collect();
    let driven_revision = shell.revision();
    let driven_items = shell.track_item_count();
    let driven_selection = shell.timeline_selection();
    let driven_playhead = shell.timeline_playhead();
    let driven_bg = clip_span_seconds(&shell.timeline_snapshot().expect("seated"), background);
    let driven_audio = clip_span_seconds(&shell.timeline_snapshot().expect("seated"), audio);

    // 座席は OS lock を握っている。replay が開けるよう先に殻ごと落とす。
    // ファイルは**消さない** — このセッションは保存していないので、開いた時点の
    // 初期状態はファイルにそのまま残っている。
    drop(shell);

    let replayed = ShellGateway::replay(&intents);
    assert_eq!(replayed.revision(), driven_revision, "writer 世代が一致する");
    assert_eq!(replayed.track_item_count(), driven_items, "Document の件数が一致する");
    let seat = replayed.project().expect("replay も座っている");
    assert_eq!(
        seat.editor().selected_layers().to_vec(),
        driven_selection,
        "選択が一致する"
    );
    assert!(
        (seat.editor().playhead_seconds() - driven_playhead).abs() < 1e-4,
        "playhead が一致する"
    );
    let replayed_doc = seat.snapshot();
    assert_eq!(
        clip_span_seconds(&replayed_doc, background),
        driven_bg,
        "動かした clip の位置まで一致する"
    );
    assert_eq!(clip_span_seconds(&replayed_doc, audio), driven_audio);
}

/// **同じ Message 列は同じ表示状態を作る。** zoom / scroll は intent にしない
/// 決定の対として、Message 列 replay が view の再現を持つ。
#[test]
fn the_same_message_sequence_reproduces_the_view_state() {
    let (path, _names) = fixture_project("view_replay");

    let session = |shell: &mut Shell| {
        key_step(
            shell,
            [iced::event::Event::Keyboard(
                iced::keyboard::Event::ModifiersChanged(iced::keyboard::Modifiers::COMMAND),
            )],
        );
        pointer_step(
            shell,
            Point::new(420.0, 300.0),
            [iced::event::Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x: 0.0, y: 2.0 },
            })],
        );
        key_step(
            shell,
            [iced::event::Event::Keyboard(
                iced::keyboard::Event::ModifiersChanged(iced::keyboard::Modifiers::SHIFT),
            )],
        );
        pointer_step(
            shell,
            Point::new(420.0, 300.0),
            [iced::event::Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x: 0.0, y: -2.0 },
            })],
        );
    };

    let mut first = seated_shell(&path);
    session(&mut first);
    let first_view = first.timeline_pane().view;
    let first_intents = first.intent_count();
    drop(first);

    let mut second = seated_shell(&path);
    session(&mut second);
    assert_eq!(
        second.timeline_pane().view,
        first_view,
        "同じ Message 列が同じ view にならない"
    );
    assert_eq!(
        second.intent_count(),
        first_intents,
        "view 操作が intent を増やしている"
    );
}

// ---------------------------------------------------------------------------
// 波形帯
// ---------------------------------------------------------------------------

/// **落とした曲は波形帯になる。** decode は別 thread で、`WaveformPolled` の刻みで
/// 届く(帯の高さと縮約の意味は egui 版と共用 — ここで審判するのは座席の配線)。
#[test]
fn a_dropped_soundtrack_grows_a_waveform_band() {
    use motolii_shell_iced::timeline::WaveformBandState;

    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("iced_timeline_waveform");
    let project = dir.join("fresh.json");
    let tone = common::starter_media_dir().join("starter-tone.wav");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project),
        ..ScriptedPrompts::default()
    });
    let pressed = common::press(
        iced_test::simulator(view(&shell)),
        motolii_shell_iced::view::NEW_PROJECT,
    );
    drain(&mut shell, pressed);
    assert!(shell.is_seated());

    let dropped = common::feed(
        iced_test::simulator(view(&shell)),
        [common::file_dropped(&tone), common::redraw()],
    );
    drain(&mut shell, dropped);
    assert!(
        shell
            .timeline_snapshot()
            .expect("seated")
            .soundtrack
            .is_some(),
        "まだ曲が無い project へ落ちた音声は曲になる(import_seat の決定)"
    );

    // decode 完了まで poll(製品では `window::frames()` の購読が刻む)。
    for _ in 0..600 {
        drain(&mut shell, vec![Message::WaveformPolled]);
        if matches!(shell.waveform_state(), WaveformBandState::Ready(_)) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    match shell.waveform_state() {
        WaveformBandState::Ready(peaks) => {
            assert!(peaks.source_seconds() > 0.0, "波形が空");
        }
        other => panic!("波形が届かない: {other:?}"),
    }
}
