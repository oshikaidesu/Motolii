//! File 束(MB-1、裁定176 `docs/reviews/2026-08-22-file-dialog-decision.md`)が
//! 使う OS 側の副作用の注入口。
//!
//! **非同期(2026-08-22 再発注)**: 最初の実装は `rfd::FileDialog`/
//! `rfd::MessageDialog` の同期 API を `Shell::update` から直に呼んでいた
//! (裁定176 の doc コメント「実測で問題があれば async 版へ切り替える」)。
//! ネイティブ dialog はモーダルで、開いている間 OS のイベントループへ制御を
//! 返さない — iced はシングルスレッドの GUI イベントループなので、同期呼び出しは
//! **窓の再描画・他 pane の入力を dialog を閉じるまで丸ごと止める**(実測は
//! していないが `rfd` 自身の doc が「native dialog は基本モーダル」と明記して
//! おり、他の native GUI フレームワークでも同じ制約が知られている)。今回は
//! `rfd::Async*Dialog`(`impl Future` を返す)+ `iced::Task::perform` の組へ
//! 差し替える ── production の [`RfdDialogs`] だけがこの crate の外から見える
//! `rfd::` 呼び出しを持ち(grep 1本で境界を確認できる)、trait 越しに
//! `Pin<Box<dyn Future<...> + Send>>` を返す。
//!
//! ## rfd 0.15.4 の同期/非同期 API(実ソース確認、`~/.asdf/…/rfd-0.15.4/src/`)
//! - `rfd::FileDialog`/`rfd::MessageDialog`: 同期。`.pick_file()`/`.save_file()`/
//!   `.show()` が呼び出しスレッドをブロックしたまま `Option<PathBuf>`/
//!   `MessageDialogResult` を直接返す。
//! - `rfd::AsyncFileDialog`/`rfd::AsyncMessageDialog`: 非同期。builder は同期版と
//!   同じ chain(`add_filter`/`set_file_name`/…)だが、終端 API
//!   (`pick_file`/`pick_files`/`save_file`/`show`)が `impl Future<Output = …>`
//!   を返す。**non-wasm32 では戻り値の実体は
//!   `Pin<Box<dyn Future<Output = T> + Send>>`**
//!   (`rfd-0.15.4/src/backend.rs::DialogFutureType`)── つまり `Send` を満たすので
//!   `iced::Task::perform`(`impl Future<Output=A> + MaybeSend + 'static`)へ
//!   そのまま渡せる。`pick_file`/`pick_files`/`save_file` は `Option<FileHandle>`/
//!   `Option<Vec<FileHandle>>` を運び、`FileHandle::path(&self) -> &Path` で
//!   `PathBuf` へ落とす。
//!
//! ## iced::Task との組み方
//! `Shell::update` は各腕で `self.dialogs.pick_open_path()` 等が返す
//! [`DialogFuture`] を `Task::perform(future, Message::Xxx)` に包んで返すだけ
//! (副作用の実行自体は runtime executor 側)。**dirty 確認→path 選択のように
//! 複数の非同期段を直列化する場合**(Open、`Shell::confirm_then_pick_open`
//! 参照)は、2つの `DialogFuture` を1つの `async move { … }` へ包んで
//! 単一の `Task::perform` にする ── `Message` 往復を1段挟まずに済み、
//! 「確認をキャンセルしたら path dialog を出さない」を async/await の
//! 早期 return だけで表現できる。
//!
//! `quit()` だけは非同期化しない ── `std::process::exit` は戻ってこない
//! 呼び出しで、`Future` に包む意味が無い(呼び出し即終了)。
//!
//! **test は実際の OS dialog も `std::process::exit` も呼べない**(呼ぶと
//! 描画も持たないヘッドレス試験プロセスにダイアログが出て詰まるか、
//! `quit()` がテストバイナリ自身を道連れにする)── 発注書 ORACLE の
//! 「dialog 呼び出しを注入可能な境界へ」はこの trait そのもの。
//! `tests/suite/file_drive.rs` は `Shell::new_with_dialogs` へ缶詰応答の
//! fake(`std::future::ready` で即完了する `DialogFuture`)を渡し、返ってきた
//! `Task<Message>` を `drain_task`(`iced_runtime::task::into_stream` + 手動
//! poll、`file_drive.rs` 冒頭 doc 参照)で同期に汲み取って `Shell::update` へ
//! 戻す(`menu_drive.rs` と同じ「`Shell` の公開 API だけを叩く」black box 手口)。

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

/// 注入可能な OS dialog の戻り値型。non-wasm32 の
/// `rfd::backend::DialogFutureType<T>` と同じ形(`Send + 'static`)── production
/// (`RfdDialogs`)はそのまま `rfd::Async*Dialog` の戻り値を包むだけ、test
/// (fake)は `std::future::ready(..)` を包む。どちらも `iced::Task::perform`
/// (`impl Future<Output=A> + MaybeSend + 'static`)へそのまま渡せる。
pub type DialogFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// [`crate::Shell`] が保持する唯一の OS 副作用口。5メソッドが New Project/
/// Save As/Save a Copy/Open/Quit/素材取り込み/書き出し先選択の全てをまかなう
/// (Save As と Save a Copy は `pick_save_path` を共有 ── 違いは呼び手側の
/// `current_path` の扱いだけ、`Shell::perform_save_as`/
/// `Shell::perform_save_a_copy` 参照)。
pub trait FileDialogs: std::fmt::Debug {
    /// 未保存の変更を破棄してよいか確認する(New Project/Quit/Open の dirty
    /// ガード、`Shell::confirm_then`/`Shell::confirm_then_pick_open` からのみ
    /// 呼ばれる ── dirty でなければこの関数自体が呼ばれない)。true = 破棄して
    /// 続行。
    fn confirm_discard(&self) -> DialogFuture<bool>;
    /// クラッシュ復帰の確認(C-1 波C「autosave が書くだけで読み返されない」、
    /// `Shell::boot` の起動時チェックからのみ呼ばれる)。true = autosave 世代を
    /// 読み込んで復元(黙って上書きしない ── 復元後は未保存●が点く)、
    /// false = 何もしない(本体ファイルの内容のまま)。
    fn confirm_recover_autosave(&self) -> DialogFuture<bool>;
    /// 開く project の path を選ぶ(Open、id 1226)。`None` = キャンセル。
    fn pick_open_path(&self) -> DialogFuture<Option<PathBuf>>;
    /// 保存先 path を選ぶ(Save As/Save a Copy 共通の入口)。`None` = キャンセル
    /// (Escape・ダイアログを閉じる、等)── 呼び手は何もしない。
    fn pick_save_path(&self) -> DialogFuture<Option<PathBuf>>;
    /// 取り込む素材の path を選ぶ(複数選択可、File > Import Media…・
    /// normal-map id 592「Import (media/file)」の第2の入口 ── 従来は OS drop
    /// のみだった)。キャンセル/0件選択は空 `Vec` ──
    /// `Message::AdmitPaths` は空 `Vec` を渡されても無害(`Shell::admit` の
    /// ループが単に0回回るだけ)。
    fn pick_import_paths(&self) -> DialogFuture<Vec<PathBuf>>;
    /// 書き出し先の path を選ぶ(Export 窓、`export_pane::Message::
    /// PickOutputPath`)。`default_file_name` は dialog の初期ファイル名
    /// (拡張子込み)。`None` = キャンセル。
    fn pick_export_path(&self, default_file_name: String) -> DialogFuture<Option<PathBuf>>;
    /// プロセスを終了する。**確認を通過した後にだけ呼ばれる**
    /// (`Shell::update` の `Message::QuitConfirmed(true)` 腕を参照)。
    fn quit(&self);
}

/// production 実装。**生の `rfd`/`std::process::exit` 呼び出しはこの struct の
/// 中だけに閉じる**(この crate の他の場所からは `rfd::` を直接書かない ──
/// grep 1本で境界を確認できる)。
#[derive(Debug, Default, Clone, Copy)]
pub struct RfdDialogs;

/// プロジェクトファイルの filter(Open/Save As/Save a Copy 共通)。
const PROJECT_FILTER_NAME: &str = "Motolii project";
const PROJECT_EXTENSIONS: &[&str] = &["motolii"];

impl FileDialogs for RfdDialogs {
    fn confirm_discard(&self) -> DialogFuture<bool> {
        Box::pin(async {
            rfd::AsyncMessageDialog::new()
                .set_title("Motolii")
                .set_description("保存されていない変更があります。破棄しますか?")
                .set_level(rfd::MessageLevel::Warning)
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
                .await
                == rfd::MessageDialogResult::Yes
        })
    }

    fn confirm_recover_autosave(&self) -> DialogFuture<bool> {
        Box::pin(async {
            rfd::AsyncMessageDialog::new()
                .set_title("Motolii")
                .set_description(
                    "前回の保存より新しい自動保存が見つかりました。復元しますか?",
                )
                .set_level(rfd::MessageLevel::Info)
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
                .await
                == rfd::MessageDialogResult::Yes
        })
    }

    fn pick_open_path(&self) -> DialogFuture<Option<PathBuf>> {
        Box::pin(async {
            rfd::AsyncFileDialog::new()
                .add_filter(PROJECT_FILTER_NAME, PROJECT_EXTENSIONS)
                .pick_file()
                .await
                .map(|handle| handle.path().to_path_buf())
        })
    }

    fn pick_save_path(&self) -> DialogFuture<Option<PathBuf>> {
        Box::pin(async {
            rfd::AsyncFileDialog::new()
                .add_filter(PROJECT_FILTER_NAME, PROJECT_EXTENSIONS)
                .set_file_name("untitled.motolii")
                .save_file()
                .await
                .map(|handle| handle.path().to_path_buf())
        })
    }

    fn pick_import_paths(&self) -> DialogFuture<Vec<PathBuf>> {
        Box::pin(async {
            // 拡張子は `Shell::guess_asset_type` が既に受理している3種と
            // 完全一致させる(新しい対応形式を発明しない ── フィルタが
            // 実能力より広いと「選べるのに admit で弾かれる」体験になる)。
            rfd::AsyncFileDialog::new()
                .add_filter("Video", &["mp4", "mov", "mkv", "webm", "avi", "m4v"])
                .add_filter("Image", &["jpg", "jpeg", "png", "gif", "webp", "bmp", "svg"])
                .add_filter("Audio", &["wav", "mp3", "aac", "flac", "ogg", "m4a"])
                .pick_files()
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|handle| handle.path().to_path_buf())
                .collect()
        })
    }

    fn pick_export_path(&self, default_file_name: String) -> DialogFuture<Option<PathBuf>> {
        Box::pin(async move {
            // `motolii_export_pane::CONTAINER_CODEC_LABEL`(MP4/H.264、実対応が
            // 1種のみ)と同じ拡張子 ── 選べる形式より広いフィルタを見せない。
            rfd::AsyncFileDialog::new()
                .add_filter("MP4", &["mp4"])
                .set_file_name(default_file_name)
                .save_file()
                .await
                .map(|handle| handle.path().to_path_buf())
        })
    }

    fn quit(&self) {
        std::process::exit(0);
    }
}
