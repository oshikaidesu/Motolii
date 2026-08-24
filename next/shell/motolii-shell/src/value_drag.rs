//! Inspectorの横断値ドラッグadapter。
//!
//! responsibility: wire
//! Composition/AutoSave/Background/Text色の宛先を束ねるが、各domainの意味は所有せず、
//! press→draft→releaseの共通gestureとdomain moduleへの委譲だけを担当する。

use crate::{inspector_pane, settings_pane, Shell};

/// [`ValueDragState`] が指す先。4つの家系(Composition/AutoSave/Background/
/// TextDocumentStyle 色)を1つの `Option` へ束ねる — press は排他的に1つしか
/// 起きない(`inspector_drag`/`inspector_text_style_drag` の排他と同じ形)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ValueDragTarget {
    CompWidth,
    CompHeight,
    CompFps,
    CompDuration,
    AutoSaveIntervalMinutes,
    AutoSaveGenerations,
    Background(settings_pane::BackgroundChannel),
    Color(
        inspector_pane::color::ColorTarget,
        inspector_pane::color::ColorChannel,
    ),
}

/// 値セルのキャプション drag-to-scrub、進行中の一時状態。
/// [`inspector_pane::FieldDragState`] と同じ形の縮小版 — Document の
/// transient overlay は使わない(対象に `LayerId + PropertyId` の宛先が無い
/// 家系がある)。**move 中は「既存の draft へ書き戻すだけ」** — text_input が
/// 下書きから表示を読む既存の経路(`comp_field_cell`/`channel_cell` 等)を
/// そのまま使うので、drag 中の値も Enter 編集中と同じ見た目になる。
pub(crate) struct ValueDragState {
    target: ValueDragTarget,
    /// press 時点の値(対象ごとの「表示単位」— px・fps 小数・フレーム数・分・
    /// 世代数・0..255 チャンネル)。
    start_value: f64,
    /// 最初の `PointerMoved` で確定する基準 x。`None` の間は click か drag か
    /// まだ未確定(`FieldDragState::origin_x` と同じ理由)。
    origin_x: Option<f32>,
    /// 少なくとも1回動いたか。release の確定要否の判定に使う。
    moved: bool,
}

/// [`ValueDragTarget`] ごとの px あたりの感度。`inspector_pane::transform::
/// drag_step_per_pixel` と同じ「値の意味域に合わせた目安」(実窓較正はこの
/// 発注の範囲外)。
fn value_drag_step_per_pixel(target: ValueDragTarget) -> f64 {
    match target {
        // 解像度・尺は 1px = 1単位(Position と同じ 1:1、`drag_step_per_pixel` 参照)。
        ValueDragTarget::CompWidth
        | ValueDragTarget::CompHeight
        | ValueDragTarget::CompDuration => 1.0,
        // fps は 1..240 の域を 100px 強で走査できる程度。
        ValueDragTarget::CompFps => 0.1,
        // 間隔(分)は 1..1440 の域。
        ValueDragTarget::AutoSaveIntervalMinutes => 0.5,
        // 世代数は 1..50 の域、10px で1段動く。
        ValueDragTarget::AutoSaveGenerations => 0.1,
        // RGBA は 0..255、Position と同じ 1:1。
        ValueDragTarget::Background(_) | ValueDragTarget::Color(_, _) => 1.0,
    }
}

impl Shell {
    /// 値セルのキャプション press — click か drag かはまだ未確定
    /// (`ValueDragState::origin_x` が `None` のまま、`start_field_drag` と
    /// 同じ形)。対応する値が読めない(comp が無い・選択レイヤが無い等)なら
    /// 黙って無視 — drag は始まらない。既に別の drag が進行中なら多重起動しない。
    pub(crate) fn start_value_drag(&mut self, target: ValueDragTarget) {
        if self.value_drag.is_some() {
            return;
        }
        let Some(start_value) = self.value_drag_start_value(target) else {
            return;
        };
        self.value_drag = Some(ValueDragState {
            target,
            start_value,
            origin_x: None,
            moved: false,
        });
    }

    /// press 時点の「表示単位」の現在値。読み取りは各domainへ委譲する。
    fn value_drag_start_value(&self, target: ValueDragTarget) -> Option<f64> {
        match target {
            ValueDragTarget::CompWidth
            | ValueDragTarget::CompHeight
            | ValueDragTarget::CompFps
            | ValueDragTarget::CompDuration => self.composition_value_drag_start_value(target),
            ValueDragTarget::AutoSaveIntervalMinutes
            | ValueDragTarget::AutoSaveGenerations
            | ValueDragTarget::Background(_) => self.settings_value_drag_start_value(target),
            ValueDragTarget::Color(_, _) => self.color_value_drag_start_value(target),
        }
    }

    /// window 全体の cursor 移動。drag が armed/dragging でなければ即 no-op
    /// (`continue_field_drag` と同じ形)。draftの書き戻しはdomainへ委譲する。
    pub(crate) fn continue_value_drag(&mut self, point: iced::Point) {
        let Some(state) = self.value_drag.as_mut() else {
            return;
        };
        let Some(origin_x) = state.origin_x else {
            state.origin_x = Some(point.x);
            return;
        };
        let delta_px = point.x - origin_x;
        if delta_px == 0.0 && !state.moved {
            return;
        }
        let target = state.target;
        let start_value = state.start_value;
        let fine = self.keyboard_modifiers.shift();
        let factor = if fine {
            inspector_pane::DRAG_SHIFT_FACTOR
        } else {
            1.0
        };
        let raw = start_value + f64::from(delta_px) * value_drag_step_per_pixel(target) * factor;
        match target {
            ValueDragTarget::CompWidth
            | ValueDragTarget::CompHeight
            | ValueDragTarget::CompFps
            | ValueDragTarget::CompDuration => self.write_composition_value_drag_draft(target, raw),
            ValueDragTarget::AutoSaveIntervalMinutes
            | ValueDragTarget::AutoSaveGenerations
            | ValueDragTarget::Background(_) => self.write_settings_value_drag_draft(target, raw),
            ValueDragTarget::Color(_, _) => self.write_color_value_drag_draft(target, raw),
        }
        if let Some(state) = self.value_drag.as_mut() {
            state.moved = true;
        }
    }

    /// 左クリック release。確定側もdomainの既存 commit_*へ委譲する。
    pub(crate) fn finish_value_drag(&mut self) {
        let Some(state) = self.value_drag.take() else {
            return;
        };
        if !state.moved {
            return;
        }
        match state.target {
            ValueDragTarget::CompWidth
            | ValueDragTarget::CompHeight
            | ValueDragTarget::CompFps
            | ValueDragTarget::CompDuration => self.finish_composition_value_drag(state.target),
            ValueDragTarget::AutoSaveIntervalMinutes
            | ValueDragTarget::AutoSaveGenerations
            | ValueDragTarget::Background(_) => self.finish_settings_value_drag(state.target),
            ValueDragTarget::Color(_, _) => self.finish_color_value_drag(state.target),
        }
    }
}
