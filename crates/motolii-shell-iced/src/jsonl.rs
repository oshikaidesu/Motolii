//! 窓の記録の追記先。**同じ形の JSONL** で、中身(1行に何を書くか)は呼び手が決める。
//!
//! egui runner(`blitz_shell/runner.rs` の `JsonlLog`)と同じ形・同じ方針である:
//! 書けなかった時は黙らないが、**窓は落とさない**(記録先が消えても編集中の
//! project を失わせない)。

use std::io::Write as _;
use std::path::{Path, PathBuf};

/// 1本の JSONL 追記先。
pub(crate) struct JsonlLog {
    file: std::io::BufWriter<std::fs::File>,
    path: PathBuf,
    /// 失敗を言うときの名乗り("intent log" / "status log")。
    what: &'static str,
}

impl JsonlLog {
    /// 追記先を作る(既に在れば作り直す)。親ディレクトリが無ければ作る。
    pub(crate) fn create(path: &Path, what: &'static str) -> Result<Self, String> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("{what} {} を作れない: {error}", path.display()))?;
        }
        let file = std::fs::File::create(path)
            .map_err(|error| format!("{what} {} を作れない: {error}", path.display()))?;
        Ok(Self {
            file: std::io::BufWriter::new(file),
            path: path.to_path_buf(),
            what,
        })
    }

    /// 1行 = 1件。台帳の event 型をそのまま流すので、行の形(field の順)は
    /// 型の宣言そのものである。1行ずつ flush するので、窓が固まっても落ちても、
    /// そこまでの記録は外に残る。
    ///
    /// 書けなくなったら `false` を返す(呼び手はそこで止める)。
    pub(crate) fn append(&mut self, event: &impl serde::Serialize) -> bool {
        let what = self.what;
        let line = match serde_json::to_string(event) {
            Ok(line) => line,
            Err(error) => {
                println!("motolii-shell-iced: {what} の1行を組めない: {error}");
                return true;
            }
        };
        if let Err(error) = writeln!(self.file, "{line}").and_then(|()| self.file.flush()) {
            println!(
                "motolii-shell-iced: {what} {} へ書けない: {error}",
                self.path.display()
            );
            return false;
        }
        true
    }
}
