# M3 U1b-2 latest result投影契約

作成日: 2026-07-21
状態: **決定 / U1b-2実装完了**

## 1. 目的

U1b-2は、U1b-1のgeneration付き実`RenderedFrame`をegui event loopへ戻し、
最新要求と一致する結果だけをU1a-1の安定display slotへGPU copyする。

```text
event-loop producer ── submit ── render worker
       ▲                                │
       │ repaint signal                 └─ latest result
       │                                       │
       └──────── event-loop drain / stale gate ┘
                              │
                              └─ accepted resultだけ既存display slotへcopy
```

U1b-2は通知、worker owner/client分離、stale gate、安定slot更新、完了配送順反転fixtureを
閉じる。seek UI、再生clock、resizeによるrender desc変更、display pool増設、診断UI、
性能閾値は追加しない。

## 2. ownerとclientの寿命

U1b-1 workerを次のprivateな二つへ分ける。

- **owner**: `JoinHandle`を唯一所有する。`run_native`を呼ぶshell側に残す
- **client**: request/result shared stateとrepaint signal登録口だけを共有し、
  `MotoliiApp`がevent loop中に所有する。join handleを持たない

shellは同じ`Arc<GpuCtx>`とbootstrap `Arc<Document>`で初期静止slotを準備した後、
常駐worker ownerを作る。app constructionへclientと、同じDocument/time/desc/Qualityの
初回requestを渡す。appはevent loop開始後にそのrequestをsubmitする。

`eframe::run_native`の成功・runtime error・app construction errorのどの場合も、
戻った後にshellがownerをcloseしてjoinする。event-loop中の`logic`、`ui`、
widget callback、appの`Drop`はjoinしない。runtime errorとjoin errorが同時に起きた場合は
両者の型付きsourceを失わないprivate合成errorを既存`ShellError::Runtime` sourceへ入れ、
公開`ShellError` variantを追加しない。

clientの全cloneがdropされてもownerがcloseされるとは限らない。正常終了の正本は
shellの明示close+joinであり、ownerの`Drop`はU1b-1どおりclose通知だけで同期joinしない。

## 3. repaint signal

worker coreへtoolkit非依存のprivate `Arc<dyn Fn() + Send + Sync>` signal slotを一つ追加する。
`MotoliiApp::new`が`CreationContext`のclone済み`egui::Context`を捕捉するclosureを登録し、
closureが行うのは`Context::request_repaint()`だけとする。
signal登録ごとに非ゼロ単調epochを割り当てる。

workerはresultをresult mutexへpublishして全lockを解放した後、signalをcloneし、
どのrequest/result/notifier lockも保持せず呼ぶ。signalはwidget、layout、Document、
display slot、`TextureId`を直接変更しない。worker moduleの型にegui/eframe型を入れない。

登録とpublishの競合で通知を失わない。

1. publish側はresult格納後に登録済みsignalを読んで呼ぶ
2. 登録側はsignal格納後にresult slotが非空なら即座に一度呼ぶ
3. 両方が呼ぶ重複repaintは許すが、resultがあるのに両方とも呼ばない状態は禁止する

signal panicはrender executor panicと混同しない。`catch_unwind`でworker thread外への
unwindを止め、呼んだsignal epochをprivateな`repaint_signal_failed` snapshotへ記録する。
notifier lockを取り直した時に同じepochがまだ登録中ならそのcallbackを除去し、新しいepochへ
同時に差し替わっていれば新callbackを消さない。panicしたcallbackを後続publishで再試行しない。
resultは保持し、workerは後続requestを処理する。panic payloadを文字列化しない。

appは通常の各event先頭でfailure snapshotを読む。未処理のfailed epochがあれば、
clone済み`egui::Context`から新しいsignalを登録し直し、同じeventで既存resultを即drainする。
再登録時にresultが残っていれば§3の登録側規則でrepaintも要求する。
signal panic後、次のOS/user eventが来るまで自動wakeを保証しない。通常returnするsignalに
対する「欠達なし」とpanic縮退を混同せず、診断表示・無限retry・watchdog threadを
U1b-2へ追加しない。

## 4. event-loop stale gate

repaint signalはevent loopの次frameを予約するだけでcopyを行わない。
copyの起動点はwake後の`MotoliiApp::logic`冒頭に固定し、そこで一度だけlatest resultを
drainして次の順序を実行する。`ui`、paint callback、repaint closureからcopyしない。

1. clientの`latest_accepted_generation`を読む
2. result generationがlatest acceptedと一致しなければ、textureをcopyせずresultをdropする
3. result generationが`last_displayed_generation`以下なら同様にdropする
4. latest resultが型付きerrorなら既存displayを保持し、copyしない
5. latest `RenderedFrame`だけを既存display slotへcopyする
6. copy成功後にだけ`last_displayed_generation`を進める
7. `TextureId`、stable view、native registrationは変更しない

typed worker error、descriptor mismatch、GPU copy errorはそのresultを消費して既存displayを
保持し、同じgenerationをrepaintごとに再試行しない。`last_displayed_generation`を
進めず、後から受理された新しいgenerationのresultだけを次の投影候補とする。
retry policy、error UI、同generationの再renderは後続診断/操作契約まで発明しない。

製品のrequest submitterはevent-loop上の単一clientだけとする。latest snapshotを読んでから
copyを終えるまで別threadが新requestを受理する設計はU1b-2へ入れない。将来producerを
増やす場合はstale checkとcopyの直列化境界を再設計する。

resultは一frameでlatest slotを一度だけ`try_take_latest`する。空なら何もしない。
worker resultを待つloop、sleep、channel recv、`device.poll(Wait)`をevent loopへ置かない。

## 5. display slotの更新

U1a-1で一度だけ作成・登録した`DisplaySlot`をapp終了まで使う。
U1b-2は既存private `copy`をevent-loop adapterから呼べる`pub(crate)`へ広げるだけで、
texture/view/slot ID/`TextureId`を作り直さない。

U1b-2の製品requestはbootstrapと同じ`FrameDesc`、`Quality::DRAFT`であり、
rendered descは既存slotと一致する。descriptor mismatchは型付き拒否し、既存displayを
保持する。loop内texture再生成、resize/DPIごとの再登録、品質変更用の第二slot、
GR-1 refcount poolを追加しない。render descやQualityを動的に変える接続はU1g等の
後続契約でdisplay poolを決めてから行う。

copyはGPU texture-to-textureだけで、CPU readback、map、visible bounds計算を行わない。
event loopがMotolii frameをrenderするのではなく、worker完了済み出力を表示用textureへ
copyするだけである。

## 6. 完了配送順反転fixture

production workerは単一であるためGPU job自体は直列完了する。U1b-2が防ぐべきraceは、
consumerが取得済みの古いresultがevent-loop queueで遅れ、新しいresultの後から
投影候補になる場合である。

fixtureはproductionと同じstale gateへgeneration付きresult envelopeを注入し、
配送順を意図的に反転する。

1. generation 1 resultをconsumerが取得済みとして保留する
2. latest acceptedをgeneration 2へ進める
3. generation 2 resultを先にevent-loop adapterへ配送し、generation 2を1回だけcopyする
4. 保留したgeneration 1 resultを後から配送する
5. copy回数、表示generation、display payloadがgeneration 2のまま不変であることを検査する

fixture executor/display payloadはGPU textureを偽装する公開抽象にしない。
private stale decision reducerとcopy spyで配送規則を固定し、別renderer、CPU preview、
第二のproduct result queueを作らない。

これとは別に実GPU/実window smokeで、同じ共有deviceのworker resultが
`request_repaint`後のevent loopで既存slotへ1回以上copyされ、
`last_displayed_generation == latest_accepted_generation`、slot ID/registration countが
不変であることを確認する。window smokeは自動closeし、joinが`run_native`帰還後であることを
raw evidenceへ残す。実GPU smokeで完了順反転を捏造しない。

## 7. U1a invariantの更新

U1a-1の静止slotではlifecycle中の全evidence不変を検査していた。U1b-2以後は正当な
result採用でcopy countが増えるため、lifecycle不変を次へ狭める。

- Document JSON不変
- slot ID不変
- native registration countは1のまま
- window resize、scale factor、minimize/restoreだけではrequest generationを増やさない

copy countはlatest result採用時だけ増えてよい。stale/error/descriptor mismatch、
layout操作、resize/DPI/minimize/restoreだけでは増えない。既存U1a window smokeを
この新しい不変条件でも通し、U1aのDocument・register-once契約を弱めない。

## 8. errorと非目標

U1b-2のprivate adapter errorはworkerの型付きerrorと`DisplaySlotError`を構造のまま保持する。
公開`ShellError`、`StaticPreviewError`へvariantを追加しない。製品surfaceへerror文字列を
仮表示せず、既存frameを保持する。

次は非目標である。

- seek/transport UI、再生、parameter編集からの連続request
- error/status/HUD、activity、retry button
- output desc、Quality、overscan、DPIに応じたdisplay pool変更
- 複数worker、複数producer、GPU work強制cancel
- FIFO result queue、全generation表示、CPU bridge/readback
- 新規公開API、serde、Document/journal/Undo/plugin契約
- U2b single writer、U2c共通gesture、U1g deadline/DRS

## 9. STOP条件

次のどれかが必要に見えた時点で実装を止める。

- event-loop threadでrender、join、blocking recv、sleep、同期readbackを行う
- workerからegui widget、layout、slot、`TextureId`を直接変更する
- request/repaint/result lockを同時保持する
- descriptor mismatchをtexture再生成やregisterし直しで埋める
- stale/error resultでcopy countまたはdisplay generationを進める
- public error/API、永続形式、Document意味、plugin契約の変更
- 完了順反転のためproductionを複数worker/FIFO queueへ変える
- test期待値変更、lint抑制、panic、文字列走査で契約を迂回する

Rerun source、crate、assetは本契約の根拠・依存・移植に使わない。

## 10. U1b-2完了条件

1. ownerだけがjoin handleを持ち、client/appはevent loopでjoinできない
2. `run_native`の全帰還経路でownerをclose+joinする
3. toolkit非依存signalの登録/publish競合で通知を失わず、lock外でだけ呼ぶ
4. signal panicはepoch付きsnapshotとなり、そのcallbackを除去する。次eventで再登録・drainし、
   resultとworker継続を失わない
5. copyはwake後の`MotoliiApp::logic`冒頭だけ。repaint callback、`ui`、paintから行わない
6. generation不一致、既表示以下、error、descriptor mismatchはdisplay copyゼロで、
   同resultを再試行しない
7. latest成功resultだけcopyし、成功後だけdisplay generationを進める
8. 配送順`2→1` fixtureでgeneration 2のcopy 1回だけ、表示payload/generationは2のまま
9. 実GPU/実windowでworker resultをevent loopへ投影し、slot ID不変、registration 1、
   latest generation一致、Document JSON不変
10. resize/DPI/minimize/restoreだけではgeneration/copy countが増えない
11. app/worker product sourceにrender、join、readback、loop内texture作成、再登録がない
12. U1g/U2b/U2c、display pool、診断UI、公開APIを実装していない
13. `cargo fmt --all -- --check`、`./scripts/check-docs.sh`、
    `./scripts/check-ui-toolkit-deps.sh`、`cargo clippy --workspace --all-targets -- -D warnings`、
    `cargo test --workspace`が通る

これを満たした時だけU1b-2を完了とし、最新mainからU2b-1を単独実行する。
