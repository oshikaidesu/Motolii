
use std::collections::BTreeMap;

use crate::doc::core::RationalTime;
use serde::de::{self, Deserialize, Deserializer};
use serde::{Deserialize as DeserializeDerive, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, DeserializeDerive,
)]
#[serde(transparent)]
pub struct AssetId(u64);

impl AssetId {
    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

impl std::fmt::Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AssetError {
    #[error("AssetId {id} already exists")]
    Duplicate { id: u64 },
    #[error("AssetId {id} not found")]
    NotFound { id: u64 },
    #[error("AssetId {id} is retired (below next={next}); reuse forbidden")]
    Retired { id: u64, next: u64 },
    #[error("AssetId space exhausted")]
    Exhausted,
    #[error("AssetTable next ({next}) must be greater than max entry id ({max_id})")]
    InvalidNext { next: u64, max_id: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetStatus {
    Unchecked,
    Present { resolved_path: String },
    Missing,
    Unreadable { reason: String },
}

impl Default for AssetStatus {
    fn default() -> Self {
        Self::Unchecked
    }
}

/* motolii-component
id = "asset.relink"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["relink", "RelinkAsset"]
meaning = ["RelinkAsset"]
evaluation = ["relink_updates_only_the_asset_path", "relink_preserves_asset_identity"]
render = ["AssetStatus", "AssetListItem"]
observable = ["relinking_a_missing_asset_makes_it_present"]
*/

#[derive(Debug, Clone, PartialEq, Eq, Serialize, DeserializeDerive)]
pub struct Asset {
    pub id: AssetId,
    pub name: String,
    pub asset_type: String,
    pub content_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_absolute: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_project_relative: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(skip)]
    pub status: AssetStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tail_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<RationalTime>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetDraft {
    pub name: String,
    pub asset_type: String,
    pub content_hash: String,
    pub path_absolute: Option<String>,
    pub path_project_relative: Option<String>,
    pub file_name: Option<String>,
    pub size_bytes: Option<u64>,
    pub head_hash: Option<String>,
    pub tail_hash: Option<String>,
    pub duration: Option<RationalTime>,
}

impl AssetDraft {
    pub fn from_probed_source(
        asset_type: impl Into<String>,
        fingerprint: &crate::doc::store::SourceFingerprintV1,
        path_absolute: &std::path::Path,
        project_root: Option<&std::path::Path>,
    ) -> Self {
        let file_name = path_absolute
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        let name = path_absolute
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .or_else(|| file_name.clone())
            .unwrap_or_else(|| path_absolute.to_string_lossy().into_owned());
        let path_project_relative = project_root
            .and_then(|root| path_absolute.strip_prefix(root).ok())
            .map(|relative| Asset::normalize_path(&relative.to_string_lossy()));
        Self {
            name,
            asset_type: asset_type.into(),
            content_hash: fingerprint.content_hash(),
            path_absolute: Some(Asset::normalize_path(&path_absolute.to_string_lossy())),
            path_project_relative,
            file_name,
            size_bytes: Some(fingerprint.size_bytes()),
            head_hash: None,
            tail_hash: None,
            duration: None,
        }
    }

    fn into_asset(self, id: AssetId) -> Asset {
        let mut asset = Asset {
            id,
            name: self.name,
            asset_type: self.asset_type,
            content_hash: self.content_hash,
            path_absolute: self.path_absolute,
            path_project_relative: self.path_project_relative,
            file_name: self.file_name,
            size_bytes: self.size_bytes,
            head_hash: self.head_hash,
            tail_hash: self.tail_hash,
            duration: self.duration,
            status: AssetStatus::default(),
        };
        asset.normalize_self();
        asset
    }
}

impl Asset {
    pub fn normalize_path(path: &str) -> String {
        path.replace('\\', "/")
    }

    pub fn resolve_status(&self, project_root: Option<&std::path::Path>) -> AssetStatus {
        if self.path_absolute.is_none() && self.path_project_relative.is_none() {
            return AssetStatus::Unchecked;
        }

        if let Some(abs) = &self.path_absolute {
            match std::fs::canonicalize(abs) {
                Ok(resolved) => {
                    return AssetStatus::Present {
                        resolved_path: Self::normalize_path(&resolved.to_string_lossy()),
                    };
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                }
                Err(err) => {
                    return AssetStatus::Unreadable {
                        reason: err.to_string(),
                    };
                }
            }
        }

        if let Some(rel) = &self.path_project_relative {
            if let Some(root) = project_root {
                let candidate = root.join(rel);
                match std::fs::canonicalize(&candidate) {
                    Ok(resolved) => {
                        return AssetStatus::Present {
                            resolved_path: Self::normalize_path(&resolved.to_string_lossy()),
                        };
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                        return AssetStatus::Missing;
                    }
                    Err(err) => {
                        return AssetStatus::Unreadable {
                            reason: err.to_string(),
                        };
                    }
                }
            }
        }

        AssetStatus::Missing
    }

    fn normalize_self(&mut self) {
        if let Some(abs) = self.path_absolute.as_mut() {
            *abs = Self::normalize_path(abs);
        }
        if let Some(rel) = self.path_project_relative.as_mut() {
            *rel = Self::normalize_path(rel);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AssetTable {
    next: u64,
    #[serde(serialize_with = "serialize_assets")]
    entries: BTreeMap<AssetId, Asset>,
}

#[derive(DeserializeDerive)]
struct RawAssetTable {
    next: u64,
    entries: Vec<Asset>,
}

fn serialize_assets<S>(entries: &BTreeMap<AssetId, Asset>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(entries.len()))?;
    for asset in entries.values() {
        seq.serialize_element(asset)?;
    }
    seq.end()
}

impl<'de> Deserialize<'de> for AssetTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawAssetTable::deserialize(deserializer)?;
        AssetTable::try_from_raw(raw).map_err(de::Error::custom)
    }
}

impl Default for AssetTable {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetTable {
    pub fn new() -> Self {
        Self {
            next: 0,
            entries: BTreeMap::new(),
        }
    }

    fn try_from_raw(raw: RawAssetTable) -> Result<Self, AssetError> {
        let mut entries = BTreeMap::new();
        for mut asset in raw.entries {
            if entries.contains_key(&asset.id) {
                return Err(AssetError::Duplicate { id: asset.id.0 });
            }
            asset.normalize_self();
            entries.insert(asset.id, asset);
        }
        if let Some((max_id, _)) = entries.iter().next_back() {
            if raw.next <= max_id.0 {
                return Err(AssetError::InvalidNext {
                    next: raw.next,
                    max_id: max_id.0,
                });
            }
        }
        Ok(Self {
            next: raw.next,
            entries,
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, id: AssetId) -> Option<&Asset> {
        self.entries.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Asset> {
        self.entries.values()
    }

    pub fn peek_next(&self) -> u64 {
        self.next
    }

    pub fn find_by_content_hash(&self, content_hash: &str) -> Option<AssetId> {
        self.entries
            .values()
            .find(|asset| asset.content_hash == content_hash)
            .map(|asset| asset.id)
    }

    pub fn admit(&mut self, draft: AssetDraft) -> Result<AssetId, AssetError> {
        if let Some(existing) = self.find_by_content_hash(&draft.content_hash) {
            return Ok(existing);
        }
        let id = AssetId(self.next);
        if self.entries.contains_key(&id) {
            return Err(AssetError::Duplicate { id: id.0 });
        }
        let next = self.next.checked_add(1).ok_or(AssetError::Exhausted)?;
        let asset = draft.into_asset(id);
        self.entries.insert(id, asset);
        self.next = next;
        Ok(id)
    }

    pub fn insert(&mut self, mut asset: Asset) -> Result<(), AssetError> {
        if self.entries.contains_key(&asset.id) {
            return Err(AssetError::Duplicate { id: asset.id.0 });
        }
        if asset.id.0 < self.next {
            return Err(AssetError::Retired {
                id: asset.id.0,
                next: self.next,
            });
        }
        let floor = asset.id.0.checked_add(1).ok_or(AssetError::Exhausted)?;
        asset.normalize_self();
        self.entries.insert(asset.id, asset);
        if floor > self.next {
            self.next = floor;
        }
        Ok(())
    }

    pub fn restore(&mut self, mut asset: Asset) -> Result<(), AssetError> {
        if self.entries.contains_key(&asset.id) {
            return Err(AssetError::Duplicate { id: asset.id.0 });
        }
        let floor = asset.id.0.checked_add(1).ok_or(AssetError::Exhausted)?;
        asset.normalize_self();
        self.entries.insert(asset.id, asset);
        if floor > self.next {
            self.next = floor;
        }
        Ok(())
    }

    pub fn remove(&mut self, id: AssetId) -> Result<Asset, AssetError> {
        self.entries
            .remove(&id)
            .ok_or(AssetError::NotFound { id: id.0 })
    }

    pub fn relink(
        &mut self,
        id: AssetId,
        path_absolute: &std::path::Path,
        project_root: Option<&std::path::Path>,
    ) -> Result<(), AssetError> {
        let asset = self
            .entries
            .get_mut(&id)
            .ok_or(AssetError::NotFound { id: id.0 })?;
        asset.path_absolute = Some(Asset::normalize_path(&path_absolute.to_string_lossy()));
        asset.path_project_relative = project_root
            .and_then(|root| path_absolute.strip_prefix(root).ok())
            .map(|relative| Asset::normalize_path(&relative.to_string_lossy()));
        asset.file_name = path_absolute
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        Ok(())
    }
}
