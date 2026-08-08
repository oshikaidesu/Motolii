# CU-201R random MOVE / TRIM oracle acceptance

- 日付: 2026-08-04
- 実装commit: `d0f7dfecd72b66ba1fc4bccb359e5706eb04be32`
- 状態: **DONE / REVIEW ACCEPT**

## 1. 結論

[CU-201R oracle決定](2026-08-04-cu-201r-random-move-trim-oracle-decision.md)どおり、既存`d2_command` fixed-seed `proptest`、shrinking、`DocumentWriter` prepare/apply/Undoだけを再利用し、same-lane MOVE、left TRIM、right TRIMの32 cases × 64 accepted step = 2,048 step系列を固定した。

各stepでstable `LayerId`重複0、Track item数・順序、二つのsentinel Clip全文、target envelope/source、Document validationを確認する。全64 gestureをUndoした後、Document全文、stable-id counter、undo長0を初期値と照合する。新dependency、PRNG、simulator、production helper、schema、journal、snap/group/ripple意味は追加していない。

## 2. 外部施工と検収

初回fresh Sparkは一ファイルcandidateを作ったが、dynamic tool-cycle予算を越え、入力を黙ってclampする候補を含んだため未採用とした。同じ契約境界のexact finding修正をfresh Luna maxへ発注し、provider-native途中streamを保存・観測した。Lunaも予定context/tool-cycle予算を大幅に超えたため、その最終文は採用資格に使っていない。

主担当Codexが一ファイルscope、実diff、全oracleを再実行した。実装familyと分離したfresh Opus mediumのblind reviewは`ACCEPT / SCOPE PASS / MUTATION NONE / P0 NONE / P1 NONE`を返し、Writer/validation原文の`EVIDENCE_GAP`だけを要求した。exact rangeだけを追加したfresh Opus low closureは`CLOSED / MUTATION NONE / P0 NONE / P1 NONE / P2 NONE / EVIDENCE_GAP NONE`だった。

## 3. 自動oracle

- `cargo test --locked -p motolii-doc --test d2_command cu_201r`: PASS、1件、2,048 generated/accepted step
- `cargo test --locked -p motolii-ui timeline_move`: PASS、4件
- `cargo test --locked -p motolii-ui timeline_trim`: PASS、4件
- `cargo test --locked -p motolii-ui product_runtime`: PASS、26件
- `cargo fmt --all -- --check`: PASS
- `cargo clippy --locked -p motolii-doc --test d2_command -- -D warnings`: PASS
- `git diff --check`: PASS

Cancel 0はrandom no-op variantで偽装せず、既存MOVE/TRIM/HOST product laneを再実行した。これは通常window、pointer-loss、通常製品Undo/Redo、reopenの外部・E2E証拠ではない。

## 4. 次の一本道

`CU-201R`を`DONE / REVIEW ACCEPT`とし、`CU-201E`を`DO`へ進める。`CU-201E`は通常製品windowのmove→trim→Undo/Redo→reopen、same identity、保存interval、UI drag state非永続を閉じる。pointer-lossの実機gateはE内で分離して記録する。ユーザー目視は引き続きM3全体の最終HUMAN checklistへ送る。

残余親`CU-201P`はsnap threshold、slip/slide/roll/ripple、multi-select等を残すため`SPLIT / WAIT_TARGET`を維持する。
