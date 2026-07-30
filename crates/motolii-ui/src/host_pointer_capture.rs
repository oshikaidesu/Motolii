//! WebView境界を越えるpointer lifecycleをHost内へ閉じる。

use crate::{EffectiveTrigger, InputPhase, KeyToken, Modifier, Modifiers};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HostPointerSample {
    pub(crate) position: [f64; 2],
    pub(crate) left_button_down: bool,
    pub(crate) window_focused: bool,
    pub(crate) escape_pressed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HostPointerCancel {
    Escape,
    CaptureLost,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum HostPointerCandidate {
    Moved {
        generation: u64,
        position: [f64; 2],
    },
    Released {
        generation: u64,
        position: [f64; 2],
    },
    Cancelled {
        generation: u64,
        reason: HostPointerCancel,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HostPointerClick {
    pub(crate) position: [f64; 2],
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PointerUpEntry {
    sequence: u64,
    position: [f64; 2],
}

#[derive(Debug, Default)]
struct PointerUpQueue {
    next_sequence: u64,
    entries: std::collections::VecDeque<PointerUpEntry>,
}

impl PointerUpQueue {
    fn enqueue(&mut self, position: [f64; 2]) -> Option<u64> {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.checked_add(1)?;
        self.entries
            .push_back(PointerUpEntry { sequence, position });
        Some(sequence)
    }

    fn high_water(&self) -> u64 {
        self.next_sequence
    }

    fn claim_first_after(&mut self, arm_sequence: u64) -> Option<PointerUpEntry> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.sequence >= arm_sequence)?;
        self.entries.remove(index)
    }

    fn pop_unclaimed(&mut self) -> Option<PointerUpEntry> {
        self.entries.pop_front()
    }
}

#[derive(Debug, Default)]
pub(crate) struct HostPointerCaptureState {
    active: Option<ActiveCapture>,
}

#[derive(Debug)]
struct ActiveCapture {
    generation: u64,
    arm_sequence: u64,
}

impl HostPointerCaptureState {
    pub(crate) fn is_active(&self) -> bool {
        self.active.is_some()
    }

    fn active_generation(&self) -> Option<u64> {
        self.active.as_ref().map(|active| active.generation)
    }

    pub(crate) fn arm(&mut self, generation: u64, arm_sequence: u64) -> bool {
        if self.active.is_some() {
            return false;
        }
        self.active = Some(ActiveCapture {
            generation,
            arm_sequence,
        });
        true
    }

    fn update(
        &mut self,
        sample: HostPointerSample,
        queue: &mut PointerUpQueue,
    ) -> Option<HostPointerCandidate> {
        let active = self.active.as_mut()?;
        let generation = active.generation;
        if !sample.window_focused {
            self.active = None;
            return Some(HostPointerCandidate::Cancelled {
                generation,
                reason: HostPointerCancel::CaptureLost,
            });
        }
        if sample.escape_pressed {
            self.active = None;
            return Some(HostPointerCandidate::Cancelled {
                generation,
                reason: HostPointerCancel::Escape,
            });
        }
        if let Some(entry) = queue.claim_first_after(active.arm_sequence) {
            self.active = None;
            return Some(HostPointerCandidate::Released {
                generation,
                position: entry.position,
            });
        }
        if sample.left_button_down {
            return Some(HostPointerCandidate::Moved {
                generation,
                position: sample.position,
            });
        }
        None
    }
}

#[cfg(target_os = "macos")]
pub(crate) struct PlatformPointerCapture {
    window: objc2::rc::Retained<objc2_app_kit::NSWindow>,
    state: HostPointerCaptureState,
    up_queue: std::sync::Arc<std::sync::Mutex<PointerUpQueue>>,
    command_inbox: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<EffectiveTrigger>>>,
    host_commands_enabled: std::sync::Arc<std::sync::atomic::AtomicBool>,
    click_monitor: objc2::rc::Retained<objc2::runtime::AnyObject>,
    armed_after_event_timestamp: f64,
    last_logged_position: Option<[f64; 2]>,
}

#[cfg(target_os = "macos")]
impl PlatformPointerCapture {
    pub(crate) fn new(
        window: &winit::window::Window,
        wake: std::sync::Arc<dyn Fn() + Send + Sync>,
    ) -> Result<Self, PlatformPointerCaptureError> {
        use objc2::rc::Retained;
        use objc2_app_kit::NSView;
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

        let handle = window
            .window_handle()
            .map_err(|_| PlatformPointerCaptureError::WindowHandle)?;
        let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
            return Err(PlatformPointerCaptureError::WrongPlatform);
        };
        // SAFETY: WindowHandleのlifetime中はns_viewが有効であり、retainして所有を得る。
        let view: Retained<NSView> = unsafe { Retained::retain(handle.ns_view.as_ptr().cast()) }
            .ok_or(PlatformPointerCaptureError::MissingView)?;
        let window = view
            .window()
            .ok_or(PlatformPointerCaptureError::MissingWindow)?;
        let up_queue = std::sync::Arc::new(std::sync::Mutex::new(PointerUpQueue::default()));
        let monitor_inbox = std::sync::Arc::clone(&up_queue);
        let command_inbox =
            std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new()));
        let monitor_command_inbox = std::sync::Arc::clone(&command_inbox);
        let host_commands_enabled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let monitor_commands_enabled = std::sync::Arc::clone(&host_commands_enabled);
        let monitor_wake = std::sync::Arc::clone(&wake);
        let monitor_window = window.clone();
        let monitor =
            block2::RcBlock::new(move |event: std::ptr::NonNull<objc2_app_kit::NSEvent>| {
                // SAFETY: AppKitはlocal monitor呼び出し中のevent生存を保証する。
                let event = unsafe { event.as_ref() };
                if event.r#type() == objc2_app_kit::NSEventType::LeftMouseUp {
                    if let Some(content) = monitor_window.contentView() {
                        let content_point =
                            content.convertPoint_fromView(event.locationInWindow(), None);
                        let content_height = content.bounds().size.height;
                        let position = top_down_logical(
                            content_point.x,
                            content_point.y,
                            content.isFlipped(),
                            content_height,
                        );
                        let enqueued = match monitor_inbox.lock() {
                            Ok(mut inbox) => inbox.enqueue(position).is_some(),
                            Err(_) => false,
                        };
                        if enqueued {
                            monitor_wake();
                        }
                    }
                } else if monitor_commands_enabled.load(std::sync::atomic::Ordering::Acquire) {
                    if let Some(trigger) = mac_history_trigger(event) {
                        if let Ok(mut inbox) = monitor_command_inbox.lock() {
                            inbox.push_back(trigger);
                            wake();
                            return std::ptr::null_mut();
                        }
                    }
                }
                event as *const objc2_app_kit::NSEvent as *mut objc2_app_kit::NSEvent
            });
        // SAFETY: blockは受け取ったlive eventをそのまま返し、monitor tokenはDropで除去する。
        let click_monitor = unsafe {
            objc2_app_kit::NSEvent::addLocalMonitorForEventsMatchingMask_handler(
                objc2_app_kit::NSEventMask::LeftMouseUp | objc2_app_kit::NSEventMask::KeyDown,
                &monitor,
            )
        }
        .ok_or(PlatformPointerCaptureError::EventMonitor)?;
        Ok(Self {
            window,
            state: HostPointerCaptureState::default(),
            up_queue,
            command_inbox,
            host_commands_enabled,
            click_monitor,
            armed_after_event_timestamp: 0.0,
            last_logged_position: None,
        })
    }

    pub(crate) fn arm(&mut self, generation: u64) -> bool {
        let queue = match self.up_queue.lock() {
            Ok(queue) => queue,
            Err(_) => {
                // なぜ: lock失敗は bool で失敗を表現するしかなく、take_place_intent は false を
                // PointerCaptureAlreadyActive に写像するため、拒否のみが忠実な結果になるため。
                return false;
            }
        };
        let arm_sequence = queue.high_water();
        if !self.state.arm(generation, arm_sequence) {
            return false;
        }
        self.last_logged_position = None;
        self.armed_after_event_timestamp = self
            .window
            .currentEvent()
            .map(|event| event.timestamp())
            .unwrap_or(0.0);
        true
    }

    pub(crate) fn is_active(&self) -> bool {
        self.state.is_active()
    }

    pub(crate) fn poll(
        &mut self,
    ) -> Result<Option<HostPointerCandidate>, PlatformPointerCaptureError> {
        use objc2_app_kit::{NSEvent, NSEventType};

        let content = self
            .window
            .contentView()
            .ok_or(PlatformPointerCaptureError::MissingView)?;
        // window event streamはWebKitのtracking loop中に止まり得るため、
        // global screen pointをwindow/content座標へ変換する。
        let screen_point = NSEvent::mouseLocation();
        let window_point = self.window.convertPointFromScreen(screen_point);
        let content_point = content.convertPoint_fromView(window_point, None);
        let content_height = content.bounds().size.height;
        let position = top_down_logical(
            content_point.x,
            content_point.y,
            content.isFlipped(),
            content_height,
        );
        if position_changed(self.last_logged_position, position) {
            crate::ui_numeric_trace::emit(format_args!(
                "kind=appkit-pointer generation={} raw_x={:.3} raw_y={:.3} content_height={:.3} \
                 content_is_flipped={} logical_x={:.3} logical_y={:.3}",
                self.state.active_generation().unwrap_or(0),
                content_point.x,
                content_point.y,
                content_height,
                content.isFlipped(),
                position[0],
                position[1],
            ));
            self.last_logged_position = Some(position);
        }
        let window_focused = self.window.isKeyWindow();
        let escape_pressed = window_focused
            && self.window.currentEvent().is_some_and(|event| {
                event.r#type() == NSEventType::KeyDown
                    && event.keyCode() == 53
                    && !event.isARepeat()
                    && event.timestamp() > self.armed_after_event_timestamp
            });
        let mut queue = self
            .up_queue
            .lock()
            .map_err(|_| PlatformPointerCaptureError::UpQueuePoisoned)?;
        let sample = HostPointerSample {
            position,
            left_button_down: NSEvent::pressedMouseButtons() & 1 == 1,
            window_focused,
            escape_pressed,
        };
        Ok(self.state.update(sample, &mut queue))
    }

    pub(crate) fn poll_click(
        &mut self,
    ) -> Result<Option<HostPointerClick>, PlatformPointerCaptureError> {
        let mut queue = self
            .up_queue
            .lock()
            .map_err(|_| PlatformPointerCaptureError::UpQueuePoisoned)?;
        Ok(queue.pop_unclaimed().map(|entry| HostPointerClick {
            position: entry.position,
        }))
    }

    pub(crate) fn set_host_commands_enabled(&self, enabled: bool) {
        self.host_commands_enabled
            .store(enabled, std::sync::atomic::Ordering::Release);
    }

    pub(crate) fn poll_command(
        &mut self,
    ) -> Result<Option<EffectiveTrigger>, PlatformPointerCaptureError> {
        self.command_inbox
            .lock()
            .map_err(|_| PlatformPointerCaptureError::CommandInboxPoisoned)
            .map(|mut inbox| inbox.pop_front())
    }
}

#[cfg(target_os = "macos")]
fn top_down_logical(x: f64, y: f64, is_flipped: bool, height: f64) -> [f64; 2] {
    if is_flipped {
        [x, y]
    } else {
        [x, height - y]
    }
}

#[cfg(target_os = "macos")]
fn mac_history_trigger(event: &objc2_app_kit::NSEvent) -> Option<EffectiveTrigger> {
    use objc2_app_kit::NSEventModifierFlags;

    if event.r#type() != objc2_app_kit::NSEventType::KeyDown || event.isARepeat() {
        return None;
    }
    let flags = event.modifierFlags();
    let characters = event.charactersIgnoringModifiers()?.to_string();
    if !flags.contains(NSEventModifierFlags::Command)
        || flags.intersects(NSEventModifierFlags::Control | NSEventModifierFlags::Option)
        || !is_history_character(&characters)
    {
        return None;
    }
    let modifiers = Modifiers::try_new(
        [
            Some(Modifier::Meta),
            flags
                .contains(NSEventModifierFlags::Shift)
                .then_some(Modifier::Shift),
        ]
        .into_iter()
        .flatten(),
    )
    .ok()?;
    Some(EffectiveTrigger::Keyboard {
        key: KeyToken::Ascii(crate::AsciiKey::try_new('z').ok()?),
        modifiers,
        phase: InputPhase::Press,
    })
}

#[cfg(target_os = "macos")]
fn is_history_character(characters: &str) -> bool {
    characters.eq_ignore_ascii_case("z")
}

#[cfg(target_os = "macos")]
impl Drop for PlatformPointerCapture {
    fn drop(&mut self) {
        // SAFETY: tokenはaddLocalMonitorの返値で、このownerが一度だけ除去する。
        unsafe { objc2_app_kit::NSEvent::removeMonitor(&self.click_monitor) };
    }
}

#[cfg(not(target_os = "macos"))]
#[derive(Debug, Default)]
pub(crate) struct PlatformPointerCapture {
    state: HostPointerCaptureState,
}

#[cfg(not(target_os = "macos"))]
impl PlatformPointerCapture {
    pub(crate) fn new(
        _window: &winit::window::Window,
        _wake: std::sync::Arc<dyn Fn() + Send + Sync>,
    ) -> Result<Self, PlatformPointerCaptureError> {
        Ok(Self::default())
    }

    pub(crate) fn arm(&mut self, generation: u64) -> bool {
        self.state.arm(generation, 0)
    }

    pub(crate) fn is_active(&self) -> bool {
        self.state.is_active()
    }

    pub(crate) fn poll(
        &mut self,
    ) -> Result<Option<HostPointerCandidate>, PlatformPointerCaptureError> {
        Ok(None)
    }

    pub(crate) fn poll_click(
        &mut self,
    ) -> Result<Option<HostPointerClick>, PlatformPointerCaptureError> {
        Ok(None)
    }

    pub(crate) fn set_host_commands_enabled(&self, _enabled: bool) {}

    pub(crate) fn poll_command(
        &mut self,
    ) -> Result<Option<EffectiveTrigger>, PlatformPointerCaptureError> {
        Ok(None)
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PlatformPointerCaptureError {
    #[error("native window handle is unavailable")]
    WindowHandle,
    #[error("native window handle is not AppKit")]
    WrongPlatform,
    #[error("native content view is unavailable")]
    MissingView,
    #[error("native window is unavailable")]
    MissingWindow,
    #[error("native local pointer event monitor is unavailable")]
    EventMonitor,
    #[error("native pointer up queue lock is poisoned")]
    UpQueuePoisoned,
    #[error("native command inbox lock is poisoned")]
    CommandInboxPoisoned,
}

fn position_changed(previous: Option<[f64; 2]>, current: [f64; 2]) -> bool {
    previous.is_none_or(|previous| {
        (previous[0] - current[0]).abs() > 0.01 || (previous[1] - current[1]).abs() > 0.01
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(left_button_down: bool, window_focused: bool) -> HostPointerSample {
        HostPointerSample {
            position: [120.0, 64.0],
            left_button_down,
            window_focused,
            escape_pressed: false,
        }
    }

    #[test]
    fn typed_start_arms_only_one_capture() {
        let mut capture = HostPointerCaptureState::default();
        let queue = PointerUpQueue::default();

        assert!(capture.arm(1, queue.high_water()));
        assert!(!capture.arm(2, queue.high_water()));
    }

    #[test]
    fn first_poll_after_tracking_loop_still_emits_release() {
        let mut capture = HostPointerCaptureState::default();
        let mut queue = PointerUpQueue::default();
        assert!(capture.arm(1, queue.high_water()));

        // なぜ: CU-108RDS §2-4 でarm後の最初のpollでup claimを優先するため。
        assert_eq!(queue.enqueue([311.0, 172.0]), Some(0));
        assert_eq!(
            capture.update(sample(true, true), &mut queue),
            Some(HostPointerCandidate::Released {
                generation: 1,
                position: [311.0, 172.0]
            })
        );
        assert_eq!(capture.update(sample(true, true), &mut queue), None);
        assert_eq!(queue.pop_unclaimed(), None);
    }

    #[test]
    fn unarmed_up_stays_an_ordinary_click() {
        let mut capture = HostPointerCaptureState::default();
        let mut queue = PointerUpQueue::default();

        assert_eq!(queue.enqueue([120.0, 64.0]), Some(0));
        assert_eq!(capture.update(sample(false, true), &mut queue), None);
        assert_eq!(
            queue.pop_unclaimed(),
            Some(PointerUpEntry {
                sequence: 0,
                position: [120.0, 64.0]
            })
        );
        assert_eq!(queue.pop_unclaimed(), None);
    }

    #[test]
    fn pre_arm_up_stays_while_post_arm_up_releases() {
        let mut capture = HostPointerCaptureState::default();
        let mut queue = PointerUpQueue::default();

        assert_eq!(queue.enqueue([10.0, 20.0]), Some(0));
        assert!(capture.arm(7, queue.high_water()));
        assert_eq!(queue.enqueue([11.0, 21.0]), Some(1));
        assert_eq!(
            capture.update(sample(true, true), &mut queue),
            Some(HostPointerCandidate::Released {
                generation: 7,
                position: [11.0, 21.0]
            })
        );
        assert_eq!(
            queue.pop_unclaimed(),
            Some(PointerUpEntry {
                sequence: 0,
                position: [10.0, 20.0]
            })
        );
        assert_eq!(queue.pop_unclaimed(), None);
    }

    #[test]
    fn up_after_claim_stays_an_ordinary_click() {
        let mut capture = HostPointerCaptureState::default();
        let mut queue = PointerUpQueue::default();

        assert!(capture.arm(1, queue.high_water()));
        assert_eq!(queue.enqueue([21.0, 31.0]), Some(0));
        assert_eq!(
            capture.update(sample(true, true), &mut queue),
            Some(HostPointerCandidate::Released {
                generation: 1,
                position: [21.0, 31.0]
            })
        );
        assert_eq!(queue.enqueue([22.0, 32.0]), Some(1));
        assert_eq!(
            queue.pop_unclaimed(),
            Some(PointerUpEntry {
                sequence: 1,
                position: [22.0, 32.0]
            })
        );
        assert_eq!(queue.pop_unclaimed(), None);
    }

    #[test]
    fn release_uses_the_enqueued_point_not_the_sample_point() {
        let mut capture = HostPointerCaptureState::default();
        let mut queue = PointerUpQueue::default();
        assert!(capture.arm(1, queue.high_water()));
        assert_eq!(queue.enqueue([400.0, 300.0]), Some(0));

        // なぜ: CU-108RDS §2-5 で物理UP座標を優先するため。
        assert_eq!(
            capture.update(sample(true, true), &mut queue),
            Some(HostPointerCandidate::Released {
                generation: 1,
                position: [400.0, 300.0]
            })
        );
        assert!(capture.update(sample(false, true), &mut queue).is_none());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn top_down_logical_flips_only_non_flipped_views() {
        assert_eq!(top_down_logical(11.0, 17.0, true, 111.0), [11.0, 17.0]);
        assert_eq!(top_down_logical(11.0, 17.0, false, 111.0), [11.0, 94.0]);
    }

    #[test]
    fn each_physical_up_reaches_exactly_one_consumer() {
        let mut capture = HostPointerCaptureState::default();
        let mut queue = PointerUpQueue::default();
        let points = [[10.0, 20.0], [11.0, 21.0], [12.0, 22.0]];

        assert_eq!(queue.enqueue(points[0]), Some(0));
        assert_eq!(queue.enqueue(points[1]), Some(1));
        assert!(capture.arm(7, queue.high_water()));
        assert_eq!(queue.enqueue(points[2]), Some(2));

        let mut released_positions = Vec::new();
        loop {
            match capture.update(sample(true, true), &mut queue) {
                Some(HostPointerCandidate::Released { position, .. }) => {
                    released_positions.push(position);
                }
                None => break,
                Some(
                    HostPointerCandidate::Moved { .. } | HostPointerCandidate::Cancelled { .. },
                ) => {}
            }
        }

        let mut drained_positions = Vec::new();
        while let Some(entry) = queue.pop_unclaimed() {
            drained_positions.push(entry.position);
        }

        for point in points {
            let released = released_positions
                .iter()
                .filter(|position| **position == point)
                .count();
            let drained = drained_positions
                .iter()
                .filter(|position| **position == point)
                .count();
            assert_eq!(
                released + drained,
                1,
                "point should reach exactly one consumer: {point:?}"
            );
        }
    }

    #[test]
    fn focus_loss_cancels_and_leaves_the_up_claimable_as_a_click() {
        let mut capture = HostPointerCaptureState::default();
        let mut queue = PointerUpQueue::default();
        assert!(capture.arm(1, queue.high_water()));

        assert_eq!(
            capture.update(sample(true, false), &mut queue),
            Some(HostPointerCandidate::Cancelled {
                generation: 1,
                reason: HostPointerCancel::CaptureLost
            })
        );

        // なぜ: CU-108RDS §2-3・4 でキャンセル時に物理UPを消費/重複配信しないため。
        assert_eq!(queue.enqueue([201.0, 202.0]), Some(0));
        assert_eq!(capture.update(sample(false, true), &mut queue), None);
        assert_eq!(
            queue.pop_unclaimed(),
            Some(PointerUpEntry {
                sequence: 0,
                position: [201.0, 202.0]
            })
        );
        assert_eq!(queue.pop_unclaimed(), None);
    }

    #[test]
    fn escape_cancels_and_leaves_the_up_claimable_as_a_click() {
        let mut capture = HostPointerCaptureState::default();
        let mut queue = PointerUpQueue::default();
        assert!(capture.arm(1, queue.high_water()));
        let mut escaped = sample(true, true);
        escaped.escape_pressed = true;

        assert_eq!(
            capture.update(escaped, &mut queue),
            Some(HostPointerCandidate::Cancelled {
                generation: 1,
                reason: HostPointerCancel::Escape
            })
        );

        // なぜ: CU-108RDS §2-3・4 でキャンセル時に物理UPを消費/重複配信しないため。
        assert_eq!(queue.enqueue([203.0, 204.0]), Some(0));
        assert_eq!(capture.update(sample(false, true), &mut queue), None);
        assert_eq!(
            queue.pop_unclaimed(),
            Some(PointerUpEntry {
                sequence: 0,
                position: [203.0, 204.0]
            })
        );
        assert_eq!(queue.pop_unclaimed(), None);
    }

    #[test]
    fn empty_queue_never_releases_and_keeps_capture_active() {
        let mut capture = HostPointerCaptureState::default();
        let mut queue = PointerUpQueue::default();
        assert!(capture.arm(1, queue.high_water()));

        assert_eq!(capture.update(sample(false, true), &mut queue), None);
        assert!(capture.is_active());
        assert_eq!(capture.update(sample(false, true), &mut queue), None);
        assert!(capture.is_active());
    }

    #[test]
    fn pre_arm_up_is_never_claimable_by_that_capture() {
        let mut capture = HostPointerCaptureState::default();
        let mut queue = PointerUpQueue::default();
        assert_eq!(queue.enqueue([101.0, 201.0]), Some(0));
        assert!(capture.arm(1, queue.high_water()));

        assert_eq!(capture.update(sample(false, true), &mut queue), None);
        assert_eq!(capture.update(sample(false, true), &mut queue), None);
        assert!(capture.is_active());
        assert_eq!(
            queue.pop_unclaimed(),
            Some(PointerUpEntry {
                sequence: 0,
                position: [101.0, 201.0]
            })
        );
    }

    #[test]
    fn claim_disarms_so_no_second_entry_is_claimed() {
        let mut capture = HostPointerCaptureState::default();
        let mut queue = PointerUpQueue::default();
        assert!(capture.arm(1, queue.high_water()));
        assert_eq!(queue.enqueue([30.0, 40.0]), Some(0));
        assert_eq!(queue.enqueue([31.0, 41.0]), Some(1));
        assert_eq!(
            capture.update(sample(true, true), &mut queue),
            Some(HostPointerCandidate::Released {
                generation: 1,
                position: [30.0, 40.0]
            })
        );
        assert_eq!(capture.update(sample(true, true), &mut queue), None);
        assert_eq!(
            queue.pop_unclaimed(),
            Some(PointerUpEntry {
                sequence: 1,
                position: [31.0, 41.0]
            })
        );
        assert_eq!(queue.pop_unclaimed(), None);
    }

    #[test]
    fn overflow_refuses_enqueue_without_wrapping() {
        let mut queue = PointerUpQueue {
            next_sequence: u64::MAX,
            ..PointerUpQueue::default()
        };

        assert_eq!(queue.enqueue([1.0, 2.0]), None);
        assert_eq!(queue.high_water(), u64::MAX);
        assert_eq!(queue.pop_unclaimed(), None);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn shifted_history_key_remains_the_same_physical_shortcut() {
        assert!(is_history_character("z"));
        assert!(is_history_character("Z"));
        assert!(!is_history_character("x"));
        assert!(!is_history_character("zz"));
    }

    #[test]
    fn pointer_trace_ignores_sub_hundredth_logical_point_noise() {
        assert!(position_changed(None, [10.0, 20.0]));
        assert!(!position_changed(Some([10.0, 20.0]), [10.009, 19.991]));
        assert!(position_changed(Some([10.0, 20.0]), [10.011, 20.0]));
    }
}
