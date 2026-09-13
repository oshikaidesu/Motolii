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
    #[error("STAGE `{0}` は知らない(pass / warp / surface / field)")]
    UnknownStage(String),
    #[error("TIME_OFFSET: {0}")]
    TimeOffset(String),
}

pub mod shadertoy;

/// どの stage に差すか。`pass` は 2D の texture→texture、`surface`/`field` は網の hook(fork の `MeshProgram`)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IsfStage {
    #[default]
    Pass,
    /// Material-local XY image warp, evaluated before placement and spatial fields.
    Warp,
    Surface,
    Field,
    /// 世界の平面で切る。shader は無く、欄だけ(fork の 1 式を板・点群・網が読む)。
    Clip,
}

impl IsfStage {
    fn from_isf_name(name: &str) -> Option<Self> {
        match name {
            "pass" => Some(Self::Pass),
            "warp" => Some(Self::Warp),
            "surface" => Some(Self::Surface),
            "field" => Some(Self::Field),
            "clip" => Some(Self::Clip),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsfInputType {
    Image,
    Float,
    /// 選択肢(ISF の `long` + `LABELS`)。値は番号。
    Long,
    Bool,
    Point2D,
    /// 3 成分の点(ISF の `point3D`)。窓には vec3 の部品が無いので、z は既定のまま(取説に明記)。
    Point3D,
    Color,
    /// 層を指す欄(Motolii の拡張。ISF には無い)。値は LayerId。shader へは何も届かない —
    /// 対になる image に `"LAYER"` で名指されると、ホストがその層の絵をその image に入れる。
    Layer,
}

impl IsfInputType {
    fn from_isf_name(name: &str) -> Option<Self> {
        match name {
            "image" => Some(Self::Image),
            "float" => Some(Self::Float),
            "long" => Some(Self::Long),
            "bool" => Some(Self::Bool),
            "point2D" => Some(Self::Point2D),
            "point3D" => Some(Self::Point3D),
            "color" => Some(Self::Color),
            "layer" => Some(Self::Layer),
            _ => None,
        }
    }

    pub fn component_count(self) -> usize {
        match self {
            Self::Image => 0,
            Self::Float | Self::Long | Self::Bool | Self::Layer => 1,
            Self::Point2D => 2,
            Self::Point3D => 3,
            Self::Color => 4,
        }
    }

    fn glsl_uniform_type(self) -> &'static str {
        match self {
            Self::Image => "sampler2D",
            Self::Float | Self::Long | Self::Bool | Self::Layer => "float",
            Self::Point2D => "vec2",
            Self::Point3D => "vec3",
            Self::Color => "vec4",
        }
    }
}

#[derive(Clone, Debug)]
pub struct IsfInput {
    pub name: String,
    /// 窓に出る英語(`LABEL`)。無ければ name。
    pub label: Option<String>,
    /// `long` の選択肢(`LABELS`)。
    pub labels: Option<Vec<String>>,
    pub ty: IsfInputType,
    pub default: [f32; 4],
    pub min: Option<[f32; 4]>,
    pub max: Option<[f32; 4]>,
    pub maps: Option<serde_json::Value>,
    /// 性格の宣言(`SUBTYPE`、Blender の語彙)。無ければ使われ方から読む。
    pub subtype: Option<String>,
    /// 畳んでおく欄(`ADVANCED`)。
    pub advanced: bool,
    /// 主役の欄(`HERO`)。無ければ宣言順の先頭が主役。
    pub hero: bool,
    /// image の欄だけ: 層の絵を**別の時刻**で読む(`TIME_OFFSET`)。
    /// ホストが供給するので、2 枚目以降でもこれを宣言していれば繋がる。
    /// 効果が自分で覚えるのではなく渡されるだけなので、純関数のまま(`plugin-resources.md` §6)。
    pub time_offset: Option<TimeOffset>,
    /// image の欄だけ: `"LAYER"` で名指した層の欄(TYPE layer)が指す層の絵が入る。
    pub layer_field: Option<String>,
}

/// 別の時刻のずれ(秒。負が過去)。作者が固定するか、float の欄を名指しして利用者に回させる。
#[derive(Clone, Debug, PartialEq)]
pub enum TimeOffset {
    Fixed(f32),
    /// その名前の float 欄の値(秒)。欄は普段の仕組みで Inspector に出る。
    Param(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsfDimension { Fixed(u32), Divided(u32), Tiles(u32, u32) }
impl IsfDimension {
    pub fn resolve(self, source: u32) -> u32 { match self { Self::Fixed(n) => n, Self::Divided(n) => source.div_ceil(n).max(1), Self::Tiles(tile, size) => source.div_ceil(tile).saturating_mul(size).clamp(1,16384) } }
}

/// ISF pass; constant WIDTH/HEIGHT expressions size temporary targets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsfPass {
    pub width: Option<IsfDimension>,
    pub height: Option<IsfDimension>,
    /// 後続のパスから**この名前で読める**。最後のパスは省略でき、呼び手の出力へ描く。
    pub target: Option<String>,
    /// 32bit float の中間(蓄積・HDR)。
    pub float: bool,
    pub channels: u8,
    /// 前のフレームの中身を保つ(ISF の PERSISTENT)。持ち主は効果ではなく host: 層 × 効果ごとの
    /// 状態として compositor が持ち、時刻 t は入点からの漸化式で決まる(plugin-resources.md §6-3)。
    pub persistent: bool,
}

#[derive(Clone, Debug)]
pub struct IsfPadding {
    pub param: String,
    pub scale: f32,
}

#[derive(Clone, Debug)]
pub struct IsfManifest {
    pub id: Option<String>,
    /// 棚に出す名前(`LABEL`)。
    pub label: Option<String>,
    pub stage: IsfStage,
    pub expose: bool,
    pub output_float: bool,
    pub linear_sampling: bool,
    pub specialize_passes: bool,
    /// 本文が時計(`TIME` `TIMEDELTA` `FRAMEINDEX` `DATE`)を読むか。読む効果だけ時刻で焼き直す。
    pub uses_clock: bool,
    /// A zero value of this input disables reads from the composited backdrop.
    pub backdrop_input: Option<String>,
    /// The roughness-like input that decides how far down the backdrop's mip chain reads go.
    pub backdrop_blur_input: Option<String>,
    pub padding: Option<IsfPadding>,
    /// 溢れの法: 素材の coverage の外へ出た出力(光・影)を、層の Blend と独立にこの混ぜ方で下へ合成する
    /// (`"SPILL": "screen" | "add" | "multiply"`。Photoshop の layer style が効果ごとに blend を持つのと同じ)。
    pub spill: Option<String>,
    pub description: Option<String>,
    pub inputs: Vec<IsfInput>,
    pub passes: Vec<IsfPass>,
}

impl Default for IsfManifest {
    fn default() -> Self {
        Self {
            id: None,
            label: None,
            stage: IsfStage::Pass,
            expose: true,
            linear_sampling: false,
            specialize_passes: false,
            uses_clock: false,
        output_float: false,
            backdrop_input: None,
            backdrop_blur_input: None,
            padding: None,
            spill: None,
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

    /// 中間 buffer の名前(初出順)。ISF の TARGET は名前付き buffer なので、同じ名前を
    /// 複数のパスが書けば同じ 1 枚を使い回す(jump flood や cascade の ping-pong)。
    /// 前のフレームを保つ target(宣言順)。
    pub fn persistent_targets(&self) -> Vec<&str> {
        self.passes.iter().filter(|p| p.persistent).filter_map(|p| p.target.as_deref()).collect()
    }

    pub fn target_slots(&self) -> Vec<&str> {
        let mut slots: Vec<&str> = Vec::new();
        for name in self.passes.iter().filter_map(|p| p.target.as_deref()) {
            if !slots.contains(&name) {
                slots.push(name);
            }
        }
        slots
    }

    /// 名前付き buffer が float かどうか(その名前を最初に宣言したパスに従う)。
    pub fn target_is_float(&self, name: &str) -> bool {
        self.passes.iter().find(|p| p.target.as_deref() == Some(name)).is_some_and(|p| p.float)
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
    let label = value.get("LABEL").and_then(|v| v.as_str()).map(str::to_owned);
    let stage = match value.get("STAGE").and_then(|v| v.as_str()) {
        None => IsfStage::Pass,
        Some(name) => IsfStage::from_isf_name(name).ok_or_else(|| IsfError::UnknownStage(name.to_owned()))?,
    };
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
    let spill = value.get("SPILL").map(|v| match v.as_str() {
        Some(mode @ ("screen" | "add" | "multiply")) => Ok(mode.to_owned()),
        _ => Err(IsfError::Validate("SPILL must be \"screen\", \"add\" or \"multiply\"".into())),
    }).transpose()?;
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
            let label = entry.get("LABEL").and_then(|v| v.as_str()).map(str::to_owned);
            let subtype = entry.get("SUBTYPE").and_then(|v| v.as_str()).map(str::to_owned);
            let advanced = entry.get("ADVANCED").and_then(|v| v.as_bool()).unwrap_or(false);
            let hero = entry.get("HERO").and_then(|v| v.as_bool()).unwrap_or(false);
            let labels = entry.get("LABELS").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect::<Vec<_>>());
            let layer_field = entry.get("LAYER").and_then(|v| v.as_str()).map(str::to_owned).filter(|_| ty == IsfInputType::Image);
            let time_offset = entry.get("TIME_OFFSET").and_then(|v| match v {
                serde_json::Value::Number(n) => n.as_f64().map(|v| TimeOffset::Fixed(v as f32)),
                serde_json::Value::String(name) => Some(TimeOffset::Param(name.clone())),
                _ => None,
            }).filter(|_| ty == IsfInputType::Image);
            inputs.push(IsfInput {
                name: name.to_owned(),
                label,
                labels,
                subtype,
                advanced,
                hero,
                ty,
                default,
                min,
                max,
                maps,
                time_offset,
                layer_field,
            });
        }
    }
    for input in &inputs {
        if let Some(name) = &input.layer_field {
            if !inputs.iter().any(|p| &p.name == name && p.ty == IsfInputType::Layer) {
                return Err(IsfError::TimeOffset(format!("{}: LAYER が名指す layer の欄 {name} が無い", input.name)));
            }
        }
        if let Some(TimeOffset::Param(name)) = &input.time_offset {
            let ok = inputs.iter().any(|p| &p.name == name && p.ty == IsfInputType::Float);
            if !ok {
                return Err(IsfError::TimeOffset(format!("{} が名指す float の欄 {name} が無い", input.name)));
            }
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
            if truthy("PERSISTENT") && entry.get("TARGET").and_then(|v| v.as_str()).is_none() {
                return Err(IsfError::Validate("PERSISTENT pass needs a TARGET".into()));
            }
            let dimension = |key: &str| -> Result<Option<IsfDimension>, IsfError> {
                let Some(value) = entry.get(key) else { return Ok(None) };
                if let Some((tile, size)) = value.as_str().and_then(|s| s.strip_prefix(&format!("ceil(${key}/"))).and_then(|s| s.split_once(")*")) {
                    if let (Ok(tile), Ok(size)) = (tile.parse::<u32>(), size.parse::<u32>()) {
                        if tile > 0 && size > 0 && size <= 16384 { return Ok(Some(IsfDimension::Tiles(tile, size))); }
                    }
                    return Err(IsfError::Validate(format!("invalid {key} tile expression")));
                }
                if let Some(divisor) = value.as_str().and_then(|s| s.strip_prefix(&format!("${key}/"))).and_then(|s| s.parse::<u32>().ok()).filter(|n| *n > 0) {
                    return Ok(Some(IsfDimension::Divided(divisor)));
                }
                let number = value.as_f64().or_else(|| value.as_str()?.parse().ok());
                match number {
                    Some(n) if n.is_finite() && n >= 1.0 && n <= 16384.0 && n.fract() == 0.0 => Ok(Some(IsfDimension::Fixed(n as u32))),
                    _ => Err(IsfError::Validate(format!("{key} must be a positive constant integer no greater than 16384"))),
                }
            };
            passes.push(IsfPass {
                width: dimension("WIDTH")?,
                height: dimension("HEIGHT")?,
                target: entry
                    .get("TARGET")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned),
                float: truthy("FLOAT"),
                persistent: truthy("PERSISTENT"),
                channels: match entry.get("CHANNELS").and_then(|v| v.as_u64()) { None => 4, Some(n @ (1 | 2 | 4)) => n as u8, _ => return Err(IsfError::Validate("CHANNELS must be 1, 2, or 4".into())) },
            });
        }
    }
    // BACKDROP_INPUT = 下の合成を受ける口。surface では「読むかどうか」の数の欄、pass では
    // 下の合成そのものが入る image の欄(2 枚目。アライトモーションの「背景のコピー」の型)。
    let backdrop_input = value.get("BACKDROP_INPUT").map(|v| {
        let name = v.as_str().ok_or_else(|| IsfError::Validate("BACKDROP_INPUT must name an input".into()))?;
        let ok = match stage {
            IsfStage::Surface => inputs.iter().any(|p| p.name == name && matches!(p.ty, IsfInputType::Float | IsfInputType::Long | IsfInputType::Bool)),
            IsfStage::Pass => {
                let images: Vec<&IsfInput> = inputs.iter().filter(|p| p.ty == IsfInputType::Image).collect();
                images.get(1).is_some_and(|p| p.name == name) && images.len() == 2 && inputs.iter().all(|p| p.time_offset.is_none())
            }
            _ => false,
        };
        if !ok {
            return Err(IsfError::Validate("BACKDROP_INPUT: surface では数の欄を、pass では 2 枚目の image(唯一の追加の image、TIME_OFFSET とは併用しない)を名指す".into()));
        }
        Ok(name.to_owned())
    }).transpose()?;
    let backdrop_blur_input = value.get("BACKDROP_BLUR").map(|v| {
        let name = v.as_str().ok_or_else(|| IsfError::Validate("BACKDROP_BLUR must name a float input".into()))?;
        if stage != IsfStage::Surface || !inputs.iter().any(|p| p.name == name && matches!(p.ty, IsfInputType::Float)) {
            return Err(IsfError::Validate("BACKDROP_BLUR must name a float surface parameter".into()));
        }
        Ok(name.to_owned())
    }).transpose()?;
    Ok((
        IsfManifest {
            id,
            label,
            stage,
            expose,
            output_float,
            linear_sampling: value.get("FILTER").and_then(|v| v.as_str()) == Some("linear"),
            specialize_passes: value.get("SPECIALIZE_PASSES").and_then(|v| v.as_bool()).unwrap_or(false),
            uses_clock: reads_clock(&body),
            backdrop_input,
            backdrop_blur_input,
            padding,
            spill,
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
    // 中間 buffer(PASSES の TARGET)。image 入力の後ろに、初出順で並ぶ(Rust 側の target_slots と同じ順)。
    for (slot, name) in manifest.target_slots().iter().enumerate() {
        let tex_binding = super::vism::image_texture_binding(image_order.len() + slot);
        let samp_binding = tex_binding + 1;
        out.push_str(&format!(
            "layout(set = 0, binding = {tex_binding}) uniform texture2D {name}__tex;\n\
             layout(set = 0, binding = {samp_binding}) uniform sampler {name}__samp;\n\
             #define {name} sampler2D({name}__tex, {name}__samp)\n"
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
        "layout(set = 1, binding = {binding}) uniform RenderInfo {{ vec2 RENDERSIZE; }};\n",
        binding = super::vism::render_size_binding(param_order.len())
    ));
    // (段, TIME, TIMEDELTA, FRAMEINDEX) が 1 つの vec4 で届く(ISF の綴りへ写す)。
    // DATE は壁時計で、どの時刻を 2 回描いても同じ絵にするため 0 固定。
    out.push_str(&format!(
        "layout(set = 1, binding = {binding}) uniform PassInfo {{ vec4 isf_PassInfo; }};\n",
        binding = super::vism::pass_index_binding(param_order.len())
    ));
    out.push_str("#define PASSINDEX int(isf_PassInfo.x)\n");
    out.push_str("#define TIME isf_PassInfo.y\n");
    out.push_str("#define TIMEDELTA isf_PassInfo.z\n");
    out.push_str("#define FRAMEINDEX int(isf_PassInfo.w)\n");
    out.push_str("#define DATE vec4(0.0)\n\n");

    // ISF の座標は下端が 0(GL の作法)、wgpu の texture は上端が 0。読む時に裏返す。
    out.push_str("#define IMG_THIS_PIXEL(image) texture(image, vec2(isf_FragNormCoord.x, 1.0 - isf_FragNormCoord.y))\n");
    out.push_str("#define IMG_NORM_PIXEL(image, coord) texture(image, vec2((coord).x, 1.0 - (coord).y))\n\n");

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

#[cfg(test)]
mod manifest_tests {
    use super::*;

    fn manifest(header: &str) -> Result<IsfManifest, IsfError> {
        parse_isf_source(&format!("/*{{ \"INPUTS\": [{{\"NAME\":\"source\",\"TYPE\":\"image\"}}, {{\"NAME\":\"gain\",\"TYPE\":\"float\",\"DEFAULT\":1.0}}]{header} }}*/ fn f() {{}}")).map(|(m, _)| m)
    }

    /// 複数パスの器: 何段目か(PASSINDEX)と中間 buffer の名前が、shader から見えること。
    /// どちらも host 側の束縛は在ったのに宣言が無く、PASSES を書いた効果が通らなかった
    /// (2026-09-12、実写の bloom で発覚)。
    #[test]
    fn a_multi_pass_effect_can_see_its_index_and_its_buffers() {
        let source = "/*{ \"INPUTS\": [{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}], \"PASSES\": [{\"TARGET\":\"half\"}, {}] }*/\n\
                      void main() { gl_FragColor = PASSINDEX == 0 ? IMG_THIS_PIXEL(inputImage) : IMG_THIS_PIXEL(half); }";
        let (_manifest, _vertex, fragment) = compiled_stages(source).expect("PASSES が書ける");
        assert!(fragment.contains("fn main"), "{fragment}");
    }

    /// 上下の向き: ISF の座標は下端が 0、wgpu の texture は上端が 0。読む macro で裏返す。
    /// 裏返さないと実写が上下逆になる(対称な効果では気づけない)。
    #[test]
    fn the_picture_is_read_right_side_up() {
        let (manifest, body) = parse_isf_source("/*{ \"INPUTS\": [{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}] }*/ void main() {}").unwrap();
        let (images, params) = crate::render::compositor::effects::vism::orders(&manifest);
        let glsl = wrap_fragment_source(&manifest, &images, &params, &body);
        assert!(glsl.contains("1.0 - isf_FragNormCoord.y"), "{glsl}");
    }

    /// TIME_OFFSET が名指す欄が無い(か float でない)なら、黙って 0 にせず名前を挙げて断る。
    #[test]
    fn a_time_offset_naming_a_missing_field_is_refused() {
        let source = "/*{ \"INPUTS\": [{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}, {\"NAME\":\"past\",\"TYPE\":\"image\",\"TIME_OFFSET\":\"nope\"}] }*/ void main() {}";
        let error = parse_isf_source(source).err().expect("断る").to_string();
        assert!(error.contains("nope") && error.contains("past"), "{error}");
    }

    /// 時計は ISF の綴り(TIME / TIMEDELTA / FRAMEINDEX / DATE)で、ホストの uniform として届く。
    /// 読む効果だけ `uses_clock` が立つ(注釈の中の語や TIMER のような別名では立たない)。
    #[test]
    fn the_clock_is_declared_and_only_readers_are_marked() {
        let reader = "/*{ \"INPUTS\": [{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}] }*/\nvoid main() { gl_FragColor = vec4(sin(TIME), float(FRAMEINDEX), TIMEDELTA, 1.0) + DATE; }";
        let (manifest, _v, fragment) = compiled_stages(reader).expect("時計が読める");
        assert!(manifest.uses_clock);
        assert!(fragment.contains("fn main"), "{fragment}");
        let quiet = "/*{ \"INPUTS\": [] }*/\n// TIME is only mentioned here\nfloat TIMER = 1.0; void main() { gl_FragColor = vec4(TIMER); }";
        assert!(!parse_isf_source(quiet).unwrap().0.uses_clock);
    }

    /// point3D は 3 成分の欄。窓には x,y だけが出て z は既定のまま(取説に明記)。
    #[test]
    fn a_point3d_field_has_three_components() {
        let source = "/*{ \"INPUTS\": [{\"NAME\":\"p\",\"TYPE\":\"point3D\",\"DEFAULT\":[1.0,2.0,3.0]}] }*/ void main() { gl_FragColor = vec4(p, 1.0); }";
        let (manifest, _v, fragment) = compiled_stages(source).expect("point3D が通る");
        let p = &manifest.inputs[0];
        assert_eq!((p.ty, p.ty.component_count()), (IsfInputType::Point3D, 3));
        assert_eq!(&p.default[..3], &[1.0, 2.0, 3.0]);
        assert!(fragment.contains("fn main"), "{fragment}");
    }

    /// image の LAYER が名指す欄が無い(か layer でない)なら、名前を挙げて断る。
    #[test]
    fn an_image_naming_a_missing_layer_field_is_refused() {
        let source = "/*{ \"INPUTS\": [{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}, {\"NAME\":\"matte\",\"TYPE\":\"image\",\"LAYER\":\"nope\"}] }*/ void main() {}";
        let error = parse_isf_source(source).err().expect("断る").to_string();
        assert!(error.contains("nope") && error.contains("matte"), "{error}");
    }

    #[test]
    fn constant_pass_dimensions_follow_isf_target_declarations() {
        let m = manifest(r#", "PASSES": [{"TARGET":"summary","WIDTH":"64","HEIGHT":"8","FLOAT":true},{}]"#).unwrap();
        assert_eq!((m.passes[0].width, m.passes[0].height), (Some(IsfDimension::Fixed(64)), Some(IsfDimension::Fixed(8))));
        assert_eq!((m.passes[1].width, m.passes[1].height), (None, None));
        let m = manifest(r#", "PASSES": [{"TARGET":"half","WIDTH":"$WIDTH/2","HEIGHT":"$HEIGHT/2"}]"#).unwrap();
        assert_eq!(m.passes[0].width.unwrap().resolve(97), 49);
        for value in ["0", "-1", "1.5", "16385", "\"$WIDTH/0\""] {
            assert!(manifest(&format!(", \"PASSES\": [{{\"TARGET\":\"summary\",\"WIDTH\":{value}}}]")).is_err());
        }
    }
}

/// 本文が時計の名前を識別子として読むか(注釈は外して見る)。
fn reads_clock(body: &str) -> bool {
    let text = shadertoy::strip_comments_pub(body);
    let is_ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
    ["TIME", "TIMEDELTA", "FRAMEINDEX", "DATE"].iter().any(|name| {
        text.match_indices(name).any(|(i, _)| {
            let before = text[..i].chars().last().is_none_or(|c| !is_ident(c));
            let after = text[i + name.len()..].chars().next().is_none_or(|c| !is_ident(c));
            before && after
        })
    })
}
