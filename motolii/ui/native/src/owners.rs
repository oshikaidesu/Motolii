//! 再生の 1 コマを**持ち主ごとに**畳む。誰が何 ms 食ったかを、合計 1 つに潰さずに出す。
//!
//! 名前と並びは `FrameMeasurement::OWNERS` ただ 1 箇所。ここは数えて並べるだけで、
//! 段を増やしたければ向こうへ足す(こちらは何も変わらない)。
//! 面ごとに分ける — Stage と Camera は同じ時刻の同じ書類でも別の仕事をするため。

use crate::render::compositor::FrameMeasurement;

const OWNERS: usize = FrameMeasurement::OWNERS.len();

#[derive(Default)]
pub(crate) struct FrameOwners {
    /// 面の名前 → 持ち主ごとの標本(µs)。
    views: Vec<(String, [Vec<u64>; OWNERS])>,
    /// 段の中に含まれる内訳(submit / wait / readback)。
    included: Vec<(String, [Vec<u64>; 3])>,
    /// ▶ の時点の落とした数。番人(`Frames`)が通しで数えるので、差だけを見る。
    dropped_base: u64,
    /// resolve の中を、回したデータで割った行。**名前は固定しない** —
    /// 層の種類や相が増えれば、ここへ勝手に行が増える。面ごと・行名ごとの標本。
    inside: Vec<(String, Vec<(String, Vec<u64>)>)>,
    /// 面ごとの「重かった層」(µs, 層番号, 種類)。
    worst: Vec<(String, Vec<(u64, u64, String)>)>,
}

impl FrameOwners {
    pub(crate) fn clear(&mut self, dropped_base: u64) {
        self.views.clear();
        self.included.clear();
        self.inside.clear();
        self.worst.clear();
        self.dropped_base = dropped_base;
    }

    pub(crate) fn dropped_since_play(&self, now: u64) -> u64 {
        now.saturating_sub(self.dropped_base)
    }

    /// resolve の中身。`rows` は (群, 名前, 合計µs, 本数)。
    pub(crate) fn push_inside(&mut self, view: &str, rows: &[(&'static str, String, u64, u32)], worst: &[(u64, u64, &'static str)]) {
        if rows.is_empty() { return }
        let slot = match self.inside.iter().position(|(name, _)| name == view) {
            Some(i) => i,
            None => { self.inside.push((view.to_owned(), Vec::new())); self.inside.len() - 1 }
        };
        let frames = self.views.iter().find(|(n, _)| n == view).map_or(0, |(_, o)| o[0].len());
        for (group, name, us, count) in rows {
            let key = if *count > 1 { format!("{group}/{name} ×{count}") } else { format!("{group}/{name}") };
            let bucket = match self.inside[slot].1.iter().position(|(n, _)| *n == key) {
                Some(i) => i,
                None => {
                    // 途中から現れた行は、それまでの拍を 0 で埋める(列がずれないように)。
                    self.inside[slot].1.push((key, vec![0; frames.saturating_sub(1)]));
                    self.inside[slot].1.len() - 1
                }
            };
            self.inside[slot].1[bucket].1.push(*us);
        }
        let slot = match self.worst.iter().position(|(name, _)| name == view) {
            Some(i) => i,
            None => { self.worst.push((view.to_owned(), Vec::new())); self.worst.len() - 1 }
        };
        for (us, layer, kind) in worst {
            self.worst[slot].1.push((*us, *layer, (*kind).to_owned()));
        }
    }

    pub(crate) fn push(&mut self, view: &str, m: FrameMeasurement) {
        let slot = match self.views.iter().position(|(name, _)| name == view) {
            Some(i) => i,
            None => {
                self.views.push((view.to_owned(), Default::default()));
                self.included.push((view.to_owned(), Default::default()));
                self.views.len() - 1
            }
        };
        for (samples, us) in self.views[slot].1.iter_mut().zip(m.owners()) {
            samples.push(us);
        }
        for (samples, us) in self.included[slot].1.iter_mut().zip(m.included()) {
            samples.push(us);
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.views.iter().all(|(_, o)| o[0].is_empty())
    }

    /// 1 拍の予算 `budget_us` に対して、持ち主ごとの median / p90 / max と、
    /// **p90 の取り分**(誰が予算を食っているか)を並べる。
    pub(crate) fn report(&self, head: &str, budget_us: u64, dropped: u64) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(out, "PROBE room=playback-owners {head} budget={:.3}ms dropped={}", budget_us as f64 / 1000.0, dropped);
        for ((view, owners), (_, included)) in self.views.iter().zip(&self.included) {
            let frames = owners[0].len();
            let whole: Vec<u64> = (0..frames).map(|i| owners.iter().map(|s| s[i]).sum()).collect();
            let p90_whole = percentile(&sorted(&whole), 0.9);
            let _ = writeln!(out, "\n{view}  frames={frames}  1 frame p90={:.3}ms  over budget={}",
                p90_whole as f64 / 1000.0,
                whole.iter().filter(|&&us| us > budget_us).count());
            let _ = writeln!(out, "{:<14}{:>9}{:>9}{:>9}{:>8}", "owner (ms)", "median", "p90", "max", "p90%");
            for (name, samples) in FrameMeasurement::OWNERS.iter().zip(owners) {
                let s = sorted(samples);
                let share = if p90_whole == 0 { 0.0 } else { percentile(&s, 0.9) as f64 * 100.0 / p90_whole as f64 };
                let _ = writeln!(out, "{:<14}{}{:>7.0}%", name, stat(&s), share);
            }
            let _ = writeln!(out, "{:<14}{}", "= 1 frame", stat(&sorted(&whole)));
            for (name, samples) in FrameMeasurement::INCLUDED.iter().zip(included) {
                let _ = writeln!(out, "  · {:<10}{}", name, stat(&sorted(samples)));
            }
            if let Some((_, rows)) = self.inside.iter().find(|(n, _)| n == view) {
                let _ = writeln!(out, "\n  resolve の中(回したデータで割った):");
                let mut rows: Vec<_> = rows.iter().map(|(n, s)| (n.clone(), sorted(s))).collect();
                rows.sort_by_key(|(_, s)| std::cmp::Reverse(percentile(s, 0.9)));
                for (name, s) in &rows {
                    let share = if p90_whole == 0 { 0.0 } else { percentile(s, 0.9) as f64 * 100.0 / p90_whole as f64 };
                    let _ = writeln!(out, "  {:<24}{}{:>7.0}%", name, stat(s), share);
                }
            }
            if let Some((_, worst)) = self.worst.iter().find(|(n, _)| n == view) {
                let mut worst = worst.clone();
                worst.sort_unstable_by(|a, b| b.0.cmp(&a.0));
                worst.dedup_by_key(|(_, layer, _)| *layer);
                let names: Vec<String> = worst.iter().take(5)
                    .map(|(us, layer, kind)| format!("L{layer} {kind} {:.3}ms", *us as f64 / 1000.0)).collect();
                if !names.is_empty() { let _ = writeln!(out, "  重かった層: {}", names.join(", ")); }
            }
        }
        out
    }
}

fn sorted(v: &[u64]) -> Vec<u64> {
    let mut v = v.to_vec();
    v.sort_unstable();
    v
}

fn percentile(sorted: &[u64], fraction: f64) -> u64 {
    if sorted.is_empty() { return 0 }
    sorted[((sorted.len() as f64 * fraction).ceil() as usize).clamp(1, sorted.len()) - 1]
}

fn stat(sorted: &[u64]) -> String {
    if sorted.is_empty() { return format!("{:>9}{:>9}{:>9}", "—", "—", "—") }
    format!("{:>9.3}{:>9.3}{:>9.3}",
        sorted[sorted.len() / 2] as f64 / 1000.0,
        percentile(sorted, 0.9) as f64 / 1000.0,
        sorted[sorted.len() - 1] as f64 / 1000.0)
}

impl crate::EditorRuntime {
    /// Ⅱ で 1 度だけ。誰が 1 拍を食ったかを窓の外へ出す(毎コマは何も書かない)。
    pub(crate) fn report_owners(&mut self) {
        if self.owners.is_empty() { return }
        let composition = self.doc.view().composition().ok().flatten();
        let (head, budget_us) = match composition {
            Some(c) => {
                let (spec, fps) = (c.spec(), c.fps.as_f64());
                let name = self.last_script.as_ref()
                    .map(|(p, _, _)| p.rsplit('/').next().unwrap_or(p).to_owned())
                    .or_else(|| self.path.clone())
                    .unwrap_or_else(|| "untitled".into());
                let stage = self.viewer.stage_window.map_or_else(|| "—".into(), |w| format!("{}x{}", w.width, w.height));
                (format!("doc={name} comp={}x{} stage={stage} fps={fps}", spec.width, spec.height),
                 (1_000_000.0 / fps) as u64)
            }
            None => ("doc=none".to_owned(), 16_667),
        };
        let dropped = self.owners.dropped_since_play(self.frames.skipped());
        let report = self.owners.report(&head, budget_us, dropped);
        eprint!("{report}");
        let _ = std::fs::write("/tmp/motolii-playback-owners.txt", &report);
        self.owners.clear(self.frames.skipped());
    }
}
