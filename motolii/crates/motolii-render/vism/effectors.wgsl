// 関数の棚の 3 段目(module `package::effectors`): Effector(Notch の Clone Effector / Unreal Motion Design の Effector / MASH・MoGraph の Effector)。
// 調査 2026-09-18(docs/reviews/2026-09-18-gpu-mograph-survey.md): 3 社とも「法 × 形の重み」で、法の中身は Notch だけが text(HLSL)。
// ここでは重み(形)だけを関数にし、法はどの札でも書ける。使い方: `return ef_apply(law, ef_box(centre, box_lo, box_hi, soft) * strength);`
// 引用のルール: 札はここの関数を import して呼ぶだけ。ここを直せば引いた札全部に届く。
import package::motolii::{ Offset, now_lo, now_hi };
import package::cavalry::{ cv_range, cv_noise };
import package::processing::random;

/// 今の中心(前の段の結果込み)。段は同じコマで順に走るので、前の札が動かした後の位置で形を判定できる = 関係のチェーン。
fn now_centre(k: u32) -> vec2f { return (now_lo(k) + now_hi(k)) * 0.5; }

/// 重み w で Offset を「無し」へ寄せる(rotate は度)(w = 1 で法そのまま、0 で NO_OFFSET)。Notch の Strength / Unreal の Effector Weight。
fn ef_apply(d: Offset, w: f32) -> Offset {
    let t = clamp(w, 0.0, 1.0);
    return Offset(d.translate * t, d.rotate * t, mix(1.0, d.scale, t), mix(vec4f(1.0), d.tint, t));
}

/// Sphere(Radial): 中心から Radius で 1 → 0、縁は soft(0..1、Radius に対する割合)で滑らかに。
fn ef_sphere(p: vec2f, centre: vec2f, radius: f32, soft: f32) -> f32 {
    let r = max(radius, 1e-6);
    return 1.0 - smoothstep(1.0 - clamp(soft, 0.0, 1.0), 1.0, length(p - centre) / r);
}

/// Box: lo..hi の中で 1、外へ soft(px)で落ちる。
fn ef_box(p: vec2f, lo: vec2f, hi: vec2f, soft: f32) -> f32 {
    let s = max(soft, 1e-6);
    let q = max(lo - p, p - hi);
    let outside = length(max(q, vec2f(0.0)));
    return 1.0 - smoothstep(0.0, s, outside);
}

/// Plane: 法線 n の平面を距離 offset に置き、表側で 1、裏へ soft(px)で落ちる。Unreal の Plane Effector / Cavalry の Linear Falloff。
fn ef_plane(p: vec2f, normal: vec2f, offset: f32, soft: f32) -> f32 {
    let d = dot(p, normalize(normal)) - offset;
    return smoothstep(-max(soft, 1e-6), 0.0, d);
}

/// Step(Unreal の Step Effector / Cavalry の Range): 番号の割合 u = k/(n-1) が lo..hi の中で 1。
fn ef_step(k: u32, n: u32, lo: f32, hi: f32, soft: f32) -> f32 {
    return cv_range(k, n, lo, hi, soft);
}

/// Noise(Unreal の Noise Effector / Notch の Noise Falloff): 位置と時刻の noise を 0..1 に。
fn ef_noise(p: vec2f, frequency: f32, evolution: f32) -> f32 {
    return cv_noise(p, frequency, evolution) * 0.5 + 0.5;
}

/// Seed(Unity VFX の particle seed): 番号ごとの固定の乱数 0..1、法の「ばらつき」用。
fn ef_seed(k: u32, seed: u32) -> f32 {
    return random(k, seed);
}

/// Age(Unity の age / lifetime): born 秒に生まれ life 秒で 1 になる割合 0..1。時刻の純関数(記憶なし)。
fn ef_age(time: f32, born: f32, life: f32) -> f32 {
    return clamp((time - born) / max(life, 1e-6), 0.0, 1.0);
}
