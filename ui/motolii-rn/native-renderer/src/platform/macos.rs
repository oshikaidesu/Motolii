use std::ffi::c_void;

use crate::{
    renderer_core::{PointerPhase, RenderStats, RendererCore, SceneKind},
    rerun_stage::StageTransformProjection,
};

pub(crate) struct MacOsSurfaceRenderer {
    core: RendererCore,
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
        let core = RendererCore::new(instance, surface, width, height, scene)?;
        Ok(Self { core })
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
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

    pub(crate) fn stage_transform_projection(&self) -> Option<StageTransformProjection> {
        self.core.stage_transform_projection()
    }

    pub(crate) fn set_stage_transform_projection(&mut self, projection: StageTransformProjection) -> bool {
        self.core.set_stage_transform_projection(projection)
    }

    pub(crate) fn timeline_hit_test(&self, x: f64, y: f64) -> Option<(i32, f64)> {
        self.core.timeline_hit_test(x, y)
    }

    pub(crate) fn timeline_pointer(
        &mut self,
        phase: PointerPhase,
        x: f64,
        y: f64,
    ) -> Option<(i32, f64)> {
        self.core.timeline_pointer(phase, x, y)
    }

    pub(crate) fn stage_pointer(&mut self, phase: PointerPhase, x: f64, y: f64) {
        self.core.stage_pointer(phase, x, y);
    }

    pub(crate) fn stats(&self) -> RenderStats {
        self.core.stats()
    }
}
