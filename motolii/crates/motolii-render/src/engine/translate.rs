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

pub fn known_effects() -> std::sync::Arc<[EffectDescriptor]> {
    crate::render::compositor::catalog_snapshot().descriptors.clone()
}
