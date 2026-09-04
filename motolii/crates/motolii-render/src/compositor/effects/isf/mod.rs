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

#[derive(Clone, Debug)]
pub struct IsfPadding {
    pub param: String,
    pub scale: f32,
}

#[derive(Clone, Debug)]
pub struct IsfManifest {
    pub id: Option<String>,
    pub expose: bool,
    pub output_float: bool,
    pub padding: Option<IsfPadding>,
    pub description: Option<String>,
    pub inputs: Vec<IsfInput>,
    pub passes: Vec<IsfPass>,
}

impl Default for IsfManifest {
    fn default() -> Self {
        Self {
            id: None,
            expose: true,
            output_float: false,
            padding: None,
            description: None,
            inputs: Vec::new(),
            passes: Vec::new(),
        }
    }
}

impl IsfManifest {
    pub fn image_inputs(&self) -> impl Iterator<Item = &IsfInput> {
        self.inputs
            .iter()
            .filter(|input| input.ty == IsfInputType::Image)
    }

    pub fn param_inputs(&self) -> impl Iterator<Item = &IsfInput> {
        self.inputs
            .iter()
            .filter(|input| input.ty != IsfInputType::Image)
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
    let id = value.get("ID").and_then(|v| v.as_str()).map(str::to_owned);
    let expose = value
        .get("EXPOSE")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let output_float = value
        .get("OUTPUT_FLOAT")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let padding = value.get("PADDING").and_then(|v| {
        Some(IsfPadding {
            param: v.get("PARAM")?.as_str()?.to_owned(),
            scale: v.get("SCALE").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
        })
    });
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
            id,
            expose,
            output_float,
            padding,
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
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .map_err(|e| IsfError::Validate(e.to_string()))?;
    naga::back::wgsl::write_string(&module, &info, naga::back::wgsl::WriterFlags::empty())
        .map_err(|e| IsfError::WgslWrite(e.to_string()))
}

pub(super) fn compiled_stages(isf_source: &str) -> Result<(IsfManifest, String, String), IsfError> {
        let (manifest, filter_body) = parse_isf_source(isf_source)?;
        let (image_order, param_order) = super::vism::orders(&manifest);

        let fragment_glsl =
            wrap_fragment_source(&manifest, &image_order, &param_order, &filter_body);
        let fragment_wgsl = compile_glsl_to_wgsl(&fragment_glsl, naga::ShaderStage::Fragment)?;
        let vertex_wgsl = compile_glsl_to_wgsl(VERTEX_SOURCE, naga::ShaderStage::Vertex)?;

    Ok((manifest, vertex_wgsl, fragment_wgsl))
}
