//! motolii-doc: ドキュメント所有権骨格(F-2) + D1-prelude(M2E-12) + D1aスキーマ + D1b検証 + D1c永続化。
//!
//! 読み手(レンダ・書き出し・解析)は`Arc<Document>`スナップショットのみを受け取る。
//! `edit`は戻り値を持たない — 参照漏洩で凍結を封じないため。
//!
//! **D1a**: スキーマ本体。**D1b**: 保存前`validate`(ガード1)。**D1c**: アトミック保存/読込。ジャーナルはD1d。
//! **D8**: 単一writer + スナップショット配布の並行契約(型denyは`mut_document_deny`、完走は`d8_ownership`)。
//! **D1c-FU(#101)**: `ResourceLimits`(入力上限、監査S10)と`OpenMode`(read/write互換分離、監査S14)。
//! **D3**: Document→レンダグラフ変換(`graph` / `EvaluationTime`)。
//! **D1e**: 旧形式migration(`migrate`)。loadは拒否のまま、変換は明示API。

mod affine;
mod asset;
mod audio_edit;
mod bpm;
mod command;
mod doc_keyframe;
mod doc_value;
mod duplicate;
mod effect_prepare;
mod eval_time;
mod graph;
mod ids;
pub mod journal;
mod legacy_effect_migrate;
mod limits;
mod migrate;
mod param;
pub mod param_eval;
pub mod param_expect;
pub mod pathgeom;
mod persist;
mod plugin_resolution;
mod schema;
mod spatial_resolve;
mod stable_id;
mod track_id;
mod undo;
mod validate;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub use affine::{compose_local, compose_transform, resolve_transform, Affine2D};
pub use asset::{Asset, AssetError, AssetId, AssetTable};
pub use audio_edit::{build_import_clip_source, plan_detach_audio, ImportAvMode};
pub use bpm::{Bpm, BpmError};
pub use command::{
    collect_layer_ids, layer_names_for_item, Command, CommandError, CommandKind, GestureId,
    MergeKey, ParentLocator, PropertyId, ScalarPropertyId,
};
pub use doc_keyframe::{DocKeyframe, DocKeyframeError, DocKeyframeTrack};
pub use doc_value::DocValue;
pub use duplicate::DuplicateError;
pub use effect_prepare::{DraftDocParam, DraftKeyframe, EffectDefinitionDraft, PrepareError};
pub use eval_time::{
    EvaluationTime, D3_CLIP_LOCAL_TO_SOURCE_VIA_TIMEMAP, M1_SOURCE_PTS_EQUALS_TIMELINE,
};
pub use graph::{
    build_document_frame_graph, resolve_asset_path, DocumentFrameGraph, GraphError, VideoSlot,
    CLEAR_LAYER_SOURCE, RECT_LAYER_SOURCE,
};
pub use ids::{LayerId, LayerIdError, LayerIdTable};
pub use journal::{
    generation_path_for_document, journal_path_for_document,
    legacy_shared_motolii_dir_for_document, legacy_staging_dir_for_document, load_catalog,
    motolii_dir_for_document, project_lock_path_for_document, project_sidecar_dir_for_document,
    restore_attempted_path, DurabilityStage, FaultPlan, FsOpKind, GenerationCatalog,
    GenerationEntry, JournalEdit, JournalRecordKind, JournalScanStop,
    LegacySidecarMigrationDisposition, LegacySidecarMigrationReport, OpenProjectOutcome,
    PinGenerationOptions, ProjectError, ProjectSession, RecordingFs, RecoveryError, RecoverySource,
    RotateOptions, SaveProjectOptions, SessionError, StdFs, WalError,
};
pub use limits::{ResourceLimitError, ResourceLimits};
pub use migrate::{
    bump_min_reader_for_nest_schema_change, count_document, legacy_timemap_source, migrate_bytes,
    migrate_bytes_with_limits, modern_timemap_source, semantic_fingerprint, DocumentCounts,
    MigrateError, MigrateFileOptions, MigrateFileResult, MigrationReport, SemanticFingerprint,
    BACKUP_SUFFIX, LATEST_DOCUMENT_VERSION,
};
pub use param::{DocParam, LookAtAxis};
pub use param_eval::{eval_look_at_rotation, look_at_angle, ParamEvalError, ResolvedLayerParams};
pub use param_expect::{ExpectedValueType, ParamConstraints};
pub use pathgeom::PathOpError;
pub use persist::{
    check_migration_allowed, classify_open_mode, detect_cloud_sync, load_document,
    load_document_bytes, load_document_bytes_with_limits, load_document_with_limits, CloudSyncHint,
    OpenMode, OpenedDocument, PersistError, SaveAbortAfter, SaveOptions, READER_VERSION,
    WRITER_VERSION,
};
pub use plugin_resolution::{
    open_project_resolved, prepare_plugin_recipe, DocumentPluginError, PluginDiagnostic,
    PluginDiagnosticReason, PluginSlotId, PreparedDocumentPlugins, PreparedPluginRecipe,
    ResolvedOpenProjectOutcome,
};
pub use schema::{
    asset_components_require_newer_reader, AudioComponent, AudioOutOfRange, BlendMode, Clip,
    ClipSource, ClippingMaskSettings, CompCameraDoc, CompositeOrder, Composition, CompositionError,
    EffectDefinition, EffectInstance, EffectUse, Group, ItemEnvelope, LineJoin, MaskMode, PathOp,
    PointType, Soundtrack, SoundtrackError, StandardShape, StreamKind, StreamSelector, Track,
    TrackItem, Transform2D, TrimMode, VectorContent, VectorRecipe, VideoComponent,
};
pub use spatial_resolve::resolve_document_spaces;
pub use stable_id::{
    EffectDefinitionId, EffectId, KeyframeId, StableIdError, StableIdReservation, StableIdSeq,
};
pub use track_id::{TrackId, TrackIdError, TrackIdTable};
pub use undo::{Macro, UndoError, UndoHistory, UndoLimit};
pub use validate::{
    DocumentError, MIN_READER_VERSION_FOR_ASSET_COMPONENTS, MIN_READER_VERSION_FOR_COMP_CAMERA,
    MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS,
};

fn default_min_reader_version() -> u32 {
    1
}

/// プロジェクト状態。`ProjectV1`とは非継承・版番独立(M2E-11①)。
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
    /// EffectUse / EffectDefinition / KeyframeId 共有カウンタ(A8 / D1l)。
    #[serde(default)]
    pub next_stable_id: StableIdSeq,
    /// D1l: 共有Effect recipe台帳。Useから参照。orphan(参照0)を許可する。
    #[serde(default)]
    pub effect_definitions: Vec<EffectDefinition>,
    /// 未知キー保持(unknown-keys roundtrip)。
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

impl Document {
    /// 現行writerが新しく作るDocumentの唯一の生成口。
    pub fn new_current() -> Self {
        Self::empty_at_version(
            persist::WRITER_VERSION,
            validate::MIN_READER_VERSION_FOR_COMP_CAMERA,
        )
    }

    /// 旧版fixture・明示migration専用。製品の新規作成には`new_current`を使う。
    ///
    /// ```compile_fail
    /// #![deny(deprecated)]
    /// let _ = motolii_doc::Document::new_v1();
    /// ```
    #[deprecated(
        since = "0.1.0",
        note = "legacy/migration fixtures only; product code must use Document::new_current()"
    )]
    pub fn new_v1() -> Self {
        Self::empty_at_version(1, 1)
    }

    fn empty_at_version(version: u32, min_reader_version: u32) -> Self {
        Self {
            version,
            min_reader_version,
            composition: Composition::new_v1(),
            bpm: Bpm::DEFAULT,
            soundtrack: None,
            assets: AssetTable::new(),
            layers: LayerIdTable::new(),
            track_ids: TrackIdTable::new(),
            tracks: Vec::new(),
            next_stable_id: StableIdSeq::new(),
            effect_definitions: Vec::new(),
            extra: Map::new(),
        }
    }

    pub fn effect_definition(&self, id: EffectDefinitionId) -> Option<&EffectDefinition> {
        self.effect_definitions.iter().find(|d| d.id == id)
    }

    pub fn effect_definition_mut(
        &mut self,
        id: EffectDefinitionId,
    ) -> Option<&mut EffectDefinition> {
        self.effect_definitions.iter_mut().find(|d| d.id == id)
    }

    pub fn effect_use_count(&self, definition_id: EffectDefinitionId) -> usize {
        self.effect_use_ids(definition_id).len()
    }

    pub fn effect_use_ids(&self, definition_id: EffectDefinitionId) -> Vec<EffectId> {
        fn collect_in_items(
            items: &[TrackItem],
            definition_id: EffectDefinitionId,
            out: &mut Vec<EffectId>,
        ) {
            for item in items {
                match item {
                    TrackItem::Clip(clip) => {
                        for use_ in &clip.envelope.effects {
                            if use_.definition_id == definition_id {
                                out.push(use_.id);
                            }
                        }
                    }
                    TrackItem::Group(group) => {
                        for use_ in &group.envelope.effects {
                            if use_.definition_id == definition_id {
                                out.push(use_.id);
                            }
                        }
                        collect_in_items(&group.children, definition_id, out);
                    }
                }
            }
        }
        let mut out = Vec::new();
        for track in &self.tracks {
            collect_in_items(&track.items, definition_id, &mut out);
        }
        out
    }

    pub fn find_effect_use(&self, layer: LayerId, use_id: EffectId) -> Option<&EffectUse> {
        crate::command::find_envelope(self, layer)?
            .effects
            .iter()
            .find(|u| u.id == use_id)
    }
}

/// バックグラウンド成果をwriterへ返すメッセージ(適用はwriterのみ)。
#[derive(Debug, Clone, PartialEq)]
pub enum WriterMessage {
    /// プレースホルダ: 将来のプロキシ生成完了等。
    SetBpm(Bpm),
}

/// ドキュメントの唯一の書き手。`&mut Document`を外部に漏らさない。
///
/// D2: `Command`の適用は必ず`apply_command`(内部でUndoHistoryへ積む)経由。
/// `edit`/`apply`は移行済み呼び出し元のための足場であり、undo履歴には積まれない
/// (選択/hover/IME等のUI状態や、Command化されていない旧経路専用 — 実装ガード5)。
///
/// D1l B-3: Effect identity 採番の正規経路は `prepare_*` のみ。旧 allocate API は公開されない:
/// ```compile_fail
/// # fn main() {
/// let catalog = std::sync::Arc::new(motolii_plugin::reference::reference_catalog().unwrap());
/// let mut w = motolii_doc::DocumentWriter::new(motolii_doc::Document::new_current(), catalog).unwrap();
/// let _ = w.allocate_effect_id();
/// # }
/// ```
/// ```compile_fail
/// # fn main() {
/// let catalog = std::sync::Arc::new(motolii_plugin::reference::reference_catalog().unwrap());
/// let mut w = motolii_doc::DocumentWriter::new(motolii_doc::Document::new_current(), catalog).unwrap();
/// let _ = w.allocate_effect_definition_id();
/// # }
/// ```
/// ```compile_fail
/// # fn main() {
/// let catalog = std::sync::Arc::new(motolii_plugin::reference::reference_catalog().unwrap());
/// let mut w = motolii_doc::DocumentWriter::new(motolii_doc::Document::new_current(), catalog).unwrap();
/// let _ = w.allocate_unique_effect_pair();
/// # }
/// ```
///
/// Draft 型は永続化できない(serde 無し):
/// ```compile_fail
/// # fn main() {
/// use motolii_doc::EffectDefinitionDraft;
/// fn _serialize(d: &EffectDefinitionDraft) -> impl serde::Serialize + '_ { d }
/// # }
/// ```
/// ```compile_fail
/// # fn main() {
/// use motolii_doc::DraftDocParam;
/// fn _serialize(d: &DraftDocParam) -> impl serde::Serialize + '_ { d }
/// # }
/// ```
/// ```compile_fail
/// # fn main() {
/// use motolii_doc::DraftKeyframe;
/// fn _serialize(d: &DraftKeyframe) -> impl serde::Serialize + '_ { d }
/// # }
/// ```
/// ```compile_fail
/// # fn main() {
/// use motolii_doc::EffectDefinitionDraft;
/// fn _deserialize<T: for<'de> serde::Deserialize<'de>>() {}
/// _deserialize::<EffectDefinitionDraft>();
/// # }
/// ```
/// ```compile_fail
/// # fn main() {
/// use motolii_doc::DraftDocParam;
/// fn _deserialize<T: for<'de> serde::Deserialize<'de>>() {}
/// _deserialize::<DraftDocParam>();
/// # }
/// ```
/// ```compile_fail
/// # fn main() {
/// use motolii_doc::DraftKeyframe;
/// fn _deserialize<T: for<'de> serde::Deserialize<'de>>() {}
/// _deserialize::<DraftKeyframe>();
/// # }
/// ```
/// ```compile_fail
/// # fn main() {
/// use motolii_doc::EffectDefinitionDraft;
/// fn _owned<T: serde::de::DeserializeOwned>() {}
/// _owned::<EffectDefinitionDraft>();
/// # }
/// ```
/// ```compile_fail
/// # fn main() {
/// use motolii_doc::DraftDocParam;
/// fn _owned<T: serde::de::DeserializeOwned>() {}
/// _owned::<DraftDocParam>();
/// # }
/// ```
/// ```compile_fail
/// # fn main() {
/// use motolii_doc::DraftKeyframe;
/// fn _owned<T: serde::de::DeserializeOwned>() {}
/// _owned::<DraftKeyframe>();
/// # }
/// ```
#[derive(Debug)]
pub struct DocumentWriter {
    doc: Document,
    catalog: Arc<motolii_plugin::PluginCatalog>,
    /// 編集世代。決定性テスト・無効化伝播の席(監査F-8)。
    pub revision: u64,
    undo: UndoHistory,
    /// gesture_id発行カウンタ。UI側の操作単位ごとに`begin_gesture`で1つ取る
    /// (Documentスキーマには入れない実行時のみの値 — #103⑨)。
    next_gesture: u64,
}

impl DocumentWriter {
    pub fn new(
        doc: Document,
        catalog: Arc<motolii_plugin::PluginCatalog>,
    ) -> Result<Self, DocumentPluginError> {
        Self::with_undo_limits(doc, catalog, UndoLimit::Unlimited, UndoLimit::Unlimited)
    }

    /// live/再起動後で別々のUndo深さ上限を設定して構築する(残小項目【決定】2026-07-13)。
    pub fn with_undo_limits(
        doc: Document,
        catalog: Arc<motolii_plugin::PluginCatalog>,
        live_limit: UndoLimit,
        restart_limit: UndoLimit,
    ) -> Result<Self, DocumentPluginError> {
        doc.validate()?;
        doc.prepare_plugins(&catalog)?;
        Ok(Self {
            doc,
            catalog,
            revision: 0,
            undo: UndoHistory::new(live_limit, restart_limit),
            next_gesture: 0,
        })
    }

    pub fn snapshot(&self) -> Arc<Document> {
        Arc::new(self.doc.clone())
    }

    /// 編集はクロージャ経由のみ。戻り値なし — 参照を外に返せない。
    ///
    /// undo履歴には積まれない旧来経路。Command化された操作は`apply_command`を使う。
    pub fn edit(&mut self, f: impl FnOnce(&mut Document)) {
        f(&mut self.doc);
        self.revision = self.revision.wrapping_add(1);
    }

    /// バックグラウンド成果の適用。undo履歴には積まれない(UI都合の非可逆反映)。
    pub fn apply(&mut self, msg: WriterMessage) {
        match msg {
            WriterMessage::SetBpm(bpm) => self.doc.bpm = bpm,
        }
        self.revision = self.revision.wrapping_add(1);
    }

    /// 新しいgesture(1操作単位)を開始し、そのIDを返す。以後このIDで積んだcommandは
    /// 同一gestureの間、merge key(S18)が一致する限り1つのmacroへ畳まれる(#103⑨)。
    pub fn begin_gesture(&mut self) -> GestureId {
        let id = GestureId::from_raw(self.next_gesture);
        self.next_gesture = self.next_gesture.wrapping_add(1);
        id
    }

    /// atomic command(実装ガード5: 決定済みの値)を適用し、undo履歴へ積む。
    /// 単一writer境界(このメソッドだけがDocumentを書き換える)。
    pub fn apply_command(
        &mut self,
        gesture: GestureId,
        command: Command,
    ) -> Result<(), CommandError> {
        let before_doc = self.doc.clone();
        let before_undo = self.undo.clone();
        self.undo.push(&mut self.doc, gesture, command)?;
        if let Err(error) = self.doc.prepare_plugins(&self.catalog) {
            self.doc = before_doc;
            self.undo = before_undo;
            return Err(CommandError::Plugin(error));
        }
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        self.undo.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.undo.can_redo()
    }

    pub fn undo_len(&self) -> usize {
        self.undo.undo_len()
    }

    pub fn redo_len(&self) -> usize {
        self.undo.redo_len()
    }

    /// 直前のgesture(1 macro分)を丸ごと取り消す。
    pub fn undo(&mut self) -> Result<(), UndoError> {
        self.undo.undo(&mut self.doc)?;
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }

    /// 直前にundoしたgestureを丸ごと再適用する。
    pub fn redo(&mut self) -> Result<(), UndoError> {
        self.undo.redo(&mut self.doc)?;
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }

    /// `LayerId`を新規発行する(非再利用)。Command構築前の下準備用。
    /// 台帳エントリも作る。`AddTrackItem`へ渡す場合は同じ名前を`layer_names`へ含め、
    /// undo時に台帳から外せるようにする。エントリ無しの予約だけなら`reserve_layer_id`。
    pub fn allocate_layer_id(
        &mut self,
        display_name: impl Into<String>,
    ) -> Result<LayerId, LayerIdError> {
        self.doc.layers.allocate(display_name)
    }

    /// `LayerId`を予約する(カウンタのみ進める。台帳エントリは作らない)。
    /// エントリは`AddTrackItem.layer_names`のapplyで載せる。
    pub fn reserve_layer_id(&mut self) -> Result<LayerId, LayerIdError> {
        self.doc.layers.reserve()
    }

    /// `KeyframeId`を新規発行する(A8、非再利用)。同上。
    pub fn allocate_keyframe_id(&mut self) -> Result<KeyframeId, StableIdError> {
        let id = self.doc.next_stable_id.allocate()?;
        self.bump_versions_for_stable_ids();
        Ok(KeyframeId::from_raw(id))
    }

    fn bump_versions_for_stable_ids(&mut self) {
        let floor = validate::MIN_READER_VERSION_FOR_STABLE_IDS;
        self.doc.min_reader_version = self.doc.min_reader_version.max(floor);
        self.doc.version = self.doc.version.max(floor);
    }

    /// `source`のsubtreeを複製し、直後に挿入するcommandを1 gestureとして適用する
    /// (「duplicate/paste時: subtree内参照は新ID再写像、外向き参照は維持」)。
    pub fn duplicate_track_item(&mut self, source: LayerId) -> Result<GestureId, DuplicateError> {
        let command = duplicate::duplicate_track_item(&mut self.doc, source)?;
        let gesture = self.begin_gesture();
        self.undo.push(&mut self.doc, gesture, command)?;
        self.revision = self.revision.wrapping_add(1);
        Ok(gesture)
    }

    /// 読み取り専用: コマンド構築側が現在の`ItemEnvelope`を読むためのヘルパ。
    pub fn find_envelope(&self, target: LayerId) -> Option<&ItemEnvelope> {
        command::find_envelope(&self.doc, target)
    }

    /// 保存前検証。失敗してもwriter内部のDocumentは不変(検証のみ — ガード1)。
    pub fn validate(&self) -> Result<(), DocumentPluginError> {
        self.doc.validate()?;
        self.doc.prepare_plugins(&self.catalog)?;
        Ok(())
    }

    /// D1l B-3: 新 Use+Definition を counter clone 上で準備する。成功・失敗とも live Document 不変。
    pub fn prepare_create_effect(
        &self,
        target: LayerId,
        index: usize,
        draft: EffectDefinitionDraft,
    ) -> Result<Command, PrepareError> {
        effect_prepare::prepare_create_effect(&self.doc, target, index, draft)
    }

    /// D1l B-3: 既存 Definition へ新 Use を準備する。
    pub fn prepare_link_effect_use(
        &self,
        target: LayerId,
        index: usize,
        definition_id: EffectDefinitionId,
    ) -> Result<Command, PrepareError> {
        effect_prepare::prepare_link_effect_use(&self.doc, target, index, definition_id)
    }

    /// D1l B-3: 対象 Use の Definition をローカル複製する。
    pub fn prepare_copy_local_effect(&self, use_id: EffectId) -> Result<Command, PrepareError> {
        effect_prepare::prepare_copy_local_effect(&self.doc, use_id)
    }
}

/// 読み手API: スナップショットだけを受け、書き込めない。
pub fn render_with_snapshot(doc: &Arc<Document>) -> u32 {
    doc.version
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    fn reference_writer(doc: Document) -> DocumentWriter {
        DocumentWriter::new(
            doc,
            Arc::new(motolii_plugin::reference::reference_catalog().unwrap()),
        )
        .unwrap()
    }
    use motolii_core::RationalTime;

    #[test]
    fn writer_is_sole_mutator_readers_get_arc() {
        let mut writer = reference_writer(Document::new_current());
        let snap_before = writer.snapshot();
        assert_eq!(render_with_snapshot(&snap_before), WRITER_VERSION);
        assert_eq!(writer.revision, 0);

        writer.edit(|doc| {
            doc.bpm = Bpm::try_new(140, 1).unwrap();
        });
        assert_eq!(writer.revision, 1);

        let snap_after = writer.snapshot();
        assert_eq!(snap_before.version, WRITER_VERSION);
        assert_eq!(snap_after.version, WRITER_VERSION);
        assert_ne!(snap_before.bpm, snap_after.bpm);
    }

    #[test]
    fn background_message_applies_only_via_writer() {
        let mut writer = reference_writer(Document::new_current());
        writer.apply(WriterMessage::SetBpm(Bpm::try_new(100, 1).unwrap()));
        assert_eq!(writer.snapshot().bpm.num(), 100);
        assert_eq!(writer.revision, 1);
    }

    #[test]
    fn document_json_roundtrip_empty() {
        let doc = Document::new_current();
        let json = serde_json::to_string(&doc).unwrap();
        let back: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, back);
    }

    #[test]
    fn min_reader_version_defaults_when_absent() {
        let json = r#"{
            "version":5,
            "composition":{
                "aspect_num":16,
                "aspect_den":9,
                "duration":{"num":10,"den":1},
                "fps":{"num":30,"den":1},
                "camera":{
                    "kind":"planar_orthographic",
                    "center":{"const":{"Vec2":[0.0,0.0]}},
                    "roll_radians":{"const":{"F64":0.0}},
                    "height":{"const":{"F64":1.0}}
                }
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
