# M3 U2c-4 Transient Diagnostic Envelope契約

作成日: 2026-07-21
状態: **決定 / U2c-4実装完了**

## 1. 目的

U2c-4は、既存の領域固有errorが持つ原因構造を失わず、
Brief / Context / Inspect / Assistiveへ後で共通投影できる最小の
toolkit非依存`DiagnosticEnvelope`へ適応する。

このチケットは表示component、翻訳文、Connection/Drop rejectionを作らない。
既存error全量を一つの巨大enumへ移さず、現存する3系統の代表拒否だけで
reason、action、subjects、typed facts、recoverability、recovery candidateの境界を実証する。

## 2. 正本と現行コード事実

- [UI操作言語§8.2](../ui-interaction-language.md#82-オブジェクト自身ではなく操作境界が診断する)は、
  domain rejectionをUI境界で小さなTransient envelopeへ適応し、
  stable reason code、action kind、role付きsubject ID、typed facts、
  recoverability、recovery candidatesを保つと決定している
- [M3 U2c](../specs/M3-ui-integration.md)は、診断をserializeせず、
  Document objectへUI文言を持たせず、巨大domain error enumを作らないよう要求する
- `motolii-ui`には`InputRouterError`、`DocumentCommandRequestError`、
  privateな配送/runtime errorがある。`motolii-doc`には`CommandError`等の
  領域固有errorが既にあり、表示のために置換する必要はない
- 現行`DomainIntent`には、definitionのunlink/delete等を回復候補として提示できる
  intentがまだ無い。存在しないrecovery intentを先に追加してはならない

棚卸しした候補と初回採否は次のとおり。

| 候補 | 判定 | 理由 |
|---|---|---|
| `InputRouterError` | 採用 | UI入力境界で発生し、安定`CommandId`を保持する |
| `DocumentCommandRequestError` | 採用 | intentからD2 requestを準備する境界でexpected/actual型を保持する |
| `CommandError::DefinitionInUse` | 採用 | D2拒否で複数subject IDと別操作必要を実証できる |
| `DocumentError` | 延期 | 現行U2bのUI gesture preflightが直接返す型ではなく、variant数も多い。代表variant群とactionの対応を別契約で決める |
| plugin/render/media error | 延期 | provider固有snapshotとactionの棚卸しが未完了 |

## 3. 今回適応する3系統

| 系統 | 対象variant | 保持する意味 | 非対象 |
|---|---|---|---|
| 入力解決 | `InputRouterError::UnknownCommandId` | 試行した安定`CommandId` | 表示名、物理key、IME event列 |
| prepared request | `DocumentCommandRequestError`の現行3 variant | intent、command index、expected/actual `CommandKind`、Document所有不一致 | selection/target推測、command再生成 |
| D2 command | 実際の`CommandKind::DeleteEffectDefinition`適用から返った`CommandError::DefinitionInUse`だけ | definition ID、blocking use ID群、件数、別操作が必要 | 他action由来の同variant、他の`CommandError`全量、unlink intentの発明 |

`DefinitionInUse`は`RemoveEffect`等のcleanup経路からも返り得るため、errorだけから
試行actionを逆算しない。callerが実際に試した`CommandKind`をadapterへ渡し、
`DeleteEffectDefinition`との組だけを受理する。他の組はtypedな
`UnsupportedDiagnosticSource`として適応を拒否し、
元errorを文字列へ潰してfallback envelopeを作らない。後続対応はvariant群ごとに
reason/action/factsを決める別変更とする。

## 4. 共通型の閉集合

### 4.1 reasonとaction

`DiagnosticReasonCode`は今回の既存拒否だけに対応する次の閉集合とする。

- `UnknownCommand`
- `EmptyDocumentCommands`
- `NonDocumentIntent`
- `DocumentCommandKindMismatch`
- `EffectDefinitionInUse`

`DiagnosticActionKind`は次の閉集合とする。

- `InvokeCommand`
- `PrepareDocumentEdit`
- `DeleteEffectDefinition`

enum variant名を人間向け文として表示せず、翻訳文や文言IDはU2c-5へ残す。

### 4.2 role付きsubject

`DiagnosticSubject`は安定identityと関係上の役割を同じvariantで保持する。

- `AttemptedCommand(CommandId)`
- `EffectDefinition(u64)`
- `BlockingEffectUse(u64)`

表示名、layer名、property path、配列indexをidentityへ昇格しない。
生`u64`は現行`CommandError`が公開しているdefinition/use IDの忠実な適応に限り、
新しいDocument参照型を発明しない。

### 4.3 typed facts

`DiagnosticFact`は次の閉集合とする。

- `CommandKindMismatch { index, expected, actual }`
- `RequestedIntent(DomainIntent)`
- `StateOwnerMismatch { expected, actual }`
- `BlockingSubjectCount { count }`

`StateOwnerMismatch`は既存`DomainIntent::owner()`の結果を
`UiStateOwner`のまま保持する。errorの`Display`文字列を解析せず、
expected/actual、index、件数を文字列へ平坦化しない。

### 4.4 recoverabilityと候補

`DiagnosticRecoverability`は次の3段階とする。

- `RetryWithChangedInput`
- `RequiresAnotherAction`
- `Unrecoverable`

今回の写像は次で固定する。

| reason | recoverability | recovery candidates |
|---|---|---|
| `UnknownCommand` | `RetryWithChangedInput` | 空 |
| `EmptyDocumentCommands` | `RetryWithChangedInput` | 空 |
| `NonDocumentIntent` | `RetryWithChangedInput` | 空 |
| `DocumentCommandKindMismatch` | `RetryWithChangedInput` | 空 |
| `EffectDefinitionInUse` | `RequiresAnotherAction` | 空 |

最初の4拒否は入力を正して同じactionを再試行できるため、回復不能へ潰さない。
`EffectDefinitionInUse`は別操作が必要だが、
該当する既存`DomainIntent`が無いためcandidateを捏造しない。

## 5. Envelopeとadapter境界

`DiagnosticEnvelope`はreason、action、subjects、facts、recoverability、
recovery candidatesを所有する。順序はadapterが決定し、subjects/factsを
set化または表示順で並べ替えない。

reasonごとの完全写像は次である。`[]`は空列を表す。

| reason | action | subjects（順序固定） | facts（順序固定） | recoverability | candidates |
|---|---|---|---|---|---|
| `UnknownCommand` | `InvokeCommand` | `[AttemptedCommand(id)]` | `[]` | `RetryWithChangedInput` | `[]` |
| `EmptyDocumentCommands` | `PrepareDocumentEdit` | `[]` | `[]` | `RetryWithChangedInput` | `[]` |
| `NonDocumentIntent` | `PrepareDocumentEdit` | `[]` | `[RequestedIntent(intent), StateOwnerMismatch { expected: Document, actual: intent.owner() }]` | `RetryWithChangedInput` | `[]` |
| `DocumentCommandKindMismatch` | `PrepareDocumentEdit` | `[]` | `[RequestedIntent(intent), CommandKindMismatch { index, expected, actual }]` | `RetryWithChangedInput` | `[]` |
| `EffectDefinitionInUse` | `DeleteEffectDefinition` | `[EffectDefinition(id), BlockingEffectUse(use_ids[0]), ...]` | `[BlockingSubjectCount { count: use_ids.len() }]` | `RequiresAnotherAction` | `[]` |

`RequestedIntent`はobject IDではなくtyped factであり、subjectsへ混ぜない。

adapterは領域別の独立関数とする。

- `adapt_input_router_error(&InputRouterError) -> DiagnosticEnvelope`
- `adapt_document_command_request_error(&DocumentCommandRequestError) -> DiagnosticEnvelope`
- `adapt_command_error(CommandKind, &CommandError) -> Result<DiagnosticEnvelope, UnsupportedDiagnosticSource>`

envelopeは元error、`std::error::Error` trait object、表示文字列、UI callback、
Document snapshot、toolkit型を所有しない。adapterはDocument、writer、history、
render generationを変更しない。

`DiagnosticEnvelope`のfieldとconstructorはprivateにし、本番の公開組立口は上記adapterだけとする。
reason、action、subjects、facts、recoverability、candidatesは副作用のないread-only getterで
参照でき、列はsliceとして返す。可変参照やparts差替えAPIは出さない。
将来の領域固有rejectionが巨大共通error enumへ統合せず参加できることは、
module内test-onlyな`FutureCommandLookupRejection`と専用adapterで固定する。
fixtureは既存`UnknownCommand`と同じ意味を持つ別のerror型から同じ完全写像を得るが、
本番公開constructorや共通`DomainRejection` enumを追加しない。

`UnsupportedDiagnosticSource`は`UnsupportedCommandError { action: CommandKind }`だけを持つ。
元errorは借用したcallerが保持し、Errへvariant名や`Display`文字列を複製しない。

## 6. 自動審判

1. 3系統の全対象variantをadapterへ通し、reason/action/subjects/facts/
   recoverability/candidatesが上表と完全一致する
2. `DefinitionInUse`のdefinition IDと全blocking use IDを順序どおりsubjectsへ残し、
   `BlockingSubjectCount`が一致する
3. 非対象`CommandError`は`UnsupportedDiagnosticSource`となり、
   genericな「失敗」envelopeへfallbackしない
4. test-only future rejection型がmodule-private constructorを専用adapterから使い、
   本番公開constructorと共通巨大error enumを追加せず同じ完全写像を得る
5. source/trait検査でenvelopeと全構成型にserde実装、toolkit型、Document、
   writer、UI文言、callbackが無い。adapterはerrorの`to_string()`/`Display`結果を解析しない
6. 公開adapter署名がerror参照と必要な`CommandKind`以外にDocument、writer、queue、
   render clientを受け取らない。source検査と既存snapshotを囲むfixtureで、
   adapter前後のDocument serialize、revision、Undo/Redo、render generation不変
7. `cargo fmt --all -- --check`、`./scripts/check-docs.sh`、
   `./scripts/check-ui-toolkit-deps.sh`、
   `cargo clippy --workspace --all-targets -- -D warnings`、
   `cargo test --workspace`を通す

## 7. 非目標

- Brief / Context / Inspect / Assistiveの表示modelとcomponent（U2c-5）
- target highlight、cursor説明、semantic badge、色、icon、focus state（U2c-3）
- `ConnectionRejection`、`DropRejection`、selection、target preflight
- 全`CommandError`、`DocumentError`、plugin/render/media errorの一括適応
- 翻訳文、表示文言ID、Help URL、外部検索
- recovery Intent、command、callbackの新設または自動実行
- Document schema、journal、Undo、cache key、plugin契約への診断保存

## 8. STOP条件

次のいずれかが必要に見えた時点で実装を止める。

- 領域固有errorを共通error enumへ移す、または全errorを一括matchする
- errorの`Display`文字列を解析してreason/subject/factを作る
- 表示名、layer名、property path、物理入力、button名をidentityへ使う
- 存在しないConnection/Drop/recovery Intentを先に発明する
- 診断からDocument、writer、render worker、UI callbackを直接変更する
- envelope、UI event、診断文をserializeする
- egui/eframe/winit型、px/DPI、表示文言を共通型へ入れる
- 既存error variant、D2 API、Document意味、plugin契約、永続形式の変更が必要になる
