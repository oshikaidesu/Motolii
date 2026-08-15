const SET_TIME_THROTTLE_MS: u64 = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ScrubPointerPhase {
    Down,
    Move,
    Up,
    Cancel,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ScrubTimePump {
    down_frame: Option<i64>,
    last_dispatch_ms: Option<u64>,
    sent_since_down: bool,
}

impl ScrubTimePump {
    pub(super) fn new() -> Self {
        Self {
            down_frame: None,
            last_dispatch_ms: None,
            sent_since_down: false,
        }
    }

    pub(super) fn is_active(&self) -> bool {
        self.down_frame.is_some()
    }

    fn should_send_throttled(&self, now_ms: u64) -> bool {
        self.last_dispatch_ms
            .is_none_or(|last| now_ms.saturating_sub(last) >= SET_TIME_THROTTLE_MS)
    }

    pub(super) fn next_frame(
        &mut self,
        phase: ScrubPointerPhase,
        bar: f64,
        now_ms: u64,
        fps_num: i64,
        fps_den: i64,
    ) -> Option<i64> {
        if fps_num <= 0 || fps_den <= 0 {
            return None;
        }
        let frame = crate::host_bridge::frame_from_scrub_bar(bar, fps_num, fps_den);
        match phase {
            ScrubPointerPhase::Down => {
                self.down_frame = Some(frame);
                self.last_dispatch_ms = Some(now_ms);
                self.sent_since_down = true;
                Some(frame)
            }
            ScrubPointerPhase::Move => {
                if !self.should_send_throttled(now_ms) {
                    return None;
                }
                self.last_dispatch_ms = Some(now_ms);
                self.sent_since_down = true;
                Some(frame)
            }
            ScrubPointerPhase::Up => {
                self.last_dispatch_ms = Some(now_ms);
                self.sent_since_down = true;
                self.down_frame = None;
                Some(frame)
            }
            ScrubPointerPhase::Cancel => {
                let dispatch_frame = self.down_frame;
                self.down_frame = None;
                if self.sent_since_down {
                    self.sent_since_down = false;
                    return dispatch_frame;
                }
                None
            }
        }
    }
}
