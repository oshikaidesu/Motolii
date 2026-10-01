use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::key::{NodeIdentity, NodeKey};

/// One immutable node in a compiled graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNode {
    key: NodeKey,
    identity: NodeIdentity,
}

impl GraphNode {
    pub fn new(identity: NodeIdentity) -> Self {
        let key = NodeKey::for_identity(&identity);
        Self { key, identity }
    }

    pub fn key(&self) -> NodeKey {
        self.key
    }

    pub fn identity(&self) -> &NodeIdentity {
        &self.identity
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TopologyError {
    MissingInput { node: NodeKey, input: NodeKey },
    MissingRoot(NodeKey),
    Cycle(NodeKey),
    HashCollision(NodeKey),
}

impl fmt::Display for TopologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingInput { node, input } => {
                write!(f, "node {:?} refers to missing input {:?}", node, input)
            }
            Self::MissingRoot(root) => write!(f, "graph root {:?} is missing", root),
            Self::Cycle(node) => write!(f, "graph contains a cycle at {:?}", node),
            Self::HashCollision(key) => write!(f, "NodeKey hash collision at {:?}", key),
        }
    }
}

impl std::error::Error for TopologyError {}

/// The compile-time dependency graph. It never contains a `StoreView`, so
/// evaluation and view projection cannot begin a second document read.
#[derive(Clone, Debug, Default)]
pub struct GraphTopology {
    nodes: BTreeMap<NodeKey, GraphNode>,
    roots: Vec<NodeKey>,
    downstream: BTreeMap<NodeKey, Vec<NodeKey>>,
}

impl GraphTopology {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn try_new(
        nodes: impl IntoIterator<Item = GraphNode>,
        roots: Vec<NodeKey>,
    ) -> Result<Self, TopologyError> {
        let mut indexed = BTreeMap::new();
        for node in nodes {
            match indexed.entry(node.key) {
                std::collections::btree_map::Entry::Vacant(slot) => {
                    slot.insert(node);
                }
                std::collections::btree_map::Entry::Occupied(slot)
                    if slot.get().identity == node.identity => {}
                std::collections::btree_map::Entry::Occupied(_) => {
                    return Err(TopologyError::HashCollision(node.key))
                }
            }
        }
        for root in &roots {
            if !indexed.contains_key(root) {
                return Err(TopologyError::MissingRoot(*root));
            }
        }

        let mut downstream: BTreeMap<NodeKey, Vec<NodeKey>> = BTreeMap::new();
        for node in indexed.values() {
            for input in &node.identity.inputs {
                if !indexed.contains_key(input) {
                    return Err(TopologyError::MissingInput {
                        node: node.key,
                        input: *input,
                    });
                }
                downstream.entry(*input).or_default().push(node.key);
            }
        }
        for children in downstream.values_mut() {
            children.sort_unstable();
            children.dedup();
        }

        let topology = Self {
            nodes: indexed,
            roots,
            downstream,
        };
        for root in &topology.roots {
            let mut visiting = BTreeSet::new();
            let mut complete = BTreeSet::new();
            topology.visit(*root, &mut visiting, &mut complete, &mut Vec::new())?;
        }
        Ok(topology)
    }

    pub fn roots(&self) -> &[NodeKey] {
        &self.roots
    }

    pub fn node(&self, key: NodeKey) -> Option<&GraphNode> {
        self.nodes.get(&key)
    }

    pub fn reachable(&self) -> Vec<NodeKey> {
        let mut visiting = BTreeSet::new();
        let mut complete = BTreeSet::new();
        let mut ordered = Vec::new();
        for root in &self.roots {
            // Construction has already checked the topology. If it became
            // invalid here, that is an internal bug rather than user input.
            self.visit(*root, &mut visiting, &mut complete, &mut ordered)
                .expect("validated FrameGraph topology");
        }
        ordered
    }

    pub fn downstream_of(&self, changed: impl IntoIterator<Item = NodeKey>) -> BTreeSet<NodeKey> {
        let mut dirty = BTreeSet::new();
        let mut pending: Vec<_> = changed
            .into_iter()
            .filter(|key| self.nodes.contains_key(key))
            .collect();
        while let Some(node) = pending.pop() {
            if !dirty.insert(node) {
                continue;
            }
            if let Some(children) = self.downstream.get(&node) {
                pending.extend(children.iter().copied());
            }
        }
        dirty
    }

    fn visit(
        &self,
        node: NodeKey,
        visiting: &mut BTreeSet<NodeKey>,
        complete: &mut BTreeSet<NodeKey>,
        ordered: &mut Vec<NodeKey>,
    ) -> Result<(), TopologyError> {
        if complete.contains(&node) {
            return Ok(());
        }
        if !visiting.insert(node) {
            return Err(TopologyError::Cycle(node));
        }
        let graph_node = self
            .nodes
            .get(&node)
            .ok_or(TopologyError::MissingRoot(node))?;
        for input in &graph_node.identity.inputs {
            self.visit(*input, visiting, complete, ordered)?;
        }
        visiting.remove(&node);
        complete.insert(node);
        ordered.push(node);
        Ok(())
    }
}
