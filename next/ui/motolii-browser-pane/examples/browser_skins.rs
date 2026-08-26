use std::fmt;

use iced::widget::{column, pick_list, row, text};
use iced::{Element, Length, Task};
use motolii_browser_pane::model::AssetListItem;
use motolii_browser_pane::{pane_view, PaneState};
use motolii_store::{AssetId, AssetStatus};
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

type SkinColor = iced::Color;

fn blend(from: SkinColor, to: SkinColor, amount: f32) -> SkinColor {
    from.mix(to, amount)
}

fn skin(axis: Axis, number: u8) -> (Dimensions, Colors) {
    let mut dims = Dimensions::default();
    let base_dims = dims;
    let mut colors = Colors::default();
    let base_colors = colors;
    let index = usize::from(number.clamp(1, 10) - 1);

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
                surface_gap * 0.12,
            );
            colors.surface_panel =
                blend(colors.surface_app, base_colors.surface_panel, surface_gap);
            colors.surface_raised =
                blend(colors.surface_panel, base_colors.surface_raised, raised_gap);
            colors.surface_hover = blend(
                colors.surface_raised,
                base_colors.surface_hover,
                raised_gap * 0.75,
            );
            colors.border_default = blend(
                base_colors.border_default,
                base_colors.border_strong,
                surface_gap,
            );
            colors.border_strong = blend(base_colors.border_strong, base_colors.focus, raised_gap);
            colors.border_hairline_weak = blend(
                base_colors.border_hairline_weak,
                colors.border_default,
                raised_gap * 0.45,
            );
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

fn claim(preset: Preset) -> &'static str {
    let index = usize::from(preset.number.clamp(1, 10) - 1);
    match preset.axis {
        Axis::A => [
            "地と窪みをほぼ連続面で見せる",
            "弱いhairlineだけでカードを分ける",
            "窪みを先に読ませる",
            "面差を静かな選択合図にする",
            "標準の面差を基準にする",
            "選択面を一段だけ浮かせる",
            "境界線を操作可能性へ寄せる",
            "強い面差で大量素材を分ける",
            "hairlineと面差を同じ重さにする",
            "最大の分離で主役候補を囲う",
        ][index],
        Axis::B => [
            "最密度で一覧を止めない",
            "カード間隔を極小にする",
            "小さな素材群を優先する",
            "密度と可読性を均衡させる",
            "標準密度を基準にする",
            "選択の呼吸を少し増やす",
            "文字を詰めずに一覧を広げる",
            "比較単位をゆったり置く",
            "hero候補を大きく読む",
            "一枚の主役へ集中する",
        ][index],
        Axis::C => [
            "小さな文字でも階層を残す",
            "captionの差を抑える",
            "本文の対比を先にする",
            "mutedを静かに保つ",
            "標準の文字対比を基準にする",
            "activeだけを強くする",
            "データ色を主役へ寄せる",
            "形状色を主役へ寄せる",
            "timeline色を入口へ寄せる",
            "文字とaccentを同時に強くする",
        ][index],
    }
}

fn fixture_items() -> Vec<AssetListItem> {
    fn item(id: u64, name: &str, kind: &str) -> AssetListItem {
        AssetListItem {
            id: AssetId::from_raw(id),
            name: name.to_owned(),
            kind: kind.to_owned(),
            path: None,
            fingerprint: format!("sha256:{name}"),
            duration: None,
            status: AssetStatus::Unchecked,
        }
    }

    vec![
        item(0, "intro-clip", "video/mp4"),
        item(1, "logo-mark", "image/png"),
        item(2, "room-tone", "audio/wav"),
    ]
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
        let state = PaneState::new();
        let items = fixture_items();
        let pane = pane_view(&state, &items, None, dims, colors).map(|_| Message::Pane);
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
        .title("Motolii Browser skins")
        .theme(|app: &App| motolii_tokens_rs::theme_from_colors(&app.colors()))
        .run()
}
