use std::collections::{BTreeMap, BTreeSet};

use super::types::{
    GpuPassDesc, GpuPassKey, GpuResourceDesc, GpuResourceKey, GpuResourceLifetime,
    GpuResourceVersion,
};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuGraphStats {
    pub resources: usize,
    pub resident: usize,
    pub passes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuResourceDelta {
    Inserted,
    Unchanged,
    Changed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GpuGraphError {
    ResourceHashCollision(GpuResourceKey),
    PassHashCollision(GpuPassKey),
    MissingResource { pass: GpuPassKey, resource: GpuResourceKey },
    DuplicateProducer { resource: GpuResourceKey, first: GpuPassKey, second: GpuPassKey },
    MissingPassDependency { pass: GpuPassKey, dependency: GpuPassKey },
}

#[derive(Clone, Debug)]
struct ResourceRecord {
    desc: GpuResourceDesc,
    resident_version: Option<GpuResourceVersion>,
    last_resident_generation: u64,
}

/// Persistent logical GPU resource/execution graph.
///
/// Concrete wgpu handles deliberately do not live here. This owner is CPU
/// testable: identity, value version, dependency, residency and lifetime only.
#[derive(Default)]
pub(crate) struct GpuResourceGraph {
    resources: BTreeMap<GpuResourceKey, ResourceRecord>,
    passes: BTreeMap<GpuPassKey, GpuPassDesc>,
    producers: BTreeMap<GpuResourceKey, GpuPassKey>,
    downstream: BTreeMap<GpuResourceKey, BTreeSet<GpuResourceKey>>,
}

impl GpuResourceGraph {
    pub fn upsert_resource(
        &mut self,
        desc: GpuResourceDesc,
    ) -> Result<GpuResourceDelta, GpuGraphError> {
        let key = desc.key();
        if let Some(existing) = self.resources.get_mut(&key) {
            if existing.desc.identity != desc.identity {
                return Err(GpuGraphError::ResourceHashCollision(key));
            }
            let same = existing.desc.version == desc.version
                && existing.desc.dependencies == desc.dependencies
                && existing.desc.lifetime == desc.lifetime
                && existing.desc.alias_class == desc.alias_class
                && existing.desc.estimated_bytes == desc.estimated_bytes;
            if same {
                return Ok(GpuResourceDelta::Unchanged);
            }
            // Any descriptor change can change what the GPU slot means.
            // A lowerer bug that forgets to bump version must not preserve
            // stale residency after dependency/lifetime/alias changes.
            existing.resident_version = None;
            existing.desc = desc;
            self.rebuild_downstream();
            return Ok(GpuResourceDelta::Changed);
        }

        self.resources.insert(
            key,
            ResourceRecord {
                desc,
                resident_version: None,
                last_resident_generation: 0,
            },
        );
        self.rebuild_downstream();
        Ok(GpuResourceDelta::Inserted)
    }

    pub fn insert_pass(&mut self, pass: GpuPassDesc) -> Result<(), GpuGraphError> {
        let key = pass.key();
        if let Some(existing) = self.passes.get(&key) {
            if existing != &pass {
                return Err(GpuGraphError::PassHashCollision(key));
            }
            return Ok(());
        }

        for resource in pass.identity.reads.iter().chain(&pass.identity.writes) {
            if !self.resources.contains_key(resource) {
                return Err(GpuGraphError::MissingResource { pass: key, resource: *resource });
            }
        }
        for dependency in &pass.after {
            if !self.passes.contains_key(dependency) {
                return Err(GpuGraphError::MissingPassDependency { pass: key, dependency: *dependency });
            }
        }
        for resource in &pass.identity.writes {
            if let Some(first) = self.producers.get(resource).copied() {
                if first != key {
                    return Err(GpuGraphError::DuplicateProducer {
                        resource: *resource,
                        first,
                        second: key,
                    });
                }
            }
        }
        for resource in &pass.identity.writes {
            self.producers.insert(*resource, key);
        }
        self.passes.insert(key, pass);
        Ok(())
    }

    pub fn resource(&self, key: GpuResourceKey) -> Option<&GpuResourceDesc> {
        self.resources.get(&key).map(|record| &record.desc)
    }

    pub fn pass(&self, key: GpuPassKey) -> Option<&GpuPassDesc> {
        self.passes.get(&key)
    }

    pub fn producer(&self, resource: GpuResourceKey) -> Option<GpuPassKey> {
        self.producers.get(&resource).copied()
    }

    pub fn is_current(&self, key: GpuResourceKey) -> bool {
        self.resources.get(&key).is_some_and(|record| {
            !matches!(record.desc.lifetime, GpuResourceLifetime::Frame)
                && record.resident_version == Some(record.desc.version)
        })
    }

    pub fn mark_resident(
        &mut self,
        key: GpuResourceKey,
        version: GpuResourceVersion,
        generation: u64,
    ) {
        if let Some(record) = self.resources.get_mut(&key) {
            if record.desc.version == version {
                record.resident_version = Some(version);
                record.last_resident_generation = generation;
            }
        }
    }

    /// Mark outputs resident after successful execution. Frame scratch remains
    /// plan-local and is intentionally never cached across frames.
    pub fn mark_pass_outputs_resident(&mut self, pass: GpuPassKey, generation: u64) {
        let Some(desc) = self.passes.get(&pass) else { return };
        let writes = desc.identity.writes.clone();
        for key in writes {
            if let Some(record) = self.resources.get_mut(&key) {
                if !matches!(record.desc.lifetime, GpuResourceLifetime::Frame) {
                    record.resident_version = Some(record.desc.version);
                    record.last_resident_generation = generation;
                }
            }
        }
    }

    /// Dirty one logical resource and only its true GPU dependents.
    pub fn invalidate(
        &mut self,
        changed: impl IntoIterator<Item = GpuResourceKey>,
    ) -> BTreeSet<GpuResourceKey> {
        let mut dirty = BTreeSet::new();
        let mut pending: Vec<_> = changed.into_iter().collect();
        while let Some(key) = pending.pop() {
            if !dirty.insert(key) {
                continue;
            }
            if let Some(record) = self.resources.get_mut(&key) {
                record.resident_version = None;
            }
            if let Some(children) = self.downstream.get(&key) {
                pending.extend(children.iter().copied());
            }
        }
        dirty
    }

    pub fn retire_temporal(&mut self, generation: u64) {
        for record in self.resources.values_mut() {
            let GpuResourceLifetime::Temporal { retain_generations } = record.desc.lifetime else {
                continue;
            };
            if generation.saturating_sub(record.last_resident_generation)
                > u64::from(retain_generations)
            {
                record.resident_version = None;
            }
        }
    }

    pub fn stats(&self) -> GpuGraphStats {
        GpuGraphStats {
            resources: self.resources.len(),
            resident: self.resources.values().filter(|record| record.resident_version == Some(record.desc.version)).count(),
            passes: self.passes.len(),
        }
    }

    fn rebuild_downstream(&mut self) {
        self.downstream.clear();
        for (key, record) in &self.resources {
            for input in &record.desc.dependencies {
                self.downstream.entry(*input).or_default().insert(*key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame_graph::{NodeIdentity, NodeKey, NodeKind};
    use crate::gpu_exec::types::{
        GpuIdentitySource, GpuResourceClass, GpuResourceIdentity,
    };

    fn node(tag: u16) -> NodeKey {
        NodeKey::for_identity(&NodeIdentity::new(NodeKind::Custom(tag), vec![]))
    }

    fn resource(
        semantic: NodeKey,
        class: GpuResourceClass,
        version: u64,
        dependencies: Vec<GpuResourceKey>,
    ) -> GpuResourceDesc {
        GpuResourceDesc {
            identity: GpuResourceIdentity {
                source: GpuIdentitySource::Semantic(semantic),
                class,
                slot: 0,
            },
            version: GpuResourceVersion::new(version),
            dependencies,
            lifetime: GpuResourceLifetime::Persistent,
            alias_class: None,
            estimated_bytes: 0,
        }
    }

    #[test]
    fn invalidation_stops_at_actual_gpu_dependents() {
        let mut graph = GpuResourceGraph::default();
        let content = resource(node(1), GpuResourceClass::Content, 1, vec![]);
        let content_key = content.key();
        graph.upsert_resource(content).unwrap();

        let effect = resource(node(2), GpuResourceClass::Effect, 1, vec![content_key]);
        let effect_key = effect.key();
        graph.upsert_resource(effect).unwrap();

        let composite = resource(node(3), GpuResourceClass::Composite, 1, vec![effect_key]);
        let composite_key = composite.key();
        graph.upsert_resource(composite).unwrap();

        let unrelated = resource(node(4), GpuResourceClass::Content, 1, vec![]);
        let unrelated_key = unrelated.key();
        graph.upsert_resource(unrelated).unwrap();

        for key in [content_key, effect_key, composite_key, unrelated_key] {
            let version = graph.resource(key).unwrap().version;
            graph.mark_resident(key, version, 1);
        }

        let dirty = graph.invalidate([content_key]);
        assert_eq!(dirty, BTreeSet::from([content_key, effect_key, composite_key]));
        assert!(graph.is_current(unrelated_key));
    }

    #[test]
    fn identity_survives_version_changes() {
        let mut graph = GpuResourceGraph::default();
        let first = resource(node(7), GpuResourceClass::Placement, 1, vec![]);
        let key = first.key();
        assert_eq!(graph.upsert_resource(first).unwrap(), GpuResourceDelta::Inserted);
        graph.mark_resident(key, GpuResourceVersion::new(1), 1);
        assert!(graph.is_current(key));

        let second = resource(node(7), GpuResourceClass::Placement, 2, vec![]);
        assert_eq!(second.key(), key);
        assert_eq!(graph.upsert_resource(second).unwrap(), GpuResourceDelta::Changed);
        assert!(!graph.is_current(key));
    }
}
