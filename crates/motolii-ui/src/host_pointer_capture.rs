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

pub(crate) fn appkit_content_position(
    point: [f64; 2],
    content_height: f64,
    content_is_flipped: bool,
) -> [f64; 2] {
    [
        point[0],
        if content_is_flipped {
            point[1]
        } else {
            content_height - point[1]
        },
    ]
}

#[derive(Debug, Default)]
pub(crate) struct HostPointerCaptureState {
    active: Option<ActiveCapture>,
}

#[derive(Debug)]
struct ActiveCapture {
    generation: u64,
    last_pressed_position: Option<[f64; 2]>,
}

impl HostPointerCaptureState {
    pub(crate) fn is_active(&self) -> bool {
        self.active.is_some()
    }

    fn active_generation(&self) -> Option<u64> {
        self.active.as_ref().map(|active| active.generation)
    }

    pub(crate) fn arm(&mut self, generation: u64) -> bool {
        if self.active.is_some() {
            return false;
        }
        self.active = Some(ActiveCapture {
            generation,
            last_pressed_position: None,
        });
        true
    }

    pub(crate) fn update(&mut self, sample: HostPointerSample) -> Option<HostPointerCandidate> {
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
        if sample.left_button_down {
            active.last_pressed_position = Some(sample.position);
            return Some(HostPointerCandidate::Moved {
                generation,
                position: sample.position,
            });
        }

        // typed startが到着した時点でbutton-downは成立済みなので、最初のpollが
        // tracking loop終了後でもreleaseを失わない。
        let position = active.last_pressed_position.unwrap_or(sample.position);
        self.active = None;
        Some(HostPointerCandidate::Released {
            generation,
            position,
        })
    }

    fn release_at(
        &mut self,
        position: [f64; 2],
        window_focused: bool,
    ) -> Option<HostPointerCandidate> {
        let active = self.active.take()?;
        if !window_focused {
            return Some(HostPointerCandidate::Cancelled {
                generation: active.generation,
                reason: HostPointerCancel::CaptureLost,
            });
        }
        Some(HostPointerCandidate::Released {
            generation: active.generation,
            position,
        })
    }
}

#[cfg(target_os = "macos")]
pub(crate) struct PlatformPointerCapture {
    window: objc2::rc::Retained<objc2_app_kit::NSWindow>,
    state: HostPointerCaptureState,
    click_inbox: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<HostPointerClick>>>,
    release_position: std::sync::Arc<std::sync::Mutex<Option<[f64; 2]>>>,
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
        let click_inbox =
            std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new()));
        let monitor_inbox = std::sync::Arc::clone(&click_inbox);
        let release_position = std::sync::Arc::new(std::sync::Mutex::new(None));
        let monitor_release_position = std::sync::Arc::clone(&release_position);
        let command_inbox =
            std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new()));
        let monitor_command_inbox = std::sync::Arc::clone(&command_inbox);
        let host_commands_enabled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let monitor_commands_enabled = std::sync::Arc::clone(&host_commands_enabled);
        let monitor_window = window.clone();
        let monitor = block2::RcBlock::new(
            move |event: std::ptr::NonNull<objc2_app_kit::NSEvent>| {
                // SAFETY: AppKitはlocal monitor呼び出し中のevent生存を保証する。
                let event = unsafe { event.as_ref() };
                if event.r#type() == objc2_app_kit::NSEventType::LeftMouseUp {
                    if let Some(content) = monitor_window.contentView() {
                        let content_point =
                            content.convertPoint_fromView(event.locationInWindow(), None);
                        let content_height = content.bounds().size.height;
                        let position = appkit_content_position(
                            [content_point.x, content_point.y],
                            content_height,
                            content.isFlipped(),
                        );
                        crate::ui_numeric_trace::emit(format_args!(
                            "kind=appkit-release event_timestamp={:.6} raw_x={:.3} raw_y={:.3} \
                             content_height={:.3} content_is_flipped={} logical_x={:.3} logical_y={:.3}",
                            event.timestamp(),
                            content_point.x,
                            content_point.y,
                            content_height,
                            content.isFlipped(),
                            position[0],
                            position[1],
                        ));
                        if let Ok(mut release_position) = monitor_release_position.lock() {
                            *release_position = Some(position);
                        }
                        if let Ok(mut inbox) = monitor_inbox.lock() {
                            inbox.push_back(HostPointerClick { position });
                            wake();
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
            },
        );
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
            click_inbox,
            release_position,
            command_inbox,
            host_commands_enabled,
            click_monitor,
            armed_after_event_timestamp: 0.0,
            last_logged_position: None,
        })
    }

    pub(crate) fn arm(&mut self, generation: u64) -> bool {
        let Ok(mut release_position) = self.release_position.lock() else {
            return false;
        };
        *release_position = None;
        self.last_logged_position = None;
        if !self.state.arm(generation) {
            return false;
        }
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
        let position = appkit_content_position(
            [content_point.x, content_point.y],
            content_height,
            content.isFlipped(),
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
        let exact_release = self
            .release_position
            .lock()
            .map_err(|_| PlatformPointerCaptureError::ReleasePositionPoisoned)?
            .take();
        if let Some(position) = exact_release {
            return Ok(self.state.release_at(position, window_focused));
        }
        let escape_pressed = window_focused
            && self.window.currentEvent().is_some_and(|event| {
                event.r#type() == NSEventType::KeyDown
                    && event.keyCode() == 53
                    && !event.isARepeat()
                    && event.timestamp() > self.armed_after_event_timestamp
            });
        let sample = HostPointerSample {
            position,
            left_button_down: NSEvent::pressedMouseButtons() & 1 == 1,
            window_focused,
            escape_pressed,
        };
        // WebKit tracking中はglobal button stateがlocal LeftMouseUpより先に変わるため、
        // 実event以外をcommit終端にしない。
        if !sample.left_button_down && sample.window_focused && !sample.escape_pressed {
            return Ok(None);
        }
        Ok(self.state.update(sample))
    }

    pub(crate) fn poll_click(
        &mut self,
    ) -> Result<Option<HostPointerClick>, PlatformPointerCaptureError> {
        self.click_inbox
            .lock()
            .map_err(|_| PlatformPointerCaptureError::ClickInboxPoisoned)
            .map(|mut inbox| inbox.pop_front())
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
        self.state.arm(generation)
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
    #[error("native pointer click inbox lock is poisoned")]
    ClickInboxPoisoned,
    #[error("native pointer release position lock is poisoned")]
    ReleasePositionPoisoned,
    #[error("native command inbox lock is poisoned")]
    CommandInboxPoisoned,
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

        assert!(capture.arm(1));
        assert!(!capture.arm(2));
    }

    #[test]
    fn appkit_points_are_normalized_to_top_down_exactly_once() {
        assert_eq!(
            appkit_content_position([40.0, 25.0], 200.0, true),
            [40.0, 25.0]
        );
        assert_eq!(
            appkit_content_position([40.0, 175.0], 200.0, false),
            [40.0, 25.0]
        );
    }

    #[test]
    fn exact_appkit_release_overrides_a_stale_pressed_sample() {
        let mut capture = HostPointerCaptureState::default();
        assert!(capture.arm(7));
        assert_eq!(
            capture.update(HostPointerSample {
                position: [300.0, 180.0],
                left_button_down: true,
                window_focused: true,
                escape_pressed: false,
            }),
            Some(HostPointerCandidate::Moved {
                generation: 7,
                position: [300.0, 180.0],
            })
        );

        assert_eq!(
            capture.release_at([650.0, 300.0], true),
            Some(HostPointerCandidate::Released {
                generation: 7,
                position: [650.0, 300.0],
            })
        );
        assert!(!capture.is_active());
    }

    #[test]
    fn first_poll_after_tracking_loop_still_emits_release() {
        let mut capture = HostPointerCaptureState::default();
        assert!(capture.arm(1));

        assert_eq!(
            capture.update(sample(false, true)),
            Some(HostPointerCandidate::Released {
                generation: 1,
                position: [120.0, 64.0]
            })
        );
        assert_eq!(capture.update(sample(false, true)), None);
    }

    #[test]
    fn focus_loss_cancels_without_release() {
        let mut capture = HostPointerCaptureState::default();
        assert!(capture.arm(1));

        assert_eq!(
            capture.update(sample(true, false)),
            Some(HostPointerCandidate::Cancelled {
                generation: 1,
                reason: HostPointerCancel::CaptureLost
            })
        );
        assert_eq!(capture.update(sample(false, true)), None);
    }

    #[test]
    fn escape_cancels_active_capture_once() {
        let mut capture = HostPointerCaptureState::default();
        assert!(capture.arm(1));
        let mut escaped = sample(true, true);
        escaped.escape_pressed = true;

        assert_eq!(
            capture.update(escaped),
            Some(HostPointerCandidate::Cancelled {
                generation: 1,
                reason: HostPointerCancel::Escape
            })
        );
        assert_eq!(capture.update(escaped), None);
    }

    #[test]
    fn release_uses_last_position_observed_while_pressed() {
        let mut capture = HostPointerCaptureState::default();
        assert!(capture.arm(1));
        assert_eq!(
            capture.update(sample(true, true)),
            Some(HostPointerCandidate::Moved {
                generation: 1,
                position: [120.0, 64.0]
            })
        );

        let mut released = sample(false, true);
        released.position = [400.0, 300.0];
        assert_eq!(
            capture.update(released),
            Some(HostPointerCandidate::Released {
                generation: 1,
                position: [120.0, 64.0]
            })
        );
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

fn position_changed(previous: Option<[f64; 2]>, current: [f64; 2]) -> bool {
    previous.is_none_or(|previous| {
        (previous[0] - current[0]).abs() > 0.01 || (previous[1] - current[1]).abs() > 0.01
    })
}
