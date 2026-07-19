//! motolii-ui: egui UI adapter層。
//!
//! toolkit APIはprivate module内に閉じ、domain/coreの公開契約へは出さない。

mod shell;

#[cfg(test)]
mod tokens;

#[cfg(test)]
#[path = "../tokens/generated/u0e1_mechanism_adapter.rs"]
pub(crate) mod u0e1_mechanism_adapter;

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

#[cfg(test)]
mod u0e1_mechanism_adapter_tests {
    use egui::Color32;

    use super::u0e1_mechanism_adapter::U0e1MechanismAdapter;

    #[test]
    fn u0e1_mechanism_adapter_maps_to_egui_values() {
        let color = U0e1MechanismAdapter::mechanism_sample_color();
        assert_eq!(color, Color32::from_rgba_unmultiplied(51, 102, 153, 255));

        let alpha = U0e1MechanismAdapter::mechanism_sample_alpha();
        assert_eq!(alpha, Color32::from_rgba_unmultiplied(255, 128, 0, 128));

        assert!((U0e1MechanismAdapter::spacing_unit_a() - 8.0).abs() < f32::EPSILON);
        assert!((U0e1MechanismAdapter::spacing_unit_b() - 12.5).abs() < f32::EPSILON);
    }
}
