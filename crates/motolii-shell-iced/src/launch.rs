//! 起動要求 — 引数の読み方。
//!
//! **窓の外に置いてある**理由は、これが運転席の入口だからである。`main.rs` の
//! 中に埋めると「引数をどう読むか」を窓を開かずに確かめられない
//! (egui 側の `BlitzShellLaunch` と同じ分担)。
//!
//! M-1 の範囲は記録の2本だけだった。その後2つ増えた: `--project`(M-2 で座席を
//! 起動時に据える)と `--screenshot`(見た目の検収に常設器具が要るため。egui shell
//! `crates/motolii-ui/src/blitz_shell/main.rs` と**同じ引数の形**
//! `--screenshot <out.png> [frames]`、frames 省略時は [`DEFAULT_SCREENSHOT_FRAMES`])。
//! `--fixture` は中身が来てから。
//!
//! `--project` は読むだけがここの仕事で、開く・座らせる判断は
//! [`crate::resume::decide_resume`] が持つ(窓を開く前に呼べるようにするため、
//! 意図的にここへは書かない)。

use std::path::PathBuf;

/// 既定の待ちフレーム数。egui 版 `blitz_shell::DEFAULT_SCREENSHOT_FRAMES` と
/// 同じ値 — 非同期の届き物(サムネイル・波形)が来るのを待つ。
pub const DEFAULT_SCREENSHOT_FRAMES: u32 = 10;

/// `--screenshot` の要求。窓を1枚だけ描いてPNGにし、そのまま終了する。
/// 合体した絵を**人が窓を開かずに確認する**ための口で、製品機能ではない
/// (egui 版 `blitz_shell::ScreenshotRequest` と同じ役目)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenshotRequest {
    pub path: PathBuf,
    /// 待つフレーム数。
    pub frames: u32,
}

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
    /// 撮影の要求(`--screenshot`)。製品機能ではない検証器具。
    pub screenshot: Option<ScreenshotRequest>,
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
            } else if arg == "--screenshot" {
                let path = PathBuf::from(value_after(&args, i, arg)?);
                i += 2;
                let mut frames = DEFAULT_SCREENSHOT_FRAMES;
                // frames は省略可能(次の引数が数値のときだけ食う。egui 版と同じ)。
                if let Some(parsed) = args.get(i).and_then(|value| value.parse().ok()) {
                    frames = parsed;
                    i += 1;
                }
                launch.screenshot = Some(ScreenshotRequest { path, frames });
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
