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
    effects
        .iter()
        .filter_map(|effect| {
            let descriptor = known_effects()
                .iter()
                .find(|descriptor| descriptor.plugin_id == effect.plugin_id)?;
            let params: Vec<(String, f32)> = effect
                .params
                .iter()
                .filter_map(|(name, value)| match value {
                    crate::doc::store::Value::F64(value) => Some((name.clone(), *value as f32)),
                    _ => None,
                })
                .collect();
            let padding = descriptor.padding.map_or(0, |padding| {
                let value = params
                    .iter()
                    .find(|(name, _)| name == padding.param)
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

pub struct EffectDescriptor {
    pub plugin_id: &'static str,
    pub params: &'static [EffectParamDescriptor],
    pub(crate) padding: Option<EffectPaddingDescriptor>,
    pub(crate) output_format: wgpu::TextureFormat,
}

pub struct EffectParamDescriptor {
    pub name: &'static str,
    pub default: f64,
    pub range: Option<(f64, f64)>,
}

#[derive(Clone, Copy)]
pub(crate) struct EffectPaddingDescriptor {
    param: &'static str,
    scale: f32,
}

fn params_from_manifest(
    manifest: &crate::render::compositor::IsfManifest,
) -> &'static [EffectParamDescriptor] {
    let params: Vec<EffectParamDescriptor> = manifest
        .param_inputs()
        .map(|input| EffectParamDescriptor {
            name: Box::leak(input.name.clone().into_boxed_str()),
            default: f64::from(input.default[0]),
            range: input
                .min
                .zip(input.max)
                .map(|(min, max)| (f64::from(min[0]), f64::from(max[0]))),
        })
        .collect();
    Box::leak(params.into_boxed_slice())
}

pub fn known_effects() -> &'static [EffectDescriptor] {
    static KNOWN: std::sync::OnceLock<Vec<EffectDescriptor>> = std::sync::OnceLock::new();
    KNOWN.get_or_init(|| {
        crate::render::compositor::vism_definitions()
            .iter()
            .filter(|definition| definition.manifest.expose)
            .map(|definition| EffectDescriptor {
                plugin_id: definition.plugin_id(),
                params: params_from_manifest(&definition.manifest),
                padding: definition.manifest.padding.as_ref().map(|padding| {
                    EffectPaddingDescriptor {
                        param: Box::leak(padding.param.clone().into_boxed_str()),
                        scale: padding.scale,
                    }
                }),
                output_format: definition.output_format(),
            })
            .collect()
    })
}
