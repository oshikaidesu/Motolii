//! トラックの恒久ID台帳。LayerIdと同型(再利用禁止)。

use std::collections::BTreeMap;

use serde::de::{self, Deserialize, Deserializer};
use serde::{Deserialize as DeserializeDerive, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, DeserializeDerive,
)]
#[serde(transparent)]
pub struct TrackId(u64);

impl TrackId {
    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TrackIdError {
    #[error("TrackId {id} already exists")]
    Duplicate { id: u64 },
    #[error("TrackId {id} not found")]
    NotFound { id: u64 },
    #[error("TrackId {id} is retired (below next={next}); reuse forbidden")]
    Retired { id: u64, next: u64 },
    #[error("TrackId space exhausted")]
    Exhausted,
    #[error("TrackIdTable next ({next}) must be greater than max entry id ({max_id})")]
    InvalidNext { next: u64, max_id: u64 },
}

/// トラックID台帳。表示名はIDと別。削除後も再利用しない。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrackIdTable {
    next: u64,
    #[serde(serialize_with = "serialize_entries")]
    entries: BTreeMap<TrackId, String>,
}

#[derive(Serialize, DeserializeDerive)]
struct RawEntry {
    id: TrackId,
    name: String,
}

#[derive(DeserializeDerive)]
struct RawTrackIdTable {
    next: u64,
    entries: Vec<RawEntry>,
}

fn serialize_entries<S>(
    entries: &BTreeMap<TrackId, String>,
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

impl<'de> Deserialize<'de> for TrackIdTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawTrackIdTable::deserialize(deserializer)?;
        TrackIdTable::try_from_raw(raw).map_err(de::Error::custom)
    }
}

impl Default for TrackIdTable {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackIdTable {
    pub fn new() -> Self {
        Self {
            next: 0,
            entries: BTreeMap::new(),
        }
    }

    fn try_from_raw(raw: RawTrackIdTable) -> Result<Self, TrackIdError> {
        let mut entries = BTreeMap::new();
        for entry in raw.entries {
            if entries.insert(entry.id, entry.name).is_some() {
                return Err(TrackIdError::Duplicate { id: entry.id.0 });
            }
        }
        if let Some((max_id, _)) = entries.iter().next_back() {
            if raw.next <= max_id.0 {
                return Err(TrackIdError::InvalidNext {
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

    pub fn display_name(&self, id: TrackId) -> Option<&str> {
        self.entries.get(&id).map(String::as_str)
    }

    pub fn allocate(&mut self, display_name: impl Into<String>) -> Result<TrackId, TrackIdError> {
        let id = TrackId(self.next);
        let next = self.next.checked_add(1).ok_or(TrackIdError::Exhausted)?;
        self.entries.insert(id, display_name.into());
        self.next = next;
        Ok(id)
    }

    pub fn insert(
        &mut self,
        id: TrackId,
        display_name: impl Into<String>,
    ) -> Result<(), TrackIdError> {
        if self.entries.contains_key(&id) {
            return Err(TrackIdError::Duplicate { id: id.0 });
        }
        if id.0 < self.next {
            return Err(TrackIdError::Retired {
                id: id.0,
                next: self.next,
            });
        }
        let floor = id.0.checked_add(1).ok_or(TrackIdError::Exhausted)?;
        self.entries.insert(id, display_name.into());
        if floor > self.next {
            self.next = floor;
        }
        Ok(())
    }

    pub fn remove(&mut self, id: TrackId) -> Result<String, TrackIdError> {
        self.entries
            .remove(&id)
            .ok_or(TrackIdError::NotFound { id: id.0 })
    }

    pub fn contains(&self, id: TrackId) -> bool {
        self.entries.contains_key(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_not_reuse_id_after_remove() {
        let mut table = TrackIdTable::new();
        let a = table.allocate("V1").unwrap();
        table.remove(a).unwrap();
        let b = table.allocate("V2").unwrap();
        assert_ne!(a, b);
        assert_eq!(b.get(), 1);
    }
}
