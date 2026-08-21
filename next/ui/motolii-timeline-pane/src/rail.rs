//! Rail(行ヘッダ列)の実 widget 化(TL-arch Phase 1、
//! `docs/reviews/2026-08-22-timeline-canvas-widget-survey.md` §6・利用者採択
//! 「推奨で行く」)。
//!
//! ## 構成(発注書 OUTCOME)
//! 行 = container 列・スウォッチ = 着色 `container`(裁定172 §2 の比率関数を
//! `super::lane_bar` からそのまま借用)・名前 = 実 `text()`(iced native の
//! 省略記号 — `super::lane_bar` モジュール doc の §2.5 節が根拠: `canvas::Text`
//! は `ellipsis` フィールドを無視するハードコードされたバグがあり、実 widget
//! の `text()` 経路だけが cosmic-text で正しく効かせる)・M/S/L = 実
//! `button()`。時間場(bar・ルーラー・菱形)は `super::canvas`/`super::key_rows`
//! のまま(Phase 2 の範囲、NON-GOALS)。
//!
//! ## 単一源(縦整列、発注書「複製コピー禁止」)
//! `super::canvas::draw` は各層行の描画 top を
//! `ruler_height + TimelinePane::layer_row_top(index)`(`projection::
//! layer_row_top` — 選択 layer の下の property 帯ぶんだけ後続行を押し下げる
//! 式)から出す。この rail 列は**その式を計算し直さない** — 代わりに canvas
//! と全く同じ順序(`rows` を先頭から辿り、選択 layer の直後に
//! `property_rows` を丸ごと挿入する、下の [`view`] のループそのものが
//! `layer_row_top` の定義: `base = row_height * index; index > selected なら
//! property 帯ぶん加算` と同型)で widget を `Column` へ積む。`Column` の
//! 縦積みは iced のレイアウトエンジンが「直前までの子の高さの累計」から
//! 自動的に計算する — 2つの式を手で同期させる必要が構造的に無い(widget の
//! 積み上げそのものが単一の正本)。`Column::with_children`(`column()` 関数)
//! の既定 `spacing` は 0.0(upstream 実測、`widget/src/column.rs`)— これを
//! 明示的に崩さないことがこの一致の前提(行間に余白を挟むとズレる)。
//! 各行の高さは `dims.row_height`/`dims.timeline_param_row_height` という
//! `canvas.rs`/`key_rows.rs` と同じトークンから `Length::Fixed` で与える
//! (raw px を発明しない)。
//!
//! 行の区切り線(hairline)・rail/clip 境界・選択ハイライトは `container` の
//! `border`/`background`(border-box — 既存 bounds を侵さない、
//! `inspector_pane.rs::bordered_row` と同じ流儀)で表現する。**追加の
//! `Space` 要素で区切りを表現しない** — それだと行ごとに実レイアウト高さが
//! 増え、上の「単一源」の前提(累計高さの一致)が崩れる。
//!
//! ## gesture
//! 行選択(旧 `lane_bar::Hit::Row`)は行全体を包む `mouse_area.on_press`。
//! M/S/L(旧 `lane_bar::Hit::Glyph`)は `button.on_press` — `mouse_area` は
//! 子の `update()` を先に呼び、子が `capture_event()` すればそこで終わる
//! (iced 実測、`widget/src/mouse_area.rs::Widget::update`)ので、M/S/L
//! ボタンを押した時に行選択が二重発火することはない(旧 canvas 版の
//! 「Glyph hit は Row hit より優先」という優先順位を、iced 自身の event
//! capture 規則がそのまま再現する — 新しい調停ロジックを書く必要が無い)。
//! 時間場側の gesture(move/trim・scrub・キー選択)は `super::input`/
//! `super::key_rows` に無改修のまま残る。

use iced::widget::{button, column, container, mouse_area, row, text, Space};
use iced::{Background, Border, Element, Length};

use motolii_store::LayerId;

use super::lane_bar::{
    glyph_label, glyph_size_px, glyph_text_size_px, name_column_width, swatch_color,
    swatch_radius_px, swatch_size_px, Glyph,
};
use super::projection::{PropertyRowProjection, RowProjection};
use super::TimelinePane;
use crate::tokens::{Colors, Dimensions};
use crate::Message;

/// rail 列。`TimelinePane::view` から `row![rail::view(&pane), canvas]` の
/// 左腕として呼ばれる(`&pane` を借用するだけ — `pane` はこの後 `canvas(self)`
/// へ move される、呼び出し側の doc 参照)。
pub(crate) fn view(pane: &TimelinePane) -> Element<'static, Message> {
    let dims = pane.dims;
    let colors = pane.colors;
    let rail_width = pane.rail_width();
    let row_height = dims.row_height;
    let ruler_height = pane.ruler_height();
    let param_row_height = pane.param_row_height();
    let total_height = pane.content_height().max(ruler_height);

    let mut children: Vec<Element<'static, Message>> =
        Vec::with_capacity(pane.rows.len() + pane.property_rows.len() + 1);
    children.push(corner(dims, colors, ruler_height));

    // **単一源節の実装そのもの**: `super::projection::layer_row_top` と
    // 同じ順序・同じ挿入位置(選択 layer の直後)で積むだけ — 押し下げ量を
    // 計算する式はここには無い(iced のレイアウトが累計する)。
    for (index, proj) in pane.rows.iter().enumerate() {
        children.push(layer_row(proj, dims, colors, row_height, rail_width));
        if pane.selected_row_index == Some(index) {
            for (band_index, prow) in pane.property_rows.iter().enumerate() {
                children.push(property_row(prow, dims, colors, param_row_height, band_index));
            }
        }
    }

    container(column(children).width(Length::Fixed(rail_width)))
        .width(Length::Fixed(rail_width))
        .height(Length::Fixed(total_height))
        .style(move |_theme| container::Style {
            background: Some(Background::Color(colors.surface_panel)),
            border: Border {
                color: colors.border_default,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// mock `.corner`(rail 上端、ルーラーと同じ高さ)。地のまま(スウォッチ・
/// 名前・M/S/L は無い)— 下端の border が rail/クリップ面のルーラー境界
/// (`super::canvas::draw` の `draw_hairline`)と同じ役目を rail 側で担う。
fn corner(dims: Dimensions, colors: Colors, ruler_height: f32) -> Element<'static, Message> {
    container(Space::new().width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fixed(ruler_height))
        .style(move |_theme| container::Style {
            border: Border {
                color: colors.border_default,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// 1層ぶんの rail 行(スウォッチ+名前+M/S/L)。行全体を `mouse_area` で包み
/// 選択(`Message::Select`)を発火する — M/S/L 自身のクリックは
/// `button.on_press` が先に capture するので、行選択と二重発火しない
/// (モジュール doc「gesture」節参照)。
fn layer_row(
    proj: &RowProjection,
    dims: Dimensions,
    colors: Colors,
    row_height: f32,
    rail_width: f32,
) -> Element<'static, Message> {
    let id = proj.id;
    let swatch_size = swatch_size_px(row_height);
    let swatch_radius = swatch_radius_px(swatch_size);
    let swatch_fill = swatch_color(proj, &colors);

    let swatch: Element<'static, Message> = container(
        Space::new()
            .width(Length::Fixed(swatch_size))
            .height(Length::Fixed(swatch_size)),
    )
    .width(Length::Fixed(swatch_size))
    .height(Length::Fixed(swatch_size))
    .style(move |_theme| container::Style {
        background: Some(Background::Color(swatch_fill)),
        border: Border { radius: swatch_radius.into(), ..Border::default() },
        ..container::Style::default()
    })
    .into();

    let name_color = if proj.hidden { colors.text_muted } else { colors.text_primary };
    let name_content = if proj.name.is_empty() {
        format!("layer {}", id.0)
    } else {
        proj.name.clone()
    };
    let name_max_width = name_column_width(&dims, rail_width, row_height);
    let name: Element<'static, Message> = text(name_content)
        .size(dims.caption_text)
        .color(name_color)
        .width(Length::Fixed(name_max_width))
        .align_y(iced::alignment::Vertical::Center)
        .wrapping(iced::widget::text::Wrapping::None)
        .ellipsis(iced::widget::text::Ellipsis::End)
        .into();

    let glyphs: Element<'static, Message> = row([
        glyph_button(id, Glyph::Mute, proj, dims, colors),
        glyph_button(id, Glyph::Solo, proj, dims, colors),
        glyph_button(id, Glyph::Lock, proj, dims, colors),
    ])
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center)
    .into();

    let content = row([
        swatch,
        Space::new().width(Length::Fixed(dims.spacing_s)).into(),
        name,
        Space::new().width(Length::Fill).into(),
        glyphs,
    ])
    .align_y(iced::alignment::Vertical::Center)
    .padding([0.0, dims.spacing_s]);

    let selected = proj.selected;
    let row_container = container(content)
        .width(Length::Fill)
        .height(Length::Fixed(row_height))
        .align_y(iced::alignment::Vertical::Center)
        .style(move |_theme| container::Style {
            background: if selected {
                Some(Background::Color(colors.state_selected))
            } else {
                None
            },
            border: Border {
                color: colors.border_hairline_weak,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });

    mouse_area(row_container).on_press(Message::Select(id)).into()
}

/// M/S/L のどれか1個。旧 canvas 手描き(旧 `lane_bar.rs::draw`)の状態→色の
/// 対応をそのまま踏襲: 状態は Document(`RowProjection`)から読むだけで、
/// ボタン自身は状態を持たない(正典 §6)。
fn glyph_button(
    id: LayerId,
    glyph: Glyph,
    proj: &RowProjection,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let glyph_size = glyph_size_px(dims.row_height);
    let glyph_text_size = glyph_text_size_px(glyph_size);
    let active = match glyph {
        Glyph::Mute => proj.hidden,
        Glyph::Solo => proj.solo,
        Glyph::Lock => proj.locked,
    };
    let text_color = if active { colors.action_active } else { colors.text_secondary };
    let message = match glyph {
        Glyph::Mute => Message::ToggleMute(id),
        Glyph::Solo => Message::ToggleSolo(id),
        Glyph::Lock => Message::ToggleLock(id),
    };

    button(
        text(glyph_label(glyph))
            .size(glyph_text_size)
            .color(text_color)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fixed(glyph_size))
    .height(Length::Fixed(glyph_size))
    .padding(0)
    .on_press(message)
    .style(move |_theme, _status| button::Style {
        background: Some(Background::Color(colors.surface_hover)),
        text_color,
        ..button::Style::default()
    })
    .into()
}

/// property 行(キー行)の rail 側ラベル — property 名だけ(裁定147「名前の
/// 住所はレーンバー」を property 行にも延長)。**非対話**(菱形/bar と違い
/// property 名自体はクリック動詞を持たない、旧 `key_rows.rs::update` の
/// 「rail 側は吸収だけ」踏襲)なので `mouse_area`/`button` は要らない。
fn property_row(
    prow: &PropertyRowProjection,
    dims: Dimensions,
    colors: Colors,
    param_row_height: f32,
    band_index: usize,
) -> Element<'static, Message> {
    let zebra = band_index % 2 == 1;
    let label = text(prow.property.name().to_owned())
        .size(dims.caption_text)
        .color(colors.text_secondary)
        .align_y(iced::alignment::Vertical::Center);

    container(label)
        .width(Length::Fill)
        .height(Length::Fixed(param_row_height))
        .align_y(iced::alignment::Vertical::Center)
        .padding(iced::padding::left(dims.spacing_l * 2.0))
        .style(move |_theme| container::Style {
            background: if zebra {
                Some(Background::Color(colors.timeline_row_zebra))
            } else {
                None
            },
            border: Border {
                color: colors.border_hairline_weak,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}
