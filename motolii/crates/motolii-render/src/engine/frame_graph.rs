//! Product playback owner: compile once per revision, evaluate once per exact
//! comp time, prepare one GPU scene, then branch only for final projection.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::doc::core::{CompSpec, RationalTime, ResolvedCamera};
use crate::doc::store::{LayerId, StoreView};
use crate::frame_graph::{BlobAnalysisRequestValue, BlobAnalysisValue, CompiledGraph, EvaluatedFrame, EvaluationContext, FrameQuality, Generation, GraphNode, GraphRevision, GraphTopology, MediaExtentValue, NodeExecutor, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, OverlayAnalysisValue, OverlaySetValue, SceneProgram, SceneValue, SolverPlanValue, TimeDependency};
use crate::picture::resolved::ResolvedLayer;

use super::frame_graph_scene::GpuSceneValue;
use super::{Engine, EngineError};

pub(super) struct EngineFrameGraph {
    graph: CompiledGraph,
    program: SceneProgram,
    scene: NodeKey,
    gpu: NodeKey,
    camera: NodeKey,
    stage: NodeKey,
    comp: CompSpec,
    fps: crate::doc::store::Fps,
    background: [f32; 4],
    in_points: HashMap<LayerId, i64>,
    frame: Option<EvaluatedFrame>,
    generation: u64,
    prepare_us: u64,
    measured: bool,
}

impl EngineFrameGraph {
    fn new(view: &StoreView<'_>, revision: GraphRevision) -> Result<Self, EngineError> {
        let composition = view.composition().map_err(store)?.ok_or(EngineError::NoComposition)?;
        let (comp, fps, background) = (composition.spec(), composition.fps, composition.background);
        let in_points = view.layers().into_iter().filter_map(|layer| {
            view.meta(layer).ok().flatten().map(|meta| (layer, meta.timing.start))
        }).collect();
        let program = SceneProgram::compile(view).map_err(|error| EngineError::Store(error.to_string()))?;
        let scene = program.scene().scene;
        let document_camera = program.camera();
        let solver = program.solver().key();
        let overlay = program.overlay().output();
        let mut nodes: Vec<_> = program.nodes().collect();
        let mut gpu_identity = NodeIdentity::new(NodeKind::GpuScene, vec![scene, document_camera, solver, overlay]);
        gpu_identity.time_dependency = TimeDependency::Exact;
        let gpu = GraphNode::new(gpu_identity);
        let projection = |role| {
            let mut identity = NodeIdentity::new(NodeKind::CameraProjection, vec![gpu.key()]);
            identity.parameters = vec![role]; identity.time_dependency = TimeDependency::Exact;
            GraphNode::new(identity)
        };
        let camera = projection(0); let stage = projection(1);
        nodes.extend([gpu.clone(), camera.clone(), stage.clone()]);
        let topology = GraphTopology::try_new(nodes, vec![scene, document_camera, gpu.key(), camera.key(), stage.key()]).map_err(|error| EngineError::Store(error.to_string()))?;
        Ok(Self { graph: CompiledGraph::with_topology(revision, topology), program, scene, gpu: gpu.key(), camera: camera.key(), stage: stage.key(), comp, fps, background, in_points, frame: None, generation: 0, prepare_us: 0, measured: false })
    }
    fn matches(&self, revision: GraphRevision, time: RationalTime) -> bool { self.graph.revision() == revision && self.frame.as_ref().is_some_and(|frame| frame.time() == time) }

    fn evaluate_at(
        &mut self,
        engine: &mut Engine,
        time: RationalTime,
        quality: FrameQuality,
        force_gpu: bool,
    ) -> Result<(), EngineError> {
        if force_gpu {
            self.graph.invalidate([self.gpu]);
        }
        self.generation += 1;
        let mut executor = ProgramExecutor {
            engine,
            program: &self.program,
            comp: self.comp,
            fps: self.fps,
        };
        let evaluated = self.graph.evaluate(
            &mut executor,
            time,
            quality,
            Generation::new(self.generation),
        )?;
        self.frame = Some(evaluated);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)] enum ProjectionRole { Camera, Stage }
#[derive(Clone)] struct Projection { scene: Arc<GpuSceneValue>, role: ProjectionRole }

struct ProgramExecutor<'a> { engine: &'a mut Engine, program: &'a SceneProgram, comp: CompSpec, fps: crate::doc::store::Fps }
impl NodeExecutor for ProgramExecutor<'_> {
    type Error = EngineError;

    fn dynamic_inputs(&mut self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Result<Vec<crate::frame_graph::DynamicInput>, Self::Error> {
        self.program.dynamic_inputs(node, inputs, context).map_err(|error| EngineError::Store(error.to_string()))
    }

    fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> {
        if node.identity().kind == NodeKind::MediaExtent {
            let source = self.program.content().extent_source(node.key())
                .cloned()
                .ok_or_else(|| unsupported(node.identity().kind))?;
            let size = self.engine.material_extent(&source.path, self.comp).unwrap_or([0.0; 3]);
            return Ok(NodeValue::new(MediaExtentValue { source, size }));
        }
        match self.program.execute(node, &inputs, &context) {
            Ok(value) => return Ok(value),
            Err(crate::frame_graph::SceneProgramError::Unsupported(_)) => {}
            Err(error) => return Err(EngineError::Store(error.to_string())),
        }
        match node.identity().kind {
            NodeKind::AnalysisBlob => {
                let request = direct::<BlobAnalysisRequestValue>(node, &inputs, 0)?;
                let previous = inputs.at(1).and_then(|value| value.downcast_ref::<BlobAnalysisValue>());
                let value = self.engine.frame_graph_blob_analysis(request, previous, context.time, self.comp, self.fps)?;
                Ok(NodeValue::new(value))
            }
            NodeKind::AnalysisOverlay => {
                let scene = direct::<SceneValue>(node, &inputs, 0)?;
                let effect = inputs.at(1)
                    .and_then(|value| value.downcast_ref::<crate::frame_graph::EffectValue>())
                    .and_then(|value| value.0.as_ref())
                    .ok_or_else(|| unsupported(node.identity().kind))?;
                let solver = direct::<SolverPlanValue>(node, &inputs, 2)?;
                let camera = *direct::<ResolvedCamera>(node, &inputs, 3)?;
                let previous = inputs.at(4).and_then(|value| value.downcast_ref::<OverlayAnalysisValue>());
                let (layer, _order, parent) = self.program.overlay().recipe(node.key())
                    .ok_or_else(|| unsupported(node.identity().kind))?;
                let value = self.engine.frame_graph_overlay_analysis(
                    layer, parent, effect, scene, solver, camera, previous, context.time, self.comp,
                )?;
                Ok(NodeValue::new(value))
            }
            NodeKind::GpuScene => {
                let scene = direct::<SceneValue>(node, &inputs, 0)?;
                let camera = *direct::<ResolvedCamera>(node, &inputs, 1)?;
                let solver = direct::<SolverPlanValue>(node, &inputs, 2)?;
                let overlays = direct::<OverlaySetValue>(node, &inputs, 3)?;
                self.engine.install_frame_graph_overlays(overlays);
                let frame = context.time.try_to_frame_round(self.fps).unwrap_or(0) as f32;
                self.engine.compositor.clock = Some([
                    context.time.as_seconds_f64() as f32,
                    self.fps.den() as f32 / self.fps.num() as f32,
                    frame,
                ]);
                let prepared = self.engine.prepare_gpu_scene_with_solver(scene, solver, self.comp, camera, context.time, self.fps)?;
                Ok(NodeValue::new(Arc::new(prepared)))
            }
            NodeKind::CameraProjection => {
                let scene = inputs.at(0).and_then(|value| value.downcast_ref::<Arc<GpuSceneValue>>()).cloned().ok_or_else(|| unsupported(node.identity().kind))?;
                let role = match node.identity().parameters.as_slice() { [0] => ProjectionRole::Camera, [1] => ProjectionRole::Stage, _ => return Err(unsupported(node.identity().kind)) };
                Ok(NodeValue::new(Arc::new(Projection { scene, role })))
            }
            kind => Err(unsupported(kind)),
        }
    }
}

impl Engine {
    /// Camera value already evaluated by the revision-scoped production graph.
    /// Editor read paths use this after preparing the same frame; no second
    /// SceneProgram/Graph is compiled just to discover the observer.
    pub fn frame_graph_cached_camera(
        &self,
        view: &StoreView<'_>,
        time: RationalTime,
    ) -> Option<ResolvedCamera> {
        let state = self.frame_graph.as_ref()?;
        if !state.matches(GraphRevision::new(view.revision_key()), time) { return None; }
        state.frame.as_ref()?
            .value(state.program.camera())?
            .downcast_ref::<ResolvedCamera>()
            .copied()
    }

    /// Local and inherited world transform evaluated by the production
    /// TransformProgram for this exact revision/time.
    pub fn frame_graph_cached_transform(
        &self,
        view: &StoreView<'_>,
        time: RationalTime,
        id: LayerId,
    ) -> Option<(crate::frame_graph::TransformValue, crate::frame_graph::TransformValue)> {
        let state = self.frame_graph.as_ref()?;
        if !state.matches(GraphRevision::new(view.revision_key()), time) { return None; }
        let binding = state.program.transforms().binding(id)?;
        let frame = state.frame.as_ref()?;
        let local = frame.value(binding.local)?.downcast_ref::<crate::frame_graph::TransformValue>().copied()?;
        let world = frame.value(binding.world)?.downcast_ref::<crate::frame_graph::TransformValue>().copied()?;
        Some((local, world))
    }

    /// Editor read model. Evaluate the production graph once and hand the
    /// semantic scene to editor geometry directly. Do not project back through
    /// ResolvedLayer: that would recreate the migration bridge we are deleting.
    pub fn frame_graph_editor_scene(
        &mut self,
        view: &StoreView<'_>,
        time: RationalTime,
    ) -> Result<SceneValue, EngineError> {
        let state = self.evaluated_frame_graph(view, time, FrameQuality::Preview { scale: 1 })?;
        let scene = state.frame.as_ref()
            .and_then(|frame| frame.value(state.scene))
            .and_then(|value| value.downcast_ref::<SceneValue>())
            .cloned()
            .ok_or_else(|| EngineError::Store("FrameGraph semantic scene is missing".into()))?;
        self.frame_graph = Some(state);
        Ok(scene)
    }

    /// Borrow the semantic scene already evaluated for this exact revision/time.
    /// Read-only editor paths use this so Camera/Stage status never compiles a
    /// second graph or asks the legacy resolver for a parallel scene.
    pub fn frame_graph_cached_scene(
        &self,
        view: &StoreView<'_>,
        time: RationalTime,
    ) -> Option<&SceneValue> {
        let state = self.frame_graph.as_ref()?;
        if !state.matches(GraphRevision::new(view.revision_key()), time) { return None; }
        state.frame.as_ref()?.value(state.scene)?.downcast_ref::<SceneValue>()
    }

    pub(in crate::engine) fn evaluate_frame_graph_semantics(
        &mut self,
        view: &StoreView<'_>,
        time: RationalTime,
    ) -> Result<(SceneValue, ResolvedCamera, CompSpec, crate::doc::store::Fps), EngineError> {
        let composition = view.composition().map_err(store)?.ok_or(EngineError::NoComposition)?;
        let program = SceneProgram::compile(view).map_err(|error| EngineError::Store(error.to_string()))?;
        let scene_key = program.scene().scene;
        let camera_key = program.camera();
        let topology = GraphTopology::try_new(program.nodes(), vec![scene_key, camera_key])
            .map_err(|error| EngineError::Store(error.to_string()))?;
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(view.revision_key()), topology);
        let mut executor = ProgramExecutor {
            engine: self,
            program: &program,
            comp: composition.spec(),
            fps: composition.fps,
        };
        let frame = graph.evaluate(
            &mut executor,
            time,
            FrameQuality::Export,
            Generation::new(1),
        )?;
        let scene = frame.value(scene_key)
            .and_then(|value| value.downcast_ref::<SceneValue>())
            .cloned()
            .ok_or_else(|| EngineError::Store("FrameGraph semantic scene is missing".into()))?;
        let camera = frame.value(camera_key)
            .and_then(|value| value.downcast_ref::<ResolvedCamera>())
            .copied()
            .unwrap_or_default();
        Ok((scene, camera, composition.spec(), composition.fps))
    }

    pub fn frame_graph_document_camera(
        &mut self,
        view: &StoreView<'_>,
        time: RationalTime,
    ) -> Result<ResolvedCamera, EngineError> {
        let state = self.evaluated_frame_graph(view, time, FrameQuality::Preview { scale: 1 })?;
        let camera = state.frame.as_ref()
            .and_then(|frame| frame.value(state.program.camera()))
            .and_then(|value| value.downcast_ref::<ResolvedCamera>())
            .copied()
            .unwrap_or_default();
        self.frame_graph = Some(state);
        Ok(camera)
    }

    /// First visual cassette slice. Semantic evaluation is lowered through the
    /// backend-neutral RenderGraph; concrete layer resources still come from the
    /// frozen GpuScene oracle until Raster/Transfer are migrated individually.
    ///
    /// This deliberately makes the new boundary draw real pixels before replacing
    /// resource preparation. The RenderGraph, not GpuScene, owns the ordered
    /// composite description on this path.
    fn cassette_plain_layers(
        &mut self,
        scene: &SceneValue,
        comp: CompSpec,
        projection_camera: ResolvedCamera,
    ) -> Result<Option<Vec<crate::render::compositor::LayerWithPasses>>, EngineError> {
        use crate::frame_graph::SceneContentValue;
        use crate::render::compositor::{Layer, LayerContent, LayerWithPasses};

        let mut layers = Vec::new();
        for source in &scene.layers {
            if !source.effects.is_empty()
                || !source.after_effects.is_empty()
                || !source.image_sources.is_empty()
                || !source.masks.is_empty()
                || source.matte.is_some()
                || source.clip_to_below
                || source.flatten
                || source.environment
            {
                return Ok(None);
            }

            let (content, natural) = match &source.content {
                SceneContentValue::None => continue,
                SceneContentValue::Text(text) => {
                    self.text_texture_from_shapes(&text.shapes(), source.layer, comp)?
                }
                SceneContentValue::Shape(shapes) => {
                    let stretched;
                    let shapes = if source.shape_stretch != [1.0, 1.0] {
                        stretched = crate::picture::shapes_ops::stretch_outline(shapes, source.shape_stretch);
                        stretched.as_slice()
                    } else {
                        shapes.as_slice()
                    };
                    self.shape_texture_from_shapes(
                        shapes,
                        source.layer,
                        true,
                        0.05,
                        comp,
                        None,
                        source.shape_stretch == [1.0, 1.0],
                    )?
                }
                SceneContentValue::Material(material) => {
                    self.mesh_content_for(&material.source.path, comp)?
                }
                SceneContentValue::Media { source: media, time } => {
                    self.file_content_for(&media.path, *time, source.layer, comp)?
                }
                SceneContentValue::Particles(value) => {
                    let frame = super::ParticleFrame::from_particles(
                        &value.particles,
                        value.turbulence,
                        value.links,
                    );
                    let natural = [
                        frame.bounds.max[0].max(1.0),
                        frame.bounds.max[1].max(1.0),
                    ];
                    (
                        Some(LayerContent::Cloud {
                            positions: frame.positions,
                            colors: frame.colors,
                            bounds: frame.bounds,
                            point_size: 1.0,
                            sizes: Some(frame.sizes),
                            sprites: true,
                            links: frame.links,
                        }),
                        natural,
                    )
                }
                // A plate is already a nested semantic composite. Keep it on the
                // frozen oracle until RenderGraph owns nested composite resources.
                SceneContentValue::Plate(_) => return Ok(None),
            };
            let Some(content) = content else { continue };

            let placement = crate::doc::core::LayerPlacement {
                transform: source.transform.affine,
                world_transform: Some(source.transform.spatial),
                opacity: source.opacity,
                order: i32::from(source.order),
                z: source.transform.spatial.translation.z,
                rotation_x: 0.0,
                rotation_y: 0.0,
                plane: None,
            };
            let layer = Layer {
                content,
                size: natural,
                placement,
                projection: source.projection,
                projection_camera,
                blend_mode: crate::render::engine::translate::translate_blend_mode(source.blend)?,
                shading: Default::default(),
                displace: Default::default(),
                clip: None,
                shadow: 0.0,
                outline: self.outline_id(source.layer),
                frame: None,
            };
            layers.push(LayerWithPasses {
                layer,
                passes: Vec::new(),
                padding: 0,
                pass_sources: Vec::new(),
                cut: Vec::new(),
            });
        }
        Ok(Some(layers))
    }

    pub(in crate::engine) fn render_frame_graph_cassette_pixels(
        &mut self,
        view: &StoreView<'_>,
        time: RationalTime,
        include_background: bool,
        camera_override: Option<ResolvedCamera>,
    ) -> Result<Vec<u8>, EngineError> {
        // Important: the supported cassette path evaluates only semantic roots.
        // It must not construct/evaluate NodeKind::GpuScene just to discover
        // whether the new lowering/backend path can render the scene.
        let (scene, document_camera, comp, _fps) =
            self.evaluate_frame_graph_semantics(view, time)?;
        let _graph = crate::render_lowering::lower_scene(&scene)
            .map_err(|_| EngineError::Store("Render lowering failed".into()))?;

        if let Some(mut layers) = self.cassette_plain_layers(&scene, comp, document_camera)? {
            for layer in &mut layers {
                layer.layer.projection_camera = document_camera;
            }
            let camera = camera_override.unwrap_or(document_camera);
            let background = if include_background {
                view.composition().map_err(store)?
                    .map_or(crate::render::compositor::NO_BACKGROUND, |c| c.background)
            } else {
                crate::render::compositor::NO_BACKGROUND
            };
            return self.compositor.render_with_effects(comp, camera, &layers, background)
                .map_err(Into::into);
        }

        // Unsupported semantics remain on the frozen reference path until their
        // Raster/Filter/Composite work has been migrated with parity evidence.
        self.render_frame_graph_pixels(view, time, include_background, camera_override)
    }

    pub(in crate::engine) fn render_frame_graph_pixels(
        &mut self,
        view: &StoreView<'_>,
        time: RationalTime,
        include_background: bool,
        camera_override: Option<ResolvedCamera>,
    ) -> Result<Vec<u8>, EngineError> {
        let mut state = self.evaluated_frame_graph(view, time, FrameQuality::Export)?;
        let prepared = state.frame.as_ref()
            .and_then(|frame| frame.value(state.gpu))
            .and_then(|value| value.downcast_ref::<Arc<GpuSceneValue>>())
            .cloned()
            .ok_or_else(|| EngineError::Store("FrameGraph GPU scene is missing".into()))?;
        let document_camera = state.frame.as_ref()
            .and_then(|frame| frame.value(state.program.camera()))
            .and_then(|value| value.downcast_ref::<ResolvedCamera>())
            .copied()
            .unwrap_or_default();
        let camera = camera_override.unwrap_or(document_camera);
        let mut layers = prepared.layers.clone();
        for layer in &mut layers {
            layer.layer.projection_camera = document_camera;
        }
        let background = if include_background { state.background } else { crate::render::compositor::NO_BACKGROUND };
        let pixels = self.compositor.render_with_effects(state.comp, camera, &layers, background)?;
        self.frame_graph = Some(state);
        Ok(pixels)
    }

    pub(in crate::engine) fn render_frame_graph_to_texture_output(
        &mut self,
        view: &StoreView<'_>,
        time: RationalTime,
        include_background: bool,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        let mut state = self.evaluated_frame_graph(view, time, FrameQuality::Export)?;
        let prepared = state.frame.as_ref()
            .and_then(|frame| frame.value(state.gpu))
            .and_then(|value| value.downcast_ref::<Arc<GpuSceneValue>>())
            .cloned()
            .ok_or_else(|| EngineError::Store("FrameGraph GPU scene is missing".into()))?;
        let document_camera = state.frame.as_ref()
            .and_then(|frame| frame.value(state.program.camera()))
            .and_then(|value| value.downcast_ref::<ResolvedCamera>())
            .copied()
            .unwrap_or_default();
        let mut layers = prepared.layers.clone();
        for layer in &mut layers {
            layer.layer.projection_camera = document_camera;
        }
        let background = if include_background { state.background } else { crate::render::compositor::NO_BACKGROUND };
        let texture = self.compositor.render_to_texture(state.comp, document_camera, &layers, background)?;
        self.frame_graph = Some(state);
        Ok(texture)
    }

    fn evaluated_frame_graph(&mut self, view: &StoreView<'_>, time: RationalTime, quality: FrameQuality) -> Result<EngineFrameGraph, EngineError> {
        let revision = GraphRevision::new(view.revision_key());
        let mut state = match self.frame_graph.take() { Some(state) if state.graph.revision() == revision => state, _ => EngineFrameGraph::new(view, revision)? };
        self.compositor.feedback_set_revision(view.revision_key());
        self.feedback_keys_seen.clear();
        if !state.matches(revision, time) {
            let started = std::time::Instant::now();
            if let Err(error) = state.evaluate_at(self, time, quality, false) {
                self.frame_graph = Some(state);
                return Err(error);
            }
            state.prepare_us = started.elapsed().as_micros() as u64;
            state.measured = false;
        }
        Ok(state)
    }

    pub(super) fn semantic_layer_for(&self, view: &StoreView<'_>, time: RationalTime, id: LayerId) -> Option<&crate::frame_graph::SceneLayerValue> {
        self.frame_graph_cached_scene(view, time)?.layer(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_frame_graph_into_window(
        &mut self,
        view: &StoreView<'_>,
        time: RationalTime,
        target: &wgpu::Texture,
        camera: ResolvedCamera,
        include_background: bool,
        outline: &[LayerId],
        window: crate::render::compositor::Window,
        projection: crate::frame_graph::ViewProjection,
    ) -> Result<(), EngineError> {
        let mut state = self.evaluated_frame_graph(view, time, FrameQuality::Preview { scale: 1 })?;
        let frame_start = std::time::Instant::now();
        self.compositor.measurement = Default::default();
        if !state.measured {
            self.compositor.measurement.resolve_us = state.prepare_us;
            state.measured = true;
        }
        self.outline_layers = outline.iter().copied().take(255).collect();
        self.outline_order = self.outline_layers.clone();

        self.render_frame_graph_projection(
            &state,
            target,
            camera,
            include_background,
            window,
            projection,
        )?;

        let now = time.try_to_frame_round(state.fps).unwrap_or(0);
        if let Some(start) = self.frame_graph_feedback_replay_start(&state, now) {
            self.replay_frame_graph_feedback(
                &mut state,
                start,
                now,
                camera,
                include_background,
                window,
                projection,
            )?;

            let c = &mut self.compositor;
            c.baked_effects.clear(&mut c.effect_scratch);
            self.feedback_keys_seen.clear();
            state.evaluate_at(self, time, FrameQuality::Preview { scale: 1 }, true)?;
            self.render_frame_graph_projection(
                &state,
                target,
                camera,
                include_background,
                window,
                projection,
            )?;
        }

        self.outline_layers.clear();
        self.compositor.measurement.total_us = frame_start.elapsed().as_micros() as u64;
        self.frame_graph = Some(state);
        Ok(())
    }

    fn render_frame_graph_projection(
        &mut self,
        state: &EngineFrameGraph,
        target: &wgpu::Texture,
        camera: ResolvedCamera,
        include_background: bool,
        window: crate::render::compositor::Window,
        projection: crate::frame_graph::ViewProjection,
    ) -> Result<(), EngineError> {
        let root = match projection {
            crate::frame_graph::ViewProjection::Camera => state.camera,
            crate::frame_graph::ViewProjection::Stage => state.stage,
            _ => return Err(EngineError::Store("Unsupported playback projection".into())),
        };
        let projected = state.frame.as_ref()
            .and_then(|frame| frame.value(root))
            .and_then(|value| value.downcast_ref::<Arc<Projection>>())
            .cloned()
            .ok_or_else(|| EngineError::Store("FrameGraph projection is missing".into()))?;
        let expected = if projection == crate::frame_graph::ViewProjection::Camera {
            ProjectionRole::Camera
        } else {
            ProjectionRole::Stage
        };
        if projected.role != expected {
            return Err(EngineError::Store("FrameGraph projection role mismatch".into()));
        }

        let document_camera = state.frame.as_ref()
            .and_then(|frame| frame.value(state.program.camera()))
            .and_then(|value| value.downcast_ref::<ResolvedCamera>())
            .copied()
            .unwrap_or_default();
        let projection_camera = window.projection_camera.unwrap_or(document_camera);
        let mut layers = projected.scene.layers.clone();
        for layer in &mut layers {
            layer.layer.projection_camera =
                if layer.layer.projection == crate::doc::store::LayerProjection::TwoD {
                    document_camera
                } else {
                    projection_camera
                };
        }
        self.stamp_frame_graph_window_feedback(&mut layers, window);
        let background = if include_background {
            state.background
        } else {
            crate::render::compositor::NO_BACKGROUND
        };
        self.compositor.render_into_window(target, state.comp, camera, &layers, background, window)?;
        Ok(())
    }

    fn frame_graph_feedback_replay_start(
        &mut self,
        state: &EngineFrameGraph,
        now: i64,
    ) -> Option<i64> {
        let seen = std::mem::take(&mut self.feedback_keys_seen);
        let mut unique = HashSet::new();
        let mut start: Option<i64> = None;
        for key in seen.into_iter().filter(|key| unique.insert(*key)) {
            let Some(have) = self.compositor.feedback_frame(key) else { continue };
            if have == now && !self.compositor.feedback_is_fresh(key) {
                continue;
            }
            let in_point = state.in_points.get(&key.layer).copied().unwrap_or(0);
            let from = self.compositor.feedback_restore(key, now - 1)
                .map_or(in_point, |checkpoint| checkpoint + 1);
            if from < now {
                start = Some(start.map_or(from, |current| current.min(from)));
            }
        }
        start
    }

    #[allow(clippy::too_many_arguments)]
    fn replay_frame_graph_feedback(
        &mut self,
        state: &mut EngineFrameGraph,
        start: i64,
        now: i64,
        camera: ResolvedCamera,
        include_background: bool,
        window: crate::render::compositor::Window,
        projection: crate::frame_graph::ViewProjection,
    ) -> Result<(), EngineError> {
        let replay_target = self.compositor.ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("motolii-frame-graph-feedback-replay"),
            size: wgpu::Extent3d {
                width: window.width,
                height: window.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::render::compositor::PRESENTABLE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        for frame in start..now {
            self.pending_frame_copies.clear();
            self.feedback_keys_seen.clear();
            let at = RationalTime::try_from_frame(frame, state.fps)
                .map_err(|error| EngineError::Time(error.to_string()))?;
            state.evaluate_at(self, at, FrameQuality::Preview { scale: 1 }, true)?;
            self.render_frame_graph_projection(
                state,
                &replay_target,
                camera,
                include_background,
                window,
                projection,
            )?;
        }
        self.pending_frame_copies.clear();
        self.feedback_keys_seen.clear();
        Ok(())
    }
}



#[cfg(test)]
mod cassette_tests {
    use super::*;
    use motolii_edit::{Document, Intent};
    use crate::doc::store::{Composition, Fps};

    #[test]
    fn plain_still_scene_renders_real_pixels_without_gpu_scene_resource_preparation() {
        let dir = tempfile::tempdir().unwrap();
        let image_path = dir.path().join("cassette-red.png");
        image::RgbaImage::from_pixel(32, 32, image::Rgba([255, 0, 0, 255]))
            .save(&image_path)
            .unwrap();

        let mut doc = Document::new().with_programs(crate::extensions::bundled());
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 1,
            background: [0.0, 0.0, 0.0, 1.0],
        })).unwrap();
        super::super::environment_tests::file_layer(&mut doc, 1, 0, &image_path);

        let mut engine = Engine::new().unwrap();
        let (scene, camera, comp, _) = engine
            .evaluate_frame_graph_semantics(&doc.view(), RationalTime::ZERO)
            .unwrap();
        let cassette = engine
            .cassette_plain_layers(&scene, comp, camera)
            .unwrap()
            .expect("plain still scene must be fully owned by the cassette path");
        assert_eq!(cassette.len(), 1);

        let pixels = engine
            .render_with_camera_override(&doc.view(), RationalTime::ZERO, true, None)
            .unwrap();
        assert_eq!(pixels.len(), 64 * 64 * 4);
        assert!(pixels.chunks_exact(4).any(|p| p[0] > 200 && p[1] < 40 && p[2] < 40));
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    }
}


fn direct<'a, T: 'static>(node: &GraphNode, inputs: &'a NodeInputs, index: usize) -> Result<&'a T, EngineError> { inputs.at(index).and_then(|value| value.downcast_ref::<T>()).ok_or_else(|| EngineError::Store(format!("FrameGraph {:?} has invalid input {index}", node.identity().kind))) }
fn store(error: crate::doc::store::StoreError) -> EngineError { EngineError::Store(error.to_string()) }
fn unsupported(kind: NodeKind) -> EngineError { EngineError::Store(format!("FrameGraph program executor does not support {kind:?}")) }
