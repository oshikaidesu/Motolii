# M3 U2b-1 single writer配送E2E契約

作成日: 2026-07-21
状態: **決定 / 実装待ち**

## 1. 目的

U2b-1は、U0c/U2aで閉じた
`NormalizedInput → DomainIntent → DocumentCommandRequest`を製品の編集runtimeへ接続し、
既存D2のsingle writerで1 macroを適用した後の`Arc<Document>` snapshotを
UIとrender workerが読む最小E2Eを実装する。

このチケットはselection、target解決、Undo/Redoの新しいUI command、物理key adapterを
完成させるものではない。targetを持たない`DeleteTargetedItems`から削除対象を推測せず、
fixtureまたは将来の上流preflightが完成させたrequestだけを配送する。

## 2. 正本と現行コード事実

- [M3仕様 U2b](../specs/M3-ui-integration.md)は
  UI→command→writer→`Arc<Document>`購読、edit/Undo/Redo往復、
  UI状態だけの変更でserialize不変を要求する
- [U2a-1契約](2026-07-20-m3-u2a-1-command-adapter-contract.md)は
  現行Document intentを`DeleteTargetedItems`、対応D2 commandを
  `RemoveTrackItem`に限定し、selection/target/indexの推測を禁止している
- `DocumentCommandRequest`は完成済みcommand列を順序どおり所有し、
  `DocumentWriter::apply_macro`へ渡せる。request自身はwriter、gesture ID、
  toolkit型、serde面を持たない
- `DocumentWriter`は`apply_macro`、`undo`、`redo`と`snapshot() -> Arc<Document>`を
  既に持つ。Document schema、journal、Undo形式の追加は不要である
- 現行shellはbootstrapの`Arc<Document>`をrender requestへ渡すが、
  製品中の編集ownerと編集後snapshotの配送口をまだ持たない
- 現行入力adapterには安定command
  `motolii.edit.delete_targeted_items`がある。一方、製品selectionと
  preflightは未実装であり、通常起動時に完成済みrequestを供給する主体はまだ無い

## 3. U2b-1で固定する所有と配送

### 3.1 event-loop-local edit runtime

`motolii-ui`内にprivateな`DocumentEditRuntime`を置き、唯一の
`DocumentWriter`を所有させる。初回U2b-1では編集適用を別workerへ移さず、
egui callbackが終わった後の`App::logic`先頭で、FIFOの確定済みactionを1件ずつ処理する。

理由は次のとおり。

1. UI callbackは`DocumentCommandRequest`をqueueへ入れるだけで、
   `DocumentWriter`、`&mut Document`、gesture IDを受け取らない
2. writer呼出し箇所を`DocumentEditRuntime`へ閉じ、
   `apply_macro`、`undo`、`redo`以外の直接編集を追加しない
3. command preflight、media decode、render等の重い仕事はこのruntimeへ入れない。
   後続workerは完成済みrequestを返す読み手であり、writerを共有しない

queueはDocument editをlatest値で置換しない。確定済みgestureの順序を保ち、
同じactionを暗黙retryまたは重複適用しない。

### 3.2 actionとsnapshot

private actionは次の閉集合とする。

- `Apply(DocumentCommandRequest)`
- `Undo`
- `Redo`

`Apply`はrequestを消費し、`into_commands()`を
`DocumentWriter::apply_macro`へちょうど1回渡す。`Undo`/`Redo`は既存writer APIを
ちょうど1回呼ぶ。成功したactionごとにだけ新しい`Arc<Document>` snapshotと
writer revisionを発行する。失敗時はtyped errorをTransientに保持し、
snapshot、render generation、Document、Undo/Redoを進めない。

UIは発行済み`Arc<Document>`だけを読む。render workerへは同じsnapshotを含む
新しい`RenderRequest`を送る。UI、render worker、fixtureへ`&mut Document`または
`DocumentWriter`を渡さない。

### 3.3 normalized eventとprepared requestの接続

E2Eの編集入口は、既存builtin command IDを持つ`NormalizedInput::Command`を
`InputRouter`へ通した`RouterOutput::Intent`とする。`Click`の
`DeleteTargetedItems`だけが、上流から同時に渡された
`DocumentCommandRequest`とintent一致を確認してqueueへ入る。

requestが無い、phaseまたはintentが違う、request.intentとrouter intentが違う場合は
変更ゼロのprivate typed rejectionとする。ここでselection、target、index、
表示名を検索・補完しない。

製品window E2Eは、このnormalized eventと完成済みfixture requestを注入して
実配送経路を通す。これは物理Delete key、selection UI、hit-testの完成を主張しない。
通常起動へfixture target、隠しbutton、固定IDによる削除を残さない。

### 3.4 Undo/Redoの範囲

U2b-1のfixtureは、同じedit runtimeへ`Apply → Undo → Redo`を順に配送し、
各成功snapshotを購読する。ただし現行`DomainIntent`/builtin registryへ
Undo/Redoを追加しない。Undo/Redoの製品command surface、shortcut、対象labelは
U2hの意味決定後に接続する。

この限定により、U2b-1は既存D2往復とsnapshot配送を実証するが、
未決のUI command意味を発明しない。

## 4. 自動審判

1. 完成済み1件の`RemoveTrackItem` fixtureをbuiltin delete commandから配送し、
   `apply_macro` 1回、revision +1、Undo 1件、Redo 0件、削除済みsnapshotを得る
2. `Undo` 1回で開始時Document serializeへ完全一致し、`Redo` 1回で
   削除後serializeへ完全一致する。各段階で新しい`Arc<Document>`を購読し、
   古いsnapshotの内容は変わらない
3. 製品window smokeでedit後snapshotがrender workerの新generationへ渡り、
   event loopがlatest resultを既存display slotへcopyする。slot ID、
   TextureId、registration countは不変
4. request欠落、intent/phase不一致、D2失敗、Undo無し、Redo無しはtyped rejection。
   Document serialize、revision、Undo/Redo、発行snapshot、render generationが不変
5. layout resize/hide/reset等のUI-only操作を編集の前後へ挟んでも、
   Document serialize、revision、Undo/Redo、発行snapshotは変わらない
6. source/AST検査で、writer所有者は`DocumentEditRuntime`だけ、
   UI callbackはqueue追加だけ、render workerは`Arc<Document>`読み手だけである。
   `motolii-ui`公開型に`DocumentWriter`、`&mut Document`、egui/eframe/winit型を増やさない
7. `cargo fmt --all -- --check`、`./scripts/check-docs.sh`、
   `./scripts/check-ui-toolkit-deps.sh`、
   `cargo clippy --workspace --all-targets -- -D warnings`、
   `cargo test --workspace`を通す

## 5. 非目標

- selection/focus/target preflight、複数targetの順序決定
- 物理Delete key、Undo/Redo shortcut、toolbar/menu、対象label
- drag途中のpreview、適用後Cancel、公開transaction lifecycle
- background writer、複数writer、requestのlatest置換、暗黙retry
- Document schema、journal/serialize、D2 command、plugin契約、公開raw APIの追加
- 診断共通component、永続diagnostic、egui memoryによるDocument複製
- Rerunその他の外部製品構造からの逆算

## 6. STOP条件

次のいずれかが必要に見えた時点で実装を止める。

- `DeleteTargetedItems`からselection、target、親、indexを推測する
- Undo/Redoの新しい`DomainIntent`、`CommandId`、keymap既定を決める
- UI callbackへwriterまたは`&mut Document`を渡す
- command適用を非同期化するためwriterを共有・複製する
- D2 errorを握り潰してsnapshotまたはrender generationを進める
- UI event、gesture、snapshot envelopeをDocument/journalへ保存する
- 既存D2 API、Document意味、公開API、永続形式、plugin契約の変更が必要になる
