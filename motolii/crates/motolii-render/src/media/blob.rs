//! Blob — 絵から塊を拾って追う芯(2026-09-13 裁定、宿題台帳 §3)。先例は TouchDesigner の Blob Track TOP:
//! 単色の元に閾値 → 輪郭、大きさで足切り、前のコマから一定距離以内なら同じ ID、見失っても一定時間なら復活。
//! 拾う元は 3 つ(明るさ / 動き / 色)、ID は持続する・しないの両方。機械学習は使わない。
//! 塊を配置として resolve へ渡す橋はまだ無い(書類の評価は画素を読めない。相談待ち)。

/// 何を塊と見るか。値は 0〜1(非乗算の sRGB を 255 で割った物)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlobSource {
    /// 明るさが閾値より上(`invert` なら下)。
    Luminance { threshold: f32, invert: bool },
    /// 前のコマとの差(RGB の最大差)が閾値より上。前のコマが無ければ何も拾わない。
    Motion { threshold: f32 },
    /// 指定の色との距離(RGB のユークリッド距離)が許容より近い。
    Color { target: [f32; 3], tolerance: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlobSettings {
    pub source: BlobSource,
    /// 画素数の下限・上限。
    pub min_area: u32,
    pub max_area: u32,
    /// 多すぎる時は大きい順にここまで。
    pub max_blobs: usize,
    /// ID を持続する(前のコマの塊を継ぐ)。false なら毎コマ位置の順に番号を振る。
    pub persist: bool,
    /// 同じ塊と見なす 1 コマの移動の上限(px)。
    pub max_move: f32,
    /// 見失ってから同じ ID で戻れるコマ数。
    pub revive_frames: u32,
}

/// 1 コマで拾った塊(ID を振る前)。箱は画素の端を含む `[min, max + 1)`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Region {
    pub min: [u32; 2],
    pub max: [u32; 2],
    pub area: u32,
    pub center: [f32; 2],
}

/// ID を振った塊。`age` は同じ ID で続いたコマ数(0 は生まれたコマ)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blob {
    pub id: u32,
    pub region: Region,
    pub age: u32,
}

/// 非乗算 RGBA8 の 1 コマから塊を拾う(8 近傍の連結成分、2 回走査 + union-find)。
pub fn detect(rgba: &[u8], width: u32, height: u32, previous: Option<&[u8]>, settings: &BlobSettings) -> Vec<Region> {
    let (w, h) = (width as usize, height as usize);
    if rgba.len() < w * h * 4 {
        return Vec::new();
    }
    let channel = |px: &[u8], i: usize| [px[i * 4] as f32 / 255.0, px[i * 4 + 1] as f32 / 255.0, px[i * 4 + 2] as f32 / 255.0];
    let inside = |i: usize| -> bool {
        let c = channel(rgba, i);
        match settings.source {
            BlobSource::Luminance { threshold, invert } => (0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2] > threshold) != invert,
            BlobSource::Motion { threshold } => previous.filter(|p| p.len() >= w * h * 4).is_some_and(|p| {
                let before = channel(p, i);
                (0..3).map(|k| (c[k] - before[k]).abs()).fold(0.0, f32::max) > threshold
            }),
            BlobSource::Color { target, tolerance } => (0..3).map(|k| (c[k] - target[k]).powi(2)).sum::<f32>().sqrt() < tolerance,
        }
    };
    let mut labels = vec![0u32; w * h];
    let mut parent: Vec<u32> = vec![0];
    fn find(parent: &mut [u32], mut x: u32) -> u32 {
        while parent[x as usize] != x {
            parent[x as usize] = parent[parent[x as usize] as usize];
            x = parent[x as usize];
        }
        x
    }
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if !inside(i) {
                continue;
            }
            let mut neighbours = [0u32; 4];
            if x > 0 { neighbours[0] = labels[i - 1]; }
            if y > 0 {
                neighbours[1] = labels[i - w];
                if x > 0 { neighbours[2] = labels[i - w - 1]; }
                if x + 1 < w { neighbours[3] = labels[i - w + 1]; }
            }
            let mut root = 0u32;
            for n in neighbours.into_iter().filter(|n| *n != 0) {
                let r = find(&mut parent, n);
                root = if root == 0 { r } else {
                    let (a, b) = (root.min(r), root.max(r));
                    parent[b as usize] = a;
                    a
                };
            }
            if root == 0 {
                root = parent.len() as u32;
                parent.push(root);
            }
            labels[i] = root;
        }
    }
    let mut regions: std::collections::BTreeMap<u32, (Region, [f64; 2])> = std::collections::BTreeMap::new();
    for y in 0..h {
        for x in 0..w {
            let label = labels[y * w + x];
            if label == 0 {
                continue;
            }
            let root = find(&mut parent, label);
            let entry = regions.entry(root).or_insert((Region { min: [x as u32, y as u32], max: [x as u32, y as u32], area: 0, center: [0.0; 2] }, [0.0; 2]));
            let r = &mut entry.0;
            r.min = [r.min[0].min(x as u32), r.min[1].min(y as u32)];
            r.max = [r.max[0].max(x as u32), r.max[1].max(y as u32)];
            r.area += 1;
            entry.1[0] += x as f64 + 0.5;
            entry.1[1] += y as f64 + 0.5;
        }
    }
    let mut out: Vec<Region> = regions.into_values()
        .map(|(mut r, sum)| { r.center = [(sum[0] / r.area as f64) as f32, (sum[1] / r.area as f64) as f32]; r })
        .filter(|r| r.area >= settings.min_area && r.area <= settings.max_area)
        .collect();
    out.sort_by(|a, b| b.area.cmp(&a.area).then(a.min[1].cmp(&b.min[1])).then(a.min[0].cmp(&b.min[0])));
    out.truncate(settings.max_blobs);
    out
}

/// ID を振る。持続するなら状態を持つので、描く側は入点から 1 コマずつ `step` する(feedback と同じ漸化式)。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlobTracker {
    next_id: u32,
    /// (塊, 見失ってからのコマ数)。
    known: Vec<(Blob, u32)>,
}

impl BlobTracker {
    pub fn step(&mut self, mut regions: Vec<Region>, settings: &BlobSettings) -> Vec<Blob> {
        if !settings.persist {
            // 毎コマ独立: 上から下、左から右の順に 0 から。
            regions.sort_by(|a, b| a.min[1].cmp(&b.min[1]).then(a.min[0].cmp(&b.min[0])));
            return regions.into_iter().enumerate().map(|(i, region)| Blob { id: i as u32, region, age: 0 }).collect();
        }
        // 近い組から順に継ぐ(貪欲)。同じ前の塊は 1 回だけ。
        let mut pairs: Vec<(f32, usize, usize)> = Vec::new();
        for (r, region) in regions.iter().enumerate() {
            for (k, (known, missing)) in self.known.iter().enumerate() {
                let d = ((region.center[0] - known.region.center[0]).powi(2) + (region.center[1] - known.region.center[1]).powi(2)).sqrt();
                if d <= settings.max_move * (*missing + 1) as f32 {
                    pairs.push((d, r, k));
                }
            }
        }
        pairs.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        let mut taken_region = vec![None::<usize>; regions.len()];
        let mut taken_known = vec![false; self.known.len()];
        for (_, r, k) in pairs {
            if taken_region[r].is_none() && !taken_known[k] {
                taken_region[r] = Some(k);
                taken_known[k] = true;
            }
        }
        let mut next = Vec::new();
        let mut out = Vec::new();
        for (r, region) in regions.into_iter().enumerate() {
            let blob = match taken_region[r] {
                Some(k) => Blob { id: self.known[k].0.id, region, age: self.known[k].0.age + 1 + self.known[k].1 },
                None => { let id = self.next_id; self.next_id += 1; Blob { id, region, age: 0 } }
            };
            out.push(blob);
            next.push((blob, 0));
        }
        for (k, (blob, missing)) in self.known.iter().enumerate() {
            if !taken_known[k] && *missing < settings.revive_frames {
                next.push((*blob, missing + 1));
            }
        }
        self.known = next;
        out.sort_by_key(|b| b.id);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: u32 = 64;
    const H: u32 = 48;

    fn settings(source: BlobSource, persist: bool) -> BlobSettings {
        BlobSettings { source, min_area: 4, max_area: u32::MAX, max_blobs: 64, persist, max_move: 12.0, revive_frames: 3 }
    }

    /// 黒地に白い四角 (x, y, 一辺) を描く。
    fn frame(squares: &[(u32, u32, u32)], color: [u8; 3]) -> Vec<u8> {
        let mut px = vec![0u8; (W * H * 4) as usize];
        for p in px.chunks_exact_mut(4) { p[3] = 255; }
        for &(x0, y0, side) in squares {
            for y in y0..(y0 + side).min(H) {
                for x in x0..(x0 + side).min(W) {
                    let i = ((y * W + x) * 4) as usize;
                    px[i..i + 3].copy_from_slice(&color);
                }
            }
        }
        px
    }

    #[test]
    fn bright_squares_become_regions_with_their_boxes() {
        let s = settings(BlobSource::Luminance { threshold: 0.5, invert: false }, false);
        let regions = detect(&frame(&[(4, 4, 10), (40, 20, 6), (30, 40, 1)], [255; 3]), W, H, None, &s);
        assert_eq!(regions.len(), 2, "1 画素の点は面積の下限で落ちる: {regions:?}");
        assert_eq!((regions[0].min, regions[0].max, regions[0].area), ([4, 4], [13, 13], 100));
        assert_eq!(regions[0].center, [9.0, 9.0]);
        assert_eq!((regions[1].min, regions[1].max, regions[1].area), ([40, 20], [45, 25], 36));
    }

    #[test]
    fn a_diagonal_touch_is_one_region() {
        let s = settings(BlobSource::Luminance { threshold: 0.5, invert: false }, false);
        let regions = detect(&frame(&[(10, 10, 4), (14, 14, 4)], [255; 3]), W, H, None, &s);
        assert_eq!(regions.len(), 1, "角で触れる 2 つは 8 近傍で 1 つ");
        assert_eq!(regions[0].area, 32);
    }

    #[test]
    fn motion_picks_only_what_changed_and_colour_picks_only_its_colour() {
        let motion = settings(BlobSource::Motion { threshold: 0.2 }, false);
        let before = frame(&[(4, 4, 8), (40, 4, 8)], [255; 3]);
        let after = frame(&[(4, 4, 8), (44, 4, 8)], [255; 3]);
        assert!(detect(&after, W, H, None, &motion).is_empty(), "前のコマが無ければ動きは無い");
        let moved = detect(&after, W, H, Some(&before), &motion);
        assert!(moved.iter().all(|r| r.min[0] >= 40), "止まった四角は拾わない: {moved:?}");
        assert!(!moved.is_empty());
        let red = settings(BlobSource::Color { target: [1.0, 0.0, 0.0], tolerance: 0.3 }, false);
        let mut picture = frame(&[(4, 4, 8)], [255, 0, 0]);
        let blue = frame(&[(40, 4, 8)], [0, 0, 255]);
        for (p, b) in picture.chunks_exact_mut(4).zip(blue.chunks_exact(4)) { if b[2] == 255 { p.copy_from_slice(b); } }
        let found = detect(&picture, W, H, None, &red);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].min, [4, 4]);
    }

    #[test]
    fn persistent_ids_follow_moving_blobs_and_revive_after_a_short_gap() {
        let s = settings(BlobSource::Luminance { threshold: 0.5, invert: false }, true);
        let mut tracker = BlobTracker::default();
        let step = |tracker: &mut BlobTracker, squares: &[(u32, u32, u32)]| tracker.step(detect(&frame(squares, [255; 3]), W, H, None, &s), &s);
        let first = step(&mut tracker, &[(4, 4, 6), (40, 30, 6)]);
        let ids = |blobs: &[Blob], x: u32, y: u32| blobs.iter().find(|b| b.region.min == [x, y]).map(|b| b.id);
        let (left, right) = (ids(&first, 4, 4).unwrap(), ids(&first, 40, 30).unwrap());
        assert_ne!(left, right);
        // 左は右へ 6 px、右は左へ 6 px。
        let second = step(&mut tracker, &[(10, 4, 6), (34, 30, 6)]);
        assert_eq!((ids(&second, 10, 4), ids(&second, 34, 30)), (Some(left), Some(right)), "動いても同じ ID");
        // 右が 2 コマ消えて戻る(revive 3 コマ以内)。
        step(&mut tracker, &[(16, 4, 6)]);
        step(&mut tracker, &[(22, 4, 6)]);
        let back = step(&mut tracker, &[(28, 4, 6), (34, 30, 6)]);
        assert_eq!((ids(&back, 28, 4), ids(&back, 34, 30)), (Some(left), Some(right)), "少しの間見失っても同じ ID で戻る");
        // 新しく出た塊は新しい ID。
        let fresh = step(&mut tracker, &[(34, 4, 6), (34, 30, 6), (4, 40, 6)]);
        assert!(ids(&fresh, 4, 40).is_some_and(|id| id != left && id != right), "新しい塊は新しい ID");
    }

    #[test]
    fn without_persistence_ids_are_the_reading_order_of_each_frame() {
        let s = settings(BlobSource::Luminance { threshold: 0.5, invert: false }, false);
        let mut tracker = BlobTracker::default();
        let blobs = tracker.step(detect(&frame(&[(40, 4, 6), (4, 30, 6), (4, 4, 6)], [255; 3]), W, H, None, &s), &s);
        let order: Vec<[u32; 2]> = blobs.iter().map(|b| b.region.min).collect();
        assert_eq!(order, vec![[4, 4], [40, 4], [4, 30]], "上から下、左から右");
        assert_eq!(blobs.iter().map(|b| b.id).collect::<Vec<_>>(), vec![0, 1, 2]);
    }
}
