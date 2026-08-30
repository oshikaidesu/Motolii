
use crate::EngineError;

pub(crate) fn translate_blend_mode(
    mode: motolii_store::BlendMode,
) -> Result<motolii_compositor::BlendMode, EngineError> {
    use motolii_compositor::BlendMode as Dst;
    use motolii_store::BlendMode as Src;
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
    mode: motolii_store::MatteMode,
) -> motolii_compositor::MatteMode {
    use motolii_compositor::MatteMode as Dst;
    use motolii_store::MatteMode as Src;
    match mode {
        Src::Alpha => Dst::Alpha,
        Src::InvertedAlpha => Dst::InvertedAlpha,
        Src::Luma => Dst::Luma,
        Src::InvertedLuma => Dst::InvertedLuma,
    }
}

#[cfg(test)]
mod translate_blend_mode_tests {
    use super::translate_blend_mode;

    #[test]
    fn add_is_accepted() {
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::Add).unwrap(),
            motolii_compositor::BlendMode::Add
        );
    }

    #[test]
    fn separable_modes_are_accepted() {
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::Multiply).unwrap(),
            motolii_compositor::BlendMode::Multiply
        );
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::SoftLight).unwrap(),
            motolii_compositor::BlendMode::SoftLight
        );
    }

    #[test]
    fn nonseparable_modes_are_accepted() {
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::Hue).unwrap(),
            motolii_compositor::BlendMode::Hue
        );
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::Saturation).unwrap(),
            motolii_compositor::BlendMode::Saturation
        );
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::Color).unwrap(),
            motolii_compositor::BlendMode::Color
        );
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::Luminosity).unwrap(),
            motolii_compositor::BlendMode::Luminosity
        );
    }
}

#[cfg(test)]
mod translate_matte_mode_tests {
    use super::translate_matte_mode;

    #[test]
    fn all_four_matte_modes_translate_one_to_one() {
        assert_eq!(
            translate_matte_mode(motolii_store::MatteMode::Alpha),
            motolii_compositor::MatteMode::Alpha
        );
        assert_eq!(
            translate_matte_mode(motolii_store::MatteMode::InvertedAlpha),
            motolii_compositor::MatteMode::InvertedAlpha
        );
        assert_eq!(
            translate_matte_mode(motolii_store::MatteMode::Luma),
            motolii_compositor::MatteMode::Luma
        );
        assert_eq!(
            translate_matte_mode(motolii_store::MatteMode::InvertedLuma),
            motolii_compositor::MatteMode::InvertedLuma
        );
    }
}

pub(crate) fn translate_effect_passes(
    effects: &[motolii_store::ResolvedEffect],
) -> Vec<motolii_compositor::EffectPass> {
    effects
        .iter()
        .filter_map(|effect| match effect.plugin_id.as_str() {
            "motolii.glow" => translate_glow_params(&effect.params),
            "motolii.isf_bloom" => Some(translate_isf_params(&effect.params)),
            "motolii.gradient" => Some(motolii_compositor::EffectPass::Gradient),
            "motolii.tri_led" => Some(motolii_compositor::EffectPass::TriLed),
            _ => None,
        })
        .collect()
}

fn translate_isf_params(params: &[(String, motolii_store::Value)]) -> motolii_compositor::EffectPass {
    motolii_compositor::EffectPass::Isf {
        params: params
            .iter()
            .filter_map(|(name, value)| match value {
                motolii_store::Value::F64(v) => Some((name.clone(), *v as f32)),
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

fn params_from_manifest(manifest: &motolii_compositor::IsfManifest) -> &'static [EffectParamDescriptor] {
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
                params: params_from_manifest(motolii_compositor::isf_bloom_manifest()),
            },
            EffectDescriptor {
                plugin_id: "motolii.gradient",
                params: &[],
            },
            EffectDescriptor {
                plugin_id: "motolii.tri_led",
                params: params_from_manifest(motolii_compositor::tri_led_manifest()),
            },
        ]
    })
}

fn translate_glow_params(params: &[(String, motolii_store::Value)]) -> Option<motolii_compositor::EffectPass> {
    let find = |name: &str, default: f64| -> Option<f64> {
        match params.iter().find(|(param_name, _)| param_name == name) {
            Some((_, motolii_store::Value::F64(v))) => Some(*v),
            Some(_other_type) => None,
            None => Some(default),
        }
    };
    let threshold = find("threshold", GLOW_DEFAULT_THRESHOLD)?;
    let intensity = find("intensity", GLOW_DEFAULT_INTENSITY)?;
    let radius = find("radius", GLOW_DEFAULT_RADIUS)?;
    Some(motolii_compositor::EffectPass::Glow {
        threshold: threshold as f32,
        intensity: intensity as f32,
        radius: radius as f32,
    })
}

#[cfg(test)]
mod translate_effect_passes_tests {
    use super::translate_effect_passes;
    use motolii_store::ResolvedEffect;

    #[test]
    fn no_effects_yields_no_passes() {
        assert_eq!(translate_effect_passes(&[]), Vec::new());
    }

    #[test]
    fn unknown_plugin_id_is_skipped_silently() {
        let effects = vec![
            ResolvedEffect {
                plugin_id: "motolii.not-yet-implemented".to_owned(),
                params: vec![],
            },
            ResolvedEffect {
                plugin_id: "third-party.whatever".to_owned(),
                params: vec![],
            },
        ];
        assert_eq!(translate_effect_passes(&effects), Vec::new());
    }
}

#[cfg(test)]
mod known_effects_tests {
    use super::{known_effects, translate_effect_passes};
    use motolii_store::ResolvedEffect;

    #[test]
    fn known_effects_are_all_actually_drawable() {
        for descriptor in known_effects() {
            let effect = ResolvedEffect {
                plugin_id: descriptor.plugin_id.to_owned(),
                params: vec![],
            };
            assert_eq!(
                translate_effect_passes(&[effect]).len(),
                1,
                "known_effects() says {} is drawable but translate_effect_passes skipped it",
                descriptor.plugin_id,
            );
        }
    }

    #[test]
    fn known_effects_is_exactly_glow_and_isf_bloom_and_gradient_and_tri_led_today() {
        assert_eq!(known_effects().len(), 4);
        assert_eq!(known_effects()[0].plugin_id, "motolii.glow");
        assert_eq!(known_effects()[0].params.len(), 3);
        assert_eq!(known_effects()[1].plugin_id, "motolii.isf_bloom");
        assert_eq!(known_effects()[1].params.len(), 3);
        assert_eq!(known_effects()[2].plugin_id, "motolii.gradient");
        assert_eq!(known_effects()[2].params.len(), 0);
        assert_eq!(known_effects()[3].plugin_id, "motolii.tri_led");
        assert_eq!(known_effects()[3].params.len(), 1);
    }

    #[test]
    fn known_effects_isf_bloom_catalog_matches_the_generic_manifest() {
        let hand_written = known_effects()
            .iter()
            .find(|descriptor| descriptor.plugin_id == "motolii.isf_bloom")
            .expect("motolii.isf_bloom is in the catalog");
        let manifest = motolii_compositor::isf_bloom_manifest();
        let generic: Vec<_> = manifest.param_inputs().collect();

        assert_eq!(
            hand_written.params.len(),
            generic.len(),
            "ISF_BLOOM_PARAMS と bloom.fs の INPUTS で param 数が食い違っている"
        );
        for param in hand_written.params {
            let from_manifest = generic
                .iter()
                .find(|input| input.name == param.name)
                .unwrap_or_else(|| panic!("bloom.fs の INPUTS に `{}` が無い", param.name));
            assert_eq!(
                param.default,
                f64::from(from_manifest.default[0]),
                "`{}` の既定値が ISF_BLOOM_PARAMS と bloom.fs の INPUTS で食い違っている",
                param.name
            );
        }
    }
}
