//! U1a-1: 静止 viewport の公開境界（private 実装詳細は crate 内ユニットテストへ）。

use motolii_ui::{run_shell, ShellError};

#[test]
fn public_shell_api_is_minimal() {
    fn assert_shell_entrypoint() -> Result<(), ShellError> {
        run_shell()
    }
    let _ = assert_shell_entrypoint as fn() -> Result<(), ShellError>;
}
