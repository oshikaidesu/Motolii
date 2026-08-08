# CU-201R random MOVE / TRIM sequence oracle decision

- 日付: 2026-08-04
- 状態: **SPEC DONE / IMPLEMENT DO**
- 親: CU-201 / U3b / VS-2

## 1. 粒の目標と背骨

受理済み`CU-201P-MOVE` / `CU-201P-TRIM`が使う既存`SetClipStart`、`TrimClipIn`、`TrimClipOut`を、再現可能なrandom有効操作列で合成し、identity重複0、no-ripple、全Undoで初期Document完全一致を固定する。広い親`CU-201P`のsnap、slip/slide/roll/ripple、multi-select、group操作を期待値から発明しない。

```text
AUTHORITY -> CU-201S / M-S / T-S / accepted MOVE and TRIM
INTERNAL TARGET -> existing d2_command proptest sequence
OWNER -> DocumentWriter command/history oracle
WRITE ROUTE -> prepare_* -> one fresh gesture -> apply_command -> undo
GAP -> accepted move/trimを混ぜた系列propertyがない
RESOLUTION ROUTE -> existing fixed-seed proptest patternをREUSE / REDUCE
DISPOSITION -> one test-only grain
```

## 2. 既知実装採択

`MECHANISM CLASS`はstateful property-based command sequence testingである。

- repo既存: `crates/motolii-doc/tests/d2_command.rs::random_multi_gesture_sequence_undo_redo_restores_semantic_state`
- workspace既存: `proptest = "1"`、`RngSeed::Fixed`、shrinking
- 製品既存: `DocumentWriter::prepare_set_clip_start` / `prepare_trim_clip_in` / `prepare_trim_clip_out`、`begin_gesture`、`apply_command`、`undo`

採択routeは`REUSE / REDUCE`。新しいrandom crate、独自PRNG、汎用state-machine framework、UI crateへのdev dependency追加は棄却する。`BUILD JUSTIFICATION: NONE`、`BUILD: FORBIDDEN`。

## 3. exact oracle

固定fixtureは同一Track上にtarget Clipと二つ以上のsentinel Clipを持ち、全Clipのstable `LayerId`、Track item順、非target Clip全文を初期oracleとして保持する。生成するのは有効な次の3操作だけである。

1. targetのsame-lane MOVE
2. targetのleft TRIM
3. targetのright TRIM

各stepは現在snapshotから有効なabsolute edgeを導き、対応する既存Writer prepareを呼び、`Some(Command)`だけをfresh gestureへ一回applyする。same-valueはcommit数へ数えない。各accepted step後にDocument validation、LayerId multiset重複0、item数・順序不変、全sentinel Clip全文不変、targetのenvelope / source不変を確認する。ここでの「相対位置維持」はno-ripple契約に従う非target同士のinterval差不変であり、未採択のgroup moveを意味しない。

全accepted gestureをUndoした後、Document全文、stable-id counter、undo長が初期値へ戻ることを確認する。系列は固定seedとshrinkingを持ち、少なくとも2,000 step相当をCIで実行する。

Cancel 0は既存`timeline_move_gesture` / `timeline_trim_gesture` / HOST-INPUT cancelの自動testを同じvalidation laneで再実行する。random列へ意味のないno-op variantを足してCancelを証明したことにしない。

## 4. allowlist

- `crates/motolii-doc/tests/d2_command.rs`
- oracle補助が不可避な場合だけ同test file内のprivate helper

production code、Cargo manifest、公開API、Document schema、journal format、gesture実装、既存test期待値は変更しない。

## 5. validation

```text
cargo test --locked -p motolii-doc --test d2_command cu_201r
cargo test --locked -p motolii-ui timeline_move
cargo test --locked -p motolii-ui timeline_trim
cargo test --locked -p motolii-ui product_runtime
cargo fmt --all -- --check
cargo clippy --locked -p motolii-doc --test d2_command -- -D warnings
git diff --check
```

## 6. STOP / 次手

既存prepare routeだけで有効列を構築できない、あるいはrelative invariantがgroup/ripple意味を必要とする場合はその部分だけを停止し、Solへ`REDUCE / REMAP`で戻す。production helper、clamp、snap、group operationを作らない。

実装・独立review受入後だけ`CU-201E`を`DO`へ進める。通常window、pointer-loss、通常製品Undo/Redo、reopen、ユーザー目視は本粒へ入れない。
