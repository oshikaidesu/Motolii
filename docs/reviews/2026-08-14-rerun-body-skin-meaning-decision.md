# Rerun相乗り先・意味源・日常ループ座席

日付: 2026-08-14  
状態: **決定**

> **追補 (2026-08-15):** 利用者裁定により、Timeline **engine** としての Rerun Time Panel 相乗りは撤回。行き先は Motolii egui Timeline。本文は歴史として残す。Stage spatial の Rerun は維持。正本は[Motolii egui Timeline engine](2026-08-15-egui-timeline-engine-authority.md)。

## 決定

**この切り方は Timeline に限る。** Stage／Inspector／Browser／Host／Preview＝Export の既決は触らない。

Timeline だけ、次の二語で割る。

| | 扱うもの | しないこと |
|---|---|---|
| **Rerunにある** | Time Panel（scrub／play／streams／time control／density） | Motolii で Timeline を新造しない。egui Timeline wasm、CanvasKit 画家正本、第二 Time Panel を作らない |
| **普通の編集ソフトの Timeline にあって Rerun にない** | layer／clip／key の作者意味（`timeline_skia.rs` と Document／D2） | Time Panel をキー writer にしない。Skia の意味だけを Rerun Time Panel へ載せる |

利用者確認(2026-08-14): Timeline は Rerun Time Panel のスキン変更＋ Skia 意味の追加であり、Motolii Timeline の一からの再実装ではない。

| 座席 | 正本 | 役割 |
|---|---|---|
| Timeline 表示 | Rerun Time Panel（egui は Rerun の席に付いてくる） | 皮は fork の `re_time_panel`／`re_ui`。公式 Viewer の皮は `WASM_REBUILD` まで stock |
| Timeline 編集意味 | `timeline_skia.rs` の token／hit／key と Document／D2 | Rerun Time Panel に無い作者意味。描画engineではない |

[Web窓を含む製品projection正本化](2026-08-14-web-window-product-reflection-authority.md)の視覚・UX原本撤回は維持する。Spatial の `ADOPT / WRAP` は[Rerun採択再締結](2026-08-10-m5-rerun-spatial-viewer-adoption-reclosure-decision.md)のまま。

## 既存文の処分

次を引用し、**Timeline の製品成長経路**としては撤回する。歴史・probe・residual 本文は消さない。再締結本文は書き換えない。

1. [2026-08-07再基線](2026-08-07-m3-react-native-rust-skia-runtime-rebaseline.md) §1 Timeline行、§5.4、§6 の rust-skia Timeline 成長経路
   → Timeline engine として撤回。意味源としては残す。
2. [UI runtime責任境界](../ui-runtime-architecture.md)「Timeline／Curveとauthoring overlayをrust-skia」
   → Timeline 描画engineとしての rust-skia を撤回。行き先は Rerun Time Panel。
3. [2026-08-08 Skia REJECT→ADOPT](2026-08-08-skia-reject-to-adopt-authority-reconciliation.md)の Timeline／Curve を rust-skia 標準とする文
   → Timeline engine標準を撤回。意味源の保持に縮小。
4. 同日午後に書いた「画家の即時ループ＝Web CanvasKit Timeline」「`timeline_egui` は cassette」「Motolii egui wasm は第二甲板」
   → Timeline に限った仮置きとして撤回。CanvasKit は Rerun Time を隠していた間の葉。Motolii 自己 Timeline の wasm 化は新造なのでしない。

## 非目標

- rust-skia または Motolii egui を製品 Timeline engine として新造すること
- CanvasKit Timeline を画家正本にすること
- Time Panel を Document のキー writer にすること
- この切り方を Stage／Inspector／Browser／Host 全体へ拡張すること
- Rerun 採択再締結本文の書き換え

## 現在状態

4173 は公式 Viewer の Time Panel を `expanded` で出せる。皮の Motolii token は fork の `WASM_REBUILD` が要る。キー意味は Host RRD の `motolii_time` へ載せる工程。CanvasKit Timeline 枠はまだ残っているが Timeline 正本ではない。
