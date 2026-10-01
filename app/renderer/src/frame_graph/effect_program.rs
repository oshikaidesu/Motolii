use std::collections::BTreeMap;

use crate::doc::eval::Value;
use crate::doc::store::{EffectScope, LayerId, PropertyId, StoreError, StoreView};
use crate::picture::resolved::ResolvedEffect;

use super::{EvaluationContext, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TimeDependency};

#[derive(Clone, Debug, PartialEq)] pub struct EffectValue(pub Option<ResolvedEffect>);
#[derive(Clone, Debug, PartialEq, Eq)] pub struct EffectBinding { pub layer: LayerId, pub effects: Vec<NodeKey> }

#[derive(Clone)] struct Recipe { plugin_id: String, enabled: Option<usize>, scope: Option<usize>, params: Vec<(String, usize)> }

#[derive(Debug)] pub enum EffectProgramError { Store(StoreError), InvalidInput(NodeKind) }
impl std::fmt::Display for EffectProgramError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }
impl std::error::Error for EffectProgramError {}
impl From<StoreError> for EffectProgramError { fn from(value: StoreError) -> Self { Self::Store(value) } }

pub struct EffectProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    bindings: BTreeMap<LayerId, EffectBinding>,
    placement: BTreeMap<NodeKey, crate::doc::store::kind::PlacementProgram>,
    sampling: BTreeMap<NodeKey, crate::doc::store::kind::SamplingProgram>,
    snap: BTreeMap<NodeKey, crate::doc::store::kind::SnapProgram>,
}

impl EffectProgram {
    pub fn compile(view: &StoreView<'_>, properties: &PropertyProgram) -> Result<Self, EffectProgramError> {
        let mut nodes = BTreeMap::new(); let mut recipes = BTreeMap::new(); let mut bindings = BTreeMap::new(); let mut placement = BTreeMap::new(); let mut sampling = BTreeMap::new(); let mut snap = BTreeMap::new();
        for layer in view.layers() {
            let property_names = view.properties(layer);
            let mut keys = Vec::new();
            for effect in view.effects(layer)? {
                let mut inputs = Vec::new();
                let enabled = properties.node_for(layer, &PropertyId::effect_enabled(effect.id)).map(|key| { let at = inputs.len(); inputs.push(key); at });
                let scope = properties.node_for(layer, &PropertyId::effect_scope(effect.id)).map(|key| { let at = inputs.len(); inputs.push(key); at });
                let prefix = format!("effect.{}.param.", effect.id.0);
                let mut params = Vec::new();
                for property in &property_names {
                    let Some(name) = property.name().strip_prefix(&prefix) else { continue };
                    if let Some(key) = properties.node_for(layer, property) { let at = inputs.len(); inputs.push(key); params.push((name.to_owned(), at)); }
                }
                let mut identity = NodeIdentity::new(NodeKind::Effect, inputs);
                identity.parameters = effect.plugin_id.as_bytes().iter().copied().chain([0]).chain(params.iter().flat_map(|(name, _)| name.as_bytes().iter().copied().chain([0]))).collect();
                identity.time_dependency = TimeDependency::Exact;
                let node = GraphNode::new(identity);
                let placement_program = view.placement_program(&effect.plugin_id);
                let sampling_program = view.sampling_program(&effect.plugin_id);
                let snap_program = view.snap_program(&effect.plugin_id);
                recipes.entry(node.key()).or_insert(Recipe { plugin_id: effect.plugin_id, enabled, scope, params });
                if let Some(program) = placement_program {
                    placement.entry(node.key()).or_insert(program);
                }
                if let Some(program) = sampling_program {
                    sampling.entry(node.key()).or_insert(program);
                }
                if let Some(program) = snap_program {
                    snap.entry(node.key()).or_insert(program);
                }
                nodes.entry(node.key()).or_insert(node.clone());
                keys.push(node.key());
            }
            if !keys.is_empty() { bindings.insert(layer, EffectBinding { layer, effects: keys }); }
        }
        Ok(Self { nodes, recipes, bindings, placement, sampling, snap })
    }
    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn binding(&self, layer: LayerId) -> Option<&EffectBinding> { self.bindings.get(&layer) }
    pub fn is_placement(&self, key: NodeKey) -> bool { self.placement.contains_key(&key) }
    pub fn placement_program(&self, key: NodeKey) -> Option<crate::doc::store::kind::PlacementProgram> { self.placement.get(&key).copied() }
    pub fn sampling_program(&self, key: NodeKey) -> Option<crate::doc::store::kind::SamplingProgram> { self.sampling.get(&key).copied() }
    pub fn snap_program(&self, key: NodeKey) -> Option<crate::doc::store::kind::SnapProgram> { self.snap.get(&key).cloned() }
    pub fn plugin_id(&self, key: NodeKey) -> Option<&str> { self.recipes.get(&key).map(|recipe| recipe.plugin_id.as_str()) }
    pub fn execute(&self, node: &GraphNode, inputs: &NodeInputs, _context: &EvaluationContext) -> Option<Result<NodeValue, EffectProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some((|| {
            let enabled = recipe.enabled.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()).map_or(true, |value| matches!(value, Value::Bool(true)));
            if !enabled { return Ok(NodeValue::new(EffectValue(None))); }
            let scope = match recipe.scope.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) { Some(Value::Enum(value)) => EffectScope::from_enum_value(*value).ok_or(EffectProgramError::InvalidInput(node.identity().kind))?, None => EffectScope::Each, _ => return Err(EffectProgramError::InvalidInput(node.identity().kind)) };
            let params = recipe.params.iter().map(|(name, index)| inputs.at(*index).and_then(|value| value.downcast_ref::<Value>()).cloned().map(|value| (name.clone(), value)).ok_or(EffectProgramError::InvalidInput(node.identity().kind))).collect::<Result<_, _>>()?;
            Ok(NodeValue::new(EffectValue(Some(ResolvedEffect { plugin_id: recipe.plugin_id.clone(), params, scope }))))
        })())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{EffectInstance, LayerMeta, LayerSource, LayerTiming};
    use crate::frame_graph::{CompiledGraph, FrameQuality, Generation, GraphRevision, GraphTopology, NodeExecutor, SceneProgram, SceneProgramError, SceneValue};
    use motolii_edit::{Document, Intent};

    struct Executor<'a>(&'a SceneProgram);
    impl NodeExecutor for Executor<'_> {
        type Error = SceneProgramError;
        fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> { self.0.execute(node, &inputs, &context) }
    }

    #[test]
    fn effect_params_and_scope_are_evaluated_once_into_the_scene() {
        let mut doc = Document::new(); let layer = LayerId(1); let id = crate::doc::store::EffectId(7);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Null, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetEffects { layer, effects: vec![EffectInstance { id, plugin_id: "test.effect".into() }] },
            Intent::SetConstant { layer, property: PropertyId::effect_param(id, "amount").unwrap(), value: Value::F64(0.5) },
            Intent::SetConstant { layer, property: PropertyId::effect_scope(id), value: Value::Enum(EffectScope::Whole.enum_value()) },
        ]).unwrap();
        let program = SceneProgram::compile(&doc.view()).unwrap();
        let root = program.scene().scene;
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology); let mut executor = Executor(&program);
        let frame = graph.evaluate(&mut executor, crate::doc::core::RationalTime::ZERO, FrameQuality::Export, Generation::new(1)).unwrap();
        let scene = frame.value(root).and_then(|value| value.downcast_ref::<SceneValue>()).unwrap();
        assert_eq!(scene.layers[0].effects[0].params, vec![("amount".into(), Value::F64(0.5))]);
        assert!(scene.layers[0].after_effects.is_empty(), "scope is only a Group distribution rule");
    }
}
