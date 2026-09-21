use std::collections::{BTreeMap, BTreeSet};

use crate::doc::store::StoreView;

use super::{ContentProgram, ContentProgramError, EvaluationContext, GraphNode, NodeInputs, NodeKey, NodeValue, PropertyProgram, PropertyProgramError, TransformProgram, TransformProgramError};

#[derive(Debug)]
pub enum SceneProgramError {
    Property(PropertyProgramError),
    Content(ContentProgramError),
    Transform(TransformProgramError),
    Unsupported(super::NodeKind),
}
impl std::fmt::Display for SceneProgramError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }
impl std::error::Error for SceneProgramError {}
impl From<PropertyProgramError> for SceneProgramError { fn from(value: PropertyProgramError) -> Self { Self::Property(value) } }
impl From<ContentProgramError> for SceneProgramError { fn from(value: ContentProgramError) -> Self { Self::Content(value) } }
impl From<TransformProgramError> for SceneProgramError { fn from(value: TransformProgramError) -> Self { Self::Transform(value) } }

/// One immutable revision program. Compiler bindings are the only place that
/// remembers layer ids; runtime execution follows content-addressed edges.
pub struct SceneProgram {
    properties: PropertyProgram,
    content: ContentProgram,
    transforms: TransformProgram,
    nodes: BTreeMap<NodeKey, GraphNode>,
    roots: BTreeSet<NodeKey>,
}

impl SceneProgram {
    pub fn compile(view: &StoreView<'_>) -> Result<Self, SceneProgramError> {
        let properties = PropertyProgram::compile(view)?;
        let content = ContentProgram::compile(view)?;
        let transforms = TransformProgram::compile(view, &properties)?;
        let mut nodes = BTreeMap::new();
        for node in properties.nodes().chain(content.nodes()).chain(transforms.nodes()) { nodes.insert(node.key(), node); }
        let roots = properties.bindings().map(|binding| binding.node)
            .chain(content.bindings().flat_map(|binding| [binding.content, binding.extent, binding.material]).flatten())
            .chain(transforms.bindings().map(|binding| binding.world)).collect();
        Ok(Self { properties, content, transforms, nodes, roots })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn roots(&self) -> impl ExactSizeIterator<Item = NodeKey> + '_ { self.roots.iter().copied() }
    pub fn properties(&self) -> &PropertyProgram { &self.properties }
    pub fn content(&self) -> &ContentProgram { &self.content }
    pub fn transforms(&self) -> &TransformProgram { &self.transforms }

    pub fn execute(&self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Result<NodeValue, SceneProgramError> {
        if let Some(value) = self.properties.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.content.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.transforms.execute(node, inputs, context) { return value.map_err(Into::into); }
        Err(SceneProgramError::Unsupported(node.identity().kind))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::eval::Value;
    use crate::doc::store::{LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId};
    use crate::frame_graph::{CompiledGraph, FrameQuality, Generation, GraphRevision, GraphTopology, NodeExecutor};
    use motolii_edit::{Document, Intent};

    struct Executor<'a>(&'a SceneProgram);
    impl NodeExecutor for Executor<'_> {
        type Error = SceneProgramError;
        fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> { self.0.execute(node, &inputs, &context) }
    }

    #[test]
    fn one_revision_program_evaluates_authored_values_and_shared_content_without_store_reads() {
        let mut doc = Document::new();
        let cube = LayerId(1);
        let opacity = PropertyId::new("Opacity").unwrap();
        doc.apply_all([
            Intent::AddLayer(cube),
            Intent::SetMeta { layer: cube, meta: LayerMeta { source: LayerSource::File { path: "/builtins/cube-v1.obj".into(), fingerprint: Some("cube-v1".into()) }, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetConstant { layer: cube, property: opacity.clone(), value: Value::F64(0.75) },
        ]).unwrap();
        let program = SceneProgram::compile(&doc.view()).unwrap();
        let topology = GraphTopology::try_new(program.nodes(), program.roots().collect()).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let frame = graph.evaluate(&mut executor, crate::doc::core::RationalTime::ZERO, FrameQuality::Export, Generation::new(1)).unwrap();
        let property = program.properties().node_for(cube, &opacity).unwrap();
        let material = program.content().binding(cube).unwrap().material.unwrap();
        assert_eq!(frame.value(property).and_then(|value| value.downcast_ref::<Value>()), Some(&Value::F64(0.75)));
        assert!(frame.value(material).and_then(|value| value.downcast_ref::<super::super::MaterialValue>()).is_some());
    }
}
