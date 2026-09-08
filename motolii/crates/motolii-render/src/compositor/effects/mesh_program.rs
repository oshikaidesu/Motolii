//! 網の hook(surface / field)を fork の `MeshProgram` へ差す — 最小コアの口 C(2026-09-07)。
//!
//! 作者の file は `fn field(in: FieldIn, p: FieldParams) -> FieldOut` か
//! `fn surface(in: SurfaceIn, p: SurfaceParams) -> vec3f` を書く。欄の struct と、instance の 16 float から
//! それを組む wrapper はここが manifest から生成する。欄は field → surface の順に slot を取る。

use std::sync::Arc;

use re_renderer::renderer::{MeshProgram, MeshProgramDesc};

use super::catalog::EffectStage;
use super::VismDefinition;
use crate::doc::store::ResolvedEffect;

/// instance が hook へ渡せる float の数(頂点属性 16 か所の上限から)(fork の `GpuMeshInstance::params`)。
pub(crate) const PARAM_SLOTS: usize = 12;

/// 網 1 枚の描き方: どの変種で、欄に何を入れるか。
#[derive(Clone, Default)]
pub struct MeshShading {
    pub program: Option<Arc<MeshProgram>>,
    pub params: [f32; PARAM_SLOTS],
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
            super::IsfStage::Pass | super::IsfStage::Clip => {}
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
pub(crate) fn program_desc(field: Option<&VismDefinition>, surface: Option<&VismDefinition>) -> Result<MeshProgramDesc, String> {
    let field_count = field.map_or(0, |d| d.manifest.param_inputs().count());
    let surface_count = surface.map_or(0, |d| d.manifest.param_inputs().count());
    if field_count + surface_count > PARAM_SLOTS {
        return Err(format!("hook の欄が合わせて {PARAM_SLOTS} 個を越える"));
    }
    Ok(MeshProgramDesc {
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
        let p = params(&[ResolvedEffect { plugin_id: "x.t".into(), params: vec![("along".into(), crate::doc::store::Value::F64(1.0))] }], Some(&def), None);
        assert_eq!(&p[..2], &[2.0, 1.0]);
    }
}
