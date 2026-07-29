//! WebView境界を越えるpointer lifecycleをHost内へ閉じる。

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
}

#[cfg(target_os = "macos")]
pub(crate) struct PlatformPointerCapture {
    window: objc2::rc::Retained<objc2_app_kit::NSWindow>,
    state: HostPointerCaptureState,
    click_inbox: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<HostPointerClick>>>,
    click_monitor: objc2::rc::Retained<objc2::runtime::AnyObject>,
    armed_after_event_timestamp: f64,
}

#[cfg(target_os = "macos")]
impl PlatformPointerCapture {
    pub(crate) fn new(window: &winit::window::Window) -> Result<Self, PlatformPointerCaptureError> {
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
        let monitor_window = window.clone();
        let monitor =
            block2::RcBlock::new(move |event: std::ptr::NonNull<objc2_app_kit::NSEvent>| {
                // SAFETY: AppKitはlocal monitor呼び出し中のevent生存を保証する。
                let event = unsafe { event.as_ref() };
                if let Some(content) = monitor_window.contentView() {
                    let content_point =
                        content.convertPoint_fromView(event.locationInWindow(), None);
                    let content_height = content.bounds().size.height;
                    if let Ok(mut inbox) = monitor_inbox.lock() {
                        inbox.push_back(HostPointerClick {
                            position: [content_point.x, content_height - content_point.y],
                        });
                    }
                }
                event as *const objc2_app_kit::NSEvent as *mut objc2_app_kit::NSEvent
            });
        // SAFETY: blockは受け取ったlive eventをそのまま返し、monitor tokenはDropで除去する。
        let click_monitor = unsafe {
            objc2_app_kit::NSEvent::addLocalMonitorForEventsMatchingMask_handler(
                objc2_app_kit::NSEventMask::LeftMouseUp,
                &monitor,
            )
        }
        .ok_or(PlatformPointerCaptureError::EventMonitor)?;
        Ok(Self {
            window,
            state: HostPointerCaptureState::default(),
            click_inbox,
            click_monitor,
            armed_after_event_timestamp: 0.0,
        })
    }

    pub(crate) fn arm(&mut self, generation: u64) -> bool {
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
        let window_focused = self.window.isKeyWindow();
        let escape_pressed = window_focused
            && self.window.currentEvent().is_some_and(|event| {
                event.r#type() == NSEventType::KeyDown
                    && event.keyCode() == 53
                    && !event.isARepeat()
                    && event.timestamp() > self.armed_after_event_timestamp
            });
        let sample = HostPointerSample {
            position: [content_point.x, content_height - content_point.y],
            left_button_down: NSEvent::pressedMouseButtons() & 1 == 1,
            window_focused,
            escape_pressed,
        };
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
}
