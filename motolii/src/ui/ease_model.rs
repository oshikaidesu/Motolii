
use crate::doc::store::Interp;

/// 実機で確かめた既定値(2026-07-19 観察台帳)。
pub(super) const KINDS: &[Interp] = &[
    Interp::Linear,
    Interp::Bezier { x1: 0.42, y1: 0.0, x2: 0.58, y2: 1.0 },
    Interp::Bounce { first_dip: 0.27, dip: 0.2 },
    Interp::Elastic { limit: 1.5, period: 0.3, damp: 0.35 },
    Interp::Cyclic { period: 2.0 / 7.0, peak: 0.5, linear: 0.0, envelope_end: 0.0 },
    Interp::Random { seed: 500.0, grain: 0.15, center_u: 0.5, center_v: 0.75, bias: 0.5 },
    Interp::Steps { width: 0.178, smooth: 0.0 },
    Interp::ElasticSteps { width: 0.2, elasticity: 0.5 },
];

/// 掴める点。`at` は載る場所、`moved` はそこへ運んだ時の新しい形。
/// 位置は正規化(u は時間、v は値)で持つ — px は renderer の都合。
pub(super) struct Handle {
    pub at: (f64, f64),
    pub moved: fn(Interp, (f64, f64)) -> Interp,
}

fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    v.clamp(lo, hi)
}

/// Overshoot が OFF の間、手で動かす handle は値の範囲へ縛る(AM-KG-07)。
/// 型そのものが終点を越える物(Elastic 系)はこの縛りの外。
pub(super) fn hold_in_range(interp: Interp, free: bool) -> Interp {
    if free || overshoots(interp) {
        return interp;
    }
    match interp {
        Interp::Bezier { x1, y1, x2, y2 } => Interp::Bezier {
            x1,
            y1: clamp(y1, 0.0, 1.0),
            x2,
            y2: clamp(y2, 0.0, 1.0),
        },
        other => other,
    }
}

/// 型が終点を越えるか。Overshoot の既定はこれで決まる(AM-KG-07)。
pub(super) fn overshoots(interp: Interp) -> bool {
    matches!(interp, Interp::Elastic { .. } | Interp::ElasticSteps { .. })
}

pub(super) fn handles(interp: Interp) -> Vec<Handle> {
    match interp {
        Interp::Hold | Interp::Linear => Vec::new(),
        Interp::Bezier { x1, y1, x2, y2 } => vec![
            Handle {
                at: (x1, y1),
                moved: |i, p| match i {
                    Interp::Bezier { x2, y2, .. } => Interp::Bezier {
                        x1: clamp(p.0, 0.0, 1.0),
                        y1: p.1,
                        x2,
                        y2,
                    },
                    other => other,
                },
            },
            Handle {
                at: (x2, y2),
                moved: |i, p| match i {
                    Interp::Bezier { x1, y1, .. } => Interp::Bezier {
                        x1,
                        y1,
                        x2: clamp(p.0, 0.0, 1.0),
                        y2: p.1,
                    },
                    other => other,
                },
            },
        ],
        // 谷の頂点そのものが handle。
        Interp::Bounce { first_dip, dip } => vec![Handle {
            at: (first_dip, dip),
            moved: |_, p| Interp::Bounce {
                first_dip: clamp(p.0, 0.06, 0.7),
                dip: clamp(p.1, 0.0, 0.9),
            },
        }],
        // 天井線の右端と、曲線から垂れる波の先端。
        Interp::Elastic { limit, period, damp } => vec![
            Handle {
                at: (0.97, limit),
                moved: |i, p| match i {
                    Interp::Elastic { period, damp, .. } => Interp::Elastic {
                        limit: clamp(p.1, 1.02, 2.0),
                        period,
                        damp,
                    },
                    other => other,
                },
            },
            Handle {
                at: (period, damp),
                moved: |i, p| match i {
                    Interp::Elastic { limit, .. } => Interp::Elastic {
                        limit,
                        period: clamp(p.0, 0.12, 0.7),
                        damp: clamp(p.1, 0.0, 0.97),
                    },
                    other => other,
                },
            },
        ],
        Interp::Cyclic { period, peak, linear, envelope_end } => vec![
            Handle {
                at: (period, 0.0),
                moved: |i, p| match i {
                    Interp::Cyclic { peak, linear, envelope_end, .. } => Interp::Cyclic {
                        period: clamp(p.0, 0.08, 0.95),
                        peak,
                        linear,
                        envelope_end,
                    },
                    other => other,
                },
            },
            Handle {
                at: (peak * period, 1.0),
                moved: |i, p| match i {
                    Interp::Cyclic { period, linear, envelope_end, .. } => Interp::Cyclic {
                        period,
                        peak: clamp(p.0 / period.max(1e-6), 0.02, 0.98),
                        linear,
                        envelope_end,
                    },
                    other => other,
                },
            },
            Handle {
                at: (linear * period, 1.5),
                moved: |i, p| match i {
                    Interp::Cyclic { period, peak, envelope_end, .. } => Interp::Cyclic {
                        period,
                        peak,
                        linear: clamp(p.0 / period.max(1e-6), 0.0, 1.0),
                        envelope_end,
                    },
                    other => other,
                },
            },
            Handle {
                at: (1.0, envelope_end),
                moved: |i, p| match i {
                    Interp::Cyclic { period, peak, linear, .. } => Interp::Cyclic {
                        period,
                        peak,
                        linear,
                        envelope_end: clamp(p.1, 0.0, 0.95),
                    },
                    other => other,
                },
            },
        ],
        Interp::Random { seed, grain, center_u, center_v, bias } => vec![
            Handle {
                at: (0.005, 0.15 + (seed / 999.0) * 0.7),
                moved: |i, p| match i {
                    Interp::Random { grain, center_u, center_v, bias, .. } => Interp::Random {
                        seed: clamp((p.1 - 0.15) / 0.7 * 999.0, 1.0, 999.0).round(),
                        grain,
                        center_u,
                        center_v,
                        bias,
                    },
                    other => other,
                },
            },
            Handle {
                at: (grain / 0.5, 1.0),
                moved: |i, p| match i {
                    Interp::Random { seed, center_u, center_v, bias, .. } => Interp::Random {
                        seed,
                        grain: clamp(p.0 * 0.5, 0.0, 0.5),
                        center_u,
                        center_v,
                        bias,
                    },
                    other => other,
                },
            },
            Handle {
                at: (center_u, center_v),
                moved: |i, p| match i {
                    Interp::Random { seed, grain, bias, .. } => Interp::Random {
                        seed,
                        grain,
                        center_u: clamp(p.0, 0.05, 0.95),
                        center_v: clamp(p.1, -0.2, 1.2),
                        bias,
                    },
                    other => other,
                },
            },
            Handle {
                at: (bias, 0.0),
                moved: |i, p| match i {
                    Interp::Random { seed, grain, center_u, center_v, .. } => Interp::Random {
                        seed,
                        grain,
                        center_u,
                        center_v,
                        bias: clamp(p.0, 0.0, 1.0),
                    },
                    other => other,
                },
            },
        ],
        // 白 anchor が段幅、黄 satellite の横のずれが平滑幅。
        Interp::Steps { width, smooth } => vec![
            Handle {
                at: (width, 0.0),
                moved: |i, p| match i {
                    Interp::Steps { smooth, .. } => Interp::Steps {
                        width: clamp(p.0, 0.06, 0.5),
                        smooth,
                    },
                    other => other,
                },
            },
            Handle {
                at: (width + smooth, 1.0),
                moved: |i, p| match i {
                    Interp::Steps { width, .. } => Interp::Steps {
                        width,
                        smooth: clamp(p.0 - width, 0.0, 0.45),
                    },
                    other => other,
                },
            },
        ],
        Interp::ElasticSteps { width, elasticity } => vec![
            Handle {
                at: (width, 0.0),
                moved: |i, p| match i {
                    Interp::ElasticSteps { elasticity, .. } => Interp::ElasticSteps {
                        width: clamp(p.0, 0.09, 0.5),
                        elasticity,
                    },
                    other => other,
                },
            },
            Handle {
                at: (0.02, elasticity),
                moved: |i, p| match i {
                    Interp::ElasticSteps { width, .. } => Interp::ElasticSteps {
                        width,
                        elasticity: clamp(p.1, 0.0, 1.0),
                    },
                    other => other,
                },
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_at_least_one_handle_except_the_straight_ones() {
        for interp in KINDS.iter().copied() {
            let n = handles(interp).len();
            match interp {
                Interp::Linear => assert_eq!(n, 0),
                _ => assert!(n > 0, "{} に掴む所が無い", interp.kind()),
            }
        }
    }

    /// 掴んだ点へ運ぶと、そこが新しい載り場所になる(往復して同じ場所)。
    #[test]
    fn moving_a_handle_puts_it_where_it_was_dropped() {
        for interp in KINDS.iter().copied() {
            for (i, handle) in handles(interp).into_iter().enumerate() {
                let target = (0.3, 0.4);
                let moved = (handle.moved)(interp, target);
                let after = handles(moved);
                let at = after[i].at;
                // 可動域で切られる分はずれてよい。型が変わらないことと、
                // 動かした軸が近づいたことだけを見る。
                assert_eq!(moved.kind(), interp.kind(), "型が変わった");
                let before = handle.at;
                let closer = (at.0 - target.0).abs() <= (before.0 - target.0).abs() + 1e-9
                    || (at.1 - target.1).abs() <= (before.1 - target.1).abs() + 1e-9;
                assert!(closer, "{} の handle {i} が運んだ方へ寄っていない", interp.kind());
            }
        }
    }
}
