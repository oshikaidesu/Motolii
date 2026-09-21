use std::collections::BTreeMap;

use crate::doc::core::Fps;
use crate::doc::eval::Value;
use crate::doc::store::{LayerId, LayerTiming, PropertyId, StoreError, StoreView};

use super::{EvaluationContext, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TimeDependency};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisibilityValue {
    /// Hidden and timing decide whether the layer has a contribution at this time.
    pub active: bool,
    /// Solo deliberately survives hidden/out-of-range state. This matches the
    /// legacy resolver: any_solo is decided before per-layer visibility.
    pub solo: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisibilityBinding {
    pub layer: LayerId,
    pub node: NodeKey,
}

#[derive(Clone)]
struct Recipe {
    hidden: Option<usize>,
    solo: Option<usize>,
    static_hidden: bool,
    static_solo: bool,
    timing: LayerTiming,
    fps: Option<Fps>,
}

#[derive(Debug)]
pub enum VisibilityProgramError {
    Store(StoreError),
    InvalidInput(NodeKind),
}

impl std::fmt::Display for VisibilityProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for VisibilityProgramError {}
impl From<StoreError> for VisibilityProgramError {
    fn from(value: StoreError) -> Self {
        Self::Store(value)
    }
}

/// Exact-time layer participation. Compilation owns all document reads;
/// evaluation sees only property nodes, authored timing and the comp fps.
pub struct VisibilityProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    bindings: BTreeMap<LayerId, VisibilityBinding>,
}

impl VisibilityProgram {
    pub fn compile(
        view: &StoreView<'_>,
        properties: &PropertyProgram,
    ) -> Result<Self, VisibilityProgramError> {
        let fps = view.composition()?.map(|composition| composition.fps);
        let mut nodes = BTreeMap::new();
        let mut recipes = BTreeMap::new();
        let mut bindings = BTreeMap::new();

        for layer in view.layers() {
            let Some(meta) = view.meta(layer)? else { continue };
            let attrs = view.attrs(layer)?.unwrap_or_default();
            let mut inputs = Vec::new();
            let hidden = properties.node_for(layer, &PropertyId::hidden()).map(|key| {
                let at = inputs.len();
                inputs.push(key);
                at
            });
            let solo = properties.node_for(layer, &PropertyId::solo()).map(|key| {
                let at = inputs.len();
                inputs.push(key);
                at
            });

            let mut identity = NodeIdentity::new(NodeKind::Visibility, inputs);
            identity.parameters.push(u8::from(attrs.hidden));
            identity.parameters.push(u8::from(attrs.solo));
            identity.parameters.extend_from_slice(&meta.timing.start.to_be_bytes());
            identity.parameters.extend_from_slice(&meta.timing.duration.to_be_bytes());
            identity.parameters.extend_from_slice(&meta.timing.source_in.to_be_bytes());
            identity.parameters.extend_from_slice(&meta.timing.speed.num().to_be_bytes());
            identity.parameters.extend_from_slice(&meta.timing.speed.den().to_be_bytes());
            if let Some(fps) = fps {
                identity.parameters.extend_from_slice(&fps.num().to_be_bytes());
                identity.parameters.extend_from_slice(&fps.den().to_be_bytes());
            }
            identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity);
            let key = node.key();
            nodes.entry(key).or_insert(node);
            recipes.entry(key).or_insert(Recipe {
                hidden,
                solo,
                static_hidden: attrs.hidden,
                static_solo: attrs.solo,
                timing: meta.timing,
                fps,
            });
            bindings.insert(layer, VisibilityBinding { layer, node: key });
        }

        Ok(Self { nodes, recipes, bindings })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ {
        self.nodes.values().cloned()
    }

    pub fn binding(&self, layer: LayerId) -> Option<VisibilityBinding> {
        self.bindings.get(&layer).copied()
    }

    pub fn execute(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        context: &EvaluationContext,
    ) -> Option<Result<NodeValue, VisibilityProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some((|| {
            let read_bool = |index: Option<usize>, default: bool| -> Result<bool, VisibilityProgramError> {
                match index.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) {
                    Some(Value::Bool(value)) => Ok(*value),
                    Some(_) => Err(VisibilityProgramError::InvalidInput(node.identity().kind)),
                    None => Ok(default),
                }
            };
            let hidden = read_bool(recipe.hidden, recipe.static_hidden)?;
            let solo = read_bool(recipe.solo, recipe.static_solo)?;
            let in_range = match recipe.fps {
                Some(fps) => context
                    .time
                    .try_to_frame_floor(fps)
                    .map(|frame| recipe.timing.covers(frame))
                    .unwrap_or(false),
                None => false,
            };
            Ok(NodeValue::new(VisibilityValue {
                active: in_range && !hidden,
                solo,
            }))
        })())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{Composition, Fps, LayerAttrsPatch, LayerMeta, LayerSource, LayerTiming};
    use crate::frame_graph::{CompiledGraph, FrameQuality, Generation, GraphRevision, GraphTopology, NodeExecutor};
    use motolii_edit::{Document, Intent};

    struct Executor<'a>(&'a VisibilityProgram);
    impl NodeExecutor for Executor<'_> {
        type Error = VisibilityProgramError;
        fn execute(
            &mut self,
            node: &GraphNode,
            inputs: NodeInputs,
            context: EvaluationContext,
        ) -> Result<NodeValue, Self::Error> {
            self.0.execute(node, &inputs, &context)
                .unwrap_or(Err(VisibilityProgramError::InvalidInput(node.identity().kind)))
        }
    }

    #[test]
    fn hidden_and_trim_control_activity_but_do_not_erase_solo() {
        let mut doc = Document::new();
        let fps = Fps::try_new(30, 1).unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::SetComposition(Composition {
                width: 640,
                height: 360,
                fps,
                duration_frames: 120,
                background: [0.0; 4],
            }),
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Null,
                    order: 0,
                    timing: LayerTiming { start: 10, duration: 10, source_in: 0, speed: crate::doc::store::Speed::NORMAL },
                },
            },
            Intent::SetAttrs {
                layer,
                patch: LayerAttrsPatch { hidden: Some(true), solo: Some(true), ..Default::default() },
            },
        ]).unwrap();

        let properties = PropertyProgram::compile(&doc.view()).unwrap();
        let program = VisibilityProgram::compile(&doc.view(), &properties).unwrap();
        let root = program.binding(layer).unwrap().node;
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);

        for (frame, expected_active) in [(9, false), (10, false), (19, false), (20, false)] {
            let evaluated = graph.evaluate(
                &mut executor,
                crate::doc::core::RationalTime::try_from_frame(frame, fps).unwrap(),
                FrameQuality::Export,
                Generation::new((frame + 1) as u64),
            ).unwrap();
            let value = evaluated.value(root).and_then(|value| value.downcast_ref::<VisibilityValue>()).unwrap();
            assert_eq!(value.active, expected_active);
            assert!(value.solo, "solo participates in global solo even while hidden/out of range");
        }
    }
}
