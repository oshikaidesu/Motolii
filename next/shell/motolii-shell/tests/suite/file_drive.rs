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

use motolii_shell::file_dialogs::FileDialogs;
use motolii_shell::{resolve_navigation_key, Message, Shell};

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

// (旧 text_targets/center/click_at ヘルパーは MB-2 の menubar 化で不要に
// なった — 開いた menu へは同一 Simulator の `find`/`click` で届く。)

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

/// menu 側: File を開くと4項目が現れ、クリックすると shortcut と**同じ**
/// `Message` variant が publish される — S6 併存(menu と shortcut が同じ
/// 入口)の直接証拠。`shell.update` は呼ばない(このテストは「何が publish
/// されるか」だけを見る — 実際に適用すると Save As/Quit が fake とはいえ
/// 状態を動かすので、他のテストと責務を分ける)。
///
/// MB-2: menubar 化により開閉状態は widget 内部になった — 旧
/// `ToggleFileMenu` → `update` → view 作り直しではなく、**同一 Simulator
/// 内で** bar click → 項目 click を続ける(`menu_drive.rs` 冒頭 doc の手口)。
/// 項目 click で menu が閉じる(vendored の `close_on_item_click` 既定)ため、
/// 項目ごとに fresh な Simulator を作る。
#[test]
fn clicking_file_menu_items_publishes_the_same_messages_as_their_shortcuts() {
    let table: Vec<(&str, Option<&str>, fn(&Message) -> bool)> = vec![
        ("New Project", Some("Cmd+N"), |m| matches!(m, Message::NewProjectRequested)),
        ("Save As…", Some("Cmd+Shift+S"), |m| matches!(m, Message::SaveAsRequested)),
        // Save a Copy は normal-map の shortcut 出典ゼロ — 併記なし(飾り
        // shortcut を発明しない)。
        ("Save a Copy…", None, |m| matches!(m, Message::SaveACopyRequested)),
        ("Quit", Some("Cmd+Q"), |m| matches!(m, Message::QuitRequested)),
    ];

    for (item, shortcut, is_expected) in table {
        let (shell, _fake) = shell_with_fake();
        let mut ui = iced_test::simulator(shell.view());
        ui.click("File").expect("バー項目 File が見つからない");
        if let Some(shortcut) = shortcut {
            ui.find(shortcut).unwrap_or_else(|e| {
                panic!("File > {item:?} の shortcut 表記 {shortcut:?} が見えない: {e:?}")
            });
        }
        ui.click(item)
            .unwrap_or_else(|e| panic!("File menu の項目 {item:?} へ届かない: {e:?}"));

        let messages: Vec<Message> = ui.into_messages().collect();
        assert_eq!(
            messages.len(),
            1,
            "File > {item:?} click が期待どおり1件の Message を出さない: {messages:?}"
        );
        assert!(
            is_expected(&messages[0]),
            "File > {item:?} click が shortcut と同じ Message を出していない: {messages:?}"
        );
    }
}
