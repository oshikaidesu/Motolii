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

pub(super) fn sample(p1: f64, p2: f64, s: f64) -> f64 {
    let inv = 1.0 - s;
    3.0 * inv * inv * s * p1 + 3.0 * inv * s * s * p2 + s * s * s
}

fn sample_derivative(p1: f64, p2: f64, s: f64) -> f64 {
    let inv = 1.0 - s;
    3.0 * inv * inv * p1 + 6.0 * inv * s * (p2 - p1) + 3.0 * s * s * (1.0 - p2)
}

pub(super) fn solve_curve_x(x1: f64, x2: f64, x: f64) -> f64 {
    const EPS: f64 = 1e-7;

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
