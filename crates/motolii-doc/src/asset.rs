//! Asset一般定義(F-10 / 実装ガード10)。
//!
//! D1aはパス+type+content_hashのメタのみ。opaqueペイロード本体はImporterが作り
//! GpuAssetCacheが持つ。Documentは多重キーでファイル実体を指す。

use std::collections::BTreeMap;

use serde::de::{self, Deserialize, Deserializer};
use serde::{Deserialize as DeserializeDerive, Serialize};

/// アセットの恒久ID。表示名は別フィールド。
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

/// パスは常に `/` 区切りへ正規化して保持する(クロスOS roundtrip)。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, DeserializeDerive)]
pub struct Asset {
    pub id: AssetId,
    pub name: String,
    /// opaque type文字列(例: `video/mp4`, `image/svg+xml`, `pointcloud.octree.v1`)。
    pub asset_type: String,
    /// 内容ハッシュ(ホストが計算。コアは解釈しない)。
    pub content_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_absolute: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_project_relative: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tail_hash: Option<String>,
}

impl Asset {
    pub fn normalize_path(path: &str) -> String {
        path.replace('\\', "/")
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

/// アセット台帳。削除後もIDを再利用しない(LayerIdと同型)。
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

    /// 全エントリを走査する(#101 ResourceLimits の string bytes 検査用)。
    pub fn iter(&self) -> impl Iterator<Item = &Asset> {
        self.entries.values()
    }

    pub fn allocate(
        &mut self,
        name: impl Into<String>,
        asset_type: impl Into<String>,
        content_hash: impl Into<String>,
    ) -> Result<AssetId, AssetError> {
        let id = AssetId(self.next);
        // LayerIdTableと同型の二重防御(next不変条件が破れた場合の安全網)
        if self.entries.contains_key(&id) {
            return Err(AssetError::Duplicate { id: id.0 });
        }
        let next = self.next.checked_add(1).ok_or(AssetError::Exhausted)?;
        let asset = Asset {
            id,
            name: name.into(),
            asset_type: asset_type.into(),
            content_hash: content_hash.into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: None,
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
        };
        self.entries.insert(id, asset);
        self.next = next;
        Ok(id)
    }

    /// 既存IDで挿入。`id < next`は退役済みとして拒否(再利用禁止)。
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

    /// 削除。採番カウンタは戻さない(再利用禁止)。
    pub fn remove(&mut self, id: AssetId) -> Result<Asset, AssetError> {
        self.entries
            .remove(&id)
            .ok_or(AssetError::NotFound { id: id.0 })
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
        let id = table.allocate("a", "video/mp4", "h").unwrap();
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
            }),
            Err(AssetError::Retired {
                id: id.get(),
                next: next_before
            })
        );
        assert_eq!(table.next, next_before);
    }

    #[test]
    fn asset_table_roundtrip_keeps_multi_keys() {
        let mut table = AssetTable::new();
        let id = table.allocate("intro", "video/mp4", "sha256:abc").unwrap();
        table.remove(id).unwrap();
        // 新しいIDで多重キー付きを insert
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
            })
            .unwrap();

        let json = serde_json::to_string(&table).unwrap();
        let back: AssetTable = serde_json::from_str(&json).unwrap();
        let a = back.get(id2).unwrap();
        assert_eq!(a.path_absolute.as_deref(), Some("D:/media/intro.mp4"));
        assert_eq!(a.path_project_relative.as_deref(), Some("media/intro.mp4"));
    }
}
