//! Stage の視覚スキン比較器。
//!
//! この example は Stage の意味配置を複製しない。30個の `(Dimensions, Colors)`
//! だけを差し替え、実際の [`StageOverlay::view`] を毎回そのまま呼ぶ。
//! fixture は `tests/zoom_fence.rs` の CompSpec と ObservationCamera から転記している。

use std::fmt;

use iced::widget::{column, container, pick_list, row, text};
use iced::{Background, Border, Color, Element, Length, Task};

use motolii_core::{CompSpec, ResolvedCamera};
use motolii_engine::ObservationCamera;
use motolii_stage_pane::StageOverlay;
use motolii_tokens_rs::{Colors, Dimensions, theme_from_colors};

/// `tests/zoom_fence.rs:24` の fixture。
const COMP: CompSpec = CompSpec {
    width: 640,
    height: 360,
};

fn presets() -> Vec<Preset> {
    [Axis::A, Axis::B, Axis::C]
        .into_iter()
        .flat_map(|axis| (1..=10).map(move |n| Preset { axis, n }))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    A,
    B,
    C,
}

impl Axis {
    fn code(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::A => "A 面の分離",
            Self::B => "B 密度",
            Self::C => "C 文字と対比",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Preset {
    axis: Axis,
    n: u8,
}

impl Preset {
    fn claim(self) -> &'static str {
        let index = self.n.saturating_sub(1).min(9) as usize;
        match self.axis {
            Axis::A => [
                "地と窪みを最小差で分ける",
                "地の差を一段だけ見せる",
                "窪みの境界を静かに立てる",
                "hairlineを面差より先に読ませる",
                "パネル面を均一な作業地にする",
                "raisedを操作可能な面として浮かせる",
                "hoverの面差を明確にする",
                "強い縁と弱い縁を役割分担する",
                "フレーム枠を面から切り出す",
                "Ableton系の三段階を最大化する",
            ][index],
            Axis::B => [
                "詰めても操作対象を削らない",
                "短い視線で枠を追える密度にする",
                "見出しとStageの間を縮める",
                "compactな比較を優先する",
                "標準密度の基準点を置く",
                "標準から少し呼吸を足す",
                "余白でキャンバスを主役にする",
                "ゆったりした観察距離を作る",
                "説明行を読みやすく保つ",
                "プレゼンテーション用の最大密度差を出す",
            ][index],
            Axis::C => [
                "小さな文字でも役割を失わせない",
                "captionを控えめな補助線にする",
                "本文と補助文の差を広げる",
                "focusだけを最初に拾わせる",
                "accentを一点の合図に絞る",
                "dataとshapeの色差を保つ",
                "フレーム枠を本文より強くする",
                "選択色を明るい文字と同居させる",
                "強い対比で操作点を探しやすくする",
                "hero確認のための最大コントラストを置く",
            ][index],
        }
    }
}

impl fmt::Display for Preset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{:02}", self.axis.code(), self.n)
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    PresetSelected(Preset),
    Pane,
}

#[derive(Debug, Clone, Copy)]
struct App {
    preset: Preset,
}

impl Default for App {
    fn default() -> Self {
        Self {
            preset: Preset {
                axis: Axis::A,
                n: 1,
            },
        }
    }
}

impl App {
    fn colors(&self) -> Colors {
        skin(self.preset.axis, self.preset.n).1
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PresetSelected(preset) => self.preset = preset,
            Message::Pane => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let (dims, colors) = skin(self.preset.axis, self.preset.n);
        let theme = dims.theme();

        let picker = pick_list(Some(self.preset), presets(), |preset: &Preset| {
            preset.to_string()
        })
        .on_select(Message::PresetSelected)
        .padding([theme.space.xs, theme.space.s])
        .style(move |_theme, status| {
            let background = match status {
                pick_list::Status::Active => colors.surface_raised,
                pick_list::Status::Hovered => colors.surface_hover,
                pick_list::Status::Opened { .. } => colors.surface_hover,
                pick_list::Status::Disabled => colors.surface_panel,
            };

            pick_list::Style {
                text_color: colors.text_primary,
                placeholder_color: colors.text_secondary,
                handle_color: colors.text_secondary,
                background: Background::Color(background),
                border: Border {
                    color: colors.border_default,
                    width: theme.stroke.hairline,
                    ..Border::default()
                },
            }
        });

        let header = container(
            row![
                text("STAGE SKINS")
                    .size(theme.text.caption)
                    .color(colors.text_secondary),
                picker,
                text(format!(
                    "{} / {:02} — {}",
                    self.preset.axis.label(),
                    self.preset.n,
                    self.preset.claim()
                ))
                .size(theme.text.body)
                .color(colors.text_primary),
            ]
            .spacing(theme.space.m),
        )
        .padding([theme.space.s, theme.space.m])
        .width(Length::Fill)
        .height(Length::Fixed(theme.size.panel_header))
        .style(move |_theme| container::Style {
            background: Some(Background::Color(colors.surface_panel)),
            border: Border {
                color: colors.border_hairline_weak,
                width: theme.stroke.hairline,
                ..Border::default()
            },
            ..container::Style::default()
        });

        // Stage の canvas view は本物をそのまま呼ぶ。外側は面と縁だけを持つ
        // 最小 container で、pane の描画・入力面を覆わない。
        let stage = StageOverlay::new(
            COMP,
            ResolvedCamera::default(),
            Some(fixture_observation()),
            dims,
            colors,
        )
        .view()
        .map(|_| Message::Pane);

        let stage = container(stage)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(Background::Color(colors.surface_raised)),
                border: Border {
                    color: colors.border_strong,
                    width: theme.stroke.hairline,
                    ..Border::default()
                },
                ..container::Style::default()
            });

        container(column![header, stage].spacing(theme.space.s))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(Background::Color(colors.surface_app)),
                ..container::Style::default()
            })
            .into()
    }
}

/// `tests/zoom_fence.rs:54-57` の pan/zoom fixtureを、同テストの
/// `ObservationCamera::default()`から組み立てる。意味データは増やさない。
fn fixture_observation() -> ObservationCamera {
    let mut observation = ObservationCamera::default();
    observation.pan = [10.0, 20.0];
    observation.zoom = 1.0;
    observation
}

/// 30案の唯一のスキン生成口。各軸は既定 token から出発し、既存 field だけを
/// dimensionless な係数と既存色同士の blend で変える。
fn skin(axis: Axis, n: u8) -> (Dimensions, Colors) {
    let mut dims = Dimensions::default();
    let mut colors = Colors::default();
    let index = n.saturating_sub(1).min(9) as f32;
    let base_dims = dims;
    let base_colors = colors;

    match axis {
        Axis::A => {
            let t = index / 9.0;
            dims.border_width = base_dims.border_width * (0.60 + 0.08 * index);
            dims.focus_indicator_width = base_dims.focus_indicator_width * (0.75 + 0.10 * index);

            colors.surface_app = blend(
                base_colors.surface_app,
                base_colors.surface_panel,
                0.08 + 0.22 * t,
            );
            colors.surface_panel = blend(
                base_colors.surface_panel,
                base_colors.surface_raised,
                0.08 + 0.34 * t,
            );
            colors.surface_raised = blend(
                base_colors.surface_raised,
                base_colors.surface_hover,
                0.10 + 0.34 * t,
            );
            colors.surface_hover = blend(
                base_colors.surface_hover,
                base_colors.focus,
                0.04 + 0.16 * t,
            );
            colors.border_default = blend(
                base_colors.border_default,
                base_colors.border_hairline_weak,
                0.08 + 0.28 * t,
            );
            colors.border_strong = blend(
                base_colors.border_strong,
                base_colors.focus,
                0.08 + 0.30 * t,
            );
            colors.border_hairline_weak = blend(
                base_colors.border_hairline_weak,
                base_colors.border_strong,
                0.04 + 0.28 * t,
            );
        }
        Axis::B => {
            let density = 0.72 + 0.08 * index;
            dims.spacing_xs = base_dims.spacing_xs * density;
            dims.spacing_s = base_dims.spacing_s * density;
            dims.spacing_m = base_dims.spacing_m * density;
            dims.spacing_l = base_dims.spacing_l * density;
            dims.row_height = base_dims.row_height * density;
            dims.transport_band = base_dims.transport_band * density;
            dims.pane_header_height = base_dims.pane_header_height * density;
            dims.panel_header_height = base_dims.panel_header_height * density;
            dims.timeline_param_row_height = base_dims.timeline_param_row_height * density;
            dims.timeline_transport_height = base_dims.timeline_transport_height * density;
            dims.timeline_transport_button_width =
                base_dims.timeline_transport_button_width * density;
            dims.timeline_transport_gap = base_dims.timeline_transport_gap * density;
            // interactive_target_min は密度軸でも縮めない。
        }
        Axis::C => {
            let contrast = index / 9.0;
            let type_scale = 0.86 + 0.035 * index;
            dims.title_text = base_dims.title_text * type_scale;
            dims.body_text = base_dims.body_text * type_scale;
            dims.caption_text = base_dims.caption_text * type_scale;
            dims.micro_text = base_dims.micro_text * type_scale;

            colors.text_primary = blend(
                base_colors.text_primary,
                base_colors.focus,
                0.10 + 0.30 * contrast,
            );
            colors.text_secondary = blend(
                base_colors.text_secondary,
                base_colors.text_primary,
                0.08 + 0.35 * contrast,
            );
            colors.text_muted = blend(
                base_colors.text_muted,
                base_colors.text_secondary,
                0.05 + 0.25 * contrast,
            );
            colors.focus = blend(
                base_colors.focus,
                base_colors.action_active,
                0.05 + 0.35 * contrast,
            );
            colors.action_active = blend(
                base_colors.action_active,
                base_colors.focus,
                0.10 + 0.35 * contrast,
            );
            colors.data = blend(
                base_colors.data,
                base_colors.action_active,
                0.08 + 0.45 * contrast,
            );
            colors.shape = blend(
                base_colors.shape,
                base_colors.action_active,
                0.08 + 0.45 * contrast,
            );
            colors.way_timeline = blend(
                base_colors.way_timeline,
                base_colors.shape,
                0.08 + 0.45 * contrast,
            );
        }
    }

    (dims, colors)
}

/// Colors の既存ロール同士だけを混ぜる。新しい色値・新しい意味ロールは作らない。
fn blend(from: Color, to: Color, amount: f32) -> Color {
    Color {
        r: from.r + (to.r - from.r) * amount,
        g: from.g + (to.g - from.g) * amount,
        b: from.b + (to.b - from.b) * amount,
        a: from.a + (to.a - from.a) * amount,
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title("Motolii Stage Skins")
        .theme(|state: &App| theme_from_colors(&state.colors()))
        .run()
}
