//! 起動要求 — 引数の読み方。
//!
//! **窓の外に置いてある**理由は、これが運転席の入口だからである。`main.rs` の
//! 中に埋めると「引数をどう読むか」を窓を開かずに確かめられない
//! (egui 側の `BlitzShellLaunch` と同じ分担)。
//!
//! M-1 の範囲は記録の2本だけだった。`--screenshot` / `--fixture` は
//! 中身(Stage / Timeline)が来てから。
//!
//! M-2 で `--project` が加わった。読むだけがここの仕事で、開く・座らせる判断は
//! [`crate::resume::decide_resume`] が持つ(窓を開く前に呼べるようにするため、
//! 意図的にここへは書かない)。

use std::path::PathBuf;

/// 起動要求。toolkit 型を含まない。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Launch {
    /// 窓で**起きた原因**の流し先(`--intent-log`)。出た列はそのまま
    /// `ShellGateway::replay` へ食わせられる。
    pub intent_log: Option<PathBuf>,
    /// 窓が**言ったこと**の流し先(`--status-log`)。
    pub status_log: Option<PathBuf>,
    /// 起動時に開く project(`--project`)。指定が無ければ引数なし起動の
    /// 「続きが開く」(`crate::resume::decide_resume`)へ回る。
    pub project: Option<PathBuf>,
}

impl Launch {
    /// 引数を読む。**知らない引数は黙って無視しない**(起動失敗にする) —
    /// 「付けたつもりで効いていなかった」実行を作らない。
    pub fn parse(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let args: Vec<String> = args.collect();
        let mut launch = Self::default();
        let mut i = 0;
        while i < args.len() {
            let arg = args[i].as_str();
            if let Some(rest) = arg.strip_prefix("--intent-log=") {
                launch.intent_log = Some(PathBuf::from(rest));
                i += 1;
            } else if let Some(rest) = arg.strip_prefix("--status-log=") {
                launch.status_log = Some(PathBuf::from(rest));
                i += 1;
            } else if arg == "--intent-log" {
                launch.intent_log = Some(PathBuf::from(value_after(&args, i, arg)?));
                i += 2;
            } else if arg == "--status-log" {
                launch.status_log = Some(PathBuf::from(value_after(&args, i, arg)?));
                i += 2;
            } else if let Some(rest) = arg.strip_prefix("--project=") {
                launch.project = Some(PathBuf::from(rest));
                i += 1;
            } else if arg == "--project" {
                launch.project = Some(PathBuf::from(value_after(&args, i, arg)?));
                i += 2;
            } else {
                return Err(format!("知らない引数: {arg}"));
            }
        }
        Ok(launch)
    }
}

/// `--flag <value>` の value。無ければ起動失敗。
fn value_after(args: &[String], at: usize, flag: &str) -> Result<String, String> {
    args.get(at + 1)
        .cloned()
        .ok_or_else(|| format!("{flag} には path が要る"))
}
