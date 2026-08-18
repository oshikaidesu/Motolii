//! Timeline のキー編集(2026-08-19 M-6 キー編集レーン)の運転席 — red 先行。
//!
//! ## なぜ pixel を押さないのか
//!
//! property 行(`RowKind::Property`)は `TimelineFoldState::params_open` が
//! 立った時だけ画面に出るが、その開閉の**トグルは構造レーン(Group/Rename/
//! Lock/畳み開閉)の担当**で、この worktree の時点ではまだ配線されていない
//! (`canvas.rs::scene()` は `TimelineFoldState::default()` 固定 — 発注の柵
//! 「開閉のトグルは構造レーンの担当。本レーンは開いている時に正しく描くだけ」)。
//! そのため `iced_test::Simulator` で実クリックを打っても、property 行の
//! 菱形には物理的に当たらない。
//!
//! pixel→[`TimelineMsg`] の翻訳(`keys::grab_message` / `keys::menu_request` /
//! `semantics::key_hit_test`)は `keys.rs` / `semantics.rs` のユニットテストが
//! 別に審判済み。ここは**その続きの区間**——`TimelineMsg` を直接
//! `Shell::update` へ流し、`TimelinePane::plan` → `ShellGateway::dispatch` →
//! Document / journal / Undo — を審判する。`MoveClips` / `TrimClip` が
//! 実証した「release の1件だけが intent になる」設計そのものは
//! `drive_timeline.rs` が既に審判済みなので、ここでは繰り返さない。
//!
//! 検収の絵(`--screenshot`)は params_open を一時的にローカルで開いて撮った
//! もので、この diff には入っていない(`docs/reviews/evidence/
//! iced-timeline-port/keys/README.md` に手順と残差を記す)。

mod common;

use std::collections::HashMap;
use std::path::PathBuf;

use common::drain;
use motolii_doc::LayerId;
use motolii_shell_iced::timeline::TimelineMsg;
use motolii_shell_iced::{view, Message, ScriptedPrompts, Shell};
use motolii_ui::blitz_shell::{seconds_to_us, UiEditParam, UiKeyInterp};
use motolii_ui::timeline_editor::{lab_fixture, param_keys};
use motolii_ui::timeline_rows::ParamRef;

/// fixture(`lab_fixture` — egui 版と共用)を一時 project に保存する。
/// `drive_timeline.rs::fixture_project` と同じ経路だが、`tests/*.rs` は
/// 別々のクレートとしてコンパイルされるので複製している(小さいので許容)。
fn fixture_project(tag: &str) -> (PathBuf, HashMap<LayerId, String>) {
    let (document, names) = lab_fixture();
    let dir = motolii_testkit::tmp_dir(&format!("iced_timeline_keys_{tag}"));
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

fn timeline(msg: TimelineMsg) -> Message {
    Message::Timeline(msg)
}

/// キーボードだけの1フレーム(`drive_timeline.rs::key_step` と同じ経路)。
fn key_step(shell: &mut Shell, events: impl IntoIterator<Item = iced::event::Event>) {
    let messages = common::feed(iced_test::simulator(view(shell)), events);
    drain(shell, messages);
}

// ---------------------------------------------------------------------------
// Q1 文法(キー版): click=選択 / drag=移動 / release=確定 / Esc=復元 /
// Delete=削除 / 右クリック相当=補間 / 空トラックのクリック=playhead へ追加
// ---------------------------------------------------------------------------

/// **click(掴んで、動かさず離す)= 選択のみ。** Document は無傷で、
/// `MoveParamKey` も journal に残らない。
#[test]
fn clicking_a_key_selects_it_without_writing() {
    let (path, names) = fixture_project("click_select");
    let shared_left = layer_named(&names, "Shared left");
    let mut shell = seated_shell(&path);
    let before = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        shared_left,
        ParamRef::Position,
    );
    let (_, t0) = before[0];
    let revision_before = shell.revision();

    drain(
        &mut shell,
        vec![timeline(TimelineMsg::KeyGrabbed {
            layer: shared_left,
            param: UiEditParam::Position,
            at_seconds: t0,
            additive: false,
        })],
    );
    assert_eq!(
        shell.timeline_pane().selected_keys,
        vec![(shared_left, UiEditParam::Position, t0)],
        "掴んだキーが選ばれている"
    );

    drain(&mut shell, vec![timeline(TimelineMsg::PointerReleased)]);
    assert_eq!(
        shell.revision(),
        revision_before,
        "動かしていない release は無傷"
    );
    assert!(
        !shell
            .intents()
            .iter()
            .any(|e| serde_json::to_string(&e.intent)
                .unwrap()
                .contains("move_param_key")),
        "動いていないのに move_param_key が journal に載った"
    );
}

/// **drag = 移動、release = 確定。** 動かしたキーだけが動き、1ドラッグ =
/// 1 Undo 単位で Cmd+Z 1回で戻る。
#[test]
fn dragging_a_key_moves_it_and_lands_one_undo_unit() {
    let (path, names) = fixture_project("drag_move");
    let shared_left = layer_named(&names, "Shared left");
    let mut shell = seated_shell(&path);
    let before = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        shared_left,
        ParamRef::Scale,
    );
    assert_eq!(before.len(), 2);
    let (_, grabbed_t) = before[0];
    let other_t = before[1].1;
    let revision_before = shell.revision();

    drain(
        &mut shell,
        vec![
            timeline(TimelineMsg::KeyGrabbed {
                layer: shared_left,
                param: UiEditParam::Scale,
                at_seconds: grabbed_t,
                additive: false,
            }),
            timeline(TimelineMsg::PointerMoved { at_seconds: 4.0 }),
            timeline(TimelineMsg::PointerReleased),
        ],
    );

    let after = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        shared_left,
        ParamRef::Scale,
    );
    assert_eq!(after.len(), 2, "本数は変わらない");
    assert!(
        after.iter().any(|(_, t)| (*t - 4.0).abs() < 1e-2),
        "掴んだキーが 4.0s 付近へ動いた: {after:?}"
    );
    assert!(
        after.iter().any(|(_, t)| (*t - other_t).abs() < 1e-6),
        "もう1本はそのまま: {after:?}"
    );
    assert!(shell.revision() > revision_before, "revision が進む");

    let moves: Vec<String> = shell
        .intents()
        .iter()
        .map(|e| serde_json::to_string(&e.intent).unwrap())
        .filter(|line| line.contains("move_param_key"))
        .collect();
    assert_eq!(moves.len(), 1, "1ドラッグ = 1 move intent: {moves:?}");

    key_step(&mut shell, common::command_key('z'));
    let restored = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        shared_left,
        ParamRef::Scale,
    );
    assert!(
        restored.iter().any(|(_, t)| (*t - grabbed_t).abs() < 1e-3),
        "Cmd+Z 1回で掴む前の時刻へ戻る: {restored:?}"
    );
}

/// **Esc = 復元。** 掴んで引きずって Esc したら Document は無傷で、
/// journal に move intent は残らない(release まで Document を触らない設計)。
#[test]
fn escape_during_a_key_drag_restores_without_a_trace() {
    let (path, names) = fixture_project("esc_cancel");
    let shared_left = layer_named(&names, "Shared left");
    let mut shell = seated_shell(&path);
    let before = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        shared_left,
        ParamRef::Position,
    );
    let (_, t0) = before[0];
    let revision_before = shell.revision();
    let intents_before = shell.intents().len();

    drain(
        &mut shell,
        vec![
            timeline(TimelineMsg::KeyGrabbed {
                layer: shared_left,
                param: UiEditParam::Position,
                at_seconds: t0,
                additive: false,
            }),
            timeline(TimelineMsg::PointerMoved { at_seconds: 8.0 }),
            timeline(TimelineMsg::GestureCancelled),
            timeline(TimelineMsg::PointerReleased),
        ],
    );

    let after = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        shared_left,
        ParamRef::Position,
    );
    assert_eq!(after, before, "Esc で戻っていない: {after:?}");
    assert_eq!(shell.revision(), revision_before, "Document は無傷のはず");
    let new_intents: Vec<String> = shell
        .intents()
        .iter()
        .skip(intents_before)
        .map(|e| serde_json::to_string(&e.intent).unwrap())
        .collect();
    assert!(
        !new_intents
            .iter()
            .any(|line| line.contains("move_param_key")),
        "Esc したのに move intent が残った: {new_intents:?}"
    );
}

/// **キーが選ばれていれば、Delete はキーを消す。** clip 選択の削除
/// (`DeleteSelection`)へは回さない — 別レーンの担当と衝突しない。
#[test]
fn delete_removes_the_selected_key_not_the_clip() {
    let (path, names) = fixture_project("delete_key");
    let shared_right = layer_named(&names, "Shared right");
    let mut shell = seated_shell(&path);
    let before = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        shared_right,
        ParamRef::Position,
    );
    assert_eq!(before.len(), 3);
    let (_, doomed_t) = before[1];
    assert_eq!(shell.track_item_count(), 3, "fixture は top-level 3件");

    drain(
        &mut shell,
        vec![timeline(TimelineMsg::KeyGrabbed {
            layer: shared_right,
            param: UiEditParam::Position,
            at_seconds: doomed_t,
            additive: false,
        })],
    );
    drain(&mut shell, vec![timeline(TimelineMsg::DeletePressed)]);

    let after = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        shared_right,
        ParamRef::Position,
    );
    assert_eq!(after.len(), 2, "1つ消えた: {after:?}");
    assert!(
        !after.iter().any(|(_, t)| (*t - doomed_t).abs() < 1e-6),
        "消えたのは選んだキー"
    );
    assert_eq!(shell.track_item_count(), 3, "層(clip)は消えない");
    assert!(
        shell.timeline_pane().selected_keys.is_empty(),
        "消した後は選択も空(egui と同じ)"
    );

    let intent_lines: Vec<String> = shell
        .intents()
        .iter()
        .map(|e| serde_json::to_string(&e.intent).unwrap())
        .collect();
    assert!(
        !intent_lines
            .iter()
            .any(|line| line.contains("delete_selection")),
        "キー選択の Delete が clip の DeleteSelection へ漏れた: {intent_lines:?}"
    );
    assert!(
        intent_lines
            .iter()
            .any(|line| line.contains("remove_param_key")),
        "remove_param_key が journal に載っていない: {intent_lines:?}"
    );
}

/// **補間の指定(右クリックメニュー相当)は Position だけが D2 の口を持つ。**
/// `KeyInterpChosen` は intent としては出るが、Position 以外は revision が
/// 進まず、理由が status に残る(egui 版 `key_menu` と同じ穴)。
#[test]
fn interp_choice_only_writes_for_position() {
    let (path, names) = fixture_project("interp");
    let group = layer_named(&names, "Title scene"); // Group: Position keys [2.8, 10.1]
    let shared_left = layer_named(&names, "Shared left"); // Scale keys [2.4, 5.8]
    let mut shell = seated_shell(&path);

    let (_, position_t) = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        group,
        ParamRef::Position,
    )[0];
    let revision_before = shell.revision();
    drain(
        &mut shell,
        vec![timeline(TimelineMsg::KeyInterpChosen {
            layer: group,
            param: UiEditParam::Position,
            at_us: seconds_to_us(position_t),
            interp: UiKeyInterp::Linear,
        })],
    );
    assert!(
        shell.revision() > revision_before,
        "Position は D2 の口があるので書ける"
    );

    let (_, scale_t) = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        shared_left,
        ParamRef::Scale,
    )[0];
    let revision_before = shell.revision();
    drain(
        &mut shell,
        vec![timeline(TimelineMsg::KeyInterpChosen {
            layer: shared_left,
            param: UiEditParam::Scale,
            at_us: seconds_to_us(scale_t),
            interp: UiKeyInterp::Linear,
        })],
    );
    assert_eq!(
        shell.revision(),
        revision_before,
        "Scale は D2 の口が無いので書かない"
    );
    assert!(
        shell
            .editor_status()
            .is_some_and(|status| status.contains("Position-only")),
        "理由が status に残る: {:?}",
        shell.editor_status()
    );
}

/// **property 行の空いたトラックを押すと、playhead へキーを打つ。**
/// 既存の `KeyParamAtPlayhead`(M-4b が Inspector 用に足した1本)を
/// Timeline からも呼べることの審判(2026-08-19 M-6。新しい intent は作らない)。
#[test]
fn clicking_the_empty_track_adds_a_key_at_the_playhead() {
    let (path, names) = fixture_project("add_key");
    let background = layer_named(&names, "Background"); // fixture: キーを1本も持たない
    let mut shell = seated_shell(&path);
    assert!(
        param_keys(
            &shell.timeline_snapshot().expect("seated"),
            background,
            ParamRef::Opacity
        )
        .is_empty(),
        "fixture は Background に Opacity キーを持たない"
    );

    // playhead を 2.0s へ(fps の格子に載る値 — scrub のルーラ操作と同じ経路)。
    drain(
        &mut shell,
        vec![
            timeline(TimelineMsg::ScrubStarted { at_seconds: 2.0 }),
            timeline(TimelineMsg::PointerReleased),
        ],
    );
    assert!(
        (shell.timeline_playhead() - 2.0).abs() < 1e-3,
        "playhead が 2.0s に立っていない: {}",
        shell.timeline_playhead()
    );

    drain(
        &mut shell,
        vec![timeline(TimelineMsg::KeyTrackClicked {
            layer: background,
            param: UiEditParam::Opacity,
        })],
    );

    let after = param_keys(
        &shell.timeline_snapshot().expect("seated"),
        background,
        ParamRef::Opacity,
    );
    assert_eq!(after.len(), 1, "1本増えた: {after:?}");
    assert!(
        (after[0].1 - 2.0).abs() < 1e-3,
        "playhead の時刻に立つ: {after:?}"
    );

    let intent_lines: Vec<String> = shell
        .intents()
        .iter()
        .map(|e| serde_json::to_string(&e.intent).unwrap())
        .collect();
    assert!(
        intent_lines
            .iter()
            .any(|line| line.contains("key_param_at_playhead")),
        "既存の KeyParamAtPlayhead をそのまま呼んでいない: {intent_lines:?}"
    );
}
