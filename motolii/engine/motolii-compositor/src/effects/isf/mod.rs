//! 生 ISF(Interactive Shader Format)ファイル1本を、**generic な**経路で
//! wgpu pipeline まで持っていく——「境界の名は ISF」(supervisor 裁定、この
//! module doc は `EffectPass::Isf` の唯一の実体)。
//!
//! [`EffectPass::Glow`](crate::EffectPass::Glow) は `threshold`/`intensity`/`radius`
//! という**named field**を Rust の enum variant に直書きし、`GlowPipelines` も
//! その3つの param を名指しで uniform buffer へ詰めていた——2本目の効果を同じ形
//! (`EffectPass::Bloom{threshold,intensity,radius}` のような専用 variant)で足すと、
//! 「2 variant の Rust enum が動く」以上の証拠にならない(ISF 採否の判断材料には
//! ならない、supervisor 裁定の指摘そのもの)。
//!
//! ここでは逆に、**このファイル自身は「threshold」も「intensity」も一度も
//! 名指さない**——[`parse_isf_source`] は `INPUTS` 配列を歩くだけ、
//! [`wrap_fragment_source`] はその配列から `uniform` 宣言と bind group layout を
//! **個数分だけ**生成する、[`IsfProgram::record`] も名前で引き当てるだけ。
//! 別の単発 ISF filter(`INPUTS` の中身が全く違う物)に差し替えても、この
//! ファイルは1行も変わらない——変わるのは `bloom.fs` 1ファイルだけ。
//!
//! ## なぜ bloom/glow か(2026-08-29、2回目の supervisor 裁定)
//!
//! 最初は brightness/contrast/saturation の色調整1本で組んだが、「今の Glow
//! (`effects/glow.rs`、M5 spike からの移植)の質に不満がある、ISF 側の evidence
//! probe をそのまま『ちゃんとした Glow』に育てて差し替え候補にできないか」という
//! 指示を受けて bloom/glow へ差し替えた。**この shader の閾値抽出・加算合成の式
//! (`bloom.fs` の `brightPass`/最終行)は `effects::glow` の `bright_fs`/
//! `composite_fs` と数値的に同一**(bloom.fs の JSON `CREDIT` 参照)——移植したのは
//! 数式の妥当性チェックのためであって、Rust パイプライン構造(`GlowPipelines`)は
//! 1行も再利用していない。**5x5 kernel は Glow の 5-tap 1D 重みの外積**なので
//! blur の質そのものは Glow の2パス分離 blur と数学的に同値(差は境界近傍の
//! clamp 挙動だけ、下の「1パス近似で崩れた所」節)。
//!
//! ## パイプライン: JSON ヘッダ → GLSL 分離 → naga で WGSL 化 → wgpu
//!
//! 1. [`parse_isf_source`] — ISF の実際の配布形式(`/*{ ... }*/` の JSON ヘッダ +
//!    生 GLSL 本体、1ファイル)をそのまま読む。JSON は `serde_json::Value` で
//!    汎用に歩く(`INPUTS[].{NAME,TYPE,DEFAULT,MIN,MAX}` だけを見る——
//!    ISF spec のこの4キー以外は無視)。
//! 2. [`wrap_fragment_source`] — 実在する ISF host(VDMX/Vuo/isf.js)がやる事と
//!    同じ役割: `IMG_THIS_PIXEL`等のマクロ・`uniform`宣言・fullscreen quad の
//!    vertex 段は、どの host も ISF ファイル自身の中身ではなく **host 側が足す**
//!    (ISF spec は「ファイルは filter 本体だけを書く、残りは host の契約」と
//!    定義している)。この関数がその「host 側」を manifest から機械的に合成する。
//! 3. [`compile_glsl_to_wgsl`] — `naga::front::glsl`(`Cargo.toml` で
//!    `glsl-in` feature を新規に有効化——[`crate`] 直下の Cargo.toml 参照)で
//!    GLSL → `naga::Module`、`naga::valid::Validator` で検証、
//!    `naga::back::wgsl::write_string` で WGSL テキストへ書き出す。
//!    **得られる WGSL テキストは、`effects::glow` の `SHADER` 定数と全く同じ形**
//!    (`device.create_shader_module(ShaderSource::Wgsl(..))`)——違いはその WGSL が
//!    Rust ソースに手で書かれているか、実行時に GLSL から生成されるかだけ。
//! 4. [`IsfProgram::compile`] — bind group layout(`group(0)`=image 入力・
//!    `group(1)`=非 image 入力、どちらも **manifest の個数ぶんだけ動的に**
//!    entry を作る)と render pipeline を1回だけ組む(`GlowPipelines::new` と
//!    同じ「初回生成して以後使い回す」規律)。
//!
//! ## 「ほぼそのまま」で通らなかった所(evidence——boundary writeup 本体は
//! コミットメッセージ/PR 説明にまとめる、ここは技術的事実だけ)
//!
//! - naga の GLSL frontend は **`#version 440/450/460 core` しか受け付けない**
//!   (`naga-29.0.4/src/front/glsl/parser.rs` `handle_directive`)。実在する ISF
//!   ファイルの大半は `#version 120` 相当(バージョン行無し、`gl_FragColor` を
//!   組み込み変数として使う古い GLSL ES 2 時代の慣用)——host 側の preamble が
//!   `#version 450 core` を強制で足し、`gl_FragColor` も**組み込みではなく
//!   ただの `out` 変数として自前宣言**する(naga はこの名前を builtin として
//!   認識しない——`variables.rs::lookup_variable` に載っていない)。
//! - naga は頂点 index の組み込み変数名を **`gl_VertexIndex`**(WGSL 風)としてしか
//!   認識しない——正式な GLSL の `gl_VertexID` ではない([`VERTEX_SOURCE`] 参照)。
//!   ISF filter 自身は vertex 段を持たないので実害は無い(host 側の
//!   fullscreen-quad だけが this の影響を受ける)。
//! - naga は `uniform`/sampler の global に **`layout(binding=N)` を必須**とする
//!   (無いと `"uniform/buffer blocks require layout(binding=X)"` で拒否)。
//!   実在する ISF ファイルはこの qualifier を書かない(Vulkan/wgpu 固有の要求で
//!   あって GLSL/ISF の contract ではない)——host 側の preamble が
//!   manifest から機械的に注入する(`wrap_fragment_source` 参照)。
//! - **一番大きかった食い違い**: naga の GLSL frontend は `uniform sampler2D x;`
//!   という「combined image sampler」の直接宣言を受け付けない
//!   (`naga-29.0.4/src/front/glsl/types.rs` の型 parser は `texture2D`/`sampler`/
//!   `image2D` は認識するが `sampler2D` は認識しない——それを裏付けるのが
//!   naga 自身の `parser_tests.rs` にある実例 `texture(sampler2D(tex, tex_sampler), uv)`:
//!   naga は **`texture2D`(image のみ)と `sampler`(sampler のみ)を別々の
//!   global として要求し、使う場所で `sampler2D(image, sampler)` という
//!   コンストラクタ構文で combine する**、Vulkan の separate-sampler 流儀)。
//!   実在する ISF ファイルはほぼ確実に desktop GLSL の伝統的な `uniform sampler2D`
//!   1行を書く——host 側 preamble は image input 1つにつき `texture2D`+`sampler`
//!   の2 global を生成し、`#define <name> sampler2D(<name>__tex, <name>__samp)`
//!   で filter 本体から見た時に元の1つの名前のまま使えるよう texturally 埋め合わせる
//!   (`wrap_fragment_source` 参照)。**Rust 側の bind group layout も image 1つに
//!   つき2 binding(texture+sampler)** になる——`IsfProgram::compile`/`record`。
//! - GLSL の `uniform bool` は書けるが、WGSL の host-shareable 領域(`uniform`)は
//!   `bool` の layout を定義していない。ここでは **`IsfInputType::Bool` も
//!   float(0.0/1.0)として `uniform` 宣言する**——ISF の「bool」を特別扱いせず、
//!   host 側で widen する(GLSL 側にも bool の uniform block を書かない——
//!   two-birds: naga 側の制約も避けつつ生成コードを一本化できる)。
//!
//! ## 1パス近似で崩れた所(bloom 固有、色調整1本だった時には無かった論点)
//!
//! **真の ISF bloom は複数 pass(`PASSES` 配列、ダウンサンプル/mip chain、
//! persistent buffer)を使うのが普通**——閾値抽出→(数段の)blur→合成、を
//! 別々の `PASSES` エントリに分けて中間結果を `PERSISTENT` buffer で持ち回る
//! (ISF spec の multi-pass 機能そのもの)。**このファイルは意図的に単一 pass**
//! (発注の timebox 指示——「複数 pass の ISF は spec surface が広すぎるので、
//! 今回は誠実に単一 pass 近似へ縮める」)。単一 pass の代償:
//!
//! - 5x5 の 25-tap を1回で読む(Glow の2パス分離 blur は 5-tap×2=10 回)——
//!   数学的な blur の質は同じだが GPU の読み出し回数は増える。動画1枚の
//!   effect としては無視できる差(fullscreen pass 1回のみ)。
//! - **境界(layer の縁)の扱いが Glow の2パス版と厳密には一致しない**——
//!   Glow は「水平 blur で clamp → その結果を垂直 blur で再度 clamp」を
//!   2回に分けて行うので、縁から半径ぶんの帯は「一度 blur されて滲んだ後の
//!   値をもう一度 blur した」結果になる。この 25-tap 版は元 texture を直接
//!   1回だけ 2D で読むので、縁の clamp は1回で終わる——画面内部では同じ結果、
//!   縁の数texel 帯だけ厳密には別の値になる(見た目には気付きにくい差だが、
//!   ピクセル一致は取れない——`tests/isf.rs` は「変わった」ことだけを縛り、
//!   Glow とのバイト一致は主張しない)。
//! - ダウンサンプル(mip chain)を使う「本物」の bloom はもっと広い範囲を
//!   安く滲ませられる(大きい blur 半径を狭い kernel で近似できる)——この
//!   単一 pass 版は `radius` を上げると kernel の実効範囲(5x5×radius)が
//!   そのまま texture 読み出し回数に直結する固定コストのまま。
//!
//! ## 意図的に配線していない物(scope、supervisor の timebox 指示どおり)
//!
//! - `TIME` — ISF の標準 uniform だが bloom は時間に依存しないので host 側は
//!   宣言していない(`RENDERSIZE` は bloom の texel stepping に要るので今回は
//!   配線した——色調整版で「意図的に配線していない」としていたのを覆した)。
//! - `image` 型 input は1本の layer texture(`EffectPass` が既に持っている
//!   「対象 layer 自身の描画結果」)にしか対応していない——ISF の `image` input が
//!   複数(2枚以上を合成する filter)ある場合、`effective_layer_textures` 側は
//!   まだ1枚しか渡していない(`render_effects.rs` の呼び出し側の制約——
//!   `IsfProgram` 自体は image binding を manifest の個数ぶん動的に作るので、
//!   複数画像の bind group layout は既に組めるが、呼び手が1枚しか渡さない)。
//! - `point2D`/`color` 型 input は manifest には正しく現れる(`IsfInputType`
//!   参照)が、[`EffectPass::Isf`] が運ぶ値は `Vec<(String, f32)>`(スカラー1個)
//!   までしか運べない——`IsfProgram::record` は多成分 input を manifest の
//!   `default` で埋める(値は来ない、既定のまま)。今回の1本
//!   (threshold/intensity/radius、全部 float)はこの制約に触れない。

use std::path::PathBuf;

use re_renderer::{
    BindGroupLayoutDesc, FileSystem as _, GpuBindGroupLayoutHandle, GpuRenderPipelineHandle,
    PipelineLayoutDesc, RenderContext, RenderPipelineDesc, ShaderModuleDesc, get_filesystem,
};

/// 実際に配線する ISF filter 1本(module doc 参照)。**生ファイルそのもの**——
/// この Rust ファイルは中身を1バイトも書き換えない。
pub(crate) const BLOOM_SOURCE: &str = include_str!("bloom.fs");

/// この ISF pass の出力 format。`effects::glow` の `GLOW_INTERMEDIATE_FORMAT` と
/// 同じ役割(pipeline はサイズ・呼び出し元 layer に依存しない固定 format を持つ、
/// `blend::SEPARABLE_BLEND_TARGET_FORMAT` も同型)——layer texture の実際の format
/// (`Compositor::upload_rgba`/YUV 変換とも `Rgba8Unorm` に揃う、一次確認は
/// `render_effects.rs` の `EffectPass::intermediate_format` doc)と一致させてある。
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
}

/// ISF の `INPUTS[].TYPE` — spec が定義する値のうち、この adapter が扱う4種
/// (`image`/`float`/`bool`/`point2D`/`color`。`long`(enum)/`audio`/`audioFFT`/
/// `event` は今回対象外——`from_isf_name` が `None` を返し、その input は
/// manifest に載らない=無音で無視される、`translate_effect_passes` の
/// 「未知は無音で skip」と同じ fail-closed の形)。
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

    /// 意味のある成分数(`default`/`min`/`max` の `[f32; 4]` のうち先頭何個が
    /// 有効か)。
    pub fn component_count(self) -> usize {
        match self {
            Self::Image => 0,
            Self::Float | Self::Bool => 1,
            Self::Point2D => 2,
            Self::Color => 4,
        }
    }

    /// host preamble の `uniform` 宣言に使う GLSL 型。**`Bool` も `float`**
    /// (module doc「did NOT survive unmodified」節参照——widen は generic に
    /// 行う、bool 専用分岐ではない)。`Image` はここでは使わない——image input は
    /// [`wrap_fragment_source`] が `texture2D`+`sampler` の2 global へ別途展開する
    /// (`sampler2D` 直接宣言が naga で通らないため、module doc 参照)。
    fn glsl_uniform_type(self) -> &'static str {
        match self {
            Self::Image => "sampler2D",
            Self::Float | Self::Bool => "float",
            Self::Point2D => "vec2",
            Self::Color => "vec4",
        }
    }
}

/// `INPUTS` 配列の1要素。**このファイルは "threshold" も "intensity" も
/// 知らない**——`NAME`/`TYPE`/`DEFAULT`/`MIN`/`MAX` という ISF spec のキー名しか
/// 見ない(module doc 冒頭)。`MAPS` は ISF spec には無い拡張(Vism 独自の
/// 語彙、1 param が複数の内部値を駆動する対応表)——ISF 側の `.fs` には
/// 現れないので `Option`。展開(1 param → 複数 const)はまだやらない、ここでは
/// 素通しするだけ。
#[derive(Clone, Debug)]
pub struct IsfInput {
    pub name: String,
    pub ty: IsfInputType,
    /// 常に4要素、意味のある成分数は `ty.component_count()`。
    pub default: [f32; 4],
    pub min: Option<[f32; 4]>,
    pub max: Option<[f32; 4]>,
    pub maps: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Default)]
pub struct IsfManifest {
    pub description: Option<String>,
    pub inputs: Vec<IsfInput>,
}

impl IsfManifest {
    pub fn image_inputs(&self) -> impl Iterator<Item = &IsfInput> {
        self.inputs.iter().filter(|input| input.ty == IsfInputType::Image)
    }

    pub fn param_inputs(&self) -> impl Iterator<Item = &IsfInput> {
        self.inputs.iter().filter(|input| input.ty != IsfInputType::Image)
    }
}

/// `manifest.inputs` を「`group(0)` の image binding 順」と「`group(1)` の
/// param binding 順」へ振り分ける。**[`wrap_fragment_source`] と
/// [`IsfProgram::compile`] の両方がこれ1つを呼ぶ**——同じ振り分けを2箇所で
/// 別々に書くと、GLSL 側の `layout(binding=N)` と Rust 側の
/// `BindGroupLayoutEntry`/`BindGroupEntry` の `binding` がずれる事故になる
/// (single source of truth)。
fn assign_bindings(manifest: &IsfManifest) -> (Vec<usize>, Vec<usize>) {
    let mut images = Vec::new();
    let mut params = Vec::new();
    for (index, input) in manifest.inputs.iter().enumerate() {
        if input.ty == IsfInputType::Image {
            images.push(index);
        } else {
            params.push(index);
        }
    }
    (images, params)
}

/// image input 1本につき group(0) の binding を2つ使う(texture + sampler、
/// module doc「一番大きかった食い違い」)。i 番目の image input の
/// texture binding は `image_texture_binding(i)`、sampler binding はその+1。
fn image_texture_binding(image_index_in_order: usize) -> u32 {
    (image_index_in_order * 2) as u32
}

/// `group(1)` のうち named param が使い切った次の binding。`RENDERSIZE`
/// (ISF 標準 uniform、`INPUTS` には現れない host 提供値)専用の予約枠——
/// [`wrap_fragment_source`]/[`IsfProgram::compile`]/[`IsfProgram::record`] の
/// 3箇所が同じ式を呼ぶ(single source of truth、`assign_bindings` と同じ理由)。
fn render_size_binding(param_order: &[usize]) -> u32 {
    param_order.len() as u32
}

/// ISF ファイルの先頭 `/*{ ... }*/` を JSON として読み、その後ろを filter 本体
/// (生 GLSL、host が生成する物へそのまま追記される)として返す。
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
    Ok((IsfManifest { description, inputs }, body))
}

/// JSON の `DEFAULT`/`MIN`/`MAX` は ISF spec 上、型によって数値1個(`float`)か
/// 配列(`point2D`=2要素・`color`=4要素)——どちらも `[f32; 4]` へ汎用に読む
/// (型で分岐しない、`component_count` が使う側の解釈を決める)。
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

/// host 側 vertex 段(module doc 参照——ISF filter 自体は vertex 段を持たない)。
/// `effects::glow` の `SHADER` 内 `vs_main` と同じ fullscreen triangle。
/// **`gl_VertexIndex`**(naga 方言、module doc「did NOT survive unmodified」)。
const VERTEX_SOURCE: &str = r#"#version 450 core

layout(location = 0) out vec2 isf_FragNormCoord;

void main() {
    vec2 positions[3] = vec2[3](vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    vec2 pos = positions[gl_VertexIndex];
    gl_Position = vec4(pos, 0.0, 1.0);
    isf_FragNormCoord = pos * 0.5 + 0.5;
}
"#;

/// `manifest`/`image_order`/`param_order` から host preamble を合成し、
/// `filter_body`(ISF ファイルの JSON ヘッダより後ろ、無改造)を追記する
/// (module doc 「host preamble」節・「一番大きかった食い違い」節)。
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

    // image input 1つにつき texture2D + sampler の2 global(naga は
    // `uniform sampler2D` の直接宣言を受け付けない、module doc 参照)。
    // filter 本体からは元の1つの名前のまま見えるよう `#define` で combine する。
    for (order_index, &index) in image_order.iter().enumerate() {
        let input = &manifest.inputs[index];
        let tex_binding = image_texture_binding(order_index);
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
        binding = render_size_binding(param_order)
    ));

    // ISF spec のマクロ族のうち、この preamble が実際に使う分だけ。
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

/// GPU に建った1本の ISF program。**初回生成して以後使い回す**
/// (`GlowPipelines::new` と同じ規律、`Compositor::with_device` が1回だけ呼ぶ)。
pub(crate) struct IsfProgram {
    manifest: IsfManifest,
    /// `ctx.gpu_resources.render_pipelines` が持つプール資源へのハンドル
    /// (`re_renderer::renderer::rectangles::RectangleRenderer` と同じ「pipeline は
    /// プールから取る」規律——`Renderer::create_renderer` は使っていない、下の
    /// module 内コメント参照。プールが `wgpu::RenderPipeline` の所有権を持つので
    /// この構造体はハンドルだけ持つ)。
    pipeline: GpuRenderPipelineHandle,
    texture_layout: GpuBindGroupLayoutHandle,
    params_layout: GpuBindGroupLayoutHandle,
    /// 全 image input で共有する既定 sampler(nearest — module doc「一番大きかった
    /// 食い違い」の補足: `texture()` を texel 境界ぴったりで呼ぶので filtering の
    /// 必要が無い。`SamplerBindingType::Filtering` で作る——`Rgba8Unorm` は
    /// filterable な format なので、texture 側も filterable と宣言する必要が
    /// あった、`compile` 内のコメント参照)。**sampler pool は
    /// `re_renderer::wgpu_resources` が非公開 module なので `GpuSamplerHandle`/
    /// `SamplerDesc` を外部 crate から名指せない**——ここだけ raw `wgpu::Sampler`
    /// のまま(pipeline/bind group layout のようなプール化はできない)。
    sampler: wgpu::Sampler,
    image_order: Vec<usize>,
    param_order: Vec<usize>,
}

impl IsfProgram {
    pub(crate) fn compile(
        ctx: &RenderContext,
        isf_source: &str,
        output_format: wgpu::TextureFormat,
    ) -> Result<Self, IsfError> {
        let device = &ctx.device;
        let (manifest, filter_body) = parse_isf_source(isf_source)?;
        let (image_order, param_order) = assign_bindings(&manifest);

        let fragment_glsl = wrap_fragment_source(&manifest, &image_order, &param_order, &filter_body);
        let fragment_wgsl = compile_glsl_to_wgsl(&fragment_glsl, naga::ShaderStage::Fragment)?;
        let vertex_wgsl = compile_glsl_to_wgsl(VERTEX_SOURCE, naga::ShaderStage::Vertex)?;

        // `ShaderModuleDesc::source` は `PathBuf`(ファイル参照)で、これが
        // `ctx.gpu_resources.shader_modules` プールの唯一の入力経路
        // (`resolver` フィールドは `pub(crate)` で外部 crate から触れない)。
        // WGSL は実行時に GLSL から生成したテキストなので、書き込んでから参照する。
        // `OsFileSystem` は `create_file` を実装せず panic するので、ディスクモードでは
        // 実ファイルへ落とす(`file_system.rs:26-31`)。
        #[cfg(load_shaders_from_disk)]
        let (vertex_path, fragment_path) = {
            let dir = std::env::temp_dir().join("motolii-isf-wgsl");
            std::fs::create_dir_all(&dir).map_err(|e| IsfError::WgslWrite(e.to_string()))?;
            let vertex_path = dir.join("vertex.wgsl");
            let fragment_path = dir.join("fragment.wgsl");
            std::fs::write(&vertex_path, vertex_wgsl.as_bytes())
                .map_err(|e| IsfError::WgslWrite(e.to_string()))?;
            std::fs::write(&fragment_path, fragment_wgsl.as_bytes())
                .map_err(|e| IsfError::WgslWrite(e.to_string()))?;
            (vertex_path, fragment_path)
        };
        #[cfg(not(load_shaders_from_disk))]
        let (vertex_path, fragment_path) = {
            let vertex_path = PathBuf::from("motolii-compositor/isf/vertex.wgsl");
            let fragment_path = PathBuf::from("motolii-compositor/isf/fragment.wgsl");
            get_filesystem()
                .create_file(&vertex_path, vertex_wgsl.into())
                .map_err(|e| IsfError::WgslWrite(e.to_string()))?;
            get_filesystem()
                .create_file(&fragment_path, fragment_wgsl.into())
                .map_err(|e| IsfError::WgslWrite(e.to_string()))?;
            (vertex_path, fragment_path)
        };

        let vertex_handle = ctx.gpu_resources.shader_modules.get_or_create(
            ctx,
            &ShaderModuleDesc {
                label: "motolii-compositor-isf-vertex".into(),
                source: vertex_path,
                extra_workaround_replacements: Vec::new(),
            },
        );
        let fragment_handle = ctx.gpu_resources.shader_modules.get_or_create(
            ctx,
            &ShaderModuleDesc {
                label: "motolii-compositor-isf-fragment".into(),
                source: fragment_path,
                extra_workaround_replacements: Vec::new(),
            },
        );

        let mut texture_entries: Vec<wgpu::BindGroupLayoutEntry> = Vec::with_capacity(image_order.len() * 2);
        for order_index in 0..image_order.len() {
            let tex_binding = image_texture_binding(order_index);
            texture_entries.push(wgpu::BindGroupLayoutEntry {
                binding: tex_binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // `texture()`(GLSL の combined sampler 呼び出し)は WGSL の
                // `textureSample` へ翻訳される——filtering 命令なので、texture 側も
                // filterable と宣言する必要がある(`textureLoad` 系を使う
                // `effects::glow`/`blend`/`matte` の `filterable: false` とはここが違う
                // ——このモジュールが `texture()` を選んだ generic な帰結であって、
                // 個別の pass の都合ではない)。
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            texture_entries.push(wgpu::BindGroupLayoutEntry {
                binding: tex_binding + 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }
        let texture_layout = ctx.gpu_resources.bind_group_layouts.get_or_create(
            device,
            &BindGroupLayoutDesc {
                label: "motolii-compositor-isf-texture-layout".into(),
                entries: texture_entries,
            },
        );

        let mut param_entries: Vec<wgpu::BindGroupLayoutEntry> = (0..param_order.len() as u32)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        param_entries.push(wgpu::BindGroupLayoutEntry {
            binding: render_size_binding(&param_order),
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
        let params_layout = ctx.gpu_resources.bind_group_layouts.get_or_create(
            device,
            &BindGroupLayoutDesc {
                label: "motolii-compositor-isf-params-layout".into(),
                entries: param_entries,
            },
        );

        let pipeline_layout = ctx.gpu_resources.pipeline_layouts.get_or_create(
            ctx,
            &PipelineLayoutDesc {
                label: "motolii-compositor-isf-pipeline-layout".into(),
                entries: vec![texture_layout, params_layout],
            },
        );

        let pipeline = ctx.gpu_resources.render_pipelines.get_or_create(
            ctx,
            &RenderPipelineDesc {
                label: "motolii-compositor-isf-pipeline".into(),
                pipeline_layout,
                vertex_entrypoint: "main".to_owned(),
                vertex_handle,
                fragment_entrypoint: "main".to_owned(),
                fragment_handle,
                vertex_buffers: Default::default(),
                render_targets: re_renderer::external::smallvec::smallvec![Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
            },
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("motolii-compositor-isf-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            manifest,
            pipeline,
            texture_layout,
            params_layout,
            sampler,
            image_order,
            param_order,
        })
    }

    /// 1 pass を `encoder` へ積む。`params` は `(name, value)` の対——名前で
    /// manifest と突き合わせ、無ければ manifest の `default` を使う
    /// (`translate_glow_params` の「track が無ければ既定値」と同じ規約)。
    /// `render_size` は `RENDERSIZE`(ISF 標準 uniform、呼び手の layer 実寸)。
    /// **多成分 input(`point2D`/`color`)は `default` のまま**——module doc
    /// 「意図的に配線していない物」参照、`params` はスカラーしか運ばない。
    pub(crate) fn record(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        source_view: &wgpu::TextureView,
        dst_view: &wgpu::TextureView,
        params: &[(String, f32)],
        render_size: [f32; 2],
    ) {
        let device = &ctx.device;
        let queue = &ctx.queue;
        // pool 資源はハンドルでしか持てない(`compile` 参照)——実体の
        // `&wgpu::BindGroupLayout`/`&wgpu::RenderPipeline` は read lock 越しに
        // 引く。lock guard は `pass` を使い終わるまでこの関数の中で生き続ける。
        let bind_group_layouts = ctx.gpu_resources.bind_group_layouts.resources();
        let texture_layout = bind_group_layouts
            .get(self.texture_layout)
            .expect("isf texture bind group layout");
        let params_layout = bind_group_layouts
            .get(self.params_layout)
            .expect("isf params bind group layout");

        let mut texture_entries: Vec<wgpu::BindGroupEntry> = Vec::with_capacity(self.image_order.len() * 2);
        for order_index in 0..self.image_order.len() {
            let tex_binding = image_texture_binding(order_index);
            // 今回配線している ISF filter は image input 1本のみ(module doc
            // 「意図的に配線していない物」)。manifest の image binding 数ぶん
            // 生成はするが、実データは呼び手が渡す1枚を使い回す。
            texture_entries.push(wgpu::BindGroupEntry {
                binding: tex_binding,
                resource: wgpu::BindingResource::TextureView(source_view),
            });
            texture_entries.push(wgpu::BindGroupEntry {
                binding: tex_binding + 1,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            });
        }
        let texture_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-isf-texture-bind"),
            layout: texture_layout,
            entries: &texture_entries,
        });

        // uniform buffer は layer ごと・呼び出しごとに値が変わるので使い回さない
        // (`GlowPipelines::record` と同じ役割分担)。
        let mut buffers: Vec<wgpu::Buffer> = Vec::with_capacity(self.param_order.len() + 1);
        for &index in &self.param_order {
            let input = &self.manifest.inputs[index];
            let count = input.ty.component_count().max(1);
            let mut components = input.default;
            if let Some((_, value)) = params.iter().find(|(name, _)| name == &input.name) {
                components[0] = *value;
            }
            let mut bytes = vec![0u8; count * 4];
            for i in 0..count {
                bytes[i * 4..i * 4 + 4].copy_from_slice(&components[i].to_le_bytes());
            }
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("motolii-compositor-isf-param"),
                size: bytes.len() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&buffer, 0, &bytes);
            buffers.push(buffer);
        }
        let render_info_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-compositor-isf-render-info"),
            size: 8,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut render_info_bytes = [0u8; 8];
        render_info_bytes[0..4].copy_from_slice(&render_size[0].to_le_bytes());
        render_info_bytes[4..8].copy_from_slice(&render_size[1].to_le_bytes());
        queue.write_buffer(&render_info_buffer, 0, &render_info_bytes);
        buffers.push(render_info_buffer);

        let param_entries: Vec<wgpu::BindGroupEntry> = buffers
            .iter()
            .enumerate()
            .map(|(binding, buffer)| wgpu::BindGroupEntry {
                binding: binding as u32,
                resource: buffer.as_entire_binding(),
            })
            .collect();
        let params_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-isf-params-bind"),
            layout: params_layout,
            entries: &param_entries,
        });

        let render_pipelines = ctx.gpu_resources.render_pipelines.resources();
        let pipeline = render_pipelines
            .get(self.pipeline)
            .expect("isf render pipeline");

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("motolii-compositor-isf-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: dst_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &texture_bind, &[]);
        pass.set_bind_group(1, &params_bind, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`parse_isf_source`] は本当に `NAME`/`TYPE`/`DEFAULT`/`MIN`/`MAX` しか
    /// 見ていない——このテストが `bloom.fs` 固有の param 名を書いているのは
    /// 「manifest から実際に読めた値」を確認するためであって、parser 自体が
    /// それらの名前を知っているわけではない(名前を変えて別の ISF ファイルを
    /// 差し替えてもこの関数は無改造で動く、module doc 冒頭の主張)。
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

    /// GLSL → naga → WGSL の変換が実際に成功し、テキストが出てくることを縛る
    /// (`IsfProgram::compile` は GPU device が要るので headless GPU 込みの
    /// live 試験は `tests/isf.rs` 側 — ここは device 非依存の純粋変換だけ)。
    #[test]
    fn compiles_the_wrapped_fragment_source_to_wgsl() {
        let (manifest, body) = parse_isf_source(BLOOM_SOURCE).expect("parse");
        let (image_order, param_order) = assign_bindings(&manifest);
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
