//! 履歴の一本線。編集の段と、保存・起動・異常終了のような記録を同じ列に並べる。
//!
//! 段の位置は Document の edit_head。点の id は走行ごとに一度きりの連番で、
//! 段が捨てられても再利用しない。点は親を指すので、将来の分岐は
//! 「同じ親を持つ 2 本目」で足せる(合流はこの版の外)。
use serde_json::{Value as J, json};
use std::path::PathBuf;

/// 覚えておく点の数。窓に出る列と disk の両方で同じ。
pub const LIMIT: usize = 200;

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub id: String,
    pub parent: Option<String>,
    /// 戻れる段。前の走行から読んだ点は None で、押しても動かない。
    pub head: Option<i64>,
    pub kind: String,
    pub label: String,
    pub at: f64,
}

impl Entry {
    fn to_json(&self) -> J {
        json!({"id":self.id,"parent":self.parent,"head":self.head,"kind":self.kind,"label":self.label,"at":self.at})
    }
    fn from_json(j: &J) -> Option<Self> {
        Some(Self {
            id: j["id"].as_str()?.to_owned(),
            parent: j["parent"].as_str().map(str::to_owned),
            head: j["head"].as_i64(),
            kind: j["kind"].as_str()?.to_owned(),
            label: j["label"].as_str()?.to_owned(),
            at: j["at"].as_f64()?,
        })
    }
}

pub struct Ledger {
    entries: Vec<Entry>,
    session: u64,
    seq: u64,
    file: Option<PathBuf>,
}

fn now() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// 記録の置き場。Swift の設定と同じ MotoliiStage5 の下。
pub fn default_file() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library/Application Support/MotoliiStage5")
            .join("history.jsonl"),
    )
}

/// 操作名を窓に出す文字にする。camelCase を切って先頭だけ大文字。
/// 表を持たないので、命令を足せば列の名前も一緒に付いてくる。
pub fn label(op: &str) -> String {
    let mut out = String::with_capacity(op.len() + 4);
    for (i, c) in op.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push(' ');
            }
            out.push(c.to_ascii_lowercase());
        } else if i == 0 {
            out.extend(c.to_uppercase());
        } else {
            out.push(c);
        }
    }
    out
}

impl Ledger {
    /// 前の走行の記録を読み、閉じた印が無ければ「異常終了」を 1 行足す。
    pub fn open(file: Option<PathBuf>) -> Self {
        let mut entries = Vec::new();
        if let Some(path) = &file {
            if let Ok(text) = std::fs::read_to_string(path) {
                for line in text.lines() {
                    if let Some(entry) =
                        serde_json::from_str::<J>(line).ok().as_ref().and_then(Entry::from_json)
                    {
                        // 前の走行の段には戻れない。押せない点として残す。
                        entries.push(Entry { head: None, ..entry });
                    }
                }
            }
        }
        let interrupted = entries.last().is_some_and(|e| e.kind != "end");
        let mut ledger = Self {
            entries,
            session: (now() * 1000.0) as u64,
            seq: 0,
            file,
        };
        ledger.trim();
        if interrupted {
            ledger.record("warning", "Previous session ended without closing", None);
        }
        ledger
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    fn trim(&mut self) {
        if self.entries.len() > LIMIT {
            let drop = self.entries.len() - LIMIT;
            self.entries.drain(..drop);
        }
    }

    fn push(&mut self, head: Option<i64>, kind: &str, label: String) {
        self.seq += 1;
        let entry = Entry {
            id: format!("{}-{}", self.session, self.seq),
            parent: self.entries.last().map(|e| e.id.clone()),
            head,
            kind: kind.to_owned(),
            label,
            at: now(),
        };
        self.entries.push(entry);
        self.trim();
        self.flush();
    }

    /// 記録を 1 行足す。保存・起動・異常はこれで編集と同じ列に乗る。
    pub fn record(&mut self, kind: &str, label: impl Into<String>, head: Option<i64>) {
        self.push(head, kind, label.into());
    }

    /// 命令を 1 つ実行した後の後始末。段が進んだ分だけ点を足し、
    /// 戻っていた先から編集したなら、捨てられた先の点も一緒に消す。
    pub fn note(&mut self, op: &str, before: i64, after: i64) {
        match op {
            "save" => return self.record("save", "Saved", Some(after)),
            "new" => return self.record("open", "New document", Some(after)),
            "undo" | "redo" | "historyGoto" => return,
            _ => {}
        }
        if after <= before {
            return;
        }
        self.entries.retain(|e| e.head.is_none_or(|h| h <= before));
        let name = label(op);
        for head in before + 1..=after {
            self.push(Some(head), "edit", name.clone());
        }
    }

    /// 正しく閉じた印。次の走行はこれが無ければ異常終了として出す。
    pub fn close(&mut self) {
        self.record("end", "Session closed", None);
    }

    /// 窓に渡す列。今どこに居るかは head で照合する。
    pub fn snapshot(&self, head: i64) -> J {
        json!({
            "head": head,
            "entries": self.entries.iter().map(Entry::to_json).collect::<Vec<_>>(),
        })
    }

    fn flush(&mut self) {
        let Some(path) = self.file.clone() else { return };
        if let Some(dir) = path.parent() {
            if std::fs::create_dir_all(dir).is_err() {
                return;
            }
        }
        let mut text = String::new();
        for entry in &self.entries {
            text.push_str(&entry.to_json().to_string());
            text.push('\n');
        }
        // 書けなくても編集は続く。記録が落ちた事は次に読んだ時の欠けとして出る。
        let _ = std::fs::write(&path, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("motolii-history-{name}-{}.jsonl", std::process::id()))
    }

    #[test]
    fn operation_names_become_readable_labels() {
        assert_eq!(label("setProperty"), "Set property");
        assert_eq!(label("moveLayers"), "Move layers");
        assert_eq!(label("paste"), "Paste");
    }

    #[test]
    fn every_advanced_step_becomes_one_point() {
        let mut ledger = Ledger::open(None);
        ledger.note("create", 0, 1);
        ledger.note("setProperty", 1, 3);
        let heads: Vec<_> = ledger.entries().iter().filter_map(|e| e.head).collect();
        assert_eq!(heads, vec![1, 2, 3]);
        assert_eq!(ledger.entries()[2].label, "Set property");
    }

    #[test]
    fn points_carry_stable_unique_ids_and_a_parent() {
        let mut ledger = Ledger::open(None);
        ledger.note("create", 0, 1);
        ledger.note("paste", 1, 2);
        let first = ledger.entries()[0].clone();
        let second = ledger.entries()[1].clone();
        assert_ne!(first.id, second.id);
        assert_eq!(second.parent.as_deref(), Some(first.id.as_str()));
        ledger.note("undo", 2, 1);
        assert_eq!(ledger.entries()[0], first, "戻っても既に在る点の id は変わらない");
    }

    #[test]
    fn editing_after_undo_drops_the_abandoned_points() {
        let mut ledger = Ledger::open(None);
        ledger.note("create", 0, 1);
        ledger.note("paste", 1, 2);
        ledger.note("undo", 2, 1);
        ledger.note("delete", 1, 2);
        let labels: Vec<_> = ledger.entries().iter().map(|e| e.label.clone()).collect();
        assert_eq!(labels, vec!["Create", "Delete"]);
    }

    #[test]
    fn records_share_the_column_with_edits() {
        let mut ledger = Ledger::open(None);
        ledger.note("create", 0, 1);
        ledger.note("save", 1, 1);
        let kinds: Vec<_> = ledger.entries().iter().map(|e| e.kind.clone()).collect();
        assert_eq!(kinds, vec!["edit", "save"]);
        assert_eq!(ledger.entries()[1].head, Some(1), "保存の点も同じ段を指す");
    }

    #[test]
    fn a_session_that_never_closed_is_reported_once() {
        let path = temp("interrupted");
        let _ = std::fs::remove_file(&path);
        let mut first = Ledger::open(Some(path.clone()));
        first.note("create", 0, 1);
        drop(first);

        let second = Ledger::open(Some(path.clone()));
        let last = second.entries().last().unwrap();
        assert_eq!(last.kind, "warning");
        assert_eq!(second.entries()[0].head, None, "前の走行の段には戻れない");

        let mut third = Ledger::open(Some(path.clone()));
        third.close();
        let fourth = Ledger::open(Some(path.clone()));
        assert_eq!(fourth.entries().last().unwrap().kind, "end");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn the_column_stops_growing_at_the_limit() {
        let mut ledger = Ledger::open(None);
        ledger.note("create", 0, (LIMIT + 50) as i64);
        assert_eq!(ledger.entries().len(), LIMIT);
        assert_eq!(ledger.entries().last().unwrap().head, Some((LIMIT + 50) as i64));
    }
}
