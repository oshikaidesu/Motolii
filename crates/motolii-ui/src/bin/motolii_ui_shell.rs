use std::env;

use motolii_ui::{run_shell, ShellError};

/// GPU 不可用時の専用 exit code（本バイナリ内でのみ定義）。
const GPU_UNAVAILABLE_EXIT: i32 = 77;

fn main() {
    if let Err(err) = run_from_args() {
        eprintln!("{err}");
        let code = if matches!(err, ShellError::Gpu(_)) {
            GPU_UNAVAILABLE_EXIT
        } else {
            1
        };
        std::process::exit(code);
    }
}

fn run_from_args() -> Result<(), ShellError> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--auto-close-after-frames" => {
                let value = args.next().unwrap_or_else(|| {
                    eprintln!("missing value for --auto-close-after-frames");
                    std::process::exit(2);
                });
                let frames: u32 = value.parse().unwrap_or_else(|_| {
                    eprintln!("invalid frame count: {value}");
                    std::process::exit(2);
                });
                // テスト専用 seam: 公開 API を増やさずバイナリ経由で auto-close を渡す。
                env::set_var(
                    "MOTOLII_UI_SHELL_AUTO_CLOSE_AFTER_FRAMES",
                    frames.to_string(),
                );
            }
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }
    run_shell()
}
