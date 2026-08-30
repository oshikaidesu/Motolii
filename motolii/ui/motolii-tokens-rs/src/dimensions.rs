
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
#[serde(default)]
pub struct ComponentValues {
    pub browser: BrowserValues,
    pub settings: SettingsValues,
    pub stage: StageValues,
    pub timeline: TimelineValues,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
#[serde(default)]
pub struct BrowserValues {
    pub grid_columns: usize,
    pub card_width_row_height_ratio: f32,
    pub thumb_aspect_w: f32,
    pub thumb_aspect_h: f32,
    pub filter_chip_corner_radius_row_height_ratio: f32,
    pub panel_height_row_height_ratio: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
#[serde(default)]
pub struct SettingsValues {
    pub checkerboard_tile_target_pt: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
#[serde(default)]
pub struct StageValues {
    pub grid_divisions: u32,
    pub action_safe_inset: f32,
    pub title_safe_inset: f32,
    pub grid_ink_factor: f32,
    pub fit_to_selection_margin: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
#[serde(default)]
pub struct TimelineValues {
    pub graph_plot_padding: f32,
    pub graph_handle_hit: f32,
    pub graph_grid_line_width: f32,
    pub graph_curve_line_width: f32,
    pub graph_control_line_width: f32,
    pub graph_handle_radius: f32,
    pub key_diamond_size: f32,
    pub key_hit: f32,
    pub target_cell_ratio: f32,
    pub viewport_default_size: f32,
}

impl Default for ComponentValues {
    fn default() -> Self {
        Self {
            browser: BrowserValues::default(),
            settings: SettingsValues::default(),
            stage: StageValues::default(),
            timeline: TimelineValues::default(),
        }
    }
}

impl Default for BrowserValues {
    fn default() -> Self {
        Self {
            grid_columns: 2,
            card_width_row_height_ratio: 6.0,
            thumb_aspect_w: 16.0,
            thumb_aspect_h: 9.0,
            filter_chip_corner_radius_row_height_ratio: 0.4,
            panel_height_row_height_ratio: 14.0,
        }
    }
}

impl Default for SettingsValues {
    fn default() -> Self {
        Self {
            checkerboard_tile_target_pt: 8.0,
        }
    }
}

impl Default for StageValues {
    fn default() -> Self {
        Self {
            grid_divisions: 9,
            action_safe_inset: 0.05,
            title_safe_inset: 0.10,
            grid_ink_factor: 0.10 / 0.22,
            fit_to_selection_margin: 0.9,
        }
    }
}

impl Default for TimelineValues {
    fn default() -> Self {
        Self {
            graph_plot_padding: 16.0,
            graph_handle_hit: 12.0,
            graph_grid_line_width: 1.0,
            graph_curve_line_width: 2.0,
            graph_control_line_width: 1.0,
            graph_handle_radius: 5.0,
            key_diamond_size: 8.0,
            key_hit: 12.0,
            target_cell_ratio: 0.52,
            viewport_default_size: 100.0,
        }
    }
}

impl ComponentValues {
    pub fn scaled(self, ui_scale: f32) -> Self {
        let s = ui_scale;
        Self {
            browser: self.browser,
            settings: SettingsValues {
                checkerboard_tile_target_pt: self.settings.checkerboard_tile_target_pt * s,
            },
            stage: self.stage,
            timeline: TimelineValues {
                graph_plot_padding: self.timeline.graph_plot_padding * s,
                graph_handle_hit: self.timeline.graph_handle_hit * s,
                graph_grid_line_width: self.timeline.graph_grid_line_width * s,
                graph_curve_line_width: self.timeline.graph_curve_line_width * s,
                graph_control_line_width: self.timeline.graph_control_line_width * s,
                graph_handle_radius: self.timeline.graph_handle_radius * s,
                key_diamond_size: self.timeline.key_diamond_size * s,
                key_hit: self.timeline.key_hit * s,
                ..self.timeline
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
pub struct Dimensions {
    #[serde(default)]
    pub components: ComponentValues,
    pub row_height: f32,
    pub transport_band: f32,
    pub title_text: f32,
    pub body_text: f32,
    pub caption_text: f32,
    pub micro_text: f32,
    pub spacing_xs: f32,
    pub spacing_s: f32,
    pub spacing_m: f32,
    pub spacing_l: f32,
    pub border_width: f32,
    #[serde(default = "default_interactive_target_min")]
    pub interactive_target_min: f32,
    #[serde(default = "default_focus_indicator_width")]
    pub focus_indicator_width: f32,
    pub panel_header_height: f32,
    pub inspector_panel_width: f32,
    pub inspector_row_height: f32,
    pub inspector_section_header_height: f32,
    pub inspector_value_width: f32,
    #[serde(default = "default_inspector_glyph_width")]
    pub inspector_glyph_width: f32,
    #[serde(default = "default_pane_header_height")]
    pub pane_header_height: f32,
    #[serde(default = "default_timeline_lane_bar_width")]
    pub timeline_lane_bar_width: f32,
    #[serde(default = "default_timeline_param_row_height")]
    pub timeline_param_row_height: f32,
    #[serde(default = "default_timeline_transport_height")]
    pub timeline_transport_height: f32,
    #[serde(default = "default_timeline_transport_button_width")]
    pub timeline_transport_button_width: f32,
    #[serde(default = "default_timeline_transport_gap")]
    pub timeline_transport_gap: f32,
    #[serde(default = "default_graph_editor_plot_height")]
    pub graph_editor_plot_height: f32,
    #[serde(default = "default_graph_control_label_width")]
    pub graph_control_label_width: f32,
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
    #[serde(default = "default_browser_tab_bar_height")]
    pub browser_tab_bar_height: f32,
    #[serde(default = "default_browser_tab_underline")]
    pub browser_tab_underline: f32,
    #[serde(default = "default_browser_list_thumb_width")]
    pub browser_list_thumb_width: f32,
    #[serde(default = "default_menubar_menu_width")]
    pub menubar_menu_width: f32,
    #[serde(default = "default_menubar_corner_radius")]
    pub menubar_corner_radius: f32,
    #[serde(default = "default_gizmo_handle_size")]
    pub gizmo_handle_size: f32,
    #[serde(default = "default_gizmo_hit_radius")]
    pub gizmo_hit_radius: f32,
    #[serde(default = "default_gizmo_rotate_offset")]
    pub gizmo_rotate_offset: f32,
    #[serde(default = "default_gizmo_anchor_radius")]
    pub gizmo_anchor_radius: f32,
}

fn default_inspector_glyph_width() -> f32 {
    26.0
}

fn default_interactive_target_min() -> f32 {
    24.0
}

fn default_focus_indicator_width() -> f32 {
    2.0
}

fn default_pane_header_height() -> f32 {
    18.0
}

fn default_timeline_lane_bar_width() -> f32 {
    150.0
}

fn default_timeline_param_row_height() -> f32 {
    16.67
}

fn default_timeline_transport_height() -> f32 {
    30.0
}

fn default_timeline_transport_button_width() -> f32 {
    30.0
}

fn default_timeline_transport_gap() -> f32 {
    2.0
}

fn default_graph_editor_plot_height() -> f32 {
    180.0
}

fn default_graph_control_label_width() -> f32 {
    20.0
}

fn default_ui_scale() -> f32 {
    1.18
}

fn default_browser_tab_bar_height() -> f32 {
    26.0
}

fn default_browser_tab_underline() -> f32 {
    2.0
}

fn default_browser_list_thumb_width() -> f32 {
    46.0
}

fn default_menubar_menu_width() -> f32 {
    192.0
}

fn default_menubar_corner_radius() -> f32 {
    4.0
}

fn default_gizmo_handle_size() -> f32 {
    8.0
}

fn default_gizmo_hit_radius() -> f32 {
    8.0
}

fn default_gizmo_rotate_offset() -> f32 {
    16.0
}

fn default_gizmo_anchor_radius() -> f32 {
    4.0
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            components: ComponentValues::default(),
            row_height: 20.0,
            transport_band: 30.0,
            title_text: 12.0,
            body_text: 11.0,
            caption_text: 9.0,
            micro_text: 8.0,
            spacing_xs: 2.0,
            spacing_s: 4.0,
            spacing_m: 8.0,
            spacing_l: 12.0,
            border_width: 1.0,
            interactive_target_min: 24.0,
            focus_indicator_width: 2.0,
            panel_header_height: 29.0,
            inspector_panel_width: 496.0,
            inspector_row_height: 25.0,
            inspector_section_header_height: 26.0,
            inspector_value_width: 64.0,
            inspector_glyph_width: 26.0,
            pane_header_height: 18.0,
            timeline_lane_bar_width: 150.0,
            timeline_param_row_height: 16.67,
            timeline_transport_height: 30.0,
            timeline_transport_button_width: 30.0,
            timeline_transport_gap: 2.0,
            graph_editor_plot_height: 180.0,
            graph_control_label_width: 20.0,
            ui_scale: 1.18,
            browser_tab_bar_height: 26.0,
            browser_tab_underline: 2.0,
            browser_list_thumb_width: 46.0,
            menubar_menu_width: 192.0,
            menubar_corner_radius: 4.0,
            gizmo_handle_size: 8.0,
            gizmo_hit_radius: 8.0,
            gizmo_rotate_offset: 16.0,
            gizmo_anchor_radius: 4.0,
        }
    }
}

impl Dimensions {
    pub fn parse(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|error| error.to_string())
    }

    pub fn debug_source_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tokens/dimensions.json")
    }

    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
        Self::parse(&text)
    }

    pub fn scaled(&self, ui_scale: f32) -> Self {
        let s = ui_scale;
        Self {
            components: self.components.scaled(s),
            row_height: self.row_height * s,
            transport_band: self.transport_band * s,
            title_text: self.title_text * s,
            body_text: self.body_text * s,
            caption_text: self.caption_text * s,
            micro_text: self.micro_text * s,
            spacing_xs: self.spacing_xs * s,
            spacing_s: self.spacing_s * s,
            spacing_m: self.spacing_m * s,
            spacing_l: self.spacing_l * s,
            border_width: self.border_width.max(1.0),
            interactive_target_min: self.interactive_target_min * s,
            focus_indicator_width: self.focus_indicator_width * s,
            panel_header_height: self.panel_header_height * s,
            inspector_panel_width: self.inspector_panel_width * s,
            inspector_row_height: self.inspector_row_height * s,
            inspector_section_header_height: self.inspector_section_header_height * s,
            inspector_value_width: self.inspector_value_width * s,
            inspector_glyph_width: self.inspector_glyph_width * s,
            pane_header_height: self.pane_header_height * s,
            timeline_lane_bar_width: self.timeline_lane_bar_width * s,
            timeline_param_row_height: self.timeline_param_row_height * s,
            browser_tab_bar_height: self.browser_tab_bar_height * s,
            browser_tab_underline: self.browser_tab_underline * s,
            browser_list_thumb_width: self.browser_list_thumb_width * s,
            timeline_transport_height: self.timeline_transport_height * s,
            timeline_transport_button_width: self.timeline_transport_button_width * s,
            timeline_transport_gap: self.timeline_transport_gap * s,
            graph_editor_plot_height: self.graph_editor_plot_height * s,
            graph_control_label_width: self.graph_control_label_width * s,
            menubar_menu_width: self.menubar_menu_width * s,
            menubar_corner_radius: self.menubar_corner_radius * s,
            gizmo_handle_size: self.gizmo_handle_size * s,
            gizmo_hit_radius: self.gizmo_hit_radius * s,
            gizmo_rotate_offset: self.gizmo_rotate_offset * s,
            gizmo_anchor_radius: self.gizmo_anchor_radius * s,
            ui_scale: self.ui_scale,
        }
    }
}

#[cfg(test)]
mod ui_scale_tests {
    use super::Dimensions;

    #[test]
    fn scaling_by_one_is_the_identity() {
        let dims = Dimensions::default();
        let scaled = dims.scaled(1.0);
        assert_eq!(scaled, dims, "1.0倍で寸法が変わってしまっている");
    }

    #[test]
    fn scaling_by_one_point_five_multiplies_every_dimension_but_the_border() {
        let dims = Dimensions::default();
        let scaled = dims.scaled(1.5);

        assert_eq!(scaled.row_height, dims.row_height * 1.5);
        assert_eq!(scaled.transport_band, dims.transport_band * 1.5);
        assert_eq!(scaled.title_text, dims.title_text * 1.5);
        assert_eq!(scaled.body_text, dims.body_text * 1.5);
        assert_eq!(scaled.caption_text, dims.caption_text * 1.5);
        assert_eq!(scaled.micro_text, dims.micro_text * 1.5);
        assert_eq!(scaled.spacing_xs, dims.spacing_xs * 1.5);
        assert_eq!(scaled.spacing_s, dims.spacing_s * 1.5);
        assert_eq!(scaled.spacing_m, dims.spacing_m * 1.5);
        assert_eq!(scaled.spacing_l, dims.spacing_l * 1.5);
        assert_eq!(scaled.panel_header_height, dims.panel_header_height * 1.5);
        assert_eq!(
            scaled.inspector_panel_width,
            dims.inspector_panel_width * 1.5
        );
        assert_eq!(scaled.inspector_row_height, dims.inspector_row_height * 1.5);
        assert_eq!(
            scaled.inspector_section_header_height,
            dims.inspector_section_header_height * 1.5
        );
        assert_eq!(
            scaled.inspector_value_width,
            dims.inspector_value_width * 1.5
        );
        assert_eq!(
            scaled.inspector_glyph_width,
            dims.inspector_glyph_width * 1.5
        );
        assert_eq!(
            scaled.graph_editor_plot_height,
            dims.graph_editor_plot_height * 1.5
        );
        assert_eq!(
            scaled.graph_control_label_width,
            dims.graph_control_label_width * 1.5
        );
    }

    #[test]
    fn the_border_width_never_scales_past_its_one_pixel_floor() {
        let dims = Dimensions::default();
        assert_eq!(dims.scaled(1.5).border_width, 1.0);
        assert_eq!(dims.scaled(1.0).border_width, 1.0);
        let thin = Dimensions {
            border_width: 0.4,
            ..dims
        };
        assert_eq!(thin.scaled(1.0).border_width, 1.0);
        assert_eq!(thin.scaled(1.5).border_width, 1.0);
    }

    #[test]
    fn the_canonical_type_band_matches_the_mock() {
        let dims = Dimensions::default();
        assert_eq!(dims.title_text, 12.0);
        assert_eq!(dims.body_text, 11.0);
        assert_eq!(dims.caption_text, 9.0);
        assert_eq!(dims.micro_text, 8.0);
    }

    #[test]
    fn the_inspector_section_height_matches_the_mock() {
        assert_eq!(Dimensions::default().inspector_section_header_height, 26.0);
    }
}
