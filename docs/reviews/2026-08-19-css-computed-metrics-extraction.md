# CSS 計算値の抽出器具と、iced 側寸法定数との突き合わせ

日付: 2026-08-19
状態: **観察**

Blitz(`blitz-dom` + `stylo`)を「HTML/CSS → 計算済みの値」の変換器として使い、
`docs/mocks-ui/public/{inspector,browser,timeline}-library.html` を機械的に
layout・style 解決した結果を JSON で取り出す器具 `motolii-css-metrics` を作った。
抽出した値と、現在の iced 実装が持つ寸法定数(`crates/motolii-shell-iced/src/
inspector_pane.rs` の `dims` module、`timeline/semantics.rs` の `pub const`
群、`view.rs` の Browser 帯 padding)を突き合わせ、どこが一致しどこが割れて
いるかをここに書く。**この文書はどれが正しいかを裁定しない** — 観察であり、
Inspector 側は「既にほぼ正確に転記されていた」ことの機械確認、Timeline 側は
「css mock ではなく別の参照(スクリーンショット)から寸法を採った」既存の
自己申告が実測とも整合することの確認になった。

## 器具

- bin: `motolii-ui` の `motolii-css-metrics`(`crates/motolii-ui/src/css_metrics/main.rs`)
- 中身は `motolii_ui::css_metrics::extract(html_path, viewport) -> Result<Vec<serde_json::Value>, String>`
  として `crates/motolii-ui/src/css_metrics/mod.rs` に公開してある。bin は CLI の皮を
  被せて JSON をファイルへ書くだけ。**公開関数にした理由は1つ** —
  `motolii-shell-iced` 側の oracle テスト(下記)が同じ抽出ロジックを直接呼べる
  ようにするため(JSON ファイルを経由させたり bin を subprocess で叩いたり
  しない)
- GPU 不要。`document.resolve(0.0)` を2回呼んで layout と stylo の computed
  style を解くだけで、`blitz-paint` / wgpu は一切呼ばない
  (`motolii-blitz-dump` が使う `BlitzSurface` 経路とは別系統)
- 素の html は `<link rel="stylesheet" href="...">` で css を引くが、
  `blitz-html` は `<link>` の href を解決しない(実測)。器具側で `<style>`
  へ in-memory 展開してから `HtmlDocument::from_html` に渡す
  (ファイルは書き換えない)。href が `/` 始まりの場合、html 自身の隣
  (`public/`)→ その一段上、の順で探す(inspector の
  `/src/tokens/mock-candidates.css` は `public/` の外に居る — Vite dev
  server が `public/` と project root を両方 `/` へ重ねて出す構成の写し)

### 使い方

```text
cargo run -p motolii-ui --bin motolii-css-metrics -- inspector out/inspector.json
cargo run -p motolii-ui --bin motolii-css-metrics -- browser   out/browser.json
cargo run -p motolii-ui --bin motolii-css-metrics -- timeline  out/timeline.json
cargo run -p motolii-ui --bin motolii-css-metrics -- all       out/
cargo run -p motolii-ui --bin motolii-css-metrics -- inspector out/i.json --viewport 520x900
```

リポジトリ root から実行する(html への相対パスは器具内で `html_path` 自身
からの `.parent()` 連鎖でのみ解くので、実は cwd に依存しない — テスト側は
`CARGO_MANIFEST_DIR` から絶対パスを渡している)。

### 出力の実例(inspector、抜粋)

```json
{
  "path": "body > main.inspectorShell > header.panelHeader",
  "tag": "header",
  "classes": ["panelHeader"],
  "box": { "x": 0.0, "y": 0.0, "w": 495.0, "h": 29.0 },
  "padding": { "top": 0.0, "right": 9.0, "bottom": 0.0, "left": 9.0 },
  "border": { "top": 0.0, "right": 0.0, "bottom": 1.0, "left": 0.0 },
  "computed": {
    "background": "rgba(0, 0, 0, 0)",
    "border_color": { "top": "currentcolor", "bottom": "rgb(59, 59, 59)", "left": "currentcolor", "right": "currentcolor" },
    "font_size": "16px",
    "gap": { "row": "7px", "column": "7px" }
  }
}
```

```json
{
  "path": "body > main.inspectorShell > header.panelHeader::before",
  "tag": "div",
  "box": { "x": 9.0, "y": 8.0, "w": 3.0, "h": 13.0 },
  "computed": { "background": "rgb(142, 176, 134)" }
}
```

2つめの行は `::before` 生成 box(下記参照) — `.panelHeader::before` の
`width:3px; height:13px; background: var(--mock-role-way-inspector)` が
そのまま出ている。

### 出力の schema

要素1個 = JSON配列の1要素:

```text
{ path, id, classes, tag,
  box: {x,y,w,h},           // final_layout(taffy)。x/y は DOM 祖先の location を足し上げた絶対座標
  padding/border/margin: {top,right,bottom,left},  // 同じく final_layout
  computed: { background, color, font_size, font_family,
              border_radius: {top_left,top_right,bottom_right,bottom_left},
              border_color:  {top,right,bottom,left},
              gap: {row, column} } }
```

`border_radius` / `border_color` は当初 top 側だけの代表値を出す設計にしていたが、
実測で `.panelHeader`(`border-bottom` だけを持つ)の `border-top-color` が
`currentcolor` を返し、実際に見える `#3b3b3b` の線(bottom)を取り逃がすことが
分かった。この mock は角丸をほぼ使わず、線のほとんどが `border-bottom` だけの
行区切りなので、top 代表は体系的に間違う。四辺・四隅を丸ごと出す形に直した。

## 「対象は個別レイアウトの絶対座標ではない」という設計注に対する結果

supervisor の見立て(全要素の絶対座標を丸ごと持ってくる形にしない、行数や
名前の長さで変わらない値だけを取る)は妥当だった、と言うより**実測でさらに
強められた**: `box.x/y` は同じ mock でも viewport や手前の兄弟要素の内容量で
動くので、比較に使ったのは一貫して `box.w/h`・`padding`・`border`・`gap`・
`computed.*` — 位置に依存しない値だけである。座標そのものは今回は
デバッグ用の副産物として残しているだけで、report の突き合わせには使っていない。

## `::before`/`::after` を歩かないと帯幅(band width)が1個も取れない

最初の実装は `NodeData::Element` だけを歩き、`NodeData::AnonymousBlock` を
無条件に飛ばしていた。これは実害があった — この mock の「行の帯」
(`.propertyRow::before{width:3px}`)や「panel header の accent bar」
(`.panelHeader::before{width:3px;height:13px}`)は**軒並み `::before` 生成
box で作られている**。`blitz-dom` はこれらを `NodeData::Element` ではなく
`NodeData::AnonymousBlock`(タグは placeholder の `"div"`)として作る
(`layout/construct.rs` の `flush_pseudo_elements`)。`Node::before` /
`Node::after`(`Option<usize>`)を明示的に辿るよう直し、path に `::before` /
`::after` suffix を付けて出すようにしたところ、Inspector の要素数が
289 → 357 に増え、`dims::ROW_BAND_W`(3px)・`HEADER_ACCENT_W/H`(3px/13px)
の実体を初めて捕まえられた。**帯幅という supervisor の名指しした対象そのもの
が、この mock では pseudo-element 経由でしか実体化していない**という発見。

## 突き合わせ結果

### Inspector — `crates/motolii-shell-iced/src/inspector_pane.rs` の `mod dims`

11個の定数すべてが実測と**一致**した(表の「抽出値」は `motolii-css-metrics
inspector` の実行結果、viewport 520x900)。

| dims 定数 | 値(px) | css 出所 | 抽出値(px) | 判定 |
|---|---|---|---|---|
| `PANEL_HEADER_H` | 29.0 | `.panelHeader{height:...29px}` | 29.0 | 一致 |
| `HEADER_ACCENT_W` | 3.0 | `.panelHeader::before{width:3px}` | 3.0 | 一致 |
| `HEADER_ACCENT_H` | 13.0 | `.panelHeader::before{height:13px}` | 13.0 | 一致 |
| `SUMMARY_H` | 46.0 | `.selectionSummary{height:46px}` | 46.0 | 一致 |
| `LAYER_STATE_W` | 22.0 | `.layerStateButton{width:22px}` | 22.0 | 一致 |
| `LAYER_STATE_H` | 21.0 | `.layerStateButton{height:21px}` | 21.0 | 一致 |
| `COLUMN_HEADER_H` | 21.0 | `.columnHeader{height:21px}` | 21.0 | 一致 |
| `SECTION_H` | 23.0 | `.tableSection h2{height:23px}` | 23.0 | 一致 |
| `ROW_BAND_W` | 3.0 | `.propertyRow::before{width:3px}` | 3.0 | 一致(pseudo 経由でのみ検出) |
| `VALUE_COL_W` | 64.0 | grid `repeat(3, 64px)` | 64.0 | 一致(解決後の実測 grid 幅) |
| `FX_BADGE_W` | 17.0 | `.effectBadge{width:17px}` | 17.0 | 一致 |
| `FX_BADGE_H` | 13.0 | `.effectBadge{height:13px}` | 13.0 | 一致 |
| `FX_PILL_MIN_W` | 25.0 | `.effectEnable{min-width:25px}` | 25.0 | 一致 |
| `FX_PILL_H` | 15.0 | `.effectEnable{height:15px}` | 15.0 | 一致 |
| `KIND_ICON` | 15.0 | `.propertyName i{width:15px;height:15px}` | 15.0 | 一致 |

副産物として、色も1件クロスチェックできた: `.panelHeader::before` の
`background` が `rgb(142, 176, 134)`(= `#8eb086`)で、`panel_header()` が
渡す `Tokens::DARK.way_inspector` と一致する。

`.propertyRow` 自体(`min-height:25px`)は `dims` に対応する定数を持たない
(iced 側は行の高さを `Fill` + 内容量に委ねている、コメントに明記あり)。
実測では行の高さがちょうど 25px に落ち着いている(コンテンツがそのフロアを
超えていない)ことを確認した。CSS の `min-height` は「フロア」なので、
コンテンツがそれより大きい行では iced 側もそのまま追随する(both are
content-driven above the floor)。逆にコンテンツが 25px 未満になり得る行では
CSS はフロアで止まるが iced 側には対応するフロアが無い —
**この意味での相違は今回の3枚のfixtureでは顕在化しなかった**が、fixture
依存の観察であることは明記しておく。

### Browser — `crates/motolii-shell-iced/src/view.rs` の `browser_panel`

| iced 側 | 値 | css 対応 | 抽出値 | 判定 |
|---|---|---|---|---|
| header の `.padding([6, 8])` | 上下6・左右8 | `.browserHeader{height:26px;padding:0 8px}` | box.h=26、padding top/bottom=0・left/right=8 | **左右は一致・上下は不一致**(css は固定高さ+`align-items:center`、iced は上下padding で近似) |
| tray の `.padding([5, 8])` | 上下5・左右8 | `.selectionTray{height:27px;padding:0 6px}` | box.h=27、padding top/bottom=0・left/right=6 | **不一致**(上下0→5、左右6→8の両方がズレる) |
| `BROWSER_PANE_W = 316.0` | — | `.libraryBrowser{width:min(100%,700px)}` | (viewport900で)700 | **意図的不一致** — `view.rs:46-48` のコメントで明記: pane 幅は dock の既定 share(egui-tiles 側の中央列 1:1 実測)から採っており、mock は単体フルスクリーンの寸法。両者は最初から異なる問いに答えている |
| `INSPECTOR_PANE_W = 300.0` | — | `.inspectorShell{width:min(100%,496px)}` | (viewport520で)496 | 同上(意図的不一致) |

`.locationRow`(rail の項目、`dims` 相当の定数なし)は**計測できなかった** —
static HTML の `<main class="libraryBrowser">` に `data-tab` 属性が付いて
おらず(`<script>` が `pointerdown` 等で動的に設定する)、CSS の
`.tabScoped{display:none}` が既定で効くため、4つの `tabScoped-*` 内側は
Blitz が JS を実行しないこの器具では**すべて display:none のまま**になる
(box が全部 0×0)。「静的初期状態しか見えない」という器具の限界が
Browser fixture で最も強く出た形(下記「詰まった点」参照)。

### Timeline — `crates/motolii-shell-iced/src/timeline/semantics.rs`

| iced 定数 | 値 | css 対応 | 抽出値 | 判定 |
|---|---|---|---|---|
| `ROW_H = 24.0` | 24.0 | `.timelineRow{height:24px}` | 24.0 | **一致** |
| `RAIL_W = 210.0` | 210.0 | `.arrangement{grid-template-columns:196px ...}`(`.columnHead`/`.layerCell` 実測幅) | 196.0 | **意図的不一致** — `semantics.rs:30-33` のコメントに明記済み: 「M/S(2個)と種別色の四角を横に並べても名前が詰まらない値(2026-08-19、`/tmp/egui-same-doc.png` に合わせて 196→210 へ)」。**css mock ではなく egui 側スクリーンショットが出所と自己申告**しており、実測はその申告と矛盾しない |
| `TRANSPORT_H = 30.0` | 30.0 | 直接対応なし。`.timelineHead{height:34px}` とは**別の帯**(iced の transport 帯は playhead 読み・行数・grid刻みを表示、css の `.timelineHead` はロゴ+transport+Snap/Fit) | `.timelineHead`=34.0 | **不一致**(34≠30)。`TRANSPORT_H`(30)は偶然 `.overview` の高さ(30px)と同値だが、意味的に無関係な帯を指している — コメントも `/tmp/egui-same-doc.png` 由来と明記、css mock を出所と称してはいない |
| `OVERVIEW_H = 22.0` | 22.0 | `.overview{height:30px}` | 30.0 | **不一致**(22≠30)。同じく `/tmp/egui-same-doc.png` 由来と自己申告 |
| `RULER_H = 36.0` | 36.0 | css に直接対応する値なし(`.arrangement{grid-template-rows:27px minmax(0,1fr)}` の残り領域がルーラ+行の面) | `.columnHead`(同じ行1)=27.0 | 対応なし。コメントは「egui 版 `RULER_H` と同値」とだけ言い、css mock を出所としていない |
| `TRIM_EDGE = 8.0` | 8.0 | 見た目要素ではなく、ドラッグの当たり判定幅(操作仕様) | — | 対応なし(意味が違う。css 比較の対象外) |

Timeline は Inspector と方法論が違う: `inspector_pane.rs` の `dims` は
「`inspector-library.css:行番号`」を1個ずつ引用して転記しているのに対し、
`timeline/semantics.rs` は寸法の根拠を一貫して `/tmp/egui-same-doc.png`
(egui 版のスクリーンショット)に置いている。**この文書はどちらが正しい方法
かを裁定しない** — ただし「Timeline の寸法は css mock からの転記ではない」
という既存のコメントの自己申告は、今回の実測(css 側の値と 3/5 が食い違う)
と矛盾しない、という確認にはなった。

## oracle テスト

`crates/motolii-shell-iced/tests/css_metrics_oracle.rs` に追加した。

- **Timeline 側**は `timeline::semantics` の定数が `pub` なので実物を
  `use` して比較する(真の両側チェック — 将来 css か semantics.rs の
  どちらかが変われば、対応する assert が落ちる)
- **Inspector 側**は `inspector_pane.rs` の `mod dims` が非 pub module
  (外部 crate から到達不能)で、かつ `inspector_pane.rs` は今回の柵で
  書き換え禁止(3レーン並走中)なので、`dims` の値をこのテストへ
  literal で転記して pin した。**片側の保証にしかならない** —
  `docs/mocks-ui/public/inspector-library.css` が変わればこのテストは
  落ちるが、`inspector_pane.rs` 側の `dims` が変わってもこのテストは
  気づかない(inspector_pane.rs を人が触った時に、このテストの literal
  も一緒に直す運用が要る)
- 既知の不一致(Timeline の `RAIL_W`/`TRANSPORT_H`/`OVERVIEW_H`)は
  「一致」を主張する assert にしない — 現状の両側の値をそれぞれ literal
  で pin するだけの別テストにして、テストを赤いまま残さずに将来のドリフト
  だけ拾えるようにした

## Blitz 側で詰まった点

1. **`<link>` の href を Blitz は解決しない**。素の html をそのまま
   `HtmlDocument::from_html` に渡すと css が一切当たらない(全要素が UA
   既定値になる)。器具側で in-memory inline するしかない
2. **`href="/..."` の解決根が2階層に割れている**。`inspector-library.html`
   は `/inspector-library.css`(`public/` 直下)と
   `/src/tokens/mock-candidates.css`(`public/` の外)を同じ `/` 起点で
   引いており、Vite dev server の「`public/` と project root を両方 `/`
   へ重ねる」挙動を知らないと1本目の href 解決根だけでは404になる
3. **`::before`/`::after` は `NodeData::AnonymousBlock`**(`NodeData::Element`
   ではない)として生成される。木を「実 DOM の子」だけで歩くと、生成
   box(この mock では帯・accent bar のほぼ全部)を取り逃がす。
   `Node::before`/`Node::after` を明示的に辿る必要がある(上記参照)
4. **JS 駆動の属性(`data-tab`・`data-view` 等)は反映されない** — Blitz は
   `<script>` を実行しない。この3枚は程度が違う: Timeline の折りたたみ状態
   (`hidden` 属性)は**静的 html に直接書かれている**ため正しく反映される
   一方、Browser の `data-tab="media"` は**JS が実行時に設定する**ため、
   静的抽出では sidebar/filter shelf の中身が丸ごと `display:none` になる
5. **色の computed value は `currentcolor` を返し得る**(`border-top-color`
   など、その辺の border が実際には描かれない場合)。単一の代表値サンプリング
   (top 固定)は、この mock のように border-bottom だけを使う意匠だと
   体系的に間違った答えを返す(上記「四辺・四隅を丸ごと出す」の節)
6. **`stylo`/`stylo_traits` を `motolii-ui` の直接依存に追加した**(`blitz-dom`
   0.3.0-beta.1 が固定する版 `0.19.0` と揃えている)。`node.primary_styles()`
   が返す `ComputedValues` の生成 `clone_X()`/`get_X()` アクセサ群と
   `ToCss::to_css_string()` を直接呼ぶには、`style`/`style_traits` を
   crate 名で書ける必要があった(inherent メソッド呼び出し自体は
   transitive dep でも通るが、`use style_traits::ToCss;` のような trait
   import と、関数シグネチャに型名を書く箇所には要る)

## 次に自動化できそうなこと

- **`inspector_pane.rs` の `mod dims` を `pub(crate)` にできれば**
  (パネル書き換え柵が解けた回で)、oracle テストの Inspector 側も
  Timeline 側と同じ「真の両側チェック」に格上げできる
- **Browser の JS 初期状態**(`data-tab="media"` 等)を、html の
  `<script>` から静的に読み取って `extract()` 呼び出し前に属性だけ
  in-memory で補ってやれば、sidebar/filter shelf の寸法も抽出できる
  (JS エンジンを持ち込む必要はない — 初期値は `<script>` 冒頭の
  determinstic な代入で足りるはず)
- **grid の各列幅**(Inspector の `minmax(132px,1fr)` 列、Timeline の
  `.layerCell{grid-template-columns:18px 15px minmax(0,1fr) 25px 34px 34px}`)
  を個別に突き合わせる表をもう1段作れば、iced 側の `row![...].spacing(...)`
  個別要素の幅とも比較できる(今回は代表的な dims 定数だけを追った)
- **抽出結果 JSON を fixture として repo に固定コミットし**、CI で
  「fixture と再抽出結果が一致するか」を見れば、mock html/css の変更が
  無自覚に紛れ込むこと自体も検出できる(今回は `out/*.json` は
  worktree 内の生成物のままで committed していない)
