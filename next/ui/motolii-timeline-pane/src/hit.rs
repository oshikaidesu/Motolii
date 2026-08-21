//! `Hit` 型と当たり判定(`hit_test`)。click/drag した先が bar か空白部かだけを
//! 判定する純粋な読み取り。bar の中のどこ(本体か端か)は [`BarPart`]/
//! [`classify_bar_part`] が別腕で持つ — move/trim ジェスチャ(第2波T2、正典
//! `next/reference/timeline-grammar.md` §1 EXACT TARGET 1)の発注元。
//!
//! **判定は押した瞬間の座標で行うこと**(呼び出し側の責任、正典 §1): ドラッグが
//! 報告される時点ではポインタは既に数 px 動いており、その座標で毎回判定し直すと
//! 「右端を掴んで左へ引く」が本体掴みに化ける(R1 M-2 の明記されたバグ修正)。
//! この2関数はどちらも1点の座標を受けるだけの純関数なので、呼び出し側
//! (`super::input`)が `ButtonPressed` の座標だけを渡せば自然にこの規則を守る。

use iced::Point;

use motolii_store::LayerId;

use super::projection::{frame_to_x, layer_row_at_y, RowProjection};

/// click/drag した先が何か。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// この layer の bar の上。
    Bar(LayerId),
    /// ルーラーまたは行の空白部(scrub の対象)。
    Blank,
}

/// bar の中のどこを掴んだか — 本体(move)か端(trim)か。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarPart {
    /// 本体。move の対象。
    Body,
    /// 頭(In)側の端。trim In の対象。
    EdgeIn,
    /// 尻(Out)側の端。trim Out の対象。
    EdgeOut,
}

/// bar 端の掴み幅(px)。正典 §1 `TRIM_EDGE` と同値。
pub const TRIM_EDGE: f32 = 8.0;

/// `point` がどこに当たったか。ルーラー帯(`ruler_height` 未満)は常に [`Hit::Blank`]。
/// 行の内側では bar の区間(`start..start+duration`)だけを [`Hit::Bar`] とし、
/// それ以外はその行の空白部として [`Hit::Blank`] を返す。
///
/// 行の縦位置は `super::projection::layer_row_top` が正本(T3b EXACT TARGET 3)
/// — ここは `param_row_height`/`property_row_count`/`selected_index` を受けて
/// その逆写像([`layer_row_at_y`])で行 index を求める。展開している property
/// 行の帯の内側(`layer_row_at_y` が `None` を返す y)は常に [`Hit::Blank`]
/// (その帯は `key_rows.rs` が先に自己完結で吸収するので、ここに来るのは
/// 安全側の振る舞いとして)。
#[allow(clippy::too_many_arguments)]
pub fn hit_test(
    point: Point,
    rows: &[RowProjection],
    ruler_height: f32,
    row_height: f32,
    width: f32,
    duration_frames: i64,
    param_row_height: f32,
    property_row_count: usize,
    selected_index: Option<usize>,
) -> Hit {
    if point.y < ruler_height || row_height <= 0.0 {
        return Hit::Blank;
    }
    let Some(row_index) = layer_row_at_y(
        point.y - ruler_height,
        row_height,
        param_row_height,
        property_row_count,
        selected_index,
    ) else {
        return Hit::Blank;
    };
    let Some(row) = rows.get(row_index) else {
        return Hit::Blank;
    };
    let (start_x, end_x) = bar_span_x(row, width, duration_frames);
    if point.x >= start_x && point.x < end_x {
        Hit::Bar(row.id)
    } else {
        Hit::Blank
    }
}

/// bar の x 範囲(クリップ面ローカル座標)。[`hit_test`] と [`classify_bar_part`] が
/// 同じ式(`frame_to_x`)から求める — 2箇所で別の換算式を持たない。
pub fn bar_span_x(row: &RowProjection, width: f32, duration_frames: i64) -> (f32, f32) {
    let start_x = frame_to_x(row.start, width, duration_frames);
    let end_x = frame_to_x(row.start + row.duration, width, duration_frames).max(start_x + 1.0);
    (start_x, end_x)
}

/// `point_x` が `[start_x, end_x)` の bar の中のどこに当たったか。**幅
/// `TRIM_EDGE * 3.0`(24px)未満の bar は端を出さない**(誤トリム防止、正典 §1
/// 「幅 24px 未満の bar は端を差し出さない」)。呼び出し側は押した瞬間の
/// `point_x` だけを渡すこと(モジュール doc 参照)。
pub fn classify_bar_part(point_x: f32, start_x: f32, end_x: f32) -> BarPart {
    if end_x - start_x < TRIM_EDGE * 3.0 {
        return BarPart::Body;
    }
    if point_x - start_x <= TRIM_EDGE {
        BarPart::EdgeIn
    } else if end_x - point_x <= TRIM_EDGE {
        BarPart::EdgeOut
    } else {
        BarPart::Body
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: u64, start: i64, duration: i64) -> RowProjection {
        RowProjection {
            id: LayerId(id),
            name: String::new(),
            hidden: false,
            solo: false,
            locked: false,
            label_color: None,
            start,
            duration,
            selected: false,
            dragging: false,
            depth: 0,
            has_children: false,
            children_open: true,
        }
    }

    /// 幅24px未満(細い bar)は端を出さない(誤トリム防止)。
    #[test]
    fn narrow_bars_never_offer_an_edge() {
        // duration_frames == width なので 1px == 1frame。幅20frame(<24)。
        let r = row(1, 0, 20);
        let (start_x, end_x) = bar_span_x(&r, 300.0, 300);
        assert_eq!(classify_bar_part(start_x, start_x, end_x), BarPart::Body);
        assert_eq!(classify_bar_part(end_x - 1.0, start_x, end_x), BarPart::Body);
    }

    /// 幅が十分な bar は端 `TRIM_EDGE` px を trim、それ以外は本体。
    #[test]
    fn wide_bars_offer_edges_within_trim_edge_px() {
        let r = row(1, 100, 100); // 幅100frame == 100px(1:1縮尺)。
        let (start_x, end_x) = bar_span_x(&r, 300.0, 300);
        assert_eq!(start_x, 100.0);
        assert_eq!(end_x, 200.0);

        assert_eq!(classify_bar_part(100.0, start_x, end_x), BarPart::EdgeIn);
        assert_eq!(classify_bar_part(108.0, start_x, end_x), BarPart::EdgeIn);
        assert_eq!(classify_bar_part(109.0, start_x, end_x), BarPart::Body);
        assert_eq!(classify_bar_part(150.0, start_x, end_x), BarPart::Body);
        assert_eq!(classify_bar_part(191.0, start_x, end_x), BarPart::Body);
        assert_eq!(classify_bar_part(192.0, start_x, end_x), BarPart::EdgeOut);
        assert_eq!(classify_bar_part(199.0, start_x, end_x), BarPart::EdgeOut);
    }

    #[test]
    fn hit_test_still_reports_only_the_layer_on_bar_hits() {
        let rows = vec![row(1, 100, 50)];
        assert_eq!(
            hit_test(Point::new(120.0, 25.0), &rows, 20.0, 20.0, 300.0, 300, 0.0, 0, None),
            Hit::Bar(LayerId(1)),
        );
        assert_eq!(
            hit_test(Point::new(10.0, 25.0), &rows, 20.0, 20.0, 300.0, 300, 0.0, 0, None),
            Hit::Blank,
        );
    }

    /// T3b EXACT TARGET/ORACLE: property 行(キー行)が展開されている間、その
    /// **下の** layer への bar クリックは押し下げぶんズレずに正しい layer へ
    /// 当たる(`layer_row_top` と `layer_row_at_y` が一致する、絵と当たりの統合)。
    #[test]
    fn hit_test_accounts_for_expanded_property_row_band() {
        // layer 0(選択・property 行2本展開)、layer 1(その下)。
        let rows = vec![row(0, 0, 10), row(1, 100, 50)];
        let row_height = 20.0;
        let param_row_height = 10.0;
        let property_row_count = 2;
        let selected_index = Some(0);

        // layer 1 の bar は展開後、ruler(20) + layer0行(20) + 帯(10*2=20) = 60 から。
        let hit = hit_test(
            Point::new(120.0, 65.0),
            &rows,
            20.0,
            row_height,
            300.0,
            300,
            param_row_height,
            property_row_count,
            selected_index,
        );
        assert_eq!(hit, Hit::Bar(LayerId(1)));

        // 帯の内側(相対 y=25 = ruler(20)+選択行(20)以降・帯[20,40)内)は
        // 層行ではない — Blank。
        let inside_band = hit_test(
            Point::new(120.0, 45.0),
            &rows,
            20.0,
            row_height,
            300.0,
            300,
            param_row_height,
            property_row_count,
            selected_index,
        );
        assert_eq!(inside_band, Hit::Blank);
    }
}
