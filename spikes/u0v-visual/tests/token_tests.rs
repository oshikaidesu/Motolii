use u0v_visual::{
    check_text_contrast, load_all_themes, load_theme, manifest_dir, contrast_ratio, ResolvedToken,
    ThemeId, TOKEN_FILES,
};

#[test]
fn dtcg_schema_and_theme_key_parity() {
    let themes = load_all_themes(&manifest_dir()).expect("all themes load");
    assert_eq!(themes.len(), TOKEN_FILES.len());
    let count = themes[0].tokens.len();
    assert!(count > 40, "expected semantic token set, got {count}");
    for t in &themes {
        check_text_contrast(t).expect("text contrast");
    }
}

#[test]
fn default_theme_is_dark() {
    assert_eq!(ThemeId::default_first(), ThemeId::MotoliiDark);
}

#[test]
fn custom_fixture_differs_from_dark_accent() {
    let dir = manifest_dir();
    let dark = load_theme(&dir, "motolii-dark", "tokens/motolii-dark.json").unwrap();
    let custom = load_theme(&dir, "custom-fixture", "tokens/custom-fixture.json").unwrap();
    let dark_sel = dark.tokens.get("color.accent.selection").unwrap();
    let custom_sel = custom.tokens.get("color.accent.selection").unwrap();
    assert_ne!(format!("{dark_sel:?}"), format!("{custom_sel:?}"));
}

#[test]
fn theme_pref_roundtrip() {
    let path = std::env::temp_dir().join("motolii-u0v-test-pref.json");
    u0v_visual::save_theme_pref(&path, ThemeId::MotoliiLight).unwrap();
    assert_eq!(u0v_visual::load_theme_pref(&path).unwrap(), ThemeId::MotoliiLight);
    let _ = std::fs::remove_file(path);
}

#[test]
fn invalid_theme_file_falls_back_to_dark() {
    let dir = manifest_dir();
    let broken = dir.join("tokens/broken-test.json");
    std::fs::write(&broken, r#"{"$schema":"https://www.designtokens.org/schemas/2025.10/formatSchema.json","color":{}}"#).unwrap();
    let result = u0v_visual::load_theme(&dir, "broken", "tokens/broken-test.json");
    assert!(result.is_err());
    let _ = std::fs::remove_file(broken);
    let (theme, diag) = u0v_visual::load_theme_safe(&dir, ThemeId::MotoliiLight);
    assert!(diag.is_none());
    assert!(theme.tokens.contains_key("color.surface.bg"));
}

#[test]
fn content_primary_contrast_above_45() {
    let dark = load_theme(&manifest_dir(), "motolii-dark", "tokens/motolii-dark.json").unwrap();
    let bg = match dark.tokens.get("color.surface.bg").unwrap() {
        ResolvedToken::Color(c) => c,
        _ => panic!(),
    };
    let fg = match dark.tokens.get("color.content.primary").unwrap() {
        ResolvedToken::Color(c) => c,
        _ => panic!(),
    };
    assert!(contrast_ratio(fg, bg) >= 4.5);
}
