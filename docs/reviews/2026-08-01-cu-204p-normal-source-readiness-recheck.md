# CU-204P normal source readiness 再確認

- 日付: 2026-08-01
- 状態: **RECHECK DONE / CU-204P WAIT**
- 親: CU-204 / U2c-5

## 1. 結論

rolling HEAD `99bdc3af`で再確認しても、現行5 `DiagnosticReasonCode`へ到達する
通常製品操作は0件である。CU-204Pの表示配線を開始せず、`WAIT`を維持する。

同じadapter、同じtest-only source、同じdiagnostic-only injectionを再提出しない。次回再開には、
別途認可・実装された通常製品操作が既存5 reasonの一つを自然に生成する新しいコード事実が必要である。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| `AUTHORITY` | [CU-204S](2026-07-31-cu-204-staged-diagnostic-projection-split-decision.md)、[Diagnostic Envelope契約](2026-07-21-m3-u2c-4-diagnostic-envelope-contract.md) |
| `INTERNAL TARGET` | `DiagnosticEnvelope`、`project_diagnostic`、`InspectorHostRuntime`、product `Feedback` |
| `OWNER` | 将来の診断はHost Transient単一slot。Reactはlocal presentationだけ |
| `WRITE ROUTE` | candidate 0件なのでDocument writeなし。Host snapshotのread-only投影だけ |
| `GAP` | production source call 0、Host slot 0、snapshot field 0、decoder field 0、Feedback consumer 0 |
| `RESOLUTION ROUTE` | sourceを捏造せず`WAIT`。実在operation成立後に同じprivate Host seamを`REUSE` |
| `DISPOSITION` | `DROP` current implementation attempt / `KEEP` parent task |

## 3. 5 reasonの到達性

| reason | 現行source | 通常製品到達性 |
|---|---|---|
| `UnknownCommand` | `adapt_input_router_error(InputRouterError::UnknownCommandId)` | 0。通常keymapは登録済みcommandだけ。不明commandはproduct runtimeでfatalであり、表示へ変えない |
| `EmptyDocumentCommands` | `DocumentCommandRequest::try_new` | 0。通常product runtimeは同requestを構築しない |
| `NonDocumentIntent` | 同上 | 0 |
| `DocumentCommandKindMismatch` | 同上 | 0 |
| `EffectDefinitionInUse` | `DeleteEffectDefinition`適用時の`CommandError::DefinitionInUse` | 0。Core commandはあるが通常製品のDelete Definition操作が無い |

adapterと`project_diagnostic`のproduction runtime callは0件。残る参照は定義、re-export、
compile-time signature assertion、testに限られる。
empty Undo/Redo、Inspector IPC rejection、stale gesture、Browser rejectionを上記reasonへ読み替えない。

## 4. consumer gap

現行Inspector snapshotはrevision、document、nodes、target、optional active Effectだけを持つ。
exact JS decoderはこのkey集合外を拒否し、`InspectorCandidate` propsにもdiagnosticは無い。
product `Feedback`は既存componentとして存在するが、通常Inspector branchのconsumerではない。

したがって表示開始、replacement、clearは未実装である。将来sourceが成立した時だけ、
一つの最新Host Transient slotへ`Some`をpublishし、次の拒否で原子的に置換、次の関連成功または
source消滅publishで同じsnapshotから`None`を送る。このlifetime案はsource成立時にコード事実へ
再照合し、今は実装契約として発効させない。

## 5. STOP

- unknown commandまたはdiagnostic-only routeを注入する
- fatal errorを無断でnonfatal UIへ変更する
- 既存reasonを別の拒否へ再利用・改名する
- 第二owner、queue、永続化、文字列解析を追加する
- candidate空のままaction/callbackを作る
- source不在のままsnapshot/decoder/Feedbackだけを接続してCU-204PをDONEにする
