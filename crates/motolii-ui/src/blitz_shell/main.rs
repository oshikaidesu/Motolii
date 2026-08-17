//! 移植済みの Blitz パネルを1つの窓へ合体させて出すだけの実行体。
//!
//! ```text
//! cargo run -p motolii-ui --bin motolii-blitz-shell
//! cargo run -p motolii-ui --bin motolii-blitz-shell -- --project my-project.json
//! cargo run -p motolii-ui --bin motolii-blitz-shell -- --screenshot out/shell.png [frames]
//! ```
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
        "usage: motolii-blitz-shell [--project <project.json>] [--screenshot <out.png> [frames]]"
    );
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut project: Option<PathBuf> = None;
    let mut screenshot: Option<ScreenshotRequest> = None;
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
    }) {
        eprintln!("motolii-blitz-shell: {error}");
        std::process::exit(1);
    }
}
