//! 裁定172 §3 の指示による Inspector 比率台帳の機械照合
//! (`docs/reviews/2026-08-22-inspector-ratio-ledger.md` の実測本体はそちら —
//! この柵はその台帳の主要数値を「実装定数を `use` した両側チェック」として
//! 固定するだけの regression lock、`crates/motolii-shell-iced/tests/
//! css_metrics_oracle.rs` と同型の手口)。
//!
//! ## I-tokens(2026-08-22)による更新
//!
//! I-ratio 台帳が発見した二重モック構造(`inspector_row_height`/
//! `inspector_value_width`/`inspector_glyph_width` は `ui-scale-and-z.html`
//! 由来、`inspector_panel_width` だけ `inspector-library.css` 由来)を、I-tokens
//! レーンが4値とも `next/reference/mocks/inspector-library.html` v3.1(利用者
//! 合格・転写正本統一)へ束で再転写して解消した。この柵は「モックと実装が
//! **一致しない**」ことを固定する pin だったが、再転写後は**一致する**側の
//! pin へ書き換える(0.55 の lock は撤去・0.44 帯内の lock へ)。
//!
//! ## モック側はなぜ literal 定数か(旧 css_metrics_oracle.rs との違い)
//!
//! `next/` workspace は `motolii-ui`(`css_metrics::extract` — Blitz による
//! HTML/CSS の機械抽出器具)を members に含まない(`next/Cargo.toml` 実測 —
//! rerun fork 以外の余計な依存を引かない設計、`next/reference/mocks/
//! ui-scale-and-z.html` 側の egui/skia 切断と同じ理由)。旧 workspace
//! (`crates/`)の `css_metrics_oracle.rs` のように `extract()` を直接呼んで
//! 両側とも「実物」にすることはできない — モック側は
//! `next/reference/mocks/inspector-library.html`(旧 `docs/mocks-ui/public/
//! inspector-library.css` から寸法・レイアウトは逐語移植 — 値は同一)からの
//! 引用リテラル(行番号を doc comment に明記)として pin し、実装側だけ
//! `motolii_tokens_rs::Dimensions::default()` を `use` して読む。**実装側が
//! 変われば必ず落ちる**ので「両側チェック」の意味は保っている(モック側が
//! 動くのは人間がこの css ファイルを編集した時だけ — その時はこのテストの
//! 定数も手で更新する)。
//!
//! ## この柵が固定する主張(I-tokens 後)
//!
//! (1) `inspector-library.css` 実測の `body_text 相当 / row_height 相当`
//! (`.propertyName span` 11px / `.propertyRow` min-height 25px)= 0.44 は
//! 裁定168 の帯(0.42±0.05)の**内**。
//! (2) 現行実装の `Dimensions::default().theme().text.body / .inspector_row_height`
//! も **同じ 0.44**(`inspector_row_height` を I-tokens で 25 へ再転写した
//! ため)— 同じ帯の**内**(`lib.rs` の
//! `inspector_character_size_ratio_is_locked_within_the_charter_168_band`
//! と同じ pin、ここでは Inspector 固有モックの実測値と並べて再確認する)。
//! (3) 両者が一致する ==「φ FINDING(0.55)は I-tokens の束再転写で根治された」
//! の機械的な証拠。

use motolii_tokens_rs::Dimensions;

/// `next/reference/mocks/inspector-library.css:375-382` `.propertyName span`
/// — 値行ラベル文字(`dims.theme().text.body` の実装対応)。**I-tokens(2026-08-22)**:
/// 出典パスを旧 `docs/mocks-ui/public/inspector-library.css` から転写正本
/// (`next/reference/mocks/`、利用者合格 v3.1)へ更新 — 値・行番号とも寸法は
/// 逐語移植(無改変)なので数値そのものは不変。
const MOCK_PROPERTY_NAME_FONT_PX: f32 = 11.0;

/// 同 css:300-306 `.propertyRow{min-height:25px}` — 値行の行高
/// (`dims.inspector_row_height` の実装対応)。
const MOCK_PROPERTY_ROW_HEIGHT_PX: f32 = 25.0;

/// 同 css:412-426 `.valueCell input` — 値本体の文字(既定フォント。旧mockの
/// monospace 指定は実装側では未使用 — `value_cell` 参照)。
const MOCK_VALUE_CELL_FONT_PX: f32 = 9.0;

/// 同 css:142 `grid-template-columns: minmax(132px,1fr) repeat(3,64px)
/// 26px` — 値セル(X/Y/Z 1つぶん)幅。
const MOCK_VALUE_CELL_WIDTH_PX: f32 = 64.0;

/// 同上、グリッド末尾 `26px` — Key 列幅。
const MOCK_KEY_COLUMN_WIDTH_PX: f32 = 26.0;

/// 同 css:44 `.inspectorShell{width:min(100%,496px)}` — pane 幅
/// (`dims.inspector_panel_width` の実装対応)。**I-tokens(2026-08-22)**:
/// 束の4値目 — 旧台帳(裁定172 §3)時点でも既にこの値だったが、他3値
/// (row_height/value_width/glyph_width)と別モック由来だったため噛み合わ
/// なかった(§3.1)。4値とも同じ mock 由来へ揃えたことで直接比較できる。
const MOCK_PANEL_WIDTH_PX: f32 = 496.0;

/// 同 css:165-166 `.tableSection h2{height:26px}` — section 見出し帯高
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

/// 台帳の主張(2、I-tokens 後): 実装(`Dimensions::default()`)の文字寸比は
/// モック実測比と**一致し**、同じ帯の内にある(`lib.rs` の既存 pin と同じ値・
/// 同じ結論 — ここでは Inspector 固有モックの実測 0.44 と並べて「一致する」
/// ことまで確認する、両側チェックの本体)。**旧テスト名(`..._still_diverges_
/// from_the_mock_measured_ratio`)は I-tokens の再転写で前提が崩れたため
/// 反転**(0.55 の lock は撤去)。
#[test]
fn implementation_character_size_ratio_now_matches_the_mock_measured_ratio() {
    let dims = Dimensions::default();
    let impl_ratio = dims.theme().text.body / dims.inspector_row_height;
    let mock_ratio = MOCK_PROPERTY_NAME_FONT_PX / MOCK_PROPERTY_ROW_HEIGHT_PX;

    assert_eq!(
        impl_ratio, 0.44,
        "dims.theme().text.body/dims.inspector_row_height が動いた — I-tokens の \
         再転写値(0.44)から動いたなら、このテスト・`lib.rs` の同型 pin・台帳を \
         三箇所とも更新すること"
    );
    assert!(
        in_band(impl_ratio),
        "実装比 {impl_ratio} が裁定168 の帯(0.42±0.05)から外れた — \
         I-tokens の再転写(inspector_row_height=25)が崩れている疑い"
    );
    assert_eq!(
        impl_ratio, mock_ratio,
        "実装比とモック実測比が一致しなくなった — I-tokens の束再転写 \
         (inspector_row_height を inspector-library.css 実測値25へ)が \
         崩れている疑い、台帳・FINDING を見直すこと"
    );
}

/// 台帳 §3.1 → I-tokens 後: `inspector_value_width`/`inspector_glyph_width`
/// は旧(裁定172 §3 時点)は `inspector_panel_width`(496)とは別モック由来で
/// 噛み合わなかった(300px pane 正規化でしか辻褄が合わなかった、旧テスト名
/// `..._match_the_old_mock_only_after_normalizing_to_the_newer_mocks_pane_
/// width` はその状態を固定していた)。**I-tokens(2026-08-22)で4値とも同じ
/// mock(inspector-library v3.1)へ束で再転写した**結果、`inspector_value_
/// width`/`inspector_glyph_width` は今度こそ `inspector_panel_width`(496、
/// 同じく inspector-library v3.1 由来)を分母に取った時点でモック実測比と
/// **直接一致する**(300px 正規化という迂回が不要になった) — この解消を固定する。
#[test]
fn value_and_glyph_width_ratios_match_the_mock_directly_against_the_496_panel_width() {
    let dims = Dimensions::default();

    let mock_value_ratio = MOCK_VALUE_CELL_WIDTH_PX / MOCK_PANEL_WIDTH_PX;
    let impl_value_ratio = dims.inspector_value_width / dims.inspector_panel_width;
    assert_eq!(
        impl_value_ratio, mock_value_ratio,
        "inspector_value_width/inspector_panel_width がモック比 \
         {mock_value_ratio} と一致しない(実測 {impl_value_ratio}) — \
         I-tokens の束再転写(value_width=64, panel_width=496 とも \
         inspector-library v3.1 由来)が崩れている疑い"
    );

    let mock_key_ratio = MOCK_KEY_COLUMN_WIDTH_PX / MOCK_PANEL_WIDTH_PX;
    let impl_glyph_ratio = dims.inspector_glyph_width / dims.inspector_panel_width;
    assert_eq!(
        impl_glyph_ratio, mock_key_ratio,
        "inspector_glyph_width/inspector_panel_width がモックの Key 列幅比 \
         {mock_key_ratio} と一致しない(実測 {impl_glyph_ratio}) — \
         I-tokens の束再転写(glyph_width=26, panel_width=496 とも \
         inspector-library v3.1 由来)が崩れている疑い"
    );
}

/// I-tokens が根治したことの直接証拠: 旧台帳(§3.1)の「300px pane 正規化を
/// 経由しないと一致しない」という迂回そのものがもう要らないことを固定する
/// (300 正規化した比と496直接比の両方が今度は同じモック比に一致する —
/// 二重モック構造が解消されたので、どちらの分母を使っても同じ結論に着地する)。
#[test]
fn the_300px_pane_workaround_is_no_longer_needed_after_the_i_tokens_unification() {
    const OLD_UI_SCALE_AND_Z_PANE_WIDTH_PX: f32 = 300.0;
    let dims = Dimensions::default();

    let mock_value_ratio = MOCK_VALUE_CELL_WIDTH_PX / MOCK_PANEL_WIDTH_PX;
    let impl_value_ratio_vs_496 = dims.inspector_value_width / dims.inspector_panel_width;
    let impl_value_ratio_vs_300 = dims.inspector_value_width / OLD_UI_SCALE_AND_Z_PANE_WIDTH_PX;

    assert_eq!(
        impl_value_ratio_vs_496, mock_value_ratio,
        "496 を分母に取った比がモック比と一致しない — 束再転写が崩れている疑い"
    );
    assert_ne!(
        impl_value_ratio_vs_300, mock_value_ratio,
        "300px(旧 ui-scale-and-z.html 正規化)を分母に取った比が496直接比と \
         同じ結論に着地してしまった — 二重モック構造の痕跡が再発した疑い \
         (I-tokens は 300px 正規化に依存しない構造へ揃えたはず)"
    );
}

/// 台帳 §1a/§2 → I-tokens 後: section 見出し帯の高さは分子(26px)がもとから
/// 両モックで一致していたが、分母の行高が違っていた(旧mock25 vs 実装20)ため
/// 比がズレていた。I-tokens で `inspector_row_height` を25へ束再転写した結果、
/// 分子(26)・分母(25)とも同じ mock 由来になり、比そのものが一致する
/// (旧テスト名 `..._numerator_matches_but_ratio_diverges_via_row_height` は
/// もう成立しないため反転)。
#[test]
fn section_header_height_ratio_now_matches_the_mock_via_the_re_transcribed_row_height() {
    let dims = Dimensions::default();
    assert_eq!(
        dims.inspector_section_header_height, MOCK_SECTION_HEADER_HEIGHT_PX,
        "section 見出し帯の絶対px(分子)が両モック間の一致(26px)から動いた"
    );
    let mock_ratio = MOCK_SECTION_HEADER_HEIGHT_PX / MOCK_PROPERTY_ROW_HEIGHT_PX;
    let impl_ratio = dims.inspector_section_header_height / dims.inspector_row_height;
    assert_eq!(
        mock_ratio, impl_ratio,
        "section_header_height/row_height の比がモックと一致しなくなった — \
         I-tokens の再転写(inspector_row_height=25)が崩れている疑い、\
         台帳 §5-1/5-3 を見直すこと"
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
    // 直接は呼べない — `dims.theme().text.body` が式の唯一の入力であることを
    // ドキュメントとして固定する)。
    assert_eq!(dims.theme().text.body, 11.0);
    assert_eq!(MOCK_VALUE_CELL_FONT_PX, 9.0);
    assert_ne!(dims.theme().text.body, MOCK_VALUE_CELL_FONT_PX);
}
