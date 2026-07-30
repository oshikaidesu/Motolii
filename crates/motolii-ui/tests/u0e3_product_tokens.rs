#[allow(non_snake_case)]
mod generated {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../ui/motolii-tokens/generated/tokens.rs"
    ));
}

#[test]
fn generated_product_dark_theme_compiles_with_accepted_roles() {
    let dark = generated::generated_theme(generated::GeneratedThemeId::MotoliiDark);

    assert_eq!(
        dark.color__surface__app,
        egui::Color32::from_rgb(20, 20, 20)
    );
    assert_eq!(
        dark.color__action__active,
        egui::Color32::from_rgb(216, 181, 116)
    );
    assert_eq!(
        dark.color__way__timeline,
        egui::Color32::from_rgb(204, 149, 135)
    );
}
