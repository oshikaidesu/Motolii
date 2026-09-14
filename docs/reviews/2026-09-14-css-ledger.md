# CSS の恩恵の台帳 — Web が軽々と使っている表現を、動画の骨格へ

2026-09-14 観察。利用者「CSS のメリットをどんどん書き上げる。そこが Motolii の明らかな強みになる。誰もまだ難しい表現だと思っているが、実際はそうではなく、無理やり対応させているからで、事実 Web は軽々とこの表現を使用している」。

前提は [3 層(骨格・肌・魂)](../concept.md) と [箱と流し込みの法](2026-09-14-layout-law.md)。この台帳は採否を決めない。1 つずつ法にする時に、ここから引いて相談する。

## なぜ「難しい表現」に見えていたか

モーショングラフィックスの道具は、画素を重ねる合成の系譜(AE)か、メッシュと分配の系譜(C4D の Cloner、MASH、Cavalry)で育った。どちらも**入れ物が箱ではない**。そこへ「子の大きさを知って並べ直す」「中身に合わせて伸びる」「幅で折り返す」を後から足すと、式(`sourceRectAtTime()`)、拡張(Pins & Boxes)、部品の分裂(Cavalry の Layout / Bounding Box / Align)になる。難しいのは表現ではなく、**土台の単位が違うこと**。

Web は最初から要素が箱で、並べ方は値を 1 つ選ぶだけ。2026 年の今、計算は軽く(taffy は数百の箱をマイクロ秒で解く)、語彙は完成し、LLM は CSS を一番流暢に話す。

## 篩

- **取る**: 箱・並べ方・大きさの単位・文字組み・分配(番号)・値の型。**時刻の純関数に写せる物**
- **取らない**: カスケード、詳細度、セレクタ、継承の暗黙、media query(箱の大きさで切り替える container query に置き換わる)、float・table・margin の相殺など Web 固有の歴史
- 状態の記号: **済** = Motolii にある / **一部** / **無** / **係が違う** = 別の法(合成・時間)で既に持つ

## A. 箱と並べ方(済)

| CSS | 動画の表現 | AE で難しい理由 | Motolii |
|---|---|---|---|
| Flexbox(`flex-direction`・`justify-content`・`align-items`・`gap`・`flex-wrap`) | 名前が伸びると帯と隣が追う、箇条書きが詰まる | 層に大きさが無い → 式でつなぐ | 済(Display Flex) |
| Grid(`grid-template-columns / rows`、`span`、線の `fr`) | bento、格子の squash & stretch、呼吸するポスター | 矩形を 1 枚ずつ手でキー | 済(線の太さに鍵) |
| Hug / Fill / Fixed、`min/max-*` | 中身に合う箱、セルいっぱいの素材 | 同上 | 済 |
| `zoom` と `transform` | 押す拡大と押さない拡大の使い分け | 区別が無い | 済(層の Scale = zoom)、Transform 効果は無 |
| `position: relative / absolute` | 並びの中で揺らす、列から外す | — | 済 |
| `object-fit` | セルに動画・丸を Cover / Contain | 手で拡大率を計算 | 済(形)、画・動画は無(元の寸法の口が未) |
| `overflow: clip`、`border-radius` | 角丸セルで切る | マットを層ごとに作る | 済 |
| 文字の `width` で折り返し | 列の太さで歌詞の改行が組み直る | 段落の枠のキーは「事故」扱い | 済(文字の Fill) |

## B. 並べ方の続き

| CSS | Web の状況 | 動画の表現 | AE で難しい理由 | Motolii |
|---|---|---|---|---|
| `grid-template-areas` | 全ブラウザ | 格子を文字の絵で宣言(`"hero hero stat logo"`)。script と LLM がさらに短く読める | 無い | 無 |
| `subgrid` | Baseline 2023 | 入れ子のセルの文字の下端を隣と揃える | 無い | 無 |
| `grid-lanes`(旧 masonry) | Safari 26 が先行、他は 2026 に出荷予定 | 高さの違うカードが隙間なく積み上がる | 手で詰める | 無 |
| `aspect-ratio` | 全ブラウザ | 幅が変わっても正方形のセル | 式 | 無 |
| `minmax()`・`auto-fit`・`auto-fill` | 全ブラウザ | 枠に入る数だけ列が増減 | 無い | 無 |
| `flex-grow` の比 | 全ブラウザ | 1 : 2 : 1 で伸びる帯、比にキーで重心が移る | 無い | 一部(Fill は 1 だけ) |
| anchor positioning | 2024–25 に出荷 | 動く物に付いてくる吹き出し・注釈・価格札 | 親子と式 | 無 |

## C. 大きさの単位と、比率違いの書き出し

| CSS | Web の状況 | 動画の表現 | AE で難しい理由 | Motolii |
|---|---|---|---|---|
| container query `@container`、単位 `cqw / cqh / cqi` | 全ブラウザ(単位は Firefox が一部) | **16:9 で作った作品を 9:16・1:1 へ並びを組み直して書き出す**。箱が細ければ縦積みへ | 比率ごとに作り直すのが普通 | 無 |
| `%`・`calc()`・`clamp()`・`min()/max()` | 全ブラウザ | 解像度に依らない余白と文字の大きさ | 数値の決め打ち | 無(解像度と素材座標の法とつなぐ) |
| `em` / `lh` | 全ブラウザ | 文字の大きさに比例する余白・アイコン | 式 | 無 |
| `interpolate-size` / `calc-size()` | 新しい(Chromium 先行) | **Hug ↔ Fixed を補間**して、中身に合う箱へ滑らかに開閉 | 無い | 無(並べる法の上で自然) |

## D. 文字組み(リリック・日本語の MV に効く)

| CSS | Web の状況 | 動画の表現 | AE で難しい理由 | Motolii |
|---|---|---|---|---|
| `text-wrap: balance` | Baseline 2024 | どの幅でも行の長さが揃う。動く改行が崩れない | 手で改行 | 無 |
| `text-wrap: pretty` | Chromium・Safari 26 | 最終行に 1 語だけ残さない | 手で改行 | 無 |
| `text-box-trim` | Chromium・Safari 18.2 | 字面の上下(cap・alphabetic)でぴったり揃える | 目で合わせる | 無(行の箱の上で自然) |
| `writing-mode: vertical-rl`、`text-orientation`、`text-combine-upright`(縦中横) | 全ブラウザ | **縦書きの歌詞、縦中横の数字** | 1 字ずつ層を組む | 無 |
| `shape-outside` | 全ブラウザ | **文字が物の輪郭(歌う人の影、Blob)を避けて流れる** | ほぼ不可能 | 無(Blob の輪郭を形として渡せる) |
| `initial-letter` | Safari・Chromium | 大きな頭文字が数行に沈む | 手で組む | 無 |
| `background-clip: text` | 全ブラウザ | 文字の形に動画・グラデーション | マット | 係が違う(マット・クリッピングで済) |
| `-webkit-text-stroke`・`paint-order` | 全ブラウザ | 縁取りを塗りの下に | 線の順番 | 一部(既定の黒の縁取りあり) |
| `font-variation-settings` | 全ブラウザ | 太さの軸にキー | 済(可変軸) | 済 |

## E. 分配と時間

| CSS | Web の状況 | 動画の表現 | AE で難しい理由 | Motolii |
|---|---|---|---|---|
| `sibling-index()` / `sibling-count()` | Chromium 138(2025)・Safari 26、Firefox 実装中 | **兄弟の番号で時間差**(`animation-delay: calc(sibling-index() * 80ms)`)。文字も格子のセルも同じ規則 | Text Animator は文字だけ | 一部(Each = 番号比例)。行・列・中央から、は無 |
| GSAP の stagger `grid` / `from: center`(先例) | 事実上の標準 | 中央から外へ、端から、ランダムに | 同上 | 無 |
| `linear()` イージング | 全ブラウザ | 任意の曲線(バネ)を点列で | 式 | 係が違う(Bounce / Elastic / Steps のキー) |
| `animation-composition: add` | 全ブラウザ | 揺れを重ねて足す | 式 | 係が違う(効果の積み) |
| `@starting-style` | 新しい | 出現した瞬間の姿から始める | 手でキー | 一部(Position のずれで登場) |
| View Transitions / FLIP | 全ブラウザ(同一文書) | **並び替え・折り返しで、物が新しい場所へ滑る** | 手で全部キー | 無(法で延期) |
| scroll / view timeline | Chromium・Safari | 物が画面の線を通る時に進む演出 | 式 | 無(時刻の代わりに位置を読む口) |

## F. 絵(多くは合成の係が既に持つ)

| CSS | 動画の表現 | Motolii |
|---|---|---|
| `mix-blend-mode`・`isolation` | 17 の混ぜ方 | 係が違う(W3C Compositing を rerun から借りている) |
| `clip-path`(`inset / circle / polygon`、補間できる) | 図形で開く・閉じるワイプ | 係が違う(マスク)。形の語彙だけ借りられる |
| `mask-image` のグラデーション | フェードする縁 | 係が違う(マスクの feather) |
| `filter`・`backdrop-filter` | すりガラスのカード | 一部(効果)。**下を読む法は未** |
| 線形・放射・`conic-gradient` | 背景の光 | 一部(塗りのグラデーション) |
| `color-mix()`・`oklch` | 知覚で均等な色の補間とパレット | 一部(Oklab 補間) |
| `@property`(型付きの変数、補間できる) | **1 つの色・余白の変数にキー → 全体が変わる** | 無(property の slot / link が近い、要確認) |
| `offset-path` | 物が道に沿って進む | 一部(Repeater の Along Path) |

## 効きそうな順(Motolii の目的 = MV・リリック・SNS)

1. **container query と単位** — 1 本作れば全配信先の比率へ組み直して出す。AE に無い強みが一番大きい
2. **`grid-template-areas`** — script と LLM の文を一段短くする。並べる法の上に乗るだけ
3. **縦書き・縦中横、`text-wrap: balance`、`text-box-trim`** — 日本語の MV の土台
4. **分配(`sibling-index` と stagger の `from`)** — 文字・セル・複製を同じ規則で揺らす。9/07 のゴースト、9/11 の Each / Random の続き
5. **`shape-outside`** — 文字が物を避けて流れる。Blob Track の輪郭とつながる
6. **`interpolate-size` と FLIP** — Hug ↔ Fixed の開閉、並び替え・折り返しで滑る
7. **anchor positioning** — 動く物に付いてくる注釈
8. **`@property` の変数** — 既存の slot / link と照合してから

## 出典

- [sibling-index() — MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/sibling-index)、[Web platform features explorer: sibling-count](https://web-platform-dx.github.io/web-features-explorer/features/sibling-count/)
- [text-wrap — MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/text-wrap)、[CSS text-box-trim — Chrome for Developers](https://developer.chrome.com/blog/css-text-box-trim)
- [Animate to height: auto — Chrome for Developers](https://developer.chrome.com/docs/css-ui/animate-to-height-auto)、[interpolate-size — MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/interpolate-size)
- [Grid lanes layout — MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Grid_layout/Grid_lanes)
- [Container queries and units in action — web.dev](https://web.dev/articles/baseline-in-action-container-queries)
- [CSS zoom vs transform: scale](https://modern-css.com/scaling-elements-without-transform-hacks/)
- 並べる法の出典は [箱と流し込みの法](2026-09-14-layout-law.md) を参照
