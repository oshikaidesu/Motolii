// 関数の棚(WESL の module `package::cavalry`): Cavalry のノードを関数として引用する。下の段は processing.wgsl(map・norm・random・noise)。
// manifest の頭が無い .wgsl は札でなく module。札は `import package::cavalry::{ cv_ease };` で引く。
// 名前と欄は Cavalry の語のまま(docs.cavalry.scenegroup.co)。記憶を持つノード(Lerp・Trails・Dynamics)は無い — 解き手側。
// 引用のルール: 札はここの関数を import して呼ぶだけ。ここを直せば引いた札全部に届く(利用者 2026-09-18)。
import package::processing::{ random, noise };

// ── Utilities ────────────────────────────────────────────────────────────────

/// Random: 番号と種から [-1, 1](Processing の random を中心 0 に)。
fn cv_random(index: u32, seed: u32) -> f32 {
    return random(index, seed) * 2.0 - 1.0;
}

/// Noise: 2D の値ノイズ、[-1, 1]。Frequency で細かさ、Evolution で時刻を混ぜる(Processing の noise の上)。
fn cv_noise(p: vec2f, frequency: f32, evolution: f32) -> f32 {
    let q = p * frequency + vec2f(evolution * 0.37, evolution * 0.71);
    return noise(q.x, q.y);
}

/// Oscillator: Waveform 0 = Sine, 1 = Triangle, 2 = Square, 3 = Sawtooth。Frequency は Hz、Phase は周。
fn cv_oscillator(time: f32, frequency: f32, phase: f32, waveform: u32) -> f32 {
    let x = fract(time * frequency + phase);
    switch waveform {
        case 1u: { return 1.0 - 4.0 * abs(x - 0.5); }
        case 2u: { return select(-1.0, 1.0, x < 0.5); }
        case 3u: { return x * 2.0 - 1.0; }
        default: { return sin(x * 6.2831853); }
    }
}

/// Ease(Cavalry の Interpolation の型、GSAP/CSS の名前): 0 = linear, 1 = in, 2 = out, 3 = inOut(power 2), 4 = back out, 5 = elastic out。
fn cv_ease(u: f32, kind: u32) -> f32 {
    let t = clamp(u, 0.0, 1.0);
    switch kind {
        case 1u: { return t * t; }
        case 2u: { return 1.0 - (1.0 - t) * (1.0 - t); }
        case 3u: { return select(2.0 * t * t, 1.0 - 2.0 * (1.0 - t) * (1.0 - t), t >= 0.5); }
        case 4u: { let s = 1.70158; let v = t - 1.0; return 1.0 + v * v * ((s + 1.0) * v + s); }
        case 5u: { return select(1.0 - pow(2.0, -10.0 * t) * cos(t * 10.0 * 6.2831853 / 3.0), 1.0, t >= 1.0); }
        default: { return t; }
    }
}

/// Stagger(Cavalry の Stagger / GSAP の stagger): 番号 k、数 n、From 0 = start, 1 = end, 2 = center。返すのは遅れ(秒)。
fn cv_stagger(k: u32, n: u32, amount: f32, origin: u32) -> f32 {
    let m = f32(max(n, 1u) - 1u);
    let i = f32(k);
    switch origin {
        case 1u: { return (m - i) * amount; }
        case 2u: { return abs(i - m * 0.5) * amount; }
        default: { return i * amount; }
    }
}

/// Falloff(Cavalry の Falloff / MoGraph の Falloff): 距離 d を Radius で 1 → 0 に、Ease の型で。
fn cv_falloff(d: f32, radius: f32, kind: u32) -> f32 {
    return 1.0 - cv_ease(clamp(d / max(radius, 1e-6), 0.0, 1.0), kind);
}

/// Range(Cavalry の Range / AE の Range Selector): [lo, hi] の番号だけ 1、外は 0、縁は soft で滑らかに。
fn cv_range(k: u32, n: u32, lo: f32, hi: f32, soft: f32) -> f32 {
    let x = f32(k) / f32(max(n, 1u) - 1u);
    let s = max(soft, 1e-6);
    return smoothstep(lo - s, lo, x) * (1.0 - smoothstep(hi, hi + s, x));
}

// ── Distributions(Duplicator の Layout に相当、番号 → 位置)────────────────────

/// Grid: cols 列、cell の間隔、中心が原点。
fn cv_grid(k: u32, n: u32, cols: u32, cell: vec2f) -> vec2f {
    let c = max(cols, 1u);
    let rows = (n + c - 1u) / c;
    return vec2f((f32(k % c) - f32(c - 1u) * 0.5) * cell.x, (f32(k / c) - f32(rows - 1u) * 0.5) * cell.y);
}

/// Circle: 半径 radius、Start Angle から等間隔(度)。
fn cv_circle(k: u32, n: u32, radius: f32, start_degrees: f32) -> vec2f {
    let a = radians(start_degrees) + 6.2831853 * f32(k) / f32(max(n, 1u));
    return vec2f(cos(a), sin(a)) * radius;
}

/// Line: from → to に等間隔。
fn cv_line(k: u32, n: u32, from_point: vec2f, to_point: vec2f) -> vec2f {
    return mix(from_point, to_point, f32(k) / f32(max(n, 1u) - 1u));
}

/// Spiral(Cavalry の Distribution: Spiral): turns 周、半径は 0 → radius。
fn cv_spiral(k: u32, n: u32, radius: f32, turns: f32) -> vec2f {
    let u = f32(k) / f32(max(n, 1u) - 1u);
    let a = u * turns * 6.2831853;
    return vec2f(cos(a), sin(a)) * radius * u;
}
