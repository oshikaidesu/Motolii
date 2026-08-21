# Browser pane 比率台帳 — browser-library.{html,css} 実測(利用者裁定 2026-08-22 朝)

日付: 2026-08-22 / 状態: **実測+実装追随**(Inspector I-ratio→I-tokens の2段を
1レーンで実施。`docs/reviews/2026-08-22-inspector-ratio-ledger.md` と同型の
手口 — B3 が「構造のみ借用・比率は自前宣言」したため実窓がモックと別物に
見えている、という利用者実機不合格への直接対応)

## 結論(先頭)

**B3(`next/ui/motolii-browser-pane/src/lib.rs`)の視覚定数のうち、rail 行/
filter チップ/検索欄/結果件数の文字が `caption_text`(9px、自前判断)を使って
いたが、`browser-library.css` 実測ではこの階層は例外なく `micro_text`
(8px)である。加えてボタン類(rail 行・filter チップ)に padding/radius が
一切指定されておらず iced 既定値に落ちていた** — 転写対象。**card grid の
文字(`micro_text`)・サムネ縦横比(16/9)・card 内 padding(`spacing_xs`)は
既に一致**(転写不要、3節)。**card 幅・rail:catalog 比・card 間 gap は
`next/shell/motolii-shell/src/screenshot.rs` が同じ定数を直読みして矩形近似を
描いており(shell 側は ALLOWLIST 外)、ここを動かすと screenshot 器具との
二重保守/desync を作る — 転写対象外・FINDING として記録**(4節)。

## 0. 転写元の確認

`next/ui/motolii-browser-pane/src/lib.rs` 冒頭 doc(B3 節)は転写元を
`browser-library.html`(`.thumbnailGrid`/`.libraryCard`/`.libraryThumb`/
`.cardCopy` の構造)と自己申告している。本台帳はその申告どおり
`next/reference/mocks/browser-library.css`(旧 `docs/mocks-ui/public/
browser-library.css` から寸法無改変で移植済み、ファイル冒頭コメント参照)を
実測する。この crate が**実装していない**要素(browserHeader/browserToolbar/
libraryTabs/`.librarySidebar h2`/catalogHeader/viewModes/selectionTray/
tagEditor/contextMenu — B3 doc 「予約地」節で明記済み)は照合対象外(5節に
一覧だけ残す)。

## 1. モック実測台帳(`next/reference/mocks/browser-library.css`)

denominator は各値の並びで明記。この crate は既に `CARD_WIDTH_ROW_HEIGHT_
RATIO`/`PANEL_HEIGHT_ROW_HEIGHT_RATIO` を **分母 = `Dimensions::row_height`
(既定20px)** で宣言済み(`lib.rs:76-85,297-303` doc)なので、本台帳もこの crate
独自の慣例(裁定165「形は比率で定数化・分母明記」)を踏襲し、分母は
一貫して `row_height` を使う。

### 1a. この crate が実装する要素の高さ・幅・padding・font-size

| 要素 | css(行番号) | 値 | 備考 |
|---|---|---|---|
| `.librarySidebar`(rail 全体) | css:111-118 | `width:112px; padding:2px 0 6px` | 横 padding は**0**(行側が持つ) |
| `.locationRow`(rail 1行、`scope_button` 対応) | css:136-151 | `height:19px; padding:2px 6px 0; font-size:8px; line-height:1` | |
| `.locationRow.indent` | css:154 | `padding-left:12px` | 未実装(サブ階層なし、5節) |
| `.filterShelf`(filter shelf 全体) | css:183-192 | `min-height:24px; gap:2px; padding:2px 4px` | |
| `.filterShelf button` / `.clearFilter`(filter チップ・Clear、`scope_button`/Clear 対応) | css:201-212,229 | `min-height:17px; padding:2px 4px; border-radius:8px; font-size:8px` | Clear は同じルールを継承(html:484 `class="clearFilter"` は `.filterShelf button` と同一セレクタに同居) |
| `#library-search`(検索欄) | css:74-84 | `height:21px; padding:0 6px; font-size:8px` | |
| `.resultSummary`(結果件数、"Results N" 対応) | css:217-226 | `height:21px; padding:0 6px` / `strong{font-size:8px}` `span{font-size:8px}` | |
| `.thumbnailGrid`(card grid) | css:228-236 | `grid-template-columns:repeat(2,1fr); gap:0; padding:0 1px 3px` | 既定2列 = `GRID_COLUMNS`(実装2)と一致 |
| `.libraryCard` | css:238-247 | `padding:2px; border-radius:0` | |
| `.libraryThumb` | css:253-260 | `aspect-ratio:16/9; padding:2px; border:1px` | |
| `.libraryThumb b`(サムネ内グリフ、通常種別) | css:264 | `font-size:8px` | `.thumb-create`/`.thumb-cyan` の20pxは意図的な例外(css:273-279 コメント明記、{8,9,11,12}帯の唯一の例外) |
| `.cardCopy strong`(カード名) | css:281-283 | `font-size:8px; margin-top:2px` | |
| `.cardCopy small`(カード caption) | css:281,284 | `font-size:8px` | |

### 1b. 比率換算(分母 = `row_height` 20px)

| 項目 | mock px | /20 | 実装(旧) | 実装(新) | 判定 |
|---|---|---|---|---|---|
| rail 行文字 | 8 | 0.40 | `caption_text`=9(0.45) | `micro_text`=8(0.40) | **転写**(2節) |
| filter チップ文字 | 8 | 0.40 | `caption_text`=9 | `micro_text`=8 | **転写** |
| 検索欄文字 | 8 | 0.40 | `caption_text`=9 | `micro_text`=8 | **転写** |
| 結果件数文字 | 8 | 0.40 | `caption_text`=9 | `micro_text`=8 | **転写** |
| filter チップ角丸 | 8 | 0.40 | `0.0`(未指定) | `FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO`(0.4)×row_height=8 | **転写**(新ローカル比率定数) |
| rail 行角丸 | 0(指定なし) | 0 | `0.0` | `0.0`(不変) | 適合(変更不要) |
| card padding | 2 | 0.10 | `spacing_xs`=2 | 不変 | 適合(既に一致) |
| card 名/caption 文字 | 8 | 0.40 | `micro_text`=8 | 不変 | 適合(既に一致) |
| サムネ縦横比 | 16/9 | — | `THUMB_ASPECT_W/H`=16/9 | 不変 | 適合(既に一致) |

### 1c. padding(裁定167 型の梯子ではなく、この crate の既存慣例=既存 spacing token の中から最近傍を選ぶ)

`lib.rs` 冒頭 doc(32-43行)が明記するとおり、この crate は**新しい padding
段を発明しない**方針(裁定167 ラダーの新設ではなく既存 `spacing_xs/spacing_s`
(+今回追加で`spacing_m`も rail 行に採用、後述)の中から選ぶ)。この方針自体は
維持し、**どのトークンを選ぶか**だけを実測へ合わせ直す。

| 要素 | mock padding | 最近傍 token(候補) | 選択 | 根拠 |
|---|---|---|---|---|
| rail 行(`.locationRow`) | `2px 6px 0` | 垂直: `spacing_xs`(2、一致) / 水平: `spacing_s`(4)と`spacing_m`(8)が同着(共に差2) | `[spacing_xs, spacing_m]` | 同着タイブレーク=**このリポジトリ全体で単行ボタンの定番の組**(`motolii-inspector-pane`・`motolii-settings-pane`×4・`motolii-stage-pane`・`motolii-shell` が同一の `[dims.spacing_xs, dims.spacing_m]` を採用済み、grep実測7箇所)。crate 間の慣例と揃える |
| filter チップ/Clear(`.filterShelf button`) | `2px 4px` | 垂直: `spacing_xs`(2、一致) / 水平: `spacing_s`(4、**一致**) | `[spacing_xs, spacing_s]` | 完全一致、タイブレーク不要 |
| `.librarySidebar` 自体(rail 容器) | `2px 0 6px`(横0) | 垂直: `spacing_xs`(2、上端と一致) / 水平: 0 | `[spacing_xs, 0.0]` | 横 padding は行側(上記)が持つので容器側は0が正しい転写。旧実装は容器に一律 `spacing_s`(4)を掛けており、行側 padding が今回新設されると横方向が二重(容器4+行8=12)になる不整合を持っていた — 転写で解消 |

## 2. 転写した差分(`next/ui/motolii-browser-pane/src/lib.rs`)

1. **文字寸(4箇所)**: `filter_shelf_view` 検索欄・`scope_button`(rail 行/
   filter チップ共有)・Clear ボタン・`catalog_view` 結果件数 —
   `dims.caption_text`(9)→`dims.micro_text`(8)。mock はこの階層を例外なく
   8px にしている(1a節)。
2. **filter チップ/Clear の角丸**: 新ローカル比率定数
   `FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO = 0.4`(`× row_height` で
   既定8px、mock `border-radius:8px` と厳密一致)を追加し、`chip_style` に
   `radius` 引数を足して `scope_button`(filter チップ呼び出し側)・Clear
   ボタンへ適用。rail 行呼び出し側は `0.0`(mock `.locationRow` に角丸指定
   なし = 直角)のまま。
3. **padding(3箇所)**: rail 行/filter チップ/Clear ボタンに `.padding(...)`
   を明示追加(旧実装は無指定 = iced 既定値に依存していた)。rail 行容器
   (`rail_view` の外側 `container`)の padding を `spacing_s` 一律から
   `[spacing_xs, 0.0]` へ変更(1c節、二重 padding の解消)。

## 3. 転写不要(既に一致)

- card 内 padding(`spacing_xs`=2、mock `.libraryCard{padding:2px}` と一致)
- card 名/caption 文字(`micro_text`=8、mock `.cardCopy strong/small
  {font-size:8px}` と一致)
- サムネ縦横比(`THUMB_ASPECT_W/H`=16/9、mock `.libraryThumb{aspect-
  ratio:16/9}` と一致)
- grid 列数(`GRID_COLUMNS`=2、mock 既定 `data-view` 未指定時の
  `repeat(2,minmax(0,1fr))` と一致)
- サムネ内グリフ文字(`micro_text`=8、mock `.libraryThumb b{font-size:8px}`
  通常種別ルールと一致。`.thumb-create`/`.thumb-cyan` の20pxはmock自身が
  「{8,9,11,12}帯からの意図的な例外」と明記する装飾グリフなので対象外)

## 4. 転写対象外(FINDING — shell 側の直読みでロックされている)

**この crate の `pub const`(`CARD_WIDTH_ROW_HEIGHT_RATIO`/`PANEL_HEIGHT_ROW_
HEIGHT_RATIO`/`GRID_COLUMNS`/`THUMB_ASPECT_W`/`THUMB_ASPECT_H`)と共有
`Dimensions` token(`spacing_xs`/`spacing_s`)の一部は、`next/shell/
motolii-shell/src/screenshot.rs`(ALLOWLIST 外・NON-GOALS「shell 変更」)が
**同じ定数/トークンを直接読んで矩形近似を描いている**ため、値だけを動かすと
実 widget と screenshot 器具が desync する。実測との乖離があっても、この
レーン単独では安全に閉じられない(Inspector 台帳の「tokens crate 側の話、
ALLOWLIST 外」と同じ構造の FINDING)。

1. **rail:catalog 比**: 実装は `FillPortion(1)`(rail)/`FillPortion(4)`
   (catalog)(`lib.rs:102,117,152` — `Length::FillPortion`)。mock は
   `.librarySidebar{width:112px}` 固定+catalog可変(700px全体からの残り
   588px、112:588 ≈ 1:5.25)で、現在の1:4より1:5.25の方が近い。しかし
   `screenshot.rs:508`(`let rail_w = area.w / 5.0;`)は**現行の1:4を前提に
   ハードコード**されている(1/(1+4)=1/5 の近似式) — このレーンで
   `FillPortion` 比を変えると shell 側のコメントも数式も追随できず
   desync する。**転写しない**、位置レーンへの引き渡し事項(RETURN 参照)。
2. **card 間 gap**: 実装は `card_grid_view` の行内/行間とも `dims.
   spacing_s`(4px、`lib.rs:337,341`)。mock `.thumbnailGrid{gap:0}` は
   0px — 乖離しているが、`screenshot.rs:516,523,529` が `dims.spacing_s`
   を**同じ役割で直接読んで**カード矩形間隔を計算しており、ここも
   同型の desync リスク。**転写しない**。
3. **card 幅そのもの(絶対px)**: mock は `.libraryCard` に固定幅を持たず
   `.thumbnailGrid{grid-template-columns:repeat(2,minmax(0,1fr))}` で
   catalog 幅(700px全体からの残り)を等分する**可変幅**。実装は
   `card_width = dims.row_height × CARD_WIDTH_ROW_HEIGHT_RATIO`(固定
   120px、`lib.rs:352`)— pane 全体の実幅(700px)は「位置レーン」が
   docking/ウィンドウ幅として決める領分であってこの crate の値ではない
   (CURRENT STATE 節「モックの幾何(700px幅×全高の縦パネル)は位置レーンへの
   引き渡し情報」)。したがって「モックの実幅700pxから逆算した絶対px」を
   このレーンで転写する行為自体が筋違い — Inspector 台帳の
   `inspector_panel_width`(496→300裁定待ち)と同型の「上位裁定待ち」FINDING
   として記録するに留める。`CARD_WIDTH_ROW_HEIGHT_RATIO`(6.0)は変更しない。

## 5. 対象外(この crate が実装していない要素、5節参照 = B3 doc「予約地」)

`browserHeader`/`browserToolbar`(履歴ボタン・ツールバーボタン)/
`libraryTabs`/`.librarySidebar h2`(LIBRARY見出し)/`.locationRow.indent`
(サブ階層インデント)/`catalogHeader`/`.viewModes`(表示切替)/
`selectionTray`/`tagEditor`/`contextMenu` — B3 doc(`lib.rs:27-30`)が
明記する予約地のまま。視覚正本としては記録するが、この波では転写しない
(NON-GOALS「要素の増減はしない」)。

## オラクル

`next/ui/motolii-browser-pane/tests/browser_ratio_ledger.rs` — 本台帳の主要
主張を pin する(inspector_ratio_ledger.rs と同型、モック側は css 引用
リテラル・実装側は `Dimensions::default()` を `use` する両側チェック):

1. `FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO × row_height == 8.0`(mock
   `border-radius:8px` の pin)。
2. rail 行文字・filter チップ文字・検索欄文字・結果件数文字が **全て
   `micro_text`(8) を使い、`caption_text`(9) は使わない**ことを、
   `iced_test::simulator` で実レンダリングした `Target::Text` の bounds
   高さを card copy(既に `micro_text` 確定済みの基準)と比較する形で固定
   (rail_filter_atlas.rs と同じ atlas 手口)。
