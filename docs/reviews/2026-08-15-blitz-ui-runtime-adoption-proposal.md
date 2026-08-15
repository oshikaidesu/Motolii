# UI基盤を Blitz(HTML/CSS) + テクスチャ合成へ移す提案

ステータス: **比較中 / 裁定未記入**。本文書は[実測プローブ](2026-08-15-blitz-ui-runtime-probe.md)を根拠に
**処分すべき既決と移行の形を固定するための起案**であり、裁定そのものではない。

**この文書を根拠に施工を始めない。** 裁定欄が空である間は[レビュー規律](README.md)1が適用される。

## 1. 提案する構成

```
Motolii プロセス(1つ)
├── Document / single writer / Undo / journal          Rust。変更なし
├── ホスト窓 + wgpu29 デバイス                          eframe(egui) 現行のまま
├── Stage                                              Rerun SpatialStage(維持)
└── UI面(Timeline / パネル / Inspector)
    ├── Blitz が HTML/CSS を **自前テクスチャ**へ描く
    ├── 入力ルーティングは **Motolii側**が持つ
    └── 密で文字を持たない面は blitz-dom の **custom widget** で1ノード化
```

境界は **「テクスチャを返す」「イベントを受け取る」の2本**に限定する。
これによりUI技術は後から差し替え可能に保たれる。

## 2. 処分すべき既決（**手続き上必須**）

2026-08-08 の[Skia裁定](2026-08-08-skia-reject-to-adopt-authority-reconciliation.md)は、
手続き上の失敗として次を記録している。

> **既決を覆す変更は覆す対象を明示的に引用して処分を書く**（新決定を足すだけでは逆引きで衝突が見えない）

本提案が採択される場合、**以下3件の処分を同じ変更で書くこと**。引用なしに新決定を足すと
2026-08-07 とまったく同じ失敗を繰り返す。

| # | 対象 | 現在地 | 想定処分 | 理由 |
|---|---|---|---|---|
| 1 | **RN + rust-skia 再基線**（2026-08-07） | `decision-index` 132行 / [再基線決定](2026-08-07-m3-react-native-rust-skia-runtime-rebaseline.md) | **撤回** | RN↔Rust橋が実装8,247行+テスト7,273行、`host_bridge` 4,488行に達し、直近60コミットの36件(60%)が所有権・世代・生存期間の同期バグ。原因はRNが同一状態に第二の所有者を作ること（Motoliiの `single writer` と正面衝突） |
| 2 | **Skia REJECT→ADOPT 裁定**（2026-08-08） | `decision-index` 166行 | **再判定** | ADOPTの実質的根拠は「旧REJECTの前提『Velloと重複』が、Velloが製品標準から外れたことで消滅した」こと。**Blitz は Vello** なので前提が復活し、今度はSkia側が重複になる。alpha・色の懸念とSkia型を漏らさない規律は**維持**する |
| 3 | **Web窓 projection 正本化**（2026-08-14） | `decision-index` 22行 | **撤回** | 利用者裁定により Web/モバイル対応は行わないと確定（2026-08-15）。RN採択の残る唯一の実利が消滅 |

あわせて[N-OVERLAY依存ゲート](2026-08-08-n-overlay-dependency-gate.md)の `EXIT` 条項
（「fixtureをskia非依存に保ち、交換時は overlay描画層のみ」）を発動する形になる。

## 3. 根拠（[プローブ](2026-08-15-blitz-ui-runtime-probe.md)実測、2026-08-15）

| 論点 | 実測 |
|---|---|
| 自前 wgpu29 テクスチャへの描画 | **PASS**（ピクセル完全一致） |
| 自前ルーティングのイベント→DOM→絵の変化 | **PASS** |
| 日本語 IME 4項目 | **合格**（利用者審判） |
| clip/trim/key/playhead の掴み | **合格**（利用者審判） |
| **現行Timeline UIの再現** | **合格**（利用者審判）。実寸・実配色を`timeline_egui/`から写して一致 |
| DOM の天井 | 約3,600ノード（resolve 約4.0µs/ノード） |
| custom widget | resolve が消え、天井が約20,000プリミティブへ |
| 差分更新 | 32.25ms → **8.31ms**（900クリップ・毎フレームズーム） |
| `timeline_skia` のSkia依存 | 基本描画9種のみ。シェーダ/フィルタ/SkSL **0件** |
| 移植面 | 描画層 約1,200行。論理層 約1,570行は renderer非依存 |
| ライセンス | Apache-2.0 OR MIT、CLAなし、fork可 |

## 4. 未了（裁定前に埋めるべきもの）

| # | 未了 | 重さ |
|---|---|---|
| 1 | **egui側を同条件で測っていない** | 比較が片手落ち。IME・手触り・性能をBlitzだけ測った状態 |
| 2 | **反対側レビュー未実施** | 規律2 |
| 3 | ~~ドッキング~~ | **2026-08-15に調査済み。塞がる見込みが立った**（下記） |
| 4 | 透過合成（Stageの上へ重ねる） | 未検証 |
| 5 | `dioxus-native 0.8` は alpha、`blitz-* 0.3.0-beta.1` | 成熟度。0.7→0.8でカスタム描画APIが移動している |
| 6 | モックは静止画で、現行UIそのものでの編集操作は未接続 | P5/P6で部分的には確認済み |

### ドッキングの移植可能性（2026-08-15 実測）

`egui_tiles`(rerun-io, MIT OR Apache-2.0) 5,136行の egui 依存度をファイル別に計測した。

| ファイル | 行数 | egui言及 | 内容 |
|---|---|---|---|
| `tiles.rs` | 1,049 | **2行 (0%)** | タイル格納・GC |
| `container/mod.rs` | 277 | **2行 (0%)** | コンテナ種別 |
| `tile.rs` | 75 | 2行 (2%) | |
| `lib.rs` | 385 | 15行 (3%) | |
| `grid.rs` | 711 | 32行 (4%) | グリッド配置計算 |
| `linear.rs` | 590 | 28行 (4%) | 分割配置計算 |
| `tree.rs` | 933 | 55行 (5%) | ツリー操作 + D&D状態機械 |
| `tabs.rs` | 502 | 41行 (8%) | タブ |
| `behavior.rs` | 614 | 50行 (8%) | **差し替え口（設計上の継ぎ目）** |

**約95%がツールキット非依存。** 使用している egui 型は `Ui`(30) / `Id`(19) / `CursorIcon`(9) /
`Style`(7) / `Sense`(6) / `Context`(5) など約100箇所で、大半は描画とヒット感知の入口。

移植は3層に分かれる。

- **そのまま**: `tiles.rs` + `container/mod.rs` + `tile.rs` 約1,400行
- **ほぼそのまま**: `grid.rs` + `linear.rs` + `tabs.rs` + `tree.rs` 約2,700行（`Rect`/`Vec2` を `kurbo` へ置換）
- **書き直す**: `behavior.rs` と各 `ui()` 約600行（描画・カーソル・ヒット判定。**元から差し替え前提**）

1点だけ実作業: `egui::Id`/`Context` で **egui のメモリストアにドラッグ状態を保存**している
（`smooth_preview_rect` 等）ため、自前の状態保持へ置き換える。HashMap 一つで足りる。

**Web側のドッキングライブラリ(dockview / Golden Layout / rc-dock / FlexLayout)は使えない** —
いずれもJSでロジックを書いており、BlitzはJSエンジンを持たない。
成立する形は「ツリーとD&D状態機械はRust、配置結果の描画はCSS flex/grid」であり、
これは**Blitzが同一プロセスのRustライブラリだから可能**（webviewでは境界を跨ぐため不可）。

**残る最重量の未了は 1(egui同条件測定) と 2(反対側レビュー)。**

## 5. 移行の形（採択された場合）

境界が2本に限定されるため、段階移行が可能。

1. Timeline面をBlitzで描き、既存の `timeline_skia/` 論理層（hit/geometry/session）を再利用
2. `draw_str` → `draw_glyphs` + parley、fixture の PNG 出力を `image` crate へ（skia非依存化）
3. パネル/Inspector を順次Blitzへ
4. `rn_product_host`（実装8,247+テスト7,273）と `host_bridge`(4,488) を退役

**一括置換をしない。** 各段でoracleを保ち、`timeline_skia` は意味/hit の源として残す
（2026-08-15 egui裁定と同じ扱い）。

## 裁定

（未記入 — 利用者が記入する）
