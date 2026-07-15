slint::include_modules!();

mod token_gen;
mod timeline;

use std::path::PathBuf;

pub use token_gen::*;
pub use timeline::*;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/theme.rs"));
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/apply_theme.rs"));

/// 解決済み token を Slint Theme global へ反映。
pub fn apply_theme_tokens(ui: &MainWindow, theme: &ThemeTokens) {
    apply_resolved(ui, theme);
}

#[derive(Debug, thiserror::Error)]
pub enum ThemeLoadError {
    #[error("token: {0}")]
    Token(#[from] TokenError),
    #[error("unknown theme: {0}")]
    Unknown(String),
    #[error("diagnostic: {0}")]
    Diagnostic(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub fn load_theme_by_id(manifest_dir: &std::path::Path, id: ThemeId) -> Result<ThemeTokens, ThemeLoadError> {
    let rel = TOKEN_FILES
        .iter()
        .find(|(name, _)| *name == id.as_str())
        .map(|(_, p)| *p)
        .ok_or_else(|| ThemeLoadError::Unknown(id.as_str().to_string()))?;
    let theme = load_theme(manifest_dir, id.as_str(), rel)?;
    check_text_contrast(&theme)?;
    Ok(theme)
}

pub fn load_theme_safe(
    manifest_dir: &std::path::Path,
    requested: ThemeId,
) -> (ThemeTokens, Option<String>) {
    match load_theme_by_id(manifest_dir, requested) {
        Ok(t) => (t, None),
        Err(e) => {
            let diag = format!("theme fallback to dark: {e}");
            match load_theme_by_id(manifest_dir, ThemeId::MotoliiDark) {
                Ok(t) => (t, Some(diag)),
                Err(e2) => panic!("built-in dark theme must load: {e2}"),
            }
        }
    }
}

pub fn save_theme_pref(path: &std::path::Path, id: ThemeId) -> Result<(), ThemeLoadError> {
    let data = serde_json::json!({ "theme": id.as_str() });
    std::fs::write(path, serde_json::to_string_pretty(&data).unwrap())?;
    Ok(())
}

pub fn load_theme_pref(path: &std::path::Path) -> Result<ThemeId, ThemeLoadError> {
    let raw = std::fs::read_to_string(path).map_err(|e| ThemeLoadError::Diagnostic(e.to_string()))?;
    let v: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| ThemeLoadError::Diagnostic(e.to_string()))?;
    let name = v
        .get("theme")
        .and_then(|t| t.as_str())
        .ok_or_else(|| ThemeLoadError::Diagnostic("missing theme field".into()))?;
    ThemeId::all()
        .iter()
        .copied()
        .find(|id| id.as_str() == name)
        .ok_or_else(|| ThemeLoadError::Unknown(name.to_string()))
}

pub fn manifest_dir() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"))
}

pub fn theme_id_from_index(idx: i32) -> ThemeId {
    match idx {
        1 => ThemeId::MotoliiLight,
        2 => ThemeId::CustomFixture,
        _ => ThemeId::MotoliiDark,
    }
}

pub fn index_from_theme_id(id: ThemeId) -> i32 {
    match id {
        ThemeId::MotoliiLight => 1,
        ThemeId::CustomFixture => 2,
        ThemeId::MotoliiDark => 0,
    }
}

pub fn set_locale(locale: &str) {
    let lang = match locale {
        "en" => "en",
        "pseudo" => "pseudo",
        _ => "ja",
    };
    let _ = slint::select_bundled_translation(lang);
}

/// ヘッドレス: timeline texture + Image::try_from 契約を検証。
pub fn structural_timeline_evidence(out_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use motolii_gpu::download_rgba;
    std::fs::create_dir_all(out_dir)?;
    let (gpu, _parts) = motolii_gpu::GpuCtx::new_for_ui()?;
    let theme = load_theme_by_id(&manifest_dir(), ThemeId::MotoliiDark)?;
    let texture = render_timeline_for_theme(&gpu, &theme)?;
    let _img = slint::Image::try_from(texture.clone())
        .map_err(|e| format!("Image::try_from failed: {e:?}"))?;
    let rgba = download_rgba(&gpu, &texture)?;
    let path = out_dir.join("timeline-texture.png");
    image::save_buffer(&path, &rgba, 960, 280, image::ColorType::Rgba8)?;
    let manifest = serde_json::json!({
        "ticket": "U0V",
        "image_try_from": "ok",
        "path": path.file_name().and_then(|s| s.to_str()),
    });
    std::fs::write(
        out_dir.join("u0v-struct-manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    Ok(())
}
