//! Document 所有の素材台帳(裁定162: Browser の一覧の正本 = この台帳)。
//!
//! 旧 workspace `crates/motolii-doc/src/asset.rs:131-455` からの移植(2026-08-21)。
//! 再実装ではない — `AssetId`/`AssetError`/`Asset`/`AssetDraft`/`AssetTable` を
//! そのまま運ぶ。`SourceFingerprintV1`(内容の指紋、同ファイルの旧14-128行)は
//! 既に `fingerprint.rs` へ移植済み(2026-08-20 リセット)なので、ここでは
//! [`crate::SourceFingerprintV1`] を使う側として参照するだけで作り直さない。
//!
//! D1a はパス+type+content_hash のメタのみ。opaque ペイロード本体は Importer が作り
//! GpuAssetCache が持つ(旧文書の注記そのまま — 意味は変わらない)。Document は
//! 多重キーでファイル実体を指す。
//!
//! **bin-first**(取り込んでから配置する、AE/Premiere/Resolve 共通のワークフロー)を
//! Document が表現できるようにするための台帳(裁定162 の問い: 取り込んだが未配置の
//! 素材の置き場が next の store に無かった)。[`crate::LayerSource::Media`](裁定79)は
//! 「配置済み layer が指す素材」であって、取り込んだが未配置の素材の置き場ではない —
//! この2つは別の関心事であり、この切片(裁定162 の第一波)は台帳とその読み口までで、
//! `LayerSource::Media` との参照統合はしない(後続裁定)。

use std::collections::BTreeMap;

use motolii_core::RationalTime;
use serde::de::{self, Deserialize, Deserializer};
use serde::{Deserialize as DeserializeDerive, Serialize};

/// アセットの恒久 ID。表示名は別フィールド。
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

/// パスは常に `/` 区切りへ正規化して保持する(クロス OS roundtrip)。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, DeserializeDerive)]
pub struct Asset {
    pub id: AssetId,
    pub name: String,
    /// opaque type 文字列(例: `video/mp4`, `image/svg+xml`, `pointcloud.octree.v1`)。
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
    /// 素材そのものの長さ(probe が測った container 総尺)。**分かる時だけ**入る —
    /// 生成系・stream など尺を持たない素材は `None` のまま。
    ///
    /// **空なら書き出さない**ので、旧文書のバイト列は変わらず、旧 reader は未知キーとして
    /// 往復する(ロケータ/`resolution` と同じ互換方針)。版も上げない。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<RationalTime>,
}

/// 新規 Asset を準備するための非永続 payload。`AssetId` は台帳(`AssetTable::admit`、
/// `Intent::AdmitAsset` の書き口)が付与する。
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
    /// probe が測った素材の総尺。既定 `None` — 呼び手(CLI import / GUI drop)が
    /// `probe_admission_source` の値を入れる。
    pub duration: Option<RationalTime>,
}

impl AssetDraft {
    /// probe 済み source の plain 値から新規 file-backed draft を組む(IO なし・純関数)。
    ///
    /// - `content_hash`/`size_bytes` は正準 `SourceFingerprintV1`
    ///   (`motolii-source-v1:sha256:<64 lowercase hex>` + exact size)から写す
    /// - `head_hash`/`tail_hash` は legacy hint(2026-08-08 serial-core 決定で
    ///   identity authority から退役済み)であり、新規 admission では発行しない
    /// - `name` は file stem、`file_name` は file name
    /// - `path_project_relative` は `path_absolute` が `project_root` 配下の時だけ
    ///   純粋な prefix 計算で入れる(ファイルコピー・実在確認はしない)
    ///
    /// probe 自体(ffprobe/hash IO)はホスト側が持つ。ここは plain 値のみ。
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
            // 尺は plain 値の probe 結果であり、この純関数の入力には無い。
            // 知っている呼び手(admission 経路)が後から入れる。
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
        };
        asset.normalize_self();
        asset
    }
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

/// アセット台帳。削除後も ID を再利用しない(`LayerId` と同型)。
///
/// **Document 所有**(裁定162)— `Intent::AdmitAsset`/`Intent::RemoveAsset` を通じて
/// `Composition:assets` component へ丸ごと JSON で書く(`markers`/`slots` と同じ
/// 「1 component = 1 表」の流儀、`components.rs::descriptor_assets` 参照)。
/// undo/redo は edit timeline の latest-at 移動そのもの(`document.rs` の crate doc
/// 参照)なので、この型自身は自前の履歴機構を持たない — `insert`/`restore` は
/// 旧台帳(自前の undo を持っていた)の呼び出し口をそのまま残しているが、
/// この crate の書き口(`Document::write`)が実際に使うのは `admit`/`remove` だけ。
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

    /// 全エントリを走査する(`AssetId` 昇順、`BTreeMap` の内部順そのまま)。
    pub fn iter(&self) -> impl Iterator<Item = &Asset> {
        self.entries.values()
    }

    /// 次に採番される生値(エントリは作らない)。
    pub fn peek_next(&self) -> u64 {
        self.next
    }

    /// `content_hash` が一致する既存 asset を探す(裁定162 の重複統合の下地 —
    /// [`Self::admit`] が呼ぶ)。
    pub fn find_by_content_hash(&self, content_hash: &str) -> Option<AssetId> {
        self.entries
            .values()
            .find(|asset| asset.content_hash == content_hash)
            .map(|asset| asset.id)
    }

    /// `draft` を台帳へ迎え入れる。**同一 `content_hash` の draft は台帳を増やさず
    /// 既存 id を返す**(裁定162: 「同じファイルをもう一度 import した」を2件目の
    /// エントリにしない — 旧台帳がフィンガープリントで持っていた重複統合の意味)。
    pub fn admit(&mut self, draft: AssetDraft) -> Result<AssetId, AssetError> {
        if let Some(existing) = self.find_by_content_hash(&draft.content_hash) {
            return Ok(existing);
        }
        let id = AssetId(self.next);
        // LayerIdTable と同型の二重防御(next 不変条件が破れた場合の安全網)。
        if self.entries.contains_key(&id) {
            return Err(AssetError::Duplicate { id: id.0 });
        }
        let next = self.next.checked_add(1).ok_or(AssetError::Exhausted)?;
        let asset = draft.into_asset(id);
        self.entries.insert(id, asset);
        self.next = next;
        Ok(id)
    }

    /// 既存 ID で挿入。`id < next` は退役済みとして拒否(再利用禁止)。
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

    /// Undo/Redo 用: 退役済み(`id < next`)でも同じ Asset を台帳へ戻す。
    /// 通常の新規挿入には使わず、採番カウンタは巻き戻さない。
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
        // 新しい ID で多重キー付きを insert
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
            })
            .unwrap();

        let json = serde_json::to_string(&table).unwrap();
        let back: AssetTable = serde_json::from_str(&json).unwrap();
        let a = back.get(id2).unwrap();
        assert_eq!(a.path_absolute.as_deref(), Some("D:/media/intro.mp4"));
        assert_eq!(a.path_project_relative.as_deref(), Some("media/intro.mp4"));
    }

    /// 裁定162 の重複統合: 同一 `content_hash` の2度目の `admit` は台帳を増やさず、
    /// 1度目と同じ id を返す。
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
}
