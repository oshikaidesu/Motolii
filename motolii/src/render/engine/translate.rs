
use crate::render::engine::EngineError;

pub(crate) fn translate_blend_mode(
    mode: crate::doc::store::BlendMode,
) -> Result<crate::render::compositor::BlendMode, EngineError> {
    use crate::render::compositor::BlendMode as Dst;
    use crate::doc::store::BlendMode as Src;
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
    use crate::render::compositor::MatteMode as Dst;
    use crate::doc::store::MatteMode as Src;
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
        .filter_map(|effect| match effect.plugin_id.as_str() {
            "motolii.glow" => translate_glow_params(&effect.params),
            "motolii.isf_bloom" => Some(translate_isf_params(&effect.params)),
            "motolii.gradient" => Some(crate::render::compositor::EffectPass::Gradient),
            "motolii.tri_led" => Some(crate::render::compositor::EffectPass::TriLed),
            _ => None,
        })
        .collect()
}

fn translate_isf_params(params: &[(String, crate::doc::store::Value)]) -> crate::render::compositor::EffectPass {
    crate::render::compositor::EffectPass::Isf {
        params: params
            .iter()
            .filter_map(|(name, value)| match value {
                crate::doc::store::Value::F64(v) => Some((name.clone(), *v as f32)),
                _ => None,
            })
            .collect(),
    }
}

const GLOW_DEFAULT_THRESHOLD: f64 = 1.0;
const GLOW_DEFAULT_INTENSITY: f64 = 0.75;
const GLOW_DEFAULT_RADIUS: f64 = 1.0;

pub struct EffectDescriptor {
    pub plugin_id: &'static str,
    pub params: &'static [EffectParamDescriptor],
}

pub struct EffectParamDescriptor {
    pub name: &'static str,
    pub default: f64,
    pub range: Option<(f64, f64)>,
}

const GLOW_PARAMS: &[EffectParamDescriptor] = &[
    EffectParamDescriptor {
        name: "threshold",
        default: GLOW_DEFAULT_THRESHOLD,
        range: None,
    },
    EffectParamDescriptor {
        name: "intensity",
        default: GLOW_DEFAULT_INTENSITY,
        range: None,
    },
    EffectParamDescriptor {
        name: "radius",
        default: GLOW_DEFAULT_RADIUS,
        range: None,
    },
];

fn params_from_manifest(manifest: &crate::render::compositor::IsfManifest) -> &'static [EffectParamDescriptor] {
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
        vec![
            EffectDescriptor {
                plugin_id: "motolii.glow",
                params: GLOW_PARAMS,
            },
            EffectDescriptor {
                plugin_id: "motolii.isf_bloom",
                params: params_from_manifest(crate::render::compositor::isf_bloom_manifest()),
            },
            EffectDescriptor {
                plugin_id: "motolii.gradient",
                params: &[],
            },
            EffectDescriptor {
                plugin_id: "motolii.tri_led",
                params: params_from_manifest(crate::render::compositor::tri_led_manifest()),
            },
        ]
    })
}

fn translate_glow_params(params: &[(String, crate::doc::store::Value)]) -> Option<crate::render::compositor::EffectPass> {
    let find = |name: &str, default: f64| -> Option<f64> {
        match params.iter().find(|(param_name, _)| param_name == name) {
            Some((_, crate::doc::store::Value::F64(v))) => Some(*v),
            Some(_other_type) => None,
            None => Some(default),
        }
    };
    let threshold = find("threshold", GLOW_DEFAULT_THRESHOLD)?;
    let intensity = find("intensity", GLOW_DEFAULT_INTENSITY)?;
    let radius = find("radius", GLOW_DEFAULT_RADIUS)?;
    Some(crate::render::compositor::EffectPass::Glow {
        threshold: threshold as f32,
        intensity: intensity as f32,
        radius: radius as f32,
    })
}
