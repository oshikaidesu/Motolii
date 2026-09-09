# Blend desk — 見本で選ぶ

2026-09-09 利用者裁定。目的は**選んだ材料が混ざった結果を見る**ことで、用語を読むことではない。候補ごとに合成を回すのは費用の面で却下された。

## 形

- 札 1 枚 = 1 mode。札の絵が見本そのもの — 選んだ層の色を `blend_preview.rs` の `BEDS`(黒→白の 4 段 + 青 + 橙)に置いた帯。`snapshot.rs` が選択中の層にだけ `blendPreviews` を載せる既存の経路をそのまま使う。GPU・媒体の復号・フレーム連動の生成はしない。
- 札は W3C Compositing の族順(normal → darken → lighten → contrast → inversion → component)。見出しは置かない。隣り合う札の絵が同じ性格を示すので、族は帯の見た目で分かる。
- 文字は mode 名だけ。説明・題・札の外の見出し・preset の棚は持たない。Desk 側が既に題を出す。
- 狭い Desk(最小 260×160)で成立する密度: 札は幅 70 以上で列数を箱幅から決め、高さは 36(帯 22 + 名前)。

## 操作

指を乗せる → `previewBlend`(90ms まとめ、最新のものだけ送る)で Stage に出る。外す・Escape・選択替え → `cancelPreview`。押す → `setAttrs` 1 回。port は preview 系以外の op の前に preview を畳むので、**Undo は 1 回**で消える。

`previewBlend` は port では層 1 枚(`layer`)。見本は選択の最後の層に出し、確定は選択全部に効く。カメラと施錠された層は対象外。
