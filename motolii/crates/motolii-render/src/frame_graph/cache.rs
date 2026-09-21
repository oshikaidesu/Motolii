use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

use super::{NodeKey, WorkKey};

/// A completed node result. The graph only owns its identity and lifetime;
/// node adapters own the concrete CPU or GPU value type.
#[derive(Clone)]
pub struct NodeValue(Arc<dyn Any + Send + Sync>);

impl fmt::Debug for NodeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NodeValue(..)")
    }
}

impl NodeValue {
    pub fn new<T>(value: T) -> Self
    where
        T: Any + Send + Sync,
    {
        Self(Arc::new(value))
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.0.downcast_ref()
    }
}

/// The result cache is indexed exclusively by `WorkKey`; the graph never
/// infers a result from a revision, layer id, or view.
#[derive(Default)]
pub(super) struct ResultCache {
    completed: BTreeMap<WorkKey, NodeValue>,
}

impl ResultCache {
    pub(super) fn get(&self, key: WorkKey) -> Option<NodeValue> {
        self.completed.get(&key).cloned()
    }

    pub(super) fn insert(&mut self, key: WorkKey, value: NodeValue) {
        self.completed.insert(key, value);
    }

    pub(super) fn invalidate_nodes(&mut self, dirty: &BTreeSet<NodeKey>) -> usize {
        let before = self.completed.len();
        self.completed.retain(|key, _| !dirty.contains(&key.node()));
        before - self.completed.len()
    }

    pub(super) fn len(&self) -> usize {
        self.completed.len()
    }
}
