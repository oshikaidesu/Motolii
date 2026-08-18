//! `--intent-log` 相当 — 原因のログを JSONL で外へ流す。
//!
//! 行の形は `motolii_ui::blitz_shell::IntentEvent` の宣言そのもの
//! (`{"seq":n,"intent":{"kind":"…",…}}`)で、egui shell の `--intent-log` と
//! **同じ形**である。同じ列を `ShellGateway::replay` へ食わせれば再現するのも同じ。
//!
//! 書けなかった時は黙らないが、窓は落とさない(記録先が消えても編集中の
//! project を失わせない)— これも egui 版 `runner.rs` の `JsonlLog` と同じ方針。

use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::shell::Shell;

/// 原因のログの追記先。
pub struct IntentLog {
    file: std::io::BufWriter<std::fs::File>,
    path: PathBuf,
    /// 既に流した行数。`Shell` の journal の続きから流す。
    written: usize,
}

impl IntentLog {
    /// 追記先を作る(既に在れば作り直す)。
    pub fn create(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("intent log {} を作れない: {error}", path.display()))?;
        }
        let file = std::fs::File::create(path)
            .map_err(|error| format!("intent log {} を作れない: {error}", path.display()))?;
        Ok(Self {
            file: std::io::BufWriter::new(file),
            path: path.to_path_buf(),
            written: 0,
        })
    }

    /// journal のうち、まだ流していない分を書く。1行ずつ flush するので、
    /// 窓が固まっても落ちても、そこまでの記録は外に残る。
    pub fn flush(&mut self, shell: &Shell) {
        for event in shell.intents_since(self.written) {
            let line = match serde_json::to_string(&event) {
                Ok(line) => line,
                Err(error) => {
                    println!("motolii-shell-iced: intent log の1行を組めない: {error}");
                    continue;
                }
            };
            if let Err(error) = writeln!(self.file, "{line}").and_then(|()| self.file.flush()) {
                println!(
                    "motolii-shell-iced: intent log {} へ書けない: {error}",
                    self.path.display()
                );
                return;
            }
        }
        self.written = shell.intent_count();
    }
}
