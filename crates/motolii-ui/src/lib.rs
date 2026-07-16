//! motolii-ui: Slint UI シェル層。
//!
//! Slint API は private module 内に閉じ、domain / core クレートの公開契約へは出さない。

mod shell;

/// 製品 UI クレートの識別子。依存方向 CI の許可リストと一致させる。
pub const CRATE_ID: &str = "motolii-ui";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCrateInfo {
    pub crate_id: &'static str,
    pub slint_linked: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error("slint runtime is not linked into {crate_id}")]
    SlintNotLinked { crate_id: &'static str },
}

/// U0a 骨格: workspace 上で Slint 依存が解決できることを返す。
pub fn crate_info() -> Result<UiCrateInfo, UiError> {
    let slint_linked = shell::slint_linked();
    if !slint_linked {
        return Err(UiError::SlintNotLinked { crate_id: CRATE_ID });
    }
    Ok(UiCrateInfo {
        crate_id: CRATE_ID,
        slint_linked,
    })
}
