//! drop 受け皿 — DnD affordance。hover 中はハイライトし、受入可否を色で言う。
//!
//! ドロップそのものは窓の口([`crate::window_input`] の `FileDropped`)が運ぶ。
//! この widget は「いまどこに落ちるか・受け入れられるか」を**見せる**係で、
//! enter / leave を消費側へ伝える(受入判定 `accepting` の正本は消費側)。

use crate::widgets::palette;

/// drop 面の語彙。**この enum が公開契約**(消費側 capsule と同文)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropEvent {
    /// cursor が面に入った。
    HoverEnter,
    /// cursor が面から出た(窓から出た時も含む)。
    HoverLeave,
}

/// `inner` を drop 受け皿で包む。
pub fn drop_zone<'a, M>(
    inner: iced::Element<'a, M>,
    accepting: bool,
    on_event: impl Fn(DropEvent) -> M + 'a,
) -> iced::Element<'a, M>
where
    M: 'a,
{
    let _ = (inner, accepting, on_event, palette::PALETTE);
    todo!("red 先行 — 実装は次コミット")
}
