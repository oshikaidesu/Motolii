use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::doc::core::RationalTime;
use crate::doc::eval::{KeyframeTrack, Value};
use crate::doc::store::{slot::translate_link, LayerId, PropertyBase, PropertyId, PropertyLink, PropertySource, SlotId, StoreError, StoreView};

use super::{EvaluationContext, GraphNode, InputTime, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, TimeDependency};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PropertyBinding {
    pub layer: LayerId,
    pub property: PropertyId,
    pub node: NodeKey,
}

#[derive(Clone)]
enum Recipe {
    Constant(Value),
    Track(KeyframeTrack),
    Link(PropertyLink),
    Sum,
}

#[derive(Debug)]
pub enum PropertyProgramError {
    Store(StoreError),
    Encode(serde_json::Error),
    Cycle { layer: LayerId, property: PropertyId },
    MissingSlot(SlotId),
    InvalidInput(NodeKind),
}

impl std::fmt::Display for PropertyProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for PropertyProgramError {}
impl From<StoreError> for PropertyProgramError { fn from(value: StoreError) -> Self { Self::Store(value) } }
impl From<serde_json::Error> for PropertyProgramError { fn from(value: serde_json::Error) -> Self { Self::Encode(value) } }

/// Immutable authored value graph. It owns cloned recipes; evaluation never
/// reads StoreView and layer ids live only in the compiler binding table.
pub struct PropertyProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    bindings: BTreeMap<(LayerId, PropertyId), NodeKey>,
    slots: HashMap<SlotId, KeyframeTrack>,
}

impl PropertyProgram {
    pub fn compile(view: &StoreView<'_>) -> Result<Self, PropertyProgramError> {
        let mut program = Self { nodes: BTreeMap::new(), recipes: BTreeMap::new(), bindings: BTreeMap::new(), slots: view.slots()?.into_iter().map(|slot| (slot.id, slot.track)).collect() };
        let mut visiting = BTreeSet::new();
        for layer in view.layers() {
            for property in view.properties(layer) {
                program.compile_property(view, layer, property, &mut visiting)?;
            }
        }
        Ok(program)
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn node_for(&self, layer: LayerId, property: &PropertyId) -> Option<NodeKey> { self.bindings.get(&(layer, property.clone())).copied() }
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = PropertyBinding> + '_ {
        self.bindings.iter().map(|((layer, property), node)| PropertyBinding { layer: *layer, property: property.clone(), node: *node })
    }

    pub fn execute(&self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Option<Result<NodeValue, PropertyProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some(match recipe {
            Recipe::Constant(value) => Ok(NodeValue::new(value.clone())),
            Recipe::Track(track) => Ok(NodeValue::new(track.eval(context.time))),
            Recipe::Link(link) => {
                let source = inputs.at(0).and_then(|value| value.downcast_ref::<Value>()).cloned().ok_or(PropertyProgramError::InvalidInput(node.identity().kind));
                source.and_then(|value| translate_link(&link.plugin_id, &link.params, value).ok_or(PropertyProgramError::InvalidInput(node.identity().kind))).map(NodeValue::new)
            }
            Recipe::Sum => {
                let mut values = inputs.iter().filter_map(|(_, value)| value.downcast_ref::<Value>().cloned());
                let Some(first) = values.next() else { return Some(Err(PropertyProgramError::InvalidInput(node.identity().kind))); };
                Ok(NodeValue::new(values.fold(first, |current, next| current.add(&next).unwrap_or(current))))
            }
        })
    }

    fn compile_property(&mut self, view: &StoreView<'_>, layer: LayerId, property: PropertyId, visiting: &mut BTreeSet<(LayerId, PropertyId)>) -> Result<Option<NodeKey>, PropertyProgramError> {
        let binding = (layer, property.clone());
        if let Some(key) = self.bindings.get(&binding) { return Ok(Some(*key)); }
        if !visiting.insert(binding.clone()) { return Err(PropertyProgramError::Cycle { layer, property }); }
        let source = view.property_source(layer, &property)?;
        let key = match source {
            Some(source) => self.compile_source(view, source, visiting)?,
            None => None,
        };
        visiting.remove(&binding);
        if let Some(key) = key { self.bindings.insert(binding, key); }
        Ok(key)
    }

    fn compile_source(&mut self, view: &StoreView<'_>, source: PropertySource, visiting: &mut BTreeSet<(LayerId, PropertyId)>) -> Result<Option<NodeKey>, PropertyProgramError> {
        let mut values = Vec::new();
        if let Some(base) = source.base {
            let (kind, recipe, parameters, timed) = match base {
                PropertyBase::Constant(value) => (NodeKind::PropertyConstant, Recipe::Constant(value.clone()), serde_json::to_vec(&value)?, false),
                PropertyBase::Track(track) => (NodeKind::PropertyTrack, Recipe::Track(track.clone()), serde_json::to_vec(&track)?, true),
                PropertyBase::Slot(slot) => {
                    let track = self.slots.get(&slot).cloned().ok_or(PropertyProgramError::MissingSlot(slot))?;
                    (NodeKind::PropertyTrack, Recipe::Track(track.clone()), serde_json::to_vec(&track)?, true)
                }
            };
            values.push(self.intern(kind, vec![], vec![], parameters, timed, recipe));
        }
        for link in source.modulators {
            let Some(source_key) = self.compile_property(view, link.source_layer, link.source_property.clone(), visiting)? else { continue };
            let input_time = if link.time_offset == RationalTime::ZERO { InputTime::Same } else { InputTime::Offset { delta: link.time_offset, clamp_to_zero: false } };
            let parameters = serde_json::to_vec(&link)?;
            values.push(self.intern(NodeKind::PropertyLink, vec![source_key], vec![input_time], parameters, true, Recipe::Link(link)));
        }
        Ok(match values.len() {
            0 => None,
            1 => Some(values[0]),
            _ => Some(self.intern(NodeKind::PropertySum, values.clone(), vec![InputTime::Same; values.len()], vec![], true, Recipe::Sum)),
        })
    }

    fn intern(&mut self, kind: NodeKind, inputs: Vec<NodeKey>, input_times: Vec<InputTime>, parameters: Vec<u8>, timed: bool, recipe: Recipe) -> NodeKey {
        let mut identity = NodeIdentity::new(kind, inputs).with_input_times(input_times);
        identity.parameters = parameters;
        identity.time_dependency = if timed { TimeDependency::Exact } else { TimeDependency::Static };
        let node = GraphNode::new(identity);
        let key = node.key();
        self.nodes.entry(key).or_insert(node);
        self.recipes.entry(key).or_insert(recipe);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::eval::{Interp, Keyframe};
    use crate::doc::store::{LayerMeta, LayerSource, LayerTiming};
    use crate::frame_graph::{CompiledGraph, FrameQuality, Generation, GraphRevision, GraphTopology, NodeExecutor};
    use motolii_edit::{Document, Intent};

    struct Executor<'a>(&'a PropertyProgram);
    impl NodeExecutor for Executor<'_> {
        type Error = PropertyProgramError;
        fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> {
            self.0.execute(node, &inputs, &context).unwrap_or(Err(PropertyProgramError::InvalidInput(node.identity().kind)))
        }
    }

    #[test]
    fn equal_values_share_nodes_and_links_read_their_exact_source_time() {
        let mut doc = Document::new();
        let (a, b, c) = (LayerId(1), LayerId(2), LayerId(3));
        let constant = PropertyId::new("Shared Constant").unwrap();
        let source = PropertyId::new("Source Value").unwrap();
        let linked = PropertyId::new("Linked Value").unwrap();
        let track = KeyframeTrack::try_from_keys(vec![
            Keyframe { t: RationalTime::ZERO, value: Value::F64(1.0), interp: Interp::Linear, spatial: None },
            Keyframe { t: RationalTime::from_seconds(1), value: Value::F64(3.0), interp: Interp::Linear, spatial: None },
        ]).unwrap();
        for (order, layer) in [a, b, c].into_iter().enumerate() {
            doc.apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Null, order: order as i16, timing: LayerTiming::place(0, None, 90) } },
            ]).unwrap();
        }
        doc.apply_all([
            Intent::SetConstant { layer: a, property: constant.clone(), value: Value::F64(7.0) },
            Intent::SetConstant { layer: b, property: constant.clone(), value: Value::F64(7.0) },
            Intent::SetTrack { layer: a, property: source.clone(), track },
            Intent::SetPropertyLink { layer: c, property: linked.clone(), link: PropertyLink { source_layer: a, source_property: source.clone(), time_offset: RationalTime::from_seconds(-1), plugin_id: "motolii.link.identity".into(), params: Vec::new() } },
        ]).unwrap();

        let program = PropertyProgram::compile(&doc.view()).unwrap();
        assert_eq!(program.node_for(a, &constant), program.node_for(b, &constant));
        let root = program.node_for(c, &linked).unwrap();
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let frame = graph.evaluate(&mut executor, RationalTime::from_seconds(1), FrameQuality::Export, Generation::new(1)).unwrap();
        assert_eq!(frame.value(root).and_then(|value| value.downcast_ref::<Value>()), Some(&Value::F64(1.0)));
    }
}
