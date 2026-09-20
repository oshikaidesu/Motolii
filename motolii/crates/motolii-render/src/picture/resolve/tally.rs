//! resolve の中を**データで割る**。段の名前を手で並べない — 回している物が名乗る。
//!
//! 層は `LayerSource::kind()`(新しい種類を足すと `match` が落ちるので、必ず名乗る)。
//! 固定の相は数が少なく、増えるのは層・効果の側なので、そちらだけ自動で行が増える。
//! 既定では**何もしない**(`begin` を呼んだ拍だけ数える) — 毎コマの CPU を増やさないため。

use std::cell::RefCell;

thread_local! {
    static TALLY: RefCell<Option<Vec<(&'static str, String, u64, u32)>>> = const { RefCell::new(None) };
    /// 一番食った層だけ名指しで残す(全層ぶん名前を作ると、計器が重くなる)。
    static WORST: RefCell<Vec<(u64, u64, &'static str)>> = const { RefCell::new(Vec::new()) };
}

const WORST_KEPT: usize = 5;

/// この拍だけ数える。
pub fn begin() {
    TALLY.with(|t| *t.borrow_mut() = Some(Vec::new()));
    WORST.with(|w| w.borrow_mut().clear());
}

/// 数えているか。層ごとの時計を作る前の関所。
pub fn counting() -> bool {
    TALLY.with(|t| t.borrow().is_some())
}

/// この拍で重かった層を上から `WORST_KEPT` 本だけ。
pub fn worst(layer: crate::doc::store::LayerId, kind: &'static str, us: u64) {
    WORST.with(|w| {
        let mut w = w.borrow_mut();
        if w.len() == WORST_KEPT && w.last().is_some_and(|(worst, _, _)| *worst >= us) { return }
        w.push((us, layer.0, kind));
        w.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        w.truncate(WORST_KEPT);
    });
}

pub fn take_worst() -> Vec<(u64, u64, &'static str)> {
    WORST.with(|w| std::mem::take(&mut *w.borrow_mut()))
}

/// 畳んだ行(`group`, `name`, µs, 本数)。読んだら止まる。
pub fn take() -> Vec<(&'static str, String, u64, u32)> {
    TALLY.with(|t| t.borrow_mut().take()).unwrap_or_default()
}

pub fn add(group: &'static str, name: &str, us: u64) {
    TALLY.with(|t| {
        let mut slot = t.borrow_mut();
        let Some(rows) = slot.as_mut() else { return };
        match rows.iter_mut().find(|(g, n, _, _)| *g == group && n == name) {
            Some((_, _, total, count)) => { *total += us; *count += 1 }
            None => rows.push((group, name.to_owned(), us, 1)),
        }
    });
}

/// 生きている間を測って、落ちる時に足す。
pub struct Span {
    group: &'static str,
    name: &'static str,
    started: std::time::Instant,
    on: bool,
}

impl Span {
    pub fn new(group: &'static str, name: &'static str) -> Self {
        let on = TALLY.with(|t| t.borrow().is_some());
        Self { group, name, started: std::time::Instant::now(), on }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        if self.on {
            add(self.group, self.name, self.started.elapsed().as_micros() as u64);
        }
    }
}

/// 数えている時だけ時計を持つ。`None` なら `Drop` も走らない。
pub fn span(group: &'static str, name: &'static str) -> Option<Span> {
    TALLY.with(|t| t.borrow().is_some()).then(|| Span::new(group, name))
}
