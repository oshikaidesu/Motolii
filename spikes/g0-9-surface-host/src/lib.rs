pub const LEFT_WEBVIEW_WIDTH: f64 = 240.0;
pub const RIGHT_WEBVIEW_WIDTH: f64 = 260.0;
pub const STAGE_SHARE: f64 = 0.72;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceLayout {
    pub logical_width: f64,
    pub logical_height: f64,
    pub scale_factor: f64,
    pub native_x: f32,
    pub native_width: f32,
    pub stage_height: f32,
    pub timeline_y: f32,
    pub timeline_height: f32,
}

impl SurfaceLayout {
    pub fn try_new(physical_width: u32, physical_height: u32, scale_factor: f64) -> Option<Self> {
        if physical_width == 0
            || physical_height == 0
            || !scale_factor.is_finite()
            || scale_factor <= 0.0
        {
            return None;
        }

        let logical_width = f64::from(physical_width) / scale_factor;
        let logical_height = f64::from(physical_height) / scale_factor;
        let native_logical_width = logical_width - LEFT_WEBVIEW_WIDTH - RIGHT_WEBVIEW_WIDTH;
        if native_logical_width <= 1.0 {
            return None;
        }

        let native_x = (LEFT_WEBVIEW_WIDTH * scale_factor) as f32;
        let native_width = (native_logical_width * scale_factor) as f32;
        let stage_height = (f64::from(physical_height) * STAGE_SHARE) as f32;
        let timeline_y = stage_height;
        let timeline_height = physical_height as f32 - stage_height;

        Some(Self {
            logical_width,
            logical_height,
            scale_factor,
            native_x,
            native_width,
            stage_height,
            timeline_y,
            timeline_height,
        })
    }

    pub fn cursor_is_over_webview(self, physical_x: f64) -> bool {
        let left_edge = LEFT_WEBVIEW_WIDTH * self.scale_factor;
        let right_edge = (self.logical_width - RIGHT_WEBVIEW_WIDTH) * self.scale_factor;
        physical_x < left_edge || physical_x >= right_edge
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AcceptanceCounters {
    pub acquire_count: u64,
    pub present_count: u64,
    pub readback_count: u64,
    pub resize_events: u32,
    pub layout_epoch: u64,
    pub native_drag_moves: u32,
    pub native_drag_crossed_webview: bool,
    pub native_drag_released: bool,
    pub web_drag_started: u32,
    pub web_drag_moved: u32,
    pub web_drag_ended: u32,
    pub web_input_events: u32,
}

impl AcceptanceCounters {
    pub fn present_invariant_holds(self) -> bool {
        self.acquire_count > 0
            && self.readback_count == 0
            && self.acquire_count == self.present_count
    }

    pub fn resize_target_passes(self, target: u32) -> bool {
        self.resize_events >= target && self.layout_epoch >= u64::from(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_partitions_one_surface_without_overlap() {
        let layout = SurfaceLayout::try_new(2400, 1600, 2.0).unwrap();
        assert_eq!(layout.logical_width, 1200.0);
        assert_eq!(layout.native_x, 480.0);
        assert_eq!(layout.native_width, 1400.0);
        assert_eq!(layout.stage_height + layout.timeline_height, 1600.0);
        assert_eq!(layout.timeline_y, layout.stage_height);
    }

    #[test]
    fn zero_small_and_invalid_surfaces_are_rejected() {
        assert!(SurfaceLayout::try_new(0, 100, 1.0).is_none());
        assert!(SurfaceLayout::try_new(100, 0, 1.0).is_none());
        assert!(SurfaceLayout::try_new(100, 100, 0.0).is_none());
        assert!(SurfaceLayout::try_new(100, 100, f64::NAN).is_none());
        assert!(SurfaceLayout::try_new(400, 800, 1.0).is_none());
    }

    #[test]
    fn cursor_regions_use_physical_pixels_at_any_scale() {
        let layout = SurfaceLayout::try_new(2400, 1600, 2.0).unwrap();
        assert!(layout.cursor_is_over_webview(479.0));
        assert!(!layout.cursor_is_over_webview(480.0));
        assert!(!layout.cursor_is_over_webview(1879.0));
        assert!(layout.cursor_is_over_webview(1880.0));
    }

    #[test]
    fn acceptance_requires_balanced_present_and_no_readback() {
        let good = AcceptanceCounters {
            acquire_count: 42,
            present_count: 42,
            resize_events: 100,
            layout_epoch: 100,
            ..Default::default()
        };
        assert!(good.present_invariant_holds());
        assert!(good.resize_target_passes(100));

        assert!(!AcceptanceCounters {
            readback_count: 1,
            ..good
        }
        .present_invariant_holds());
        assert!(!AcceptanceCounters {
            present_count: 41,
            ..good
        }
        .present_invariant_holds());
        assert!(!AcceptanceCounters::default().present_invariant_holds());
    }
}
