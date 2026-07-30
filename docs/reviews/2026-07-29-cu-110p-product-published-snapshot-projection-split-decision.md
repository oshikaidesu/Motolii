# CU-110P 通常製品published snapshot投影の分割決定

- 日付: 2026-07-29
- 状態: **決定 / CU-110P SPLIT**
- 現在粒: **CU-110PS DO**

## 1. 結論

`CU-110`成功時にProduct Hostが採用した一つのpublished snapshot / primary /
`projection_generation`を、通常製品のStage、Timeline、Inspectorへ投影する責任を
`CU-110P`として次の三粒へ分割する。

1. `CU-110PS`: 同じsnapshotを既存latest-only render workerへ送り、完了した最新generationだけを
   既存VRAM display slotへcopyしてnative Stageを更新する。
2. `CU-110PT`: [CU-110PT0](2026-07-29-cu-110pt0-native-timeline-projection-envelope-decision.md)の
   非所有・非保存のcomposition全域envelopeと同じsnapshotを既存`project_timeline`へ渡し、
   native Timeline viewportへbarを描画する。選択入力は含めない。
3. `CU-110PI`: 同じsnapshotとprimaryから既決Inspector read-model inputを作り、
   product-owned `InspectorCandidate`の既存read-only identity projectionを通常製品WebViewへ載せる。

三面の入力は同じHost envelopeを使うが、GPU worker、native frame、WebView paintの完了時刻を
同期barrierにしない。三面同時pixel更新を「atomic publish」と呼ばず、stale generationを各consumerで
拒否する。

## 2. 順序

`CU-110PS → CU-110PT → CU-110PI → CU-106P → CU-111 → CU-108`とする。

- Stageを最初にする。現状はD2成功後も静止textureのままで、利用者成果が見えない。
- Timelineを次にする。headless projectionとnative viewportは既にあり、React Timelineを作らない。
- InspectorはR4C製品所有、`CU-0A08IP` decoder、`CU-0A08ITP` identity JSXを再利用する。
  `S`分類値、mock state、typed editing intent、`U4a-2`をread-only投影へ混ぜない。
- `CU-106P`はnative Timelineのnon-test caller成立後にproduction pointer入力と
  selection-only producerを同じ差分で閉じる。
- `CU-111`は三面consumer成立後に既存stable `CommandId`とsingle writerへUndo/Redoを接続する。
- `CU-108`だけが実機でPlace→三面→Undo→Redoの同一revision / `LayerId`をE2E判定する。

## 3. 再利用

- Stage: `RenderWorker` / `RenderWorkerClient`、`LatestResultProjection`と同じlatest-only判定、
  `DisplaySlot::copy`、既存same-device/queue Surface。
- Timeline: `project_timeline`、`TimelineProjection`、`TimelineHit`、native Timeline viewport。
- Inspector: `InspectorCandidate`、`decodeInspectorReadModel`、Inspector専用post-promotion chain。
- publish: `PublishedDocument`のsnapshot / primary / `projection_generation`だけを正本とし、
  surface別Document clone、selection store、generation counterを作らない。

## 4. 非目標

- 公開API、Document、serde、journal、plugin契約、永続layout形式の変更。
- React Stage / Timeline、egui製品面、mock/diagnostic routeによる代用。
- Inspectorの`S`分類値、editing intent、U4a-2、自動生成parameter panel。
- Timeline selection、focus、visible range、Undo/Redoを三面投影粒へ束ねること。
- CPU readback、同期GPU待ち、surface別snapshot / primary / history。

## 5. STOP

1. Stage更新に別renderer、CPU pixel path、二つ目display slot正本が必要になる。
2. Timeline描画にReact `TimelineCandidate`または新しいDocument意味が必要になる。
3. Inspector通常製品callerがmock state、legacy script、fixture default、`S`値を表示しないと
   成立しない。
4. 三面のために公開transport、Document field、selection / Undoの第二正本が必要になる。
5. visual threshold、golden、既存受入期待値を実装都合で変更したくなる。

## 6. handoff

`CU-110P`は`SPLIT`。`CU-110PS`、`CU-110PT0`、`CU-110PT`は`DONE`。
次の唯一のPRODUCT-ASSET `DO`は`CU-110PI`。
token後続は`WAIT`を維持する。
