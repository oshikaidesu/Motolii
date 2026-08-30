
use std::sync::atomic::{AtomicU32, Ordering};

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

pub(crate) fn css_root(percent: u32) -> String {
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
    )
}

/// ラベル配色の枚数。DTCG 正本(`next/ui/motolii-tokens-rs`)由来の唯一の生きた値。
pub(crate) const LABEL_PALETTE_LEN: usize = 12;
