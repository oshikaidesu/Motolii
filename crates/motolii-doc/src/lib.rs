//! motolii-doc: ドキュメント所有権の最小骨格(F-2 / FG-C3) + D1-prelude(M2E-12)。
//!
//! M1完了後に、「writer以外に`&mut Document`が無い」ことを型で固定する。
//! 読み手(レンダ・書き出し・解析)は`Arc<Document>`スナップショットのみを受け取る。
//!
//! `edit`は戻り値を持たない — 呼び出し側が内部への参照を保持できる形で凍結すると、
//! 内部表現・ロック・スナップショット方式の差し替えが永久に封じられる(AE型の漏れ)。
//!
//! **D1-prelude**(M2E-12): version/互換の枠と所有権骨格のみ。トラック・クリップ等の
//! スキーマ本体は含めない(本体はゲート達成後のD1)。

mod ids;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use motolii_core::TimeMap;

pub use ids::{LayerId, LayerIdError, LayerIdTable};

fn default_min_reader_version() -> u32 {
    1
}

/// プロジェクト状態の最小プレースホルダ。スキーマ本体はM2-D1。
///
/// `min_reader_version` / `extra` は前方互換の枠(実装ガード7)。拒否は
/// `min_reader_version` 超過時のみ。未知キーは `extra` に保持し再保存で書き戻す。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub version: u32,
    /// このリーダーが理解すべき最小スキーマ版。旧リーダーはこれを見て拒否する。
    #[serde(default = "default_min_reader_version")]
    pub min_reader_version: u32,
    /// クリップ時間写像の予約口(F-4)。v1はプロジェクト直下に1本。
    pub time_map: TimeMap,
    /// 未知キー保持(unknown-keys roundtrip)。スキーマ本体フィールドをここに足さない。
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

impl Document {
    pub fn new_v1() -> Self {
        Self {
            version: 1,
            min_reader_version: 1,
            time_map: TimeMap::identity(),
            extra: Map::new(),
        }
    }
}

/// バックグラウンド成果をwriterへ返すメッセージ(適用はwriterのみ)。
#[derive(Debug, Clone, PartialEq)]
pub enum WriterMessage {
    /// プレースホルダ: 将来のプロキシ生成完了等。
    SetTimeMap(TimeMap),
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
            WriterMessage::SetTimeMap(map) => self.doc.time_map = map,
        }
        self.revision = self.revision.wrapping_add(1);
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
            doc.time_map = TimeMap::offset(RationalTime::from_seconds(1), RationalTime::ZERO);
        });
        assert_eq!(writer.revision, 1);

        let snap_after = writer.snapshot();
        assert_eq!(snap_before.version, 1);
        assert_eq!(snap_after.version, 2);
        assert_ne!(snap_before.time_map, snap_after.time_map);
    }

    #[test]
    fn background_message_applies_only_via_writer() {
        let mut writer = DocumentWriter::new(Document::new_v1());
        writer.apply(WriterMessage::SetTimeMap(
            TimeMap::constant_speed(RationalTime::ZERO, RationalTime::ZERO, 2, 1).unwrap(),
        ));
        assert_eq!(writer.snapshot().time_map.speed_num, 2);
        assert_eq!(writer.revision, 1);
    }

    #[test]
    fn document_json_roundtrip() {
        let doc = Document {
            version: 1,
            min_reader_version: 1,
            time_map: TimeMap::offset(RationalTime::from_seconds(3), RationalTime::ZERO),
            extra: Map::new(),
        };
        let json = serde_json::to_string(&doc).unwrap();
        let back: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, back);
    }

    #[test]
    fn min_reader_version_defaults_when_absent() {
        let json = r#"{
            "version":1,
            "time_map":{
                "source_start":{"num":0,"den":1},
                "timeline_start":{"num":0,"den":1},
                "speed_num":1,
                "speed_den":1
            }
        }"#;
        let doc: Document = serde_json::from_str(json).unwrap();
        assert_eq!(doc.min_reader_version, 1);
        assert!(doc.extra.is_empty());
    }
}
