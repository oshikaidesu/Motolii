//! 箱のブロック(`"STAGE": "block"`): 物の箱の並びと、全員の今のずれ(state)を GPU の storage buffer に置き、
//! ブロックが state を少しずつ書き換える計算シェーダー(提案 2026-09-15、利用者「わたしの推奨は gpu」「この天井を作るべきでない」)。
//!
//! 作者の file は `fn block(k: u32) -> Offset` と、欄 1 つにつき module 階の `override` 1 本(`@label` `@range` `@options` `@reach`)、
//! 札の名は `fn block` か file の頭の `@id` `@label` `@description`(`@rounds` `@scope` `@physics` も)。頭の JSON(`/*{ "STAGE": "block" }*/`、
//! `fn block(k: u32, p: BlockParams)`)も移行中は通る。`k` は物の番号、返すのは物の component へ足す分:
//! 位置(足す)・回転(足す)・大きさ(掛ける)・色と不透明 `tint`(掛ける、rgb が色の倍率、a が不透明の倍率)。
//! 描く側はこれを物ごとの motion として読む(頂点で位置・回転・大きさ、画素で色と不透明)。時間のずれは別の口(書類の Stagger)。
//! 読める物: `objects[k]`(箱・住む箱・間合い・重み・組)、`host`(時刻・物の数・何回目)、`now_lo(k)` / `now_hi(k)`(今のずれ込みの箱)、
//! `neighbor_count(k)` / `neighbor(k, i)`(同じ組で近くに居る物。全員を回らずに済む — 物の数に天井を作らない)。
//! 欄の struct と、掛かった物(`members`)全員に掛ける外枠、ずれを state へ足す所はここが manifest から組む。
//! `"ROUNDS": n` なら n 回続けて解く(毎回、前の回の state を読む)。

use super::isf::IsfManifest;

/// 欄の slot の数(uniform の `array<vec4f, 6>`)。
pub(crate) const PARAM_SLOTS: usize = 24;

/// 1 つの物の箱(comp の座標、描く時と同じ置き方)。`room_*` は住む箱(並べる Group の箱)。
/// `group` は同じ住む箱に居る物の印(押し合いは同じ組の中だけ)、`margin` は間合い、`weight` は譲る比。
/// `parent_slot` / `anchor_slot` は名指しの相手の物の番号(親の層 / Position Anchor の層、無ければ `NO_OBJECT`)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockItem {
    pub lo: [f32; 2],
    pub hi: [f32; 2],
    pub room_lo: [f32; 2],
    pub room_size: [f32; 2],
    pub radius: f32,
    pub group: u32,
    pub margin: f32,
    pub weight: f32,
    pub parent_slot: u32,
    pub anchor_slot: u32,
}

/// 相手が居ない印(WGSL の `NO_OBJECT`)。
pub const NO_OBJECT: u32 = u32::MAX;

impl Default for BlockItem {
    fn default() -> Self {
        Self { lo: [0.0; 2], hi: [0.0; 2], room_lo: [0.0; 2], room_size: [0.0; 2], radius: 0.0, group: 0, margin: 0.0, weight: 0.0, parent_slot: NO_OBJECT, anchor_slot: NO_OBJECT }
    }
}

/// 物ごとのずれ(位置の差・回転の差(度)・大きさの倍率・色と不透明の倍率 `tint` = rgb + a)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockOffset {
    pub translate: [f32; 2],
    pub rotate: f32,
    pub scale: f32,
    pub tint: [f32; 4],
}

impl Default for BlockOffset {
    fn default() -> Self {
        Self { translate: [0.0; 2], rotate: 0.0, scale: 1.0, tint: [1.0; 4] }
    }
}

/// 1 つの物の byte(WGSL の `Item` と同じ並び)。
pub(crate) const ITEM_BYTES: u64 = 56;
pub(crate) const OFFSET_BYTES: u64 = 32;

pub(crate) fn item_bytes(items: &[BlockItem]) -> Vec<u8> {
    let mut out = Vec::with_capacity(items.len() * ITEM_BYTES as usize);
    for it in items {
        for v in [it.lo[0], it.lo[1], it.hi[0], it.hi[1], it.room_lo[0], it.room_lo[1], it.room_size[0], it.room_size[1], it.radius] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.extend_from_slice(&it.group.to_le_bytes());
        out.extend_from_slice(&it.margin.to_le_bytes());
        out.extend_from_slice(&it.weight.to_le_bytes());
        out.extend_from_slice(&it.parent_slot.to_le_bytes());
        out.extend_from_slice(&it.anchor_slot.to_le_bytes());
    }
    out
}

pub(crate) fn offset_bytes(offsets: &[BlockOffset]) -> Vec<u8> {
    offsets.iter().flat_map(|o| [o.translate[0], o.translate[1], o.rotate, o.scale, o.tint[0], o.tint[1], o.tint[2], o.tint[3]]).flat_map(f32::to_le_bytes).collect()
}

pub(crate) fn offsets_from_bytes(bytes: &[u8]) -> Vec<BlockOffset> {
    bytes.chunks_exact(OFFSET_BYTES as usize).map(|c| {
        let f = |i: usize| f32::from_le_bytes([c[i], c[i + 1], c[i + 2], c[i + 3]]);
        BlockOffset { translate: [f(0), f(4)], rotate: f(8), scale: f(12), tint: [f(16), f(20), f(24), f(28)] }
    }).collect()
}

/// 近くに居る物の一覧(CSR: `starts[k]..starts[k + 1]` が物 k の相手)。同じ組の物を、組の一番大きい箱(+ 両側の届く距離 `reach`)の 2 倍の升目に振り、
/// 周り 3×3 の升目の物を相手にする。物の数に比例する(全組を回らない)。押し合いで升目より遠くへ動く物は相手を取りこぼしうる。
pub(crate) fn neighbors(items: &[BlockItem], reach: f32) -> (Vec<u32>, Vec<u32>) {
    use std::collections::HashMap;
    let mut extent: HashMap<u32, f32> = HashMap::new();
    for it in items {
        let e = (it.hi[0] - it.lo[0]).abs().max((it.hi[1] - it.lo[1]).abs()) + 2.0 * it.margin.max(reach);
        let slot = extent.entry(it.group).or_insert(1.0);
        *slot = slot.max(e);
    }
    let cell_of = |it: &BlockItem| {
        let size = extent[&it.group] * 2.0;
        let c = [(it.lo[0] + it.hi[0]) * 0.5, (it.lo[1] + it.hi[1]) * 0.5];
        ((c[0] / size).floor() as i64, (c[1] / size).floor() as i64)
    };
    let mut cells: HashMap<(u32, i64, i64), Vec<u32>> = HashMap::new();
    for (k, it) in items.iter().enumerate() {
        let (x, y) = cell_of(it);
        cells.entry((it.group, x, y)).or_default().push(k as u32);
    }
    let mut starts = Vec::with_capacity(items.len() + 1);
    let mut list = Vec::new();
    for (k, it) in items.iter().enumerate() {
        starts.push(list.len() as u32);
        let (x, y) = cell_of(it);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(bucket) = cells.get(&(it.group, x + dx, y + dy)) {
                    list.extend(bucket.iter().copied().filter(|j| *j != k as u32));
                }
            }
        }
    }
    starts.push(list.len() as u32);
    (starts, list)
}

/// 作者も外枠も同じ型と束ねを読む。
pub(crate) const PRELUDE: &str = "struct Item { lo: vec2f, hi: vec2f, room_lo: vec2f, room_size: vec2f, radius: f32, group: u32, margin: f32, weight: f32, parent_slot: u32, anchor_slot: u32 };\n\
struct Offset { translate: vec2f, rotate: f32, scale: f32, tint: vec4f };\n\
struct BlockHost { time: f32, members: u32, objects: u32, round: u32, source: u32 };\n\
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
         \x20   if gid.x >= host.members {{ return; }}\n\
         \x20   let k = members[gid.x];\n\
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
    use super::isf::{IsfInput, IsfInputType, IsfScope, IsfStage, TimeBase, TimeSource};
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

/// 計算シェーダーとして通るかを naga で確かめる(棚に載せる前)。
pub(crate) fn validate(source: &str) -> Result<(), String> {
    let module = naga::front::wgsl::parse_str(source).map_err(|e| e.emit_to_string(source))?;
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module)
        .map_err(|e| e.to_string())?;
    if !module.entry_points.iter().any(|e| e.name == "motolii_block_main" && e.stage == naga::ShaderStage::Compute) {
        return Err("block: 外枠が組めない".into());
    }
    Ok(())
}

fn storage(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only }, has_dynamic_offset: false, min_binding_size: None },
        count: None,
    }
}

fn uniform(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
        count: None,
    }
}

fn pipeline(device: &wgpu::Device, label: &str, source: &str, entries: &[wgpu::BindGroupLayoutEntry], entry: &str) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some(label), source: wgpu::ShaderSource::Wgsl(source.into()) });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some(label), entries });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some(label), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some(entry),
        compilation_options: Default::default(),
        cache: None,
    });
    (pipeline, layout)
}

/// 1 本のブロックの GPU の道。
pub(crate) struct BlockProgram {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    pub(crate) rounds: u32,
}

/// 1 コマの物の世界(全員の箱・state の 2 本・掛かった物の番号)。buffer は物の数が増えた時だけ作り直す。
pub(crate) struct BlockWorld {
    capacity: u64,
    objects: wgpu::Buffer,
    states: [wgpu::Buffer; 2],
    neighbor_starts: wgpu::Buffer,
    neighbor_list: wgpu::Buffer,
    /// 今の state が `states` のどちらか。
    pub(crate) current: usize,
    pub(crate) count: u32,
}

impl BlockProgram {
    pub(crate) fn new(device: &wgpu::Device, label: &str, source: &str, rounds: u32) -> Self {
        let (pipeline, layout) = pipeline(device, label, source, &[storage(0, true), storage(1, true), uniform(2), uniform(3), storage(4, false), storage(5, true), storage(6, true), storage(7, true)], "motolii_block_main");
        Self { pipeline, layout, rounds: rounds.max(1) }
    }

    /// 掛かった物(`members`)に `rounds` 回掛ける。毎回、前の state を読み、写してから書く(掛からない物はそのまま)。
    pub(crate) fn record(&self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, time: f32, members: &[u32], params: &[f32]) {
        self.record_from(device, queue, encoder, world, time, members, params, u32::MAX);
    }

    /// 場(`SCOPE: room`)は元の物の番号を `host.source` で渡す。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_from(&self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, time: f32, members: &[u32], params: &[f32], source: u32) {
        if members.is_empty() || world.count == 0 {
            return;
        }
        // 掛かった物の番号も同じ理由でブロックごとに別の buffer。
        let members_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-members"), size: (members.len() * 4) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&members_buffer, 0, &members.iter().flat_map(|m| m.to_le_bytes()).collect::<Vec<u8>>());
        let mut slots = [0.0f32; PARAM_SLOTS];
        for (slot, value) in slots.iter_mut().zip(params) {
            *slot = *value;
        }
        // uniform の書き込みは submit の時にまとめて届き、同じ buffer は最後の値が勝つ。ブロックごとの欄は別の buffer で渡す。
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-params"), size: 96, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&params_buffer, 0, &slots.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        let bytes = u64::from(world.count) * OFFSET_BYTES;
        for round in 0..self.rounds {
            // uniform は submit 前の最後の書き込みが勝つので、回ごとに違う host は別の小さな buffer で渡す。
            let host = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-host"), size: 32, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            let mut h = [0u8; 32];
            h[0..4].copy_from_slice(&time.to_le_bytes());
            h[4..8].copy_from_slice(&(members.len() as u32).to_le_bytes());
            h[8..12].copy_from_slice(&world.count.to_le_bytes());
            h[12..16].copy_from_slice(&round.to_le_bytes());
            h[16..20].copy_from_slice(&source.to_le_bytes());
            queue.write_buffer(&host, 0, &h);
            let (from, to) = (world.current, 1 - world.current);
            encoder.copy_buffer_to_buffer(&world.states[from], 0, &world.states[to], 0, bytes);
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("motolii-block"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: world.objects.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: world.states[from].as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: host.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: params_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 4, resource: world.states[to].as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 5, resource: members_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 6, resource: world.neighbor_starts.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 7, resource: world.neighbor_list.as_entire_binding() },
                ],
            });
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("motolii-block"), timestamp_writes: None });
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups((members.len() as u32).div_ceil(64), 1, 1);
            }
            world.current = to;
        }
    }
}

impl BlockWorld {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let buffer = |size: u64, usage| device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-world"), size, usage, mapped_at_creation: false });
        let state = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
        Self {
            capacity: 1,
            objects: buffer(ITEM_BYTES, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST),
            states: [buffer(OFFSET_BYTES, state), buffer(OFFSET_BYTES, state)],
            neighbor_starts: buffer(8, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST),
            neighbor_list: buffer(4, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST),
            current: 0,
            count: 0,
        }
    }

    /// このコマの物の箱を置き、state を 0(ずれ無し)から始める。
    pub(crate) fn begin(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, items: &[BlockItem], reach: f32) {
        self.begin_from(device, queue, items, reach, &[]);
    }

    /// 始まりの state を渡して始める(外の解き手 — Rapier — が出したずれを土台にする)。
    pub(crate) fn begin_from(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, items: &[BlockItem], reach: f32, seed: &[BlockOffset]) {
        let count = items.len().max(1) as u64;
        if count > self.capacity {
            self.capacity = count.next_power_of_two();
            let buffer = |size: u64, usage| device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-world"), size, usage, mapped_at_creation: false });
            let state = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
            self.objects = buffer(self.capacity * ITEM_BYTES, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST);
            self.states = [buffer(self.capacity * OFFSET_BYTES, state), buffer(self.capacity * OFFSET_BYTES, state)];
        }
        self.count = items.len() as u32;
        self.current = 0;
        if !items.is_empty() {
            queue.write_buffer(&self.objects, 0, &item_bytes(items));
            let mut start = vec![BlockOffset::default(); items.len()];
            for (slot, value) in start.iter_mut().zip(seed) {
                *slot = *value;
            }
            queue.write_buffer(&self.states[0], 0, &offset_bytes(&start));
        }
        let (starts, list) = neighbors(items, reach);
        let storage = |bytes: &[u8]| {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-neighbors"), size: bytes.len().max(4) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            queue.write_buffer(&buffer, 0, bytes);
            buffer
        };
        self.neighbor_starts = storage(&starts.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        self.neighbor_list = storage(&list.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
    }

    pub(crate) fn state(&self) -> &wgpu::Buffer {
        &self.states[self.current]
    }
}

/// 付いて行く(付いて置く札の相手がブロックで動いた分だけ、札も動く): 対 (札, 相手) ごとに、札の state に相手の state を足す。
/// 1 回で 1 段(札の札は次の回)。
pub(crate) struct FollowPass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

const FOLLOW_PASS: &str = "struct Offset { translate: vec2f, rotate: f32, scale: f32, tint: vec4f };\n\
@group(0) @binding(0) var<storage, read> state_in: array<Offset>;\n\
@group(0) @binding(1) var<storage, read_write> state_out: array<Offset>;\n\
@group(0) @binding(2) var<storage, read> pairs: array<vec2u>;\n\
@group(0) @binding(3) var<uniform> count: vec4u;\n\
@compute @workgroup_size(64)\n\
fn main(@builtin(global_invocation_id) gid: vec3u) {\n\
    if gid.x >= count.x { return; }\n\
    let pair = pairs[gid.x];\n\
    let own = state_in[pair.x];\n\
    state_out[pair.x] = Offset(own.translate + state_in[pair.y].translate, own.rotate, own.scale, own.tint);\n\
}\n";

impl FollowPass {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let (pipeline, layout) = pipeline(device, "motolii-block-follow", FOLLOW_PASS, &[storage(0, true), storage(1, false), storage(2, true), uniform(3)], "main");
        Self { pipeline, layout }
    }

    pub(crate) fn record(&self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, pairs: &[(u32, u32)]) {
        if pairs.is_empty() || world.count == 0 {
            return;
        }
        let pair_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-follow-pairs"), size: (pairs.len() * 8) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&pair_buffer, 0, &pairs.iter().flat_map(|(a, b)| [a.to_le_bytes(), b.to_le_bytes()].concat()).collect::<Vec<u8>>());
        let count = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-follow-count"), size: 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&count, 0, &[(pairs.len() as u32).to_le_bytes(), [0; 4], [0; 4], [0; 4]].concat());
        let (from, to) = (world.current, 1 - world.current);
        encoder.copy_buffer_to_buffer(&world.states[from], 0, &world.states[to], 0, u64::from(world.count) * OFFSET_BYTES);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-block-follow"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: world.states[from].as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: world.states[to].as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: pair_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: count.as_entire_binding() },
            ],
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("motolii-block-follow"), timestamp_writes: None });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((pairs.len() as u32).div_ceil(64), 1, 1);
        }
        world.current = to;
    }
}

/// state(comp のずれ)を、物ごとの comp → world の向きで world のずれにして描く側の motion へ書く(物ごと vec4 × 4:
/// 位置と回り、真ん中と大きさ、軸、色と不透明)。
pub(crate) struct WorldPass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

const WORLD_PASS: &str = r#"struct Offset { translate: vec2f, rotate: f32, scale: f32, tint: vec4f };
struct Basis { u: vec4f, v: vec4f, centre: vec4f };
@group(0) @binding(0) var<storage, read> bases: array<Basis>;
@group(0) @binding(1) var<storage, read> state: array<Offset>;
@group(0) @binding(2) var<storage, read_write> motion: array<vec4f>;
@group(0) @binding(3) var<uniform> count: vec4u;
@group(0) @binding(4) var<storage, read> links: array<vec2u>;
fn world_offset(k: u32) -> vec3f {
    let t = state[k].translate;
    return bases[k].u.xyz * t.x + bases[k].v.xyz * t.y;
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let k = gid.x;
    if k >= count.x + count.y { return; }
    var base = k * 4u;
    if k >= count.x {
        base = (count.x + (k - count.x) * 2u) * 4u;
    }
    if base + 4u > arrayLength(&motion) { return; }
    if k >= count.x {
        // A connector: entry of kind 1 — offset at A, A, B − A, offset at B (see re_renderer motion_offset).
        let link = links[k - count.x];
        let a = bases[link.x].centre.xyz;
        let b = bases[link.y].centre.xyz;
        motion[base] = vec4f(world_offset(link.x), 0.0);
        motion[base + 1u] = vec4f(a, 1.0);
        motion[base + 2u] = vec4f(b - a, 1.0);
        motion[base + 3u] = vec4f(world_offset(link.y), 1.0);
        return;
    }
    let u = bases[k].u.xyz;
    let v = bases[k].v.xyz;
    let n = cross(u, v);
    let axis = select(vec3f(0.0, 0.0, 1.0), n / length(n), length(n) > 1e-12);
    motion[base] = vec4f(world_offset(k), radians(state[k].rotate));
    motion[base + 1u] = vec4f(bases[k].centre.xyz, state[k].scale);
    motion[base + 2u] = vec4f(axis, 0.0);
    motion[base + 3u] = state[k].tint;
}
"#;

impl WorldPass {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let (pipeline, layout) = pipeline(device, "motolii-block-world-pass", WORLD_PASS, &[storage(0, true), storage(1, true), storage(2, false), uniform(3), storage(4, true)], "main");
        Self { pipeline, layout }
    }

    /// `bases` は物ごとの (comp の x の 1px の world, y の 1px の world, 物の真ん中の world)。真ん中は回る軸が通る所。
    /// `links` はつなぐ線ごとの (元の物, 先の物): 物の後ろに 2 項ずつ(kind 1 は前の 1 項、紐は 2 項)両端のずれを書く。
    /// 返すのは bases の buffer(紐の解きが同じ物を読む)。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(&self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &BlockWorld, bases: &[([f32; 3], [f32; 3], [f32; 3])], links: &[(u32, u32)], motion: &wgpu::Buffer) -> Option<wgpu::Buffer> {
        if world.count == 0 {
            return None;
        }
        let basis = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-bases"), size: (bases.len().max(1) * 48) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&basis, 0, &bases.iter().flat_map(|(u, v, c)| [u[0], u[1], u[2], 0.0, v[0], v[1], v[2], 0.0, c[0], c[1], c[2], 0.0]).flat_map(f32::to_le_bytes).collect::<Vec<u8>>());
        let count = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-count"), size: 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&count, 0, &[world.count.to_le_bytes(), (links.len() as u32).to_le_bytes(), [0; 4], [0; 4]].concat());
        let link_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-links"), size: (links.len().max(1) * 8) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&link_buffer, 0, &links.iter().flat_map(|(a, b)| [a.to_le_bytes(), b.to_le_bytes()].concat()).collect::<Vec<u8>>());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-block-world-pass"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: basis.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: world.state().as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: motion.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: count.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: link_buffer.as_entire_binding() },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("motolii-block-world-pass"), timestamp_writes: None });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups((world.count + links.len() as u32).div_ceil(64), 1, 1);
        drop(pass);
        Some(basis)
    }
}

/// 紐(Line Path = Rope): つなぐ線の 3 次ベジェの腹の 2 点だけがばね + 減衰で遅れて付いて来る(利用者 2026-09-18
/// 「紐が物理の挙動になれば」「ベジェでいけないの?」)。状態は紐 1 本に 2 点の位置と速度、GPU に置いたまま
/// (読み戻さない)。描く側の motion の kind 2 の項(つなぐ線の 2 つ目の項)に、描いた時の腹と今の腹を書く。
pub(crate) struct RopePass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    states: [wgpu::Buffer; 2],
    capacity: u64,
    current: usize,
    /// 最後に解いたコマ(続きのコマでなければ静止から始める)。
    pub(crate) last_frame: Option<i64>,
}

const ROPE_PASS: &str = r#"struct Offset { translate: vec2f, rotate: f32, scale: f32, tint: vec4f };
struct Basis { u: vec4f, v: vec4f, centre: vec4f };
struct Host { objects: u32, ropes: u32, dt: f32, reset: u32 };
@group(0) @binding(0) var<storage, read> bases: array<Basis>;
@group(0) @binding(1) var<storage, read> state: array<Offset>;
@group(0) @binding(2) var<storage, read_write> motion: array<vec4f>;
@group(0) @binding(3) var<uniform> host: Host;
@group(0) @binding(4) var<storage, read> links: array<vec2u>;
@group(0) @binding(5) var<storage, read> ropes: array<vec4f>;
@group(0) @binding(6) var<storage, read> rope_in: array<vec4f>;
@group(0) @binding(7) var<storage, read_write> rope_out: array<vec4f>;
fn world_offset(k: u32) -> vec3f {
    let t = state[k].translate;
    return bases[k].u.xyz * t.x + bases[k].v.xyz * t.y;
}
fn controls(a: vec3f, b: vec3f, down: vec3f, sag: f32) -> array<vec3f, 2> {
    let d = b - a;
    return array<vec3f, 2>(a + d / 3.0 + down * sag, a + d * (2.0 / 3.0) + down * sag);
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let r = gid.x;
    if r >= host.ropes { return; }
    let rope = ropes[r];
    let c = u32(rope.x);
    let link = links[c];
    let a = bases[link.x].centre.xyz;
    let b = bases[link.y].centre.xyz;
    let down = normalize(bases[link.x].v.xyz);
    let sag = length(b - a) * rope.y / 100.0;
    let drawn = controls(a, b, down, sag);
    let la = a + world_offset(link.x);
    let lb = b + world_offset(link.y);
    let rest = controls(la, lb, down, sag);
    var p0 = rope_in[r * 4u].xyz;
    var v0 = rope_in[r * 4u + 1u].xyz;
    var p1 = rope_in[r * 4u + 2u].xyz;
    var v1 = rope_in[r * 4u + 3u].xyz;
    if host.reset == 1u {
        p0 = rest[0]; v0 = vec3f(0.0); p1 = rest[1]; v1 = vec3f(0.0);
    }
    // ばね(硬さ rope.z)と減衰(rope.w)、4 回に刻む。時刻の関数ではなく解き手の側(記憶を持つ)。
    let dt = host.dt / 4.0;
    for (var i = 0u; i < 4u; i = i + 1u) {
        v0 = v0 + (rest[0] - p0) * rope.z * dt - v0 * rope.w * dt;
        p0 = p0 + v0 * dt;
        v1 = v1 + (rest[1] - p1) * rope.z * dt - v1 * rope.w * dt;
        p1 = p1 + v1 * dt;
    }
    rope_out[r * 4u] = vec4f(p0, 0.0);
    rope_out[r * 4u + 1u] = vec4f(v0, 0.0);
    rope_out[r * 4u + 2u] = vec4f(p1, 0.0);
    rope_out[r * 4u + 3u] = vec4f(v1, 0.0);
    let base = (host.objects + c * 2u) * 4u;
    if base + 8u > arrayLength(&motion) { return; }
    motion[base + 2u] = vec4f(b - a, 2.0);
    motion[base + 4u] = vec4f(drawn[0], 0.0);
    motion[base + 5u] = vec4f(drawn[1], 0.0);
    motion[base + 6u] = vec4f(p0, 0.0);
    motion[base + 7u] = vec4f(p1, 0.0);
}
"#;

impl RopePass {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let (pipeline, layout) = pipeline(device, "motolii-rope-pass", ROPE_PASS, &[storage(0, true), storage(1, true), storage(2, false), uniform(3), storage(4, true), storage(5, true), storage(6, true), storage(7, false)], "main");
        let buffer = |device: &wgpu::Device, size: u64| device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-rope-state"), size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        Self { pipeline, layout, states: [buffer(device, 64), buffer(device, 64)], capacity: 1, current: 0, last_frame: None }
    }

    /// `ropes` は (つなぐ線の番号, Slack %, 硬さ, 減衰)。`frame` が前の続きでなければ静止から始める。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &BlockWorld, bases_buffer: &wgpu::Buffer, links: &[(u32, u32)], ropes: &[(u32, f32, f32, f32)], motion: &wgpu::Buffer, frame: i64, fps: f32) {
        if ropes.is_empty() || world.count == 0 {
            self.last_frame = None;
            return;
        }
        let count = ropes.len() as u64;
        let mut reset = self.last_frame != Some(frame - 1);
        if count > self.capacity {
            self.capacity = count.next_power_of_two();
            let buffer = |size: u64| device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-rope-state"), size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            self.states = [buffer(self.capacity * 64), buffer(self.capacity * 64)];
            reset = true;
        }
        self.last_frame = Some(frame);
        let link_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-rope-links"), size: (links.len().max(1) * 8) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&link_buffer, 0, &links.iter().flat_map(|(a, b)| [a.to_le_bytes(), b.to_le_bytes()].concat()).collect::<Vec<u8>>());
        let rope_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-ropes"), size: count * 16, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&rope_buffer, 0, &ropes.iter().flat_map(|(c, slack, k, d)| [(*c as f32).to_le_bytes(), slack.to_le_bytes(), k.to_le_bytes(), d.to_le_bytes()].concat()).collect::<Vec<u8>>());
        let host = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-rope-host"), size: 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&host, 0, &[world.count.to_le_bytes(), (count as u32).to_le_bytes(), (1.0 / fps.max(1.0)).to_le_bytes(), (reset as u32).to_le_bytes()].concat());
        let (from, to) = (self.current, 1 - self.current);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-rope-pass"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: bases_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: world.state().as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: motion.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: host.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: link_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 5, resource: rope_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 6, resource: self.states[from].as_entire_binding() },
                wgpu::BindGroupEntry { binding: 7, resource: self.states[to].as_entire_binding() },
            ],
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("motolii-rope-pass"), timestamp_writes: None });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((count as u32).div_ceil(64), 1, 1);
        }
        self.current = to;
    }
}

/// 試験と計測の口: state を読み戻す(描く道は読み戻さない)。
pub fn read_state(device: &wgpu::Device, queue: &wgpu::Queue, world: &BlockWorld, encoder: wgpu::CommandEncoder) -> Vec<BlockOffset> {
    let mut encoder = encoder;
    let bytes = u64::from(world.count) * OFFSET_BYTES;
    let staging = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-read"), size: bytes, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    encoder.copy_buffer_to_buffer(world.state(), 0, &staging, 0, bytes);
    queue.submit([encoder.finish()]);
    staging.slice(..).map_async(wgpu::MapMode::Read, |r| r.unwrap());
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let out = offsets_from_bytes(&staging.slice(..).get_mapped_range());
    staging.unmap();
    out
}

/// test の入口: 棚に載った札(disk の構成では vism/ の今の file、焼き込みでは埋めた物 — 描く時と同じ道)。
#[cfg(test)]
pub(crate) fn catalog_definition(name: &str) -> super::VismDefinition {
    super::catalog::catalog_snapshot().definitions.iter().find(|d| d.source.name == name).unwrap_or_else(|| panic!("{name} is not on the shelf")).clone()
}

#[cfg(test)]
pub(crate) fn program_for(device: &wgpu::Device, name: &str) -> BlockProgram {
    let definition = catalog_definition(name);
    assert_eq!(definition.manifest.stage, super::isf::IsfStage::Block);
    BlockProgram::new(device, "block-test", &definition.vertex_text, definition.manifest.rounds)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gpu() -> crate::render::engine::Engine {
        crate::render::engine::Engine::new().unwrap()
    }

    /// Bounce を GPU のブロックに移しても、doc の CPU の折り返し(`layout::bounced`)と同じずれになる。
    #[test]
    fn the_bounce_block_folds_boxes_like_the_cpu_law() {
        let engine = gpu();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let program = program_for(device, "bounce");
        let m = 1.0; // doc の CANVAS_MARGIN(箱の素材座標の原点)
        let mut items = Vec::new();
        let mut rng = 7u32;
        let mut next = || { rng = rng.wrapping_mul(1664525).wrapping_add(1013904223); (rng >> 8) as f32 / (1u32 << 24) as f32 };
        for index in 0..4000u32 {
            let (size, radius) = if index % 2 == 0 { ([900.0, 520.0], 40.0) } else { ([420.0, 420.0], 210.0) };
            let w = 10.0 + next() * 60.0;
            let x = m - 800.0 + next() * 2400.0;
            let y = m - 800.0 + next() * 2400.0;
            items.push(BlockItem { lo: [x, y], hi: [x + w, y + w], room_lo: [m, m], room_size: size, radius, group: 0, margin: 0.0, weight: 1.0, ..Default::default() });
        }
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items, 0.0);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        let members: Vec<u32> = (0..items.len() as u32).collect();
        program.record(device, queue, &mut encoder, &mut world, 0.0, &members, &[1.0]);
        let gpu = read_state(device, queue, &world, encoder);
        let error = pollster::block_on(scope.pop());
        assert!(error.is_none(), "the block pipeline and its buffers validate: {error:?}");
        for (k, (item, offset)) in items.iter().zip(&gpu).enumerate() {
            let cpu = crate::doc::store::layout::bounced(item.lo, item.hi, item.room_size, item.radius);
            let d = (offset.translate[0] - cpu[0]).abs().max((offset.translate[1] - cpu[1]).abs());
            assert!(d < 0.05, "item {k}: gpu {:?} cpu {cpu:?}", offset.translate);
        }
    }

    /// 口: ブロックは位置のずれだけでなく、物の component(回転・大きさ・色と不透明)へ書ける。
    /// 位置と回転は足す、大きさと色は掛ける — 同じブロックを 2 回掛けると 2 倍でなく 2 乗になる(棚の札は要らない、test の中の式)。
    #[test]
    fn a_block_writes_scale_and_tint_and_they_compose_by_multiplying() {
        let engine = gpu();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let source = "/*{ \"ID\": \"test.dim\", \"STAGE\": \"block\", \"INPUTS\": [ {\"NAME\":\"opacity\",\"TYPE\":\"float\",\"DEFAULT\":0.5} ] }*/\n\
            fn block(k: u32, p: BlockParams) -> Offset { return Offset(vec2f(1.0, 0.0), 10.0, 0.5, vec4f(1.0, 0.5, 0.25, p.opacity)); }";
        let (manifest, body) = super::super::isf::parse_isf_source(source).unwrap();
        let full = module_source(&manifest, &body, &[]).unwrap();
        validate(&full).unwrap();
        let program = BlockProgram::new(device, "block-test-dim", &full, 1);
        let items = [BlockItem { lo: [0.0, 0.0], hi: [10.0, 10.0], room_lo: [0.0, 0.0], room_size: [100.0, 100.0], radius: 0.0, group: 0, margin: 0.0, weight: 1.0, ..Default::default() }];
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items, 0.0);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        program.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[0.5]);
        program.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[0.5]);
        let state = read_state(device, queue, &world, encoder);
        assert!(pollster::block_on(scope.pop()).is_none());
        let o = state[0];
        assert_eq!(o.translate, [2.0, 0.0], "position adds");
        assert_eq!(o.rotate, 20.0, "rotation adds");
        assert_eq!(o.scale, 0.25, "scale multiplies");
        assert_eq!(o.tint, [1.0, 0.25, 0.0625, 0.25], "tint and opacity multiply");
    }

    /// 名指しの口(利用者 2026-09-19、CSS の `anchor()` / AE の parent): `parent(k)` / `anchor(k)` は相手の今の Offset
    /// (同じコマで先に走った block の結果込み)、相手が居なければ `NO_OFFSET`。同じ stage でも読まれる側が先に走る。
    #[test]
    fn a_block_reads_its_parents_and_anchors_current_offset_and_none_without_one() {
        let engine = gpu();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let build = |name: &str, source: &str| {
            let (manifest, body) = wgsl_manifest(name, source).unwrap();
            let full = module_source(&manifest, &body, &[]).unwrap();
            validate(&full).unwrap();
            BlockProgram::new(device, name, &full, 1)
        };
        let mover = build("mover", "fn block(k: u32) -> Offset { return Offset(vec2f(5.0, -3.0), 12.0, 2.0, vec4f(0.5, 1.0, 1.0, 1.0)); }");
        let child = build("child", "fn block(k: u32) -> Offset { let p = parent(k); return Offset(p.translate, p.rotate, 1.0, vec4f(1.0)); }");
        let anchored = build("anchored", "fn block(k: u32) -> Offset { let a = anchor(k); return Offset(a.translate, 0.0, a.scale, a.tint); }");
        let items = [
            BlockItem { lo: [0.0, 0.0], hi: [10.0, 10.0], ..Default::default() },
            BlockItem { lo: [20.0, 0.0], hi: [30.0, 10.0], parent_slot: 0, ..Default::default() },
            BlockItem { lo: [40.0, 0.0], hi: [50.0, 10.0], anchor_slot: 0, ..Default::default() },
            BlockItem { lo: [60.0, 0.0], hi: [70.0, 10.0], ..Default::default() },
        ];
        assert_eq!(item_bytes(&items).len(), items.len() * ITEM_BYTES as usize, "the host packs what the WGSL Item declares");
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items, 0.0);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        mover.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[]);
        child.record(device, queue, &mut encoder, &mut world, 0.0, &[1, 3], &[]);
        anchored.record(device, queue, &mut encoder, &mut world, 0.0, &[2], &[]);
        let state = read_state(device, queue, &world, encoder);
        assert!(pollster::block_on(scope.pop()).is_none());
        assert_eq!((state[1].translate, state[1].rotate), ([5.0, -3.0], 12.0), "the child moves and turns with its parent");
        assert_eq!((state[2].translate, state[2].scale, state[2].tint), ([5.0, -3.0], 2.0, [0.5, 1.0, 1.0, 1.0]), "the anchored thing reads the hub's whole offset");
        assert_eq!(state[3], BlockOffset::default(), "a thing with neither parent nor anchor reads NO_OFFSET");
    }

    /// override の札: 欄は `override`(`@label` `@range` `@options`)、本文は名前のまま読む。manifest は JSON の頭と同じ形になり、
    /// 組んだ計算シェーダーは naga を通り、JSON の札(上の test)と同じ結果を出す。
    #[test]
    fn an_override_block_declares_its_inputs_in_wgsl() {
        use super::super::isf::{IsfInputType, IsfScope};
        let source = "@id(\"test.dim\") @description(\"dims\")\n\
            @rounds(2) @scope(\"room\") @physics(\"ROOM\", \"ancestor\") @physics(\"SUBSTEPS\", 4)\n\n\
            @label(\"Opacity\") @range(0.0, 1.0)\n\
            override opacity: f32 = 0.5;\n\
            @label(\"Shape\") @options(\"Sphere\", \"Box\", \"In Out\")\n\
            override shape: u32 = 1;\n\
            @reach\n\
            override reach: f32 = -80.0;\n\
            override lit: bool = true;\n\n\
            // a comment between\n\
            fn block(k: u32) -> Offset {\n\
                var a = opacity;\n\
                if shape != 1u || !lit { a = 0.0; }\n\
                return Offset(vec2f(1.0, 0.0), 10.0, 0.5, vec4f(1.0, 0.5, 0.25, a));\n}";
        let (manifest, body) = wgsl_manifest("dim_test", source).unwrap();
        assert_eq!(manifest.id.as_deref(), Some("test.dim"));
        assert_eq!(manifest.label.as_deref(), Some("Dim Test"), "label falls back to the file stem in Title Case");
        assert_eq!(manifest.description.as_deref(), Some("dims"));
        assert_eq!((manifest.rounds, manifest.scope, manifest.reach.as_deref()), (2, IsfScope::Room, Some("reach")));
        assert_eq!(manifest.physics.get("ROOM").and_then(|v| v.as_str()), Some("ancestor"));
        assert_eq!(manifest.physics.get("SUBSTEPS").and_then(|v| v.as_f64()), Some(4.0));
        let inputs: Vec<_> = manifest.inputs.iter().map(|i| (i.name.as_str(), i.label.as_deref(), i.ty, i.default[0], i.min.map(|m| m[0]), i.max.map(|m| m[0]), i.labels.clone())).collect();
        assert_eq!(inputs, [
            ("opacity", Some("Opacity"), IsfInputType::Float, 0.5, Some(0.0), Some(1.0), None),
            ("shape", Some("Shape"), IsfInputType::Long, 1.0, None, None, Some(vec!["Sphere".into(), "Box".into(), "In Out".into()])),
            ("reach", None, IsfInputType::Float, -80.0, None, None, None),
            ("lit", None, IsfInputType::Bool, 1.0, None, None, None),
        ]);
        assert!(body.contains("var<private> shape: u32 = 1;") && body.contains("shape = u32(block_params[0][1]);") && body.contains("lit = (block_params[0][3] != 0.0);"), "{body}");
        assert!(!body.contains('@'), "no Motolii attribute survives into the WESL text: {body}");
        let full = module_source(&manifest, &body, &[]).unwrap();
        validate(&full).unwrap();
        let engine = gpu();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let program = BlockProgram::new(device, "block-test-override", &full, 1);
        let items = [BlockItem { lo: [0.0, 0.0], hi: [10.0, 10.0], room_lo: [0.0, 0.0], room_size: [100.0, 100.0], radius: 0.0, group: 0, margin: 0.0, weight: 1.0, ..Default::default() }];
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items, 0.0);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        program.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[0.5, 1.0, 0.0, 1.0]);
        program.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[0.5, 1.0, 0.0, 1.0]);
        let state = read_state(device, queue, &world, encoder);
        assert!(pollster::block_on(scope.pop()).is_none());
        let o = state[0];
        assert_eq!((o.translate, o.rotate, o.scale, o.tint), ([2.0, 0.0], 20.0, 0.25, [1.0, 0.25, 0.0625, 0.25]), "same as the JSON-head block");
        assert!(wgsl_manifest("x", "override a: f32 = 1.0;\nfn main() {}").is_err(), "no fn block, no 札");
        assert!(wgsl_manifest("x", "@range(0.0, 1.0)\noverride a: f32 = 1.0;\n@rounds(3)\nfn block(k: u32) -> Offset { return NO_OFFSET; }").is_ok(), "attributes on fn block are the file's");
    }

    /// 描く側へ渡す motion は物ごと vec4 × 4: (位置, 回り)・(真ん中, 大きさ)・(軸, _)・(色, 不透明)。大きさと色が落ちずに届く。
    #[test]
    fn the_world_pass_hands_scale_and_tint_to_the_drawing_side() {
        let engine = gpu();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let items = [BlockItem { lo: [0.0, 0.0], hi: [10.0, 10.0], room_lo: [0.0, 0.0], room_size: [100.0, 100.0], radius: 0.0, group: 0, margin: 0.0, weight: 1.0, ..Default::default() }];
        let mut world = BlockWorld::new(device);
        world.begin_from(device, queue, &items, 0.0, &[BlockOffset { translate: [3.0, 0.0], rotate: 0.0, scale: 0.5, tint: [1.0, 0.5, 0.25, 0.5] }]);
        let motion = device.create_buffer(&wgpu::BufferDescriptor { label: Some("test-motion"), size: 64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        let _ = WorldPass::new(device).record(device, queue, &mut encoder, &world, &[([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [5.0, 5.0, 0.0])], &[], &motion);
        let staging = device.create_buffer(&wgpu::BufferDescriptor { label: Some("test-read"), size: 64, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        encoder.copy_buffer_to_buffer(&motion, 0, &staging, 0, 64);
        queue.submit([encoder.finish()]);
        staging.slice(..).map_async(wgpu::MapMode::Read, |r| r.unwrap());
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        let words: Vec<f32> = staging.slice(..).get_mapped_range().chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
        staging.unmap();
        assert_eq!(&words[0..3], &[3.0, 0.0, 0.0], "offset in world");
        assert_eq!(&words[4..8], &[5.0, 5.0, 0.0, 0.5], "centre, then scale");
        assert_eq!(&words[12..16], &[1.0, 0.5, 0.25, 0.5], "tint, then opacity");
    }

    /// 紐: つなぐ線の 2 つ目の項に、描いた時の腹(端の間の 1/3・2/3 を Slack だけ下げた点)と今の腹が書かれ、種は 2。
    /// 端が動いていなければ今の腹は描いた腹と同じ(静止から始める)。
    #[test]
    fn the_rope_pass_writes_the_bellies_of_a_cubic_behind_the_things() {
        let engine = gpu();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let items = [
            BlockItem { lo: [0.0, 0.0], hi: [10.0, 10.0], room_lo: [0.0, 0.0], room_size: [100.0, 100.0], radius: 0.0, group: 0, margin: 0.0, weight: 1.0, ..Default::default() },
            BlockItem { lo: [30.0, 0.0], hi: [40.0, 10.0], room_lo: [0.0, 0.0], room_size: [100.0, 100.0], radius: 0.0, group: 0, margin: 0.0, weight: 1.0, ..Default::default() },
        ];
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items, 0.0);
        let bases = [([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]), ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [30.0, 0.0, 0.0])];
        let motion = device.create_buffer(&wgpu::BufferDescriptor { label: Some("test-motion"), size: 4 * 64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        let basis = WorldPass::new(device).record(device, queue, &mut encoder, &world, &bases, &[(0, 1)], &motion).unwrap();
        RopePass::new(device).record(device, queue, &mut encoder, &world, &basis, &[(0, 1)], &[(0, 20.0, 60.0, 6.0)], &motion, 0, 30.0);
        let staging = device.create_buffer(&wgpu::BufferDescriptor { label: Some("test-read"), size: 4 * 64, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        encoder.copy_buffer_to_buffer(&motion, 0, &staging, 0, 4 * 64);
        queue.submit([encoder.finish()]);
        staging.slice(..).map_async(wgpu::MapMode::Read, |r| r.unwrap());
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        let w: Vec<f32> = staging.slice(..).get_mapped_range().chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
        staging.unmap();
        let v = |i: usize| [w[i * 4], w[i * 4 + 1], w[i * 4 + 2], w[i * 4 + 3]];
        assert_eq!(v(9), [0.0, 0.0, 0.0, 1.0], "A, then the connector's own flag");
        assert_eq!(v(10), [30.0, 0.0, 0.0, 2.0], "B − A, kind 2 = rope");
        assert_eq!(v(12)[..3], [10.0, 6.0, 0.0], "drawn c0: a third along, sagging by 20 % of the span");
        assert_eq!(v(13)[..3], [20.0, 6.0, 0.0], "drawn c1");
        assert_eq!(v(14)[..3], [10.0, 6.0, 0.0], "live c0 starts at rest");
        assert_eq!(v(15)[..3], [20.0, 6.0, 0.0], "live c1 starts at rest");
    }

    /// 棚の block の一覧(切り分け用): 今日置いた札が段 Block で載っているか。
    #[test]
    fn todays_packages_are_blocks_on_the_shelf() {
        let catalog = crate::render::compositor::effects::catalog::catalog_snapshot();
        let blocks: Vec<(String, String)> = catalog.definitions.iter().filter(|d| d.manifest.stage == super::super::isf::IsfStage::Block).map(|d| (d.source.name.clone(), d.plugin_id().to_owned())).collect();
        eprintln!("blocks: {blocks:?}\nerrors: {:?}", catalog.errors);
        for name in ["concentrick", "falloff_reveal", "formations", "zero_gravity", "wave"] {
            assert!(blocks.iter().any(|(n, _)| n == name), "{name} is not a block on the shelf: {blocks:?} / {:?}", catalog.errors);
        }
    }

    /// ブロックは fx と同じ棚に、段 Block として並ぶ(置き場・頭の JSON・読み直しが同じ)。
    #[test]
    fn a_block_sits_on_the_shelf_like_an_effect() {
        let refresh = crate::render::compositor::effects::catalog::refresh_effect_catalog();
        assert!(!refresh.errors.iter().any(|e| ["bounce", "push_apart", "four_color_gradient", "inner_shadow"].iter().any(|n| e.contains(n))), "{:?}", refresh.errors);
        let catalog = crate::render::compositor::effects::catalog::catalog_snapshot();
        let bounce = catalog.descriptors.iter().find(|d| d.plugin_id == "motolii.bounce_block").expect("on the shelf");
        assert_eq!(bounce.stage, crate::render::compositor::effects::catalog::EffectStage::Block);
        assert_eq!(bounce.params.iter().map(|p| p.label.as_str()).collect::<Vec<_>>(), ["Strength"]);
        for fx in ["motolii.four_color_gradient", "motolii.inner_shadow"] {
            assert!(catalog.descriptors.iter().any(|d| d.plugin_id == fx), "{fx} on the shelf");
        }
    }
}
