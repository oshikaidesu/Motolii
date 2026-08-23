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

/// 「いま参照できるかどうか」— **環境の事実であって作品の内容ではない**
/// (A05: `next/reference/axis/A05-missing.tsv` の2行目、`next/reference/procedures/P3`
/// 後半の「別マシンで開く」動線)。同じ project でも、開くマシン・その瞬間の
/// ディスク状態によって変わりうる値なので **Document には入れない**(=このフィールドは
/// `#[serde(skip)]` — 保存 JSON には一度も書かれない。既存 project ファイルは
/// このフィールドの有無に関わらずバイト単位で不変)。`Asset::resolve_status` を
/// 呼んだ側だけが更新する、読み込み直後は必ず [`AssetStatus::Unchecked`]。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetStatus {
    /// `resolve_status` をまだ呼んでいない。または呼んでも判定しようがない
    /// (パスを1つも持たない = ファイル実体を伴わない素材、生成系など)。
    /// **「在る」と偽らない既定値**(判断が割れたら厳しい側へ — Present を既定にしない)。
    Unchecked,
    /// 絶対 → 相対の順で解決に成功した。実際に使えたパスを保持する
    /// (`/` 区切りへ正規化済み、[`Asset::normalize_path`] と同じ規約)。
    Present { resolved_path: String },
    /// 絶対・相対のどちらの経路でも見つからなかった(`io::ErrorKind::NotFound`)。
    Missing,
    /// パスは見えたが読めなかった(権限拒否・ループ・NotFound 以外の IO 種別)。
    /// **理由を握りつぶさず持ち歩く**(黙って近似しない)。
    Unreadable { reason: String },
}

impl Default for AssetStatus {
    fn default() -> Self {
        Self::Unchecked
    }
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
    /// 「いま参照できるか」の環境事実。**保存しない**([`AssetStatus`] doc 参照)。
    /// 新規 [`Asset`] は常に `Unchecked` から始まり、[`Asset::resolve_status`] を
    /// 呼んだ側だけが更新する。
    #[serde(skip)]
    pub status: AssetStatus,
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

    /// 「いま参照できるか」を実際に IO で確かめる(A05: パス解決の経路を1本に揃える)。
    ///
    /// 順序は**絶対 → 相対 → 失敗**([`AssetStatus`] doc・A05-missing.tsv の設計判断)。
    /// `project_root` は `path_project_relative` の起点(通常は project file の
    /// 置き場所)。どちらのパスも持たない asset(生成系など)は [`AssetStatus::Unchecked`]
    /// を返す — 「無い」と「確かめようがない」を混同しない。
    ///
    /// 借用元: `std::fs::canonicalize`(標準ライブラリ、裁定215 (A) — 「上流に既にある」の
    /// 最も単純な例。symlink 解決込みの実在確認を自前で書く理由がない)。
    ///
    /// **失敗を握りつぶさない**: `NotFound` は `Missing` として明示的に返し、それ以外の
    /// `io::Error`(権限拒否・ループ等)は理由文字列ごと `Unreadable` として返す。
    /// 呼び出し側が `Result` の代わりにこの enum を受け取る形なのは、「解決できない」
    /// こと自体が異常系ではなく素材の状態そのものだから(`?` で伝播させたくない)。
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
                    // 絶対パスで見つからない → 相対パスへ倒す。
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

        // 絶対パスが無かった/相対で解決できなかった、かつ他に試す経路が無い。
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
                status: AssetStatus::default(),
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

    /// テスト専用の使い捨てディレクトリ(`tempfile` crate を足さない — 標準ライブラリの
    /// `env::temp_dir` + プロセス/時刻由来の一意名だけで足りる、裁定215 の「借りる」)。
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

    /// 絶対パスが見つからない時だけ相対パスへ倒す(**絶対 → 相対 → 失敗**の順序)。
    #[test]
    fn resolve_status_falls_back_to_relative_path_when_absolute_is_gone() {
        let dir = unique_scratch_dir("relative");
        let file = dir.join("clip.mp4");
        std::fs::write(&file, b"payload").unwrap();
        let canonical = std::fs::canonicalize(&file).unwrap();

        let asset = blank_asset(
            // 絶対パスは存在しない場所を指す(取り込み後にファイルが動いた想定)。
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
        // project_root が無い(=どこからの相対か分からない)なら試しようがない → Missing。
        let asset = blank_asset(None, Some("clip.mp4".into()));
        assert_eq!(asset.resolve_status(None), AssetStatus::Missing);
    }

    /// **後方互換の柵**: `status` を持たない旧形式 JSON(このフィールドが存在しなかった
    /// 頃に保存された project)がそのまま読める。`#[serde(skip)]` は無い入力を拒否せず
    /// `Default`(`Unchecked`)で埋める。
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

        // 書き戻しても `status` キーは決して現れない(バイト単位で旧形式のまま)。
        let rewritten = serde_json::to_value(&asset).unwrap();
        assert!(rewritten.get("status").is_none());
    }

    /// `AssetTable` 全体の往復でも `status` は書き出されない — 既存 project の
    /// バイト列を変えない、という `duration` フィールドと同じ互換方針の確認。
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
