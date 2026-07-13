//! ドキュメント内レイヤーの恒久ID(M2E-15 / 監査SC-3・F-7)。
//!
//! 配置はmotolii-doc。eval/プラグイン契約には露出せず、D3で解決済み参照へ落とす。

use std::collections::BTreeMap;

use serde::de::{self, Deserialize, Deserializer};
use serde::{Deserialize as DeserializeDerive, Serialize};

/// レイヤーの恒久ID。表示名とは別(表示名は`LayerIdTable`の値側)。
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, DeserializeDerive,
)]
#[serde(transparent)]
pub struct LayerId(u64);

impl LayerId {
    pub const fn get(self) -> u64 {
        self.0
    }

    /// テスト/復元用。通常の採番は`LayerIdTable::allocate`経由。
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LayerIdError {
    #[error("LayerId {id} already exists")]
    Duplicate { id: u64 },
    #[error("LayerId {id} not found")]
    NotFound { id: u64 },
    #[error("LayerId {id} is retired (below next={next}); reuse forbidden")]
    Retired { id: u64, next: u64 },
    #[error("LayerId space exhausted")]
    Exhausted,
    #[error("LayerIdTable next ({next}) must be greater than max entry id ({max_id})")]
    InvalidNext { next: u64, max_id: u64 },
}

/// レイヤーID台帳。削除後もIDを再利用しない。
///
/// スキーマ本体(クリップ/トラック)は持たない — 表示名のみを席として予約する。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayerIdTable {
    /// 次に割り当てる生値。削除しても戻さない。
    next: u64,
    /// id → 表示名(IDと別フィールド)。JSONは配列で重複を検出する。
    #[serde(serialize_with = "serialize_entries")]
    entries: BTreeMap<LayerId, String>,
}

#[derive(Serialize, DeserializeDerive)]
struct RawEntry {
    id: LayerId,
    name: String,
}

#[derive(DeserializeDerive)]
struct RawLayerIdTable {
    next: u64,
    entries: Vec<RawEntry>,
}

fn serialize_entries<S>(
    entries: &BTreeMap<LayerId, String>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(entries.len()))?;
    for (id, name) in entries {
        seq.serialize_element(&RawEntry {
            id: *id,
            name: name.clone(),
        })?;
    }
    seq.end()
}

impl<'de> Deserialize<'de> for LayerIdTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawLayerIdTable::deserialize(deserializer)?;
        LayerIdTable::try_from_raw(raw).map_err(de::Error::custom)
    }
}

impl Default for LayerIdTable {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerIdTable {
    pub fn new() -> Self {
        Self {
            next: 0,
            entries: BTreeMap::new(),
        }
    }

    fn try_from_raw(raw: RawLayerIdTable) -> Result<Self, LayerIdError> {
        let mut entries = BTreeMap::new();
        for entry in raw.entries {
            if entries.insert(entry.id, entry.name).is_some() {
                return Err(LayerIdError::Duplicate { id: entry.id.0 });
            }
        }
        Self::validate_next(raw.next, &entries)?;
        Ok(Self {
            next: raw.next,
            entries,
        })
    }

    fn validate_next(next: u64, entries: &BTreeMap<LayerId, String>) -> Result<(), LayerIdError> {
        if let Some((max_id, _)) = entries.iter().next_back() {
            if next <= max_id.0 {
                return Err(LayerIdError::InvalidNext {
                    next,
                    max_id: max_id.0,
                });
            }
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains(&self, id: LayerId) -> bool {
        self.entries.contains_key(&id)
    }

    pub fn display_name(&self, id: LayerId) -> Option<&str> {
        self.entries.get(&id).map(String::as_str)
    }

    /// 全エントリを走査する(#101 ResourceLimits の string bytes 検査用)。
    pub fn iter(&self) -> impl Iterator<Item = (LayerId, &str)> {
        self.entries.iter().map(|(id, name)| (*id, name.as_str()))
    }

    /// 新しいIDを採番して挿入する。削除済みIDは再利用しない。
    pub fn allocate(&mut self, display_name: impl Into<String>) -> Result<LayerId, LayerIdError> {
        let id = LayerId(self.next);
        if self.entries.contains_key(&id) {
            return Err(LayerIdError::Duplicate { id: id.0 });
        }
        let next = self.next.checked_add(1).ok_or(LayerIdError::Exhausted)?;
        self.entries.insert(id, display_name.into());
        self.next = next;
        Ok(id)
    }

    /// 未使用の新しいIDを明示挿入する。`id < next`は退役済みとして拒否(再利用禁止)。
    /// ロード復元は`Deserialize`が台帳を直接構築する。
    pub fn insert(
        &mut self,
        id: LayerId,
        display_name: impl Into<String>,
    ) -> Result<(), LayerIdError> {
        if self.entries.contains_key(&id) {
            return Err(LayerIdError::Duplicate { id: id.0 });
        }
        if id.0 < self.next {
            return Err(LayerIdError::Retired {
                id: id.0,
                next: self.next,
            });
        }
        // 挿入前に完了: MAXは next を進められないため拒否(Err後に表は不変)
        let floor = id.0.checked_add(1).ok_or(LayerIdError::Exhausted)?;
        self.entries.insert(id, display_name.into());
        if floor > self.next {
            self.next = floor;
        }
        Ok(())
    }

    /// 削除。採番カウンタは戻さない(再利用禁止)。
    pub fn remove(&mut self, id: LayerId) -> Result<String, LayerIdError> {
        self.entries
            .remove(&id)
            .ok_or(LayerIdError::NotFound { id: id.0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_id_serde_roundtrip() {
        let id = LayerId::from_raw(42);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "42");
        let back: LayerId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn layer_id_table_serde_roundtrip() {
        let mut table = LayerIdTable::new();
        let a = table.allocate("背景").unwrap();
        let b = table.allocate("文字").unwrap();
        table.remove(a).unwrap();
        let json = serde_json::to_string(&table).unwrap();
        let back: LayerIdTable = serde_json::from_str(&json).unwrap();
        assert_eq!(table, back);
        assert!(!back.contains(a));
        assert_eq!(back.display_name(b), Some("文字"));
        // 削除後も next が保持され、再割当が起きない
        let c = back.clone().allocate("新規").unwrap();
        assert_ne!(c, a);
        assert_eq!(c.get(), 2);
    }

    #[test]
    fn rejects_duplicate_insert() {
        let mut table = LayerIdTable::new();
        let id = LayerId::from_raw(7);
        table.insert(id, "first").unwrap();
        assert_eq!(
            table.insert(id, "second"),
            Err(LayerIdError::Duplicate { id: 7 })
        );
        assert_eq!(table.display_name(id), Some("first"));
    }

    #[test]
    fn does_not_reuse_id_after_remove() {
        let mut table = LayerIdTable::new();
        let first = table.allocate("a").unwrap();
        assert_eq!(first.get(), 0);
        table.remove(first).unwrap();
        let second = table.allocate("b").unwrap();
        assert_eq!(second.get(), 1);
        assert_ne!(second, first);
        assert!(!table.contains(first));
    }

    #[test]
    fn deserialize_rejects_next_not_above_max_entry() {
        let json = r#"{
            "next": 1,
            "entries": [{"id": 1, "name": "a"}, {"id": 0, "name": "b"}]
        }"#;
        let err = serde_json::from_str::<LayerIdTable>(json).unwrap_err();
        assert!(
            err.to_string().contains("next") && err.to_string().contains("max entry"),
            "unexpected err: {err}"
        );
    }

    #[test]
    fn deserialize_rejects_duplicate_ids() {
        let json = r#"{
            "next": 3,
            "entries": [{"id": 1, "name": "a"}, {"id": 1, "name": "b"}]
        }"#;
        let err = serde_json::from_str::<LayerIdTable>(json).unwrap_err();
        assert!(
            err.to_string().contains("already exists"),
            "unexpected err: {err}"
        );
    }

    #[test]
    fn insert_max_id_is_atomic_on_exhausted() {
        let mut table = LayerIdTable::new();
        let before_len = table.len();
        let max = LayerId::from_raw(u64::MAX);
        assert_eq!(table.insert(max, "x"), Err(LayerIdError::Exhausted));
        assert!(!table.contains(max));
        assert_eq!(table.len(), before_len);
        assert_eq!(table.next, 0);
    }

    #[test]
    fn insert_rejects_retired_id_after_remove() {
        let mut table = LayerIdTable::new();
        let id = table.allocate("a").unwrap();
        table.remove(id).unwrap();
        let next_before = table.next;
        let len_before = table.len();
        assert_eq!(
            table.insert(id, "reuse"),
            Err(LayerIdError::Retired {
                id: id.get(),
                next: next_before
            })
        );
        assert!(!table.contains(id));
        assert_eq!(table.len(), len_before);
        assert_eq!(table.next, next_before);
    }
}
