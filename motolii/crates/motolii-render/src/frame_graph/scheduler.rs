use std::collections::BTreeSet;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use super::cache::ResultCache;
use super::{Generation, GraphTopology, NodeKey};

/// A generation lease can travel with asynchronous node work. Completion must
/// check it before retaining a result or presenting a surface.
#[derive(Clone)]
pub(super) struct GenerationLease {
    generation: Generation,
    current: Arc<AtomicU64>,
}

impl GenerationLease {
    pub(super) fn generation(&self) -> Generation {
        self.generation
    }

    pub(super) fn is_current(&self) -> bool {
        self.current.load(Ordering::Acquire) == self.generation.get()
    }
}

/// Advances monotonically. Beginning a newer generation invalidates every
/// lease from earlier work without polling the Document or the UI.
pub(super) struct GenerationGate {
    current: Arc<AtomicU64>,
    has_current: bool,
    active: bool,
    cancelled_generations: u64,
}

impl Default for GenerationGate {
    fn default() -> Self {
        Self {
            current: Arc::new(AtomicU64::new(0)),
            has_current: false,
            active: false,
            cancelled_generations: 0,
        }
    }
}

impl GenerationGate {
    pub(super) fn begin(&mut self, generation: Generation) -> Option<GenerationLease> {
        if self.has_current {
            let current = Generation::new(self.current.load(Ordering::Acquire));
            if generation < current {
                return None;
            }
            if generation > current && self.active {
                self.cancelled_generations += 1;
            }
        }
        self.current.store(generation.get(), Ordering::Release);
        self.has_current = true;
        self.active = true;
        Some(GenerationLease {
            generation,
            current: Arc::clone(&self.current),
        })
    }

    pub(super) fn is_current(&self, generation: Generation) -> bool {
        self.has_current && self.current.load(Ordering::Acquire) == generation.get()
    }

    pub(super) fn finish(&mut self, generation: Generation) {
        if self.is_current(generation) {
            self.active = false;
        }
    }

    pub(super) fn cancelled_generations(&self) -> u64 {
        self.cancelled_generations
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct DirtyNodes {
    pub nodes: BTreeSet<NodeKey>,
    pub evicted_results: usize,
}

/// Owns cache invalidation and the generation clock. It does not own document
/// topology and does not know about compositor targets.
#[derive(Default)]
pub(super) struct FrameScheduler {
    cache: ResultCache,
    gate: GenerationGate,
    node_executions: u64,
    node_reuses: u64,
    cancelled_evaluations: u64,
}

impl FrameScheduler {
    pub(super) fn cache(&self) -> &ResultCache {
        &self.cache
    }

    pub(super) fn cache_mut(&mut self) -> &mut ResultCache {
        &mut self.cache
    }

    pub(super) fn begin(&mut self, generation: Generation) -> Option<GenerationLease> {
        let lease = self.gate.begin(generation);
        if lease.is_none() {
            self.cancelled_evaluations += 1;
        }
        lease
    }

    pub(super) fn may_publish(&self, generation: Generation) -> bool {
        self.gate.is_current(generation)
    }

    pub(super) fn finish(&mut self, generation: Generation) {
        self.gate.finish(generation);
    }

    /// Invalidate a changed node and only its reachable descendants. Every
    /// cached time/quality variant of those nodes is evicted together.
    pub(super) fn invalidate(
        &mut self,
        topology: &GraphTopology,
        changed: impl IntoIterator<Item = NodeKey>,
    ) -> DirtyNodes {
        let nodes = topology.downstream_of(changed);
        let evicted_results = self.cache.invalidate_nodes(&nodes);
        DirtyNodes {
            nodes,
            evicted_results,
        }
    }

    pub(super) fn record_execution(&mut self) {
        self.node_executions += 1;
    }

    pub(super) fn record_reuse(&mut self) {
        self.node_reuses += 1;
    }

    pub(super) fn record_cancelled_evaluation(&mut self) {
        self.cancelled_evaluations += 1;
    }

    pub(super) fn stats(&self) -> SchedulerStats {
        SchedulerStats {
            node_executions: self.node_executions,
            node_reuses: self.node_reuses,
            cancelled_generations: self.gate.cancelled_generations(),
            cancelled_evaluations: self.cancelled_evaluations,
            cached_results: self.cache.len(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SchedulerStats {
    pub node_executions: u64,
    pub node_reuses: u64,
    pub cancelled_generations: u64,
    pub cancelled_evaluations: u64,
    pub cached_results: usize,
}
