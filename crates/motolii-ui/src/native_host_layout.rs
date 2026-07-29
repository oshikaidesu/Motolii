//! product Hostのlogical layout epochとnative viewport写像。

use motolii_core::FrameDesc;

use crate::layout::{BUILT_IN_TOP_SHARES, BUILT_IN_VERTICAL_SHARES};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LogicalRect {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
}

impl LogicalRect {
    pub(crate) fn contains(self, point: [f64; 2]) -> bool {
        point[0] >= self.x
            && point[0] < self.x + self.width
            && point[1] >= self.y
            && point[1] < self.y + self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PhysicalRect {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct NativeHostLayout {
    pub(crate) epoch: u64,
    pub(crate) browser: LogicalRect,
    pub(crate) stage: LogicalRect,
    pub(crate) timeline: LogicalRect,
    pub(crate) stage_physical: PhysicalRect,
    pub(crate) timeline_physical: PhysicalRect,
}

impl NativeHostLayout {
    pub(crate) fn try_new(
        epoch: u64,
        physical_width: u32,
        physical_height: u32,
        scale_factor: f64,
        frame: FrameDesc,
    ) -> Option<Self> {
        if physical_width == 0
            || physical_height == 0
            || !scale_factor.is_finite()
            || scale_factor <= 0.0
        {
            return None;
        }
        let width = f64::from(physical_width) / scale_factor;
        let height = f64::from(physical_height) / scale_factor;
        let vertical_total = f64::from(BUILT_IN_VERTICAL_SHARES.iter().sum::<u32>());
        let top_height = height * f64::from(BUILT_IN_VERTICAL_SHARES[0]) / vertical_total;
        let top_total = f64::from(BUILT_IN_TOP_SHARES.iter().sum::<u32>());
        let browser_width = width * f64::from(BUILT_IN_TOP_SHARES[0]) / top_total;
        let stage_panel = LogicalRect {
            x: browser_width,
            y: 0.0,
            width: width * f64::from(BUILT_IN_TOP_SHARES[1]) / top_total,
            height: top_height,
        };
        let source_aspect = f64::from(frame.width) / f64::from(frame.height);
        let panel_aspect = stage_panel.width / stage_panel.height;
        let (stage_width, stage_height) = if panel_aspect > source_aspect {
            (stage_panel.height * source_aspect, stage_panel.height)
        } else {
            (stage_panel.width, stage_panel.width / source_aspect)
        };
        let stage = LogicalRect {
            x: stage_panel.x + (stage_panel.width - stage_width) / 2.0,
            y: stage_panel.y + (stage_panel.height - stage_height) / 2.0,
            width: stage_width,
            height: stage_height,
        };
        let timeline = LogicalRect {
            x: 0.0,
            y: top_height,
            width,
            height: height - top_height,
        };
        Some(Self {
            epoch,
            browser: LogicalRect {
                x: 0.0,
                y: 0.0,
                width: browser_width,
                height: top_height,
            },
            stage,
            timeline,
            stage_physical: physical_rect(stage, scale_factor, physical_width, physical_height)?,
            timeline_physical: physical_rect(
                timeline,
                scale_factor,
                physical_width,
                physical_height,
            )?,
        })
    }

    pub(crate) fn stage_ndc(self, point: [f64; 2]) -> Option<[f64; 2]> {
        if !point[0].is_finite() || !point[1].is_finite() || !self.stage.contains(point) {
            return None;
        }
        Some([
            ((point[0] - self.stage.x) / self.stage.width) * 2.0 - 1.0,
            1.0 - ((point[1] - self.stage.y) / self.stage.height) * 2.0,
        ])
    }
}

fn physical_rect(
    logical: LogicalRect,
    scale: f64,
    max_width: u32,
    max_height: u32,
) -> Option<PhysicalRect> {
    let x = (logical.x * scale).round();
    let y = (logical.y * scale).round();
    let right = ((logical.x + logical.width) * scale).round();
    let bottom = ((logical.y + logical.height) * scale).round();
    if !x.is_finite()
        || !y.is_finite()
        || !right.is_finite()
        || !bottom.is_finite()
        || x < 0.0
        || y < 0.0
    {
        return None;
    }
    let x = (x as u32).min(max_width);
    let y = (y as u32).min(max_height);
    let right = (right as u32).min(max_width);
    let bottom = (bottom as u32).min(max_height);
    Some(PhysicalRect {
        x,
        y,
        width: right.saturating_sub(x),
        height: bottom.saturating_sub(y),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_core::{ColorSpace, PixelFormat};

    fn frame() -> FrameDesc {
        FrameDesc::try_packed(1920, 1080, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true).unwrap()
    }

    #[test]
    fn built_in_shares_project_one_stage_and_timeline_viewport() {
        let layout = NativeHostLayout::try_new(7, 1000, 800, 1.0, frame()).unwrap();
        assert_eq!(layout.epoch, 7);
        assert_eq!(layout.browser.width, 200.0);
        assert_eq!(
            layout.timeline,
            LogicalRect {
                x: 0.0,
                y: 600.0,
                width: 1000.0,
                height: 200.0
            }
        );
        assert_eq!(layout.stage_physical.width, 600);
        assert_eq!(layout.timeline_physical.height, 200);
    }

    #[test]
    fn stage_hit_test_is_latest_layout_local_and_y_up() {
        let layout = NativeHostLayout::try_new(1, 1000, 800, 1.0, frame()).unwrap();
        let center = [
            layout.stage.x + layout.stage.width / 2.0,
            layout.stage.y + layout.stage.height / 2.0,
        ];
        assert_eq!(layout.stage_ndc(center), Some([0.0, 0.0]));
        assert_eq!(
            layout.stage_ndc([layout.stage.x, layout.stage.y]),
            Some([-1.0, 1.0])
        );
        assert_eq!(layout.stage_ndc([10.0, 10.0]), None);
    }
}
