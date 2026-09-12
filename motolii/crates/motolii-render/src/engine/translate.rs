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
    translate_image_effects(effects, crate::render::compositor::EffectStage::Pass)
}

pub(crate) fn translate_image_effects(effects: &[crate::doc::store::ResolvedEffect], stage: crate::render::compositor::EffectStage) -> Vec<crate::render::compositor::EffectPass> {
    let catalog = known_effects();
    effects
        .iter()
        .filter_map(|effect| {
            let descriptor = catalog.iter()
                .find(|descriptor| descriptor.plugin_id == effect.plugin_id)?;
            if descriptor.stage != stage {
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
                (value.abs() * padding.scale).ceil() as u32
            });
            Some(crate::render::compositor::EffectPass {
                image_time_offsets: descriptor.image_time_offsets.iter().map(|offset| match offset {
                    crate::render::compositor::effects::isf::TimeOffset::Fixed(seconds) => *seconds,
                    crate::render::compositor::effects::isf::TimeOffset::Param(name) => params.iter()
                        .find(|(n, _)| n == name).map(|(_, v)| *v)
                        .or_else(|| descriptor.params.iter().find(|p| &p.name == name).map(|p| p.default as f32))
                        .unwrap_or(0.0),
                }).collect(),
                plugin_id: effect.plugin_id.clone(),
                params,
                padding,
                output_format: descriptor.output_format,
                spill: descriptor.spill,
            })
        })
        .collect()
}

pub(crate) fn translate_plate_passes(effects: &[crate::doc::store::ResolvedEffect]) -> Vec<crate::render::compositor::EffectPass> {
    let catalog = known_effects();
    effects.iter().flat_map(|effect| {
        match catalog.iter().find(|d| d.plugin_id == effect.plugin_id).map(|d| d.stage) {
            Some(stage @ (crate::render::compositor::EffectStage::Pass | crate::render::compositor::EffectStage::Warp)) => translate_image_effects(std::slice::from_ref(effect), stage),
            _ => Vec::new(),
        }
    }).collect()
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
        complexity: read("complexity").floor().clamp(1.0, 8.0) as u32,
        evolution: read("evolution"),
        offset: glam::vec3(read("offset_x"), read("offset_y"), read("offset_z")) + read("seed") * read("size").max(1e-3) * glam::vec3(0.137,0.173,0.193),
    }
}

/// Clip の欄 → 層の枠での切断。平面を世界へ写すのは描く側(枠が決まる所)。
pub(crate) fn translate_clip(effects: &[crate::doc::store::ResolvedEffect]) -> Option<crate::render::compositor::ClipSpec> {
    const ID: &str = "motolii.clip";
    let effect = effects.iter().rev().find(|e| e.plugin_id == ID)?;
    let catalog = known_effects();
    let descriptor = catalog.iter().find(|d| d.plugin_id == ID)?;
    let read = |name: &str| -> f32 {
        effect.params.iter().find(|(n, _)| n == name).and_then(|(_, v)| match v {
            crate::doc::store::Value::F64(v) => Some(*v as f32),
            crate::doc::store::Value::Enum(v) => Some(*v as f32),
            crate::doc::store::Value::Bool(b) => Some(f32::from(u8::from(*b))),
            _ => None,
        }).or_else(|| descriptor.params.iter().find(|p| p.name == name).map(|p| p.default as f32)).unwrap_or(0.0)
    };
    let axis = match read("axis").round() as i32 {
        1 => -glam::Vec3::X,
        2 => glam::Vec3::Y,
        3 => -glam::Vec3::Y,
        4 => glam::Vec3::Z,
        5 => -glam::Vec3::Z,
        _ => glam::Vec3::X,
    };
    Some(crate::render::compositor::ClipSpec { axis, offset: read("offset"), cap: read("cap") > 0.5 })
}

pub fn known_effects() -> std::sync::Arc<[EffectDescriptor]> {
    crate::render::compositor::catalog_snapshot().descriptors.clone()
}

#[cfg(test)]
mod shelf_tests {
    use crate::render::compositor::EffectStage;

    /// 別の時刻のずれは、欄の値(無ければ欄の既定)で決まる — 作者が固定するのではなく利用者が回す。
    #[test]
    fn a_time_offset_named_after_a_field_takes_the_field_value() {
        use crate::doc::store::{ResolvedEffect, Value};
        let dialed = ResolvedEffect { plugin_id: "motolii.time_difference".into(), params: vec![("offset".into(), Value::F64(-1.5))], ..Default::default() };
        let untouched = ResolvedEffect { plugin_id: "motolii.time_difference".into(), params: vec![], ..Default::default() };
        assert_eq!(super::translate_effect_passes(&[dialed])[0].image_time_offsets(), &[-1.5]);
        assert_eq!(super::translate_effect_passes(&[untouched])[0].image_time_offsets(), &[-0.2], "既定は manifest の欄の既定");
    }

    /// hook の効果(Glass・Turbulent Displace)は棚に並び、pass にならず、欄は manifest から。
    #[test]
    fn hook_effects_are_on_the_shelf_and_make_no_pass() {
        let catalog = super::known_effects();
        let glass = catalog.iter().find(|d| d.plugin_id == "motolii.glass").expect("Glass は棚に在る");
        assert_eq!((glass.stage, glass.label.as_str()), (EffectStage::Surface, "Glass"));
        assert!(glass.params.iter().any(|p| p.name == "ior" && p.label == "Refraction"));
        let turbulence = catalog.iter().find(|d| d.plugin_id == "motolii.turbulent_displace").expect("棚に在る");
        assert_eq!(turbulence.stage, EffectStage::Field);
        assert_eq!(turbulence.params.iter().find(|p| p.name == "along").and_then(|p| p.choices.clone()), Some(vec!["Normal".to_owned(), "XYZ".to_owned()]));
        for id in ["motolii.glass", "motolii.turbulent_displace"] {
            let effect = crate::doc::store::ResolvedEffect { plugin_id: id.into(), params: vec![], ..Default::default() };
            assert!(super::translate_effect_passes(std::slice::from_ref(&effect)).is_empty());
        }
        let effect = crate::doc::store::ResolvedEffect { plugin_id: "motolii.turbulent_displace".into(), params: vec![], ..Default::default() };
        assert_eq!(super::translate_point_displace(std::slice::from_ref(&effect)).amount, 50.0);
    }
}

/// 立体を作る族(Extrude・Bevel)の読み取り。どちらも無ければ None。depth 0 で Bevel だけなら
/// 「縁だけ丸い板」(奥行きは丸みの半径)。
pub(crate) fn translate_solid(effects: &[crate::doc::store::ResolvedEffect]) -> Option<crate::render::compositor::extrude::Solid> {
    use crate::doc::store::solid::{BEVEL, EXTRUDE};
    let catalog = known_effects();
    let read = |id: &str, name: &str| -> Option<f32> {
        let effect = effects.iter().rev().find(|e| e.plugin_id == id)?;
        let descriptor = catalog.iter().find(|d| d.plugin_id == id)?;
        Some(effect.params.iter().find(|(n, _)| n == name).and_then(|(_, v)| match v {
            crate::doc::store::Value::F64(v) => Some(*v as f32),
            crate::doc::store::Value::Enum(v) => Some(*v as f32),
            _ => None,
        }).or_else(|| descriptor.params.iter().find(|p| p.name == name).map(|p| p.default as f32)).unwrap_or(0.0))
    };
    let depth = read(EXTRUDE, "depth");
    let radius = read(BEVEL, "radius");
    if depth.is_none() && radius.is_none() { return None; }
    Some(crate::render::compositor::extrude::Solid {
        depth: depth.unwrap_or(0.0).max(0.0),
        bevel: radius.map(|radius| crate::render::compositor::extrude::Bevel {
            radius: radius.max(0.0),
            segments: read(BEVEL, "segments").unwrap_or(6.0).round().clamp(1.0, 32.0) as u32,
            chamfer: read(BEVEL, "profile").unwrap_or(0.0) > 0.5,
        }).filter(|b| b.radius > 0.0),
    })
}
