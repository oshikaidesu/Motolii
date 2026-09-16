# 文字組みの法 — CSS の text-autospace・text-spacing-trim・hanging-punctuation を写す

2026-09-17。[台帳 2](2026-09-17-css-ledger-2.md) 文字組みの節の「取る」3 本のうち、shaping の後段で済む 3 本。ソフトが持つのは道具(法)だけ、意味(見た目)は作る人の物。欄の名前は CSS の語(既決)。

## 意図

日本語のリリックは漢字・仮名・欧文・数字・「」。、が 1 行に混ざる。AE では境に半角空白を手で打ち、行頭の括弧は字ごとにカーニングを打つ — 改行が変わると全部やり直し。CSS はこれを 3 つの法にしている。法は字列と書体の純関数で、時刻を持たない。行分割の後に端の字を測って x をずらすだけなので、折り返し・避けて流す組み(shape-outside)の両方に同じ法が掛かる。

## 札(文字の層の段落の欄。窓と台本は同じ名前で書く)

| 欄 | 選択肢(CSS の値をそのまま) | 既定(CSS の初期値) | 効き |
|---|---|---|---|
| `Text Autospace` | Normal / No Autospace | Normal | 漢字と欧文の字・数字が直に隣る境に、漢字の字送りの 1/8 を足す。空白や約物が間にあれば境ではない。Center / Right の測りに入る(letter-spacing と加算) |
| `Text Spacing Trim` | Normal / Space All / Space First / Trim Start | Normal | 隣どうしの全角約物は書体の `chws` で詰める(Space All 以外)。Trim Start は行頭の全角の開き括弧の左半分を削る。Space First は段落の 1 行目だけ削らない。Normal は行頭を詰めない |
| `Hanging Punctuation` | None / First / Allow End / Force End / Last / First Allow End / First Force End / First Last / Allow End Last / Force End Last / First Allow End Last / First Force End Last | None | First = 段落の最初の行の行頭の開き括弧・引用符・全角空白を箱の外へ。Last = 最後の行の行末の閉じ括弧・引用符を外へ。Force End = 行末の句読点を外へ、Allow End = 入り切らない時だけ。ぶら下がった字は行の測りに入らない(Center / Right は残りで寄せる) |

台本: `text(...).set("Text Spacing Trim", "Trim Start").set("Hanging Punctuation", "First Force End Last")`。見本 `ui/native/src/editor/script/examples/cjk_lyrics.js`。

## 先例

- CSS Text Module Level 4 §8.4 [text-autospace](https://drafts.csswg.org/css-text-4/#text-autospace-property)(§8.4.1 Inter-script Spacing に量と字の種)、§8.5 [text-spacing-trim](https://drafts.csswg.org/css-text-4/#text-spacing-trim-property)(§8.5.1 隣どうしの詰め、§8.5.2 字の種、§8.5.3 日本語の行頭 3 式)、§9.2.1 [hanging-punctuation](https://drafts.csswg.org/css-text-4/#hanging-punctuation-property)
- MDN: [text-autospace](https://developer.mozilla.org/en-US/docs/Web/CSS/text-autospace) **Baseline 2025-11**、[text-spacing-trim](https://developer.mozilla.org/en-US/docs/Web/CSS/text-spacing-trim) Chromium のみ・Baseline でない、[hanging-punctuation](https://developer.mozilla.org/en-US/docs/Web/CSS/hanging-punctuation) Safari のみ・Baseline でない
- 字の種の定規は Unicode: general category(`unicode-general-category`)、East Asian Width(`unicode-width`)、script extension(`unicode-script`)。手書きの表は仕様が code point で列挙している句読点だけ

## 借りた物と、Motolii が足した物

借りた(仕様のまま):
- autospace の量 **1/8 の字送り**(§8.4.1「1/8 of the CJK advance measure, i.e 0.125ic」。台帳 2 の「1/4 em」は歴史的な値で、CSS は控えめに 1/8 を選んだと仕様が言う)。字の種は仕様の 3 定義(ideographs / non-ideographic letters / numerals)
- trim の値の意味と、行頭の削り = 左半分(`halt` と同じ量)。隣どうしは `chws`(仕様: `hwid` は使ってはならない)
- hanging の文法 `first || [force-end | allow-end] || last` を 12 の選択肢に展開、字の種は Ps/Pi/Pf・Pe/Pf/Pi・' "・U+3000、句読点は仕様の表の 13 字。「ぶら下がった字は揃えの測りに入らない」
- hanging は欧文の約物にも掛かる(MDN の例は « » )。貼れるかの検分: 欧文だけの行で autospace と trim は不動、`.` は force-end で下がる — 仕様の通り

足した(仕様が開けている所。**利用者の裁定待ち**、黙って決めない):
1. **段落 = 改行で区切った 1 行**。CSS の first / last / space-first は「要素の最初の行・強制改行の後」で決まる。Motolii の文字の層は改行を含むので、改行ごとに `<p>` と見なした(リリックは 1 行 1 段落)。層全体を 1 要素とするなら first は層の 1 行目だけになる
2. **ic = font-size**。仕様は 水 の字送りで測ってよいと言う。ヒラギノは正方なので同じ。プロポーショナルな漢字の書体では差が出る
3. **行末の詰め(closing punctuation が入り切らない時の半角)と Allow End の「入り切らない」は行分割の中の判定**。cosmic-text の行分割に手を入れていないので、行末は「行の幅が折返し幅を超えた時」だけ効く。入り切らないと判定して句読点を前の行に残す(禁則の緩め)は未実装 — 必要なら行分割の側(text-wrap の係)で
4. **autospace は行分割の後に足す**。折り返す行は境の数 × 1/8 だけ幅を超え得る(CSS は分割前に測る)。歌詞の行では見えない量だが、Fill で並ぶ長い段落では右端が揃わない
5. **中黒・コロンの分類**(§8.5.2、言語で違う)は日本語の慣習(句読点 = 閉じ、コロン = 中黒)にした。`chws` が書体側で処理するので Motolii の側には句読点の分類しか無い
6. **縦書きは未対応**(vhal / vchw、上下の削り)。横書きだけ

## 絵

見張り(`watch_shot` → `zz_watch`)の 4 コマの敷き詰め(`*-sheet.png`)と、読める大きさの 1 コマ目(`*-frame0.png`、半分に縮小)。左が法を全部切った列(No Autospace / Space All / None)、右が法を押した列(Normal / Trim Start / First Force End Last)。金の細い線が列の左端。右の列で、漢字と欧文・数字の境に 1/8 の間が入り、行頭の 「『 が線の外へ下がって 2 行目の 夜 と字面が揃う。

- 角ゴ(Hiragino Sans): [evidence/2026-09-17-cjk-typography/gothic-frame0.png](evidence/2026-09-17-cjk-typography/gothic-frame0.png)、[gothic-sheet.png](evidence/2026-09-17-cjk-typography/gothic-sheet.png)
- 明朝(Toppan Bunkyu Mincho、台本の font を差し替えただけ): [mincho-frame0.png](evidence/2026-09-17-cjk-typography/mincho-frame0.png)、[mincho-sheet.png](evidence/2026-09-17-cjk-typography/mincho-sheet.png)
- 欧文だけの行(貼れるか): [latin-frame0.png](evidence/2026-09-17-cjk-typography/latin-frame0.png)、[latin-sheet.png](evidence/2026-09-17-cjk-typography/latin-sheet.png) — autospace・trim は不動、行頭の " ' と行末の . だけ仕様の通り下がる

見つけた別件(この法の外、未修正): **Hiragino Kaku Gothic Pro / ProN・Hiragino Mincho Pro / ProN は台帳に載るのに、組むと PingFang SC に落ちる**(cosmic-text の face 照合。Hiragino Sans・Hiragino Maru Gothic ProN・Toppan Bunkyu Mincho は通る)。台本は `Hiragino Sans` にした。また文字の層の既定は `palt`(プロポーショナル約物)が入っているので、Trim Start は「空いた半分だけ削る」(§8.5.1「足しも引きもしない」)で palt の 「 には掛からず、ぶら下げだけが効く。

## 反映先

`crates/motolii-doc/src/vector/text.rs`(`TextAutospace`・`TextSpacingTrim`・`HangingPunctuation`、`line_laws`、`class`)、`store/text.rs TextAlignmentOptions`、`store/view/resolve.rs authored_text_document`、`store/names.rs`、`ui/native/src/snapshot.rs`(段落の欄)、test `law_tests::*`。
