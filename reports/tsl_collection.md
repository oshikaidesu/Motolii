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
