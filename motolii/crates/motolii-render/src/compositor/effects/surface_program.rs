//! Shared surface programs and the mesh vertex hook, compiled from the effect manifest.
//!
//! 作者の file は `fn field(in: FieldIn, p: FieldParams) -> FieldOut` か
//! `fn surface(in: SurfaceIn, p: SurfaceParams) -> vec3f` を書く。欄の struct と、instance の 24 float から
//! それを組む wrapper はここが manifest から生成する。欄は field → surface の順に slot を取る。

use std::sync::Arc;

use re_renderer::renderer::{SurfaceProgram, SurfaceProgramDesc};

use super::catalog::EffectStage;
use super::VismDefinition;
use crate::doc::store::ResolvedEffect;

/// instance が hook へ渡せる float の数(頂点属性 16 か所の上限、法線行列を shader で出して 6 本)(fork の `GpuMeshInstance::params`)。
pub(crate) const PARAM_SLOTS: usize = 24;

/// 場が板を動かす時の格子の細かさ(一辺の升の数)。板は四角 1 枚では 4 隅しか動かせないので、
/// mesh の頂点と同じ密さで動けるよう割る。場を持たない層は割らない(既定の 1 = 三角形 2 枚)。
pub(crate) const FIELD_GRID: u32 = 128;

/// A shared program and its parameter values for a surface.
#[derive(Clone, Default)]
pub struct SurfaceShading {
    pub program: Option<Arc<SurfaceProgram>>,
    pub params: [f32; PARAM_SLOTS],
    pub reads_backdrop: bool,
    /// backdrop の mip を何段まで読むかを決める粗さ(manifest の `BACKDROP_BLUR`、無ければ 1 = 全段)。
    pub backdrop_roughness: f32,
}

impl SurfaceShading {
    /// 板を割る升の数。場を持つ効果が乗っている時だけ割る。
    pub fn field_grid(&self) -> u32 {
        match &self.program {
            Some(p) if p.desc().field.is_some() => FIELD_GRID,
            _ => 1,
        }
    }
}

/// 効果列から hook を拾う。同じ stage が複数あれば下(後)が勝つ。
pub(crate) fn hooks<'a>(effects: &'a [ResolvedEffect], definitions: &'a [VismDefinition]) -> (Option<&'a VismDefinition>, Option<&'a VismDefinition>) {
    let mut field = None;
    let mut surface = None;
    for effect in effects {
        let Some(def) = definitions.iter().find(|d| d.plugin_id() == effect.plugin_id) else { continue };
        match def.manifest.stage {
            super::IsfStage::Field => field = Some(def),
            super::IsfStage::Surface => surface = Some(def),
            super::IsfStage::Pass | super::IsfStage::Warp | super::IsfStage::Clip => {}
        }
    }
    (field, surface)
}

fn wgsl_ident(name: &str) -> String {
    let mut out: String = name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    if out.chars().next().is_none_or(|c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

/// 欄の struct と wrapper を足した snippet。`offset` は最初の欄の slot。
fn snippet(def: &VismDefinition, stage: EffectStage, offset: usize) -> String {
    let (params_ty, hook, sig, call) = match stage {
        EffectStage::Field => ("FieldParams", "motolii_field", "in: FieldIn) -> FieldOut", "field"),
        _ => ("SurfaceParams", "motolii_surface", "in: SurfaceIn) -> vec3f", "surface"),
    };
    let inputs: Vec<String> = def.manifest.param_inputs().map(|p| wgsl_ident(&p.name)).collect();
    let mut out = String::new();
    out.push_str(&format!("struct {params_ty} {{\n"));
    if inputs.is_empty() {
        out.push_str("    _unused: f32,\n");
    }
    for name in &inputs {
        out.push_str(&format!("    {name}: f32,\n"));
    }
    out.push_str("};\n\n");
    out.push_str(def.hook_source());
    out.push_str("\n\n");
    let args: Vec<String> = if inputs.is_empty() {
        vec!["0.0".into()]
    } else {
        (0..inputs.len()).map(|i| { let slot = offset + i; format!("in.params[{}][{}]", slot / 4, slot % 4) }).collect()
    };
    out.push_str(&format!("fn {hook}({sig} {{\n    let p = {params_ty}({});\n    return {call}(in, p);\n}}\n", args.join(", ")));
    out
}

/// 変種の宣言。欄の slot は field → surface の順。
pub(crate) fn program_desc(field: Option<&VismDefinition>, surface: Option<&VismDefinition>) -> Result<SurfaceProgramDesc, String> {
    let field_count = field.map_or(0, |d| d.manifest.param_inputs().count());
    let surface_count = surface.map_or(0, |d| d.manifest.param_inputs().count());
    if field_count + surface_count > PARAM_SLOTS {
        return Err(format!("hook の欄が合わせて {PARAM_SLOTS} 個を越える"));
    }
    Ok(SurfaceProgramDesc {
        label: format!("{}+{}", field.map_or("-", |d| d.plugin_id()), surface.map_or("-", |d| d.plugin_id())),
        field: field.map(|d| snippet(d, EffectStage::Field, 0)),
        surface: surface.map(|d| snippet(d, EffectStage::Surface, field_count)),
    })
}

/// 効果の値を slot に並べる。無い欄は manifest の既定。
pub(crate) fn params(effects: &[ResolvedEffect], field: Option<&VismDefinition>, surface: Option<&VismDefinition>) -> [f32; PARAM_SLOTS] {
    let mut out = [0.0f32; PARAM_SLOTS];
    let mut slot = 0;
    for def in [field, surface].into_iter().flatten() {
        let effect = effects.iter().rev().find(|e| e.plugin_id == def.plugin_id());
        for input in def.manifest.param_inputs() {
            let value = effect
                .and_then(|e| e.params.iter().find(|(n, _)| n == &input.name))
                .and_then(|(_, v)| match v {
                    crate::doc::store::Value::F64(v) => Some(*v as f32),
                    crate::doc::store::Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
                    _ => None,
                })
                .unwrap_or(input.default[0]);
            if slot < PARAM_SLOTS {
                out[slot] = value;
            }
            slot += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snippet_builds_params_struct_and_wrapper_in_manifest_order() {
        let src = super::super::VismSource { name: "t".into(), extension: "wgsl".into(),
            source: "/*{ \"ID\": \"x.t\", \"STAGE\": \"field\", \"INPUTS\": [ {\"NAME\":\"amount\",\"TYPE\":\"float\",\"DEFAULT\":2.0}, {\"NAME\":\"along\",\"TYPE\":\"long\",\"LABELS\":[\"A\",\"B\"]} ] }*/\nfn field(in: FieldIn, p: FieldParams) -> FieldOut { return FieldOut(vec3f(p.amount), in.normal); }".into() };
        let (manifest, body) = super::super::isf::parse_isf_source(&src.source).unwrap();
        let def = VismDefinition { subtypes: Vec::new(), source: src, manifest, interface: String::new(), vertex_text: body.clone(), fragment_text: body, vertex_entry: String::new(), fragment_entry: String::new() };
        let desc = program_desc(Some(&def), None).unwrap();
        let field = desc.field.unwrap();
        assert!(field.contains("struct FieldParams {\n    amount: f32,\n    along: f32,\n};"), "{field}");
        assert!(field.contains("let p = FieldParams(in.params[0][0], in.params[0][1]);"), "{field}");
        assert!(desc.surface.is_none());
        let p = params(&[ResolvedEffect { plugin_id: "x.t".into(), params: vec![("along".into(), crate::doc::store::Value::F64(1.0))], ..Default::default() }], Some(&def), None);
        assert_eq!(&p[..2], &[2.0, 1.0]);
    }
}

impl crate::render::compositor::Compositor {
    /// 効果列の hook(field / surface)から共有プログラムを組む。変種は catalog の世代ごとに覚える。
    pub(crate) fn surface_shading(&mut self, effects: &[crate::doc::store::ResolvedEffect]) -> Result<SurfaceShading, String> {
        self.surface_shading_for(effects, false)
    }

    pub(crate) fn surface_shading_for(&mut self, effects: &[crate::doc::store::ResolvedEffect], unlit: bool) -> Result<SurfaceShading, String> {
        self.refresh_catalog_programs();
        let catalog = self.catalog.clone();
        let (field, surface) = hooks(effects, &catalog.definitions);
        if !unlit && field.is_none() && surface.is_none() {
            return Ok(SurfaceShading::default());
        }
        let key = format!("{unlit}|{}|{}|{}", field.map_or("", |d| d.plugin_id()), surface.map_or("", |d| d.plugin_id()), catalog.generation);
        let program = match self.surface_programs.get(&key) {
            Some(program) => program.clone(),
            None => {
                let mut desc = program_desc(field, surface)?;
                if unlit && surface.is_none() { desc.surface = Some("fn motolii_surface(in: SurfaceIn) -> vec3f { if frame.sun_color.w > 0.0 { return vec3f(0.0); } return in.albedo * sun_shade(in.world_position, in.normal, 0.5); }".into()); }
                let program = Arc::new(SurfaceProgram::new(&self.ctx, desc).map_err(|e| e.to_string())?);
                self.surface_programs.insert(key, program.clone());
                program
            }
        };
        let params = params(effects, field, surface);
        let offset = field.map_or(0, |d| d.manifest.param_inputs().count());
        let reads_backdrop = surface.is_some_and(|d| {
            d.manifest.backdrop_input.as_ref().map_or(true, |name| {
                d.manifest.param_inputs().position(|p| &p.name == name)
                    .is_none_or(|i| params[offset + i] != 0.0)
            })
        });
        let backdrop_roughness = surface.and_then(|d| {
            let name = d.manifest.backdrop_blur_input.as_ref()?;
            let i = d.manifest.param_inputs().position(|p| &p.name == name)?;
            Some(params[offset + i].clamp(0.0, 1.0))
        }).unwrap_or(1.0);
        Ok(SurfaceShading { program: Some(program), params, reads_backdrop, backdrop_roughness })
    }

}

#[cfg(test)]
mod program_contract {
    use crate::doc::store::ResolvedEffect;

    fn compiled_without_validation_error(compositor: &mut crate::render::compositor::Compositor, effects: &[ResolvedEffect]) {
        let scope = compositor.ctx.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shading = compositor.surface_shading(effects).unwrap();
        let error = pollster::block_on(scope.pop());
        assert!(error.is_none(), "{}", error.unwrap());
        assert_eq!(shading.program.is_some(), !effects.is_empty());
    }

    /// wgpu の validation error は非同期なので、error scope で拾って契約にする:
    /// 既定の変種と、棚の hook(Glass・Turbulent Displace・両方)を差した変種が compile できる。
    #[test]
    fn default_and_shelf_hook_programs_compile() {
        let mut compositor = crate::render::compositor::Compositor::headless().unwrap();
        let scope = compositor.ctx.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let desc = re_renderer::renderer::SurfaceProgramDesc { label: "probe".into(), field: None, surface: None };
        re_renderer::renderer::SurfaceProgram::new(&compositor.ctx, desc).unwrap();
        let error = pollster::block_on(scope.pop());
        assert!(error.is_none(), "{}", error.unwrap());

        let glass = ResolvedEffect { plugin_id: "motolii.glass".into(), params: vec![], ..Default::default() };
        let turbulence = ResolvedEffect { plugin_id: "motolii.turbulent_displace".into(), params: vec![], ..Default::default() };
        compiled_without_validation_error(&mut compositor, &[]);
        compiled_without_validation_error(&mut compositor, std::slice::from_ref(&glass));
        compiled_without_validation_error(&mut compositor, std::slice::from_ref(&turbulence));
        compiled_without_validation_error(&mut compositor, &[turbulence.clone(), glass.clone()]);
        // 同じ組は同じ変種。
        let a = compositor.surface_shading(&[turbulence.clone(), glass.clone()]).unwrap().program.unwrap();
        let b = compositor.surface_shading(&[glass, turbulence]).unwrap().program.unwrap();
        assert!(std::sync::Arc::ptr_eq(&a, &b));
        assert_eq!(compositor.surface_programs.len(), 3);
    }
}

/// 場は板の絵そのものを動かす — 四角の外へ出る(実 GPU)。
/// 出られない実装(拾う場所をずらすだけ)だと、外の画素は必ず 0 のままになる。
#[cfg(test)]
mod field_leaves_the_rectangle {
    use crate::doc::store::{property, Composition, Document, EffectId, EffectInstance, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
    use crate::render::engine::Engine;

    const SIZE: u32 = 64;
    /// 層は 32×32 を (16,16) に置く。
    const BOX_MIN: u32 = 16;
    const BOX_MAX: u32 = 48;

    fn document(path: &std::path::Path, amount: f64) -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([BOX_MIN as f64, BOX_MIN as f64]) },
            Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.turbulent_displace".into() }] },
            // XYZ = 法線でなく面内へも動く向き。
            Intent::SetConstant { layer, property: PropertyId::effect_param(EffectId(0), "along").unwrap(), value: Value::F64(1.0) },
            Intent::SetConstant { layer, property: PropertyId::effect_param(EffectId(0), "amount").unwrap(), value: Value::F64(amount) },
            Intent::SetConstant { layer, property: PropertyId::effect_param(EffectId(0), "size").unwrap(), value: Value::F64(20.0) },
        ]).unwrap();
        doc
    }

    /// 元の四角の外で描かれている画素の数。
    fn outside(frame: &[u8]) -> usize {
        let mut count = 0;
        for y in 0..SIZE {
            for x in 0..SIZE {
                if (BOX_MIN..BOX_MAX).contains(&x) && (BOX_MIN..BOX_MAX).contains(&y) { continue; }
                let i = ((y * SIZE + x) * 4) as usize;
                if frame[i..i + 3].iter().copied().max().unwrap() > 0 { count += 1; }
            }
        }
        count
    }

    #[test]
    fn the_picture_moves_past_its_own_corners() {
        let dir = tempfile::tempdir().unwrap();
        let png = dir.path().join("red.png");
        let pixels: Vec<u8> = [255u8, 0, 0, 255].into_iter().cycle().take(32 * 32 * 4).collect();
        image::save_buffer(&png, &pixels, 32, 32, image::ColorType::Rgba8).unwrap();
        let mut engine = Engine::new().unwrap();

        let still = engine.render_frame(&document(&png, 0.0).view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        assert_eq!(outside(&still), 0, "動かさなければ四角の中だけ");

        let moved = engine.render_frame(&document(&png, 20.0).view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        assert!(outside(&moved) > 0, "場が絵を四角の外へ出す");
    }
}
