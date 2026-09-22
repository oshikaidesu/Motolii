use crate::doc::core::{CompSpec, LayerPlacement, ResolvedCamera};
use crate::doc::store::LayerId;
use crate::frame_graph::{SceneContentValue, SceneImageSourceValue, SceneLayerValue, SceneValue, SolverPlanValue};
use crate::render::compositor::{BlendMode as CompositeBlendMode, Layer, LayerWithPasses};
use crate::render::engine::{Engine, EngineError};

impl Engine {
    pub(crate) fn set_frame_graph_source_clock(&mut self, time: crate::doc::core::RationalTime) {
        let delta = self.compositor.clock.map_or(1.0 / 30.0, |clock| clock[1].max(1.0e-9));
        let frame = (time.as_seconds_f64() as f32 / delta).round();
        self.compositor.clock = Some([time.as_seconds_f64() as f32, delta, frame]);
    }

    pub(super) fn stamp_frame_graph_window_feedback(
        &mut self,
        layers: &mut [LayerWithPasses],
        window: crate::render::compositor::Window,
    ) {
        for entry in layers {
            let screen_chain = entry.layer.content.texture().is_none()
                || entry.passes.iter().any(|pass| pass.reads_backdrop || pass.reads_composite());
            for pass in &mut entry.passes {
                if let Some(mut key) = pass.feedback {
                    if screen_chain || pass.reads_backdrop || pass.reads_composite() {
                        key.screen = Some(window.size());
                        pass.feedback = Some(key);
                    }
                    if self.feedback_namespace == 0 {
                        self.gpu_history.observe(key);
                    }
                }
            }
        }
    }
}


#[cfg(test)]
mod tests;
