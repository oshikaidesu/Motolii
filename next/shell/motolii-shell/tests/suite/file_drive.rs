//! 運転席 — MB-1(File 束、裁定176 `docs/reviews/2026-08-22-file-dialog-decision.md`)
//! + 第2波(2026-08-22、rfd 非同期化+Open/Import Media…/書き出し先選択の追加発注)。
//! New Project/Save As/Save a Copy/Open/Quit の dirty ガード・実書き込み往復・
//! menu+shortcut 両入口・素材取り込みの file dialog 経路を見る。
//!
//! **production の `Shell::new()`(= 生の [`RfdDialogs`](motolii_shell::file_dialogs::RfdDialogs))は
//! 一度もここで呼ばない** — Save As/Save a Copy/Open/Import Media は実際に OS の
//! ネイティブ file dialog を開こうとし(ヘッドレス CI では固まる)、Quit は
//! `std::process::exit(0)` で**テストバイナリ自身を道連れにする**(`suite_main.rs`
//! が `tests/suite/*.rs` 全部を1バイナリへ束ねているため、他ファイルの残りの
//! テストも巻き添えで消える)。この2つの理由がまさに `file_dialogs.rs` の
//! `FileDialogs` trait を注入可能にした理由そのもの — ここでは常に
//! [`Shell::new_with_dialogs`] + [`FakeDialogs`] を使う(`FakeDialogs::quit` は
//! 回数を記録するだけで実際には終了しない)。
//!
//! ## 非同期化(第2波)と `drive`/`drain_task`
//!
//! `FileDialogs` の全メソッドが `rfd::Async*Dialog` に合わせて
//! `Pin<Box<dyn Future<Output=T> + Send>>` を返すようになった
//! (`file_dialogs.rs` 冒頭 doc の「なぜ非同期か」参照)。`Shell::update` は
//! それを `iced::Task::perform` で包んで返すだけなので、`Shell::update` を
//! 呼んだだけでは副作用(確認→リセット、path 選択→保存、等)が完結しない —
//! 返ってきた `Task<Message>` を汲み取って後続の `Message` を
//! もう一度 `Shell::update` へ流し込む必要がある。
//!
//! [`drain_task`] が `iced_runtime::task::into_stream` + 手動 poll
//! (`std::task::Waker::noop()`、Rust 1.85 で安定化)で `Task<Message>` から
//! 出てきた `Message` を同期に取り出す ── **fake の `DialogFuture` は全部
//! `std::future::ready(..)` で作った即完了 future なので、有限回の re-poll
//! (`iced_runtime::task::Task::stream` が挟む1回の `yield_now()` 分)で
//! 必ず `Poll::Ready` になる**(production の `RfdDialogs`(実際に OS へ
//! 問い合わせる)を渡した場合はこの前提が崩れて `drain_task` が上限超過で
//! panic する ── これも「production dialogs をここで使わない」を機械的に
//! 強制する副次効果)。
//! [`drive`] はその汲み取りを再帰的に行い、`Shell::update` を1回呼ぶのと同じ
//! 感覚で「確認→確定」「path 選択→保存/読込」を最後まで駆動する。
//!
//! 同じ理由で、`q0_fence.rs`(全 target を自動クリックする横断柵)へ File
//! ドロップダウンの状態は追加していない — 追加すると `Quit` ボタンが機械的に
//! クリックされ、production dialogs 経由なら test binary を殺しかねない
//! (`menu.rs::Item::message` が `Option` ではなく必須フィールドである時点で
//! 「on_press 無しの死に chrome」は型で排除済みなので、Q0 適合は静的に保証
//! されている — 自動クリック柵を通す必要は無い。加えて q0_fence は
//! `Shell::update` を一切呼ばない ── click で `Message` が publish されるかだけを
//! 見るので、実際に dialog を開く/終了する副作用も起きない)。

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use iced::keyboard::{Key, Modifiers};

use motolii_shell::file_dialogs::{DialogFuture, FileDialogs};
use motolii_shell::{resolve_navigation_key, Message, Shell};

// ---------------------------------------------------------------------------
// `Task<Message>` を同期に汲み取る(fake は即完了 future しか返さない前提)
// ---------------------------------------------------------------------------

/// `iced::Task<Message>` から出てきた `Message` を全部取り出す。`iced::Task`
/// は `iced_runtime::task::Task` の re-export そのもの(フォーク
/// `src/lib.rs::pub mod task { pub use crate::runtime::task::{Handle, Task}; }`
/// 実測)なので、`iced_runtime::task::into_stream` へそのまま渡せる。
///
/// **前提**: 渡す `Task` の中の `DialogFuture` が全部 `std::future::ready(..)`
/// で作った即完了 future であること(`FakeDialogs` の実装がそう)。production
/// の `RfdDialogs`(実際に OS の dialog を待つ)を混ぜると本当に `Pending` の
/// まま進まなくなり、下の上限に当たって panic する ── これは意図的な柵で、
/// production dialogs をこの helper 経由でテストしないことを機械的に強制する。
///
/// **`iced_runtime::task::Task::stream`(`Task::perform`/`Task::future` の
/// 内部実体、`runtime/src/task.rs` 実測)は必ず先頭に1回 `yield_now()` を
/// 挟む**(cooperative yield ── 初回 poll は `cx.waker().wake_by_ref()` を
/// 呼んでから `Poll::Pending` を返し、次の poll で `Poll::Ready(())` になる)。
/// 実行中の iced runtime ならこの wake が re-poll を予約するだけだが、ここは
/// 手動 poll ループなので **`Pending` を「まだ完了していない」ではなく
/// 「もう一度 poll しろ」の合図として扱い、無条件に re-poll する**。fake の
/// future は real I/O 待ちが無い(`std::future::ready` 由来)ので、有限回の
/// re-poll で必ず `Ready` に落ち着く。
pub(crate) fn drain_task(task: iced::Task<Message>) -> Vec<Message> {
    use iced_runtime::Action;
    use std::task::{Context, Poll, Waker};

    let Some(mut stream) = iced_runtime::task::into_stream(task) else {
        return Vec::new();
    };
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut out = Vec::new();
    // 1件あたりの再 poll 上限(`yield_now()` の1回分+安全マージン)。これを
    // 超えたら本当に `Pending` のまま止まっている(production dialogs を
    // 紛れ込ませた等)とみなす。
    const MAX_CONSECUTIVE_PENDING: usize = 1_000;
    let mut consecutive_pending = 0usize;
    loop {
        match stream.as_mut().poll_next(&mut cx) {
            Poll::Ready(Some(Action::Output(message))) => {
                out.push(message);
                consecutive_pending = 0;
            }
            // Widget/Window/…等の非 Output action ── perform ベースの Task
            // からは出てこない想定だが、出ても無視する(drain の目的は
            // publish された Message だけ)。
            Poll::Ready(Some(_other)) => consecutive_pending = 0,
            Poll::Ready(None) => break,
            Poll::Pending => {
                consecutive_pending += 1;
                assert!(
                    consecutive_pending <= MAX_CONSECUTIVE_PENDING,
                    "テストの dialog future が既定回数の re-poll でも完結しなかった \
                     — FakeDialogs は std::future::ready の即完了 future しか \
                     返さないはず(production の RfdDialogs をここで使っていないか \
                     確認する)"
                );
            }
        }
    }
    out
}

/// `Shell::update` を呼び、返ってきた `Task<Message>` を [`drain_task`] で
/// 汲み取って出てきた `Message` を**再帰的に** `Shell::update` へ流し込む
/// (File 束の非同期腕は「確認→確定」「path 選択→保存/読込」のように高々
/// 1〜2段しか無いので、これで production の runtime executor と同じ最終
/// 状態に落ち着く)。
pub(crate) fn drive(shell: &mut Shell, message: Message) {
    let task = shell.update(message);
    for follow_up in drain_task(task) {
        drive(shell, follow_up);
    }
}

// ---------------------------------------------------------------------------
// fake FileDialogs — 缶詰応答(即完了 future)+ 呼び出し回数の記録
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct FakeDialogsState {
    confirm_discard_response: Cell<bool>,
    confirm_discard_calls: Cell<u32>,
    open_path_response: RefCell<Option<PathBuf>>,
    save_path_response: RefCell<Option<PathBuf>>,
    import_paths_response: RefCell<Vec<PathBuf>>,
    // Export 窓の書き出し先選択(`export_drive.rs` が使う ── この module の
    // `FakeDialogs` は sibling module から `crate::file_drive::FakeDialogs` の
    // 形で再利用される、テスト binary が `suite_main.rs` で1本に束ねられている
    // ため)。
    export_path_response: RefCell<Option<PathBuf>>,
    quit_calls: Cell<u32>,
}

/// [`FileDialogs`] の test 実装。`Rc` で包んであるのは、`Shell` へ渡す
/// `Box<dyn FileDialogs>`(所有権が移る)とは別に、test 側が呼び出し回数・
/// 応答を読み書きできる**もう1つの取っ手**を残すため(`Cell`/`RefCell` で
/// 内部可変性を持つので `&self` のままで足りる — trait のメソッドはどれも
/// `&self`)。全メソッドは `Box::pin(std::future::ready(..))` で**即完了**の
/// `DialogFuture` を返す ── [`drain_task`] が1回の poll で汲み取れる前提。
#[derive(Debug, Clone, Default)]
pub(crate) struct FakeDialogs(Rc<FakeDialogsState>);

impl FakeDialogs {
    pub(crate) fn set_confirm_discard(&self, value: bool) {
        self.0.confirm_discard_response.set(value);
    }

    pub(crate) fn confirm_discard_calls(&self) -> u32 {
        self.0.confirm_discard_calls.get()
    }

    pub(crate) fn set_open_path(&self, path: PathBuf) {
        *self.0.open_path_response.borrow_mut() = Some(path);
    }

    pub(crate) fn set_save_path(&self, path: PathBuf) {
        *self.0.save_path_response.borrow_mut() = Some(path);
    }

    pub(crate) fn set_import_paths(&self, paths: Vec<PathBuf>) {
        *self.0.import_paths_response.borrow_mut() = paths;
    }

    pub(crate) fn set_export_path(&self, path: PathBuf) {
        *self.0.export_path_response.borrow_mut() = Some(path);
    }

    fn quit_calls(&self) -> u32 {
        self.0.quit_calls.get()
    }
}

impl FileDialogs for FakeDialogs {
    fn confirm_discard(&self) -> DialogFuture<bool> {
        self.0.confirm_discard_calls.set(self.0.confirm_discard_calls.get() + 1);
        let response = self.0.confirm_discard_response.get();
        Box::pin(std::future::ready(response))
    }

    // C-1 波C: `Shell::boot` の起動時チェックからしか呼ばれず、この drive の
    // 試験は `Shell::new_with_dialogs`(boot を経由しない)しか使わないので、
    // ここは呼ばれない前提の固定応答でよい(呼び出し回数を数える柵は無し)。
    fn confirm_recover_autosave(&self) -> DialogFuture<bool> {
        Box::pin(std::future::ready(false))
    }

    fn pick_open_path(&self) -> DialogFuture<Option<PathBuf>> {
        Box::pin(std::future::ready(self.0.open_path_response.borrow().clone()))
    }

    fn pick_save_path(&self) -> DialogFuture<Option<PathBuf>> {
        Box::pin(std::future::ready(self.0.save_path_response.borrow().clone()))
    }

    fn pick_import_paths(&self) -> DialogFuture<Vec<PathBuf>> {
        Box::pin(std::future::ready(self.0.import_paths_response.borrow().clone()))
    }

    fn pick_export_path(&self, _default_file_name: String) -> DialogFuture<Option<PathBuf>> {
        Box::pin(std::future::ready(self.0.export_path_response.borrow().clone()))
    }

    fn quit(&self) {
        // **実際には終了しない** — production(`RfdDialogs::quit`)との唯一の
        // 違いがここ。他は缶詰応答という点で同格。
        self.0.quit_calls.set(self.0.quit_calls.get() + 1);
    }
}

pub(crate) fn shell_with_fake() -> (Shell, FakeDialogs) {
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

    drive(&mut shell, Message::NewProjectRequested);

    assert_eq!(
        fake.confirm_discard_calls(),
        0,
        "dirty ではないのに確認ダイアログを呼んでいる"
    );
}

#[test]
fn new_project_prompts_when_dirty_and_only_resets_the_document_after_confirmation() {
    let (mut shell, fake) = shell_with_fake();
    drive(&mut shell, Message::AddLayer);
    assert_eq!(shell.layer_count(), 1);
    assert!(shell.is_project_dirty(), "AddLayer 後なのに dirty になっていない");

    // 缶詰応答の既定は false(キャンセル)。
    drive(&mut shell, Message::NewProjectRequested);
    assert_eq!(fake.confirm_discard_calls(), 1, "dirty なのに確認していない");
    assert_eq!(shell.layer_count(), 1, "キャンセルしたのに Document がリセットされている");
    assert!(shell.is_project_dirty(), "キャンセルしたのに dirty が消えている");

    // 確認 → 実際にリセットされる。undo 履歴も消えている(revision 初期化)。
    fake.set_confirm_discard(true);
    drive(&mut shell, Message::NewProjectRequested);
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
    drive(&mut shell, Message::QuitRequested);
    assert_eq!(fake.confirm_discard_calls(), 0, "dirty ではないのに確認している");
    assert_eq!(fake.quit_calls(), 1, "clean な Document で Quit したのに終了していない");
}

#[test]
fn quit_is_cancelled_when_the_user_declines_to_discard_dirty_changes() {
    let (mut shell, fake) = shell_with_fake();
    drive(&mut shell, Message::AddLayer);
    fake.set_confirm_discard(false);

    drive(&mut shell, Message::QuitRequested);

    assert_eq!(fake.confirm_discard_calls(), 1, "dirty なのに確認していない");
    assert_eq!(fake.quit_calls(), 0, "キャンセルしたのに終了している");

    fake.set_confirm_discard(true);
    drive(&mut shell, Message::QuitRequested);
    assert_eq!(fake.quit_calls(), 1, "確認後も終了していない");
}

// ---------------------------------------------------------------------------
// (c) Save As — 一時 dir への実書き込み → 読み戻し一致
// ---------------------------------------------------------------------------

#[test]
fn save_as_writes_a_real_file_that_reads_back_with_identical_content() {
    let (mut shell, fake) = shell_with_fake();
    drive(&mut shell, Message::AddLayer);
    drive(&mut shell, Message::AddLayer);
    assert_eq!(shell.layer_count(), 2);

    let dir = motolii_testkit::tmp_dir("file-drive-save-as");
    let path = dir.join("save-as-roundtrip.motolii");
    fake.set_save_path(path.clone());

    drive(&mut shell, Message::SaveAsRequested);

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
    drive(&mut shell, Message::AddLayer);
    // `_fake.save_path_response` は既定で `None`(キャンセル相当)。

    drive(&mut shell, Message::SaveAsRequested);

    assert_eq!(shell.current_path(), None, "キャンセルしたのに current_path が設定されている");
    assert!(shell.is_project_dirty(), "キャンセルしたのに dirty が消えている");
}

// ---------------------------------------------------------------------------
// (c-2) 平の Save(id 1224、C-1 波C) — current_path 既知なら無言で上書き
// ---------------------------------------------------------------------------

/// 検収条件そのもの(発注書 OUTCOME 1「平の Save が存在しない」)。
/// `current_path` を確定させた後、別 path をダイアログ応答に仕込んでおく ──
/// `SaveRequested` がもし内部で `pick_save_path` を呼んでいたら、保存先が
/// その別 path へずれるはず。ずれない(= 元の path へ上書きされる)ことが
/// 「ダイアログを一切開いていない」ことの直接証拠になる(呼び出し回数を
/// 数える柵ではなく、観測可能な結果で確かめる)。
#[test]
fn save_requested_overwrites_the_known_path_without_opening_a_dialog() {
    let (mut shell, fake) = shell_with_fake();
    drive(&mut shell, Message::AddLayer);

    let dir = motolii_testkit::tmp_dir("file-drive-save-requested");
    let path = dir.join("plain-save.motolii");
    fake.set_save_path(path.clone());
    drive(&mut shell, Message::SaveAsRequested);
    assert_eq!(shell.current_path(), Some(path.as_path()));
    assert!(!shell.is_project_dirty());

    // ダイアログを開いたら誤ってここへ書かれるはずの、別の path。
    let decoy = dir.join("should-not-be-touched.motolii");
    fake.set_save_path(decoy.clone());

    drive(&mut shell, Message::AddLayer);
    assert!(shell.is_project_dirty(), "追加編集の後なのに dirty になっていない");

    drive(&mut shell, Message::SaveRequested);

    assert_eq!(
        shell.current_path(),
        Some(path.as_path()),
        "SaveRequested がダイアログ経由の別 path へ current_path を差し替えている \
         = current_path が既知なのにパスを聞き直している"
    );
    assert!(
        !decoy.exists(),
        "SaveRequested がダイアログの応答先へ書いている(黙って同じ場所へ書くはずが確認している)"
    );
    assert!(!shell.is_project_dirty(), "SaveRequested の後も dirty のまま");

    let loaded = motolii_store::Document::load(&path).expect("SaveRequested が既存 path を上書きしていない");
    assert_eq!(loaded.view().layers().len(), 2, "SaveRequested が最新の編集を書いていない");
}

/// 一度も保存していない新規 project(`current_path` が `None`)は Save As と
/// 同じ path 選択へ合流する(先例: 初回保存はどの製品でもパスを聞くしか
/// ない)── ダイアログを一切出さない上の試験と対にして、両方の分岐を
/// 1本ずつだけ押さえる。
#[test]
fn save_requested_falls_back_to_the_save_dialog_when_no_path_is_known_yet() {
    let (mut shell, fake) = shell_with_fake();
    drive(&mut shell, Message::AddLayer);
    assert_eq!(shell.current_path(), None);

    let dir = motolii_testkit::tmp_dir("file-drive-save-requested-first");
    let path = dir.join("first-save.motolii");
    fake.set_save_path(path.clone());

    drive(&mut shell, Message::SaveRequested);

    assert_eq!(shell.current_path(), Some(path.as_path()), "初回 SaveRequested が path を確定させていない");
    assert!(!shell.is_project_dirty());
    assert!(path.exists(), "初回 SaveRequested が実 file を書いていない");
}

// ---------------------------------------------------------------------------
// (d) Save a Copy — 現 path 維持のまま別名へ書く
// ---------------------------------------------------------------------------

#[test]
fn save_a_copy_writes_a_file_without_touching_the_current_path_or_dirty_state() {
    let (mut shell, fake) = shell_with_fake();
    drive(&mut shell, Message::AddLayer);

    let dir = motolii_testkit::tmp_dir("file-drive-save-a-copy");
    let primary = dir.join("primary.motolii");
    fake.set_save_path(primary.clone());
    drive(&mut shell, Message::SaveAsRequested);
    assert_eq!(shell.current_path(), Some(primary.as_path()));
    assert!(!shell.is_project_dirty());

    // Save As 後にさらに編集して dirty へ戻す。
    drive(&mut shell, Message::AddLayer);
    assert!(shell.is_project_dirty());

    let copy_path = dir.join("a-copy.motolii");
    fake.set_save_path(copy_path.clone());
    drive(&mut shell, Message::SaveACopyRequested);

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
// (e) Open — dirty ガード → path 選択 → Document 丸ごと差し替え
// ---------------------------------------------------------------------------

/// Save As と同じ実書き込み経路で1本 project file を用意する(Open 試験の
/// 入力を作るための下請け ── `Shell::doc` は private field なので、`drive` +
/// `SaveAsRequested` を経由するのが唯一の公開 API)。
fn write_project_with_layers(dir: &std::path::Path, name: &str, layers: usize) -> PathBuf {
    let (mut shell, fake) = shell_with_fake();
    for _ in 0..layers {
        drive(&mut shell, Message::AddLayer);
    }
    let path = dir.join(name);
    fake.set_save_path(path.clone());
    drive(&mut shell, Message::SaveAsRequested);
    path
}

#[test]
fn open_replaces_the_document_when_the_current_one_is_clean() {
    let dir = motolii_testkit::tmp_dir("file-drive-open-clean");
    let path = write_project_with_layers(&dir, "to-open.motolii", 3);

    let (mut shell, fake) = shell_with_fake();
    assert!(!shell.is_project_dirty());
    fake.set_open_path(path.clone());

    drive(&mut shell, Message::OpenRequested);

    assert_eq!(fake.confirm_discard_calls(), 0, "dirty ではないのに確認している");
    assert_eq!(shell.layer_count(), 3, "Open が読み込んだ layer 数と一致しない");
    assert_eq!(shell.current_path(), Some(path.as_path()), "Open 後に current_path が更新されていない");
    assert!(!shell.is_project_dirty(), "Open 直後なのに dirty のまま");
    assert!(!shell.can_undo(), "Open 直後は undo できないはず(flattened 読込の仕様どおり)");
}

#[test]
fn open_prompts_when_dirty_and_only_loads_after_confirmation() {
    let dir = motolii_testkit::tmp_dir("file-drive-open-dirty");
    let path = write_project_with_layers(&dir, "to-open.motolii", 5);

    let (mut shell, fake) = shell_with_fake();
    drive(&mut shell, Message::AddLayer);
    assert_eq!(shell.layer_count(), 1);
    fake.set_open_path(path.clone());

    // 缶詰応答の既定は false(キャンセル)— 確認したのに読み込まれない。
    drive(&mut shell, Message::OpenRequested);
    assert_eq!(fake.confirm_discard_calls(), 1, "dirty なのに確認していない");
    assert_eq!(shell.layer_count(), 1, "キャンセルしたのに Document が差し替わっている");
    assert_eq!(shell.current_path(), None, "キャンセルしたのに current_path が変わっている");

    // 確認 → 実際に読み込まれる。
    fake.set_confirm_discard(true);
    drive(&mut shell, Message::OpenRequested);
    assert_eq!(fake.confirm_discard_calls(), 2);
    assert_eq!(shell.layer_count(), 5, "確認後も Document が差し替わっていない");
    assert_eq!(shell.current_path(), Some(path.as_path()));
}

#[test]
fn open_does_nothing_when_the_path_dialog_is_cancelled_after_confirmation() {
    let (mut shell, fake) = shell_with_fake();
    drive(&mut shell, Message::AddLayer);
    fake.set_confirm_discard(true);
    // `open_path_response` は既定で `None`(キャンセル相当)。

    drive(&mut shell, Message::OpenRequested);

    assert_eq!(fake.confirm_discard_calls(), 1, "確認自体は起きるはず");
    assert_eq!(shell.layer_count(), 1, "path 選択をキャンセルしたのに Document が変わっている");
    assert_eq!(shell.current_path(), None, "path 選択をキャンセルしたのに current_path が設定された");
    assert!(shell.is_project_dirty(), "path 選択をキャンセルしたのに dirty が消えている");
}

// ---------------------------------------------------------------------------
// (f) Import Media… — 複数選択 → 既存の AdmitPaths 経路へ合流
// ---------------------------------------------------------------------------

/// **ORACLE**: File > Import Media… は新しい記帳経路を作らない ──
/// `FileDialogs::pick_import_paths` が返した path をそのまま
/// `Message::AdmitPaths` へ渡すだけ(`browser_drive.rs` の drop 経由の admit
/// と同じ最終状態に落ち着くことがこのテストの主張)。
#[test]
fn import_media_requested_admits_the_picked_paths_just_like_a_drop() {
    let dir = motolii_testkit::tmp_dir("file-drive-import-media");
    let a = dir.join("clip-a.bin");
    let b = dir.join("clip-b.bin");
    std::fs::write(&a, b"a").unwrap();
    std::fs::write(&b, b"b").unwrap();

    let (mut shell, fake) = shell_with_fake();
    fake.set_import_paths(vec![a, b]);

    drive(&mut shell, Message::ImportMediaRequested);

    assert_eq!(shell.assets().len(), 2, "Import Media が2件とも台帳に載せていない");
}

#[test]
fn import_media_requested_is_a_harmless_noop_when_the_dialog_is_cancelled() {
    let (mut shell, _fake) = shell_with_fake();
    // `import_paths_response` は既定で空 `Vec`(キャンセル相当)。

    drive(&mut shell, Message::ImportMediaRequested);

    assert_eq!(shell.assets().len(), 0, "キャンセルしたのに台帳に何か載っている");
    assert_eq!(shell.layer_count(), 0, "キャンセルしたのに layer が置かれている");
}

// ---------------------------------------------------------------------------
// (g) S6 併存 — menu と shortcut が同じ Message へ収束する(構造証明)
// ---------------------------------------------------------------------------

// (旧 text_targets/center/click_at ヘルパーは MB-2 の menubar 化で不要に
// なった — 開いた menu へは同一 Simulator の `find`/`click` で届く。)

/// shortcut 側: `resolve_navigation_key`(`nav_drive.rs`/`shortcut_drive.rs` と
/// 同じ入口)が File 束4動詞を正しい `Message` へ解決する。**`Shell::update` は
/// 一切呼ばない**(crate 冒頭 doc の理由どおり — 特に Quit を呼ぶと production
/// なら test binary が終了する)。Open/Import Media は normal-map の shortcut
/// 出典が無いので割当自体が無い(`menu.rs::menus` doc 参照) — ここでは検分
/// しない。
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

/// menu 側: File を開くと7項目(New Project/Open…/Save As…/Save a Copy…/
/// Export…/Import Media…/Quit)が現れ、クリックすると shortcut と**同じ**
/// `Message` variant が publish される — S6 併存(menu と shortcut が同じ
/// 入口)の直接証拠。この表は Export… を含まない(従来どおり ──
/// `export_drive.rs` が Cmd+E/`ToggleExportDialog` を別途検分する)。
/// `shell.update` は呼ばない(このテストは「何が publish されるか」だけを見る
/// — 実際に適用すると Save As/Quit が fake とはいえ状態を動かすので、他の
/// テストと責務を分ける)。
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
        // id 1226「Open Project」— shortcut 出典ゼロ(menu.rs::menus doc)。
        ("Open…", None, |m| matches!(m, Message::OpenRequested)),
        ("Save As…", Some("Cmd+Shift+S"), |m| matches!(m, Message::SaveAsRequested)),
        // Save a Copy は normal-map の shortcut 出典ゼロ — 併記なし(飾り
        // shortcut を発明しない)。
        ("Save a Copy…", None, |m| matches!(m, Message::SaveACopyRequested)),
        // id 592「Import (media/file)」の第2の入口 — shortcut 出典はあるが
        // 実キーの一次資料未確認なので併記しない(menu.rs::menus doc)。
        ("Import Media…", None, |m| matches!(m, Message::ImportMediaRequested)),
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
