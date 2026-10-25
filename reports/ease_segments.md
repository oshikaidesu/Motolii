# 区間イーズを伸ばす（10/25）

区間イーズ＝キーとキーの間を、時間に対してどう進ませるか。Alight Motion はそれを 9 つの型に固めて止めている。Motolii はその先へ伸ばす。下はお手本から集めた「区間の測り方」。

## 土台：Alight Motion（実機で取った）
- 型：ベジェ・Bounce・Elastic・Cyclic・Random・Steps・Elastic Steps（と直線・止める）。区間ごとに 1 本。
- 既定の曲線と取っ手：[alight_ease/default_curves.csv](alight_ease/default_curves.csv)、[default_handles.json](alight_ease/default_handles.json)。
- 詰め切れていない所（まねない）：段々系は最後の 1 コマで 1 へ跳ぶ。Random は 0〜1 の外へはみ出したまま。区間をまたいだ見方・直し方が無い。

## お手本の測り方

| 測り方 | 中身 | 出典 |
|---|---|---|
| 式の族 | Sine・Quad・Cubic・Quart・Quint・Expo・Circ・Back・Bounce・Elastic、それぞれ In / Out / In-Out | Blender（Interpolation、Easing、Dynamic Effects）、GSAP（power1〜4・back・bounce・circ・elastic・expo・sine）、Penner（Motolii は Godot の Tween で借りている） |
| 族のつまみ | Back の行き過ぎ量、Elastic の Amplitude・Period | Blender |
| 点で描く | 点を並べて、どんな曲線でも近似する。点ごとに時間の位置、同じ値を 2 か所で平らな所 | CSS Easing 2 の `linear()` |
| 道で描く | 曲線を道（path）で描いて名前を付けて使い回す | GSAP CustomEase |
| 段々の跳び方 | `steps(n, jump-start / jump-end / jump-none / jump-both)`：最初・最後・中だけ・両端のどこで跳ぶか | CSS Easing 2 |
| ばね | stiffness・damping・mass・velocity・bounce・visualDuration・restSpeed・restDelta。今の速さを引き継いで始められる | Motion（motion.dev）の spring |
| ばらつき | 元の型（template）にばらつきを足す：強さ・点の数・始めと終わりで弱める（taper）・毎回変える・0〜1 に収める（clamp） | GSAP RoughEase |
| 揺れ | 揺れの回数、型（easeOut・easeInOut・anticipate・uniform・random） | GSAP CustomWiggle |
| 跳ね | 強さ、始めで終わる、跳ねた瞬間に潰れる量（squash） | GSAP CustomBounce |
| 真ん中を遅く | 始めと終わりの速さは保ったまま、真ん中だけゆっくり | GSAP SlowMo |
| 大きさの区間 | 倍率の区間を指数で（1→2 と 0.5→7 が同じ手触り） | GSAP ExpoScaleEase |
| 速さで決める | キーごとに入る速さ・出る速さと影響度（%）。速さのグラフは値のグラフの微分 | After Effects（速度グラフ、キーの速度） |
| 区間をまたぐ | 真ん中のキーの時間を、速さが揃うように自動でずらす（ロービング） | After Effects |
| 取っ手の型 | Automatic・Auto Clamped・Vector・Aligned・Free | Blender |

## 伸ばす方向（Alight Motion に無いもの）
1. **型を増やす**：式の族と、そのつまみ（Blender・GSAP・Penner）。
2. **型に縛られない**：点で描く（`linear()`）、道で描く（CustomEase）、利用者の式（Motolii の式のプリセットの仕組みがもうある）。
3. **型を重ねる**：元の型にばらつき・揺れ・跳ねを足す（RoughEase・CustomWiggle・CustomBounce の template / taper / clamp）。
4. **区間をまたぐ**：ばねで速さを次の区間へ引き継ぐ、ロービングで速さを揃える、速度グラフで複数の区間をまとめて見て直す。
5. **端の扱いを選べる**：段々の跳び方（jump-*）、0〜1 に収めるか（clamp）。Alight Motion の「最後に跳ぶ」「はみ出したまま」はここで直る。
6. **値の型に合わせる**：大きさは指数で（ExpoScale）。

## 出典
- [CSS Easing Functions Level 2（W3C）](https://www.w3.org/TR/css-easing-2/)
- [GSAP Eases](https://gsap.com/docs/v3/Eases/)
- [Motion spring](https://motion.dev/docs/spring)
- [Blender Manual: F-Curve Properties](https://docs.blender.org/manual/en/latest/editors/graph_editor/fcurves/properties.html)
- [Adobe: Speed between keyframes](https://helpx.adobe.com/ca/after-effects/desktop/animate-in-after-effects/speed-between-keyframes/speed.html)
