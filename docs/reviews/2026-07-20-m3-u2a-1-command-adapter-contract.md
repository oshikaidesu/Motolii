# U2a-1 gesture command adapter契約

ステータス: **決定／U2a-0・U2a-1実装完了**（実装前の停止線を解消した契約）。対象は
[M3仕様 U2a](../specs/M3-ui-integration.md)と
[U2a-1](2026-07-16-m3-ui-concept-to-tickets.md)であり、D2の公開API、
Document schema、journal/serialize形式、plugin契約を置換しない。

## 1. 現行コード事実とgap

- D2には`DocumentWriter::begin_gesture`、`apply_command`、
  `GestureId + command kind + target + property`のmerge keyがあり、1 gesture=1 macroは
  実装済みである。公開`begin/update/commit/cancel` transaction APIは無い
- 現行`DomainIntent`でDocument所有なのは`DeleteTargetedItems`だけである。ただし
  selection、target、親、index、削除前snapshotを持たず、単独では
  `Command::RemoveTrackItem`を構築できない
- `EnableReduceMotion`、`ResetWorkspaceProfile`、`FitStageView`、
  `CancelInFlightGesture`はDocument所有ではなく、D2 commandへ変換しない
- 適用後Cancelの巻戻し方法とdrag途中の仮適用／overlayは未決である
- 現行`apply_command`の原子性は1 commandだけを覆う。複数commandを同じ
  `GestureId`で逐次適用して後続が失敗すると、先行分とUndo macroが残り、
  redoも失われる

したがって、payload無し`DomainIntent`だけからD2 commandを作る実装、
selection/target解決の先行実装、公開transaction APIの追加は行わない。

## 2. U2a-0: D2 atomic macroの追加契約

U2a-1より先に、D2へ**確定済みcommand列を一括適用するone-shot macro**を追加する。
これは公開gesture lifecycleやtransactionではなく、1回の呼出しで完結する
`DocumentWriter::apply_macro(Vec<Command>) -> Result<GestureId, CommandError>`とする。

1. 空列は型付き`CommandError`で拒否し、状態を一切変えない
2. 非空列には内部で新しい`GestureId`を1回だけ発行し、全commandを既存
   `UndoHistory::push`へ同じIDで渡す。merge keyとinverseを再実装しない
3. 途中の`Command::apply`またはplugin prepareが1つでも失敗したら、Document、
   Undo/Redo、revision、gesture counterを呼出し前へ戻し、元のtyped errorを返す
4. 全件成功時だけrevisionを1増やし、`GestureId`を返す。Undo 1回で全件を戻し、
   Redo 1回で全件を再適用する
5. command列はruntime値であり、macro、gesture、UI eventをjournal/serializeへ追加しない
6. command列は先頭から順に既存`UndoHistory::push`へ渡す。各commandのtarget、parent、
   index、itemは直前までのcommand適用後Documentに対してvalidでなければならない。
   `apply_macro`は並べ替え、index再計算、欠落補完をしない

この追加は既存`apply_command`の意味を変更しない。既存呼出しのrevision粒度、
エラー、Undo/Redoを保ち、U2a-0だけを独立した1チケット・1コミットで実装する。

## 3. U2a-1で固定する最小adapter境界

U2a-1は**決定済みD2 commandを伴うDocument intentを、single writerが1 macroとして
適用できるruntime-only requestへ変換する境界**に限定する。

1. UI入口は`DomainIntent`と、Document snapshotを読んだ上流preflightが構築済みの
   1個以上の`Command`をadapterへ渡す。adapterはselection、target、旧値、新値、
   index、IDを推測・補完しない。U2a-1実装と試験が供給してよいprepared commandは
   fixtureまたは呼出し側が既に完成させた値だけである。Transient selection、focus、
   表示名検索、Stage/Timeline hit-test、`layer_names_for_item`を隠したproduction
   preflight moduleは変更許可範囲外とする。複数削除のfixtureも適用順で既にvalidな
   列だけを供給し、同一parentなら高いindexから並べる等の決定をfixture側で済ませる。
   adapterは並べ替えやindex再計算をしない
2. 現行の代表操作は`DeleteTargetedItems`だけとし、同intentには
   `CommandKind::RemoveTrackItem`だけを許す。複数targetは複数の決定済み
   `RemoveTrackItem`を1 requestへ入れる
3. adapterの成果物はDocument、journal、keymapへserializeしないruntime-only値とする。
   egui/eframe/winit型、物理入力、表示名、UI座標を含めない
4. single writer側はrequestのcommand列をU2a-0の`apply_macro`へ1回だけ渡す。
   adapterはGestureIdを発行せず、D2のmerge key、Undo、typed errorを再実装・複製しない
5. adapterの公開入口とUI callbackはrequestを作るだけで`&mut Document` /
   `DocumentWriter`を持たない。U2a-1の同期fixtureでwriter適用を検証してよいが、
   製品E2Eの配送と`Arc<Document>`購読はU2b-1が所有する
6. 空command列、Document外intent、intentとcommand kindの不一致は型付きで拒否する。
   D2 no-opを合格扱いせず、未知の組合せを黙って通さない

この境界は新しい公開gesture lifecycleではない。request 1個を「確定済みの1 gesture」
として扱い、途中状態を表現しない。

## 4. Cancelの範囲

U2a-1の`Cancel変更ゼロ`は、**writerが`apply_macro`を呼び出す前**に限る。

- routerの`Cancel`、Escape、pointer capture loss、window focus lossで未確定操作を
  破棄した場合、requestをwriterへ渡さず、Document、revision、Undo/Redoを変えない
- request構築前／構築後の破棄は同じく変更ゼロ
- `apply_macro`完了後にCancelとして巻き戻すAPIは作らない

drag途中の仮適用／overlay、公開`begin/update/commit/cancel`型はU2a-1の非目標である。
適用途中の失敗はU2a-0のatomic macroが変更ゼロを保証する。後続が適用済みgestureの
Cancelを必要とした時点で、
既存D2の失敗時原子性とUndo契約を保った独立仕様を先に決める。

## 5. 自動審判

- U2a-0: 空列と先頭／中間／末尾command失敗を注入し、Document serialize、revision、
  gesture counter、Undo/Redoが呼出し前と同一。成功時は同target・同property更新が
  既存merge keyで畳まれ、異targetはmacro内の別commandになり、Undo/Redo各1回
- U2a-1: 1件と複数targetのvalid `RemoveTrackItem` requestがU2a-0経由でUndo/Redo
  各1回。別requestは別`GestureId`となり、同targetでも別Undoになる
- 空列、4種のDocument外intent、`DeleteTargetedItems`と
  `RemoveTrackItem`以外のcommandの組合せはDocument・revision・履歴を変えず型付き拒否
- Cancel／safety cancelでrequestを配送しなければDocument serialize、revision、
  Undo/Redoが不変
- adapterにegui/eframe/winit型、serde derive、独自merge planner、公開transaction型、
  `&mut Document` / `DocumentWriter`が無いことを型またはASTで検査する

## 6. STOP / 非目標

- selectionやtargetの正本が必要になったらU2hまで進めずSTOPする
- `DeleteTargetedItems`以外のDocument intent、または新しいD2 command対応は、意味と
  target payloadが決まる個別チケットで追加する
- 適用済みmacroをCancel名目でUndo履歴から消さない
- `UndoHistory`や`Command::merge_key`をUI側へ複製しない
- `motolii-doc`の公開面追加はU2a-0の`apply_macro`と空列errorだけに限定する。
  既存D2 APIの意味、Document schema、journal、serialize、plugin契約、
  既存テスト期待値を変更しない
- 部分適用を隠すための例外、lint抑制、生JSON／文字列走査、raw APIを追加しない
