//! Tailwind-like utility access to the shared UI theme.
//!
//! `Dimensions` remains the deserialization boundary and JSON source of truth.
//! This module is the public composition API for UI code: common values are
//! selected by semantic namespace (`space`, `text`, `size`, `stroke`, and
//! `target`) instead of by reaching into a component's numeric fields.
//!
//! Pane-specific ratios and geometry stay under `Dimensions::components`.
//! Keeping those values out of this API is intentional: a Browser card ratio
//! must not silently become a Timeline or Inspector convention.

use crate::Dimensions;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiTheme {
    pub space: SpaceScale,
    pub text: TextScale,
    pub size: SizeScale,
    pub stroke: StrokeScale,
    pub target: TargetScale,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpaceScale {
    pub xs: f32,
    pub s: f32,
    pub m: f32,
    pub l: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextScale {
    pub title: f32,
    pub body: f32,
    pub caption: f32,
    pub micro: f32,
}

/// Shared bands only. Inspector cells, graph controls, and transport buttons
/// remain component-owned because their geometry is not a cross-pane scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SizeScale {
    pub row: f32,
    pub transport: f32,
    pub panel_header: f32,
    pub pane_header: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrokeScale {
    pub hairline: f32,
    pub focus: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TargetScale {
    pub minimum: f32,
}

impl Dimensions {
    /// Return the shared utility theme for this already-scaled dimensions set.
    ///
    /// `Shell::dims` is still the one and only `ui_scale` multiplication point;
    /// this method only names and groups values. Calling it in a pane is cheap
    /// because the result is a small `Copy` value.
    pub fn theme(&self) -> UiTheme {
        UiTheme {
            space: SpaceScale {
                xs: self.spacing_xs,
                s: self.spacing_s,
                m: self.spacing_m,
                l: self.spacing_l,
            },
            text: TextScale {
                title: self.title_text,
                body: self.body_text,
                caption: self.caption_text,
                micro: self.micro_text,
            },
            size: SizeScale {
                row: self.row_height,
                transport: self.transport_band,
                panel_header: self.panel_header_height,
                pane_header: self.pane_header_height,
            },
            stroke: StrokeScale {
                hairline: self.border_width,
                focus: self.focus_indicator_width,
            },
            target: TargetScale {
                minimum: self.interactive_target_min,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Dimensions;

    #[test]
    fn shared_theme_is_a_named_view_of_the_dimensions_source() {
        let dims = Dimensions::default();
        let theme = dims.theme();

        assert_eq!(theme.space.m, dims.spacing_m);
        assert_eq!(theme.text.body, dims.body_text);
        assert_eq!(theme.size.row, dims.row_height);
        assert_eq!(theme.stroke.hairline, dims.border_width);
        assert_eq!(theme.target.minimum, dims.interactive_target_min);
    }
}
