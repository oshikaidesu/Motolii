//! モデルと `update` — この殻の唯一の可変状態。
//!
//! 中身は `ShellGateway` 1つだけである。座席も transcript も journal も
//! ゲートウェイの中に在り、この型からは**読みしか出せない**。
//! 「journal を通らずに製品状態へ着く道」を新しい殻でも作らない、という
//! 2026-08-18 の構造の強制をそのまま引き継いでいる
//! (柵: `tests/intent_gateway_fence.rs`)。

use motolii_ui::blitz_shell::{IntentEvent, ShellGateway, ShellTranscript, UiIntent};

use crate::message::Message;
use crate::prompts::ShellPrompts;

/// iced ホストの shell 状態。
pub struct Shell {
    /// 製品状態へ触れる唯一の口。**この crate は他の道を持たない。**
    gateway: ShellGateway,
    /// 人に訊く口。窓は `NativePrompts`、テスト・CLI 駆動は `ScriptedPrompts`。
    prompts: Box<dyn ShellPrompts>,
}

impl Shell {
    /// 座席なしで始める(スタート画面)。
    pub fn new(prompts: impl ShellPrompts + 'static) -> Self {
        Self {
            gateway: ShellGateway::new(ShellTranscript::default()),
            prompts: Box::new(prompts),
        }
    }

    /// Message 1件を受ける。**ここが Message → `UiIntent` の唯一の写像**である。
    ///
    /// dialog が答えなければ何も起きない(intent も記録されない)。
    /// 「起こそうとした行動」だけが journal に載る、という規律は egui shell と同じ。
    pub fn update(&mut self, message: Message) {
        match message {
            Message::NewProjectPressed => {
                let Some(path) = self.prompts.new_project_path() else {
                    return;
                };
                let _ = self.gateway.dispatch(UiIntent::NewProject { path });
            }
            Message::OpenProjectPressed => {
                let Some(path) = self.prompts.open_project_path() else {
                    return;
                };
                let _ = self.gateway.dispatch(UiIntent::OpenProject { path });
            }
        }
    }

    /// live project が座っているか。スタート画面を出すかどうかがこれで決まる。
    pub fn is_seated(&self) -> bool {
        self.gateway.is_seated()
    }

    /// status 帯が映す最新の一言。何も言われていなければ帯を出さない。
    pub fn latest_report(&self) -> Option<String> {
        self.gateway.transcript().latest()
    }

    /// 原因のログ全文(順のまま)。`--intent-log` と replay がこれを読む。
    pub fn intents(&self) -> Vec<IntentEvent> {
        self.gateway.journal().entries()
    }

    /// 既に `count` 行流した側が、その後に増えた分だけ受け取る([`IntentLog`] 用)。
    ///
    /// [`IntentLog`]: crate::IntentLog
    pub(crate) fn intents_since(&self, count: usize) -> Vec<IntentEvent> {
        self.gateway.journal().since(count)
    }

    /// journal に溜まっている行数。
    pub fn intent_count(&self) -> usize {
        self.gateway.journal().len()
    }
}
