//! eframe App: 事前登録済み viewport texture の投影のみ。

use std::sync::Arc;

use egui::Vec2;

use crate::display_pool::DisplayPool;
use crate::layout_preset;

#[derive(Debug, thiserror::Error)]
pub(crate) enum AppConstructionError {
    #[error("wgpu render state is not available")]
    MissingWgpuRenderState,
}

pub(crate) struct MotoliiApp {
    _pool: Arc<DisplayPool>,
    viewport_texture: egui::TextureId,
    viewport_size: Vec2,
    frames_seen: u32,
    auto_close_after_frames: Option<u32>,
}

impl MotoliiApp {
    pub(crate) fn new(
        cc: &eframe::CreationContext<'_>,
        pool: Arc<DisplayPool>,
        auto_close_after_frames: Option<u32>,
    ) -> Result<Self, AppConstructionError> {
        let render_state = cc
            .wgpu_render_state
            .as_ref()
            .ok_or(AppConstructionError::MissingWgpuRenderState)?;
        let viewport_texture = {
            let mut renderer = render_state.renderer.write();
            pool.register_once(&render_state.device, &mut renderer)
        };
        let desc = pool.desc();
        Ok(Self {
            _pool: pool,
            viewport_texture,
            viewport_size: Vec2::new(desc.width as f32, desc.height as f32),
            frames_seen: 0,
            auto_close_after_frames,
        })
    }
}

impl eframe::App for MotoliiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        layout_preset::paint(ui, self.viewport_texture, self.viewport_size);
        if let Some(limit) = self.auto_close_after_frames {
            self.frames_seen = self.frames_seen.saturating_add(1);
            if self.frames_seen >= limit {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}
