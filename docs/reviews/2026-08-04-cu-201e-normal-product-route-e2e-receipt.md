# CU-201E normal product move trim reopen E2E receipt

- 日付: 2026-08-04
- 状態: **DONE / PRODUCT_E2E_PASS / EXTERNAL_POINTER_GATE_PENDING / HUMAN_DEFERRED**
- 対象HEAD: `15e8201dbab2288b4d1ed2de5ea3fc16129cd18b`
- 対象binary: `/private/tmp/MotoliiCU201E.app/Contents/MacOS/motolii_ui_shell`
- executable SHA-256: `296eaf318780566b497fb34ffa73e53b8d3af7c070910a00225f519848786857`
- wrapper bundle id: `com.motolii.cu201e`（既存`motolii_ui_shell`をそのまま起動する一時wrapper）
- fixture project path: `/private/tmp/motolii-cu201e-run-20260804.json`
- 初期project SHA-256: `ef9946ca2a65b6c9ffc0a05124464dbeccd6579bfa02a3901ffadea77998569b`
- 初期 interval: LayerId `0`, start `1/1`, duration `6/1`, source_start `0/1`

## 1. 実行列

通常のnativeトップレベル製品windowで、次の順序を実行した。

1. body drag MOVE
2. 左端 TRIM
3. `Command+Z` ×2
4. `Command+Shift+Z` ×2
5. window close
6. 同じprojectを通常windowで再起動

上記は `replay` で以下のWALとして再現される。

- MOVE WAL: `SetClipStart target=0 old=1/1 new=100251589/50000000`
- TRIM WAL: `TrimClipIn target=0 old_start=100251589/50000000 old_duration=6/1 old_source_start=0/1 new_start=49922537/20000000 new_duration=550890493/100000000 new_source_start=49109507/100000000`
- Undo WAL append: exact reverse TRIM first, then exact reverse MOVE
- Redo WAL append: exact MOVE first, then exact TRIM

## 2. reopen receipt

再起動後、通常windowには1本のTimeline bar（`CU-201E target`）が表示され、左/右エッジがpost-Redo TRIM位置に一致した。

Inspectorは未選択（empty）状態で、selection / dragの状態がDocumentへ永続化されていないことを確認した。

## 3. pointer-loss probe（分離記録）

再起動後の最終intervalで、Computer Use起動点 `(650,650)` から外部 ` (1300,650)` へのドラッグを実施。

- 観測イベント: `kind=timeline-move state=begin layer=0 generation=0` を取得
- 終端観測: `kind=timeline-hit layout_epoch=4 logical_x=1401.562 logical_y=672.781 hit=None`
- 欠落: `state=cancel ... reason=window-focus-or-capture-loss` を観測できず

最終WAL path: `/private/tmp/motolii-cu201e-run-20260804.json.motolii/journal.wal`（`2577` bytes, SHA-256 `47c4d147d055f59388698b5930beebd07c2dff3acc53098e7761108a86893333`）で内容変更なくプロセス終了。Provider起点の `CursorLeft` 不足のため、このgateは未PASS。

再起動時のスクリーンショット（SHA-256 `e01d135c767ac29f0e82acebf7869fc4036901745db0d7f1e1ba7adb0ace1e1f`）は一時pathのみで非権威。

## 4. 既知実装採択サマリ

- MECHANISM CLASS: normal product native-window E2E receipt
- KNOWN IMPLEMENTATION SEARCH/CANDIDATES: CU-205E / CU-108、既存`motolii_ui_shell`、ProjectSession/WAL、keymap、native Timeline
- ADOPTION ROUTE: `REUSE / REDUCE`（temporary `.app` wrapper + normal product runtime + adjacent WAL + machine observation）
- REJECTED: headless-only 完了、new E2E framework、synthetic pointer-loss、custom event injection
- THIN MOTOLII SEAM/RESIDUAL: receipt and ledger status only。pointer-loss は外部ゲート、最終HUMANは最終チェックへ残置
- BUILD JUSTIFICATION: NONE
- BUILD: FORBIDDEN

## 5. 状態遷移

- `CU-201E`: `DO -> DONE / PRODUCT_E2E_PASS / EXTERNAL_POINTER_GATE_PENDING / HUMAN_DEFERRED`

pointer-loss は別external gateとして切り分け、通常Undo/RedoはE2E証跡により完了。最終HUMAN判定はM3最終チェックへ集約。
