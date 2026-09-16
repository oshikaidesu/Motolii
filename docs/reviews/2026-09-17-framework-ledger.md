# フレームワークの抽象の台帳(4)— 何を抽象化したか、何が標準に吸われたか、何が記憶を持つか

2026-09-17 調査(公式の取説から)。対象: GSAP・Motion(旧 Framer Motion)・anime.js v4・Web Animations API、Remotion(コマが唯一の正、深く)・Theatre.js・Rive・Lenis。軸は「表現」ではなく **抽象の語彙**。各語彙を、純関数か記憶か・CSS/標準の対応語・意味を貸すか・Motolii の台本にあるか、で読む。[台帳 3(表現別)](2026-09-17-expression-ledger.md) の続き。

利用者(2026-09-17)「ハック的な手法の形骸化は Web にもある。次に見るべきはいかに抽象化したか」「Remotion は面白くない」— 正しさだけでは絵は出ない。時刻が正なのは形式(rrd)の側で、触る側は絵を見ながら押せないといけない。

---

# Web アニメ枠組み 4 つの語彙 — GSAP / Motion / anime.js v4 / WAAPI(2026-09-17)

篩: **純/記憶** = 時刻の純関数か、前フレームや DOM の状態を持つか。**CSS/標準** = 対応する CSS・WAAPI の語。**意味** = 完成した見た目・名前付きの型を貸すか。**Motolii** = 台本の語(`comp/text/rectangle/group/.set/.key(s)/.effect/.time/.projection`、欄名は例の `.set("…")`)で既にあるか。

## 1. GSAP v3(+ ScrollTrigger / Flip / SplitText、3.13 で全部無償)

| 語彙 | 引数 | 純/記憶 | CSS/標準 | 意味 | Motolii |
|---|---|---|---|---|---|
| `gsap.to/from/fromTo/set` | target, `{duration, delay, ease, repeat, yoyo, repeatDelay, stagger, keyframes, immediateRender, overwrite, snap, modifiers, onUpdate}` | 純(from は現在値を DOM から読む = 記憶) | `@keyframes` + `animation-*`(duration/delay/iteration/direction) | ease 名を貸す | `.key/.keys(name, [[t, v, "Bezier"]])` = fromTo。from(現在値から)は無 |
| `gsap.timeline({defaults, repeat, yoyo, paused, smoothChildTiming})` | `.to(..., position)`: `3`, `"+=1"`, `"-=1"`, `"<"`, `">"`, `"label+=2"`, `"<25%"`; `.addLabel/.call/.tweenTo/.seek/.progress/.time/.timeScale` | 純(子の時刻の和) | 無(WAAPI Level 2 の GroupEffect/SequenceEffect は未実装) | 無 | 層の in 点 + `.time(a, len)`。**相対位置 `<`/`>`/label が無い**(数値を手で足す) |
| `stagger` | `0.1` / `{each, amount, from: 'start'|'center'|'end'|'edges'|'random'|index|[x,y], grid: [rows,cols]|'auto', axis, ease, repeat, yoyo}` / `(i, el, list) => delay` | 純(番号 → 遅れ) | 台帳 E `sibling-index()`、無(CSS に from/grid は無い) | from/grid が「散り方」を貸すが数値 | `"Stagger"`, `"Stagger From": "Center"`(Arrive/箱の札)。**edges/random/[x,y]/grid/axis/ease/関数** が無 |
| `ease` | `"power1-4.in/out/inOut"`, `back(1.7)`, `elastic(1,0.3)`, `bounce`, `circ`, `expo`, `sine`, `steps(n)`, `rough`, `slow`, `expoScale`, `CustomEase`, 3.13 `easeReverse` | 純 | `animation-timing-function` = cubic-bezier / `linear()` / `steps()`。elastic/bounce は `linear()` で近似 | **貸す**(名前 = 完成した動きの型) | `"Bezier"/"Linear"/"Hold"` + `"Transition Easing": "Ease In Out"`。名前の棚は無(Bezier の手打ち) |
| `ScrollTrigger` | `{trigger, start:"top bottom", end, scrub: true|秒, pin, pinSpacing, snap, toggleActions:"play none none reverse", onEnter/…Back, once, markers, containerAnimation, horizontal}` | `scrub:true` 純(s→t)。`scrub:0.5` = **lerp の記憶**。toggleActions = 状態機械 | `animation-timeline: scroll()/view()`, `animation-range` | 無 | scroll = 時刻で済(台帳 2 法 3)。pin・toggle は動画に無い |
| `Flip.getState / Flip.from(state, {duration, ease, absolute, nested, scale, simple, onEnter, onLeave, spin, fade, prune})`, `Flip.fit(a, b, {scale, fitChild})`, `data-flip-id` | 前後の矩形を測って transform で差を消す | **記憶**(直前の DOM 矩形を保持) | 無(`view-transition` が近い) | 無(矩形の写像) | 済: `"Transition Duration/Easing"`(間合いの法、状態を持たない Transition)。`fit`(別の箱に合わせる)は無 |
| `SplitText.create(el, {type:"chars,words,lines", mask, autoSplit, aria, charsClass, deepSlice, smartWrap, ignore})` → `.chars/.words/.lines/.masks`, `.revert()` | 文字を箱に割る | 純(字列と font の関数) | 無(CSS は `::first-letter` まで) | type 3 種と mask(行の overflow:hidden)= 道具 | **無**: text の Each(字/語/行)が台本に見えない。mask は Clip で済 |

見立て: (1) 持ち上げた hack = **timeline の相対位置(`<`/label)と stagger の `from/grid`**、Flip(DOM の矩形差)、SplitText(DOM の字割り)。(2) 標準に吸われた = scrub → `animation-timeline`、ease → `linear()`/`steps()`、Flip → View Transitions。(3) Motolii に無い = 相対位置の口(`"<"`/label)、stagger の `from: edges/random/[x,y]`・`grid`・関数、text の chars/words/lines の Each、ease の名前の棚。

## 2. Motion(旧 Framer Motion、motion.dev)

| 語彙 | 引数 | 純/記憶 | CSS/標準 | 意味 | Motolii |
|---|---|---|---|---|---|
| `<motion.div initial animate exit>` | 値 obj / variant 名 / `[0,100,0]`(null = 前値を保つ) | animate は **状態遷移**(現在値から目標へ、時刻の関数でない) | `transition`(CSS の状態遷移) | 無 | 済: `"Transition Duration/Easing"`(値が変わったら間合いで追う)。動画は key で書く |
| `transition` | `{type:'tween'|'spring'|'inertia', duration, delay, ease, times, repeat, repeatType:'loop'|'reverse'|'mirror', repeatDelay}` | tween 純 | `transition-*`, `animation-direction: alternate` = reverse | ease 名(`anticipate`, `backInOut`, `circIn`…)を貸す | mirror(値も逆)は無、往復の key で済 |
| `spring` | `{stiffness=1, damping=10, mass=1, velocity, bounce=0.25, visualDuration, restSpeed, restDelta}` | **記憶**(積分器。ただし目標固定なら閉形式 = 純関数に写せる) | `linear()` に焼く(Motion 自身が WAAPI へ焼く) | 無(数値 3 つ) | physics の留め具(重さ/硬さ/ばらつき)。`bounce+visualDuration` の言い換えは Arrive の `"Bounce"/"Arrive"` に近い |
| `inertia` | `{velocity, power, timeConstant, min, max, bounceStiffness, bounceDamping, modifyTarget}` | 記憶(速度から減速) | 無 | 無 | 無(drag の手放し用、動画では不要) |
| `variants` + orchestration | `{when:'beforeChildren'|'afterChildren', delayChildren, staggerChildren, staggerDirection}`、dynamic variant `(custom) => {}` | 純(親の名前 → 子へ伝播) | 無 | 無 | 箱の札が子の時刻をずらす(Stagger)= staggerChildren。**名前付き状態の伝播**は無(動画は時刻で足りる) |
| `layout` / `layoutId` / `LayoutGroup` / `layoutRoot` / `layoutAnchor{x,y}` | `true|'position'|'size'|'preserve-aspect'`、scale correction | **記憶**(FLIP、前レイアウトを保持) | `view-transition-name` | 無 | 済: 間合いの法(Transition は状態を持たない)。`'position'|'size'` の絞りと共有要素(layoutId)は無 |
| `AnimatePresence` | `mode:'wait'|'popLayout'|'sync', initial, custom, propagate, onExitComplete` | **記憶**(消えた要素を DOM に残す = 退場状態) | 無 | 無 | 層の out 点 = 純関数。要らない |
| `whileHover/Tap/Focus/Drag/InView` + `viewport{once, amount, margin}` | ゲート | 状態機械 | `:hover`, IntersectionObserver | 無 | 層の in 点で済 |
| `animate(target, keyframes, {at:'<'|'+0.5'|label, stagger(0.1,{from, startDelay, ease})})`(sequence) | vanilla 版 timeline、`.time/.speed/.play/.pause/.then` | 純 | WAAPI(mini animate は `Element.animate` そのもの) | 無 | timeline と同じ: `at` の相対語が無 |
| `useScroll/useTransform/useSpring/useVelocity/useMotionValue`, `transform(v,[in],[out])`, `mix/wrap/clamp` | 値 → 値 | useSpring/useVelocity は記憶、transform は純 | `calc()/clamp()/mod()`(台帳 2 G) | 無 | 台帳 2 G の関数。**「別の値から値を作る」口(useTransform)**は台本に無 |

見立て: (1) 持ち上げた hack = **React の状態 → 動きの自動遷移**(animate/variants/AnimatePresence/layout)と spring の `bounce+visualDuration`。(2) 標準へ = mini animate は WAAPI そのもの、layout は View Transitions、ease は `linear()`。(3) Motolii に無い = `useTransform`(値の写像)の口、`layout:'position'|'size'` の絞り、共有要素。状態系は動画では in/out 点と key に落ちる。

## 3. anime.js v4

| 語彙 | 引数 | 純/記憶 | CSS/標準 | 意味 | Motolii |
|---|---|---|---|---|---|
| `animate(targets, {to, from, duration, delay, ease, loop, alternate, reversed, loopDelay, autoplay, frameRate, playbackRate, playbackEase, composition:'replace'|'none'|'blend', modifier, keyframes[]/{'50%':…}})` | 関数値 `(el, i, total) => v` | 純(composition:'blend' は前の tween と足す) | `animation-*`、`composite: replace|add|accumulate` = composition | ease 名を貸す | `.key(s)`。**composition(重ね方)**は無、`playbackEase`(全体を歪める)= `.time` の remap に近いが無 |
| `ease` | `'linear'`, `'in/out/inOut(power)'`, `'outElastic(amp, period)'`, `'outBounce'`, `'steps(n)'`, `'cubicBezier(…)'`, `'irregular(len, rand)'`, `createSpring({mass, stiffness, damping, velocity})` | spring は**期間を先に解いて**純関数化(anime は duration を計算する) | `linear()`/`steps()`/cubic-bezier | **貸す** | 上と同じ、名前の棚が無。spring→duration の解法は Arrive/留め具に使える |
| `stagger(value, {start, from:'first'|'center'|'last'|index, reversed, grid:[cols,rows], axis:'x'|'y', ease, modifier, total, use})` | value は数・`[a,b]` の範囲・単位付き文字列。**値にも使える**(delay 以外に `translateX: stagger(10)`) | 純 | 台帳 E `sibling-index()` | 無 | `"Stagger"`。**範囲値 `[a,b]`・grid/axis・use(別の性質で番号付け)**が無 |
| `createTimeline({defaults, loop, alternate, playbackEase})` | `.add(t, params, pos)`, `.sync(anim, pos)`, `.set`, `.call`, `.label(name)`, `.stretch(ms)`, `.seek`, `.refresh`; pos = `500`, `'+=500'`, `'-=250'`, `'<'`, `'<<'`, `label`, `'<-=100'` | 純 | 無 | 無 | 相対位置と `stretch`(全体の伸縮)が無 |
| `createTimer`, `createAnimatable`, `createDraggable`, `createScope` | 時計・値の口・drag・media query の枠 | Animatable は現在値から補間 = 記憶 | 無 | 無 | `comp` が時計。要らない |
| `onScroll({container, target, axis, enter, leave, sync: true|'play reverse'|ease|smooth 数, debug})` | scroll = 時計 | sync 数 = lerp の記憶 | `animation-timeline`, `animation-range` | 無 | scroll = 時刻で済 |
| `utils.random/round/clamp/mapRange/lerp/wrap/snap/interpolate/roundPad/get/set/$` | 値の関数 | lerp(x,y,t) は純、`utils.lerp` の「追従」用法は記憶 | `calc/clamp/round/mod/random()`(台帳 2 G) | 無 | 台帳 2 G の候補と同じ |
| `svg.morphTo(path, precision)`, `svg.createDrawable(el)` → `draw: '0 1'`, `svg.createMotionPath(path)` → `{translateX, translateY, rotate}` | 形の補間・線の 0→1・道 | 純 | `d` の補間 / `stroke-dashoffset` / `offset-path` | 無 | Offset Path 済(`"Offset Path": "Border Box"`)。**Trim Path(`draw '0 1'` の始点・終点)と morph** は無(台帳 3 見立て 2) |
| `splitText(el, {lines, words, chars})`, `scrambleText`, `engine.timeUnit/speed/fps/precision` | 字割り、文字化け、時計の単位 | 純 | 無 | scramble は完成形寄り(△) | text の Each 無、glyph 置換の軸 無(台帳 3 #9) |

見立て: (1) 持ち上げた hack = **stagger を値にも使う**(`[a,b]` 範囲・grid)、`composition:'blend'`(加算の重ね)、spring を duration に解く、SVG の draw/morph。(2) 標準へ = composition → WAAPI `composite`、onScroll → `animation-timeline`、utils → CSS math。(3) Motolii に無い = 相対位置・`stretch`、stagger の範囲値/grid/use、Trim Path・morph、`composite` の宣言、`playbackEase`。

## 4. Web Animations API(WAAPI、標準)

| 語彙 | 引数 | 純/記憶 | CSS/標準 | 意味 | Motolii |
|---|---|---|---|---|---|
| `Element.animate(keyframes, options)` / `new KeyframeEffect(target, keyframes, timing)` | keyframes: 配列 `[{transform, offset:0.3, easing, composite}]` か obj `{opacity:[1,0]}`; timing: `{duration(ms), delay, endDelay, iterations, iterationStart, direction:'normal'|'reverse'|'alternate'|'alternate-reverse', easing, fill:'none'|'forwards'|'backwards'|'both', composite, iterationComposite:'replace'|'accumulate', pseudoElement}` | 純(currentTime → 値、状態機械なし。fill は範囲外の値の規則) | = CSS `animation-*` の正本 | 無(`easing` は cubic-bezier/steps/linear() の生の語) | `.keys` = keyframes。**offset(0-1 の相対)・endDelay・iterations/direction・fill** は無(Motolii は絶対秒、繰り返しは `mod`) |
| `composite: 'replace'|'add'|'accumulate'`(effect 全体 / key ごと) | `add` = 下に足す(`blur(2) blur(3)`)、`accumulate` = 量を合算(`blur(5)`) | 純 | CSS `animation-composition` | 無 | **無**: 同じ欄に 2 本のアニメを重ねる宣言(effect 同士の重ね方) |
| `Animation` | `.play/.pause/.reverse/.finish/.cancel`, `.currentTime`, `.startTime`, `.playbackRate`, `.updatePlaybackRate`, `.playState:'idle'|'running'|'paused'|'finished'`, `.ready/.finished`(Promise), `.persist/.commitStyles`, `.replaceState`, `onfinish/oncancel/onremove` | `currentTime` を置けば純。play/pause は状態機械 | = CSS animation-play-state | 無 | `comp` の時刻 = `currentTime`。`playbackRate` = `.time(a, len)` の伸縮 |
| `AnimationTimeline`: `DocumentTimeline`, `ScrollTimeline({source, axis:'block'|'inline'})`, `ViewTimeline({subject, axis, inset})`, `rangeStart/rangeEnd:'entry 0%'|'cover 100%'` | 時計の差し替え | 純(scroll 位置 → 進み) | `animation-timeline: scroll()/view()`, `animation-range` | 無 | scroll = 時刻で済。**時計の差し替え(別の値を時刻にする口)**は台帳 2 法 3 と同じ相談 |
| `effect.getComputedTiming()` → `{progress, currentIteration, activeDuration, endTime}`, `updateTiming`, `getKeyframes/setKeyframes` | 読み出し | 純 | 無 | 無 | 無(台本は書くだけ) |
| `document.getAnimations()`, `el.getAnimations({subtree})` | 列挙 | — | 無 | 無 | 無 |
| `GroupEffect` / `SequenceEffect`(Level 2、未実装) | 並列・直列の木 | 純 | 無 | 無 | timeline 相当が標準にまだ無い = GSAP/anime が残る理由 |

見立て: (1) 持ち上げた hack = CSS animation を JS の物(`Animation`)にして `currentTime` を触れるようにした、`composite` で重ねる。(2) 吸収された側 = これが標準。GSAP の scrub・Motion の mini animate・anime の onScroll は Timeline へ流れる。(3) Motolii に無い = `offset`(相対 key)、`iterations/direction/fill` の繰り返しの宣言、`composite`、時計の差し替え口。timeline(Group/Sequence)は標準にも無い。

## Sources

- GSAP: https://gsap.com/docs/v3/GSAP/Timeline/ ・ https://gsap.com/resources/getting-started/Staggers ・ https://gsap.com/docs/v3/Plugins/ScrollTrigger/ ・ https://gsap.com/docs/v3/Plugins/Flip/ ・ https://gsap.com/docs/v3/Plugins/SplitText/
- Motion: https://motion.dev/docs/animate ・ https://motion.dev/docs/react-animation ・ https://motion.dev/docs/react-transitions ・ https://motion.dev/docs/react-layout-animations(react-motion-component は timeout)
- anime.js v4: https://animejs.com/documentation/ ・ /documentation/stagger ・ /documentation/timeline
- WAAPI: https://developer.mozilla.org/en-US/docs/Web/API/Web_Animations_API/Using_the_Web_Animations_API ・ https://developer.mozilla.org/en-US/docs/Web/API/KeyframeEffect/composite
- Motolii の語彙: motolii/ui/native/src/editor/script/examples/*.js(`.set` の欄名を grep、`"Stagger"/"Stagger From"/"Transition Duration"/"Transition Easing"/"Offset Path"/.time/.projection`)
- 台帳: docs/reviews/2026-09-17-expression-ledger.md ・ 2026-09-17-css-ledger-2.md

---

# Web の動画・時系列枠組み 4 つ — Remotion / Theatre.js / Rive / Lenis(2026-09-17)

篩は台帳 3 と同じ: 時刻の純関数か(記憶=前フレームを持つか)/ 意味を貸すか(preset は意味、法は道具)/ Motolii の口(comp/text/rectangle/group/.set/.keys/.effect/.time)に既にあるか。出典は remotion.dev/docs、theatrejs.com/docs、rive.app/docs、github.com/darkroomengineering/lenis(packages/core/src)。

---

## 1. Remotion(最深)— 動画 = `frame` の純関数

React の component が `useCurrentFrame()` を読み、`<Composition fps width height durationInFrames>` の各コマを headless Chrome で撮って FFmpeg で束ねる。**時計は frame 1 つ**、状態機械なし。「コマ t の絵を出せ」は component を frame=t で 1 回描くだけ(hold の決定性は設計の定義そのもの)。

| 語彙 | 引数 | 純関数か記憶か | 対応する概念 | 意味を貸すか | Motolii |
|---|---|---|---|---|---|
| `useCurrentFrame()` | — | 純(文脈の frame。`<Sequence>` で 0 起点にずれる) | 層の in 点からの経過 | 法 | 済: 層の `.time(from, dur)`、コマ時刻は rrd の時刻 |
| `useVideoConfig()` | — | 純 | comp の fps/width/height/duration(Sequence 内ならその長さ) | 法 | 済: `comp({fps,width,height,seconds})` |
| `interpolate(v,[in],[out],{extrapolateLeft/Right: extend\|clamp\|wrap\|identity, easing, output: linear\|perceptual-scale, posterize})` | 数→数、多区間可、区間ごとの easing 配列 | 純 | キーフレーム補間(AE の linear()/ease())。`wrap` = mod、`posterize` = コマ落とし、`perceptual-scale` = 面積補正 | 法 | 済: `.keys("P",[[t,v,"Bezier"],…])`。**無い**: extrapolate `wrap`/`identity`、`posterize`(Posterize Time 相当)、perceptual-scale |
| `spring({frame,fps,config:{mass,damping,stiffness,overshootClamping},from,to,durationInFrames,durationRestThreshold,delay,reverse})` | 0→1 | **純に見えるが実装は積分器**: `spring-utils.ts` の `springCalculation` は `for f=0..floor(frame)` で `advance()` を回す(各段は減衰比ごとの閉形式 under/critically-damped、キャッシュ 2 段)。結果は frame の関数だが計算量 O(frame) | AE の Elastic/Overshoot、CSS `linear()` の近似 | 法(config は数値のみ) | △: `.keys` の "Elastic"/"Bounce"、`.effect("Bounce")`。**Motolii は閉形式で 1 コマを直接出せる**(Remotion より純) |
| `Easing.{linear,quad,cubic,poly(n),sin,circle,exp,ease,elastic(b),back(s),bounce,bezier(x1,y1,x2,y2),in/out/inOut}` | t→t | 純 | CSS `cubic-bezier`/`ease-*`(Reanimated 由来) | 法 | 済: "Bezier"/"Ease In Out"/"Linear"/"Elastic"/"Bounce"。**無い**: `poly(n)`、`back(s)` の引数、`bezier` の 4 値指定 |
| `<Sequence from durationInFrames layout premountFor name>` | 子の frame を `from` 起点へ | 純(frame の平行移動) | 層の in/out(AE のレイヤー時間)。`premountFor` = 先読み(描画都合) | 法 | 済: `.time(a, len)`(sync.js の `during`) |
| `<Series>` + `Series.Sequence durationInFrames offset` | 連結、負 offset で重なり | 純 | 直列の並べ(AE の Sequence Layers) | 法 | △: 台本で累積 t を計算(sync.js)。「前の終わりから」の語が無い |
| `<Loop durationInFrames times layout>` + `useLoop()` | frame を `mod` | 純 | `animation-iteration-count`、AE の loopOut | 法 | ×: **無い**。台帳 3 の 7・10 と同じ `mod(t)`、時間の法として 1 つ |
| `<Freeze frame active>` | 子の frame を定数に | 純(定数関数) | AE の Freeze Frame / Time Remap の hold | 法 | ×: 無い。Time Remap の hold として同型 |
| `<OffthreadVideo src trimBefore trimAfter playbackRate volume muted transparent toneMapped>` | コマ t の画を FFmpeg で抜いて `<Img>` | 純(コマ抜き。再生器の記憶を使わない)。preview は `<video>` + `acceptableTimeShiftInSeconds` 0.45s で seek | 動画素材の Time Remap | 法 | 済: re_video の復号(可視層のみ)。`toneMapped` = HDR→bt709 の旗と同じ議論 |
| `<Audio src volume(frame) trimBefore trimAfter playbackRate loop muted toneFrequency>` | 音量は `(frame)=>number` | 純(音量が frame の関数) | 音の層、音量キー | 法 | △: 音は「消す・1 行・既存へ丸投げ」の方針。音量 = 時刻の関数 という 1 行は同型 |
| `useAudioData` + `visualizeAudio({audioData,frame,fps,numberOfSamples})` | frame → スペクトル配列 | 純(音の解析を frame で引く) | AE の Convert Audio to Keyframes | 法 | ×: 無い。音の「焼き」を時刻で引く形は rrd の時系列に置ける |
| `staticFile(path)` | public/ → URL | — | 素材の参照(フットプリント) | — | 済: rrd の blob / path |
| `delayRender()/continueRender()/cancelRender()` | 非同期を待つ、30s timeout | 描画の都合(純関数の外側) | 素材の読み込み待ち | — | 不要: rerun は blob を持ってから描く |
| `@remotion/transitions` `<TransitionSeries>` + `.Sequence/.Transition{timing,presentation}/.Overlay`、`linearTiming({durationInFrames})`/`springTiming({config})`、`fade()/slide({direction})/wipe()/flip()/clockWipe()/iris()/cube()/none()` | 重なり分だけ全長が縮む(40+60−30=70) | 純(`presentationProgress` = 時刻の関数) | 遷移(合成の順 + Clip/Opacity/Transform)。台帳 3 の 3「遷移の型」 | **意味**(slide/wipe/iris は「見せ方」の名前) | ×: 名前で貸さない。中身は Clip の inset 1 辺・Opacity・Position の 2 層重ね = 既存の法で書ける。**timing/presentation の分離**(いつ/どう)は借りる価値あり |
| `@remotion/layout-utils` `measureText({text,fontFamily,fontSize,fontWeight,letterSpacing})`→`{width,height}`、`fitText({text,withinWidth,fontFamily})`→`{fontSize}` | DOM 測定、キャッシュ | 純(入力同じなら同じ) | 文字の箱の寸法、`font-size` の自動合わせ | 法 | △: 文字の箱は layout(Flex/Grid)が測る。`fitText` 相当(幅に合わせる字の大きさ)は無い |
| `@remotion/animation-utils` `makeTransform([rotate(),translate(),scale(),skew(),perspective(),matrix3d()…])`→string、`interpolateStyles(v,[in],[styles],opts)` | CSS transform 文字列の合成 / style object の補間 | 純 | CSS `transform` の順序(台帳 1 の transform 列) | 法 | 済: Position/Rotation/Scale/Tilt/Position Z。**transform の並び順**を利用者が決める語は無い |
| `@remotion/shapes` `<Rect/Circle/Ellipse/Triangle/Star/Pie/Polygon/Heart>`、`makeRect()…`→`{path,width,height,transformOrigin,instructions}` | 寸法→SVG path | 純 | パラメトリック形状 | 法 | 済: rectangle/ellipse。Star/Pie/Polygon は無い |
| `@remotion/paths` `evolvePath(progress,path)`→`{strokeDasharray,strokeDashoffset}`、`getLength`、`getPointAtLength(path,len)`→`{x,y}`、`getTangentAtLength`、`interpolatePath`、`warpPath` | progress・長さ → 値 | 純 | Trim Paths(台帳 3 の 6)、`offset-path`/`offset-distance`(Along Path)、形の補間、パスの歪み | 法 | 済: `.effect("Trim Paths")`、`Offset Path`+`Offset Distance`。**無い**: `interpolatePath`(形の morph、台帳 2 `shape()` 相談中)、`warpPath` |
| `@remotion/noise` `noise2D/3D/4D(seed,x,y[,z,w])`→[-1,1] | simplex、seed で決定的 | 純 | AE の wiggle を「時刻を x に入れる」で純にした形 | 法 | △: `.effect("Oscillator")`/"Wave" はある。**seed 付き noise(t)** は無い。`random(seed)` はある |
| 順番(stagger) | 語彙なし: `<Sequence from={i*delay}>` を map で並べる、or `spring({frame: frame - i*delay})` | 純 | 台帳 E の sibling-index | 法 | 済: `Stagger`/`Stagger From` の札、Arrive の `Stagger` |

**Remotion の裁き(4 行)**
1. 純関数の徹底は Motolii と同じ芯。Sequence/Loop/Freeze は全部「frame の平行移動・mod・定数化」= 時間の法 3 つ。Motolii には `.time` しかない → **Loop と Freeze(hold)を時間の法として足す**のが最短。
2. spring は「frame の関数」だが実装は frame 0 から積分(キャッシュで隠す)。Motolii は閉形式の減衰振動で 1 コマを直接出す方が rrd の「任意時刻を引く」に合う。
3. 意味を貸すのは `@remotion/transitions` の preset だけ。それも `timing`(いつ)と `presentation`(どう)を分けて数値 progress で受ける設計は法として綺麗。名前は借りず、分離だけ借りる。
4. 音は `volume(frame)`・`visualizeAudio(frame)` で「音も時刻の関数」に落とす。DAW を再発明せず、解析結果を時系列として引く形は rrd と相性がよい。

---

## 2. Theatre.js — 頁の中の Sequence Editor

| 語彙 | 引数 | 純関数か記憶か | 対応する概念 | 意味を貸すか | Motolii |
|---|---|---|---|---|---|
| `getProject(id,{state})` | JSON の state を読む | 状態は JSON(studio が localStorage に持つ) | プロジェクト/書類 | 法 | rrd が書類 |
| `project.sheet(name[, instanceId])` | 論理の束、instance で複製 | — | comp / precomp の instance | 法 | comp、group |
| `sheet.object(key, {props})` | prop 型: number(range,nudgeMultiplier)/string/boolean/compound/stringLiteral/rgba/image/file | 型と初期値。detach しても値は「記憶」される(書類の側) | 層の property 群 | 法 | `.set()` の欄 |
| `obj.value` / `obj.onValuesChange(cb)` / `obj.props`(pointer) | 現在値の読み・購読 | position の関数(補間結果を push) | AE の expression の読み口 | 法 | 描画側が rrd を引く |
| `sheet.sequence` `.position`(秒)、`.play({iterationCount,range,rate,direction})`、`.pause()`、`.attachAudio()`、`.pointer` | 再生頭 | **値は position の純関数**(キーフレーム + cubic-bezier handle)。`play` は position を進める時計で、値の側に記憶なし | タイムライン、再生、音の同期 | 法 | rrd の時刻 = position |
| keyframe(値・位置・tween handle、aggregate keyframe) | studio の UI | 純 | AE のキーフレーム、Graph Editor | 法 | `.keys` |
| `studio`(`@theatre/studio`) | 頁に載る編集 UI、state を export | — | Motolii の Flutter UI に当たる | — | 別世界 |

**Theatre.js の裁き(4 行)**
1. データモデルは「object の props が sequence.position の純関数」= Motolii と同じ。play/pause は position を動かす外側の時計で、値には記憶なし。
2. 抽象は最小 4 つ: project / sheet / object(props) / sequence。Motolii の comp / layer / 欄 / rrd 時刻に 1:1。
3. 貸してくるのは prop 型(range, rgba, compound)だけで意味は無い。`compound`(入れ子の欄)と `stringLiteral`(選択肢)は Motolii の札の型と同型。
4. 新味は「studio が頁に載る」= 完成品の中に編集器が同居する。Motolii では vism の自走がそれ。

---

## 3. Rive — 状態機械のランタイム(Motolii が採らない形の見本)

| 語彙 | 引数 | 純関数か記憶か | 対応する概念 | 意味を貸すか | Motolii |
|---|---|---|---|---|---|
| Timeline animation(one-shot / loop / ping-pong) | キーフレーム | 純(1 本だけなら時刻の関数) | AE の comp | 法 | 済 |
| State Machine: Animation State / Entry / Exit / Any State | graph(状態=timeline) | **記憶**: 今どの状態か、遷移の途中か | ゲームの FSM、CSS `:hover`+transition | 法 | **採らない**(時計は時刻 1 つ) |
| Transition: path, conditions, duration(既定 0)、exit time(%)、interpolation、pause source、allow exit during、randomize exit、actions | 条件 = 入力の比較 | 記憶(履歴依存) | CSS `transition` + `:hover` の条件 | 法 | 採らない。ただし「exit time」= 前の終わりから、は Series の offset と同型で時刻で書ける |
| Inputs: Boolean / Number / Trigger(瞬間だけ true)、Listeners(pointer → input) | 外から値を入れる | 入力は外部の時計 | 変数 / expression control | 法 | △: 数値 1 つを外から与える口は「札の数値」で済む。Trigger は無い(時刻でよい) |
| Blend State: 1D(number 1 つで 2 timeline を混ぜる)/ Additive(複数の number で加算) | 数→混合率 | **純**(入力の関数) | AE の Pose morph、CSS `animation-composition: add` | 法 | ×: **無い**。「2 本の焼きを数値で混ぜる」は時刻の純関数で、良い抽象 |
| Layers(1 つの state machine に複数の同時レーン) | — | 記憶 | 多重の FSM | 法 | 採らない |
| Data Binding(2025): View Model / Instance / Binding / 更新は editor・runtime・state machine・script。Inputs は deprecated | 場面の性質 ← データ | 純(データの関数)、場面の階層から独立 | AE の Essential Graphics、CSS 変数 | 法 | ○: 「欄を名前で外から差す」は札の欄 = rrd の時系列と同型 |
| Layouts(2025): Row/Column、Absolute/Relative、Fill/Hug、padding/gap、nest | flex 型 | 純 | CSS flexbox | 法 | 済: Display Flex/Grid、Sizing Fixed/Fill |
| Nested Artboards + exposed inputs | 部品 | — | precomp + 札の欄 | 法 | 済: group |

**Rive の裁き(4 行)**
1. 状態機械は「今どこにいるか」を持つ = 履歴の関数。コマ t の絵を出すのに 0 から再生が要る。Motolii が拒む形の最も整った見本。
2. それでも Blend State(number → 2 本の焼きの混合)と Data Binding(欄を名前で外から差す)は純関数で、Motolii の札に置ける。**「数値 1 つで 2 つの焼きを混ぜる」は Field の Spread/Turn と同じ 1 ダイヤル思想**。
3. Layouts は CSS flexbox の写し。Motolii の Flex/Grid と同じ判断を 2025 に別会社もした。
4. Trigger/Listener(pointer 起点)は動画に無い時計。台帳 3 の結論通り「in 点 = 時刻」に置き換える。

---

## 4. Lenis — `lerp` の記憶を時刻の関数に書き直す

| 語彙 | 引数 | 純関数か記憶か | 対応する概念 | 意味を貸すか | Motolii |
|---|---|---|---|---|---|
| options `lerp`(0.1) / `duration`+`easing` / `wheelMultiplier` / `syncTouch`+`syncTouchLerp`(0.075)+`touchInertiaExponent`(1.7) / `infinite` / `autoRaf` | — | 設定 | — | 法 | — |
| state: `targetScroll`, `animatedScroll`, `velocity`, `direction`, `isScrolling` | — | **記憶 3 つ**: 目標 `to`、現在値 `value`、(duration 式では)`currentTime` | 台帳 3 の 8(カーソル追従)・Codrops 6(ScrollSmoother lag) | 法 | 物理の「留め具」 |
| `animate.advance(dt)` lerp 式: `value = damp(value, to, lerp*60, dt)`、`damp(x,y,λ,dt) = lerp(x, y, 1 − exp(−λ·dt))` | 1 次遅れ、fps 非依存 | 記憶(前フレームの value) | 指数平滑、RC 低域通過 | 法 | 下に書き直し |
| `animate.advance(dt)` duration 式: `currentTime += dt; p = clamp(currentTime/duration); value = from + (to−from)·easing(p)` | 有限長 | `from`/`to`/開始時刻を持てば**純**(= キーフレーム 1 区間) | CSS transition | 法 | 済: `.keys` の 1 区間 |
| `onVirtualScroll` → `scrollTo(targetScroll + delta)` | 入力で `to` を差し替え | 入力は外部の時計 | 目標 g(t) | 法 | 目標は時刻の関数として書類にある |

**書き直し**: lerp 式の連続極限は `dv/dt = λ (g(t) − v)`、λ = lerp·60(既定 6/s、時定数 τ = 1/λ ≈ 0.17s)。解は畳み込み
`v(t) = ∫₀^∞ λ e^{−λs} g(t − s) ds`(指数核、面積 1)+ 初期項 `e^{−λt}(v₀ − g(0))`。
目標 g が Motolii のキーフレーム(区分多項式/Bezier)なら積分は閉形式: 区間ごとの多項式 × 指数の積分で、**任意の t を直接出せる**。g が定数区間なら `v = g + (v_in − g)e^{−λ(t−t_in)}`、g が線形 `a+bt` なら `v = a + b t − b/λ + C e^{−λt}`(定常で b/λ だけ遅れる = **時刻のずらし τ**、それに過渡の指数 1 項)。
つまり留め具は「重さ(1/λ = τ)」1 つの数値で、(1) 目標が滑らかなら **time-shift τ の近似**(`.time` を τ だけ後ろへ)、(2) 正確には指数核の畳み込みで、どちらも時刻の純関数。2 次(ばね+減衰)にすると核が減衰振動 `e^{−ζωt} sin(ω_d t)` になり、Remotion の spring と同じ式に合流する(spring = ステップ入力の畳み込み)。

**Lenis の裁き(4 行)**
1. 持つ記憶は `to` と `value`(と `currentTime`)。入力(scroll)が時刻の関数として書類にあれば、`value` は畳み込みで再現でき記憶は要らない。
2. duration 式は既に純(キーフレーム 1 区間)。lerp 式だけが記憶で、これが物理の留め具の最小形(1 次)。
3. 留め具の数値は重さ(τ)だけにできる。物理の法「住所 4 つ、数値は重さ/硬さ/ばらつき」に合う。
4. 近似は time-shift、正確は指数核。2 次にすれば spring と同じ式で、Remotion の spring も「ステップ入力に留め具を掛けた結果」と読める。

---

## Remotion と rrd

| | Remotion(frame が真) | Motolii(rrd の時刻が真) |
|---|---|---|
| 真理 | `frame` 整数、fps は comp 固定 | 時刻(秒、VFR も受ける)。fps は出力の都合 |
| 書類 | React のコード(TSX)。値は関数の中 | rrd の時系列(欄ごとの焼き)。値は表 |
| 「コマ t を出せ」 | component を frame=t で描く。spring は 0..t を積分(キャッシュ) | 時刻 t で時系列を引く。物理も閉形式で t を直接 |
| 素材 | `<OffthreadVideo>` = FFmpeg でコマ抜き、`delayRender` で待つ | re_video で可視層のみ復号、blob は書類に |
| 音 | `volume(frame)`、`visualizeAudio(frame)`、FFmpeg で合流 | 既存へ丸投げ。解析は時系列として置ける |
| 時間の法 | Sequence(ずらし)/ Loop(mod)/ Freeze(定数)/ Series(直列)/ TransitionSeries(重なり) | `.time` のみ |
| 意味 | transitions の preset(slide/wipe/iris) | 貸さない |

**Remotion にあって Motolii に無い**: Loop・Freeze・Series の 3 つの時間の法(全部 frame の写像で純)、`interpolate` の `wrap`/`posterize`/perceptual-scale、seed 付き `noise(t)`、`interpolatePath`(形の morph)、`fitText`(幅に合わせる字)、`timing/presentation` の分離(遷移の「いつ」と「どう」)、音の解析を時刻で引く口。
**Motolii にあって Remotion に無い**: 時系列そのものが書類(Remotion は関数がコードに埋まり、UI から値を掴めない → Studio は props の編集止まり)、物理(箱・物・場・留め具)を閉形式で任意時刻に(Remotion は spring 以外の物理を持たず、spring も積分)、layout(Flex/Grid)が動きの法と同じ書類に乗る(Remotion は CSS に丸投げ、DOM 測定は browser のみ)、GPU の合成(Remotion は Chrome のスクショ = 描画は借り物)、VFR の素材を秒で受ける。
**共通の芯**: 時計は 1 つ、状態機械なし、hold は「その t で 1 回描く」で決定。Remotion が 2020 年から動画でこれを証明しているのは、Motolii の「時刻の純関数」が Web 側の先例に沿う証拠。

