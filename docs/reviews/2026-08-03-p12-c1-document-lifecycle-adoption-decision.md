# P12-C1 Desktop project lifecycle adoption decision

## 決定

- 状態: `決定`
- 粒: `P12-C1` (M3-P12-PROJECT)
- 仕様方針: Motoliiの既存project/lifecycle基盤の既知意味論を採択し、文書保存・再読込・ロック・閲覧権限のみを接続する。
  新規document-framework、dirtyフラグ、saved_revision/last_saved/unsaved prompt、Save-as=Export経路は採用しない。
- 結果: この粒は `SPEC_ONLY` とし、実装は未承認。

## 先例（official links）

- Apple Final Cut Pro 継続自動保存（連続保存の意味論参照）
  - https://support.apple.com/en-mk/guide/final-cut-pro/ver79aa3d71/mac
- AppKit `NSDocument` のセマンティクス
  - https://developer.apple.com/documentation/appkit/nsdocument
- `NSDocument` の保存操作種別（Save / Save As / Save To）
  - https://developer.apple.com/documentation/appkit/nsdocument/saveoperationtype

これらはOS実装の参考として採択範囲を固定する用途であり、macOS実装をそのまま採用したり、product architecture を移植しない。

## 既知コード事実

- `crates/motolii-ui/src/document_edit_runtime.rs` は publish前に `ProjectSession::save_with_journal(..., checkpoint false)` を同期実行し、journal失敗時はRuntimeをpoisonしてpublishを失敗扱いとする。
- `crates/motolii-ui/src/product_runtime_adapter.rs` の現行 `CloseRequested` は即時終了であり、close完了の順序・in-flight writeの失敗投影は未接続。
- `crates/motolii-ui/src/shell.rs` は ProjectSession を開くが `OpenMode` を受けず、接続層での差分admissionは未接続。
- `crates/motolii-doc/src/journal/recover.rs` は `OpenedDocument.document` を復元して main に適用しつつ、`open_mode` を破棄している。
- `crates/motolii-doc/src/journal/project.rs` の edit-only commitとcheckpointはいずれもpersistの `OpenMode` ガードを呼ばず、checkpointは `journal/wal.rs` からmainを直接置換する。
- `crates/motolii-doc/src/persist.rs` は `ReadWrite / ReadOnlyNewer / Reject` を既存分類し、persist直系の保存／移行では`ReadOnlyNewer`をtyped拒否する。ただしjournal経路へこの保護は未接続。
- `ProjectSession` は固定 `document_path` と排他的lockを保持し、保存先移譲の公開APIはない。
- Motolii UI 側に dirty/saved_revision/last_saved/Save-As target といった状態はない。

## 採択マッピング

### 1) Motolii現行 + 既知実装対応

| Motoliiの接続点 | 採択対象意味論 | 接続方向 |
|---|---|---|
| Edit durability / publish durability | publish前のjournal write成功を編集受理の耐久条件として扱う | `ProjectSession::save_with_journal` を通る既存publish経路を採用 |
| New / Open / reopen | open成功後はmain fileと成功済みjournal tailから回復し、close後の再openでも受理済み編集を失わない | catalog/session/recover/journal読込を既存routeへ維持 |
| Save（手動） | コンパクト化/チェックポイントのみ。初回 durabilityを代替しない | 現行 `checkpoint` 系保存ルートを保存操作として明示 |
| ReadOnlyNewer | 閲覧モードのみ。writable編集routeへ入れない | `OpenMode`の分類を回復・session・product runtimeへ伝播し、journal edit/checkpointへ到達させない |
| Save As | 新しいlocationとidentityを持つprojectへの移譲transactionとして扱う | destination作成・lock/session取得がすべて成功した後だけownershipを切り替え、失敗/取消では元projectを維持 |

### 2) 採択外

- Unsaved判定ダイアログ、dirtyフラグ、`saved_revision`・`last_saved`の新規導入。
- 既知保存成功をUI-ownedな「未保存」画面で吸収する挙動。
- raw path open、lock steal、保存とExportの同一経路化。

## 接続チケット（AUTHORITY / INTERNAL TARGET / OWNER / WRITE ROUTE / GAP / RESOLUTION ROUTE / DISPOSITION）

| AUTHORITY | INTERNAL TARGET | OWNER | WRITE ROUTE | GAP | RESOLUTION ROUTE | DISPOSITION |
|---|---|---|---|---|---|---|
| `crates/motolii-doc/src/persist.rs` | `OpenMode`（`ReadWrite / ReadOnlyNewer / Reject`） | persist compatibility判定 / product runtime admission | `load_document*`の`OpenedDocument`から編集runtime入場まで | `ProjectSession::open`の回復結果が`OpenMode`を保持せず、shellまで伝播しない | 回復・session・product runtime境界で既存`OpenMode`を失わず渡す実装粒を別途閉じる | `SPEC_ONLY` |
| `crates/motolii-doc/src/journal/recover.rs` / `journal/project.rs` / `journal/wal.rs` | catalog回復 path / journal replay / checkpoint | `ProjectSession` | `recover`→journal replay / checkpoint経路を既存で維持 | main読込成功時も`OpenedDocument.open_mode`を破棄し、journal edit/checkpointにもwritable guardがない | 回復結果へ既存`OpenMode`を再接続し、journal write入場を閉じる実装粒を別途定義する | `SPEC_ONLY` |
| `crates/motolii-ui/src/document_edit_runtime.rs` | published snapshot durability | `PublishedDocument` publish runtime | `save_with_journal`（checkpoint false） | close/publishの失敗投影と入退出 ordering が未接続 | close ordering と poisoned failure policyを後続仕様粒で閉じる | `SPEC_ONLY` |
| `crates/motolii-ui/src/product_runtime_adapter.rs` / `shell.rs` / catalog lock | window close / reopen / save-as path transfer | `ProductApp` runtime owner | DocumentEditRuntime の既存経路 | close時に`Write`の終端可視化が未接続、Save Asのpath transfer未実装 | in-flight/pending write and lock/session handoff policyを明示 | `SPEC_ONLY` |
| `crates/motolii-ui/src/product_runtime_adapter.rs` + `rfd` | OS dialog + 事前検証 | `rfd` 採否層（別途） | Save-As destination picker / folder admit | rfd接続、cancel/failureのtyped surface投影が未接続 | `P06-C1`後に評価し、rfdは必要最小で接続 | `ADOPTION_PROBE` |

## 決定表

| 項目 | 判定 | 適用 |
|---|---|---|
| 健全な writable project を publish 成功させた後、close前に unsaved 提示が必要か | 不要 | 既存 journal durability と reopen replay で再開可 |
| Manual Save を最初の durability として扱うか | 不要 | Save は compaction/checkpointのみ |
| ReadOnlyNewer を writable の save 経路へ通すか | 否 | 閲覧モードに閉じる |
| Save As を Export に混同するか | 否 | Save As は別identity移譲 transaction（未実装） |
| P12-C1 で実装を許可するか | 否 | `SPEC_ONLY` のみ |

## negative oracles

- `dirty` / `saved_revision` / `last_saved` / untracked unsaved prompt を導入してはいけない。
- `ReadOnlyNewer` を writable 扱いしない。
- `CloseRequested` を成功条件扱いして未確定writeを黙示的成功にしない。
- Save As と Export を同一routeに統合しない。
- 既存 `ProjectSession` の固定lock/pathを無視し、raw path openやlock stealを導入しない。
- 既知OS実装をそのまま Motolii product architecture に転写しない。

## STOP / non-goals

### STOP

- rfd 採択と同時の実装接続
- `P12-C1` で新規framework/新規child-idを定義すること
- UI-owned dirty/saved_revision/last_saved stateの追加

### non-goals

- 新規dirty/saved_revision状態の実装
- product-codeの変更
- 新しいpublic API、schema、journalフォーマット変更
- Document新規runtimeの追加、依存crate追加、react component変更
- M3 DOの新設実装化

## connection residual

- `ReadOnlyNewer` の閲覧入場とjournal edit/checkpoint拒否を決定的に接続すること
- close ordering / in-flight 書込失敗 projection
- Save As destination 選択→同一family atomically create/copy→lock/session移譲のexact経路
- rfd接続（キャンセル・失敗・エラー投影）

## non-goals（再掲）

- "実装完了"を意味する既存粒子への接続（この grainは `SPEC_ONLY`）
- UI-owned unsavedフラグや再読み込み時の保存同値性回避
- Save Asを export 経路へ寄せる設計
