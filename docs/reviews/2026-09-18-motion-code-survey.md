# 動きのシェーダコード調査 — compute.toys / Shadertoy / easing ライブラリ(2026-09-18)

対象: 「番号 k × 時刻 t の純関数」で書かれた気持ちのいい動き(Cavalry / AE / MoGraph 型)。流体・GI・レイマーチは外す。
Motolii 側の棚は `motolii/crates/motolii-render/vism/cavalry.wgsl`(cv_ease / cv_stagger / cv_falloff / cv_range / cv_oscillator / cv_grid・circle・line・spiral / cv_random / cv_noise)。

## 要約 5 行

1. compute.toys(public-shaders、wgsl 445 本 + slang 67 本)は物理・GI・フラクタルが大半で、番号×時刻の動きは 15 本足らず。芯は 3 本 — **#2024 ease function balls**(Penner 30 種を WGSL で完備)、**#3081 Camaradas 8k**(BPM の hold→transition で配置間を morph)、**#2374 Revision 2025**(`floor/fract` の止めて跳ぶ、`t - 0.1*prog` の記憶なし trail)。
2. 最良の一次資料は compute.toys でなく **Book of Shaders の kynd「MotionToolKit」(2016、13 本)** — `linearstep(begin,end,t)` で時刻に窓を切り、ease を掛け、`fract(t + k/n)` で追いかけさせる。Sequencing 1 本で wipe・collision・chase・spinner を全部組む。Motolii の「時刻の純関数」の先例そのもの。
3. easing の正本は **glsl-easings(MIT、Penner 30 種)** → **lygia animation/easing(WGSL と WESL 両方あり、33 本、Prosperity 非商用)** の系譜。Motolii の `cv_ease` は 6 種しか無く、bounce・sine・expo・circ・quart/quint・in/inOut の back/elastic が欠ける。
4. iq の「useful little functions」(expImpulse・sustainedImpulse・cubicPulse・expStep・gain・parabola・pcurve)は「叩いて戻る」「一瞬光る」の 1 行法。cavalry に無い。Golan Levin の 1 つまみ ease(doubleExponentialSigmoid)は「ease 1 本で決め切る」に合う。
5. Shadertoy は Cloudflare の人間確認で読めず(突破しない方針)、題名・作者・URL のみ。記憶が要るのは spring(#2563 の `mix(pos, target, easing)`)と storage 累積の trail(#2752)だけで、あとは全部 k×t の純関数で書ける。

## compute.toys の動きの札 top 15

採点: `easeIn/Out・elastic・bounce・stagger・bpm・beat・morph・pingpong・fract(time)` の出現 − `raymarch・fluid・pathtrac・pass_in` の出現。上位を読み、番号×時刻で書かれている順に並べ直した。作者は各 .json の `profile.username`、license 欄はいずれも空(compute.toys 側の既定を要確認)。

### 1. #2024 ease function balls — alph13
- URL: https://compute.toys/view/2024
- 動き: 6×5 の升に 30 個の玉、升番号 `frag_idx` で easing を選び、`custom.gap` 秒で左右往復(`n % 2` で向きを反転)。sine/quad/cubic/quart/quint/expo/circ/back/elastic/bounce × in/out/inOut の全部が WGSL で書いてある。
- 手: **ease**(番号→種類は repeat の索引)。
```wgsl
    let t = time.elapsed % custom.gap / custom.gap;
    let n = floor(time.elapsed / custom.gap);
    let se = vec2f(-40., 40.) * select(-1., 1., n % 2. == 0.);
    // ...
fn o_ease_elastic(x: f32) -> f32 {
    return  select(
        step(0., x),
        pow(2, -10 * x) * sin((10 * x - 0.75) * C4) + 1,
        x >= 0 && x <= 1,
    );
}
fn i_ease_bounce(x: f32) -> f32 {
    return 1 - o_ease_bounce(1 - x);
}
```

### 2. #3081 Camaradas 8k demo pieces — saruga(Revision 2026)
- URL: https://compute.toys/view/3081
- 動き: タングラム 7 片が `state_letter_c/a/m/…` の配置表を BPM 170 で回る。1 相 6 拍 = 4 拍 hold + 2 拍 transition、`smoothstep` で片ごとの Transform2D(pos/scale/anchor/angle)を mix。
- 手: **tome-hame(拍で止めて動く)+ morph(配置 A → B)**。番号 `piece_index` × 時刻の純関数、記憶なし。
```wgsl
    let beat = time_val * (BPM / 60.0);
    let total_beats = BEATS_PER_PHASE * f32(NUM_PHASES);
    let beat_mod = beat % total_beats;

    let phase = u32(floor(beat_mod / BEATS_PER_PHASE)) % NUM_PHASES;
    let beat_in_phase = beat_mod - f32(phase) * BEATS_PER_PHASE;
    let next_phase = (phase + 1u) % NUM_PHASES;

    var f: f32;
    if (beat_in_phase < BEATS_PER_HOLD) {
        f = 0.0;
    } else {
        let t = (beat_in_phase - BEATS_PER_HOLD) / BEATS_PER_TRANSITION;
        f = smoothstep(0.0, 1.0, t);
    }
```

### 3. #2374 20250421_Revision 2025 — x0b5vr(0b5vr、live coding)
- URL: https://compute.toys/view/2374
- 動き: `movefuck` = hash の目標点を `floor(t)` で切り替え、`smoothstep(0, 0.1, fract(t))` で 0.1 秒だけ跳んで残り 0.9 秒止まる(止めて跳ぶ)。`posHolo(prog)` は `time - 0.1*prog` の時間差で同じ式を評価して残像を作る(記憶なしの trail)。
- 手: **tome-hame / trail(時間差の重ね)**。
```wgsl
fn movefuck(t_: f32, heck: float3) -> float3 {
    var t = t_;
    t += cyclic(heck + 2.0 + t, 0.5, 1.0).x;

    return mix(
        hash3f(heck + floor(t)),
        hash3f(heck + floor(t) + 1.0),
        smoothstep(0.0, 0.1, fract(t))
    );
}
// ...
        let prog = float(i) / SPLAT_ITER_F + random3f().x;
        let t = time.elapsed - 0.1 * prog;
```

### 4. #2781 pngine logo — saruga
- URL: https://compute.toys/view/2781
- 動き: 13×15 の升で「P」の字を抜き(`is_cell_hidden`)、`sin(col*0.5 + row*0.3 + time*2)` の斜め波で升の箱を伸縮。
- 手: **repeat(grid)+ stagger(番号→位相)+ kinetic typography(升で字)**。
```wgsl
    // Wave travels diagonally across the grid
    let wave = sin(grid_col * 0.5 + grid_row * 0.3 + time_offset * 2.0);

    // Box size pulses with the wave (smaller = more margin)
    let box_scale = map_range(wave, -1.0, 1.0, 0.1, 0.5);
    let margin = 1.0 - box_scale;
```

### 5. #2752 Butterflies and Trails — Kamoshika(SESSIONS 2025 shader jam)
- URL: https://compute.toys/view/2752
- 動き: `T = time * BPM/60 * 0.5` で拍に時計を合わせ、`smoothSqWave` の滑らかな矩形波で羽ばたき。残像は storage buffer に加算(記憶あり)。
- 手: **oscillator(smooth square)+ tome(BPM)**、trail は memory-needed。
```wgsl
const BPM = 148.;
fn smoothSqWave(p: f32, f: f32) -> f32 {
    let x = p - 0.5;
    let odd = fmod(floor(x), 2.);
    let factor = f * (odd * 2. - 1.);
    let res = smoothstep(0.5 - factor, 0.5 + factor, fract(x));
    return res * 2. - 1.;
}
    var T = time.elapsed * BPM / 60. * 0.5; // BPMに合わせて速さを変更した時間
```

### 6. #2097 fake 3d grid — alph13
- URL: https://compute.toys/view/2097
- 動き: `pingpong` の三角波を `smoothstep` で丸めて ±30° に写し、升目を回す。
- 手: **oscillator(triangle → smooth)**。cv_oscillator の waveform 1 + smoothstep で同値。
```wgsl
    var t = pingpong(time.elapsed, 1.2);
    t = smoothstep(0., 1.2, t);
    t = mix(-PI / 6., PI / 6., t);
fn pingpong(x: f32, l: f32) -> f32 {
    return abs(modulo(x, 2. * l) - l);
```

### 7. #2098 animated sdf onion — alph13
- URL: https://compute.toys/view/2098
- 動き: 升ごとに違う SDF、`fract(0.5*t)` を smoothstep で 1 周の角度に、2 秒ごとに `s = u32(mod(t,4)/2)` で描き方を切替。
- 手: **repeat(grid)+ ease(周期 1 周)+ tome-hame(段階切替)**。
```wgsl
    let r = fract(0.5 * time.elapsed);
    let t = 2.0 * PI * smoothstep(0., 1., r);
    // ...
    let s = u32(modulo(time.elapsed, 4.) / 2.);
    let a = -fract(time.elapsed) * 16.;
```

### 8. #2137 a try of truchet — alph13
- URL: https://compute.toys/view/2137
- 動き: Truchet の升、`o_ease_circ` と `linearstep` で角の丸みを動かす。
- 手: **ease(circ out)+ repeat**。
```wgsl
fn linearstep(e0: f32, e1: f32, x: f32) -> f32 {
    return clamp((x - e0) / (e1 - e0), 0.0, 1.0);
}
fn o_ease_circ(x: f32) -> f32 {
    let t = 1 - x;
    return sqrt(1 - t * t);
```

### 9. #1044 Asahi illusion — altunenes
- URL: https://compute.toys/view/1044
- 動き: `curve(x,a,b) = smoothstep(a,b,x)*smoothstep(b,a,x)` の山で葉の色を往復。
- 手: **oscillator(pulse 1 山)**。iq の cubicPulse と同じ役。
```wgsl
fn curve(x: f32, a: f32, b: f32) -> f32 {
    let y: f32 = smoothstep(a, b, x) * smoothstep(b, a, x);
```

### 10. #1553 Concentric triangles — chriscamplin(Xor の写し)
- URL: https://compute.toys/view/1553
- 動き: 同心の三角 i 本、`angle + time` で全部回す(番号 i が半径)。
- 手: **repeat(番号→半径)+ 等速回転**。
```wgsl
    let t = time.elapsed * 0.5;
        let triangle = sdEquilateralTriangle(uv, i, angle + time.elapsed);
```

### 11. #2117 checkerboard tunnel/hyperspace — alph13
- URL: https://compute.toys/view/2117
- 動き: `fract(time)` で 1 周期ぶん奥へ流す(継ぎ目なしの loop)。
- 手: **repeat(時刻の loop)**。
```wgsl
    let i = size.y * custom.r_lambda / r + 2. * custom.r_lambda * fract(time.elapsed);
```

### 12. #1752 Colorspace conversion visualized — schobbejack
- URL: https://compute.toys/view/1752
- 動き: RGB 立方体の点群を YCbCr の配置へ、点ごとに `sin(sin(...))^4` の位相ずれで往復(morph の stagger)。
- 手: **morph(配置 A → B)+ stagger(点ごとの位相)**。
```wgsl
    let col = mix(
        // ...
        pow( abs(sin(sin((time.elapsed+rgb.x+r)*0.2*PI/2.)*PI/2.)), 4.)
```

### 13. #1383 Vectorscope clock — schobbejack
- URL: https://compute.toys/view/1383
- 動き: `t % period / period * 2π` で秒・分・時・12 時間の針。
- 手: **tome(周期の位相)**。
```wgsl
    let fm = array<f32,4>(1.,60.,3600.,43200.);
    let m = floor(clamp(uv.y+2.5,1.,4.)-1.);
    let r = t%fm[int(m)] * (1./fm[int(m)])*pi2;
```

### 14. #2563 spring angle constraint — alph13
- URL: https://compute.toys/view/2563
- 動き: 前フレームの位置を storage に持ち、`mix(position, target, easing)` で追従(記憶あり)。
- 手: **memory-needed(Cavalry の Dynamics / Lerp)**。cavalry.wgsl の方針どおり棚には置かない。
```wgsl
    data.position = mix(data.position, vec2f(mouse.pos), custom.easing);
```

### 15. #908 Grid — alph13
- URL: https://compute.toys/view/908
- 動き: `sin(4*time)` で升全体を上下に揺らす。
- 手: **oscillator(sine)**。
```wgsl
    fragCoord -= vec2f(0.0, sin(4 * time.elapsed) * screen_size.y);
```

(落とした物: #2718 Descending Jumps は拡散 sim、#1465 fake motion は静止画の錯視、slang 67 本に動きの札は無し。#2385 Mills の `lerp(0.6,1,sin(time)*.5+.5)` 程度。)

## Shadertoy と easing ライブラリ

### kynd「MotionToolKit」(Book of Shaders gallery、2016-09-09)— 最重要
- 一覧: https://thebookofshaders.com/examples/?chapter=motionToolKit 、各ソースは `https://thebookofshaders.com/log/<id>.frag`
- 13 本: Easing Functions(160909064320)/ Timing functions 1(160909064357・160909064458)/ Distance Field Shapes(160909064528)/ Mixing(160909041106)/ 4 Beat(160909064609)/ Polyrhythm(160909064651)/ Blinking(160909064723)/ Tween Animations(160909064802)/ Chasing(160909064829)/ Collision(160909065019)/ Wipes(160909065049)/ Sequencing(160909065147)
- 法: **時刻に窓を切る `linearstep(begin, end, t)`** → ease → `mix(from, to, ease(...))`。行きと帰りは `p_go - p_back` の引き算。追いかけは `fract(t + k/n)` の位相ずらし(= stagger)。拍は `stepUpDown(begin, end, fract(t))`。
```glsl
float linearstep(float begin, float end, float t) {
    return clamp((t - begin) / (end - begin), 0.0, 1.0);
}
float linearstepUpDown(float upBegin, float upEnd, float downBegin, float downEnd, float t) {
    return linearstep(upBegin, upEnd, t) - linearstep(downBegin, downEnd, t);
}
float stepUpDown(float begin, float end, float t) {
  return step(begin, t) - step(end, t);
}
```
```glsl
// Tween Animations: 行き(0.1〜0.5)と帰り(0.6〜1.0)を引き算で 1 本に
    float t2 = linearstep(0.1, 0.5, t);
    float p2 = easeInOutCubic(t2);
    float t3 = linearstep(0.6, 1.0, t);
    float p3 = easeInOutCubic(t3);
    v = max(v, dotPlot(st - vec2(mix(0.2, 0.8, p2 - p3), 0.4)));
```
```glsl
// Chasing: 同じ式を位相 k/4 ずらして 4 つ
float chasers(vec2 st, float t) {
    t = fract(t);
    float v = chaser(st, fract(t));
    v = max(v, chaser(st, fract(t + 0.25)));
    v = max(v, chaser(st, fract(t + 0.5)));
    v = max(v, chaser(st, fract(t + 0.75)));
    return v;
}
```
```glsl
// Collision: 行きは easeIn、ぶつかって戻るのは easeOut
float collider(vec2 p, vec2 b, vec2 e, float t) {
    float t0 = linearstep(0.0, 0.5, t);
    float p0 = easeInCubic(t0);
    float t1 = linearstep(0.5, 1.0, t);
    float p1 = easeOutQuad(t1);
    return rectPlot(p - mix(b, e, p0 - p1), vec2(0.05));
}
```
- 4 Beat / Polyrhythm: `fract(u_time / 2.0)` を 1/8 ずつ `stepUpDown` で 4 分割 → 拍の升。Blinking: `fract(u_time / n)` で 1・2・3・4 秒周期を並べる(ポリリズム)。
- ライセンス: 各 .frag の頭は `// Author @kyndinfo - 2016` のみ、明示なし(repo の LICENSE は本文の All rights reserved)。**値と型だけ写す**。

### glsl-easings(glslify / stackgl、Hugh Kennedy)— MIT
- https://github.com/glslify/glsl-easings — Penner の 30 種(back/bounce/circular/cubic/elastic/exponential/linear/quadratic/quartic/quintic/sine × in/out/inOut)。lygia と compute.toys #2024 の元。
```glsl
float elasticOut(float t) {
  return sin(-13.0 * (t + 1.0) * HALF_PI) * pow(2.0, -10.0 * t) + 1.0;
}
float backInOut(float t) {
  float f = t < 0.5
    ? 2.0 * t
    : 1.0 - (2.0 * t - 1.0);
  float g = pow(f, 3.0) - f * sin(f * PI);
  return t < 0.5
    ? 0.5 * g
    : 0.5 * (1.0 - g) + 0.5;
}
```

### lygia `animation/easing`(Patricio Gonzalez Vivo)— Prosperity 3.0(非商用)/ Patron
- https://lygia.xyz/animation/easing 、https://github.com/patriciogonzalezvivo/lygia/tree/main/animation/easing
- **WGSL と WESL の両方**が既にある(`backOut.wesl` は `import lygia::animation::easing::backIn::{backIn};` で引く — Motolii の `import package::cavalry::{...}` と同じ形)。33 本。`animation/spriteLoop.wgsl` は `time % (end - start)` のコマ送り。`math/` に gain・parabola・bump・map・smootherstep・cubicMix。
```wgsl
fn elasticOut(t: f32) -> f32 { return sin(-13.0 * (t + 1.0) * HALF_PI) * pow(2.0, -10.0 * t) + 1.0; }
fn backOut(t: f32) -> f32 { return 1.0 - backIn(1.0 - t); }
fn gain(x: f32, k: f32) -> f32 {
    let a = 0.5 * pow(2.0 * select(1.0-x, x, x<0.5), k);
    return select(1.0-a, a, x<0.5);
}
fn parabola(x: f32, k: f32) -> f32 { return pow( 4.0*x*(1.0-x), k ); }
fn spriteLoop(tex: texture_2d<f32>, samp: sampler, st: vec2f, grid: vec2f, start_index: f32, end_index: f32, time: f32) -> vec4f {
  let frame = time % (end_index - start_index);
  return sampleSprite(tex, samp, st, grid, start_index + frame);
}
```

### Book of Shaders 第 5 章 Shaping functions
- https://thebookofshaders.com/05/ — step / smoothstep / sin / pow / exp / log / sqrt / fract / floor / ceil、`smoothstep(0.2,0.5,x) - smoothstep(0.5,0.8,x)` の山。参照先として glsl-easings・Golan Levin・iq・kynd を挙げる。

### Inigo Quilez「useful little functions」
- https://iquilezles.org/articles/functions/ (ライセンス表記なし、式が短い)
```glsl
float expImpulse( float x, float k ) { float h = k*x; return h*exp(1.0-h); }
float sustainedImpulse( float x, float f, float k ) {
  float s = max(x-f,0.0);
  return min( x*x/(f*f), 1.0+(2.0/f)*s*exp(-k*s)); }
float cubicPulse( float x, float c, float w ) {
  x = abs(x - c); if( x>w ) return 0.0; x /= w; return 1.0 - x*x*(3.0-2.0*x); }
float expStep( float x, float k ) { return exp2(-pow(x,k)); }
float pcurve( float x, float a, float b ) {
  float k = pow(a+b,a+b)/(pow(a,a)*pow(b,b)); return k*pow(x,a)*pow(1.0-x,b); }
float almostUnitIdentity( float x ) { return x*x*(2.0-x); }
```
- 役: expImpulse = 叩いて減衰(1 拍のアタック)、sustainedImpulse = 立ち上がって振れて 1 に落ち着く(overshoot 1 回)、cubicPulse = 一瞬の山、expStep = 0 に落ちる falloff の型、pcurve = 山の左右非対称。

### Golan Levin「Shaping Functions」(flong.com、© 2020)
- https://www.flong.com/archive/texts/code/shapers_exp/ (poly / exp / circ / bez の 4 頁)
- `exponentialEasing(x, a)`・`doubleExponentialSigmoid(x, a)`・`doubleExponentialSeat`・`logisticSigmoid`: つまみ a 1 個で in↔out・きつさを連続に変える。「ease 1 本で決め切る」に合う型。
```glsl
float doubleExponentialSigmoid (float x, float a){
  a = 1.0-a;
  float y = 0;
  if (x<=0.5){ y = (pow(2.0*x, 1.0/a))/2.0; }
  else { y = 1.0 - (pow(2.0*(1.0-x), 1.0/a))/2.0; }
  return y;
}
```

### Shadertoy(題名・作者・URL のみ。ページは Cloudflare の人間確認で読めず、突破はしない)
- 既定ライセンスは CC BY-NC-SA 3.0(各作品で上書きあり)。
- Easing functions — FabriceNeyret2 — https://www.shadertoy.com/view/wtcczf
- Easing Functions Colormix — FabriceNeyret2 — https://www.shadertoy.com/view/ms3cD2
- Easing Functions - interpolation — 作者未確認 — https://www.shadertoy.com/view/sd3fzB
- byte work Ease functions — 作者未確認 — https://www.shadertoy.com/view/wlsXDf
- Lerp / Smoothstep Implementation — 作者未確認 — https://www.shadertoy.com/view/ftyBDm
- Loading Spinner — 作者未確認 — https://www.shadertoy.com/view/NdXSzM
- Google Loading Spinner — 作者未確認 — https://www.shadertoy.com/view/3tVXz3
- shadertoy loading animation — 作者未確認 — https://www.shadertoy.com/view/dsV3zc
- BPM Flash(リズムゲーム用、拍で光る)— 作者未確認 — https://www.shadertoy.com/view/dtBGzw
- Beating Circles / Beat Visualization 1 — https://www.shadertoy.com/view/4d23Ww 、https://www.shadertoy.com/view/MsGfWd
- Squares Analytic Motion Blur — https://www.shadertoy.com/view/wtcSzB
- Easy Text in Shaders — https://www.shadertoy.com/view/dsGXDt 、ShadertoyText(knarkowicz)https://github.com/knarkowicz/ShadertoyText
- Simple glowing clock — https://www.shadertoy.com/view/7tyyWc
- 代わりに読めた同型: Godot Shaders「Loading shader」(duongxinh2003、CC0)https://godotshaders.com/shader/loading-shader/ — 点 i の半径を `mod(TIME * -speed - i * 0.1, 1.0)` で追いかけさせる(= stagger を fract で)。
```glsl
        float dot_radius = (spinner_radius_factor / 0.7) * 0.1 * mod(TIME * -rotation_speed - i * 0.1, 1.0);
```

### Slang playground
- https://shader-slang.org/slang-playground/ 、demo は https://github.com/shader-slang/slang-playground/tree/main/public/demos (circle / ocean / gsplat2d / painting / autodiff …)。動きは `circle.slang`(Shadertoy XdlSDs の写し、dynamite、CC BY-NC-SA 3.0)の `frame / 60` の等速回転程度。MoGraph 型の例は無し。

## 表

| source | motion law | index×time pure? | needs memory? | Motolii の関数 | license |
|---|---|---|---|---|---|
| compute.toys #2024 ease function balls | Penner 30 種 ease、往復 `n%2` で反転 | yes | no | cv_ease(6 種のみ)→ **欠け: bounce/sine/expo/circ/quart/quint、in/inOut の back・elastic** | 記載なし |
| compute.toys #3081 Camaradas 8k | BPM hold→transition、配置表の morph | yes | no | 無し → **beat / hold-transition / mixTransform** | 記載なし |
| compute.toys #2374 Revision 2025 | `floor/fract` の止めて跳ぶ、`t - 0.1*prog` の trail | yes | no | 無し → **snap(movefuck)/ time-offset trail** | 記載なし |
| compute.toys #2781 pngine logo | grid + `sin(col*a + row*b + t)` の波、升で字 | yes | no | cv_grid + cv_oscillator(位相を番号で) | 記載なし |
| compute.toys #2752 Butterflies and Trails | BPM 時計、smoothSqWave、storage 累積の残像 | 動きは yes | trail は yes | cv_oscillator(square は硬い)→ **smoothSqWave** | 記載なし |
| compute.toys #2097 fake 3d grid | pingpong + smoothstep | yes | no | cv_oscillator(1) + smoothstep で同値 | 記載なし |
| compute.toys #2098 animated sdf onion | fract→smoothstep 1 周、2 秒段階切替 | yes | no | cv_ease + `floor(t/2)` | 記載なし |
| compute.toys #2137 truchet | o_ease_circ、linearstep | yes | no | 欠け: circ out、linearstep | 記載なし |
| compute.toys #1044 Asahi illusion | `smoothstep(a,b,x)*smoothstep(b,a,x)` の山 | yes | no | 無し → cubicPulse 相当 | 記載なし |
| compute.toys #1553 Concentric triangles | 番号→半径、等速回転 | yes | no | cv_circle 系 + 等速 | 記載なし |
| compute.toys #2117 hyperspace | `fract(time)` の loop | yes | no | fract で足りる | 記載なし |
| compute.toys #1752 Colorspace | 配置 A→B、点ごと位相 | yes | no | 無し → morph(mix) + stagger | 記載なし |
| compute.toys #1383 Vectorscope clock | `t % period / period` | yes | no | fract で足りる | 記載なし |
| compute.toys #2563 spring angle | `mix(pos, target, k)` 追従 | no | **yes** | 置かない(Dynamics は解き手側) | 記載なし |
| compute.toys #908 Grid | `sin(4t)` | yes | no | cv_oscillator(0) | 記載なし |
| kynd MotionToolKit Easing | Penner quad/cubic/expo × in/out/inOut | yes | no | cv_ease の欠け | 明示なし |
| kynd Timing functions | `linearstep(b,e,t)`・`linearstepUpDown`・`stepUpDown` | yes | no | **欠け: linearstep / stepUpDown**(cv_range は番号用) | 明示なし |
| kynd 4 Beat / Polyrhythm / Blinking | `fract(t/n)` を stepUpDown で分割 | yes | no | fract + stepUpDown | 明示なし |
| kynd Tween / Chasing / Collision | 窓→ease→mix、行き−帰り、`fract(t + k/n)` | yes | no | cv_stagger(遅れ秒)で位相は作れる、窓が無い | 明示なし |
| kynd Wipes / Sequencing | clockWipe・stripe・ringSpinner を窓で並べる | yes | no | 欠け: clockWipe(角度の wipe) | 明示なし |
| glsl-easings | Penner 30 種 | yes | no | cv_ease の欠け | MIT |
| lygia animation/easing(wgsl・wesl) | Penner 33 本、spriteLoop | yes | no | cv_ease の欠け、spriteLoop(コマ送り) | Prosperity 3.0 / Patron |
| lygia math | gain・parabola・bump・map・smootherstep | yes | no | map は processing.wgsl に有、**gain / parabola / smootherstep 無し** | Prosperity 3.0 / Patron |
| iq functions | expImpulse・sustainedImpulse・cubicPulse・expStep・pcurve・almostUnitIdentity | yes | no | **全部無し** | 表記なし |
| Golan Levin shapers | exponentialEasing(x,a)・doubleExponentialSigmoid(x,a) | yes | no | 無し(つまみ 1 個の ease) | © 2020 Golan Levin |
| Book of Shaders ch.5 | step/smoothstep/pow/exp の造形 | yes | no | 組み込み | 本文 All rights reserved |
| Godot Loading shader | `mod(T*-s - i*0.1, 1)` の追いかけ | yes | no | cv_stagger + fract | CC0 |
| Shadertoy 各種(未読) | ease 一覧・spinner・BPM flash・text | 題名から yes | — | — | CC BY-NC-SA 3.0(既定) |
| Slang playground circle | `frame/60` 等速 | yes | no | — | CC BY-NC-SA 3.0 |

## 棚に無い関数(署名付き)

名前は出所のまま(発明しない)。置き場は `cavalry.wgsl` か、Cavalry の語でない物は別 module(`easing` / `functions`)が筋 — 裁定は利用者。

### 1. Penner の ease 全種(glsl-easings / lygia / kynd / #2024)
cv_ease は linear・in・out・inOut(power 2)・backOut・elasticOut の 6 種。lygia の名で足す(WESL 版がそのまま先例)。
```wgsl
fn sineIn(t: f32) -> f32;      fn sineOut(t: f32) -> f32;      fn sineInOut(t: f32) -> f32;
fn cubicIn(t: f32) -> f32;     fn cubicOut(t: f32) -> f32;     fn cubicInOut(t: f32) -> f32;
fn quarticIn(t: f32) -> f32;   fn quarticOut(t: f32) -> f32;   fn quarticInOut(t: f32) -> f32;
fn quinticIn(t: f32) -> f32;   fn quinticOut(t: f32) -> f32;   fn quinticInOut(t: f32) -> f32;
fn exponentialIn(t: f32) -> f32; fn exponentialOut(t: f32) -> f32; fn exponentialInOut(t: f32) -> f32;
fn circularIn(t: f32) -> f32;  fn circularOut(t: f32) -> f32;  fn circularInOut(t: f32) -> f32;
fn backIn(t: f32) -> f32;      fn backInOut(t: f32) -> f32;
fn elasticIn(t: f32) -> f32;   fn elasticInOut(t: f32) -> f32;
fn bounceIn(t: f32) -> f32;    fn bounceOut(t: f32) -> f32;    fn bounceInOut(t: f32) -> f32;
```
(cv_ease の `kind` 番号を増やすなら Cavalry の Interpolation の並びに合わせる。)

### 2. 時刻の窓(kynd MotionToolKit)
Sequencing の芯。cv_range は番号の窓、これは時刻の窓。
```wgsl
fn linearstep(begin: f32, end: f32, t: f32) -> f32;
fn linearstepUpDown(upBegin: f32, upEnd: f32, downBegin: f32, downEnd: f32, t: f32) -> f32;
fn stepUpDown(begin: f32, end: f32, t: f32) -> f32;
fn clockWipe(p: vec2f, t: f32) -> f32;
```

### 3. 拍で止めて動く(#3081 Camaradas)
`beat = t * bpm / 60`、hold と transition を拍で切る。#3081 の名は `get_animated_transform` で個別すぎるので、出所の変数名 `beat` と `mixTransform` を採る。
```wgsl
fn beat(time: f32, bpm: f32) -> f32;                                        // 拍の時刻
fn hold_transition(beat: f32, beats_per_hold: f32, beats_per_transition: f32) -> vec2f; // (phase, f)
fn mixTransform(a: Transform2D, b: Transform2D, t: f32) -> Transform2D;    // Motolii では Offset の mix
```

### 4. 止めて跳ぶ(#2374、名は出所どおり `movefuck` — 改名は利用者の裁定)
```wgsl
fn movefuck(t: f32, seed: vec3f) -> vec3f;   // mix(hash(floor t), hash(floor t + 1), smoothstep(0, 0.1, fract t))
```

### 5. 滑らかな矩形波(#2752 Kamoshika)
cv_oscillator の square は硬い。
```wgsl
fn smoothSqWave(p: f32, f: f32) -> f32;
```

### 6. iq の 1 行法(iquilezles.org/articles/functions)
```wgsl
fn expImpulse(x: f32, k: f32) -> f32;
fn sustainedImpulse(x: f32, f: f32, k: f32) -> f32;
fn cubicPulse(x: f32, c: f32, w: f32) -> f32;
fn expStep(x: f32, k: f32) -> f32;
fn pcurve(x: f32, a: f32, b: f32) -> f32;
fn almostUnitIdentity(x: f32) -> f32;
fn gain(x: f32, k: f32) -> f32;        // lygia math/gain.wgsl と同名
fn parabola(x: f32, k: f32) -> f32;    // lygia math/parabola.wgsl と同名
```

### 7. つまみ 1 個の ease(Golan Levin)
```wgsl
fn exponentialEasing(x: f32, a: f32) -> f32;
fn doubleExponentialSigmoid(x: f32, a: f32) -> f32;
```

### 8. コマ送り(lygia animation/spriteLoop)
```wgsl
fn spriteLoop(tex: texture_2d<f32>, samp: sampler, st: vec2f, grid: vec2f, start_index: f32, end_index: f32, time: f32) -> vec4f;
```

### 置かない物(記憶が要る)
- #2563 の `mix(position, target, easing)`(Dynamics / Lerp)、#2752 の storage 累積 trail。cavalry.wgsl の頭書きどおり解き手側。trail は #2374 の `t - 0.1 * prog` の時間差評価で記憶なしに出せる。

## Sources

- compute-toys/public-shaders: https://github.com/compute-toys/public-shaders (clone は scratchpad/motion-code/public-shaders)
- compute.toys #2024 https://compute.toys/view/2024 、#3081 https://compute.toys/view/3081 、#2374 https://compute.toys/view/2374 、#2781 https://compute.toys/view/2781 、#2752 https://compute.toys/view/2752 、#2097 https://compute.toys/view/2097 、#2098 https://compute.toys/view/2098 、#2137 https://compute.toys/view/2137 、#1044 https://compute.toys/view/1044 、#1553 https://compute.toys/view/1553 、#2117 https://compute.toys/view/2117 、#1752 https://compute.toys/view/1752 、#1383 https://compute.toys/view/1383 、#2563 https://compute.toys/view/2563 、#908 https://compute.toys/view/908
- kynd MotionToolKit: https://thebookofshaders.com/examples/?chapter=motionToolKit 、ソース https://thebookofshaders.com/log/160909064320.frag 〜 160909065147.frag(repo: https://github.com/patriciogonzalezvivo/thebookofshaders/tree/master/motionToolKit)
- Book of Shaders ch.5: https://thebookofshaders.com/05/
- glsl-easings: https://github.com/glslify/glsl-easings (MIT)
- lygia: https://lygia.xyz/animation/easing 、https://github.com/patriciogonzalezvivo/lygia (Prosperity 3.0 / Patron、README_WESL.md)
- Inigo Quilez functions: https://iquilezles.org/articles/functions/
- Golan Levin shapers: https://www.flong.com/archive/texts/code/shapers_exp/ (poly / circ / bez も同階層)
- Godot Shaders Loading shader: https://godotshaders.com/shader/loading-shader/ (CC0)
- Shadertoy: https://www.shadertoy.com/view/wtcczf 、ms3cD2 、sd3fzB 、wlsXDf 、ftyBDm 、NdXSzM 、3tVXz3 、dsV3zc 、dtBGzw 、4d23Ww 、MsGfWd 、wtcSzB 、dsGXDt 、7tyyWc 、FabriceNeyret2 https://www.shadertoy.com/user/FabriceNeyret2
- ShadertoyText: https://github.com/knarkowicz/ShadertoyText
- Slang playground: https://shader-slang.org/slang-playground/ 、https://github.com/shader-slang/slang-playground/tree/main/public/demos
- Motolii の棚: motolii/crates/motolii-render/vism/cavalry.wgsl 、processing.wgsl(map / norm / random / noise)
