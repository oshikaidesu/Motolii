use std::hash::{Hash, Hasher};

use crate::doc::store::LayerId;
use crate::frame_graph::NodeKey;

use super::types::{GpuResourceClass, GpuResourceIdentity};

/// Stable owner of one feedback recurrence. Window dimensions are deliberately
/// not part of the owner: screen-scoped history is specialized by the sink.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct GpuHistoryOwner {
    pub effect: NodeKey,
    pub layer: LayerId,
    pub instance: u32,
    pub chain: u8,
    pub pass_index: u16,
    pub namespace: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum GpuHistoryTarget {
    Layer,
    Screen([u32; 2]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct GpuHistoryKey {
    pub owner: GpuHistoryOwner,
    pub target: GpuHistoryTarget,
}

impl GpuHistoryKey {
    pub fn identity(self) -> GpuResourceIdentity {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        GpuResourceIdentity::synthetic(hasher.finish(), GpuResourceClass::History, 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuHistoryStep {
    Reuse,
    Advance,
    Restart,
}

/// Recurrence scheduling is a control-plane decision, not shader behavior.
pub(crate) fn history_step(have: Option<i64>, now: Option<i64>) -> GpuHistoryStep {
    match (have, now) {
        (Some(have), Some(now)) if have == now => GpuHistoryStep::Reuse,
        (Some(have), Some(now)) if have + 1 == now => GpuHistoryStep::Advance,
        _ => GpuHistoryStep::Restart,
    }
}

/// A state already produced for the current frame is usable only if it did not
/// come from a restart initial condition. Fresh-at-current must still replay.
pub(crate) fn history_is_current(have: i64, now: i64, fresh: bool) -> bool {
    have == now && !fresh
}

pub(crate) fn history_replay_from(in_point: i64, checkpoint: Option<i64>) -> i64 {
    checkpoint.map_or(in_point, |frame| frame + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame_graph::{NodeIdentity, NodeKind};

    fn effect() -> NodeKey {
        NodeKey::for_identity(&NodeIdentity::new(NodeKind::Custom(92), vec![]))
    }

    fn owner(layer: u64, namespace: u64) -> GpuHistoryOwner {
        GpuHistoryOwner {
            effect: effect(),
            layer: LayerId(layer),
            instance: 0,
            chain: 0,
            pass_index: 1,
            namespace,
        }
    }

    #[test]
    fn recurrence_step_is_reuse_advance_or_restart() {
        assert_eq!(history_step(Some(10), Some(10)), GpuHistoryStep::Reuse);
        assert_eq!(history_step(Some(10), Some(11)), GpuHistoryStep::Advance);
        assert_eq!(history_step(Some(10), Some(12)), GpuHistoryStep::Restart);
        assert_eq!(history_step(None, Some(0)), GpuHistoryStep::Restart);
    }

    #[test]
    fn history_identity_separates_owner_namespace_and_screen_sink() {
        let a = GpuHistoryKey { owner: owner(1, 0), target: GpuHistoryTarget::Layer }.identity().key();
        let other_layer = GpuHistoryKey { owner: owner(2, 0), target: GpuHistoryTarget::Layer }.identity().key();
        let other_namespace = GpuHistoryKey { owner: owner(1, 7), target: GpuHistoryTarget::Layer }.identity().key();
        let screen_a = GpuHistoryKey { owner: owner(1, 0), target: GpuHistoryTarget::Screen([1920, 1080]) }.identity().key();
        let screen_b = GpuHistoryKey { owner: owner(1, 0), target: GpuHistoryTarget::Screen([1280, 720]) }.identity().key();

        assert_ne!(a, other_layer);
        assert_ne!(a, other_namespace);
        assert_ne!(screen_a, screen_b);
        assert_ne!(a, screen_a);
    }

    #[test]
    fn replay_starts_after_checkpoint_or_at_in_point() {
        assert_eq!(history_replay_from(4, None), 4);
        assert_eq!(history_replay_from(4, Some(12)), 13);
        assert!(history_is_current(20, 20, false));
        assert!(!history_is_current(20, 20, true));
    }
}
