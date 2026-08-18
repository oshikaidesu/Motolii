//! Inspector pane の絵(M-4b)。**モデルを書く道はここに無い** — 受け取った
//! [`InspectorSeat`] を並べ、押された事実を [`InspectorEvent`] として返すだけ。
//!
//! 構成は 2026-08-13 裁定の採択案 A(4 section 常設)+ 同日の key add UX 決定:
//!
//! 1. identity 行(名前・種別・M / S)。押下状態は Document 由来の model から
//! 2. TRANSFORM: Position / Scale(Vec2 = 1 header + 1 key ボタン + X/Y 値行)、
//!    Rotation / Opacity(値行に key ボタン)。key ボタンは3状態
//! 3. EFFECTS: 共有 FX の一覧 + ON/OFF。param は F64 / Vec2 / Vec3 / Color だけ行を出す
//! 4. Audio: **出さない** — gain のエディタ操作 API がまだ無く、接続できない
//!    chrome は置かない(Q0。座席が立ったら section ごと足す)
//!
//! ## 見た目の正本(2026-08-18 視覚再現レーン)
//!
//! 訂正後の正本は egui shell の絵ではなく、`docs/mocks-ui/public/
//! inspector-library.css` + `inspector-library.html` そのもの(この節の関数の
//! doc コメントは全部 `css:行番号` でそこを指す)。色は [`theme::Tokens`] の
//! semantic role だけを読み、**独自 hex は1つも置かない**。ここで足した寸法
//! (`dims` module)は行高・帯幅など幅に依らない値だけを css から写した —
//! 3D の Z 列や shape swatch など、この製品の read-model
//! (`crate::inspector_model`)にまだ無いデータを要求する箇所は**足さない**
//! (Q0: 死に chrome 禁止。無い物は無いままにして、`docs/reviews/evidence/
//! iced-visual-fidelity/README.md` に残差として書く)。
//!
//! スクラブ部品と key ボタンは [`crate::widgets`] の本物(統合 wave で stub から
//! 差し替えた)。

use iced::widget::{button, column, container, row, rule, space, text};
use iced::{Center, Element, Fill};

use motolii_ui::blitz_shell::UiEditParam;

use crate::inspector_model::{
    arity, param_label, EffectParamValue, EffectSection, InspectorModel, InspectorSeat, ParamRow,
};
use crate::message::Message;
use crate::theme::Tokens;
use crate::widgets::{key_button, scrub_value, ScrubEvent, ScrubSpec};

/// pane の見出し。
pub const TITLE: &str = "Inspector";
/// 選択なしの空状態(egui 版 `seat_live` と同じ文言)。
pub const NO_SELECTION: &str = "No selection \u{2014} pick a layer in the Timeline";
/// TRANSFORM section の見出し。
pub const TRANSFORM: &str = "TRANSFORM";
/// APPEARANCE section の見出し(inspector-library.html:29-60 — TRANSFORM の下)。
pub const APPEARANCE: &str = "APPEARANCE";
/// EFFECTS section の見出し。
pub const EFFECTS: &str = "EFFECTS";
/// 共有 FX が1つも無いときの正直な一言(幽霊行を出さない — Q7)。
pub const NO_EFFECTS: &str = "No shared FX";
/// 最下部の常設ヒント行(inspector-library.html のフッタと同じ役目だが、
/// 文言は**この製品で実際に効く gesture だけ**を言う — `scrub_value` は
/// ドラッグ/ダブルクリック/Esc しか持たず、右クリックのリセットは無い)。
pub const HINT: &str = "Drag to scrub \u{b7} double-click to type \u{b7} Esc to cancel";

/// 寸法 — `docs/mocks-ui/public/inspector-library.css` の写し(幅に依らない
/// 値だけ。pane 幅は 300px で css 正本の 496px より狭いので、列幅そのものは
/// [`transform_rows`] 側で pane 幅に収まる値を別に決めている)。
mod dims {
    /// inspector-library.css:29 `.panelHeader { height: 29px }`
    pub const PANEL_HEADER_H: f32 = 29.0;
    /// inspector-library.css:37 `.panelHeader::before { width:3px; height:13px }`
    pub const HEADER_ACCENT_W: f32 = 3.0;
    pub const HEADER_ACCENT_H: f32 = 13.0;
    /// inspector-library.css:76 `.selectionSummary { height: 46px }`
    pub const SUMMARY_H: f32 = 46.0;
    /// inspector-library.css:99 `.layerStateButton { width:22px; height:21px }`
    pub const LAYER_STATE_W: f32 = 22.0;
    pub const LAYER_STATE_H: f32 = 21.0;
    /// inspector-library.css:122-126 `.columnHeader { height: 21px }`
    pub const COLUMN_HEADER_H: f32 = 21.0;
    /// inspector-library.css:141-142 `.tableSection h2 { height: 23px }`
    pub const SECTION_H: f32 = 23.0;
    /// inspector-library.css:291 `.propertyRow::before { width: 3px }`
    pub const ROW_BAND_W: f32 = 3.0;
    /// 値セル1本ぶんの幅。css 正本は 64px(inspector-library.css:118) — pane が
    /// 狭くても同じ値を使う(独自の値を発明しない。はみ出れば scrollable が拾う)。
    pub const VALUE_COL_W: f32 = 64.0;
    /// inspector-library.css:207 `.effectBadge { width:17px; height:13px }`
    pub const FX_BADGE_W: f32 = 17.0;
    pub const FX_BADGE_H: f32 = 13.0;
    /// inspector-library.css:217 `.effectEnable { min-width:25px; height:15px }`
    pub const FX_PILL_MIN_W: f32 = 25.0;
    pub const FX_PILL_H: f32 = 15.0;
    /// inspector-library.css:313-314 `.propertyName i { width:15px; height:15px }`
    /// (host TRANSFORM/APPEARANCE 行の kind icon。Q0: FX param 行はこの round の
    /// 残差指示に無いので触らない — icon は host 4 param だけに足す)。
    pub const KIND_ICON: f32 = 15.0;
}

/// Inspector で押された事実。「では何をするか」(intent 化)は
/// `Shell::update` が決める(M-1 と同じ層の分け方)。
#[derive(Debug, Clone, PartialEq)]
pub enum InspectorEvent {
    /// M を押した。押下状態は Document が持つので反転要求だけ。
    ToggleMute,
    /// S を押した。同上。
    ToggleSolo,
    /// ◇ / ◆ を押した(playhead へキーを打つ / 値を更新する)。
    KeyPressed(UiEditParam),
    /// スクラブ部品の1事象。
    Scrub {
        param: UiEditParam,
        component: usize,
        event: ScrubEvent,
    },
    /// 共有 FX の ON/OFF。`enabled` は**これから成るべき状態**。
    SetEffectEnabled { definition_id: u64, enabled: bool },
}

/// pane ぜんたい。`seat` は accepted snapshot から導出済みの投影で、
/// この関数は状態を持たない(毎フレーム作り直してよい)。
///
/// inspector-library.css:18 `.inspectorShell { background: surface-panel }` —
/// 帯・行の chrome が地の色まで塗り切れるよう、外側の padding は持たない
/// (padding は各 section/row が自分で持つ)。
pub fn inspector<'a>(seat: InspectorSeat, editor_status: Option<String>) -> Element<'a, Message> {
    let meta = match &seat {
        InspectorSeat::Ready(model) => Some(model.item_kind.to_owned()),
        _ => None,
    };

    // identity 行 + 列見出しは**選択が在るときだけ**、かつ scroll の外(固定)。
    // inspector-library.css:76,108 の写し(`.selectionSummary` は
    // `.tableScroller` の外、`.columnHeader` はスクロール域の中で `sticky`)。
    // この iced 版は sticky を持たないので列見出しごと外に出す —
    // 常に見えている方が sticky の近似としては素直(残差として README に書く)。
    let mut header = column![panel_header(meta)];
    let sections: Element<'_, Message> = match seat {
        InspectorSeat::NoSelection => muted_notice(NO_SELECTION),
        InspectorSeat::Multi(count) => {
            muted_notice(&format!("{count} items selected \u{2014} pick one to edit"))
        }
        InspectorSeat::Unreadable(reason) => muted_notice(&reason),
        InspectorSeat::Ready(model) => {
            header = header.push(identity(&model)).push(column_header());
            let mut body = column![transform(&model)];
            if let Some(appearance) = appearance(&model) {
                body = body.push(appearance);
            }
            body.push(effects(model.effects)).into()
        }
    };

    // TRANSFORM/EFFECTS は行数が伸びる(FX が増える)ので scrollable に包む —
    // 包まないと、伸びた分だけ既存行が押し潰されて 0 高さになり、押せない
    // ボタンが出る(2026-08-18 視覚再現レーンで実測して踏んだ罠)。
    let scroller = iced::widget::scrollable(sections).width(Fill).height(Fill);

    let mut whole = column![header, scroller];
    if let Some(status) = editor_status.filter(|status| !status.is_empty()) {
        whole = whole.push(
            container(text(status).size(11).style(crate::theme::style::text_muted)).padding([4, 9]),
        );
    }
    whole = whole.push(footer_hint());

    container(whole.width(Fill).height(Fill))
        .width(Fill)
        .height(Fill)
        .style(panel_style)
        .into()
}

/// 空/多数/不読の一言。inspector-library.html にこの3状態の見本は無い
/// (製品側の Q7 拡張)ので、chrome は最小限(padding だけ)にする。
fn muted_notice<'a>(line: &str) -> Element<'a, Message> {
    container(
        text(line.to_owned())
            .size(12)
            .style(crate::theme::style::text_secondary),
    )
    .padding(10)
    .into()
}

/// 0. panel header — 左に `way_inspector` の accent bar、"Inspector" の見出し、
/// 右に対象の種別(選択が在るときだけ)。inspector-library.css:29-47。
fn panel_header<'a>(meta: Option<String>) -> Element<'a, Message> {
    let mut header = row![
        color_bar(
            dims::HEADER_ACCENT_W,
            dims::HEADER_ACCENT_H,
            Tokens::DARK.way_inspector
        ),
        text(TITLE).size(11).font(weight(iced::font::Weight::Bold)),
    ]
    .spacing(7)
    .align_y(Center);
    if let Some(meta) = meta {
        header = header
            .push(space().width(Fill))
            .push(text(meta).size(9).style(crate::theme::style::text_muted));
    }
    column![
        container(header)
            .padding(iced::padding::left(9).right(9))
            .width(Fill)
            .height(dims::PANEL_HEADER_H)
            .align_y(Center)
            .style(bottom_border_style),
        rule::horizontal(1).style(crate::theme::style::separator),
    ]
    .into()
}

/// 塗りだけの帯(accent bar に使う。固定高さ)。
fn color_bar<'a>(w: f32, h: f32, color: iced::Color) -> Element<'a, Message> {
    container(space().width(w).height(h))
        .width(w)
        .height(h)
        .style(move |_theme: &iced::Theme| container::Style {
            background: Some(color.into()),
            ..container::Style::default()
        })
        .into()
}

/// 行の左に立つ縦帯(property row / FX header 帯)。行の実高さは呼び出し側の
/// 内容で決まる(可変)ので、固定高さでは潰れる — `color_bar` の h=0 で帯が
/// 消えていたのがこの round で見つかった不具合(2026-08-19)。ここは `Fill` で
/// 親の高さに追随する。inspector-library.css:291 `.propertyRow::before`。
fn row_band<'a>(w: f32, color: iced::Color) -> Element<'a, Message> {
    container(space().width(w).height(Fill))
        .width(w)
        .height(Fill)
        .style(move |_theme: &iced::Theme| container::Style {
            background: Some(color.into()),
            ..container::Style::default()
        })
        .into()
}

/// TRANSFORM/APPEARANCE の host param 1つぶんの kind icon。
/// inspector-library.css:313-326 `.propertyName i`(border + glyph は両方
/// `--property-color`)。角丸は css 側にも無いので足さない(Ableton 風フラット、
/// 2026-07-14 裁定)。
fn kind_icon<'a>(glyph: &'static str, color: iced::Color) -> Element<'a, Message> {
    container(
        text(glyph)
            .size(9)
            .shaping(text::Shaping::Advanced)
            .align_x(Center)
            .align_y(Center)
            .style(move |_theme: &iced::Theme| iced::widget::text::Style { color: Some(color) }),
    )
    .width(dims::KIND_ICON)
    .height(dims::KIND_ICON)
    .center_x(dims::KIND_ICON)
    .center_y(dims::KIND_ICON)
    .style(move |_theme: &iced::Theme| container::Style {
        border: iced::Border {
            color,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

/// host TRANSFORM/APPEARANCE param → kind icon の字。inspector-library.css:
/// 328-355 の絵を単一グリフへ簡約(色は property-color 側で運ぶので、ここは
/// 形の意味だけ選ぶ): Position=十字、Rotation=回転矢印、Scale=双方向矢印、
/// Opacity=半月。Fill は口が無いので割り当てない(Q0)。
fn transform_kind_glyph(param: UiEditParam) -> &'static str {
    match param {
        UiEditParam::Position => "+",
        UiEditParam::Rotation => "\u{21bb}",
        UiEditParam::Scale => "\u{2194}",
        UiEditParam::Opacity => "\u{25d0}",
    }
}

/// 下辺だけの区切り線を持つ帯。`iced::Border` は4辺一律なので、
/// 実際の1px線は別 widget(`rule::horizontal`)ではなく、この bg 用
/// container の直後に呼び出し側が [`crate::theme::style::separator`] を
/// 積む(既存 `view.rs` の帯と同じ流儀)。ここは背景色だけを塗る。
fn bottom_border_style(theme: &iced::Theme) -> container::Style {
    let t = Tokens::resolve(theme);
    container::Style {
        background: Some(t.surface_panel.into()),
        border: iced::Border {
            color: t.border_default,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// 太字 font(iced はこの fork で letter-spacing を持たないので、section
/// 見出しの字間はここでは再現しない — `docs/reviews/evidence/
/// iced-visual-fidelity/README.md` の残差)。
fn weight(weight: iced::font::Weight) -> iced::Font {
    iced::Font {
        weight,
        ..iced::Font::DEFAULT
    }
}

/// 1. identity 行。名前・種別と、M / S(押下状態は Document 由来)。
/// inspector-library.css:76-107 `.selectionSummary`。
fn identity<'a>(model: &InspectorModel) -> Element<'a, Message> {
    let meta = match model.child_count {
        Some(children) => format!(
            "{} \u{b7} {} children \u{b7} {} shared FX",
            model.item_kind,
            children,
            model.effects.len()
        ),
        None => format!(
            "{} \u{b7} {} shared FX",
            model.item_kind,
            model.effects.len()
        ),
    };
    let body = row![
        column![
            text(model.layer_name.clone())
                .size(11)
                .font(weight(iced::font::Weight::Bold)),
            text(meta).size(9).style(crate::theme::style::text_muted)
        ]
        .spacing(2),
        space().width(Fill),
        flag_button(
            "M",
            model.muted,
            Message::Inspector(InspectorEvent::ToggleMute)
        ),
        flag_button(
            "S",
            model.solo,
            Message::Inspector(InspectorEvent::ToggleSolo)
        ),
    ]
    .spacing(6)
    .align_y(Center);

    column![
        container(body)
            .padding([6, 10])
            .width(Fill)
            .height(dims::SUMMARY_H)
            .align_y(Center)
            .style(raised_panel_style),
        rule::horizontal(1).style(crate::theme::style::separator),
    ]
    .into()
}

/// identity 行の面 — `surface_raised`(inspector-library.css:83)。
fn raised_panel_style(theme: &iced::Theme) -> container::Style {
    let t = Tokens::resolve(theme);
    container::Style {
        background: Some(t.surface_raised.into()),
        border: iced::Border {
            color: t.border_default,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// M / S の1枚。inspector-library.css:99-103 の3状態(既定 / M pressed /
/// S pressed)を token だけで写す。
fn flag_button<'a>(label: &'a str, pressed: bool, message: Message) -> Element<'a, Message> {
    button(
        text(label)
            .size(9)
            .font(weight(iced::font::Weight::ExtraBold)),
    )
    .padding(0)
    .width(dims::LAYER_STATE_W)
    .height(dims::LAYER_STATE_H)
    .style(move |theme: &iced::Theme, status| flag_button_style(theme, status, label, pressed))
    .on_press(message)
    .into()
}

fn flag_button_style(
    theme: &iced::Theme,
    status: button::Status,
    label: &str,
    pressed: bool,
) -> button::Style {
    let t = Tokens::resolve(theme);
    // S = action_active(css `[data-layer-state="solo"]`)、M = text_muted
    // (css `[data-layer-state="mute"]`)。
    let accent = if label == "S" {
        t.action_active
    } else {
        t.text_muted
    };
    let (border, background, text_color) = match (pressed, status) {
        (true, _) => (
            accent,
            iced::Color { a: 0.18, ..accent }.into(),
            if label == "S" { accent } else { t.text_primary },
        ),
        (false, button::Status::Hovered | button::Status::Pressed) => {
            (t.border_strong, t.surface_panel, t.text_primary)
        }
        (false, _) => (t.border_default, t.surface_app, t.text_muted),
    };
    button::Style {
        background: Some(background.into()),
        text_color,
        border: iced::Border {
            color: border,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..button::Style::default()
    }
}

/// 列見出し行(Property / X / Y)。inspector-library.css:115-138。
/// この pane は Z 列を持たない(製品が2D transformなので発明しない)。
fn column_header<'a>() -> Element<'a, Message> {
    let heading = |label: &'static str| {
        text(label)
            .size(8)
            .font(weight(iced::font::Weight::Bold))
            .style(crate::theme::style::text_muted)
    };
    column![
        container(
            row![
                heading("Property"),
                space().width(Fill),
                container(heading("X"))
                    .width(dims::VALUE_COL_W)
                    .center_x(dims::VALUE_COL_W),
                container(heading("Y"))
                    .width(dims::VALUE_COL_W)
                    .center_x(dims::VALUE_COL_W),
            ]
            .spacing(0)
            .align_y(Center),
        )
        // property_row の content padding([4,8])と揃える — X/Y 見出しの右端が
        // 値セルの右端の真上に来る(残差5: 「列が揃っていない」の対処)。
        .padding(iced::padding::left(8).right(8))
        .width(Fill)
        .height(dims::COLUMN_HEADER_H)
        .align_y(Center)
        .style(section_band_style),
        rule::horizontal(1).style(crate::theme::style::separator),
    ]
    .into()
}

/// section 帯の面 — `surface_app`(inspector-library.css:128,149。property 行
/// の `surface_panel` より暗い「凹んだ」帯)。
fn section_band_style(theme: &iced::Theme) -> container::Style {
    let t = Tokens::resolve(theme);
    container::Style {
        background: Some(t.surface_app.into()),
        border: iced::Border {
            color: t.border_default,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// 2. TRANSFORM section — Position / Scale / Rotation。
/// APPEARANCE(Opacity)はここでは出さない — inspector-library.html:29-60 の
/// 並びどおり TRANSFORM の下に別 section として立てる(`appearance` 関数)。
/// この分割は表示のグルーピングだけで、`model.transform` の中身・順序・
/// read-model は一切変えていない(意味は触らない柵)。
fn transform<'a>(model: &InspectorModel) -> Element<'a, Message> {
    let mut section = column![section_heading(TRANSFORM, None)];
    for param_row in &model.transform {
        if param_row.param != UiEditParam::Opacity {
            section = section.push(transform_rows(param_row));
        }
    }
    section.into()
}

/// APPEARANCE section — この read-model が実際に持つ intent は Opacity だけ
/// (Fill は書き口が無いので出さない、Q0)。1行も無ければ section ごと省く
/// (幽霊 section を作らない — Q7 と同じ規律)。
fn appearance<'a>(model: &InspectorModel) -> Option<Element<'a, Message>> {
    let opacity = model
        .transform
        .iter()
        .find(|row| row.param == UiEditParam::Opacity)?;
    Some(column![section_heading(APPEARANCE, None), transform_rows(opacity)].into())
}

/// section 見出し行。inspector-library.css:141-162。fold chevron は**足さない**
/// — この製品の read-model は section の折り畳み状態を持たず、押しても
/// 何も起きないボタンは Q0 違反になる(残差として README に書く)。
fn section_heading<'a>(label: &'static str, count: Option<String>) -> Element<'a, Message> {
    let mut row = row![text(label)
        .size(8)
        .font(weight(iced::font::Weight::ExtraBold))
        .style(crate::theme::style::text_secondary)]
    .spacing(0)
    .align_y(Center);
    if let Some(count) = count {
        row = row
            .push(space().width(Fill))
            .push(text(count).size(8).style(crate::theme::style::text_muted));
    }
    column![
        container(row)
            .padding(iced::padding::left(10).right(10))
            .width(Fill)
            .height(dims::SECTION_H)
            .align_y(Center)
            .style(section_band_style),
        rule::horizontal(1).style(crate::theme::style::separator),
    ]
    .into()
}

/// Transform 1 param ぶんの行。Vec2 は 1 header + 1 key ボタン + X/Y 値行、
/// scalar は 1 行に値と key ボタン(2026-08-13 mapping)。
///
/// 左の色帯(`property-color`)は inspector-library.html の host 行と同じ
/// 割当: Position=data / Rotation=action-active / Scale=shape /
/// Opacity=text-secondary(inspector-library.html:29-60)。
fn transform_rows<'a>(param_row: &ParamRow) -> Element<'a, Message> {
    let param = param_row.param;
    let label = param_label(param);
    let band = row_band_color(param);
    let glyph = transform_kind_glyph(param);
    if !param_row.editable {
        // 閉じていない DocParam 種。値は出すが、触れる部品を置かない(Q0)。
        return property_row(
            band,
            row![
                kind_icon(glyph, band),
                text(label)
                    .size(10)
                    .style(crate::theme::style::text_primary_style),
                space().width(Fill),
                text(format_components(&param_row.components)).size(10),
            ]
            .spacing(7)
            .align_y(Center),
        );
    }
    let key = key_button(
        param_row.key_state,
        Message::Inspector(InspectorEvent::KeyPressed(param)),
    );
    if arity(param) == 2 {
        // header 行(icon + 名前 + key ボタン)。X / Y は独立キーに見せない。
        let header = property_row(
            band,
            row![
                kind_icon(glyph, band),
                text(label)
                    .size(10)
                    .font(weight(iced::font::Weight::Semibold)),
                space().width(Fill),
                key,
            ]
            .spacing(7)
            .align_y(Center),
        );
        let mut rows = column![header];
        for (component, axis) in ["X", "Y"].into_iter().enumerate() {
            let value = param_row.components.get(component).copied().unwrap_or(0.0);
            rows = rows.push(property_row(
                band,
                row![
                    // css:410 `.valueCell > b { color: var(--property-color) }`
                    // — X/Y の字は property-color を持つ小さなラベル。
                    text(axis).size(9).style(move |_theme: &iced::Theme| {
                        iced::widget::text::Style { color: Some(band) }
                    }),
                    space().width(Fill),
                    container(scrub(param, component, value, band)).width(dims::VALUE_COL_W),
                ]
                .padding(iced::padding::left(dims::KIND_ICON + 7.0 + 12.0))
                .spacing(6)
                .align_y(Center),
            ));
        }
        rows.into()
    } else {
        let value = param_row.components.first().copied().unwrap_or(0.0);
        property_row(
            band,
            row![
                kind_icon(glyph, band),
                text(label)
                    .size(10)
                    .font(weight(iced::font::Weight::Semibold)),
                space().width(Fill),
                container(scrub(param, 0, value, band)).width(dims::VALUE_COL_W),
                key,
            ]
            .spacing(7)
            .align_y(Center),
        )
    }
}

/// TRANSFORM host param の帯色。inspector-library.html:29-60 の写し。
fn row_band_color(param: UiEditParam) -> iced::Color {
    let t = &Tokens::DARK;
    match param {
        UiEditParam::Position => t.data,
        UiEditParam::Rotation => t.action_active,
        UiEditParam::Scale => t.shape,
        UiEditParam::Opacity => t.text_secondary,
    }
}

/// 1行を組む共通の chrome — 左の 3px 色帯 + `surface_panel` の地 +
/// 下罫線1px。inspector-library.css:282-294(`.propertyRow` / `::before`)。
/// 下罫線はこのレーンで見つかった欠落 — bg 用 style の border は width 0 の
/// まま(コメントの意図どおり別 widget で積む約束だったが積んでいなかった)。
fn property_row<'a>(
    band: iced::Color,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        row![
            row_band(dims::ROW_BAND_W, band),
            container(content.into())
                .padding([4, 8])
                .width(Fill)
                .style(row_panel_style),
        ],
        rule::horizontal(1).style(crate::theme::style::separator),
    ]
    .into()
}

fn row_panel_style(theme: &iced::Theme) -> container::Style {
    let t = Tokens::resolve(theme);
    container::Style {
        background: Some(t.surface_panel.into()),
        ..container::Style::default()
    }
}

/// 値行のスクラブ部品1枚。仕様(範囲・刻み)は param ごとにここで決める。
/// `accent` は行の property-color(inspector-library.css:391 `.valueCell.dragging`
/// の枠色 = `--property-color` — action-active 固定ではなく行ごとに違う)。
fn scrub<'a>(
    param: UiEditParam,
    component: usize,
    value: f64,
    accent: iced::Color,
) -> Element<'a, Message> {
    let spec = ScrubSpec {
        value,
        decimals: 3,
        // Opacity だけ 0..1 に閉じる(範囲外の要求は入口で塞ぐ — Q3)。
        min: matches!(param, UiEditParam::Opacity).then_some(0.0),
        max: matches!(param, UiEditParam::Opacity).then_some(1.0),
        // 正準座標(原点中央・高さ1.0)も不透明度も 0..1 桁の世界(egui 版
        // DragValue の speed 0.005 と同じ判断で、刻みは細かく)。
        step: 0.01,
        integer: false,
        accent,
    };
    scrub_value(spec, move |event| {
        Message::Inspector(InspectorEvent::Scrub {
            param,
            component,
            event,
        })
    })
}

/// 3. EFFECTS section。
fn effects<'a>(effects: Vec<EffectSection>) -> Element<'a, Message> {
    let mut section = column![section_heading(EFFECTS, None)];
    if effects.is_empty() {
        return section
            .push(
                container(
                    text(NO_EFFECTS)
                        .size(11)
                        .style(crate::theme::style::text_secondary),
                )
                .padding([6, 10]),
            )
            .into();
    }
    for effect in effects {
        let EffectSection {
            definition_id,
            plugin_id,
            enabled,
            params,
        } = effect;
        section = section.push(effect_header(definition_id, plugin_id, enabled));
        for param in params {
            section = section.push(effect_param_row(param));
        }
    }
    section.into()
}

/// FX 1本の header 行 — `FX` badge + 名前(uppercase)+ ON/OFF pill。
/// inspector-library.css:199-220。effect ごとの色は read-model に無いので、
/// css の既定値どおり `way_plugins` を**全 FX 共通**で使う
/// (`--effect-color` 未設定時の CSS の既定と同じ選択。発明ではない)。
fn effect_header<'a>(definition_id: u64, plugin_id: String, enabled: bool) -> Element<'a, Message> {
    let t = &Tokens::DARK;
    let accent = t.way_plugins;
    let badge = container(
        text("FX")
            .size(7)
            .font(weight(iced::font::Weight::ExtraBold))
            .style(move |_theme: &iced::Theme| iced::widget::text::Style {
                color: Some(accent),
            }),
    )
    .width(dims::FX_BADGE_W)
    .height(dims::FX_BADGE_H)
    .center_x(dims::FX_BADGE_W)
    .center_y(dims::FX_BADGE_H)
    .style(move |_theme: &iced::Theme| container::Style {
        background: Some(iced::Color { a: 0.18, ..accent }.into()),
        border: iced::Border {
            color: accent,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    });

    let toggle = button(
        text(if enabled { "ON" } else { "OFF" })
            .size(7)
            .font(weight(iced::font::Weight::ExtraBold)),
    )
    .padding([0, 4])
    .width(dims::FX_PILL_MIN_W)
    .height(dims::FX_PILL_H)
    .style(move |_theme: &iced::Theme, _status| fx_pill_style(accent, enabled))
    .on_press(Message::Inspector(InspectorEvent::SetEffectEnabled {
        definition_id,
        enabled: !enabled,
    }));

    let content = row![
        badge,
        text(plugin_id.to_uppercase())
            .size(8)
            .font(weight(iced::font::Weight::ExtraBold)),
        space().width(Fill),
        toggle,
    ]
    .spacing(7)
    .align_y(Center);

    row![
        row_band(dims::ROW_BAND_W, accent),
        container(content)
            .padding([0, 8])
            .width(Fill)
            .height(dims::SECTION_H)
            .align_y(Center)
            .style(move |_theme: &iced::Theme| container::Style {
                background: Some(iced::Color { a: 0.10, ..accent }.into()),
                ..container::Style::default()
            }),
    ]
    .into()
}

/// ON/OFF pill の面。inspector-library.css:217-220。**角丸0**
/// (`.effectEnable` に border-radius は無い — 名前は "pill" だが実体は
/// 四角。UI視覚言語のpill化禁止に沿う)。
fn fx_pill_style(accent: iced::Color, enabled: bool) -> button::Style {
    let t = &Tokens::DARK;
    if enabled {
        button::Style {
            background: Some(iced::Color { a: 0.16, ..accent }.into()),
            text_color: accent,
            border: iced::Border {
                color: iced::Color { a: 0.72, ..accent },
                width: 1.0,
                radius: 0.0.into(),
            },
            ..button::Style::default()
        }
    } else {
        button::Style {
            background: Some(iced::Color::TRANSPARENT.into()),
            text_color: t.text_muted,
            border: iced::Border {
                color: t.border_default,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..button::Style::default()
        }
    }
}

/// FX param 1行。kind→色 の割当は inspector-library.css:297-302
/// (`data-param-kind`)を `EffectParamValue` の閉じた語彙へ写す:
/// F64=scalar→data、Vec2/Vec3=vector→way_inspector、Color はその値自身を
/// 帯にする(新色の発明ではなく、param が既に持っている値)。
fn effect_param_row<'a>(param: crate::inspector_model::EffectParamRow) -> Element<'a, Message> {
    let t = &Tokens::DARK;
    let band = match param.value {
        EffectParamValue::F64(_) => t.data,
        EffectParamValue::Vec2(_) | EffectParamValue::Vec3(_) => t.way_inspector,
        EffectParamValue::Color([r, g, b, a]) => iced::Color {
            r: r as f32,
            g: g as f32,
            b: b as f32,
            a: a as f32,
        },
    };
    property_row(
        band,
        row![
            text(param.id)
                .size(9)
                .style(crate::theme::style::text_primary_style),
            space().width(Fill),
            text(format_effect_value(param.value))
                .size(9)
                .font(iced::Font::MONOSPACE),
        ]
        .padding(iced::padding::left(12))
        .spacing(6)
        .align_y(Center),
    )
}

/// footer のヒント行。inspector-library.css:519 相当(footer)。
fn footer_hint<'a>() -> Element<'a, Message> {
    container(text(HINT).size(9).style(crate::theme::style::text_muted))
        .padding([5, 9])
        .width(Fill)
        .style(section_band_style)
        .into()
}

fn format_components(components: &[f64]) -> String {
    components
        .iter()
        .map(|value| format!("{value:.3}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// FX param の値の見せ方。Color は swatch の代わりに 16進(独自色 UI を発明しない)。
fn format_effect_value(value: EffectParamValue) -> String {
    match value {
        EffectParamValue::F64(v) => format!("{v:.3}"),
        EffectParamValue::Vec2([x, y]) => format!("{x:.3}, {y:.3}"),
        EffectParamValue::Vec3([x, y, z]) => format!("{x:.3}, {y:.3}, {z:.3}"),
        EffectParamValue::Color([r, g, b, _]) => format!(
            "#{:02X}{:02X}{:02X}",
            (r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (b.clamp(0.0, 1.0) * 255.0).round() as u8
        ),
    }
}

/// pane 全体の面 — `surface_panel`(inspector-library.css:26)。
fn panel_style(theme: &iced::Theme) -> container::Style {
    let t = Tokens::resolve(theme);
    container::Style {
        background: Some(t.surface_panel.into()),
        border: iced::Border {
            color: t.border_default,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}
