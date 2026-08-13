//! Document外の user keymap override。表の owner は keymap、JSON は keymap_codec。

use std::path::{Path, PathBuf};

use crate::{decode_keymap_json, BuiltinKeymap, KeymapCodecLimits, KeymapDelta};

pub const USER_KEYMAP_FILE_NAME: &str = "keymap.json";

pub fn default_user_keymap_override_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library/Application Support/Motolii")
            .join(USER_KEYMAP_FILE_NAME),
    )
}

/// ファイルが無い・読めない・適用できないときは空delta（Ableton default のまま）。
pub fn load_user_keymap_override(path: Option<&Path>, base: &BuiltinKeymap) -> KeymapDelta {
    let Some(path) = path else {
        return KeymapDelta::default();
    };
    let Ok(bytes) = std::fs::read(path) else {
        return KeymapDelta::default();
    };
    let Ok(loaded) = decode_keymap_json(&bytes, KeymapCodecLimits::new(64 * 1024, 16, 128, 1024))
    else {
        return KeymapDelta::default();
    };
    loaded.to_resolver_delta(base).unwrap_or_default()
}
