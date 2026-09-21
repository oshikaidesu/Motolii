# Stage 5 FrameGraph — 実行モデルの正本

状態: **設計正本**。2026-09-21 利用者裁定。

施工wave、検証、並列laneは[実装・検証・並列施工シミュレーション](frame-graph-execution-simulation.md)を使う。

この文書は「作り直すこと」を目的にしない。目的は、Motolii を普通にリアルタイム再生できる編集機にすること。そのために、現在の Flutter UI を維持したまま、分散した一拍の責任を一つの評価グラフへ収束する。

## 二行の製品定義

- **Motolii は Rerun を AE にする。**
- **Flutter は完成した絵を再生し、編集命令を渡す。**

## 変えない物

- 現在の Flutter UI の外観、パネル、操作、Inspector、Timeline
- `Intent → Document → Undo` の編集契約
- Rerun Store の作品・時刻データ
- re_renderer と現在の GPU 表現
- IOSurface の直接出力、再生中の GPU wait/readback 0
- 静止画、再生、scrub、export が同じ作品意味を読むこと

現在の実装は、新しい実行モデルの構造として残す物ではない。画素、操作、Undo、seek、保存・再読込、export の oracle として使う。

## Layer と Node

Layer は利用者が編集する authoring language である。重ね順、親子、matte、effect stack などの作品意味を持つ。実行時の仕事の単位にはしない。

Document revision が変わった時、Layer の作品意味を内部の FrameGraph へ compile する。再生中は既にある Graph を時刻で評価する。

```text
Flutter Layer UI
    → Intent
    → Document / Rerun Store
    → compile(revision)
    → FrameGraph
    → evaluate(time, preview quality)
    → re_renderer
    → IOSurface
    → Flutter Texture
```

一つの Layer は複数の Node へ分かれる。

```text
Text Layer
  TextContent → TextShape ─┐
  TextStyle ───────────────┤
  Transform ───────────────┤→ Effects → Composite

Cube Layer
  MediaExtent → MeshSource → Material → Transform → Composite
```

Layer を複製した時、同じ `TextShape`、`MeshSource`、`Material` は同じ Node を参照する。Position だけが異なる複製は Transform 以降だけを分ける。

## どこまでを Node にするか

Node は「Layer を細かく分解した物」ではなく、**独立して共有・cache・無効化・時刻参照したい計算の境界**である。

Node にする判断基準:

- 完全入力が同じなら、別 Layer / 別 view からでも結果を共有したい。
- 入力の一部だけが変わった時、その計算と下流だけを無効化したい。
- 時刻 `t` や別時刻 `t'` を明示した依存として持ちたい。
- renderer の前に semantic value のまま保持する意味がある。

逆に、UI 上の分類、単なる struct の分割、GPU pass 一個ごとの都合は Node にしない。評価 Node 数と GPU pass 数は一致しなくてよい。連続する image effect は論理 Node を保ったまま、lowering 後に shader/pass fusion してよい。

```text
AUTHORING
Layer / Group / Follow / LookAt / Effect stack / Mask / Matte
        ↓ compile

EVALUATION DAG
Property / MediaFrame / TextShape / Geometry / Layout / Transform
Particle / Effect / Mask / Matte / GroupComposite / SceneComposite / History
        ↓ lower / optimize

GPU EXECUTION
texture / buffer / mesh / point cloud / fused shader / render pass / re_renderer scene
```

利用者が触るのは authoring language。FrameGraph は内部 IR。GPU graph は実行計画であり、三者を同じ node vocabulary にしない。

### Group は scope、必要な時だけ plate

Group は常に texture を作る Layer ではない。既定では親子・Layout・Transform・scope を与えるだけで、子はそのまま Scene へ寄与する。

```text
Group
 ├─ Child A ───────────────┐
 ├─ Child B ───────────────┼→ Scene
 └─ Child C ───────────────┘
        ↑
      Layout
```

Group 全体に `Whole` effect、matte 後処理、flatten など「子の合成結果を一枚として扱う意味」が付いた時だけ compiler が plate 境界を作る。

```text
Child A ─┐
Child B ─┼→ GroupComposite → Whole effects → Group Contribution
Child C ─┘
```

AE の「まとめて効果を掛けたいから precomp を作る」という回避策を Document 構造へ要求しない。Group という既存の意味から compiler が必要な中間合成を作る。

### 最初に Node 化する境界

- Property / Track / Link
- local Transform / inherited WorldTransform
- Layout / Flow
- Text shaping / Shape geometry
- Media source / exact-time MediaFrame
- Effect（論理上は一 Effect 一 Node、GPU では融合可）
- Mask / Matte
- Particle（pixel ではなく positions / sizes / colors / links の semantic value）
- 必要な時だけ GroupComposite / SceneComposite

Layer 自体を一 Node にしない。Layer はこれらの binding と ordered contribution を記述する authoring unit である。

### Feedback は時間 edge、Freeze は cache policy

Feedback を self-cycle の Node にしない。必要な過去値を host-owned history から明示的な時間 edge として読む。

```text
Source(t) ───────────────┐
Source/History(t - 1) ───┼→ Feedback → Result(t)
```

`InputTime::Offset` と同じ法で「何フレーム前を読むか」を Graph に宣言する。plugin 自身が隠れた履歴を持つ旧式には戻さない。

Freeze は原則として作品意味の `FreezeNode` にしない。任意 subgraph の完成結果を disk / RAM / VRAM に保持する評価戦略であり、cache を消しても同じ作品意味が得られることを保つ。

## 旧実装を戻してよい境界

旧実装は **FrameGraph の下流 adapter / GPU resource implementation としては再利用してよい**。戻してよい物:

- text / shape の既存 rasterize・tessellate・cache
- still / video decode、GPU frame cache、mesh / point-cloud import
- `LayerWithPasses`、effect pass、matte、flatten、feedback の実装部品
- re_renderer / compositor の最終描画

戻してはいけない物:

- `StoreView + time` から始まって全 Layer を `Vec<ResolvedLayer>` に作り直す playback owner
- `layers_from_resolved()` を FrameGraph の恒久 fallback として丸ごと呼ぶこと
- Camera / Stage / export ごとに同じ作品意味を再評価すること
- Node 入力に無い Document 読みを adapter の中へ隠すこと

移植は「旧 owner を復活」ではなく、旧 owner に埋まっていた意味と実装部品を Node / edge / lowerer へ一つずつほどく。

## Node の同一性

Node の名前は結果へ影響する完全な入力から作る。

```text
NodeKey = hash(
  node kind,
  input NodeKey list,
  evaluated parameters,
  exact timebase,
  source/resource version,
  preview/export quality
)
```

- 同じ `NodeKey` は一度だけ実行する。
- Layer ID は結果へ影響する時だけ含める。単なる複製の共有を妨げない。
- revision や時刻だけの粗い memo を正本にしない。
- Node の入力に無い変更は、その Node と下流を無効化しない。
- hash collision は同一性として黙って受け入れない。kind と完全な入力記述で検証できる形にする。

## Graph の寿命

### Document を編集した時

1. UI が Intent を送る。
2. Document が一つの取引として Rerun Store を更新する。
3. revision が進む。
4. compiler が変わった作品意味だけを Node へ写す。
5. 同じ NodeKey の既存結果は残す。
6. 変更 Node とその下流だけを未評価にする。

### 時刻が進んだ時

1. native clock が正確な comp time を一度決める。
2. 現在の出力から到達可能な Node だけを辿る。
3. time-dependent Node の key を更新する。
4. 未評価 Node だけを実行する。
5. 古い generation の未完了仕事を cancel する。
6. 完成した最新 frame だけを publish する。

Graph topology は毎フレーム作らない。静的 Node は毎フレーム Store を読まない。

## 一拍の法則

```text
1 document revision + 1 comp time
    = 1 reachable evaluation graph
    + 必要な view 数だけの最終投影
```

Camera、Stage、reflection、export preview は共通の upstream Node を読む。

```text
Shared World
  ├─ Camera projection → Camera surface
  ├─ User projection   → Stage surface
  ├─ Probe projection  → Reflection
  └─ Export projection → Export target
```

Camera は最後に覗く。Camera を増やしても Text、Layout、world transform、media extent、effect の意味評価を増やさない。

## 責任

| 責任 | 唯一の owner | 出力 |
|---|---|---|
| 編集取引・Undo | Document | Rerun Store revision |
| Graph topology の compile | motolii-render FrameGraph compiler | immutable Graph |
| exact comp time | native playback clock | RationalTime |
| Node 同一性・依存・無効化 | FrameGraph | NodeKey / dirty set |
| Node 実行順・cancel | FrameGraph scheduler | completed Node values |
| GPU resource | re_renderer / compositor | GPU handles |
| view camera・ROI | view projection | render target commands |
| presentable surface | native host | IOSurface |
| UI・操作・表示 | Flutter | Intent / Texture presentation |

責任を一つの巨大 struct や file に集める意味ではない。一拍の進行を決める owner を一人にし、各 Node は明示した入力と出力だけを持つ。

## Flutter の契約

Flutter は作品の第二正本を持たない。

- Intent、play、pause、seek を送る。
- 完成した external Texture を表示する。
- 再生開始時に `(start frame, host time, fps)` を受け、playhead は表示側で進める。
- pause、seek、loop、drop、revision change でだけ補正する。
- 毎フレーム Document snapshot、JSON、render request を送受信しない。
- picture が遅れても button、drag、menu、text input を止めない。

UI の外観と操作を変えない。内部の再生通知を変える時も、playhead、Frame 表示、stop 後の正確な位置を同じに保つ。

## Preview と Export

- Preview は期限を持つ。間に合わない仕事は cancel できる。
- 操作中だけ preview quality を落とせる。
- adaptive quality は NodeKey に含め、full-quality result と混同しない。
- Export は全frame、完全品質、決定的な結果を要求する。
- Preview と Export は同じGraph意味を読み、scheduler policy と出力品質だけを変える。

## Cache

Cache はGraphの外から結果を推測する仕組みにしない。NodeKeyと完成結果を対応させる。

1. re_renderer/GPU resource・VRAM
2. RAM
3. 必要な高価値成果だけdisk

最初から三段すべてを実装しない。Node同一性と一拍の共有を先に成立させる。cacheを切った状態でも同じ画素が出ることをoracleにする。

## 現行経路から退役させる責任

- view ごとに `StoreView + time` から始める `render_frame_into_window`
- 全 Layer を毎回 `Vec<ResolvedLayer>` へ作り直す playback path
- Camera / Stage ごとの Text・Shape・world transform・AnalysisInputs 再評価
- `AnalysisInputs` が空の時だけ成立する `resolved_memo`
- status / Inspector のための同一frame再評価
- Flutter との毎フレーム snapshot / JSON / render command
- 変更されていない Layer の全走査、文字列 ID 再構築、資源再作成

旧経路を恒久 fallback として残さない。比較中はoracleとして別の試験入口から呼び、cutoverと同じ変更で製品の旧入口を削除する。

## 移行順

### F0 — Graph 契約

既存の責任地図を次の表へ写す。

| Node kind | 入力 | 出力 | 時刻依存 | 無効化条件 | 現owner | 既知実装 |
|---|---|---|---|---|---|---|

Node の意味が決まるまで実装を分散しない。

### F1 — Compiler と executor の最小縦切り

`stagger_reflow.js` を Layer → Graph → 同じ画素まで通す。Text、Shape、Group、Layout、Camera、Cubeだけを含める。UIは変更しない。

### F2 — Playback cutover

native clockが一度Graphを評価し、CameraとStageを同じ結果から描く。Cube追加後に旧経路を使わない。製品playbackの旧入口を削除する。

### F3 — 編集と時間

seek、scrub、keyframe、parent、Undo/Redo、preview editで必要なNodeだけを無効化する。古いgenerationをcancelする。

### F4 — Effect・media・analysis

Mask、effect stack、video decode、audio、Blob/Overlay、reflectionをNodeへ移す。一機能ずつ旧ownerを削除する。

### F5 — 一経路化

still、playback、scrub、Stage、Camera、exportを同じGraphへ揃え、旧resolve hot pathと移行adapterを削除する。

## 合格条件

### 構造

- 同じNodeKeyを一frameで二度実行しない。
- Camera + Stageで upstream scene evaluation は1回。
- view側からRerun StoreのLayer評価を開始できない。
- Graph topology compileはrevision単位、frame単位ではない。
- Layer追加数ではなく、変更されたNodeとGPU instanceに費用が対応する。

### 性能

- `stagger_reflow.js` + Cube、30fps: warm p90 < 33.333ms、期限内drop 0。
- 同じCube 100個: MeshSource 1、Material 1、Transform/instance 100。
- Camera + Stage: scene evaluation 1、projection/draw 2。
- playback: GPU wait 0、readback 0、CPU pixel copy 0。
- static frame:意味評価0。Textureの再提示だけで済む。

### 製品

- 現在のFlutter UIの比較画像でlayout、色、panel、controlが同じ。
- playhead、seek、pause、loop、Frame表示が一致する。
- edit → visible frame → Undo → save/reopen → export が同じ結果になる。
- Previewで品質を落としてもExportの画素は変わらない。

## 実装規律

- 一つのNode ownerを足す時、同じ責任の旧ownerをcutoverで削除する。
- cacheやadapterで古い呼び出し構造を温存しない。
- oracleのために旧実装をproductionへ複製しない。
- 新しいGraphの進捗を行数やcompileで報告しない。削除した旧経路、Node実行数、画素、実窓、frame budgetで報告する。
- UIを性能実験の変数にしない。

## 先例

LumitはRust backend + Flutter frontendで、Compositionを一frameのevaluation DAGへcompileし、同じcontent hashの仕事を一度だけ実行する。古い仕事のcancel、GPU-first、UIとpictureの分離、previewのadaptive degradation、VRAM/RAM/disk cacheを採る。

- https://docs.lumitlab.com/engine/overview/
- https://docs.lumitlab.com/engine/render-pipeline/
- https://docs.lumitlab.com/engine/performance/
- https://docs.lumitlab.com/engine/cache/

LumitはGPLv3のためコードを持ち込まない。DAG、content identity、cancellation、GPU-first、UI境界という公開された設計原理をMotoliiのRerun/re_rendererへ写す。
