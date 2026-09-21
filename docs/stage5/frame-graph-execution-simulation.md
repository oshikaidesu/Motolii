# FrameGraph 実装・検証・並列施工シミュレーション

状態: **施工シミュレーション**。実装の正本は [FrameGraph実行モデル](frame-graph.md)。

目的は、テストと並列作業そのものを増やさず、既存部品をFrameGraphの一本の経路へ最短で結線すること。

## 結論

- 最初のPlayback cutoverはrenderer全体の作り直しではない。中央配線の差し替えである。
- 有効な並列数は、統合owner 1人 + 実装lane 3人。
- 直列spineとcutoverは一人が所有する。
- 並列laneは新規`frame_graph/`配下の別fileだけを触る。
- lane中の検証は`cargo check`。画素・GPU・実窓はcutover時に一度だけ確認する。
- 旧実装はoracleとして読むが、production fallbackや複製testとして残さない。

## 規模の見立て

現在すでにある物:

- revision単位の構造cacheとdynamic property分類: `store/scratch.rs`
- 時刻ごとのlayoutと静的Taffy plan: `picture/frame.rs`、`picture/flow.rs`
- text、shape、media、mesh、material、GPU resource cache
- native clock、非同期GPU完了、double-buffered IOSurface
- CameraとStageのtargetを一拍で列挙するhost loop
- re_renderer/compositorの最終投影

最初の工事で作るのは、これらを一度だけ呼ぶownerと、入力・出力の型である。

概算:

| 範囲 | 既存file変更 | 新規file | 追加 | 削除・置換 |
|---|---:|---:|---:|---:|
| F1/F2 Camera・Stage共有まで | 4〜6 | 3〜5 | 300〜600行 | 100〜250行 |
| Text・Shape・Cubeのcontent共有まで | 合計6〜8 | 合計5〜8 | 700〜1,200行 | 300〜600行 |

これは予算ではなく施工量の目安。完了条件は行数ではなく、旧ownerの削除、Node実行数、画素、frame budget。

## 残す物と外す入口

### 残す

| 既存部品 | FrameGraphでの役割 |
|---|---|
| Document / StoreView / Undo | authoring source |
| `LayoutCache::Structure` | compiler入力、静的topologyとdynamic分類 |
| `layout_frame` / `FlowCache` | Layout Node evaluator |
| text / shape / media / mesh cache | content Nodeの既存実装 |
| `LayerWithPasses` / effect / matte / feedback | GPU側Nodeの出力先 |
| `Compositor::render_into_window` | viewごとの最終投影 |
| native clock / `Frames` | generationとGPU完了 |
| Swift target列挙 / IOSurface publish | view targetとpresent |
| Flutter UI | authoring UIとTexture表示 |

### F2で外す

- playbackのviewごとに`StoreView + time`から始まる`render_frame_into_window`
- playback中のCamera/Stage別`resolved_layers`
- viewごとのText・Shape・document camera・AnalysisInputs再評価
- `AnalysisInputs`が空の時だけ共有する`resolved_memo`
- snapshot/statusの同一frame再resolve fallback

feedback、physics、Blob/Overlayはview/window状態も読む。F1では上流sceneだけを共有し、これらを無理に完成GPU layerとして共有しない。

## 直列契約 S0

最初に統合owner一人が型だけを固定する。

```rust
compile(&StoreView) -> CompiledGraph
evaluate(&mut CompiledGraph, time, quality, generation) -> EvaluatedFrame
render_view(&EvaluatedFrame, projection, target) -> Submission
```

絶対条件:

- `render_view`は`StoreView`を受け取れない。
- Nodeは明示した入力NodeKey以外を読めない。
- NodeKeyと依存edgeはcompile時に決まる。
- generationを越えた結果はpublishできない。
- UI、Swift ABI、Document/IntentはS0で変更しない。

S0の所有file:

```text
motolii-render/src/frame_graph/mod.rs
motolii-render/src/frame_graph/key.rs
motolii-render/src/frame_graph/topology.rs
motolii-render/src/frame_graph/value.rs
motolii-render/src/lib.rs        # module宣言だけ
```

## Wave P1 — 3レーン並列

S0の型を変更せず、別worktree・別fileで進める。

| lane | 所有file | 成果 | 触らないfile |
|---|---|---|---|
| Compiler | `frame_graph/compiler.rs`、`key.rs`、`topology.rs` | Layer意味→Node、content identity、共有edge | `engine.rs`、`render.rs` |
| Evaluator | `frame_graph/evaluate.rs`、`scheduler.rs`、`cache.rs` | dirty伝播、generation、cancel、実行counter | `engine.rs`、`render.rs` |
| Node adapters | `frame_graph/nodes/*.rs` | 既存layout/text/shape/media関数をNode evaluatorとして包む | 既存evaluator本体 |

このwaveで既存hot pathを変更しない。旧経路と新Graphをproductionで二重実行しない。

## 直列cutover S1

統合ownerだけが中央を触る。

1. `Engine`が`CompiledGraph`を一つ所有する。
2. revision変更時だけcompileする。
3. native tickが`evaluate`を一度呼ぶ。
4. CameraとStageが同じ`EvaluatedFrame`を`render_view`へ渡す。
5. snapshot/Inspectorもそのframe結果を読む。
6. `resolved_memo`とplaybackの旧入口を削除する。

衝突file:

```text
motolii-render/src/engine.rs
motolii-render/src/engine/render.rs
motolii-render/src/engine/analysis.rs
motolii-render/src/picture/resolve.rs
ui/native/src/lib.rs
ui/native/src/port.rs
ui/macos/Runner/MainFlutterWindow.swift
```

これらは並列laneへ配らない。

## Wave P2 — 読み手と非playback経路

| lane | 所有file | 成果 |
|---|---|---|
| native coordinator | 新規`ui/native/src/frame_coordinator.rs`、既存`frames.rs` | clock、generation、surface提出 |
| 読み手移行 | `snapshot.rs`、`snapshot_cache.rs`、`editor/stage.rs` | status、Inspector、cageが再resolveせずframeを読む |
| 非playback | export、warm-up、analysisの専用file | still、export、analysisを同じGraph意味へ寄せる |

P2終了後、統合ownerが直接`resolved_layers`を呼ぶ箇所を検索し、製品hot path、編集時query、test/oracleへ分類する。

## 直列削除 S2

- 製品playbackから旧resolve入口を削除する。
- view側から作品評価を開始できなくする。
- production fallbackを残さない。
- 移行専用adapterを残件表へ送らず、そのwaveで削除する。
- Flutterの見た目と操作は変更しない。

Flutterのplayhead補間や毎frame channel削減は、FrameGraph成立後の独立waveにする。UIをFrameGraph施工の変数にしない。

## 並列速度のシミュレーション

作業量を同じ大きさの単位として表す。

```text
直列: S0 + A + B + C + S1 + D + E + F + S2 = 9単位
並列: S0 + max(A,B,C) + S1 + max(D,E,F) + S2 = 5単位
```

理論上のcritical pathは約56%。実際はS1の統合と初回buildが重いため、期待値は1.5〜2倍程度。4人を超えると中央fileの衝突、別Node型、重複cacheが増え、逆に遅くなる。

Cargoは全laneで同時に起動しない。各laneは編集と自己点検を並列化し、wave合流後に統合ownerが温かいtargetで一度だけcheckする。

## テスト方針

### lane中

各laneで画素testや実窓を回さない。wave合流後に一本だけ:

```sh
cargo check -p motolii-ui --lib
```

`motolii-ui`が`motolii-render`をFlutter向けfeature込みで引くため、Graphからnative bridgeまでの型、借用、feature、呼び残しを確認できる。

`cargo check`が証明しない物:

- 同じNodeKeyが一度だけ実行された
- CameraとStageがupstreamを共有した
- dirty範囲とcancelが正しい
- 旧経路と画素が一致した
- surface、playhead、UIが同期した
- p90、drop、GPU wait/readbackが予算内

### Graph自身の最小契約

新しく必要なのは画素testではなく、次の安い実行回数契約だけ。

1. 同じNodeKeyを2 root・100 instanceから参照してもexecute 1回。
2. 1 Node変更でそのNodeと下流だけ再実行。topology compileはrevisionごと。
3. 新generation完成後に旧generationをpublishしない。

これらはFrameGraphのAPIと一緒に置く。旧実装を複製したoracleを書かない。

### cutover時の既存oracle

画素の答えは既存testを使う。

```sh
cargo test -p motolii-render --lib \
  the_window_target_holds_the_same_bytes_as_the_export_readback \
  -- --test-threads=2

cargo test -p motolii-render --lib \
  the_three_projections_coincide_under_the_default_camera \
  -- --test-threads=2

cargo test -p motolii-ui --lib \
  the_stage_window_draws_beyond_the_frame_and_the_camera_does_not \
  -- --test-threads=2

cargo test -p motolii-ui --lib \
  the_selected_cage_is_the_drawn_pixels \
  -- --test-threads=2

cargo test -p motolii-ui --lib \
  the_frame_ready_signal_fires_once_per_render \
  -- --test-threads=2

cargo test -p motolii-ui --lib \
  play_then_tick_advances_the_frame \
  -- --test-threads=2
```

全GPU testを毎waveで回さない。既知の事前失敗`the_cage_follows_the_drawn_text_under_an_orbited_camera`を今回の回帰へ混ぜない。

### 性能

既存probeを使う。

```sh
MOTOLII_PROBE_SCRIPT="$PWD/motolii/ui/native/src/editor/script/examples/stagger_reflow.js" \
MOTOLII_PROBE_SECONDS=12 \
cargo test -p motolii-ui --lib frame_owners \
  -- --ignored --nocapture --test-threads=1
```

既存出力へ`graph compiles`、`Node executions`、`Node reuse`、`cancelled generations`を加える。新しいprobe入口を増やさない。

### 実窓

実窓はF2 cutoverとF5一経路化の二回だけ。

F2:

```text
stagger_reflow.jsを開く
→ Cubeを追加
→ play
→ pause
→ seek
→ Stage Fit/resize
```

確認:

- UI layout、panel、controlが同じ
- CameraとStageが同じframe
- playhead、Frame表示、絵が一致
- `stage-window verdict=matched lag=0`
- warm p90 < 33.333ms、期限内drop 0
- scene evaluation 1、projection 2

F5:

```text
edit → visible frame → Undo → save/reopen → export
```

## Wave gate

| 段階 | lane | 合流時 | 実窓 |
|---|---|---|---|
| F0 | 文書のみ | `check-docs.sh`、`git diff --check` | 不要 |
| F1 | 編集のみ | `cargo check -p motolii-ui --lib`、Graph 3契約 | 不要 |
| F2 | 編集のみ | 既存画素・投影・frame-ready、`frame_owners` | 1回 |
| F3 | 編集のみ | preview projection、clock、dirty/cancel | 終了時1回 |
| F4 | 編集のみ | 触った機能の既存filter | 表示が変わる機能だけ |
| F5 | 編集のみ | 旧入口検索0、対象crateの既存oracle | 最終1回 |

## Go / Stop

### Go

- S0の型が一意に決まる。
- laneごとの所有fileが重ならない。
- old/new比較はoracle入口だけで、製品二重実行をしない。
- cutoverで削除する旧ownerが先に列挙されている。
- UIが差分に含まれない。

### Stop

- laneが`engine.rs`、`render.rs`、native ABIを独自に変更し始める。
- NodeKeyに完全入力ではなくLayer IDやrevisionだけを入れる。
- cacheで旧呼び出し構造を延命する。
- productionに旧経路fallbackを残す。
- compile greenを共有・画素・実窓の証明として報告する。

