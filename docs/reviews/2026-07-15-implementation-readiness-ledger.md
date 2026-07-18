# Relative / Stage / Shared Effect / Bounds / Duplicator 実装準備台帳（2026-07-15）

ステータス: **運用正本**。先例監査後の各タスクを、実装者が未決を推測せず着手できる単位へ分類する。Issue化は「意味が固定済みの実装」または「後続を凍結するため完了条件が固定済みのspike」に限る。

ただし[M2基盤再締結ゲート](2026-07-15-m2-foundation-reclosure-gate.md)の発効中は、本書のM3行より同ゲートを優先する。M3の意味決定や既存Issueは着手許可ではなく、U1f/U2f/U2gを含む製品実装は再締結後のM3入場PRでIDと依存を再翻訳するまで停止する。

## 状態語

| 状態 | 意味 | 実装者の行動 |
|---|---|---|
| `READY` | 意味・依存・完了条件・非目標が固定済み | 依存merge後、Issueから1 PRで実装 |
| `SPIKE` | 製品schema/APIは未決だが、比較fixtureと凍結出口が固定済み | fixture/reportだけを実装し、製品fieldを追加しない |
| `WAIT` | 自身の意味は固定済みだが、前段の型/schemaが未merge | Issue化せず依存完了後に現物再確認 |
| `BLOCKED` | 永続意味または公開契約に未決がある | 決定Issueだけ進め、製品実装禁止 |
| `UI-PROTOTYPE` | Domain意味は固定済み、投影UIは交換可能 | Document/schemaを変えず、操作ログを残して作り直し可 |

## M2 — Document / evaluation

| ID | 状態 | 固定済み | 未決/依存 | Issue化 |
|---|---|---|---|---|
| GAP-14 | `DONE` | 参照中Delete=Reject、Unlink=RemoveUse、Copy Local=Materialize、orphan=Keep、未知plugin同一規則 | —（決定済） | [#166](https://github.com/oshikaidesu/Motolii/issues/166)、[lifecycle決定](2026-07-15-shared-effect-lifecycle-decision.md) |
| D1l | `DONE`（main到達／再締結証跡待ち） | Definition/Use分離、ordered stack、inline migration、非隣接共有、GAP-14 lifecycle+journal/Undo境界、新規Document v4生成口 | M2再締結時の独立追補レビューと証跡対応付け | `a23a4ad`、`74af37e`、lint follow-up `02192c2` |
| D3e | `DONE` | 各Use位置で個別評価、Groupは子合成後1回、source非消費、prepared params評価 | D1l実装main到達済み、D3 | `crates/motolii-doc/tests/d3e_shared_effect_eval.rs`、`crates/motolii-export/tests/d3e_preview_export_same.rs` |

### GAP-14の出口（完了）

- [x] 判定語付きで`Delete Definition while used`、`Unlink Use`、`Copy Local`、`Delete last Use`を全て決めた
- [x] 各操作のDocument前後、Undo/Redo、未知plugin、再保存後を表にした
- [x] D1lのmigrationとvalidationへ必要な不変条件を列挙した
- [x] schema/API/production codeは変更しない（本PRはdocs-only）
- Cascade delete-all / Purge unused / 一斉Make Uniqueは**延期**（D1l完了条件に入れない）

GAP-14 lifecycle、PR #197、[2026-07-16新規Document v4生成追補](2026-07-16-d1l-current-document-constructor-decision.md)の出口とD1l実装はmain到達済み。D1lの完了条件はM2再締結時にmain SHA／test名へ対応付けて独立追補レビューし、D3eは既存のDefinition解決配線を完了扱いせず専用評価PRで閉じる。

## M3 — UI projection

| ID | 状態 | 固定済み | 可変UI | 依存/Issue化 |
|---|---|---|---|---|
| U1f | `BLOCKED`（意味決定のみ保持） | Final frame不変、off-frame objectを表示/選択、K0を待たない、UI thread readback禁止 | scrim濃度、outline、full-pixel表示、境界装飾 | 再締結とM3入場PR待ち。U1b/U0e/D1kは未翻訳候補。[#169](https://github.com/oshikaidesu/Motolii/issues/169)は着手不可 |
| U2f | `BLOCKED`（意味決定のみ保持） | 全Position Const/keyへ同じEdit-Space差分、時刻/補間/接線不変、1 Undo、Cancel 0、Bake/offset/helperなし | modifier物理key、HUD、ghost、path表示 | 再締結とM3入場PR待ち。U0c/U0d/U2a/U2cは未翻訳候補。[#168](https://github.com/oshikaidesu/Motolii/issues/168)は着手不可 |
| U2g | `BLOCKED`（意味決定のみ保持） | Effect Definition out→Layer Use in、非隣接、1 drag=1 Use、Group/Explicit意味差 | gutter、routing、bundle/stub、socket形状 | 再締結、D1l/D3e、M3入場PR待ち。Issue化しない |

### UI PRの共通禁止

- egui/eframe/winit型、px/DPI、pointer event列、線routingをDocument/domain APIへ保存しない
- prototype都合でD2 command、Effect Use、CompCamera、Boundsの意味を変更しない
- screenshotだけで完了しない。操作fixture、Undo、Cancel、非blockingを自動判定する

## M4 — Bounds / cache

| ID | 状態 | 固定済み | spikeで決める | 依存/Issue化 |
|---|---|---|---|---|
| K0 | `SPIKE` | RoD≠RoI≠texture bounds、`Finite/Infinite/Unknown`、transparent-black範囲外、Unknown保守fallback、preview/export同一関数 | runtime型の最小形、各組込nodeの領域関数、Host clamp表 | D3後。[#167](https://github.com/oshikaidesu/Motolii/issues/167) |
| K1 | `WAIT` | cache予算/LRU/並行安全の審判 | K0型を使うkey/window | K0後 |
| K2 | `WAIT` | Definition変更→全Use invalidation | D1lの実ID型 | D1l/D3e/K1後 |

K0は最適化実装Issueではない。全域評価とのpixel一致を証明する領域契約spikeであり、Document schema、GPU alpha readback、tight Visual Bounds、固定VRAM予算を追加しない。

## M5 — Duplicator / Element Domain

| ID | 状態 | 固定済み | spikeで決める | 依存/Issue化 |
|---|---|---|---|---|
| P0I | `SPIKE` | `InstanceId != index`、明示seed、PRNG version、typed channels、nested context、domain別identity | Distribution別continuity、source local transform、nested ID、algorithm version、materialize境界 | 独立。[#170](https://github.com/oshikaidesu/Motolii/issues/170) |
| P7a | `WAIT` | recipeだけ保存しderived instance列を保存しない | P0Iの決定型 | P0I後 |
| P7b | `WAIT` | GPU instance、Timeline row非増殖、preview/export同一 | P7a schema、K1 cache | P7a/P2/K1後 |
| P7c | `WAIT` | pure Behaviour、seed+ID random | P7b Context型 | P7b後 |
| P7U | `WAIT` + `UI-PROTOTYPE` | Document意味とD2操作 | Inspector/connection/context表示 | P7c/U2g後 |

### P0Iの必須反例

1. Linear/Grid/Radial/Pathのcount増減、中央insert、reorder、distribution type変更
2. indexは変わるが同一と宣言したinstanceのID/random/motion sampleが残るケース
3. 同一性を保てない編集を「再生成」と明示するケース
4. Duplicator内Duplicatorの親子IDとcontext depth
5. Input Shapeのlocal transform/animationを無視・加算・別channel化した比較
6. Text Cluster/Word/Line、Shape Path、Clone Instanceで寿命が異なる反例

## Issue昇格順

```text
作成済み: GAP-14 #166, K0 #167, U2f #168（着手停止）, U1f #169（着手停止）, P0I #170

GAP-14 decision（完了 / #166）
  -> PR #197 journal/Undo追補（P0/P1=0・merge済み）
    -> D1l schema/migration（main到達済み）
    -> D3e evaluation（READY）
    -> U2g UI prototype
    -> K2 invalidation

P0I decision
  -> P7a schema
  -> P7b runtime/GPU
  -> P7c Behaviours
  -> P7U UI prototype

D3
  -> K0 contract
  -> K1 cache
```

図中のU2gを含むM3昇格は再締結ゲート解除後に限る。発効中は上表の`BLOCKED`を優先する。

後続Issueを前倒し作成しない。前段merge時に型名・依存・fixtureを最新mainで再確認し、仕様とコードが一致した時点で1 Issue=1 PRへ翻訳する。
