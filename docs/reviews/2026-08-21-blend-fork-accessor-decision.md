# 裁定161: blend 逐次合成は fork へ「素直な口」を1本足す(main_target accessor)

日付: 2026-08-21 / 決定者: supervisor(後任セッション、全面委任の範囲内)/ 種別: 実装経路の裁定

## 事実(レーンβ = BL1 の実測、`tests/sequential.rs` に再現可能な証拠)

- `render_sequential`(layer 毎 accumulator 逐次合成)は単層・空 comp では既存 `render` とバイト一致するが、**重なり半透明で不一致**(1264/4096px 差・max Δ49/ch)
- 真因はフレームライフサイクルではなく **gamma 空間**: `ViewBuilder::composite()`(fork の公開唯一の取り出し口)は呼び出し毎に unmultiply→srgb→premultiply を焼き込む。一括経路では sRGB タグ付き main_target 内で線形合成が1回で済むものが、逐次経路では layer 毎に gamma 符号化済みの値を固定機能 blend で混ぜることになり数学的に別物になる
- fork の `ViewTargetSetup.main_target_*` は private・accessor 無し(`view_builder.rs:52-67`、pinned rev `483b8559` で確認)
- 「第二 render pass 禁止」柵は障害ではない(N 個の ViewBuilder+同一 submit は構造的に成立)

## 裁定

**(a) fork(rerun/re_renderer)へ main_target の read アクセサを1本足す** — [[wrapper-over-hack]](2026-08-18 裁定、初出=Rerun カメラ注入)の直接適用。境界に素直な口を1本作る方が、(b) `rectangles.rs` の頂点/フラグメント shader と srgb 数学を自 crate へ複製するより保守面積が桁で小さい。fork は rerun のみ許可(motolii-next-reset)の範囲内。

- 却下 (b): shader 複製はスクラッチ再発明(保守最低限違反)。fork が既に持つ数学を2箇所目に増やす
- BL3(分離可能11モード)/BL4(非分離4モード)は、この accessor 経由で線形空間の dst を読む WGSL として実装する — R9 の切片割りは write-set をこの前提で読み替える

## 影響

- 次レーン **BL1b**: fork へ accessor 追加(fork 側 数行+テスト)→ `render_sequential` の overlap 不一致テスト(#[ignore] 中)を緑化して ignore を外す、が受入条件
- `render_sequential` scaffold と証拠テストは merge 済み(`fe0724d7` 系列)— 削除しない(歴史証拠)
