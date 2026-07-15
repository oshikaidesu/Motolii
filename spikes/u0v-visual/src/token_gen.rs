//! DTCG token 読み込み・検証・Rust/Slint 生成。

use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const TOKEN_FILES: &[(&str, &str)] = &[
    ("motolii-dark", "tokens/motolii-dark.json"),
    ("motolii-light", "tokens/motolii-light.json"),
    ("custom-fixture", "tokens/custom-fixture.json"),
];

pub const REQUIRED_REGIONS: &[&str] = &[
    "asset",
    "preview",
    "property",
    "timeline",
    "transport",
    "context",
];

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("schema: {0}")]
    Schema(String),
    #[error("theme mismatch: {0}")]
    ThemeMismatch(String),
}

#[derive(Debug, Clone)]
pub struct ResolvedColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

#[derive(Debug, Clone)]
pub enum ResolvedToken {
    Color(ResolvedColor),
    Dimension(f32),
}

#[derive(Debug, Clone)]
pub struct ThemeTokens {
    pub id: String,
    pub tokens: BTreeMap<String, ResolvedToken>,
}

#[derive(Deserialize)]
struct DtcgColorValue {
    #[serde(rename = "colorSpace")]
    color_space: String,
    components: [f32; 3],
    alpha: f32,
}

#[derive(Deserialize)]
struct DtcgDimensionValue {
    value: f32,
    unit: String,
}

pub fn load_theme(manifest_dir: &Path, id: &str, rel_path: &str) -> Result<ThemeTokens, TokenError> {
    let path = manifest_dir.join(rel_path);
    let raw = fs::read_to_string(&path)?;
    let root: Value = serde_json::from_str(&raw)?;
    validate_schema(&root)?;
    let mut tokens = BTreeMap::new();
    flatten_tokens(&root, String::new(), &mut tokens)?;
    Ok(ThemeTokens {
        id: id.to_string(),
        tokens,
    })
}

pub fn load_all_themes(manifest_dir: &Path) -> Result<Vec<ThemeTokens>, TokenError> {
    let themes: Result<Vec<_>, _> = TOKEN_FILES
        .iter()
        .map(|(id, path)| load_theme(manifest_dir, id, path))
        .collect();
    let themes = themes?;
    ensure_same_keys(&themes)?;
    Ok(themes)
}

fn validate_schema(root: &Value) -> Result<(), TokenError> {
    let schema = root
        .get("$schema")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TokenError::Schema("missing $schema".into()))?;
    if !schema.contains("2025.10") {
        return Err(TokenError::Schema(format!("unexpected schema: {schema}")));
    }
    if root.get("color").is_none() && root.get("dimension").is_none() {
        return Err(TokenError::Schema("no color or dimension groups".into()));
    }
    let mut probe = BTreeMap::new();
    flatten_tokens(root, String::new(), &mut probe)?;
    if probe.is_empty() {
        return Err(TokenError::Schema("no resolvable tokens".into()));
    }
    Ok(())
}

fn flatten_tokens(
    value: &Value,
    prefix: String,
    out: &mut BTreeMap<String, ResolvedToken>,
) -> Result<(), TokenError> {
    let Some(obj) = value.as_object() else {
        return Ok(());
    };
    for (key, child) in obj {
        if key.starts_with('$') {
            continue;
        }
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        if let Some(token_type) = child.get("$type").and_then(|v| v.as_str()) {
            match token_type {
                "color" => {
                    let cv: DtcgColorValue = serde_json::from_value(
                        child
                            .get("$value")
                            .cloned()
                            .ok_or_else(|| TokenError::Schema(format!("{path}: missing $value")))?,
                    )
                    .map_err(|e| TokenError::Schema(format!("{path}: {e}")))?;
                    if cv.color_space != "srgb" {
                        return Err(TokenError::Schema(format!(
                            "{path}: only srgb supported, got {}",
                            cv.color_space
                        )));
                    }
                    let r = (cv.components[0].clamp(0.0, 1.0) * 255.0).round() as u8;
                    let g = (cv.components[1].clamp(0.0, 1.0) * 255.0).round() as u8;
                    let b = (cv.components[2].clamp(0.0, 1.0) * 255.0).round() as u8;
                    out.insert(
                        path,
                        ResolvedToken::Color(ResolvedColor {
                            r,
                            g,
                            b,
                            a: cv.alpha.clamp(0.0, 1.0),
                        }),
                    );
                }
                "dimension" => {
                    let dv: DtcgDimensionValue = serde_json::from_value(
                        child
                            .get("$value")
                            .cloned()
                            .ok_or_else(|| TokenError::Schema(format!("{path}: missing $value")))?,
                    )
                    .map_err(|e| TokenError::Schema(format!("{path}: {e}")))?;
                    if dv.unit != "px" {
                        return Err(TokenError::Schema(format!(
                            "{path}: only px supported, got {}",
                            dv.unit
                        )));
                    }
                    out.insert(path, ResolvedToken::Dimension(dv.value));
                }
                other => {
                    return Err(TokenError::Schema(format!("{path}: unsupported $type {other}")));
                }
            }
        } else if child.is_object() {
            flatten_tokens(child, path, out)?;
        }
    }
    Ok(())
}

fn ensure_same_keys(themes: &[ThemeTokens]) -> Result<(), TokenError> {
    if themes.is_empty() {
        return Ok(());
    }
    let base = &themes[0].tokens;
    for theme in &themes[1..] {
        if theme.tokens.keys().collect::<Vec<_>>() != base.keys().collect::<Vec<_>>() {
            return Err(TokenError::ThemeMismatch(format!(
                "{} keys differ from {}",
                theme.id, themes[0].id
            )));
        }
    }
    Ok(())
}

pub fn color_to_hex(c: &ResolvedColor) -> String {
    if (c.a - 1.0).abs() < f32::EPSILON {
        format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
    } else {
        let a = (c.a.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}{:02x}", c.r, c.g, c.b, a)
    }
}

pub fn slint_prop_name(token_path: &str) -> String {
    token_path.replace('.', "-")
}

fn rust_field_name(token_path: &str) -> String {
    token_path.replace('.', "_").replace('-', "_")
}

pub fn generate_slint_globals(theme: &ThemeTokens) -> String {
    let mut lines = vec![
        "// AUTO-GENERATED — edit tokens/*.json and rebuild".to_string(),
        "export global Theme {".to_string(),
    ];
    for (path, token) in &theme.tokens {
        let name = slint_prop_name(path);
        match token {
            ResolvedToken::Color(c) => {
                lines.push(format!("    in-out property <brush> {name}: {};", color_to_hex(c)));
            }
            ResolvedToken::Dimension(px) => {
                lines.push(format!("    in-out property <length> {name}: {px}px;"));
            }
        }
    }
    lines.push("}".to_string());
    lines.join("\n")
}

pub fn generate_apply_theme(theme: &ThemeTokens) -> String {
    let mut lines = vec![
        "// AUTO-GENERATED".to_string(),
        "pub fn apply_resolved(ui: &crate::MainWindow, theme: &crate::token_gen::ThemeTokens) {".to_string(),
        "    let g = crate::Theme::get(ui);".to_string(),
        "    let _ = theme;".to_string(),
    ];
    for (path, token) in &theme.tokens {
        let setter = format!("set_{}", rust_field_name(path));
        match token {
            ResolvedToken::Color(c) => {
                if (c.a - 1.0).abs() < f32::EPSILON {
                    lines.push(format!(
                        "    g.{setter}(slint::Brush::from(slint::Color::from_rgb_u8({}, {}, {})));",
                        c.r, c.g, c.b
                    ));
                } else {
                    let a = (c.a * 255.0).round() as u8;
                    lines.push(format!(
                        "    g.{setter}(slint::Brush::from(slint::Color::from_argb_u8({a}, {}, {}, {})));",
                        c.r, c.g, c.b
                    ));
                }
            }
            ResolvedToken::Dimension(px) => {
                lines.push(format!("    g.{setter}({px:.1});"));
            }
        }
    }
    lines.push("}".to_string());
    lines.join("\n")
}

fn to_variant(id: &str) -> String {
    id.split('-')
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

pub fn generate_rust_theme(themes: &[ThemeTokens]) -> String {
    let mut lines = vec![
        "// AUTO-GENERATED — edit tokens/*.json and rebuild".to_string(),
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]".to_string(),
        "pub enum ThemeId {".to_string(),
    ];
    for (id, _) in TOKEN_FILES {
        lines.push(format!("    {},", to_variant(id)));
    }
    lines.push("}".to_string());
    lines.push(String::new());
    lines.push("impl ThemeId {".to_string());
    lines.push("    pub fn all() -> &'static [ThemeId] {".to_string());
    let all: Vec<_> = TOKEN_FILES
        .iter()
        .map(|(id, _)| format!("ThemeId::{}", to_variant(id)))
        .collect();
    lines.push(format!("        &[{}]", all.join(", ")));
    lines.push("    }".to_string());
    lines.push("    pub fn default_first() -> ThemeId { ThemeId::MotoliiDark }".to_string());
    lines.push("    pub fn as_str(self) -> &'static str {".to_string());
    lines.push("        match self {".to_string());
    for (id, _) in TOKEN_FILES {
        lines.push(format!("            ThemeId::{} => \"{id}\",", to_variant(id)));
    }
    lines.push("        }".to_string());
    lines.push("    }".to_string());
    lines.push("}".to_string());
    lines.push(String::new());

    lines.push("#[derive(Debug, Clone)]".to_string());
    lines.push("pub struct ThemePalette {".to_string());
    for (path, _) in &themes[0].tokens {
        let field = rust_field_name(path);
        match themes[0].tokens.get(path).unwrap() {
            ResolvedToken::Color(_) => lines.push(format!("    pub {field}: [u8; 4],")),
            ResolvedToken::Dimension(_) => lines.push(format!("    pub {field}: f32,")),
        }
    }
    lines.push("}".to_string());
    lines.push(String::new());

    lines.push("impl ThemePalette {".to_string());
    for theme in themes {
        let variant = to_variant(&theme.id);
        lines.push(format!("    pub fn {}() -> Self {{", variant.to_lowercase()));
        lines.push("        Self {".to_string());
        for (path, token) in &theme.tokens {
            let field = rust_field_name(path);
            match token {
                ResolvedToken::Color(c) => {
                    let a = (c.a * 255.0).round() as u8;
                    lines.push(format!("            {field}: [{}, {}, {}, {a}],", c.r, c.g, c.b));
                }
                ResolvedToken::Dimension(px) => {
                    lines.push(format!("            {field}: {px:.1},"));
                }
            }
        }
        lines.push("        }".to_string());
        lines.push("    }".to_string());
    }
    lines.push("}".to_string());

    lines.join("\n")
}

pub fn relative_luminance(c: &ResolvedColor) -> f32 {
    fn channel(v: f32) -> f32 {
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
    let r = channel(c.r as f32 / 255.0);
    let g = channel(c.g as f32 / 255.0);
    let b = channel(c.b as f32 / 255.0);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

pub fn contrast_ratio(fg: &ResolvedColor, bg: &ResolvedColor) -> f32 {
    let l1 = relative_luminance(fg);
    let l2 = relative_luminance(bg);
    let (lighter, darker) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

pub fn check_text_contrast(theme: &ThemeTokens) -> Result<(), TokenError> {
    let bg = theme
        .tokens
        .get("color.surface.bg")
        .and_then(|t| match t {
            ResolvedToken::Color(c) => Some(c),
            _ => None,
        })
        .ok_or_else(|| TokenError::Schema("missing color.surface.bg".into()))?;
    if let Some(ResolvedToken::Color(fg)) = theme.tokens.get("color.content.primary") {
        let ratio = contrast_ratio(fg, bg);
        if ratio < 4.5 {
            return Err(TokenError::Schema(format!(
                "color.content.primary vs surface.bg contrast {ratio:.2} < 4.5 in {}",
                theme.id
            )));
        }
    }
    if let Some(ResolvedToken::Color(fg)) = theme.tokens.get("color.content.secondary") {
        let ratio = contrast_ratio(fg, bg);
        if ratio < 3.0 {
            return Err(TokenError::Schema(format!(
                "color.content.secondary vs surface.bg contrast {ratio:.2} < 3.0 in {}",
                theme.id
            )));
        }
    }
    Ok(())
}

pub fn write_generated(manifest_dir: &Path, out_dir: &Path) -> Result<(), TokenError> {
    fs::create_dir_all(out_dir)?;
    let themes = load_all_themes(manifest_dir)?;
    for theme in &themes {
        check_text_contrast(theme)?;
    }
    // Slint globals: dark as compile-time default; runtime applies resolved palette.
    let slint = generate_slint_globals(&themes[0]);
    fs::write(out_dir.join("theme.slint"), slint)?;
    let rust = generate_rust_theme(&themes);
    fs::write(out_dir.join("theme.rs"), rust)?;
    let apply = generate_apply_theme(&themes[0]);
    fs::write(out_dir.join("apply_theme.rs"), apply)?;

    let manifest = serde_json::json!({
        "generator": "u0v-visual/token_gen",
        "themes": themes.iter().map(|t| t.id.clone()).collect::<Vec<_>>(),
        "token_count": themes[0].tokens.len(),
    });
    fs::write(
        out_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    Ok(())
}

pub fn manifest_dir() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"))
}
