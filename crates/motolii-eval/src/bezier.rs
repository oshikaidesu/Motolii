/// CSSの`cubic-bezier(x1, y1, x2, y2)`と同じ定義のイージング曲線。
/// 端点は(0,0)と(1,1)に固定。x∈[0,1]に対するyを返す。
///
/// x(s)は単調である必要があるため x1, x2 ∈ [0,1] を要求する(yは範囲外可 = オーバーシュート)。
pub fn cubic_bezier_ease(x1: f64, y1: f64, x2: f64, y2: f64, x: f64) -> f64 {
    debug_assert!(
        (0.0..=1.0).contains(&x1) && (0.0..=1.0).contains(&x2),
        "cubic_bezier_ease: x1/x2 must be in [0,1]"
    );
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let s = solve_curve_x(x1, x2, x);
    sample(y1, y2, s)
}

/// ベジェの1成分 B(s) を計算(端点0,1固定形)。
fn sample(p1: f64, p2: f64, s: f64) -> f64 {
    // B(s) = 3(1-s)^2 s p1 + 3(1-s) s^2 p2 + s^3
    let inv = 1.0 - s;
    3.0 * inv * inv * s * p1 + 3.0 * inv * s * s * p2 + s * s * s
}

fn sample_derivative(p1: f64, p2: f64, s: f64) -> f64 {
    let inv = 1.0 - s;
    3.0 * inv * inv * p1 + 6.0 * inv * s * (p2 - p1) + 3.0 * s * s * (1.0 - p2)
}

/// x(s) = x を満たすsをNewton法+二分法フォールバックで解く。
fn solve_curve_x(x1: f64, x2: f64, x: f64) -> f64 {
    const EPS: f64 = 1e-7;

    // Newton法(高速パス)
    let mut s = x;
    for _ in 0..8 {
        let err = sample(x1, x2, s) - x;
        if err.abs() < EPS {
            return s;
        }
        let d = sample_derivative(x1, x2, s);
        if d.abs() < 1e-6 {
            break;
        }
        s -= err / d;
    }

    // 二分法(確実パス)
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    s = x.clamp(lo, hi);
    while hi - lo > EPS {
        if sample(x1, x2, s) < x {
            lo = s;
        } else {
            hi = s;
        }
        s = (lo + hi) / 2.0;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_fixed() {
        assert_eq!(cubic_bezier_ease(0.42, 0.0, 0.58, 1.0, 0.0), 0.0);
        assert_eq!(cubic_bezier_ease(0.42, 0.0, 0.58, 1.0, 1.0), 1.0);
    }

    #[test]
    fn linear_curve_is_identity() {
        for i in 0..=10 {
            let x = i as f64 / 10.0;
            let y = cubic_bezier_ease(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0, x);
            assert!((y - x).abs() < 1e-6, "x={x} y={y}");
        }
    }

    #[test]
    fn ease_in_out_is_symmetric_and_monotone() {
        let f = |x: f64| cubic_bezier_ease(0.42, 0.0, 0.58, 1.0, x);
        assert!((f(0.5) - 0.5).abs() < 1e-6);
        let mut prev = 0.0;
        for i in 1..=100 {
            let y = f(i as f64 / 100.0);
            assert!(y >= prev, "not monotone at {i}");
            prev = y;
        }
        // ease-in-outは前半で入力より下、後半で上
        assert!(f(0.25) < 0.25);
        assert!(f(0.75) > 0.75);
    }

    #[test]
    fn overshoot_allowed_in_y() {
        // yが[0,1]を超えるカーブ(バウンス的)も解ける
        let y = cubic_bezier_ease(0.3, 1.5, 0.7, 1.5, 0.5);
        assert!(y > 1.0);
    }
}
