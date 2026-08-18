//! `--status-log` — **結果**のログを JSONL で外へ流す。
//!
//! 行の形は `motolii_ui::blitz_shell::StatusEvent` の宣言そのもの
//! (`{"seq":n,"text":"…"}`)で、egui runner の `--status-log` と**同じ形**である。
//! [`IntentLog`](crate::IntentLog) と並べて読むと「何をしようとして(intent)、
//! 何が起きたか(status)」が揃う。
//!
//! ここが在ることで、CLI から窓を駆動する実行は**必ず失敗の記録を持つ** —
//! `eprintln!` で消える失敗を作らない、という 2026-08-18 の規律の iced 側の受け皿。

use std::path::Path;

use crate::jsonl::JsonlLog;
use crate::shell::Shell;

/// 結果のログの追記先。
pub struct StatusLog {
    log: JsonlLog,
    /// 既に流した行数。`Shell` の transcript の続きから流す。
    written: usize,
}

impl StatusLog {
    /// 追記先を作る(既に在れば作り直す)。
    pub fn create(path: &Path) -> Result<Self, String> {
        Ok(Self {
            log: JsonlLog::create(path, "status log")?,
            written: 0,
        })
    }

    /// transcript のうち、まだ流していない分を書く。
    pub fn flush(&mut self, shell: &Shell) {
        for event in shell.statuses_since(self.written) {
            if !self.log.append(&event) {
                return;
            }
        }
        self.written = shell.status_count();
    }
}
