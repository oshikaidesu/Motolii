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

## 借りる先（中身とライセンスを確かめた）

| 測り方 | 借りる先 | ライセンス | 中身 |
|---|---|---|---|
| 式の族＋つまみ（Power・Quad〜Quint・Sine・Circ・Expo・Bounce・Back(overshoot)・Elastic(amplitude, period)、in / out / inOut / outIn） | anime.js `src/easings/eases/parser.js` | MIT | Penner（2001）の式。outIn まである |
| 点で描く `linear()`（% で時間の位置） | anime.js `src/easings/linear/index.js` | MIT | CSS `linear()` と同じ書き方 |
| 段々 | anime.js `src/easings/steps/index.js` | MIT | `steps(n, fromStart)`（跳び方は 2 つだけ。jump-none・jump-both は CSS Easing 2 の定義から足す） |
| ばらつき | anime.js `src/easings/irregular/index.js` | MIT | `irregular(length, randomness)`：0〜1 に収めたまま増え続ける点を `linear()` で結ぶ（RoughEase の代わり） |
| ばね | anime.js `src/easings/spring/index.js`、Motion `packages/motion-dom/src/animation/generators/spring.ts` | MIT | bounce（−1〜1、SwiftUI と同じ減衰比への写し方）・知覚の長さ、または stiffness・damping・mass・velocity |
| ベジェ | gre/bezier-easing `src/index.js` | MIT | 3 次式を閉じた形で解く（Baranovskiy の解き方） |
| 速さ・影響度（AE） | bodymovin-extension `bundle/jsx/utils/keyframeHelper.jsx` | MIT | 出る側：x＝影響度/100、y＝速さ/平均の速さ×影響度/100。入る側：x＝1−影響度/100、y＝1−速さ/平均の速さ×影響度/100（AE のキーを Lottie のベジェに直す所） |
| GSAP（RoughEase・CustomWiggle・CustomBounce・SlowMo・ExpoScale） | 借りない | 独自（「All rights reserved」、OSI でない） | 考え方だけ表に残す。ばらつきは anime.js の irregular、揺れ・跳ねは Penner と spring で作る |

## Motolii の区間の持ち方と、入れ方
- 今：区間ごとに「型＋数字 5 つ」（`filament/interp.h` の 9 型）か「名前の付いた道」（`filament/ways.h`、Cavalry の Magic Easing の名前、SQLite の式）か「利用者の式」（ways/ の 1 ファイル＝1 式、`bezier()` が使える）。式は SQLite とスクリプト（QuickJS）の両方で同じ文字が動く。
- **anime.js をそのまま QuickJS で動かす**のが一番借りる形（Motolii は QuickJS をもう持っている）。式を写さず、区間の型の名前とつまみで anime.js の関数を呼ぶ。
- 写すなら、SQLite の 1 式に収まる物（Penner の族、Back、Elastic、steps、ばねの閉じた形）は `ways.h` に出典つきで。点の並び（`linear()`・irregular）は 1 式に収まらないので QuickJS 側。
- Alight Motion の 6 種は、この上で「既定の数字の組」として持つ（`alight_ease/` の曲線に合わせる）。

## 出典
- [CSS Easing Functions Level 2（W3C）](https://www.w3.org/TR/css-easing-2/)
- [GSAP Eases](https://gsap.com/docs/v3/Eases/)
- [Motion spring](https://motion.dev/docs/spring)
- [Blender Manual: F-Curve Properties](https://docs.blender.org/manual/en/latest/editors/graph_editor/fcurves/properties.html)
- [Adobe: Speed between keyframes](https://helpx.adobe.com/ca/after-effects/desktop/animate-in-after-effects/speed-between-keyframes/speed.html)
- [anime.js（MIT）](https://github.com/juliangarnier/anime)
- [Motion（MIT）](https://github.com/motiondivision/motion)
- [bezier-easing（MIT）](https://github.com/gre/bezier-easing)
- [bodymovin-extension（MIT）](https://github.com/bodymovin/bodymovin-extension)
- [GSAP の README（独自ライセンス）](https://github.com/greensock/GSAP)
