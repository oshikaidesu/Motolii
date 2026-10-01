//! 札の本文 → 計算シェーダーの全文(WESL で束ねる)と、override の札の読み取り。
//! GPU には触らない — ここは文字だけ。口は `module_source` / `wgsl_manifest` / `validate` の 3 つ。

use super::*;

/// 作者も外枠も同じ型と束ねを読む。
pub(crate) const PRELUDE: &str = "struct Item { lo: vec2f, hi: vec2f, room_lo: vec2f, room_size: vec2f, radius: f32, group: u32, margin: f32, weight: f32, parent_slot: u32, anchor_slot: u32 };\n\
struct Offset { translate: vec2f, rotate: f32, scale: f32, tint: vec4f };\n\
struct BlockHost { time: f32, members: u32, objects: u32, round: u32, source: u32, base: u32, last: u32 };\n\
@group(0) @binding(0) var<storage, read> objects: array<Item>;\n\
@group(0) @binding(1) var<storage, read> state_in: array<Offset>;\n\
@group(0) @binding(2) var<uniform> host: BlockHost;\n\
@group(0) @binding(3) var<uniform> block_params: array<vec4f, 6>;\n\
@group(0) @binding(4) var<storage, read_write> state_out: array<Offset>;\n\
@group(0) @binding(5) var<storage, read> members: array<u32>;\n\
@group(0) @binding(6) var<storage, read> neighbor_starts: array<u32>;\n\
@group(0) @binding(7) var<storage, read> neighbor_list: array<u32>;\n\
const NO_OFFSET: Offset = Offset(vec2f(0.0), 0.0, 1.0, vec4f(1.0));\n\
fn neighbor_count(k: u32) -> u32 { return neighbor_starts[k + 1u] - neighbor_starts[k]; }\n\
fn neighbor(k: u32, i: u32) -> u32 { return neighbor_list[neighbor_starts[k] + i]; }\n\
fn now_lo(k: u32) -> vec2f { return objects[k].lo + state_in[k].translate; }\n\
fn now_hi(k: u32) -> vec2f { return objects[k].hi + state_in[k].translate; }\n\
const NO_OBJECT: u32 = 0xffffffffu;\n\
fn parent(k: u32) -> Offset { let j = objects[k].parent_slot; if j == NO_OBJECT { return NO_OFFSET; } return state_in[j]; }\n\
fn anchor(k: u32) -> Offset { let j = objects[k].anchor_slot; if j == NO_OBJECT { return NO_OFFSET; } return state_in[j]; }\n\n";

fn wgsl_ident(name: &str) -> String {
    let mut out: String = name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    if out.chars().next().is_none_or(|c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

/// 札の本文の頭にある `import ...;` を集める(WESL は import を file の先頭に置く決まり。札は manifest の後に書く)。
fn hoist_imports(body: &str) -> (String, String) {
    let (mut imports, mut rest) = (String::new(), String::new());
    let mut cursor = body;
    while !cursor.is_empty() {
        let trimmed = cursor.trim_start();
        if trimmed.starts_with("import ") {
            let end = trimmed.find(';').map_or(trimmed.len(), |i| i + 1);
            imports.push_str(&trimmed[..end]);
            imports.push('\n');
            cursor = &trimmed[end..];
        } else {
            let line_end = cursor.find('\n').map_or(cursor.len(), |i| i + 1);
            rest.push_str(&cursor[..line_end]);
            cursor = &cursor[line_end..];
        }
    }
    (imports, rest)
}

/// 棚の module 名 → WESL の path(`package::<name>`)。
fn module_path(name: &str) -> wesl::ModulePath {
    wesl::ModulePath { origin: wesl::syntax::PathOrigin::Absolute, components: vec![name.to_owned()] }
}

/// 作者の本体に、型・束ね・欄の struct・外枠を足した計算シェーダーの全文。
/// 束ね(PRELUDE)は module `motolii`、棚の module は `modules`(名前, 本文)。札は `import package::cavalry::{ cv_ease };` で引く(WESL)。
/// 欄の渡し方は 2 つ: 頭の JSON の札(`BlockParams` を引数で受ける)と、override の札(`wgsl_manifest` が `var<private>` に
/// 書き換えた欄を、外枠が毎コマ `motolii_block_inputs()` で uniform から代入してから `block(k)` を呼ぶ)。
pub(crate) fn module_source(manifest: &IsfManifest, body: &str, modules: &[(String, String)]) -> Result<String, String> {
    let names: Vec<String> = manifest.param_inputs().map(|p| wgsl_ident(&p.name)).collect();
    if names.len() > PARAM_SLOTS {
        return Err(format!("block の欄は {PARAM_SLOTS} 個まで"));
    }
    let legacy = body.contains("BlockParams");
    let (imports, body) = hoist_imports(body);
    let mut out = String::from("import package::motolii::{ Item, Offset, BlockHost, NO_OFFSET, NO_OBJECT, neighbor_count, neighbor, now_lo, now_hi, parent, anchor, objects, state_in, host, block_params, state_out, members, neighbor_starts, neighbor_list };\n");
    out.push_str(&imports);
    if legacy {
        out.push_str("struct BlockParams {\n");
        if names.is_empty() {
            out.push_str("    _unused: f32,\n");
        }
        for name in &names {
            out.push_str(&format!("    {name}: f32,\n"));
        }
        out.push_str("};\n\n");
    }
    out.push_str(&body);
    let call = if legacy {
        let args: Vec<String> = if names.is_empty() { vec!["0.0".into()] } else { (0..names.len()).map(|s| format!("block_params[{}][{}]", s / 4, s % 4)).collect() };
        format!("let d = block(k, BlockParams({}));", args.join(", "))
    } else {
        "motolii_block_inputs();\n    let d = block(k);".to_owned()
    };
    out.push_str(&format!(
        "\n\n@compute @workgroup_size(64)\nfn motolii_block_main(@builtin(global_invocation_id) gid: vec3u) {{\n\
         \x20   let motolii_at = host.base + gid.x;\n\
         \x20   if motolii_at >= host.last {{ return; }}\n\
         \x20   let k = members[motolii_at];\n\
         \x20   {call}\n\
         \x20   let s = state_in[k];\n\
         \x20   state_out[k] = Offset(s.translate + d.translate, s.rotate + d.rotate, s.scale * d.scale, s.tint * d.tint);\n}}\n"
    ));
    let mut resolver = wesl::VirtualResolver::new();
    resolver.add_module(module_path("motolii"), PRELUDE.into());
    for (name, source) in modules {
        resolver.add_module(module_path(name), source.as_str().into());
    }
    resolver.add_module(module_path("block"), out.into());
    let mut compiler = wesl::Wesl::new_barebones().set_custom_resolver(resolver);
    compiler
        .set_options(wesl::CompileOptions { imports: true, condcomp: true, strip: false, lazy: false, validate: false, ..Default::default() })
        .set_mangler(wesl::ManglerKind::None);
    let compiled = compiler.compile(&module_path("block")).map_err(|e| format!("wesl: {e}"))?;
    Ok(compiled.to_string())
}

/// 属性の引数: 文字列・数・名前(true / false)。
#[derive(Clone, Debug, PartialEq)]
enum AttrArg {
    Str(String),
    Num(f64),
    Word(String),
}

#[derive(Clone, Debug, PartialEq)]
struct Attr {
    name: String,
    args: Vec<AttrArg>,
}

impl Attr {
    fn str(&self, i: usize) -> Result<&str, String> {
        match self.args.get(i) {
            Some(AttrArg::Str(s)) => Ok(s),
            _ => Err(format!("@{}: {} 番目は文字列", self.name, i + 1)),
        }
    }
    fn num(&self, i: usize) -> Result<f64, String> {
        match self.args.get(i) {
            Some(AttrArg::Num(v)) => Ok(*v),
            _ => Err(format!("@{}: {} 番目は数", self.name, i + 1)),
        }
    }
}

/// 1 行の頭から `@name(args)` を続けて読む。残りの文字列(属性でない部分)を返す。
fn parse_attrs<'a>(mut text: &'a str, out: &mut Vec<Attr>) -> Result<&'a str, String> {
    loop {
        text = text.trim_start();
        let Some(rest) = text.strip_prefix('@') else { return Ok(text) };
        let end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(rest.len());
        let name = rest[..end].to_owned();
        if name.is_empty() {
            return Err("@ の後に名前が無い".into());
        }
        let mut args = Vec::new();
        text = &rest[end..];
        if let Some(mut inner) = text.trim_start().strip_prefix('(') {
            loop {
                inner = inner.trim_start();
                if let Some(after) = inner.strip_prefix(')') {
                    text = after;
                    break;
                }
                if let Some(s) = inner.strip_prefix('"') {
                    let mut value = String::new();
                    let mut chars = s.char_indices();
                    let close = loop {
                        match chars.next() {
                            Some((_, '\\')) => { if let Some((_, c)) = chars.next() { value.push(c); } }
                            Some((i, '"')) => break i,
                            Some((_, c)) => value.push(c),
                            None => return Err(format!("@{name}: 文字列が閉じていない")),
                        }
                    };
                    args.push(AttrArg::Str(value));
                    inner = &s[close + 1..];
                } else {
                    let end = inner.find(|c: char| c == ',' || c == ')' || c.is_whitespace()).unwrap_or(inner.len());
                    let word = &inner[..end];
                    if word.is_empty() {
                        return Err(format!("@{name}: 引数が読めない"));
                    }
                    args.push(match wgsl_number(word) {
                        Some(v) => AttrArg::Num(v),
                        None => AttrArg::Word(word.to_owned()),
                    });
                    inner = &inner[end..];
                }
                inner = inner.trim_start();
                if let Some(after) = inner.strip_prefix(',') {
                    inner = after;
                } else if !inner.starts_with(')') {
                    return Err(format!("@{name}: `,` か `)` が要る"));
                }
            }
        }
        out.push(Attr { name, args });
    }
}

/// WGSL の数の字句(`3.0` `-80.0` `1u` `2i` `0.5f`)を f64 に。
fn wgsl_number(word: &str) -> Option<f64> {
    let core = word.strip_suffix(['u', 'i', 'f', 'h']).unwrap_or(word);
    core.parse::<f64>().ok()
}

fn title_case(stem: &str) -> String {
    stem.split('_').filter(|w| !w.is_empty()).map(|w| {
        let mut c = w.chars();
        c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
    }).collect::<Vec<_>>().join(" ")
}

/// override の札(頭の JSON が無い .wgsl で `fn block(` を持つ物)を読み、manifest と、WESL に渡せる本文にする。
/// wgsl-parse 0.4.2 の字句に文字列が無い(`@label("Shape")` は lex できない)ので、頭はここで読んで剥がす:
/// - 属性の並びは次の `override` か `fn block(` に付く。空行で切れた並びは file の属性(`@id` `@label` `@description` `@rounds` `@scope` `@physics`)。
///   同じ物を `fn block` に付けてもよい。
/// - `override name: ty = init;` は欄 1 つ(宣言順)。`f32` = float、`u32` / `i32` = long、`bool` = bool。`@label` `@range(min, max)`
///   `@options("A", "B")` `@reach`。行は `var<private> name: ty = init;` に書き換え、外枠が毎コマ uniform から代入する
///   (`motolii_block_inputs()`)— 本文は欄を名前のまま読める。
pub(crate) fn wgsl_manifest(name: &str, source: &str) -> Result<(IsfManifest, String), String> {
    use crate::render::compositor::effects::isf::{IsfInput, IsfInputType, IsfScope, IsfStage, TimeBase, TimeSource};
    if source.contains("BlockParams") {
        return Err("override の札は `fn block(k: u32) -> Offset`(`p: BlockParams` は頭の JSON の札だけ)".into());
    }
    let mut manifest = IsfManifest { stage: IsfStage::Block, ..Default::default() };
    let mut pending: Vec<Attr> = Vec::new();
    let mut file_attrs: Vec<Attr> = Vec::new();
    let mut body = String::new();
    let mut assigns = String::new();
    let mut found_block = false;
    for line in source.lines() {
        let mut text = line.trim_start();
        let mut owned = String::new();
        if text.starts_with('@') {
            let rest = parse_attrs(text, &mut pending)?;
            if rest.trim().is_empty() {
                continue;
            }
            owned = rest.to_owned();
            text = &owned;
        }
        if let Some(decl) = text.strip_prefix("override ") {
            let decl = decl.trim().strip_suffix(';').ok_or_else(|| format!("{name}: override は `;` で終える"))?;
            let (lhs, init) = decl.split_once('=').ok_or_else(|| format!("{name}: override {decl}: 既定値(`= …`)が要る"))?;
            let (ident, ty) = lhs.split_once(':').ok_or_else(|| format!("{name}: override {decl}: 型(`: f32`)が要る"))?;
            let (ident, ty, init) = (ident.trim(), ty.trim(), init.trim());
            let (kind, wgsl_ty) = match ty {
                "f32" => (IsfInputType::Float, "f32"),
                "u32" => (IsfInputType::Long, "u32"),
                "i32" => (IsfInputType::Long, "i32"),
                "bool" => (IsfInputType::Bool, "bool"),
                other => return Err(format!("{name}: override {ident}: 欄の型は f32 / u32 / i32 / bool({other})")),
            };
            let default = match init {
                "true" => 1.0,
                "false" => 0.0,
                v => wgsl_number(v).ok_or_else(|| format!("{name}: override {ident}: 既定値は数か true / false({v})"))?,
            } as f32;
            let mut input = IsfInput {
                name: ident.to_owned(), label: None, labels: None, ty: kind, default: [default, 0.0, 0.0, 0.0], min: None, max: None, maps: None,
                subtype: None, advanced: false, hero: false, time_offset: None, time_base: TimeBase::Offset, layer_field: None, time_source: TimeSource::Own,
            };
            for a in pending.drain(..) {
                match a.name.as_str() {
                    "label" => input.label = Some(a.str(0)?.to_owned()),
                    "range" => {
                        input.min = Some([a.num(0)? as f32, 0.0, 0.0, 0.0]);
                        input.max = Some([a.num(1)? as f32, 0.0, 0.0, 0.0]);
                    }
                    "options" => input.labels = Some((0..a.args.len()).map(|i| a.str(i).map(str::to_owned)).collect::<Result<_, _>>()?),
                    "reach" => manifest.reach = Some(ident.to_owned()),
                    other => return Err(format!("{name}: override {ident}: 知らない属性 @{other}(label / range / options / reach)")),
                }
            }
            let slot = manifest.inputs.len();
            let read = format!("block_params[{}][{}]", slot / 4, slot % 4);
            let cast = match wgsl_ty { "f32" => read, "bool" => format!("({read} != 0.0)"), t => format!("{t}({read})") };
            assigns.push_str(&format!("    {ident} = {cast};\n"));
            manifest.inputs.push(input);
            body.push_str(&format!("var<private> {ident}: {wgsl_ty} = {init};\n"));
            continue;
        }
        if text.starts_with("fn block(") {
            found_block = true;
            file_attrs.append(&mut pending);
        } else if text.is_empty() && !pending.is_empty() {
            file_attrs.append(&mut pending);
        }
        body.push_str(line);
        body.push('\n');
    }
    file_attrs.append(&mut pending);
    if !found_block {
        return Err(format!("{name}: `fn block(k: u32) -> Offset` が無い"));
    }
    for a in file_attrs {
        match a.name.as_str() {
            "id" => manifest.id = Some(a.str(0)?.to_owned()),
            "label" => manifest.label = Some(a.str(0)?.to_owned()),
            "description" => manifest.description = Some(a.str(0)?.to_owned()),
            "rounds" => manifest.rounds = (a.num(0)? as u32).clamp(1, 256),
            "scope" => manifest.scope = match a.str(0)? {
                "room" => IsfScope::Room,
                "members" => IsfScope::Members,
                other => return Err(format!("{name}: @scope は \"members\" か \"room\"({other})")),
            },
            "physics" => {
                let value = match a.args.get(1) {
                    Some(AttrArg::Str(s)) => serde_json::Value::String(s.clone()),
                    Some(AttrArg::Num(v)) => serde_json::json!(*v),
                    Some(AttrArg::Word(w)) if w == "true" || w == "false" => serde_json::Value::Bool(w == "true"),
                    _ => return Err(format!("{name}: @physics(\"KEY\", value) の value は文字列か数")),
                };
                manifest.physics.insert(a.str(0)?.to_owned(), value);
            }
            other => return Err(format!("{name}: 知らない属性 @{other}(id / label / description / rounds / scope / physics)")),
        }
    }
    if manifest.id.is_none() {
        manifest.id = Some(format!("motolii.{name}"));
    }
    if manifest.label.is_none() {
        manifest.label = Some(title_case(name));
    }
    body.push_str(&format!("\nfn motolii_block_inputs() {{\n{assigns}}}\n"));
    Ok((manifest, body))
}

/// 描く device が持っている物(まだ device が無ければ WebGPU の地の物だけ)。`u64::MAX` = まだ見ていない印。
static DEVICE_CAPABILITIES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(u64::MAX);

/// 描く側の device が出来たら、その持ち物を棚の検証へ渡す(棚が組まれるのは device の後)。
/// 写す規則は自前で持たない — 上流(`wgpu_naga_bridge`、wgpu 本体が pipeline を組む時に使う物)をそのまま借りる。
/// downlevel は native の compliant(Metal / Vulkan / DX12)。
pub(crate) fn note_device(device: &wgpu::Device) {
    let caps = wgpu_naga_bridge::features_to_naga_capabilities(device.features(), wgpu::DownlevelFlags::compliant());
    DEVICE_CAPABILITIES.store(caps.bits(), std::sync::atomic::Ordering::Relaxed);
}

/// 棚の検証が使う持ち物。`Capabilities::all()` は嘘で、device に無い機能(f16・subgroup・ray query)の札が
/// 棚に載ってしまい、描く時に pipeline を組む所で落ちる。
fn device_capabilities() -> naga::valid::Capabilities {
    match DEVICE_CAPABILITIES.load(std::sync::atomic::Ordering::Relaxed) {
        u64::MAX => naga::valid::Capabilities::default(),
        bits => naga::valid::Capabilities::from_bits_truncate(bits),
    }
}

/// 計算シェーダーとして通るかを naga で確かめる(棚に載せる前)。
pub(crate) fn validate(source: &str) -> Result<(), String> {
    let module = naga::front::wgsl::parse_str(source).map_err(|e| e.emit_to_string(source))?;
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), device_capabilities())
        .validate(&module)
        .map_err(|e| e.to_string())?;
    if !module.entry_points.iter().any(|e| e.name == "motolii_block_main" && e.stage == naga::ShaderStage::Compute) {
        return Err("block: 外枠が組めない".into());
    }
    Ok(())
}
