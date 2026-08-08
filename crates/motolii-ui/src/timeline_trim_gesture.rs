//! Trim中に確定値を書き込まず、releaseだけ既存writerへ渡すためのTransient lifecycle。

use motolii_core::{RationalTime, RationalTimeError};
use motolii_doc::LayerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelineTrimRequest {
    In {
        layer: LayerId,
        new_start: RationalTime,
    },
    Out {
        layer: LayerId,
        new_end: RationalTime,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelineTrimEdge {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum TimelineTrimGestureError {
    #[error("timeline trim time arithmetic overflowed")]
    TimeOverflow,
}

impl From<RationalTimeError> for TimelineTrimGestureError {
    fn from(_: RationalTimeError) -> Self {
        Self::TimeOverflow
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TimelineTrimGesture {
    layer: LayerId,
    edge: TimelineTrimEdge,
    initial_pointer: RationalTime,
    initial_start: RationalTime,
    initial_end: RationalTime,
    generation: u64,
}

impl TimelineTrimGesture {
    pub(crate) const fn begin(
        layer: LayerId,
        edge: TimelineTrimEdge,
        initial_pointer: RationalTime,
        initial_start: RationalTime,
        initial_end: RationalTime,
        generation: u64,
    ) -> Self {
        Self {
            layer,
            edge,
            initial_pointer,
            initial_start,
            initial_end,
            generation,
        }
    }

    pub(crate) const fn layer(&self) -> LayerId {
        self.layer
    }

    pub(crate) const fn edge(&self) -> TimelineTrimEdge {
        self.edge
    }

    pub(crate) const fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) const fn initial_start(&self) -> RationalTime {
        self.initial_start
    }

    pub(crate) const fn initial_end(&self) -> RationalTime {
        self.initial_end
    }

    pub(crate) fn preview(
        &self,
        pointer_time: RationalTime,
    ) -> Result<TimelineTrimRequest, TimelineTrimGestureError> {
        let delta = pointer_time.try_sub(self.initial_pointer)?;
        match self.edge {
            TimelineTrimEdge::Left => Ok(TimelineTrimRequest::In {
                layer: self.layer,
                new_start: self.initial_start.try_add(delta)?,
            }),
            TimelineTrimEdge::Right => Ok(TimelineTrimRequest::Out {
                layer: self.layer,
                new_end: self.initial_end.try_add(delta)?,
            }),
        }
    }

    pub(crate) fn release(
        self,
        pointer_time: RationalTime,
    ) -> Result<Option<TimelineTrimRequest>, TimelineTrimGestureError> {
        let request = self.preview(pointer_time)?;
        let unchanged = match request {
            TimelineTrimRequest::In { new_start, .. } => new_start == self.initial_start,
            TimelineTrimRequest::Out { new_end, .. } => new_end == self.initial_end,
        };
        Ok((!unchanged).then_some(request))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(num: i64) -> RationalTime {
        RationalTime::try_new(num, 10).unwrap()
    }

    #[test]
    fn left_and_right_preview_apply_only_pointer_delta() {
        let layer = LayerId::from_raw(7);
        let left = TimelineTrimGesture::begin(
            layer,
            TimelineTrimEdge::Left,
            time(10),
            time(20),
            time(80),
            4,
        );
        let right = TimelineTrimGesture::begin(
            layer,
            TimelineTrimEdge::Right,
            time(10),
            time(20),
            time(80),
            4,
        );

        assert_eq!(
            left.preview(time(25)).unwrap(),
            TimelineTrimRequest::In {
                layer,
                new_start: time(35),
            }
        );
        assert_eq!(
            right.preview(time(25)).unwrap(),
            TimelineTrimRequest::Out {
                layer,
                new_end: time(95),
            }
        );
    }

    #[test]
    fn same_pointer_release_is_a_noop() {
        for edge in [TimelineTrimEdge::Left, TimelineTrimEdge::Right] {
            let gesture = TimelineTrimGesture::begin(
                LayerId::from_raw(1),
                edge,
                time(10),
                time(20),
                time(80),
                0,
            );
            assert_eq!(gesture.release(time(10)).unwrap(), None);
        }
    }

    #[test]
    fn arithmetic_overflow_cancels_without_a_request() {
        let gesture = TimelineTrimGesture::begin(
            LayerId::from_raw(3),
            TimelineTrimEdge::Right,
            RationalTime::ZERO,
            RationalTime::ZERO,
            RationalTime::try_new(i64::MAX, 1).unwrap(),
            0,
        );
        assert_eq!(
            gesture.preview(RationalTime::try_new(1, 1).unwrap()),
            Err(TimelineTrimGestureError::TimeOverflow)
        );
    }
}
