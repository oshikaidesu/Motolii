//! Timeline の視覚スキン30案。
//!
//! この example は Timeline の widget 木を持たない。既存の fixture を
//! `StoreView`/`Session` へ投影し、本物の `TimelinePane::view_with_transport`
//! に `Dimensions` と `Colors` だけを渡す。

use iced::widget::{column, pick_list, row, text};
use iced::{Element, Length, Task};
use motolii_shell_state::Session;
use motolii_store::{
    Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, Speed,
};
use motolii_timeline_pane::tokens::{Colors, Dimensions};
use motolii_timeline_pane::TimelinePane;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Axis {
    A,
    B,
    C,
}

impl Axis {
    fn label(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Preset {
    axis: Axis,
    number: u8,
}

impl Preset {
    fn label(self) -> String {
        format!("{}{:02}", self.axis.label(), self.number)
    }

    fn claim(self) -> &'static str {
        claim(self.axis, self.number)
    }
}

fn presets() -> Vec<Preset> {
    [Axis::A, Axis::B, Axis::C]
        .into_iter()
        .flat_map(|axis| (1..=10).map(move |number| Preset { axis, number }))
        .collect()
}

const A_CLAIMS: [&str; 10] = [
    "地と窪みの差を最小にして一枚面へ寄せる",
    "hairlineだけを読みやすくして面差は抑える",
    "地・パネル・raisedを三段として見せる",
    "railと時間面の境界を静かに分ける",
    "surfaceの段差を中庸にして長時間編集へ寄せる",
    "窪みを強め、transportと本体を分離する",
    "境界線を強め、密なレーンでも輪郭を保つ",
    "raised面を前へ出し、rulerの所在を明確にする",
    "hoverと通常面の差を主役にして操作口を見せる",
    "面差とhairlineを最大にして構造を即時に読ませる",
];

const B_CLAIMS: [&str; 10] = [
    "最小の間隔で編集情報を一画面へ集める",
    "行高を抑えつつ踏面は維持する",
    "transportとレーンを圧縮して構造を優先する",
    "詰まり気味の標準編集密度にする",
    "短い確認と連続編集の均衡を取る",
    "既定値を中心に余白の呼吸を足す",
    "行名と時間面を見失わないゆったりさにする",
    "property行の読み分けを優先して広げる",
    "transportを広く取り、操作の滞在場所を作る",
    "最もゆったりしたhero確認用の密度にする",
];

const C_CLAIMS: [&str; 10] = [
    "文字階層を近づけて連続したタイムラインにする",
    "captionとmicroを控えめにして本文を主役にする",
    "primaryとsecondaryの差で行名を先に読ませる",
    "mutedを沈め、時間面の情報を残す",
    "既定の対比を少しだけ明るくして長時間作業へ寄せる",
    "focusとactiveを一点へ集めて操作口を示す",
    "dataとway_timelineを前へ出し、素材の流れを見せる",
    "shapeとdataを分けてキー編集の種類を示す",
    "文字とaccentの対比を強め、heroの変化を読ませる",
    "最大の文字階層とink差で確認時の一瞥性を取る",
];

fn claim(axis: Axis, number: u8) -> &'static str {
    let index = number.saturating_sub(1).min(9) as usize;
    match axis {
        Axis::A => A_CLAIMS[index],
        Axis::B => B_CLAIMS[index],
        Axis::C => C_CLAIMS[index],
    }
}

fn step(number: u8) -> f32 {
    number.saturating_sub(1).min(9) as f32 / 9.0
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

/// 30案の唯一のスキン生成口。全案とも既定tokenから始め、既存の寸法・色の
/// フィールドだけを、軸ごとの無次元係数とtoken同士のblendで振る。
fn skin(axis: Axis, n: u8) -> (Dimensions, Colors) {
    let p = step(n);
    let mut dims = Dimensions::default();
    let mut colors = Colors::default();

    match axis {
        Axis::A => {
            let app = colors.surface_app;
            let panel = colors.surface_panel;
            let raised = colors.surface_raised;
            let hover = colors.surface_hover;
            let border_default = colors.border_default;
            let border_strong = colors.border_strong;
            let hairline = colors.border_hairline_weak;

            dims.border_width *= 0.55 + 0.95 * p;
            dims.focus_indicator_width *= 0.70 + 0.95 * p;

            colors.surface_app = blend(app, panel, 0.08 + 0.52 * p);
            colors.surface_panel = blend(panel, raised, 0.10 + 0.72 * p);
            colors.surface_raised = blend(raised, hover, 0.08 + 0.76 * p);
            colors.surface_hover = blend(hover, colors.focus, 0.04 + 0.34 * p);
            colors.border_default = blend(border_default, border_strong, 0.08 + 0.82 * p);
            colors.border_strong = blend(border_strong, colors.focus, 0.06 + 0.62 * p);
            colors.border_hairline_weak = blend(hairline, border_default, 0.06 + 0.78 * p);
            colors.timeline_time_band = blend(colors.timeline_time_band, raised, 0.08 + 0.24 * p);
            colors.timeline_row_zebra = blend(colors.timeline_row_zebra, hover, 0.08 + 0.24 * p);
            colors.timeline_grid_minor =
                blend(colors.timeline_grid_minor, border_default, 0.08 + 0.40 * p);
            colors.timeline_grid_major =
                blend(colors.timeline_grid_major, border_strong, 0.08 + 0.40 * p);
        }
        Axis::B => {
            let spacing = 0.72 + 0.50 * p;
            dims.spacing_xs *= spacing;
            dims.spacing_s *= 0.74 + 0.52 * p;
            dims.spacing_m *= 0.74 + 0.58 * p;
            dims.spacing_l *= 0.74 + 0.64 * p;
            dims.row_height *= 0.78 + 0.44 * p;
            dims.transport_band *= 0.82 + 0.36 * p;
            dims.pane_header_height *= 0.82 + 0.36 * p;
            dims.panel_header_height *= 0.84 + 0.32 * p;
            dims.timeline_param_row_height *= 0.80 + 0.38 * p;
            dims.timeline_transport_height *= 0.82 + 0.36 * p;
            dims.timeline_transport_button_width *= 0.82 + 0.36 * p;
            dims.timeline_transport_gap *= 0.72 + 0.64 * p;
            // interactive_target_min は密度軸でも縮めない。踏面の最低条件は共通契約。
        }
        Axis::C => {
            dims.title_text *= 0.84 + 0.32 * p;
            dims.body_text *= 0.84 + 0.36 * p;
            dims.caption_text *= 0.82 + 0.40 * p;
            dims.micro_text *= 0.84 + 0.42 * p;

            let primary = colors.text_primary;
            let secondary = colors.text_secondary;
            let muted = colors.text_muted;
            let focus = colors.focus;
            let active = colors.action_active;
            let data = colors.data;
            let shape = colors.shape;
            let way = colors.way_timeline;

            colors.text_primary = blend(primary, focus, 0.04 + 0.56 * p);
            colors.text_secondary = blend(secondary, primary, 0.04 + 0.64 * p);
            colors.text_muted = blend(muted, secondary, 0.04 + 0.72 * p);
            colors.focus = blend(focus, active, 0.04 + 0.66 * p);
            colors.action_active = blend(active, focus, 0.04 + 0.78 * p);
            colors.data = blend(data, active, 0.04 + 0.72 * p);
            colors.shape = blend(shape, data, 0.04 + 0.72 * p);
            colors.way_timeline = blend(way, shape, 0.04 + 0.78 * p);
            colors.state_selected = blend(colors.state_selected, active, 0.04 + 0.62 * p);
        }
    }

    (dims, colors)
}

fn fps30() -> Fps {
    Fps::try_new(30, 1).expect("30/1 は正の既約 fps")
}

/// `tests/transport_fence.rs` の Composition と `tests/split_fence.rs` の
/// `place_layer` を転記した固定 fixture。意味データはこのexampleで発明しない。
fn fixture() -> (Document, Session) {
    let mut document = Document::new();
    document
        .apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: fps30(),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .expect("comp 設定");

    document
        .apply_all([
            Intent::AddLayer(LayerId(1)),
            Intent::SetMeta {
                layer: LayerId(1),
                meta: LayerMeta {
                    source: LayerSource::Solid {
                        rgba: [255, 0, 0, 255],
                        width: 64,
                        height: 64,
                    },
                    order: 0,
                    timing: LayerTiming {
                        start: 10,
                        duration: 90,
                        source_in: 5,
                        speed: Speed::NORMAL,
                    },
                },
            },
        ])
        .expect("Solid layer 配置");

    (document, Session::default())
}

struct App {
    document: Document,
    session: Session,
    preset: Preset,
}

impl Default for App {
    fn default() -> Self {
        let (document, session) = fixture();
        Self {
            document,
            session,
            preset: Preset {
                axis: Axis::A,
                number: 1,
            },
        }
    }
}

#[derive(Clone, Debug)]
enum Message {
    Select(Preset),
    Pane,
}

fn update(state: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Select(preset) => state.preset = preset,
        Message::Pane => {}
    }
    Task::none()
}

fn view(state: &App) -> Element<'_, Message> {
    let (dims, colors) = skin(state.preset.axis, state.preset.number);
    let line = format!(
        "軸{}・{:02}・{}",
        state.preset.axis.label(),
        state.preset.number,
        state.preset.claim()
    );
    let selector = pick_list(Some(state.preset), presets(), |preset: &Preset| {
        preset.label()
    })
    .on_select(Message::Select);
    let header = row![text(line).width(Length::Fill), selector]
        .spacing(dims.spacing_m)
        .align_y(iced::alignment::Vertical::Center);

    let store = state.document.view();
    let pane = TimelinePane::new(
        &store,
        &state.session,
        dims,
        colors,
        iced::keyboard::Modifiers::default(),
    )
    .with_playing(true)
    .view_with_transport()
    .map(|_| Message::Pane);

    column![header, pane]
        .spacing(dims.spacing_s)
        .padding(dims.spacing_m)
        .into()
}

fn main() -> iced::Result {
    iced::application(App::default, update, view)
        .title("Motolii — Timeline skin drafts")
        .theme(|state: &App| {
            let (_, colors) = skin(state.preset.axis, state.preset.number);
            motolii_tokens_rs::theme_from_colors(&colors)
        })
        .centered()
        .run()
}
