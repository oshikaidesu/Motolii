//! GPU execution control plane.
//!
//! This module deliberately sits below the semantic FrameGraph.  Semantic
//! nodes describe authored meaning; this graph describes resident GPU
//! resources, their dependencies and the work needed by a concrete sink.

use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};

use crate::frame_graph::{NodeKey, WorkKey};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct GpuResourceId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct GpuContentCacheKey {
    pub(crate) work: WorkKey,
    pub(crate) variant: u64,
}

impl GpuResourceId {
    pub(crate) fn from_parts(kind: GpuResourceKind, semantic: NodeKey, discriminator: u64) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        kind.hash(&mut hasher);
        semantic.hash(&mut hasher);
        discriminator.hash(&mut hasher);
        Self(hasher.finish())
    }

    pub(crate) fn from_dependencies(
        kind: GpuResourceKind,
        dependencies: impl IntoIterator<Item = GpuResourceId>,
        discriminator: u64,
    ) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        kind.hash(&mut hasher);
        for dependency in dependencies {
            dependency.hash(&mut hasher);
        }
        discriminator.hash(&mut hasher);
        Self(hasher.finish())
    }

    #[cfg(test)]
    fn raw(value: u64) -> Self { Self(value) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum GpuResourceKind {
    Content,
    Placement,
    Effect,
    Mask,
    Clip,
    Matte,
    Plate,
    History,
    Composite,
    Sink,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum GpuResidency {
    Frame,
    Retained,
    Temporal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GpuResourceDesc {
    pub(crate) id: GpuResourceId,
    pub(crate) kind: GpuResourceKind,
    pub(crate) semantic: Option<NodeKey>,
    pub(crate) dependencies: Vec<GpuResourceId>,
    pub(crate) residency: GpuResidency,
}

impl GpuResourceDesc {
    pub(crate) fn new(
        id: GpuResourceId,
        kind: GpuResourceKind,
        semantic: Option<NodeKey>,
        dependencies: impl IntoIterator<Item = GpuResourceId>,
        residency: GpuResidency,
    ) -> Self {
        let mut dependencies: Vec<_> = dependencies.into_iter().collect();
        dependencies.sort_unstable();
        dependencies.dedup();
        Self { id, kind, semantic, dependencies, residency }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuGraphStats {
    pub(crate) resources: usize,
    pub(crate) retained: usize,
    pub(crate) temporal: usize,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct GpuResourceGraph {
    resources: BTreeMap<GpuResourceId, GpuResourceDesc>,
    downstream: BTreeMap<GpuResourceId, BTreeSet<GpuResourceId>>,
}

impl GpuResourceGraph {
    pub(crate) fn upsert(&mut self, desc: GpuResourceDesc) {
        if let Some(old) = self.resources.insert(desc.id, desc.clone()) {
            for dependency in old.dependencies {
                if let Some(children) = self.downstream.get_mut(&dependency) {
                    children.remove(&old.id);
                    if children.is_empty() {
                        self.downstream.remove(&dependency);
                    }
                }
            }
        }
        for dependency in &desc.dependencies {
            self.downstream.entry(*dependency).or_default().insert(desc.id);
        }
    }

    pub(crate) fn get(&self, id: GpuResourceId) -> Option<&GpuResourceDesc> {
        self.resources.get(&id)
    }

    pub(crate) fn invalidate(
        &self,
        changed: impl IntoIterator<Item = GpuResourceId>,
    ) -> BTreeSet<GpuResourceId> {
        let mut dirty = BTreeSet::new();
        let mut pending: Vec<_> = changed.into_iter().collect();
        while let Some(id) = pending.pop() {
            if !self.resources.contains_key(&id) || !dirty.insert(id) {
                continue;
            }
            if let Some(children) = self.downstream.get(&id) {
                pending.extend(children.iter().copied());
            }
        }
        dirty
    }

    pub(crate) fn reachable_from(
        &self,
        roots: impl IntoIterator<Item = GpuResourceId>,
    ) -> BTreeSet<GpuResourceId> {
        let mut reachable = BTreeSet::new();
        let mut pending: Vec<_> = roots.into_iter().collect();
        while let Some(id) = pending.pop() {
            if !reachable.insert(id) {
                continue;
            }
            if let Some(resource) = self.resources.get(&id) {
                pending.extend(resource.dependencies.iter().copied());
            }
        }
        reachable
    }

    pub(crate) fn retain_reachable(
        &mut self,
        roots: impl IntoIterator<Item = GpuResourceId>,
    ) -> BTreeSet<GpuResourceId> {
        let reachable = self.reachable_from(roots);
        let removed: BTreeSet<_> = self.resources.keys()
            .filter(|id| !reachable.contains(id))
            .copied()
            .collect();
        if removed.is_empty() {
            return removed;
        }
        self.resources.retain(|id, _| !removed.contains(id));
        self.rebuild_downstream();
        removed
    }

    pub(crate) fn stats(&self) -> GpuGraphStats {
        let mut stats = GpuGraphStats { resources: self.resources.len(), ..Default::default() };
        for resource in self.resources.values() {
            match resource.residency {
                GpuResidency::Retained => stats.retained += 1,
                GpuResidency::Temporal => stats.temporal += 1,
                GpuResidency::Frame => {}
            }
        }
        stats
    }

    fn rebuild_downstream(&mut self) {
        self.downstream.clear();
        for resource in self.resources.values() {
            for dependency in &resource.dependencies {
                if self.resources.contains_key(dependency) {
                    self.downstream.entry(*dependency).or_default().insert(resource.id);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum GpuPassKind {
    Upload,
    Raster,
    Effect,
    Compute,
    Composite,
    Present,
    Readback,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GpuPass {
    pub(crate) kind: GpuPassKind,
    pub(crate) reads: Vec<GpuResourceId>,
    pub(crate) writes: Vec<GpuResourceId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuSink {
    Preview,
    Stage,
    Export,
    Headless,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuExecutionPlan {
    pub(crate) passes: Vec<GpuPass>,
}

impl GpuExecutionPlan {
    pub(crate) fn push(&mut self, pass: GpuPass) {
        self.passes.push(pass);
    }

    pub(crate) fn resource_intervals(
        &self,
    ) -> BTreeMap<GpuResourceId, (usize, usize)> {
        let mut intervals = BTreeMap::new();
        for (index, pass) in self.passes.iter().enumerate() {
            for resource in pass.reads.iter().chain(pass.writes.iter()) {
                intervals
                    .entry(*resource)
                    .and_modify(|(_, last)| *last = index)
                    .or_insert((index, index));
            }
        }
        intervals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn desc(
        id: u64,
        kind: GpuResourceKind,
        dependencies: &[u64],
        residency: GpuResidency,
    ) -> GpuResourceDesc {
        GpuResourceDesc::new(
            GpuResourceId::raw(id),
            kind,
            None,
            dependencies.iter().copied().map(GpuResourceId::raw),
            residency,
        )
    }

    #[test]
    fn placement_invalidation_does_not_rebuild_content() {
        let mut graph = GpuResourceGraph::default();
        graph.upsert(desc(1, GpuResourceKind::Content, &[], GpuResidency::Retained));
        graph.upsert(desc(2, GpuResourceKind::Placement, &[1], GpuResidency::Frame));
        graph.upsert(desc(3, GpuResourceKind::Composite, &[2], GpuResidency::Frame));

        let dirty = graph.invalidate([GpuResourceId::raw(2)]);

        assert_eq!(
            dirty,
            BTreeSet::from([GpuResourceId::raw(2), GpuResourceId::raw(3)])
        );
        assert!(!dirty.contains(&GpuResourceId::raw(1)));
    }

    #[test]
    fn content_invalidation_reaches_placement_and_composite() {
        let mut graph = GpuResourceGraph::default();
        graph.upsert(desc(1, GpuResourceKind::Content, &[], GpuResidency::Retained));
        graph.upsert(desc(2, GpuResourceKind::Placement, &[1], GpuResidency::Frame));
        graph.upsert(desc(3, GpuResourceKind::Composite, &[2], GpuResidency::Frame));

        assert_eq!(
            graph.invalidate([GpuResourceId::raw(1)]),
            BTreeSet::from([
                GpuResourceId::raw(1),
                GpuResourceId::raw(2),
                GpuResourceId::raw(3),
            ])
        );
    }

    #[test]
    fn unrelated_static_content_survives_another_layers_change() {
        let mut graph = GpuResourceGraph::default();
        graph.upsert(desc(1, GpuResourceKind::Content, &[], GpuResidency::Retained));
        graph.upsert(desc(2, GpuResourceKind::Placement, &[1], GpuResidency::Frame));
        graph.upsert(desc(10, GpuResourceKind::Content, &[], GpuResidency::Retained));
        graph.upsert(desc(11, GpuResourceKind::Placement, &[10], GpuResidency::Frame));
        graph.upsert(desc(20, GpuResourceKind::Composite, &[2, 11], GpuResidency::Frame));

        let dirty = graph.invalidate([GpuResourceId::raw(2)]);

        assert!(dirty.contains(&GpuResourceId::raw(2)));
        assert!(dirty.contains(&GpuResourceId::raw(20)));
        assert!(!dirty.contains(&GpuResourceId::raw(10)));
        assert!(!dirty.contains(&GpuResourceId::raw(11)));
    }

    #[test]
    fn execution_plan_exposes_resource_liveness_intervals() {
        let a = GpuResourceId::raw(1);
        let b = GpuResourceId::raw(2);
        let c = GpuResourceId::raw(3);
        let mut plan = GpuExecutionPlan::default();
        plan.push(GpuPass { kind: GpuPassKind::Raster, reads: vec![], writes: vec![a] });
        plan.push(GpuPass { kind: GpuPassKind::Effect, reads: vec![a], writes: vec![b] });
        plan.push(GpuPass { kind: GpuPassKind::Composite, reads: vec![b], writes: vec![c] });

        let intervals = plan.resource_intervals();

        assert_eq!(intervals[&a], (0, 1));
        assert_eq!(intervals[&b], (1, 2));
        assert_eq!(intervals[&c], (2, 2));
    }

    #[test]
    fn unreachable_frame_resources_can_be_retired_without_dropping_live_retained_content() {
        let mut graph = GpuResourceGraph::default();
        graph.upsert(desc(1, GpuResourceKind::Content, &[], GpuResidency::Retained));
        graph.upsert(desc(2, GpuResourceKind::Placement, &[1], GpuResidency::Frame));
        graph.upsert(desc(3, GpuResourceKind::Content, &[], GpuResidency::Retained));

        let removed = graph.retain_reachable([GpuResourceId::raw(2)]);

        assert_eq!(removed, BTreeSet::from([GpuResourceId::raw(3)]));
        assert!(graph.get(GpuResourceId::raw(1)).is_some());
        assert!(graph.get(GpuResourceId::raw(2)).is_some());
    }
}
