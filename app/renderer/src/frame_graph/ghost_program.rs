use std::collections::BTreeMap;

use crate::doc::core::RationalTime;
use crate::doc::store::{LayerId, LayerSource, StoreError, StoreView};

use super::group_composite_program::SceneFragmentValue;
use super::scene_program::{SceneContentValue, SceneContributionValue};
use super::{EvaluationContext, GraphNode, InputTime, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, TimeDependency};

#[derive(Clone)]
struct Recipe;

#[derive(Debug)]
pub(super) enum GhostProgramError {
    Store(StoreError),
    InvalidInput(NodeKind),
}

impl std::fmt::Display for GhostProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for GhostProgramError {}
impl From<StoreError> for GhostProgramError {
    fn from(value: StoreError) -> Self { Self::Store(value) }
}

pub(super) struct GhostProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    bindings: BTreeMap<LayerId, NodeKey>,
}

impl GhostProgram {
    pub(super) fn compile(
        view: &StoreView<'_>,
        sources: &BTreeMap<LayerId, NodeKey>,
    ) -> Result<Self, GhostProgramError> {
        let fps = view.composition()?.map(|composition| composition.fps);
        let mut nodes = BTreeMap::new();
        let mut recipes = BTreeMap::new();
        let mut bindings = BTreeMap::new();

        for layer in view.layers() {
            let Some(meta) = view.meta(layer)? else { continue };
            if meta.source == LayerSource::Group { continue; }
            let attrs = view.attrs(layer)?.unwrap_or_default();
            if attrs.environment { continue; }
            let (Some(delay), Some(fps), Some(source)) = (attrs.ghost, fps, sources.get(&layer).copied()) else { continue };
            let delta = RationalTime::try_from_frame(-delay, fps).map_err(|error| StoreError::Property(error.to_string()))?;
            let mut identity = NodeIdentity::new(NodeKind::TemporalCopy, vec![source, source]).with_input_times(vec![
                InputTime::Offset { delta, clamp_to_zero: false },
                InputTime::Same,
            ]);
            identity.parameters.extend_from_slice(&layer.0.to_be_bytes());
            identity.parameters.extend_from_slice(&delay.to_be_bytes());
            identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity);
            let key = node.key();
            nodes.entry(key).or_insert(node);
            recipes.entry(key).or_insert(Recipe);
            bindings.insert(layer, key);
        }

        Ok(Self { nodes, recipes, bindings })
    }

    pub(super) fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub(super) fn binding(&self, layer: LayerId) -> Option<NodeKey> { self.bindings.get(&layer).copied() }

    pub(super) fn execute(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        _context: &EvaluationContext,
    ) -> Option<Result<NodeValue, GhostProgramError>> {
        self.recipes.get(&node.key())?;
        Some((|| {
            let mut ghost = contributions(inputs.at(0).ok_or(GhostProgramError::InvalidInput(node.identity().kind))?)
                .ok_or(GhostProgramError::InvalidInput(node.identity().kind))?;
            let current = contributions(inputs.at(1).ok_or(GhostProgramError::InvalidInput(node.identity().kind))?)
                .ok_or(GhostProgramError::InvalidInput(node.identity().kind))?;

            ghost.retain(|contribution| contribution.layer.is_some());
            for contribution in &mut ghost {
                mark_ghost(contribution);
            }
            ghost.extend(current);
            Ok(NodeValue::new(SceneFragmentValue { contributions: ghost }))
        })())
    }
}

fn contributions(value: &NodeValue) -> Option<Vec<SceneContributionValue>> {
    if let Some(contribution) = value.downcast_ref::<SceneContributionValue>() {
        Some(vec![contribution.clone()])
    } else {
        value.downcast_ref::<SceneFragmentValue>().map(|fragment| fragment.contributions.clone())
    }
}

fn mark_ghost(contribution: &mut SceneContributionValue) {
    let Some(layer) = contribution.layer.as_mut() else { return };
    layer.ghost = true;
    layer.freeze_eligible = false;
    if let SceneContentValue::Plate(plate) = &mut layer.content {
        for member in &mut plate.members {
            mark_ghost(member);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::eval::{Interp, Keyframe, KeyframeTrack, Value};
    use crate::doc::store::{Composition, Fps, LayerAttrsPatch, LayerMeta, LayerTiming, PropertyId, property};
    use crate::frame_graph::{CompiledGraph, FrameQuality, Generation, GraphRevision, GraphTopology, NodeExecutor, SceneProgram, SceneProgramError, SceneValue};
    use motolii_edit::{Document, Intent};

    struct Executor<'a>(&'a SceneProgram);
    impl NodeExecutor for Executor<'_> {
        type Error = SceneProgramError;
        fn dynamic_inputs(&mut self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Result<Vec<super::super::DynamicInput>, Self::Error> {
            self.0.dynamic_inputs(node, inputs, context)
        }
        fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> {
            self.0.execute(node, &inputs, &context)
        }
    }

    #[test]
    fn ghost_reads_the_same_contribution_at_an_offset_time_before_current() {
        let fps = Fps::try_new(10, 1).unwrap();
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: 100, height: 100, fps, duration_frames: 40, background: [0.0; 4] })).unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Null, order: 0, timing: LayerTiming::place(0, None, 30) } },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { ghost: Some(Some(5)), ..Default::default() } },
        ]).unwrap();
        let track = KeyframeTrack::try_from_keys(vec![
            Keyframe { t: RationalTime::ZERO, value: Value::Vec2([0.0, 0.0]), interp: Interp::Linear, spatial: None },
            Keyframe { t: RationalTime::try_from_frame(10, fps).unwrap(), value: Value::Vec2([100.0, 0.0]), interp: Interp::Linear, spatial: None },
        ]).unwrap();
        doc.apply(Intent::SetTrack { layer, property: PropertyId::new(property::POSITION).unwrap(), track }).unwrap();

        let program = SceneProgram::compile(&doc.view()).unwrap();
        let root = program.scene().scene;
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let at = RationalTime::try_from_frame(8, fps).unwrap();
        let frame = graph.evaluate(&mut executor, at, FrameQuality::Export, Generation::new(1)).unwrap();
        let scene = frame.value(root).and_then(|value| value.downcast_ref::<SceneValue>()).unwrap();
        assert_eq!(scene.layers.len(), 2);
        assert!(scene.layers[0].ghost && !scene.layers[1].ghost);
        assert_eq!(scene.layers.iter().map(|layer| layer.transform.affine.translation.x.round()).collect::<Vec<_>>(), [30.0, 80.0]);
    }
}
