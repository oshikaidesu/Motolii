# CU-201P-TRIM implementation acceptance

日付: 2026-08-04

実装commit: `da4dcf75611f3ad24cbb24140e5030718d5f9c11`

状態: `IMPLEMENTED / REVIEW ACCEPT / HUMAN_DEFERRED`

## 1. 結論

[CU-201P-TRIM既知意味採択](2026-08-03-cu-201p-trim-edge-known-semantics-adoption-decision.md)のBlender VSE handle hit規則を、既存`ProductTimelineProjection`、`ProductApp` transient、`DocumentEditRuntime`の`TrimClipIn` / `TrimClipOut` writer routeへ縮小接続した。公開`TimelineHit`、`TimelineProjection::hit_test`、Document schema、journal形式、snap意味は変更していない。

実装はcrate-privateな`ProductTimelineHit`とtrim専用gestureへ閉じ、Key優先、Left-before-Right、`min(15 logical px, width/4)`、width 25 / derived height 16未満のbody縮退を保持する。press時のlayer、edge、pointer、interval、generationを固定し、drag中はread-only preview、release時だけ既存Writer prepare/applyを一回通す。same-value、cancel、stale、overflow、target消失、prepare拒否はDocument / journal / history / revision write 0である。

## 2. 既知実装採択

`MECHANISM CLASS`はTimeline bar hit refinementとedge-drag transient lifecycleである。固定Blender sourceのhandle hitを`PATTERN / REDUCE`、既存MotoliiのMOVE lifecycleとtrim writerを`REUSE`した。outside padding、dual handle、GPL code、generic gesture framework、Stage capture、zoom、snapは持ち込んでいない。`BUILD JUSTIFICATION: NONE`、`BUILD: FORBIDDEN`を維持する。

## 3. 外部施工と独立検収

fresh `gpt-5.6-luna` maxへ固定capsuleを発注し、`scripts/run-observed-cli.py`でprovider-native JSON途中stream、生stderr、終了状態を保存・実行中観測した。sessionはexit 0だったが、予定したdynamic context / tool-result cycle予算を大きく超えたため、model最終文は採用資格に使っていない。主担当Codexが実diff、allowlist、開始前後fingerprint、保持WIP非変更、全oracleを再照合した。

その実diffだけをexact原文で連結したblind evidence envelopeをfresh `claude-opus-5` mediumへread-onlyで渡した。初回判定は`ACCEPT / SCOPE PASS / MUTATION NONE / P0 NONE / P1 NONE / EVIDENCE_GAP NONE`で、旧timeline-hit traceとの診断互換だけがP2だった。公開`Option<TimelineHit>` traceを復元する最小修正後、fresh low closure reviewは`CLOSED / MUTATION NONE / P0 NONE / P1 NONE / P2 NONE / EVIDENCE_GAP NONE`だった。reviewerによるmutationはない。

## 4. Oracle

- `cargo fmt --all -- --check`: PASS
- `cargo test --locked -p motolii-ui timeline_trim`: PASS、対象4件
- `cargo test --locked -p motolii-ui document_edit_runtime`: PASS、対象37件
- `cargo test --locked -p motolii-ui product_runtime`: PASS、対象26件
- `cargo test --locked -p motolii-ui --lib`: PASS、186件
- `cargo clippy --locked -p motolii-ui --all-targets -- -D warnings`: PASS
- `git diff --check`: PASS

private hit境界、selection、pointer delta / no-jump、release一回trim / 一回Undo、cancel / stale / invalid zero-writeを直接試験した。repository greenを通常製品E2Eや人間審判へ繰り上げない。

## 5. 次の一本道

`CU-201P-TRIM`を受理し、受理済みMOVE/TRIMだけを対象とする`CU-201R`を`DO`へ進める。広い親`CU-201P`はsnap threshold、slip/slide/roll/ripple、multi-select等の未閉鎖targetを残すため`SPLIT / WAIT_TARGET`を維持する。`CU-201R`はそれらを発明・代替しない。

`CU-201E`は`CU-201R`後とし、pointer-loss、通常製品Undo/Redo、reopenをそこで確認する。ユーザー目視は粒ごとに要求せず、M3全体の最終HUMAN checklistへ集約する。

## 6. 保持物

旧TRIM worktreeの利用者WIPは削除、stash、reset、追記を行わず保持した。受理commitはfresh local main系のclean worktreeで作成した。
