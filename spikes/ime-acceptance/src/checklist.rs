//! M3実装ガード1の4項目チェックリスト定義と合否記録マニフェスト。
//! 実機審判は開発主機GUI — 本モジュールは記録形式の正本。

use serde::{Deserialize, Serialize};

pub const TICKET: &str = "M3-GUARD-1";
pub const ISSUE: &str = "#56";

/// チェックリスト項目 ID (仕様 [M3-ui-integration.md](../../docs/specs/M3-ui-integration.md) 実装ガード1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChecklistId {
    PreeditUnderline,
    CandidateFollowsCursor,
    EnterNotEatenByShortcut,
    LongLyricContinuousInput,
}

impl ChecklistId {
    pub const ALL: [ChecklistId; 4] = [
        ChecklistId::PreeditUnderline,
        ChecklistId::CandidateFollowsCursor,
        ChecklistId::EnterNotEatenByShortcut,
        ChecklistId::LongLyricContinuousInput,
    ];

    pub fn spec_text(self) -> &'static str {
        match self {
            ChecklistId::PreeditUnderline => "preedit下線表示",
            ChecklistId::CandidateFollowsCursor => "候補ウィンドウがカーソル位置に追従",
            ChecklistId::EnterNotEatenByShortcut => {
                "変換中のEnterがアプリのショートカットに食われない"
            }
            ChecklistId::LongLyricContinuousInput => "長文歌詞の連続入力",
        }
    }

    pub fn manual_steps(self) -> &'static str {
        match self {
            ChecklistId::PreeditUnderline => {
                "TextInputにフォーカスし「nihongo」等をローマ字入力。変換前のpreedit文字列に下線(またはIME標準の未確定表示)が出るか確認。"
            }
            ChecklistId::CandidateFollowsCursor => {
                "変換候補を出し、カーソルをフィールド内で移動。候補ウィンドウがカーソル近傍に追従するか確認。"
            }
            ChecklistId::EnterNotEatenByShortcut => {
                "変換未確定のままEnter。下部の「ショートカット発火ログ」にEnterが記録されなければ合格(確定に使われた)。"
            }
            ChecklistId::LongLyricContinuousInput => {
                "READMEの長文歌詞サンプルをコピペまたは連続入力。欠落・化け・異常なカーソル飛びがないか確認。"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Pending,
    Pass,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistEntry {
    pub id: ChecklistId,
    pub spec: String,
    pub verdict: Verdict,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformRun {
    pub platform: String,
    pub ime_backend: String,
    pub display_server: String,
    pub entries: Vec<ChecklistEntry>,
    pub overall: Verdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceManifest {
    pub ticket: &'static str,
    pub issue: &'static str,
    pub spike_crate: &'static str,
    pub recorded_at: String,
    pub runner: String,
    pub environment_note: String,
    pub overall: Verdict,
    pub platforms: Vec<PlatformRun>,
    pub automation: AutomationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationStatus {
    pub build: Verdict,
    pub set_ime_allowed_static_check: Verdict,
    pub checklist_harness: Verdict,
}

impl AcceptanceManifest {
    /// 骨格landed時点のテンプレ — 全項目 pending、合否は偽らない。
    pub fn skeleton_template() -> Self {
        let entries = ChecklistId::ALL
            .map(|id| ChecklistEntry {
                id,
                spec: id.spec_text().into(),
                verdict: Verdict::Pending,
                notes: String::new(),
            })
            .to_vec();

        Self {
            ticket: TICKET,
            issue: ISSUE,
            spike_crate: "spikes/ime-acceptance",
            recorded_at: "未実走".into(),
            runner: String::new(),
            environment_note: "クラウドエージェントはGUI実機なし — 開発主機での実走待ち".into(),
            overall: Verdict::Pending,
            platforms: vec![PlatformRun {
                platform: "(未記入)".into(),
                ime_backend: "(未記入)".into(),
                display_server: "(未記入)".into(),
                entries: entries.clone(),
                overall: Verdict::Pending,
            }],
            automation: AutomationStatus {
                build: Verdict::Pending,
                set_ime_allowed_static_check: Verdict::Pending,
                checklist_harness: Verdict::Pending,
            },
        }
    }
}
