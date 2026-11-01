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

やり方：`door.h` がコードの形を読み分け、`later door` か窓の ⌘V で舞台に置く。実物を Git の外に取り、1 本ずつ通して 2 秒目の絵を目で見た。数は 11/1 夜の最後の走行の物で、その後の小さな直し（下の注）の後の全走行は Mac が落ちて終わっていない。入口の見本は `filament/check/door.sh` が毎回確かめる。

| 入口 | どこから取ったか | 本数 | 絵が出た | 出なかった物 |
|---|---|---|---|---|
| Turtletoy | 作品の頁の `<pre id="editor">` | 14 | 14 | 無し |
| tixy | tixy の見本 | 19 | 19 | 無し（他に 1 本は私の抜き出しミス） |
| Dwitter | 公開 API（総数 26,542 本） | 24 | 19 | 黒 3 本は原因未確認、絵文字の帯 3 本は絵文字が出ないだけで絵としては欠ける、重い 2 本は時間切れ |
| Paper.js | paperjs.org の見本 | 14 | 11 | 3 本はマウスか画像が要る |
| Hydra | hydra-examples | 4 | 2 | 1 本はコードどおり最後が黒、1 本はコメントだけ |
| Shadertoy（自分で描く） | GitHub のコード検索の Shadertoy 形のファイル | 14 | 8 | `#include`、Common が別、自前の uniform など 6 本。タブ全部の JSON は下の節で 261/300 |
| Shadertoy（画像を読む） | 同上、動画の上で | 26 | 24 が入る | 多くは複数バッファの 1 枚だけなので見た目は素通し。`#iUniform` と自前の uniform の 2 本は入らない |
| three.js | 公式 examples r186 | 10 | 6 | 線だけの 1 本は入らない。外のモデルやフォントが要る 3 本は中身が出ない |
| Two.js、Canvas | 自分の見本のみ | 各 1 | 各 1 | 実物は通していない |
| GLSL Sandbox | 自分の見本のみ | 1 | 1 | 実物は通していない |

注：最後の走行の後に、色のまとめ方、複製を 1000 までにする、`#define iTime` を消す、貼り付けの照合を最後に 1 回にする、を直した。`check/door.sh` と箱 2000 個の場面では確かめた。`#define iTime` を消して s21 が通るかは確かめていない。

Motolii の中にブラウザは無い。`libmotolii.dylib` に WebKit も Chromium も繋がっていない。コードは QuickJS で走り、2D の形は ThorVG、シェーダーは Dawn へ行く。ただし 2D の入口の絵は世界の中で 1 枚の板。Shadertoy と Hydra は舞台の大きさの四角に掛かる効果。世界に本物の立体で立つのは three.js と p5 の WEBGL だけ。

three.js：形と材質の組ごとに 1 つの立体。立体が 48 を超えると同じ形の近い色をまとめ、それでも超えれば同じ形を 1 つにする。1 つの立体の複製は 1000 まで。カメラは動きに焼くので、世界のカメラで回り込める。立たない物はチェックリストの T の行。

Mac が落ちた件（11/2 00:50）：カーネルの watchdog で再起動。ログは圧縮メモリ 100%、スワップ 35 個。窓の全試験と実物の全走行を私が同時に流していた。20:50 の JetsamEvent に `flutter_tester` 1 個で 33.37 GB の記録がある。落ちた瞬間のプロセス表は残っていないので、00:50 の犯人は確定ではない。

直した事：スケッチの QuickJS に 2 GB の上限（`JS_SetMemoryLimit`）。10 万個を描く dweet は 2.3 GB で「out of memory」の表示で止まる。上限を入れた後、窓の全試験 118 本を見張り付きで 1 本だけ流して、全部通り最大 3.1 GB。20:50 の 33 GB はどのテストだったかは分かっていない。重い物は同時に流さない。

山の大きさ：Dwitter の総数は見つからなかった。Shadertoy の 25,859 は、ラッパーの説明にある日付なしの例で、API に公開された分だけの数。fxhash は一次情報が無い。Art Blocks の「90% が鎖の上だけで描ける」は本人の発表。

権利：Shadertoy は、何も書いていない作品は CC BY-NC-SA 3.0 と二次情報が揃って言う。規約の本文では確かめていない。仕事に使えないのは OpenProcessing と同じ。Dwitter は About に書いていない。Vidvox の掲示板に、Shadertoy から写した物へ MIT を付けて上げた人の報告があるので、ISF の配布物を拾う時も出所を見る。

まだ食えていない物は「OpenProcessing 168 作の結果」のとおり。WEBGL は 41 作が白い箱だけ、シェーダーは 33 作が使い、p5 2.x 向けの 73 作は 1.11 の台で走っている。

## 2 日目に足した入口の実測

いずれも Git の外（作業用の場所）に取って 1 本ずつ流した。作者の権利は各作品のまま。

| 入口 | 取った所 | 取り方 | 結果 |
|---|---|---|---|
| Bonzomatic | CookieCollective/Live-Coding-Sources（ライセンス表示なし）の 3 回分 16 本 | GitHub のコード検索でフォルダを見つけ、raw を curl | 16/16 が塗って動く。音（`texFFT`）と画像（`texNoise` など）はゼロで読むので、音で動く作品は止まって見える |
| twigl（つぶやき GLSL） | darosh/px-stream の `assets/mini-shaders`（XorDev、Yohei Nishitsuji など。多くは CC BY-NC-SA 4.0 と作品に書いてある）から無作為に 24 本 | 同じ | 18/24。落ちた 6 本は全部 `#include`（その倉庫の書き方で twigl の形ではない） |

## Shadertoy のタブを全部食う

問い：「食えるものを全て食えるように」。11/1 の貼り付けは 1 タブだけで、自分で描く作品が 8/14 だった。

見た所と読み方：ユーザーの昔の倉庫 `oshikaidesu/rust_ae` の `docs/design/vism/shadertoy-import.md`（1〜70 行）と `shadertoy.rs`（90〜210 行）を読んだ。VVISF-GL の `ShadertoyConverter.cpp`（BSD-3-Clause、1345 行）は該当の場所だけ読んだ（タブの並び、出力の id、関数の扱いの説明）。Shadertoy のサイトは見ていない。実物は `GabeRundlett/shadertoy-api-shaders`（36,668 本の API の写し）の GitHub から無作為に 300 本を raw で取り、Git の外に置いた。Cloudflare の確認は通していない。

やり方：Hydra と同じく入口で 1 本の ISF に変えて Program の棚に置く。タブは書かれたまま残し、違いはマクロで周りから言う。`iChannelN` は番号にして `door_read` が読むので、関数に渡すサンプラーもそのまま動く。2 つのタブにある同じ名前は、タブごとに別の名前へマクロで分ける（rust_ae はここを断っていた）。`iTime` などは本物の変数にしたので、作品の引数の `iResolution` や `#define time iTime` とぶつからない。1 タブの貼り付けも同じ道に乗せ、前の正規表現は捨てた。

測り方：1 本ずつ `later door` → 8 秒を 480×270 で書き出し、0 コマ目から順に回した。1 秒、3 秒、7.9 秒の絵を取り、色の数と動きで分けた。見張り付き、最大 302 MB。

| 結果 | 本数 |
|---|---|
| 絵が出て動く | 261 |
| 黒 | 30 |
| 入らない | 9 |

300 本のうち 90 本が複数タブ。黒 30 本のうち 26 本は入力待ち：マウス 19、テクスチャ 11、動画 6、キーボード 3、ほか音など（重複あり）。入力に頼らないのに黒い 4 本（Bio Life、lime green fractal、1/2, 1、Homeopathic Shader）は原因を見ていない。

この走行の後に `uniform` の残りを直して `wttyWB` は通った。全 300 本は流し直していない。入らない 9 本の内訳：

- 重くて打ち切り 1（Spectral Cornell Box。前の走行は黒）
- 作品の中の `#define mainImage` 1
- ローカル変数 `length` のあとに関数 `length` を呼ぶ 1
- シフト幅が 32 以上の定数で Tint が拒む 1
- 精度付きの関数の重ね定義 1
- コメント末尾の `\` の後の警告で止まる 1
- Cube A のタブ 1（断る言葉を出す）
- 理由を出さず書き出しが無い 1（`4llfR4`、原因未確認）
- `uniform` の残り 1（`wttyWB`、上のとおり直した）

マウスは Effect のカードの `Mouse` つまみ（0〜1）になる。`js_<ファイル>_door_mouse_x` と `_y` にキーが打てる。粉のシミュレーション（Basic powder thing）にキーを打つと、軌跡どおりに粉が出て下に積もるのを絵で見た。Shadertoy ではマウスで描くしかない物が、動画の中で打ち直せる。

入力の扱い：テクスチャ・動画・ウェブカメラは置いた物の絵を読む。何も選んでいないと四角の 1 色なので、写真やノイズの作品は別物になる。キーボード・音・キューブ・ボリュームはゼロを読む。

足りない事はチェックリストの T の行：組み込みのノイズ、タブを 1 枚ずつ貼る入口、mipmap、32 bit の絵、時間を飛ばした時。

JSON の手に入れ方：サイトの画面からはタブごとにしかコピーできない。JSON は自分の作品の書き出しか API の写しになる。

## p5 の shader() を通す（11/2）

見た所：OpenProcessing の Discover の 3D タブから取った 60 作のうち、WEBGL と書く 32 作。うち 17 作はタブを別ファイルのまま取り直した（1.6 秒に 1 回の API。Git の外）。

やり方：本物の p5.js 1.11.13 を走らせ、`shader()` と `setUniform` を受け取るだけにする。平らな物（rect・plane・quad・beginShape）を塗る時は、その GLSL をそのまま ISF の効果として書き出す。

- 頂点シェーダーは `isf.cpp` にある ISF の .vs の道を通す。自分で書き直していない
- `aPosition` と `aTexCoord` は画素の位置から出す。平らな物なら同じ値になる
- p5 の行列（`uProjectionMatrix`）を使う頂点シェーダーは y が上から、使わない物は下から数える
- `setUniform` の値は 0.1 秒ごとに 30 秒読む。動かない数はつまみ、動く数はキー、一直線なら 2 キー（0 秒と 600 秒）
- 値がスケッチの大きさに比例する vec2 と vec3 は舞台の大きさに置き換える
- 0〜1 で動かない vec3 と vec4 は色のつまみ
- 最初の sampler2D は置いた先の絵を読む。動画にも掛かる。残りの sampler2D は何も読まない
- `loadShader` の 2 ファイルは .js の隣にあれば読み込んで .js の中に入れる。隣に無ければ理由を言って止まる

見つかった事が 2 つある。

1. **セミコロンを書かない作者のスケッチが動いていなかった。** 線の目印を `(__L=59,rect)(…)` と書いていたので、前の行の末尾に繋がって読まれていた。`__at(59,rect)(…)` に直した。これは WEBGL だけでなく 2D の作品にも効く。168 作の数は取り直していない
2. **Hydra の動く値はキーになっていなかった。** 前に「0.1 秒ごとに 30 秒ぶんのキーになる」と書いたのは誤り。キーの名前に効果のファイル名が付いておらず、どれも刺さっていなかった。直して `list` と絵で確かめた

結果（WEBGL 32 作、1 本ずつ、最大 657 MB、1 本 50 秒まで）：

| 結果 | 本数 | 中身 |
|---|---|---|
| 正しそうに見える | 7 | shader() の 6 本（2978719、2986290、3004455、3012850、3017691、3041731）と 2983693 |
| 動くが色か形が違う | 5 | 3026224、3026321、3027051、3037642、3043452。多くは立体に塗るシェーダーの系統で、塗りが出ない |
| 白い四角 | 3 | 2994531、3006412、3038344。ellipse や image を箱に取り違えている |
| 何も出ない | 5 | 2983658、2984836、2987225、3019565、3036535 |
| 50 秒で切れた | 8 | 3003417 は p5 の線の計算で 1 コマ 0.8 秒。3027758 など 5 本はライブラリ込みの 2900 行の系統。3029535 と 3043371 の原因は見ていない |
| 止まった | 4 | 2984234、2985676、3036518 は p5 2.x の命令、3019066 は外のライブラリ `brush` |

「正しそうに見える」は 1 枚の絵を僕が見た判定で、元の作品と並べて比べてはいない。shader() の 6 本は 1 本ずつ流し直して絵を見た。1 回目の一括では、ファイルの隣に GLSL が無い形で流したので 5 本が止まった。上の表はその 5 本を正しい形で流した結果に置き換えてある。

届いていない物：立体に塗るシェーダー、絵を読み戻すシェーダー（前のコマの絵）、sampler2D の 2 枚目以降、配列の値の動き、`mouseX` のようなつまみにならない入力。30 秒より先で動く値は最後の値で止まる（一直線の物は 600 秒まで）。

## プログラムの置き場

問い：「programタブが新しく作った方がいいのかな」「createとかのブラウザ画面とコード画面の切り替えができるようにした方がいいのかしら」「インスペクターで見ておきたいからな、ノブの数をライブで保存しながら見ておきたいし」。

見た所と読み方：Godot の exported properties と First look at the editor、Cavalry の JavaScript Editor と JavaScript Layers は本文を読んだ。Unity の Inspecting scripts と TouchDesigner の Custom Parameters、Text DAT は検索の要約だけ。

| ソフト | コードの場所 | つまみの場所 | 保存 |
|---|---|---|---|
| Godot | 上の Script 画面で打つ。ファイルは FileSystem の棚 | `@export` の値がインスペクターに出る | 値もディスクに残る |
| Unity | Project の窓のアセット | 保存した変数がインスペクターに自動で出る | シーンと一緒 |
| Cavalry | 層の JavaScript 欄、または JavaScript Editor の窓 | Attribute Editor | タブは閉じても残る。ファイルには Save As |
| TouchDesigner | Text DAT の中、編集は別の窓 | Parameter Dialog のカスタムパラメータ | プロジェクトの中 |

4 つで揃う事：つまみはインスペクターで、コードとは分ける。Motolii のつまみは今もインスペクターにある（`js_line_N`、Script のカード）。
ファイルとして棚に並べるのは Godot と Unity の 2 つ。Motolii の棚は Media、Effects、Fonts、Colors、Create と種類ごとなので、Program も 1 つの棚にする。
割れる事：コードをどこで打つか。Godot は真ん中を切り替え、Cavalry は別の窓。Motolii は舞台を見ながら直したいので、左の棚の表示を一覧とコードで切り替える。ここは見本から取っていない（出典なし）。

貼ったコードは今、シェーダーが Effects、スケッチが Create に入る。Program ができたら全部そこへ入れ、言語で分けない。

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
