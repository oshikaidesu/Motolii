//! Inspector ピクセル忠実度の柵(発注書: 「mock の CSS 計算値を期待表として
//! 固定し、iced_test で実 widget ツリーの bounds を読んで ±1px で照合する」)。
//!
//! ## 出典(視覚正本 `next/reference/mocks/ui-scale-and-z.html`, `--s:1.00` 時の
//! CSS 計算値・手計算)
//!
//! ```text
//! --row:      20px         property/column-header 行高
//! --section:  26px         ptitle(パネルタイトル)/sec(TRANSFORM等)見出し高
//! --sp1:       2px         grid gap(X/Y/Z/Key の間隔)
//! --sp2:       4px         ident 帯 padding(縦)
//! --sp4:       8px         ident/cols/prow/sec/hint 帯 padding(横)
//! 値セル:     38 × (row-4=16) px   grid-template-columns の 38px 段、`.prow .v` 高
//! glyph:       18 × (row-2=18) px  --hit、`.glyph` 高
//! pane:       300px        **例外**扱い(下記)
//! ```
//!
//! これらの数値そのものは既に `motolii_shell::tokens::Dimensions::default()` の
//! 各フィールドと `tokens.rs::ui_scale_tests`(`the_canonical_type_band_matches_
//! the_mock`/`the_inspector_section_height_matches_the_mock` 等)が固定している。
//! **この柵の仕事はその先**: token の値が合っていても、`.width(Length::Fill)`
//! の付け忘れのような配線ミスで実際に描かれる box が違う形になる事故は token
//! 側の柵では捕まらない — 実際に `view()` した widget 木の bounds を読んで
//! 初めて分かる(この柵が最初に見つけた実例が
//! [`the_full_width_section_bars_span_the_entire_pane_at_the_mock_section_height`])。
//!
//! ## 例外(意図的に mock と違える点)
//!
//! - **pane 幅 496px**(mock `--pane:300`ではない): 旧 `docs/mocks-ui/public/
//!   inspector-library.css` `.inspectorShell{width:496px}` を出典として据え置き
//!   (`tokens.rs::Dimensions::inspector_panel_width` doc、`next/reference/
//!   CANON.md` に明記)。300 への変更は利用者裁定待ち — この柵は 496 が
//!   token どおりに描画へ反映されていることだけを確かめる
//!   ([`the_pane_frame_uses_the_documented_width_exception`])。
//!
//! ## 対象外(正直な限界 — 照合しない)
//!
//! - **フォント依存の高さ**: `text`/`text_input` の実測高は iced_test のレンダラ
//!   既定フォント("Fira Sans")の line-height 依存で、mock を描いたブラウザの
//!   フォントとは別物 — px 一致を主張できない。ident 帯(名前欄+種別)や
//!   hint 行のような、`.height(Length::Fixed(..))` で token 固定していない
//!   要素の高さはここでは照合しない(幅だけ照合する)。
//! - **letter-spacing**(mock `.sec{letter-spacing:.08em}`): `iced_core::text`/
//!   `iced_widget::text` を実測したが letter-spacing に相当するフィールドが
//!   存在しない(0.14.0 時点)。新依存無しでは実装不可 — 既知の限界として
//!   ここに書くだけに留める(token 化のしようがない)。
//! - **border-bottom-only**: mock の `.ptitle`/`.ident`/`.cols`/`.prow`/`.hint`
//!   は下辺(または上辺)だけの罫線だが、`iced_core::Border`(`border.rs` 実測)
//!   は4辺一律にしかできない(per-edge API が無い)。現実装はコンテナ全周へ
//!   同じ罫線を引いている — 新依存無しでは直せない、既知の限界(2026-08-21
//!   線化 pass で `.cols`/`.prow` 行にも同じ trade-off を拡張した —
//!   `inspector_pane.rs::bordered_row`)。**`.sec` だけは例外** — mock 自体が
//!   `.sec` に border を持たない(letter-spacing/ink だけで見出しを作る)ので、
//!   2026-08-21 に `section_header` の背景塗り(`surface_app`)を削除し、
//!   border も付けていない(mock どおり無地)。
//! - **`Target::TextInput`/`Target::Container` は style(背景色・border色・
//!   padding)を一切持たない**(`iced_selector::target::Target` 実測 — 公開
//!   フィールドは `id`/`bounds`/`visible_bounds`/`content` のみ)。このため
//!   hairline の色や `value_cell`/`name_field` の内余白そのものは、この柵
//!   (widget bounds の幾何比較)では原理的に検証できない — 2026-08-21 の
//!   線化 pass はこれらを `inspector_pane.rs` 側の純粋関数
//!   (`value_cell_padding`/`name_field_padding`)へ切り出し、
//!   `inspector_pane.rs::tests` でトークン値そのものを直接照合する形に
//!   倒した(この柵の対象外として正直に記録するだけに留める)。
//! - mock `.ptitle` の `<i>` アイコン・`<em>` 種別バッジ、`.sec` の letter-spacing
//!   ぶんの gap(`--sp3`=6px)はこの実装では未描画(header は文言のみ) —
//!   sp3 を消費する場所が無いので token 化していない。
//!
//! ## この柵が見つけて直した実バグ(修正前の実測値、`git log` 参照)
//!
//! - `header`/`section_header`/`hint_row` の `container` に
//!   `.width(Length::Fill)` が無く、mock の block 要素(pane 全幅の帯)ではなく
//!   文字幅ぶんの箱にしかならなかった(修正前実測: header 幅 67.552px、
//!   TRANSFORM 65.401px、APPEARANCE 68.191px、ATTRS 40.813px、hint
//!   124.801px — いずれも `inspector_panel_width`(496)より大幅に狭い)。
//! - 値セル(X/Y/Z)の `text_input` が既定 padding
//!   (`iced_widget::text_input::DEFAULT_PADDING` = 5px 全辺)のままで、固定高
//!   16px(`value_cell_height` = row-4)の箱に押し込まれると paragraph 領域が
//!   `Padding::fit`/`Limits::resolve` の実測どおり 16 - 2*5 = 6px まで潰れて
//!   いた。`.padding(0.0)` で解消(修正後は 14.3px の行が 16px の箱の中に
//!   余裕を持って収まる)。
//! - ident 帯の名前欄(`text_input`)も同じ padding 問題で高さが 24.3px
//!   (mock の想定より約10px 余計に伸びる)になっていた。`.padding(0.0)` で
//!   14.3px に縮み、ident 帯自体の高さも 44px→34px へ縮んだ。

use iced_test::selector::{Candidate, Target};

use motolii_shell::inspector_pane;
use motolii_shell::tokens::{Dimensions, Tokens};
use motolii_shell::{Message, Shell};

/// `ident_band_drive.rs`/`q0_fence.rs` と同じ手口(`Simulator` に `find_all` が
/// 無いので `find` を「まだ拾っていない候補」で尽きるまで繰り返す)。共有
/// ヘルパー化は発注範囲の外(既存2ファイルも複製している)。**`inspector_pane::
/// view` を直叩きする**ので `Message` は pane ローカル(裁定160 切片8)。
fn collect_targets(element: iced::Element<'_, inspector_pane::Message>) -> Vec<Target> {
    let mut ui = iced_test::simulator(element);
    let mut found: Vec<Target> = Vec::new();
    loop {
        let already = found.clone();
        let selector = move |candidate: Candidate<'_>| -> Option<Target> {
            let target = Target::from(candidate);
            if already.contains(&target) {
                None
            } else {
                Some(target)
            }
        };
        match ui.find(selector) {
            Ok(target) => found.push(target),
            Err(_) => break,
        }
        assert!(found.len() <= 5_000, "candidate 列挙が終わらない");
    }
    found
}

/// `--s:1.00`(mock の第1スナップショット)= `Dimensions::default()`。1層選択
/// 済みの Inspector 投影を作る(`AddLayer` 直後は全 field 編集可・un-keyed ——
/// `ui_scale_fence.rs::selection_and_dims` と同じ形)。
fn selected_inspector_targets() -> (Vec<Target>, Dimensions) {
    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AddLayer);
    let selection = shell
        .inspector_selection()
        .expect("AddLayer 直後は選択があるはず");

    let tokens = Tokens::default();
    let dims = tokens.dims.scaled(tokens.ui_scale);
    let colors = tokens.colors;

    let targets = collect_targets(inspector_pane::view(
        Some(&selection),
        None,
        None,
        dims,
        colors,
    ));
    (targets, dims)
}

const EPS: f32 = 1.0; // 発注書の許容 ±1px

/// 指定した幅・高さ(±1px)の `Container` candidate を全部集める。button/row/
/// column/container はどれも自分を `Container` として登録する(`q0_fence.rs`
/// doc 実測どおり)ので、grid の箱・帯はすべてこれで拾える。
fn containers_matching<'a>(targets: &'a [Target], width: f32, height: f32) -> Vec<&'a Target> {
    targets
        .iter()
        .filter(|t| matches!(t, Target::Container { .. }))
        .filter(|t| {
            let b = t.bounds();
            (b.width - width).abs() <= EPS && (b.height - height).abs() <= EPS
        })
        .collect()
}

fn find_text<'a>(targets: &'a [Target], content: &str) -> iced::Rectangle {
    targets
        .iter()
        .find(|t| matches!(t, Target::Text { content: c, .. } if c == content))
        .unwrap_or_else(|| panic!("Text {content:?} が見つからない"))
        .bounds()
}

// ---------------------------------------------------------------------------
// 例外: pane 幅
// ---------------------------------------------------------------------------

/// mock `--pane:300` に対する意図的な例外(496px 据え置き)。この柵は
/// 「496 が token どおりに描画されている」ことだけを確かめる — 300 を要求する
/// 柵ではない(`tokens.rs::Dimensions::inspector_panel_width` doc 参照)。
#[test]
fn the_pane_frame_uses_the_documented_width_exception() {
    let (targets, dims) = selected_inspector_targets();
    let pane_width = targets
        .first()
        .expect("最初の candidate が pane 自身の Container のはず")
        .bounds()
        .width;
    assert!(
        (pane_width - dims.inspector_panel_width).abs() <= EPS,
        "pane 幅が token({}) と食い違う: 実測 {pane_width}",
        dims.inspector_panel_width
    );
    assert_eq!(
        dims.inspector_panel_width, 496.0,
        "例外の値そのものが動いた — 496 据え置きの前提が崩れている(利用者裁定待ちの数値なので、\
         意図的な変更なら exceptions 一覧のコメントも合わせて更新すること)"
    );
}

// ---------------------------------------------------------------------------
// 帯(header/section)— row=20 / section=26 の box が pane 全幅で並ぶこと
// ---------------------------------------------------------------------------

/// **この柵が見つけたバグの回帰**: header(ptitle)/TRANSFORM/APPEARANCE/ATTRS
/// の4本の帯は mock の block 要素どおり「pane 全幅 × section 高(26)」の箱で
/// あること。修正前は文字幅ぶんの箱(67.552/65.401/68.191/40.813px)にしか
/// なっていなかった(`.width(Length::Fill)` 付け忘れ)。
#[test]
fn the_full_width_section_bars_span_the_entire_pane_at_the_mock_section_height() {
    let (targets, dims) = selected_inspector_targets();
    let bars = containers_matching(
        &targets,
        dims.inspector_panel_width,
        dims.inspector_section_header_height,
    );
    assert_eq!(
        bars.len(),
        4,
        "pane 全幅×26px の帯(header+TRANSFORM+APPEARANCE+ATTRS)が4本のはずが{}本: {bars:?}",
        bars.len()
    );
}

/// column-header 行(Property/X/Y/Z/Key)+ Transform 5行(Position/Scale/
/// Rotation/Anchor/Opacity)+ Blend 行 + Speed 行(SP1 第一波、supervisor
/// 決定1 — Blend の下に同じ `.prow` grammar で足した) = 8本、すべて
/// 「pane 全幅 × row 高(20)」であること(mock `.cols`/`.prow` の box)。
#[test]
fn the_full_width_property_rows_span_the_entire_pane_at_the_mock_row_height() {
    let (targets, dims) = selected_inspector_targets();
    let rows = containers_matching(&targets, dims.inspector_panel_width, dims.inspector_row_height);
    assert_eq!(
        rows.len(),
        8,
        "pane 全幅×20px の行(cols見出し+Transform5行+Blend+Speed)が8本のはずが{}本: {rows:?}",
        rows.len()
    );
}

/// ident 帯(名前+種別+M/S glyph)も mock の block 要素どおり pane 全幅。高さは
/// `text_input`/`text` のフォント依存なので照合しない(ファイル冒頭「対象外」
/// 参照)— header 直後(y = section 高)にある、他より明らかに低い(スクロール
/// 領域丸ごとではない)Container を探して幅だけ見る。
#[test]
fn the_ident_band_spans_the_full_pane_width() {
    let (targets, dims) = selected_inspector_targets();
    let ident = targets
        .iter()
        .filter(|t| matches!(t, Target::Container { .. }))
        .find(|t| {
            let b = t.bounds();
            b.x.abs() <= EPS
                && (b.y - dims.inspector_section_header_height).abs() <= EPS
                && b.height < dims.inspector_row_height * 3.0
        })
        .unwrap_or_else(|| panic!("ident 帯の Container が見つからない(header 直後、x=0)"));
    assert!(
        (ident.bounds().width - dims.inspector_panel_width).abs() <= EPS,
        "ident 帯が pane 全幅になっていない: {:?}",
        ident.bounds()
    );
}

// ---------------------------------------------------------------------------
// grid の箱 — 値セル 38×(row-4) / glyph 18×(row-2)
// ---------------------------------------------------------------------------

/// mock `.prow .v { height: calc(var(--row) - 4*var(--s)*1px) }` の箱。
/// present(編集可)/absent(「—」)/animated(表示のみ)/blank(Opacity の空セル)
/// のどの状態でも同じ形(`value_cell` の doc コメントどおり)— Transform 5行
/// × 3セル(X/Y/Z)= 15個。
#[test]
fn the_value_cells_match_the_mock_38_by_row_minus_4_grid() {
    let (targets, dims) = selected_inspector_targets();
    let expected_height = dims.inspector_row_height - dims.spacing_s; // mock の `4` = spacing_s
    let cells = containers_matching(&targets, dims.inspector_value_width, expected_height);
    assert_eq!(
        cells.len(),
        15,
        "38×{expected_height}px の値セルが15個(5行×3列)のはずが{}個: {cells:?}",
        cells.len()
    );
}

/// mock `.glyph { width: var(--hit); height: calc(var(--row) - 2*var(--s)*1px) }`。
/// M(mute)glyph は結線済みなので `button` の `Container` candidate として
/// 見える(S/Key の予約枠は `Space` = `operate()` 未実装で不可視、
/// `ident_band_drive.rs` doc のとおり)— ちょうど1個のはず。
#[test]
fn the_mute_glyph_matches_the_mock_18_by_row_minus_2_square() {
    let (targets, dims) = selected_inspector_targets();
    assert_eq!(dims.inspector_glyph_width, 18.0, "--hit の値がずれている");
    let expected_height = dims.inspector_row_height - dims.spacing_xs; // mock の `2` = spacing_xs
    let glyphs = containers_matching(&targets, dims.inspector_glyph_width, expected_height);
    assert_eq!(
        glyphs.len(),
        1,
        "18×{expected_height}px の M glyph 箱が1個のはずが{}個: {glyphs:?}",
        glyphs.len()
    );
}

// ---------------------------------------------------------------------------
// column-header の grid gap(sp1=2)・左右 padding(sp4=8)
// ---------------------------------------------------------------------------

/// mock `.cols,.prow { grid-template-columns: 1fr repeat(3,38px) hit;
/// gap: var(--sp1) }` — X→Y→Z→Key の間隔が一律 `38 + sp1(2)` = 40px、
/// 左端(Property)は `spacing_m`(--sp4=8)、右端(Key の右)も同じ8pxで
/// pane 幅ちょうどに収まること。
#[test]
fn the_column_header_grid_matches_the_mock_gap_and_side_padding() {
    let (targets, dims) = selected_inspector_targets();
    let property = find_text(&targets, "Property");
    let x = find_text(&targets, "X");
    let y = find_text(&targets, "Y");
    let z = find_text(&targets, "Z");
    let key = find_text(&targets, "Key");

    let hop = dims.inspector_value_width + dims.spacing_xs; // 38 + sp1(2) = 40
    assert!((y.x - x.x - hop).abs() <= EPS, "X→Y の gap が sp1(2px) どおりでない: {} vs {hop}", y.x - x.x);
    assert!((z.x - y.x - hop).abs() <= EPS, "Y→Z の gap が sp1(2px) どおりでない: {} vs {hop}", z.x - y.x);
    assert!(
        (key.x - z.x - hop).abs() <= EPS,
        "Z→Key の gap が sp1(2px) どおりでない: {} vs {hop}",
        key.x - z.x
    );

    assert!(
        (property.x - dims.spacing_m).abs() <= EPS,
        "Property の左 padding が spacing_m(--sp4=8) と違う: {}",
        property.x
    );
    assert!(
        (key.x + dims.inspector_glyph_width + dims.spacing_m - dims.inspector_panel_width).abs() <= EPS,
        "Key 列の右 padding が spacing_m(--sp4=8) と違う(pane 幅に収まらない)"
    );
}

// ---------------------------------------------------------------------------
// 150%(mock 第2スナップショット `--s:1.50`)でも grid の形が保たれること
// ---------------------------------------------------------------------------

/// mock の2枚目(`--s:1.50`)と同じ倍率。`ui_scale_fence.rs` は「全寸法に
/// 1.5 が掛かる」ことを `Dimensions` 単体で見ているが、ここでは実際に描いた
/// widget 木でも 26px 高の帯が4本・20px 高の行が8本のまま(数が変わらない
/// = grid の**形**が保たれる)ことを見る(SP1 で行数が7→8に増えた上での基準)。
#[test]
fn the_grid_shape_is_preserved_at_150_percent_scale() {
    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AddLayer);
    let selection = shell.inspector_selection().expect("selection");

    let tokens = Tokens {
        ui_scale: 1.5,
        ..Tokens::default()
    };
    let dims = tokens.dims.scaled(tokens.ui_scale);
    let colors = tokens.colors;

    let targets = collect_targets(inspector_pane::view(Some(&selection), None, None, dims, colors));

    let bars = containers_matching(&targets, dims.inspector_panel_width, dims.inspector_section_header_height);
    assert_eq!(bars.len(), 4, "150%でも26px相当の帯が4本のはずが{}本", bars.len());

    let rows = containers_matching(&targets, dims.inspector_panel_width, dims.inspector_row_height);
    assert_eq!(rows.len(), 8, "150%でも20px相当の行が8本のはずが{}本", rows.len());

    let value_height = dims.inspector_row_height - dims.spacing_s;
    let cells = containers_matching(&targets, dims.inspector_value_width, value_height);
    assert_eq!(cells.len(), 15, "150%でも値セルが15個のはずが{}個", cells.len());
}
