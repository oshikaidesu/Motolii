//! `--intent-log` — **原因**のログを JSONL で外へ流す。
//!
//! 行の形は `motolii_ui::blitz_shell::IntentEvent` の宣言そのもの
//! (`{"seq":n,"intent":{"kind":"…",…}}`)で、egui shell の `--intent-log` と
//! **同じ形**である。同じ列を `ShellGateway::replay` へ食わせれば再現するのも同じ。
//!
//! 対になる**結果**のログは [`StatusLog`](crate::StatusLog)。

use std::path::Path;

use crate::jsonl::JsonlLog;
use crate::shell::Shell;

/// 原因のログの追記先。
pub struct IntentLog {
    log: JsonlLog,
    /// 既に流した行数。`Shell` の journal の続きから流す。
    written: usize,
}

impl IntentLog {
    /// 追記先を作る(既に在れば作り直す)。
    pub fn create(path: &Path) -> Result<Self, String> {
        Ok(Self {
            log: JsonlLog::create(path, "intent log")?,
            written: 0,
        })
    }

    /// journal のうち、まだ流していない分を書く。
    pub fn flush(&mut self, shell: &Shell) {
        for event in shell.intents_since(self.written) {
            if !self.log.append(&event) {
                return;
            }
        }
        self.written = shell.intent_count();
    }
}
