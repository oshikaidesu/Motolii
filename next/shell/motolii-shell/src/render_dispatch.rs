//! responsibility: wire
//!
//! Messageの領域分配だけを持つWIRE。意味の書き込みは各domain moduleへ委譲し、
//! Shell rootへ機能責任を戻さない。

use iced::Task;

use crate::{Message, Shell};

impl Shell {
    /// `Shell::update` から委譲される領域別 dispatch(2026-08-23 SP-1 レーン、
    /// `docs/reviews/2026-08-23-shell-split-plan.md` の続き)。**中身は無改変** —
    /// 元の巨大な `update()` match の腕をそのままここへ移しただけ(裁定どおり
    /// 移送と委譲だけ、バグ修正・整形は混ぜない)。渡された `message` がこの
    /// 領域の variant でなければ `Err(message)` で突き返す — `crate::dispatch_message`
    /// の chain-of-responsibility が次の領域dispatchへ渡す。**新しい Message 枝は
    /// ここへ腕を1本足すだけで済み、`lib.rs` は触らない**(MC-1 と同じ効能)。
    pub(crate) fn dispatch_render(&mut self, message: Message) -> Result<Task<Message>, Message> {
        let mut task = Task::none();
        match message {
            Message::Settings(msg) => task = self.update_settings(msg),
            Message::Stage(msg) => self.update_stage(msg),
            Message::Gizmo(event) => self.update_gizmo(event),
            Message::ShapeTool(msg) => self.update_shape_tool(msg),
            Message::Sheet(msg) => self.sheet_toggles = self.sheet_toggles.apply(msg),
            Message::Marker(msg) => self.update_marker(msg),
            Message::PaneClicked(pane) => self.panes.set_focused(pane),
            Message::PaneResized(event) => self.panes.apply_resize(event),
            Message::PaneDragged(event) => self.panes.apply_drag(event),
            other => return Err(other),
        }
        Ok(task)
    }
}
