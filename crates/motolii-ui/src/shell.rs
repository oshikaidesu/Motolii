//! egui依存はこのprivate moduleに閉じ、公開APIへ型を出さない。

use motolii_gpu::UiSharedDeviceParts;

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

#[cfg(test)]
mod tests {
    struct AdapterProbe;

    impl eframe::App for AdapterProbe {
        fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {}
    }

    #[test]
    fn eframe_app_api_churn_stays_inside_adapter() {
        fn assert_app<T: eframe::App>() {}
        assert_app::<AdapterProbe>();
    }
}
