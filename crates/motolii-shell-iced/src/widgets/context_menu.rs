//! 右クリックメニュー — overlay。項目を選ぶか、外を押す / Esc で消える。
//!
//! 出す・消すの判断は呼び出し側が持つ(このメニューは自分を閉じない —
//! [`MenuEvent::Chosen`] / [`MenuEvent::Dismissed`] を言うだけで、view から
//! 外すのは消費側の update である)。

use crate::widgets::palette;

/// メニューの1項目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    /// 選ばれた時に [`MenuEvent::Chosen`] が運ぶ id。
    pub id: u32,
    /// 表示する字。
    pub label: String,
    /// false なら「今この文脈で無効」(Q0 — 未実装の飾りには使わない)。
    pub enabled: bool,
}

/// メニューの語彙。**この enum が公開契約**(消費側 capsule と同文)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEvent {
    /// 有効な項目が選ばれた。
    Chosen(u32),
    /// 外を押した、または Esc。
    Dismissed,
}

/// 1項目の高さ。
pub const ITEM_H: f32 = 22.0;
/// メニューの上下 padding。
pub const MENU_PAD_Y: f32 = 4.0;
/// 項目の左右 padding。
pub const ITEM_PAD_X: f32 = 12.0;
/// メニューの最小幅。
pub const MENU_MIN_W: f32 = 120.0;

/// `index` 番目の項目の当たり中心(メニューが `at` に出た時)。
/// テストと消費側が同じ幾何を見るための口。窓端 clamp が効いた場合はずれる。
pub fn item_position(at: iced::Point, index: usize) -> iced::Point {
    iced::Point::new(
        at.x + ITEM_PAD_X,
        at.y + MENU_PAD_Y + ITEM_H * (index as f32 + 0.5),
    )
}

/// メニューを1つ組む。`at` は窓座標(右クリック位置)。
pub fn context_menu<'a, M>(
    items: Vec<MenuItem>,
    at: iced::Point,
    on_event: impl Fn(MenuEvent) -> M + 'a,
) -> iced::Element<'a, M>
where
    M: 'a,
{
    let _ = (items, at, on_event, palette::PALETTE);
    todo!("red 先行 — 実装は次コミット")
}
