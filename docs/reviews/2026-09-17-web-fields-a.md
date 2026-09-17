# Web の欄 a — Clip 辺・Split・Loop(2026-09-17)

[Web の写し](2026-09-17-web-clone-gallery.md) の「要る欄」のうち、台本を短くする 3 つを足した。芯は利用者の原則: **ソフトが持つのは意味でなく道具、欄の名前は Web の語、値は時刻の純関数、Web の意味(既定込み)をそのまま写す、離散の切替の後に連続の収まりが置ける**。

## 1. Clip 辺 — `clip-path: inset(top right bottom left round r)`

- **意図**: 帯で開く・拭き取る・角から拭く・道の残像・面が四隅へ開く(s2・s3・s5・c7・c10)。今までは 1 辺を動かすのに **鍵 3 本**(箱の Height + 箱の Position + 中身の Position)で「遠い辺を留める」形にしていた。
- **札**: 並べる Group の欄 `Clip Top / Clip Right / Clip Bottom / Clip Left`(CSS の inset の順、px、既定 0、辺ごとに鍵が打てる、負も可 = CSS)と `Clip Radius`(= inset の `round`、既定 0)。
- **先例**: CSS `clip-path: inset()`(CSS Masking 1 §6 basic-shape)。`inset()` の `round` は `border-radius` と別の値なので、Overflow Clip の角(Border Radius)と Clip Radius は別の欄。clip-path は**要素ごと**に掛かる(自分の背景も切る)、overflow は箱の外の子孫だけ。
- **借りた物**: 既にあった Overflow Clip の切り方(`layout.rs clip_box` → `resolve.rs clipped_masks` が祖先の箱を Intersect の mask で子孫へ渡す)。**足した物**: `clip_inset`(4 辺で削った箱、Clip Radius)、`clipped_masks` が自分自身の inset も切る。描く側は触っていない(mask は既存)。
- **絵**: s2・s3・s5・c7・c10 の evidence は差し替え前と同じ絵(見比べて確認)。s3 は Web の通り「上の画を右から削る」形に戻せた(Width の鍵で要った回避 = 詰まった所 2・3 が消えた)。c10 は箱の Background が inset で切れる(CSS の通り)。
- **test**: `clip_inset_cuts_the_box_from_each_edge_like_css_inset`(`inset(10% 0 0 0)` の 400×300 → 上 30 px が消える、CSS の辺の順、辺が向かいを越えたら空、Overflow Clip と inset は 2 つの切り)。
- **仕様が開けている所(決めていない)**: (1) `%` の単位 — 値の仕組みに % が無いので px だけ(台本は箱の寸法を知っているので写せた)。(2) inset は Display の Group だけ(箱の大きさは並べる箱から取る、Display None の Group・文字・画の層には効かない)。(3) `clip-path` の他の形(circle / ellipse / polygon / path)は無し(s5・c10 の polygon は辺の enum で足りた)。(4) 3D の層への切り口(既存の Overflow と同じ制限)。
