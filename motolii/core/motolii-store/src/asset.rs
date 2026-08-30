
use std::collections::BTreeMap;

use motolii_core::RationalTime;
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
        fingerprint: &crate::SourceFingerprintV1,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_normalization_uses_forward_slash() {
        assert_eq!(Asset::normalize_path(r"C:\proj\a.mp4"), "C:/proj/a.mp4");
    }

    #[test]
    fn insert_rejects_retired_id_after_remove() {
        let mut table = AssetTable::new();
        let id = table
            .admit(AssetDraft {
                name: "a".into(),
                asset_type: "video/mp4".into(),
                content_hash: "h".into(),
                path_absolute: None,
                path_project_relative: None,
                file_name: None,
                size_bytes: None,
                head_hash: None,
                tail_hash: None,
                duration: None,
            })
            .unwrap();
        table.remove(id).unwrap();
        let next_before = table.next;
        assert_eq!(
            table.insert(Asset {
                id,
                name: "reuse".into(),
                asset_type: "video/mp4".into(),
                content_hash: "h".into(),
                path_absolute: None,
                path_project_relative: None,
                file_name: None,
                size_bytes: None,
                head_hash: None,
                tail_hash: None,
                duration: None,
                status: AssetStatus::default(),
            }),
            Err(AssetError::Retired {
                id: id.get(),
                next: next_before
            })
        );
        assert_eq!(table.next, next_before);
    }

    #[test]
    fn restore_reinstates_identity_without_rewinding_next() {
        let mut table = AssetTable::new();
        assert_eq!(table.peek_next(), 0);

        let id = table
            .admit(AssetDraft {
                name: "a".into(),
                asset_type: "video/mp4".into(),
                content_hash: "h".into(),
                path_absolute: None,
                path_project_relative: None,
                file_name: None,
                size_bytes: None,
                head_hash: None,
                tail_hash: None,
                duration: None,
            })
            .unwrap();
        let asset = table.remove(id).unwrap();
        assert_eq!(table.peek_next(), 1);

        table.restore(asset.clone()).unwrap();
        assert_eq!(table.get(id), Some(&asset));
        assert_eq!(table.peek_next(), 1);
        assert_eq!(
            table.restore(asset),
            Err(AssetError::Duplicate { id: id.get() })
        );
        assert_eq!(table.peek_next(), 1);

        let future = Asset {
            id: AssetId::from_raw(3),
            name: "future".into(),
            asset_type: "image/png".into(),
            content_hash: "future-hash".into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: None,
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
            duration: None,
            status: AssetStatus::default(),
        };
        table.restore(future).unwrap();
        assert_eq!(table.peek_next(), 4);
        assert_eq!(
            table
                .admit(AssetDraft {
                    name: "next".into(),
                    asset_type: "image/png".into(),
                    content_hash: "next-hash".into(),
                    path_absolute: None,
                    path_project_relative: None,
                    file_name: None,
                    size_bytes: None,
                    head_hash: None,
                    tail_hash: None,
                    duration: None,
                })
                .unwrap(),
            AssetId::from_raw(4)
        );
    }

    #[test]
    fn asset_table_roundtrip_keeps_multi_keys() {
        let mut table = AssetTable::new();
        let id = table
            .admit(AssetDraft {
                name: "intro".into(),
                asset_type: "video/mp4".into(),
                content_hash: "sha256:abc".into(),
                path_absolute: None,
                path_project_relative: None,
                file_name: None,
                size_bytes: None,
                head_hash: None,
                tail_hash: None,
                duration: None,
            })
            .unwrap();
        table.remove(id).unwrap();
        let id2 = AssetId::from_raw(1);
        table
            .insert(Asset {
                id: id2,
                name: "intro".into(),
                asset_type: "video/mp4".into(),
                content_hash: "sha256:abc".into(),
                path_absolute: Some(r"D:\media\intro.mp4".into()),
                path_project_relative: Some("media\\intro.mp4".into()),
                file_name: Some("intro.mp4".into()),
                size_bytes: Some(1024),
                head_hash: Some("h".into()),
                tail_hash: Some("t".into()),
                duration: None,
                status: AssetStatus::default(),
            })
            .unwrap();

        let json = serde_json::to_string(&table).unwrap();
        let back: AssetTable = serde_json::from_str(&json).unwrap();
        let a = back.get(id2).unwrap();
        assert_eq!(a.path_absolute.as_deref(), Some("D:/media/intro.mp4"));
        assert_eq!(a.path_project_relative.as_deref(), Some("media/intro.mp4"));
    }

    #[test]
    fn admit_deduplicates_by_content_hash() {
        let mut table = AssetTable::new();
        let draft = || AssetDraft {
            name: "clip".into(),
            asset_type: "video/mp4".into(),
            content_hash: "sha256:same".into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: None,
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
            duration: None,
        };
        let first = table.admit(draft()).unwrap();
        let second = table.admit(draft()).unwrap();
        assert_eq!(first, second);
        assert_eq!(table.len(), 1);
    }

    fn unique_scratch_dir(tag: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "motolii-store-asset-test-{tag}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn blank_asset(path_absolute: Option<String>, path_project_relative: Option<String>) -> Asset {
        Asset {
            id: AssetId::from_raw(0),
            name: "a".into(),
            asset_type: "video/mp4".into(),
            content_hash: "h".into(),
            path_absolute,
            path_project_relative,
            file_name: None,
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
            duration: None,
            status: AssetStatus::default(),
        }
    }

    fn table_with_asset() -> (AssetTable, AssetId) {
        let mut table = AssetTable::new();
        let id = table
            .admit(AssetDraft {
                name: "original".into(),
                asset_type: "video/mp4".into(),
                content_hash: "sha256:identity".into(),
                path_absolute: Some("/project/missing/original.mp4".into()),
                path_project_relative: Some("missing/original.mp4".into()),
                file_name: Some("original.mp4".into()),
                size_bytes: Some(42),
                head_hash: None,
                tail_hash: None,
                duration: None,
            })
            .unwrap();
        (table, id)
    }

    #[test]
    fn relink_updates_only_the_asset_path() {
        let (mut table, id) = table_with_asset();
        table
            .relink(
                id,
                std::path::Path::new("/project/media/found.mp4"),
                Some(std::path::Path::new("/project")),
            )
            .unwrap();
        let asset = table.get(id).unwrap();
        assert_eq!(asset.path_absolute.as_deref(), Some("/project/media/found.mp4"));
        assert_eq!(asset.path_project_relative.as_deref(), Some("media/found.mp4"));
        assert_eq!(asset.file_name.as_deref(), Some("found.mp4"));
        assert_eq!(asset.name, "original");
        assert_eq!(asset.content_hash, "sha256:identity");
        assert_eq!(asset.size_bytes, Some(42));
    }

    #[test]
    fn relink_preserves_asset_identity() {
        let (mut table, id) = table_with_asset();
        table
            .relink(id, std::path::Path::new("/elsewhere/found.mp4"), None)
            .unwrap();
        let asset = table.get(id).unwrap();
        assert_eq!(asset.id, id);
        assert_eq!(asset.content_hash, "sha256:identity");
        assert_eq!(asset.asset_type, "video/mp4");
    }

    #[test]
    fn relinking_a_missing_asset_makes_it_present() {
        let dir = unique_scratch_dir("relink");
        let file = dir.join("found.mp4");
        std::fs::write(&file, b"payload").unwrap();
        let (mut table, id) = table_with_asset();
        table.relink(id, &file, Some(&dir)).unwrap();
        assert!(matches!(
            table.get(id).unwrap().resolve_status(Some(&dir)),
            AssetStatus::Present { .. }
        ));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn resolve_status_is_unchecked_without_any_path() {
        let asset = blank_asset(None, None);
        assert_eq!(asset.resolve_status(None), AssetStatus::Unchecked);
    }

    #[test]
    fn resolve_status_present_via_absolute_path() {
        let dir = unique_scratch_dir("absolute");
        let file = dir.join("clip.mp4");
        std::fs::write(&file, b"payload").unwrap();
        let canonical = std::fs::canonicalize(&file).unwrap();

        let asset = blank_asset(Some(file.to_string_lossy().into_owned()), None);
        let status = asset.resolve_status(None);
        assert_eq!(
            status,
            AssetStatus::Present {
                resolved_path: Asset::normalize_path(&canonical.to_string_lossy())
            }
        );

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn resolve_status_falls_back_to_relative_path_when_absolute_is_gone() {
        let dir = unique_scratch_dir("relative");
        let file = dir.join("clip.mp4");
        std::fs::write(&file, b"payload").unwrap();
        let canonical = std::fs::canonicalize(&file).unwrap();

        let asset = blank_asset(
            Some(
                dir.join("moved-away")
                    .join("clip.mp4")
                    .to_string_lossy()
                    .into_owned(),
            ),
            Some("clip.mp4".into()),
        );
        let status = asset.resolve_status(Some(&dir));
        assert_eq!(
            status,
            AssetStatus::Present {
                resolved_path: Asset::normalize_path(&canonical.to_string_lossy())
            }
        );

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn resolve_status_missing_when_neither_path_resolves() {
        let dir = unique_scratch_dir("missing");
        let asset = blank_asset(
            Some(dir.join("gone.mp4").to_string_lossy().into_owned()),
            Some("also-gone.mp4".into()),
        );
        assert_eq!(asset.resolve_status(Some(&dir)), AssetStatus::Missing);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn resolve_status_missing_when_only_relative_given_but_no_project_root() {
        let asset = blank_asset(None, Some("clip.mp4".into()));
        assert_eq!(asset.resolve_status(None), AssetStatus::Missing);
    }

    #[test]
    fn asset_deserializes_from_pre_status_field_json() {
        let legacy_json = r#"{
            "id": 7,
            "name": "intro",
            "asset_type": "video/mp4",
            "content_hash": "sha256:abc",
            "path_absolute": "/media/intro.mp4",
            "path_project_relative": "media/intro.mp4",
            "file_name": "intro.mp4",
            "size_bytes": 1024
        }"#;
        let asset: Asset = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(asset.id, AssetId::from_raw(7));
        assert_eq!(asset.status, AssetStatus::Unchecked);

        let rewritten = serde_json::to_value(&asset).unwrap();
        assert!(rewritten.get("status").is_none());
    }

    #[test]
    fn asset_table_roundtrip_never_serializes_status() {
        let mut table = AssetTable::new();
        table
            .admit(AssetDraft {
                name: "clip".into(),
                asset_type: "video/mp4".into(),
                content_hash: "sha256:roundtrip".into(),
                path_absolute: None,
                path_project_relative: None,
                file_name: None,
                size_bytes: None,
                head_hash: None,
                tail_hash: None,
                duration: None,
            })
            .unwrap();

        let json = serde_json::to_string(&table).unwrap();
        assert!(!json.contains("status"));
        let back: AssetTable = serde_json::from_str(&json).unwrap();
        assert_eq!(back, table);
    }
}
