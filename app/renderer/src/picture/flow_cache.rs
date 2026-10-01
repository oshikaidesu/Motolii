use super::*;

/// Renderer state for the borrowed Taffy solver. It is held by `Engine`, not
/// `Document`: the tree is GPU-adjacent preparation state, never Undo state.
#[derive(Default)]
pub struct FlowCache {
    pub(super) document: std::cell::RefCell<Option<usize>>,
    pub(super) roots: std::cell::RefCell<HashMap<LayerId, CachedTree>>,
    #[cfg(test)]
    pub(super) layout_passes: std::cell::Cell<usize>,
    #[cfg(test)]
    pub(super) plan_builds: std::cell::Cell<usize>,
}

pub(super) struct CachedTree {
    pub(super) tree: TaffyTree<Measure>,
    pub(super) root: CachedNode,
    pub(super) blockers: Vec<NodeId>,
}

pub(super) struct CachedNode {
    pub(super) id: NodeId,
    pub(super) key: PlanKey,
    pub(super) measure: Option<Measure>,
    pub(super) leaf: Option<LeafSpec>,
    pub(super) group: Option<(LayerId, bool)>,
    pub(super) children: Vec<CachedNode>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PlanKey { Group(LayerId), Leaf(LayerId, bool), GridPlaceholder }

pub(super) struct PlanNode {
    pub(super) key: PlanKey,
    pub(super) style: Style,
    pub(super) measure: Option<Measure>,
    pub(super) leaf: Option<LeafSpec>,
    pub(super) group: Option<(LayerId, bool)>,
    pub(super) children: Vec<PlanNode>,
}

#[derive(Clone, Copy)]
pub(super) struct LeafSpec {
    pub(super) text_fill: bool,
    pub(super) layer: LayerId,
    pub(super) bounds: [f32; 4],
    pub(super) sizing: [Sizing; 2],
    pub(super) fit: i64,
}

impl PlanNode {
    pub(super) fn same_shape(&self, cached: &CachedNode) -> bool {
        self.key == cached.key && self.children.len() == cached.children.len()
            && self.children.iter().zip(&cached.children).all(|(next, held)| next.same_shape(held))
    }

    pub(super) fn create(&self, tree: &mut TaffyTree<Measure>) -> Result<CachedNode, StoreError> {
        let children: Result<Vec<_>, _> = self.children.iter().map(|child| child.create(tree)).collect();
        let children = children?;
        let ids: Vec<_> = children.iter().map(|child| child.id).collect();
        let id = match self.measure {
            Some(measure) if ids.is_empty() => tree.new_leaf_with_context(self.style.clone(), measure),
            Some(_) => tree.new_with_children(self.style.clone(), &ids),
            None if ids.is_empty() => tree.new_leaf(self.style.clone()),
            None => tree.new_with_children(self.style.clone(), &ids),
        }.map_err(layout_error)?;
        Ok(CachedNode { id, key: self.key, measure: self.measure, leaf: self.leaf, group: self.group, children })
    }

    /// Taffy marks an ancestor dirty only when style or measurement context changes.
    pub(super) fn update(&self, tree: &mut TaffyTree<Measure>, cached: &mut CachedNode) -> Result<bool, StoreError> {
        debug_assert!(self.same_shape(cached));
        let mut dirty = false;
        if tree.style(cached.id).map_err(layout_error)? != &self.style {
            tree.set_style(cached.id, self.style.clone()).map_err(layout_error)?;
            dirty = true;
        }
        if cached.measure != self.measure {
            tree.set_node_context(cached.id, self.measure).map_err(layout_error)?;
            cached.measure = self.measure;
            dirty = true;
        }
        cached.leaf = self.leaf;
        cached.group = self.group;
        for (next, held) in self.children.iter().zip(&mut cached.children) { dirty |= next.update(tree, held)?; }
        Ok(dirty)
    }

    pub(super) fn collect(&self, cached: &CachedNode, leaves: &mut Vec<Leaf>, groups: &mut Vec<(NodeId, LayerId, bool)>) {
        if let Some(group) = self.group { groups.push((cached.id, group.0, group.1)); }
        if let Some(leaf) = self.leaf { leaves.push(Leaf { node: cached.id, text_fill: leaf.text_fill, layer: leaf.layer, bounds: leaf.bounds, sizing: leaf.sizing, fit: leaf.fit }); }
        for (next, held) in self.children.iter().zip(&cached.children) { next.collect(held, leaves, groups); }
    }
}

impl CachedNode {
    pub(super) fn collect(&self, leaves: &mut Vec<Leaf>, groups: &mut Vec<(NodeId, LayerId, bool)>) {
        if let Some(group) = self.group { groups.push((self.id, group.0, group.1)); }
        if let Some(leaf) = self.leaf { leaves.push(Leaf { node: self.id, text_fill: leaf.text_fill, layer: leaf.layer, bounds: leaf.bounds, sizing: leaf.sizing, fit: leaf.fit }); }
        for child in &self.children { child.collect(leaves, groups); }
    }
}
