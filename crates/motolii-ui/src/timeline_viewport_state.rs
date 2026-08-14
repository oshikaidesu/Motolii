//! egui Timeline の時間・スクロール表示状態。
//!
//! Document の意味や編集を持たず、Skia/Godot 系 Timeline の viewport 操作だけを
//! `TimelineViewport` へ変換する。編集 command、selection、easing はここへ入れない。

use motolii_core::RationalTime;

use crate::timeline_projection::TimelineViewport;

const TIME_QUANTUM_DENOMINATOR: i64 = 1_000_000;
const DEFAULT_MIN_SPAN_SECONDS: f64 = 0.001;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum TimelineViewportStateError {
    #[error("composition duration must be positive")]
    InvalidDuration,
    #[error("viewport span must be positive and finite")]
    InvalidSpan,
    #[error("viewport geometry must be finite and positive")]
    InvalidGeometry,
    #[error("viewport time conversion overflowed")]
    TimeOverflow,
}

/// Timeline の表示範囲と縦スクロールだけを保持する。
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TimelineViewportState {
    composition_duration: RationalTime,
    viewport: TimelineViewport,
    min_span_seconds: f64,
    vertical_offset: f32,
}

impl TimelineViewportState {
    pub(crate) fn new(
        composition_duration: RationalTime,
    ) -> Result<Self, TimelineViewportStateError> {
        let duration_seconds = composition_duration.as_seconds_f64();
        if composition_duration <= RationalTime::ZERO
            || !duration_seconds.is_finite()
            || duration_seconds <= 0.0
        {
            return Err(TimelineViewportStateError::InvalidDuration);
        }

        Ok(Self {
            composition_duration,
            viewport: TimelineViewport {
                start: RationalTime::ZERO,
                end: composition_duration,
            },
            min_span_seconds: DEFAULT_MIN_SPAN_SECONDS.min(duration_seconds),
            vertical_offset: 0.0,
        })
    }

    pub(crate) fn viewport(&self) -> TimelineViewport {
        self.viewport
    }

    pub(crate) fn vertical_offset(&self) -> f32 {
        self.vertical_offset
    }

    /// Overview の選択範囲を composition の割合で指定する。
    pub(crate) fn select_overview(
        &mut self,
        start_fraction: f32,
        end_fraction: f32,
    ) -> Result<(), TimelineViewportStateError> {
        if !start_fraction.is_finite()
            || !end_fraction.is_finite()
            || !(0.0..=1.0).contains(&start_fraction)
            || !(0.0..=1.0).contains(&end_fraction)
            || end_fraction <= start_fraction
        {
            return Err(TimelineViewportStateError::InvalidSpan);
        }
        let duration = self.composition_duration.as_seconds_f64();
        self.set_viewport_seconds(
            f64::from(start_fraction) * duration,
            f64::from(end_fraction) * duration,
        )
    }

    /// 横ホイール／横パンを pointer 位置に依存せず適用する。
    pub(crate) fn pan_horizontal_pixels(
        &mut self,
        delta_pixels: f32,
        viewport_width: f32,
    ) -> Result<(), TimelineViewportStateError> {
        if !delta_pixels.is_finite() || !viewport_width.is_finite() || viewport_width <= 0.0 {
            return Err(TimelineViewportStateError::InvalidGeometry);
        }
        let span = self.viewport_span_seconds();
        let delta_seconds = -f64::from(delta_pixels) * span / f64::from(viewport_width);
        self.set_viewport_seconds(
            self.viewport.start.as_seconds_f64() + delta_seconds,
            self.viewport.end.as_seconds_f64() + delta_seconds,
        )
    }

    /// 縦ホイールを row content の範囲へ clamp する。
    pub(crate) fn scroll_vertical_pixels(
        &mut self,
        delta_pixels: f32,
        content_height: f32,
        viewport_height: f32,
    ) -> Result<(), TimelineViewportStateError> {
        if !delta_pixels.is_finite()
            || !content_height.is_finite()
            || !viewport_height.is_finite()
            || content_height < 0.0
            || viewport_height < 0.0
        {
            return Err(TimelineViewportStateError::InvalidGeometry);
        }
        let max_offset = (content_height - viewport_height).max(0.0);
        self.vertical_offset = (self.vertical_offset - delta_pixels).clamp(0.0, max_offset);
        Ok(())
    }

    /// Cmd/Ctrl+wheel の zoom。anchor の時間を同じ画面位置に残す。
    /// `zoom_factor > 1` が拡大、`0 < zoom_factor < 1` が縮小。
    pub(crate) fn zoom_at(
        &mut self,
        anchor_pixels: f32,
        viewport_width: f32,
        zoom_factor: f32,
    ) -> Result<(), TimelineViewportStateError> {
        if !anchor_pixels.is_finite()
            || !viewport_width.is_finite()
            || viewport_width <= 0.0
            || !zoom_factor.is_finite()
            || zoom_factor <= 0.0
        {
            return Err(TimelineViewportStateError::InvalidGeometry);
        }
        let anchor_fraction =
            (f64::from(anchor_pixels) / f64::from(viewport_width)).clamp(0.0, 1.0);
        let old_start = self.viewport.start.as_seconds_f64();
        let old_span = self.viewport_span_seconds();
        let anchor_time = old_start + old_span * anchor_fraction;
        let duration = self.composition_duration.as_seconds_f64();
        let new_span = (old_span / f64::from(zoom_factor)).clamp(self.min_span_seconds, duration);
        let new_start = anchor_time - new_span * anchor_fraction;
        self.set_viewport_seconds(new_start, new_start + new_span)
    }

    /// Skia/Godot の wheel 分岐を state 層だけで処理する。
    pub(crate) fn apply_wheel(
        &mut self,
        delta_x: f32,
        delta_y: f32,
        command_modifier: bool,
        anchor_pixels: Option<f32>,
        viewport_width: f32,
        content_height: f32,
        viewport_height: f32,
    ) -> Result<(), TimelineViewportStateError> {
        if command_modifier {
            if let Some(anchor_pixels) = anchor_pixels {
                if delta_y != 0.0 {
                    let magnitude = (1.0 + delta_y.abs() * 0.01).clamp(1.01, 2.0);
                    let factor = if delta_y > 0.0 {
                        magnitude
                    } else {
                        1.0 / magnitude
                    };
                    self.zoom_at(anchor_pixels, viewport_width, factor)?;
                }
            }
        } else {
            self.pan_horizontal_pixels(delta_x, viewport_width)?;
            self.scroll_vertical_pixels(delta_y, content_height, viewport_height)?;
        }
        Ok(())
    }

    /// key / playhead / marker へ pixel 閾値で吸着する。
    pub(crate) fn snap_time(
        &self,
        time: RationalTime,
        candidates: &[RationalTime],
        threshold_pixels: f32,
        viewport_width: f32,
    ) -> Option<RationalTime> {
        if !threshold_pixels.is_finite()
            || threshold_pixels < 0.0
            || !viewport_width.is_finite()
            || viewport_width <= 0.0
        {
            return None;
        }
        let span = self.viewport_span_seconds();
        let threshold_seconds = f64::from(threshold_pixels) * span / f64::from(viewport_width);
        let time_seconds = time.as_seconds_f64();
        candidates
            .iter()
            .copied()
            .filter(|candidate| {
                *candidate >= self.viewport.start && *candidate <= self.viewport.end
            })
            .filter_map(|candidate| {
                let distance = (candidate.as_seconds_f64() - time_seconds).abs();
                (distance <= threshold_seconds).then_some((distance, candidate))
            })
            .min_by(|(left, _), (right, _)| left.total_cmp(right))
            .map(|(_, candidate)| candidate)
    }

    fn viewport_span_seconds(&self) -> f64 {
        self.viewport.end.as_seconds_f64() - self.viewport.start.as_seconds_f64()
    }

    fn set_viewport_seconds(
        &mut self,
        mut start_seconds: f64,
        mut end_seconds: f64,
    ) -> Result<(), TimelineViewportStateError> {
        let duration = self.composition_duration.as_seconds_f64();
        let span = end_seconds - start_seconds;
        if !start_seconds.is_finite()
            || !end_seconds.is_finite()
            || !span.is_finite()
            || span <= 0.0
        {
            return Err(TimelineViewportStateError::InvalidSpan);
        }
        let span = span.clamp(self.min_span_seconds, duration);
        end_seconds = start_seconds + span;
        if start_seconds < 0.0 {
            end_seconds -= start_seconds;
            start_seconds = 0.0;
        }
        if end_seconds > duration {
            start_seconds -= end_seconds - duration;
            end_seconds = duration;
        }
        start_seconds = start_seconds.max(0.0);
        end_seconds = end_seconds.min(duration);
        if end_seconds <= start_seconds {
            return Err(TimelineViewportStateError::InvalidSpan);
        }
        self.viewport = TimelineViewport {
            start: seconds_to_time(start_seconds)?,
            end: seconds_to_time(end_seconds)?,
        };
        Ok(())
    }
}

fn seconds_to_time(seconds: f64) -> Result<RationalTime, TimelineViewportStateError> {
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(TimelineViewportStateError::TimeOverflow);
    }
    let scaled = seconds * TIME_QUANTUM_DENOMINATOR as f64;
    if !scaled.is_finite() || scaled > i64::MAX as f64 {
        return Err(TimelineViewportStateError::TimeOverflow);
    }
    RationalTime::try_new(scaled.round() as i64, TIME_QUANTUM_DENOMINATOR)
        .map_err(|_| TimelineViewportStateError::TimeOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seconds(value: i64) -> RationalTime {
        RationalTime::try_new(value, 1).unwrap()
    }

    #[test]
    fn overview_and_pan_stay_inside_composition() {
        let mut state = TimelineViewportState::new(seconds(10)).unwrap();
        state.select_overview(0.25, 0.75).unwrap();
        assert_eq!(
            state.viewport().start.as_seconds_f64(),
            2.5,
            "0.25 of a 10s composition is 2.5s"
        );
        assert_eq!(
            state.viewport().end.as_seconds_f64(),
            7.5,
            "0.75 of a 10s composition is 7.5s"
        );

        state.pan_horizontal_pixels(-10_000.0, 1000.0).unwrap();
        assert_eq!(state.viewport().end, seconds(10));
    }

    #[test]
    fn zoom_preserves_pointer_anchor_time() {
        let mut state = TimelineViewportState::new(seconds(10)).unwrap();
        state.select_overview(0.0, 1.0).unwrap();
        state.zoom_at(250.0, 1000.0, 2.0).unwrap();
        let viewport = state.viewport();
        assert!((viewport.start.as_seconds_f64() - 1.25).abs() < 0.000002);
        assert!((viewport.end.as_seconds_f64() - 6.25).abs() < 0.000002);
    }

    #[test]
    fn snap_uses_pixel_threshold_and_viewport_candidates() {
        let mut state = TimelineViewportState::new(seconds(10)).unwrap();
        state.select_overview(0.0, 1.0).unwrap();
        let candidates = [seconds(2), seconds(8)];
        assert_eq!(
            state.snap_time(
                seconds(2)
                    .try_add(RationalTime::try_new(1, 100).unwrap())
                    .unwrap(),
                &candidates,
                12.0,
                1000.0
            ),
            Some(seconds(2))
        );
        assert_eq!(state.snap_time(seconds(9), &candidates, 12.0, 1000.0), None);
    }

    #[test]
    fn wheel_separates_zoom_from_pan_and_vertical_scroll() {
        let mut state = TimelineViewportState::new(seconds(10)).unwrap();
        state
            .apply_wheel(10.0, -20.0, false, None, 1000.0, 500.0, 100.0)
            .unwrap();
        assert!(state.vertical_offset() > 0.0);
        let before = state.viewport();
        state
            .apply_wheel(0.0, 20.0, true, Some(500.0), 1000.0, 500.0, 100.0)
            .unwrap();
        assert!(state.viewport().end > before.start);
        let after_span = state
            .viewport()
            .end
            .try_sub(state.viewport().start)
            .expect("zoomed span");
        let before_span = before.end.try_sub(before.start).expect("prior span");
        assert!(after_span < before_span);
    }
}
