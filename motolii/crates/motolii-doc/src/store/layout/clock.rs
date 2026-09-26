//! A layer's clock: what time a layer's values are read at, as data. The order shifts of its ancestors (outermost first,
//! each by the one order law, `schedule_delay`), then the layer's own Loop fold. Built from the document once, it maps
//! time without the document, so the document (`value_at`) and the renderer's graph read values at the same time.

use super::*;
use crate::doc::eval::KeyframeTrack;
use crate::doc::store::slot::PropertyBase;

/// One row's value over time, as the document holds it (a preview in progress wins, as in `value_at`).
#[derive(Clone, Debug, PartialEq)]
pub enum RowValue {
    Missing,
    Constant(Value),
    Track(KeyframeTrack),
}

impl RowValue {
    fn at(&self, t: RationalTime) -> Option<Value> {
        match self {
            Self::Missing => None,
            Self::Constant(v) => Some(v.clone()),
            Self::Track(track) => Some(track.eval(t)),
        }
    }
    fn number(&self, t: RationalTime) -> f64 {
        match self.at(t) {
            Some(Value::F64(v)) => v,
            Some(Value::Enum(v)) => v as f64,
            _ => 0.0,
        }
    }
    fn choice(&self, t: RationalTime) -> i64 {
        match self.at(t) {
            Some(Value::Enum(v)) => v,
            Some(Value::F64(v)) => v.round() as i64,
            _ => 0,
        }
    }
}

/// Member i of n under a holder: the holder's order rows, read unshifted at the time they are asked.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderStep {
    pub index: usize,
    pub count: usize,
    pub stagger: RowValue,
    pub from: RowValue,
    pub from_end: RowValue,
}

impl OrderStep {
    fn shift(&self, base: RationalTime) -> Result<RationalTime, StoreError> {
        let stagger = self.stagger.number(base);
        let delay = schedule_delay(if stagger > 1e-9 { stagger } else { 0.0 }, self.from.choice(base), self.from_end.choice(base) == 1, self.index, self.count);
        shift_by(base, delay)
    }
}

/// Move a time by a delay in seconds (positive is later: the time read is earlier), never before zero.
pub fn shift_by(base: RationalTime, delay: f64) -> Result<RationalTime, StoreError> {
    if delay == 0.0 {
        return Ok(base);
    }
    let off = RationalTime::try_new((delay.abs() * 1_000_000.0).round() as i64, 1_000_000).map_err(|e| StoreError::Property(format!("stagger: {e}")))?;
    let shifted = if delay < 0.0 { base.try_add(off) } else { base.try_sub(off) }.map_err(|e| StoreError::Property(format!("stagger: {e}")))?;
    Ok(if shifted.as_seconds_f64() < 0.0 { RationalTime::ZERO } else { shifted })
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerClock {
    /// The ancestors' order steps, outermost first.
    pub steps: Vec<OrderStep>,
    pub loop_duration: RowValue,
    pub loop_direction: RowValue,
}

impl LayerClock {
    /// Nothing moves time: no order step can shift and no Loop is written.
    pub fn is_identity(&self) -> bool {
        self.steps.iter().all(|s| s.stagger == RowValue::Missing) && self.loop_duration == RowValue::Missing
    }

    /// The layer's time after its ancestors' order shifts (what `layer_time` means).
    pub fn ordered(&self, t: RationalTime) -> Result<RationalTime, StoreError> {
        self.steps.iter().try_fold(t, |base, step| step.shift(base))
    }

    /// The time the layer's values are read at: order shifts, then the Loop fold. The Loop rows are read at the ordered
    /// time as the document reads any unfolded row of the layer (its order shift applied once more), as `looped_time` did.
    pub fn local(&self, t: RationalTime) -> Result<RationalTime, StoreError> {
        let shifted = self.ordered(t)?;
        self.local_from_ordered(shifted)
    }

    /// The Loop fold of an already ordered time.
    pub fn local_from_ordered(&self, shifted: RationalTime) -> Result<RationalTime, StoreError> {
        if self.loop_duration == RowValue::Missing {
            return Ok(shifted);
        }
        let read = self.ordered(shifted)?;
        let duration = self.loop_duration.number(read);
        if duration <= 1e-9 {
            return Ok(shifted);
        }
        let seconds = shifted.as_seconds_f64();
        let round = (seconds / duration).floor();
        let along = seconds - round * duration;
        let odd = round.rem_euclid(2.0) >= 1.0;
        let local = match self.loop_direction.choice(read) {
            1 => duration - along,
            2 => if odd { duration - along } else { along },
            3 => if odd { along } else { duration - along },
            _ => along,
        };
        RationalTime::try_new((local * 1_000_000.0).round() as i64, 1_000_000).map_err(|e| StoreError::Property(format!("loop: {e}")))
    }

    /// The time a row of this layer is read at: order rows unshifted, Loop rows shifted but not folded, the rest local.
    pub fn for_row(&self, property: &str, t: RationalTime) -> Result<RationalTime, StoreError> {
        if is_schedule_row(property) {
            Ok(t)
        } else if is_loop_row(property) {
            self.ordered(t)
        } else {
            self.local(t)
        }
    }
}

impl StoreView<'_> {
    fn row_value(&self, layer: LayerId, name: &str) -> Result<RowValue, StoreError> {
        let property = PropertyId::new(name)?;
        let path = layer.entity_path();
        if !self.ignore_transients_flag() {
            if let Some(v) = self.transient_value_at(&path, &property) {
                return Ok(RowValue::Constant(v));
            }
        }
        let Some(source) = self.property_source(layer, &property)? else { return Ok(RowValue::Missing) };
        Ok(match source.base {
            Some(PropertyBase::Constant(v)) => RowValue::Constant(v),
            Some(PropertyBase::Track(track)) => RowValue::Track(track),
            Some(PropertyBase::Slot(slot)) => self.slot_track(&slot)?.map_or(RowValue::Missing, RowValue::Track),
            None => RowValue::Missing,
        })
    }

    /// The layer's clock, built from the document once per view and revision.
    pub fn layer_clock(&self, layer: LayerId) -> Result<std::sync::Arc<LayerClock>, StoreError> {
        let key = (self.revision_key(), layer);
        if let Some(hit) = self.layout_memo().borrow().clocks.get(&key).cloned() {
            return Ok(hit);
        }
        let clock = std::sync::Arc::new(self.build_clock(layer)?);
        let mut scratch = self.layout_memo().borrow_mut();
        if scratch.clocks.len() > 4096 {
            scratch.clocks.clear();
        }
        scratch.clocks.insert(key, clock.clone());
        Ok(clock)
    }

    fn build_clock(&self, layer: LayerId) -> Result<LayerClock, StoreError> {
        let mut steps = Vec::new();
        let mut at = layer;
        while let Some(parent) = self.attrs(at)?.unwrap_or_default().parent {
            let siblings = self.schedule_children(parent)?;
            if let Some(index) = siblings.iter().position(|&s| s == at) {
                steps.push(OrderStep {
                    index,
                    count: siblings.len(),
                    stagger: self.row_value(parent, STAGGER)?,
                    from: self.row_value(parent, STAGGER_FROM)?,
                    from_end: self.row_value(parent, FROM_END)?,
                });
            }
            at = parent;
        }
        steps.reverse();
        Ok(LayerClock { steps, loop_duration: self.row_value(layer, LOOP_DURATION)?, loop_direction: self.row_value(layer, LOOP_DIRECTION)? })
    }
}
