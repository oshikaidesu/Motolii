//! 全パネル共通の目盛り。Tailwind と同じ考え方 — 値は数直線から選ぶものであって、
//! その場で書くものではない。`14` と `13` が並ぶ理由を後から誰も説明できないため。
//!
//! `scale` だけが可変で、寸法系は全部それに掛かる。UI 全体を 1% 刻みで拡縮できるのは
//! これが1箇所しか無いから。色は拡縮しないので `scale` を掛けない。
use std::sync::atomic::{AtomicU32, Ordering};

/// UI 全体の拡縮(%)。100 が等倍。`script_mod!` の式へ焼き込まれるので、
/// 変更後は `cx.request_live_edit()` で再評価させる(makepad が iOS の safe-area
/// inset に使っているのと同じ経路)。
static UI_SCALE_PERCENT: AtomicU32 = AtomicU32::new(100);

pub const UI_SCALE_MIN: i32 = 50;
pub const UI_SCALE_MAX: i32 = 300;

pub fn ui_scale_percent() -> i32 {
    UI_SCALE_PERCENT.load(Ordering::Relaxed) as i32
}

pub fn set_ui_scale_percent(percent: i32) -> i32 {
    let percent = percent.clamp(UI_SCALE_MIN, UI_SCALE_MAX);
    UI_SCALE_PERCENT.store(percent as u32, Ordering::Relaxed);
    percent
}

pub fn ui_scale() -> f64 {
    ui_scale_percent() as f64 / 100.0
}

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*

    let s = #(crate::tokens::ui_scale())

    // 余白 — 2px 刻み。名前は Tailwind と同じく「いくつ分か」で読む
    let space = {
        s1: 2.0 * s
        s2: 4.0 * s
        s3: 6.0 * s
        s4: 8.0 * s
        s5: 10.0 * s
        s6: 12.0 * s
        s8: 16.0 * s
        s10: 20.0 * s
        s12: 24.0 * s
    }

    // 字の大きさ — 役割で呼ぶ。数字を直接書かない
    let text = {
        xs: 7.5 * s
        sm: 8.0 * s
        base: 8.75 * s
        md: 9.0 * s
        lg: 10.0 * s
        xl: 11.0 * s
    }

    // 寸法 — 行の高さと帯の高さ。ここが UI の密度そのもの
    let size = {
        row: 21.0 * s
        row_tight: 18.0 * s
        bar: 22.0 * s
        toolbar: 26.0 * s
        cap: 20.0 * s
        chip: 16.0 * s
        field: 17.0 * s
        icon_sm: 11.0 * s
        icon: 12.0 * s
        icon_lg: 17.0 * s
        rail: 132.0 * s
        pane: 300.0 * s
        chrome: 32.0 * s
        status: 20.0 * s
        menu: 24.0 * s
        transport: 24.0 * s
    }

    // 境の線。太さは1つしか無い。色は「何の境か」で分ける —
    // 線は操作の持ち主か座標系が変わる所にだけ引く
    let rule = {
        size: 1.0
        owner: #x434343
        seam: #x2d2d2d
        pane: #x343434
        surface: #x1d1d1d
    }

    // 面 — 明るさの段。暗い方が奥
    let face = {
        sunk: #x282828
        bar: #x3d3d3d
        panel: #x4f4f4f
        head: #x606060
        hover: #x5c5c5c
        down: #x2d2d2d
        pressed: #x444444
    }

    // 字と記号
    let ink = {
        strong: #xe4e4e4
        body: #xd8d8d8
        muted: #x9d9d9d
        faint: #x757575
        glyph: #xa0a0a0
    }

    mod.tokens = {
        scale: s
        space: space
        text: text
        size: size
        rule: rule
        face: face
        ink: ink
    }
}
