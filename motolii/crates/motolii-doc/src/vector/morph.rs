//! 輪郭同士の morph(学習なし)。文字ごとに輪郭を最小費用で対にし、弧長で打ち直した点を
//! 巡回して合わせ、直線で運ぶ。余った輪郭は自分の重心へ潰す(生える)。
//! 中間は折れ線、両端(t = 0・1)は元の輪郭そのもの。
//! 先例: Flubber / GSAP MorphSVG の点合わせ、fontTools `interpolatable` の輪郭対応(ハンガリー法)。

use crate::doc::vector::geom::bezier_point;
use crate::doc::vector::{Contour, Point};

/// 1 輪郭あたりの点数(中間の折れ線)。
const SAMPLES: usize = 96;
/// 対応付けの費用に使う粗い点数。
const COARSE: usize = 24;

/// 文字ごとに輪郭を対にして混ぜる。`*_glyphs` は輪郭ごとの文字の番(組んだ順)。
/// 返す番は文字の番(両側で同じ番同士が対になる)。
pub fn morph_glyphs(a: &[Contour], a_glyphs: &[usize], b: &[Contour], b_glyphs: &[usize], t: f64) -> Vec<(Contour, usize)> {
    let glyphs = a_glyphs.iter().chain(b_glyphs).copied().max().map_or(0, |n| n + 1);
    let mut out = Vec::new();
    for glyph in 0..glyphs {
        let pick = |cs: &[Contour], gs: &[usize]| cs.iter().zip(gs).filter(|(_, g)| **g == glyph).map(|(c, _)| c.clone()).collect::<Vec<_>>();
        for contour in morph_paths(&pick(a, a_glyphs), &pick(b, b_glyphs), t) {
            out.push((contour, glyph));
        }
    }
    out
}

/// 輪郭の集まり同士を混ぜる。
pub fn morph_paths(a: &[Contour], b: &[Contour], t: f64) -> Vec<Contour> {
    if t <= 0.0 { return a.to_vec(); }
    if t >= 1.0 { return b.to_vec(); }
    let pa: Vec<Vec<Point>> = a.iter().filter_map(|c| resample(&flatten(c), SAMPLES)).collect();
    let pb: Vec<Vec<Point>> = b.iter().filter_map(|c| resample(&flatten(c), SAMPLES)).collect();
    let ca: Vec<Vec<Point>> = pa.iter().map(|p| resample(p, COARSE).unwrap_or_default()).collect();
    let cb: Vec<Vec<Point>> = pb.iter().map(|p| resample(p, COARSE).unwrap_or_default()).collect();
    let cost: Vec<Vec<f64>> = ca.iter().map(|x| cb.iter().map(|y| align(x, y).1).collect()).collect();
    let pairs = assignment(&cost);
    let mut out = Vec::new();
    let mut used_b = vec![false; pb.len()];
    for (i, j) in &pairs {
        used_b[*j] = true;
        let (aligned, _) = align(&pa[*i], &pb[*j]);
        out.push(Contour::closed(pa[*i].iter().zip(&aligned).map(|(p, q)| lerp(*p, *q, t))));
    }
    for (i, p) in pa.iter().enumerate() {
        if pairs.iter().all(|(k, _)| *k != i) {
            let c = centroid(p);
            out.push(Contour::closed(p.iter().map(|q| lerp(*q, c, t))));
        }
    }
    for (j, p) in pb.iter().enumerate() {
        if !used_b[j] {
            let c = centroid(p);
            out.push(Contour::closed(p.iter().map(|q| lerp(c, *q, t))));
        }
    }
    out
}

fn lerp(a: Point, b: Point, t: f64) -> Point {
    Point { x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t }
}

fn centroid(p: &[Point]) -> Point {
    let n = p.len().max(1) as f64;
    Point { x: p.iter().map(|q| q.x).sum::<f64>() / n, y: p.iter().map(|q| q.y).sum::<f64>() / n }
}

/// 曲線を折れ線に(閉じた輪郭として扱う)。
fn flatten(contour: &Contour) -> Vec<Point> {
    let v = &contour.vertices;
    let mut out = Vec::new();
    for i in 0..v.len() {
        let next = (i + 1) % v.len();
        if next == 0 && !contour.closed { out.push(v[i].point); break; }
        for k in 0..8 {
            out.push(bezier_point(&v[i], &v[next], k as f64 / 8.0));
        }
    }
    out
}

/// 弧長で等間隔に n 点。長さの無い輪郭は None。
fn resample(poly: &[Point], n: usize) -> Option<Vec<Point>> {
    if poly.len() < 2 { return None; }
    let mut lengths = Vec::with_capacity(poly.len() + 1);
    lengths.push(0.0);
    for i in 0..poly.len() {
        let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
        lengths.push(lengths[i] + ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt());
    }
    let total = *lengths.last()?;
    if total <= 1e-9 { return None; }
    let mut out = Vec::with_capacity(n);
    let mut seg = 0;
    for k in 0..n {
        let s = total * k as f64 / n as f64;
        while seg + 1 < lengths.len() - 1 && lengths[seg + 1] < s { seg += 1; }
        let (p, q) = (poly[seg], poly[(seg + 1) % poly.len()]);
        let span = lengths[seg + 1] - lengths[seg];
        let u = if span <= 1e-12 { 0.0 } else { (s - lengths[seg]) / span };
        out.push(lerp(p, q, u));
    }
    Some(out)
}

fn signed_area(p: &[Point]) -> f64 {
    (0..p.len()).map(|i| { let q = p[(i + 1) % p.len()]; p[i].x * q.y - q.x * p[i].y }).sum::<f64>() * 0.5
}

/// `b` を `a` に最も近い向き・始点に回した写しと、その費用(点の距離の二乗和)。
fn align(a: &[Point], b: &[Point]) -> (Vec<Point>, f64) {
    let n = a.len().min(b.len());
    let mut b: Vec<Point> = b[..n].to_vec();
    if signed_area(a).signum() != signed_area(&b).signum() { b.reverse(); }
    let mut best = (0, f64::INFINITY);
    for shift in 0..n {
        let cost: f64 = (0..n).map(|i| { let q = b[(i + shift) % n]; (a[i].x - q.x).powi(2) + (a[i].y - q.y).powi(2) }).sum();
        if cost < best.1 { best = (shift, cost); }
    }
    b.rotate_left(best.0);
    (b, best.1)
}

/// 最小費用の 1 対 1 の対応(ハンガリー法、行 ≤ 列に転置して解く)。返すのは (行, 列)。
fn assignment(cost: &[Vec<f64>]) -> Vec<(usize, usize)> {
    let rows = cost.len();
    let cols = cost.first().map_or(0, Vec::len);
    if rows == 0 || cols == 0 { return Vec::new(); }
    if rows > cols {
        let transposed: Vec<Vec<f64>> = (0..cols).map(|j| (0..rows).map(|i| cost[i][j]).collect()).collect();
        return assignment(&transposed).into_iter().map(|(j, i)| (i, j)).collect();
    }
    // e-maxx の O(n²m) 版。1 始まりの添字で u/v がポテンシャル、p[j] が列 j に付いた行。
    let (n, m) = (rows, cols);
    let mut u = vec![0.0; n + 1];
    let mut v = vec![0.0; m + 1];
    let mut p = vec![0usize; m + 1];
    let mut way = vec![0usize; m + 1];
    for i in 1..=n {
        p[0] = i;
        let mut j0 = 0;
        let mut minv = vec![f64::INFINITY; m + 1];
        let mut used = vec![false; m + 1];
        loop {
            used[j0] = true;
            let i0 = p[j0];
            let mut delta = f64::INFINITY;
            let mut j1 = 0;
            for j in 1..=m {
                if used[j] { continue; }
                let cur = cost[i0 - 1][j - 1] - u[i0] - v[j];
                if cur < minv[j] { minv[j] = cur; way[j] = j0; }
                if minv[j] < delta { delta = minv[j]; j1 = j; }
            }
            for j in 0..=m {
                if used[j] { u[p[j]] += delta; v[j] -= delta; } else { minv[j] -= delta; }
            }
            j0 = j1;
            if p[j0] == 0 { break; }
        }
        loop {
            let j1 = way[j0];
            p[j0] = p[j1];
            j0 = j1;
            if j0 == 0 { break; }
        }
    }
    (1..=m).filter(|&j| p[j] != 0).map(|j| (p[j] - 1, j - 1)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(cx: f64, cy: f64, r: f64) -> Contour {
        Contour::closed([Point { x: cx - r, y: cy - r }, Point { x: cx + r, y: cy - r }, Point { x: cx + r, y: cy + r }, Point { x: cx - r, y: cy + r }])
    }

    fn bounds(c: &Contour) -> [f64; 4] {
        c.vertices.iter().fold([f64::MAX, f64::MAX, f64::MIN, f64::MIN], |b, v| [b[0].min(v.point.x), b[1].min(v.point.y), b[2].max(v.point.x), b[3].max(v.point.y)])
    }

    /// 両端は元の輪郭のまま、途中は点の数が揃った折れ線で、大きさは両者の間。
    #[test]
    fn ends_are_the_originals_and_the_middle_is_between() {
        let a = vec![square(0.0, 0.0, 10.0)];
        let b = vec![square(100.0, 0.0, 30.0)];
        assert_eq!(morph_paths(&a, &b, 0.0), a);
        assert_eq!(morph_paths(&a, &b, 1.0), b);
        let mid = morph_paths(&a, &b, 0.5);
        assert_eq!(mid.len(), 1);
        assert_eq!(mid[0].vertices.len(), SAMPLES);
        let bb = bounds(&mid[0]);
        assert!((bb[0] - 30.0).abs() < 1.0 && (bb[2] - 70.0).abs() < 1.0, "{bb:?}");
    }

    /// 向きが逆の相手は反転して合わせる: 反転前と同じ中間になる。
    #[test]
    fn a_reversed_target_aligns_to_the_same_middle() {
        let a = vec![square(0.0, 0.0, 10.0)];
        let b = vec![square(50.0, 0.0, 10.0)];
        let mut reversed = b.clone();
        reversed[0].vertices.reverse();
        let (x, y) = (morph_paths(&a, &b, 0.5), morph_paths(&a, &reversed, 0.5));
        let far = x[0].vertices.iter().zip(&y[0].vertices).map(|(p, q)| (p.point.x - q.point.x).abs() + (p.point.y - q.point.y).abs()).fold(0.0, f64::max);
        assert!(far < 1e-6, "{far}");
    }

    /// 輪郭の数が違えば、近い物同士が対になり、余りは重心へ潰れる(生える)。
    #[test]
    fn extra_contours_collapse_to_their_centroid() {
        let a = vec![square(0.0, 0.0, 10.0), square(200.0, 0.0, 10.0)];
        let b = vec![square(205.0, 0.0, 10.0)];
        let mid = morph_paths(&a, &b, 0.5);
        assert_eq!(mid.len(), 2);
        let sizes: Vec<f64> = mid.iter().map(|c| { let b = bounds(c); b[2] - b[0] }).collect();
        // 200 の角が 205 の角と対になり(幅 20 のまま)、0 の角が半分に潰れる。
        assert!(sizes.iter().any(|w| (w - 20.0).abs() < 1.0) && sizes.iter().any(|w| (w - 10.0).abs() < 1.0), "{sizes:?}");
        let grown = morph_paths(&b, &a, 0.5);
        assert_eq!(grown.len(), 2);
    }

    /// 文字の番で区切る: 1 文字目同士・2 文字目同士が対になり、片側にしか無い文字は潰れる。
    #[test]
    fn glyphs_pair_by_ordinal() {
        let a = vec![square(0.0, 0.0, 10.0), square(100.0, 0.0, 10.0)];
        let b = vec![square(0.0, 50.0, 10.0)];
        let mid = morph_glyphs(&a, &[0, 1], &b, &[0], 0.5);
        assert_eq!(mid.iter().map(|(_, g)| *g).collect::<Vec<_>>(), vec![0, 1]);
        let first = bounds(&mid[0].0);
        assert!((first[1] - 15.0).abs() < 1.0, "{first:?}");
    }

    /// 対応は最小費用: 近い相手を取り合わない。
    #[test]
    fn assignment_is_minimum_cost() {
        assert_eq!(assignment(&[vec![1.0, 10.0], vec![10.0, 1.0]]), vec![(0, 0), (1, 1)]);
        assert_eq!(assignment(&[vec![10.0, 1.0], vec![1.0, 10.0]]), vec![(1, 0), (0, 1)]);
        let mut wide = assignment(&[vec![5.0, 1.0, 9.0]]);
        wide.sort();
        assert_eq!(wide, vec![(0, 1)]);
        let mut tall = assignment(&[vec![5.0], vec![1.0], vec![9.0]]);
        tall.sort();
        assert_eq!(tall, vec![(1, 0)]);
    }
}
