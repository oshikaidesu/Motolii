//! **拒否理由が実際に status 帯へ出る**ことの運転席(2026-08-19 拒否可視性レーン)。
//!
//! `TimelineEditor::take_rejections` → `ShellGateway::edit`/`with_editor` →
//! `ShellTranscript::report` → `Shell::latest_report` → `view::status_band`
//! という配管そのものは既に通っている(`intent.rs` の `edit`/`with_editor` が
//! 呼び手を問わず `take_rejections` を毎回 transcript へ写す)。これまで
//! 無かったのは「実際に絵まで出るか」を審判するテストである —
//! `drive_timeline_structure.rs` のロック中ドラッグテスト(`the_lock_button_
//! sets_an_explicit_value_and_blocks_the_move_gesture`)は位置が動かないこと
//! だけを見ていて、`latest_report()` の文言は誰も見ていなかった。
//!
//! `TimelineEditor::reject(` の全呼び出しを洗い、iced から実際に到達できる
//! ものだけをここで審判する(到達できない・判定に迷うものは実装せず、
//! RETURN の EVIDENCE_GAP に書く)。

mod common;

use std::collections::HashMap;
use std::path::PathBuf;

use common::drain;
use iced::mouse;
use iced::Point;
use motolii_doc::LayerId;
use motolii_shell_iced::timeline::semantics::{initial_view, PaneGeometry, ROW_H};
use motolii_shell_iced::timeline::TimelineMsg;
use motolii_shell_iced::{view, Message, Outcome, ScriptedPrompts, Shell};
use motolii_ui::blitz_shell::{seconds_to_us, UiEditParam, UiKeyInterp};
use motolii_ui::timeline_editor::{lab_fixture, param_keys, TimelineView};
use motolii_ui::timeline_rows::ParamRef;

// ---------------------------------------------------------------------------
// 支度(`drive_timeline_structure.rs` / `drive_timeline_keys.rs` と同じ型。
// test binary 間で共有できないので複製する)。
// ---------------------------------------------------------------------------

fn fixture_project(tag: &str) -> (PathBuf, HashMap<LayerId, String>) {
    let (document, names) = lab_fixture();
    let dir = motolii_testkit::tmp_dir(&format!("iced_rejections_{tag}"));
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
    let events: Vec<iced::event::Event> = std::iter::once(cursor_moved(at)).chain(events).collect();
    let _ = ui.simulate(events);
    let messages: Vec<Message> = ui.into_messages().collect();
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

fn timeline(msg: TimelineMsg) -> Message {
    Message::Timeline(msg)
}

/// TimelineMsg を1つずつ、直接 `Shell::update` へ流す(`drive_timeline_keys.rs`
/// と同じ型)。**pixel を経由しない**——bar/L ボタンの当たり判定は既に
/// `drive_timeline.rs` / `drive_timeline_structure.rs` が審判済みなので、
/// ここでは「その Message が来たら断りが帯まで届くか」だけを訊く。
fn dispatch(shell: &mut Shell, msg: TimelineMsg) -> Outcome {
    drain(shell, vec![timeline(msg)])
}

/// **最新の一言が transcript に載っていて、かつ実際に status 帯の絵として
/// 見えている**ことを1箇所で審判する。`drive_seat.rs` の
/// `the_start_screen_seats_a_project_without_a_native_dialog` と同じ型
/// (「言ったつもりで出ていない」を通さない)。
fn assert_reason_is_shown(shell: &Shell, needle: &str) {
    let latest = shell.latest_report();
    assert!(
        latest.as_deref().is_some_and(|text| text.contains(needle)),
        "latest_report に理由が乗っていない。期待の断片: {needle:?} / 実際: {latest:?}"
    );
    let latest = latest.expect("checked above");
    let mut ui = iced_test::simulator(view(shell));
    assert!(
        ui.find(latest.as_str()).is_ok(),
        "status 帯が最新の一言を映していない。実際: {latest:?}"
    );
}

// ---------------------------------------------------------------------------
// 1) BarGrabbed(Body)— ロック中の clip を掴むと、動かす前に断りが帯へ出る。
//    `TimelinePane::plan` の 2026-08-19 修復(0幅 move の probe)を審判する。
// ---------------------------------------------------------------------------

#[test]
fn dragging_a_locked_clip_reports_the_lock_reason_in_the_status_band() {
    let (path, names) = fixture_project("locked_move");
    let background = layer_named(&names, "Background");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let view0 = fresh_view();

    dispatch(&mut shell, TimelineMsg::LockPressed { layer: background });
    assert!(
        shell.timeline_snapshot().is_some_and(|document| {
            motolii_shell_iced::timeline::semantics::row_own_lock(&document, background)
        }),
        "前提: L を押したらロックが掛かる"
    );

    let before = motolii_doc::find_item_location(&shell.timeline_snapshot().unwrap(), background)
        .map(|(_, _, item)| match item {
            motolii_doc::TrackItem::Clip(c) => c.start,
            _ => panic!("Background は clip のはず"),
        })
        .expect("location");

    let y = geometry.row_top(1, 0.0) + ROW_H / 2.0;
    drag_gesture(
        &mut shell,
        Point::new(geometry.time_to_x(view0, 2.0), y),
        Point::new(geometry.time_to_x(view0, 5.0), y),
    );

    let after = motolii_doc::find_item_location(&shell.timeline_snapshot().unwrap(), background)
        .map(|(_, _, item)| match item {
            motolii_doc::TrackItem::Clip(c) => c.start,
            _ => panic!("Background は clip のはず"),
        })
        .expect("location");
    assert_eq!(before, after, "ロック中は掴んでも動かない(前提の再確認)");

    assert_reason_is_shown(&shell, "Background is locked");
}

// ---------------------------------------------------------------------------
// 2) BarGrabbed(Edge)— ロック中の clip の端を掴んでも同じ断りが出る。
// ---------------------------------------------------------------------------

#[test]
fn trimming_a_locked_clip_reports_the_lock_reason_in_the_status_band() {
    let (path, names) = fixture_project("locked_trim");
    let background = layer_named(&names, "Background");
    let mut shell = seated_shell(&path);
    let geometry = geometry(&shell);
    let view0 = fresh_view();

    dispatch(&mut shell, TimelineMsg::LockPressed { layer: background });

    // Background は 0.0..13.4s。右端の内側4pxを掴む(`drive_timeline.rs` の
    // trim テストと同じ距離)。
    let y = geometry.row_top(1, 0.0) + ROW_H / 2.0;
    drag_gesture(
        &mut shell,
        Point::new(geometry.time_to_x(view0, 13.4) - 4.0, y),
        Point::new(geometry.time_to_x(view0, 10.0), y),
    );

    assert_reason_is_shown(&shell, "Background is locked");
}

// ---------------------------------------------------------------------------
// 3) RenameStarted — ロック中の行は編集欄を開く前に断られる
//    (`TimelinePane::plan` の RenameLayer probe。egui 版と違い、iced は
//    commit 時ではなく開始時点で断る)。
// ---------------------------------------------------------------------------

#[test]
fn starting_a_rename_on_a_locked_layer_reports_the_lock_reason() {
    let (path, names) = fixture_project("locked_rename");
    let background = layer_named(&names, "Background");
    let mut shell = seated_shell(&path);

    dispatch(&mut shell, TimelineMsg::LockPressed { layer: background });
    dispatch(&mut shell, TimelineMsg::RenameStarted { layer: background });

    assert!(
        shell.timeline_pane().renaming.is_none(),
        "ロック中は編集欄を開かない"
    );
    assert_reason_is_shown(&shell, "Background is locked");
}

// ---------------------------------------------------------------------------
// 4) UngroupLayer(locked)— ロック中の Group を Cmd+Shift+G しても断られる。
// ---------------------------------------------------------------------------

#[test]
fn ungrouping_a_locked_group_reports_the_lock_reason() {
    let (path, names) = fixture_project("locked_ungroup");
    let title_scene = layer_named(&names, "Title scene");
    let mut shell = seated_shell(&path);

    dispatch(&mut shell, TimelineMsg::LockPressed { layer: title_scene });
    dispatch(
        &mut shell,
        TimelineMsg::RowPicked {
            layer: title_scene,
            additive: false,
        },
    );
    dispatch(&mut shell, TimelineMsg::UngroupPressed);

    assert_reason_is_shown(&shell, "Title scene is locked");
}

// ---------------------------------------------------------------------------
// 5) UngroupLayer(not a group)— 単独選択が Group でなければ断られる。
// ---------------------------------------------------------------------------

#[test]
fn ungrouping_a_non_group_layer_reports_the_reason() {
    let (path, names) = fixture_project("ungroup_non_group");
    let background = layer_named(&names, "Background");
    let mut shell = seated_shell(&path);

    dispatch(
        &mut shell,
        TimelineMsg::RowPicked {
            layer: background,
            additive: false,
        },
    );
    dispatch(&mut shell, TimelineMsg::UngroupPressed);

    assert_reason_is_shown(&shell, "Background is not a group");
}

// ---------------------------------------------------------------------------
// 6) DuplicateSelection(全部ロック済み)— editable_roots が「N locked,
//    skipped」を積む。
// ---------------------------------------------------------------------------

#[test]
fn duplicating_an_all_locked_selection_reports_the_skip_reason() {
    let (path, names) = fixture_project("locked_duplicate");
    let background = layer_named(&names, "Background");
    let mut shell = seated_shell(&path);

    dispatch(&mut shell, TimelineMsg::LockPressed { layer: background });
    dispatch(
        &mut shell,
        TimelineMsg::RowPicked {
            layer: background,
            additive: false,
        },
    );
    let before_count = shell.track_item_count();
    dispatch(&mut shell, TimelineMsg::DuplicatePressed);

    assert_reason_is_shown(&shell, "1 locked, skipped");
    assert_eq!(
        shell.track_item_count(),
        before_count,
        "断られた duplicate は Document を増やさない"
    );
}

// ---------------------------------------------------------------------------
// 7) DeleteSelection(全部ロック済み)— 同じ editable_roots の断りが Delete
//    でも出る(clip 選択の Delete。キー選択とは別経路 — #10 参照)。
// ---------------------------------------------------------------------------

#[test]
fn deleting_an_all_locked_selection_reports_the_skip_reason() {
    let (path, names) = fixture_project("locked_delete");
    let background = layer_named(&names, "Background");
    let mut shell = seated_shell(&path);

    dispatch(&mut shell, TimelineMsg::LockPressed { layer: background });
    dispatch(
        &mut shell,
        TimelineMsg::RowPicked {
            layer: background,
            additive: false,
        },
    );
    let before_count = shell.track_item_count();
    dispatch(&mut shell, TimelineMsg::DeletePressed);

    assert_reason_is_shown(&shell, "1 locked, skipped");
    assert_eq!(
        shell.track_item_count(),
        before_count,
        "断られた delete は Document を減らさない"
    );
}

// ---------------------------------------------------------------------------
// 8) GroupLayers(親が違う)— 別々の階層の選択を Cmd+G すると断られる。
// ---------------------------------------------------------------------------

#[test]
fn grouping_a_selection_across_different_parents_reports_the_reason() {
    let (path, names) = fixture_project("group_mismatch");
    let shared_left = layer_named(&names, "Shared left"); // Title scene の子
    let background = layer_named(&names, "Background"); // トップレベル
    let title_scene = layer_named(&names, "Title scene");
    let mut shell = seated_shell(&path);

    // 子を見えるようにしないと選べない(構造テストと同じ手順)。
    dispatch(
        &mut shell,
        TimelineMsg::FoldToggled {
            layer: title_scene,
            params: false,
        },
    );
    dispatch(
        &mut shell,
        TimelineMsg::RowPicked {
            layer: shared_left,
            additive: false,
        },
    );
    dispatch(
        &mut shell,
        TimelineMsg::RowPicked {
            layer: background,
            additive: true,
        },
    );
    dispatch(&mut shell, TimelineMsg::GroupPressed);

    assert_reason_is_shown(&shell, "group: pick items in the same parent");
}

// ---------------------------------------------------------------------------
// 9) MoveParamKey(locked)— ロック中の行のキーをドラッグして離すと断られる。
//    `KeyGrabbed` の note() にはロック確認が無い(BarGrabbed と違って preview
//    は動く)ので、release で初めて intent が dispatch され、そこで断られる。
// ---------------------------------------------------------------------------

#[test]
fn moving_a_key_on_a_locked_layer_reports_the_lock_reason() {
    let (path, names) = fixture_project("locked_key_move");
    let shared_left = layer_named(&names, "Shared left");
    let mut shell = seated_shell(&path);
    let before = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        shared_left,
        ParamRef::Position,
    );
    let (_, t0) = before[0];

    dispatch(&mut shell, TimelineMsg::LockPressed { layer: shared_left });
    dispatch(
        &mut shell,
        TimelineMsg::KeyGrabbed {
            layer: shared_left,
            param: UiEditParam::Position,
            at_seconds: t0,
            additive: false,
        },
    );
    dispatch(
        &mut shell,
        TimelineMsg::PointerMoved {
            at_seconds: t0 + 2.0,
        },
    );
    dispatch(&mut shell, TimelineMsg::PointerReleased);

    let after = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        shared_left,
        ParamRef::Position,
    );
    assert_eq!(before, after, "ロック中はキーも動かない");
    assert_reason_is_shown(&shell, "Shared left is locked");
}

// ---------------------------------------------------------------------------
// 10) RemoveParamKey(locked)— ロック中の行で選んだキーを Delete しても
//     断られる(clip 選択の Delete とは別経路 — #7 参照)。
// ---------------------------------------------------------------------------

#[test]
fn deleting_a_key_on_a_locked_layer_reports_the_lock_reason() {
    let (path, names) = fixture_project("locked_key_delete");
    let shared_left = layer_named(&names, "Shared left");
    let mut shell = seated_shell(&path);
    let before = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        shared_left,
        ParamRef::Position,
    );
    let (_, t0) = before[0];

    dispatch(&mut shell, TimelineMsg::LockPressed { layer: shared_left });
    dispatch(
        &mut shell,
        TimelineMsg::KeyGrabbed {
            layer: shared_left,
            param: UiEditParam::Position,
            at_seconds: t0,
            additive: false,
        },
    );
    dispatch(&mut shell, TimelineMsg::DeletePressed);

    let after = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        shared_left,
        ParamRef::Position,
    );
    assert_eq!(before, after, "ロック中はキーを消せない");
    assert_reason_is_shown(&shell, "Shared left is locked");
}

// ---------------------------------------------------------------------------
// 11) SetParamKeyInterp(Position 以外)— D2 に口が無いので断られる。
//     `drive_timeline_keys.rs::interp_choice_only_writes_for_position` は
//     `editor_status()` しか見ていない — ここでは transcript / 帯まで見る。
// ---------------------------------------------------------------------------

#[test]
fn choosing_a_non_position_interp_reports_the_reason_in_the_status_band() {
    let (path, names) = fixture_project("interp_reject");
    let shared_left = layer_named(&names, "Shared left");
    let mut shell = seated_shell(&path);
    let (_, scale_t) = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        shared_left,
        ParamRef::Scale,
    )[0];
    let revision_before = shell.revision();

    dispatch(
        &mut shell,
        TimelineMsg::KeyInterpChosen {
            layer: shared_left,
            param: UiEditParam::Scale,
            at_us: seconds_to_us(scale_t),
            interp: UiKeyInterp::Linear,
        },
    );

    assert_eq!(
        shell.revision(),
        revision_before,
        "Scale は D2 の口が無いので書かない(前提の再確認)"
    );
    assert_reason_is_shown(&shell, "Position-only in D2");
}
