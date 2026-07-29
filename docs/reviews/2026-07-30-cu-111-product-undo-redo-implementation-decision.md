# CU-111 製品Undo/Redo配送 実装決定

- 日付: 2026-07-30
- 状態: **決定**
- CU-111: **DONE**

## 1. 成立したproduction経路

通常製品HostがParent focusを持つ間だけAppKit local monitorで
`Command+Z` / `Command+Shift+Z`の非repeat key downを観測し、toolkit非依存の
`EffectiveTrigger`へ変換する。既存keymap resolverと`InputRouter`を通して
安定`CommandId` `motolii.edit.undo` / `motolii.edit.redo`へ解決し、
既存`DocumentEditQueue` / `DocumentEditRuntime::process_next`へ配送する。
Browser Web focus中のnative key eventは消費しない。

Undoは直前forward commandのinverseを、Redoは同じforward commandをjournalへ先にcommitし、
Apply / Placeと同じprivate durability helperを通した後だけ
`DocumentWriter::undo` / `redo`を実行する。成功したcurrent Documentを一度採用し、
selection reconcile後の同じsnapshot / primary / projection generationを
Stage / Timeline / Inspectorへ再投影する。

## 2. private prepared-action projection

`motolii-doc`の公開stack peek APIやraw writerを追加せず、
UI edit runtime内だけに単一commandのforward `Command`を保持する
非正本`HistoryProjection`を置いた。通常製品runtimeだけがwriterを所有し、
Apply / Place成功後とUndo / Redo成功後にprojectionを更新する。
prepare時には`DocumentWriter::undo_len` / `redo_len`との完全一致を必須とし、
外部変異または不整合はtyped rejectしてjournal、Document、publishを0にする。
project再open時のUndo履歴復元は行わない。

## 3. 証拠

```text
cargo test -p motolii-ui --test cu111_product_undo_redo -- --test-threads=1
cargo test -p motolii-ui --all-targets -- --test-threads=1
cargo clippy -p motolii-ui --all-targets -- -D warnings
cargo test --workspace -- --test-threads=1
```

unit testはApply→Undo→Redoのrevision / generation増加、Undo時selection clear、
Redo時selection非復元、journal再open後の同値を固定する。負例はempty history、
projection / writer不一致、generation枯渇、journal失敗、post-durable poisonを固定する。
production reachability guardはHost focus gate、stable CommandId、既存resolver / router /
queue / runtime、Apply / Place / Undo / Redo共通durability helper、
公開stack API不在を固定する。

## 4. 非目標

- restartをまたぐUndo履歴、複数command macro、履歴UI。
- React内のUndo正本、surface別history、toolbar。
- user keymap設定UI、IME一般処理、recovery UX。
- Document / serde / plugin契約、公開prepared-action API。
- visual threshold、golden、既存試験期待値の変更。

## 5. 次

`CU-111`は`DONE`。次の唯一のPRODUCT-ASSET `DO`は、
同じRectangle / revision / `LayerId`が三面へ現れ、Undoで消え、
Redoで同じIDで復帰することを通常製品routeで閉じるE2E `CU-108`。
token後続は`WAIT`を維持する。
