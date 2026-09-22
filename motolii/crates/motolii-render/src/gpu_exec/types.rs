use std::hash::{Hash, Hasher};

use crate::frame_graph::NodeKey;

/// Stable logical resource role. This vocabulary is intentionally separate
/// from semantic NodeKind and from concrete wgpu pass types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum GpuResourceClass {
    Content,
    Placement,
    Effect,
    Mask,
    Matte,
    Plate,
    History,
    Composite,
    Scratch,
    Sink,
}

/// A logical GPU resource is either anchored to one semantic recipe or is a
/// synthetic execution resource created by the GPU planner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum GpuIdentitySource {
    Semantic(NodeKey),
    Synthetic(u64),
}

/// Stable identity of one logical GPU slot. `slot` disambiguates multiple
/// resources owned by the same semantic node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct GpuResourceIdentity {
    pub source: GpuIdentitySource,
    pub class: GpuResourceClass,
    pub slot: u32,
}

impl GpuResourceIdentity {
    pub fn semantic(node: NodeKey, class: GpuResourceClass, slot: u32) -> Self {
        Self { source: GpuIdentitySource::Semantic(node), class, slot }
    }

    pub fn synthetic(tag: u64, class: GpuResourceClass, slot: u32) -> Self {
        Self { source: GpuIdentitySource::Synthetic(tag), class, slot }
    }

    pub fn key(self) -> GpuResourceKey {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        GpuResourceKey(hasher.finish())
    }
}

/// Compact lookup key. The graph retains the complete identity and rejects a
/// hash collision instead of silently accepting it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct GpuResourceKey(pub(crate) u64);

/// Identity and version are deliberately separate.
///
/// Identity = long-lived GPU slot.
/// Version = value currently required in that slot.
///
/// Advancing comp time alone must not create a new identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct GpuResourceVersion(pub(crate) u64);

impl GpuResourceVersion {
    pub fn as_u64(self) -> u64 { self.0 }
}

impl GpuResourceVersion {
    pub const STATIC: Self = Self(0);

    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Runtime fingerprint of the GPU-visible evaluated value.
    ///
    /// This intentionally hashes canonical value bytes, not comp time,
    /// generation, or "node executed" status. Many semantic nodes are Exact
    /// today even when their evaluated value did not change.
    pub fn from_canonical(encoded: &crate::frame_graph::CanonicalEncoder) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        encoded.as_bytes().hash(&mut hasher);
        Self(hasher.finish())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuResourceLifetime {
    Persistent,
    Temporal { retain_generations: u16 },
    Frame,
}

/// Compatibility class for transient aliasing. The concrete executor chooses
/// the tag from format/usage/sample-count/alignment rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct GpuAliasClass {
    pub tag: u64,
    pub bytes: u64,
    pub alignment: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GpuResourceDesc {
    pub identity: GpuResourceIdentity,
    pub version: GpuResourceVersion,
    pub dependencies: Vec<GpuResourceKey>,
    pub lifetime: GpuResourceLifetime,
    pub alias_class: Option<GpuAliasClass>,
    pub estimated_bytes: u64,
}

impl GpuResourceDesc {
    pub fn key(&self) -> GpuResourceKey {
        self.identity.key()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum GpuPassKind {
    Upload,
    Compute,
    Render,
    Composite,
    Copy,
    Present,
    Readback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct GpuPassKey(pub(crate) u64);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct GpuPassIdentity {
    pub kind: GpuPassKind,
    pub tag: u64,
    pub reads: Vec<GpuResourceKey>,
    pub writes: Vec<GpuResourceKey>,
}

impl GpuPassIdentity {
    pub fn key(&self) -> GpuPassKey {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        GpuPassKey(hasher.finish())
    }
}

/// One logical unit of GPU work, not a semantic FrameGraph node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GpuPassDesc {
    pub identity: GpuPassIdentity,
    pub after: Vec<GpuPassKey>,
    pub cacheable: bool,
    pub side_effect: bool,
}

impl GpuPassDesc {
    pub fn key(&self) -> GpuPassKey {
        self.identity.key()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct GpuPhysicalSlot(pub(crate) u32);
