//! 部品契約の薄い stub — スクラブ値と key ボタン。
//!
//! // INTEGRATION: swap to widgets module
//!
//! 本物は widgets レーンが実装中(2026-08-18 M-4 並走)。ここは**契約どおりの
//! 型と口**だけを先に立てて、Inspector 側の配線とテストを先に進めるための
//! 薄い代替である。差し替えるときはこの module を消して import 先を
//! widgets module へ向けるだけでよいように、**契約の形を変えない**。
//!
//! 契約からの逸脱は1点だけ: `scrub_value` の `M` に `Clone + 'a` を足している。
//! iced の `button` が `Message: Clone` を要求するためで、本物の widget
//! (自前の `Widget` 実装で `on_event` を遅延に呼ぶ)では外れる想定。
//!
//! stub の挙動:
//! - `scrub_value` は「− 値 +」の3枚。押すと `step` ぶん動いた値の
//!   [`ScrubEvent::Committed`] を1発出す(1押し = 1確定 = 1 Undo に写る)。
//!   ドラッグでの `Started` / `Changed` / `Cancelled` は本物の widget が出す。
//! - `key_button` は ◇ / ◆ の1枚。[`KeyState`] は**呼び手が accepted snapshot
//!   から導出して渡す**(この widget は状態を持たない)。

use iced::widget::{button, row, text};
use iced::Element;

/// スクラブ部品が出す事象。`Started`→`Changed`*→(`Committed`|`Cancelled`) の列。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrubEvent {
    Started,
    Changed(f64),
    Committed(f64),
    Cancelled,
}

/// スクラブ部品の1枚ぶんの仕様。`value` は**表示する現在値**で、部品は
/// 値を覚えない(正本は accepted snapshot)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrubSpec {
    pub value: f64,
    pub decimals: usize,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: f64,
    pub integer: bool,
}

impl ScrubSpec {
    /// 範囲へ収める(範囲外の要求は入口で塞ぐ — accepted-no-op を作らない)。
    fn clamp(&self, value: f64) -> f64 {
        let mut value = value;
        if let Some(min) = self.min {
            value = value.max(min);
        }
        if let Some(max) = self.max {
            value = value.min(max);
        }
        if self.integer {
            value = value.round();
        }
        value
    }

    fn label(&self) -> String {
        if self.integer {
            format!("{:.0}", self.value)
        } else {
            format!("{:.*}", self.decimals, self.value)
        }
    }
}

/// スクラブ値1枚。stub は − / + の1押しで `Committed` を出す。
pub fn scrub_value<'a, M: Clone + 'a>(
    spec: ScrubSpec,
    on_event: impl Fn(ScrubEvent) -> M + 'a,
) -> Element<'a, M> {
    let decreased = spec.clamp(spec.value - spec.step);
    let increased = spec.clamp(spec.value + spec.step);
    let mut minus = button(text("\u{2212}").size(12)).padding([1, 6]);
    if decreased != spec.value {
        minus = minus.on_press(on_event(ScrubEvent::Committed(decreased)));
    }
    let mut plus = button(text("+").size(12)).padding([1, 6]);
    if increased != spec.value {
        plus = plus.on_press(on_event(ScrubEvent::Committed(increased)));
    }
    row![minus, text(spec.label()).size(13), plus]
        .spacing(6)
        .align_y(iced::Center)
        .into()
}

/// key ボタンの3状態。**導出元は accepted snapshot だけ**(2026-08-13裁定)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    /// キーを1つも持たない → ◇(灰 outline)
    Unkeyed,
    /// keyframes を持つが playhead には無い → ◇(accent outline)
    Animated,
    /// playhead にキーがある → ◆(accent fill)
    Current,
}

/// key ボタン1枚。押した意味(add か値更新か)は呼び手が状態から決める。
pub fn key_button<'a, M: Clone + 'a>(state: KeyState, on_press: M) -> Element<'a, M> {
    let glyph = match state {
        KeyState::Unkeyed | KeyState::Animated => "\u{25c7}",
        KeyState::Current => "\u{25c6}",
    };
    let style = match state {
        KeyState::Unkeyed => button::secondary as fn(&iced::Theme, button::Status) -> button::Style,
        KeyState::Animated | KeyState::Current => button::primary,
    };
    button(text(glyph).size(12))
        .padding([1, 6])
        .style(style)
        .on_press(on_press)
        .into()
}
