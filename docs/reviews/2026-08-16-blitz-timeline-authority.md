# BlitzがTimelineの正本

日付: 2026-08-16
状態: **決定**

## 決定

利用者裁定(2026-08-16): **製品 Timeline の正本は `crates/motolii-ui/src/timeline_blitz/`**（Blitz = HTML/CSS ＋ custom widget）とする。今後の Timeline はここへ積む。

`timeline_skia_raster.rs` は**意味・hit・oracle の源として残す**が、**製品経路ではない**。現に呼び手は無い（唯一の消費者だった `rn_product_host/timeline_gpu.rs` を[同日に畳んだ](2026-08-16-web-window-and-rn-product-fold.md)）。

| 座席 | 正本 |
|---|---|
| Timeline の描画・行・clip・key | **`timeline_blitz/`** |
| 意味・hit・oracle の源 | `timeline_skia_raster.rs`（残置。製品経路ではない） |
| 編集意味 | Document／D2（変更なし） |
| ジェスチャの論理 | `timeline_move_gesture` / `timeline_trim_gesture` / `timeline_viewport_state` / `timeline_intent_adapter`（renderer 非依存。**再実装しない**） |
| ドッキング | `egui_tiles`（下記の構造上の帰結） |
| Stage spatial | Rerun Spatial Viewer（変更なし） |

## 経緯 — この座席は今日2回動いた

1. [2026-08-15](2026-08-15-egui-timeline-engine-authority.md)は「engine は Motolii egui」と決定した。**しかし現物では `timeline_egui` は製品のどこからも呼ばれていなかった**
2. [同日の訂正](2026-08-16-skia-timeline-authority-correction.md)で「製品 Timeline は Skia」とした。**これは RN 製品についての事実だった**
3. その RN を同日に畳んだので、**Skia の呼び手も消えた**
4. 本決定で **Blitz を正本とする**

**3回とも、文書ではなく現物の参照グラフが判断材料だった。** 座席表に正本の file 名を書いても、それが呼ばれているかは別問題である。本文書の表を読むときも、**呼び手を1回辿ってから使うこと**。

## 併せて確定すること

- [Blitz移植発注capsule](../blitz-port-order-capsules.md) の C1 は「Timeline描画をBlitzへ（**意味は移さない**）」だった。**意味も Blitz が持つ**へ変わる。C3（key帯 custom widget）は 2026-08-16 に完了済み
- `timeline_blitz/` の doc コメントが出所として指す `timeline_egui/*` は**撤去済み**。原文は `git show f209da9d^:` で辿れる。出所の記述は正本変更に合わせて順次書き換える
- **行高は 20px 固定**（[2026-08-08決定(3)](2026-08-08-timeline-design-decisions-and-skia-fixtures.md)）。移植元の `clamp(20,24)` は決定違反だったので 2026-08-16 に訂正済み

## ホストが egui であることは構造上の帰結（Timeline の設計を縛る）

Rerun Spatial Viewer は `show(ui, ..)` で **`&mut egui::Ui` を要求する egui ウィジェット**である。よって:

- **ホストが egui でないと Stage が動かない**
- したがって**ドッキングも egui の責任**（`egui_tiles`）
- **CSS／Blitz へドッキングを移す案は成立しない**。Blitz の custom widget へ渡るのは wgpu の描画コールバックだけで `egui::Ui` ではないため、Stage を入れるには **egui をもう1つ立てる**ことになる。それは 2026-08-16 に 18.6 → 2.1〜3.0 ms/frame のために撤去したものであり、戻すと Stage の入力も落ちる
- **Stage は `egui_wgpu::Callback` ではなく `show(ui)` で挿す**

Timeline を Blitz に置いても、**それを載せる器は egui のまま**である。この2つを混同しないこと。

## 密な面の描き方（C3、実施済み）

clip と key は **DOM ノードではなく custom widget 1ノード**（`timeline_blitz/surface.rs`）。根拠は [P8実測](2026-08-15-blitz-ui-runtime-probe.md#p8--custom-widget-で密な面を1ノードにするスプレッドシート型):

| 描き方 | 天井 | 単価 |
|---|---|---|
| DOM ノード | 約3,600 | `resolve` 4.0µs/個 |
| 描画プリミティブ | 約20,000 | 0.73µs/個 |

行・playhead・clip名は **DOM のまま**（本数がトラック数で有界、CSS で触れる利点が勝る。文字は widget が描けない）。

**重ね順は 行 → 面 → 文字。** DOM の後勝ちなので、custom widget を先に出すと後段の文字を覆う。

## 残余（未決）

- **入力(C2)が未配線。** `surface.rs` の `handle_event` は空。既存のジェスチャ論理へ繋ぐ作業であって、書き起こしではない
- **ジェスチャのしきい値が存在しない**（移動開始px、長押しms、ダブルクリック間隔）。値の出所が要る
- **ショートカット**は `motolii-input` の keymap resolver（[2026-08-14決定](2026-08-14-web-window-product-reflection-authority.md)の「唯一の表」）を **egui 側で**引く。**Blitz へは配らない**（JSが無い＝対話はホストの仕事、が上流の設計）
- `timeline_skia_raster` を今後どう扱うか（oracle として生かす手順を作るのか、いずれ畳むのか）
