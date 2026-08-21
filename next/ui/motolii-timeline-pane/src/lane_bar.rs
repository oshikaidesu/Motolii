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

use super::projection::{layer_row_at_y, layer_row_top, RowProjection};
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

// ---------------------------------------------------------------------------
// 名前の切り詰め(裁定168 施工・違反(A)の根治: 長いレイヤー名が M/S/L チップへ
// 素通しで重なる)。
// ---------------------------------------------------------------------------

/// 兄弟要素間の gap(裁定167 の梯子下段: `0.075 × 行高`、px 最近傍丸め)。
/// `inspector_pane.rs::sibling_gap_px` と同型(値そのものは pane ごとに token
/// 経由で別々に持つ、式だけ揃える — Timeline と Inspector は別 crate なので
/// 共有 fn は置けない)。ここでは名前列とチップ群の間の緩衝に使う。
fn sibling_gap_px(row_height: f32) -> f32 {
    (row_height * 0.075).round()
}

/// 全角相当(CJK 統合漢字・ひらがな・カタカナ・ハングル等)の判定。
/// [`default_measure`] の等幅近似だけに使う簡易版 — 東アジア文字幅の一般実装
/// ではなく、レーン名の切り詰めに要る最小限のブロックだけを対象にする。
fn is_wide_char(ch: char) -> bool {
    matches!(ch as u32,
        0x1100..=0x115F   // ハングル字母
        | 0x2E80..=0xA4CF // CJK 部首・記号・かな・カタカナ・ハングル音節手前まで
        | 0xAC00..=0xD7A3 // ハングル音節
        | 0xF900..=0xFAFF // CJK 互換漢字
        | 0xFF00..=0xFF60 // 全角記号
        | 0xFFE0..=0xFFE6
        | 0x20000..=0x3FFFD // CJK 拡張(サロゲート外)
    )
}

/// 等幅近似の文字幅測定器(既定の測定クロージャ)。**近似であることを明記** —
/// `canvas::Text` は Simulator から実測できない(KNOWN.md 実測、
/// `inspector_pane.rs` crate doc の同じ理由)ため、実描画側は決定論的な近似
/// (半角=0.6em/文字・全角相当=1.0em/文字、`em` は呼び出し側が渡すフォント
/// サイズ)で切り詰め幅を見積もる。実グリフ幅の代わりであって、正確な測定
/// ではない。
pub(crate) fn default_measure(em: f32) -> impl Fn(&str) -> f32 {
    move |s: &str| {
        s.chars()
            .map(|ch| if is_wide_char(ch) { em } else { em * 0.6 })
            .sum()
    }
}

/// `name` を `max_width` 以内へ切り詰める純関数(EXACT TARGET 1)。全体が
/// `max_width` に収まればそのまま返す。収まらなければ末尾から1文字ずつ削って
/// 「…」を付けた候補を試し、最初に収まった物を返す。1文字も入らなければ
/// 空文字列(呼び出し側は何も描かない判断になる)。
///
/// `measure` は幅測定クロージャ — 実描画側は [`default_measure`] を渡すが、
/// 決定論的なテストは既知の測定器を注入できる(fixture 前提の柵にするため)。
pub(crate) fn truncate_to_width(
    name: &str,
    max_width: f32,
    measure: impl Fn(&str) -> f32,
) -> String {
    if max_width <= 0.0 {
        return String::new();
    }
    if measure(name) <= max_width {
        return name.to_owned();
    }
    let chars: Vec<char> = name.chars().collect();
    for len in (0..chars.len()).rev() {
        let candidate: String = chars[..len].iter().collect::<String>() + "…";
        if measure(&candidate) <= max_width {
            return candidate;
        }
    }
    String::new()
}

/// 名前列の実効幅 — rail 幅から M/S/L チップ群と裁定167 の緩衝 gap を差し引いた
/// 残り(EXACT TARGET 1: `fill_text` 前にこの幅で [`truncate_to_width`] へ渡す)。
/// スウォッチ+左右余白ぶんの開始 x は `draw()` の名前描画位置(`dims.spacing_s
/// * 2.0 + swatch_size`)と同じ式 — 2箇所で別の値を発明しない。
fn name_column_width(dims: &Dimensions, rail_width: f32, row_height: f32) -> f32 {
    let swatch_size = dims.spacing_m;
    let name_start_x = dims.spacing_s * 2.0 + swatch_size;
    let chip_start_x = glyph_slots(dims, rail_width)[0].x;
    let gap = sibling_gap_px(row_height);
    (chip_start_x - name_start_x - gap).max(0.0)
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
///
/// 行の縦位置は `super::projection::layer_row_top` が正本(T3b EXACT
/// TARGET 3) — `param_row_height`/`property_row_count`/`selected_index` を
/// 受けてその逆写像([`layer_row_at_y`])で行 index を求め、glyph の y 範囲も
/// 同じ [`layer_row_top`] から出す(`super::lane_bar::draw` と同じ式)。
#[allow(clippy::too_many_arguments)]
pub(crate) fn hit_test(
    point: Point,
    rows: &[RowProjection],
    ruler_height: f32,
    row_height: f32,
    rail_width: f32,
    dims: &Dimensions,
    param_row_height: f32,
    property_row_count: usize,
    selected_index: Option<usize>,
) -> Option<Hit> {
    if point.x < 0.0 || point.x >= rail_width || point.y < ruler_height || row_height <= 0.0 {
        return None;
    }
    let row_index = layer_row_at_y(
        point.y - ruler_height,
        row_height,
        param_row_height,
        property_row_count,
        selected_index,
    )?;
    let row = rows.get(row_index)?;

    let row_top = ruler_height
        + layer_row_top(row_height, param_row_height, property_row_count, selected_index, row_index);
    let glyph_h = glyph_height(dims, row_height);
    let glyph_y0 = row_top + (row_height - glyph_h) / 2.0;
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

        // 名前(裁定147: クリップ面から退去した名前の住所)。**裁定168 施工**:
        // M/S/L チップへ素通しで重なっていた違反(A)の根治 — `fill_text` 前に
        // 実効幅で切り詰める(はみ出す名前は末尾「…」)。
        let name_color = if row.hidden { colors.text_muted } else { colors.text_primary };
        let name = if row.name.is_empty() {
            format!("layer {}", row.id.0)
        } else {
            row.name.clone()
        };
        let name_max_width = name_column_width(dims, rail_width, row_height);
        let name = truncate_to_width(&name, name_max_width, default_measure(dims.caption_text));
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 実窓較正で発覚した違反(A)の fixture 長名 — 全角(カタカナ)のみで、
    /// 等幅近似(全角=1.0em/文字)でも十分に列幅を超える。
    const LONG_CJK_NAME: &str = "グリッチトランジション";

    #[test]
    fn truncate_to_width_passes_a_short_name_through_unchanged() {
        let measure = default_measure(9.0);
        let name = "Layer 1";
        assert_eq!(truncate_to_width(name, 100.0, measure), "Layer 1");
    }

    #[test]
    fn truncate_to_width_shortens_a_long_cjk_name_with_an_ellipsis() {
        let em = 9.0;
        let measure = default_measure(em);
        let full_width: f32 = LONG_CJK_NAME.chars().count() as f32 * em; // 全角=1.0em
        assert!(full_width > 50.0, "fixture が列幅内に収まってしまう前提が崩れている");

        let truncated = truncate_to_width(LONG_CJK_NAME, 50.0, default_measure(em));
        assert!(truncated.ends_with('…'), "省略記号が付いていない: {truncated:?}");
        assert!(
            truncated.chars().count() < LONG_CJK_NAME.chars().count(),
            "切り詰められていない: {truncated:?}"
        );
        assert!(
            measure(&truncated) <= 50.0,
            "切り詰め後も列幅を超えている: {truncated:?}"
        );
    }

    #[test]
    fn truncate_to_width_returns_empty_when_not_even_the_ellipsis_fits() {
        let measure = default_measure(9.0);
        assert_eq!(truncate_to_width(LONG_CJK_NAME, 0.5, measure), "");
    }

    #[test]
    fn truncate_to_width_is_a_no_op_at_zero_or_negative_width() {
        let measure = default_measure(9.0);
        assert_eq!(truncate_to_width("x", 0.0, measure), "");
        let measure = default_measure(9.0);
        assert_eq!(truncate_to_width("x", -1.0, measure), "");
    }

    /// EXACT TARGET 1: 名前列の実効幅は rail 幅からチップ群+裁定167 の緩衝を
    /// 引いた残りで、既定 dims では常に正でチップ群より手前に収まる。
    #[test]
    fn name_column_width_is_positive_and_ends_before_the_chip_block_at_default_dims() {
        let dims = Dimensions::default();
        let rail_width = dims.timeline_lane_bar_width;
        let row_height = dims.row_height;

        let width = name_column_width(&dims, rail_width, row_height);
        assert!(width > 0.0, "既定 dims で名前列の実効幅が非正: {width}");

        let name_start_x = dims.spacing_s * 2.0 + dims.spacing_m;
        let chip_start_x = glyph_slots(&dims, rail_width)[0].x;
        assert!(
            name_start_x + width <= chip_start_x,
            "名前列がチップ群の開始位置を超えている(緩衝gapが効いていない): \
             name_start_x={name_start_x} width={width} chip_start_x={chip_start_x}"
        );
    }

    #[test]
    fn sibling_gap_px_matches_the_ladder_bottom_rung_rounded_to_the_nearest_pixel() {
        // 裁定167 下段: 0.075 × 行高、px 最近傍丸め。既定 row_height=20 では
        // 1.5 → 2.0(半丁は遠い方= away-from-zero 丸め、Rust の f32::round どおり)。
        assert_eq!(sibling_gap_px(20.0), 2.0);
        assert_eq!(sibling_gap_px(40.0), 3.0);
    }
}
