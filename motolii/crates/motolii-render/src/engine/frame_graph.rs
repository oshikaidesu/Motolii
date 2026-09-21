//! Initial adapters from the existing resolve helpers into FrameGraph values.
//!
//! This file deliberately delegates all authored-value evaluation to the
//! existing Engine/render helpers. It is a wiring boundary, not a second
//! resolve implementation.

use std::collections::HashMap;
use std::sync::Arc;

use crate::doc::core::{CompSpec, RationalTime, ResolvedCamera};
use crate::doc::store::{LayerId, ShapeNode, StoreView, TextDocument};
use crate::frame_graph::{
    build_initial_topology, CompiledGraph, EvaluatedFrame, EvaluationContext, FrameQuality,
    Generation, GraphRevision, InitialTopology, NodeExecutor, NodeInputs, NodeKey, NodeKind,
    NodeValue,
};
use crate::picture::resolved::ResolvedLayer;

use super::render::{collect_shape_documents, collect_text_documents};
use super::{Engine, EngineError};

pub(super) struct EngineFrameGraph {
    graph: CompiledGraph,
    world: NodeKey,
    camera: NodeKey,
    stage: NodeKey,
    frame: Option<EvaluatedFrame>,
    generation: u64,
    prepare_us: u64,
    measured: bool,
}

impl EngineFrameGraph {
    fn new(revision: GraphRevision) -> Result<Self, EngineError> {
        let InitialTopology {
            output,
            world,
            camera,
            stage,
            ..
        } = build_initial_topology().map_err(|error| EngineError::Store(error.to_string()))?;
        Ok(Self {
            graph: CompiledGraph::with_topology(revision, output.topology),
            world: world.0,
            camera: camera.0,
            stage: stage.0,
            frame: None,
            generation: 0,
            prepare_us: 0,
            measured: false,
        })
    }

    fn matches(&self, revision: GraphRevision, time: RationalTime) -> bool {
        self.graph.revision() == revision
            && self
                .frame
                .as_ref()
                .is_some_and(|frame| frame.time() == time)
    }
}

#[derive(Clone)]
pub(super) struct ResolvedWorld {
    pub layers: Arc<Vec<ResolvedLayer>>,
    pub comp: CompSpec,
    pub background: [f32; 4],
}

#[derive(Clone)]
pub(super) struct TextDocuments {
    pub documents: Arc<HashMap<LayerId, TextDocument>>,
}

#[derive(Clone)]
pub(super) struct ShapeDocuments {
    pub documents: Arc<HashMap<LayerId, Vec<ShapeNode>>>,
}

#[derive(Clone, Copy)]
pub(super) struct DocumentCamera {
    pub camera: ResolvedCamera,
}

#[derive(Clone)]
pub(super) struct SharedScene {
    pub world: Arc<ResolvedWorld>,
    pub text: Arc<TextDocuments>,
    pub shapes: Arc<ShapeDocuments>,
    pub document_camera: Arc<DocumentCamera>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ProjectionRole {
    Camera,
    Stage,
}

#[derive(Clone)]
pub(super) struct Projection {
    pub scene: Arc<SharedScene>,
    pub role: ProjectionRole,
}

/// Borrows the existing renderer and one read-only document snapshot for a
/// single graph evaluation. No payload reaches back into `StoreView`.
pub(super) struct InitialNodeExecutor<'a> {
    engine: &'a mut Engine,
    view: StoreView<'a>,
}

impl<'a> InitialNodeExecutor<'a> {
    pub(super) fn new(engine: &'a mut Engine, view: StoreView<'a>) -> Self {
        Self { engine, view }
    }
}

impl NodeExecutor for InitialNodeExecutor<'_> {
    type Error = EngineError;

    fn execute(
        &mut self,
        node: &crate::frame_graph::GraphNode,
        inputs: NodeInputs,
        context: EvaluationContext,
    ) -> Result<NodeValue, Self::Error> {
        match node.identity().kind {
            NodeKind::ResolvedWorld => {
                let composition = self
                    .view
                    .composition()
                    .map_err(|error| EngineError::Store(error.to_string()))?
                    .ok_or(EngineError::NoComposition)?;
                Ok(value(Arc::new(ResolvedWorld {
                    layers: Arc::new(
                        self.engine
                            .resolved_with_analysis(&self.view, context.time)?,
                    ),
                    comp: composition.spec(),
                    background: composition.background,
                })))
            }
            NodeKind::TextDocuments => {
                let world = input::<ResolvedWorld>(node, &inputs, 0)?;
                Ok(value(Arc::new(TextDocuments {
                    documents: Arc::new(collect_text_documents(
                        &self.view,
                        &world.layers,
                        context.time,
                    )?),
                })))
            }
            NodeKind::ShapeDocuments => {
                let world = input::<ResolvedWorld>(node, &inputs, 0)?;
                Ok(value(Arc::new(ShapeDocuments {
                    documents: Arc::new(collect_shape_documents(
                        &self.view,
                        &world.layers,
                        context.time,
                    )?),
                })))
            }
            NodeKind::DocumentCamera => {
                let world = input::<ResolvedWorld>(node, &inputs, 0)?;
                Ok(value(Arc::new(DocumentCamera {
                    camera: self.engine.resolve_camera_in(
                        &self.view,
                        &world.layers,
                        context.time,
                    )?,
                })))
            }
            // The initial compiler gives the contribution all four values;
            // SceneComposite then forwards this one shared upstream result.
            NodeKind::SharedScene => Ok(value(Arc::new(SharedScene {
                world: input(node, &inputs, 0)?,
                text: input(node, &inputs, 1)?,
                shapes: input(node, &inputs, 2)?,
                document_camera: input(node, &inputs, 3)?,
            }))),
            NodeKind::SceneComposite => Ok(value(input::<SharedScene>(node, &inputs, 0)?)),
            NodeKind::CameraProjection => {
                let scene = input::<SharedScene>(node, &inputs, 0)?;
                let role = match node.identity().parameters.as_slice() {
                    [0] => ProjectionRole::Camera,
                    [1] => ProjectionRole::Stage,
                    _ => return Err(unsupported(node.identity().kind)),
                };
                Ok(value(Arc::new(Projection { scene, role })))
            }
            kind => Err(unsupported(kind)),
        }
    }
}

impl Engine {
    /// The resolved world already published by FrameGraph for this document
    /// revision and time. Status/bounds readers consume this value instead of
    /// maintaining a second renderer-owned memo.
    pub fn resolved_for(
        &self,
        view: &StoreView<'_>,
        time: RationalTime,
    ) -> Option<Vec<ResolvedLayer>> {
        let state = self.frame_graph.as_ref()?;
        if !state.matches(GraphRevision::new(view.revision_key()), time) {
            return None;
        }
        state
            .frame
            .as_ref()?
            .value(state.world)?
            .downcast_ref::<Arc<ResolvedWorld>>()
            .map(|world| world.layers.as_ref().clone())
    }

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
        let revision = GraphRevision::new(view.revision_key());
        let mut state = match self.frame_graph.take() {
            Some(state) if state.graph.revision() == revision => state,
            _ => EngineFrameGraph::new(revision)?,
        };
        if !state.matches(revision, time) {
            state.generation += 1;
            let generation = Generation::new(state.generation);
            let started = std::time::Instant::now();
            let mut executor = InitialNodeExecutor::new(self, view.clone());
            let evaluated = state.graph.evaluate(
                &mut executor,
                time,
                FrameQuality::Preview { scale: 1 },
                generation,
            );
            state.prepare_us = started.elapsed().as_micros() as u64;
            state.measured = false;
            match evaluated {
                Ok(frame) => state.frame = Some(frame),
                Err(error) => {
                    self.frame_graph = Some(state);
                    return Err(error);
                }
            }
        }
        let root = match projection {
            crate::frame_graph::ViewProjection::Camera => state.camera,
            crate::frame_graph::ViewProjection::Stage => state.stage,
            _ => {
                self.frame_graph = Some(state);
                return Err(EngineError::Store("Unsupported playback projection".into()));
            }
        };
        let projected = state
            .frame
            .as_ref()
            .and_then(|frame| frame.value(root))
            .and_then(|value| value.downcast_ref::<Arc<Projection>>())
            .cloned()
            .ok_or_else(|| EngineError::Store("FrameGraph projection is missing".into()))?;
        let expected_role = match projection {
            crate::frame_graph::ViewProjection::Camera => ProjectionRole::Camera,
            crate::frame_graph::ViewProjection::Stage => ProjectionRole::Stage,
            _ => unreachable!(),
        };
        if projected.role != expected_role {
            self.frame_graph = Some(state);
            return Err(EngineError::Store(
                "FrameGraph projection role mismatch".into(),
            ));
        }
        let scene = Arc::clone(&projected.scene);

        let frame_start = std::time::Instant::now();
        self.compositor.measurement = Default::default();
        if !state.measured {
            self.compositor.measurement.resolve_us = state.prepare_us;
            state.measured = true;
        }
        self.outline_layers = outline.iter().copied().take(255).collect();
        self.outline_order = self.outline_layers.clone();
        let projection_camera = window
            .projection_camera
            .unwrap_or(scene.document_camera.camera);
        self.feedback_window = Some(window);
        let layer_start = std::time::Instant::now();
        let built = self.layers_from_resolved(
            view,
            scene.world.comp,
            camera,
            projection_camera,
            time,
            &scene.world.layers,
            &scene.text.documents,
            &scene.shapes.documents,
        );
        self.compositor.measurement.layer_build_us = layer_start.elapsed().as_micros() as u64;
        self.feedback_window = None;
        let mut layers = match built {
            Ok(layers) => layers,
            Err(error) => {
                self.frame_graph = Some(state);
                return Err(error);
            }
        };
        for layer in &mut layers {
            if layer.layer.projection == crate::doc::store::LayerProjection::TwoD {
                layer.layer.projection_camera = scene.document_camera.camera;
            }
        }
        let background = if include_background {
            scene.world.background
        } else {
            crate::render::compositor::NO_BACKGROUND
        };
        let drawn = self.compositor.render_into_window(
            target,
            scene.world.comp,
            camera,
            &layers,
            background,
            window,
        );
        self.outline_layers.clear();
        self.compositor.measurement.total_us = frame_start.elapsed().as_micros() as u64;
        self.frame_graph = Some(state);
        Ok(drawn?)
    }
}

fn value<T>(payload: Arc<T>) -> NodeValue
where
    T: Send + Sync + 'static,
{
    NodeValue::new(payload)
}

fn input<T>(
    node: &crate::frame_graph::GraphNode,
    inputs: &NodeInputs,
    index: usize,
) -> Result<Arc<T>, EngineError>
where
    T: Send + Sync + 'static,
{
    if node.identity().inputs.get(index).is_none() {
        return Err(EngineError::Store(format!(
            "FrameGraph {:?} is missing input {index}",
            node.identity().kind
        )));
    }
    inputs
        .at(index)
        .and_then(|value| value.downcast_ref::<Arc<T>>())
        .cloned()
        .ok_or_else(|| {
            EngineError::Store(format!(
                "FrameGraph {:?} has invalid input {index}",
                node.identity().kind
            ))
        })
}

fn unsupported(kind: NodeKind) -> EngineError {
    EngineError::Store(format!(
        "FrameGraph initial executor does not support {kind:?}"
    ))
}
