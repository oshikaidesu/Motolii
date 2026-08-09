//! Asset一般定義(F-10 / 実装ガード10)。
//!
//! D1aはパス+type+content_hashのメタのみ。opaqueペイロード本体はImporterが作り
//! GpuAssetCacheが持つ。Documentは多重キーでファイル実体を指す。

use std::collections::BTreeMap;
use std::io::{self, Read};

use serde::de::{self, Deserialize, Deserializer};
use serde::{Deserialize as DeserializeDerive, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFingerprintV1 {
    digest: [u8; 32],
    size_bytes: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum SourceFingerprintError {
    #[error("failed reading source for fingerprint: {source}")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("byte count overflowed u64 during fingerprinting")]
    ByteCountOverflow,
}

impl SourceFingerprintV1 {
    pub fn from_reader<R: Read>(mut reader: R) -> Result<Self, SourceFingerprintError> {
        let mut hasher = Sha256::new();
        let mut total = 0u64;
        let mut buffer = [0u8; 8192];

        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
            total = total
                .checked_add(count as u64)
                .ok_or(SourceFingerprintError::ByteCountOverflow)?;
        }

        let digest = hasher.finalize().into();

        Ok(Self {
            digest,
            size_bytes: total,
        })
    }

    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    pub fn content_hash(&self) -> String {
        use std::fmt::Write as _;
        let mut hash = String::with_capacity(89);
        hash.push_str("motolii-source-v1:sha256:");
        for byte in &self.digest {
            write!(&mut hash, "{byte:02x}").expect("writing to String cannot fail");
        }
        hash
    }
}

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
    use std::io::{self, Error, ErrorKind, Read};

    struct OneByteAtATimeReader {
        source: Vec<u8>,
        cursor: usize,
    }

    impl OneByteAtATimeReader {
        fn new(data: impl Into<Vec<u8>>) -> Self {
            Self {
                source: data.into(),
                cursor: 0,
            }
        }
    }

    impl Read for OneByteAtATimeReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.cursor >= self.source.len() {
                return Ok(0);
            }
            if buf.is_empty() {
                return Ok(0);
            }
            buf[0] = self.source[self.cursor];
            self.cursor += 1;
            Ok(1)
        }
    }

    struct FailingReader {
        fail_with: Error,
        first_call: bool,
    }

    impl FailingReader {
        fn new(kind: ErrorKind, message: &str) -> Self {
            Self {
                fail_with: Error::new(kind, message.to_string()),
                first_call: false,
            }
        }
    }

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            if self.first_call {
                Ok(0)
            } else {
                self.first_call = true;
                Err(Error::new(
                    self.fail_with.kind(),
                    self.fail_with.to_string(),
                ))
            }
        }
    }

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

    #[test]
    fn source_fingerprint_empty_bytes() {
        let fp = SourceFingerprintV1::from_reader(io::Cursor::new(Vec::<u8>::new())).unwrap();
        assert_eq!(fp.content_hash(), "motolii-source-v1:sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(fp.size_bytes(), 0);
    }

    #[test]
    fn source_fingerprint_abc_bytes() {
        let fp = SourceFingerprintV1::from_reader(io::Cursor::new(b"abc")).unwrap();
        assert_eq!(fp.content_hash(), "motolii-source-v1:sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(fp.size_bytes(), 3);
    }

    #[test]
    fn source_fingerprint_one_byte_chunked_reader() {
        let fp = SourceFingerprintV1::from_reader(OneByteAtATimeReader::new(b"abc")).unwrap();
        assert_eq!(fp.content_hash(), "motolii-source-v1:sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(fp.size_bytes(), 3);
    }

    #[test]
    fn source_fingerprint_reader_error_preserves_kind_and_message() {
        let kind = ErrorKind::Other;
        let message = "reader failed";
        let err = SourceFingerprintV1::from_reader(FailingReader::new(kind, message)).unwrap_err();
        assert!(matches!(err, SourceFingerprintError::Io { .. }));
        let io_err = match err {
            SourceFingerprintError::Io { source } => source,
            SourceFingerprintError::ByteCountOverflow => unreachable!(),
        };
        assert_eq!(io_err.kind(), kind);
        assert_eq!(io_err.to_string(), message.to_string());
    }
}
