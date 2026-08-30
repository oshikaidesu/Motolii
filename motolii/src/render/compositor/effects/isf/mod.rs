

#[cfg(not(load_shaders_from_disk))]
use std::path::PathBuf;

#[cfg(not(load_shaders_from_disk))]
use re_renderer::{get_filesystem, FileSystem as _};
use re_renderer::RenderContext;

use super::vism::{ShaderStageSource, VismProgram};

pub(crate) const BLOOM_SOURCE: &str = include_str!("../../../../../vism/bloom.fs");

pub(crate) const ISF_TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

#[derive(Debug, thiserror::Error)]
pub(crate) enum IsfError {
    #[error("ISF ファイルに `/*{{ ... }}*/` の JSON ヘッダが見つからない")]
    MissingHeader,
    #[error("ISF ヘッダの JSON を読めない: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("naga が GLSL を解析できない({stage:?}): {detail}")]
    GlslParse {
        stage: naga::ShaderStage,
        detail: String,
    },
    #[error("naga が生成した Module を検証できない: {0}")]
    Validate(String),
    #[error("naga が WGSL を書き出せない: {0}")]
    WgslWrite(String),
    #[error("PERSISTENT なバッファは採らない(任意の時刻へ飛べるので、持ち越すと絵が操作の履歴に依存する)")]
    PersistentBuffer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsfInputType {
    Image,
    Float,
    Bool,
    Point2D,
    Color,
}

impl IsfInputType {
    fn from_isf_name(name: &str) -> Option<Self> {
        match name {
            "image" => Some(Self::Image),
            "float" => Some(Self::Float),
            "bool" => Some(Self::Bool),
            "point2D" => Some(Self::Point2D),
            "color" => Some(Self::Color),
            _ => None,
        }
    }

    pub fn component_count(self) -> usize {
        match self {
            Self::Image => 0,
            Self::Float | Self::Bool => 1,
            Self::Point2D => 2,
            Self::Color => 4,
        }
    }

    fn glsl_uniform_type(self) -> &'static str {
        match self {
            Self::Image => "sampler2D",
            Self::Float | Self::Bool => "float",
            Self::Point2D => "vec2",
            Self::Color => "vec4",
        }
    }
}

#[derive(Clone, Debug)]
pub struct IsfInput {
    pub name: String,
    pub ty: IsfInputType,
    pub default: [f32; 4],
    pub min: Option<[f32; 4]>,
    pub max: Option<[f32; 4]>,
    pub maps: Option<serde_json::Value>,
}

/// ISF `PASSES` の1つ。`PERSISTENT` は拒否し、`WIDTH`/`HEIGHT` の式は読まない
/// (中間ターゲットは常に描画サイズ)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsfPass {
    /// 後続のパスから**この名前で読める**。最後のパスは省略でき、呼び手の出力へ描く。
    pub target: Option<String>,
    /// 32bit float の中間(蓄積・HDR)。
    pub float: bool,
}

#[derive(Clone, Debug, Default)]
pub struct IsfManifest {
    pub description: Option<String>,
    pub inputs: Vec<IsfInput>,
    pub passes: Vec<IsfPass>,
}

impl IsfManifest {
    pub fn image_inputs(&self) -> impl Iterator<Item = &IsfInput> {
        self.inputs.iter().filter(|input| input.ty == IsfInputType::Image)
    }

    pub fn param_inputs(&self) -> impl Iterator<Item = &IsfInput> {
        self.inputs.iter().filter(|input| input.ty != IsfInputType::Image)
    }
}



pub(crate) fn parse_isf_source(source: &str) -> Result<(IsfManifest, String), IsfError> {
    let trimmed = source.trim_start();
    if !trimmed.starts_with("/*") {
        return Err(IsfError::MissingHeader);
    }
    let header_end = trimmed.find("*/").ok_or(IsfError::MissingHeader)?;
    let json_text = &trimmed[2..header_end];
    let body = trimmed[header_end + 2..].to_owned();

    let value: serde_json::Value = serde_json::from_str(json_text)?;
    let description = value
        .get("DESCRIPTION")
        .and_then(|v| v.as_str())
        .map(str::to_owned);

    let mut inputs = Vec::new();
    if let Some(array) = value.get("INPUTS").and_then(|v| v.as_array()) {
        for entry in array {
            let Some(name) = entry.get("NAME").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(ty) = entry
                .get("TYPE")
                .and_then(|v| v.as_str())
                .and_then(IsfInputType::from_isf_name)
            else {
                continue;
            };
            let default = read_components(entry.get("DEFAULT"));
            let min = entry.get("MIN").map(|v| read_components(Some(v)));
            let max = entry.get("MAX").map(|v| read_components(Some(v)));
            let maps = entry.get("MAPS").cloned();
            inputs.push(IsfInput {
                name: name.to_owned(),
                ty,
                default,
                min,
                max,
                maps,
            });
        }
    }
    let mut passes = Vec::new();
    if let Some(array) = value.get("PASSES").and_then(|v| v.as_array()) {
        for entry in array {
            let truthy = |key: &str| {
                entry
                    .get(key)
                    .map(|v| v.as_bool().unwrap_or(v.as_i64().unwrap_or(0) != 0))
                    .unwrap_or(false)
            };
            if truthy("PERSISTENT") {
                return Err(IsfError::PersistentBuffer);
            }
            passes.push(IsfPass {
                target: entry
                    .get("TARGET")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned),
                float: truthy("FLOAT"),
            });
        }
    }
    Ok((
        IsfManifest {
            description,
            inputs,
            passes,
        },
        body,
    ))
}

fn read_components(value: Option<&serde_json::Value>) -> [f32; 4] {
    let mut out = [0.0f32; 4];
    let Some(value) = value else { return out };
    match value {
        serde_json::Value::Array(items) => {
            for (slot, item) in out.iter_mut().zip(items.iter()) {
                *slot = item.as_f64().unwrap_or(0.0) as f32;
            }
        }
        serde_json::Value::Number(n) => out[0] = n.as_f64().unwrap_or(0.0) as f32,
        serde_json::Value::Bool(b) => out[0] = if *b { 1.0 } else { 0.0 },
        _ => {}
    }
    out
}

const VERTEX_SOURCE: &str = r#"#version 450 core

layout(location = 0) out vec2 isf_FragNormCoord;

void main() {
    vec2 positions[3] = vec2[3](vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    vec2 pos = positions[gl_VertexIndex];
    gl_Position = vec4(pos, 0.0, 1.0);
    isf_FragNormCoord = pos * 0.5 + 0.5;
}
"#;

fn wrap_fragment_source(
    manifest: &IsfManifest,
    image_order: &[usize],
    param_order: &[usize],
    filter_body: &str,
) -> String {
    let mut out = String::new();
    out.push_str("#version 450 core\n\n");
    out.push_str("layout(location = 0) in vec2 isf_FragNormCoord;\n");
    out.push_str("layout(location = 0) out vec4 gl_FragColor;\n\n");

    for (order_index, &index) in image_order.iter().enumerate() {
        let input = &manifest.inputs[index];
        let tex_binding = super::vism::image_texture_binding(order_index);
        let samp_binding = tex_binding + 1;
        out.push_str(&format!(
            "layout(set = 0, binding = {tex_binding}) uniform texture2D {name}__tex;\n",
            name = input.name
        ));
        out.push_str(&format!(
            "layout(set = 0, binding = {samp_binding}) uniform sampler {name}__samp;\n",
            name = input.name
        ));
        out.push_str(&format!(
            "#define {name} sampler2D({name}__tex, {name}__samp)\n",
            name = input.name
        ));
    }
    out.push('\n');
    for (binding, &index) in param_order.iter().enumerate() {
        let input = &manifest.inputs[index];
        let glsl_ty = input.ty.glsl_uniform_type();
        out.push_str(&format!(
            "layout(set = 1, binding = {binding}) uniform Param_{name} {{ {glsl_ty} {name}; }};\n",
            name = input.name
        ));
    }
    out.push_str(&format!(
        "layout(set = 1, binding = {binding}) uniform RenderInfo {{ vec2 RENDERSIZE; }};\n\n",
        binding = super::vism::render_size_binding(param_order.len())
    ));

    out.push_str("#define IMG_THIS_PIXEL(image) texture(image, isf_FragNormCoord)\n");
    out.push_str("#define IMG_NORM_PIXEL(image, coord) texture(image, coord)\n\n");

    out.push_str(filter_body);
    out
}

fn compile_glsl_to_wgsl(source: &str, stage: naga::ShaderStage) -> Result<String, IsfError> {
    let options = naga::front::glsl::Options::from(stage);
    let mut frontend = naga::front::glsl::Frontend::default();
    let module = frontend
        .parse(&options, source)
        .map_err(|errors| IsfError::GlslParse {
            stage,
            detail: errors.to_string(),
        })?;
    let info = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module)
        .map_err(|e| IsfError::Validate(e.to_string()))?;
    naga::back::wgsl::write_string(&module, &info, naga::back::wgsl::WriterFlags::empty())
        .map_err(|e| IsfError::WgslWrite(e.to_string()))
}

/// ISF の入口 — GLSL の本体を naga で WGSL へ写し、束縛はマニフェストへ委ねる。
/// プログラム本体は `super::vism::VismProgram`(WGSL の入口と同じ物)。
pub(crate) struct IsfProgram {
    inner: VismProgram,
}

impl IsfProgram {
    pub(crate) fn compile(
        ctx: &RenderContext,
        isf_source: &str,
        output_format: wgpu::TextureFormat,
    ) -> Result<Self, IsfError> {
        let (manifest, filter_body) = parse_isf_source(isf_source)?;
        let (image_order, param_order) = super::vism::orders(&manifest);

        let fragment_glsl =
            wrap_fragment_source(&manifest, &image_order, &param_order, &filter_body);
        let fragment_wgsl = compile_glsl_to_wgsl(&fragment_glsl, naga::ShaderStage::Fragment)?;
        let vertex_wgsl = compile_glsl_to_wgsl(VERTEX_SOURCE, naga::ShaderStage::Vertex)?;

        // 生成した WGSL は上流のファイルシステムへ載せる(ホットリロード時だけ実ファイル)。
        #[cfg(load_shaders_from_disk)]
        let (vertex_path, fragment_path) = (
            super::vism::stage_source_on_disk("isf-vertex", &vertex_wgsl),
            super::vism::stage_source_on_disk("isf-fragment", &fragment_wgsl),
        );
        #[cfg(not(load_shaders_from_disk))]
        let (vertex_path, fragment_path) = {
            let vertex_path = PathBuf::from("motolii-vism/isf/vertex.wgsl");
            let fragment_path = PathBuf::from("motolii-vism/isf/fragment.wgsl");
            get_filesystem()
                .create_file(&vertex_path, vertex_wgsl.into())
                .map_err(|e| IsfError::WgslWrite(e.to_string()))?;
            get_filesystem()
                .create_file(&fragment_path, fragment_wgsl.into())
                .map_err(|e| IsfError::WgslWrite(e.to_string()))?;
            (vertex_path, fragment_path)
        };

        Ok(Self {
            inner: VismProgram::new(
                ctx,
                "motolii-vism-isf",
                manifest,
                ShaderStageSource {
                    path: vertex_path,
                    entry_point: "main".to_owned(),
                },
                ShaderStageSource {
                    path: fragment_path,
                    entry_point: "main".to_owned(),
                },
                output_format,
            ),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        scratch: &mut super::EffectScratch,
        source_view: &wgpu::TextureView,
        dst_view: &wgpu::TextureView,
        params: &[(String, f32)],
        render_size: [f32; 2],
    ) {
        self.inner.record(
            ctx,
            encoder,
            scratch,
            &[source_view],
            dst_view,
            params,
            render_size,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_isf_header_and_body_generically() {
        let (manifest, body) = parse_isf_source(BLOOM_SOURCE).expect("parse");
        assert_eq!(manifest.inputs.len(), 4);
        assert_eq!(manifest.image_inputs().count(), 1);
        assert_eq!(manifest.param_inputs().count(), 3);

        let threshold = manifest
            .inputs
            .iter()
            .find(|input| input.name == "threshold")
            .expect("threshold input");
        assert_eq!(threshold.ty, IsfInputType::Float);
        assert_eq!(threshold.default[0], 1.0);

        let intensity = manifest
            .inputs
            .iter()
            .find(|input| input.name == "intensity")
            .expect("intensity input");
        assert_eq!(intensity.default[0], 0.75);

        assert!(body.contains("void main()"));
        assert!(body.contains("IMG_THIS_PIXEL(inputImage)"));
    }

    #[test]
    fn reads_passes_and_refuses_persistent_buffers() {
        let with_passes = r#"/*{
  "INPUTS": [],
  "PASSES": [
    { "TARGET": "bright", "FLOAT": true },
    { }
  ]
}*/
void main() {}"#;
        let (manifest, _) = parse_isf_source(with_passes).expect("PASSES を読める");
        assert_eq!(manifest.passes.len(), 2);
        assert_eq!(manifest.passes[0].target.as_deref(), Some("bright"));
        assert!(manifest.passes[0].float);
        assert_eq!(manifest.passes[1].target, None);

        let persistent = r#"/*{
  "INPUTS": [],
  "PASSES": [ { "TARGET": "acc", "PERSISTENT": true } ]
}*/
void main() {}"#;
        assert!(
            matches!(
                parse_isf_source(persistent),
                Err(IsfError::PersistentBuffer)
            ),
            "PERSISTENT は受け付けない(StatefulFilter 拒否)"
        );
    }

    #[test]
    fn compiles_the_wrapped_fragment_source_to_wgsl() {
        let (manifest, body) = parse_isf_source(BLOOM_SOURCE).expect("parse");
        let (image_order, param_order) = crate::render::compositor::effects::vism::orders(&manifest);
        let glsl = wrap_fragment_source(&manifest, &image_order, &param_order, &body);
        let wgsl = compile_glsl_to_wgsl(&glsl, naga::ShaderStage::Fragment)
            .expect("naga: GLSL -> WGSL (fragment)");
        assert!(wgsl.contains("fn main"), "WGSL に main が無い:\n{wgsl}");
    }

    #[test]
    fn compiles_the_host_vertex_source_to_wgsl() {
        let wgsl = compile_glsl_to_wgsl(VERTEX_SOURCE, naga::ShaderStage::Vertex)
            .expect("naga: GLSL -> WGSL (vertex)");
        assert!(wgsl.contains("fn main"), "WGSL に main が無い:\n{wgsl}");
    }
}
