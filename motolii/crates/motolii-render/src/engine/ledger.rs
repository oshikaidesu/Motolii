//! Who worked this frame, and why. Every piece of per-frame work that runs claims itself here
//! with its stage, its owner and its reason; work that was reused makes no claim. A frame over its
//! budget reports its claims, most expensive first, so the cause names itself.

use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub struct Claim {
    /// evaluate, prepare, compose, blocks, draw.
    pub stage: &'static str,
    pub who: String,
    pub why: String,
    pub us: u64,
    /// Claims of the same stage, owner and reason in this frame.
    pub count: u32,
}

#[derive(Default)]
pub struct FrameLedger {
    claims: Vec<Claim>,
    last_report: Option<Instant>,
    /// The clock frame being drawn and what its views have spent so far: every view drawn for
    /// one tick shares the tick's budget.
    tick: Option<crate::doc::core::RationalTime>,
    views: Vec<String>,
    spent: Duration,
}

impl FrameLedger {
    pub fn clear(&mut self) {
        self.claims.clear();
        self.tick = None;
        self.views.clear();
        self.spent = Duration::ZERO;
    }

    /// Starts `view` of the tick at `time`. Another time, or a view already drawn (a still frame
    /// redrawn), starts a new tick.
    pub fn begin_view(&mut self, time: crate::doc::core::RationalTime, view: String) {
        if self.tick != Some(time) || self.views.contains(&view) {
            self.clear();
            self.tick = Some(time);
        }
        self.views.push(view);
    }

    /// Adds a finished view's time to its tick and returns the tick's total so far.
    pub fn end_view(&mut self, spent: Duration) -> Duration {
        self.spent += spent;
        self.spent
    }

    pub fn claim(&mut self, stage: &'static str, who: impl Into<String>, why: impl Into<String>, spent: Duration) {
        let (who, why, us) = (who.into(), why.into(), spent.as_micros() as u64);
        match self.claims.iter_mut().find(|c| c.stage == stage && c.who == who && c.why == why) {
            Some(claim) => { claim.us += us; claim.count += 1; }
            None => self.claims.push(Claim { stage, who, why, us, count: 1 }),
        }
    }

    pub fn claims(&self) -> &[Claim] {
        &self.claims
    }

    /// The frame's claims, costliest first, when it took longer than its budget. At most once a
    /// second, so a slow scene does not flood the log.
    pub fn report_if_over(&mut self, spent: Duration, budget: Duration) -> Option<String> {
        if spent <= budget { return None; }
        let now = Instant::now();
        if self.last_report.is_some_and(|at| now.duration_since(at) < Duration::from_secs(1)) { return None; }
        self.last_report = Some(now);
        let mut claims = self.claims.clone();
        claims.sort_by(|a, b| b.us.cmp(&a.us));
        let mut report = format!(
            "MOTOLII_SLOW_FRAME {:.1} ms for every view of the tick (budget {:.1} ms)",
            spent.as_secs_f64() * 1e3,
            budget.as_secs_f64() * 1e3,
        );
        for claim in claims.iter().take(8) {
            let times = if claim.count > 1 { format!(" ×{}", claim.count) } else { String::new() };
            report.push_str(&format!("\n  {:>7.2} ms  {:<8} {}{} — {}", claim.us as f64 / 1e3, claim.stage, claim.who, times, claim.why));
        }
        Some(report)
    }
}
