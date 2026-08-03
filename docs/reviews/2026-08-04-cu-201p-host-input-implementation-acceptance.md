# CU-201P-HOST-INPUT 実装受入と後続解禁

- 日付: 2026-08-04
- 実装: `f79625b86cc683d4f154d060481ca7e5f71baae8`
- 状態: **IMPLEMENTED / REVIEW ACCEPT / EXTERNAL GATES PARTIAL / HUMAN DEFERRED**
- 正本: [HOST-INPUT背骨決定](2026-08-04-cu-201p-host-input-spine-decision.md)

## 1. 受入結果

`product_runtime_adapter.rs`だけをraw winit ownerとし、cursor、primary pointer、focus/pointer loss、
modifiers、logical Escape、IMEを既存typed入力へ畳んだ。`ProductApp`は既存`InputRouter`へ
DragStart / DragEnd / Cancel command / SafetyInterruptを送り、cancelはTimeline Transientとpreviewだけを
破棄する。product builtin keymapはversion 2へ上がり、modifierなしEscapeを既存
`motolii.gesture.cancel`へ割り当てた。wire codec v1、Document、journal、public API、AppKit history routeは不変である。

fresh `claude-opus-5`のread-only reviewはprovider-native `stream-json`を保存・実行中観測して実施した。
第一waveは`ACCEPT / P0 NONE / P1 NONE / MUTATION NONE`、追加原文を与えたfresh closure waveは
`CLOSED / P0 NONE / P1 NONE / EVIDENCE_GAP NONE / MUTATION NONE`。開始前後でworktreeはclean、
HEADと3 allowlist fileのindex objectは不変だった。raw logは一時evidence treeに保存し、採否は本記録と
repository / Mac oracleを主担当が再照合した。

## 2. oracle

- `raw_input_boundary` 5/5、adapter inline 3/3、InputRouter、product builtin Escape、fmt、clippy、
  `git diff --check`はPASS。
- 通常Mac製品windowでactive move中のlogical Escapeとfocus lossはcancel traceへ到達し、
  project / journal write 0。
- Computer Useで左右のwindow外へdragした試行はwindow外座標のmove eventまでは観測したが、
  providerが`CursorLeft`を生成せず、pointer-loss実機gateをPASSとは数えない。fixture本体hashは不変。
- full package / local validationのredは、変更前から存在するTimeline source scanner期待値と
  protected-assets policyであり、本粒へtest/oracle変更を持ち込まない。

## 3. 状態遷移

- `P01-C1`: `DONE`。単一ApplicationHandler / run_appとadapter外raw型0を再締結。
- `CU-201P-HOST-INPUT`: `IMPLEMENTED / REVIEW ACCEPT / EXTERNAL_POINTER_GATE_PENDING / HUMAN_DEFERRED`。
- `CU-201P-MOVE`: `IMPLEMENTED / RECLOSED / EXTERNAL_POINTER_GATE_PENDING / HUMAN_DEFERRED`。
- `CU-201P-TRIM`: `DO`。保持WIPをfresh local main系へrebaseして再開できる。
- `CU-201P`: 残余targetは`SPLIT / WAIT_TARGET`のまま。

pointer-lossと通常Undo/Redoの残りは`CU-201E`の自動製品laneへ送り、各粒のユーザー目視は要求しない。
ユーザー目視項目は粒の規模にかかわらずM3全体の最終HUMAN checklistへ集約し、技術実装の後続を止めない。
ただし未実施gateをPASSやM3完成へ繰り上げない。

## 4. 後続の一本道

```text
TRIM implementation
  -> CU-201R random move/trim sequence oracle
  -> CU-201E normal product move/trim/reopen E2E + remaining external gates
  -> M3-final HUMAN checklist
```
