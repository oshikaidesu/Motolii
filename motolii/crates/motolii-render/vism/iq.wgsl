// 関数の棚(module `package::iq`): Inigo Quilez の 1 行の形の関数(iquilezles.org/articles/functions)と Golan Levin のつまみ 1 個の ease。
// 名前は出典のまま(gain と parabola は lygia の math と同名)。出典: docs/reviews/2026-09-18-motion-code-survey.md

/// 急に立ち上がって k で減る。x = 1/k で山。
fn expImpulse(x: f32, k: f32) -> f32 { return k * x * exp(1.0 - k * x); }

/// f まで 2 次で立ち上がり、後は k で緩く落ちる(押して離す)。
fn sustainedImpulse(x: f32, f: f32, k: f32) -> f32 {
    let s = max(x - f, 0.0);
    return min(x * x / (f * f), 1.0 + (2.0 / f) * s * exp(-k * s));
}

/// c を中心に幅 w の丸い山。
fn cubicPulse(x: f32, c: f32, w: f32) -> f32 {
    var d = abs(x - c);
    if d > w { return 0.0; }
    d = d / w;
    return 1.0 - d * d * (3.0 - 2.0 * d);
}

/// 1 から k の速さで落ちる段(n で角の丸さ)。
fn expStep(x: f32, k: f32, n: f32) -> f32 { return exp(-k * pow(x, n)); }

/// a と b で山の位置と形を決める 0..1 の山。
fn pcurve(x: f32, a: f32, b: f32) -> f32 {
    let k = pow(a + b, a + b) / (pow(a, a) * pow(b, b));
    return k * pow(x, a) * pow(1.0 - x, b);
}

/// ほぼ恒等だが 0 の近くだけ丸い。
fn almostUnitIdentity(x: f32) -> f32 { return x * x * (2.0 - x); }

/// k > 1 で S 字を強く、< 1 で弱く(Schlick の gain)。
fn gain(x: f32, k: f32) -> f32 {
    let a = 0.5 * pow(2.0 * select(1.0 - x, x, x < 0.5), k);
    return select(1.0 - a, a, x < 0.5);
}

/// 0 と 1 で 0、0.5 で 1 の山。k で尖り。
fn parabola(x: f32, k: f32) -> f32 { return pow(4.0 * x * (1.0 - x), k); }

/// Golan Levin: a < 0.5 で ease out、> 0.5 で ease in。つまみ 1 個。
fn exponentialEasing(x: f32, a: f32) -> f32 {
    let e = 0.00001;
    let c = min(max(a, e), 1.0 - e);
    if c < 0.5 { return pow(x, 2.0 * c); }
    return pow(x, 1.0 / (1.0 - 2.0 * (c - 0.5)));
}

/// Golan Levin: a で強さが変わる S 字(0.5 で直線)。
fn doubleExponentialSigmoid(x: f32, a: f32) -> f32 {
    let e = 0.00001;
    let c = 1.0 - min(max(a, e), 1.0 - e);
    if x <= 0.5 { return pow(2.0 * x, 1.0 / c) * 0.5; }
    return 1.0 - pow(2.0 * (1.0 - x), 1.0 / c) * 0.5;
}
