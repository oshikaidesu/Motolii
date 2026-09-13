//! 字の輪郭を画に分ける。骨格(筆順付きの折れ線、em の箱を 0..255 に写した物)を字の箱へ合わせ、
//! 塗りの画素を最寄りの画に割り当て、画ごとの領域を輪郭へ戻す。
//! 先例: EasyFont(Lian 2018)の参照骨格への登録、StrokeStyles(Berio 2022)の画の分解。ここは
//! 学習も medial axis も持たず、骨格が与えられた時の最小の割り当てだけ。

use tiny_skia::{FillRule, Paint, Pixmap, Transform};

use crate::doc::vector::raster::{path_bounds, to_tiny_skia};
use crate::doc::vector::{Contour, Point};

/// 割り当てに使う画素の一辺。
const R: usize = 192;
/// 画の領域を継ぎ目ぶん太らせる幅(px)。
const SEAM: usize = 2;
/// 開いた輪(筆の あ)を閉じた外形 + 欠けとして見るための閉じ半径(px)。
const CLOSE: usize = 4;

/// 画ごとの輪郭。画が骨格に無ければ空。
pub fn segment(contours: &[Contour], template: &[Vec<[f32; 2]>]) -> Vec<Vec<Contour>> {
    let Some(bounds) = path_bounds(&contours.to_vec()) else { return Vec::new() };
    let side = (bounds[2] - bounds[0]).max(bounds[3] - bounds[1]).max(1e-6) * 1.12;
    let center = [(bounds[0] + bounds[2]) * 0.5, (bounds[1] + bounds[3]) * 0.5];
    let box0 = [center[0] - side * 0.5, center[1] - side * 0.5];
    let scale = R as f64 / side;
    let Some(mask) = fill(contours, box0, scale) else { return Vec::new() };

    // 骨格の箱を字の箱(塗りの箱)へ。
    let pts: Vec<[f32; 2]> = template.iter().flatten().copied().collect();
    let (kx0, ky0) = (pts.iter().map(|p| p[0]).fold(f32::MAX, f32::min), pts.iter().map(|p| p[1]).fold(f32::MAX, f32::min));
    let (kx1, ky1) = (pts.iter().map(|p| p[0]).fold(f32::MIN, f32::max), pts.iter().map(|p| p[1]).fold(f32::MIN, f32::max));
    let to_px = |p: [f32; 2]| -> [f64; 2] {
        let fx = if kx1 > kx0 { (p[0] - kx0) / (kx1 - kx0) } else { 0.5 } as f64;
        let fy = if ky1 > ky0 { (p[1] - ky0) / (ky1 - ky0) } else { 0.5 } as f64;
        let x = bounds[0] + fx * (bounds[2] - bounds[0]);
        let y = bounds[1] + fy * (bounds[3] - bounds[1]);
        [(x - box0[0]) * scale, (y - box0[1]) * scale]
    };
    let strokes: Vec<Vec<[f64; 2]>> = template.iter().map(|s| s.iter().map(|p| to_px(*p)).collect()).collect();

    // 画素ごとに最寄りの画。
    let mut label = vec![u16::MAX; R * R];
    for y in 0..R {
        for x in 0..R {
            if !mask[y * R + x] { continue; }
            let p = [x as f64 + 0.5, y as f64 + 0.5];
            let mut best = (f64::INFINITY, 0usize);
            for (k, s) in strokes.iter().enumerate() {
                let d = s.windows(2).map(|w| segment_distance(p, w[0], w[1])).fold(f64::INFINITY, f64::min);
                if d < best.0 { best = (d, k); }
            }
            label[y * R + x] = best.1 as u16;
        }
    }

    let to_units = |p: [f64; 2]| Point { x: box0[0] + p[0] / scale, y: box0[1] + p[1] / scale };
    (0..strokes.len()).map(|k| {
        let region: Vec<bool> = label.iter().map(|l| *l == k as u16).collect();
        let region = and(&dilate(&region, SEAM), &mask);
        let raw = trace(&region);
        // 閉じた外形 + 欠け: 開いた輪と閉じた輪が対になれるように、画の外形を閉じてから欠けを穴として持つ。
        let outer = dilate(&close(&region, CLOSE), 1);
        let gaps = open(&and(&outer, &not(&region)), 1);
        let closed: Vec<Vec<[f64; 2]>> = trace(&outer).into_iter().chain(trace(&gaps).into_iter().map(|mut g| { g.reverse(); g })).collect();
        let polys = if raw.len() <= 1 { raw } else { closed };
        polys.into_iter().map(|poly| Contour::closed(poly.into_iter().map(to_units))).collect()
    }).collect()
}

fn fill(contours: &[Contour], box0: [f64; 2], scale: f64) -> Option<Vec<bool>> {
    let path = to_tiny_skia(&contours.to_vec(), Point::ZERO)?;
    let mut pixmap = Pixmap::new(R as u32, R as u32)?;
    let mut paint = Paint::default();
    paint.set_color_rgba8(255, 255, 255, 255);
    paint.anti_alias = false;
    let transform = Transform::from_translate(-box0[0] as f32, -box0[1] as f32).post_scale(scale as f32, scale as f32);
    pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
    Some(pixmap.pixels().iter().map(|p| p.alpha() > 127).collect())
}

fn segment_distance(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len2 = dx * dx + dy * dy;
    let u = if len2 <= 1e-12 { 0.0 } else { (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / len2).clamp(0.0, 1.0) };
    let (qx, qy) = (a[0] + u * dx, a[1] + u * dy);
    ((p[0] - qx).powi(2) + (p[1] - qy).powi(2)).sqrt()
}

fn dilate(m: &[bool], r: usize) -> Vec<bool> {
    let mut out = m.to_vec();
    for _ in 0..r {
        let src = out.clone();
        for y in 0..R {
            for x in 0..R {
                if src[y * R + x] { continue; }
                let hit = (y > 0 && src[(y - 1) * R + x]) || (y + 1 < R && src[(y + 1) * R + x]) || (x > 0 && src[y * R + x - 1]) || (x + 1 < R && src[y * R + x + 1]);
                if hit { out[y * R + x] = true; }
            }
        }
    }
    out
}

fn erode(m: &[bool], r: usize) -> Vec<bool> {
    not(&dilate(&not(m), r))
}

fn close(m: &[bool], r: usize) -> Vec<bool> { erode(&dilate(m, r), r) }
fn open(m: &[bool], r: usize) -> Vec<bool> { dilate(&erode(m, r), r) }
fn not(m: &[bool]) -> Vec<bool> { m.iter().map(|b| !b).collect() }
fn and(a: &[bool], b: &[bool]) -> Vec<bool> { a.iter().zip(b).map(|(x, y)| *x && *y).collect() }

/// 二値画像の境界を閉じた折れ線に(marching squares、画素中心が格子点)。外側と穴で向きが逆になる。
fn trace(m: &[bool]) -> Vec<Vec<[f64; 2]>> {
    use std::collections::HashMap;
    let at = |x: isize, y: isize| -> bool { x >= 0 && y >= 0 && (x as usize) < R && (y as usize) < R && m[y as usize * R + x as usize] };
    // 各セル (x, y) の 4 角: a=(x,y) b=(x+1,y) c=(x+1,y+1) d=(x,y+1)。辺の中点を頂点にする。
    let mut segments: Vec<([i32; 2], [i32; 2])> = Vec::new(); // 2 倍座標で整数に
    let top = |x: isize, y: isize| [2 * x as i32 + 1, 2 * y as i32];
    let right = |x: isize, y: isize| [2 * x as i32 + 2, 2 * y as i32 + 1];
    let bottom = |x: isize, y: isize| [2 * x as i32 + 1, 2 * y as i32 + 2];
    let left = |x: isize, y: isize| [2 * x as i32, 2 * y as i32 + 1];
    for y in -1..R as isize {
        for x in -1..R as isize {
            let case = (at(x, y) as u8) | (at(x + 1, y) as u8) << 1 | (at(x + 1, y + 1) as u8) << 2 | (at(x, y + 1) as u8) << 3;
            // 塗りを左手に見る向き。
            let mut push = |from: [i32; 2], to: [i32; 2]| segments.push((from, to));
            match case {
                0 | 15 => {}
                1 => push(left(x, y), top(x, y)),
                2 => push(top(x, y), right(x, y)),
                3 => push(left(x, y), right(x, y)),
                4 => push(right(x, y), bottom(x, y)),
                5 => { push(left(x, y), top(x, y)); push(right(x, y), bottom(x, y)); }
                6 => push(top(x, y), bottom(x, y)),
                7 => push(left(x, y), bottom(x, y)),
                8 => push(bottom(x, y), left(x, y)),
                9 => push(bottom(x, y), top(x, y)),
                10 => { push(top(x, y), right(x, y)); push(bottom(x, y), left(x, y)); }
                11 => push(bottom(x, y), right(x, y)),
                12 => push(right(x, y), left(x, y)),
                13 => push(right(x, y), top(x, y)),
                14 => push(top(x, y), left(x, y)),
                _ => unreachable!(),
            }
        }
    }
    let mut next: HashMap<[i32; 2], Vec<usize>> = HashMap::new();
    for (i, (from, _)) in segments.iter().enumerate() { next.entry(*from).or_default().push(i); }
    let mut used = vec![false; segments.len()];
    let mut loops = Vec::new();
    for start in 0..segments.len() {
        if used[start] { continue; }
        let mut poly = Vec::new();
        let mut i = start;
        loop {
            used[i] = true;
            let (from, to) = segments[i];
            poly.push([from[0] as f64 * 0.5 + 0.5, from[1] as f64 * 0.5 + 0.5]);
            let Some(candidates) = next.get(&to) else { break };
            let Some(&j) = candidates.iter().find(|j| !used[**j]) else { break };
            i = j;
        }
        if poly.len() >= 8 { loops.push(poly); }
    }
    loops.retain(|p| area(p).abs() >= 0.0004 * (R * R) as f64);
    loops
}

fn area(p: &[[f64; 2]]) -> f64 {
    (0..p.len()).map(|i| { let q = p[(i + 1) % p.len()]; p[i][0] * q[1] - q[0] * p[i][1] }).sum::<f64>() * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Contour {
        Contour::closed([Point { x: x0, y: y0 }, Point { x: x1, y: y0 }, Point { x: x1, y: y1 }, Point { x: x0, y: y1 }])
    }

    /// 十字(横棒 + 縦棒が 1 つの輪郭)を、横・縦 2 本の骨格で 2 つの画に分ける。
    #[test]
    fn a_cross_splits_into_its_two_strokes() {
        // 十字を 1 つの輪郭で描く(重なり無し)。
        let cross = Contour::closed([
            (40.0, 0.0), (60.0, 0.0), (60.0, 40.0), (100.0, 40.0), (100.0, 60.0), (60.0, 60.0),
            (60.0, 100.0), (40.0, 100.0), (40.0, 60.0), (0.0, 60.0), (0.0, 40.0), (40.0, 40.0),
        ].map(|(x, y)| Point { x, y }));
        let template = vec![vec![[0.0, 128.0], [255.0, 128.0]], vec![[128.0, 0.0], [128.0, 255.0]]];
        let strokes = segment(&[cross], &template);
        assert_eq!(strokes.len(), 2);
        let bounds = |cs: &[Contour]| path_bounds(&cs.to_vec()).unwrap();
        let (h, v) = (bounds(&strokes[0]), bounds(&strokes[1]));
        assert!(h[2] - h[0] > 90.0 && h[3] - h[1] < 40.0, "horizontal stroke {h:?}");
        assert!(v[3] - v[1] > 90.0 && v[2] - v[0] < 40.0, "vertical stroke {v:?}");
    }

    /// 穴のある画(輪)は穴が残る: 外側と穴で向きが逆。
    #[test]
    fn a_ring_keeps_its_hole() {
        let ring = vec![rect(0.0, 0.0, 100.0, 100.0), {
            let mut inner = rect(30.0, 30.0, 70.0, 70.0);
            inner.vertices.reverse();
            inner
        }];
        let template = vec![vec![[0.0, 0.0], [255.0, 0.0], [255.0, 255.0], [0.0, 255.0], [0.0, 0.0]]];
        let strokes = segment(&ring, &template);
        assert_eq!(strokes.len(), 1);
        let signs: Vec<bool> = strokes[0].iter().map(|c| {
            let p: Vec<[f64; 2]> = c.vertices.iter().map(|v| [v.point.x, v.point.y]).collect();
            area(&p) > 0.0
        }).collect();
        assert_eq!(strokes[0].len(), 2, "{signs:?}");
        assert_ne!(signs[0], signs[1]);
    }
}
