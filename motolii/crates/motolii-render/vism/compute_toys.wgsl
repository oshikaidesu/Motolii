// 関数の棚(module `package::compute_toys`): compute.toys の公開倉庫から、番号 × 時刻の純関数の動き 3 本。
// #3081 Camaradas(saruga): 拍で止めて動く。#2374 Revision 2025(0b5vr): 止めて跳ぶ movefuck。#2752 Kamoshika: 滑らかな矩形波。
// 名前は出典のまま。出典: docs/reviews/2026-09-18-motion-code-survey.md
import package::processing::random;

/// 秒 → 拍。
fn beat(time: f32, bpm: f32) -> f32 { return time * bpm / 60.0; }

/// beats_per_hold 拍止まり、beats_per_transition 拍で次へ。返すのは (何回目, 進み 0..1 を smoothstep で)。
fn hold_transition(b: f32, beats_per_hold: f32, beats_per_transition: f32) -> vec2f {
    let cycle = beats_per_hold + beats_per_transition;
    let phase = floor(b / cycle);
    let inside = b - phase * cycle;
    return vec2f(phase, smoothstep(beats_per_hold, cycle, inside));
}

/// 止まって、跳ぶ: 1 秒ごとに新しい乱数の位置へ、最初の 0.1 秒で移る。seed は物ごとの種。
fn movefuck(t: f32, seed: u32) -> vec3f {
    let i = u32(max(floor(t), 0.0));
    let a = vec3f(random(i, seed), random(i, seed + 1u), random(i, seed + 2u));
    let b = vec3f(random(i + 1u, seed), random(i + 1u, seed + 1u), random(i + 1u, seed + 2u));
    return mix(a, b, smoothstep(0.0, 0.1, fract(t)));
}

/// 滑らかな矩形波: f が大きいほど角が立つ(1 で正弦、大きいと矩形)。-1..1。
fn smoothSqWave(p: f32, f: f32) -> f32 { return clamp(sin(p * 6.2831853) * f, -1.0, 1.0); }
