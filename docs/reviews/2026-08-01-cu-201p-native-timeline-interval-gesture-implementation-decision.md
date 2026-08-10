# CU-201P native Timeline interval gesture 実装決定

- 日付: 2026-08-01
- 状態: **IMPLEMENTED / PRODUCT ROUTE VERIFIED**
- 親: CU-201 / U3b / VS-2

## 1. 結論

通常製品windowのnative Timelineで、既存のClip intervalを左端trim、body move、右端trimとして
直接操作できるようにした。drag中はHost Transient previewだけを更新し、成功releaseだけを既存の
`SetClipStart` / `TrimClipIn` / `TrimClipOut` prepareからjournal-first D2へ一回配送する。

新しい公開API、Document意味、journal schema、汎用gesture/snap機構は追加していない。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| `AUTHORITY` | [CU-201S](2026-08-01-cu-201-u3b-move-trim-snap-responsibility-split-decision.md)、[CU-201M-S](2026-08-01-cu-201m-s-clip-start-command-contract-decision.md)、[CU-201T-S](2026-08-01-cu-201t-s-clip-trim-timemap-contract-decision.md)、[CU-201N-S](2026-08-01-cu-201n-s-timeline-snap-contract-decision.md)、[M3 U3b](../specs/M3-ui-integration.md) |
| `INTERNAL TARGET` | `HostPointerCapture`、`BrowserHostRuntime`、`DocumentEditRuntime`、`project_timeline`、`SetClipStart` / `TrimClipIn` / `TrimClipOut` |
| `OWNER` | drag candidate / snap / previewはHost Transient。確定intervalはDocument、描画はlocal presentation |
| `WRITE ROUTE` | AppKit press → private interval gesture → transient preview → terminal再照合 →既存Writer prepare → journal-first D2 →一snapshot再投影 |
| `GAP` | native Timelineのpointer gestureが既存interval commandへ未接続だった |
| `RESOLUTION ROUTE` | 既存pointer capture、Timeline projection、Writer prepare、publish routeを`REUSE` |
| `DISPOSITION` | `PASS`。次は`CU-201R`の決定的な系列負例 |

## 3. 実装境界

- AppKit `LeftMouseDown`を既存Host captureへ加え、press / move / releaseを同じ境界で扱う
- 選択barの両端8 logical pxをtrim handle、中央をmove hitとする
- frame gridと別Clip edgeだけをCU-201N-Sの辞書順で選び、previewへ反映する
- layout epoch、projection generation、capture generation、target identityをterminalで再照合する
- 成功時だけshared writerへ一件queueし、採用snapshotをStage / Timeline / Inspectorへ同じgenerationで再投影する
- native Timelineにtransient barと選択edge handleを描く。描画loop内のGPU resource生成は0

## 4. 自動検証

- `cargo check -p motolii-ui --locked`
- `cargo clippy -p motolii-ui --locked --all-targets -- -D warnings`
- `cargo test -p motolii-ui --locked`: unit 175件と全integration testが成功
- `git diff --check`

追加oracleは、trimのcommit / Undo / Redo、same-value no-op、edge/body hit、move preview、frame snap、
別Clip edgeとframeのtie、LayerId tieを含む。CU-201Rの系列負例は本粒へ混ぜず後続とする。

## 5. 通常製品route観測

一時bundle `com.motolii.cu201p`から通常製品windowと既存projectを開き、次を実操作した。

1. bar右端dragで`TrimClipOut`が一件commitされ、表示barが短縮
2. bar body dragで`SetClipStart`が一件commitされ、表示barが移動
3. Command+Zで移動前へ戻り、Command+Shift+Zで移動後へ戻る
4. windowを閉じ、同じprojectを再openしてtrim / move後のintervalを表示

journal tailでは`TrimClipOut`の`old_duration=10` / `new_duration=229/30`と、
`SetClipStart`の`0 → 17/15 → 0 → 17/15`を確認した。binary SHA-256は
`c7a0378795e017a3698597a8d830b606f67d6a75c688f77ff7c043b59c78fd2c`、観測後journal SHA-256は
`ff76d5868b26129f1b16d876bbd08e5eb8d4f746727c997293440bd0f4fa8e4a`。

これはCU-201Pの製品route到達証拠であり、CU-201R完了前にCU-201Eを先取りして`DONE`とはしない。

## 6. 非目標

- playhead、beat、marker、keyframeをsnap targetへ追加しない
- snap設定、collision、ripple、lane変更、roll/slip/retimeを追加しない
- React Timelineやdiagnostic専用routeを成果にしない
- drag state、logical px、layoutをDocument / journalへ保存しない
- visual threshold / goldenを変更しない
