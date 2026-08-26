use std::fmt;
use std::path::Path;

use iced::widget::{column, pick_list, row, text};
use iced::{Element, Length, Task};
use motolii_export_pane::{view, ExportQuality, ExportRange, ViewModel, WorkAreaFrames};
use motolii_store::{Composition, Fps};
use motolii_tokens_rs::{Colors, Dimensions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    A,
    B,
    C,
}

impl fmt::Display for Axis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Preset {
    axis: Axis,
    number: u8,
}

impl Preset {
    const fn new(axis: Axis, number: u8) -> Self {
        Self { axis, number }
    }
}

impl fmt::Display for Preset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{:02}", self.axis, self.number)
    }
}

fn presets() -> Vec<Preset> {
    [Axis::A, Axis::B, Axis::C]
        .into_iter()
        .flat_map(|axis| (1..=10).map(move |number| Preset::new(axis, number)))
        .collect()
}

fn blend(from: iced::Color, to: iced::Color, amount: f32) -> iced::Color {
    iced::Color {
        r: from.r + (to.r - from.r) * amount,
        g: from.g + (to.g - from.g) * amount,
        b: from.b + (to.b - from.b) * amount,
        a: from.a + (to.a - from.a) * amount,
    }
}

fn skin(axis: Axis, number: u8) -> (Dimensions, Colors) {
    let mut dims = Dimensions::default();
    let base_dims = dims;
    let mut colors = Colors::default();
    let base_colors = colors;
    let index = usize::from(number.saturating_sub(1).min(9));
    match axis {
        Axis::A => {
            let surface_gap = [0.08, 0.14, 0.20, 0.26, 0.32, 0.38, 0.44, 0.50, 0.56, 0.62][index];
            let raised_gap = [0.08, 0.12, 0.16, 0.20, 0.24, 0.28, 0.32, 0.36, 0.40, 0.44][index];
            let line_weight = [0.70, 0.82, 0.94, 1.06, 1.18, 1.30, 1.44, 1.60, 1.78, 2.00][index];
            dims.border_width = base_dims.border_width * line_weight;
            dims.focus_indicator_width = base_dims.focus_indicator_width * line_weight;
            colors.surface_app = blend(
                base_colors.surface_app,
                base_colors.surface_panel,
                surface_gap * 0.35,
            );
            colors.surface_panel = blend(
                base_colors.surface_app,
                base_colors.surface_raised,
                surface_gap,
            );
            colors.surface_raised = blend(colors.surface_panel, base_colors.focus, raised_gap);
            colors.surface_hover =
                blend(colors.surface_raised, base_colors.focus, raised_gap * 0.75);
            colors.border_default = blend(
                base_colors.surface_app,
                base_colors.border_strong,
                surface_gap,
            );
            colors.border_strong = blend(colors.border_default, base_colors.focus, raised_gap);
            colors.border_hairline_weak =
                blend(colors.border_default, base_colors.focus, raised_gap * 0.45);
        }
        Axis::B => {
            let density = [0.70, 0.78, 0.86, 0.94, 1.00, 1.08, 1.18, 1.30, 1.44, 1.60][index];
            dims.spacing_xs = base_dims.spacing_xs * density;
            dims.spacing_s = base_dims.spacing_s * density;
            dims.spacing_m = base_dims.spacing_m * density;
            dims.spacing_l = base_dims.spacing_l * density;
            dims.row_height = base_dims.row_height * density;
            dims.transport_band = base_dims.transport_band * density;
            dims.panel_header_height = base_dims.panel_header_height * density;
            dims.pane_header_height = base_dims.pane_header_height * density;
            dims.inspector_row_height = base_dims.inspector_row_height * density;
            dims.timeline_param_row_height = base_dims.timeline_param_row_height * density;
            dims.timeline_transport_height = base_dims.timeline_transport_height * density;
            dims.timeline_transport_button_width =
                base_dims.timeline_transport_button_width * density;
            dims.timeline_transport_gap = base_dims.timeline_transport_gap * density;
        }
        Axis::C => {
            let type_scale = [0.78, 0.84, 0.90, 0.96, 1.00, 1.06, 1.14, 1.24, 1.36, 1.50][index];
            let contrast = [0.08, 0.16, 0.24, 0.32, 0.40, 0.48, 0.56, 0.64, 0.72, 0.80][index];
            let accent = [
                base_colors.action_active,
                base_colors.data,
                base_colors.shape,
                base_colors.way_timeline,
                base_colors.status_ok,
                base_colors.status_warning,
                base_colors.action_active,
                base_colors.data,
                base_colors.shape,
                base_colors.way_timeline,
            ][index];
            dims.title_text = base_dims.title_text * type_scale;
            dims.body_text = base_dims.body_text * type_scale;
            dims.caption_text = base_dims.caption_text * type_scale;
            dims.micro_text = base_dims.micro_text * type_scale;
            colors.text_primary = blend(base_colors.text_primary, base_colors.focus, contrast);
            colors.text_secondary = blend(
                base_colors.text_secondary,
                colors.text_primary,
                contrast * 0.75,
            );
            colors.text_muted = blend(
                base_colors.text_muted,
                colors.text_secondary,
                contrast * 0.55,
            );
            colors.focus = blend(base_colors.focus, accent, contrast * 0.35);
            colors.action_active = blend(base_colors.action_active, accent, 0.72);
            colors.data = blend(base_colors.data, accent, 0.62);
            colors.shape = blend(base_colors.shape, accent, 0.62);
            colors.way_timeline = blend(base_colors.way_timeline, accent, 0.62);
        }
    }
    colors.state_selected = blend(colors.surface_raised, colors.action_active, 0.18);
    colors.state_disabled = blend(colors.text_muted, colors.surface_panel, 0.40);
    (dims, colors)
}

fn claim(preset: Preset) -> String {
    match preset.axis {
        Axis::A => format!("面の分離: 地/窪み/hairlineを{}段階にする", preset.number),
        Axis::B => format!("密度: spacingとrowを{}段階にする", preset.number),
        Axis::C => format!("文字と対比: text段とaccentを{}段階にする", preset.number),
    }
}

fn fixture_comp() -> Composition {
    Composition {
        width: 1920,
        height: 1080,
        fps: Fps::try_new(30, 1).expect("30fps"),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }
}

fn wired_model<'a>(composition: &'a Composition, out_path: Option<&'a Path>) -> ViewModel<'a> {
    ViewModel {
        composition: Some(composition),
        out_path,
        quality: ExportQuality::Normal,
        range: ExportRange::Whole,
        work_area: Some(WorkAreaFrames {
            start: 100,
            end: 200,
        }),
        progress: None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Message {
    Select(Preset),
    Pane,
}

#[derive(Debug, Clone, Copy)]
struct App {
    selected: Preset,
}

impl Default for App {
    fn default() -> Self {
        Self {
            selected: Preset::new(Axis::A, 1),
        }
    }
}

impl App {
    fn colors(&self) -> Colors {
        skin(self.selected.axis, self.selected.number).1
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        if let Message::Select(preset) = message {
            self.selected = preset;
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let (dims, colors) = skin(self.selected.axis, self.selected.number);
        let composition = fixture_comp();
        let model = wired_model(&composition, Some(Path::new("/tmp/out.mp4")));
        let pane = view(model, dims, colors).map(|_| Message::Pane);
        column![
            row![
                text(format!(
                    "{} · {} · {}",
                    self.selected,
                    self.selected.axis,
                    claim(self.selected)
                ))
                .size(dims.theme().text.body)
                .color(colors.text_primary),
                pick_list(Some(self.selected), presets(), |preset: &Preset| preset
                    .to_string(),)
                .on_select(Message::Select)
                .text_size(dims.theme().text.body)
                .width(Length::Shrink),
            ]
            .spacing(dims.theme().space.m)
            .align_y(iced::alignment::Vertical::Center),
            pane,
        ]
        .spacing(dims.theme().space.s)
        .padding(dims.theme().space.m)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title("Motolii Export skins")
        .theme(|app: &App| motolii_tokens_rs::theme_from_colors(&app.colors()))
        .run()
}
