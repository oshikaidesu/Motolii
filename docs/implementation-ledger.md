# 実装進行台帳

最終確認: **2026-07-20**

このファイルは、実装者が「次に何をするか」を1枚で判断するための現場用台帳。M0〜M5の意味や完了条件を再定義せず、現在の依存関係と発注順だけを示す。

## 使い方

1. まず本ページの「現在選択中の1件」を確認する。
2. Issueと該当する[マイルストーン仕様](specs/README.md)のタスク行・実装ガードを読む。
3. 依存が1件でも未mergeなら着手しない。
4. 完了時は、実装PR内で仕様のタスク表と本ページを同時に更新する。

情報が食い違う場合の優先順位は次の通り。

1. **意味・完了条件**: `docs/specs/M*.md` と判定済みdecision文書
2. **実際のmerge状態**: GitHub Issue / PR / main
3. **発注順・現在地**: 本ページ
4. **未仕様化の候補**: [backlog.md](backlog.md)

本ページを根拠にschema、公開API、既存タスクの意味を変更してはならない。

## 状態語

| 状態 | 現場での意味 |
|---|---|
| `DO` | 意味と完了条件が固定済み。記載依存のmerge確認後、今すぐ着手できる |
| `ISSUE` | 意味は固定済み。最新mainで型名を再確認してIssue化する |
| `WAIT` | 後続タスク。依存が終わるまでIssue化・実装しない |
| `DECIDE` | 意味または公開契約が未決。decision/spec PRだけ進める |
| `ACTIVE` | 実装または修復が進行中。重複着手しない |
| `DONE` | main到達済み |
| `LATER` | v1.xまたはv2へ明示延期 |

## 現在地

| Phase | 状態 | 現在の出口 |
|---|---|---|
| M0 | `DONE` | spike完了 |
| M1 | `DONE` | exit demo・E2E golden・凍結ゲート宣言済み |
| M2 | **基盤再締結済み** | D1l、D3e、D1m、CAM-G0→D1j→D1k-S→D1k→D3fとA〜C証跡はmain発効済み。D5は再締結の閉集合外で、骨格到達・統合審判pending |
| M3 | **段階発注可 / Uシリーズ直列選択中** | U0a、U0b-1、U0b-2、U0c-1、U0c-2、U0d-1、U0d-2、U0d-3、U2a-0、U2a-1完了。Grok反対側批判で割れを検出したため、U1a-1は静止viewport契約を先に固定してから単独実装 |
| M4 | **契約spike可** | K0でRoD/RoIのruntime契約を凍結。その後K1階層基盤→K7 group freeze→K8全曲Draft coverageへ進む |
| M5 | **identity spike可** | P0IでDuplicator/Instance identityを凍結 |

[M2基盤再締結ゲート](reviews/2026-07-15-m2-foundation-reclosure-gate.md)はmainで解除済み。M3はU0a入場完了後に段階発注可だが、現在選択中のUシリーズはSelected U seriesの前枝番がmainへ到達した時だけ次の1枝番を発注する。

## 主クリティカルパス

```text
Shared Effect:
D1l DONE → D3e → U2g（M3入場後）→ K2

Selected U series:
U0a DONE → U0b-1 DONE → U0b-2 DONE → U0c-1 DONE → U0c-2 DONE → U0d-1 DONE → U0d-2 DONE → U0d-3 DONE
→ U2a-0 DONE → U2a-1 DONE → U1a-1 → U1a-2 → U1b-1 → U1b-2 → U2b-1 → U2c-1〜5
→ U0e-1 → U0e-2 → G0-6H → U0e-3 → U3a → U4a-1 → U4a-2

Unified Camera:
CAM-G0 → D1j → D1k-S → D1k → D3f → U1f #169 → U2d

Rerun learning（製品実装ではないsource監査はM3入場前も可）:
RR-0 inventory → RR-1〜RR-8 asset判定 → RR-9統合縦切り
詳細: reviews/2026-07-20-rerun-learning-transfer-plan.md

Editor scripting:
U2a → U2b → U9a → U9b → U9c
F-11 + K0 → K1b + K1c → K7 → SCR-4 (Accumulation/Feedback Canvas)

Bounds / cache:
D3 → K0 #167 → K1b → K2

Resource pressure / preview:
K0 → K1a → K1b → K1c
K1c + K4 → K1d
G0-8 + K1a → U0f
U1b + U1c + U5 + K1d → U1g → U1h

MV whole-song cache / freeze:
K1b + K1c + D3 → K7a
K7a + K2 → K7b → K7c → U8b
K1d + D3 → K8a
K7c + K8a + D5 → K8b

Duplicator:
P0I #170 → P7a → P7b → P7c → P7U
```

## 現在選択中の1件

全体には独立spikeもあるが、ユーザー選択中のUシリーズは意味・所有境界を優先して
1チケットずつ直列に進める。旧night 3分岐は直接統合しない。

| 優先 | ID | Phase | 状態 | Issue | 依存確認 | 完了後 |
|---|---|---|---|---|---|---|
| 1 | U1a-1 | M3 | `ACTIVE` | — | U0a、U0b-2、G0-1、D3、U2a-1完了。[静止viewport契約](reviews/2026-07-21-m3-u1a-1-static-viewport-contract.md)を先にmainへ到達させ、最新mainからshellだけを実装する | U1a-2を単独実行 |

K0 [#167](https://github.com/oshikaidesu/Motolii/issues/167)とP0I
[#170](https://github.com/oshikaidesu/Motolii/issues/170)は論理上`DO`の独立spikeだが、
Uシリーズ直列選択中は未選択とし、同時着手しない。

## 次にIssue化するもの

前段PRがmainへ入った時点で、最新の型名・fixture・依存を確認してから起票する。

| 順序 | ID | Phase | 状態 | 起票条件 | 次の出口 |
|---|---|---|---|---|---|
| 1 | D1j | M2 | `DONE` | CAM-G0 merge（D1lはmain到達済み） | v5 planar camera schema/default migration |
| 2 | U1b-1 | M3 | `WAIT` | U1a-1 merge | render worker/latest mailbox。古い結果E2EはU1b-2 |
| 3 | U3a | M3 | `WAIT` | U0a + U0b merge | timeline基盤、U2gのUI依存を解除 |
| 4 | U2g | M3 | `WAIT` | D1l + D3e + U0e + U2b + U3a merge | Effect常時接続線 |
| 5 | K1a | M4 | `WAIT` | K0 merge | ResourceLedgerとhard budget。backendの空きVRAM値を正本にしない |
| 6 | K1b | M4 | `WAIT` | K1a merge | cache同一性/LRU/並行store |
| 7 | K1c | M4 | `WAIT` | K1a + K1b merge | VRAM/RAM/disk階層admissionと退避 |
| 8 | K1d | M4 | `WAIT` | K1c + K4 merge | 容量pressureとdeadlineを分離したpreview縮退signal |
| 9 | K7a | M4 | `WAIT` | K1b + K1c + D3 merge | group子合成のatomic bake成果物境界 |
| 10 | K7b | M4 | `WAIT` | K7a + K2 merge | 依存時間区間だけの無効化と旧世代再利用 |
| 11 | K7c | M4 | `WAIT` | K7a + K7b merge | bake hit時の内部graph置換と再freeze |
| 12 | K8a | M4 | `WAIT` | K1b + K1c + K1d + D3 merge | 全曲Draft coverage planner |
| 13 | K8b | M4 | `WAIT` | K7c + K8a + D5 merge | 100GB accounting fixtureと通し再生E2E |
| 14 | U0f | M3 | `WAIT` | G0-2 + G0-8 + U0b + K1a merge | resource policyをUser settingsへ。Documentへ入れない |
| 15 | U1g | M3 | `WAIT` | U1b + U1c + U5 + K1d merge | Transport時刻不変の最新frame表示/コマ落ち |
| 16 | U1h | M3 | `WAIT` | U0e + U0f + U1g merge | Performance/Memory settingsとpressure HUD |
| 17 | P7a | M5 | `WAIT` | P0I merge | Duplicator recipe schema |
| 18 | U9a | M3 | `WAIT` | U2b merge | 汎用one-shot Generator hook。script runtime型を公開契約へ焼かない |
| 19 | U9b | M3/v1.x | `WAIT` | U9a merge | Motolii ShapeScript。Paper.js互換やp5.js互換を名乗らない |
| 20 | U9c | M3/v1.x | `WAIT` | U9b merge | SVG materialize adapter。DOM/XMLをDocument意味へしない |
| 21 | SCR-4 | M4/v1.x | `WAIT` | U9b + F-11 + K0/K1b/K1c/K7 | 非clear drawをホスト所有Feedbackへ翻訳。隠しcanvasを作らない |

## 凍結済みだが依存待ちのIssue

| ID | 状態 | Issue | 待っているもの | 注意 |
|---|---|---|---|---|
| U2f | `BLOCKED` | [#168](https://github.com/oshikaidesu/Motolii/issues/168) | U0c、U0d、U2a、U2c | one-shotだけ。永続offset/Modifierへ広げない |
| U1f | `BLOCKED` | [#169](https://github.com/oshikaidesu/Motolii/issues/169) | U1b、U0e、D1k、D3 camera follow-up | K0は依存ではない。保守的Draftで成立させる |

## 先に仕様を直すもの

| 対象 | 状態 | 問題 | 現場の行動 |
|---|---|---|---|
| [#51](https://github.com/oshikaidesu/Motolii/issues/51) | `DECIDE` / stale | Issue本文の`camera: Option<CompCamera>`・`None=DEFAULT`は、現行D1j/D1kの「全Compositionに常在」「Render入力必須」「DEFAULT直書き拒否」と不一致 | #51をそのまま実装しない。D1j schema → D1k runtime → D3接続の3PRへ再翻訳する |
| G0-2 | `DONE` | 入力/キーマップ/a11y最小意味論 | [M3着手前決定§2](reviews/2026-07-16-m3-preflight-decisions.md#2-g0-2-inputとui状態の意味)に従いU0bをIssue化 |
| G0-3 | `DONE` / `縮小採用` | plugin UIモデル | v1はHost自動生成panelのみ。plugin所有toolkit/native/wgpu UIを実装しない |
| G0-4 | `DONE` | UI性能測定プロトコル | U1c/U3a等でraw結果を取り、絶対閾値は別改訂 |
| G0-6H | `WAIT` / `HUMAN` | 視覚token/認知審判 | U0e-2が作る5 reference screenの目視後に具体tokenを固定しU0e-3へ |
| G0-7 | `DONE` | Direct/Tool/Advanced conformance | UI操作言語とU2c fixtureへ従う |
| G0-8 | `WAIT` / `MEASURE` | resource予算preset/安全余白/hysteresisの具体値 | G0-4手順+K1a実測後に値だけ固定。P3/P3aの意味は変更しない |

## M3への入場判定

U0a(egui骨格+依存方向CI)は本入場で完了。M2基盤再締結は解除済み。下表は論理上の直前条件を示すが、現在のUシリーズではSelected U seriesの直列順が追加の運用条件となる。#180/#191≠入場完了。

| 目的 | 必要な直前条件 |
|---|---|
| UI shellを始める | Selected U seriesのU2a-1までmain到達 + U1a固有依存 |
| Rerun sourceを読む・資産分類する | 入場前も可。commit/license/version、Motoliiへの転移条件、`DEPEND/VENDOR/PORT/PATTERN/REJECT`だけを文書化 |
| Rerun由来crate追加・vendoring・移植を始める | U0a入場 + [Rerun学習・転移計画](reviews/2026-07-20-rerun-learning-transfer-plan.md)の対象RRレーン反対側レビュー |
| 静止previewを出す | U0a + D3 + U1a |
| 枠外Stageを作る | U1b + U0e + D1k + D3 camera follow-up |
| Relative Moveを作る | U0c + U0d + U2a + U2c |
| Effect接続線を作る | D1l + D3e + U0e + U2b + U3a |
| 編集時Generator hookを作る | U2b。まずruntime非依存のD2 command batch境界だけを固定 |
| ShapeScriptを作る | U9a + D1i-2。正準座標・object/path/group・拒否表を先に固定 |
| SVG adapterを作る | U9b。viewport/Y-down変換と安全な採用subsetを先に固定 |
| 蓄積描画を作る | U9b + F-11 + K0/K1b/K1c/K7。畳めるshape履歴を先にmaterializeし、残りだけFeedbackへ昇格 |
| resource設定を出す | G0-2 + G0-8 + U0b + K1a → U0f。設定はUser settings、pressure実測値はTransient |
| 重いpreviewを追従させる | U1b + U1c + U5 + K1d → U1g。project fps/audio clockを変えず表示frameだけ落とす |

したがって現在の短い運用判断は、**M2基盤再締結とD3e、D1m、CAM-G0、D1j、D1k-S、D1k、D3f、M3 U0a、U0b-1、U0b-2、U0c-1、U0c-2、U0d-1、U0d-2、U0d-3、U2a-0、U2a-1は完了済み**。初回Uシリーズは並走させず、U2a-1のmain merge後に`U1a-1`だけを進める。Rerunのcommit固定source監査と資産分類は可能だが、現在のUシリーズ実装と並走させない。D5は骨格を完了扱いせず、本番preview／GPU計測／実機E2Eを後続へ残す。

## 更新規則

- Issue作成時: ID、Issue URL、依存、完了後の出口を追加する。
- PR merge時: 対象を`DONE`へ移すか行を削り、直接の後続を`ISSUE`または`DO`へ上げる。
- decision完了時: `DECIDE`を消し、実装タスクを`ISSUE`へ上げる。
- 依存や型名が変わった時: Issue本文と本ページを同じspec PRで更新する。
- 完了条件、型シグネチャ、意味論表は本ページへ複製しない。
- GitHubのcheckboxが古い場合はmain/PRを確認し、本ページだけでなくIssue本文も同期する。

## 詳細への入口

- 全マイルストーン仕様: [specs/README.md](specs/README.md)
- M2: [M2-document-model.md](specs/M2-document-model.md)
- M3: [M3-ui-integration.md](specs/M3-ui-integration.md)
- M4: [M4-cache-and-analysis.md](specs/M4-cache-and-analysis.md)
- M5: [M5-3d-and-post.md](specs/M5-3d-and-post.md)
- 横断バックログ: [backlog.md](backlog.md)
- Recent motion readiness: [2026-07-15-implementation-readiness-ledger.md](reviews/2026-07-15-implementation-readiness-ledger.md)
