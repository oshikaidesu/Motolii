# Colour 再検証 r2 (2026-10-02)

方法: lab_book を実機で操作、等倍撮影 + zoom。コード変更なし。tool-results dir =
/Users/member_ottoto/.claude/projects/-Users-member-ottoto-rust-ae-Motolii/77f277f7-741f-4e9f-8e94-ddd760f7f088/tool-results/ (以下は blob 名)。

| # | 項目 | 判定 | 証拠 (Palette B) |
|---|---|---|---|
| 1 | Timeline Face(黄)バー上の key 菱形 | FIXED | 菱形に暗い縁が付き、黄バー上でも見える。blob-1790911507293-orgxk6.jpg (zoom ...509006-avok5b) |
| 2 | I4 Animate on: 静止行の菱形 / On スイッチ / A の色 | FIXED | B: 静止行は落ち着いた青灰の白抜き。On は暗い板 + 細い菱の輪 + 紫の knob(赤茶の塗り無し)。blob-...524742-s4leuw.jpg。A: 紫青、ピンク無し blob-...532495-1vvr3t.jpg。Grey blob-...537152-ftxscy.jpg。A+deut blob-...542159-1hixbf.jpg |
| 3 | Browser B2 / B5 が Colour に反応 / 空の「!」 | 部分的 FIXED | B2 有効チップ下線は Grey=紫、B=青で反応(細い 1px、zoom ...475456 B / ...484702 Grey)。B2 Ableton blob-...473889-bmbulp.jpg。B3-a no match は橙の「!」付き blob-...494676-wvugyo.jpg。B5 own frame は破線枠が青、選択候補の左棒は白のまま blob-...465589-iczf60.jpg。**B2 zero results の空状態は照準アイコンのみで「!」無し**(blob-...490667-aocw5x.jpg) |
| 4 | I2 error: 赤 + 「!」、Follow 橙と別 | FIXED | 枠は赤、数値の左に丸「!」、Follow 点は橙で明確に別。blob-...551779-0vp0kq.jpg |
| 5 | I9 Lasso 外枠が選択色、Mode switch の下線 | FIXED | Lasso の枠・選択点が青 blob-...519119-j1myjw.jpg。Mode switch は EDIT に青の下線 blob-...562051-97h4p6.jpg(Grey は下線無し)。Whip(I9-a)は橙のまま(家族色なので意図通り)blob-...515510-lyydei.jpg |
| 6 | mixed と changed の区別 | FIXED | B: 複数行は「—」のみ・下線/角印/青無し、変更行は青ラベル+青値+角印。Grey は両方に角印+点線下線が付き同じ見た目(従来通り) |
| 7 | Grey 回帰 | OK | Grey の I4 / I2 / Mode switch に新しい印は無い(I2 error に「!」無し、Mode 下線無し)。blob-...573535-nnbvl3.jpg |

## 新しい不具合
- B2 zero results の空状態に「!」が無い(B3-a no match には有る)。印の付け漏れ。
- B5 の選択候補の左棒・B2 のリスト選択棒は白のまま(selected 色に反応しない)。軽微。
- B2 の有効チップ反応は 1px の下線の色だけで弱い(塗りは灰のまま)。

## 判定
- Palette B: RECOMMENDED(旧指摘 1,2,4,7 は解消。残りは上の軽微 3 点)。
- Palette A: 条件付き OK。ピンク衝突と赤茶スイッチは解消し、紫青で B と同等に落ち着く。key 黄は B より強いがもう Face 上でも見える。deut で changed(青)と mode(紫青)の差は小さい(形で分かれる)。

## 保存した画像 (research/shots/)
colour-r2-timeline-B.jpg, colour-r2-I4-B.jpg, colour-r2-I4-A.jpg, colour-r2-I2-error-B.jpg, colour-r2-B2-B.jpg, colour-r2-I9-lasso-B.jpg
