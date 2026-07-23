# Rectangle dropのD2個別契約案（2026-07-21）

状態: **比較中 / 反対側レビューP0=0・P1=6、案Dは未採択**。BrowserからのRectangle dropを既存D2へ
接続するための選択肢と停止線を固定する。
本書は公開API、`Command` variant、Document/journal形式、selection保存形式を実装・採択しない。
全体の依存順と製品完成条件は[製品モック一括回収計画](2026-07-21-m3-product-mock-recovery-plan.md)を正とする。

## 1. 対象と非目標

対象は次の一本だけである。

```text
valid Rectangle drop
  -> stable LayerIdを一度だけ発行
  -> 既存Command::AddTrackItemを一度だけcommit
  -> 同じArc<Document> + Transient selection snapshot
     -> Preview / Timeline bar / Inspector
  -> Undo 1回で三面から消える
```

Timeline barは別のDocument objectではない。同じ`TrackItem::Clip`の`LayerId`、`start`、`duration`から
導出する。Inspectorも別のShape状態を持たず、同じsnapshot内でselectionが指すItemを投影する。

次は非目標とする。

- `VectorRecipe::StandardShape::Rect`のD3 lowering、Vello描画、fill/stroke意味の決定
- Timelineのlayout/render、U2h selection、React Inspectorの製品実装
- drag lifecycleを公開transaction APIにすること
- community panel、plugin API、外部D&Dへ同じwire形式を公開すること
- dropの既定`start`、`duration`、挿入track/indexを本書で発明すること
- journalへDOM event、pointer軌跡、drag ID、selectionを保存すること

正本の基本Shapeは型付き`VectorRecipe`だが、現行D3はVector sourceを描画できない。既存
`RECT_LAYER_SOURCE`を使う場合は編集契約だけを測るfixtureと明記し、製品Rectangleの完成に数えない。

## 2. 現行コード事実

1. `DocumentWriter::apply_macro(Vec<Command>)`は空列拒否、同一`GestureId`、途中失敗時のDocument、
   Undo/Redo、revision、gesture counter rollback、成功時revision 1増加を実装済みである。
2. `Command::AddTrackItem` / `RemoveTrackItem`は既存journal payloadであり、Item本体とsubtreeの
   `layer_names`を運ぶ。inverseは同じItemとIDを復元する。
3. `LayerIdTable`はIDを非再利用とする。UndoはItemと名前entryを外すが、採番counterを巻き戻さない。
4. `reserve_layer_id`と`allocate_layer_id`はlive Documentを即時変更する。drag start、preview、
   Stage外drop、Cancelより前に呼べば「Document変更ゼロ」を破る。
5. 現行`DocumentCommandRequest`が許すDocument intentは`DeleteTargetedItems`だけで、command kindは
   `RemoveTrackItem`だけである。配置をDeleteへ偽装してはならない。
6. U2bのedit runtimeは成功したApply/Undo/Redoだけ`Arc<Document>`をpublishするが、selection/target
   解決は所有しない。
7. selectionはTransientであり、Document、journal、Undo macroへ入れない。三面共有はU2hの責務である。

## 3. 不変条件

### 3.1 commit前

- drag start/updateはTransient previewだけを変更する
- `reserve_layer_id`、`allocate_layer_id`、D2、snapshot publishを呼ばない
- Escape、pointer cancel、capture/focus loss、Stage外releaseはDocument serialize、revision、
  Undo/Redo長、LayerId counterを完全不変にする
- preview messageが欠落しても、drop自身の最終座標だけからcommit値を決める

### 3.2 commit

- 有効drop 1件につき新規`LayerId` 1件、`AddTrackItem` 1件、`apply_macro` 1回
- apply直前に候補IDがlive writerの次IDであることをtypedに検査し、既存台帳entryの黙認をfresh createと数えない
- ID発行、Item挿入、plugin/Document validate、Undo積載のどれかが失敗したら全状態を呼出前へ戻す
- command payloadは決定済みの正準Y-up座標、`RationalTime`、親、index、Item、名前を持つ。CSS px、
  window座標、DPI、drag IDは持たない
- 同じdropの再配送は成功の再適用ではなく、既処理結果の再観測として扱う
- writer commit receiptは同じevent-loop turnで先にterminal化し、その後のprojectionは失敗しても再生成可能にする

### 3.3 commit後

- publish単位は`Arc<Document>`、document revision、Transient selection snapshotの整合した組とする
- Preview、Timeline、Inspectorは同じrevisionと`LayerId`を報告する
- Reactはraw Documentやselectionのwriterにならない
- Undo 1回でItem、Timeline bar、選択Inspector投影が消える
- Redoは同じItemと`LayerId`を復元する。LayerId counterは巻き戻さない

## 4. LayerId発行とAddTrackItem原子性の選択肢

| 案 | 形 | 既存journal互換 | 失敗時原子性 | 判定 |
|---|---|---|---|---|
| A | drop前に`reserve_layer_id`し、後で既存`apply_macro(AddTrackItem)` | 維持 | reserve後のpreflight/apply失敗でcounterだけ残る | **棄却候補**。Cancel境界と原子性を分断する |
| B | snapshotの`peek_next`をHostが`LayerId::from_raw`へ変換してcommandを作る | 維持 | apply内の`restore`ならrollback可能 | **停止線**。`from_raw`を製品採番へ転用し、stale snapshot競合と採番責務をHostへ漏らす |
| C | `CreateTrackItem`とLayerId reservationを新しい永続Command variantにする | 変更 | command内で閉じられる | **延期候補**。既存AddTrackItemとjournal/migration面を増やす根拠がない |
| D | `motolii-doc`内の狭いplannerがlive Documentを変えず次ID入りの既存`AddTrackItem`を準備し、同じwriter call stackでlive-next照合後に`apply_macro`する | 維持 | ID counter更新を既存command apply/rollback内へ置ける | **候補・未採択**。現U2b queueのprepared requestでは不可分性を保証できない |
| D0 | U2bのPlace専用private経路がwriter snapshotの`LayerIdTable` clone上で候補IDを作り、同じ同期call stackでlive-next照合→既存`AddTrackItem`をapplyする | 維持 | Dと同じ。汎用公開plannerを増やさない | **最小候補・未採択**。Place draftの全意味を仕様で固定してから比較する |
| E | UI側にclone writer/独自transaction/独自Undoを作り、成功後に差分を戻す | 不明 | 二重writerになる | **棄却候補** |

案Dを採る場合も、この文書からAPI名や公開可視性を確定しない。最低条件は次である。

1. plannerは`motolii-doc`の採番規律を再利用し、UI側がraw IDをmintしない
2. plannerはlive Document、counter、history、revisionを変えない
3. planner成果物は既存`Command::AddTrackItem`であり、新しいjournal形を作らない
4. actionをqueueからpopした後、writer現状態でplanし、同じ同期call stackでlive-next照合とapplyを行う。
   prepared requestを再queueせず、yieldや別editを挟まない
5. stale parent/index、ID exhausted、invalid Item、plugin prepare失敗をtyped errorで返す
6. 成功時に発行した`LayerId`をcommit receiptとして返し、同じevent-loop turnでdragをterminal化してから
   selection投影を開始する

plannerを公開しないとU2bへ接続できない、または汎用closure/transaction APIが必要に見えた時点でSTOPし、
M2 D2の個別仕様改訂と反対側レビューへ戻る。

## 5. Place intentとrequest境界

配置は既存Delete intentへ混ぜず、将来の個別チケットで次の三層に分ける候補とする。

```text
Web transport: Rectangle candidate + final pointer + drag identity
  -> Host preflight: target track/index/time + canonical position + fixed recipe
  -> Document request: PlaceRectangle + prepared AddTrackItem
```

- 安定`CommandId`候補は`motolii.create.place_rectangle`。文字列は未採択であり、registry/keymap追加を
  本書では行わない
- `DomainIntent`候補`PlaceRectangle`は操作目的だけを表し、pointer、DPI、WebView、DOM型を含めない
- requestはruntime-only、非serdeで、intentと`CommandKind::AddTrackItem`の組合せを閉じて検査する
- Reactの`shape.kind=rectangle`は候補catalog keyであり、任意のDocument JSON、plugin ID、paramsを
  Hostへ注入する口にしない
- Host preflightは現在のsingle-writer snapshotからtarget、index、時刻、正準位置を決定する。
  adapterが表示名やDOM順から推測しない
- request受理をinteraction stateのCommit点とし、受理後のEscapeを履歴から消すAPIは作らない

未決の`start`、`duration`、track/index、source recipe、selection policyのいずれかを暗黙defaultで
埋める必要がある間は、Place intent/requestを公開しない。

## 6. exactly-once drag dedupe

D2 command自体へtransport IDを追加しない。Host coordinatorのTransient protocolは最低限、次を持つ候補とする。

```text
DragKey = (webview_instance_epoch, drag_ordinal)
Event   = (DragKey, event_sequence, layout_epoch, kind, final_pointer?)
```

- `webview_instance_epoch`はHostがWebView生成ごとに注入するruntime値
- `drag_ordinal`と`event_sequence`はinstance内で単調増加。`pointerId`をdrag identityにしない
- dropはauthoritativeな最終pointerを必須とし、最後のpreview位置をcommitへ流用しない
- 同じsequence以下、terminal後のpreview/cancel/drop、古いinstance/layout epochをtyped staleとして拒否する
- coordinatorは同一WebViewでactive dragを最大1件に制限し、追加pointer/pen同時dragをtyped busyとして拒否する。
  複数active dragを許す設計へ広げる場合は、単一high-watermarkではなくactive `DragKey`集合とterminal floor/結果表を
  先に仕様化する
- instanceごとのhigh-watermarkと有界なterminal結果表を持つ。表から結果詳細を退役しても、完了済みdragを
  再び新規扱いしない
- terminal結果はcommit成功だけでなくtarget拒否、planner/apply失敗、Cancelも区別して記録する。失敗時は
  rollback完了後にterminal化し、同じdropの自動再試行で後からDocumentを変更しない
- duplicate dropはD2 queueへ二度入れず、最初のcommit receipt、同じtyped failure、または
  typed already-terminalを返す
- dedupe stateはDocument/journal/workspaceへ保存しない。process再起動を跨ぐWeb IPC event replayを許さない
- WebView reload後は新epochにし、旧epochの遅延messageを全拒否する

これは正常なruntime配送のexactly-once境界であり、network分散transactionやprocess再起動後の
transport再送保証を意味しない。

## 7. selection snapshot

selectionは`LayerId`を指すTransient projectionで、Document snapshotと別々に購読者へ配らない。
候補となるpublish envelopeは概念上、次を同時に運ぶ。

```text
PublishedUiState {
  revision,
  document: Arc<Document>,
  selection: { primary: Option<LayerId>, members: ... },
}
```

型名・集合型・generationはU2hで決めるため未採択である。不変条件だけを先に置く。

- Apply成功receiptの新`LayerId`をprimary selection候補にする
- selection内の全IDは同封Documentに存在しなければならない。Undo後はdangling IDを残さない
- Preview、Timeline、Inspectorは個別selection storeを持たない
- rename、Timeline packing、React reload、panel表示順はselection identityを変えない
- selection変更だけではDocument、revision、journal、Undoを変えない

Redo時の自動再選択は未決である。最低oracleは「同じLayerIdのShapeとbarが戻り、dangling selectionが無い」。
Inspectorまで自動復帰させるなら、selectionをD2へ保存せず、どのruntime edit receiptを再選択根拠にするかを
U2hで先に決める。Undo履歴へselectionを密輸して合格させない。

## 8. journal互換可能性と未解決durability

既存`AddTrackItem`は`JournalEdit` payloadへ包めるため、新しい永続variantを増やさずに済む可能性がある。
ただし現U2b `DocumentEditRuntime`は`apply_macro`とsnapshot publishだけを所有し、WALの`commit_edit`を呼ばない。
従って「成功時にjournalへ1件残る」は未成立であり、本書の合格事実にしない。

drag message、dedupe表、selection、commit receiptはjournalへ記録しない。将来のjournal統合では、
`apply_macro`成功後に`commit_edit`が失敗した場合の正本、retry、publish順を別契約で決める。

- Undoのinverseは既存`RemoveTrackItem`で、同じItemと`layer_names`を使う
- Redoは同じ`LayerId`をrestoreする
- 非再利用規律によりUndo後もLayerId counterは進んだまま。これは失敗ではない
- Cancel/Stage外drop/duplicate/staleはD2 commandを生成しない。journal接続後はrecord 0を別審判で確認する
- 新旧reader、migration、journal v1/v2変換、未知field保持の期待値を変更しない

既存journalが完全payloadの`AddTrackItem`を保存できない、またはRedoに新しいID reservation payloadが必要と
判明した場合は案Dを採択せず、M2解凍へ戻る。

## 9. 自動oracle案

共通fixtureはempty target track、固定composition、固定playhead、固定canonical drop位置を持つ。
sourceが製品Vector未対応の間はfixture-only rect sourceであることをテスト名と文書へ残す。

1. start→preview×N→Esc / capture loss / focus loss / Stage外drop:
   serialize、revision、Undo/Redo長、LayerId counter、journal bytesが完全不変
2. preview 0件→valid drop: drop座標でItem 1件、`apply_macro` 1回、revision +1、Undo +1
3. duplicate dropを2回以上配送: Item、ID、revision、Undoは1回分のみ。journal件数はjournal統合後の別審判
4. terminal後に小さいsequenceのpreview/cancel/drop: 全semantic state不変
5. plannerまたはapplyの先頭/validate/plugin prepare失敗注入: Document、counter、history、revision、
   selection publishが呼出前と同一。rollback後にdragは同じtyped failureでterminal化し、duplicateでもD2 0回
6. accepted drop: Preview、Timeline bar、Inspector projectionが同じrevisionと`LayerId`
7. Undo 1回: Item、bar、Inspectorのdangling selectionが消える。LayerId counterは巻き戻らない
8. Redo 1回: Itemとbarが同じ`LayerId`で戻る。selection自動復帰はU2h決定に従う
9. 別`drag_ordinal`: 次の非再利用`LayerId`となり、別Undo step
10. WebView reload: 新epochだけ受理し、旧epochの全messageはD2 0回

## 10. 実装開始の停止線

次の全てがYesになるまで製品コードへ実装しない。

- LayerId planner案がM2 D2の個別仕様と反対側レビューで採択された
- Place intentのtarget、start、duration、source recipe、canonical positionが決まった
- U2h selection publishとRedo時selection policyが決まった
- U3a Timeline projectionが同じLayerId/Clipを読める
- 製品Rectangleを名乗る場合はVectorRecipe D3/GPU経路が成立した
- React worktreeがmainのU2a/U2bへ再結合された
- fixtureにCancel、failure atomicity、duplicate/stale、Undo/Redo負例が揃った
- apply、dedupe terminal化、journal durability、snapshot/selection publishの順序と失敗時正本が決まった

許可前に行えるのは、既存APIのread-only監査、fixture-only protocol test、選択肢の反対側レビューである。
`Command` variant、serde、journal、公開raw API、UI独自writer/Undo、Document selection fieldを先行追加しない。

## 11. 反対側レビュー結果

2026-07-21のread-only反対側レビューは**P0=0 / P1=6**で、案Dを採択不可とした。

1. U2bはjournalを所有せず、journal 1件という断言とdurabilityが未成立
2. prepared request queueではplanとapplyの間に別editが入り得る
3. `AddTrackItem`は既存`LayerIdTable` entryを黙って採用し得るためfreshnessのlive-next検査が必要
4. commit、dedupe terminal化、journal、projection publishの失敗順序が未決
5. instance high-watermarkは単一active dragを暗黙に仮定していた
6. drop位置をTransformとShape centerのどちらへ置くか、size/scale、表示名が未決

この指摘を受け、最小候補D0、live-next typed検査、同期call stack、receipt先行terminal化、journal別審判、
単一active drag制約を本文へ反映した。D0も採択ではなく、Place draftの全意味とU2h/U3aが決まるまで停止する。
