use std::ffi::OsString;
use std::path::Path;

use motolii_ui::{run_shell, run_shell_with_project, ShellError};

const GPU_UNAVAILABLE_EXIT: i32 = 77;
const USAGE_REJECT_EXIT: i32 = 2;

fn main() {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let result = match args.len() {
        0 => run_shell(),
        1 => {
            let arg = &args[0];
            if arg.to_string_lossy().starts_with('-') {
                usage_reject();
            }
            run_shell_with_project(Path::new(arg))
        }
        _ => usage_reject(),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        let code = if matches!(error, ShellError::Gpu(_)) {
            GPU_UNAVAILABLE_EXIT
        } else {
            1
        };
        std::process::exit(code);
    }
}

fn usage_reject() -> ! {
    eprintln!("MOTOLII_USAGE_REJECT invalid argv for motolii_ui_shell");
    std::process::exit(USAGE_REJECT_EXIT);
}
