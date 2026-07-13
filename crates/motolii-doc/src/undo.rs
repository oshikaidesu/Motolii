//! D2: Undo/Redo履歴。1 gesture=1 macro(#103⑨)、merge key=S18、
//! Undo深さはlive/再起動後で別limit・既定0=unlimited(残小項目【決定】2026-07-13)。
//!
//! 深さの数値は仕様真理ではない — 呼び出し側(アプリ設定)が注入する運用値。

use thiserror::Error;

use crate::command::{Command, CommandError, GestureId};
use crate::Document;

/// Undoの深さ設定。`0`=unlimited(Qt既定)。呼び出し側の運用設定であり、
/// Documentスキーマには焼かない(GR-PV-2: 恒久面を狭く保つ)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoLimit {
    Unlimited,
    Bounded(usize),
}

impl UndoLimit {
    /// `n == 0` → unlimited。それ以外は`n`個のgesture(macro)を上限にする。
    pub const fn from_setting(n: u32) -> Self {
        if n == 0 {
            Self::Unlimited
        } else {
            Self::Bounded(n as usize)
        }
    }
}

impl Default for UndoLimit {
    fn default() -> Self {
        Self::Unlimited
    }
}

/// 1 gesture = 1 macro(#103⑨)。undo/redoは常にmacro単位で丸ごと適用/逆適用する。
#[derive(Debug, Clone, PartialEq)]
pub struct Macro {
    pub gesture: GestureId,
    pub commands: Vec<Command>,
}

impl Macro {
    /// 同一merge key(S18)のcommandを1つに畳む。無ければ末尾に追加する。
    fn merge_or_push(&mut self, command: Command) {
        let key = command.merge_key(self.gesture);
        let existing = self
            .commands
            .iter()
            .position(|c| c.merge_key(self.gesture) == key);
        match existing {
            Some(i) => self.commands[i] = merge_pair(&self.commands[i], command),
            None => self.commands.push(command),
        }
    }
}

/// 同一merge keyの2コマンドを1つへ畳む: 先頭の`old`を残し、後着の`new`を採る。
/// (構造系Add/Remove/AddTrackItem/RemoveTrackItemは同一gesture内で繰り返される想定が
/// ないため、後着をそのまま採用する — 呼び出し規律の誤りを隠さずそのまま反映する)
fn merge_pair(first: &Command, second: Command) -> Command {
    match (first.clone(), second) {
        (
            Command::SetProperty {
                target,
                property,
                old_value,
                ..
            },
            Command::SetProperty { new_value, .. },
        ) => Command::SetProperty {
            target,
            property,
            old_value,
            new_value,
        },
        (Command::SetBlendMode { target, old, .. }, Command::SetBlendMode { new, .. }) => {
            Command::SetBlendMode { target, old, new }
        }
        (Command::SetClippingMask { target, old, .. }, Command::SetClippingMask { new, .. }) => {
            Command::SetClippingMask { target, old, new }
        }
        (
            Command::SetTransformParent { target, old, .. },
            Command::SetTransformParent { new, .. },
        ) => Command::SetTransformParent { target, old, new },
        (
            Command::SetEffectEnabled {
                target,
                effect,
                old,
                ..
            },
            Command::SetEffectEnabled { new, .. },
        ) => Command::SetEffectEnabled {
            target,
            effect,
            old,
            new,
        },
        (_, second) => second,
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum UndoError {
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error("nothing to undo")]
    NothingToUndo,
    #[error("nothing to redo")]
    NothingToRedo,
}

/// Undo/Redo履歴本体。liveはpush時、restartは復元時にそれぞれのlimitでトリムする。
#[derive(Debug, Clone)]
pub struct UndoHistory {
    live_limit: UndoLimit,
    restart_limit: UndoLimit,
    undo_stack: Vec<Macro>,
    redo_stack: Vec<Macro>,
}

impl UndoHistory {
    pub fn new(live_limit: UndoLimit, restart_limit: UndoLimit) -> Self {
        Self {
            live_limit,
            restart_limit,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// 再起動後(ジャーナルreplay等)の履歴復元。`restart_limit`でトリムする
    /// (liveとは別limit — 残小項目【決定】)。復元経路そのものはD1d/D1eの領分で、
    /// ここは「渡されたmacro列をどう刈るか」の設定面だけを持つ。
    pub fn from_restored(
        macros: Vec<Macro>,
        live_limit: UndoLimit,
        restart_limit: UndoLimit,
    ) -> Self {
        let mut history = Self {
            live_limit,
            restart_limit,
            undo_stack: macros,
            redo_stack: Vec::new(),
        };
        history.trim_to_restart();
        history
    }

    fn trim_front_to(stack: &mut Vec<Macro>, limit: UndoLimit) {
        if let UndoLimit::Bounded(max) = limit {
            while stack.len() > max {
                stack.remove(0);
            }
        }
    }

    fn trim_to_live(&mut self) {
        Self::trim_front_to(&mut self.undo_stack, self.live_limit);
    }

    fn trim_to_restart(&mut self) {
        Self::trim_front_to(&mut self.undo_stack, self.restart_limit);
    }

    pub fn undo_len(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_len(&self) -> usize {
        self.redo_stack.len()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// commandを適用し、履歴へ積む(同一gesture内は merge key で畳む — #103⑨・S18)。
    /// 新規editはredoを破棄する(標準的なundo系の挙動)。
    pub fn push(
        &mut self,
        doc: &mut Document,
        gesture: GestureId,
        command: Command,
    ) -> Result<(), CommandError> {
        command.apply(doc)?;
        self.redo_stack.clear();
        match self.undo_stack.last_mut() {
            Some(top) if top.gesture == gesture => top.merge_or_push(command),
            _ => {
                self.undo_stack.push(Macro {
                    gesture,
                    commands: vec![command],
                });
                self.trim_to_live();
            }
        }
        Ok(())
    }

    /// 直前のmacro(1 gesture分)を丸ごと逆適用する。
    pub fn undo(&mut self, doc: &mut Document) -> Result<(), UndoError> {
        let popped = self.undo_stack.pop().ok_or(UndoError::NothingToUndo)?;
        for command in popped.commands.iter().rev() {
            command.inverse().apply(doc)?;
        }
        self.redo_stack.push(popped);
        Ok(())
    }

    /// 直前にundoしたmacroを丸ごと再適用する。
    pub fn redo(&mut self, doc: &mut Document) -> Result<(), UndoError> {
        let popped = self.redo_stack.pop().ok_or(UndoError::NothingToRedo)?;
        for command in &popped.commands {
            command.apply(doc)?;
        }
        self.undo_stack.push(popped);
        Ok(())
    }
}
