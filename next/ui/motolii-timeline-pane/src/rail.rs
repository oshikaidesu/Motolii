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
//!
//! ## ツリー行(裁定173 H2)
//! `RowProjection::depth`/`has_children`/`children_open`(`projection::rows`
//! 参照 — `attrs.parent` を辺として読んだ木の flatten)を rail 側で描く。
//! インデントは [`indent_step_px`]、開閉は [`fold_toggle`]。**朝の一瞥キュー**:
//! インデント幅の比率(裁定167 の余白梯子の頂点段 `0.30×行高`)はモックに
//! 親子行の実例が無いため実測ではなく宣言 — 実窓で親子行を見てから、梯子の
//! 他段(0.15/0.075)の方が近ければ [`indent_step_px`] を直す(段の中間値は
//! 発明しない、S4 段量子化柵と同型)。

use iced::widget::{button, column, container, mouse_area, row, text, text_input, Space};
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

    // **oracle「fold 既定=全展開で現行の見た目不変」の直接の実装**: 木が
    // 1つも無い Document(全 layer が `depth == 0 && !has_children`、
    // parent を1つも設定していない導入前の全 Document がこれに該当)では、
    // ツリーの前置き(インデント+fold 三角)を**1px も足さない** — 行ごとに
    // `depth`/`has_children` を見て可変長にすると、既存の全フラット
    // Document でさえ rail の swatch/名前/M・S・L が一律で右へずれてしまい
    // 「見た目不変」を破る。木が実在する時だけ、`layer_row` へ前置きの
    // 有無を渡す(この bool は行ごとではなく Document 全体で1つ — 兄弟間の
    // 縦の格子を崩さないため)。
    let has_any_tree = pane.rows.iter().any(|row| row.depth > 0 || row.has_children);

    // **単一源節の実装そのもの**: `super::projection::layer_row_top` と
    // 同じ順序・同じ挿入位置(選択 layer の直後)で積むだけ — 押し下げ量を
    // 計算する式はここには無い(iced のレイアウトが累計する)。
    for (index, proj) in pane.rows.iter().enumerate() {
        // inline rename 中の行だけ、名前 text を text_input へ差し替える
        // (第3切片、正典 §6 — `TimelinePane::with_rename` が運ぶ下書き)。
        let rename_draft = pane
            .rename
            .as_ref()
            .filter(|(layer, _)| *layer == proj.id)
            .map(|(_, draft)| draft.as_str());
        children.push(layer_row(proj, dims, colors, row_height, rail_width, has_any_tree, rename_draft));
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

/// ツリー行1段ぶんのインデント幅(裁定173 H2)。裁定167 の余白梯子
/// (`MARGIN_RATIO` 系列 `{0.30, 0.15, 0.075}×行高`)の頂点段をそのまま
/// 1段の量として転用する — **宣言であって実測ではない**(モックに親子行の
/// 実例が無い、module doc「朝の一瞥キュー」節参照)。px へは梯子の他の比率
/// 定数(`TARGET_CELL_RATIO` 等)と同じ「最近傍丸め」の作法を踏襲する。
/// fold 三角のヒット領域も同じ幅を使う(`fold_toggle` — インデント1段と
/// fold ボタンが同じ格子に揃う)。
fn indent_step_px(row_height: f32) -> f32 {
    (row_height * 0.30).round()
}

/// fold 三角(開閉ボタン)。`has_children` が `false` の行は同じ幅の空白を
/// 返す(旧世界 `timeline_rows.rs`「空の Group は矢印を出さない」規則の踏襲
/// — 矢印の有無に関わらずインデントの格子は揃う)。
fn fold_toggle(proj: &RowProjection, colors: Colors, size: f32) -> Element<'static, Message> {
    if !proj.has_children {
        return Space::new().width(Length::Fixed(size)).height(Length::Fixed(size)).into();
    }
    let id = proj.id;
    let glyph = if proj.children_open { "\u{25BE}" } else { "\u{25B8}" }; // ▾ / ▸
    button(
        text(glyph)
            .size(size)
            .color(colors.text_secondary)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fixed(size))
    .height(Length::Fixed(size))
    .padding(0)
    .on_press(Message::ToggleFold(id))
    .style(move |_theme, _status| button::Style {
        background: None,
        text_color: colors.text_secondary,
        ..button::Style::default()
    })
    .into()
}

/// 1層ぶんの rail 行(スウォッチ+名前+M/S/L)。行全体を `mouse_area` で包み
/// 選択(`Message::Select`)を発火する — M/S/L 自身のクリックは
/// `button.on_press` が先に capture するので、行選択と二重発火しない
/// (モジュール doc「gesture」節参照)。
///
/// `has_any_tree`(`view()` が Document 全体を1度見て決める、行ごとではない
/// bool)が `false` の時はツリーの前置き(インデント・fold 三角)を**1px も
/// 足さない** — oracle「fold 既定=全展開で現行の見た目不変」の実装本体
/// (`view()` の doc 参照)。
fn layer_row(
    proj: &RowProjection,
    dims: Dimensions,
    colors: Colors,
    row_height: f32,
    rail_width: f32,
    has_any_tree: bool,
    rename_draft: Option<&str>,
) -> Element<'static, Message> {
    let id = proj.id;
    let indent_step = indent_step_px(row_height);
    // ツリーの前置き幅(インデント + fold 三角の1段)。子の無い行も同じ幅の
    // 空白を出す(`fold_toggle` 参照)ので、深さが揃っていれば縦の格子は
    // 常に一致する。木が1つも無ければ幅0(`tree_prefix` 自体を積まない)。
    let tree_prefix_width = if has_any_tree { indent_step * (proj.depth as f32 + 1.0) } else { 0.0 };
    let tree_prefix: Option<Element<'static, Message>> = has_any_tree.then(|| {
        let indent: Element<'static, Message> = Space::new()
            .width(Length::Fixed(indent_step * proj.depth as f32))
            .height(Length::Fixed(row_height))
            .into();
        let fold = fold_toggle(proj, colors, indent_step);
        row([indent, fold]).into()
    });
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
    let name_max_width = (name_column_width(&dims, rail_width, row_height) - tree_prefix_width).max(0.0);
    // inline rename 中(正典 §6)は同じ幅の text_input へ差し替える —
    // 毎打鍵は `Message::RenameEdited`、Enter は `RenameCommit`(空名拒否・
    // 同名 no-op は `PaneState::commit_rename` の柵)。placeholder は静止表示と
    // 同じ `layer {id}` の顔。padding は縦0(inspector `ident_band` と同じ柵 —
    // 既定 padding が乗ると固定行高 26px の中で欄が縦に太る)。
    let name: Element<'static, Message> = match rename_draft {
        Some(draft) => text_input(format!("layer {}", id.0), draft.to_owned())
            .on_input(Message::RenameEdited)
            .on_submit(Message::RenameCommit)
            .size(dims.caption_text)
            .padding([0.0, dims.spacing_xs])
            .width(Length::Fixed(name_max_width))
            .into(),
        None => text(name_content)
            .size(dims.caption_text)
            .color(name_color)
            .width(Length::Fixed(name_max_width))
            .align_y(iced::alignment::Vertical::Center)
            .wrapping(iced::widget::text::Wrapping::None)
            .ellipsis(iced::widget::text::Ellipsis::End)
            .into(),
    };

    let glyphs: Element<'static, Message> = row([
        glyph_button(id, Glyph::Mute, proj, dims, colors),
        glyph_button(id, Glyph::Solo, proj, dims, colors),
        glyph_button(id, Glyph::Lock, proj, dims, colors),
    ])
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center)
    .into();

    // `tree_prefix` は木が実在する時だけ積む(`Vec` — `has_any_tree` で
    // 要素数が変わるため固定長配列の `row![]` は使えない)。木が無ければ
    // 導入前と要素数・幅ともに完全に同じ `Row` になる。
    let mut content_children: Vec<Element<'static, Message>> = Vec::with_capacity(6);
    if let Some(tree_prefix) = tree_prefix {
        content_children.push(tree_prefix);
    }
    content_children.push(swatch);
    content_children.push(Space::new().width(Length::Fixed(dims.spacing_s)).into());
    content_children.push(name);
    content_children.push(Space::new().width(Length::Fill).into());
    content_children.push(glyphs);

    let content = row(content_children).align_y(iced::alignment::Vertical::Center).padding([0.0, dims.spacing_s]);

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
/// 住所はレーンバー」を property 行にも延長)。
///
/// **第3切片で対話化**(map 509・正典 §8.1 SelectAllKeysOfProperty
/// 「property 名クリックでその property の全キー選択」): 行全体を `mouse_area`
/// で包み `Message::SelectAllKeysOfProperty` を発火する。カーソルは Pointer
/// (§5.5「カーソル形状は意味の予告」 — 押せる物には予告を出す、Q0)。
/// 旧 doc の「非対話」は §8.1 の正典項目で上書きされた(採用済みの意味を
/// rail 側の入口として結線した形)。
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

    let body = container(label)
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
        });

    mouse_area(body)
        .on_press(Message::SelectAllKeysOfProperty {
            layer: prow.layer,
            property: prow.property.clone(),
        })
        .interaction(iced::mouse::Interaction::Pointer)
        .into()
}
