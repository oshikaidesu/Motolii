//! 窓の枠の記憶(前回の続きから)。host.rs が 800 行を越えたので分けた。
use crate::ui::host::settings_dir;

/// 窓の枠(論理 px)。~/Library/Application Support/Motolii/window.json。
#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub(crate) struct WindowFrame {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) w: u32,
    pub(crate) h: u32,
}

pub(crate) fn load_window_frame() -> Option<WindowFrame> {
    let file = settings_dir()?.join("window.json");
    let frame: WindowFrame = serde_json::from_str(&std::fs::read_to_string(file).ok()?).ok()?;
    (frame.w >= 400 && frame.h >= 300).then_some(frame)
}

/// 画面のどれにも掛からない枠は位置を捨てる(外部 display を外した後に窓が見えない所へ出ない)。
/// `monitors` は論理 px の (x, y, w, h)。空(まだ分からない)なら触らない。
pub(crate) fn place_on_screen(frame: WindowFrame, monitors: &[(i32, i32, u32, u32)]) -> Option<(i32, i32)> {
    if monitors.is_empty() {
        return Some((frame.x, frame.y));
    }
    let visible = monitors.iter().any(|&(mx, my, mw, mh)| {
        let (r, b) = (mx + mw as i32, my + mh as i32);
        frame.x + 64 < r && frame.x + frame.w as i32 > mx + 64 && frame.y + 32 < b && frame.y > my - 32
    });
    visible.then_some((frame.x, frame.y))
}

/// 憶えた位置がどの画面にも掛からない(外した display)なら、既定の位置へ戻す。
pub(crate) fn nudge_onto_screen(window: &dyn dioxus_native::winit::window::Window, event_loop: &dyn dioxus_native::winit::event_loop::ActiveEventLoop) {
    let monitors: Vec<(i32, i32, u32, u32)> = event_loop
        .available_monitors()
        .map(|m| {
            let (p, s, k) = (m.position().unwrap_or_default(), m.current_video_mode().map(|v| v.size()).unwrap_or_default(), m.scale_factor().max(0.5));
            ((p.x as f64 / k) as i32, (p.y as f64 / k) as i32, (s.width as f64 / k) as u32, (s.height as f64 / k) as u32)
        })
        .collect();
    if load_window_frame().is_some_and(|f| place_on_screen(f, &monitors).is_none()) {
        window.set_outer_position(dioxus_native::winit::dpi::LogicalPosition::new(40.0, 40.0).into());
    }
}

pub(crate) fn save_window_frame(window: &dyn dioxus_native::winit::window::Window) {
    let Some(dir) = settings_dir() else { return };
    let scale = window.scale_factor().max(0.5);
    let size = window.surface_size();
    let pos = window.outer_position().unwrap_or_default();
    let frame = WindowFrame {
        x: (pos.x as f64 / scale) as i32,
        y: (pos.y as f64 / scale) as i32,
        w: (size.width as f64 / scale) as u32,
        h: (size.height as f64 / scale) as u32,
    };
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string(&frame) {
        let _ = std::fs::write(dir.join("window.json"), json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_off_every_display_loses_its_position() {
        let main = [(0, 0, 1440, 900)];
        let f = WindowFrame { x: 100, y: 50, w: 1200, h: 800 };
        assert_eq!(place_on_screen(f, &main), Some((100, 50)));
        let gone = WindowFrame { x: 2000, y: 50, w: 1200, h: 800 };
        assert_eq!(place_on_screen(gone, &main), None, "an unplugged external display must not hide the window");
        assert_eq!(place_on_screen(gone, &[]), Some((2000, 50)), "unknown displays: leave it");
        let json = serde_json::to_string(&f).unwrap();
        let back: WindowFrame = serde_json::from_str(&json).unwrap();
        assert_eq!((back.x, back.y, back.w, back.h), (100, 50, 1200, 800));
    }
}
