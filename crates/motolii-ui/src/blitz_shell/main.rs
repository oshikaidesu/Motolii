//! 移植済みの Blitz パネルを1つの窓へ合体させて出すだけの実行体。
//!
//! ```text
//! cargo run -p motolii-ui --bin motolii-blitz-shell
//! cargo run -p motolii-ui --bin motolii-blitz-shell -- --project my-project.json
//! cargo run -p motolii-ui --bin motolii-blitz-shell -- --screenshot out/shell.png [frames]
//! cargo run -p motolii-ui --bin motolii-blitz-shell -- --status-log out/shell.jsonl
//! cargo run -p motolii-ui --bin motolii-blitz-shell -- --intent-log out/shell-intents.jsonl
//! ```
//!
//! `--status-log` は窓が言ったこと（status 帯の全文 + 面の失敗）を JSONL
//! (`{"seq":n,"text":"…"}`) で追記する。CLI から窓を検証する実行が
//! **必ず機械可読の失敗記録を持つ**ための口で、製品機能ではない。
//!
//! `--intent-log` はその対で、**何をしようとしたか**（New / Open / Save /
//! Export / Cancel / 素材の取り込み）を同型の JSONL
//! (`{"seq":n,"intent":{"kind":"…",…}}`) で追記する。dialog の答え（path）まで
//! 値として入っているので、出た列はそのまま replay に食わせられる
//! （2026-08-18「ログと構造の強制」）。
//!
//! この bin は**引数を `BlitzShellLaunch` に写すだけ**で、窓・eframe・撮影の中身は
//! `crates/motolii-ui/src/blitz_shell/runner.rs` にある(公開APIを toolkit-free に保つ
//! U0a の境界規律のため)。見た目も配置もここで決めない。
//!
//! `--project` は実プロジェクトを開く。Timeline は native エディタになって
//! その Document を編集でき(移動/トリム/選択/Undo/Redo)、Stage は編集後の
//! snapshot を映す。**開けなければ起動失敗**で、fixture へ黙って落ちない。
//! `--project` 無しは従来どおり fixture 展示(開発動線・screenshot テスト)。

use std::path::PathBuf;

use motolii_ui::blitz_shell::{
    run_blitz_shell, BlitzShellLaunch, ScreenshotRequest, DEFAULT_SCREENSHOT_FRAMES,
};

fn usage() -> ! {
    eprintln!(
        "usage: motolii-blitz-shell [--project <project.json>] [--fixture] \
         [--status-log <out.jsonl>] [--intent-log <out.jsonl>] \
         [--screenshot <out.png> [frames]]"
    );
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut project: Option<PathBuf> = None;
    let mut screenshot: Option<ScreenshotRequest> = None;
    let mut status_log: Option<PathBuf> = None;
    let mut intent_log: Option<PathBuf> = None;
    let mut fixture = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--project" => {
                let Some(path) = args.get(index + 1) else {
                    usage()
                };
                project = Some(PathBuf::from(path));
                index += 2;
            }
            "--fixture" => {
                fixture = true;
                index += 1;
            }
            "--status-log" => {
                let Some(path) = args.get(index + 1) else {
                    usage()
                };
                status_log = Some(PathBuf::from(path));
                index += 2;
            }
            "--intent-log" => {
                let Some(path) = args.get(index + 1) else {
                    usage()
                };
                intent_log = Some(PathBuf::from(path));
                index += 2;
            }
            "--screenshot" => {
                let Some(path) = args.get(index + 1) else {
                    usage()
                };
                let mut frames = DEFAULT_SCREENSHOT_FRAMES;
                index += 2;
                // frames は省略可能(次が数値のときだけ食う)。
                if let Some(parsed) = args.get(index).and_then(|value| value.parse().ok()) {
                    frames = parsed;
                    index += 1;
                }
                screenshot = Some(ScreenshotRequest {
                    path: PathBuf::from(path),
                    frames,
                });
            }
            _ => usage(),
        }
    }

    if let Err(error) = run_blitz_shell(BlitzShellLaunch {
        project,
        screenshot,
        fixture,
        status_log,
        intent_log,
    }) {
        eprintln!("motolii-blitz-shell: {error}");
        std::process::exit(1);
    }
}
