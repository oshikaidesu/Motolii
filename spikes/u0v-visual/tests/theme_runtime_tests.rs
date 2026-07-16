use std::fs;

use u0v_visual::{
    brush_rgb_u8, color_brush_from_token, load_theme_by_id, manifest_dir, ThemeId,
};

const SHELL_COLOR_PATHS: &[&str] = &["color.surface.bg", "color.accent.selection"];

#[test]
fn apply_generated_reads_runtime_theme_tokens() {
    let apply = fs::read_to_string(manifest_dir().join("generated/apply_theme.rs")).unwrap();
    assert!(
        !apply.contains("let _ = theme"),
        "apply must not ignore runtime theme"
    );
    assert!(
        apply.contains("color_brush_from_token(theme,"),
        "apply must resolve colors from theme.tokens at runtime"
    );
    assert!(
        !apply.contains("from_rgb_u8(106, 168, 232)"),
        "apply must not hardcode dark accent RGB"
    );
}

#[test]
fn runtime_theme_switch_produces_distinct_shell_brushes() {
    let dir = manifest_dir();
    let dark = load_theme_by_id(&dir, ThemeId::MotoliiDark).unwrap();
    let light = load_theme_by_id(&dir, ThemeId::MotoliiLight).unwrap();
    let custom = load_theme_by_id(&dir, ThemeId::CustomFixture).unwrap();

    for path in SHELL_COLOR_PATHS {
        let dark_brush = color_brush_from_token(&dark, path).expect(path);
        let light_brush = color_brush_from_token(&light, path).expect(path);
        let custom_brush = color_brush_from_token(&custom, path).expect(path);

        let dark_rgb = brush_rgb_u8(dark_brush);
        let light_rgb = brush_rgb_u8(light_brush);
        let custom_rgb = brush_rgb_u8(custom_brush);

        assert_ne!(dark_rgb, light_rgb, "{path}: dark vs light");
        assert_ne!(dark_rgb, custom_rgb, "{path}: dark vs custom");
        assert_ne!(light_rgb, custom_rgb, "{path}: light vs custom");
    }
}

#[test]
#[ignore = "requires display/libxkbcommon; run manually with cargo test -- --ignored"]
fn runtime_theme_switch_changes_theme_global_with_window() {
    use slint::Global;
    use u0v_visual::{apply_theme_tokens, MainWindow, Theme};

    let app = MainWindow::new().expect("MainWindow");
    let dir = manifest_dir();
    let dark = load_theme_by_id(&dir, ThemeId::MotoliiDark).unwrap();
    let light = load_theme_by_id(&dir, ThemeId::MotoliiLight).unwrap();

    apply_theme_tokens(&app, &dark);
    let dark_bg = brush_rgb_u8(Theme::get(&app).get_color_surface_bg());

    apply_theme_tokens(&app, &light);
    let light_bg = brush_rgb_u8(Theme::get(&app).get_color_surface_bg());

    assert_ne!(dark_bg, light_bg);
}
