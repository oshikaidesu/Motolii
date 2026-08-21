//! File 束(MB-1、裁定176 `docs/reviews/2026-08-22-file-dialog-decision.md`)が
//! 使う OS 側の副作用の注入口。
//!
//! **New Project/Save As/Save a Copy/Quit は3種類の副作用しか要らない**
//! ── 破棄確認・保存先 path 選択・プロセス終了。この3つを [`FileDialogs`]
//! trait の3メソッドへそのまま対応させ、`Shell` は `Box<dyn FileDialogs>` を
//! 1本持つだけにした。production(`Shell::new()`)は [`RfdDialogs`](rfd 経由)
//! を渡す ── 裁定176 の「wraps>移植>スクラッチ、自前 file browser は作らない」
//! の直接適用。
//!
//! **test は実際の OS dialog も `std::process::exit` も呼べない**(呼ぶと
//! 描画も持たないヘッドレス試験プロセスにダイアログが出て詰まるか、
//! `quit()` がテストバイナリ自身を道連れにする)── 発注書 ORACLE の
//! 「dialog 呼び出しを注入可能な境界へ」はこの trait そのもの。
//! `tests/suite/file_drive.rs` は `Shell::new_with_dialogs` へ缶詰応答の
//! fake を渡して `Shell::update` だけを叩く(`menu_drive.rs` と同じ
//! 「`Shell` の公開 API だけを叩く」black box 手口)。

use std::path::PathBuf;

/// [`crate::Shell`] が保持する唯一の OS 副作用口。3メソッドが
/// New Project/Save As/Save a Copy/Quit の全てをまかなう(Save As と
/// Save a Copy は `pick_save_path` を共有 ── 違いは呼び手側の
/// `current_path` の扱いだけ、`Shell::perform_save_as`/
/// `Shell::perform_save_a_copy` 参照)。
pub trait FileDialogs: std::fmt::Debug {
    /// 未保存の変更を破棄してよいか確認する(New Project/Quit の dirty ガード、
    /// `Shell::confirm_discard_if_needed` からのみ呼ばれる ── dirty でなければ
    /// この関数自体が呼ばれない)。true = 破棄して続行。
    fn confirm_discard(&self) -> bool;
    /// 保存先 path を選ぶ(Save As/Save a Copy 共通の入口)。`None` = キャンセル
    /// (Escape・ダイアログを閉じる、等)── 呼び手は何もしない。
    fn pick_save_path(&self) -> Option<PathBuf>;
    /// プロセスを終了する。**確認を通過した後にだけ呼ばれる**
    /// (`Shell::update` の `Message::QuitRequested` 腕を参照 ── dirty かつ
    /// `confirm_discard` が false を返した場合はここへ到達しない)。
    fn quit(&self);
}

/// production 実装。**生の `rfd`/`std::process::exit` 呼び出しはこの struct の
/// 中だけに閉じる**(この crate の他の場所からは `rfd::` を直接書かない ──
/// grep 1本で境界を確認できる)。
#[derive(Debug, Default, Clone, Copy)]
pub struct RfdDialogs;

impl FileDialogs for RfdDialogs {
    fn confirm_discard(&self) -> bool {
        // 裁定176: dialog はブロッキングでも可(iced の `update` 内から同期
        // 呼び出し)。実測で問題があれば async 版(`rfd::AsyncMessageDialog`)へ
        // 切り替える判断を残す ── 今回は実装コストを増やさない側を採る。
        rfd::MessageDialog::new()
            .set_title("Motolii")
            .set_description("保存されていない変更があります。破棄しますか?")
            .set_level(rfd::MessageLevel::Warning)
            .set_buttons(rfd::MessageButtons::YesNo)
            .show()
            == rfd::MessageDialogResult::Yes
    }

    fn pick_save_path(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .add_filter("Motolii project", &["motolii"])
            .set_file_name("untitled.motolii")
            .save_file()
    }

    fn quit(&self) {
        std::process::exit(0);
    }
}
