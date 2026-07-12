//! motolii-doc: ドキュメント所有権骨格(F-2) + D1-prelude(M2E-12) + D1aスキーマ + D1b検証 + D1c永続化。
//!
//! 読み手(レンダ・書き出し・解析)は`Arc<Document>`スナップショットのみを受け取る。
//! `edit`は戻り値を持たない — 参照漏洩で凍結を封じないため。
//!
//! **D1a**: スキーマ本体。**D1b**: 保存前`validate`(ガード1)。**D1c**: アトミック保存/読込。**D1d**: ジャーナル。

mod asset;
mod bpm;
mod ids;
mod journal;
mod param;
mod persist;
mod schema;
mod track_id;
mod validate;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub use asset::{Asset, AssetError, AssetId, AssetTable};
pub use bpm::{Bpm, BpmError};
pub use ids::{LayerId, LayerIdError, LayerIdTable};
pub use journal::{
    edit_payload, inject_bad_checksum_at_last_frame, inject_clear_fingerprint,
    inject_corrupt_catalog, inject_corrupt_journal_tail, inject_corrupt_main,
    inject_orphan_snapshot_after_first_frame, inject_salt_mismatch_frame, load_catalog,
    open_project, save_project_with_journal, scan_journal, GenerationCatalog, GenerationEntry,
    JournalEdit, JournalFrame, JournalHeader, JournalRecordKind, JournalScanOutcome,
    JournalScanStop, OpenProjectOutcome, PinGenerationOptions, ProjectError, RecoverySource,
    ReplayFailure, ReplayOutcome, RotateOptions, SaveProjectOptions, ScanJournalOptions,
};
pub use param::{DocParam, LookAtAxis};
pub use persist::{
    detect_cloud_sync, load_document, load_document_bytes, save_document,
    save_document_with_options, CloudSyncHint, PersistError, SaveAbortAfter, SaveOptions,
    READER_VERSION,
};
pub use schema::{
    BlendMode, Clip, ClipSource, ClippingMaskSettings, Composition, CompositionError,
    EffectInstance, Group, ItemEnvelope, MaskMode, PathOp, Soundtrack, SoundtrackError, Track,
    TrackItem, Transform2D,
};
pub use track_id::{TrackId, TrackIdError, TrackIdTable};
pub use validate::DocumentError;

fn default_min_reader_version() -> u32 {
    1
}

/// プロジェクト状態。`ProjectV1`とは非継承・版番独立(M2E-11①)。
///
/// `CompCamera`は含めない(#55)。未知キーは`extra`に保持し再保存で書き戻す。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub version: u32,
    #[serde(default = "default_min_reader_version")]
    pub min_reader_version: u32,
    pub composition: Composition,
    pub bpm: Bpm,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soundtrack: Option<Soundtrack>,
    #[serde(default)]
    pub assets: AssetTable,
    #[serde(default)]
    pub layers: LayerIdTable,
    #[serde(default)]
    pub track_ids: TrackIdTable,
    #[serde(default)]
    pub tracks: Vec<Track>,
    /// 未知キー保持(unknown-keys roundtrip)。
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

impl Document {
    pub fn new_v1() -> Self {
        Self {
            version: 1,
            min_reader_version: 1,
            composition: Composition::new_v1(),
            bpm: Bpm::DEFAULT,
            soundtrack: None,
            assets: AssetTable::new(),
            layers: LayerIdTable::new(),
            track_ids: TrackIdTable::new(),
            tracks: Vec::new(),
            extra: Map::new(),
        }
    }
}

/// バックグラウンド成果をwriterへ返すメッセージ(適用はwriterのみ)。
#[derive(Debug, Clone, PartialEq)]
pub enum WriterMessage {
    /// プレースホルダ: 将来のプロキシ生成完了等。
    SetBpm(Bpm),
}

/// ドキュメントの唯一の書き手。`&mut Document`を外部に漏らさない。
#[derive(Debug)]
pub struct DocumentWriter {
    doc: Document,
    /// 編集世代。決定性テスト・無効化伝播の席(監査F-8)。
    pub revision: u64,
}

impl DocumentWriter {
    pub fn new(doc: Document) -> Self {
        Self { doc, revision: 0 }
    }

    pub fn snapshot(&self) -> Arc<Document> {
        Arc::new(self.doc.clone())
    }

    /// 編集はクロージャ経由のみ。戻り値なし — 参照を外に返せない。
    ///
    /// D2で`apply(Command)`に置換される足場。呼び出し追加禁止。
    pub fn edit(&mut self, f: impl FnOnce(&mut Document)) {
        f(&mut self.doc);
        self.revision = self.revision.wrapping_add(1);
    }

    /// バックグラウンド成果の適用。D2でメッセージ→Command変換に置換する。
    pub fn apply(&mut self, msg: WriterMessage) {
        match msg {
            WriterMessage::SetBpm(bpm) => self.doc.bpm = bpm,
        }
        self.revision = self.revision.wrapping_add(1);
    }

    /// 保存前検証。失敗してもwriter内部のDocumentは不変(検証のみ — ガード1)。
    pub fn validate(&self) -> Result<(), DocumentError> {
        self.doc.validate()
    }
}

/// 読み手API: スナップショットだけを受け、書き込めない。
pub fn render_with_snapshot(doc: &Arc<Document>) -> u32 {
    doc.version
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_core::RationalTime;

    #[test]
    fn writer_is_sole_mutator_readers_get_arc() {
        let mut writer = DocumentWriter::new(Document::new_v1());
        let snap_before = writer.snapshot();
        assert_eq!(render_with_snapshot(&snap_before), 1);
        assert_eq!(writer.revision, 0);

        writer.edit(|doc| {
            doc.version = 2;
            doc.bpm = Bpm::try_new(140, 1).unwrap();
        });
        assert_eq!(writer.revision, 1);

        let snap_after = writer.snapshot();
        assert_eq!(snap_before.version, 1);
        assert_eq!(snap_after.version, 2);
        assert_ne!(snap_before.bpm, snap_after.bpm);
    }

    #[test]
    fn background_message_applies_only_via_writer() {
        let mut writer = DocumentWriter::new(Document::new_v1());
        writer.apply(WriterMessage::SetBpm(Bpm::try_new(100, 1).unwrap()));
        assert_eq!(writer.snapshot().bpm.num(), 100);
        assert_eq!(writer.revision, 1);
    }

    #[test]
    fn document_json_roundtrip_empty() {
        let doc = Document::new_v1();
        let json = serde_json::to_string(&doc).unwrap();
        let back: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, back);
    }

    #[test]
    fn min_reader_version_defaults_when_absent() {
        let json = r#"{
            "version":1,
            "composition":{
                "aspect_num":16,
                "aspect_den":9,
                "duration":{"num":10,"den":1},
                "fps":{"num":30,"den":1}
            },
            "bpm":{"num":120,"den":1}
        }"#;
        let doc: Document = serde_json::from_str(json).unwrap();
        assert_eq!(doc.min_reader_version, 1);
        assert!(doc.extra.is_empty());
        assert_eq!(doc.composition.aspect_num(), 16);
        assert_eq!(doc.composition.aspect_den(), 9);
        assert_eq!(
            doc.composition.duration,
            RationalTime::try_new(10, 1).unwrap()
        );
    }
}
