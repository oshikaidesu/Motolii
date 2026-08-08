# M3 supervisor handoff — 地図再構築と背骨4粒

日付: 2026-08-07
状態: **引き継ぎ / 施工停止 / 次発注未選定**

## 1. この文書の扱い

runner規則でも設計決定でもacceptanceでもない。2026-08-07のsupervisionで確認・統合した
Git／code事実、未閉鎖境界、次の一手候補を次のsupervisorへ渡す作業メモである。

再開時は本書をauthorityにせず、`AGENTS.md`、`docs/README.md`、decision index、
[成果駆動統合地図](../outcome-driven-integration-map.md)とcurrent codeを再照合する。

**本日はCodex利用枠逼迫のため、利用者判断でClaude（Opus 5）が代理supervisorを務めた。**
authority owner、次粒選定、採否は通常どおりsupervisor席が持つ。

## 2. Git安全境界

- authority checkout: `/Users/member_ottoto/rust_ae/Motolii`、branch `codex/supervision-authority-guard-20260804`、HEAD `1cb92362`
- **rootでreset / checkout / cleanup / stage / commit / push / main統合を行っていない。**
  root差分は既存分＋本日追加したdocsのみ
- local main worktree: `/private/tmp/motolii-r0-main-integration-20260807`、HEAD `9b2deac4`、**clean・不変**
- 背骨統合branch: `codex/r2-spine-integration-20260807`（worktree `/private/tmp/motolii-r2-spine-20260807`）
  - **local mainへ統合していない。push もしていない。**

## 3. 本日local branchへ到達した実コード

```text
7851e3d0  feat(m3): project stage layer geometry read-only
0eb2a3c0  feat(m3): transport stage pointer events to host
11c8d012  feat(m3): seat transient evaluation time in rn host
68546b8d  feat(m3): produce primary selection from stage pointer
```

統合後 `cargo test -p motolii-ui --lib` = **278 passed / 0 failed**、`r0_rn_product_seat` = 5 passed。

これで「**表示中objectをclickして選択できる**」が意味的に成立した。
gizmo、Position key書き込み、Timeline投影、Easingは**未着手**である。
`9b2deac4`同様、これを「触れる動画編集ソフト」へ繰り上げない。

### 施工体制（実測）

- 施工: Cursor Grok 4.5 medium（xAI）— allowlist遵守 **4/4**、逸脱 0
- capsule review: GPT-5.6 Sol high（OpenAI, cursor代用）— **未指定事項16件**を発注前に検出
- 検収: supervisor read-only監査＋oracle再実行
- **主担当capsule起因の欠陥は本日累計8件。施工側の逸脱は0件。** 品質のbottleneckは発注書である

## 4. 本日の中心的成果 — 地図の実測再構築

62項目（R0/R1 12・R2 11・R3 25・M4 13・M5 12）をcurrent codeへ read-only 照合した。

| | 件数 |
|---|---|
| 製品routeから到達可能 | 7 |
| 実在するが旧route／probeにのみ到達 | 30 |
| 部分的に実在 | 23 |
| **本当に存在しない** | **11** |
| コード読解では決着しない | 7 |

**M3は「作る工程」ではなく「先に作った資産を接続する工程」である。**

旧M3地図の`TARGET_MISSING`は「本当に無い」と「旧routeにあるが未接続」を1語へ潰しており、
これが系統的な悲観と誤った見積もりを生んでいた。M4/M5の地図には同種の誤りが無い
（「probeを通したか」という検証可能な事実で書かれているため）。

新地図: [成果駆動統合地図](../outcome-driven-integration-map.md)。
状態語彙を `WIRED / BUILT_UNWIRED / PARTIAL / ABSENT / UNDECIDED / EXTERNAL` へ分けた。

### 確定した誤り

- `R2-KEY-COMMAND`：旧地図は「現行`CommandKind`にkeyframe編集familyが無い」としていたが、
  `AddPositionKey` / `SetPositionKeyValue` / `SetPositionKeyInterp` は**実在する**
- `R3-MENU`：`EXISTS_OLD_ONLY` → `PARTIAL`（CommandIdを経由しない）
- `R3-PROJECT-POLICY`：`EXISTS_WIRED` → `PARTIAL`（OpenMode admission不成立）

## 5. 新規に発見したnode（旧地図のどこにも無い）

- **`N-OVERLAY`** — rust-skiaが`Cargo.toml`に存在しない。実在するのは退役対象の`vello`のみ。
  `R2-STAGE-GIZMO` / `R2-TL-NAV` / `R2-CURVE-READ` / `R2-STAGE-VIEW` の**4nodeが同じ1件で詰まる**
- **`N-ABI-SPLIT`** — `WireIntentEnvelope`は15 fieldのunion structで、本日の3粒は
  意味的にdisjointなのにここで直列化した。per-kind分解で13 fieldがcapability側へ落ちる。
  [Controlled Microkernel決定](2026-07-25-controlled-microkernel-host-module-parallelism-decision.md)の**未実装の既決**
- **`N-GIZMO-SURVEY`** — gizmo機構の既知実装調査recordが存在しない。Blenderは GPL のため `PATTERN` 限定

## 6. 仮コード（器具）

[器具境界決定](2026-08-07-provisional-call-site-sketch-instrument-decision.md)。
利用者outcomeを呼び出し側から先に書き、実名で埋まらない箇所を`???`として露出させる非compile器具。

本日7 outcome分を起草し、**実名で埋まった呼び出し39 / `???` 24**。
`???`集合とnode surveyの`ABSENT`集合の**照合が必須**で、
本日は照合の**不一致がすべて最大の発見**だった（`N-OVERLAY`、`R3-MENU`、`R3-PROJECT-POLICY`、FINDING-2）。

## 7. 前ownerへ返したfinding（修理許可ではない）

[finding返却](2026-08-07-call-site-sketch-findings-return.md)。**いずれも未検証の疑いであり欠陥と断定していない。**

- FINDING-1: `opened.open_mode`が`shell.rs:58-66`で破棄され、`ReadOnlyNewer`を弾く接続が無い → P12 owner
- FINDING-2: `AssetTable`がundo/redo対象外で、media配置をUndoしてもasset登録が残る疑い → M2 owner

## 8. 次の一手候補（未発注）

**`N-OVERLAY`を推す。** `ABSENT`の中で最も多くの下流（4node）を解放し、
利用者が「rust-skiaはどっちみち通る道」と判断済みである。
着手前に[依存優先・責任最小化ゲート](2026-07-24-dependency-first-responsibility-gate.md)を通すこと。
本日supervisorは既知実装preflightを2回飛ばしかけた。**preflightを省略しない。**

`N-GIZMO-SURVEY`は`N-OVERLAY`と独立に並列可能（read-only調査）。

## 9. 未決の相談事項（利用者提起・未着手）

仮コードを背骨やM3に留めず、**製品全体へ広げる**案が利用者から出ている。
利用者はこれを「格の違うもの」「擬態」と表現し、**一度セッションを閉じてから別途議論する**とした。

現時点で決まっていないこと: 範囲、粒度、器具境界の改訂要否、authority化の可否。
**本書はこの案を採用も棄却もしていない。** 次のsupervisorが勝手に着手・棄却しない。

## 10. 明示的非目標

- root dirty差分の整理、stage、commit、push
- 背骨branchのlocal main統合、remote push
- `N-OVERLAY`未成立のままoverlay依存node（gizmo描画、Timeline描画）を発注すること
- `BUILT_UNWIRED`（30件）の再実装
- 本日のfinding 2件をM3の接続粒として修理すること
- M4/M5のmilestone順先行実装（7 outcomeのどこからも需要が立っていない）
- 未検証の疑いを確定した欠陥として外向きに扱うこと
