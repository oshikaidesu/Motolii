//! 人に訊く口。**native dialog はこの trait の後ろだけ**に居る。
//!
//! egui shell の `blitz_shell::drive::ShellPrompts` と同じ考え方だが、あちらは
//! `pub(crate)` で外へ出ていない(公開すると kittest 由来の型が付いてくる module
//! ごと出てしまう)。ここでは M-0 が要る2本だけを、この crate の口として持つ。
//! Export と未保存確認は M-1 で足す。
//!
//! **なぜ intent の外に居るのか。** 決まった答え(path)だけが `UiIntent` の中へ
//! 入るので、journal を replay しても dialog は二度と開かない。この規律は
//! `motolii_ui::blitz_shell` から持ち越したもので、host を替えても変わらない。

use std::path::PathBuf;

/// New / Open の訊き手。
pub trait ShellPrompts {
    /// New の保存先。`None` は「やめた」= 何も起きない。
    fn new_project_path(&mut self) -> Option<PathBuf>;
    /// Open するファイル。`None` は「やめた」= 何も起きない。
    fn open_project_path(&mut self) -> Option<PathBuf>;
}

/// 窓の訊き手。egui shell の `NativePrompts` と**同じ文言・同じ既定値**で、
/// 窓が替わっても利用者から見た dialog は変わらない。
#[derive(Debug, Default, Clone, Copy)]
pub struct NativePrompts;

impl ShellPrompts for NativePrompts {
    fn new_project_path(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("New Motolii project")
            .add_filter("Motolii project", &["json"])
            .set_file_name("untitled.json")
            .save_file()
    }

    fn open_project_path(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Open Motolii project")
            .add_filter("Motolii project", &["json"])
            .pick_file()
    }
}

/// 台本の訊き手。**native dialog を一切開かない。**
///
/// `Default` は全部「答えない」なので、テストは触った口だけを置けばよい。
#[derive(Debug, Clone, Default)]
pub struct ScriptedPrompts {
    pub new_project_path: Option<PathBuf>,
    pub open_project_path: Option<PathBuf>,
}

impl ShellPrompts for ScriptedPrompts {
    fn new_project_path(&mut self) -> Option<PathBuf> {
        self.new_project_path.clone()
    }

    fn open_project_path(&mut self) -> Option<PathBuf> {
        self.open_project_path.clone()
    }
}
