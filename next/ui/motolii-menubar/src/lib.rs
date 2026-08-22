//! wraps: iced_aw menu (rev 924be285) の fork API 移植。上流が fork の iced へ追随したら vendored を捨てて戻す
//!
//! メニューバー基盤(MB-2 上書き裁定 2026-08-22 —「iced_aw が使えないなら、
//! 使えるようにすべき。ヘッダー部分(menu)が欲しいだけ」)。iced_aw は crate ごと
//! だと fork の iced(rev 73e686ee)に対し `Shell::local` の Bus/Vec 非互換で
//! ビルド不能(probe: `docs/reviews/2026-08-22-iced-aw-menu-probe.md`)なので、
//! menu widget 一式だけを [`vendored`] へ移植し、非互換2箇所を fork の新 API へ
//! 追随修正した。
//!
//! ## 公開面は最小
//!
//! iced_aw の全オプション(`menu_items!` マクロ・`DrawPath` 選択・scroll 速度…)は
//! 公開しない — [`Item`]/[`Menu`]/[`menu_bar`] の3口だけ。`Item` の形
//! (`label`/`shortcut`/`message`)は shell の既存 `menu.rs` と同じで、native-menu
//! 調査(`docs/reviews/2026-08-22-native-menu-and-stock-widgets-survey.md` §3)が
//! muda(`MenuItemBuilder::new(label).accelerator(...)`)へ 1:1 で写せるとした
//! 最小構造 — 将来ネイティブメニュー化しても item 定義層は書き直しにならない。
//!
//! ## 見た目は枠の文法(裁定179)
//!
//! - バー項目: 常時輪郭なし・素の文字。hover で面(`surface_hover`)—
//!   [`iced::widget::hover`] の base/top 差し替えで実現(輪郭を持つ button を
//!   使わない)。開いている間は vendored の `DrawPath::FakeHovering`(既定)が
//!   同じ hover 面を保つ。
//! - 開いた menu の面: 明度1段(`surface_raised` — パネル面 `surface_panel` の
//!   1段上)+角丸(`menubar_corner_radius`)。
//! - menu 項目: 輪郭なし button、hover で面。shortcut は右寄せ `text_muted`
//!   (shell `menu.rs` の dropdown と同じ読み筋)。
//!
//! 色は `motolii-tokens-rs` の [`Colors`] ロールのみ・寸法は `dimensions.json` の
//! menubar 節([`Dimensions::menubar_menu_width`]/[`Dimensions::menubar_corner_radius`])
//! と既存 spacing/type-scale 段のみ — raw 値の直書きなし(裁定142)。

use iced::widget::{button, container, hover, row, text, Space};
use iced::{Element, Length};
use motolii_tokens_rs::{Colors, Dimensions};

pub mod vendored;

// vendored コード(upstream iced_aw のまま)は `crate::style::…` で style catalog を
// 参照する — upstream のモジュール木(`src/style/`)をこの crate 根へ写す代わりに、
// 同じ path 名で再輸出して無改変のまま解決させる。
pub(crate) use vendored::style;

use vendored::menu::{Item as VendoredItem, Menu as VendoredMenu, MenuBar};

/// メニュー1項目。**`message` は必須**(Option ではない)— captured→publish 必須
/// (q0 柵、silent 項目ゼロ)を型で保証する(shell `menu.rs` の `Item` と同じ判断)。
/// `shortcut` はプレーン ASCII 表記(`Cmd+N` 等)の**表示専用** — キーの実配線は
/// shell 側 `resolve_navigation_key` の仕事で、ここでは意味を複製しない。
/// `None` は「shortcut 出典ゼロ」をそのまま表す。
pub struct Item<Message> {
    /// 動詞名(メニューに出る文字列)。
    pub label: &'static str,
    /// 右寄せで併記する shortcut 表記。表示のみ。
    pub shortcut: Option<&'static str>,
    /// click で publish する `Message`。
    pub message: Message,
}

/// トップレベルのメニュー1本("File"/"Edit" 等) — バー項目のラベルと、開いた時の
/// 項目列。
pub struct Menu<Message> {
    /// バーに出る文字列。
    pub label: &'static str,
    /// 開いた menu の項目列(上から順)。
    pub items: Vec<Item<Message>>,
}

/// メニューバー本体を組む。バー項目 click で menu が開き、項目 click で `message`
/// が publish される(開閉状態は widget 内部が持つ — shell 側に Toggle 系
/// `Message` は要らない)。
pub fn menu_bar<'a, Message: Clone + 'a>(
    menus: Vec<Menu<Message>>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'a, Message> {
    let roots: Vec<VendoredItem<'a, Message, iced::Theme, iced::Renderer>> = menus
        .into_iter()
        .map(|menu| {
            let dropdown = VendoredMenu::new(
                menu.items
                    .into_iter()
                    .map(|item| VendoredItem::new(leaf(item, dims, colors)))
                    .collect(),
            )
            .width(Length::Fixed(dims.menubar_menu_width))
            .padding(dims.spacing_xs);

            VendoredItem::with_menu(bar_item(menu.label, dims, colors), dropdown)
        })
        .collect();

    MenuBar::new(roots)
        .style(move |_theme, _status| bar_style(dims, colors))
        .into()
}

/// バー項目(トリガー)。枠の文法: 常時輪郭なし・素の文字、hover で面。
/// button を使わない — `on_press` の無い button は常に `Status::Disabled` で
/// hover を検出できず、`on_press` を持たせると menu の開閉(vendored 側が
/// バー面の press で行う)に加えて余計な `Message` が飛ぶ。`hover(base, top)` は
/// クリックを一切食わないので、開閉は vendored の `MenuBar` にそのまま届く。
fn bar_item<'a, Message: 'a>(
    label: &'static str,
    dims: Dimensions,
    colors: Colors,
) -> Element<'a, Message> {
    hover(
        bar_label(label, dims, colors, None),
        bar_label(label, dims, colors, Some(colors.surface_hover)),
    )
}

/// バー項目の実体(hover の base/top 両方が使う)。`face` が hover 時の面。
fn bar_label<'a, Message: 'a>(
    label: &'static str,
    dims: Dimensions,
    colors: Colors,
    face: Option<iced::Color>,
) -> Element<'a, Message> {
    container(text(label).size(dims.body_text).color(colors.text_primary))
        .padding([dims.spacing_xs, dims.spacing_m])
        .style(move |_theme| container::Style {
            background: face.map(iced::Background::Color),
            ..container::Style::default()
        })
        .into()
}

/// menu 項目(葉)。動詞名+右寄せ shortcut(`text_muted`)の1行 button —
/// 輪郭なし、hover/press で面(shell `menu.rs` dropdown の読み筋を踏襲しつつ、
/// 枠の文法どおり常時輪郭は消す)。
fn leaf<'a, Message: Clone + 'a>(
    item: Item<Message>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'a, Message> {
    let shortcut: Element<'a, Message> = match item.shortcut {
        Some(shortcut) => text(shortcut)
            .size(dims.caption_text)
            .color(colors.text_muted)
            .into(),
        // shortcut 出典ゼロの項目は右側を空けるだけ — 存在しない shortcut を
        // 発明しない(shell `menu.rs` と同じ判断)。
        None => Space::new().into(),
    };

    button(
        row![
            text(item.label).size(dims.body_text),
            Space::new().width(Length::Fill),
            shortcut,
        ]
        .spacing(dims.spacing_m)
        .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Fill)
    .style(move |_theme, status| leaf_style(colors, status))
    .on_press(item.message)
    .into()
}

/// 葉 button の style。枠の文法: 輪郭なし・素の文字、hover/press で面だけが変わる。
fn leaf_style(colors: Colors, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Some(iced::Background::Color(colors.surface_hover)),
        button::Status::Pressed => Some(iced::Background::Color(colors.state_selected)),
        button::Status::Active | button::Status::Disabled => None,
    };
    button::Style {
        background,
        text_color: colors.text_primary,
        border: iced::Border::default(),
        ..button::Style::default()
    }
}

/// vendored `MenuBar` の style catalog 値。バーは透明(header 帯の面に同居する —
/// バー自身は面を持たない)、開いた menu の面は明度1段(`surface_raised`)+
/// 角丸+hairline。`path`(`DrawPath::Backdrop` 用の面)は使わない —
/// `FakeHovering`(既定)では bar/leaf 自身の hover 面が path 表示を兼ねる。
fn bar_style(dims: Dimensions, colors: Colors) -> style::menu_bar::Style {
    style::menu_bar::Style {
        bar_background: iced::Background::Color(iced::Color::TRANSPARENT),
        bar_border: iced::Border::default(),
        bar_shadow: iced::Shadow::default(),
        menu_background: iced::Background::Color(colors.surface_raised),
        menu_border: iced::Border {
            color: colors.border_default,
            width: dims.border_width,
            radius: dims.menubar_corner_radius.into(),
        },
        // フラットの文法(裁定137/139 系)— 影で浮かせない。面の明度1段が層を語る。
        menu_shadow: iced::Shadow::default(),
        path: iced::Background::Color(iced::Color::TRANSPARENT),
        path_border: iced::Border::default(),
    }
}
