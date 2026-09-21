use std::collections::BTreeMap;

use crate::doc::core::RationalTime;

use super::cache::NodeValue;
use super::scheduler::{FrameScheduler, GenerationLease};
use super::{FrameQuality, Generation, GraphNode, GraphTopology, NodeKey, WorkKey};

/// The frame context supplied to an adapter. It exposes no document reader;
/// node inputs are the only upstream values an evaluator may consume.
#[derive(Clone)]
pub struct EvaluationContext {
    pub time: RationalTime,
    pub quality: FrameQuality,
    generation: GenerationLease,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynamicInput {
    pub node: NodeKey,
    pub time: RationalTime,
}

impl EvaluationContext {
    pub fn generation(&self) -> Generation {
        self.generation.generation()
    }

    pub fn is_current(&self) -> bool {
        self.generation.is_current()
    }
}

/// Values of the input edges for one node, in the compiler-declared order.
#[derive(Clone, Default)]
pub struct NodeInputs(Vec<(NodeKey, NodeValue)>);

impl NodeInputs {
    pub fn at(&self, index: usize) -> Option<&NodeValue> {
        self.0.get(index).map(|(_, value)| value)
    }

    pub fn get(&self, key: NodeKey) -> Option<&NodeValue> {
        self.0
            .iter()
            .find(|(input, _)| *input == key)
            .map(|(_, value)| value)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (NodeKey, &NodeValue)> {
        self.0.iter().map(|(key, value)| (*key, value))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Small adapter boundary for existing layout, text, shape, media, and GPU
/// evaluators. It deliberately returns an opaque value instead of imposing a
/// renderer-wide value enum.
pub trait NodeExecutor {
    type Error;

    fn dynamic_inputs(
        &mut self,
        _node: &GraphNode,
        _inputs: &NodeInputs,
        _context: &EvaluationContext,
    ) -> Result<Vec<DynamicInput>, Self::Error> {
        Ok(Vec::new())
    }

    fn execute(
        &mut self,
        node: &GraphNode,
        inputs: NodeInputs,
        context: EvaluationContext,
    ) -> Result<NodeValue, Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EvaluationState {
    Current,
    Stale,
    Cancelled,
}

/// One evaluation's shared values and instrumentation. View projection reads
/// this result; it does not invoke the executor again.
pub(super) struct ScheduledFrame {
    pub(super) state: EvaluationState,
    pub(super) values: BTreeMap<NodeKey, NodeValue>,
    pub(super) executed: Vec<NodeKey>,
    pub(super) reused: Vec<NodeKey>,
}

impl ScheduledFrame {
    fn stale(_generation: Generation) -> Self {
        Self {
            state: EvaluationState::Stale,
            values: BTreeMap::new(),
            executed: Vec::new(),
            reused: Vec::new(),
        }
    }

    fn cancelled(_generation: Generation) -> Self {
        Self {
            state: EvaluationState::Cancelled,
            values: BTreeMap::new(),
            executed: Vec::new(),
            reused: Vec::new(),
        }
    }
}

/// Evaluate topology in dependency order, reusing only an exact `WorkKey`.
/// A generation that becomes stale during adapter work cannot populate cache
/// or produce a presentable `ScheduledFrame`.
pub(super) fn evaluate<E: NodeExecutor>(
    scheduler: &mut FrameScheduler,
    topology: &GraphTopology,
    executor: &mut E,
    time: RationalTime,
    quality: FrameQuality,
    generation: Generation,
) -> Result<ScheduledFrame, E::Error> {
    let Some(lease) = scheduler.begin(generation) else {
        return Ok(ScheduledFrame::stale(generation));
    };
    let mut values = BTreeMap::new();
    let mut executed = Vec::new();
    let mut reused = Vec::new();

    #[allow(clippy::too_many_arguments)]
    fn node_at<E: NodeExecutor>(
        scheduler: &mut FrameScheduler,
        topology: &GraphTopology,
        executor: &mut E,
        key: NodeKey,
        time: RationalTime,
        root_time: RationalTime,
        quality: FrameQuality,
        lease: &GenerationLease,
        values: &mut BTreeMap<NodeKey, NodeValue>,
        executed: &mut Vec<NodeKey>,
        reused: &mut Vec<NodeKey>,
    ) -> Result<Option<NodeValue>, E::Error> {
        if !lease.is_current() {
            scheduler.record_cancelled_evaluation();
            return Ok(None);
        }
        let node = topology
            .node(key)
            .expect("reachable nodes belong to the topology");
        let work = WorkKey::for_node(node.identity(), time, quality);
        if let Some(value) = scheduler.cache().get(work) {
            scheduler.record_reuse();
            if time == root_time { values.insert(key, value.clone()); }
            reused.push(key);
            return Ok(Some(value));
        }

        let mut inputs = Vec::with_capacity(node.identity().inputs.len());
        for (index, input) in node.identity().inputs.iter().enumerate() {
            let input_time = node.identity().input_times[index].apply(time);
            let Some(value) = node_at(scheduler, topology, executor, *input, input_time, root_time, quality, lease, values, executed, reused)? else {
                return Ok(None);
            };
            inputs.push((*input, value));
        }
        let context = EvaluationContext { time, quality, generation: lease.clone() };
        let requests = executor.dynamic_inputs(node, &NodeInputs(inputs.clone()), &context)?;
        for request in requests {
            let Some(value) = node_at(
                scheduler,
                topology,
                executor,
                request.node,
                request.time,
                root_time,
                quality,
                lease,
                values,
                executed,
                reused,
            )? else {
                return Ok(None);
            };
            inputs.push((request.node, value));
        }
        let value = executor.execute(node, NodeInputs(inputs), context)?;
        if !lease.is_current() {
            scheduler.record_cancelled_evaluation();
            return Ok(None);
        }
        scheduler.cache_mut().insert(work, value.clone());
        scheduler.record_execution();
        if time == root_time { values.insert(key, value.clone()); }
        executed.push(key);
        Ok(Some(value))
    }

    for root in topology.roots() {
        if node_at(scheduler, topology, executor, *root, time, time, quality, &lease, &mut values, &mut executed, &mut reused)?.is_none() {
            return Ok(ScheduledFrame::cancelled(generation));
        }
    }

    scheduler.finish(generation);
    Ok(ScheduledFrame {
        state: EvaluationState::Current,
        values,
        executed,
        reused,
    })
}
