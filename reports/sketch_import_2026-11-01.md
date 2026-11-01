# スケッチを Motolii に持ってくる（11/1）

ユーザーの問い：「cavalry のスクリプトや p5.js のスクリプトを持ってこれるかの方が早いか」「P5JS はもうそのまま時間の流れに沿って動くだけだけど、モトリはさ、編集できるじゃん」。途中の判断：「ブラウザで貼るのは一番やめよう。で、重てえし」。作品の中身は：「200個もあるオブジェクトを展開するのもなぁ」「コードはforで作ってるから、ムズいね、増えるのとかあるやん」。

## 結論

- p5.js の作品はファイルのまま、ブラウザ無しで動く。2D は ThorVG の形、WEBGL の `box()` などは立体のコピー。作った物は git の外に置いた。
- 開く単位は「物」ではなく「描いているコードの行」。for で 300 個でも増えても 1 本。1 行が 1 本のつまみ（0〜1）になる。新しい UI は作らず、既存のつまみの仕組みに載せた。Inspector の Script カードと時間軸の行に出て、キー・保存・Undo が効く。
- `background` の行を 0 にすると背景だけ消え、作品が動画の上に乗る。作品の中身は変わらない。
- 動画は `createCapture` / `createVideo` / `loadImage` で渡す。p5 がブラウザでやりにくい「動画を素材にする」が、下に動画を置くだけで出来る。
- 擬似 3D の 2D 作品は変換しない。奥行きは p5 に渡る前に割り算で消えているので、手で書き直すしかない。
- グループは時間軸で 1 行に畳む。⌘G の直後は閉じている。

## 見た所・やり方・見つかった事

| どこ | どう見たか | 見つかった事 |
|---|---|---|
| CodingTrain/Coding-Challenges | 取ってきて手元で実行 | 177 作、Chrome で 144 作が動く。QuickJS の最初の土台で 104 作が動き 97 作が描いた。1 コマ中央値 0.8 ms、最大 155 ms。形は 1 コマ中央値 20 個、最大 7,611 個 |
| OpenProcessing の API と Discover | API を毎分 40 回の上限内で読み、217 作を取得 | p5js 169、読めない 48。コードがある 168 作を試した。ライセンスは by-nc-sa 154、by-sa 14、無し 1 |
| svgcanvas、canvas2svg | 試しに動かして測った | 1,161 形で 1 コマ 45 ms。`globalCompositeOperation` が無いので取らず、canvas の呼び出しを数字で受ける形にした |
| Motolii の窓 | 実際に開いて押した | 下の「窓で見た事」 |

Reddit は組み込みブラウザが安全上の理由で開かなかった。ミラーは使っていない。

## OpenProcessing 168 作の結果

途中でエンジンを入れ替えたので、版が混ざった値。

| 結果 | 作品数 |
|---|---|
| 絵が出た | 97（2D 56、WEBGL 41） |
| 白紙 | 60（2D 38、WEBGL 22） |
| 30 秒で打ち切り | 8 |
| Metal で停止 | 3 |

- WEBGL の 41 作は箱の位置は出るが、コピーごとの色とシェーダーが無く、見た目は別物。
- 最初の 42 作を目で見て、元の作品らしいのは約 16 作。残りは目で判定していない。
- 使っている機能（168 作中）：random 131、draw の中の background 116、noise 79、mouse 69、WEBGL 64、drawingContext 54、shader 33、text 25、blendMode 22、image 22、createGraphics 22、filter 19、loadImage 11、pixels 10。
- Metal で止まった 3 作は、4,000 形ごとに区切ったら 45 秒以内には落ちなくなった。ただし 45 秒では終わらなかったので、直ったとは言えない。

## 物の数と行の数

| 数えた物 | 中央値 | 上位 10% | 最大 |
|---|---|---|---|
| 描いている行（155 作、vertex を除く） | 4 | 17 | 132 |
| 形の数（2D 53 作、5 つの時刻のうち最大） | 3,911 | 378,401 | 1,189,297 |

- 155 作中 120 作は 10 行以下、141 作は 20 行以下。
- 53 作のうち 31 作は形の数が一定、21 作は増えるだけ、1 作は減るだけ。増える 21 作には、消さずに塗り重ねる作品も入っている。
- 行は文字の読み取りで数えた。`p.rect()` の書き方の作品は行が出ない。この 168 作では 0 作だった。

## 動画と重ねた結果

- 1 コマの時間（動画を見る作品、初回）：点 72×72 で 92 ms、筆 260 本で 9 ms、流れ線 3,600 本で 32 ms、文字 54×54 で 57 ms。千個あたりまでが窓の 60fps、それ以上は書き出し向け。
- 書き出しは歩く動画 5.2 秒で 9.0 秒、ヤギの動画で 7.2 秒。動画の素材はテスト用で、git には入れない。

## 窓で見た事

- 206 個の書類が時間軸 6 行になった。三角で開くと中身が字下げして並ぶ。
- 241217a は `background 51 / image 56 / circle 94 / circle 142` の 4 行に開いた。`background 51` を 0 にすると奥の動画が見えた。
- `add` は 1 回ずつだと 120 個目で 950 ms。`together` でまとめると 200 個で 1.8 秒。

## p5 以外の入口

問い：「他のこういうのはある？p5.js以外にも」「それともまだ全部食えてない？」。

見た所：検索 8 回、どれも要約だけ。本家の頁は Dwitter の About と Turtletoy の API の説明を読んだ。Shadertoy の利用規約は Cloudflare の確認で止まり、読めていない。確認は通していない。棚は `ASSETS.md`、`reports/effect_sets.md`、`reports/capcut_families.md`、`reports/capcut_origins.md` を見た。

やり方：入口ごとの形を真似た短い見本を自分で書き、今の Motolii に通して絵を見た。本物の作品は通していない。

| 入口 | 形 | 通す所 | 結果 |
|---|---|---|---|
| Dwitter | 140 文字の JS、1920×1080 の canvas、`u(t)` を毎秒 60 回 | スケッチ | 前書き 3 行で通った。canvas の台が受ける |
| tixy | 32 文字の式、16×16 の点 | スケッチ | 通った。`circle` の 1 行に 1 本のつまみが付く |
| Shadertoy | `mainImage` の GLSL | ISF の効果 | 前書き 4 行で通った。時間で動く。iChannel と複数バッファは試していない |
| Turtletoy | `Turtle` と `walk(i)`、白黒の線 | スケッチ | 試していない。`Turtle` の台が要る |
| Art Blocks | p5.js と three.js の作品 | p5 の物はスケッチ | 試していない。three.js の場面を動かす台は無い |
| Paper.js、Two.js | canvas に描く絵の部品、どちらも MIT | 未定 | 試していない |
| Hydra | 鎖の JS が WebGL になる | 入れない | 本体が AGPL-3.0（effect_sets.md） |

山の大きさ：Dwitter の総数は見つからなかった。Shadertoy の 25,859 は、ラッパーの説明にある日付なしの例で、API に公開された分だけの数。fxhash は一次情報が無い。Art Blocks の「90% が鎖の上だけで描ける」は本人の発表。

権利：Shadertoy は、何も書いていない作品は CC BY-NC-SA 3.0 と二次情報が揃って言う。規約の本文では確かめていない。仕事に使えないのは OpenProcessing と同じ。Dwitter は About に書いていない。Vidvox の掲示板に、Shadertoy から写した物へ MIT を付けて上げた人の報告があるので、ISF の配布物を拾う時も出所を見る。

まだ食えていない物は「OpenProcessing 168 作の結果」のとおり。WEBGL は 41 作が白い箱だけ、シェーダーは 33 作が使い、p5 2.x 向けの 73 作は 1.11 の台で走っている。

## 出典の無い決め

- 「スケッチを行で開く」は私の切り口で、手本は見つけていない。行を取る手段だけ Processing の Tweak Mode（ソースを書き換えて数字に届く）から借りた。
- グループの開閉状態はファイルに残さない。開け直すと閉じた状態から始まる。

## 読んだ深さ

- 全部読んだ：Coding Challenges と OpenProcessing のコード、p5.js と ThorVG の該当箇所、AbsentMotion のグループ周り。
- 検索の要約だけ：Apple Motion、Cavalry、After Effects、Premiere、Final Cut のグループの説明。ページ全文は読んでいない。
- Figma のグループが時間軸でどう見えるかの出典は見つからなかった。

## ライセンス

OpenProcessing の多くは by-nc-sa。ここまでは非営利のテストだけで、リポジトリには入れていない。仕事で使うなら作者ごとに許可が要る。

## 出典

- [CodingTrain/Coding-Challenges](https://github.com/CodingTrain/Coding-Challenges)
- [p5.js](https://github.com/processing/p5.js)
- [timesnap](https://github.com/tungs/timesnap)
- [Processing Tweak Mode の実装](https://github.com/processing/processing/tree/master/java/src/processing/mode/java/tweak)
- [Canvalry-scripts](https://github.com/phillip-motion/Canvalry-scripts)
- [cavalry-scripts-and-expressions](https://github.com/yevgenymakarov/cavalry-scripts-and-expressions)
- [OpenProcessing API の説明](https://documenter.gw.postman.com/view/metadata/2s9YC1Xa6X)
- [OpenProcessing Discover](https://openprocessing.org/discover/)
- [svgcanvas](https://github.com/zenozeng/svgcanvas)
- [canvas2svg](https://github.com/gliffy/canvas2svg)
- [Motion: Collapse and expand groups](https://support.apple.com/guide/motion/collapse-and-expand-groups-motn2bac0b74/mac)
- [Motion: Layers list controls](https://support.apple.com/guide/motion/layers-list-controls-motn669ec686/mac)
- [Cavalry Scene Tree](https://cavalry.studio/docs/user-interface/menus/window-menu/scene-window/scene-tree/)
- [After Effects Precomposing](https://helpx.adobe.com/after-effects/desktop/work-with-compositions/precomposing-and-nesting/precomposing-nesting-pre-rendering.html)
- [Premiere nest](https://helpx.adobe.com/ph_fil/premiere-pro/how-to/premiere-pro-nesting-sequence-cc.html)
- [FCP compound clip](https://support.apple.com/en-ae/guide/final-cut-pro/ver8e3f3205/mac)
- [Dwitter の About](https://www.dwitter.net/about)
- [Turtletoy の API](https://turtletoy.net/syntax)
- [tixy の紹介](https://flowingdata.com/2020/11/09/minimalist-creative-coding-environment)
- [shadertoy-api（数の例）](https://pypi.org/project/shadertoy-api/)
- [shadertoys-dataset](https://github.com/Vipitis/shadertoys-dataset)
- [Godot Shaders の注記（Shadertoy の権利）](https://godotshaders.com/shader/creation-by-silexars/)
- [Vidvox の掲示板（MIT の付け替え）](https://discourse.vidvox.net/t/user-bennoh-is-stealing-shaders-and-uploading-them-online-with-mit-license/2682)
- [Vidvox の掲示板（ISF への変換）](https://discourse.vidvox.net/t/two-weeks-of-isf-shader-conversion/2649)
- [Art Blocks の鎖の上の保存](https://artblocks.io/onchain)
- [Art Blocks Generator](https://docs.artblocks.io/protocol/on-chain-generator/)
- [Hydra の説明](https://hydra.ojack.xyz/docs/)
- [Paper.js](https://paperjs.org/about/)
- [Two.js](https://two.js.org/)
