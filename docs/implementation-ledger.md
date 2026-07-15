# 実装進行台帳

最終確認: **2026-07-15**

このファイルは、実装者が「次に何をするか」を1枚で判断するための現場用台帳。M0〜M5の意味や完了条件を再定義せず、現在の依存関係と発注順だけを示す。

## 使い方

1. まず本ページの「今すぐ着手できるもの」から1件を選ぶ。
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
| M2 | **実装中** | D1l/D3e、D5、統一camera follow-upをmainへ到達させる。D6は#150実装+#154修復済み |
| M3 | **入場準備** | U0aは独立着手可。G0-2/3/4/6/7と各タスクのM2依存を順次消化 |
| M4 | **契約spike可** | K0でRoD/RoIのruntime契約を凍結 |
| M5 | **identity spike可** | P0IでDuplicator/Instance identityを凍結 |

M2を全件閉じてからM3を一括開始する方式ではない。M3はタスク別入場であり、依存が閉じたレーンから並行して進める。

## 主クリティカルパス

```text
Shared Effect:
D1l #172 → D3e → U2g → K2

M3 shell / preview:
U0a → U1a → U1b → U1f #169
                 └→ U2b → U2c → U2f #168

Unified Camera:
D1j → D1k → D3 camera follow-up → U1f #169 → U2d

Timeline Effect UI:
U0a + G0-2 → U0b → U3a
D1l + D3e + U3a → U2g

Editor scripting:
U2a → U2b → U9a → U9b → U9c
F-11 + K0 → K1 → K7 → SCR-4 (Accumulation/Feedback Canvas)

Bounds / cache:
D3 → K0 #167 → K1 → K2

Duplicator:
P0I #170 → P7a → P7b → P7c → P7U
```

## 今すぐ着手できるもの

並行レーン。同じクレート・契約に触れる場合はPR間の競合を先に確認する。

| 優先 | ID | Phase | 状態 | Issue | 依存確認 | 完了後 |
|---|---|---|---|---|---|---|
| 1 | D1l | M2 | `DO` | [#172](https://github.com/oshikaidesu/Motolii/issues/172) | PR #171、D1e、D1f、D1i-2はmain到達済み。着手時にGR-PVを再確認 | D3eをIssue化 |
| 2 | U0a | M3 | `ISSUE` | — | G0-1完了。M2全体やG0-2/G0-3を待たない | U1aをIssue化 |
| 3 | D5 | M2 | `DO` | [#144](https://github.com/oshikaidesu/Motolii/issues/144) | D3、D4、D4-FU #147はmain到達済み | U5のM2依存を解除 |
| 4 | K0 | M4 | `DO` / `SPIKE` | [#167](https://github.com/oshikaidesu/Motolii/issues/167) | D3はmain到達済み。Issue本文の未checkは着手時に同期 | K1をIssue化 |
| 5 | P0I | M5 | `DO` / `SPIKE` | [#170](https://github.com/oshikaidesu/Motolii/issues/170) | 独立。製品schema/APIを追加しない | P7aをIssue化 |

## 次にIssue化するもの

前段PRがmainへ入った時点で、最新の型名・fixture・依存を確認してから起票する。

| 順序 | ID | Phase | 状態 | 起票条件 | 次の出口 |
|---|---|---|---|---|---|
| 1 | D3e | M2 | `WAIT` | D1l merge | Shared Effectを各Use位置で評価し、U2gを解禁 |
| 2 | U1a | M3 | `WAIT` | U0a merge | Slint shellと静止preview |
| 3 | U1b | M3 | `WAIT` | U1a merge | render worker/latest generation |
| 4 | U0b | M3 | `WAIT` | G0-2 decision + D2(main済み) | U0c/U2a/U3aの入口 |
| 5 | U3a | M3 | `WAIT` | U0a + U0b merge | timeline基盤、U2gのUI依存を解除 |
| 6 | U2g | M3 | `WAIT` | D1l + D3e + U0e + U2b + U3a merge | Effect常時接続線 |
| 7 | K1 | M4 | `WAIT` | K0 merge | cache予算/LRU/並行安全 |
| 8 | P7a | M5 | `WAIT` | P0I merge | Duplicator recipe schema |
| 9 | U9a | M3 | `WAIT` | U2b merge | 汎用one-shot Generator hook。script runtime型を公開契約へ焼かない |
| 10 | U9b | M3/v1.x | `WAIT` | U9a merge | Motolii ShapeScript。Paper.js互換やp5.js互換を名乗らない |
| 11 | U9c | M3/v1.x | `WAIT` | U9b merge | SVG materialize adapter。DOM/XMLをDocument意味へしない |
| 12 | SCR-4 | M4/v1.x | `WAIT` | U9b + F-11 + K0/K1/K7 | 非clear drawをホスト所有Feedbackへ翻訳。隠しcanvasを作らない |

## 凍結済みだが依存待ちのIssue

| ID | 状態 | Issue | 待っているもの | 注意 |
|---|---|---|---|---|
| U2f | `WAIT` | [#168](https://github.com/oshikaidesu/Motolii/issues/168) | U0c、U0d、U2a、U2c | one-shotだけ。永続offset/Modifierへ広げない |
| U1f | `WAIT` | [#169](https://github.com/oshikaidesu/Motolii/issues/169) | U1b、U0e、D1k、D3 camera follow-up | K0は依存ではない。保守的Draftで成立させる |

## 先に仕様を直すもの

| 対象 | 状態 | 問題 | 現場の行動 |
|---|---|---|---|
| [#51](https://github.com/oshikaidesu/Motolii/issues/51) | `DECIDE` / stale | Issue本文の`camera: Option<CompCamera>`・`None=DEFAULT`は、現行D1j/D1kの「全Compositionに常在」「Render入力必須」「DEFAULT直書き拒否」と不一致 | #51をそのまま実装しない。D1j schema → D1k runtime → D3接続の3PRへ再翻訳する |
| G0-2 | `DECIDE` | 入力/キーマップ/a11y最小意味論 | decision/spec PR後にU0bをIssue化 |
| G0-3 | `DECIDE` | plugin UIモデル | custom UI契約を実装しない。U0a/U1aと自動生成fallbackは進行可 |
| G0-4 | `DECIDE` | UI性能測定プロトコル | 絶対fps閾値を先に焼かない |
| G0-6 | `DECIDE` | 視覚token/認知審判 | U0eより前に固定 |
| G0-7 | `DECIDE` | Direct/Tool/Advanced conformance | U2cより前に固定 |

## M3への入場判定

「M3に入った」と呼べる最小条件は **U0aのmain到達**。ただし、それはM2完了を意味しない。

| 目的 | 必要な直前条件 |
|---|---|
| UI shellを始める | U0aのみ。今すぐIssue化可 |
| 静止previewを出す | U0a + D3 |
| 枠外Stageを作る | U1b + U0e + D1k + D3 camera follow-up |
| Relative Moveを作る | U0c + U0d + U2a + U2c |
| Effect接続線を作る | D1l + D3e + U0e + U2b + U3a |
| 編集時Generator hookを作る | U2b。まずruntime非依存のD2 command batch境界だけを固定 |
| ShapeScriptを作る | U9a + D1i-2。正準座標・object/path/group・拒否表を先に固定 |
| SVG adapterを作る | U9b。viewport/Y-down変換と安全な採用subsetを先に固定 |
| 蓄積描画を作る | U9b + F-11 + K0/K1/K7。畳めるshape履歴を先にmaterializeし、残りだけFeedbackへ昇格 |

したがって現在の短い運用判断は、**D1lと並行してU0aを開始する**。D1l完了後はD3eへ進み、U0系基盤が合流した時点でU2g/U2f/U1fを順に解禁する。

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
