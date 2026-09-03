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
