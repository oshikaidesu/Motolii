use std::ffi::c_void;

use crate::{
    renderer_core::{PointerPhase, RenderStats, RendererCore, SceneKind},
    rerun_stage::StageTransformProjection,
};
use motolii_ui::AppStageTransformEdit;

pub(crate) struct MacOsSurfaceRenderer {
    core: RendererCore,
    /// drawable 座標を host の logical へ戻すための画素サイズ。
    width: u32,
    height: u32,
}

impl MacOsSurfaceRenderer {
    pub(crate) fn new_stage(layer: *mut c_void, width: u32, height: u32) -> Result<Self, String> {
        Self::new(layer, width, height, SceneKind::Stage)
    }

    pub(crate) fn new_timeline(
        layer: *mut c_void,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        Self::new(layer, width, height, SceneKind::Timeline)
    }

    fn new(layer: *mut c_void, width: u32, height: u32, scene: SceneKind) -> Result<Self, String> {
        if layer.is_null() {
            return Err("CAMetalLayer pointer is null".into());
        }

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer))
        }
        .map_err(|error| format!("create CoreAnimation surface: {error}"))?;
        // 製品macos mountからhost snapshotを引く。適用はcoreのwarmup/renderが所有。
        let _ = crate::host_bridge::try_read_timeline_projection();
        let core = RendererCore::new(instance, surface, width, height, scene)?;
        Ok(Self {
            core,
            width,
            height,
        })
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.core.resize(width, height);
    }

    pub(crate) fn render(&mut self) -> Result<(), String> {
        self.core.render()
    }

    pub(crate) fn set_timeline_state(&mut self, selected_object_index: i32, playhead: f64) {
        self.core
            .set_timeline_state(selected_object_index, playhead);
    }

    pub(crate) fn set_created_item(&mut self, item_id: &str) -> bool {
        self.core.set_created_item(item_id)
    }

    pub(crate) fn fit_stage_view(&mut self) -> bool {
        self.core.fit_stage_view()
    }

    pub(crate) fn set_stage_one_to_one(&mut self) -> bool {
        self.core.set_stage_one_to_one()
    }

    pub(crate) fn stage_transform_projection(&self) -> Option<StageTransformProjection> {
        self.core.stage_transform_projection()
    }

    pub(crate) fn set_stage_transform_projection(
        &mut self,
        projection: StageTransformProjection,
    ) -> bool {
        self.core.set_stage_transform_projection(projection)
    }

    pub(crate) fn preview_stage_transform_from_app(
        &mut self,
        expected_revision: u64,
        layer_id: &str,
        edit: AppStageTransformEdit,
    ) -> Result<(), String> {
        self.core
            .preview_stage_transform_from_app(expected_revision, layer_id, edit)
    }

    pub(crate) fn commit_stage_transform_from_app(
        &mut self,
        expected_revision: u64,
        layer_id: &str,
        edit: AppStageTransformEdit,
    ) -> Result<(), String> {
        self.core
            .commit_stage_transform_from_app(expected_revision, layer_id, edit)
    }

    pub(crate) fn cancel_stage_transform_from_app(&mut self) -> Result<(), String> {
        self.core.cancel_stage_transform_from_app()
    }

    pub(crate) fn timeline_hit_test(&self, x: f64, y: f64) -> Option<(i32, f64)> {
        self.core.timeline_hit_test(x, y)
    }

    pub(crate) fn timeline_hover_cursor(&self, x: f64, y: f64) -> i32 {
        self.core.timeline_hover_cursor(x, y)
    }

    pub(crate) fn stage_hover_cursor(&self, x: f64, y: f64) -> i32 {
        self.core.stage_hover_cursor(x, y)
    }

    pub(crate) fn timeline_pointer(
        &mut self,
        phase: PointerPhase,
        x: f64,
        y: f64,
        modifiers: u32,
    ) -> Option<(i32, f64)> {
        self.core.timeline_pointer(phase, x, y, modifiers)
    }

    pub(crate) fn timeline_scroll(
        &mut self,
        delta_x: f64,
        delta_y: f64,
        magnification: f64,
        modifiers: u32,
        x: f64,
        y: f64,
    ) -> bool {
        self.core
            .timeline_scroll(delta_x, delta_y, magnification, modifiers, x, y)
    }

    pub(crate) fn timeline_keymap_delete(&self) -> bool {
        self.core.timeline_keymap_delete()
    }

    /// 実機keyDownの受け口。解決表は作らず既存host_key_eventへ渡す。
    pub(crate) fn key_down(
        &self,
        key_code: u16,
        modifier_bits: u32,
        chars_utf8: *const u8,
        chars_len: usize,
        is_repeat: bool,
        timeline_focused: bool,
    ) -> i32 {
        let result = unsafe {
            crate::host_bridge::motolii_rnapp_host_key_event(
                key_code,
                modifier_bits,
                chars_utf8,
                chars_len,
                is_repeat,
                timeline_focused,
            )
        };
        if result == 2 && timeline_focused {
            let _ = self.core.timeline_keymap_delete();
        }
        result
    }

    pub(crate) fn stage_pointer(
        &mut self,
        phase: PointerPhase,
        button: crate::renderer_core::StagePointerButton,
        modifiers: u32,
        x: f64,
        y: f64,
    ) {
        self.core.stage_pointer(phase, button, modifiers, x, y);
    }

    pub(crate) fn stage_scroll(
        &mut self,
        delta_x: f64,
        delta_y: f64,
        magnification: f64,
        modifiers: u32,
        x: f64,
        y: f64,
    ) -> bool {
        self.core
            .stage_scroll(delta_x, delta_y, magnification, modifiers, x, y)
    }

    pub(crate) fn stats(&self) -> RenderStats {
        self.core.stats()
    }
}

/// `motolii_rnapp_host_key_event` は再定義しない。renderer 経由の受け口。
///
/// # Safety
/// `handle` は本ライブラリが返した live renderer。`chars_utf8` は `chars_len == 0` なら null 可。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn motolii_macos_renderer_key_down(
    handle: *mut c_void,
    key_code: u16,
    modifier_bits: u32,
    chars_utf8: *const u8,
    chars_len: usize,
    is_repeat: bool,
    timeline_focused: bool,
) -> i32 {
    if handle.is_null() {
        return 0;
    }
    unsafe {
        (*handle.cast::<MacOsSurfaceRenderer>()).key_down(
            key_code,
            modifier_bits,
            chars_utf8,
            chars_len,
            is_repeat,
            timeline_focused,
        )
    }
}
