use std::collections::BTreeMap;

use crate::doc::store::{LayerId, LayerSource, ShapeNode, StoreError, StoreView, TextDocument};

use super::{EvaluationContext, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, TimeDependency};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaSourceValue { pub path: String, pub fingerprint: Option<String>, pub version: u64 }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialValue { pub source: MediaSourceValue }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentBinding {
    pub layer: LayerId,
    pub content: Option<NodeKey>,
    pub extent: Option<NodeKey>,
    pub material: Option<NodeKey>,
}

#[derive(Clone)]
enum Recipe {
    Text(TextDocument),
    Shape(Vec<ShapeNode>),
    MediaExtent(MediaSourceValue),
    Mesh(MediaSourceValue),
    MediaFrame(MediaSourceValue),
    Material,
}

#[derive(Debug)]
pub enum ContentProgramError { Store(StoreError), Encode(serde_json::Error), InvalidInput(NodeKind) }
impl std::fmt::Display for ContentProgramError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }
impl std::error::Error for ContentProgramError {}
impl From<StoreError> for ContentProgramError { fn from(value: StoreError) -> Self { Self::Store(value) } }
impl From<serde_json::Error> for ContentProgramError { fn from(value: serde_json::Error) -> Self { Self::Encode(value) } }

/// Revision-scoped content recipes. Layer ids are bindings only; equal text,
/// vector and media recipes point at the same content-addressed node.
pub struct ContentProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    bindings: BTreeMap<LayerId, ContentBinding>,
}

impl ContentProgram {
    pub fn compile(view: &StoreView<'_>) -> Result<Self, ContentProgramError> {
        let mut program = Self { nodes: BTreeMap::new(), recipes: BTreeMap::new(), bindings: BTreeMap::new() };
        for layer in view.layers() {
            let Some(meta) = view.meta(layer)? else { continue };
            let mut binding = ContentBinding { layer, content: None, extent: None, material: None };
            match meta.source {
                LayerSource::Text => if let Some(document) = view.text_document(layer)? {
                    let parameters = serde_json::to_vec(&document)?;
                    binding.content = Some(program.intern(NodeKind::TextContent, vec![], parameters, false, Recipe::Text(document)));
                },
                LayerSource::Shape => {
                    let shapes = view.shapes(layer)?;
                    let parameters = serde_json::to_vec(&shapes)?;
                    binding.content = Some(program.intern(NodeKind::ShapeGeometry, vec![], parameters, false, Recipe::Shape(shapes)));
                }
                LayerSource::File { path, fingerprint } => {
                    let version = resource_version(&path, fingerprint.as_deref());
                    let source = MediaSourceValue { path, fingerprint, version };
                    let parameters = serde_json::to_vec(&(source.path.as_str(), source.fingerprint.as_deref()))?;
                    binding.extent = Some(program.intern_versioned(NodeKind::MediaExtent, vec![], parameters.clone(), false, vec![version], Recipe::MediaExtent(source.clone())));
                    if crate::render::media::is_mesh_path(&source.path) {
                        let mesh = program.intern_versioned(NodeKind::MeshSource, vec![], parameters, false, vec![version], Recipe::Mesh(source));
                        binding.content = Some(mesh);
                        binding.material = Some(program.intern(NodeKind::Material, vec![mesh], vec![], false, Recipe::Material));
                    } else {
                        binding.content = Some(program.intern_versioned(NodeKind::MediaFrame, vec![], parameters, true, vec![version], Recipe::MediaFrame(source)));
                    }
                }
                _ => {}
            }
            if binding.content.is_some() || binding.extent.is_some() { program.bindings.insert(layer, binding); }
        }
        Ok(program)
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn binding(&self, layer: LayerId) -> Option<&ContentBinding> { self.bindings.get(&layer) }
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = &ContentBinding> { self.bindings.values() }

    pub fn execute(&self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Option<Result<NodeValue, ContentProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some(match recipe {
            Recipe::Text(value) => Ok(NodeValue::new(value.clone())),
            Recipe::Shape(value) => Ok(NodeValue::new(value.clone())),
            Recipe::MediaExtent(value) | Recipe::Mesh(value) => Ok(NodeValue::new(value.clone())),
            Recipe::MediaFrame(value) => Ok(NodeValue::new((value.clone(), context.time))),
            Recipe::Material => inputs.at(0).and_then(|value| value.downcast_ref::<MediaSourceValue>()).cloned()
                .map(|source| NodeValue::new(MaterialValue { source }))
                .ok_or(ContentProgramError::InvalidInput(node.identity().kind)),
        })
    }

    fn intern(&mut self, kind: NodeKind, inputs: Vec<NodeKey>, parameters: Vec<u8>, timed: bool, recipe: Recipe) -> NodeKey {
        self.intern_versioned(kind, inputs, parameters, timed, Vec::new(), recipe)
    }

    fn intern_versioned(&mut self, kind: NodeKind, inputs: Vec<NodeKey>, parameters: Vec<u8>, timed: bool, source_versions: Vec<u64>, recipe: Recipe) -> NodeKey {
        let mut identity = NodeIdentity::new(kind, inputs);
        identity.parameters = parameters;
        identity.source_versions = source_versions;
        identity.time_dependency = if timed { TimeDependency::Exact } else { TimeDependency::Static };
        let node = GraphNode::new(identity);
        let key = node.key();
        self.nodes.entry(key).or_insert(node);
        self.recipes.entry(key).or_insert(recipe);
        key
    }
}

fn resource_version(path: &str, fingerprint: Option<&str>) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    if let Some(fingerprint) = fingerprint {
        fingerprint.hash(&mut hasher);
    } else if let Ok(metadata) = std::fs::metadata(path) {
        metadata.len().hash(&mut hasher);
        metadata.modified().ok().and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok()).map(|duration| duration.as_nanos()).hash(&mut hasher);
    } else {
        path.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{LayerMeta, LayerTiming};
    use motolii_edit::{Document, Intent};

    #[test]
    fn one_hundred_cube_instances_share_one_mesh_and_one_material() {
        let mut doc = Document::new();
        for n in 1..=100 {
            let layer = LayerId(n);
            doc.apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: "/builtins/cube-v1.obj".into(), fingerprint: Some("cube-v1".into()) }, order: n as i16, timing: LayerTiming::place(0, None, 180) } },
            ]).unwrap();
        }
        let program = ContentProgram::compile(&doc.view()).unwrap();
        assert_eq!(program.nodes().filter(|node| node.identity().kind == NodeKind::MeshSource).count(), 1);
        assert_eq!(program.nodes().filter(|node| node.identity().kind == NodeKind::Material).count(), 1);
        let mesh: std::collections::BTreeSet<_> = program.bindings().filter_map(|binding| binding.content).collect();
        let materials: std::collections::BTreeSet<_> = program.bindings().filter_map(|binding| binding.material).collect();
        assert_eq!(mesh.len(), 1);
        assert_eq!(materials.len(), 1);
    }
}
