//! 運転席 — MB-1(File 束、裁定176 `docs/reviews/2026-08-22-file-dialog-decision.md`)。
//! New Project/Save As/Save a Copy/Quit の dirty ガード・実書き込み往復・
//! menu+shortcut 両入口を見る。
//!
//! **production の `Shell::new()`(= 生の [`RfdDialogs`](motolii_shell::file_dialogs::RfdDialogs))は
//! 一度もここで呼ばない** — Save As/Save a Copy は実際に OS のネイティブ file
//! dialog を開こうとし(ヘッドレス CI では固まる)、Quit は `std::process::exit(0)`
//! で**テストバイナリ自身を道連れにする**(`suite_main.rs` が `tests/suite/*.rs`
//! 全部を1バイナリへ束ねているため、他ファイルの残りのテストも巻き添えで消える)。
//! この2つの理由がまさに `file_dialogs.rs` の `FileDialogs` trait を注入可能に
//! した理由そのもの — ここでは常に [`Shell::new_with_dialogs`] + [`FakeDialogs`]
//! を使う(`FakeDialogs::quit` は回数を記録するだけで実際には終了しない)。
//!
//! 同じ理由で、`q0_fence.rs`(全 target を自動クリックする横断柵)へ File
//! ドロップダウンの状態は追加していない — 追加すると `Quit` ボタンが機械的に
//! クリックされ、production dialogs 経由なら test binary を殺しかねない
//! (`menu.rs::Item::message` が `Option` ではなく必須フィールドである時点で
//! 「on_press 無しの死に chrome」は型で排除済みなので、Q0 適合は静的に保証
//! されている — 自動クリック柵を通す必要は無い)。

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use iced::keyboard::{Key, Modifiers};
use iced_test::selector::Target;

use motolii_shell::file_dialogs::FileDialogs;
use motolii_shell::{resolve_navigation_key, Message, Shell};

use crate::target_walk::collect_targets;

// ---------------------------------------------------------------------------
// fake FileDialogs — 缶詰応答 + 呼び出し回数の記録
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct FakeDialogsState {
    confirm_discard_response: Cell<bool>,
    confirm_discard_calls: Cell<u32>,
    save_path_response: RefCell<Option<PathBuf>>,
    quit_calls: Cell<u32>,
}

/// [`FileDialogs`] の test 実装。`Rc` で包んであるのは、`Shell` へ渡す
/// `Box<dyn FileDialogs>`(所有権が移る)とは別に、test 側が呼び出し回数・
/// 応答を読み書きできる**もう1つの取っ手**を残すため(`Cell`/`RefCell` で
/// 内部可変性を持つので `&self` のままで足りる — trait のメソッドはどれも
/// `&self`)。
#[derive(Debug, Clone, Default)]
struct FakeDialogs(Rc<FakeDialogsState>);

impl FakeDialogs {
    fn set_confirm_discard(&self, value: bool) {
        self.0.confirm_discard_response.set(value);
    }

    fn confirm_discard_calls(&self) -> u32 {
        self.0.confirm_discard_calls.get()
    }

    fn set_save_path(&self, path: PathBuf) {
        *self.0.save_path_response.borrow_mut() = Some(path);
    }

    fn quit_calls(&self) -> u32 {
        self.0.quit_calls.get()
    }
}

impl FileDialogs for FakeDialogs {
    fn confirm_discard(&self) -> bool {
        self.0.confirm_discard_calls.set(self.0.confirm_discard_calls.get() + 1);
        self.0.confirm_discard_response.get()
    }

    fn pick_save_path(&self) -> Option<PathBuf> {
        self.0.save_path_response.borrow().clone()
    }

    fn quit(&self) {
        // **実際には終了しない** — production(`RfdDialogs::quit`)との唯一の
        // 違いがここ。他は缶詰応答という点で同格。
        self.0.quit_calls.set(self.0.quit_calls.get() + 1);
    }
}

fn shell_with_fake() -> (Shell, FakeDialogs) {
    let fake = FakeDialogs::default();
    let (shell, _task) = Shell::new_with_dialogs(Box::new(fake.clone()));
    (shell, fake)
}

// ---------------------------------------------------------------------------
// (a) New Project — dirty ガード → 確認 → リセット
// ---------------------------------------------------------------------------

#[test]
fn new_project_is_a_no_op_without_confirmation_when_the_document_is_clean() {
    let (mut shell, fake) = shell_with_fake();
    assert!(!shell.is_project_dirty(), "起動直後なのに dirty 扱いになっている");

    let _ = shell.update(Message::NewProjectRequested);

    assert_eq!(
        fake.confirm_discard_calls(),
        0,
        "dirty ではないのに確認ダイアログを呼んでいる"
    );
}

#[test]
fn new_project_prompts_when_dirty_and_only_resets_the_document_after_confirmation() {
    let (mut shell, fake) = shell_with_fake();
    let _ = shell.update(Message::AddLayer);
    assert_eq!(shell.layer_count(), 1);
    assert!(shell.is_project_dirty(), "AddLayer 後なのに dirty になっていない");

    // 缶詰応答の既定は false(キャンセル)。
    let _ = shell.update(Message::NewProjectRequested);
    assert_eq!(fake.confirm_discard_calls(), 1, "dirty なのに確認していない");
    assert_eq!(shell.layer_count(), 1, "キャンセルしたのに Document がリセットされている");
    assert!(shell.is_project_dirty(), "キャンセルしたのに dirty が消えている");

    // 確認 → 実際にリセットされる。undo 履歴も消えている(revision 初期化)。
    fake.set_confirm_discard(true);
    let _ = shell.update(Message::NewProjectRequested);
    assert_eq!(fake.confirm_discard_calls(), 2);
    assert_eq!(shell.layer_count(), 0, "確認後も Document がリセットされていない");
    assert!(!shell.can_undo(), "リセット後に前 project の undo 履歴が残っている(revision 初期化されていない)");
    assert!(!shell.is_project_dirty(), "リセット直後なのに dirty のまま");
    assert_eq!(shell.current_path(), None, "New Project 後も current_path が残っている");
}

// ---------------------------------------------------------------------------
// (b) Quit — 同じ dirty ガードの型(実際には終了しない fake で検分)
// ---------------------------------------------------------------------------

#[test]
fn quit_terminates_immediately_when_the_document_is_clean() {
    let (mut shell, fake) = shell_with_fake();
    let _ = shell.update(Message::QuitRequested);
    assert_eq!(fake.confirm_discard_calls(), 0, "dirty ではないのに確認している");
    assert_eq!(fake.quit_calls(), 1, "clean な Document で Quit したのに終了していない");
}

#[test]
fn quit_is_cancelled_when_the_user_declines_to_discard_dirty_changes() {
    let (mut shell, fake) = shell_with_fake();
    let _ = shell.update(Message::AddLayer);
    fake.set_confirm_discard(false);

    let _ = shell.update(Message::QuitRequested);

    assert_eq!(fake.confirm_discard_calls(), 1, "dirty なのに確認していない");
    assert_eq!(fake.quit_calls(), 0, "キャンセルしたのに終了している");

    fake.set_confirm_discard(true);
    let _ = shell.update(Message::QuitRequested);
    assert_eq!(fake.quit_calls(), 1, "確認後も終了していない");
}

// ---------------------------------------------------------------------------
// (c) Save As — 一時 dir への実書き込み → 読み戻し一致
// ---------------------------------------------------------------------------

#[test]
fn save_as_writes_a_real_file_that_reads_back_with_identical_content() {
    let (mut shell, fake) = shell_with_fake();
    let _ = shell.update(Message::AddLayer);
    let _ = shell.update(Message::AddLayer);
    assert_eq!(shell.layer_count(), 2);

    let dir = motolii_testkit::tmp_dir("file-drive-save-as");
    let path = dir.join("save-as-roundtrip.motolii");
    fake.set_save_path(path.clone());

    let _ = shell.update(Message::SaveAsRequested);

    // **実書き込み**(器具・フェイクではなく本物の `std::fs` 経路)。
    let bytes = std::fs::read(&path).expect("Save As が実 file を書いていない");
    assert!(!bytes.is_empty(), "Save As が空 file を書いている");

    // Shell 側の身分も更新されている(current_path/dirty)。
    assert_eq!(shell.current_path(), Some(path.as_path()), "Save As 後に current_path が更新されていない");
    assert!(!shell.is_project_dirty(), "Save As 直後なのに dirty のまま");

    // **読み戻し一致** — 既存の汎用 persist 経路(`Document::load`、
    // `persist.rs` doc 参照)でそのまま読める、かつ書いた内容(layer 数)が
    // 一致する。新しい保存形式を発明していないことの直接証拠(裁定176)。
    let loaded = motolii_store::Document::load(&path).expect("Save As が書いた file を読み戻せない");
    assert_eq!(loaded.view().layers().len(), 2, "読み戻した layer 数が書いた数と一致しない");
    assert!(!loaded.can_undo(), "読み戻し直後は undo できないはず(flattened 保存の仕様どおり)");
}

#[test]
fn save_as_does_nothing_when_the_dialog_is_cancelled() {
    let (mut shell, _fake) = shell_with_fake();
    let _ = shell.update(Message::AddLayer);
    // `_fake.save_path_response` は既定で `None`(キャンセル相当)。

    let _ = shell.update(Message::SaveAsRequested);

    assert_eq!(shell.current_path(), None, "キャンセルしたのに current_path が設定されている");
    assert!(shell.is_project_dirty(), "キャンセルしたのに dirty が消えている");
}

// ---------------------------------------------------------------------------
// (d) Save a Copy — 現 path 維持のまま別名へ書く
// ---------------------------------------------------------------------------

#[test]
fn save_a_copy_writes_a_file_without_touching_the_current_path_or_dirty_state() {
    let (mut shell, fake) = shell_with_fake();
    let _ = shell.update(Message::AddLayer);

    let dir = motolii_testkit::tmp_dir("file-drive-save-a-copy");
    let primary = dir.join("primary.motolii");
    fake.set_save_path(primary.clone());
    let _ = shell.update(Message::SaveAsRequested);
    assert_eq!(shell.current_path(), Some(primary.as_path()));
    assert!(!shell.is_project_dirty());

    // Save As 後にさらに編集して dirty へ戻す。
    let _ = shell.update(Message::AddLayer);
    assert!(shell.is_project_dirty());

    let copy_path = dir.join("a-copy.motolii");
    fake.set_save_path(copy_path.clone());
    let _ = shell.update(Message::SaveACopyRequested);

    // コピー先には実際に書かれている。
    let loaded = motolii_store::Document::load(&copy_path).expect("Save a Copy が file を書いていない");
    assert_eq!(loaded.view().layers().len(), 2, "コピー先の layer 数が一致しない");

    // だが「現 path 維持」— current_path も dirty 状態も Save As 時点のまま。
    assert_eq!(
        shell.current_path(),
        Some(primary.as_path()),
        "Save a Copy で current_path が書き換わっている(仕様違反)"
    );
    assert!(
        shell.is_project_dirty(),
        "Save a Copy で dirty が消えている(仕様違反 — コピーは開いている project の身分を変えない)"
    );
}

// ---------------------------------------------------------------------------
// (e) S6 併存 — menu と shortcut が同じ Message へ収束する(構造証明)
// ---------------------------------------------------------------------------

fn text_targets<'a>(targets: &'a [Target], content: &str) -> Vec<&'a Target> {
    targets
        .iter()
        .filter(|t| matches!(t, Target::Text { content: c, .. } if c == content))
        .collect()
}

fn center(target: &Target) -> iced::Point {
    let bounds = target
        .visible_bounds()
        .expect("target の bounds が見える(window 内にあるはず)");
    iced::Point::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height / 2.0)
}

fn click_at(element: iced::Element<'_, Message>, point: iced::Point) -> Vec<Message> {
    let mut ui = iced_test::simulator(element);
    ui.point_at(point);
    let _ = ui.simulate([
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
    ]);
    ui.into_messages().collect()
}

/// shortcut 側: `resolve_navigation_key`(`nav_drive.rs`/`shortcut_drive.rs` と
/// 同じ入口)が File 束4動詞を正しい `Message` へ解決する。**`Shell::update` は
/// 一切呼ばない**(crate 冒頭 doc の理由どおり — 特に Quit を呼ぶと production
/// なら test binary が終了する)。
#[test]
fn file_shortcuts_resolve_to_the_expected_messages() {
    let new_project = Key::Character("n".into());
    assert!(
        matches!(
            resolve_navigation_key(&new_project, Modifiers::COMMAND, false),
            Some(Message::NewProjectRequested)
        ),
        "Cmd+N が NewProjectRequested を出さない"
    );

    let save_as = Key::Character("s".into());
    assert!(
        matches!(
            resolve_navigation_key(&save_as, Modifiers::COMMAND | Modifiers::SHIFT, false),
            Some(Message::SaveAsRequested)
        ),
        "Cmd+Shift+S が SaveAsRequested を出さない"
    );

    let quit = Key::Character("q".into());
    assert!(
        matches!(resolve_navigation_key(&quit, Modifiers::COMMAND, false), Some(Message::QuitRequested)),
        "Cmd+Q が QuitRequested を出さない"
    );

    // captured ガード(`nav_drive.rs`/`shortcut_drive.rs` と同じ形) — text 編集中は
    // 一切奪わない。
    for (key, modifiers) in [
        (&new_project, Modifiers::COMMAND),
        (&save_as, Modifiers::COMMAND | Modifiers::SHIFT),
        (&quit, Modifiers::COMMAND),
    ] {
        assert!(
            resolve_navigation_key(key, modifiers, true).is_none(),
            "captured=true なのに {key:?}+{modifiers:?} が Message を出している"
        );
    }
}

/// menu 側: File を開くと4項目が Target として現れ、クリックすると shortcut と
/// **同じ** `Message` variant が publish される — S6 併存(menu と shortcut が
/// 同じ入口)の直接証拠。`click_at` は `Simulator` から `Message` を読むだけで
/// `shell.update` を呼ばない(このテストは「何が publish されるか」だけを見る
/// — 実際に適用すると Save As/Quit が fake とはいえ状態を動かすので、他の
/// テストと責務を分ける)。
#[test]
fn clicking_file_menu_items_publishes_the_same_messages_as_their_shortcuts() {
    let (mut shell, _fake) = shell_with_fake();

    let before = collect_targets(shell.view());
    let file_trigger = text_targets(&before, "File");
    assert_eq!(file_trigger.len(), 1, "header に \"File\" トリガーが1つだけ見えるはず: {file_trigger:?}");
    let trigger_point = center(file_trigger[0]);

    let messages = click_at(shell.view(), trigger_point);
    assert_eq!(messages.len(), 1, "File クリックが期待どおり1件の Message を出さない: {messages:?}");
    assert!(
        matches!(messages[0], Message::ToggleFileMenu),
        "File クリックが ToggleFileMenu 以外を出している: {messages:?}"
    );
    for message in messages {
        let _ = shell.update(message);
    }

    let after = collect_targets(shell.view());

    let new_project = text_targets(&after, "New Project");
    assert_eq!(new_project.len(), 1, "dropdown に New Project が見えない: {after:?}");
    let messages = click_at(shell.view(), center(new_project[0]));
    assert_eq!(messages.len(), 1);
    assert!(matches!(messages[0], Message::NewProjectRequested), "{messages:?}");

    let save_as = text_targets(&after, "Save As…");
    assert_eq!(save_as.len(), 1, "dropdown に Save As… が見えない: {after:?}");
    let messages = click_at(shell.view(), center(save_as[0]));
    assert_eq!(messages.len(), 1);
    assert!(matches!(messages[0], Message::SaveAsRequested), "{messages:?}");

    let save_a_copy = text_targets(&after, "Save a Copy…");
    assert_eq!(save_a_copy.len(), 1, "dropdown に Save a Copy… が見えない: {after:?}");
    let messages = click_at(shell.view(), center(save_a_copy[0]));
    assert_eq!(messages.len(), 1);
    assert!(matches!(messages[0], Message::SaveACopyRequested), "{messages:?}");

    let quit = text_targets(&after, "Quit");
    assert_eq!(quit.len(), 1, "dropdown に Quit が見えない: {after:?}");
    let messages = click_at(shell.view(), center(quit[0]));
    assert_eq!(messages.len(), 1);
    assert!(matches!(messages[0], Message::QuitRequested), "{messages:?}");

    // 各 shortcut 表記も併記されている(New Project/Save As/Quit の3本 —
    // Save a Copy は上で確認済みのとおり出典ゼロ)。
    assert_eq!(text_targets(&after, "Cmd+N").len(), 1, "New Project の shortcut 表記が見えない");
    assert_eq!(text_targets(&after, "Cmd+Shift+S").len(), 1, "Save As の shortcut 表記が見えない");
    assert_eq!(text_targets(&after, "Cmd+Q").len(), 1, "Quit の shortcut 表記が見えない");
}
