use crate::doc::core::{CompSpec, LayerPlacement, ResolvedCamera};
use crate::doc::store::LayerId;
use crate::frame_graph::{SceneContentValue, SceneImageSourceValue, SceneLayerValue, SceneValue, SolverPlanValue};
use crate::render::compositor::{BlendMode as CompositeBlendMode, Layer, LayerWithPasses};
use crate::render::engine::{Engine, EngineError};

impl Engine {
    pub(super) fn prepare_execution_scene_with_solver(
        &mut self,
        scene: &SceneValue,
        solver: &SolverPlanValue,
        comp: CompSpec,
        projection_camera: ResolvedCamera,
        time: crate::doc::core::RationalTime,
        fps: crate::doc::store::Fps,
    ) -> Result<crate::gpu_exec::ExecutableScene, EngineError> {
        let physics_overlays: std::collections::HashSet<LayerId> = self.overlay_frames.iter()
            .filter_map(|(layer, frame)| frame.physics.then_some(*layer))
            .collect();
        if physics_overlays.is_empty() {
            // Culling is execution planning only: semantic SceneValue remains
            // untouched and cacheable. The planner may omit only simple media
            // contributions that cannot feed analysis/matte/solver work.
            let planned = self.gpu_plan_visible_scene(scene, solver, comp, projection_camera);
            let scene = planned.as_ref().unwrap_or(scene);
            let mut prepared = self.gpu_executable_scene(scene, comp, projection_camera)?;
            self.prepare_frame_graph_blocks(scene, solver, comp, time, fps, &mut prepared)?;
            return Ok(prepared);
        }

        // Physics Trace reads the solver's current-frame state. Build the
        // ordinary scene first, solve motion/physics, then lower the complete
        // scene once the overlay can read those results.
        let base_scene = SceneValue {
            layers: scene.layers.iter()
                .filter(|layer| !physics_overlays.contains(&layer.layer))
                .cloned()
                .collect(),
        };
        let mut base = self.gpu_executable_scene(&base_scene, comp, projection_camera)?;
        self.prepare_frame_graph_blocks(scene, solver, comp, time, fps, &mut base)?;

        let mut prepared = self.gpu_executable_scene(scene, comp, projection_camera)?;
        for (id, layer) in prepared.layer_ids.iter().copied().zip(prepared.layers.iter_mut()) {
            self.attach_block_id(id, &mut layer.layer, comp);
        }
        Ok(prepared)
    }

    pub(crate) fn frame_graph_extruded_content(
        &mut self,
        source: &crate::frame_graph::SceneLayerValue,
        texture: crate::render::compositor::GpuTexture2D,
        natural: [f32; 2],
        comp: CompSpec,
        solid: crate::render::compositor::extrude::Solid,
    ) -> Result<crate::render::compositor::LayerContent, EngineError> {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        texture.handle.hash(&mut hasher);
        solid.hash_key(&mut hasher);
        source.shape_stretch[0].to_bits().hash(&mut hasher);
        source.shape_stretch[1].to_bits().hash(&mut hasher);
        let key = hasher.finish();
        if let Some((cached, model)) = self.extrusions.get(&source.layer) {
            if *cached == key {
                return Ok(crate::render::compositor::LayerContent::Model(model.clone()));
            }
        }

        let rectangle = |size: [f32; 2]| {
            let vertex = |x: f32, y: f32| re_renderer::renderer::PathVertex {
                point: glam::vec2(x, y),
                in_tangent: glam::Vec2::ZERO,
                out_tangent: glam::Vec2::ZERO,
            };
            vec![(
                vec![re_renderer::renderer::PathContour {
                    closed: true,
                    vertices: vec![
                        vertex(0.0, 0.0),
                        vertex(size[0], 0.0),
                        vertex(size[0], size[1]),
                        vertex(0.0, size[1]),
                    ],
                }],
                re_renderer::renderer::PathFillRule::NonZero,
            )]
        };

        let outlines = match &source.content {
            crate::frame_graph::SceneContentValue::Text(text) => {
                let shapes = text.shapes();
                match crate::picture::shapes_ops::content_canvas(&shapes)? {
                    Some(canvas) => crate::render::compositor::paths::outlines(&shapes, &canvas)?,
                    None => Vec::new(),
                }
            }
            crate::frame_graph::SceneContentValue::Shape(shapes) => {
                let stretched;
                let shapes = if source.shape_stretch != [1.0, 1.0] {
                    stretched = crate::picture::shapes_ops::stretch_outline(shapes, source.shape_stretch);
                    stretched.as_slice()
                } else {
                    shapes.as_slice()
                };
                match crate::picture::shapes_ops::content_canvas(shapes)? {
                    Some(canvas) => crate::render::compositor::paths::outlines(shapes, &canvas)?,
                    None => Vec::new(),
                }
            }
            _ => rectangle(natural),
        };

        let Some(model) = self.compositor.extrude_model(&outlines, texture.clone(), natural, solid)? else {
            return Ok(crate::render::compositor::LayerContent::Texture(texture));
        };
        let model = std::sync::Arc::new(model);
        self.extrusions.insert(source.layer, (key, model.clone()));
        let _ = comp;
        Ok(crate::render::compositor::LayerContent::Model(model))
    }

    pub(crate) fn frame_graph_frozen_content(
        &mut self,
        source: &SceneLayerValue,
    ) -> Option<(
        crate::render::compositor::LayerContent,
        [f32; 2],
        u32,
        Option<crate::render::compositor::effects::vism::ImageFrame>,
    )> {
        if !source.freeze_eligible || source.ghost || source.instance != 0 || self.freezing == Some(source.layer) {
            return None;
        }
        let comp_frame = self.compositor.clock?.get(2).copied()?.round() as i64;
        let layer_frame = comp_frame - source.timing_start;
        let Self { frozen, compositor, .. } = self;
        let picture = frozen.load(
            source.layer,
            layer_frame,
            &mut |bytes, width, height| compositor.upload_rgba16f("motolii-frozen", bytes, width, height).ok(),
        )?;
        Some((
            crate::render::compositor::LayerContent::LinearTexture(picture.texture),
            picture.natural,
            picture.padding,
            picture.frame,
        ))
    }

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
