//! product Hostのlogical layout epochとnative viewport写像。

use motolii_core::FrameDesc;

use crate::layout::{PanelLayout, PanelRole};
use crate::layout_geometry::project_panel_rects;

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
    pub(crate) browser: Option<LogicalRect>,
    pub(crate) inspector: Option<LogicalRect>,
    pub(crate) stage: LogicalRect,
    pub(crate) timeline: Option<LogicalRect>,
    pub(crate) stage_physical: PhysicalRect,
    pub(crate) timeline_physical: Option<PhysicalRect>,
}

impl NativeHostLayout {
    pub(crate) fn try_new(
        epoch: u64,
        physical_width: u32,
        physical_height: u32,
        scale_factor: f64,
        frame: FrameDesc,
        authority: &PanelLayout,
    ) -> Result<Option<Self>, NativeHostLayoutError> {
        if physical_width == 0
            || physical_height == 0
            || !scale_factor.is_finite()
            || scale_factor <= 0.0
        {
            return Ok(None);
        }
        let width = f64::from(physical_width) / scale_factor;
        let height = f64::from(physical_height) / scale_factor;
        // taffyの入力境界だけf32へ狭め、非finite/zeroはprojection側で拒否する。
        let rects = project_panel_rects(authority, width as f32, height as f32)
            .ok_or(NativeHostLayoutError::Projection)?;
        let required = |role| {
            rects
                .get(&role)
                .copied()
                .ok_or(NativeHostLayoutError::RequiredRoleUnavailable { role })
        };
        let browser = rects.get(&PanelRole::Browser).copied();
        let inspector = rects.get(&PanelRole::Inspector).copied();
        let stage_panel = required(PanelRole::Stage)?;
        let timeline = rects.get(&PanelRole::Timeline).copied();
        let source_aspect = f64::from(frame.width) / f64::from(frame.height);
        let panel_aspect = stage_panel.width / stage_panel.height;
        let (stage_width, stage_height) = if panel_aspect > source_aspect {
            (stage_panel.height * source_aspect, stage_panel.height)
        } else {
            (stage_panel.width, stage_panel.width / source_aspect)
        };
        let stage_viewport = LogicalRect {
            x: stage_panel.x,
            y: stage_panel.y + header_height,
            width: stage_panel.width,
            height: (stage_panel.height - header_height - transport_height).max(0.0),
        };
        let stage_transport = LogicalRect {
            x: stage_panel.x,
            y: stage_panel.y + stage_panel.height - transport_height,
            width: stage_panel.width,
            height: transport_height,
        };
        let source_aspect = f64::from(frame.width) / f64::from(frame.height);
        let stage_width = (stage_viewport.width * OUTPUT_FRAME_WIDTH_PERCENT / 100.0)
            .min(OUTPUT_FRAME_MAX_WIDTH)
            .min(stage_viewport.height * source_aspect);
        let stage_height = stage_width / source_aspect;
        let stage = LogicalRect {
            x: stage_viewport.x + (stage_viewport.width - stage_width) / 2.0,
            y: stage_viewport.y + (stage_viewport.height - stage_height) / 2.0,
            width: stage_width,
            height: stage_height,
        };
        Ok(Some(Self {
            epoch,
            browser,
            inspector,
            stage,
            timeline,
            stage_physical: physical_rect(stage, scale_factor, physical_width, physical_height)
                .ok_or(NativeHostLayoutError::Projection)?,
            timeline_physical: timeline
                .map(|timeline| {
                    physical_rect(timeline, scale_factor, physical_width, physical_height)
                        .ok_or(NativeHostLayoutError::Projection)
                })
                .transpose()?,
        }))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum NativeHostLayoutError {
    #[error("native Host layout projection failed")]
    Projection,
    #[error("native Host layout required role {role:?} is unavailable")]
    RequiredRoleUnavailable { role: PanelRole },
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
        let layout =
            NativeHostLayout::try_new(7, 1000, 800, 1.0, frame(), &PanelLayout::built_in())
                .unwrap()
                .unwrap();
        assert_eq!(layout.epoch, 7);
        assert_eq!(layout.browser.unwrap().width, 200.0);
        assert_eq!(layout.inspector.unwrap().x, 800.0);
        assert_eq!(layout.inspector.unwrap().width, 200.0);
        assert_eq!(
            layout.timeline.unwrap(),
            LogicalRect {
                x: 0.0,
                y: 600.0,
                width: 1000.0,
                height: 200.0
            }
        );
        assert_eq!(layout.stage_physical.width, 600);
        assert_eq!(layout.timeline_physical.unwrap().height, 200);
    }

    #[test]
    fn stage_hit_test_is_latest_layout_local_and_y_up() {
        let layout =
            NativeHostLayout::try_new(1, 1000, 800, 1.0, frame(), &PanelLayout::built_in())
                .unwrap()
                .unwrap();
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

    #[test]
    fn mutated_authority_reaches_native_host_geometry() {
        let mut authority = PanelLayout::built_in();
        authority
            .apply(
                crate::layout::LayoutAction::Separator {
                    path: vec![0],
                    boundary: 0,
                    action: crate::layout::SeparatorAction::IncreaseLeading,
                },
                crate::layout::LayoutConstraints {
                    viewport_width: 1_000.0,
                    stage_min_width: 320.0,
                },
            )
            .unwrap();
        let layout = NativeHostLayout::try_new(1, 1000, 800, 1.0, frame(), &authority)
            .unwrap()
            .unwrap();
        assert!(layout.browser.unwrap().width > 200.0);
    }

    #[test]
    fn hidden_auxiliary_roles_are_absent_and_stage_consumes_the_freed_space() {
        for role in PanelRole::AUXILIARY {
            let mut authority = PanelLayout::built_in();
            authority
                .apply(
                    crate::layout::LayoutAction::Hide(role),
                    crate::layout::LayoutConstraints {
                        viewport_width: 1_000.0,
                        stage_min_width: 320.0,
                    },
                )
                .unwrap();
            let layout = NativeHostLayout::try_new(1, 1000, 800, 1.0, frame(), &authority)
                .unwrap()
                .unwrap();
            match role {
                PanelRole::Browser => {
                    assert_eq!(layout.browser, None);
                    assert_eq!(layout.stage.x, 0.0);
                    assert!(layout.stage.width > 600.0);
                }
                PanelRole::Inspector => {
                    assert_eq!(layout.inspector, None);
                    assert!(layout.stage.width > 600.0);
                }
                PanelRole::Timeline => {
                    assert_eq!(layout.timeline, None);
                    assert_eq!(layout.timeline_physical, None);
                    assert!(layout.stage.y > 200.0);
                }
                PanelRole::Stage => unreachable!(),
            }
        }
    }

    #[test]
    fn dpi_projection_rounds_only_at_the_physical_boundary() {
        for (physical_width, physical_height, scale) in [(2560, 1600, 2.0), (1920, 1080, 1.25)] {
            let layout = NativeHostLayout::try_new(
                1,
                physical_width,
                physical_height,
                scale,
                frame(),
                &PanelLayout::built_in(),
            )
            .unwrap()
            .unwrap();
            let logical_width = f64::from(physical_width) / scale;
            let browser = layout.browser.unwrap();
            let inspector = layout.inspector.unwrap();
            let tiled_width = browser.width + (inspector.x - browser.width) + inspector.width;
            assert!((tiled_width - logical_width).abs() < 0.001);
            assert!(layout.stage_physical.x + layout.stage_physical.width <= physical_width);
            assert!(layout.stage_physical.y + layout.stage_physical.height <= physical_height);
            assert_eq!(
                layout.timeline_physical.unwrap().y + layout.timeline_physical.unwrap().height,
                physical_height
            );
        }
    }
}
