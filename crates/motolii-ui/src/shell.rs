//! egui shell 起動と既存 wgpu device 共有。

use std::sync::Arc;

use motolii_gpu::{GpuCtx, UiSharedDeviceParts};

use crate::app::MotoliiApp;
use crate::static_frame;

pub(crate) const AUTO_CLOSE_ENV: &str = "MOTOLII_UI_SHELL_AUTO_CLOSE_AFTER_FRAMES";

pub(crate) fn existing_wgpu_setup(parts: UiSharedDeviceParts) -> egui_wgpu::WgpuSetup {
    egui_wgpu::WgpuSetup::Existing(egui_wgpu::WgpuSetupExisting {
        instance: parts.instance,
        adapter: parts.adapter,
        device: parts.device,
        queue: parts.queue,
    })
}

/// 骨格段階ではwindowを立てず、リンク解決だけを確認する。
pub(crate) fn toolkit_linked() -> bool {
    let _adapter = existing_wgpu_setup as fn(UiSharedDeviceParts) -> egui_wgpu::WgpuSetup;
    std::mem::size_of::<egui::Context>() > 0
}

#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    #[error("GPU unavailable: {0}")]
    Gpu(#[from] motolii_gpu::GpuError),
    #[error("static viewport setup failed")]
    Setup(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("app construction failed")]
    App(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("eframe runtime failed")]
    Runtime(#[source] Box<dyn std::error::Error + Send + Sync>),
}

pub fn run_shell() -> Result<(), ShellError> {
    let auto_close = std::env::var(AUTO_CLOSE_ENV)
        .ok()
        .and_then(|value| value.parse().ok());
    run_shell_inner(auto_close)
}

fn run_shell_inner(auto_close_after_frames: Option<u32>) -> Result<(), ShellError> {
    let (gpu, parts) = GpuCtx::new_for_ui()?;
    let pool = Arc::new(
        static_frame::spawn_join_display_pool(gpu)
            .map_err(|err| ShellError::Setup(Box::new(err)))?,
    );

    let mut native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Motolii")
            .with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    native_options.wgpu_options.wgpu_setup = existing_wgpu_setup(parts);

    eframe::run_native(
        "Motolii",
        native_options,
        Box::new(move |cc| {
            MotoliiApp::new(cc, Arc::clone(&pool), auto_close_after_frames)
                .map(|app| Box::new(app) as Box<dyn eframe::App>)
                .map_err(|err| -> Box<dyn std::error::Error + Send + Sync> { Box::new(err) })
        }),
    )
    .map_err(|err| match err {
        eframe::Error::AppCreation(source) => ShellError::App(source),
        other => ShellError::Runtime(Box::new(other)),
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    #[error("probe")]
    struct ProbeError;

    struct AdapterProbe;

    impl eframe::App for AdapterProbe {
        fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {}
    }

    #[test]
    fn eframe_app_api_churn_stays_inside_adapter() {
        fn assert_app<T: eframe::App>() {}
        assert_app::<AdapterProbe>();
    }

    #[test]
    fn setup_error_preserves_opaque_source_chain() {
        let inner = static_frame::StaticFrameError::SetupThreadPanic;
        let err = ShellError::Setup(Box::new(inner));
        let ShellError::Setup(source) = &err else {
            panic!("expected ShellError::Setup, got {err:?}");
        };
        assert_eq!(source.to_string(), "setup thread panicked");
    }

    #[test]
    fn runtime_error_preserves_opaque_source_chain() {
        let inner = ProbeError;
        let err = ShellError::Runtime(Box::new(inner));
        let ShellError::Runtime(source) = &err else {
            panic!("expected ShellError::Runtime, got {err:?}");
        };
        assert_eq!(source.to_string(), "probe");
    }

    #[test]
    fn shell_error_type_name_is_toolkit_free() {
        fn assert_no_toolkit_in_type_name<T>() {
            let name = std::any::type_name::<T>();
            const TOOLKIT_PATHS: &[&str] = &[
                "egui::",
                "eframe::",
                "egui_wgpu::",
                "egui_winit::",
                "egui_tiles::",
                "egui_taffy::",
                "taffy::",
                "winit::",
            ];
            assert!(
                !TOOLKIT_PATHS.iter().any(|path| name.contains(path)),
                "exported type leaks UI toolkit in type_name: {name}"
            );
        }
        assert_no_toolkit_in_type_name::<ShellError>();
    }
}
