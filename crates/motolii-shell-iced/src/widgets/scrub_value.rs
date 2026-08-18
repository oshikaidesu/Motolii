//! 数値スクラブ — AE / AM 文法の主食。**ドラッグで増減・ダブルクリックでテキスト入力**。
//!
//! egui shell では `egui::DragValue`(`inspector_panel/mod.rs` の
//! `draw_value_field`)が担っていた席。イベント語彙は発注 capsule で固定:
//! gesture は必ず [`ScrubEvent::Started`] で開き、[`ScrubEvent::Committed`]
//! (release / Enter = 確定)か [`ScrubEvent::Cancelled`](Esc = 復元)で閉じる。
//! 途中の値は [`ScrubEvent::Changed`] が同 tick で運ぶ(Q3 / B3)。

use crate::widgets::palette;

/// scrub gesture の語彙。**この enum が公開契約**(消費側 capsule と同文)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrubEvent {
    /// gesture が開いた(ドラッグの初動、またはテキスト編集の開始)。
    /// 消費側はここで復元用の値を控える(egui 版の `BeginEdit` に当たる)。
    Started,
    /// ドラッグ中の値(clamp / integer snap 済み)。同じ値は二度言わない。
    Changed(f64),
    /// 確定(release、または Enter)。egui 版の `EndEdit` に当たる。
    Committed(f64),
    /// 取り消し(Esc)。値の復元は `Started` で控えた側の仕事。
    Cancelled,
}

/// 1つの scrub 欄の仕様。値の正本は呼び出し側(Q5 — widget は値を所有しない)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrubSpec {
    /// いま表示する値。
    pub value: f64,
    /// 表示桁(固定)。
    pub decimals: usize,
    /// 下限(あれば clamp)。
    pub min: Option<f64>,
    /// 上限(あれば clamp)。
    pub max: Option<f64>,
    /// ドラッグ 1px あたりの増分。
    pub step: f64,
    /// true なら整数へ snap する。
    pub integer: bool,
}

impl ScrubSpec {
    /// clamp と integer snap をこの順で通す(snap してから clamp — 上限 12 で
    /// 12.4 が 12 を越えて見えることがない)。
    fn constrain(&self, value: f64) -> f64 {
        let mut value = if self.integer { value.round() } else { value };
        if let Some(min) = self.min {
            value = value.max(min);
        }
        if let Some(max) = self.max {
            value = value.min(max);
        }
        value
    }

    fn format(&self, value: f64) -> String {
        format_value(value, self.decimals)
    }
}

/// 表示は `decimals` 桁固定(発注 capsule)。
pub fn format_value(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

/// scrub 欄を1つ組む。ドラッグで増減、ダブルクリックでテキスト入力。
pub fn scrub_value<'a, M>(
    spec: ScrubSpec,
    on_event: impl Fn(ScrubEvent) -> M + 'a,
) -> iced::Element<'a, M>
where
    M: 'a,
{
    let _ = (spec, on_event, palette::PALETTE);
    todo!("red 先行 — 実装は次コミット")
}
