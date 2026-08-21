//! 裁定172 §3 の指示による Inspector 比率台帳の機械照合
//! (`docs/reviews/2026-08-22-inspector-ratio-ledger.md` の実測本体はそちら —
//! この柵はその台帳の主要数値を「実装定数を `use` した両側チェック」として
//! 固定するだけの regression lock、`crates/motolii-shell-iced/tests/
//! css_metrics_oracle.rs` と同型の手口)。
//!
//! ## モック側はなぜ literal 定数か(旧 css_metrics_oracle.rs との違い)
//!
//! `next/` workspace は `motolii-ui`(`css_metrics::extract` — Blitz による
//! HTML/CSS の機械抽出器具)を members に含まない(`next/Cargo.toml` 実測 —
//! rerun fork 以外の余計な依存を引かない設計、`next/reference/mocks/
//! ui-scale-and-z.html` 側の egui/skia 切断と同じ理由)。旧 workspace
//! (`crates/`)の `css_metrics_oracle.rs` のように `extract()` を直接呼んで
//! 両側とも「実物」にすることはできない — モック側は
//! `docs/mocks-ui/public/inspector-library.css` からの引用リテラル(行番号を
//! doc comment に明記)として pin し、実装側だけ `motolii_tokens_rs::
//! Dimensions::default()` を `use` して読む。**実装側が変われば必ず落ちる**
//! ので「両側チェック」の意味は保っている(モック側が動くのは人間がこの
//! css ファイルを編集した時だけ — その時はこのテストの定数も手で更新する)。
//!
//! ## この柵が固定する主張
//!
//! (1) `inspector-library.css` 実測の `body_text 相当 / row_height 相当`
//! (`.propertyName span` 11px / `.propertyRow` min-height 25px)= 0.44 は
//! 裁定168 の帯(0.42±0.05)の**内**。
//! (2) 現行実装の `Dimensions::default().body_text / .inspector_row_height`
//! = 0.55 は同じ帯の**外**(`lib.rs` の
//! `inspector_character_size_ratio_is_locked_at_its_current_out_of_band_value`
//! と同じ pin、ここでは Inspector 固有モックの実測値と並べて再確認する)。
//! (3) 両者が一致しない ==「φ FINDING は Inspector 固有モックの実測でも
//! 裏付けられる」の機械的な証拠。

use motolii_tokens_rs::Dimensions;

/// `docs/mocks-ui/public/inspector-library.css:357-364` `.propertyName span`
/// — 値行ラベル文字(`dims.body_text` の実装対応)。
const MOCK_PROPERTY_NAME_FONT_PX: f32 = 11.0;

/// 同 css:282-288 `.propertyRow{min-height:25px}` — 値行の行高
/// (`dims.inspector_row_height` の実装対応)。
const MOCK_PROPERTY_ROW_HEIGHT_PX: f32 = 25.0;

/// 同 css:369-408 `.valueCell input` — 値本体の文字(monospace)。
const MOCK_VALUE_CELL_FONT_PX: f32 = 9.0;

/// 同 css:110-120 `grid-template-columns: minmax(132px,1fr) repeat(3,64px)
/// 26px` — 値セル(X/Y/Z 1つぶん)幅。
const MOCK_VALUE_CELL_WIDTH_PX: f32 = 64.0;

/// 同上、グリッド末尾 `26px` — Key 列幅。
const MOCK_KEY_COLUMN_WIDTH_PX: f32 = 26.0;

/// 同 css:141-154 `.tableSection h2{height:26px}` — section 見出し帯高
/// (`dims.inspector_section_header_height` の実装対応)。
const MOCK_SECTION_HEADER_HEIGHT_PX: f32 = 26.0;

const CHAR_SIZE_TARGET: f32 = 0.42;
const CHAR_SIZE_TOLERANCE: f32 = 0.05;

fn in_band(ratio: f32) -> bool {
    (ratio - CHAR_SIZE_TARGET).abs() <= CHAR_SIZE_TOLERANCE
}

/// 台帳の主張(1): モック自身の文字寸比は裁定168 の帯の内。
#[test]
fn mock_property_row_character_size_ratio_is_within_the_charter_168_band() {
    let ratio = MOCK_PROPERTY_NAME_FONT_PX / MOCK_PROPERTY_ROW_HEIGHT_PX;
    assert_eq!(
        ratio, 0.44,
        "inspector-library.css の実測比が動いた — 台帳(2026-08-22-inspector-ratio-ledger.md §1b)も更新すること"
    );
    assert!(
        in_band(ratio),
        "モック自身の文字寸比 {ratio} が裁定168 の帯(0.42±0.05)から外れた — \
         台帳の結論(0.55 の白黒判定の前提)が崩れている"
    );
}

/// 台帳の主張(2): 実装(`Dimensions::default()`)の文字寸比は同じ帯の外
/// (`lib.rs` の既存 pin と同じ値・同じ結論 — ここでは Inspector 固有モックの
/// 実測 0.44 と並べて「一致しない」ことまで確認する、両側チェックの本体)。
#[test]
fn implementation_character_size_ratio_still_diverges_from_the_mock_measured_ratio() {
    let dims = Dimensions::default();
    let impl_ratio = dims.body_text / dims.inspector_row_height;
    let mock_ratio = MOCK_PROPERTY_NAME_FONT_PX / MOCK_PROPERTY_ROW_HEIGHT_PX;

    assert_eq!(
        impl_ratio, 0.55,
        "dims.body_text/dims.inspector_row_height が動いた — 意図した是正なら \
         このテスト・`lib.rs` の同型 pin・台帳(§結論)を三箇所とも更新すること"
    );
    assert!(
        !in_band(impl_ratio),
        "FINDING が古い: 実装比 {impl_ratio} は既に裁定168 の帯に入っている — \
         台帳の結論を更新すること"
    );
    assert_ne!(
        impl_ratio, mock_ratio,
        "実装比とモック実測比が一致した — 裁定172 §3 の基準では 0.55 は \
         もう転写対象ではなく合法。台帳(結論・3節)を「モック内」側へ書き換えること"
    );
}

/// 台帳 §3.1: `inspector_value_width`/`inspector_glyph_width` は
/// `inspector_panel_width`(496、旧mock値のまま据え置き)とは噛み合わないが、
/// 転写元 `ui-scale-and-z.html` 自身の pane 幅(300)へ正規化すると旧mockの
/// 値セル幅比とほぼ一致する — 「転写ミスではなく別モックへの正しい転写」
/// という結論を数値で固定する。
#[test]
fn value_and_glyph_width_ratios_match_the_old_mock_only_after_normalizing_to_the_newer_mocks_pane_width() {
    const NEWER_MOCK_PANE_WIDTH_PX: f32 = 300.0; // ui-scale-and-z.html `--pane`

    let dims = Dimensions::default();

    let mock_value_ratio = MOCK_VALUE_CELL_WIDTH_PX / 496.0; // 旧mock自身のpane幅
    let impl_value_ratio_vs_496 = dims.inspector_value_width / dims.inspector_panel_width;
    let impl_value_ratio_vs_300 = dims.inspector_value_width / NEWER_MOCK_PANE_WIDTH_PX;

    assert!(
        (impl_value_ratio_vs_496 - mock_value_ratio).abs() > 0.02,
        "inspector_panel_width(496)を分母に取ると一致してしまう — \
         `inspector_panel_width` が既に300へ更新された可能性、台帳を見直すこと"
    );
    assert!(
        (impl_value_ratio_vs_300 - mock_value_ratio).abs() < 0.01,
        "300px pane 正規化でも旧mockの値セル幅比 {mock_value_ratio} と \
         一致しなくなった(実測 {impl_value_ratio_vs_300}) — \
         台帳 §3.1 の『別モックへの正しい転写』という結論が崩れている"
    );

    let mock_key_ratio = MOCK_KEY_COLUMN_WIDTH_PX / 496.0;
    let impl_glyph_ratio_vs_300 = dims.inspector_glyph_width / NEWER_MOCK_PANE_WIDTH_PX;
    assert!(
        (impl_glyph_ratio_vs_300 - mock_key_ratio).abs() < 0.02,
        "300px pane 正規化での glyph/key 幅比が旧mock比 {mock_key_ratio} から \
         離れすぎた(実測 {impl_glyph_ratio_vs_300}) — 台帳 §3.1 を見直すこと"
    );
}

/// 台帳 §1a/§2: section 見出し帯の高さは分子(26px)こそ両モックで一致するが、
/// 分母となる行高が違う(旧mock25 vs 実装20)ぶんだけ比が動く — この乖離も
/// `inspector_row_height` と同根であることを固定する。
#[test]
fn section_header_height_numerator_matches_but_ratio_diverges_via_row_height() {
    let dims = Dimensions::default();
    assert_eq!(
        dims.inspector_section_header_height, MOCK_SECTION_HEADER_HEIGHT_PX,
        "section 見出し帯の絶対px(分子)が両モック間の一致(26px)から動いた"
    );
    let mock_ratio = MOCK_SECTION_HEADER_HEIGHT_PX / MOCK_PROPERTY_ROW_HEIGHT_PX;
    let impl_ratio = dims.inspector_section_header_height / dims.inspector_row_height;
    assert_ne!(
        mock_ratio, impl_ratio,
        "section_header_height/row_height の比がモックと一致した — \
         inspector_row_height が25へ変わった可能性、台帳 §5-1/5-3 を見直すこと"
    );
}

/// 台帳 §1e/§2: `single_row_horizontal_inset`(0.6em)は既に裁定168 適合済み
/// — この lane で変更していないことの pin(`lib.rs` 側の柵と重複するが、
/// 台帳の「転写ゼロ」という結論を裏付けるための独立確認)。
#[test]
fn value_cell_font_and_row_height_are_unrelated_to_the_0_6em_horizontal_inset_formula() {
    let dims = Dimensions::default();
    // `single_row_horizontal_inset` は `body_text` だけを見る式(裁定168)
    // — `caption_text`/モックの value 文字(9px)は式の入力にならないことを
    // 明示するだけの薄い pin(式自体は `lib.rs` が private のためここから
    // 直接は呼べない — `dims.body_text` が式の唯一の入力であることを
    // ドキュメントとして固定する)。
    assert_eq!(dims.body_text, 11.0);
    assert_eq!(MOCK_VALUE_CELL_FONT_PX, 9.0);
    assert_ne!(dims.body_text, MOCK_VALUE_CELL_FONT_PX);
}
