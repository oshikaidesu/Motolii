//! GSAP の `stagger` の配り方(MIT、`src/gsap-core.js` 452 `distribute`)を式のまま写す。
//! `{each, amount, from, grid, axis, ease}` — 升目の上の起点からの距離を正規化し、ease に通し、幅(amount か each × 列)を掛ける。
//! `from: "random"` は GSAP が `Math.random` で並べ替える所を、台本の `random(seed)` と同じ生成器で並べ替える(時刻の純関数)。
use motolii_doc::eval::Interp;

/// GSAP の `_bigNum`。升目が無ければ 1 行(列数 = 無限)。
pub const BIG: f64 = 1e8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum From {
    Start,
    Center,
    End,
    Edges,
    Random,
    /// 番号(0 < k < 1 なら GSAP の通り比率として読む)。
    Index(f64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Axis {
    None,
    X,
    Y,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stagger {
    /// 全体の幅(秒)。0 なら `each` から。
    pub amount: f64,
    /// 1 つごとの幅(秒)。
    pub each: f64,
    pub from: From,
    /// 升目の列数(GSAP `grid: [rows, cols]` の `cols` — rows は GSAP でも使わない)。1 行なら [`BIG`]。
    pub wrap_at: f64,
    pub axis: Axis,
    /// 遅れの並びに掛ける ease(GSAP `stagger.ease`、名前は eval/gsap.rs の表で既存の型に)。
    pub ease: Option<Interp>,
    /// `from: Random` の並べ替えの種。
    pub seed: f64,
}

/// 台本の `random(seed)` と同じ生成器(prelude.js)。同じ種は同じ列を返す。
pub fn seeded(seed: f64) -> impl FnMut() -> f64 {
    let mut s: u32 = ((seed as i64 as i32 as u32).wrapping_mul(2654435761)) ^ 0x9e3779b9;
    move || {
        s = s.wrapping_add(0x6d2b79f5);
        let mut t = s;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        (t ^ (t >> 14)) as f64 / 4294967296.0
    }
}

/// GSAP `_roundPrecise`: 1e−7 で丸める(浮動小数の誤差を消す)。
fn round_precise(v: f64) -> f64 {
    (v * 10_000_000.0).round() / 10_000_000.0
}

/// `distribute(vars)` を長さ `l` の並びに掛けた、番号ごとの遅れ(秒)。
pub fn distribute(v: &Stagger, l: usize) -> Vec<f64> {
    let lf = l as f64;
    let wrap = if v.wrap_at >= 1.0 { v.wrap_at } else { BIG };
    // ratios = isNaN(from) || (0 < from < 1); ratioX = ratioY = {center:.5, edges:.5, end:1}[from] || 0
    let (ratios, ratio) = match v.from {
        From::Start | From::Random => (true, 0.0),
        From::Center | From::Edges => (true, 0.5),
        From::End => (true, 1.0),
        From::Index(k) if k > 0.0 && k < 1.0 => (true, k),
        From::Index(k) => (false, k),
    };
    let origin_x = if ratios { wrap.min(lf) * ratio - 0.5 } else { ratio % wrap };
    let origin_y = if wrap == BIG { 0.0 } else if ratios { lf * ratio / wrap - 0.5 } else { (ratio / wrap).trunc() };
    let (mut max, mut min) = (0.0f64, BIG);
    let mut distances: Vec<f64> = (0..l)
        .map(|j| {
            let jf = j as f64;
            let x = (jf % wrap) - origin_x;
            let y = origin_y - (jf / wrap).trunc();
            let d = match v.axis {
                Axis::None => (x * x + y * y).sqrt(),
                Axis::X => x.abs(),
                Axis::Y => y.abs(),
            };
            max = max.max(d);
            min = min.min(d);
            d
        })
        .collect();
    if v.from == From::Random {
        let mut next = seeded(v.seed);
        for i in (1..l).rev() {
            let j = (next() * (i + 1) as f64).floor() as usize;
            distances.swap(i, j.min(i));
        }
    }
    let span = max - min;
    // each × (wrapAt > l ? l − 1 : !axis ? max(wrapAt, l / wrapAt) : axis === "y" ? l / wrapAt : wrapAt)
    let factor = if wrap > lf {
        lf - 1.0
    } else {
        match v.axis {
            Axis::None => wrap.max(lf / wrap),
            Axis::Y => lf / wrap,
            Axis::X => wrap,
        }
    };
    let mut width = if v.amount != 0.0 { v.amount } else { v.each * factor };
    if v.from == From::Edges {
        width = -width;
    }
    let base = if width < 0.0 { -width } else { 0.0 };
    let inverted = width < 0.0;
    distances
        .iter()
        .map(|d| {
            let along = if span != 0.0 { (d - min) / span } else { 0.0 };
            let eased = match v.ease {
                Some(e) if inverted => 1.0 - e.ease(1.0 - along),
                Some(e) => e.ease(along),
                None => along,
            };
            round_precise(base + eased * width)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_doc::store::*;

    fn grid(rows_cols: [f64; 2], from: From, amount: f64, each: f64) -> Stagger {
        Stagger { amount, each, from, wrap_at: rows_cols[1], axis: Axis::None, ease: None, seed: 0.0 }
    }

    /// 参照値は本物の GSAP(node、`gsap.utils.distribute(vars)(i, a[i], a)`)。
    #[test]
    fn distribute_matches_gsap() {
        let ease = |name: &str| Some(motolii_doc::eval::gsap::interp_for(name).unwrap());
        let cases: Vec<(&str, Stagger, usize, Vec<f64>)> = vec![
            ("3x3 center amount 1", grid([3.0, 3.0], From::Center, 1.0, 0.0), 9, vec![1.0, 0.7071068, 1.0, 0.7071068, 0.0, 0.7071068, 1.0, 0.7071068, 1.0]),
            ("3x3 edges amount 1", grid([3.0, 3.0], From::Edges, 1.0, 0.0), 9, vec![0.0, 0.2928932, 0.0, 0.2928932, 1.0, 0.2928932, 0.0, 0.2928932, 0.0]),
            ("3x3 start amount 1", grid([3.0, 3.0], From::Start, 1.0, 0.0), 9, vec![0.0, 0.309017, 0.6513878, 0.309017, 0.5, 0.7807764, 0.6513878, 0.7807764, 1.0]),
            ("3x3 end amount 1", grid([3.0, 3.0], From::End, 1.0, 0.0), 9, vec![1.0, 0.7807764, 0.6513878, 0.7807764, 0.5, 0.309017, 0.6513878, 0.309017, 0.0]),
            ("3x3 index 4 amount 1", grid([3.0, 3.0], From::Index(4.0), 1.0, 0.0), 9, vec![1.0, 0.7071068, 1.0, 0.7071068, 0.0, 0.7071068, 1.0, 0.7071068, 1.0]),
            ("3x3 index 0 each 0.1", grid([3.0, 3.0], From::Index(0.0), 0.0, 0.1), 9, vec![0.0, 0.106066, 0.212132, 0.106066, 0.15, 0.2371708, 0.212132, 0.2371708, 0.3]),
            ("3x3 center each 0.1 axis x", Stagger { axis: Axis::X, ..grid([3.0, 3.0], From::Center, 0.0, 0.1) }, 9, vec![0.3, 0.0, 0.3, 0.3, 0.0, 0.3, 0.3, 0.0, 0.3]),
            ("3x3 center each 0.1 axis y", Stagger { axis: Axis::Y, ..grid([3.0, 3.0], From::Center, 0.0, 0.1) }, 9, vec![0.3, 0.3, 0.3, 0.0, 0.0, 0.0, 0.3, 0.3, 0.3]),
            ("3x3 center amount 1 power2.in", Stagger { ease: ease("power2.in"), ..grid([3.0, 3.0], From::Center, 1.0, 0.0) }, 9, vec![1.0, 0.3535534, 1.0, 0.3535534, 0.0, 0.3535534, 1.0, 0.3535534, 1.0]),
            ("3x3 edges amount 1 power2.in", Stagger { ease: ease("power2.in"), ..grid([3.0, 3.0], From::Edges, 1.0, 0.0) }, 9, vec![0.0, 0.0251263, 0.0, 0.0251263, 1.0, 0.0251263, 0.0, 0.0251263, 0.0]),
            ("1d 5 each 0.1", grid([1.0, BIG], From::Start, 0.0, 0.1), 5, vec![0.0, 0.1, 0.2, 0.3, 0.4]),
            ("1d 5 center amount 1", grid([1.0, BIG], From::Center, 1.0, 0.0), 5, vec![1.0, 0.5, 0.0, 0.5, 1.0]),
            ("1d 5 end each 0.1", grid([1.0, BIG], From::End, 0.0, 0.1), 5, vec![0.4, 0.3, 0.2, 0.1, 0.0]),
            ("2x4 center amount 1", grid([2.0, 4.0], From::Center, 1.0, 0.0), 8, vec![1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0]),
            ("2x4 center each 0.1", grid([2.0, 4.0], From::Center, 0.0, 0.1), 8, vec![0.4, 0.0, 0.0, 0.4, 0.4, 0.0, 0.0, 0.4]),
        ];
        for (name, vars, l, expected) in cases {
            let got = distribute(&vars, l);
            for (i, (g, e)) in got.iter().zip(&expected).enumerate() {
                let slack = if vars.ease.is_some() { 1e-5 } else { 1e-7 };
                assert!((g - e).abs() < slack, "{name} [{i}]: got {g}, GSAP {e}");
            }
        }
    }

    /// random は距離の並べ替え(同じ集合、同じ種なら同じ列、違う種なら違う列)。生成器は prelude.js の `random(seed)` と同じ数を返す。
    #[test]
    fn random_is_a_seeded_shuffle_of_the_distances() {
        for (seed, expected) in [(0.0, [0.358889980242, 0.105903261341, 0.675290479325]), (3.0, [0.764451351715, 0.683441911591, 0.378283232916]), (42.0, [0.882522657979, 0.483629156370, 0.104837985476])] {
            let mut next = seeded(seed);
            for e in expected {
                assert!((next() - e).abs() < 1e-12, "random({seed}) = prelude.js");
            }
        }
        let sorted = |v: Vec<f64>| { let mut v = v; v.sort_by(f64::total_cmp); v };
        let plain = distribute(&grid([3.0, 3.0], From::Start, 1.0, 0.0), 9);
        let a = distribute(&Stagger { seed: 7.0, ..grid([3.0, 3.0], From::Random, 1.0, 0.0) }, 9);
        let b = distribute(&Stagger { seed: 7.0, ..grid([3.0, 3.0], From::Random, 1.0, 0.0) }, 9);
        let c = distribute(&Stagger { seed: 8.0, ..grid([3.0, 3.0], From::Random, 1.0, 0.0) }, 9);
        assert_eq!(a, b, "same seed, same order");
        assert_ne!(a, c, "another seed, another order");
        assert_ne!(a, plain, "shuffled");
        assert_eq!(sorted(a), sorted(plain), "the same distances, from the start corner (ratio 0), in another order");
    }
}
