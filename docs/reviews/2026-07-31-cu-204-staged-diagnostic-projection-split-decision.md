# CU-204 staged diagnostic projection 分割決定

- 日付: 2026-07-31
- 状態: **決定 / CU-204S DONE**
- 親: CU-204 / U2c-5

## 1. 結論

CU-204を`S → A → P`へ分割する。

1. `CU-204S SPEC`: 現行診断、4密度、owner、通常surface、回復動線を固定する
2. `CU-204A CORE`: `DiagnosticEnvelope`からtoolkit非依存の段階投影を作る純粋adapter
3. `CU-204P PRODUCT`: 既存Inspector Host islandのprivate bridgeからproduct
   `Feedback`へ接続する

現在の5診断は`recovery_candidates()`がすべて空である。したがってCU-204では
action callbackと新しい`DomainIntent`を追加しない。回復説明は表示するが、候補が無い
状態をbutton、link、暗黙commandで偽装しない。将来、既存通常Intentを持つenvelopeが
追加された時だけ、Intent配送を独立粒で実装する。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| `AUTHORITY` | [Diagnostic Envelope契約](2026-07-21-m3-u2c-4-diagnostic-envelope-contract.md)、[UI interaction language §8.3](../ui-interaction-language.md#83-説明の段階開示)、[U2c-5](2026-07-16-m3-ui-concept-to-tickets.md) |
| `INTERNAL TARGET` | `DiagnosticEnvelope::{reason, action, subjects, facts, recoverability, recovery_candidates}`、product `Feedback`、`InspectorHostRuntime` private subscribe/publish bridge |
| `OWNER` | Host Transient projectionが診断正本を一時所有し、Reactはlocal presentationだけを所有する |
| `WRITE ROUTE` | CU-204Aはread-only純粋投影。CU-204PはHost→private bridge→decoder→Feedback。候補Intent 0件なのでwrite routeなし |
| `GAP` | `DiagnosticEnvelope`はproduct runtimeから未使用。4密度model、copy、private wire、通常surface上のconsumerが無い |
| `RESOLUTION ROUTE` | 既存Inspector Host islandとFeedbackを`REUSE`し、S/A/Pへ`REDUCE`する |
| `DISPOSITION` | `RESOLVE`をCU-204Sで閉じ、次はCU-204Aだけを`PASS` |

## 3. 現行入力集合

CU-204Aが受け取る理由は次の5件だけとする。

- `UnknownCommand`
- `EmptyDocumentCommands`
- `NonDocumentIntent`
- `DocumentCommandKindMismatch`
- `EffectDefinitionInUse`

action、subject、fact、recoverability、candidateの型と順序はenvelopeからそのまま保持する。
errorの`Display`、enumのdebug文字列、翻訳済み文から意味を逆算しない。

## 4. 4密度

全密度は同じreason、action、subjects、facts、recoverability、
recovery candidatesを保持する。違うのは表示するcopyの量だけである。

| 密度 | 表示 |
|---|---|
| `Brief` | 結果と最短の原因 |
| `Context` | Brief + 最寄りの回復説明 |
| `Inspect` | Context + typed subject/factの順序付き詳細 |
| `Assistive` | Context以上の情報を一つの順序付き完全文として読む |

`Inspect`だけにidentityを残したり、`Brief`で別reasonへ畳んだりしない。
AssistiveはDOM表示文の連結結果を読むのではなく、同じtyped projectionから生成する。

## 5. 表示copyの責任

CU-204Aは英語の製品copyを固定する。初回の固定語彙は次の意味に限定する。

| reason | result | cause |
|---|---|---|
| `UnknownCommand` | `Command unavailable` | `The command is not registered.` |
| `EmptyDocumentCommands` | `Edit not prepared` | `No document commands were prepared.` |
| `NonDocumentIntent` | `Edit not prepared` | `The requested action does not own Document state.` |
| `DocumentCommandKindMismatch` | `Edit not prepared` | `A prepared command does not match the requested document edit.` |
| `EffectDefinitionInUse` | `Effect definition is in use` | `Blocking effect uses must be removed before deleting this definition.` |

recoverabilityの説明は次へ固定する。

- `RetryWithChangedInput`: `Change the input and try again.`
- `RequiresAnotherAction`: `Complete the required action first.`
- `Unrecoverable`: `This action cannot be recovered.`

これは翻訳基盤、message ID、Help URLを決める契約ではない。

## 6. CU-204A

`crates/motolii-ui/src/diagnostic_projection.rs`に閉じる。

- `DiagnosticDensity`
- getterだけを持つ`DiagnosticProjection`
- `project_diagnostic(&DiagnosticEnvelope, DiagnosticDensity)`

公開面にserde、toolkit、DOM、callback、Document、writer、queueを出さない。
5 reason × 4 densityの表、identity/order不変、candidate空、表示量の包含関係を試験する。

## 7. CU-204P

通常surfaceは既存のright-side Inspector Host islandだけとする。別WebView、toast owner、
diagnostic-only routeを製品成果にしない。

CU-204Pは次を行う。

- `InspectorHostRuntime`のprivate snapshotへoptional transient diagnostic projectionを足す
- product-owned decoderでexact key、version、密度、reason/subject/fact順序を検証する
- 既存`InspectorCandidate`の通常read-only branch内でproduct `Feedback`を表示する
- candidateが空ならaction elementとcallbackを0件にする
- sourceが消えた次publishでは診断も消し、Document/selection/Undoへ保存しない

現行通常製品操作から、上記5 reasonのいずれかへ確実に到達するtriggerは存在しない。
`InputRouterError`は通常runtimeではfatal、`NothingToUndo/Redo`はenvelope対象外である。
したがってCU-204Pは、実在する通常sourceと表示期間・置換・clear時点がコード事実で
閉じるまで`WAIT`を維持する。diagnostic-only routeや意図的なunknown command注入で
この入場条件を代用しない。存在しないparameter preflight、Connection/Drop rejection、
effect unlinkを先に作らない。

## 8. STOP

- reason、fact、subject、recoverability、`DomainIntent`を追加する
- `DiagnosticEnvelope`または投影をDocument、journal、project serdeへ保存する
- errorの`Display`、enum名、JS表示文を解析する
- candidate空なのにbutton、callback、commandを作る
- diagnostic-only routeを通常製品接続の代わりにする
- Inspector以外の新surface、第二diagnostic owner、React側semantic storeを作る
- Diagnostic componentからDocument、writer、queueを直接変更する
