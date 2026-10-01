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

/// CSS の easing の x を t に戻す。`Bx(t) = 3(1-t)²t·x1 + 3(1-t)t²·x2 + t³` を展開すると
/// `(3x1)t + (3x2-6x1)t² + (1+3x1-3x2)t³` の三次式で、**これは三次方程式の根**。
/// 自分で Newton も二分探索も書かない — `kurbo::common::solve_cubic`(Blinn の方法、
/// c3 が 0 なら二次へ落ちる)が依存木に既に居る。
///
/// 正しい easing なら Bx は [0,1] で単調なので根は 1 つだが、数値で複数返ることがある。
/// 残差の一番小さい物を取る。
pub(super) fn solve_curve_x(x1: f64, x2: f64, x: f64) -> f64 {
    let roots = kurbo::common::solve_cubic(-x, 3.0 * x1, 3.0 * x2 - 6.0 * x1, 1.0 + 3.0 * x1 - 3.0 * x2);
    let err = |t: f64| (sample(x1, x2, t) - x).abs();
    // 端は必ず候補に入れる。`Bx(0)=0` `Bx(1)=1` なので、端が重根になる形
    // (例 x1=0.42 x2=1 で t=1 が二重根)で solve_cubic が 1 本しか返さない場合を拾う。
    let best = roots
        .into_iter()
        .filter(|t| t.is_finite())
        .map(|t| t.clamp(0.0, 1.0))
        .chain([0.0, 1.0])
        .min_by(|a, b| err(*a).total_cmp(&err(*b)))
        .unwrap_or(x);
    if err(best) <= 1e-7 {
        return best;
    }
    // 閉じた式で届かない形が残る(`solve_cubic` は「まだ完全に robust ではない」と明記)。
    // そこだけ二分で詰める。普通の easing はここへ来ない。
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    let mut s = x.clamp(lo, hi);
    while hi - lo > 1e-7 {
        if sample(x1, x2, s) < x { lo = s } else { hi = s }
        s = (lo + hi) / 2.0;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 置き換える前の解き方(Newton 8 回 + 二分)。**これが答え合わせの相手**。
    fn reference(x1: f64, x2: f64, x: f64) -> f64 {
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

    /// 既製品に置き換えても、前の答えと一致する。CSS の 5 つの ease と、
    /// 三次の係数が 0 に落ちる形(x2 - x1 = 1/3)も含めて掃く。
    #[test]
    fn the_borrowed_solver_agrees_with_what_it_replaces() {
        let curves = [
            (0.25, 0.25), (0.42, 1.0), (0.0, 0.58), (0.42, 0.58),
            (0.0, 1.0 / 3.0),   // c3 = 0 — 三次が消えて二次になる
            (1.0, 0.0), (0.0, 0.0), (1.0, 1.0),
        ];
        for (x1, x2) in curves {
            for i in 0..=200 {
                let x = i as f64 / 200.0;
                let (a, b) = (solve_curve_x(x1, x2, x), reference(x1, x2, x));
                // t そのものより、**戻した x が合っているか**で見る(平らな区間では t が一意でない)。
                let (fa, fb) = (sample(x1, x2, a), sample(x1, x2, b));
                assert!(
                    (fa - fb).abs() < 1e-6,
                    "x1={x1} x2={x2} x={x}: 借りた方 {a} → {fa} / 前の方 {b} → {fb}"
                );
            }
        }
    }

    #[test]
    fn the_ends_stay_put() {
        for (x1, y1, x2, y2) in [(0.25, 0.1, 0.25, 1.0), (0.42, 0.0, 0.58, 1.0)] {
            assert_eq!(cubic_bezier_ease(x1, y1, x2, y2, 0.0), 0.0);
            assert_eq!(cubic_bezier_ease(x1, y1, x2, y2, 1.0), 1.0);
        }
    }
}
