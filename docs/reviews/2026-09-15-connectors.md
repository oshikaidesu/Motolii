# 箱と箱をつなぐ線 — 第一級の機能

2026-09-15 利用者裁定「箱と箱を繋ぐ線は第一級の機能。特に破線や垂れた線など、物理を出すのに適切。ネイティブになるとかなり面白い、組み合わせの数が膨大になるため」。きっかけは花の PV(@manwon_man が紹介、yuk.aji)の、切り抜きカードの下を曲線がつなぐ画。

## 先例

| 先例 | 形 |
|---|---|
| leader-line.js | start / end の要素、socket(top / right / bottom / left / auto)、path(straight / arc / fluid / magnet / grid)、dash |
| FigJam のコネクタ | 物に付く端、直線・曲線・折れ線、破線、矢印 |
| Keynote の接続線 | 物を動かしても付いたまま、直線・曲線・角 |
| AE の Stroke > Dashes | Dash・Gap・Offset(Offset に鍵で流れる破線) |

## 法(案、実装済み)

1. **形の層が `Connect From` と `Connect To` を持つと、輪郭は 2 つの箱から毎コマ解いた道に差し替わる**。色・太さ・Trim Paths は形のまま。道は線の層の親の空間で解き、層の変換は道をそのまま置く
2. 端: `From Side` / `To Side`(Auto / Top / Right / Bottom / Left / Center)。Auto は相手の中心へ向かう線が箱を出る所(相手が回っても跳ばない)。箱からの隙間は線の `Margin`
3. `Line Path`: Straight / Curved(端の向きへ伸ばした 3 次)/ Elbow(軸に沿って折る)/ **Hang(懸垂線、縄の長さ = 端の距離 × (1 + Slack %)、閉じた式で時刻の純関数)**
4. 破線は `Dash` / `Dash Gap` / `Dash Offset`
5. 線の `Transition` は端を箱に付けたまま、腹(弦からのずれ)だけを遅らせる(揺れの遅れ)
6. 相手の箱は付いて置く札と同じ口(Blob Track の層は塊、並べて伸ばした形は伸ばした後)

見本 `connected_cards.js`: 跳び 0・尖り 0。

判定語: 決定(利用者裁定)、形の細部は提案。
