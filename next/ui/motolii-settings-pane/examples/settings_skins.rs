use std::fmt;

use iced::widget::{column, pick_list, row, text};
use iced::{Element, Length, Task};
use motolii_store::{Composition, Fps};
use motolii_tokens_rs::{theme_from_colors, Colors, Dimensions};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Axis {
    A,
    B,
    C,
}

impl Axis {
    fn label(self) -> &'static str {
        match self {
            Self::A => "A 面の分離",
            Self::B => "B 密度",
            Self::C => "C 文字と対比",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Preset {
    axis: Axis,
    number: u8,
}

impl Preset {
    const fn new(axis: Axis, number: u8) -> Self {
        Self { axis, number }
    }

    fn claim(self) -> &'static str {
        let claims = match self.axis {
            Axis::A => &A_CLAIMS,
            Axis::B => &B_CLAIMS,
            Axis::C => &C_CLAIMS,
        };
        claims[self.number.clamp(1, 10) as usize - 1]
    }

    fn label(self) -> String {
        format!(
            "{}{:02}",
            match self.axis {
                Axis::A => "A",
                Axis::B => "B",
                Axis::C => "C",
            },
            self.number
        )
    }
}

impl fmt::Display for Preset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label())
    }
}

fn presets() -> Vec<Preset> {
    [Axis::A, Axis::B, Axis::C]
        .into_iter()
        .flat_map(|axis| (1..=10).map(move |number| Preset::new(axis, number)))
        .collect()
}

const A_CLAIMS: [&str; 10] = [
    "地と窪みを最小差で分ける",
    "hairlineを主役にせず面差で読ませる",
    "panelだけを一段持ち上げる",
    "raised面を操作可能性の合図にする",
    "hoverの段差を先に見せる",
    "通常罫線を弱く、強調線を残す",
    "選択面と通常面の差を広げる",
    "窪みを深くして設定のまとまりを作る",
    "hairlineと強罫線の役割を分離する",
    "Ableton的な二段面を最も明瞭にする",
];

const B_CLAIMS: [&str; 10] = [
    "最小の間隔で設定値を一覧する",
    "詰まりを保ちながら行の呼吸を残す",
    "数値編集を優先したコンパクトさにする",
    "短い確認と入力を同じ密度に揃える",
    "既定密度をそのまま再確認する",
    "行間を増やして値の境界を追いやすくする",
    "長いラベルでも圧迫しない",
    "section間に十分な呼吸を置く",
    "ゆったりした確認作業を支える",
    "設定を読む時間のための最大余白にする",
];

const C_CLAIMS: [&str; 10] = [
    "本文を抑え、見出しを先に拾わせる",
    "mutedを薄くして編集値を前に出す",
    "captionの視認性を上げる",
    "本文と補助文の階層差を作る",
    "既定の文字階層を再確認する",
    "focusを強めて入力位置を明示する",
    "action色を一点の導線に絞る",
    "dataとshapeの意味差を色で補助する",
    "timeline色を設定の進行感に寄せる",
    "文字とアクセントの対比を最大化する",
];

struct App {
    preset: Preset,
    composition: Composition,
}

impl Default for App {
    fn default() -> Self {
        Self {
            preset: Preset::new(Axis::A, 1),
            composition: fixture_comp(),
        }
    }
}

#[derive(Clone, Debug)]
enum Message {
    PresetSelected(Preset),
    Pane,
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::PresetSelected(preset) => app.preset = preset,
        Message::Pane => {}
    }
    Task::none()
}

fn view(app: &App) -> Element<'_, Message> {
    let (dims, colors) = skin(app.preset.axis, app.preset.number);
    let descriptor = format!(
        "{} / {:02} — {}",
        app.preset.axis.label(),
        app.preset.number,
        app.preset.claim()
    );
    let picker = pick_list(Some(app.preset), presets(), |preset: &Preset| {
        preset.label()
    })
    .on_select(Message::PresetSelected);
    let header = row![
        text("SETTINGS SKIN")
            .size(dims.title_text)
            .color(colors.text_primary),
        picker,
        text(descriptor)
            .size(dims.caption_text)
            .color(colors.text_secondary)
            .width(Length::Fill),
    ]
    .spacing(dims.spacing_s)
    .align_y(iced::alignment::Vertical::Center);

    let pane = motolii_settings_pane::view(Some(&app.composition), None, 1.0, None, dims, colors)
        .map(|_| Message::Pane);

    column![header, pane]
        .spacing(dims.spacing_m)
        .padding(dims.spacing_m)
        .into()
}

fn main() -> iced::Result {
    iced::application(App::default, update, view)
        .title("Motolii Settings skins")
        .theme(|app: &App| theme_from_colors(&app.colors()))
        .run()
}

impl App {
    fn colors(&self) -> Colors {
        skin(self.preset.axis, self.preset.number).1
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

fn blend(from: iced::Color, to: iced::Color, amount: f32) -> iced::Color {
    let t = amount.clamp(0.0, 1.0);
    iced::Color {
        r: from.r + (to.r - from.r) * t,
        g: from.g + (to.g - from.g) * t,
        b: from.b + (to.b - from.b) * t,
        a: from.a + (to.a - from.a) * t,
    }
}

fn axis_level(number: u8) -> f32 {
    f32::from(number.clamp(1, 10) - 1) / 9.0
}

fn skin(axis: Axis, number: u8) -> (Dimensions, Colors) {
    let (dims, mut colors) = match axis {
        Axis::A => skin_a(number),
        Axis::B => skin_b(number),
        Axis::C => skin_c(number),
    };
    colors.state_selected = blend(colors.surface_raised, colors.action_active, 0.18);
    colors.state_disabled = blend(colors.text_muted, colors.surface_panel, 0.40);
    (dims, colors)
}

fn skin_a(number: u8) -> (Dimensions, Colors) {
    let t = axis_level(number);
    let defaults = Dimensions::default();
    let base_colors = Colors::default();
    let mut dims = defaults;
    let mut colors = base_colors;

    dims.border_width = defaults.border_width * (0.75 + 1.0 * t);
    dims.focus_indicator_width = defaults.focus_indicator_width * (0.75 + 0.75 * t);
    colors.surface_app = blend(
        base_colors.surface_app,
        base_colors.surface_panel,
        0.05 + 0.10 * t,
    );
    colors.surface_panel = blend(
        base_colors.surface_app,
        base_colors.surface_raised,
        0.58 + 0.18 * t,
    );
    colors.surface_raised = blend(
        base_colors.surface_panel,
        base_colors.surface_hover,
        0.20 + 0.55 * t,
    );
    colors.surface_hover = blend(
        base_colors.surface_raised,
        base_colors.text_secondary,
        0.18 + 0.38 * t,
    );
    colors.border_default = blend(
        base_colors.border_default,
        base_colors.border_strong,
        0.15 + 0.65 * t,
    );
    colors.border_strong = blend(
        base_colors.border_default,
        base_colors.text_secondary,
        0.20 + 0.55 * t,
    );
    colors.border_hairline_weak = blend(
        base_colors.border_default,
        base_colors.surface_panel,
        0.20 + 0.50 * t,
    );
    (dims, colors)
}

fn skin_b(number: u8) -> (Dimensions, Colors) {
    let t = axis_level(number);
    let defaults = Dimensions::default();
    let colors = Colors::default();
    let density = 0.78 + 0.44 * t;
    let mut dims = defaults;

    dims.spacing_xs = defaults.spacing_xs * density;
    dims.spacing_s = defaults.spacing_s * density;
    dims.spacing_m = defaults.spacing_m * density;
    dims.spacing_l = defaults.spacing_l * density;
    dims.row_height = defaults.row_height * density;
    dims.transport_band = defaults.transport_band * density;
    dims.pane_header_height = defaults.pane_header_height * density;
    dims.panel_header_height = defaults.panel_header_height * density;
    dims.inspector_row_height = defaults.inspector_row_height * density;
    dims.inspector_section_header_height = defaults.inspector_section_header_height * density;
    // 操作対象の最小寸法は密度軸でも縮めない。
    dims.interactive_target_min = defaults.interactive_target_min;
    (dims, colors)
}

fn skin_c(number: u8) -> (Dimensions, Colors) {
    let t = axis_level(number);
    let defaults = Dimensions::default();
    let base_colors = Colors::default();
    let mut dims = defaults;
    let mut colors = base_colors;

    dims.title_text = defaults.title_text * (0.88 + 0.30 * t);
    dims.body_text = defaults.body_text * (0.88 + 0.24 * t);
    dims.caption_text = defaults.caption_text * (0.86 + 0.34 * t);
    dims.micro_text = defaults.micro_text * (0.84 + 0.38 * t);
    colors.text_primary = blend(
        base_colors.text_secondary,
        base_colors.focus,
        0.35 + 0.50 * t,
    );
    colors.text_secondary = blend(
        base_colors.text_muted,
        base_colors.text_primary,
        0.25 + 0.55 * t,
    );
    colors.text_muted = blend(
        base_colors.surface_panel,
        base_colors.text_muted,
        0.55 + 0.35 * t,
    );
    colors.focus = blend(base_colors.text_primary, base_colors.focus, 0.20 + 0.65 * t);
    colors.action_active = blend(
        base_colors.text_secondary,
        base_colors.action_active,
        0.20 + 0.65 * t,
    );
    colors.data = blend(
        base_colors.text_secondary,
        base_colors.data,
        0.20 + 0.65 * t,
    );
    colors.shape = blend(
        base_colors.text_secondary,
        base_colors.shape,
        0.20 + 0.65 * t,
    );
    colors.way_timeline = blend(
        base_colors.text_secondary,
        base_colors.way_timeline,
        0.20 + 0.65 * t,
    );
    (dims, colors)
}
