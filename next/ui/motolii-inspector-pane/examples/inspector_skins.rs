//! Inspector pane visual skin gallery.
//!
//! The pane tree and its meaning live in `motolii_inspector_pane::view`. This
//! example only chooses a `(Dimensions, Colors)` pair and puts a small picker
//! above the real pane, so every candidate exercises the same production view.

use iced::widget::{column, pick_list, row, scrollable, text};
use iced::{application, Element, Length, Size};

use motolii_core::Fps;
use motolii_inspector_pane::project;
use motolii_shell_state::Session;
use motolii_store::{Composition, Document, Intent, LayerId, LayerMeta, LayerSource, LayerTiming};
use motolii_tokens_rs::{theme_from_colors, Colors, Dimensions};

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Preset {
    axis: Axis,
    number: u8,
}

impl Preset {
    fn label(self) -> String {
        format!("Axis {} · {:02}", self.axis.code(), self.number)
    }

    fn summary(self) -> String {
        format!(
            "Axis {} · {:02} · {}",
            self.axis.code(),
            self.number,
            claim(self.axis, self.number)
        )
    }
}

#[derive(Debug, Clone)]
enum Message {
    PresetChanged(Preset),
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
                number: 1,
            },
        }
    }
}

impl App {
    fn skin(&self) -> (Dimensions, Colors) {
        skin(self.preset.axis, self.preset.number)
    }

    fn colors(&self) -> Colors {
        self.skin().1
    }
}

fn main() -> iced::Result {
    application(App::default, update, view)
        .title("Motolii Inspector skins")
        .theme(|state: &App| theme_from_colors(&state.colors()))
        .window(iced::window::Settings {
            size: Size::new(
                Dimensions::default().inspector_panel_width * 1.55,
                Dimensions::default().inspector_panel_width * 1.75,
            ),
            ..Default::default()
        })
        .centered()
        .run()
}

fn update(app: &mut App, message: Message) {
    match message {
        Message::PresetChanged(preset) => app.preset = preset,
        Message::Pane => {}
    }
}

fn view(app: &App) -> Element<'_, Message> {
    let (dims, colors) = app.skin();
    let selection = fixture_selection();
    let pane = motolii_inspector_pane::view(Some(&selection), None, None, dims, colors)
        .map(|_| Message::Pane);

    let picker = pick_list(Some(app.preset), presets(), |preset: &Preset| {
        preset.label()
    })
    .on_select(Message::PresetChanged)
    .width(Length::FillPortion(1))
    .text_size(dims.theme().text.body);

    let controls = row![
        text("Inspector skin gallery").size(dims.theme().text.title),
        picker,
    ]
    .spacing(dims.theme().space.m)
    .align_y(iced::Alignment::Center);

    let claim_line = text(app.preset.summary()).size(dims.theme().text.caption);

    column![controls, claim_line, scrollable(pane)]
        .spacing(dims.theme().space.s)
        .padding(dims.theme().space.m)
        .into()
}

fn presets() -> [Preset; 30] {
    let mut values = [Preset {
        axis: Axis::A,
        number: 1,
    }; 30];

    for number in 1..=10 {
        values[usize::from(number - 1)] = Preset {
            axis: Axis::A,
            number,
        };
        values[usize::from(number + 9)] = Preset {
            axis: Axis::B,
            number,
        };
        values[usize::from(number + 19)] = Preset {
            axis: Axis::C,
            number,
        };
    }

    values
}

fn claim(axis: Axis, number: u8) -> &'static str {
    const A: [&str; 10] = [
        "面の境界を最小のhairlineで読む",
        "地と窪みを近接させて意味を保つ",
        "控えめな段差で値行を連続させる",
        "標準的な二段面をInspectorへ写す",
        "窪みを一段強めてsectionを読む",
        "hover面を明確にして入口を見つける",
        "borderの強弱でsectionの骨格を出す",
        "面の差を強めても影に頼らない",
        "強いhairlineで編集境界を固定する",
        "最大の面分離で状態を即時に読む",
    ];
    const B: [&str; 10] = [
        "最短の行間で値を一覧する",
        "詰めた余白でTransformを連続表示する",
        "小さな余白で編集密度を上げる",
        "標準密度で行間と踏面を両立する",
        "中間密度で値のまとまりを作る",
        "ゆとりある行間でsectionを分ける",
        "余白を増やして視線の休止点を作る",
        "大きめの行高で編集対象を拾う",
        "ゆったりした密度で誤操作を減らす",
        "最大の余白で一行ずつ確実に読む",
    ];
    const C: [&str; 10] = [
        "低い文字段差で静かな情報面を作る",
        "本文とcaptionの差を小さく保つ",
        "secondaryを近づけて補助情報を残す",
        "標準の文字段差とaccentを保つ",
        "focusだけを一段明るくして編集点を示す",
        "action accentを値の近くへ寄せる",
        "data色で動く値の存在を読む",
        "shape色で型の違いを静かに分ける",
        "複数の意味色を強めても面は増やさない",
        "最大の文字対比で値の読解を主役にする",
    ];

    let index = usize::from(number.saturating_sub(1).min(9));
    match axis {
        Axis::A => A[index],
        Axis::B => B[index],
        Axis::C => C[index],
    }
}

/// Build all 30 skins from the token defaults. The arrays are dimensionless
/// axis positions; no new color or pixel vocabulary is introduced here.
fn skin(axis: Axis, n: u8) -> (Dimensions, Colors) {
    let mut dims = Dimensions::default();
    let base = Colors::default();
    let mut colors = base;
    let index = usize::from(n.saturating_sub(1).min(9));

    match axis {
        Axis::A => {
            const APP_WASH: [f32; 10] =
                [0.00, 0.03, 0.06, 0.09, 0.12, 0.15, 0.18, 0.22, 0.26, 0.30];
            const PANEL_STEP: [f32; 10] =
                [0.35, 0.42, 0.49, 0.56, 0.63, 0.70, 0.77, 0.84, 0.92, 1.00];
            const RAISED_STEP: [f32; 10] =
                [0.25, 0.33, 0.41, 0.49, 0.57, 0.65, 0.73, 0.81, 0.90, 1.00];
            const HOVER_STEP: [f32; 10] =
                [0.20, 0.28, 0.36, 0.44, 0.52, 0.60, 0.68, 0.77, 0.86, 0.96];
            const BORDER_STEP: [f32; 10] =
                [0.20, 0.29, 0.38, 0.47, 0.56, 0.65, 0.74, 0.83, 0.92, 1.00];
            const STRONG_STEP: [f32; 10] =
                [0.18, 0.27, 0.36, 0.45, 0.54, 0.63, 0.72, 0.81, 0.90, 1.00];
            const HAIRLINE_STEP: [f32; 10] =
                [0.20, 0.29, 0.38, 0.47, 0.56, 0.65, 0.74, 0.83, 0.92, 1.00];
            const BORDER_WIDTH: [f32; 10] =
                [0.70, 0.80, 0.90, 1.00, 1.10, 1.20, 1.30, 1.40, 1.50, 1.60];
            const FOCUS_WIDTH: [f32; 10] =
                [0.80, 0.90, 1.00, 1.10, 1.20, 1.30, 1.40, 1.50, 1.60, 1.70];

            colors.surface_app = mix(base.surface_app, base.surface_panel, APP_WASH[index]);
            colors.surface_panel = mix(base.surface_app, base.surface_panel, PANEL_STEP[index]);
            colors.surface_raised =
                mix(base.surface_panel, base.surface_raised, RAISED_STEP[index]);
            colors.surface_hover = mix(base.surface_raised, base.surface_hover, HOVER_STEP[index]);
            colors.border_default = mix(base.surface_app, base.border_default, BORDER_STEP[index]);
            colors.border_strong = mix(base.border_default, base.border_strong, STRONG_STEP[index]);
            colors.border_hairline_weak = mix(
                base.border_default,
                base.border_hairline_weak,
                HAIRLINE_STEP[index],
            );
            dims.border_width *= BORDER_WIDTH[index];
            dims.focus_indicator_width *= FOCUS_WIDTH[index];
        }
        Axis::B => {
            const SCALE: [f32; 10] = [0.68, 0.76, 0.84, 0.92, 1.00, 1.08, 1.16, 1.24, 1.34, 1.46];
            let scale = SCALE[index];
            dims.spacing_xs *= scale;
            dims.spacing_s *= scale;
            dims.spacing_m *= scale;
            dims.spacing_l *= scale;
            dims.row_height *= scale;
            dims.transport_band *= scale;
            dims.pane_header_height *= scale;
            dims.panel_header_height *= scale;
            dims.inspector_row_height *= scale;
            dims.inspector_section_header_height *= scale;
        }
        Axis::C => {
            const TEXT_SCALE: [f32; 10] =
                [0.84, 0.90, 0.95, 1.00, 1.05, 1.10, 1.16, 1.23, 1.31, 1.40];
            const CONTRAST: [f32; 10] =
                [0.18, 0.27, 0.36, 0.45, 0.54, 0.63, 0.72, 0.81, 0.90, 1.00];
            let text_scale = TEXT_SCALE[index];
            let contrast = CONTRAST[index];

            dims.title_text *= text_scale;
            dims.body_text *= text_scale;
            dims.caption_text *= text_scale;
            dims.micro_text *= text_scale;
            colors.text_primary = mix(base.text_secondary, base.focus, contrast);
            colors.text_secondary = mix(base.text_muted, base.text_primary, contrast * 0.90);
            colors.text_muted = mix(base.surface_panel, base.text_muted, 0.42 + contrast * 0.52);
            colors.focus = mix(base.text_secondary, base.focus, 0.35 + contrast * 0.60);
            colors.action_active = mix(
                base.text_secondary,
                base.action_active,
                0.28 + contrast * 0.68,
            );
            colors.data = mix(base.text_secondary, base.data, 0.24 + contrast * 0.70);
            colors.shape = mix(base.text_secondary, base.shape, 0.24 + contrast * 0.70);
            colors.way_timeline = mix(
                base.text_secondary,
                base.way_timeline,
                0.24 + contrast * 0.70,
            );
        }
    }

    (dims, colors)
}

fn mix(from: iced::Color, to: iced::Color, amount: f32) -> iced::Color {
    from.mix(to, amount)
}

/// Existing test fixture: `tests/effects_section.rs::{doc_with_layer,
/// session_selecting}`. It deliberately uses the production projection path;
/// this example adds no synthetic inspector meaning.
fn fixture_selection() -> motolii_inspector_pane::SelectionProjection {
    let mut doc = Document::new();
    let layer = LayerId(1);
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).expect("30fps は正値"),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp を置けるはず");
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Solid {
                    rgba: [255, 0, 0, 255],
                    width: 64,
                    height: 64,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .expect("layer を置けるはず");

    let session = Session {
        selection: Some(layer),
        ..Session::default()
    };
    project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず")
}
