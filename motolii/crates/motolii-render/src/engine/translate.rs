use crate::render::engine::EngineError;

pub(crate) fn translate_blend_mode(
    mode: crate::doc::store::BlendMode,
) -> Result<crate::render::compositor::BlendMode, EngineError> {
    use crate::doc::store::BlendMode as Src;
    use crate::render::compositor::BlendMode as Dst;
    match mode {
        Src::Normal => Ok(Dst::Normal),
        Src::Add => Ok(Dst::Add),
        Src::Multiply => Ok(Dst::Multiply),
        Src::Screen => Ok(Dst::Screen),
        Src::Overlay => Ok(Dst::Overlay),
        Src::Darken => Ok(Dst::Darken),
        Src::Lighten => Ok(Dst::Lighten),
        Src::ColorDodge => Ok(Dst::ColorDodge),
        Src::ColorBurn => Ok(Dst::ColorBurn),
        Src::HardLight => Ok(Dst::HardLight),
        Src::SoftLight => Ok(Dst::SoftLight),
        Src::Difference => Ok(Dst::Difference),
        Src::Exclusion => Ok(Dst::Exclusion),
        Src::Hue => Ok(Dst::Hue),
        Src::Saturation => Ok(Dst::Saturation),
        Src::Color => Ok(Dst::Color),
        Src::Luminosity => Ok(Dst::Luminosity),
    }
}

pub(crate) fn translate_matte_mode(
    mode: crate::doc::store::MatteMode,
) -> crate::render::compositor::MatteMode {
    use crate::doc::store::MatteMode as Src;
    use crate::render::compositor::MatteMode as Dst;
    match mode {
        Src::Alpha => Dst::Alpha,
        Src::InvertedAlpha => Dst::InvertedAlpha,
        Src::Luma => Dst::Luma,
        Src::InvertedLuma => Dst::InvertedLuma,
    }
}

pub(crate) fn translate_effect_passes(
    effects: &[crate::doc::store::ResolvedEffect],
) -> Vec<crate::render::compositor::EffectPass> {
    let catalog = known_effects();
    effects
        .iter()
        .filter_map(|effect| {
            let descriptor = catalog.iter()
                .find(|descriptor| descriptor.plugin_id == effect.plugin_id)?;
            if descriptor.stage != crate::render::compositor::EffectStage::Pass {
                return None;
            }
            let params: Vec<(String, f32)> = effect
                .params
                .iter()
                .filter_map(|(name, value)| match value {
                    crate::doc::store::Value::F64(value) => Some((name.clone(), *value as f32)),
                    _ => None,
                })
                .collect();
            let padding = descriptor.padding.as_ref().map_or(0, |padding| {
                let value = params
                    .iter()
                    .find(|(name, _)| name == &padding.param)
                    .map(|(_, value)| *value)
                    .or_else(|| {
                        descriptor
                            .params
                            .iter()
                            .find(|param| param.name == padding.param)
                            .map(|param| param.default as f32)
                    })
                    .unwrap_or(0.0);
                (value.round().max(0.0) * padding.scale).round() as u32
            });
            Some(crate::render::compositor::EffectPass {
                plugin_id: effect.plugin_id.clone(),
                params,
                padding,
                output_format: descriptor.output_format,
            })
        })
        .collect()
}

pub use crate::render::compositor::{EffectDescriptor, EffectParamDescriptor};

/// Turbulent Displace の欄を点群用の CPU の写しへ。既定は棚の宣言から。
pub(crate) fn translate_point_displace(
    effects: &[crate::doc::store::ResolvedEffect],
) -> crate::render::compositor::PointDisplace {
    const ID: &str = "motolii.turbulent_displace";
    let catalog = known_effects();
    let Some(descriptor) = catalog.iter().find(|d| d.plugin_id == ID) else { return Default::default() };
    let Some(effect) = effects.iter().rev().find(|e| e.plugin_id == ID) else { return Default::default() };
    let read = |name: &str| -> f32 {
        effect.params.iter().find(|(n, _)| n == name).and_then(|(_, v)| match v {
            crate::doc::store::Value::F64(v) => Some(*v as f32),
            _ => None,
        }).or_else(|| descriptor.params.iter().find(|p| p.name == name).map(|p| p.default as f32)).unwrap_or(0.0)
    };
    crate::render::compositor::PointDisplace {
        amount: read("amount"),
        size: read("size").max(1e-3),
        complexity: read("complexity").round().clamp(1.0, 8.0) as u32,
        evolution: read("evolution"),
        offset: glam::vec3(read("offset_x"), read("offset_y"), read("offset_z")),
    }
}

pub fn known_effects() -> std::sync::Arc<[EffectDescriptor]> {
    crate::render::compositor::catalog_snapshot().descriptors.clone()
}

#[cfg(test)]
mod shelf_tests {
    use crate::render::compositor::EffectStage;

    /// hook の効果(Glass・Turbulent Displace)は棚に並び、pass にならず、欄は manifest から。
    #[test]
    fn hook_effects_are_on_the_shelf_and_make_no_pass() {
        let catalog = super::known_effects();
        let glass = catalog.iter().find(|d| d.plugin_id == "motolii.glass").expect("Glass は棚に在る");
        assert_eq!((glass.stage, glass.label.as_str()), (EffectStage::Surface, "Glass"));
        assert!(glass.params.iter().any(|p| p.name == "ior" && p.label == "Refraction"));
        let turbulence = catalog.iter().find(|d| d.plugin_id == "motolii.turbulent_displace").expect("棚に在る");
        assert_eq!(turbulence.stage, EffectStage::Field);
        assert_eq!(turbulence.params.iter().find(|p| p.name == "along").and_then(|p| p.choices.clone()), Some(vec!["Normal".to_owned(), "Space".to_owned()]));
        for id in ["motolii.glass", "motolii.turbulent_displace"] {
            let effect = crate::doc::store::ResolvedEffect { plugin_id: id.into(), params: vec![] };
            assert!(super::translate_effect_passes(std::slice::from_ref(&effect)).is_empty());
        }
        let effect = crate::doc::store::ResolvedEffect { plugin_id: "motolii.turbulent_displace".into(), params: vec![] };
        assert_eq!(super::translate_point_displace(std::slice::from_ref(&effect)).amount, 50.0);
    }
}
