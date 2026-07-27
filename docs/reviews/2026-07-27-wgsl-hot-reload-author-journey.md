# WGSL hot reload作者経路 — INF-8具体化

作成日: 2026-07-27

状態: **比較中**

対象: INF-8(a)

関連: [開発体験](../dev-experience.md)、[render worker契約](2026-07-21-m3-u1b-1-render-worker-contract.md)、[latest projection契約](2026-07-21-m3-u1b-2-latest-projection-contract.md)、[backlog](../backlog.md)

## 1. 結論

開発ビルドのWGSL hot reloadは、次の作者経路として具現化する。

```text
保存
  → watcherを変更のhintとして受ける
  → その時点の正確なbytesをsnapshot
  → 同一内容をdedupeし、候補へ単調な順序を付ける
  → shader-only境界と既存pass shapeに対してpreflight
  → render workerの既存serial pointで最新候補だけactivate
  → 通常のrender requestを1件投入
  → 既存のRenderGeneration／latest projectionで新frameを表示
```

失敗時は直前の正常な**artifact**でPreviewを描き続け、`source != active`をdevelopment専用診断面へ常時表示する。表示画像を凍結するのではなく、再生・scrubは旧artifactで継続する。未反映sourceがある間のExportは、同じrendererへ入る前に型付き診断で拒否する。

これはINF-8実装粒への仕様入力であり、現行の公開API、`PipelineCache`、`PipelineCacheKey`、`NodeDesc`、Document、永続形式、plugin payloadを変更する許可ではない。

## 2. 現行コード事実

- first-party WGSLはRustの`&'static str`として組み込まれ、起動時のlast-good artifactを作れる
- `motolii-gpu::PipelineCache`は`PipelineCacheKey { id, wgsl }`の同期get-or-createであり、invalidate、上限、非同期compile、watcher、last-goodを持たない
- `PipelineCache`と`PipelineCacheKey`は`motolii-plugin`から公開されている。INF-8都合で形を変えるとplugin公開契約へ波及する
- 現行v1 plugin objectと組込みWGSLはstaticに共有されるため、reload対象は一つのlayer instanceではなく同じplugin idを使う全layerである
- `motolii-ui`のrender workerにはlatest requestだけを次に開始する`RenderGeneration`とserial pointがあり、GPU work実行中の取消はしない
- event-loop側はlatest成功結果だけをstable display slotへ反映し、失敗時も既存表示を保持する
- M4の恒久frame cacheは未実装である。現行の`render_graph_cached`をframe cache成立の証拠にしない
- watcher、background compiler、runtime source payload、per-instance shader sourceはいずれも未実装である

## 3. 「世代」を一語へ畳まない

以下は概念上の名前であり、この文書から公開型を追加しない。

| 軸 | 役割 | 順序 |
|---|---|---|
| `SourceIdentity` | snapshotした正確なsource内容のidentity。同一内容保存のdedupeに使う | なし |
| `CandidateSequence` | build候補の新旧を決めるTransientな単調列。A→B→Aでも最後のAを最新と判定する | あり |
| `ArtifactIdentity` | activate済み実行物のidentity。`source != active`判定と将来cache keyへ使う | なし |
| `RenderGeneration` | 既存render request／display結果の新旧を決める | あり |
| `DocumentRevision` | 編集正本のrevision | hot reloadでは不変 |

`SourceIdentity`のhash値を時系列として比較しない。watcher event、mtime、wall clockも正本にせず、eventは再読込のhint、snapshotしたbytesをsourceの正本とする。

## 4. 状態と所有

状態はHostのdevelopment専用**Transient**であり、Document、User settings、Workspace profile、Project session、journalへ保存しない。

| 作者に見せる状態 | 意味 |
|---|---|
| `Up to date` | watched sourceとactive artifactが一致 |
| `Building` | 最新候補を検証中。Previewはactive artifactで継続 |
| `Rejected — showing last good` | source読込またはshader検証に失敗。active artifactは不変 |
| `Rebuild required` | parameter、binding、pass shape、asset closure等が変わりshader-only reloadでは扱えない |

古い候補の`Superseded`はtrace／計測eventであり、作者へ残留状態として見せない。最新候補だけがactive artifact、表示、診断、将来cacheへ影響できる。

### 4.1 activation点

候補のactivateは、既存render workerで「実行中のGPU workが完了し、次のlatest requestを開始する前」のserial pointへ置く。activate後は通常のrender requestを1件投入し、新artifactで描いた結果も既存`RenderGeneration`とevent-loop stale gateを通す。

hot reloadがdisplay bufferを直接書き換えたり、独自の第二rendererを作ったりしない。GPU commandの途中取消も要求しない。

## 5. PreviewとExport

### 5.1 Preview

- compile／pipeline生成に失敗してもactive artifactを捨てない
- 再生、scrub、parameter調整はactive artifactで継続する
- `source != active`、対象plugin id、最新診断をdevelopment専用routeへ表示する
- 修正sourceがactivateしたら、通常render request経由で表示を更新する

### 5.2 Export

watched sourceとactive artifactが一つでも不一致なら、Export admissionで型付き拒否する。reload中にExport専用compileを行わず、旧artifactへ暗黙pinせず、「古い版を使う」選択もv1へ作らない。

PreviewとExportは同じcanonical rendererを使う。差はrendererではなく、開発中Previewがlast-good継続を許す一方、Export admissionが未反映sourceを拒む点だけである。拒否によってDocument、journal、active artifactを変更しない。

## 6. shader-only reloadの境界

| 変更 | INF-8での扱い |
|---|---|
| function本体、定数、数式、同じentry point／binding／pass shape内のWGSL | candidate buildへ進める |
| 構文、型、shader validation、既存layoutに対するpipeline生成失敗 | `Rejected — showing last good` |
| binding追加・削除・型変更、entry point／pass shape変更 | `Rebuild required` |
| parameter宣言、NodeDesc、port、asset closure、plugin契約変更 | `Rebuild required`。definition reloadの別問題 |
| Rustロジック変更 | 製品再build＋再起動 |
| device lost／VRAM OOM | authoring errorにせずreload loopを止め、INF-4／GAP-27へ渡す |

WGSLがcompileできても意味互換は証明されない。たとえば同じlayoutのuniform field順序変更はpipeline validationを通りながら画素意味を変え得る。INF-8では文字列走査や独自reflectionでparameter意味を推測せず、この負例を既知の非証明範囲として残す。typed definition、uniform/binding生成、module/include closureはVism runtime入口の課題である。

## 7. 最小実装形

最初のprobeでは長寿命background compiler serviceを新設しない。watcherはHost dev harnessに閉じ、snapshot／dedupe／候補順序を管理する。GPU pipeline生成は既存deviceとserial pointを所有するrender worker内で同期実行してよく、compile hitchはdevelopment専用の既知制約として計測する。

構文parse等のdevice非依存作業を後にworker外へ出す余地は残すが、INF-8からasync product compiler、descriptor closure、pipeline eviction、production prewarmを逆算しない。それらはGAP-30で別に裁定する。

ファイル読込I/O失敗ではactive artifactを変えない。保存中のpartial bytesを正常に読めた結果が無効WGSLなら一時的に`Rejected`になり得るが、黒画面やactive破棄を起こさず、次のeventまたは明示`Reload`で回復する。固定debounce値、mtime安定待ち、pollingを契約にしない。

現行v1では一つのplugin idに対するactivateを、そのpluginを使う全layerへ反映する。layer単位のローカルsourceをINF-8から発明せず、per-instance sourceはV2へ送る。

## 8. 必須fixture

| ID | 操作 | 審判 |
|---|---|---|
| HR-1 | 有効な1回保存 | 最新候補を1回activateし、新artifactのframeを表示 |
| HR-2 | 100回のburst保存 | 最新候補だけactivate。古い候補は表示、active、診断へ影響しない |
| HR-3 | A→B→A | 最後のAが最新。内容hashが最初のAと同じでも古い候補扱いしない |
| HR-4 | 構文error→修正 | Previewは旧artifactで動き続け、mismatch表示後に修正版へ切替 |
| HR-5 | I/O失敗／partial save→完成 | 黒画面なし。時間閾値に依存せず次eventまたは明示Reloadで回復 |
| HR-6 | binding／parameter／pass shape変更 | compile errorへ潰さず`Rebuild required` |
| HR-7 | layout同一のuniform意味変更 | compile成功が意味互換を証明しない負例として固定 |
| HR-8 | `source != active`でExport | renderer実行前に型付き拒否。Document／journal／activeは不変 |
| HR-9 | 古い候補が遅延完了 | 新しいactiveと表示を巻き戻さない |
| HR-10 | device lost注入 | authoring errorに偽装せずreload停止と環境診断へ遷移 |

計測点は`watch received`、`source snapshot fixed`、`candidate started`、`preflight done`、`activated`、`render request accepted`、`new artifact frame displayed`とする。作者が感じる反復時間はwatch→compileではなく、**event→新artifactのframe表示**で測る。

## 9. Opus 5助言の処分

2026-07-27にread-only相談し、現行仕様・コードへ再照合した。

- **採用**: identityと順序の分離、既存render serial pointでのactivate、artifact last-good、Export admission拒否、`Rejected`と`Rebuild required`の分離、A→B→A負例
- **縮小採用**: parse／validateの分離は可能性だけ保持し、最初からbackground compiler serviceを作らない
- **延期**: pipeline cache上限・eviction・product cold compileはGAP-30、device lost/OOMはINF-4、詳細error taxonomyはGAP-27、frame cache keyはM4
- **棄却**: watcher timestampを正本にする、hot reload専用renderer、compile成功を定義互換または安全性の証明にする

## 10. 後続taskと停止線

- **INF-8**: 本状態機械、dev-only source snapshot、最新候補activate、last-good/mismatch、Export拒否、fixture、event→display計測
- **GAP-30**: product cold compile、async compiler/service、descriptor closure、owner thread、SLO、`PipelineCache`の上限・退役
- **GAP-27**: wgpu validation／pipeline／runtime／device failureの型付き診断
- **INF-4**: device lost／OOMからの復旧
- **M4**: frame cache実装時にactive `ArtifactIdentity`をcache keyへ含める
- **VSM-A8G0以後**: runtime所有source、typed uniform／binding、pass shape拡張、module/include closure
- **V2**: per-instance source、local Vism、WASM／process workerのruntime交換

以下が必要になった時点でINF-8実装を止め、該当仕様を先に改訂する。

- `PipelineCache`／`PipelineCacheKey`／`NodeDesc`／plugin traitの公開形変更
- Document、journal、serde面へのsource、watch path、generation、diagnostic保存
- frame cacheが未実装のままinvalidate APIを先行追加
- product向けbackground compilerまたは新しい長寿命service
- binding／parameter意味をraw WGSL文字列走査で推測
- hot reloadをsandbox、無限shader、TDR、OOMへの安全機構として公約
