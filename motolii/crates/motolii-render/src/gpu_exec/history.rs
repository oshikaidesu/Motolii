use std::collections::{HashMap, HashSet};

use crate::render::compositor::FeedbackKey;

use super::types::{GpuIdentitySource, GpuResourceClass, GpuResourceIdentity, GpuResourceKey};

/// Logical ownership for recursive GPU state. Concrete textures/checkpoints are
/// still serviced by the proven compositor backend during migration.
#[derive(Default)]
pub(crate) struct GpuHistoryRegistry {
    seen: HashSet<FeedbackKey>,
    logical: HashMap<FeedbackKey, GpuResourceKey>,
}

impl GpuHistoryRegistry {
    pub fn begin_frame(&mut self) { self.seen.clear(); }

    pub fn observe(&mut self, key: FeedbackKey) -> GpuResourceKey {
        self.seen.insert(key);
        *self.logical.entry(key).or_insert_with(|| history_identity(key).key())
    }

    pub fn seen(&self) -> impl ExactSizeIterator<Item = FeedbackKey> + '_ {
        self.seen.iter().copied()
    }

    pub fn resource(&self, key: FeedbackKey) -> Option<GpuResourceKey> {
        self.logical.get(&key).copied()
    }

    pub fn clear(&mut self) {
        self.seen.clear();
        self.logical.clear();
    }
}

pub(crate) fn history_identity(key: FeedbackKey) -> GpuResourceIdentity {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    GpuResourceIdentity {
        source: GpuIdentitySource::Synthetic(hasher.finish()),
        class: GpuResourceClass::History,
        slot: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::LayerId;

    #[test]
    fn namespace_and_screen_are_part_of_history_identity() {
        let base = FeedbackKey { layer: LayerId(1), copy: 0, chain: 0, index: 0, screen: None, namespace: 0 };
        let temporal = FeedbackKey { namespace: 9, ..base };
        let screen = FeedbackKey { screen: Some([1920, 1080]), ..base };
        assert_ne!(history_identity(base).key(), history_identity(temporal).key());
        assert_ne!(history_identity(base).key(), history_identity(screen).key());
    }
}

impl crate::render::engine::Engine {
    pub(crate) fn gpu_set_source_clock(&mut self, time: crate::doc::core::RationalTime) {
        let delta = self.compositor.clock.map_or(1.0 / 30.0, |clock| clock[1].max(1.0e-9));
        let frame = (time.as_seconds_f64() as f32 / delta).round();
        self.compositor.clock = Some([time.as_seconds_f64() as f32, delta, frame]);
    }

    pub(crate) fn gpu_stamp_window_feedback(
        &mut self,
        layers: &mut [crate::render::compositor::LayerWithPasses],
        window: crate::render::compositor::Window,
    ) {
        for entry in layers {
            let screen_chain = entry.layer.content.texture().is_none()
                || entry.passes.iter().any(|pass| pass.reads_backdrop || pass.reads_composite());
            for pass in &mut entry.passes {
                if let Some(mut key) = pass.feedback {
                    if screen_chain || pass.reads_backdrop || pass.reads_composite() {
                        key.screen = Some(window.size());
                        pass.feedback = Some(key);
                    }
                    if self.feedback_namespace == 0 {
                        self.gpu_history.observe(key);
                    }
                }
            }
        }
    }
}
