//! The single scheduling boundary between authored document state and views.
//! S0 deliberately contains no renderer adapter: later lanes supply node
//! evaluators and GPU submissions without changing these ownership rules.

mod analysis_program;
mod cache;
mod canonical;
mod camera_program;
mod compiler;
mod content;
mod effect_program;
mod evaluate;
mod flow_program;
mod group_program;
mod ghost_program;
mod group_composite_program;
mod initial;
mod key;
mod lookbehind_program;
mod mask_program;
mod motion_program;
mod placement_program;
mod particle_program;
mod overlay_program;
mod property;
mod relation_program;
mod program;
mod scheduler;
mod scene_program;
mod scene_policy;
mod solver_program;
mod text_program;
mod text_flow_program;
mod topology;
mod transform;
mod value;
mod visibility_program;

use std::collections::BTreeSet;

use crate::doc::core::RationalTime;
use crate::doc::store::StoreView;

pub use analysis_program::{AnalysisBinding, AnalysisProgram, AnalysisProgramError, BlobAnalysisRequestValue, BlobAnalysisValue};
pub use cache::NodeValue;
pub use canonical::{CanonicalEncoder, CanonicalError};
pub use camera_program::{CameraProgram, CameraProgramError};
pub use compiler::{CompilerOutput, GraphBuilder, LayerBinding};
pub use content::{ContentBinding, ContentProgram, ContentProgramError, MaterialValue, MediaExtentValue, MediaFrameValue, MediaSourceValue};
pub use effect_program::{EffectBinding, EffectProgram, EffectProgramError, EffectValue};
pub use evaluate::{DynamicInput, EvaluationContext, NodeExecutor, NodeInputs};
pub use flow_program::{FlowBinding, FlowFrameValue, FlowProgram, FlowProgramError, FlowSlot};
pub use group_program::{GroupBackgroundProgram, GroupBackgroundProgramError};
pub use initial::{
    build_initial_topology, CameraRoot, DocumentCamera, InitialTopology, ResolvedWorld,
    ShapeDocuments, SharedScene, StageRoot, TextDocuments,
};
pub use key::{
    FrameQuality, InputTime, NodeIdentity, NodeKey, NodeKind, QualityDependency, TimeDependency,
    WorkKey,
};
pub use mask_program::{MaskBinding, MaskProgram, MaskProgramError, MaskValue};
pub use lookbehind_program::{LookbehindProgram, LookbehindProgramError};
pub use motion_program::{MotionBinding, MotionPlanValue, MotionProgram, MotionProgramError, MotionSamplesValue};
pub use placement_program::{PlacementBinding, PlacementCopyValue, PlacementProgram, PlacementProgramError, PlacementSetValue};
pub use particle_program::{ParticleBinding, ParticleProgram, ParticleProgramError, ParticleValue};
pub use overlay_program::{OverlayAnalysisValue, OverlayProgram, OverlayProgramError, OverlaySetValue};
pub use property::{PropertyBinding, PropertyProgram, PropertyProgramError};
pub use relation_program::{RelationBinding, RelationProgram, RelationProgramError, RelationSetValue, RelationValue};
pub use solver_program::{SolverLayerValue, SolverPlanValue, SolverProgram, SolverProgramError};
pub use program::{SceneProgram, SceneProgramError};
pub use topology::{GraphNode, GraphTopology, TopologyError};
pub use transform::{TransformBinding, TransformProgram, TransformProgramError, TransformValue};
pub use text_program::{TextBinding, TextProgram, TextProgramError, TextShapeValue};
pub use text_flow_program::{TextFlowBinding, TextFlowProgram, TextFlowProgramError};
pub use scene_program::{SceneContentValue, SceneImageSourceValue, SceneLayerValue, SceneNodeError, SceneNodeProgram, ScenePlateValue, SceneProgramNodes, SceneValue};
pub use visibility_program::{VisibilityBinding, VisibilityProgram, VisibilityProgramError, VisibilityValue};
pub use value::{
    EvaluatedFrame, FrameState, Generation, GraphRevision, GraphStats, PublishedFrame,
    RenderTarget, Submission, ViewProjection,
};

/// Compile document meaning once for the current document read revision.
///
/// S0 freezes the entry point while keeping topology empty. The compiler lane
/// will replace only the topology construction, not the document/evaluation
/// boundary exposed here.
pub fn compile(view: &StoreView<'_>) -> CompiledGraph {
    CompiledGraph::empty(GraphRevision::new(view.revision_key()))
}

/// Immutable topology plus the mutable result cache and generation gate.
/// `StoreView` exists only in [`compile`], never in this owner.
pub struct CompiledGraph {
    revision: GraphRevision,
    topology: GraphTopology,
    scheduler: scheduler::FrameScheduler,
}

impl CompiledGraph {
    pub fn empty(revision: GraphRevision) -> Self {
        Self::with_topology(revision, GraphTopology::empty())
    }

    pub fn with_topology(revision: GraphRevision, topology: GraphTopology) -> Self {
        Self {
            revision,
            topology,
            scheduler: scheduler::FrameScheduler::default(),
        }
    }

    pub fn revision(&self) -> GraphRevision {
        self.revision
    }

    pub fn topology(&self) -> &GraphTopology {
        &self.topology
    }

    pub fn stats(&self) -> GraphStats {
        let scheduler = self.scheduler.stats();
        GraphStats {
            topology_compiles: 1,
            node_executions: scheduler.node_executions,
            node_reuses: scheduler.node_reuses,
            cancelled_generations: scheduler.cancelled_generations,
            cancelled_evaluations: scheduler.cancelled_evaluations,
            cached_results: scheduler.cached_results,
        }
    }

    /// Remove only changed nodes and their graph descendants from the result
    /// cache. Unrelated nodes remain available to the next generation.
    pub fn invalidate(&mut self, changed: impl IntoIterator<Item = NodeKey>) -> BTreeSet<NodeKey> {
        self.scheduler.invalidate(&self.topology, changed).nodes
    }

    /// Evaluate each reachable node at most once. S0 records the scheduling
    /// result only; the evaluator lane will attach actual node values here.
    pub fn evaluate<E: NodeExecutor>(
        &mut self,
        executor: &mut E,
        time: RationalTime,
        quality: FrameQuality,
        generation: Generation,
    ) -> Result<EvaluatedFrame, E::Error> {
        let scheduled = evaluate::evaluate(
            &mut self.scheduler,
            &self.topology,
            executor,
            time,
            quality,
            generation,
        )?;
        Ok(EvaluatedFrame {
            revision: self.revision,
            generation,
            time,
            quality,
            state: match scheduled.state {
                evaluate::EvaluationState::Current => FrameState::Current,
                evaluate::EvaluationState::Stale | evaluate::EvaluationState::Cancelled => {
                    FrameState::Stale
                }
            },
            values: scheduled.values,
            executed: scheduled.executed,
            reused: scheduled.reused,
            work_keys: scheduled.work_keys,
        })
    }

    /// A host may publish only a submission from the current generation of
    /// this graph. This is the cancellation gate for late GPU/worker results.
    pub fn publish(&self, submission: &Submission) -> Option<PublishedFrame> {
        (submission.state == FrameState::Current
            && submission.revision == self.revision
            && self.scheduler.may_publish(submission.generation))
        .then_some(PublishedFrame {
            generation: submission.generation,
            target: submission.target,
        })
    }
}

/// Turn an already-evaluated scene into a view-specific render request. This
/// intentionally cannot accept `StoreView`.
pub fn render_view(
    frame: &EvaluatedFrame,
    projection: ViewProjection,
    target: RenderTarget,
) -> Submission {
    Submission {
        revision: frame.revision,
        generation: frame.generation,
        projection,
        target,
        state: frame.state,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    struct EchoExecutor;

    impl NodeExecutor for EchoExecutor {
        type Error = ();

        fn execute(
            &mut self,
            node: &GraphNode,
            _inputs: NodeInputs,
            _context: EvaluationContext,
        ) -> Result<NodeValue, Self::Error> {
            Ok(NodeValue::new(node.key().as_u64()))
        }
    }

    fn node(kind: u16, inputs: Vec<NodeKey>) -> GraphNode {
        GraphNode::new(NodeIdentity::new(NodeKind::Custom(kind), inputs))
    }

    #[test]
    fn one_node_key_executes_once_through_two_roots_and_one_hundred_instances() {
        let shared = node(1, vec![]);
        let instances: Vec<_> = (0..100).map(|n| node(2 + n, vec![shared.key()])).collect();
        let scene = node(200, instances.iter().map(GraphNode::key).collect());
        let camera = node(201, vec![scene.key()]);
        let stage = node(202, vec![scene.key()]);
        let graph = GraphTopology::try_new(
            std::iter::once(shared.clone()).chain(instances).chain([
                scene.clone(),
                camera.clone(),
                stage.clone(),
            ]),
            vec![camera.key(), stage.key()],
        )
        .unwrap();
        let mut compiled = CompiledGraph::with_topology(GraphRevision::new(1), graph);
        let mut executor = EchoExecutor;

        let frame = compiled
            .evaluate(
                &mut executor,
                RationalTime::ZERO,
                FrameQuality::Export,
                Generation::new(1),
            )
            .unwrap();

        assert_eq!(
            frame
                .executed_nodes()
                .iter()
                .filter(|key| **key == shared.key())
                .count(),
            1
        );
        assert_eq!(
            frame
                .executed_nodes()
                .iter()
                .filter(|key| **key == scene.key())
                .count(),
            1
        );
        assert_eq!(compiled.stats().node_executions, 104);
    }

    #[test]
    fn dirtying_a_node_reexecutes_only_its_downstream_and_keeps_one_topology_per_revision() {
        let source = node(1, vec![]);
        let middle = node(2, vec![source.key()]);
        let output = node(3, vec![middle.key()]);
        let unrelated = node(4, vec![]);
        let graph = GraphTopology::try_new(
            [
                source.clone(),
                middle.clone(),
                output.clone(),
                unrelated.clone(),
            ],
            vec![output.key(), unrelated.key()],
        )
        .unwrap();
        let mut compiled = CompiledGraph::with_topology(GraphRevision::new(7), graph);
        let mut executor = EchoExecutor;
        compiled
            .evaluate(
                &mut executor,
                RationalTime::ZERO,
                FrameQuality::Export,
                Generation::new(1),
            )
            .unwrap();

        let dirty = compiled.invalidate([source.key()]);
        let frame = compiled
            .evaluate(
                &mut executor,
                RationalTime::ZERO,
                FrameQuality::Export,
                Generation::new(2),
            )
            .unwrap();

        assert_eq!(
            dirty,
            BTreeSet::from([source.key(), middle.key(), output.key()])
        );
        assert_eq!(
            frame.executed_nodes(),
            &[source.key(), middle.key(), output.key()]
        );
        assert_eq!(frame.reused_nodes(), &[unrelated.key()]);
        assert_eq!(compiled.stats().topology_compiles, 1);
    }

    #[test]
    fn exposed_scene_and_camera_survive_cached_projection_on_seek_back() {
        let scene = node(1, vec![]);
        let camera = node(2, vec![]);
        let gpu = node(3, vec![scene.key(), camera.key()]);
        let projection = node(4, vec![gpu.key()]);
        let outputs = vec![scene.key(), camera.key(), gpu.key(), projection.key()];
        let topology = GraphTopology::try_new(
            [scene, camera, gpu, projection], outputs.clone(),
        ).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = EchoExecutor;
        for (generation, time) in [RationalTime::ZERO, RationalTime::from_seconds(1), RationalTime::ZERO].into_iter().enumerate() {
            let frame = graph.evaluate(&mut executor, time, FrameQuality::Export, Generation::new(generation as u64 + 1)).unwrap();
            for key in &outputs { assert!(frame.value(*key).is_some(), "missing exposed output on seek: {key:?}"); }
            if generation == 2 { assert!(frame.executed_nodes().is_empty()); }
        }
        assert_eq!(graph.stats().topology_compiles, 1);
    }

    #[test]
    fn a_stale_generation_cannot_publish() {
        let root = node(1, vec![]);
        let graph = GraphTopology::try_new([root.clone()], vec![root.key()]).unwrap();
        let mut compiled = CompiledGraph::with_topology(GraphRevision::new(1), graph);
        let mut executor = EchoExecutor;
        let old = compiled
            .evaluate(
                &mut executor,
                RationalTime::ZERO,
                FrameQuality::Preview { scale: 1 },
                Generation::new(1),
            )
            .unwrap();
        let current = compiled
            .evaluate(
                &mut executor,
                RationalTime::ZERO,
                FrameQuality::Preview { scale: 1 },
                Generation::new(2),
            )
            .unwrap();

        let old_submission = render_view(&old, ViewProjection::Camera, RenderTarget::new(1));
        let current_submission = render_view(&current, ViewProjection::Stage, RenderTarget::new(2));

        assert_eq!(compiled.publish(&old_submission), None);
        assert_eq!(
            compiled
                .publish(&current_submission)
                .map(|frame| frame.generation),
            Some(Generation::new(2))
        );
    }

    #[test]
    fn work_keys_reuse_static_nodes_and_split_time_and_quality_dependencies() {
        let static_node = node(1, vec![]);
        let mut timed_identity = NodeIdentity::new(NodeKind::Custom(2), vec![]);
        timed_identity.time_dependency = TimeDependency::Exact;
        let timed_node = GraphNode::new(timed_identity);
        let mut quality_identity = NodeIdentity::new(NodeKind::Custom(3), vec![]);
        quality_identity.quality_dependency = QualityDependency::Sensitive;
        let quality_node = GraphNode::new(quality_identity);
        let graph = GraphTopology::try_new(
            [
                static_node.clone(),
                timed_node.clone(),
                quality_node.clone(),
            ],
            vec![static_node.key(), timed_node.key(), quality_node.key()],
        )
        .unwrap();
        let mut compiled = CompiledGraph::with_topology(GraphRevision::new(1), graph);
        let mut executor = EchoExecutor;

        compiled
            .evaluate(
                &mut executor,
                RationalTime::ZERO,
                FrameQuality::Export,
                Generation::new(1),
            )
            .unwrap();
        let advanced = compiled
            .evaluate(
                &mut executor,
                RationalTime::from_seconds(1),
                FrameQuality::Export,
                Generation::new(2),
            )
            .unwrap();
        let preview = compiled
            .evaluate(
                &mut executor,
                RationalTime::from_seconds(1),
                FrameQuality::Preview { scale: 1 },
                Generation::new(3),
            )
            .unwrap();

        assert_eq!(advanced.executed_nodes(), &[timed_node.key()]);
        assert_eq!(
            advanced.reused_nodes(),
            &[static_node.key(), quality_node.key()]
        );
        assert_eq!(preview.executed_nodes(), &[quality_node.key()]);
        assert_eq!(
            preview.reused_nodes(),
            &[static_node.key(), timed_node.key()]
        );
        assert_eq!(compiled.stats().cancelled_generations, 0);
    }

    #[test]
    fn temporal_edges_reuse_the_same_source_time_and_keep_duplicate_inputs_positional() {
        struct TemporalExecutor { sources: Vec<RationalTime>, pairs: Vec<(RationalTime, RationalTime)> }
        impl NodeExecutor for TemporalExecutor {
            type Error = ();
            fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> {
                if node.identity().kind == NodeKind::Custom(10) {
                    self.sources.push(context.time);
                    return Ok(NodeValue::new(context.time));
                }
                let now = *inputs.at(0).and_then(|value| value.downcast_ref::<RationalTime>()).unwrap();
                let before = *inputs.at(1).and_then(|value| value.downcast_ref::<RationalTime>()).unwrap();
                self.pairs.push((now, before));
                Ok(NodeValue::new((now, before)))
            }
        }

        let mut source_identity = NodeIdentity::new(NodeKind::Custom(10), vec![]);
        source_identity.time_dependency = TimeDependency::Exact;
        let source = GraphNode::new(source_identity);
        let step = RationalTime::try_new(1, 30).unwrap();
        let mut consumer_identity = NodeIdentity::new(NodeKind::Custom(11), vec![source.key(), source.key()])
            .with_input_times(vec![InputTime::Same, InputTime::Offset { delta: RationalTime::try_new(-1, 30).unwrap(), clamp_to_zero: true }]);
        consumer_identity.time_dependency = TimeDependency::Exact;
        let consumer = GraphNode::new(consumer_identity);
        let graph = GraphTopology::try_new([source.clone(), consumer.clone()], vec![consumer.key()]).unwrap();
        let mut compiled = CompiledGraph::with_topology(GraphRevision::new(1), graph);
        let mut executor = TemporalExecutor { sources: Vec::new(), pairs: Vec::new() };

        compiled.evaluate(&mut executor, step, FrameQuality::Export, Generation::new(1)).unwrap();
        compiled.evaluate(&mut executor, step.try_add(step).unwrap(), FrameQuality::Export, Generation::new(2)).unwrap();

        assert_eq!(executor.pairs, vec![(step, RationalTime::ZERO), (step.try_add(step).unwrap(), step)]);
        assert_eq!(executor.sources, vec![step, RationalTime::ZERO, step.try_add(step).unwrap()]);
        assert!(compiled.stats().node_reuses >= 1, "the second frame must reuse source@previous-time");
    }
}
