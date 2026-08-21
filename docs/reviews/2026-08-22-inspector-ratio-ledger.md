# Inspector 比率台帳 — φ FINDING「文字寸比 0.55」の白黒(裁定172 §3)

日付: 2026-08-22 / 状態: **観察+FINDING**(裁定172 §3 の指示どおり機械照合はしたが、
dims 定数そのものの書き換えは ALLOWLIST 外 — 詳細は末尾)

## 結論(先頭)

**0.55 は `docs/mocks-ui/public/inspector-library.*` 実測でも帯の外 —
ただし直す手段(`inspector_row_height` 等)はこのレーンの ALLOWLIST
(`next/ui/motolii-inspector-pane/**`)の外にある。**

- 裁定172 §3 の名指しどおり `inspector-library.css` を実測すると、`.propertyName
  span`(値行のラベル文字、`dims.body_text` の実装対応)= **11px**、
  `.propertyRow`(値行の行高、`dims.inspector_row_height` の実装対応)=
  **min-height 25px** → 比 = **11/25 = 0.44**。これは裁定168 の帯
  (0.42±0.05 = 0.37〜0.47)の**内**。
- 実装の実値は `body_text(11) / inspector_row_height(20) = 0.55` — この
  **0.44 と一致しない**。よって裁定172 §3 の基準(「0.55 がモック内なら合法・
  外なら転写対象」)に照らすと 0.55 は**モック外 = 名目上は転写対象**。
- しかし `inspector_row_height` は `next/ui/motolii-tokens-rs/tokens/
  dimensions.json` にあり、この発注書の ALLOWLIST(`next/ui/
  motolii-inspector-pane/src/**`・`tests/**`・本ドキュメント・README 1行)には
  **入っていない** — この lane からは書き換えられない。加えて後述のとおり
  `inspector_row_height`(20)は単独の値ではなく `inspector_value_width`(38)・
  `inspector_glyph_width`(18)と三点で束になった**別モック**
  (`next/reference/mocks/ui-scale-and-z.html`)からの転記であり、`inspector_
  panel_width`(496、`.inspectorShell{width:min(100%,496px)}` 由来、300 への
  変更は利用者裁定待ちと `dimensions.json` に明記済み)ともまだ整合していない
  途中状態。単独で `inspector_row_height` だけ 25 へ戻すと、38px の値セル幅に
  25px の行高という**どちらのモックにも無い組み合わせ**になり、悪化する。
  → **FINDING として記録し、転写しない**(NON-GOALS「行の構成変更」に該当)。

## 0. 最重要の発見: 実装が実際に転写元にしているモックは `inspector-library.*` ではない

台帳を起こす過程で判明した、この照合結果全体を読む上での前提: `next/ui/
motolii-inspector-pane/src/lib.rs` のコード comment は一貫して `next/
reference/mocks/ui-scale-and-z.html` を「視覚正本」と自己申告している
(`view_with_speed_draft` 冒頭・`ident_band`・`value_cell_height` 等、複数箇所)。
実際、この crate が実装する行構成(ptitle → ident → cols → sec+prow×N → hint)
は `ui-scale-and-z.html` の Inspector 断片(`.ptitle`/`.ident`/`.cols`/`.sec`+
`.prow`/`.hint`)と**1:1で一致**する一方、`inspector-library.*` にある
mode tabs・selection summary(46px の広い ident 帯)・effect stack(FX
badge/group/context menu)・extension tabs・notes panel は**どれも未実装**
(Q0 スコープ外)。`dimensions.json` の該当 `_note_*` も `inspector_row_height`/
`inspector_value_width`/`inspector_glyph_width`/`caption_text`/`body_text`/
`inspector_section_header_height` の出典を明示的に `ui-scale-and-z.html`
と書いている(`inspector_panel_width` だけ例外 — `inspector-library.css`
から転記のまま据え置き)。

したがって以下の照合表で「モック外」と出る項目の**大半**は、φ の 0.55 と
同根 — 「実装が従っている実際のモックが、裁定172 §3 が名指した
`inspector-library.*` とは別物(より単純な Q0 スコープの新モック)である」
という構造的な事情であって、実装が誤って値を書き違えたわけではない。
このことは 5 節「見送った差分」の判断根拠として効いている。

## 1. モック実測台帳(`docs/mocks-ui/public/inspector-library.{html,css}`)

分母は明記のとおり。「行高」は文脈の帯自身の高さ(`.propertyRow`=25 が
台帳の基準行)。

### 1a. 帯・行の高さ(分母=`.propertyRow` 25px)

| 帯 | css | px | 比(/25) |
|---|---|---|---|
| panelHeader(shell 見出し帯) | `.panelHeader{height:29px}` | 29 | 1.16 |
| modeTabs | `.modeTabs{height:28px}` | 28 | 1.12 |
| selectionSummary(ident 相当) | `.selectionSummary{height:46px}` | 46 | 1.84 |
| columnHeader | `.columnHeader{height:21px}` | 21 | 0.84 |
| tableSection h2(section 見出し) | `.tableSection h2{height:26px}` | 26 | 1.04 |
| **propertyRow(基準行)** | `.propertyRow{min-height:25px}` | 25 | **1.00** |
| effectStackToolbar | `min-height:22px` | 22 | 0.88 |
| advancedToggle | `min-height:22px` | 22 | 0.88 |
| effectGroupHeader | `min-height:22px` | 22 | 0.88 |
| footer | `min-height:26px` | 26 | 1.04 |

### 1b. 文字寸(分母=行高25、および自帯高)

| 要素 | css | px | /25 | /自帯高 |
|---|---|---|---|---|
| panelHeader タイトル(`strong`) | `font-size:12px` | 12 | 0.48 | 12/29=0.414 |
| panelHeader 注記(`span`) | 9px | 9 | 0.36 | 9/29=0.310 |
| selectionSummary 名前(`strong`) | 11px | 11 | 0.44 | 11/46=0.239 |
| selectionSummary 種別(`span`) | 9px | 9 | 0.36 | 9/46=0.196 |
| columnHeader | 8px | 8 | 0.32 | 8/21=0.381 |
| tableSection h2(見出し) | 8px | 8 | 0.32 | 8/26=0.308 |
| **propertyName span(値行ラベル = `dims.body_text` 対応)** | **11px** | 11 | **0.44** | 11/25=0.44 |
| propertyName small(注記) | 8px | 8 | 0.32 | — |
| **valueCell input(値本体、monospace)** | **9px** | 9 | **0.36** | 9/25=0.36 |
| keyButton/keyPlaceholder | 9px | 9 | 0.36 | — |
| footer | 9px | 9 | 0.36 | 9/26=0.346 |

**φ FINDING の対応ペアはここの太字2行** — `propertyName span`(11、`dims.
body_text` が実際にこのラベルへ使われる)/ `propertyRow`(25、`dims.
inspector_row_height` が実際にこの行の高さへ使われる)。比 = **0.44**。

なお `inspector-library.css` は 2026-08-19(コミット `a3b37f63`)に文字寸を
`{8,9,11,12}` の平坦文法へ既に統一済み(`propertyName span` はこの回で
10px→11px に修正されている) — 本台帳はこの最新状態を実測した。

### 1c. 値セル幅・key/glyph 列幅(分母=行高25、参考に pane 幅496も併記)

| 要素 | css 出所 | px | /行高25 | /pane幅496 |
|---|---|---|---|---|
| 値セル(X/Y/Z 1つぶん) | `grid-template-columns: minmax(132px,1fr) repeat(3,64px) 26px` | 64 | 2.56 | 0.129 |
| Key 列 | 同上、末尾 `26px` | 26 | 1.04 | 0.052 |

### 1d. padding/gap(裁定167 梯子 {0.30, 0.15, 0.075}×行高、分母=自帯高)

| 要素 | css | px | /自帯高 | 最近傍段 |
|---|---|---|---|---|
| panelHeader 横 padding | `padding:0 8px` | 8 | 8/29=0.276 | 0.30(誤差0.024) |
| panelHeader gap | `gap:7px` | 7 | 7/29=0.241 | 0.30(誤差0.059) |
| selectionSummary 縦 padding | `padding:6px 8px` | 6 | 6/46=0.130 | 0.15(誤差0.020) |
| selectionSummary 横 padding | 同上 | 8 | 8/46=0.174 | 0.15(誤差0.024) |
| selectionSummary gap | `gap:8px` | 8 | 8/46=0.174 | 0.15(誤差0.024) |
| tableSection h2/sectionToggle 横 padding | `padding:0 8px` | 8 | 8/26=0.308 | 0.30(誤差0.008) |
| sectionToggle gap | `gap:5px` | 5 | 5/26=0.192 | 0.15(誤差0.042) |
| propertyName 右 padding | `padding-right:8px` | 8 | 8/25=0.32 | 0.30(誤差0.02) |
| propertyName gap | `gap:7px` | 7 | 7/25=0.28 | 0.30(誤差0.02) |
| footer 横 padding | `padding:0 8px` | 8 | 8/26=0.308 | 0.30(誤差0.008) |
| footer gap | `gap:6px` | 6 | 6/26=0.231 | 0.30(誤差0.069) |

**判定**: 概ね 0.30 段(次点 0.15 段)への最近傍寄せで説明がつく — 裁定167 の
「梯子は比率最近傍選択」の枠内。selectionSummary は帯自体が飛び抜けて高い
(46px)ため 0.15 段に寄る例が多い(Timeline側の実測でも見られたパターンと
同型)。propertyName の左 padding(11px = 8px 基調 + 3px アクセントバー幅)
のように、装飾要素(帯)を内包する値は素直に段へ乗らない — これも Timeline
実測の rail 8/26 系と同じ「装飾オフセットぶんのズレ」。

### 1e. 裁定168 em族(単行の横余白0.6em・縦0.3em・固定行高はboxセンタリング)適合

- **box センタリング**: `tableSection h2`(高さ26固定、`padding:0`)・
  `footer`(min-height26、`align-items:center`、横paddingのみ)は「固定行高の
  単行はboxセンタリング(縦padding=0)」に合致。
- **横em系**: `valueCell input` の右 padding 実質6px(unit有りセルの
  `padding-right:15px` は unit記号ぶんの別枠なので除く)/ 値文字9px =
  0.667em — 0.6em系に近い。ただし `propertyName` の 8px 右padding/11px
  ラベル文字 = 0.727em で、0.6em からはやや離れる(このpaddingは icon+gap
  も内包する帯全体のinsetであって、文字だけの余白ではないため単純比較の
  対象として弱い、参考値扱い)。
- **縦em系の直接対応は無い** — `inspector-library.css` の縦paddingは
  box-centering(0)か帯全体の枠(selectionSummary の6px、ident的な複合要素の
  外枠であって「単行の縦余白」ではない)のどちらかで、単行テキスト+可変高さ
  +0.3em、という組み合わせの実例がこの mock には無い。

## 2. 実装比率(`Dimensions::default()`、`next/ui/motolii-tokens-rs/src/lib.rs`)

| dims | 値 | /`inspector_row_height`(20) | /`inspector_panel_width`(496、参考) |
|---|---|---|---|
| `title_text` | 12 | 0.60 | — |
| `body_text` | 11 | **0.55** | — |
| `caption_text` | 9 | 0.45 | — |
| `inspector_row_height` | 20 | 1.00 | 0.040 |
| `inspector_section_header_height` | 26 | 1.30 | 0.052 |
| `inspector_value_width` | 38 | 1.90 | 0.077(pane300なら0.127) |
| `inspector_glyph_width` | 18 | 0.90 | 0.036(pane300なら0.06) |
| `spacing_xs`/`spacing_s`/`spacing_m`/`spacing_l` | 2/4/8/12 | 0.10/0.20/0.40/0.60 | — |

局所式(lib.rs、dims 由来の派生 — ALLOWLIST 内):

| 式 | 値(既定dims) | 裁定 | 判定 |
|---|---|---|---|
| `single_row_horizontal_inset(body_text)` = `round(body_text*0.6)` | round(11*0.6)=7 | 168 横0.6em | **適合**(式そのものが0.6em、テスト済み) |
| `sibling_gap_px(row_height)` = `round(row_height*0.075)` | round(20*0.075)=2 (=1.5→2丸め) | 167 下段0.075 | **適合**(テスト済み) |
| `value_cell_height` = `row_height - spacing_s` | 20-4=16 | — (ui-scale-and-z.html `.prow .v` 式と同型) | inspector-library.* に対応式なし(grid stretch) |
| `glyph_height` = `row_height - spacing_xs` | 20-2=18 | — (同上) | 同上 |

## 3. 照合表(モック比 vs 実装比)

| 項目 | モック比(inspector-library.*) | 実装比 | 一致 | 分類 |
|---|---|---|---|---|
| **body_text/row_height**(φ本体) | **0.44**(11/25) | **0.55**(11/20) | ✗ | **FINDING**(dims、ALLOWLIST外+3点セット) |
| title_text/row_height | 0.48(12/25、panelHeader文脈) | 0.60(12/20) | ✗(分母となる行が別物) | 対象外(帯の役割が違う、5節参照) |
| caption_text/row_height | 0.32〜0.36(8または9、要素により相違) | 0.45(9/20) | ✗ | FINDING(同上、かつ8/9のどちらを取るかは意匠選択) |
| inspector_section_header_height/row_height | 1.04(26/25) | 1.30(26/20) | ✗(分子26は両モック一致、分母のrowが違う) | FINDING(同上、根はrow_height) |
| value_width/row_height | 2.56(64/25) | 1.90(38/20) | ✗ | FINDING(同上) |
| value_width/panel_width | 0.129(64/496) | 0.077(38/496の実パネル幅) | ✗ — ただし ui-scale-and-z.html 基準の300px pane では 38/300=0.127 で**モック内比とほぼ一致** | FINDING(パネル幅496→300裁定待ちに従属、3.1節) |
| glyph(key)_width/row_height | 1.04(26/25) | 0.90(18/20) | △近い | FINDING(同上) |
| `single_row_horizontal_inset`(0.6em式) | 式として0.6em系観測(1e節) | 式が0.6em(裁定168適用済み) | ✓ | 対象外・既に適合(変更不要) |
| `sibling_gap_px`(0.075梯子) | 梯子0.30近傍が主、0.075段の直接実例はvalueCell間gapに無い(mockはgap:1pxのみ`.cols,.prow{gap:var(--sp1)}`相当が新mock側) | 式が0.075×row_height | ✓ | 対象外・既に適合(変更不要) |

### 3.1 pane幅相対で見ると value_width/glyph_width は「別モックへの正しい転写」

`inspector_value_width`(38)を旧mockの`64`とだけ比べると大きく割れるが、
分母を`inspector_panel_width`ではなく **転写元の`ui-scale-and-z.html`
自身の pane幅(`--pane:300`)** に取り直すと `38/300=0.127` — 旧mockの
`64/496=0.129` と**ほぼ同じ比**になる(誤差0.002)。`inspector_glyph_width`
(18)も `18/300=0.06` で旧mockの `26/496=0.052` に近い。**つまり
value_width/glyph_width は「旧mockからの転写ミス」ではなく「別モック
(300px pane 前提)への転写としては辻褄が合っている」** — 割れているのは
`inspector_panel_width` 自身が旧mock値(496)のまま据え置かれている
(`dimensions.json` 明記: 300への変更は利用者裁定待ち)ことに起因する、
**同じ根の症状**。`inspector_row_height`(20)だけは `20/300=0.067` で
旧mockの `25/496=0.050` からやや外れる(密度を上げる意図的な調整の可能性)
— この1点だけは pane幅換算でも完全には説明がつかない、独立した観察として
残す。

## 4. 転写した差分

**なし。** 本レーンの ALLOWLIST(`next/ui/motolii-inspector-pane/src/**`・
`tests/**`・本ドキュメント・README 1行)内で、単独の値書き換えだけで
安全に閉じる乖離は見つからなかった — 見つかった乖離はすべて (a)
`next/ui/motolii-tokens-rs/tokens/dimensions.json` 側の値(ALLOWLIST 外)、
または (b) 裁定172 の非目標に明記された「フォント役割の変更・行の構成変更」
のどちらかに分類される(5節)。`single_row_horizontal_inset`/`sibling_gap_px`
は既に裁定167/168 へ適合済みで、変更不要と確認しただけ(3節)。

## 5. 見送った差分(FINDING)

1. **`body_text/inspector_row_height` = 0.55 は `inspector-library.*` 実測
   (0.44)とも不一致** — 修正は `inspector_row_height` の書き換えが必要だが
   ALLOWLIST 外。かつ `inspector_value_width`(38)・`inspector_glyph_width`
   (18)と三点で束になった `ui-scale-and-z.html` 転写(3.1節、pane幅300前提)
   と整合しているため、`inspector_row_height` だけを 25 へ戻すと `inspector_
   panel_width`(496、旧mock値のまま利用者裁定待ち)・`inspector_value_width`
   (38、300前提)との間で**新たな不整合**を作る。3値+pane幅の計4値を
   セットで再裁定すべき論点 — tokens crate を write-set に含む後続レーンへ。
2. **`caption_text`(9)は `inspector-library.*` 内でも 8 と 9 が混在する役割
   (columnHeader/tableSection h2 は8、selectionSummary span/footer は9)**
   — どちらを「caption の正」とするかはフォント役割の選択そのもの(裁定172
   非目標)。現に `micro_text`(8)トークンは既に予約済みで未消費
   (`dimensions.json` 明記)なので、8を使うべき箇所があるとすれば
   `column_header_row`/`section_header` の2箇所だが、これも意匠選択として
   ここでは転写しない。
3. **`inspector_section_header_height`(26)は両mockで分子(26)は一致するが、
   分母となる「行高」が違う(旧mock 25 vs 新mock 20)ぶんだけ比がズレる** —
   1と同根、tokens crate 側の話。
4. **`inspector-library.*` の effect stack / mode tabs / selection summary
   (46px)/ extension tabs / notes panel は現行実装に対応箇所が無い**(Q0
   スコープ外、0節)。これらの比率は「実装が存在しない」ため照合の対象外 —
   将来 FX UI レーンが着手する際の一次資料として本台帳に残すに留める。
5. **`propertyName` の左 padding(11px = 8pxの基調+3pxアクセントバー幅)**の
   ように装飾オフセットを内包する値は 裁定167 の梯子へ素直に乗らない
   (1d節) — Timeline側 rail 8/26 の先例と同じ性質の「梯子の例外」であって
   bug ではない、経過観察のみ。

## 6. 裁定168 への注記修正(裁定172 §3 の指示分)

裁定168 の 0.42 は元々 Timeline 実測由来であり、この台帳により Inspector
は**自分自身のモック実測でも 0.44 で近い値**(帯 0.42±0.05 に収まる)を
持つことが分かった — 「帯はグローバル定数ではなく pane ごとにモック実測
から導出する」という裁定172 §3 のpane相対化を Inspector 側でも裏付ける
結果になった。裁定168 本文の書き換えは supervisor 直施工(本レーンの
NON-GOALS)。

## オラクル

`next/ui/motolii-inspector-pane/tests/inspector_ratio_ledger.rs` — 本台帳の
主要数値(モック側は `inspector-library.css` からの引用リテラル、実装側は
`Dimensions::default()` を `use` して読む「両側チェック」)を固定する。
0.55/0.44 の両方を pin し、どちらかが黙って動いたら red になる
(css-metrics 前例と同型の regression lock)。
