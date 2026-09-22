use std::collections::BTreeMap;

use super::types::{GpuPassKey, GpuPhysicalSlot, GpuResourceKey, GpuResourceVersion};

/// Concrete backend residency is deliberately keyed by logical resource id,
/// never by frame time. The payload type is backend-owned (texture, buffer,
/// mesh handle, re_renderer object, ...).
pub(crate) struct GpuResourceStore<T> {
    entries: BTreeMap<GpuResourceKey, Resident<T>>,
    scratch: BTreeMap<GpuPhysicalSlot, T>,
}

struct Resident<T> {
    version: GpuResourceVersion,
    value: T,
    last_generation: u64,
}

impl<T> Default for GpuResourceStore<T> {
    fn default() -> Self {
        Self { entries: BTreeMap::new(), scratch: BTreeMap::new() }
    }
}

impl<T> GpuResourceStore<T> {
    pub fn current(&self, key: GpuResourceKey, version: GpuResourceVersion) -> Option<&T> {
        self.entries.get(&key).filter(|entry| entry.version == version).map(|entry| &entry.value)
    }

    pub fn current_mut(&mut self, key: GpuResourceKey, version: GpuResourceVersion) -> Option<&mut T> {
        self.entries.get_mut(&key).filter(|entry| entry.version == version).map(|entry| &mut entry.value)
    }

    pub fn install(&mut self, key: GpuResourceKey, version: GpuResourceVersion, generation: u64, value: T) -> Option<T> {
        self.entries.insert(key, Resident { version, value, last_generation: generation }).map(|old| old.value)
    }

    pub fn touch(&mut self, key: GpuResourceKey, generation: u64) {
        if let Some(entry) = self.entries.get_mut(&key) { entry.last_generation = generation; }
    }

    pub fn retain_generation(&mut self, oldest: u64) {
        self.entries.retain(|_, entry| entry.last_generation >= oldest);
    }

    pub fn scratch(&self, slot: GpuPhysicalSlot) -> Option<&T> { self.scratch.get(&slot) }
    pub fn install_scratch(&mut self, slot: GpuPhysicalSlot, value: T) -> Option<T> {
        self.scratch.insert(slot, value)
    }

    pub fn len(&self) -> usize { self.entries.len() }
}

/// Backend operation registry. Logical passes identify work; concrete GPU
/// closures/objects are registered separately so the planner remains pure.
pub(crate) struct GpuOperationTable<Op> {
    operations: BTreeMap<GpuPassKey, Op>,
}

impl<Op> Default for GpuOperationTable<Op> {
    fn default() -> Self { Self { operations: BTreeMap::new() } }
}

impl<Op> GpuOperationTable<Op> {
    pub fn install(&mut self, pass: GpuPassKey, operation: Op) -> Option<Op> {
        self.operations.insert(pass, operation)
    }
    pub fn get(&self, pass: GpuPassKey) -> Option<&Op> { self.operations.get(&pass) }
    pub fn get_mut(&mut self, pass: GpuPassKey) -> Option<&mut Op> { self.operations.get_mut(&pass) }
    pub fn remove(&mut self, pass: GpuPassKey) -> Option<Op> { self.operations.remove(&pass) }
    pub fn len(&self) -> usize { self.operations.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_identity_reuses_backend_payload_until_version_changes() {
        let key = GpuResourceKey(7);
        let mut store = GpuResourceStore::default();
        store.install(key, GpuResourceVersion::new(1), 1, String::from("texture-a"));
        assert_eq!(store.current(key, GpuResourceVersion::new(1)).map(String::as_str), Some("texture-a"));
        assert!(store.current(key, GpuResourceVersion::new(2)).is_none());
        let old = store.install(key, GpuResourceVersion::new(2), 2, String::from("texture-b"));
        assert_eq!(old.as_deref(), Some("texture-a"));
        assert_eq!(store.current(key, GpuResourceVersion::new(2)).map(String::as_str), Some("texture-b"));
    }
}
