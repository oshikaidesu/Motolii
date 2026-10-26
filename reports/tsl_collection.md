# TSL の表現を片っ端から取り寄せた（10/26）

ネットで見つかる TSL（three.js の、JS で書くシェーダー）の表現を、多言語・まとめ中心に探した。見つけた物は元の形のまま取り寄せ、three.js 自身の変換器で WGSL にし、Motolii の Dawn で描いた。

## どこを探したか

1. **GitHub**：`three/tsl` を読み込んでいるファイルを全部数えた。言語や国に関係なく拾える。
   - 1,050 件、置き場（リポジトリ）にして 518 個。
   - 星の数とライセンスを全部取り、表現が中心の物を選んだ。物理エンジンやエディタのように、TSL を部品として使っているだけの物は除いた。
   - AI に作らせた試作のような物（同じ名前の置き場が 4 つあるなど）も多く混ざっていた。
2. **まとめ（キュレーション）**：内蔵のブラウザで、それぞれのサイトの一覧を直接見た。
   - threejsresources.com の「Best TSL Tools」
   - Codrops の TSL タグ（17 本）
   - Zenn（API で直接検索）
3. **ChatGPT**：
   - アプリ（Instant）で 1 回。40 件。ライセンスを確かめてあるのは 6 件だけ。
   - Codex の CLI（`codex --search exec`、型は gpt-6-chat-thinking）で、場所ごとに 5 本を同時に。合わせて 150 行。
     - 中国語（掘金・知乎・CSDN・Gitee・Bilibili）
     - 日本語（Zenn・Qiita・note・X・ICS MEDIA）
     - 韓国語・ロシア語（velog・tistory・Habr）
     - スペイン語・ポルトガル語・フランス語・ドイツ語
     - 英語のまとめ
   - 答えの置き場は、全部 GitHub で星の数とライセンスを確かめ直した。Codex が「MIT」と書いた物のうち 5 つは、GitHub ではライセンスが無かった。

## 分かったこと

- **多言語の場所で紹介されている物の半分以上は、three.js 公式の見本（MIT）だった。** 中国語・日本語・韓国語・スペイン語のどこを見ても同じ。各国の作り手は公式の見本から学んでいて、独自の TSL のコードを公開している人は少ない。
- 公式以外で、英語以外の作者の物：
  - ICS MEDIA（日本）の流体 4 本とティルトシフト（MIT）
  - hexianWeb（中国）のレゴ風の自然・粒子の地球（ライセンス無し）
  - puxiao（中国）の TSL 教本（ライセンス無し）
  - y11i（日本）の TSL 部品（MIT）

## 取り寄せた物（`filament/tsl/sources.tsv`、48 か所、1.7 GB）

- `./tsl/fetch.sh` が、一覧に書いた版（コミット）を丸ごと `filament/third_party/tsl/<名前>/` に取ってくる。中身は変えていない。ThorVG の取り方と同じ。git には載せない。
- ライセンス：MIT が 38、Apache-2.0 が 1、CC0 が 1、「MIT-by-Codrops」が 8。
  - MIT-by-Codrops は、Codrops の記事のデモで、置き場にはライセンスが書かれていない物。Codrops の決まり「特に書いていなければ MIT」で扱った。
- 主な物：
  - 集まり：tsl-textures、TSLFX、Shader Lab、ノイズの集まり（bandinopla）、y11i の部品と psrdnoise
  - 場面：powershot（カメラの再現）、銀河、海 3 つ、炎 2 つ、粒子 5 つ、Codrops の記事のデモ 18 本ほど、など
- three.js 公式の TSL の仕上げの部品（`examples/jsm/tsl/display`、45 本）は、`filament/tsl/package.json` の three.js 0.185.1 に入っている。

**ライセンスが無い、または不明なので、取り寄せずに見るだけにした物**：
singularity と tornado（MisterPrada）、hologram-particles、three.js-tsl-sandbox（Bruno Simon）、html-in-canvas-cracks、lego-stylized-nature、tsl-scifi-earth、lib3、subsurface-refraction-shader、web-ocean-3d（素材のライセンスだけ）、ICS MEDIA の tutorial-three（All Rights Reserved）、puxiao の TSL 教本、Simon_Dev_TSL_Shaders、tsl-easings（akella）、nibi。
使うには作者への確認が要る。

## WGSL にした（`filament/tsl/*.ts`、`./tsl/run.sh <名前>`）

変換はどれも three.js 自身の変換器（`convert.ts`）。効果の作り方は、各作者の README や説明のとおりにした。

| 集まり | 作者の使い方 | WGSL になった数 |
|---|---|---|
| tsl-textures（MIT） | 説明の書く欄に入れる（`docs/<名前>.md` の `material.<欄>Node = 名前(...)`） | 52 / 52 |
| Shader Lab（Apache-2.0） | `createLayer` の初めの値 → `createPass` → `applyLayerState` | 43 種、材質 73 / 73（流体だけは、ブラウザの描き先が要る作りで作れない） |
| three.js の仕上げ（MIT） | 見本と同じく、絵を 1 枚渡す | 32 / 48（残りの 16 は場面の奥行き・カメラが要る物が中心） |
| ノイズ（bandinopla、MIT） | README と同じ入力（`uv()*6 + sin(time)`） | 18 / 18 |
| TSLFX（MIT） | `const { nodes } = 効果()` → `material.colorNode = nodes.colorNode` | 8 / 8 |

- naga（WGSL の検査）：366 本中 364 本が通る。通らない 2 本は、前と同じ naga 側の未対応（数だけの式の抽象型）。
- 場面の作品（銀河、海、炎、Codrops のデモなど）は、作品ごとに描く手順が違うので、取り寄せただけ。まだ WGSL にしていない。

## Motolii の Dawn で描いた（`./tsl/draw.sh <写真>`、一覧の絵は `reports/tsl/sheet_*.jpg`）

描けた数は 153 本中 149 本。描く順は `sheet_*.names.txt`（左から右、上から下）。

| 集まり | 描けた数 | 見た目 |
|---|---|---|
| tsl-textures | 52 / 52 | 正しく出た。白や黒のマスは、凹凸の向き・位置に使う模様で、色を変えない物 |
| ノイズ | 18 / 18 | 正しく出た |
| TSLFX | 8 / 8 | 3 本（衝撃・散り・形）は、時間 0 の最初のコマで、まだ何も出ていない |
| Shader Lab | 42 / 43 | **1 回で描く効果だけが正しい**（下を参照） |
| three.js の仕上げ | 29 / 32 | 同じく、1 回で描く物だけが正しい |

**描けなかった 4 本**

| 効果 | 理由 |
|---|---|
| Halftone（Shader Lab） | 絵を 37 枚同時に読む。WebGPU の初めの上限は 16。作者も three.js も上限を上げるよう頼んでいない。ブラウザでどう通っているかは未確認 |
| pixelationPass | 奥行きの絵が要る |
| fxaa | 数の配列（`NodeUniformBuffer`）を渡す種類で、試しがまだ対応していない |
| colorBleeding | Dawn の WGSL の変換器（Tint）が中で失敗。原因は未確認 |

**大事な残り：何段も描く効果**
- Bloom、ASCII、Annotations、Blob Tracking などは、描けても見た目が違う。元の写真のまま見えたり、真っ白になったりする。
- 原因：これらは途中で別の絵を作ってから重ねる効果。ブルームの途中の絵、文字の絵（Canvas 2D）、追跡の結果などを作る。途中の絵を作るのは TSL ではなく、効果の JS の部分（`render()`）。
- 試しは、どの絵の口にも同じ写真を渡しているので、途中の絵が無い。
- 正しく出るのは 1 回で描く効果：Displaced Rings、Circuit Bent、Dithering、Lumen Print、Mesh Gradient、Threshold、tsl-textures の全部、ノイズの全部など。

**次の課題**：何段も描く効果を正しく出すには、効果の JS の部分を、Motolii の Dawn の上でそのまま回す必要がある。

## 試しを直した所

- `dawn_probe/tsl_on_dawn.cpp`：
  - 頂点の入力を、three.js の変換器の並び（`builder.attributes`）で渡すようにした。前は「0 番が UV、1 番が位置」と決め打ちしていて、模様がマスの 1/4 にしか描かれなかった。
  - サンプラーを、three.js がテクスチャに持たせている読み方のまま作るようにした。

## 何段も描く効果を正しくした：作者の render() ごと Dawn で回す（10/26）

- 何段も描く効果（Bloom、ASCII、Annotations、Blob Tracking など）は、途中の絵を作る JS の部分（作者の `render()`）も要る。そこで、three.js の描く仕組み（WebGPURenderer）と Shader Lab の作者の「画面無しで描く口」（`createHeadlessRenderer`）を、そのまま Dawn の上で回した。
- JS から Dawn を呼ぶ口は、Dawn の作者が出している組み立て済みの物（npm の `webgpu` 0.6.2 = dawn.node、MIT）。まず Node の上で確かめた（`filament/tsl/check_node/check.ts`、Node は確かめにだけ使う）。
- 足した物（どれも、ブラウザなら最初からある物）：
  - Canvas 2D：`@napi-rs/canvas`（MIT、Skia）
  - 字形：作者の部品に同梱の 3 つ（Geist Sans・Geist Mono・BSMNT Grotesque）を、作者の `fonts.css` の名前で登録
  - `copyExternalImageToTexture`：Dawn の JS の口では「未実装」（`GPUQueue.cpp:120-125`）。Mystral と同じ手順（画素を読む → 上下反転 → α の掛け算 → `writeTexture`）で渡した
  - 描く繰り返し（`requestAnimationFrame`）：60 Hz の時計
- 作者の作りに合わせた所：
  - 効果の描く手順は裏で用意され、用意ができてから効く。なので、待ちを挟んで 40 回描く。
  - 時計を 1/60 秒ずつ進める。追跡は時刻が進んだコマだけ 1 歩進む。
  - 模様の画像は、作者の公開版の置き方（`dist/src/renderer`）からの住所で読む。
- **結果：Shader Lab の 44 種が全部描けた**（`reports/tsl/sheet_shader-lab_author-render.jpg`、順は `.names.txt`）。前は作れなかった流体（Fluid）と、上限を超えた Halftone も描けた。
- **ブラウザとの見比べ**（使う人がサイトに入れたのと同じ画像で、同じ効果をブラウザに足して比べた）：
  - Displaced Rings：大きく違う画素 15.7%、平均の色 74/64/51 に対して 73/62/45。WGSL だけで描いた時と同じ数字で、残りはブラウザの画面を撮った時の縮め方の差。
  - Bloom：Bloom が足した明るさ（Bloom 有り − 無し）が、ブラウザ +6.2、Dawn +6.2 で同じ。場所の一致（相関）0.85。残りは、ブラウザの画像の層が奥行きで画像を少し動かしている分（`parallaxMotion: orbit`）。
  - Photographic Cells：ブラウザでも初めの値では「ほとんど黒で、左と右下に少し写真が見える」。こちらも同じ。
  - ASCII、Annotations、Blob Tracking（枠・線・座標の札）、Pattern：作者の字形・模様で描けた。数字での見比べはしていない。
- まだ：
  - Magnify Lens と Pixel Trail はマウスで動かす効果で、マウスを渡していない。
  - これは Node の上の確かめ。Motolii の中で回すには、同じ口（Dawn の JS の口と `@napi-rs/canvas`、どちらも Node-API で書かれている）を Motolii の Dawn から組み立て、Hermes の Node-API（Hermes の本家に入っている、Node-API 第 10 版）に載せる。

## 問題を切り直した：作者の JS の部分は何をしているか（10/26）

- 何段も描く効果を Motolii の中で動かすために、「Web の作品を丸ごと動かす場所」を探し続けた（Mystral → Node を組み込む → Hermes → CEF）。どれも足りない所を自作で埋める形になった。既存の物が見つからなかった時点で、切り方を疑うべきだった。
- 切り直した：Shader Lab の 44 種について、作者の JS の部分が実際にしていることを、作者のコードから数えた（各 `*-pass.ts` の描き先の作成、描く呼び出し、Canvas、読み戻し、マウスの扱い）。

| JS の部分がしていること | 本数 | 効果 |
|---|---|---|
| つまみの値を渡すだけ（シェーダー 1 回。値から作る小さなデータの絵を含む） | 25 | chromatic-aberration、circuit-bent、directional-blur、displaced-rings、displacement-map、dithering、dot-grid、edge-detect、erosion、fluted-glass、gradient-map、gradient、grain、halftone、lumen-print、photographic-cells、pixelation、posterize、relief、shape、signal-rot、slice、smear、threshold、voxel |
| シェーダーを何段も回す（描き先を作って順に描く） | 12 | bloom、crt、connected-dots、flares、ink、outline、particle-grid、pixel-sorting、plotter、focus-blur、glass、photocopy |
| Canvas で文字や模様の絵を作る | 4 | ascii、pattern、annotations、blob-tracking |
| CPU で読み戻して計算する | 2 | blob-tracking（追跡）、annotations |
| マウスで動かす | 3 | fluid、magnify-lens、pixel-trail |

（blob-tracking と annotations は 2 つの欄に入る。halftone は、ブルームのつまみを入れた時だけ何段も描く。）

- **25 本（57%）は外の実行環境が要らない。** three.js の変換器で WGSL にし、Motolii の Dawn で描ける（この報告の前半）。
- 残りの 19 本が要るのは、Web の場の全部ではなく、作者の JS が three.js の描く仕組みを通してする 4 種類だけ。
  - 描き先を作る
  - 順に描く
  - Canvas で絵を作る
  - 読み戻す

## 4 種類を誰が受け持つか（10/26）

- 何段も描く 19 本の作者の JS が、three.js の描く仕組みを通してしているのは、4 種類（描き先を作る・順に描く・Canvas で絵を作る・読み戻す）。それぞれ Motolii に同じ働きの物はある。ただ、作者の JS が呼ぶのは Web の決まりの呼び出しなので、そのままでは届かない。
  - 描き先・順に描く：Motolii の画面の効果の口（`screen_door`）は、Filament の決まり（絵とサンプラーを組にして `@group(0)`、頂点は自前、`screen_door.h:27-32`）。three.js の WGSL（テクスチャは `@group(1)`、頂点も three.js の物）は載らない。そのまま描けるのは、装置を借りて描く口（`gpu_door`、`tsl_on_dawn` で確かめ済み）。
  - Canvas の絵：ThorVG は Canvas 2D の決まりを持たない（形・文字・絵の自前の部品だけ）。Shader Lab は Canvas 2D の呼び出しを 19 種類使う。ThorVG で受けると、1 つずつ訳すことになる。
  - 読み戻し：`thing_fx_cover` は Filament の物を読む口。作者の JS が呼ぶのは three.js の `readRenderTargetPixelsAsync`、その先は WebGPU の `mapAsync`。
  - 追跡の計算：`tracker.ts` は QuickJS でそのまま動く（済み）。
- なので、作者の JS から見える口を、Web の決まりの形で出す。中身は、どれも既存の物。
  - JS エンジン：Motolii の QuickJS-ng のまま
  - Node-API：NativeScript の `quickjs-api.c`（napi-android、OpenJS の MIT）
  - WebGPU：Dawn 自身の口（dawn.node、Node-API で書かれている）
  - Canvas 2D：`@napi-rs/canvas`（Node-API）
- 確かめた事実：
  - `quickjs-api.c` は、NativeScript が手を入れた QuickJS-ng 0.11.0 に合わせてある。足された関数 3 つ（`JS_IsArrayBuffer2`、`JS_NewString16`、`JS_WeakRef_Deref`、`quickjs.h:1273-1276`）を使うので、Motolii の 0.17.0 の見出しでは組み立てられない（誤り 18）。0.17 に書き換えると移植になるので、NativeScript の QuickJS-ng（`source_ng`）とひと組で借りる。
  - Node-API の関数の数：NativeScript は 130 を持つ。Dawn の口が要るのは 72 で、足りないのは `napi_fatal_error` の 1 つ。`@napi-rs/canvas` が要るのは 67 で、足りないのは裏の糸の仕事（async work と threadsafe function）の 7 つ。どれも組み立て済みの部品の中身（`nm -u`）から数えた。
  - NativeScript の組は、`.node` の部品を読み込む所を持たない。読み込みは `dlopen` して `napi_register_module_v1` を呼ぶだけで、Hermes（`hermes_napi_load_module`）と Node に手本がある。

## 効果を動かす JS の土台：QuickJS の上の Node-API を試して、Node にした（10/26）

- 作者の JS（three.js の描く仕組み、Canvas 2D）を Motolii の中で動かすには、Node-API を持つ JS の土台が要る。Dawn の WebGPU の口も、`@napi-rs/canvas` も、Node-API で書かれているため。
- Motolii の QuickJS で済ませたかったので、QuickJS の上の Node-API の既存の物を試した。
  - **NativeScript（napi-android の `quickjs-api.c`、130 関数）**：NativeScript が手を入れた QuickJS-ng 0.11 とひと組で組み立てられ、Dawn の口の読み込みと `create()` まで通った。ただし `napi_create_promise` が、約束を果たす関数を関数の中の一時的な置き場に入れて、その場所を覚えている（`quickjs-api.c:3674-3678`）。関数が終わると消える場所なので、`requestAdapter` の約束を果たす所で「関数でない」と止まった（lldb で `napi_resolve_deferred` の中と確かめた）。NativeScript の不具合。
    - 途中で、Dawn の口が読み込み時に Node の `Event`・`EventTarget`・`DOMException` を前提にしていることも分かった（`ManualConstructors.cpp:30-77`）。
  - **BabylonJS/JsRuntimeHost（MIT、QuickJS の Node-API は 107 関数）**：約束は正しく作る（果たす関数を JS の入れ物に入れる）が、Dawn の口に 4 つ、`@napi-rs/canvas` に 13 足りない。ループ（`AppRuntime`）と Web の部品の補い（時計、URL、Fetch など）は持つ。
  - **wasmerio/napi**：星 2、ライセンスが書かれていないので使えない。
- どれも、足りない所を自作で埋めることになる。Node-API の本家は Node で、Dawn の口と `@napi-rs/canvas` が前提にしている物（`setImmediate`、`Event`、`EventTarget`、`DOMException`、`structuredClone`、`URL`）を全部持つ。保守・検証する範囲が一番小さく、責任元もはっきりするので、Node を部品として組み込む（Node の組み込みの口と公式の見本 `test/embedding/embedtest.cc`）。
- 試しに書いた QuickJS 用の外側（`host.c`、約 90 行）は消した。

## Dawn の WebGPU の口を Motolii の Dawn から組み立てた（10/26）

- Dawn 自身の JS の口（dawn.node）を、Motolii の Dawn の組み立て（Filament の `out/wgpu`）に `DAWN_BUILD_NODE_BINDINGS=ON` を足して組み立てた（`filament/tsl/dawn_node.sh`）。44 秒、13 MB。
  - 材料 3 つ（node-api-headers、node-addon-api、gpuweb の `webgpu.idl`）は、Dawn の `DEPS` が決めた版で取った。
  - Go は Homebrew の 1.27.3（Dawn の指定は 1.26.6）。
- 確かめ：確かめの台本（`check_node`）で、npm の組み立て済みの口の代わりにこれを使った（`DAWN_NODE=<path>`）。Displaced Rings と Bloom で、絵は 1 バイトも違わない（640×360、違う画素 0）。
- これで、効果を描く装置が Motolii と同じ Dawn から作られる。まだ、Motolii の Filament が使っている装置そのものは渡していない（Dawn の口の `create()` は自分で装置を作る）。

## Node をやめて QuickJS の上の Node-API をもう一度試した（10/26）：途中で止めた

- 分かった事（数字は全部この Mac で動かして見た物）：
  - NativeScript の約束の不具合は、JsRuntimeHost の形（resolve と reject を 1 つの JS の入れ物に入れて、参照で持つ：`js_native_api_quickjs.cc:2314-2391`、980cfb0）に直すと、`requestAdapter()` が通った（apple、metal-3）。
  - 入口を NativeScript の `napi_run_script` と同じく「JS の中にいる」と数える形にしないと、Node-API の呼び出しの途中で後回しの処理が走る（`quickjs-api.c` の `js_enter`/`js_exit`）。
  - `requestDevice()` で 2 つ目の NativeScript の不具合に当たった：後片付けの関数が 2 回呼ばれる（FinalizationRegistry の後片付けと、入れ物そのものの後片付け）。AddressSanitizer で「二重に解放」と出た。同じ不具合の直しが NativeScript の Windows 版にある（NativeScript/windows af3a904、`packages/windows-quickjs/vendor/napi/quickjs/quickjs-api.c` の JSFinalizeValueCallback）。約束の不具合は NativeScript/runtimes（5b697b3、iOS/macOS 版）の方で直っている。直った所が木ごとにばらばら。
  - QuickJS-ng 0.17 のままにできるか：NativeScript の木は 3 つとも、NativeScript が手を入れた古い QuickJS-ng（0.8 か 0.11）と組む。Windows 版は 0.15 を横に置いても、Node-API の部分は 0.11 で組んでいる（`build.rs`）。0.17 の見出しで組むと誤り 19（`JS_IsArrayBuffer2` 8、`JS_WeakRef_Deref` 2、`JS_NewString16` 1、引数の数 4 など）。0.17 のままにするには書き換え（移植）が要る。
- 止めた理由：TSL を毎コマ JS で動かす必要がないため（下の節）。作者の JS は取り込む時に 1 回だけ動かし、その時は Node の確かめの台本（`check_node`）で足りる。試しの外側（`host.c` など）と直しのファイルは消した。

## 取り込みは 1 回、再生は three.js なし：Dawn の記録と流し直し（10/26）

- 考え方：TSL は JS でシェーダーの組み立てを作り、WGSL にする仕組み。なので、作者の three.js を取り込みの時に 1 回だけ動かし、GPU にした事を全部取っておけば、再生の時に three.js は要らない。取っておくのは WGSL だけでなく、入力の繋ぎ方（bind group）、頂点、描き先、何段描くかの順番の一式。
- 一式を取る仕組みは Dawn 自身が持っていた。
  - 記録：Dawn の上の WebGPU 層（`src/dawn/native/webgpu`、内側は同じ Dawn の Metal）が、通った呼び出しを全部、命令と中身の 2 本に書く（`CaptureContext`、`StartCapture`）。
  - 流し直し：Dawn の replay（`src/dawn/replay`、`Capture::Create` → `Replay::Create` → `Play`）。名前（label）で描き先を取り出せる（`GetObjectByLabel`）。
  - 手本：Dawn の試験 `DawnTest.cpp` の Recorder（184-230 行）と `CaptureAndReplayTests.cpp`（52 個の試験）。
- 組み立て：`filament/tsl/dawn_capture.sh` が、記録できる dawn.node を Motolii の out/wgpu とは別の場所（`out/wgpu_capture`）に組む（4 分 44 秒）。Motolii の Dawn は触らない。
  - 足したのは dawn.node の `capture=<path>` の合図だけ（`filament/tsl/dawn_capture.patch`、Dawn のコードに 2 か所、`MOTOLII_DAWN_CAPTURE` の時だけ）。アダプタの頼み方は DawnTest.cpp:708-713、記録の始め方は Recorder::CreateAndStart のとおり。
  - 確かめの台本 `check_node/check.ts` は 3 か所だけ変えた：dawn.node の設定を `DAWN_FLAGS` で渡す、最後の描き先に名前 `motolii_out` を付ける（three.js は名前を GPU の label にそのまま渡す：three.webgpu.js 76623）、最後に装置を閉じる（記録が終わる）。
- 再生：`filament/dawn_probe/replay_on_dawn.sh <記録> motolii_out 640 360 <出力>`。Motolii の Dawn（out/wgpu）に元からある replay の部品（`libreplay.a`）で流し直し、描き先を読み戻す。three.js も JS も使わない。
- 結果（640×360、ユーザーの写真）：

| 効果 | 記録（命令＋中身） | 流し直した絵と Node の絵 |
|---|---|---|
| Displaced Rings（1 段） | 65 KB ＋ 924 KB | 1 バイトも違わない、装置の誤り 0 |
| Bloom（何段も） | 158 KB ＋ 924 KB | 1 バイトも違わない、装置の誤り 0 |

  - 記録用の層を通しても、Node の絵は前（`own_*.rgba`）と 1 バイトも違わない。
  - 絵：`reports/tsl/replay_no_three.jpg`（左から写真、Displaced Rings、Bloom。どちらも流し直した物）。
- まだの事：
  - 今の記録は「その時の 41 コマ」をそのまま流す。時間・つまみ・写真を変えるには、記録の中の uniform の入れ物と写真の描き先を、名前で取り出して書き換える（`GetObjectByLabel`）。three.js がどの名前を付けるかを数えるのが次。
  - JS が毎コマ描く段や作りを変える作品（追跡、流体など）は、記録をそのまま流すだけでは足りない。44 種のうちどれがそうかは、まだ数えていない。
  - Motolii の Filament の装置の上で流し直すこと（今は replay_on_dawn が自分で装置を作る）。

### Replay を本番の再生に使えるか（10/26）：使えない

- 確かめた事：`PlayFrame()` で 1 コマずつ流せるか。
  - 結果：Displaced Rings も Bloom も、`PlayFrame()` は 1 回で記録の最後まで流れた（絵は Node と同じ）。
  - 理由（Dawn のコード）：Replay の「1 コマ」の区切りは、画面に出す瞬間（`surface.Present()`、Replay.cpp:1510-1515）だけ。画面なしの記録には区切りが無い。
- 仮に画面つきで記録して区切りを入れても、Replay は読み位置を前に進めるだけ（`PlayFrameImpl`、Replay.cpp:1594-1602）。同じコマを、時間を変えてもう一度流す口は無い。しかも各コマには、記録した時の時間を書く命令（writeBuffer）がそのまま入っている。
- なので、時間を変えて動かすこと、毎コマの時間を測ることは、Replay ではできない。Replay は「取り込んだ一式が正しいか」を確かめる道具として使い、本番の再生には使わない。
- Motolii が既に持っている「何段も描く・毎コマ時間を渡す」仕組みは ISF の PASSES（`filament/isf.cpp`：TARGET、PERSISTENT、FLOAT、大きさ。WGSL にして Motolii の画面で描く）。TSL の何段も描く効果をこの形に載せられるかは、まだ確かめていない。

### 1 コマは何でできているか：記録を数えた（10/26）

- 切り直し：「何で再生するか」を選ぶ前に、「この効果の 1 コマは実際に何でできているか」を記録から数えた。
  - 道具は `filament/dawn_probe/capture_list.sh <記録>`。Dawn 自身の記録の読み歩き（CaptureWalker、SurfaceDiscoveryVisitor、ProcessEncoderCommands）の上に、作る・書く・送る・命令の中身を 1 行ずつ出す見方を足しただけ。
- Bloom（作者の初期設定）：最初に全部作ったあと、281 回送るだけで、値を書く命令は 0。時間で変わらない。
- Displaced Rings（作者の初期設定）：作ったあと 73 回送るだけで、値を書く命令は 0。時間で変わらない。
  - なので、この 2 つでは「時間を変えて同じ絵になるか」は確かめようがない。
- CRT（毎コマ `timeUniform.value = time`、crt-pass.ts:235-236）を記録して数えた。最初に作る物が終わると、40 コマすべてが同じ形。
  1. 時間を書く：buffer #20 の 20 バイト目に 4 バイト。中身は 1/60、2/60、…、40/60 秒。
  2. 描く段 6 つ：描き先・パイプライン・入力の束がコマごとに同じ（各 41 回）。
  3. 前のコマの絵を残す写し 1 つ（CopyTextureToTexture）。
  4. 描く段 1 つ。
  - 違うのは 1 か所だけ：最初の段が読む入力の束（bind group）が 96 と 107 で 1 コマおきに入れ替わる。前のコマの絵を交互に使う作り。
  - 毎コマ新しく作る物は、命令の入れ物以外に無い。
- 分かった事：CRT は、取り込んだあと JS が毎コマやる事は「時間の 4 バイトを書く」だけ。残りは、最初に作った物を決まった順で描く命令の並び（2 コマで 1 周）で、記録の中にすべてある。
- まだの事：この並びを、時間を書き換えながら繰り返し流すこと。Dawn の Replay は前に進むだけなので、そのままでは繰り返せない。
