# Webエフェクトを静的GPUパッケージへ変換する技術・既存実装の調査

調査基準日：2026年10月26日

英語・日本語・中国語・韓国語の検索語を使い、GitHub、公式仕様、エンジンのドキュメント、Zenn、Qiita、CSDN、掘金、three.js Discourse、Redditなどを横断して調べました。

## 先に結論

**WebGPUで実行したエフェクトから、シェーダー・リソース・パス構造を抽出し、JavaScriptなしのネイティブ環境で再生するための部品は、すでにかなり揃っています。**

特に重要なのは次の3つです。

1. **WebGPUReconstruct** — ブラウザ内のWebGPU処理をキャプチャし、ネイティブDawnまたはwgpuで再実行する。現在の構成に最も近いキャプチャ基盤。
2. **WebGPU Recorder** — WebGPUのAPI呼び出しを、初期化処理・フレーム処理・バイナリデータに分けた機械可読形式として出力する。静的パッケージ生成の出発点として非常に有望。
3. **ISF（Interactive Shader Format）** — シェーダー、パラメータ、複数パス、永続バッファをポータブルなデータとして宣言し、ネイティブホストが毎フレーム実行する。目指すパッケージの設計に最も近い。

ただし、この3つのどれも、要求されている変換全体を完成品として提供しているわけではありません。

最大の未解決部分は、次の変換です。

```text
WebGPUの実行記録
       ↓
意味を持つ静的パスグラフ
       ↓
時間・ユーザー入力・外部画像を
更新可能なパラメータとして再構成
       ↓
毎フレーム繰り返し実行できる
ネイティブGPUプログラム
```

実行記録から「何がGPUに送られたか」は取得できます。

しかし、そのデータのどの部分が `time`、`mouse`、`effect intensity`、入力動画、内部状態なのかは、通常、APIトレースだけでは確定できません。

今回の調査では、**任意のthree.js TSLマルチパスエフェクトについて、この意味づけまで自動的に実施する汎用OSSは確認できませんでした。**

以下では、この境界を明確にして既存技術を整理します。

---

# 1. TSL / three.js / NodeMaterial系

## 1-1. Three.js TSL / WGSLNodeBuilder

- URL：[Three.js Shading Language](https://github.com/mrdoob/three.js/wiki/Three.js-Shading-Language)
- 公式ドキュメント：[TSL](https://threejs.org/docs/pages/TSL.html)
- ライセンス：MIT
- 状態：活発に開発中
- 適合度：★★★★★（シェーダー抽出）

### 変換

```text
TSL Node Graph
    ↓
NodeBuilder
    ↓
WGSL / GLSL
```

NodeBuilderには、大きく分けて次の処理段階があります。

- setup
- analyze
- generate

WebGPU向けにはWGSLNodeBuilder、WebGL向けにはGLSLNodeBuilderがあります。

ユーザー側ですでに確認されているように、これをヘッドレスで動かせれば、JavaScriptが必要なのはインポート時だけです。

| 項目 | 対応 |
|---|---|
| TSL → WGSL | ○ |
| TSL → GLSL | ○ |
| 単一パスのbindings | ○ |
| 時間uniformの生成 | ○ |
| 複数パスの静的manifest | × |
| ping-pongの静的記述 | × |
| JSなしでの生成済みWGSL実行 | ○（別ホストが必要） |

重要なのは、TSLが表現するノードグラフと、レンダラーが実行するパスグラフは同じものではないことです。

NodeBuilderが生成するWGSLを取得しても、

- どのRenderTargetへ書くか
- 次のパスがどのテクスチャを読むか
- 履歴バッファをいつ交換するか

までは、それだけでは完結しません。

したがって、現在のWGSLNodeBuilder抽出技術は継続利用する価値が高いです。

---

## 1-2. Three.js WebGPU Post-processing / RenderPipeline

- URL：[Three.js WebGPU Post-processing](https://threejs.org/manual/pages/webgpu-postprocessing.html)
- ライセンス：MIT
- 状態：活発
- 適合度：★★★★★（抽出対象として）

Three.js自身がマルチパス処理を組み立てる仕組みです。

たとえば、

- Bloom
- FXAA
- Depth of Field
- SSAO / SSGI
- Temporal Reprojection
- Tone Mapping

などの効果では、シェーダー生成とは別に中間RenderTargetと処理順序が必要です。

現在のthree.jsではTSLベースのRenderPipelineも発展しています。

日本語の追跡記事でも、r184のRenderPipeline、MRT、SSGI、時間的再投影などが扱われています。

参考：[Three.js Release Catch-Up r184 — Zenn](https://zenn.dev/112ka/articles/threejs-catchup-r184)

### 重要な示唆

TSLのマルチパスエフェクトを取り込むなら、次の2層を区別する必要があります。

| 層 | 取得したいもの |
|---|---|
| NodeBuilder | WGSL、binding、uniform構造 |
| RenderPipeline / WebGPU backend | パス、RT、コピー、依存関係、履歴 |

この両方を組み合わせれば、APIトレースだけに頼る必要はなくなります。

ただし、Three.jsが完成済みのポータブルRenderGraphをそのままエクスポートする公式機能は、確認できませんでした。

---

## 1-3. takahirox / tsl-node-editor

- URL：[GitHub — tsl-node-editor](https://github.com/takahirox/tsl-node-editor)
- ライセンス：MIT
- 状態：実験的プロジェクト
- 適合度：★★★☆☆

Three.js TSL用のノードエディタです。

次のエクスポートをサポートします。

- TSLソース
- Material
- JavaScript / TypeScriptアプリケーション

WebGPUプレビューとコード表示もあります。

ただし、出力の中心はTSLおよびThree.jsのコードです。

**WGSL・パスグラフ・binding manifestをセットにしたネイティブ実行パッケージのエクスポートではありません。**

READMEにも、十分にテストされていない実験的プロジェクトだと明記されています。

参考実装としては有用ですが、そのまま変換エンジンにはなりません。

---

## 1-4. basementstudio / Shader Lab

- URL：[GitHub — Shader Lab](https://github.com/basementstudio/shader-lab/blob/main/README.md)
- ライセンス：今回確認したページだけでは確定できず
- 状態：公開リポジトリを確認
- 適合度：★★★★☆（取り込み対象として）

Shader Labは、TSLを使ってGPUシェーダーを記述・プレビューする環境です。

ユーザーコードは概ね次の形になります。

```javascript
export const sketch = Fn(() => {
  const color = vec3(
    uv().x,
    uv().y,
    sin(time).mul(0.5).add(0.5)
  );

  return color;
});
```

EditorはTypeScriptをコンパイルし、TSLを利用できるサンドボックス内で評価します。

Source ModeとEffect Modeがあります。

### 今回の用途における価値

Shader Labのように、入力画像と時間が明示的に与えられる環境は、完全に任意のthree.jsアプリケーションよりも取り込みやすいはずです。

そのため、一般的なthree.js作品をいきなりすべて取り込むより、Shader Lab形式を最初の対象にするのは合理的です。

ただし、そのまま静的GPUパッケージを生成する機能は確認できませんでした。

---

## 1-5. pmndrs / postprocessing

- URL：[GitHub — postprocessing](https://github.com/pmndrs/postprocessing)
- ライセンス：Zlib
- 状態：継続開発
- 適合度：★★★☆☆

Three.js用のポストプロセスライブラリです。

```javascript
const composer = new EffectComposer(renderer);

composer.addPass(new RenderPass(scene, camera));
composer.addPass(
  new EffectPass(camera, new BloomEffect())
);
```

特徴：

- 複数パス
- 中間RenderTarget
- ping-pong
- エフェクトの合成
- 色空間管理
- カスタムシェーダー

ただし、基本的にThree.jsのWebGLRenderer向けの仕組みです。

TSL / WebGPU RenderPipelineとは別の経路として扱う必要があります。

静的エクスポーターではありません。

参考になるのは、複数のエフェクトを結合してGPUパス数を減らす設計です。

---

# 2. グラフを保存できるWebエンジン

## 2-1. Babylon.js Node Material Editor

- URL：[Babylon.js Node Editors](https://github.com/BabylonJS/Babylon.js/blob/master/.github/architecture/node-editors.md)
- ライセンス：Apache-2.0
- 状態：活発
- 適合度：★★★★☆

ノードベースのマテリアルを作成し、グラフデータを保存する仕組みです。

概念的には、

```text
Node Material
    ↓
Serialized JSON
    ↓
Babylon NodeMaterial
    ↓
GPU shader
```

という構造です。

TSLのようなシェーダーグラフをシリアライズする先行例として有用です。

ただし、JSONを解釈するのはBabylon.jsのランタイムです。

ネイティブDawn向けのWGSLパッケージを出力するものではありません。

---

## 2-2. Babylon.js Node Render Graph Editor

- URL：[Node Render Graph Editor](https://www.npmjs.com/package/@babylonjs/node-render-graph-editor)
- 実装：[GitHub](https://github.com/BabylonJS/Babylon.js/tree/master/packages/tools/nodeRenderGraphEditor)
- ライセンス：Apache-2.0
- 状態：活発。調査時点でnpmの継続公開を確認
- 適合度：★★★★☆

こちらは特に重要です。

Node Materialとは異なり、Render Graph自体を扱います。

- レンダリングパス
- ポストプロセス
- 出力ターゲット
- パス間接続
- グラフの保存・読み込み

を扱うエディタです。

Babylon.jsのNode Editor群には、グラフのimport/exportを扱うserializationToolsがあります。

### 評価

今回欲しい形式にかなり近い構造を持っています。

ただし、これはBabylon.js内部のグラフを保存する仕組みであり、任意のThree.js TSLを変換できるわけではありません。

また、生成されたJSONをBabylon.jsなしのネイティブエンジンで再生するには、別途解釈器が必要です。

---

# 3. ISF：もっとも近い静的エフェクト形式

## 3-1. Interactive Shader Format

- URL：[ISF公式](https://isf.video/)
- 仕様：[GitHub — ISF_Spec](https://github.com/mrRay/ISF_Spec)
- ライセンス：MIT
- 状態：仕様・実装とも公開、複数ホストで利用
- 適合度：★★★★★

**完成させたいパッケージの意味論に、最も近い既存形式です。**

ISFは、GLSLにJSONメタデータを付ける形式です。

例えば、次の情報を記述します。

- 入力画像
- パラメータ
- パラメータの型
- デフォルト値
- 時間
- レンダリングパス
- 中間バッファ
- 永続バッファ
- バッファのサイズ

### マルチパス

ISFは `PASSES` 配列を持っています。

各パスでは、

- `TARGET`
- `WIDTH`
- `HEIGHT`
- `PERSISTENT`
- `FLOAT`

などを指定できます。

パス番号は `PASSINDEX` で取得できます。

### 前フレーム

`PERSISTENT` を使うことで、フレームをまたいで保持されるバッファを定義できます。

つまり、フィードバックエフェクトをデータとして記述できます。

### 時間

仕様には、次のような標準uniformがあります。

- `TIME`
- `TIMEDELTA`
- `FRAMEINDEX`
- `DATE`

### 入力

画像や動画などの入力に加えて、ユーザーが調整する数値パラメータも記述できます。

| 要件 | ISF |
|---|---|
| 静的shader | ○ |
| 静的manifest | ○ |
| 複数パス | ○ |
| パス順序 | ○ |
| 時間uniform | ○ |
| knobs | ○ |
| 入力画像 | ○ |
| previous frame | ○ |
| JSなしのネイティブ再生 | ○ |
| Three.js TSL変換 | × |
| WGSLをそのまま格納する標準形式 | × |

### 制限

ISFは基本的にGLSLフラグメントシェーダーを中心とした形式です。

任意のWebGPU compute pass、複雑なstorage buffer、indirect dispatchなどを完全に表現するものではありません。

そのため、今回の構成では、ISFをそのまま採用するというより、**ISFの入力・パス・履歴バッファの意味論を参考にWGSL対応の形式を設計する**のが有望です。

---

## 3-2. ISFMSLKit

- URL：[GitHub — ISFMSLKit](https://github.com/mrRay/ISFMSLKit)
- ライセンス：BSD
- 状態：公開ソース・テストアプリを確認。最終コミット日は未確認
- 適合度：★★★★★（ネイティブホストの参考）

ISFをmacOSのMetal上で実行するライブラリです。

重要なのは、ISFがOpenGLだけの形式ではないことです。

ISFMSLKitでは、GLSLベースのISFをMetalで動作させています。

つまり、

```text
Portable shader description
        ↓
Native backend adaptation
        ↓
Metal rendering
```

という経路の実例です。

今回の場合はMetalの代わりにDawnを使います。

この実装は、パスの生成、uniformの供給、入力画像、フィードバックの扱いを調べる価値があります。

---

## 3-3. ISFのC++実装

- URL：[ISF Specification / Source Code案内](https://github.com/mrRay/ISF_Spec)
- ライセンス：各実装で要確認
- 状態：仕様側がクロスプラットフォームC++実装を案内
- 適合度：★★★★★（ホストの構造）

ISF公式仕様では、macOSとWindows向けのクロスプラットフォームC++実装が案内されています。

OpenGLベースですが、目的は今回と非常に近いです。

特に調べたいのは、次の処理です。

1. エフェクトファイルのロード
2. 入力メタデータの解析
3. GPUリソースの生成
4. パスの実行
5. persistent bufferの管理
6. 毎フレームのuniform更新

これは、静的エフェクトプレイヤーの参照実装として有力です。

---

# 4. Shadertoyマルチパスのネイティブ移植

## 4-1. ShaderToyX

- URL：[GitHub — ShaderToyX](https://github.com/vinay/ShaderToyX)
- ライセンス：今回の検索結果では未確認
- 状態：公開実装を確認。最終活動日は未確認
- 適合度：★★★★★（パス・履歴の実装）

Windows向けのネイティブShadertoy実装です。

かなり重要な先行例です。

対応内容：

- Image pass
- Buffer A–D
- 複数パス
- RGBA16F framebuffer
- ping-pong
- `iTime`
- `iTimeDelta`
- `iFrame`
- `iMouse`
- `iResolution`
- `iChannel0`–`iChannel3`

READMEでは、各BufferをRGBA16Fのping-pong framebufferペアで管理すると説明されています。

今回の要件と比較すると、

```text
Shader source
     +
Channel mapping
     +
Uniform convention
     +
Ping-pong framebuffers
     ↓
Native GPU playback
```

という点がよく一致します。

ただし、任意のWebGPU/TSLグラフをインポートするわけではありません。

Shadertoyという限定された実行モデルをネイティブで再現しています。

---

## 4-2. SHADERed

- URL：[GitHub — SHADERed](https://github.com/dfranx/SHADERed)
- ライセンス：MIT
- 状態：GitHubの公開リポジトリ・リリースを確認
- 適合度：★★★★☆

クロスプラットフォームのシェーダーIDEです。

主な機能：

- GLSL / HLSL
- SPIR-V
- 複数シェーダーパス
- Render Texture
- Compute Shader
- Custom Variables
- シェーダーデバッグ
- フレーム解析
- ShadertoyImporterプラグイン

この中で重要なのは、ShadertoyImporterです。

Shadertoyから取り込んだ効果を、独自のGPU実行環境に移すという方向性が一致します。

また、`--convert` オプションや、SpearNodeによるシェーダー出力の仕組みも確認できます。

参考：[SHADERed Releases](https://github.com/dfranx/SHADERed/releases)

ただし、ShadertoyImporterがすべてのマルチパス・フィードバック構成を完全自動変換できるかは、今回の資料では確認できませんでした。

また、TSLには直接対応していません。

---

## 4-3. ShaderAmp

- URL：[GitHub — ShaderAmp](https://github.com/ArthurTent/ShaderAmp)
- ライセンス：未確認
- 状態：公開実装を確認
- 適合度：★★★★☆

Chrome / Firefox拡張です。

ShadertoyのURLまたはJSONからシェーダーを取り込めます。

特徴：

- Shadertoy JSON import
- GLSLの自動変換
- entry pointのラッピング
- uniformの変換
- Buffer A–Dの接続
- マルチパス対応
- 独自uniformの公開
- 時間制御
- 画像・動画入力

### なぜ重要か

単にGLSLを表示するだけではなく、**Shadertoyのパス接続とuniform conventionを別の実行環境へ適合させています。**

今回のインポーターでも、これと似た変換が必要になります。

ただし、再生環境はブラウザ拡張です。

JSなしのネイティブ再生ではありません。

---

## 4-4. Natron OpenFX Shadertoy Plugin

- URL：[Natron Shadertoy Plugin](https://github.com/NatronGitHub/openfx-misc/blob/master/Shadertoy/Shadertoy.md)
- ライセンス：対象リポジトリで別途確認が必要
- 状態：公開ドキュメントを確認
- 適合度：★★★☆☆

動画編集・合成ソフトにShadertoyを取り込む実例です。

ただし、重要な制限があります。

**プラグイン単体ではマルチパスに対応していません。**

複数のShadertoyノードを接続することで、マルチパス相当の処理を構成できます。

ドキュメントには、

- 3-pass circular bokeh
- 4-pass octagonal bokeh

の例が紹介されています。

今回の用途では、自動変換よりも、動画編集ホストへshader passを接続する先行例として有用です。

---

## 4-5. TouchDesigner GLSL TOP

- 公式：[GLSL TOP](https://docs.derivative.ca/GLSL_TOP)
- チュートリアル：[Importing Shadertoy](https://github.com/interactiveimmersivehq/Introduction-to-touchdesigner/blob/master/GLSL/12-6-Importing-Shadertoy.md)
- ライセンス：TouchDesigner本体は商用ソフトウェア
- 状態：継続開発
- 適合度：★★★★☆

TouchDesignerでは、GLSL TOPでシェーダーを画像処理ノードとして実行できます。

複数のTOPを接続するとマルチパス構成になります。

また、Feedback TOPなどを利用して過去フレームを参照する構成も作れます。

Shadertoyから移植するときは、

```text
iChannel0 → sTD2DInputs[0]
iChannel1 → sTD2DInputs[1]
```

のように入力を変換します。

uniformもTouchDesignerのパラメータへ割り当てます。

2025年公開のチュートリアルでは、複数パスとfeedback loopを含むGLSLエフェクトの移植が扱われています。

[GLSL Effect Porting in TouchDesigner](https://www.youtube.com/watch?v=EjXVGOac3QA)

### 評価

ネイティブGPUホストが、外部シェーダーを動画処理グラフへ接続する実例として重要です。

ただし、TouchDesignerのプロジェクトをDawn用の静的パッケージへエクスポートできるわけではありません。

---

# 5. GPU API Capture → Replay

ここが今回の調査で、最も実装上の可能性が大きい領域です。

## 5-1. WebGPUReconstruct

- URL：[GitHub — WebGPUReconstruct](https://github.com/Chainsawkitten/WebGPUReconstruct)
- ライセンス：今回の検索結果では未確認
- 状態：2026年3月にv2.0公開を確認
- 適合度：★★★★★

**ブラウザWebGPUからネイティブWebGPUへ移すという点で、極めて近いプロジェクトです。**

WebGPUReconstructは、

```text
Browser WebGPU application
         ↓
WebGPU API capture
         ↓
Capture file
         ↓
Native Dawn / wgpu replay
```

を実現します。

2026年3月のv2.0では、次の改善が報告されています。

- Native macOS replayer
- Dawn更新
- オブジェクト寿命の追跡
- WebGPU仕様変更への対応
- 任意フレーム数でのキャプチャ停止
- プログラムから使えるJavaScriptモジュール

参考：[WebGPUReconstruct 2.0 — Reddit](https://www.reddit.com/r/webgpu/comments/1rub7qp/webgpureconstruct_20/)

以前の発表でも、Dawnまたはwgpuによるネイティブ再生が明記されています。

[Original announcement](https://www.reddit.com/r/webgpu/comments/1go06sy)

### 対応範囲

| 要件 | 評価 |
|---|---|
| Browser WebGPU capture | ○ |
| WGSLを含むGPU処理の記録 | ○ |
| 複数パス | ○（API呼び出しとして） |
| RenderTarget | ○ |
| bind group | ○ |
| コピー | ○ |
| Native Dawn replay | ○ |
| Native wgpu replay | ○ |
| 時間uniformの意味を判定 | × |
| knobsを自動抽出 | × |
| 1フレームを任意時刻でループ | 未実証 |
| 汎用静的RenderGraphへの変換 | × |

### 今回の用途での評価

既にDawnを使用しているため、技術的距離が短いです。

特に、three.jsの実行をそのままキャプチャできる可能性があります。

ただし、このツールの主目的はデバッグと再現です。

`time = 1.0` でキャプチャしたuniformを、ネイティブホストが `time = 2.0` に更新できるようにする変換は、別途必要です。

---

## 5-2. Brendan Duncan / WebGPU Recorder

- URL：[GitHub — webgpu_recorder](https://github.com/brendan-duncan/webgpu_recorder)
- ライセンス：MIT
- 状態：継続的な実装・ドキュメント更新を確認
- 適合度：★★★★★

今回、特に注目すべき実装です。

WebGPU Recorderは、WebGPU API呼び出しとGPUデータをキャプチャします。

出力には次の形式があります。

- 自己完結したHTML
- バイナリ `.wgpu`

ここで重要なのはバイナリ形式です。

### `.wgpu` の構造

READMEで、形式が公開されています。

```json
{
  "version": 1,
  "canvasWidth": 800,
  "canvasHeight": 600,
  "init": [],
  "frames": [],
  "data": []
}
```

概念としては、

```text
init
 ├─ device
 ├─ shaders
 ├─ pipelines
 ├─ textures
 ├─ buffers
 └─ bind groups

frames
 ├─ buffer updates
 ├─ render passes
 ├─ compute passes
 ├─ copies
 └─ submits

data
 └─ raw GPU data blobs
```

に分けられます。

これは、今回欲しいパッケージ構造の原材料に非常に近いです。

### 特に重要な機能

`recordMode: 2` は、任意のフレームについて、そのフレームを再生するのに必要なGPU状態を取得するモードです。

このモードでは、

- 生存中のGPUオブジェクト
- buffer内容
- texture内容
- 必要な初期状態

を記録します。

そして、不要なデータを除外します。

### 利点

キャプチャしたAPIストリームをJavaScriptソースではなく、構造化データとして取得できます。

自作コンバーター側で、

- shader module
- bind group layout
- resource
- pass
- copy
- writeBuffer

を分類しやすくなります。

### ただし制限もある

任意フレームキャプチャには、次の制限が記載されています。

- 一部のMSAA texture
- depth/stencil texture
- 読み戻し不能なtexture format
- 一部のmapped buffer
- external video texture

などは完全に復元できない場合があります。

また、付属PlayerはJavaScriptでWebGPU呼び出しを再実行します。

**`.wgpu` 自体がネイティブ用の静的エフェクト形式になっているわけではありません。**

それでも、Dawn capture/replayと比較して、機械可読な中間表現として扱いやすい点が大きな魅力です。

---

## 5-3. WebGPU Inspector

- URL：[GitHub — WebGPU Inspector](https://github.com/brendan-duncan/webgpu_inspector)
- ライセンス：今回確認した検索結果では未確定
- 状態：継続開発
- 適合度：★★★★☆

WebGPU Recorderの機能が統合されているWebGPUデバッグツールです。

対応する作業は、

- GPUコマンドの検査
- リソースの確認
- シェーダーの確認
- フレーム記録
- 再生用ファイル生成

などです。

今回の用途では、コンバーターのデバッグ環境として有用です。

ただし、uniformの意味推定や静的エフェクトmanifestの生成は提供されていません。

---

## 5-4. wgpu API Trace + Player

- URL：[wgpu公式 Debugging Applications](https://wgpu.rs/doc/wgpu/documentation/debugging/debugging_applications/index.html)
- ソース：[wgpu](https://github.com/gfx-rs/wgpu)
- ライセンス：MIT / Apache-2.0
- 状態：wgpu本体は活発。Trace機能の状態はバージョン依存
- 適合度：★★★☆☆

wgpu-coreにはAPIトレース機構があります。

```text
wgpu-core
    ↓
API trace
    ↓
trace files
    ↓
wgpu player
```

再生を目的とした仕組みです。

ただし、重要な注意点があります。

wgpuの一部バージョンでは、Trace機能の無効化や保守上の問題が見られます。

古いドキュメントと現在の実装が一致するとは限りません。

### 今回の評価

Dawnを既に使用しているなら、wgpu Traceを主軸にする必要性は低そうです。

ただし、シリアライズしたGPU呼び出しを別プロセスで再生する設計は参考になります。

こちらもパラメータ化は自動ではありません。

---

## 5-5. GFXReconstruct

- URL：[GitHub — GFXReconstruct](https://github.com/LunarG/gfxreconstruct)
- ライセンス：リポジトリのライセンスファイルで最終確認が必要
- 状態：継続開発
- 適合度：★★★☆☆

Vulkan / D3D12などのAPIをキャプチャ・再生するツールです。

主なツールは、

- `gfxrecon-replay`
- `gfxrecon-convert`
- `gfxrecon-optimize`

などです。

`gfxrecon-convert` は、キャプチャをJSON Lines形式のAPI呼び出し一覧へ変換できます。

参考：[LunarG — Mastering GFXReconstruct](https://www.lunarg.com/wp-content/uploads/2025/11/Mastering-GFXReconstruct-Parts-1-3b.pdf)

### 重要な訂正

`gfxrecon-convert` が直接、実行可能なC++エフェクトプログラムを生成するという証拠は確認できませんでした。

確認できたのは、API呼び出しのJSON Lines変換です。

GFXReconstruct内部にはAPIハンドラーを生成するコード生成機構もありますが、それはキャプチャから任意のエフェクト用C++ソースを生成する機能とは別です。

### 評価

APIトレースを別の中間表現に変換する先行例として重要です。

ただし、Dawn/WebGPUのアプリケーションを対象とするなら、WebGPUReconstructやWebGPU Recorderの方が直接的です。

---

## 5-6. RenderDoc

- URL：[GitHub — RenderDoc](https://github.com/baldurk/renderdoc)
- ライセンス：MIT
- 状態：継続開発
- 適合度：★★☆☆☆

GPUフレームキャプチャ・デバッグツールです。

取得できる情報には、

- GPUコマンド
- Pipeline State
- Render Targets
- Shader
- Buffer
- Texture
- Draw Call

などがあります。

また、シェーダーを編集して特定のGPUイベントを再実行する機能もあります。

ただし、これはデバッグ用途です。

キャプチャしたフレームを、ユーザー入力に応じて毎フレーム更新する汎用エフェクトとしてエクスポートする機能は確認できません。

---

# 6. Render Graph / Frame Graph

ここで注意したいのは、**Render Graphを持つことと、Render Graphをポータブルなファイルとして保存できることは別**という点です。

## 6-1. Google Filament FrameGraph

- URL：[Filament FrameGraph](https://google.github.io/filament/notes/framegraph.html)
- ライセンス：Apache-2.0
- 状態：Filamentの現行アーキテクチャ資料
- 適合度：★★★★★（実行側の参考）

Filament FrameGraphは、フレーム内で必要なGPUリソースと処理を管理する仕組みです。

RenderTargetの寿命、依存関係、中間リソースなどを扱います。

ただし、これはエンジン内部の仕組みです。

任意のJSONをロードしてRenderGraphを構築する汎用交換形式ではありません。

### 今回の重要点

すでにFilamentを使用している場合でも、動的に取り込んだエフェクトをFilamentのFrameGraphに登録できるかは、別途実装が必要です。

また、FilamentのFrameGraphを経由する方法と、Dawnに直接WebGPUパスを投入する方法は分けて検討する必要があります。

---

## 6-2. Unity SRP RenderGraph

- URL：[Unity RenderGraph Documentation](https://docs.unity3d.com/Packages/com.unity.render-pipelines.core@17.0/manual/render-graph-system.html)
- ライセンス：Unity本体は商用・独自ライセンス
- 状態：Unity 6で利用されている現行機構
- 適合度：★★★☆☆

UnityのRenderGraphは、

- リソース依存関係
- Render Pass
- Texture Handle
- Copy / Blit
- パスの最適化
- リソースの寿命管理

を扱います。

2026年の公式チュートリアルでも、BlitやカスタムRenderPassの構築が紹介されています。

[Unity公式 RenderGraphチュートリアル](https://www.youtube.com/watch?v=7L_8-H4W-bY)

ただし、任意のパスグラフをJSONで保存し、別エンジンで実行するための仕様ではありません。

参考になるのは、リソースの依存関係を宣言して実行順序を決める部分です。

---

## 6-3. Bevy Render Graph

- URL：[GitHub — Bevy](https://github.com/bevyengine/bevy)
- ライセンス：MIT / Apache-2.0
- 状態：継続開発
- 適合度：★★★☆☆

Rust製ゲームエンジンBevyのレンダリンググラフです。

Render Nodeと依存関係を組み立てます。

ただし、主にエンジン内のRustコードから構築するグラフです。

静的エフェクトファイルの交換形式としては設計されていません。

参考スレッド：

[Custom render graph from scratch — Reddit](https://www.reddit.com/r/bevy/comments/1fuore7)

---

## 6-4. Babylon.js Frame Graph

- URL：[Babylon.js Node Render Graph Editor](https://github.com/BabylonJS/Babylon.js/tree/master/packages/tools/nodeRenderGraphEditor)
- ライセンス：Apache-2.0
- 状態：活発
- 適合度：★★★★☆

このカテゴリでは、Babylon.jsのグラフシリアライズが比較的近いです。

理由は、実行グラフとエディタ上のグラフを結びつけ、保存・復元する仕組みがあるためです。

ただし、Babylon.js固有のノード型とランタイムに依存します。

今回の静的Dawnパッケージへそのまま変換する公式機能は確認できませんでした。

---

# 7. WGSL / GLSL / SPIR-V変換

ここは、かなり成熟しています。

ただし、シェーダー言語の変換ツールは、パスグラフを自動的には復元しません。

## 7-1. Naga

- URL：[Naga README](https://github.com/gfx-rs/wgpu/blob/trunk/naga/README.md)
- ライセンス：MIT / Apache-2.0
- 状態：活発
- 適合度：★★★★★

WGSL、SPIR-Vなどを扱うシェーダーコンパイラーです。

主要な変換先には、

- WGSL
- SPIR-V
- GLSL
- MSL
- HLSL

があります。

### 今回の用途

WGSLを直接Dawnで実行できるなら、必ずしも言語変換は必要ありません。

しかし、検証・リフレクション・別バックエンド用のコード生成には有用です。

注意点は、言語ごとの機能差によって変換が常に成功するとは限らないことです。

---

## 7-2. Tint

- URL：[GitHub — Dawn](https://github.com/google/dawn)
- ライセンス：BSD系
- 状態：活発
- 適合度：★★★★★

Dawnに含まれるWGSLコンパイラーです。

WGSLの検証や、GPUバックエンド向けのコード生成に関係します。

Dawnをすでに使っているなら、最も近いコンパイラー基盤です。

ただし、TintもAPI呼び出しからパスグラフを抽出するものではありません。

---

## 7-3. SPIRV-Cross

- URL：[GitHub — SPIRV-Cross](https://github.com/KhronosGroup/SPIRV-Cross)
- ライセンス：Apache-2.0を含む複数の許諾条件。ファイル単位の確認が必要
- 状態：継続開発
- 適合度：★★★★☆

SPIR-Vから次の形式へ変換できます。

- GLSL
- MSL
- HLSL

また、JSON Reflection出力があります。

これは重要です。

```text
SPIR-V
   ↓
Reflection JSON
   ↓
Bindings / Resources / Types
```

ただし、Reflectionで判定できるのはGPUリソースの構造です。

`float` が時間なのか強度なのかまでは判断できません。

---

## 7-4. Slang

- URL：[GitHub — Slang](https://github.com/shader-slang/slang)
- ドキュメント：[Slang FAQ](https://docs.shader-slang.org/en/latest/faq.html)
- ライセンス：Apache-2.0 with LLVM Exception
- 状態：活発
- 適合度：★★★★☆

Slangは複数のGPU言語をターゲットにするシェーダーコンパイラーです。

出力先には、

- SPIR-V
- HLSL
- GLSL
- WGSL
- MSL

などがあります。

今回の用途では、生成したシェーダーを標準化・検証する補助技術として使えます。

ただし、SlangはThree.js TSLを直接入力するものではなく、パスグラフの生成もしません。

---

## 7-5. Filament matc

- URL：[Filament Materials](https://google.github.io/filament/main/materials.html)
- ライセンス：Apache-2.0
- 状態：Filamentの公式ツール
- 適合度：★★★☆☆

`matc` はFilamentのMaterial Definitionをコンパイルします。

```text
Filament material definition
          ↓
matc
          ↓
Filament material package
```

ここで重要なのは、**WGSLを直接matcへ渡せば任意のポストプロセスになるわけではない**ことです。

FilamentのMaterialは、Filament固有のシェーディングモデル、バインディング、レンダリング契約に従います。

任意のWebGPU compute passやマルチパス効果を表現する汎用コンテナーではありません。

今回の用途では、生成したWGSLをDawn側で直接実行する方が、最初の実装としては自然に見えます。

---

# 8. その他の関連技術

## 8-1. cables.gl

- URL：[cables.gl Standalone](https://cables.gl/standalone)
- エクスポート：[Standalone executable export](https://cables.gl/docs/4_export_embed/dev_embed/export_exe/export_exe)
- CLI：[GitHub — cables-cli](https://github.com/cables-gl/cables-cli)
- ライセンス：Standaloneの公開コードはMIT
- 状態：2026年9月公開のStandalone v0.11.3を確認
- 適合度：★★★☆☆

ビジュアルノードでインタラクティブな作品を作成する環境です。

エクスポートにも対応しています。

ただし、確認できたStandalone executable exportはElectronベースです。

つまり、

```text
cables project
      ↓
Standalone application
      ↓
JavaScript / Web runtime
```

という構造です。

**JSなしのネイティブGPUパッケージを生成するものではありません。**

それでも、グラフとリソースをまとめて配布する設計として参考になります。

---

## 8-2. GPUReplay（研究論文）

- 論文：[GPUReplay: A 50-KB GPU Stack for Client ML](https://arxiv.org/abs/2105.05085)
- コード：[GitHub — GPUReplay](https://github.com/bakhi/GPUReplay)
- ライセンス：今回未確認
- 状態：研究プロジェクト。最近の保守状況は未確認
- 適合度：★★★☆☆

この研究は、GPU実行を事前に記録し、後から新しい入力で再実行するという考え方を扱います。

今回の要求との共通点は明確です。

```text
Capture once
     ↓
Replay on new input
```

ただし、対象は主に機械学習のGPUワークロードです。

一般的なWebGPUのRenderTarget、マルチパス、履歴フレーム、入力動画を扱う汎用エフェクト変換器ではありません。

概念的な先行研究として有用です。

---

# 9. 日本語・中国語・韓国語・コミュニティで見つかった関連情報

今回、非英語圏で直接「TSLマルチパスを静的Dawnパッケージに変換する」実装は確認できませんでした。

ただし、関連する技術資料は見つかりました。

| 言語・場所 | 資料 | 内容 | 今回の有用性 |
|---|---|---|---|
| 日本語・Zenn | [Three.js WebGPUとTSL](https://zenn.dev/wtshm/articles/2d55e77f48de55) | TSL→WGSL/GLSL、NodeMaterialの経緯 | 高 |
| 日本語・Zenn | [GLSL/WGSLに変換できるシェーダー言語](https://zenn.dev/jp/articles/9c62b0de08312b) | TS風言語からGLSL/WGSL生成 | 中 |
| 日本語・Zenn | [Three.js Post Processing](https://zenn.dev/t12u/articles/85bb8b1e598cd4) | 複数パスとRenderTargetのping-pong | 中 |
| 日本語・Zenn | [Three.js r184](https://zenn.dev/112ka/articles/threejs-catchup-r184) | TSL RenderPipeline、MRT、時間的再投影 | 高 |
| 中国語・掘金 | [Three.js / WebGPU関連](https://juejin.cn/post/7497077230864400411) | WebGPUとThree.jsの技術動向 | 低〜中 |
| 中国語・CSDN | [Unity URP Multi-Pass](https://blog.csdn.net/terric/article/details/136654523) | Unityでの複数Pass構成と制限 | 中 |
| 英語・three.js Discourse | [TSL and WGSL functions](https://discourse.threejs.org/t/tsl-shaders-improving-ergonomics-using-wgsl-tag-template-literal-functions/90010) | TSLとWGSLを混在させる設計 | 中 |
| 英語・Reddit | [WebGPUReconstruct 2.0](https://www.reddit.com/r/webgpu/comments/1rub7qp/webgpureconstruct_20/) | WebGPU→ネイティブ再生 | 非常に高 |
| 英語・Reddit | [GFXReconstruct discussion](https://www.reddit.com/r/vulkan/comments/18yrnu9) | GPUキャプチャの再現と分析 | 中 |
| 英語・Reddit | [TSL node editor postprocessing](https://www.reddit.com/r/threejs/comments/1r8ygm1/my_tsl_node_editor_supports_post_processing_now/) | TSLノード編集とポストプロセス | 中 |

韓国語の技術記事・GitHub検索では、今回の変換問題に直接対応する実装を十分な根拠とともに確認できませんでした。

また、知乎・Bilibili・Hacker News・note・Qiitaについても検索対象に含めましたが、今回の条件を満たす具体的な実装を確認できたわけではありません。

これらについて「該当技術は存在しない」と結論づけることはできません。

---

# 10. 最重要：キャプチャを「ループ可能なプログラム」に変える問題

ここまでの既存実装を比較すると、最も重要な違いが見えてきます。

### A. GPU Capture / Replay

```text
Frame 100
 ├─ writeBuffer(time = 1.666)
 ├─ renderPass(A)
 ├─ renderPass(B)
 ├─ copyTexture
 └─ submit
```

この記録をそのまま再生すれば、基本的には同じ時点の状態を再現します。

### B. 今回欲しいもの

```text
Every frame
 ├─ UpdateUniform(time)
 ├─ UpdateUniform(knobs)
 ├─ BindInputVideoFrame
 ├─ RenderPass(A)
 ├─ RenderPass(B)
 ├─ SwapHistoryTextures
 └─ Output
```

後者では、APIストリームの一部分が可変になっています。

さらに、前フレームのGPU結果を保持する必要があります。

ここが最大の差です。

## 問題1：uniformの意味

例えばキャプチャから、次の操作が見つかったとします。

```text
writeBuffer(
    buffer = 42,
    offset = 16,
    data = [1.666]
)
```

これだけでは、そのfloatが時間なのか、何らかの強度なのかは分かりません。

ただし、今回の構成ではTSLのソースやNodeBuilderにアクセスできます。

そのため、GPU実行前に

```text
TSL uniform node
     ↓
Generated WGSL binding
     ↓
GPUBuffer + byte offset
```

という対応表を保存できる可能性があります。

これは、純粋なブラックボックスキャプチャと比較した大きな利点です。

## 問題2：GPUリソースの寿命

一つのフレームだけを見ると、GPUリソースの用途が分からないことがあります。

例えば、あるテクスチャが

- 一時的なblur target
- 前フレームの履歴
- 外部入力
- mipmap生成先

のどれかは、複数フレームの観察が必要な場合があります。

したがって、1フレームだけのキャプチャからすべての意味を復元するのは難しいです。

## 問題3：ping-pong

次のようなエフェクトを考えます。

```text
Frame N
  History A → Render → History B

Frame N+1
  History B → Render → History A
```

これを単なるAPI呼び出し列として保存すると、フレーム間の役割交換を適切にモデル化できない可能性があります。

必要なのは、

```json
{
  "history": {
    "buffers": ["historyA", "historyB"],
    "mode": "pingpong"
  }
}
```

のような意味を持つ記述です。

ISFは、この問題に対して `PERSISTENT` という仕組みを提供しています。

---

# 11. 既存実装の総合比較

記号：

- ◎：直接対応
- ○：かなり対応
- △：一部対応、または追加実装が必要
- ×：対応しない
- ?：今回確認できず

| 技術 | Shader抽出 | Multi-pass | Bindings | Time/Knobs | History | Native replay | 静的パッケージ |
|---|---|---|---|---|---|---|---|
| Three.js WGSLNodeBuilder | ◎ | × | ◎ | ○ | × | × | △ |
| Three.js RenderPipeline | ○ | ◎ | ○ | ○ | ○ | × | × |
| tsl-node-editor | △ | ? | △ | △ | ? | × | × |
| Shader Lab | △ | △ | △ | ○ | ? | × | × |
| pmndrs postprocessing | ○ | ◎ | ○ | ○ | △ | × | × |
| Babylon Node Material | ○ | × | ○ | ○ | × | × | △ |
| Babylon Node Render Graph | △ | ◎ | △ | ○ | △ | × | ○（Babylon形式） |
| ISF | ◎ | ◎ | ○ | ◎ | ◎ | ◎（対応ホスト） | ◎ |
| ISFMSLKit | ○ | ◎ | ○ | ◎ | ◎ | ◎ | ○ |
| ShaderToyX | ◎ | ◎ | ○ | ◎ | ◎ | ◎ | △ |
| SHADERed | ◎ | ○ | ○ | ◎ | △ | ◎ | △ |
| ShaderAmp | ◎ | ◎ | ○ | ◎ | ○ | × | △ |
| Natron Shadertoy | ◎ | △ | ○ | ○ | △ | ◎ | △ |
| TouchDesigner | ◎ | ◎ | ◎ | ◎ | ◎ | ◎ | △ |
| WebGPUReconstruct | ◎ | ◎ | ◎ | × | ○（記録） | ◎ | △ |
| WebGPU Recorder | ◎ | ◎ | ◎ | × | ○（記録） | △ | ○（キャプチャ形式） |
| WebGPU Inspector | ◎ | ◎ | ◎ | × | △ | △ | △ |
| wgpu Trace | ◎ | ◎ | ◎ | × | △ | ◎ | △ |
| GFXReconstruct | ◎ | ◎ | ◎ | × | △ | ◎ | △ |
| RenderDoc | ◎ | ◎ | ◎ | × | △ | ○（デバッグ） | × |
| Filament FrameGraph | × | ◎ | ○ | △ | ○ | ◎ | × |
| Unity RenderGraph | × | ◎ | ○ | △ | ○ | ◎ | × |
| Bevy Render Graph | × | ◎ | ○ | △ | ○ | ◎ | × |
| Naga | ◎ | × | ○ | × | × | × | △ |
| Tint | ◎ | × | ○ | × | × | × | △ |
| SPIRV-Cross | ◎ | × | ◎ | × | × | × | △ |
| Slang | ◎ | × | ○ | × | × | × | △ |
| Filament matc | △ | × | ○ | △ | × | ◎ | ○（Filament形式） |

注意：この表の「○（記録）」は、履歴テクスチャがAPIコマンドやリソースとしてキャプチャされることを意味します。履歴バッファの意味論が自動抽出されるという意味ではありません。

---

# 12. 今回の構成で最も有望な設計

調査結果を踏まえると、次の構成が有望だと考えます。

これは既存プロジェクトがそのまま提供する機能ではなく、複数の実装を組み合わせる設計上の提案です。

```text
                  IMPORT TIME
               JavaScript allowed

       Three.js / TSL / Shader Lab
                    │
           Headless WebGPU
                    │
          ┌─────────┴──────────┐
          │                    │
     NodeBuilder         WebGPU Capture
          │                    │
     WGSL source          API calls
     Uniform map          Render passes
     Bindings             Resources
          │                    │
          └─────────┬──────────┘
                    │
             Static Converter
                    │
        ┌───────────┼────────────┐
        │           │            │
     Shader      Bindings     Pass Graph
        │           │            │
        └───────────┼────────────┘
                    │
             Effect Package
                    │
                    ▼
               PLAYBACK TIME
                 No JS / DOM

               Native C++
                    │
              Effect Loader
                    │
         Dawn WebGPU Device
                    │
          ┌─────────┴─────────┐
          │                   │
      Uniform Update      Pass Executor
          │                   │
      time / knobs       RT / ping-pong
          │                   │
          └─────────┬─────────┘
                    │
                GPU Output
                    │
          Filament / ThorVG
```

### 静的パッケージの例

```json
{
  "version": 1,

  "shaders": [
    {
      "id": "blur",
      "file": "blur.wgsl"
    },
    {
      "id": "composite",
      "file": "composite.wgsl"
    }
  ],

  "parameters": [
    {
      "name": "time",
      "type": "float",
      "source": "timeline.time"
    },
    {
      "name": "intensity",
      "type": "float",
      "default": 0.5,
      "range": [0.0, 1.0]
    }
  ],

  "resources": [
    {
      "id": "blurTarget",
      "format": "rgba16float",
      "size": "output"
    },
    {
      "id": "history",
      "format": "rgba16float",
      "size": "output",
      "persistent": true,
      "buffering": 2
    }
  ],

  "passes": [
    {
      "id": "blur",
      "shader": "blur",
      "type": "render",
      "inputs": ["source"],
      "output": "blurTarget"
    },
    {
      "id": "composite",
      "shader": "composite",
      "type": "render",
      "inputs": ["blurTarget", "history.previous"],
      "output": "history.current"
    }
  ],

  "output": "history.current"
}
```

この例は説明用であり、既存規格ではありません。

また、実際の形式では、shader entry point、bind group layout、buffer offset、sampler、texture view、load/store operation、dispatch/draw条件などを追加する必要があります。

### 動画編集ソフト固有の注意点

ここには、通常のWebGPUデモやISFホストと異なる重要な問題があります。

それは、動画編集ソフトでは時刻が必ずしも前向きに進まないことです。

例えば、

```text
Frame 100
Frame 101
Frame 102
Frame 50
Frame 51
```

という順序で評価される可能性があります。

フィードバックエフェクトは、通常、

```text
Frame N = f(Frame N-1)
```

という状態依存を持ちます。

そのため、「毎フレームuniformだけ更新する」というモデルだけでは、任意時刻へのシークが成立しないケースがあります。

静的パッケージには、少なくとも次の区別が必要です。

- stateless：任意時刻を独立評価できる
- stateful：前フレームに依存する
- resettable：初期状態へ戻せる
- pre-roll required：対象フレームまで前進計算が必要

これは、時間的な再現性を保証するうえで重要な設計上の追加要件です。

---

# 13. 最も近い3つ

## 第1位：WebGPUReconstruct

[GitHub](https://github.com/Chainsawkitten/WebGPUReconstruct)

### 選定理由

現在の技術構成に最も近いからです。

- WebGPUを対象とする
- ブラウザからキャプチャできる
- Native Dawnで再生できる
- 複数パスをAPIレベルで取得できる
- macOS / Windows向けネイティブ再生の方向性が一致する

最大の不足点は、記録をパラメータ化された再利用可能なプログラムに変換しないことです。

それでも、Webエフェクトをネイティブ環境へ持ち込むという最も難しい境界をすでに越えています。

**今回の技術検証で最初に詳しく読むべきリポジトリです。**

---

## 第2位：WebGPU Recorder

[GitHub](https://github.com/brendan-duncan/webgpu_recorder)

### 選定理由

出力される `.wgpu` 形式が、静的パッケージ生成の中間表現として扱いやすいからです。

特に、

```text
init
frames
data
```

という分離が重要です。

この記録から、

- pipeline定義
- shader source
- resource定義
- bind group
- render pass
- copy
- uniform upload

を抽出するコンバーターを作る道筋が比較的明確です。

また、任意フレームの初期状態を取得する機能は、長時間動作するエフェクトの取り込みに有用です。

ただし、ネイティブDawnプレイヤーは別途必要です。

**静的パッケージを自動生成するコンバーターの原材料としては、今回の調査で最も興味深い実装です。**

---

## 第3位：ISF + ISFMSLKit

[ISF Specification](https://github.com/mrRay/ISF_Spec)  
[ISFMSLKit](https://github.com/mrRay/ISFMSLKit)

### 選定理由

最終的に実現したい「プロダクトの形」に最も近いからです。

ISFはすでに、

- 静的shader
- ユーザー調整パラメータ
- 入力画像
- 複数パス
- persistent buffer
- 時間uniform
- ネイティブホストによる再生

という問題を解いています。

不足しているのは、

- WGSL中心の設計
- WebGPU computeへの対応
- より一般的なbinding layout
- Three.js TSLからの自動変換

です。

ISFMSLKitには、静的エフェクトファイルをネイティブGPU上で実行する実例があります。

**最終パッケージの仕様とホスト実装を設計する際の、最も有力な参考資料です。**

---

# 最終結論

調査で見つかった技術は、大きく3種類に分かれます。

| 分野 | 最も有望な既存技術 | 解決している問題 |
|---|---|---|
| WebGPUの実行抽出 | WebGPUReconstruct / WebGPU Recorder | GPU処理を捕まえる |
| 静的エフェクトの記述 | ISF | shader・入力・pass・履歴をデータで表す |
| ネイティブGPUでの実行 | ISFMSLKit / ShaderToyX | JSなしで毎フレーム実行する |

そして、今回必要なのは、この3種類の技術を接続する部分です。

具体的には、

**「WebGPU APIキャプチャ + TSLのuniform意味情報」から「ISFに似た静的WGSLパスグラフ」を生成するコンバーター**

です。

ここは既存技術を組み合わせて作れる可能性が十分にあります。

一方で、任意のWebエフェクトを100%自動で変換できると判断する根拠はありません。特に、JavaScriptが毎フレームGPUリソース構造を変える効果、CPU側でシミュレーションする効果、外部動画・音声・DOMに依存する効果では、追加の解析や制約が必要になります。

それでも、対象をまず「TSLで記述された画像処理とポストプロセス」に限定するなら、実現可能性はかなり高いと評価します。

最初に検証すべき核心は、次の一点です。

> **Three.jsの同一エフェクトを異なる時刻・パラメータで数フレームキャプチャし、GPUパイプラインとパス構造を固定したまま、uniform更新だけを差し替えてNative Dawn上で継続再生できるか。**

これが成立すれば、単純な単一パスだけでなく、複数パスのWebエフェクトを静的パッケージへ変換するための、最も重要な技術的根拠になります。