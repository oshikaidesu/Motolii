# CU-201R Timeline interval系列oracle決定

- 日付: 2026-08-01
- 状態: **ORACLE DONE**
- 親: CU-201 / U3b / VS-2

## 1. 結論

外部property-test依存や新しいrunnerを追加せず、既存`DocumentEditRuntime`へ固定seedの
192-step move / trim系列oracleを置いた。三つのClipへ同じ公開済みrequest境界から操作を配送し、
各stepと全Undo後、再open後のDocumentを検査する。

## 2. 固定した負例

- 各stepでLayerId集合が初期集合と完全一致し、Clipの重複・消失が0
- moveはduration、TimeMap、sourceを変えず、in/out edgeを同じdeltaだけ平行移動
- 一操作は一publish、一history entry、一queue drain
- 192回のUndoで初期Document JSON bytesへ完全一致
- Undo列をjournal replayした再openでも初期Document JSON bytesへ完全一致
- interval cancelはactive gesture / previewだけを消し、Document queue entry 0

乱数は再現可能な固定seedのLCGであり、期待値から未決のcollision、ripple、lane、snap targetを
逆算していない。操作候補はCU-201M-S / T-Sで既決のsame-lane moveとin/out trimだけである。

## 3. 検証

- `cargo test -p motolii-ui --locked --lib deterministic_move_trim_sequence_preserves_identity_and_full_undo_reopens_initial_bytes`
- `cargo test -p motolii-ui --locked --lib interval_cancel_clears_only_transient_state_and_enqueues_no_document_work`
- `cargo test -p motolii-ui --locked`
- `cargo clippy -p motolii-ui --locked --all-targets -- -D warnings`
- `./scripts/check-docs.sh`
- `git diff --check`

## 4. 次

CU-201Rを`DONE`とし、通常製品windowの同一identity、保存interval、UI drag state非永続を
CU-201E receiptへ閉じる。CU-201Pで得た先行実操作観測を、R完了後の正規順序で再照合する。
