//! レーンバー(行ヘッダ列) — スウォッチ・レイヤー名・M/S/L トグル。
//!
//! 正典 §1.5「面の構成」(裁定147)の**行の恒久 ID 面**: 選択・M/S/L・(将来の)
//! 並べ替え・リネーム・右クリックの起点。クリップ面(`super::canvas`)から
//! 退去したレイヤー名の住所もここ(裁定147「名前をクリップに描かない理由」)。
//!
//! `super::canvas`/`super::hit` と同じ役割分担 — このファイルが自分のゾーン
//! (x < rail_width)の draw と hit を両方持つ。クリップ面の当たり判定
//! (`super::hit::hit_test`)は一切触らない(呼び出し側の `super::input` が
//! 「まずレーンバー、当たらなければクリップ面」の順で振り分ける)。
//!
//! 色は Document に色ラベルが無いので発明しない — スウォッチは既存の
//! `way_timeline`(Timeline 全体のアクセント)を既定1色として使い回す。
//! M/S/L glyph の幅は新トークンを増やさず Inspector の `inspector_glyph_width`
//! (mock `--hit`)を使い回す — 同じ意味段(Key/M/S 列)の値を2箇所で発明しない。

use iced::widget::canvas;
use iced::{Point, Size};

use motolii_store::LayerId;

use crate::tokens::Dimensions;

use super::projection::RowProjection;
use super::TimelinePane;

/// M/S/L のどれか。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    Mute,
    Solo,
    Lock,
}

/// レーンバー内で click が当たった先。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// スウォッチ・名前の上(glyph 以外の行の地) — 選択の起点(裁定147)。
    Row(LayerId),
    /// M/S/L のどれか。
    Glyph(LayerId, Glyph),
}

/// glyph 1個ぶんの x 位置。
struct GlyphSlot {
    glyph: Glyph,
    x: f32,
}

/// 3 glyph の x 位置(右詰め、mock `.thead .sp{margin-left:auto}` と同じ並び:
/// 左から M/S/L)。rail の右端から `spacing_s` 空けて3個並べる。
fn glyph_slots(dims: &Dimensions, rail_width: f32) -> [GlyphSlot; 3] {
    let glyph_w = dims.inspector_glyph_width;
    let gap = dims.spacing_xs;
    let block_w = glyph_w * 3.0 + gap * 2.0;
    let start_x = rail_width - dims.spacing_s - block_w;
    [
        GlyphSlot { glyph: Glyph::Mute, x: start_x },
        GlyphSlot { glyph: Glyph::Solo, x: start_x + glyph_w + gap },
        GlyphSlot { glyph: Glyph::Lock, x: start_x + (glyph_w + gap) * 2.0 },
    ]
}

/// glyph の縦高さ。mock `.glyph{height:calc(var(--row) - 2*var(--s)*1px)}` と
/// 同じ式(`inspector_pane.rs::glyph_height` と同型 — Inspector と Timeline は
/// 別 pane なので token 経由の式を2箇所に持つ、値そのものは複製しない)。
fn glyph_height(dims: &Dimensions, row_height: f32) -> f32 {
    (row_height - dims.spacing_xs).max(1.0)
}

fn glyph_label(glyph: Glyph) -> &'static str {
    match glyph {
        Glyph::Mute => "M",
        Glyph::Solo => "S",
        Glyph::Lock => "L",
    }
}

/// `point` がレーンバー内(`0 <= x < rail_width`)のどこに当たったか。
/// レーンバーの外は `None` — 呼び出し側(`super::input`)がクリップ面の
/// `super::hit::hit_test` へ回す。
pub(crate) fn hit_test(
    point: Point,
    rows: &[RowProjection],
    ruler_height: f32,
    row_height: f32,
    rail_width: f32,
    dims: &Dimensions,
) -> Option<Hit> {
    if point.x < 0.0 || point.x >= rail_width || point.y < ruler_height || row_height <= 0.0 {
        return None;
    }
    let row_index = ((point.y - ruler_height) / row_height).floor();
    if row_index < 0.0 {
        return None;
    }
    let row = rows.get(row_index as usize)?;

    let glyph_h = glyph_height(dims, row_height);
    let glyph_y0 = ruler_height + row_height * row_index + (row_height - glyph_h) / 2.0;
    let glyph_y1 = glyph_y0 + glyph_h;
    if point.y >= glyph_y0 && point.y < glyph_y1 {
        for slot in glyph_slots(dims, rail_width) {
            if point.x >= slot.x && point.x < slot.x + dims.inspector_glyph_width {
                return Some(Hit::Glyph(row.id, slot.glyph));
            }
        }
    }
    Some(Hit::Row(row.id))
}

/// レーンバーを描く。`super::canvas::draw` と同じ `Frame` へ重ねて描く —
/// canvas widget は1トレイトにつき1 Program(`mod.rs` の trait 制約、モジュール
/// doc 参照)なので、別 canvas を新設するのではなく同じ paint pass に相乗りする。
///
/// **地(背景)は描き直さない** — canvas 全体の初期 fill(`surface_panel`、
/// `super::canvas::draw` 冒頭)が rail の地をそのまま兼ねる(mock
/// `.thead{background:panel}` と同色)。ゼブラ(裁定148)・行区切り hairline・
/// 選択ハイライトも同じ理由で `super::canvas::draw` 側が既に全幅(rail 込み)
/// で描いている — ここは rail 固有の中身(境界線・スウォッチ・名前・M/S/L)
/// だけを描く。
pub(crate) fn draw(pane: &TimelinePane, frame: &mut canvas::Frame, rail_width: f32) {
    let dims = &pane.dims;
    let colors = &pane.colors;
    let row_height = dims.row_height;
    let ruler_height = pane.ruler_height();
    let hairline = dims.border_width;

    // rail とクリップ面の境界(強い hairline、EXACT TARGET 5・裁定147)。
    let boundary = canvas::Path::line(
        Point::new(rail_width, 0.0),
        Point::new(rail_width, pane.content_height()),
    );
    frame.stroke(
        &boundary,
        canvas::Stroke::default()
            .with_color(colors.border_default)
            .with_width(hairline),
    );

    for (index, row) in pane.rows.iter().enumerate() {
        // 選択 layer の下に property 行(キー行、第2波 T3)が挿入されている間、
        // 後続の層行は押し下がる — `canvas.rs` の層行ループと同じ
        // `TimelinePane::layer_row_top` を使い、クリップ面の bar と rail の
        // 名前/M・S・L が揃った位置で描かれるようにする(`hit_test` 自身は
        // この押し下げを知らないまま — `projection::layer_row_top` の doc の
        // write-set 外 finding 参照。ここは draw だけの最小修正)。
        let row_top = ruler_height + pane.layer_row_top(index);

        // スウォッチ — Document に色ラベルが無いので `way_timeline`(Timeline
        // 全体のアクセント)を既定1色として使い回す(発明しない)。
        let swatch_size = dims.spacing_m;
        let swatch_y = row_top + (row_height - swatch_size) / 2.0;
        frame.fill_rectangle(
            Point::new(dims.spacing_s, swatch_y),
            Size::new(swatch_size, swatch_size),
            colors.way_timeline,
        );

        // 名前(裁定147: クリップ面から退去した名前の住所)。
        let name_color = if row.hidden { colors.text_muted } else { colors.text_primary };
        let name = if row.name.is_empty() {
            format!("layer {}", row.id.0)
        } else {
            row.name.clone()
        };
        frame.fill_text(canvas::Text {
            content: name,
            position: Point::new(dims.spacing_s * 2.0 + swatch_size, row_top + row_height / 2.0),
            color: name_color,
            size: iced::Pixels(dims.caption_text),
            align_y: iced::alignment::Vertical::Center,
            ..Default::default()
        });

        // M/S/L — 状態は Document(RowProjection)から読む(ボタンに状態を
        // 持たない、正典 §6)。
        let glyph_h = glyph_height(dims, row_height);
        let glyph_y = row_top + (row_height - glyph_h) / 2.0;
        for slot in glyph_slots(dims, rail_width) {
            let active = match slot.glyph {
                Glyph::Mute => row.hidden,
                Glyph::Solo => row.solo,
                Glyph::Lock => row.locked,
            };
            let (border_color, text_color) = if active {
                (colors.action_active, colors.action_active)
            } else {
                (colors.border_default, colors.text_secondary)
            };
            frame.stroke(
                &canvas::Path::rectangle(
                    Point::new(slot.x, glyph_y),
                    Size::new(dims.inspector_glyph_width, glyph_h),
                ),
                canvas::Stroke::default().with_color(border_color).with_width(hairline),
            );
            frame.fill_text(canvas::Text {
                content: glyph_label(slot.glyph).to_owned(),
                position: Point::new(slot.x + dims.inspector_glyph_width / 2.0, glyph_y + glyph_h / 2.0),
                color: text_color,
                size: iced::Pixels(dims.caption_text),
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Center,
                ..Default::default()
            });
        }
    }
}
