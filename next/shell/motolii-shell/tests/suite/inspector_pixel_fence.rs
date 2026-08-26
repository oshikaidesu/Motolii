//! Inspector ピクセル忠実度の柵(発注書: 「mock の CSS 計算値を期待表として
//! 固定し、iced_test で実 widget ツリーの bounds を読んで ±1px で照合する」)。
//!
//! ## 出典(視覚正本 `next/reference/mocks/inspector-library.html` v3.1、
//! `inspector-library.css` の CSS 計算値・手計算)
//!
//! **I-tokens(2026-08-22)で更新**: 視覚正本を旧 `ui-scale-and-z.html` から
//! `inspector-library.html` v3.1(利用者合格・転写正本統一)へ切り替え、
//! `inspector_row_height`/`inspector_value_width`/`inspector_glyph_width`/
//! `inspector_panel_width` の4値を束で再転写した(I-ratio 台帳が発見した
//! 二重モック構造 — この4値のうち3値だけ別モック由来だった状態 — の根治)。
//!
//! ```text
//! --row:      25px         .propertyRow(min-height) — property/column-header 行高
//! --section:  26px         .tableSection h2/panelHeader 相当 — 見出し高(値は旧mockと元々一致・不変)
//! --sp1:       2px         spacing_xs(グローバル token、この mock 自身には対応する gap 定義なし)
//! --sp2:       4px         spacing_s(同上)
//! --sp4:       8px         ident/cols/prow/sec/hint 帯 padding(横、グローバル token)
//! 値セル:     64 × (row-4=21) px   grid-template-columns の 64px 段(`.valueCell`)
//! glyph:       26 × (row-2=23) px  グリッド末尾26px段(Key列)・`.glyph` 高
//! pane:       496px        mock `.inspectorShell{width:min(100%,496px)}` と**直接一致**(旧裁定172 §3 時点は「例外」だったが、4値統一によりもう例外ではない)
//! ```
//!
//! **値セル/glyph の高さ式(row-4/row-2)は旧 mock(ui-scale-and-z.html)由来の
//! 内部式のまま維持**: `inspector-library.css` の `.valueCell`/`.glyph` 相当は
//! grid の `align-items:stretch` で行高いっぱいに伸びる(高さを縮める式を
//! 持たない)— この式を「グリッドの構成」まで転写し直すのは NON-GOALS
//! (「行の構成変更」)に該当するため、この発注では触れていない。既知の乖離
//! として RETURN の FINDING に記録する。
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
//! ## pane 幅(496px)— もう例外ではない
//!
//! **I-tokens(2026-08-22)以前**: pane 幅は旧 `docs/mocks-ui/public/
//! inspector-library.css` `.inspectorShell{width:496px}` から据え置いた値で、
//! 当時の視覚正本(`ui-scale-and-z.html`, `--pane:300`)とは食い違う「意図的な
//! 例外」だった。**I-tokens 後**: 視覚正本そのものを inspector-library v3.1 へ
//! 統一したので、496 は今の正本の値そのもの — 例外ではなく通常の一致項目に
//! なった([`the_pane_frame_uses_the_documented_width_and_source`] が496と
//! token の一致を確かめる、テスト自体は継続)。
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

const EPS: f32 = 1.0; // 発注書の許容 ±1px(フォント依存の位置比較など)

/// **I-tokens(2026-08-22)追い施工**: `containers_matching` 専用の厳格な
/// 許容(±0.05px)。`inspector_row_height`(25)を再転写した結果、
/// `inspector_section_header_height`(26)とわずか1px差の隣接値になった —
/// 元の `EPS`(1.0px、フォント計測のブレを吸収するための許容)のままだと
/// 25px高の箱と26px高の箱が互いを ±1px 以内とみなして誤って混ざる(実測:
/// 帯4本のはずが両方12本を返す衝突を発見)。ここで比較する高さはどれも
/// `Length::Fixed(dims....)` から直接来る決定論値(フォント計測を経由しない)
/// なので、浮動小数点の丸め誤差だけを吸収できれば十分 — 厳格側へ倒しても
/// 誤検出は増えない。
const EPS_EXACT: f32 = 0.05;

/// 指定した幅・高さ(`EPS_EXACT`)の `Container` candidate を全部集める。
/// button/row/column/container はどれも自分を `Container` として登録する
/// (`q0_fence.rs` doc 実測どおり)ので、grid の箱・帯はすべてこれで拾える。
fn containers_matching<'a>(targets: &'a [Target], width: f32, height: f32) -> Vec<&'a Target> {
    targets
        .iter()
        .filter(|t| matches!(t, Target::Container { .. }))
        .filter(|t| {
            let b = t.bounds();
            (b.width - width).abs() <= EPS_EXACT && (b.height - height).abs() <= EPS_EXACT
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
// pane 幅(I-tokens 後はもう例外ではない — mock 実測値そのもの)
// ---------------------------------------------------------------------------

/// **I-tokens(2026-08-22)で「例外」から「一致」へ**: 視覚正本を
/// inspector-library v3.1 へ統一したため、496 はもう `ui-scale-and-z.html`
/// との食い違いではなく現正本の値そのもの。この柵は「496 が token どおりに
/// 描画されている」ことを確かめる(旧テスト名の `..._exception` は誤解を招く
/// ため `..._and_source` へ改名)。
#[test]
fn the_pane_frame_uses_the_documented_width_and_source() {
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
        "pane 幅の値そのものが動いた — inspector-library v3.1 実測(496)から \
         動いたなら、この柵冒頭の doc・`tokens.rs::Dimensions::inspector_panel_width` \
         doc も合わせて更新すること"
    );
}

// ---------------------------------------------------------------------------
// 帯(header/section)— row=25(I-tokens 2026-08-22 再転写) / section=26 の
// box が pane 全幅で並ぶこと
// ---------------------------------------------------------------------------

/// **この柵が見つけたバグの回帰**: TRANSFORM/APPEARANCE/ATTRS の帯は mock の
/// block 要素どおり「pane 全幅 × section 高(26)」の箱であること。修正前は
/// 文字幅ぶんの箱にしかなっていなかった(`.width(Length::Fill)` 付け忘れ)。
///
/// 4本→3本(2026-08-22): mock v3.1 の ptitle("Inspector" header)は転写を
/// やめた — pane 名の正本は shell の pane 題帯(pane_grid title_bar)へ移り、
/// 内部 header は二重表示のため除去(題帯レーンの API 要求・supervisor 施工)。
///
/// 3本→4本(2026-08-22「レイヤーを指す」文法発注): LINK section
/// (`crate::link::link_section`)を常設で足したので、TRANSFORM/APPEARANCE/
/// ATTRS に LINK が加わり4本になった(`section_header` を再利用しているので
/// 箱の形は他3本と同一 — この柵がそのまま拾う)。
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
        "pane 全幅×26px の帯(TRANSFORM+APPEARANCE+ATTRS+LINK)が4本のはずが{}本: {bars:?}",
        bars.len()
    );
}

/// column-header 行(Property/X/Y/Z/Key)+ Transform 5行(Position/Scale/
/// Rotation/Anchor/Opacity)+ Blend 行 + Speed 行(SP1 第一波、supervisor
/// 決定1 — Blend の下に同じ `.prow` grammar で足した) = 8本、すべて
/// 「pane 全幅 × row 高(25、I-tokens 2026-08-22 再転写)」であること
/// (mock `.columnHeader`/`.propertyRow` の box)。
///
/// 8本→14本(2026-08-22「レイヤーを指す」文法発注): ATTRS の末尾に Matte 行
/// (`crate::matte::matte_row`)を1本、LINK section に標準 property 5種ぶんの
/// 行(`crate::link::link_row`)を足した(いずれも既存の `bordered_row` 文法を
/// そのまま使うので、箱の形は既存の Blend/Speed 行と同一 — 8 + 1 + 5 = 14)。
#[test]
fn the_full_width_property_rows_span_the_entire_pane_at_the_mock_row_height() {
    let (targets, dims) = selected_inspector_targets();
    let rows = containers_matching(&targets, dims.inspector_panel_width, dims.inspector_row_height);
    assert_eq!(
        rows.len(),
        14,
        "pane 全幅×{}px の行(cols見出し+Transform5行+Blend+Speed+Matte+Link5行)が14本のはずが{}本: {rows:?}",
        dims.inspector_row_height,
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
            // ptitle 除去(2026-08-22)後、ident 帯が pane 先頭(y=0)に来る
            b.x.abs() <= EPS && b.y.abs() <= EPS && b.height < dims.inspector_row_height * 3.0
        })
        .unwrap_or_else(|| panic!("ident 帯の Container が見つからない(pane 先頭、x=0)"));
    assert!(
        (ident.bounds().width - dims.inspector_panel_width).abs() <= EPS,
        "ident 帯が pane 全幅になっていない: {:?}",
        ident.bounds()
    );
}

// ---------------------------------------------------------------------------
// grid の箱 — 値セル 64×(row-4) / glyph 26×(row-2)(I-tokens 2026-08-22 再転写)
// ---------------------------------------------------------------------------

/// `.prow .v { height: calc(row - 4*spacing_s) }` の箱(高さの縮小式は
/// 旧 mock 由来の内部式のまま維持 — ファイル冒頭 doc の「値セル/glyph の
/// 高さ式」注記参照、NON-GOALS「行の構成変更」に該当するため今回は不変)。
/// present(編集可)/absent(「—」)/animated(表示のみ)/blank(Opacity の空セル)
/// のどの状態でも同じ形(`value_cell` の doc コメントどおり)— Transform 5行
/// × 3セル(X/Y/Z)= 15個。
#[test]
fn the_value_cells_match_the_mock_64_by_row_minus_4_grid() {
    let (targets, dims) = selected_inspector_targets();
    assert_eq!(dims.inspector_value_width, 64.0, "値セル幅がずれている(I-tokens 再転写値)");
    let expected_height = dims.inspector_row_height - dims.spacing_s; // mock の `4` = spacing_s
    let cells = containers_matching(&targets, dims.inspector_value_width, expected_height);
    assert_eq!(
        cells.len(),
        15,
        "64×{expected_height}px の値セルが15個(5行×3列)のはずが{}個: {cells:?}",
        cells.len()
    );
}

/// mock グリッド末尾26px段(Key 列幅)・高さは `row - spacing_xs` を基礎にし、
/// Google系の操作対象床を下回らない。M(mute)glyph と、K1 で
/// 結線された各行の Key glyph(Transform 5行)が `button` の `Container`
/// candidate として見える(S の予約枠だけが `Space` = `operate()` 未実装で
/// 不可視のまま、`ident_band_drive.rs` doc のとおり)— 1 + 5 = 6個のはず。
#[test]
fn the_mute_and_key_glyphs_match_the_mock_26_by_row_minus_2_square() {
    let (targets, dims) = selected_inspector_targets();
    assert_eq!(dims.inspector_glyph_width, 26.0, "Key 列幅がずれている(I-tokens 再転写値)");
    let expected_height = (dims.inspector_row_height - dims.spacing_xs)
        .max(dims.interactive_target_min); // mock の `2` = spacing_xs
    let glyphs = containers_matching(&targets, dims.inspector_glyph_width, expected_height);
    assert_eq!(
        glyphs.len(),
        6,
        "26×{expected_height}px の glyph 箱(M 1個 + Key 5個)が6個のはずが{}個: {glyphs:?}",
        glyphs.len()
    );
}

// ---------------------------------------------------------------------------
// column-header の grid gap(spacing_xs=2)・左右 padding(spacing_m=8)
// ---------------------------------------------------------------------------

/// mock の grid 自体には列間 gap の宣言が無い(`.columnHeader,.propertyRow`
/// は `grid-template-columns` のみ、`gap` プロパティ未指定)— この gap は
/// 実装内部の兄弟間隔式(`sibling_gap_px` = `spacing_xs` 段のグローバル
/// spacing token)によるもので、値セル幅そのもの(64px)とは独立(NON-GOALS
/// 「行の構成変更」により今回は不変、ファイル冒頭 doc 参照)。X→Y→Z→Key の
/// 間隔が一律 `64 + spacing_xs(2)` = 66px、左端(Property)は
/// `spacing_m`(8)、右端(Key の右)も同じ8pxで pane 幅ちょうどに収まること。
#[test]
fn the_column_header_grid_matches_the_mock_gap_and_side_padding() {
    let (targets, dims) = selected_inspector_targets();
    let property = find_text(&targets, "Property");
    let x = find_text(&targets, "X");
    let y = find_text(&targets, "Y");
    let z = find_text(&targets, "Z");
    let key = find_text(&targets, "Key");

    let hop = dims.inspector_value_width + dims.spacing_xs; // 64 + spacing_xs(2) = 66
    assert!((y.x - x.x - hop).abs() <= EPS, "X→Y の gap が spacing_xs(2px) どおりでない: {} vs {hop}", y.x - x.x);
    assert!((z.x - y.x - hop).abs() <= EPS, "Y→Z の gap が spacing_xs(2px) どおりでない: {} vs {hop}", z.x - y.x);
    assert!(
        (key.x - z.x - hop).abs() <= EPS,
        "Z→Key の gap が spacing_xs(2px) どおりでない: {} vs {hop}",
        key.x - z.x
    );

    assert!(
        (property.x - dims.spacing_m).abs() <= EPS,
        "Property の左 padding が spacing_m(8) と違う: {}",
        property.x
    );
    assert!(
        (key.x + dims.inspector_glyph_width + dims.spacing_m - dims.inspector_panel_width).abs() <= EPS,
        "Key 列の右 padding が spacing_m(8) と違う(pane 幅に収まらない)"
    );
}

// ---------------------------------------------------------------------------
// 150%(mock 第2スナップショット `--s:1.50`)でも grid の形が保たれること
// ---------------------------------------------------------------------------

/// mock の2枚目(`--s:1.50`)と同じ倍率。`ui_scale_fence.rs` は「全寸法に
/// 1.5 が掛かる」ことを `Dimensions` 単体で見ているが、ここでは実際に描いた
/// widget 木でも 26px 相当の帯が4本(TRANSFORM/APPEARANCE/ATTRS/LINK)・
/// 25px 相当(I-tokens 再転写)の行が14本のまま(数が変わらない = grid の
/// **形**が保たれる)ことを見る(SP1 で行数が7→8に増え、2026-08-22
/// 「レイヤーを指す」文法発注で Matte 行+LINK 5行が足されて8→14になった
/// 上での基準)。
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
    assert_eq!(bars.len(), 4, "150%でも26px相当の帯が4本のはずが{}本(ptitle除去後・LINK section 追加後)", bars.len());

    let rows = containers_matching(&targets, dims.inspector_panel_width, dims.inspector_row_height);
    assert_eq!(rows.len(), 14, "150%でも25px相当(I-tokens 再転写)の行が14本のはずが{}本", rows.len());

    let value_height = dims.inspector_row_height - dims.spacing_s;
    let cells = containers_matching(&targets, dims.inspector_value_width, value_height);
    assert_eq!(cells.len(), 15, "150%でも値セルが15個のはずが{}個", cells.len());
}
