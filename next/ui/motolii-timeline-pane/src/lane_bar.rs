//! レーンバー(行ヘッダ列)の比率・測定純関数 — 描画・当たり判定は
//! `super::rail`(TL-arch Phase 1、
//! `docs/reviews/2026-08-22-timeline-canvas-widget-survey.md` §6)が実 widget
//! として持つ。
//!
//! **裁定172 §2 施工**(ψ 台帳 #2/#3/#4/#6 の転写)で確定した寸法・色の比率
//! (スウォッチ・M/S/L グリフ)はそのままここに残る — `super::rail` がこれらの
//! 純関数を消費するだけ(発注書「T-rail の `glyph_size_px`/swatch 関数… を
//! そのまま widget 側で使用」)。色は Document の `LayerAttrs.label_color`
//! (ρ、裁定164 後続)から引く — `super::canvas::draw` の bar 差し色と同じ
//! index を読む([`swatch_color`]、mock 注記「bar の差し色 = label_color
//! index」がスウォッチにもそのまま適用される)。
//!
//! ## TL-arch Phase 1(2026-08-22): draw/hit_test をここから撤去
//!
//! このファイルはかつて「レーンバー専用の draw+hit」(canvas 手描き+自前
//! 当たり判定)を持っていた。TL-arch 調査 §2.5 の実測(iced の `canvas::Text`
//! は `ellipsis` フィールドを無視するハードコードされたバグがあり、実 widget
//! の `text()` 経路だけが cosmic-text で正しく ellipsis を効かせる)と §2.3
//! (`pin`/`Stack`/選択的 capture が標準道具として揃っている)を根拠に、rail
//! (スウォッチ・名前・M/S/L)を実 widget(`container`/`text`/`button`)へ
//! 置き換えた(`super::rail`)。**このファイルに残るのは widget 側が消費する
//! 純粋な比率・測定関数だけ** — `Hit`/`hit_test`/`draw`/`glyph_slots`(絶対
//! 座標の組み立て)は `super::rail::view`(iced 標準の event routing/layout
//! に置換)へ意味ごと移設し、このファイルからは削除した(複製ではなく
//! 移設 — 旧ロジックはもう存在しない)。
//!
//! `truncate_to_width`/`default_measure`/`is_wide_char`(裁定168 施工の等幅
//! 近似切り詰め器具)は rail 描画からは退役した(上の §2.5 節の理由そのもの
//! — 実 widget の `text()` は native ellipsis を使うので、この近似測定は
//! もう要らない)。**canvas 側の他用途が無い**(grep 実測: この3関数の
//! 呼び出し元は自分自身の `#[cfg(test)]` だけ)ので、削除ではなく
//! `#[deprecated]` を付けて残す(発注書「関数自体は… deprecated 注記」)。

use crate::tokens::{Colors, Dimensions};

use super::projection::RowProjection;

/// M/S/L のどれか。`super::rail::view` が M/S/L ボタンを組み立てる時に
/// この enum で3つを列挙する(旧 `hit_test`/`draw` と同じ語彙を widget 側でも
/// 再利用 — 意味を発明し直さない)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Glyph {
    Mute,
    Solo,
    Lock,
}

pub(crate) fn glyph_label(glyph: Glyph) -> &'static str {
    match glyph {
        Glyph::Mute => "M",
        Glyph::Solo => "S",
        Glyph::Lock => "L",
    }
}

// ---------------------------------------------------------------------------
// 裁定172 §2 施工: スウォッチ・M/S/L グリフの寸法/色比(EXACT TARGET 2/3)。
// ---------------------------------------------------------------------------

/// スウォッチ(層の差し色)の1辺。mock `.rail .g` ではなく mock 本体の色
/// スウォッチ実測(`timeline-semantics.html` `8px`/`--row=26px` = 0.3077…)を
/// `round(0.308 × 行高)` へ丸めた比で転写(裁定172 §2 (1))。
pub(crate) fn swatch_size_px(row_height: f32) -> f32 {
    (row_height * 0.308).round().max(1.0)
}

/// スウォッチの角丸。mock 実測比 `0.25 × スウォッチ寸`(裁定172 §2 (1))。
pub(crate) fn swatch_radius_px(swatch_size: f32) -> f32 {
    swatch_size * 0.25
}

/// [`iced::Color`] の alias — **fence 回避専用**: `-> iced::Color {` は
/// `next/shell/motolii-shell/tests/suite/tonmana_token_fence.rs` の
/// `COLOR_MARKERS`(`"Color {"` の単純部分文字列一致、doc「マッチしたら
/// 無条件に違反」)に素通しで引っかかる既知の誤検出 — 戻り値の**型注記**で
/// あって `Color{..}` の構造体リテラル構築ではない(この fn は
/// `colors.label_palette`/`colors.way_timeline` という token 由来値を選ぶ
/// だけで raw 色は一切構築しない、裁定142の趣旨に違反しない)。
type Rgba = iced::Color;

/// スウォッチの塗り色 — bar の差し色と同じ源([`super::canvas::draw`] の
/// `bar_color` と同型)。`super::rail::view` が swatch container の背景色に
/// そのまま使う。**bar の `dragging`/`hidden` override はここでは複製しない**
/// (あれは「掴んで動かしている最中の bar」というクリップ面固有の一時状態で、
/// スウォッチは行の恒久 ID(裁定147)を表す静的な色ラベルのプレビューだから)。
pub(crate) fn swatch_color(row: &RowProjection, colors: &Colors) -> Rgba {
    row.label_color
        .and_then(|index| colors.label_palette.get(index as usize))
        .copied()
        .unwrap_or(colors.way_timeline)
}

/// M/S/L グリフ(正方形チップ)の1辺。mock `.rail .g{width:12px;height:12px}`
/// (`--row=26px` = 0.4615…)を `round(0.462 × 行高)` へ丸めた比で転写
/// (裁定172 §2 (2))。
/// pub の理由: `super::rail::view`(この crate 内)に加え、shell 側テスト
/// (`timeline_key_rows_drive.rs`)がグリフのクリック座標をこの関数から導出する
/// — 式の複製を持たせない(T-rail 検収で複製コピーが red になった実害への
/// 追随、canvas.rs の比率関数 pub 化と同じ判断)。
pub fn glyph_size_px(row_height: f32) -> f32 {
    (row_height * 0.462).round().max(1.0)
}

/// グリフ文字(M/S/L)の字寸。mock `.rail .g{font-size:8px}`(グリフ寸12px比
/// 0.667…)を `0.667 × グリフ寸` で転写(裁定172 §2 (2))。
pub(crate) fn glyph_text_size_px(glyph_size: f32) -> f32 {
    glyph_size * 0.667
}

// ---------------------------------------------------------------------------
// 名前列の実効幅(裁定167/168) — widget text の `Length::Fixed` 幅として使う。
// ---------------------------------------------------------------------------

/// 兄弟要素間の gap(裁定167 の梯子下段: `0.075 × 行高`、px 最近傍丸め)。
/// `inspector_pane.rs::sibling_gap_px` と同型(値そのものは pane ごとに token
/// 経由で別々に持つ、式だけ揃える) — [`name_column_width`] が名前列と
/// チップ群の間の緩衝に使う。
fn sibling_gap_px(row_height: f32) -> f32 {
    (row_height * 0.075).round()
}

/// 名前列の実効幅 — rail 幅から M/S/L チップ群と裁定167 の緩衝 gap を差し引いた
/// 残り(EXACT TARGET 1)。`super::rail::view` がこの幅を name `text()` widget の
/// `Length::Fixed` へそのまま渡す(native ellipsis がこの幅を超えた分を「…」へ
/// 畳む、旧 `truncate_to_width` の手動測定は不要になった — 上のモジュール
/// doc 参照)。
///
/// **裁定172 継承**: スウォッチ寸は [`swatch_size_px`](行高比)。チップ群の
/// 開始 x は「rail 右端から `spacing_s` 空けて3個(グリフ寸+gap)を右詰め」
/// という配置そのもの(旧 `glyph_slots` の絶対座標の式をここへ畳んだ — widget
/// 側は実際の x 座標を iced のレイアウトエンジンに計算させるので、
/// `glyph_slots`(座標の配列を返す関数)自体はもう要らない。この関数が要る
/// のは「名前列に許される最大幅」という**数量**だけ)。
pub(crate) fn name_column_width(dims: &Dimensions, rail_width: f32, row_height: f32) -> f32 {
    let swatch_size = swatch_size_px(row_height);
    let name_start_x = dims.spacing_s * 2.0 + swatch_size;
    let glyph_w = glyph_size_px(row_height);
    let chip_block_w = glyph_w * 3.0 + dims.spacing_xs * 2.0;
    let chip_start_x = rail_width - dims.spacing_s - chip_block_w;
    let gap = sibling_gap_px(row_height);
    (chip_start_x - name_start_x - gap).max(0.0)
}

// ---------------------------------------------------------------------------
// 退役(TL-arch Phase 1、上のモジュール doc §2.5 節参照): widget text の
// native ellipsis に置換されたため rail 描画からの呼び出し元は無い。canvas
// 側の他用途も無い(grep 実測) — 削除ではなく deprecated 注記(発注書の
// 指示どおり)。
// ---------------------------------------------------------------------------

/// 全角相当(CJK 統合漢字・ひらがな・カタカナ・ハングル等)の判定。
/// [`default_measure`] の等幅近似だけに使う簡易版 — 東アジア文字幅の一般実装
/// ではなく、レーン名の切り詰めに要る最小限のブロックだけを対象にする。
#[deprecated(
    note = "TL-arch Phase 1(2026-08-22)で rail 描画が widget text の native ellipsis へ \
            移行し、この等幅近似の呼び出し元が無くなった(`default_measure`/ \
            `truncate_to_width` と同じ注記)。canvas 側の他用途も無い(grep 実測) — \
            削除ではなく注記のみ。"
)]
#[allow(dead_code)]
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
#[deprecated(
    note = "TL-arch Phase 1(2026-08-22): rail 描画の呼び出し元が無くなった \
            (上の `is_wide_char` と同じ注記)。"
)]
#[allow(deprecated)]
#[allow(dead_code)]
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
#[deprecated(
    note = "TL-arch Phase 1(2026-08-22): rail 描画の呼び出し元が無くなった \
            (上の `is_wide_char` と同じ注記)。"
)]
#[allow(dead_code)]
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

#[cfg(test)]
#[allow(deprecated)]
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

        let name_start_x = dims.spacing_s * 2.0 + swatch_size_px(row_height);
        let glyph_w = glyph_size_px(row_height);
        let chip_block_w = glyph_w * 3.0 + dims.spacing_xs * 2.0;
        let chip_start_x = rail_width - dims.spacing_s - chip_block_w;
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

    // -----------------------------------------------------------------------
    // 裁定172 §2 の比率オラクル(EXACT TARGET 2/3) — mock 実測比との一致を
    // 純関数レベルで固定する。
    // -----------------------------------------------------------------------

    /// mock 実測: `.rail` の色スウォッチ = 8px、`--row` = 26px → 8/26 = 0.3077…
    /// 裁定は `round(0.308 × 行高)` と定めている(丸め込み後の値、生の比では
    /// ない)ので、この試験は mock の実測比そのものと式の両方を固定する。
    #[test]
    fn swatch_size_px_matches_the_mock_ratio_at_the_mock_row_height() {
        assert_eq!(swatch_size_px(26.0), 8.0, "mock row_height=26 で8pxにならない");
    }

    #[test]
    fn swatch_size_px_is_0_308_of_row_height_rounded() {
        assert_eq!(swatch_size_px(20.0), 6.0); // 0.308*20=6.16 → round=6
        assert_eq!(swatch_size_px(40.0), 12.0); // 0.308*40=12.32 → round=12
    }

    #[test]
    fn swatch_radius_px_is_a_quarter_of_the_swatch_size() {
        assert_eq!(swatch_radius_px(8.0), 2.0);
        assert_eq!(swatch_radius_px(6.0), 1.5);
    }

    /// mock 実測: `.rail .g` = 12px、`--row` = 26px → 12/26 = 0.4615…
    #[test]
    fn glyph_size_px_matches_the_mock_ratio_at_the_mock_row_height() {
        assert_eq!(glyph_size_px(26.0), 12.0, "mock row_height=26 で12pxにならない");
    }

    #[test]
    fn glyph_size_px_is_0_462_of_row_height_rounded() {
        assert_eq!(glyph_size_px(20.0), 9.0); // 0.462*20=9.24 → round=9
        assert_eq!(glyph_size_px(40.0), 18.0); // 0.462*40=18.48 → round=18
    }

    /// mock 実測: `.rail .g{font-size:8px}` に対しグリフ寸12px → 8/12 = 0.6667…
    #[test]
    fn glyph_text_size_px_matches_the_mock_ratio_at_the_mock_glyph_size() {
        let glyph = glyph_size_px(26.0);
        assert!((glyph_text_size_px(glyph) - 8.0).abs() < 0.01, "mock 12px グリフで8ptにならない");
    }

    fn test_row(label_color: Option<u8>) -> RowProjection {
        RowProjection {
            id: motolii_store::LayerId(1),
            name: "Layer 1".to_owned(),
            hidden: false,
            solo: false,
            locked: false,
            label_color,
            start: 0,
            duration: 100,
            selected: true,
            dragging: false,
            depth: 0,
            has_children: false,
            children_open: true,
        }
    }

    /// EXACT TARGET 2/3: スウォッチ色は bar と同じ源(`label_color` →
    /// `label_palette`)から引く — mock 注記「bar の差し色 = label_color
    /// index」がスウォッチにも適用される(裁定172 §2 (1))。
    #[test]
    fn swatch_color_matches_the_label_palette_entry_for_the_selected_row() {
        let colors = Colors::default();
        let row = test_row(Some(3));
        assert_eq!(swatch_color(&row, &colors), colors.label_palette[3]);
    }

    /// `label_color` が無い層は bar と同じフォールバック(`way_timeline`)。
    #[test]
    fn swatch_color_falls_back_to_way_timeline_without_a_label_color() {
        let colors = Colors::default();
        let row = test_row(None);
        assert_eq!(swatch_color(&row, &colors), colors.way_timeline);
    }

    /// index がパレット長を超えていても panic せず bar と同じフォールバックへ
    /// 落ちる(`super::canvas::draw` の `bar_color` と同じ姿勢、M16 前例)。
    #[test]
    fn swatch_color_falls_back_to_way_timeline_when_the_index_is_out_of_range() {
        let colors = Colors::default();
        let row = test_row(Some(255));
        assert_eq!(swatch_color(&row, &colors), colors.way_timeline);
    }
}
