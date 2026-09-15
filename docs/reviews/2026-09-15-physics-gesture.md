# 物理は身振りに畳む(未決の考え)

2026-09-15。「Sync」の物理を嘘で作った見本(Overflow = Bounce)に、利用者「しょぼい、いちど motolii のことは忘れて新世代型ソフトの物理演算の理想について考えましょう。嘘も」。
続けて「人間はそこまでのパラメータを操作しながら日常生活をしているか? 違う。新しい gui が必要だ」「y 字のゴムパチンコ、スリングショット」。
判定語: 未決(意味の発明が大きい。利用者「かなり難しい意味の発明だ」)。

## 1 行

**物理の数字は、人の身振り 1 つ(引いて、放す)に畳める。**

スリングショット: 物を掴んで飛ばしたい向きと逆へ引く → 伸びたゴムが強さ → 引いている間、飛ぶ弧が見える → 放すと未来が決まる。人は初速も角度も考えず、手と目が逆算する。

確かでない物(空想のまま): 放った出来事を時間軸に残す、飛んでいる途中で掴み直す、弧の着地や時刻が狙いや拍に吸い付く嘘。

## Cavalry はどうしているか(2026-09-15 取説で確かめた)

出典: [Forge Dynamics Shape](https://cavalry.studio/docs/nodes/shapes/forge-dynamics/forge-dynamics-shape/)、[Dynamics menu](https://cavalry.studio/docs/user-interface/menus/dynamics-menu/)、[Fields](https://cavalry.studio/docs/nodes/shapes/forge-dynamics/fields/)、[Collision Events](https://cavalry.studio/docs/nodes/shapes/forge-dynamics/collision-events/)

- **2D だけ**。Box2D の上の剛体。Make Dynamic で選んだ形を Solver(Forge Dynamics Shape)の Bodies につなぐ
- 身振りは無い。**全部が欄**: 物ごとに Friction、Bounce、Density、Gravity Scale、Starting Velocity、Starting Rotational Velocity、Collision Shape Type(Box / Circle / Polygon / Chain / Soft Body)、Collision Groups。Solver に Gravity、Ground Mode(Off / Composition Bottom / Composition Edges)、Time Step、Start Frame
- 鍵と物理の橋は Body Type: Dynamic / Still / Kinematic(鍵のまま動き、他を押す)/ **Kinematic Hybrid(最後の鍵まで鍵で動き、そこから物理に入る)**。「放す」に一番近いのはこれ — 鍵で引いて、最後の鍵で放す
- 力は Field(Attractor / Buoyancy / Direction / Drag / Path / Vortex)。ぶつかった時の変化は Collision Event(色・力・物の設定・くっつく)。「Sync」は Color Collision Event と Attractor Field を使っている
- 時間: 積み上げの計算。Cache Solver で `.sdcache` に焼き、Cache Offset の鍵でずらしてループを作る。巻き戻しの振る舞いは取説に書かれていない

つまり Cavalry は物理を正しく持ち、作り手は欄と cache で付き合う。行き先や拍を約束する口は無い。

## 2D と 3D で意味が変わる所

| | 2D | 3D |
|---|---|---|
| 引く向き | 画面の上の向きがそのまま飛ぶ向き(Angry Birds) | 画面の上の 2 成分しか取れない。奥行きをどこから取るかが要る |
| 本物のスリングショットに近いのは | 横に引く見立て | **手前(見る人の方)へ引いて奥へ放つ** — 本物と同じ。引く量が奥行きの強さ |
| 下(重力) | 画面の下 | 世界の下。カメラが傾くと画面の下とずれる |
| 壁 | 箱の辺、画面の端(Cavalry の Composition Edges) | 箱の面。奥行きを持つ箱 |
| 弧の見え方 | 平面の曲線 | 遠近で縮む曲線。引いている間にカメラから弧の着地が見えない所がある |

聞くこと: 3D で引く面は、物の面(箱が乗る面)か、カメラに向いた面か、地面か。

## 変数の抽象化と削減(2026-09-15 利用者)

利用者「物理を行う絵で行うべきは変数の抽象化及び削減です」。スリングショットは言い過ぎ(日常の物理は放つより置く・押す・落とす・振る。行き先はだいたい決まっていて、物理は着き方)。

削り方の案:
- **物理の軸でなく、感じる軸へ**: 重さ(密度・重力・減衰)、弾み(跳ね返り・摩擦)、元気(揺らす強さ)。先例: iOS のばね `spring(duration:bounce:)` は質量・剛性・減衰の 3 つを 2 つにした
- **時間の言葉で**: 剛性でなく「かかる時間」
- **文脈から取る**: 壁は住む箱、速さの目安は箱の大きさ、行き先は配置と鍵
- **0 個で見られる既定**: 何も触らなくても、もっともらしく着く
