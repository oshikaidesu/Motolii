# 創作コーディングの台帳(5)— Cavalry の画の構造は for 文と関数、出所の裏は取れない

2026-09-17 調査(取説から: p5.js / Processing の reference、Nature of Code、Generative Design、ofBook、Cavalry の docs と創業者の発言)。利用者「Cavalry が欲しいのではなく Cavalry の画が欲しかった。源流は Web。箱は持った。次は GSAP、でもそこまで効果はなかった」「Cavalry の原点はクリエイティブコーディングと私は思う。次に検索すべきはそこだ」。

結論を先に: **出所としては裏が取れない**(創業者が挙げるのは Maya の MASH と「何でも何にでも繋がる」ノード型)。**構造としては当たっている**(index の文脈 = for 文の i、JS Layers = 毎コマの draw、Dynamics と Trails 以外は全部 (i, 位置, 時刻) の純関数 = Motolii の法と同じ切れ目)。CSS と GSAP に無くて創作コーディングと Cavalry にある物は 1 つ: **場 = f(index, 位置, 時刻) を、どの属性にも掛ける**(GSAP の stagger grid from:center はその 1 次元の影、Cavalry の Duplicator + Falloff + Shape Time Offset はそれをノードに包んだ物)。[台帳 4](2026-09-17-framework-ledger.md)・[Cavalry への文句](2026-09-16-cavalry-complaints.md) の続き。

---

# クリエイティブコーディングの語彙と型 — Cavalry の原点を Motolii の 4 住所で読む(2026-09-17)

利用者の説「Cavalry の原点はクリエイティブコーディング」。取説(p5.js / Processing reference、Nature of Code、Generative Design の例、ofBook、Hobbs / DesLauriers / 10 PRINT)から、「似た物が沢山、関数で少しずつ違う」画を作る語彙と型を抜き、Motolii の 4 住所(箱・物・場・留め具)と台本の語彙に当てた。
篩: (a) 意味を貸さない(数学・法だけ、出来上がりの見た目を持たない) / (b) どの素材にも掛かる / (c) 時刻の純関数。○ = 満たす、× = 満たさない、△ = 書き方次第。
純/状態: draw() は毎コマ白紙から描くので、frameCount だけから描けば純(p5: frameCount は setup で 0、draw が終わるごとに +1)。前のコマの値を足す(`v += a`、`x = lerp(x, target, k)`)と状態 = 時刻を飛ばすと絵が変わる。

## 1. 語彙

| 語彙 | 何をする | 純/状態 | 住所 | Motolii の台本 | 篩 a/b/c |
|---|---|---|---|---|---|
| `for (i)`、入れ子 `for (x, y)` | 同じ物を i / (x, y) で並べる。全ての型の骨 | 純(構造) | 箱(升目)+ 物 | ○ 台本の for が層を作る(wave_grid.js)、Repeater Along Line/Circle/Grid + Columns(placement.rs:44,70-77)、箱の Display Grid | ○/○/○ |
| `map(v, a, b, c, d)` | 範囲の写し(i → 角度、距離 → 大きさ)。withinBounds で constrain | 純 | 場(値の写し) | △ 台本の JS で計算。Repeater の Each 行(Position/Rotation/Scale/Opacity/Delay × i、placement.rs:80-84)が「i の線形写し」の窓の名前 | ○/○/○ |
| `lerp(a, b, t)` | 2 値の間、t が 0..1 の外なら外挿 | 純(t = 時刻なら) | 場 / 留め具 | ○ キーの補間(Linear/Bezier)、Transition Duration(FLIP) | ○/○/○ |
| `noise(x[, y, z])` Perlin | 滑らかな乱数、同入力同出力、「2・3 軸目は時間にも読める」(Processing)。歩幅 0.005–0.03、noiseDetail で octave、noiseSeed で種 | 純(座標と時刻の関数) | 場 | △ Turbulent Displace(Amount/Size/Complexity/**Evolution = TIME**/Seed、vism/turbulent_displace.wgsl)は網の頂点・点群の変位だけ。Rotation/Scale/Opacity へ掛かる noise の場は無し。Repeater の Random 行は種の乱数で、滑らかでない | ○/○/○ |
| `sin/cos`、位相 = i × k、`A·sin(TWO_PI·frameCount/period)` | 周期。index で位相をずらすと波、frameCount で時刻(NoC Ch.3 の単振動は純) | 純(NoC の Wave の `startAngle += 0.02` は状態だが ωt に書き直せる) | 場 | △ Wave(Amplitude/Frequency/Wavelength、glyph に GPU、sync.js:119)、Oscillator(線、pathop.rs:102)、Cyclic ease、Loop Duration/Alternate、Repeater Delay Each。属性一般へ掛ける正弦の場は無し | ○/○/○ |
| `dist(x1, y1, x2, y2)` | 点からの距離で減衰(falloff) | 純 | 場 | ○ 層の Field + Field Falloff/Scale/Push(field_lens.js)、Stagger From Center、Arrive Radial | ○/○/○ |
| `random(a, b)` + `randomSeed(s)` / `noiseSeed` | 種で決まる乱数、「同じ種なら毎回同じ絵」(Remotion、DesLauriers `setSeed` も同じ) | 純(種の関数) | 物のばらつき | ○ 台本 `random(seed)`、Repeater Random 行 + Seed(placement.rs:85-90)、Random ease、Turbulent Displace Seed。`Math.random` は赤 | ○/○/○ |
| `translate/rotate/scale` + `push/pop` | 局所座標の入れ子、style と変換を戻す | 純 | 箱(親子) | ○ group / .parent、Anchor、Transform Origin。全体状態としては禁止(script-mouth.md) | ○/○/○ |
| `beginShape/vertex/splineVertex(旧 curveVertex)` | 点列から形(PATH/LINES/TRIANGLE_STRIP…) | 純 | 物(path) | △ line + Connect From/To、Trace、Offset Path、Trim Paths。頂点列を台本で置く口は無し(polygon/star は既製) | ○/○/○ |
| `frameCount` / `millis()` / `deltaTime` | 時計 | 純(時計そのもの) | — | ○ キーの秒、`.time()`。時計は Timeline(窓) | ○/○/○ |
| `mouseX / mouseY` | 入力(Generative Design はほぼ全例が mouse を knob にする) | 状態(外部) | — | × 動画に cursor は無い → キーの層(web_s8 の Cursor)か Field の source 層 | ○/○/× |
| `constrain(v, lo, hi)` | 範囲に留める(壁) | 純 | 留め具(壁) | ○ 箱から出ない(壁は箱に付く)、Overflow Clip、Snap to Grid | ○/○/○ |
| 極座標 `x = r cos θ`、`atan2` | 円に並べる、向きを読む | 純 | 箱(円) | ○ Repeater Along Circle(Radius/Start/Sweep)、Arrive Radial、台本の Math.atan2(web_c4) | ○/○/○ |
| draw() の中の `x += (target − x) × k`(ofBook「zeno」) | 一次遅れの追従 | **状態** | 留め具 | ○ Transition Duration(FLIP)、Arrive。閉形式 τ(1 − e^(−t/τ)) で純にした先例 web_s8 / web_c6 | ○/○/× |
| `p5.Vector` add/sub/mult/limit/setMag/heading/fromAngle | 速度・加速度の代数 | 状態(velocity を持つ) | 物 | △ 台本の JS 配列。窓の名前ではない | ○/○/× |
| `applyForce` → `v += a; p += v; a = 0` | 力の積分(NoC Ch.2) | 状態 | 場 + 物 | ○ Field(Spread/Angle/Strength)+ Hardness/Heaviness/Margin、焼き(pour.js、matter.js) | ○/○/× |
| Particle / Emitter / lifespan | 毎コマ生む、死んだら消す(NoC Ch.4) | 状態(生成が時刻の関数でない) | 箱(emitter)+ 物 | ○ particles(Rate/Life/Gravity/Wind/Turbulence/Seed)は閉じた式 | ○/○/× |
| Flow field: `angle = map(noise(x, y), 0, 1, 0, TAU)` → vehicles follow | 場に従う道(NoC Ch.5、Hobbs) | 場は純、道は積分(種から決定) | 場 + 物 | × 無し。particles の Turbulence は場だが「道」は無い、Turbulent Displace は形の変位 | ○/○/△ |
| 自律 agent: seek / flee / **arrive** / wander、separation / cohesion / alignment | `steer = desired − velocity`、arrive は半径内で `map(d, 0, r, 0, maxspeed)` | 状態 | 留め具(目標) | ○ **Arrive**(From/Distance/Arrive/Bounce/Stagger/Spin)は到着の純関数、Push Apart = separation | ○/○/× |
| Spring `force = −k (|x| − rest)` / 振り子 `a = −g/r · sin θ` | 家へ戻る(NoC Ch.3) | 状態 | 留め具 | ○ Hang(Length/Swing/Settle/Beat)、Bounce/Elastic ease、Hardness | ○/○/× |
| Attraction `G·m1·m2 / d²`(d を constrain) | 1 つの引力源に沢山の mover | 状態 | 場 | ○ Field Spread 0(magnet.js) | ○/○/× |
| 再帰 / L-system(`F + − [ ]`) | 自分を呼ぶ、文字列を規則で伸ばして亀が描く(NoC Ch.8) | 純(乱数は種) | 箱の入れ子 | △ 台本の再帰で group を入れ子に。L-system 無し | ○/○/○ |
| Cellular automata(Wolfram 1D、Life) | 前の世代から次、行 = 世代(NoC Ch.7) | 状態(初期条件から決定) | 物の升目 | × 無し(焼きの型) | ○/○/× |
| `background()` を塗らない残像 | 前コマを残す | 状態 | — | × 赤 → Echo 効果(script-mouth.md) | ×/○/× |
| easing(`pct^2`、smoothstep、expo) | 0..1 の曲げ | 純 | 場(時刻の写し) | ○ ease 名(Bezier/Elastic/Steps/ElasticSteps、"expo"/"power3.in") | ○/○/○ |

## 2. 型(canonical sketches)

| 型 | p5 の芯 | 1 つ触ると全部が揃う knob | 純/状態 | Motolii の 4 住所(箱 / 物 / 場 / 留め具) | 台本 | 篩 a/b/c |
|---|---|---|---|---|---|---|
| 動く点からの距離で大きさ(升目の定番。Generative Design P_2_1_2「grid の円の大きさと位置」は mouse と種で同じ事をする) | `for (gx, gy) { d = dist(mouseX, mouseY, x, y); s = map(d, 0, 300, 40, 4); circle(x, y, s) }` | falloff 半径(300) | 純(mouse を時刻の関数に置けば) | 箱 = Grid / 物 = 円 × N / 場 = f(dist(p, lens(t))) → Scale / 留め具 = 不要 | ○ field_lens.js(Field/Falloff/Scale/Push) | ○/○/○ |
| noise 場で線の回転(Generative Design M_1_5_01「noise を角度と明るさに」) | `noiseX = map(gX, 0, res, 0, range); angle = noise(noiseX, noiseY) * TAU; push(); translate(x, y); rotate(angle); line(0, 0, L, 0); pop()` | noise の歩幅 range(粗密が一斉に変わる) | 純(3 軸目 = t で動く) | 箱 = Grid / 物 = 線 / 場 = noise(x, y, t) → Rotation / 留め具 = 不要 | × Rotation に掛かる noise の場が無い(Turbulent Displace は頂点) | ○/○/○ |
| 正弦の波、index で位相(NoC Ch.3 Wave、Generative Design M_2_1 Oscillator) | `for (x = 0; x < w; x += 24) { y = sin(angle) * A; circle(x, y); angle += dθ } startAngle += 0.02`。M_2_1 は `t = (frameCount / n) % 1; y = sin(angle*freq + phi)` で純 | dθ(波長)/ freq | △ NoC は状態、`ωt + i·dθ` なら純 | 箱 = 行 / 物 = 円 / 場 = sin(ωt + i·dθ) → Position Y / 留め具 = 不要 | △ Wave(glyph)、Oscillator(線)、Repeater Delay Each + Cyclic ease | ○/○/○ |
| Flow field + 粒子(Hobbs、NoC Ch.5、Generative Design M_1_5_03「noise 3d で 4000 agents」) | `angle = map(noise(x*s, y*s), 0, 1, 0, TAU); x += step*cos(angle); y += step*sin(angle)` | noise の scale s(Hobbs: step 0.1–0.5 % 幅、曲線の長さで fur ↔ fluid) | 場は純、道は積分(種で決定 → 焼き) | 箱 = 画面 / 物 = 曲線 or 粒子 / 場 = noise → 向き / 留め具 = 不要 | × 無し | ○/○/△ |
| 放射・極の並び、index で回転 | `for (i < n) { θ = i*TAU/n; push(); rotate(θ + frameCount*0.01); translate(r, 0); rect(…); pop() }` | n(数) | 純 | 箱 = 円 / 物 = 形 / 場 = Rotation Each = i·360/n / 留め具 = 不要 | ○ Repeater Along Circle + Rotation Each(sync.js dots、web_s10) | ○/○/○ |
| 再帰分割(Generative Design M_5_1「recursive function」、NoC Ch.8 Cantor / tree) | `divide(x, y, w, h, d) { if (d == 0) rect(…); else { r = random(.3, .7); divide(…w*r…, d−1); divide(…, d−1) } }` | depth(段)/ 分割比の種 | 純(種) | 箱 = 箱の入れ子 / 物 = 葉 / 場 = 無し / 留め具 = 無し | △ 台本の再帰で group(Display Grid)を入れ子に | ○/○/○ |
| Perlin 地形・風景の線(NoC Ch.0 2D noise、Exercise 0.9 で 3 軸目を時間に) | `for (x) { y = map(noise(xoff, yoff), 0, 1, 0, h); vertex(x, y); xoff += 0.01 } yoff += t` | xoff の歩幅(0.005–0.03) | 純(t 軸) | 箱 = 行の並び / 物 = 頂点列の線 / 場 = noise(x, row, t) → Y / 留め具 = 不要 | △ 線 + Turbulent Displace(Evolution)が近い。頂点列の口は無し | ○/○/○ |
| Random walker(NoC Ch.0、Generative Design P_2_2_1「stupid agent」) | `choice = floor(random(4)); x++ / x−− / y++ / y−−` | 歩幅 / 種 | 状態(累積、種で決定 → 焼き) | 箱 = 画面(constrain の壁)/ 物 = walker / 場 = 種の乱数 / 留め具 = 無し | △ Random / Steps ease で近似のみ | ○/○/× |
| Spring / attraction(NoC Ch.2–3、ofBook particles) | `force = sub(anchor, bob); force.setMag(−k*(force.mag() − rest)); applyForce(force)` / `strength = G*m1*m2/d²` | k(硬さ)/ G(強さ) | 状態(焼き) | 箱 = 部屋 / 物 = Hardness・Heaviness / 場 = Field(Spread 0 = 磁石、Spread 1 = 重力)/ 留め具 = Hang・Bounce | ○ magnet.js、hang.js、matter.js | ○/○/× |
| 升目の文字 × noise(Generative Design P_3 の型) | `for (i, ch) { n = noise(i*0.1, t); textSize(map(n, 0, 1, 10, 60)); text(ch, x, y) }` | t の速さ / noise の歩幅 | 純 | 箱 = Flex(Split Chars)/ 物 = glyph / 場 = noise(i, t) → Size・Opacity・Rotation / 留め具 = 不要 | △ Split Chars + Stagger ○、noise の場 ×(Wave は正弦のみ) | ○/○/○ |
| 10 PRINT(`10 PRINT CHR$(205.5+RND(1)); : GOTO 10`、Generative Design P_2_1_1「diagonals in a grid」) | `randomSeed(s); for (gx, gy) { toggle = int(random(0, 2)); toggle ? line(╲) : line(╱) }` | 確率(0.5)/ 種 | 純(種) | 箱 = Grid / 物 = 2 種の線 / 場 = 種の 2 択 / 留め具 = 不要 | △ Repeater Along Grid + Pick Random + Seed(placement.rs:49、Pick の意味は要確認) | ○/○/○ |
| Cellular automata(NoC Ch.7 Wolfram rule 90 / Life) | `for (i) next[i] = rules(cells[i−1], cells[i], cells[i+1]); square(i*w, generation*w, w)` | rule 番号(30/90/110) | 状態(初期行から決定 → 焼き) | 箱 = 升目 / 物 = cell / 場 = 近傍の規則 / 留め具 = 無し | × 無し | ○/○/× |

## 見立て — クリエイティブコーディングにあって CSS/GSAP に無いもの

1. **場 = f(index, position, time) がどの属性にも掛かる。** noise / sin / dist の同じ 1 行を Rotation にも Opacity にも Size にも当てられる。CSS/GSAP は要素ごとの値の列(keyframes、stagger)で、値が「どこにあるか」を知らない — GSAP の stagger `grid: [r, c], from: "center"` が唯一の距離の場。Cavalry の Duplicator + Fields + Shape Time Offset は、この升目・場・時計を node に包んだ物。
2. **時計が frameCount の純関数。** draw() は毎コマ白紙から描くので、状態を持たなければどのコマへ飛んでも同じ絵(Remotion の規則と同じ)。CSS/GSAP のタイムラインも純だが式ではなく値の列。
3. **種が作品の一部。** randomSeed / noiseSeed / `setSeed` で乱数が保存される(DesLauriers は seed を master control と呼ぶ)。CSS に乱数は無く、GSAP `random()` は毎回違う。
4. **半分は状態。** velocity / particles / lerp 追従 / walker / CA は時刻の純関数ではない。Motolii はこれを「焼く」(物理)か「閉形式」(web_s8 の τ(1 − e^(−t/τ)))で受けている — 住所は 場 + 留め具。
5. **Motolii に無いのは 1 つ、「属性へ掛かる連続の場」。** Turbulent Displace は形の頂点だけ、Wave は正弦・glyph だけ、Repeater の Random は滑らかでない、層の Field は「箱からの距離」だけ。表 2 の ×・△ は全てこの 1 つに落ちる(noise 回転、flow field、地形、字 × noise)。
6. **次の 1 手の候補(法なので先に相談)**: Repeater の Each / Random の隣に「Noise」の列(Size・Evolution・Seed、Turbulent Displace と同じ 3 欄)を置き、Position/Rotation/Scale/Opacity/Delay の各行へ掛ける。knob は Size 1 つ、時刻は Evolution = TIME で純。Lieberman(VoCA)の言う「変数と数字が図形をどう変えるかの感覚」がその 1 欄に乗る。

## Sources

- p5.js reference: noise https://p5js.org/reference/p5/noise/ 、frameCount https://p5js.org/reference/p5/frameCount/ 、randomSeed https://p5js.org/reference/p5/randomSeed/ 、map https://p5js.org/reference/p5/map/ 、lerp https://p5js.org/reference/p5/lerp/ 、beginShape https://p5js.org/reference/p5/beginShape/ 、p5.Vector https://p5js.org/reference/p5.Vector/ 、push https://p5js.org/reference/p5/push/
- Processing reference noise(): https://processing.org/reference/noise_.html (「2・3 軸目は時間にも読める」、歩幅 0.005–0.03)
- The Nature of Code(Shiffman): Ch.0 https://natureofcode.com/random/ 、Ch.2 https://natureofcode.com/forces/ 、Ch.3 https://natureofcode.com/oscillation/ 、Ch.4 https://natureofcode.com/particles/ 、Ch.5 https://natureofcode.com/autonomous-agents/ 、Ch.7 https://natureofcode.com/cellular-automata/ 、Ch.8 https://natureofcode.com/fractals/
- Generative Design(Bohnacker/Groß/Laub/Lazzeroni)code package: https://github.com/generative-design/Code-Package-p5.js — P_2_1_1_01、P_2_1_2_01、P_2_2_1_01、M_1_5_01、M_1_5_03、M_2_1_01、M_5_1_01 の sketch.js(generative-gestaltung.de は TLS で読めず、raw から)
- openFrameworks ofBook「Animation」(Zach Lieberman): https://openframeworks.cc/ofBook/chapters/animation.html (frame/time、zeno の lerp、sin/cos、ofNoise、particles)
- Tyler Hobbs「Flow Fields」: https://tylerxhobbs.com/essays/2020/flow-fields
- Matt DesLauriers: https://mattdesl.svbtle.com/generative-art-with-nodejs-and-canvas 、canvas-sketch-util random.md(`setSeed`、`noise2D(x, y, frequency, amplitude)`)https://github.com/mattdesl/canvas-sketch-util/blob/master/docs/random.md
- Zach Lieberman: https://zachlieberman.medium.com/ (2016 年から毎日の sketch。Medium は 403 で本文は未読)、VoCA https://voca.network/blog/2021/01/15/future-sketches-code-as-a-medium/
- 10 PRINT: https://10print.org/
- Motolii: docs/reviews/2026-09-14-script-mouth.md、docs/reviews/2026-09-16-cavalry-complaints.md、motolii/crates/motolii-doc/src/store/placement.rs:44-100(Repeater の Along/Pick/Each/Random/Delay/Seed)、motolii/crates/motolii-render/vism/turbulent_displace.wgsl(Evolution = TIME)、motolii/crates/motolii-doc/src/store/pathop.rs:102(Oscillator)、motolii/ui/native/src/editor/script/examples/*.js、docs/wiki/script.md

---

# Cavalry と creative coding — 系譜・対応表・台本・画 3 枚(2026-09-17、読むだけ)

前提: docs/reviews/2026-09-16-cavalry-complaints.md(意味は借りる・面倒は消す)と 2026-09-15-sync-anatomy.md(軸は箱)。論題「Cavalry の原点はクリエイティブコーディング」を、創業者の言葉と取説で確かめる。

## 1. 系譜 — 創業者が名指しした物(引用は原文のまま、短く)
- **MASH(Maya)→ Cavalry**。Ian Waters「How Cavalry came to be」: Mainframe で作った Maya の animation toolkit MASH を Autodesk が買収、契約が切れた後 Martin Vejdarski と 2017 年に Cavalry を始めた。「Martin and I firmly believe that to achieve the varied outcomes necessary for modern use cases, digital content creation programs (DCCs) need to be node based.」「all animation innovation seemed to be happening in the 3d space」「2d was being left behind」
- **配線が出発点**。Ian Waters「The Magic of Cavalry」(2019-01-29):「Anything can connect to anything — within reason anyway. This was the thought that started the whole Cavalry adventure」
- **procedural = いつでも戻れる**。Chris Hardcastle「Introducing Cavalry」(2019-01-29):「Cavalry is fully procedural … you can always go back and change anything at any point.」「All without expressions, all without opening a node editor」
- **node は裏に隠す、言語は JS**。Ian Waters「Some of your Cavalry questions answered」(2019-02-05): node-based か →「Yes. However you won't need to use a node graph to use Cavalry, in fact, we haven't even built a node graph editor yet.」「We're looking at using JavaScript for our scripting language for 1.0」
- **動機は 2D の停滞**。Adam Jenns(Creative Bloq):「frustrated that all the innovation in animation seemed to be happening in 3d with the world of 2d/2.5d left largely stagnant」「a tool that was real-time and procedural」
- **外からの位置づけ**。School of Motion「A First Look」:「A Mograph Module for After Effects artists」、C4D 使いが Houdini に手を伸ばすのに似る、と。「Cavalry's easy SVG output in a visual (rather than coding) environment really lowers the barrier to entry for generative work」。同 podcast 頁(2024-10)の要約: Ian は Maya が「tons of manual coding」を要する事に苛立って道具を作った
- **裁定**: 創業者の言葉に Processing / openFrameworks は一度も出ない。名指しは MASH・node-based・procedural・「anything connects to anything」。creative coding は **用途**(generative work、cavalry.studio の説明文に "creative coding, … generative art, data visualisation")と **逃げ道**(JS Layers)として出るだけで、Ian にとってコードは苛立ちの元。→ 論題は「原点」としては裏付け無し、**「構造」としては当たる**: index context(取説「a number passed to another layer」)= for の i、JS Layers は毎コマの式、Distribution に Custom(JS)/Math がある。Cavalry は creative coding の *形*(i, p, t の関数)を、書かずに配線で組む道具。

## 2. 機能 ↔ creative coding 対応表
| 機能(取説の定義) | p5 の書き方 | 純関数か記憶か | 住所 | Motolii |
|---|---|---|---|---|
| Duplicator「copy and distribute Shapes」 | `for (i=0;i<n;i++) draw(shape, P(i))` | 純(i) | 箱(配る箱) | Repeater ✓(Count / Along / Position Each)、Display: Grid ✓ |
| Distribution: Grid | `x = (i%cols)*gx; y = floor(i/cols)*gy` | 純(i) | 箱 | Repeater Along Grid ✓、Grid Columns ✓ |
| Distribution: Circle(radial) | `a = TWO_PI*i/n; x = cx + r*cos(a)` | 純(i) | 箱 | Repeater Along Circle ✓(sync.js) |
| Distribution: Path(Use Rotation = 法線) | `p = path.pointAt(i/n); rot = path.normalAt(i/n)` | 純(i) | 箱 | Offset Path = Border Box △(border_path.js、円の縁は未) |
| Distribution: Random | `randomSeed(seed+i); x = random(w)` | 純(seed, i) | 箱 | Repeater Position Random / Seed Random ✓(grid_intent.js) |
| Falloff「1 at its centre which falls off to 0」Circle/Rect/Linear/Sweep/Shape、Graph、Probability | `k = curve(map(dist(x,y,cx,cy), 0, R, 1, 0)); s *= k` | 純(p) | 場 | Field + Field Falloff / Scale / Push ✓(field_lens.js) |
| Range Falloff(index の帯: Indices / Percentage / Transition) | `k = (i >= a && i < b) ? 1 : 0` | 純(i) | 箱の札 | Stagger From ✓(Center)、帯の指定は無し |
| Noise「Time is automatically connected to the comp's frame」Simplex/Cellular、Looping | `v = amp * noise(x*f, y*f, t*s)` | 純(p, t, seed) | 場 | **無し**(例に 0 件、zz_jitter は render の実験) |
| Value / Math / Random utilities | `v = base + off; random(seed+i)` | 純 | 留め具 | 台本の定数と random(seed) ✓ |
| Connect「data from one Layer's attribute is being directly passed to another's」(入力は 1 本) | `b.rotation = a.x * k` を毎コマ | 純(他の値の関数) | 留め具 | Field は層参照 ✓、Connect From + Trace △(grid_intent.js)、汎用は無し |
| Stagger「sequential values between a minimum and maximum」+ Graph → Shape Time Offset | `delay = map(i, 0, n-1, min, max); t_i = t - delay` | 純(i) | 箱の札 | Group Stagger / Stagger From ✓、Arrive Stagger ✓ |
| Scheduling Group「Procedurally position child layers in time」Sequencing / Overlap / Schedule from End / Ordering Policy | `start[i] = start[i-1] + dur[i-1] - overlap` | 純(i, 前の尺) | 箱の札 | Stagger From End のみ(322690e4)、**Sequence / Overlap / Order 無し** |
| Wave「looping wave distortions along … Paths」Number of Waves / Travel | `for s along path: y += A*sin(k*s + travel)` | 純(s, t) | 物(形) | Wave ✓(sync.js) |
| Oscillator「trigonometric wave patterns」Sine/Cos/Tan、Min/Max、Time Offset | `v = map(sin(TWO_PI*f*t + φ), -1, 1, min, max)` | 純(t) | 留め具 / 物 | Oscillator ✓(sync.js、Offset の鍵) |
| Forge Dynamics「a 2d physics engine built on … Box2D」Bodies / Fields(Attractor, Direction, Drag, Vortex, Path, Buoyancy)/ Ground Mode / .sdcache | `v += g*dt; p += v*dt; resolve(contacts)` | **記憶**(積分・cache) | 場 + 部屋 = 箱 | PHYSICS ✓(Field Spread/Angle/Turn/Strength、Hardness、Margin)、Bounce / Push Apart = 純関数の嘘 ✓ |
| Trails「trails (lines) from the movement of other Shapes」(実験、Start Frame) | `hist.push(p); for q of hist: vertex(q)` | **記憶**(履歴; 取説は cache か再評価か言わない) | 物 | Repeater Delay = 残像 △、線は無し |
| Level Mode(Body Settings: characters / words / lines) | `for ch of text.chars: body(ch)` | 純(粒度の i) | 箱の粒度 | Split △(Words は物にならない、rel_proof_words_fall.js) |
| JS: Editor script(api.*)/ JS Layers(ctx.index, ctx.count, ctx.positionX) | 一度組む / `return f(ctx.index, t)` を毎コマ | 組む=純、式=純(t) | 留め具 | 台本 = 一度組む ✓、毎コマの式は無し(鍵と効果が時刻の純関数) |

## 3. Cavalry 自身のスクリプト — p5 か、node builder か
取説「Getting Started with Scripting」の例(抜粋、原文):
```js
var staggerId = api.create("stagger", "Bounce In Stagger");
api.set(staggerId, {"minimum": -timeOffset, "maximum": 0});
api.connect(staggerId, "id", subMeshId, "shapeTimeOffset");   // 時間差 = Stagger の id を時計のずれに配線
api.connect(subMeshId, "id", textShapeId, "deformers");
```
- **node builder**。`api.create` → `api.set`(辞書で一括)→ `api.connect(from, "id", to, attr)` → `api.parent`。台本は一度走って scene を組む。API は「**ONLY** available in the JavaScript Editor and not available when writing expressions in JavaScript Layers」
- **p5 の draw に当たるのは JS Layers** の方: 「JavaScript Layers run every frame」、値を 1 つ return、`ctx.index` / `ctx.count` / `ctx.positionX` が入る(= i と p)。Distribution にも Custom(JS)がある
- 2 層構造: **組むのは一度、値は毎コマ (i, p, t) の関数**。Motolii の台本(comp / for / effect / key)は「組むのは一度」の側と同型。違いは、Cavalry では上の 4 行(Stagger → Time Offset の配線)が文句 1 で、Motolii は `group.set("Stagger", 0.7)` の 1 札に畳む

## 4. 画 3 枚 — 芯の sketch(p5 擬似、≤ 8 行、つまみ 1 つ)
**Sync**(ewn、scenery.io、16 秒。anatomy: 2 つの物 + 2 つが住む箱)
```js
box = shapeAt(t)                      // 円 → 四角 → 帯 → カード → 輪 → 画面(箱だけが替わる)
for (o of [black, white]) {           // 触るのは 2 つだけ
  o.p += o.v; if (!box.contains(o.p)) o.v = reflect(o.v, box.normal(o.p)) }   // 縁で跳ね返る
echo = ring(box, n, i => rotate(i/n))  // こだま: 点の輪・同心の弧・回る札は box の中心を共有
lens = intersect(black, white); clip(lens); gradient(); text("Sync", lens.center)
```
つまみ: **box**(箱の形 1 つ)。物理も跳ね返りの 1 行、残りは box の関数。
**Disco**(Ben Drake、Cavalry 2.7.2: Duplicator + Noise + Random + Falloff + Camera Planar + Spherise + Glow。頁の静止画で確認: ミラーボール)
```js
for (i<cols) for (j<rows) {
  u = (i/cols + spin*t) % 1; v = j/rows
  p = spherise(u, v)                                   // 平面の升目 → 球(Spherise)
  c = lerpColor(c1, c2, noise(i*f, j*f, t) + random(seed+i+j*cols)*0.2)
  c *= falloff(dist(p, light), R)                      // 照り = Falloff
  rect(p, w*p.z, h*p.z, c) }
glow()
```
つまみ: **spin**(u のずれ = 回転)。
**Wall Of Text**(MotionDesignAutomation、2.5.4: Duplicator + Apply Layout + Noise + Stagger + Value + JS + String Manipulator。静止画: 緑の語の壁、1 語だけ色が違う)
```js
words = layout(text, width)                    // Apply Layout = 折り返し(箱)
for ((i, w) of words) {
  k = smoothstep(0, 1, t - stagger(i))         // 語の番目 → 時刻
  n = noise(i*0.3, t*0.5)
  fill(lerpColor(dark, green, k*n)); text(w, layout.pos(i)) }
```
つまみ: **stagger の max**(語と語の間)。

## 5. 見立て(5 行)
1. Cavalry が creative coding に足したのは **for の i を配線にした事**: index context が勝手に流れ、Stagger / Random / Falloff は i を書かずに受け取る。人が `i` を書かないのが商品化の芯
2. **時刻は draw() の外**: Noise の Time は comp の frame に自動接続、Shape Time Offset で写しごとの時計をずらす。sketch の `t` を層ごとの時計にした = Motolii「値は時刻の純関数」と同型
3. **Falloff は第一級の物**(形を持ち、Graph で曲げ、Probability で 0/1)— `map(dist)` を毎回書かせない。Motolii の Field(箱が場になる)は既にこれ、Range(index の帯)だけ無い
4. **記憶は 2 つだけ**(Forge = Box2D の cache、Trails)。残りは全部 (i, p, t) の純関数 — Motolii の法と同じ線引き。Bounce / Push Apart を純関数の嘘で持つのは Cavalry より一歩先
5. 持ち帰る: (a) Scheduling Group の Sequence / Overlap / Order を **箱の札**に(文句 1・9、最大の未達)、(b) **Noise を場の 1 効果**として(在庫 0、Disco / Wall of Text の芯)、(c) Level Mode = Split の粒度を物に、(d) JS Layers は **要らない**(台本は一度組む型で足りる、毎コマの式は鍵と効果が肩代わり)、(e) Distribution の Custom / Math は台本の for に既に在る

## Sources
- Ian Waters, How Cavalry came to be — https://medium.com/cavalry-animation/how-cavalry-came-to-be-5ace628d8c28(直接は 403、r.jina.ai 経由で読んだ)
- Ian Waters, The Magic of Cavalry (2019-01-29) — https://medium.com/cavalry-animation/the-magic-of-cavalry-27b9f6acbd5e
- Ian Waters, Some of your Cavalry questions answered (2019-02-05) — https://medium.com/cavalry-animation/some-of-your-cavalry-questions-answered-8f93a94e84fd
- Chris Hardcastle, Introducing Cavalry (2019-01-29) — https://medium.com/cavalry-animation/introducing-cavalry-cfb95c37f8be
- Creative Bloq, Fed up of Adobe… — https://www.creativebloq.com/features/cavalry-2d-motion-design-software
- School of Motion, A First Look at Cavalry — https://schoolofmotion.com/blog/a-first-look-at-cavalry; podcast 頁(2024-10) — https://schoolofmotion.com/blog/cavalry-animation; Houdini of 2D — https://schoolofmotion.com/blog/cavalry-houdini-of-2d-after-effects
- CG Channel (2020-08) — https://www.cgchannel.com/2020/08/check-out-cavalry-mainframes-cool-new-motion-design-tool/
- 取説(docs.cavalry.scenegroup.co は cavalry.studio/docs へ 301): Duplicator /docs/nodes/shapes/duplicator/、Distribution Types /docs/nodes/general/distribution-types/、Falloff /docs/nodes/utilities/falloff/、Range Falloff /docs/nodes/utilities/range-falloff/、Noise /docs/nodes/behaviours/noise/、Stagger /docs/nodes/behaviours/stagger/、Scheduling Group /docs/nodes/utilities/scheduling-group/、Wave /docs/nodes/behaviours/wave/、Oscillator /docs/nodes/behaviours/oscillator/、Value /docs/nodes/behaviours/value/、Forge Dynamics Shape /docs/nodes/shapes/forge-dynamics/forge-dynamics-shape/、Fields /docs/nodes/shapes/forge-dynamics/fields/、Trails /docs/nodes/shapes/trails/、Context /docs/getting-started/key-concepts/context/、Connections /docs/getting-started/key-concepts/connections/、JavaScript Layers /docs/nodes/general/javascript-layers/、JavaScript Utility /docs/nodes/utilities/javascript-utility/、Scripting Getting Started /docs/tech-info/scripting/scripting-getting-started/、API Module /docs/tech-info/scripting/api-module/、Example Files /docs/getting-started/example-files/、Dynamic Rendering /docs/user-interface/menus/window-menu/render-manager/dynamic-rendering/
- Scenery: Sync https://scenery.io/scenes/sync-rQysTIq1bYH/、Disco https://scenery.io/scenes/disco-s92OQn68gcX、Wall Of Text https://scenery.io/scenes/wall-of-text-CLQP37IVjAu(札の一覧と静止画のみ、ファイルは落としていない)
- Motolii 在庫: motolii/ui/native/src/editor/script/examples/{field_lens,stagger_reflow,wave_grid,sync,hero,matter,trace,burst,grid_intent,border_path,rel_proof_words_fall,web_s7_marquee}.js

