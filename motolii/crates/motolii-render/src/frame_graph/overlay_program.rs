use std::collections::BTreeMap;

use crate::doc::core::{Fps, RationalTime};
use crate::doc::store::analysis::BlobMark;
use crate::doc::store::{LayerId, StoreError, StoreView};
use crate::picture::resolved::ResolvedEffect;
use crate::render::media::blob::BlobTracker;

use super::{
    DynamicInput, EffectProgram, EffectValue, EvaluationContext, GraphNode, NodeIdentity, NodeInputs,
    NodeKey, NodeKind, NodeValue, SceneValue, SolverPlanValue, TimeDependency,
};

#[derive(Clone, Debug, PartialEq)]
pub struct OverlayAnalysisValue {
    pub layer: LayerId,
    pub marks: Vec<BlobMark>,
    pub mask: Option<(Vec<u8>, u32, u32)>,
    pub params: Vec<(String, crate::doc::eval::Value)>,
    pub depths: Option<Vec<f32>>,
    pub pushes: Vec<[f32; 2]>,
    pub physics: bool,
    pub tracker: BlobTracker,
    pub previous: Option<(Vec<u8>, u32, u32)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OverlaySetValue {
    pub layers: BTreeMap<LayerId, OverlayAnalysisValue>,
}

#[derive(Clone)]
enum Recipe {
    Overlay { layer: LayerId, order: i16, parent: Option<LayerId>, start: i64, fps: Fps },
    Set { layers: Vec<LayerId> },
}

#[derive(Debug)]
pub enum OverlayProgramError {
    Store(StoreError),
    InvalidInput(NodeKind),
}

impl std::fmt::Display for OverlayProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for OverlayProgramError {}
impl From<StoreError> for OverlayProgramError {
    fn from(value: StoreError) -> Self { Self::Store(value) }
}

pub struct OverlayProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    bindings: BTreeMap<LayerId, NodeKey>,
    output: NodeKey,
}

impl OverlayProgram {
    pub fn compile(
        view: &StoreView<'_>,
        effects: &EffectProgram,
        scene: NodeKey,
        solver: NodeKey,
        camera: NodeKey,
    ) -> Result<Self, OverlayProgramError> {
        let fps = view.composition()?.map(|composition| composition.fps)
            .unwrap_or(Fps::try_new(30, 1).expect("30fps"));
        let mut nodes = BTreeMap::new();
        let mut recipes = BTreeMap::new();
        let mut bindings = BTreeMap::new();

        for layer in view.layers() {
            let Some(meta) = view.meta(layer)? else { continue };
            let Some(effect) = effects.binding(layer)
                .and_then(|binding| binding.effects.iter().copied().find(|key| {
                    effects.plugin_id(*key).is_some_and(crate::extensions::overlay::is_track_overlay)
                })) else { continue };
            let attrs = view.attrs(layer)?.unwrap_or_default();
            let mut identity = NodeIdentity::new(NodeKind::AnalysisOverlay, vec![scene, effect, solver, camera]);
            identity.parameters.extend_from_slice(&layer.0.to_be_bytes());
            identity.parameters.extend_from_slice(&meta.order.to_be_bytes());
            identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity);
            let key = node.key();
            nodes.insert(key, node);
            recipes.insert(key, Recipe::Overlay {
                layer,
                order: meta.order,
                parent: attrs.parent,
                start: meta.timing.start,
                fps,
            });
            bindings.insert(layer, key);
        }

        let layers: Vec<_> = bindings.keys().copied().collect();
        let inputs: Vec<_> = layers.iter().filter_map(|layer| bindings.get(layer).copied()).collect();
        let mut identity = NodeIdentity::new(NodeKind::OverlaySet, inputs);
        identity.time_dependency = TimeDependency::Exact;
        let node = GraphNode::new(identity);
        let output = node.key();
        nodes.insert(output, node);
        recipes.insert(output, Recipe::Set { layers });

        Ok(Self { nodes, recipes, bindings, output })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn output(&self) -> NodeKey { self.output }

    pub fn dynamic_inputs(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        context: &EvaluationContext,
    ) -> Option<Result<Vec<DynamicInput>, OverlayProgramError>> {
        let Recipe::Overlay { start, fps, .. } = self.recipes.get(&node.key())? else {
            return Some(Ok(Vec::new()));
        };
        let effect = inputs.at(1)
            .and_then(|value| value.downcast_ref::<EffectValue>())
            .and_then(|value| value.0.as_ref());
        let Some(effect) = effect else { return Some(Ok(Vec::new())); };
        let params = crate::extensions::overlay::with_defaults(&effect.plugin_id, &effect.params);
        let method = crate::extensions::overlay::number_of(&params, "method").round() as i64;
        let sequential = crate::extensions::overlay::switch_of(&params, "keep_ids") || method == 0;
        if !sequential || method == 2 {
            return Some(Ok(Vec::new()));
        }
        let frame = context.time.try_to_frame_round(*fps).unwrap_or(*start);
        if frame <= *start {
            return Some(Ok(Vec::new()));
        }
        let previous = RationalTime::try_from_frame(frame - 1, *fps)
            .map_err(|error| StoreError::Property(error.to_string()));
        Some(previous.map(|time| vec![DynamicInput { node: node.key(), time }]).map_err(Into::into))
    }

    pub fn execute(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        _context: &EvaluationContext,
    ) -> Option<Result<NodeValue, OverlayProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        match recipe {
            Recipe::Overlay { .. } => None,
            Recipe::Set { layers } => Some((|| {
                let mut out = BTreeMap::new();
                for (index, layer) in layers.iter().enumerate() {
                    let value = inputs.at(index)
                        .and_then(|value| value.downcast_ref::<OverlayAnalysisValue>())
                        .cloned()
                        .ok_or(OverlayProgramError::InvalidInput(node.identity().kind))?;
                    out.insert(*layer, value);
                }
                Ok(NodeValue::new(OverlaySetValue { layers: out }))
            })()),
        }
    }

    pub fn recipe(&self, node: NodeKey) -> Option<(LayerId, i16, Option<LayerId>)> {
        match self.recipes.get(&node)? {
            Recipe::Overlay { layer, order, parent, .. } => Some((*layer, *order, *parent)),
            Recipe::Set { .. } => None,
        }
    }
}
