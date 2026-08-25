use makepad_widgets::makepad_platform::event::ScrollPhase;
use makepad_widgets::{FingerGestureEvent, FingerScrollEvent, KeyModifiers};

/// Framework-neutral facts observed during a gesture. This layer never names
/// editor verbs such as zoom, scrub, trim, or pan.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum GesturePhase {
    #[default]
    Instant,
    Begin,
    Update,
    End,
    Momentum,
    MomentumEnd,
    Catch,
    Cancel,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum GestureDevice {
    Wheel,
    Trackpad,
    Touchscreen,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GestureModifiers {
    pub(crate) shift: bool,
    pub(crate) control: bool,
    pub(crate) alt: bool,
    pub(crate) logo: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GestureSample {
    pub(crate) phase: GesturePhase,
    pub(crate) device: GestureDevice,
    pub(crate) centroid: [f64; 2],
    /// Delta since the previous sample, in logical pixels.
    pub(crate) translation: [f64; 2],
    /// Multiplicative delta. `1.0` is neutral.
    pub(crate) scale_ratio: f64,
    /// Delta since the previous sample, in radians. `0.0` is neutral.
    pub(crate) rotation_radians: f64,
    pub(crate) modifiers: GestureModifiers,
}

impl GestureSample {
    pub(crate) fn from_makepad_scroll(event: &FingerScrollEvent) -> Self {
        Self {
            phase: match event.phase {
                ScrollPhase::None => GesturePhase::Instant,
                ScrollPhase::Began => GesturePhase::Begin,
                ScrollPhase::Touched => GesturePhase::Catch,
                ScrollPhase::Changed => GesturePhase::Update,
                ScrollPhase::Ended => GesturePhase::End,
                ScrollPhase::Momentum => GesturePhase::Momentum,
                ScrollPhase::MomentumEnded => GesturePhase::MomentumEnd,
            },
            device: if event.phase == ScrollPhase::None {
                GestureDevice::Wheel
            } else {
                GestureDevice::Trackpad
            },
            centroid: [event.abs.x, event.abs.y],
            translation: [event.scroll.x, event.scroll.y],
            scale_ratio: 1.0,
            rotation_radians: 0.0,
            modifiers: GestureModifiers::from(event.modifiers),
        }
    }

    pub(crate) fn from_makepad_gesture(event: &FingerGestureEvent) -> Self {
        use makepad_widgets::makepad_platform::event::{
            GestureDevice as MakepadGestureDevice, GesturePhase as MakepadGesturePhase,
        };

        Self {
            phase: match event.phase {
                MakepadGesturePhase::Began => GesturePhase::Begin,
                MakepadGesturePhase::Changed => GesturePhase::Update,
                MakepadGesturePhase::Ended => GesturePhase::End,
                MakepadGesturePhase::Cancelled => GesturePhase::Cancel,
            },
            device: match event.device {
                MakepadGestureDevice::Trackpad => GestureDevice::Trackpad,
                MakepadGestureDevice::Touchscreen => GestureDevice::Touchscreen,
                MakepadGestureDevice::Unknown => GestureDevice::Unknown,
            },
            centroid: [event.abs.x, event.abs.y],
            translation: [event.translation.x, event.translation.y],
            scale_ratio: event.scale,
            rotation_radians: event.rotation,
            modifiers: GestureModifiers::from(event.modifiers),
        }
    }
}

impl From<KeyModifiers> for GestureModifiers {
    fn from(value: KeyModifiers) -> Self {
        Self {
            shift: value.shift,
            control: value.control,
            alt: value.alt,
            logo: value.logo,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_transform_has_no_consumer_semantics() {
        let sample = GestureSample {
            phase: GesturePhase::Begin,
            device: GestureDevice::Touchscreen,
            centroid: [320.0, 180.0],
            translation: [0.0, 0.0],
            scale_ratio: 1.0,
            rotation_radians: 0.0,
            modifiers: GestureModifiers::default(),
        };
        assert_eq!(sample.translation, [0.0, 0.0]);
        assert_eq!(sample.scale_ratio, 1.0);
        assert_eq!(sample.rotation_radians, 0.0);
    }
}
