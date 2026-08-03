//! Timeline body-dragのTransient lifecycle。Documentへはrelease時だけ渡す。

use motolii_core::{RationalTime, RationalTimeError};
use motolii_doc::LayerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TimelineMoveRequest {
    pub(crate) layer: LayerId,
    pub(crate) new_start: RationalTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum TimelineMoveGestureError {
    #[error("timeline move time arithmetic overflowed")]
    TimeOverflow,
}

impl From<RationalTimeError> for TimelineMoveGestureError {
    fn from(_: RationalTimeError) -> Self {
        Self::TimeOverflow
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TimelineMoveGesture {
    layer: LayerId,
    initial_pointer: RationalTime,
    initial_start: RationalTime,
    generation: u64,
}

impl TimelineMoveGesture {
    pub(crate) fn begin(
        layer: LayerId,
        initial_pointer: RationalTime,
        initial_start: RationalTime,
        generation: u64,
    ) -> Self {
        Self {
            layer,
            initial_pointer,
            initial_start,
            generation,
        }
    }

    pub(crate) fn layer(self) -> LayerId {
        self.layer
    }

    pub(crate) fn generation(self) -> u64 {
        self.generation
    }

    pub(crate) fn initial_start(self) -> RationalTime {
        self.initial_start
    }

    pub(crate) fn preview(
        &self,
        pointer_time: RationalTime,
    ) -> Result<RationalTime, TimelineMoveGestureError> {
        let delta = pointer_time.try_sub(self.initial_pointer)?;
        Ok(self.initial_start.try_add(delta)?)
    }

    pub(crate) fn release(
        self,
        pointer_time: RationalTime,
    ) -> Result<Option<TimelineMoveRequest>, TimelineMoveGestureError> {
        let new_start = self.preview(pointer_time)?;
        if new_start == self.initial_start {
            return Ok(None);
        }
        Ok(Some(TimelineMoveRequest {
            layer: self.layer,
            new_start,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(num: i64) -> RationalTime {
        RationalTime::try_new(num, 10).unwrap()
    }

    #[test]
    fn body_drag_applies_pointer_delta_without_document_side_effects() {
        let layer = LayerId::from_raw(7);
        let gesture = TimelineMoveGesture::begin(layer, time(10), time(20), 4);
        assert_eq!(gesture.preview(time(25)).unwrap(), time(35));
        assert_eq!(gesture.generation(), 4);
        assert_eq!(gesture.layer(), layer);
    }

    #[test]
    fn same_value_release_is_a_noop() {
        let gesture = TimelineMoveGesture::begin(LayerId::from_raw(1), time(10), time(20), 0);
        assert_eq!(gesture.release(time(10)).unwrap(), None);
    }

    #[test]
    fn release_emits_one_typed_request() {
        let gesture = TimelineMoveGesture::begin(LayerId::from_raw(2), time(10), time(20), 9);
        assert_eq!(
            gesture.release(time(15)).unwrap(),
            Some(TimelineMoveRequest {
                layer: LayerId::from_raw(2),
                new_start: time(25),
            })
        );
    }

    #[test]
    fn arithmetic_overflow_cancels_the_gesture() {
        let gesture = TimelineMoveGesture::begin(
            LayerId::from_raw(3),
            RationalTime::ZERO,
            RationalTime::try_new(i64::MAX, 1).unwrap(),
            0,
        );
        assert_eq!(
            gesture.preview(RationalTime::from_seconds(1)),
            Err(TimelineMoveGestureError::TimeOverflow)
        );
    }
}
