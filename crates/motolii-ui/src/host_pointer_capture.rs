//! WebView境界を越えるpointer lifecycleをHost内へ閉じる。

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HostPointerSample {
    pub(crate) position: [f64; 2],
    pub(crate) left_button_down: bool,
    pub(crate) window_focused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HostPointerCancel {
    WindowFocusLost,
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

#[derive(Debug, Default)]
pub(crate) struct HostPointerCaptureState {
    active: Option<ActiveCapture>,
    next_generation: u64,
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

    pub(crate) fn arm(&mut self) -> bool {
        if self.active.is_some() {
            return false;
        }
        self.next_generation = self.next_generation.wrapping_add(1);
        self.active = Some(ActiveCapture {
            generation: self.next_generation,
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
                reason: HostPointerCancel::WindowFocusLost,
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
        Ok(Self {
            window,
            state: HostPointerCaptureState::default(),
        })
    }

    pub(crate) fn arm(&mut self) -> bool {
        self.state.arm()
    }

    pub(crate) fn is_active(&self) -> bool {
        self.state.is_active()
    }

    pub(crate) fn poll(
        &mut self,
    ) -> Result<Option<HostPointerCandidate>, PlatformPointerCaptureError> {
        use objc2_app_kit::NSEvent;

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
        let sample = HostPointerSample {
            position: [content_point.x, content_height - content_point.y],
            left_button_down: NSEvent::pressedMouseButtons() & 1 == 1,
            window_focused: self.window.isKeyWindow(),
        };
        Ok(self.state.update(sample))
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

    pub(crate) fn arm(&mut self) -> bool {
        self.state.arm()
    }

    pub(crate) fn is_active(&self) -> bool {
        self.state.is_active()
    }

    pub(crate) fn poll(
        &mut self,
    ) -> Result<Option<HostPointerCandidate>, PlatformPointerCaptureError> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(left_button_down: bool, window_focused: bool) -> HostPointerSample {
        HostPointerSample {
            position: [120.0, 64.0],
            left_button_down,
            window_focused,
        }
    }

    #[test]
    fn typed_start_arms_only_one_capture() {
        let mut capture = HostPointerCaptureState::default();

        assert!(capture.arm());
        assert!(!capture.arm());
    }

    #[test]
    fn first_poll_after_tracking_loop_still_emits_release() {
        let mut capture = HostPointerCaptureState::default();
        assert!(capture.arm());

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
        assert!(capture.arm());

        assert_eq!(
            capture.update(sample(true, false)),
            Some(HostPointerCandidate::Cancelled {
                generation: 1,
                reason: HostPointerCancel::WindowFocusLost
            })
        );
        assert_eq!(capture.update(sample(false, true)), None);
    }

    #[test]
    fn release_uses_last_position_observed_while_pressed() {
        let mut capture = HostPointerCaptureState::default();
        assert!(capture.arm());
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
