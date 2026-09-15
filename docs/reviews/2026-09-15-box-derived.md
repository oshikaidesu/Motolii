# 箱から得られる物 — 自走の記録(提案)

2026-09-15 利用者「箱から得られる効果は?」→ 残りの 8 項目の表 →「今から仕事なんでそれぞれ全部自走していけるか?」→ 形を出して「はい」。
すべて提案(取り消せる)。関係の動きは連続性の物差しで跳び 0 を確かめてから次へ。

## 1. カメラが箱を収める

- 先例: Unity Cinemachine の **Group Framing Size**(対象の箱が画面に占める割合、1 で画面いっぱい)、3D ソフトの Frame Selected
- Camera 層に **Framing Size**(0 は使わない)。Target の層の箱の中心を注視点にし、注視点の面での倍率 = Zoom / Distance から、箱の縦横のきつい方がその割合になる Distance を解く(Orbit は箱の測りに入れない)
- Target を Hold で替えると、Camera 層の **Transition** で、注視点・奥行き・Distance の対数を区間の重みで混ぜて箱から箱へ移る(Camera の欄に Transition Duration / Easing を出す)
- 見本 `camera_frames.js`(格子に並べたカードの板を、カメラが 01 → 03 → 06 → 08 → 05 → 02 と渡り、最後に板全体を収める): 跳び 0・尖り 0(物差しにカメラの注視点と距離を足した)
- 試験: `framing_size_fits_the_target_box_on_screen_and_follows_it`
