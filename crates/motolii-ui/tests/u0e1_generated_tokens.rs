#[allow(non_snake_case)]
mod generated {
    include!("fixtures/u0e1-token-generator/generated/tokens.rs");
}

#[test]
fn generated_fixture_compiles_and_exposes_all_supported_types() {
    let dark = generated::generated_theme(generated::GeneratedThemeId::FixtureDark);
    let light = generated::generated_theme(generated::GeneratedThemeId::FixtureLight);

    let _: egui::Color32 = dark.fixture_color__sample;
    let _: f32 = dark.fixture_space__sample;
    let _: f32 = light.fixture_motion__duration;
    let _: [f32; 4] = light.fixture_motion__curve;
    assert_ne!(dark.fixture_color__sample, light.fixture_color__sample);
}
