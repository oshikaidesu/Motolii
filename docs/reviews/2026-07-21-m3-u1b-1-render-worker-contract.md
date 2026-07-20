# M3 U1b-1 render worker契約

作成日: 2026-07-21
状態: **決定 / U1b-1実装完了**

## 1. 目的

U1b-1は、U1a-1のone-shot setup renderを常駐workerへ置き換える前段として、
次のprivate境界だけを成立させる。

```text
producer
  └─ submit(payload) ── latest request mailbox ── render worker
                                                    │
                                                    └─ latest result mailbox ── consumer
```

producerはconsumerやGPU完了を待たずに最新要求へ置換できる。workerは実行中のGPU workを
強制cancelせず、終了後にmailboxを読み直してその時点の最新要求だけを開始する。
結果には要求時に割り当てたgenerationを保持する。

U1b-1はworkerとmailboxの所有・失敗・終了までを閉じるが、`MotoliiApp`、
`egui::Context`、repaint通知、display textureへのコピー、`TextureId`登録、
event-loop投影、古い結果を表示しないE2EはU1b-2へ送る。

## 2. authorityと閉集合

正本はMotoliiの[M3 UI統合仕様](../specs/M3-ui-integration.md)「デバイスとスレッドの規約」と
[M3 UI境界汚染の予防](2026-07-14-m3-ui-boundary-prevention.md)である。
[GPU Preview / Viewport先例調査](2026-07-18-m3-gpu-preview-viewport-prior-art.md)のPV-2は
実`RenderedFrame`を使う早期fixtureを要求するが、公開契約、Document意味、永続形式の
根拠ではない。

U1b-1で追加してよいものは`motolii-ui`内のprivateな次の要素だけである。

- 単調generationを割り当てる単一論理producer
- 容量待ちを持たないlatest request mailbox
- `Arc<GpuCtx>`、first-party runtime、単一`RenderSession`を所有する常駐worker
- generation付きの実`RenderedFrame`または型付き失敗を置くlatest result mailbox
- 最新受理generationとterminal状態のread-only snapshot
- 明示closeと、event loop外で行うjoin
- productionと同じmailbox規則を検査する決定的executor fixture

公開API、serde型、Document field、journal、Undo、plugin契約、ユーザー設定、
workspace profile、toolkit型を追加しない。

## 3. request payloadとgeneration

private request payloadは次を値として保持する。

- `Arc<Document>`
- `Arc<DataTracks>`
- `EvaluationTime`
- `FrameDesc`
- `Quality`

producerはpayloadを受理する同じ短い排他区間でgenerationを割り当て、mailboxの未実行要求を
丸ごと置換する。generationは非ゼロ`u64`で1から始まり、受理順に厳密増加する。
producer handleを内部で複製可能にしても、割当と置換の直列化点は一つだけにする。
同じrequest stateは最新受理generationを保持し、privateな
`latest_accepted_generation` snapshotで待たずに読める。これはU1b-2がstale resultを
比較する材料であり、UI投影やnotificationを行わない。

`u64::MAX`を割り当てた後のsubmitは型付き`GenerationExhausted`を返し、既存mailbox、
最新受理generation、resultを変更しない。wrap、0への復帰、飽和して同じgenerationを
再利用する実装は禁止する。

submitの「blockしない」は次の意味に固定する。

- bounded queueの空き、consumerの受信、workerの起床、GPU完了を待たない
- mailboxの置換に必要な短いmutex取得は許す
- filesystem、network、Document検証、graph build、GPU callをsubmit内で行わない
- workerが停止済みまたはclose済みなら型付き拒否し、要求を受理したように見せない

100連続submitの審判は時間閾値を発明せず、workerを意図的に停止したbarrier中でも
100回すべてが完了し、pending payloadがgeneration 100の1件だけであることを検査する。

## 4. latest requestの意味

mailboxはqueueではなく未実行要求の単一slotである。

1. workerが待機中なら最新要求をtakeして実行中にする
2. 実行中に届いた要求はpending slotを置換し、中間要求を蓄積しない
3. 実行中の要求は取り消さず、GPU workを最後まで完了させる
4. 完了結果をresult mailboxへ置いた後、pending slotを読み直す
5. pendingがあればその時点の最新1件だけを次に実行する

pending、close、terminalを一つのrequest-state mutexで保護し、同じmutexに結び付いた
`Condvar`を起床に使う。workerは`pendingなし && closeでない && terminalでない`を
predicateとするwait loopだけで眠る。submitは同じmutex内でpendingを置換してからnotifyし、
closeも同じmutex内でflagを立ててからnotifyする。predicate確認とsleep登録の間に別の
notify-only flagを挟まず、lost wakeupで受理要求を取り残さない。spurious wakeupは
predicate再確認で吸収する。

workerがgeneration 1を実行中に2から100が届いた場合、開始してよいのは1と100だけである。
2から99のgraph build、render、result生成は禁止する。ただし1の結果がU1b-2で表示可能か
どうかは最新要求generationとの比較で決まり、U1b-1ではUIへ投影しない。

## 5. canonical render経路とGPU所有

workerは受理payloadごとに次の既存経路だけを使う。

1. `Document::validate`
2. `GpuCtx::check_health`
3. `build_document_frame_graph`
4. `render_graph_cached`
5. `GpuCtx::check_health`

workerは起動時に1回だけfirst-party runtimeと`RenderSession`を作り、request間で
`RenderSession`を再利用する。previewとFinalの別render関数を作らず、差はpayloadの
`Quality`だけとする。worker threadからDocumentを書き換えない。

`render_graph_cached`が返す実`RenderedFrame`は、M3仕様の既存契約どおり
`RenderSession`中間poolとは独立した専用出力である。U1b-1 worker自身はloop内で
`device.create_texture`、buffer、pipeline、shader module、sampler、display slotを
追加生成せず、正規rendererの専用出力をそのままresultへ移す。
専用出力のrefcount pool化はGR-1/M4候補であり、U1b-1から前倒ししない。

U1b-1はdisplay textureを作らない。U1b-2がresultを採用するとき、
M3仕様「プレビュー出力の寿命」に従って独立display copyへGPU copyしてから
安定`TextureId`へ投影する。

## 6. result mailboxと失敗

resultは次のpairを所有するprivate値である。

- requestのgeneration
- `Result<RenderedFrame, RenderWorkerError>`

result mailboxも単一slotであり、新しい完了結果は未取得の古い結果を置換する。
consumerは待たない`try_take_latest`で所有権ごと取り出す。`RenderedFrame`をclone可能に
するための`Arc`化、raw texture公開、CPU画素化は行わない。

request stateとresult slotは別mutexとし、同時に保持しない。workerはrequest lockを
解放してからrenderし、result lockだけを取ってpublishし、それを解放してからrequestを
読み直す。producerはrequest lockだけ、consumerはresult lockだけを取る。
`try_take_latest`はresult lock内の`Option::take`であり、空を返した直後にpublishされた
結果は次回takeで観測する。request/resultをまたぐlock順序や通知はU1b-1へ作らない。

`RenderWorkerError`は少なくとも既存のDocument検証、graph build、first-party runtime、
render、GPU healthの構造化errorをtransparentに保持する。文字列へ潰さない。
失敗もgeneration付きresultであり、失敗した要求の後に新しいpending要求があればworkerは
継続する。device lost等により後続も失敗する場合も、各受理generationの型付き結果として
扱い、workerからUI状態を変更しない。

executorがpanicした場合はthread境界で捕捉し、実行中generationの
`WorkerPanicked`結果を1回置く。次にrequest lock内でterminalへ遷移し、受理済みpendingを
実行せず破棄して、そのgenerationを`abandoned_pending_generation`としてterminal snapshotへ
保持してからworkerを停止する。panic payloadを表示文字列や公開errorへ流さない。
consumerは`latest_accepted_generation`と
`Running / Closed / WorkerPanicked { running_generation, abandoned_pending_generation }`
のprivate snapshotを待たずに読める。これにより、panic時に最新generationの結果を
待ち続けない。停止後のsubmitは`WorkerStopped`、joinは同じ停止事実を型付きで返す。
mutex poisoning、join panicを`unwrap`で再panicさせない。

## 7. close、shutdown、join

producerをcloseすると、それ以後のsubmitは型付き拒否する。close時点ですでに受理済みの
pending最新要求は破棄せず、workerが実行してresultを置いてから終了する。
実行中のGPU workも強制cancelしない。

正常closeとpanic停止は意図的に非対称である。正常closeは最新pendingをdrainする。
panicはworker内部の不変条件が継続可能とは限らないためpendingを実行せず、§6のterminal
snapshotへ破棄generationを残す。closeとpanicが競合した場合も、panicをterminal原因として
優先し、pendingを二重実行・無言破棄しない。

joinは明示操作であり、worker終了まで待つ可能性があるためegui event-loop threadから
呼んではならない。U1b-1の自動試験は別のowner threadまたはevent loop開始前後の外側から
closeしてjoinする。U1b-2で製品shellへ接続するときも、event loop中の`update`、
widget callback、`Drop`から同期joinしない。

ownerの`Drop`はcloseを通知するが同期joinしない。通常経路は明示close+joinを必須とし、
未joinをテスト証跡で検出する。GPU hangを隠すtimeout後detachやprocess abortを
正常終了として扱わない。

## 8. U1b-2へ残す境界

次はU1b-1へ入れない。

- result到着をevent loopへ知らせるnotifierまたはrepaint方式の決定
- `MotoliiApp`、Stage、layout surfaceへのworker接続
- latest requested generationとresult generationの比較・古い結果破棄
- `DisplaySlot`の更新pool、GPU copy、stable view、native texture登録
- 完了順反転fixtureをevent loopへ投影し、古いframeを一度も表示しないE2E
- window resize、DPI、minimize/restore、再生clock、seek gestureとの製品接続

U1b-1のresult mailboxが中間結果を置換することだけを、U1b-2の「古い結果を表示しない」
審判の代わりにしない。consumerが取得済みの古いresultが後からUIへ届く場合を
U1b-2で必ず検査する。

## 9. STOP条件

次のどれかが必要に見えた時点で実装を止め、仕様へ戻す。

- `Document`、journal、Undo、plugin、公開UI API、永続形式の変更
- egui/eframe state、`TextureId`、widget callbackをworkerへ渡す
- `download_rgba`、map、`device.poll(Wait)`等の同期readback
- requestをFIFO queueへ変え、中間generationをすべてrenderする
- GPU workの強制cancel、別preview renderer、CPU preview bridge
- `render_graph_cached`以外の重複planner/helper、raw JSON/文字列走査
- worker loopで追加GPU resourceまたはdisplay slotを直接生成する
- GR-1出力pool、U1b-2表示copy、event-loop notifierを同時実装する
- test期待値変更、lint抑制、panic、error文字列化で境界を迂回する

Rerun source、crate、assetは本契約の根拠・依存・移植に使わない。

## 10. U1b-1完了条件

1. request payload、generation、request/result mailbox、snapshot、error、shutdownがprivateでtoolkit非依存
2. workerをbarrierで停止中の100連続submitがconsumer/GPUを待たず完了し、pendingは100だけ
3. generationは1から厳密増加し、`u64::MAX`後を型付き拒否して状態不変
4. generation 1実行中に2から100を送るfixtureでexecutor開始列が`[1, 100]`
5. 実`Arc<Document>`、実`GpuCtx`、単一`RenderSession`、`render_graph_cached`から
   generation付き実`RenderedFrame`を得るGPU統合試験
6. Document不変、preview/Final共通render入口、共有device同期readbackゼロ
7. errorをgeneration付きで取り出せ、後続pendingを処理できる
8. wait predicateとの競合fixtureでlost wakeupせず、request/result lockを同時保持しない
9. panicは実行中generationの`WorkerPanicked`、terminal snapshotは破棄pending generation、
   停止後submitは`WorkerStopped`となりprocessをpanicさせない
10. close後submit拒否、受理済み最新pendingをdrainし、event loop外のjoinが完了する
11. `latest_accepted_generation` snapshotでresultのstale判定材料を読めるがUI投影しない
12. production worker sourceにegui component更新、`TextureId`、readback、
    loop内の直接GPU resource生成がないことをAST/source監査する
13. U1b-2のdisplay copy、notifier、stale-result event-loop E2Eを実装していない
14. `cargo fmt --all -- --check`、`./scripts/check-docs.sh`、
    `./scripts/check-ui-toolkit-deps.sh`、`cargo clippy --workspace --all-targets -- -D warnings`、
    `cargo test --workspace`が通る

これを満たした時だけU1b-1を完了とし、最新mainからU1b-2を単独実行する。
