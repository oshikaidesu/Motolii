# CU-G03 edit durability / publish順序決定

ステータス: **親DONE / CU-G03D 決定・DONE / CU-G03R 実装・DONE**
対象: M3 VS-1 Rectangle配置とUndo/Redo  
親粒: `CU-G03`  
子粒: `CU-G03D`（本決定）、`CU-G03R`（catalog未反映committed tail recovery guard）  
後続実装: `CU-109`、`CU-110`、`CU-111`

## 1. 目的とauthority

VS-1の製品編集で、journal、live `DocumentWriter`、revision、Transient selection、
snapshot publishのどれを先に確定するかを一意にする。

authorityは次の順に読む。

1. [M2仕様 D1d / D1m / D2](../specs/M2-document-model.md)
2. [D2 / selection / Timeline歴史回収 §3](2026-07-23-historical-d2-selection-timeline-lineage-recovery.md#3-u2b-2-place-product-core再採択)
3. [M3仕様 U2b](../specs/M3-ui-integration.md)
4. [M3縦slice実行方針 §4](2026-07-24-m3-vertical-slice-execution-decision.md#4-vs-1-rectangle配置とundoの現在slice)
5. [快適利用粒度化 CU-G03 / CU-109〜111](2026-07-22-m3-comfortable-use-granulation.md)

本決定は既存`JournalEdit` v2、`Command`、`ProjectSession`、`DocumentWriter`の意味を
変更しない。新しいjournal payload、Document field、公開raw writer APIは作らない。

## 2. 現行コード事実

- `JournalEdit` v2はversion付きの**単一`Command`**だけを永続化する。
- `commit_edit`はEdit append+fsync、Commit append+fsyncを終えてから成功する。
- edit-only journal commitはmain fingerprintを進めない。ProjectSessionのedit-only操作が
  journal commitとcatalog保存まで成功した経路では、reopen時にcommitted Editをreplayする。
  Commit fsync後・catalog保存前のerror/crashを成功経路へ数えない。
- 現行recoveryのmain fast pathはmain/catalog fingerprint一致、scan停止理由なし、
  `catalog.edits_since_snapshot == 0`だけを見ており、catalog保存前にCommit fsync済みとなった
  Edit tailの存在を判定しない。このtailを残したまま`Healthy` sessionを作ると、次の編集後の
  reopenで古いEditと新しいEditを初めてまとめてreplayし、liveで受理済みの意味とずれ得る。
- `DocumentWriter::apply_macro`は複数`Command`を受けられるが、途中失敗時はDocument、
  Undo/Redo履歴、revision、gesture採番を呼出前へ戻す。
- `DocumentWriter::undo`は保存済みcommandを逆順に`inverse()`し、`redo`はforwardを再適用する。
  現行`undo` / `redo`はlive適用失敗時にpop済みmacroをstackへ戻さない。
- journal replayは`Command::apply`とDocument validationを行うが、live `apply_macro`はさらに
  plugin catalogに対する`prepare_plugins`を要求する。
- 現行`DocumentWriter` / `UndoHistory`には次のUndo/Redo commandを読む公開口が無い。
- 現行`DocumentEditRuntime`はApply/Undo/Redo成功後だけrevision付きsnapshotを返すが、
  `ProjectSession` / journalへまだ接続していない。
- VS-1 Rectangle Placeは、accepted terminal dropごとに`AddTrackItem` **1件**を
  `apply_macro` **1回**で確定する。

したがって、現行payloadのまま耐久原子性を保証できる範囲は
**1 accepted action = 1 replay可能`Command`**である。`apply_macro`一般の複数command耐久を
この事実から推測しない。

## 3. 決定: VS-1単一Command actionの順序

Apply、Undo、Redoのaccepted actionごとに、Host single-writer coordinatorは次を直列に行う。

1. **non-live prepare / preflight**
   - live writerを変更せず、開始revisionと対象を検証する。
   - Applyはforward `Command`、Undoはundo stack先頭の単一forwardから得るinverse
     `Command`、Redoはredo stack先頭の単一forward `Command`を1件だけ確定する。
   - candidate Document上で、replay受理の`Command::apply`+Document validationと、
     live Apply受理のplugin catalog `prepare_plugins`を包含するsuperset predicateを
     成立させる。現行live Undo/Redoが後者を直接行わなくても、durable先行時は同じcandidate
     stateを検査し、replayだけが受理できるcommandをjournalへ先行させない。
   - open済みProjectSessionにrecover可能なmainまたはgeneration baseがあることを確認する。
     project初期化前のedit-only commitを許さない。
   - command数が1でない、stale、duplicate、対象不在、preflight失敗ならjournalへ進まない。
2. **durable commit**
   - 確定した1件を既存`JournalEdit` v2へ包み、保持中の`ProjectSession`経由で
     edit-only（checkpointなし）操作を行う。journal Commit fsyncだけでなく、既存catalog保存まで
     成功した時だけ本決定のdurable commit成功とする。
   - Apply/Redoはforward、Undoはinverseを記録する。forward/inverse pairを同じrecordへ入れない。
3. **live apply**
   - durable commit成功後に限り、同じsingle-writer call stackで対応する
     `apply_macro` / `undo` / `redo`をlive writerへ1回だけ適用する。
   - revisionはこのlive適用成功の既存結果として1回だけ進む。journal成功だけでは進めない。
4. **Transient reconcile**
   - live適用成功後、Documentに存在しないprimary selectionをpublish前にclearする。
   - selection、drag epoch、gesture、projection generationをjournalやDocumentへ保存しない。
5. **atomic publish**
   - 同じDocument revision、snapshot、reconcile済みselection、別軸のprojection generationを
     一つのpublish envelopeとして1回だけsurfaceへ渡す。

Apply/Undo/Redoをまたいだjournalの時系列がreopen後の最終Document意味を決める。
Undo履歴そのものは再起動後に復元せず、replay済みDocumentから新しいlive履歴を開始する。

## 4. failure authority

| 地点 | live Document / history / revision | journal | publish | 後続処分 |
|---|---|---|---|---|
| preflight拒否 | 不変 | 0 | 0 | typed rejection。自動retryしない |
| journal APIがerrorを返した | 不変 | 未commitまたはcommit済みの可能性を呼出側で推測しない | 0 | sessionをpoisonし、recover/reopen完了まで全編集をtyped reject |
| durable commit後のApply失敗 | `apply_macro`の既存rollbackにより呼出前のまま | replay対象commandはdurable | 0 | invariant failureとしてsessionをpoison。rollback recordや自動retryを作らずrecover/reopen |
| durable commit後のUndo/Redo失敗 | Document/history/revisionを権威として再利用しない。現行stackはpop済みmacroを失い得る | replay対象commandはdurable | 0 | runtimeを破棄してsessionをpoison。状態を観測・publishせずrecover/reopen |
| live適用成功 | revision +1、history更新 | 1 accepted command | 1 envelope | success receiptを返す |
| selection reconcile失敗 | Documentを巻き戻さない | durable | 0 | sessionをpoisonしrecover/reopen。古いselection付き成功をpublishしない |

`ProjectSession::save_with_journal`等の戻り値だけから「commit前失敗」と
「Commit fsync後のcatalog更新失敗」を利用側が推測してretryしてはならない。
通常のrecovery/reopenは、catalog counterだけでmain fast pathを選ばず、最後の
snapshot/checkpoint以後にcatalogへ反映されていないcommitted Edit tailがあれば既存replayへ
必ず送る。`CU-G03R`の完成範囲は`recover_project`がこのtailに
`RecoverySource::MainFile`を返さず既存replayへ入れるところまでとする。
`Healthy / Poisoned` authorityとopen後のedit-only拒否は`CU-109`が所有し、
CU-G03Rへ先取りしない。既知Edit apply失敗時のfallback修復である`GAP-24`にも混ぜない。
CU-109は公開APIや永続形式を増やさず、session内のTransientな
`Healthy / Poisoned` edit authorityを一箇所だけ所有する。`Poisoned`中はApply/Undo/Redoを
消費・再送せず、通常のproject recovery/reopen経路だけが新しい`Healthy` sessionを作れる。
Undo/Redo commandの取得はCU-111が非公開のtyped prepared-action境界として閉じ、
raw stack / raw writer / 汎用peekを公開しない。

## 5. 正例

1. accepted Rectangle Place:
   `preflight(AddTrackItem)` → journal commit → live `apply_macro([AddTrackItem])`
   → revision +1 → selection reconcile → publish 1。
2. Undo:
   top macroが1 commandであることをpreflightし、そのinverse `RemoveTrackItem`をjournalへcommit
   → live `undo` → revision +1 → dangling primary clear → publish 1。
3. Redo:
   同じforward `AddTrackItem`をjournalへcommit → live `redo` → revision +1
   → selection reconcile → publish 1。
4. Place後にUndoしてcrash/reopenすると、journalはAdd→Removeの順にreplayされ、
   Rectangleが無いDocument意味へ戻る。live Undo履歴は復元しない。

## 6. 必須負例

- journal error後にlive apply、revision増加、selection変更、成功snapshot publishを行わない。
- durable commit後のlive失敗を通常のretry可能errorとして返さない。
- UI retry、duplicate terminal、stale terminalから同じcommandを二重commitしない。
- Apply後にjournalを追記して、journal失敗時にlive rollbackで取り繕わない。
- selection、drag epoch、gesture ID、projection generationをjournalまたはDocumentへ保存しない。
- Undoをliveだけで確定して、reopen時に取り消した編集を復活させない。
- 2件以上のcommandを複数`JournalEdit`へ分け、1 gestureの原子性を名乗らない。
- main保存、checkpoint、Save UX、journal rotationをCU-G03の編集commitと同義にしない。
- CU-G03Rの必須負例: checkpoint後のEdit+Commitをjournalへ残し、disk catalogだけを
  checkpoint直後へ戻してreopenする。`RecoverySource::MainFile`を返さず、既存replay後の
  Documentがcommitted Edit適用済みで、main原本が不変であること。純checkpoint後の対照は
  従来どおり`MainFile`を返すこと。

## 7. 非目標と後続境界

- `CU-109`: 本順序のProjectSession / journal / poison / publish配線。
- `CU-G03R`: catalog未反映のcommitted Edit tailをmain fast pathから除外し、既存replayへ
  送るM2 recovery guard。新しい永続形式、catalog repair、truncate/dispose、GAP-24の
  apply-failure fallback修復、session poison、open後のwrite拒否は含めない。
  `CU-109`はこの完了を待つ。
- `CU-110`: Rectangle Placeのnon-live command prepareとaccepted terminal接続。
- `CU-111`: 製品Undo/Redo `CommandId`と、単一command top macroを非公開typed
  prepared actionへ変換する境界・配送。
- `CU-104`: primary selectionの具体policyと三面再投影。
- `CU-G09`、`CU-101`、`CU-102`: Browser projection、Rectangle意味、identity裁定。
- `CU-G04` / `CU-306S`: checkpoint、Save/reopen製品UX。
- project main / generation baseの初期作成はCU-G04側の既存project lifecycleを前提とし、
  CU-109はbase不在のsessionへedit-only commitしない。
- 複数command macroの耐久形式、再起動後Undo履歴、journal v3は別のM2仕様判断。

## 8. STOP条件

次のどれかが必要になった時点でCU-G03の既決範囲から出る。

1. 1 accepted actionを2件以上の`Command`で耐久化する
2. `JournalEdit` / `Command` / Document schema / min-readerを変更する
3. public raw writer、public journal commit、UI所有journal writerを追加する
4. post-durable failureをpoison/recovery以外で自動修復・retryする
5. Undo履歴を再起動後にも復元する
6. selectionまたはUI event列を永続化する

## 9. 完了とhandoff

親`CU-G03`、決定子`CU-G03D`、実装子`CU-G03R`は`DONE`。
CU-G03Rは[#369](https://github.com/oshikaidesu/Motolii/pull/369)で既存replay guardと
stale-catalog負例を閉じた。`CU-109`は本決定を直接authorityにできるが、
Undo/Redo prepared-action順序を再確認するまで自動着手せず、他のVS-1 blocking decisionと
同じ粒へ束ねない。次のPRODUCT-ASSET粒は`CU-101`（Rectangle Place意味決定）とする。
