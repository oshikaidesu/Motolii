// 関数の棚(WESL の module `package::easing`): Penner の ease 全種。名前は lygia / glsl-easings(MIT)と同じ、式は Penner の原式から。
// t は 0..1。cv_ease(cavalry.wgsl)は Cavalry の Interpolation の 6 種、こちらは全種を名前で呼ぶ。出典: docs/reviews/2026-09-18-motion-code-survey.md

const PI_E: f32 = 3.1415926;
const C1: f32 = 1.70158;
const C2: f32 = 2.5949095; // C1 * 1.525
const C3: f32 = 2.70158;   // C1 + 1
const C4: f32 = 2.0943951; // 2π / 3
const C5: f32 = 1.3962634; // 2π / 4.5

fn linear(t: f32) -> f32 { return t; }

fn sineIn(t: f32) -> f32 { return 1.0 - cos(t * PI_E * 0.5); }
fn sineOut(t: f32) -> f32 { return sin(t * PI_E * 0.5); }
fn sineInOut(t: f32) -> f32 { return -0.5 * (cos(PI_E * t) - 1.0); }

fn quadraticIn(t: f32) -> f32 { return t * t; }
fn quadraticOut(t: f32) -> f32 { return 1.0 - (1.0 - t) * (1.0 - t); }
fn quadraticInOut(t: f32) -> f32 { return select(1.0 - pow(-2.0 * t + 2.0, 2.0) * 0.5, 2.0 * t * t, t < 0.5); }

fn cubicIn(t: f32) -> f32 { return t * t * t; }
fn cubicOut(t: f32) -> f32 { return 1.0 - pow(1.0 - t, 3.0); }
fn cubicInOut(t: f32) -> f32 { return select(1.0 - pow(-2.0 * t + 2.0, 3.0) * 0.5, 4.0 * t * t * t, t < 0.5); }

fn quarticIn(t: f32) -> f32 { return t * t * t * t; }
fn quarticOut(t: f32) -> f32 { return 1.0 - pow(1.0 - t, 4.0); }
fn quarticInOut(t: f32) -> f32 { return select(1.0 - pow(-2.0 * t + 2.0, 4.0) * 0.5, 8.0 * t * t * t * t, t < 0.5); }

fn quinticIn(t: f32) -> f32 { return pow(t, 5.0); }
fn quinticOut(t: f32) -> f32 { return 1.0 - pow(1.0 - t, 5.0); }
fn quinticInOut(t: f32) -> f32 { return select(1.0 - pow(-2.0 * t + 2.0, 5.0) * 0.5, 16.0 * pow(t, 5.0), t < 0.5); }

fn exponentialIn(t: f32) -> f32 { return select(pow(2.0, 10.0 * t - 10.0), 0.0, t <= 0.0); }
fn exponentialOut(t: f32) -> f32 { return select(1.0 - pow(2.0, -10.0 * t), 1.0, t >= 1.0); }
fn exponentialInOut(t: f32) -> f32 {
    if t <= 0.0 { return 0.0; }
    if t >= 1.0 { return 1.0; }
    return select((2.0 - pow(2.0, -20.0 * t + 10.0)) * 0.5, pow(2.0, 20.0 * t - 10.0) * 0.5, t < 0.5);
}

fn circularIn(t: f32) -> f32 { return 1.0 - sqrt(1.0 - t * t); }
fn circularOut(t: f32) -> f32 { return sqrt(1.0 - (t - 1.0) * (t - 1.0)); }
fn circularInOut(t: f32) -> f32 {
    return select((sqrt(1.0 - pow(-2.0 * t + 2.0, 2.0)) + 1.0) * 0.5, (1.0 - sqrt(1.0 - 4.0 * t * t)) * 0.5, t < 0.5);
}

fn backIn(t: f32) -> f32 { return C3 * t * t * t - C1 * t * t; }
fn backOut(t: f32) -> f32 { let u = t - 1.0; return 1.0 + C3 * u * u * u + C1 * u * u; }
fn backInOut(t: f32) -> f32 {
    let a = 2.0 * t;
    let b = 2.0 * t - 2.0;
    return select((b * b * ((C2 + 1.0) * b + C2) + 2.0) * 0.5, (a * a * ((C2 + 1.0) * a - C2)) * 0.5, t < 0.5);
}

fn elasticIn(t: f32) -> f32 {
    if t <= 0.0 { return 0.0; }
    if t >= 1.0 { return 1.0; }
    return -pow(2.0, 10.0 * t - 10.0) * sin((t * 10.0 - 10.75) * C4);
}
fn elasticOut(t: f32) -> f32 {
    if t <= 0.0 { return 0.0; }
    if t >= 1.0 { return 1.0; }
    return pow(2.0, -10.0 * t) * sin((t * 10.0 - 0.75) * C4) + 1.0;
}
fn elasticInOut(t: f32) -> f32 {
    if t <= 0.0 { return 0.0; }
    if t >= 1.0 { return 1.0; }
    let s = sin((20.0 * t - 11.125) * C5);
    return select((pow(2.0, -20.0 * t + 10.0) * s) * 0.5 + 1.0, -(pow(2.0, 20.0 * t - 10.0) * s) * 0.5, t < 0.5);
}

fn bounceOut(t: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;
    if t < 1.0 / d1 { return n1 * t * t; }
    if t < 2.0 / d1 { let u = t - 1.5 / d1; return n1 * u * u + 0.75; }
    if t < 2.5 / d1 { let u = t - 2.25 / d1; return n1 * u * u + 0.9375; }
    let u = t - 2.625 / d1;
    return n1 * u * u + 0.984375;
}
fn bounceIn(t: f32) -> f32 { return 1.0 - bounceOut(1.0 - t); }
fn bounceInOut(t: f32) -> f32 {
    return select((1.0 + bounceOut(2.0 * t - 1.0)) * 0.5, (1.0 - bounceOut(1.0 - 2.0 * t)) * 0.5, t < 0.5);
}
