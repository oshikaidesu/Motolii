# CU-110PIR Inspector safe read-only branch 実装決定

- 日付: 2026-07-29
- 状態: **決定 / DONE**
- commit: 本文と同一commit

## 1. 結論

product-owned `InspectorCandidate`本体へ、`mode`を与えずdecode済み
`inspectorReadModel`だけを与える通常製品safe branchを追加した。

表示するのは既決targetの`layer_name`、`item_kind`、groupの場合の
`child_count`だけである。既存catalog 5 branchとそのDOM / interactionを維持し、
別Inspector copy、mock state、未決S値、編集callback、Document writerを作らない。

## 2. 実装

- safe branchは`mode === undefined && inspectorReadModel !== undefined`だけで入る。
- 既存`panelHead`とtarget identity JSXをinstalled branchと共有する。
- safe branchのDOMは`aside.inspector#inspector`、panel head、identity sectionだけ。
- source provenanceは固定旧SHAから現行SHAへのappend-only chainとして記録した。
- read-model inventoryへproduct-safe branchを追加し、既存5 modeのAST coverageを維持した。

## 3. 証跡

- ownership / decoder / inventory / source-asset guard: **94 passed**
- Inspector parity oracle: **8 passed**
- `git diff --check`: pass

visual threshold、golden、catalog modeの期待値は変更していない。

## 4. 非目標と停止線

- Host / WebView / bundle / current Document配送は`CU-110PIH`。
- selection input、Inspector編集、effect UI、public wireは追加しない。
- Document、serde、journal、plugin契約、Undo/historyは変更しない。
- `docs/mocks-ui`またはlegacy runtime import、別leaf、二重stateは追加しない。

`CU-110PIR`は`DONE`。次の唯一のPRODUCT-ASSET `DO`は`CU-110PIH`。
