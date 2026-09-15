# Cavalry をなぞってささくれを取る

2026-09-15。利用者「cavalry の例を色々見てトレースしながらささくれをとっていきましょう」。例は Cavalry の公式の見本(https://cavalry.studio/docs/getting-started/example-files/)とコミュニティの解説から 10 本を選び、軽い順になぞる。スクリプトはスクラッチ(`cav/*.js`)。

| # | 例 | 状態 |
|---|---|---|
| 1 | Concentrick | なぞった(跳び 0) |
| 2 | Infrequency | |
| 3 | Falloff の色の格子 | |
| 4 | 字ごとの跳ねる波 | |
| 5 | Ring Ting | |
| 6 | 数を数える・打つ文字 | |
| 7 | 棒グラフ | |
| 8 | Mazin | |
| 9 | ボタンの影 | |
| 10 | FUI の輪と線 | |

## 1. Concentrick

Cavalry: Circle を Duplicator(Point Distribution = 全部 0,0)、Stagger で番号ごとに半径、Oscillator で位置を揺らす。
Motolii: 墨と紙の円を Group に、Repeater(Pick = Iterate、Position Each 0、Scale Each −9 %、Delay Each −0.05 秒)、揺れは Position の鍵の Cyclic。

| ささくれ | 事実 | 処置 |
|---|---|---|
| **引いた子の写しが番号の順に重ならない** | Repeater が Group の子を引く時、最後の並べ替えが子の order で、同じ子の写しが全部まとまって重なった(墨 12 枚の上に紙 12 枚) | 直した: 引いた写しは子の一番下の order に揃え、番号の順のまま。試験 `picked_copies_stack_by_their_number_not_by_child` |
| **同じ order の写しの描き順が揺れる** | 描き順の鍵は `placement.order`。写し同士は同点で、描く側の並べ替えに委ねられていた | 直した: 解いた層を並べた後、描き順を並べた順の番号に振り直す(同点を作らない) |
| Delay Each が出る時刻まで遅らせる | Cavalry の Time Offset は動きだけずらすが、Motolii の Delay は写しの帯ごとずらす(遅れた写しは始めに居ない) | 回避: 負の Delay と、子の帯を尺より長く。**宿題**(Time Offset の意味の写しを足すか) |
| Scale Each は掛け算で増える | `(1 + s)^i`。Cavalry の Stagger は最小〜最大を番号で線形(曲線つき)に振る | 未(トンネルに見える。線形の振り方は Stagger を写す時に) |
| 円を線だけにする口が無い | スクリプトと欄から、塗りを消す・線の色を変えられない(色の lane の所管) | 回避: 墨と紙の円を交互に重ねる。**色の lane へ回す** |
| 揺らぎは鍵の Cyclic で書ける | Oscillator の代わりになる(周期は区間の割合) | ささくれでない(記録だけ) |
