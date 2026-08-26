//! Menubar の視覚スキン30案。
//!
//! メニュー項目の意味と並びは tests/menubar_oracle.rs の fixture を使い、
//! 本物の menu_bar をそのまま呼ぶ。example 側で変えるのは token pair と
//! 比較用の pick_list だけ。

use iced::widget::{column, pick_list, row, text};
use iced::{Element, Length, Task};
use motolii_menubar::{Item, Menu, menu_bar};
use motolii_tokens_rs::{Colors, Dimensions, theme_from_colors};

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

    fn claim(self) -> &'static str {
        const A: [&str; 10] = [
            "地と窪みを近づけ、メニューを静かな帯にする",
            "hairlineを弱くしてFile/Editを文字で読む",
            "barとopened面の差を一段だけ置く",
            "menuの窪みを先に拾わせる",
            "標準の二段面をMenubarへ写す",
            "hover面を操作可能性の合図にする",
            "opened面とhairlineの役割を分ける",
            "メニューの境界を密な面から切り出す",
            "強い罫線でopened面の所在を固定する",
            "最大の面差でも影に頼らず読む",
        ];
        const B: [&str; 10] = [
            "最短の余白でメニュー入口を並べる",
            "barの間隔を詰めて作業面を広げる",
            "compactなFile/Edit帯にする",
            "短い確認と操作を同じ密度に揃える",
            "標準密度を比較の基準にする",
            "menu項目の呼吸を少し増やす",
            "shortcutと動詞の間を読みやすくする",
            "項目間の区切りをゆったり置く",
            "長い動詞を落ち着いて読む密度にする",
            "hero確認用に最大の余白を取る",
        ];
        const C: [&str; 10] = [
            "文字段差を抑え、barを背景へ溶かす",
            "shortcutを静かな補助情報にする",
            "本文とshortcutの差を広げる",
            "mutedを沈め、動詞を先に読む",
            "標準の文字対比を比較の基準にする",
            "focusだけを入口の合図にする",
            "active accentを一点へ集める",
            "data色で操作の存在を補助する",
            "文字とopened面の対比を強める",
            "Menubar全体をheroの操作入口として読む",
        ];
        let index = usize::from(self.number.clamp(1, 10) - 1);
        match self.axis {
            Axis::A => A[index],
            Axis::B => B[index],
            Axis::C => C[index],
        }
    }
}

fn presets() -> Vec<Preset> {
    [Axis::A, Axis::B, Axis::C]
        .into_iter()
        .flat_map(|axis| (1..=10).map(move |number| Preset { axis, number }))
        .collect()
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

/// 既存 token の default から全30案を組み立てる唯一の口。
fn skin(axis: Axis, number: u8) -> (Dimensions, Colors) {
    let mut dims = Dimensions::default();
    let base_dims = dims;
    let mut colors = Colors::default();
    let base_colors = colors;
    let index = usize::from(number.clamp(1, 10) - 1);

    match axis {
        Axis::A => {
            let t = index as f32 / 9.0;
            dims.border_width = base_dims.border_width * (0.70 + 1.30 * t);
            dims.focus_indicator_width = base_dims.focus_indicator_width * (0.75 + 1.25 * t);
            colors.surface_app = blend(
                base_colors.surface_app,
                base_colors.surface_panel,
                0.04 + 0.22 * t,
            );
            colors.surface_panel = blend(
                base_colors.surface_panel,
                base_colors.surface_raised,
                0.10 + 0.66 * t,
            );
            colors.surface_raised = blend(
                base_colors.surface_raised,
                base_colors.surface_hover,
                0.10 + 0.70 * t,
            );
            colors.surface_hover = blend(
                base_colors.surface_hover,
                base_colors.focus,
                0.04 + 0.28 * t,
            );
            colors.border_default = blend(
                base_colors.border_default,
                base_colors.border_strong,
                0.08 + 0.78 * t,
            );
            colors.border_strong = blend(
                base_colors.border_strong,
                base_colors.focus,
                0.06 + 0.70 * t,
            );
            colors.border_hairline_weak = blend(
                base_colors.border_hairline_weak,
                base_colors.border_default,
                0.08 + 0.78 * t,
            );
        }
        Axis::B => {
            let density = 0.72 + 0.52 * index as f32 / 9.0;
            dims.spacing_xs = base_dims.spacing_xs * density;
            dims.spacing_s = base_dims.spacing_s * density;
            dims.spacing_m = base_dims.spacing_m * density;
            dims.spacing_l = base_dims.spacing_l * density;
            dims.row_height = base_dims.row_height * density;
            dims.panel_header_height = base_dims.panel_header_height * density;
            dims.pane_header_height = base_dims.pane_header_height * density;
            dims.transport_band = base_dims.transport_band * density;
            // interactive_target_min は密度軸でも共通契約として保持する。
        }
        Axis::C => {
            let t = index as f32 / 9.0;
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
            let type_scale = 0.82 + 0.38 * t;
            dims.title_text = base_dims.title_text * type_scale;
            dims.body_text = base_dims.body_text * type_scale;
            dims.caption_text = base_dims.caption_text * type_scale;
            dims.micro_text = base_dims.micro_text * type_scale;
            colors.text_primary =
                blend(base_colors.text_primary, base_colors.focus, 0.06 + 0.62 * t);
            colors.text_secondary = blend(
                base_colors.text_secondary,
                colors.text_primary,
                0.06 + 0.66 * t,
            );
            colors.text_muted = blend(
                base_colors.text_muted,
                colors.text_secondary,
                0.06 + 0.70 * t,
            );
            colors.focus = blend(base_colors.focus, accent, 0.08 + 0.48 * t);
            colors.action_active = blend(base_colors.action_active, accent, 0.18 + 0.68 * t);
            colors.data = blend(base_colors.data, accent, 0.14 + 0.62 * t);
            colors.shape = blend(base_colors.shape, accent, 0.14 + 0.62 * t);
            colors.way_timeline = blend(base_colors.way_timeline, accent, 0.14 + 0.62 * t);
        }
    }

    colors.state_selected = blend(colors.surface_raised, colors.action_active, 0.18);
    colors.state_disabled = blend(colors.text_muted, colors.surface_panel, 0.40);
    (dims, colors)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuMessage {
    NewProject,
    SaveAs,
    Undo,
}

/// tests/menubar_oracle.rs の two_menus() と同じ固定意味データ。
fn fixture_menus() -> Vec<Menu<MenuMessage>> {
    vec![
        Menu {
            label: "File",
            items: vec![
                Item {
                    label: "New Project",
                    shortcut: Some("Cmd+N"),
                    message: MenuMessage::NewProject,
                },
                Item {
                    label: "Save a Copy",
                    shortcut: None,
                    message: MenuMessage::SaveAs,
                },
            ],
        },
        Menu {
            label: "Edit",
            items: vec![Item {
                label: "Undo",
                shortcut: Some("Cmd+Z"),
                message: MenuMessage::Undo,
            }],
        },
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Message {
    Select(Preset),
    Menu(MenuMessage),
}

#[derive(Clone, Copy, Debug)]
struct App {
    selected: Preset,
}

impl Default for App {
    fn default() -> Self {
        Self {
            selected: Preset {
                axis: Axis::A,
                number: 1,
            },
        }
    }
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    if let Message::Select(preset) = message {
        app.selected = preset;
    }
    Task::none()
}

fn view(app: &App) -> Element<'_, Message> {
    let (dims, colors) = skin(app.selected.axis, app.selected.number);
    let picker = pick_list(Some(app.selected), presets(), |preset: &Preset| {
        preset.label()
    })
    .on_select(Message::Select);
    let header = row![
        text("MENUBAR SKINS")
            .size(dims.theme().text.title)
            .color(colors.text_primary)
            .width(Length::Fill),
        picker,
    ]
    .spacing(dims.theme().space.m)
    .align_y(iced::Alignment::Center);
    let claim_line = text(format!(
        "{} · {} · {}",
        app.selected.axis.label(),
        app.selected.number,
        app.selected.claim()
    ))
    .size(dims.theme().text.caption)
    .color(colors.text_secondary);
    let menu = menu_bar(fixture_menus(), dims, colors).map(Message::Menu);

    column![header, claim_line, menu]
        .spacing(dims.theme().space.s)
        .padding(dims.theme().space.m)
        .into()
}

fn main() -> iced::Result {
    iced::application(App::default, update, view)
        .title("Motolii Menubar skins")
        .theme(|app: &App| {
            let colors = skin(app.selected.axis, app.selected.number).1;
            theme_from_colors(&colors)
        })
        .run()
}
