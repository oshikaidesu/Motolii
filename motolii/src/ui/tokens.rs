
use std::sync::atomic::{AtomicU32, Ordering};

use dioxus_native::prelude::*;

pub(crate) const SURFACE_APP: [u8; 3] = [0x28, 0x28, 0x28];
pub(crate) const SURFACE_PANEL: [u8; 3] = [0x36, 0x36, 0x36];
pub(crate) const SURFACE_RAISED: [u8; 3] = [0x3e, 0x3e, 0x3e];
pub(crate) const SURFACE_HOVER: [u8; 3] = [0x46, 0x46, 0x46];
pub(crate) const LINE_DARK: [u8; 3] = [0x1a, 0x1a, 0x1a];
pub(crate) const BORDER: [u8; 3] = [0x55, 0x55, 0x55];
pub(crate) const INK: [u8; 3] = [0xb8, 0xb8, 0xb8];
pub(crate) const INK2: [u8; 3] = [0x8c, 0x8c, 0x8c];
pub(crate) const INK3: [u8; 3] = [0x75, 0x75, 0x75];
pub(crate) const ACCENT: [u8; 3] = [0xd8, 0xb5, 0x74];

pub(crate) const WAY_BROWSER: [u8; 3] = [0x6e, 0xb3, 0xae];
pub(crate) const WAY_STAGE: [u8; 3] = [0xbc, 0xa0, 0x72];
pub(crate) const WAY_INSPECTOR: [u8; 3] = [0x8e, 0xb0, 0x86];
pub(crate) const WAY_TIMELINE: [u8; 3] = [0xcc, 0x95, 0x87];

pub(crate) const TEXT_MICRO: f64 = 8.0;
pub(crate) const TEXT_DENSE: f64 = 9.0;
pub(crate) const TEXT_BASE: f64 = 11.0;
pub(crate) const TEXT_TITLE: f64 = 12.0;
pub(crate) const ROW: f64 = 20.0;
pub(crate) const SECTION: f64 = 26.0;
pub(crate) const SP1: f64 = 2.0;
pub(crate) const SP2: f64 = 4.0;
pub(crate) const SP3: f64 = 6.0;
pub(crate) const SP4: f64 = 8.0;
pub(crate) const HIT: f64 = 18.0;
pub(crate) const MOTION_DIRECT_MS: u32 = 0;
pub(crate) const MOTION_FAST_MS: u32 = 100;
pub(crate) const MOTION_STATE_MS: u32 = 150;
pub(crate) const MOTION_ENTER_MS: u32 = 200;
pub(crate) const MOTION_EASE_STANDARD: &str = "cubic-bezier(0.2, 0, 0, 1)";
pub(crate) const MOTION_EASE_ENTER: &str = "cubic-bezier(0, 0, 0, 1)";

pub(crate) fn hex(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

pub(crate) struct UiScale(AtomicU32);

impl UiScale {
    pub(crate) fn new(percent: u32) -> Self {
        Self(AtomicU32::new(percent))
    }

    pub(crate) fn percent(&self) -> u32 {
        self.0.load(Ordering::Relaxed)
    }

    pub(crate) fn set_percent(&self, percent: u32) {
        self.0.store(percent.clamp(50, 200), Ordering::Relaxed);
    }

    pub(crate) fn factor(&self) -> f64 {
        self.percent() as f64 / 100.0
    }
}

pub(crate) fn css_root(percent: u32, reduced_motion: bool) -> String {
    let motion_fast = if reduced_motion { 0 } else { MOTION_FAST_MS };
    let motion_state = if reduced_motion { 0 } else { MOTION_STATE_MS };
    let motion_enter = if reduced_motion { 0 } else { MOTION_ENTER_MS };
    // 鳥の呼吸とまばたきは純装飾なので Reduce Motion で止める(2026-08-08 §3)。
    let pet_play = if reduced_motion { "paused" } else { "running" };
    format!(
        ":root{{\
--s:{s:.2};\
--t-micro:calc({TEXT_MICRO} * var(--s) * 1px);\
--t-dense:calc({TEXT_DENSE} * var(--s) * 1px);\
--t-base:calc({TEXT_BASE} * var(--s) * 1px);\
--t-title:calc({TEXT_TITLE} * var(--s) * 1px);\
--row:calc({ROW} * var(--s) * 1px);\
--section:calc({SECTION} * var(--s) * 1px);\
--sp1:calc({SP1} * var(--s) * 1px);\
--sp2:calc({SP2} * var(--s) * 1px);\
--sp3:calc({SP3} * var(--s) * 1px);\
--sp4:calc({SP4} * var(--s) * 1px);\
--hit:calc({HIT} * var(--s) * 1px);\
--line:1px;\
--app:{app};--panel:{panel};--raised:{raised};--hover:{hover};\
--dark:{dark};--bd:{bd};\
--ink:{ink};--ink2:{ink2};--ink3:{ink3};--accent:{accent};\
--way-browser:{wb};--way-stage:{ws};--way-inspector:{wi};--way-timeline:{wt};\
--motion-direct:{motion_direct}ms;--motion-fast:{motion_fast}ms;--motion-state:{motion_state}ms;--motion-enter:{motion_enter}ms;\
--motion-ease-standard:{ease_standard};--motion-ease-enter:{ease_enter};\
--pet-play:{pet_play};\
}}",
        s = percent as f64 / 100.0,
        app = hex(SURFACE_APP),
        panel = hex(SURFACE_PANEL),
        raised = hex(SURFACE_RAISED),
        hover = hex(SURFACE_HOVER),
        dark = hex(LINE_DARK),
        bd = hex(BORDER),
        ink = hex(INK),
        ink2 = hex(INK2),
        ink3 = hex(INK3),
        accent = hex(ACCENT),
        wb = hex(WAY_BROWSER),
        ws = hex(WAY_STAGE),
        wi = hex(WAY_INSPECTOR),
        wt = hex(WAY_TIMELINE),
        motion_direct = MOTION_DIRECT_MS,
        ease_standard = MOTION_EASE_STANDARD,
        ease_enter = MOTION_EASE_ENTER,
    )
}

#[cfg(target_os = "macos")]
pub(crate) fn system_prefers_reduced_motion() -> bool {
    use objc2_app_kit::NSWorkspace;

    NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion()
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn system_prefers_reduced_motion() -> bool {
    false
}

#[cfg(not(test))]
static STYLES: Asset = asset!("/src/ui/styles.css");

#[cfg(test)]
static TEST_STYLES: &str = include_str!("styles.css");

/// 窓は asset を link で引く(dx の CSS hot reload が効く)。headless の harness には
/// 資産を配る net が無いので、同じ文面を inline で当てる。
#[cfg(not(test))]
pub(crate) fn stylesheet() -> Element {
    rsx!(link { rel: "stylesheet", href: STYLES })
}

#[cfg(test)]
pub(crate) fn stylesheet() -> Element {
    rsx!(style { {TEST_STYLES} })
}

#[cfg(test)]
#[test]
fn ui_motion_never_transitions_direct_manipulation_geometry() {
    assert!(!TEST_STYLES.contains("transition: all"));
    for property in ["left", "top", "width", "height", "transform"] {
        assert!(
            !TEST_STYLES.contains(&format!("transition-property: {property}")),
            "direct manipulation property entered the transition contract: {property}"
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn motion_tokens_follow_the_material_short_scale_and_reduce_to_zero() {
        let ordinary = super::css_root(100, false);
        assert!(ordinary.contains("--motion-direct:0ms"));
        assert!(ordinary.contains("--motion-fast:100ms"));
        assert!(ordinary.contains("--motion-state:150ms"));
        assert!(ordinary.contains("--motion-enter:200ms"));
        assert!(ordinary.contains("--motion-ease-standard:cubic-bezier(0.2, 0, 0, 1)"));

        let reduced = super::css_root(100, true);
        assert!(reduced.contains("--motion-fast:0ms"));
        assert!(reduced.contains("--motion-state:0ms"));
        assert!(reduced.contains("--motion-enter:0ms"));
    }
}

