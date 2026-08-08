# `N-OVERLAY` — rust-skia 依存ゲート通過記録

日付: 2026-08-08
状態: **ゲート通過 / 実装未着手 / 発注未実施**

## 0. 扱い

[依存優先・責任最小化ゲート](2026-07-24-dependency-first-responsibility-gate.md)§2の7段を通した記録。
**本文書は実装許可ではない。** closed orderは別途compileする。

同ゲートは「**採択済みrouteがあるclosed orderは再調査せず**、
`KNOWN IMPLEMENTATION SEARCH`へ正本pathとdecision-indexの一意な行keywordを置く」と定める。
rust-skiaは既に採択済みであるため、本件は**新機構の裁定ではなく既決routeの実行**である。

## 1. 七段の結果

```text
MECHANISM CLASS: 単一active cameraで投影された2D面へ、
  handle / outline / grid / safe area / path を dirty時だけraster/uploadして
  wgpu previewとcomposeする2D overlay renderer

KNOWN IMPLEMENTATION SEARCH:
  正本: docs/reviews/2026-08-07-m3-react-native-rust-skia-runtime-rebaseline.md §1・§2-2
  decision-index行keyword: "UI runtime React Native Fabric rust-skia Skia wgpu Stage Timeline Curve Editor"
  再調査: 不要（採択済みroute）

CANDIDATES:
  rust-skia (skia-safe)。リポジトリ外の隔離probe `skia-timeline-probe` にて
  `skia-safe 0.99.0` + `wgpu 29` + `winit 0.30.9` が実動。
  Windows target check も別probeで実施済み。
  → docs/reviews/2026-08-08-out-of-repository-asset-inventory.md

ADOPTION ROUTE: ADOPT（依存追加）

REJECTED CANDIDATES:
  vello — repo内に同等経路 `crates/motolii-ui/src/native_timeline_renderer.rs` が実在するが、
  再基線決定により新規製品UIの既定rendererから外れた退役対象。移行oracleとして保持し新規実装しない。
  egui — 2026-07-24に製品runtime採用を撤回、2026-08-07再基線で再確認。

THIN MOTOLII SEAM:
  Stage native component（既存 `MotoliiStageComponentView.mm`）、
  wgpu preview（`render_worker.rs` / `display_slot.rs` / 既存 `Arc<GpuCtx>`）、
  幾何投影（`stage_geometry_projection.rs`、2026-08-07実装）

THIN MOTOLII RESIDUAL:
  何をどの条件でdirtyとするかのpolicy、canonical↔overlay座標変換、
  overlay内容の意味（bounds / grid / safe area / handle）、fixture

IMPORTED RESPONSIBILITY:
  skia binary配布（`binary-cache` feature）、build時間、
  license notice（Skia BSD-3-clause系 / rust-skia MIT。配布物へ再掲が必要）、
  wgpu majorとの整合、3 OS供給網

EXIT:
  Motolii fixtureをskia非依存に保つ。交換時に触るのはoverlay描画層のみ。
  幾何投影・hit-test・D2 terminal intentはskiaを知らない

RETIREMENT:
  `N-OVERLAY`成立後、vello版 `native_timeline_renderer.rs` を FROZEN → RETIRE の候補とする。
  ただし退役は新route出口を同じoracleで確認した後、一つのownerが一度だけ行う。

BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN
```

## 2. ゲート通過中に判明した記録層の欠落

**`docs/references.md` に skia の項目が存在しない。**

`decision-index`には rust-skia の決定行が3本（UI runtime、Easing/Curve Editor、Stage gizmo）
あるにもかかわらず、依存候補の台帳へ登録されていない。

ゲート§2-4は「結果を**一度だけ**正本と`decision-index.md`へ記録する」と定めるため、
**`references.md`へのskia登録が`N-OVERLAY`実装の前段に必要**である。

これは2026-08-08に繰り返し観測された記録層のdrift
（[M3価値観更新](2026-08-07-m3-integration-zone-value-update.md)§6）と同じclassである。

## 3. 実装順（案・未発注）

1. `references.md` へ skia を登録（license区分、version、採択route、限定境界）
2. workspace `Cargo.toml` へ依存追加
3. overlay最小成立 — rust-skiaで矩形1つをrasterし、既存wgpu previewとcomposeしてpresent
4. ここで`N-OVERLAY`成立。`R2-STAGE-GIZMO` / `R2-TL-NAV` / `R2-CURVE-READ` / `R2-STAGE-VIEW` が解放

**3は「dirty時だけraster/upload」「CPU readbackなし」を最初のoracleに含める**
（再基線§4のruntime不変条件）。

## 4. 未決（利用者裁定が要るもの）

リポジトリ外の`skia-timeline-probe`を**参照して移管するか、依存追加のみで製品側に新規実装するか**。
[React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)が
外部成果の移管に停止線を置いているため、同種の判断が要る可能性がある。

## 5. 非目標

- 本文書を根拠に実装を発注すること
- vello版 `native_timeline_renderer.rs` を`N-OVERLAY`成立前に削除すること
- overlayへ Document意味・selection authority・hit-testを持ち込むこと
- skia型をdomain / 公開plugin契約 / Document schemaへ漏らすこと
