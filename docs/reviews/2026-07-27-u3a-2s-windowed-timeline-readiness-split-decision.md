# U3a-2S windowed native Timeline readiness分割決定

- 日付: 2026-07-27
- 状態: **決定**
- U3a-2S: **DONE**

## 1. 目的と非目標

`U3a-2`（windowed native Timeline projection / layout / hit-test + dense surface）が依存する
`G0-9`を、既存 authority に書かれた事実だけで4区分 `(A)〜(D)` に分割し、windowed native Timeline を
**今どこまで docs / 隔離証拠で閉じられるか**を確定する。renderer 勝者、semantic zoom 段階の中身、
selection 入力、公開 API、通常製品 window の結合は決めない。

本粒は docs-only である。Rust / UI / spike raw / manifest は読むだけで変更しない。

## 2. authority から引いた事実

### gate 表（G0-9L 限定確定と G0-9D 閉集合）

[G0-9段階化](2026-07-23-m3-g0-9-staged-platform-gates.md) §2 は `G0-9L` を固定 Mac の
platform prerequisite evidence 限定確定、`G0-9D` を Windows・追加 hardware・配布対象 Mac の
distribution gate とする。§6 は G0-9D の閉集合（Windows 10/11、WebView2、per-monitor-v2 DPI、
MS-IME、NVDA、第二 monitor、HDR/SDR、detach/re-dock 等）を列挙し、未所有 hardware は
`WAIT / HARDWARE` とする。§7 は `G0-9L: PASS` が意味するのは固定 Mac で L1〜L3 を満たした
platform prerequisite evidence だけであり、W0b、H1b、Motolii Studio Preview、通常製品 window 結合、
Distribution Ready、egui baseline 削除は引き続き停止すると明記する。§8 必須負例は、
Mac 合格の Windows / 追加 monitor 外挿と「G0-9L PASS = 製品粒入場可」の同義化を拒否する。

[L1測定追補](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) §2 は L1 比較 arm を
`direct_vello` と `egui_vello` の二つに限定する。§4 は CU-0G02BH / CU-0G02B / CU-0G05L を
`DONE / FROZEN` とし、`G0-9L: PASS` を fixed-Mac prerequisite evidence だけに限定する。
絶対閾値、renderer 勝者、egui 削除は決めない（同 §4）。

[`manifest.json`](../spikes/g0-9-local-platform-evidence/manifest.json) は
`gate_decision: PASS_FIXED_MAC_PREREQUISITE_EVIDENCE_ONLY`、
`responsibility_disposition.class: REUSE / FREEZE`、`new_runtime_code: false`、
`product_import: false` を記録する。

### CU-105R 処分表

[CU-105責任再確認](2026-07-27-cu-105-dense-timeline-responsibility-recheck.md) §3 は
layout/cull/hit-test を `PASS`（U3a-1I）、1k/100k を `REDUCE`（既存 capacity evidence）、
semantic zoom を `STOP`（U3a-2）、selection / playhead / range を `STOP`（CU-106-family）とする。

### CU-106S 到達性ゲート

[CU-106 selection consumer分割決定](2026-07-27-cu-106-selection-consumer-split-decision.md) §1 は
`project_timeline` / `TimelineHit` の production caller 0、pointer 入力不在、egui Timeline 製品面不在を
コード事実として固定する。§2 は CU-106P/F を `WAIT` とし、CU-106P の入場は U3a-2 入場範囲決定と
non-test consumer 成立を要する。§3 は lint 抑制・dummy caller を製品到達性として数えない到達性ゲートを列挙する。
§5 は U3a-2S で G0-9 依存の4分割を行うまで CU-106P/F を起動しないとする。

### コード事実（発注書 BASE_SHA 機械確認。本粒で再調査しない）

1. `crates/motolii-ui/src/timeline_projection.rs` は Document→top-level Clip / Position key の
   read-only projection、first-fit band、viewport cull、key 優先 Manhattan hit-test、typed unsupported /
   overflow を実装済み。selection / playhead / semantic zoom / owned range は保持しない。
2. `crates/motolii-ui/src/lib.rs` は `project_timeline` / `TimelineBar` / `TimelineHit` /
   `TimelineKey` / `TimelineMetrics` / `TimelineProjection` を pub 再 exportする。
3. `project_timeline` / `TimelineHit` の参照は integration test のみで、production caller は 0 件。
4. `crates/motolii-ui/src/` に native windowed Timeline の製品 surface module は存在しない。
5. `ui/motolii-web/src/` に time rail / bar / key / playhead を持つ React 製品 component は存在しない
   （`KeyToolsCandidate` 等の tool panel のみ）。

## 3. (A)/(B)/(C)/(D) 4区分表

| 項目 | 区分 | 根拠 authority の節番号 | 解禁しないもの |
|---|---|---|---|
| 固定 Mac `G0-9L` platform prerequisite evidence（L1〜L3 raw、単一 manifest） | (A) | [G0-9段階化](2026-07-23-m3-g0-9-staged-platform-gates.md) §2 `G0-9L` 行、§7 限定確定、[L1追補](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) §4、[manifest](../spikes/g0-9-local-platform-evidence/manifest.json) `gate_decision` | W0b、H1b、Motolii Studio Preview、通常製品 window、Windows/追加 hardware、Distribution Ready、parent `G0-9` 全 platform 合格、egui baseline 削除 |
| headless `U3a-1I` projection / layout / cull / hit-test の再利用（再実装しない） | (A) | [CU-105R](2026-07-27-cu-105-dense-timeline-responsibility-recheck.md) §3 `PASS` 行、[U3a-1分割](2026-07-26-u3a-1-headless-timeline-owner-visibility-split-decision.md) §3 | semantic zoom 段階、selection/playhead/range、windowed renderer 勝者、製品 surface module |
| 隔離 windowed Timeline / surface host / L1 計測 spike を証拠カプセルとして読む（製品 import 0） | (A) | [G0-9段階化](2026-07-23-m3-g0-9-staged-platform-gates.md) §4 harness 定義、§5 L1、[L1追補](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) §3 CU-0G02B raw | renderer 採択、絶対閾値、headless 正しさ・D2・selection consumer の合格証拠への昇格（§8 負例、[CU-105R](2026-07-27-cu-105-dense-timeline-responsibility-recheck.md) §6） |
| Windows・追加 hardware・配布 Mac・異 DPI/HDR までの platform 受入 | (B) | [G0-9段階化](2026-07-23-m3-g0-9-staged-platform-gates.md) §2 `G0-9D` 行、§6 閉集合 | macOS fixed-Mac 結果の外挿、synthetic PASS、G0-9L PASS を G0-9D 完了と同義にすること（§8） |
| 第二 monitor / detach の multi-Surface 構造証拠（G0-9L 通常 topology の代替ではない） | (B) | [G0-9段階化](2026-07-23-m3-g0-9-staged-platform-gates.md) §7 L4 表「同一display multi-Surface spike」行、§6 閉集合 | G0-9L 単一 top-level Surface 前提の製品結合、Distribution Ready 主張 |
| W0b / H1b / Motolii Studio Preview / 通常製品 window と WebView/native の結合 | (C) | [G0-9段階化](2026-07-23-m3-g0-9-staged-platform-gates.md) §2 `G0-9L` 完了時にも許可しない列、§7 停止範囲 | diagnostic harness を製品 route と呼ぶこと（§8）、G0-9L harness の製品画面再利用 |
| `TimelineHit` production caller・pointer 相当入力・CU-106P primary consumer | (C) | [CU-106S](2026-07-27-cu-106-selection-consumer-split-decision.md) §1、§2 `CU-106P` 行、§3 到達性ゲート、§5 | lint 抑制・dummy caller・公開 `DomainIntent` 追加での到達性偽装 |
| `direct_vello` / `egui_vello` renderer **採択判断**（範囲設定のみ、勝者は別粒） | (D) | [L1追補](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) §2 arm 定義、§4 非目標（勝者・閾値・egui 削除未決）、[G0-9段階化](2026-07-23-m3-g0-9-staged-platform-gates.md) §5 L1 採択記録 | 既存 raw からの勝者・優劣・CI 絶対閾値の導出（[L1追補](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) §5 負例） |

区分 (A) は **G0-9L 済 fixed-Mac prerequisite evidence と headless U3a-1I・隔離 spike 読取**まで。
区分 (B) は **G0-9D 閉集合が閉じるまで停止**する範囲。区分 (C) は **製品前提と consumer 入力**待ち。
区分 (D) は **renderer 採択を U3a-2 本体から切り出す**範囲（semantic zoom 段階の責任所在は U3a-2Z 候補）。

## 4. 既存 spike の evidence capsule 分類表

[依存優先ゲート](2026-07-24-dependency-first-responsibility-gate.md) §3 の `PASS / REDUCE / STOP` と
`FROZEN` / `DELETE-LATER` / `KEEP-AS-EVIDENCE` のみを用いる。4区分への効きは §3 参照列。

| 対象 | 処分 | RETIREMENT | 4区分 | 根拠 |
|---|---|---|---|---|
| [g0-9-windowed-timeline.md](../spikes/g0-9-windowed-timeline.md) | `REDUCE` | `KEEP-AS-EVIDENCE` | (A) L1 raw、(D) 比較入力 | [L1追補](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) §3〜§4 CU-0G02B `FROZEN`；製品統合・勝者判定は進めない（同 spike 本文 CU-0G02 節） |
| [timeline-bench.md](../spikes/timeline-bench.md) | `REDUCE` | `KEEP-AS-EVIDENCE` | (A) capacity のみ | [CU-105R](2026-07-27-cu-105-dense-timeline-responsibility-recheck.md) §3 `REDUCE` 行；headless 正しさ・D2 証明ではない（M3 U3a 行・同 §6） |
| [g0-9-surface-host.md](../spikes/g0-9-surface-host.md) | `PASS` | `KEEP-AS-EVIDENCE` | (A) topology 部分証拠 | [G0-9段階化](2026-07-23-m3-g0-9-staged-platform-gates.md) §7 L4 表；G0-9L manifest へ束ね済み。renderer 採用・G0-9 完了ではない（spike 冒頭） |
| [g0-9-timeline-visual-parity.md](../spikes/g0-9-timeline-visual-parity.md) | `REDUCE` | `KEEP-AS-EVIDENCE` | (A) 外観 oracle、(C) 製品未接続 | spike §「製品操作は未接続」；React time surface を製品 package へ移さない（[React移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)、decision-index React 境界行） |
| [g0-10-multi-surface-window.md](../spikes/g0-10-multi-surface-window.md) | `STOP` | `KEEP-AS-EVIDENCE` | (B) G0-9D 候補 | [G0-9段階化](2026-07-23-m3-g0-9-staged-platform-gates.md) §6、§7 L4「G0-9L 通常 topology の代替ではない」；異 DPI・第二 monitor は未証明（spike 状態行） |
| [manifest.json](../spikes/g0-9-local-platform-evidence/manifest.json) | `PASS` | `FROZEN` | (A) | `REUSE / FREEZE`、`PASS_FIXED_MAC_PREREQUISITE_EVIDENCE_ONLY`；製品 import 0 |

## 5. 次の最小 closed 実装/決定粒

順位付き閉集合を機械照合した結果、**`U3a-2R` のみ**を次 PRODUCT-ASSET 判断として選定する。
`U3a-2Z` は本決定 `DONE` 後も semantic zoom **責任所在**が未閉じのため候補に残すが、
順位 1 の入場条件が BASE_SHA コード事実と authority で既に成立するため同時に `DO` にしない。

### 選定: `U3a-2R`（docs-only / renderer 採択判断の**範囲設定** / 区分 (D)）

**入場条件（すべて成立）**

1. 本決定 `U3a-2S: DONE`（本 commit 完了時点で成立）。
2. [L1追補](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) §4 にある CU-0G02B raw が `DONE / FROZEN`
   （同一 §4 および [g0-9-windowed-timeline.md](../spikes/g0-9-windowed-timeline.md) CU-0G02B 節）。
3. 絶対性能閾値・renderer 勝者・egui baseline 削除が authority に未追加（同 §4 非目標）。

**exact allowlist（実行はしない。次粒 closed order の草案）**

- `docs/reviews/2026-07-27-u3a-2r-renderer-adoption-scope-decision.md`（新規 decision のみ）
- `docs/reviews/README.md`（索引 1 行追加のみ）
- `docs/decision-index.md`（1 行追加と既存 U3a-2 系参照更新のみ）
- `docs/implementation-ledger.md`（並列レーン・発注依存証跡・運用 prose のみ）
- `docs/specs/M3-ui-integration.md`（U3a 行・運用順 prose のみ。完了条件・型シグネチャ・GR-UI 割当表は変更しない）

### 順位 2: `U3a-2Z` — **WAIT**（同時 `DO` 禁止）

semantic zoom 段階の責任所在 docs 粒。[CU-105R](2026-07-27-cu-105-dense-timeline-responsibility-recheck.md) §3
`STOP`（U3a-2）の残項目。`U3a-2R` 完了後に入場条件を再照合する。

## 6. 非目標

- Rust / JSX / CSS / fixture / golden / bench / spike raw / manifest の変更。
- renderer 勝者、`direct_vello` / `egui_vello` の採択、egui baseline 削除。
- 絶対性能閾値、CI 数値閾値、60fps 公約。
- semantic zoom 段階の中身、playhead / range owner、selection 入力、pointer production 入力の決定。
- `CU-106P` / `CU-106F` / `U2h-1P` / `U3a-2` 本体実装の `DO` 昇格。
- W0b / H1b / Motolii Studio Preview / 通常製品 window / Distribution Ready の解禁。
- 公開 API、`DomainIntent` / Document / schema / plugin 契約の変更。
- native time surface（rail / bar / key / playhead / 高頻度 scrub）の React 製品資産化、`TimelineCandidate` 相当の新設（N6）。
- Rerun を根拠にすること。

## 7. STOP

1. §3 の行を authority 節番号なしで書く必要が出た（本粒では該当なしで閉じた）。
2. renderer 勝者・永続 renderer 契約・絶対閾値を本粒で決めないと閉じない（該当なし。区分 (D) は U3a-2R へ送った）。
3. 次粒のために lint 抑制・dummy caller・公開 intent が要る（該当なし。次粒は docs-only `U3a-2R`）。
4. spike 分類のために新 harness・再実測が要る（該当なし）。
5. G0-9L PASS を Windows / Distribution Ready へ外挿したくなる（拒否。§3 (B)）。
6. `docs/spikes/**` / `crates/**` / `ui/**` を変更したくなる（本粒では変更しない）。

### 必須負例（N1〜N8）

- **N1**: 「G0-9L PASS = U3a-2 入場可」と書かない（[G0-9段階化](2026-07-23-m3-g0-9-staged-platform-gates.md) §7・§8）。
- **N2**: 既存 1k / 100k spike を headless 正しさ、D2、selection consumer、renderer 採択、製品 60fps の合格証拠へ昇格させない。
- **N3**: `direct_vello` / `egui_vello` の raw から勝者・優劣・閾値を導かない。
- **N4**: diagnostic harness / spike / Native Shell Baseline を Motolii Studio Preview または通常製品 route と呼ばない。
- **N5**: 次粒 ID を 2 件以上同時に `DO` にしない。親名 `CU-106` / `U3a-2` で closed order を作れると書かない。
- **N6**: native time surface を React 製品 package の責任へ移さない。
- **N7**: 台帳・decision-index・spec のいずれかだけを更新して他を古いまま残さない。
- **N8**: `docs/mocks-ui` を現行実装として更新しない。

### React 境界（読取専用確認）

製品 React は Browser / Inspector / form / panel / `KEYS`・`LAYERS` tool panel（`KeyToolsCandidate`）を所有し、
time / Z 軸の rail・bar・key・playhead は native Rust/wgpu が所有する（decision-index UI runtime 行、
[React移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)）。本粒は source asset を 1 byte も変更しない。
