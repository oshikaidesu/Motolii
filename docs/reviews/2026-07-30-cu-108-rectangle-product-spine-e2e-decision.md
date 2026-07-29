# CU-108 Rectangle通常製品spine E2E決定

- 日付: 2026-07-30
- 状態: **決定**
- CU-108: **DONE**
- VS-1: **DONE**

## 1. 実機経路

MacBook内蔵画面、表示100%、暗い室内で、通常製品windowだけを使って次を実行した。

1. React BrowserのCreateにある製品Rectangleをnative Stageへdrag & drop。
2. Stageに白Rectangle、native Timelineに追加bar、React Inspectorに
   `Rectangle / Clip`が同じpublish後に現れることを確認。
3. native Hostへfocusを移し、`Command+Z`を実行。
4. StageのRectangleとTimelineの追加barが消え、dangling primary reconcileにより
   Inspectorも空になることを確認。
5. `Command+Shift+Z`を実行。
6. StageのRectangleとTimelineのbarが復帰することを確認。
   Inspectorは[CU-104](2026-07-27-cu-104-selection-publish-envelope-decision.md)
   §6のRedo non-restorationどおりprimaryを復元せず、同じRedo snapshot / `primary = None`
   を消費する。

diagnostic route、fixture-only Rectangle、React Timeline、別rendererは起動していない。

## 2. 実機で検出した阻害と修正

最初の実機試行ではUndoだけが成立し、Redoが発火しなかった。
AppKitの`charactersIgnoringModifiers`はShift付きZを大文字`"Z"`として返し得るが、
Host shortcut境界が小文字`"z"`との完全一致だけを許していたことが原因だった。
physical keyの意味を変えずASCII case-insensitiveな一文字Z判定へ修正し、
`z` / `Z`受理、`x` / `zz`拒否をunit testへ固定した。

公開input契約、stable `CommandId`、keymap、Document、journal、plugin契約、
Undo/Redo ownershipは変更していない。

## 3. identity / durability証拠

実機sessionのjournalは次の3 commandをこの順で持つ。

```text
AddTrackItem     layer_id=2
RemoveTrackItem  layer_id=2
AddTrackItem     layer_id=2
```

Place、Undo inverse、Redo forwardのすべてが同じ`LayerId(2)`を使う。
各成功actionはCU-111で固定した単一publishを通り、Stage / Timeline / Inspectorは
surface別Document正本やrevisionを持たず、同じadopted snapshot / primary /
projection generationを消費する。自動試験ではPlace / Undo / Redoのwriter revisionを
`1 / 2 / 3`、projection generationを`1 / 2 / 3`として固定する。

## 4. 自動試験

```text
cargo test -p motolii-ui \
  host_pointer_capture::tests::shifted_history_key_remains_the_same_physical_shortcut \
  -- --test-threads=1
cargo clippy -p motolii-ui --all-targets -- -D warnings
cargo test --workspace -- --test-threads=1
./scripts/check-docs.sh
```

## 5. 完了と停止線

`Browser → typed intent → Host → Place → Stage / Timeline / Inspector → Undo / Redo`
の通常製品spineとVS-1は`DONE`。

本粒ではtoken後続、essential focus一般化、selection chrome、履歴UI、
再起動をまたぐUndo、別surface、公開APIを追加しない。
次のPRODUCT-ASSET `DO`は本決定から自動選定せず、spine完了後の並列lane選定へ戻す。
