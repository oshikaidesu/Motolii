use motolii_ui::{run_shell, ShellError};

const GPU_UNAVAILABLE_EXIT: i32 = 77;

fn main() {
    if let Err(error) = run_shell() {
        eprintln!("{error}");
        let code = if matches!(error, ShellError::Gpu(_)) {
            GPU_UNAVAILABLE_EXIT
        } else {
            1
        };
        std::process::exit(code);
    }
}
