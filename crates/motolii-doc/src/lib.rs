//! motolii-doc: ドキュメント所有権の最小骨格(F-2 / FG-C3)。
//!
//! M1完了後に、「writer以外に`&mut Document`が無い」ことを型で固定する。
//! 読み手(レンダ・書き出し・解析)は`Arc<Document>`スナップショットのみを受け取る。
//!
//! `edit`は戻り値を持たない — 呼び出し側が内部への参照を保持できる形で凍結すると、
//! 内部表現・ロック・スナップショット方式の差し替えが永久に封じられる(AE型の漏れ)。

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use motolii_core::TimeMap;

/// プロジェクト状態の最小プレースホルダ。本スキーマはM2-D1。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub version: u32,
    /// クリップ時間写像の予約口(F-4)。v1はプロジェクト直下に1本。
    pub time_map: TimeMap,
}

impl Document {
    pub fn new_v1() -> Self {
        Self {
            version: 1,
            time_map: TimeMap::identity(),
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
}

impl DocumentWriter {
    pub fn new(doc: Document) -> Self {
        Self { doc }
    }

    pub fn snapshot(&self) -> Arc<Document> {
        Arc::new(self.doc.clone())
    }

    /// 編集はクロージャ経由のみ。戻り値なし — 参照を外に返せない。
    pub fn edit(&mut self, f: impl FnOnce(&mut Document)) {
        f(&mut self.doc);
    }

    pub fn apply(&mut self, msg: WriterMessage) {
        match msg {
            WriterMessage::SetTimeMap(map) => self.doc.time_map = map,
        }
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

        writer.edit(|doc| {
            doc.version = 2;
            doc.time_map = TimeMap::offset(RationalTime::from_seconds(1), RationalTime::ZERO);
        });

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
    }

    #[test]
    fn document_json_roundtrip() {
        let doc = Document {
            version: 1,
            time_map: TimeMap::offset(RationalTime::from_seconds(3), RationalTime::ZERO),
        };
        let json = serde_json::to_string(&doc).unwrap();
        let back: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, back);
    }
}
