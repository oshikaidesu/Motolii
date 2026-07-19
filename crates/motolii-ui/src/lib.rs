//! motolii-ui: egui UI adapter層。
//!
//! toolkit APIはprivate module内に閉じ、domain/coreの公開契約へは出さない。

mod app;
mod display_pool;
mod layout_preset;
mod shell;
mod static_frame;

pub use shell::{run_shell, ShellError};

/// 製品 UI クレートの識別子。依存方向 CI の許可リストと一致させる。
pub const CRATE_ID: &str = "motolii-ui";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCrateInfo {
    pub crate_id: &'static str,
    pub toolkit_linked: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error("egui runtime is not linked into {crate_id}")]
    ToolkitNotLinked { crate_id: &'static str },
}

/// U0a骨格: workspace上でegui依存が解決できることを返す。
pub fn crate_info() -> Result<UiCrateInfo, UiError> {
    let toolkit_linked = shell::toolkit_linked();
    if !toolkit_linked {
        return Err(UiError::ToolkitNotLinked { crate_id: CRATE_ID });
    }
    Ok(UiCrateInfo {
        crate_id: CRATE_ID,
        toolkit_linked,
    })
}
