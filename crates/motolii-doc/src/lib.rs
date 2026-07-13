//! motolii-doc: ドキュメント所有権骨格(F-2) + D1-prelude(M2E-12) + D1aスキーマ + D1b検証 + D1c永続化。
//!
//! 読み手(レンダ・書き出し・解析)は`Arc<Document>`スナップショットのみを受け取る。
//! `edit`は戻り値を持たない — 参照漏洩で凍結を封じないため。
//!
//! **D1a**: スキーマ本体。**D1b**: 保存前`validate`(ガード1)。**D1c**: アトミック保存/読込。ジャーナルはD1d。
//! **D8**: 単一writer + スナップショット配布の並行契約(型denyは`mut_document_deny`、完走は`d8_ownership`)。
//! **D1c-FU(#101)**: `ResourceLimits`(入力上限、監査S10)と`OpenMode`(read/write互換分離、監査S14)。
//! **D3**: Document→レンダグラフ変換(`graph` / `EvaluationTime`)。

mod affine;
mod asset;
mod bpm;
mod command;
mod doc_keyframe;
mod doc_value;
mod duplicate;
mod eval_time;
mod graph;
mod ids;
mod limits;
mod param;
pub mod param_eval;
pub mod param_expect;
pub mod pathgeom;
mod persist;
mod plugin_compat;
mod schema;
mod stable_id;
mod track_id;
mod undo;
mod validate;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub use affine::{compose_local, compose_transform, resolve_transform, Affine2D};
pub use asset::{Asset, AssetError, AssetId, AssetTable};
pub use bpm::{Bpm, BpmError};
pub use command::{
    collect_layer_ids, layer_names_for_item, Command, CommandError, CommandKind, GestureId,
    MergeKey, ParentLocator, PropertyId, ScalarPropertyId,
};
pub use doc_keyframe::{DocKeyframe, DocKeyframeError, DocKeyframeTrack};
pub use doc_value::DocValue;
pub use duplicate::DuplicateError;
pub use eval_time::{
    EvaluationTime, D3_CLIP_LOCAL_TO_SOURCE_VIA_TIMEMAP, M1_SOURCE_PTS_EQUALS_TIMELINE,
};
pub use graph::{
    build_document_frame_graph, resolve_asset_path, DocumentFrameGraph, GraphError,
    CLEAR_LAYER_SOURCE, RECT_LAYER_SOURCE,
};
pub use ids::{LayerId, LayerIdError, LayerIdTable};
pub use limits::{ResourceLimitError, ResourceLimits};
pub use param::{DocParam, LookAtAxis};
pub use param_eval::{ParamEvalError, ResolvedLayerParams};
pub use param_expect::{DocPluginKind, ExpectedValueType, KnownPluginInfo, ParamConstraints};
pub use pathgeom::PathOpError;
pub use persist::{
    check_migration_allowed, classify_open_mode, detect_cloud_sync, load_document,
    load_document_bytes, load_document_bytes_with_limits, load_document_with_limits, save_document,
    save_document_with_options, CloudSyncHint, OpenMode, OpenedDocument, PersistError,
    SaveAbortAfter, SaveOptions, READER_VERSION, WRITER_VERSION,
};
pub use plugin_compat::{PluginDegradation, PluginOpenWarning};
pub use schema::{
    BlendMode, Clip, ClipSource, ClippingMaskSettings, CompositeOrder, Composition,
    CompositionError, EffectInstance, Group, ItemEnvelope, LineJoin, MaskMode, PathOp, PointType,
    Soundtrack, SoundtrackError, StandardShape, Track, TrackItem, Transform2D, TrimMode,
    VectorContent, VectorRecipe,
};
pub use stable_id::{EffectId, KeyframeId, StableIdError, StableIdSeq};
pub use track_id::{TrackId, TrackIdError, TrackIdTable};
pub use undo::{Macro, UndoError, UndoHistory, UndoLimit};
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
    /// EffectId/KeyframeId共有カウンタ(A8)。非再利用の単調カウンタ — ネスト構造
    /// (`EffectInstance.id`/`DocKeyframe.id`)を持つ文書は`min_reader_version`を
    /// 2以上に上げる責務を保存側が持つ(M2E-11①のネスト規律。本フィールド自体は
    /// `default`でロード可能 — 旧文書に`effects`/keyframesが無ければ影響しない)。
    #[serde(default)]
    pub next_stable_id: StableIdSeq,
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
            next_stable_id: StableIdSeq::new(),
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
///
/// D2: `Command`の適用は必ず`apply_command`(内部でUndoHistoryへ積む)経由。
/// `edit`/`apply`は移行済み呼び出し元のための足場であり、undo履歴には積まれない
/// (選択/hover/IME等のUI状態や、Command化されていない旧経路専用 — 実装ガード5)。
#[derive(Debug)]
pub struct DocumentWriter {
    doc: Document,
    /// 編集世代。決定性テスト・無効化伝播の席(監査F-8)。
    pub revision: u64,
    undo: UndoHistory,
    /// gesture_id発行カウンタ。UI側の操作単位ごとに`begin_gesture`で1つ取る
    /// (Documentスキーマには入れない実行時のみの値 — #103⑨)。
    next_gesture: u64,
}

impl DocumentWriter {
    pub fn new(doc: Document) -> Self {
        Self::with_undo_limits(doc, UndoLimit::Unlimited, UndoLimit::Unlimited)
    }

    /// live/再起動後で別々のUndo深さ上限を設定して構築する(残小項目【決定】2026-07-13)。
    pub fn with_undo_limits(
        doc: Document,
        live_limit: UndoLimit,
        restart_limit: UndoLimit,
    ) -> Self {
        Self {
            doc,
            revision: 0,
            undo: UndoHistory::new(live_limit, restart_limit),
            next_gesture: 0,
        }
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
        self.undo.push(&mut self.doc, gesture, command)?;
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

    /// `EffectId`を新規発行する(A8、非再利用)。ネスト永続フィールド追加の規律
    /// (M2E-11①)に沿い、発行と同時に`version`と`min_reader_version`を下限まで引き上げる。
    /// `version < min_reader_version`は`validate`が拒否するため、片方だけ上げない。
    pub fn allocate_effect_id(&mut self) -> Result<EffectId, StableIdError> {
        let id = self.doc.next_stable_id.allocate()?;
        self.bump_versions_for_stable_ids();
        Ok(EffectId::from_raw(id))
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
