//! 貼られた GLSL を ISF の言い方へ写す。
//!
//! 変換器は書かない — GLSL → WGSL は ISF と同じ naga の道(`compile_glsl_to_wgsl`)を通る。
//! ここがやるのは方言の前口上だけ: Shadertoy の `mainImage` と `i*` の名前を、ISF の
//! `main` ・ `RENDERSIZE` ・宣言された入力へ結び直す。

/// 貼られた文字列の方言。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dialect {
    /// `/*{ ... }*/` の manifest を持つ。そのまま通す。
    Isf,
    /// `void mainImage(out vec4, in vec2)` を持つ。
    Shadertoy,
    /// Shadertoy の書き出し(JSON): Image + Buffer A..D + Common のタブ。
    Project,
    /// `void main()` を持つ素の fragment shader。
    Glsl,
}

pub fn dialect(source: &str) -> Option<Dialect> {
    if source.trim_start().starts_with('{') && source.contains("\"renderpass\"") {
        return Some(Dialect::Project);
    }
    let stripped = strip_comments(source);
    if source.contains("/*{") && source.contains("\"INPUTS\"") || source.contains("\"ISFVSN\"") {
        return Some(Dialect::Isf);
    }
    if stripped.contains("mainImage") {
        return Some(Dialect::Shadertoy);
    }
    if stripped.contains("void main") {
        return Some(Dialect::Glsl);
    }
    None
}

/// Shadertoy の名前のうち、ここで用意できない物。見つけたら黙って壊れず、名前を返して断る。
const UNSUPPORTED: [&str; 2] = ["iChannelResolution", "iChannelTime"];

/// Shadertoy の 1 file を ISF source(`/*{...}*/` + GLSL)へ写す。
///
/// `iChannel0..3` は使われている物だけを image 入力として宣言する。`iTime` は `SUBTYPE: TIME` の
/// 欄になり、host が時刻を流す。`iMouse` は使われていれば point2D の欄になる。
pub fn isf_from_shadertoy(id: &str, label: &str, source: &str) -> Result<String, String> {
    let body = strip_comments(source);
    if !body.contains("mainImage") {
        return Err("Shadertoy の shader には void mainImage(out vec4, in vec2) が要る".into());
    }
    if let Some(name) = UNSUPPORTED.iter().find(|name| body.contains(**name)) {
        return Err(format!("{name} はまだ写せない(Shadertoy の入力ごとの寸法・時刻)"));
    }

    let channels: Vec<String> = (0..4)
        .map(|n| format!("iChannel{n}"))
        .filter(|name| body.contains(name.as_str()))
        .collect();
    let mouse = body.contains("iMouse");

    let mut inputs: Vec<String> = channels
        .iter()
        .map(|name| format!("    {{ \"NAME\": \"{name}\", \"LABEL\": \"{name}\", \"TYPE\": \"image\" }}"))
        .collect();
    if mouse {
        inputs.push("    { \"NAME\": \"iMousePoint\", \"LABEL\": \"Mouse\", \"TYPE\": \"point2D\", \"DEFAULT\": [0.0, 0.0] }".into());
    }

    let mut out = String::new();
    out.push_str(&format!(
        "/*{{\n  \"ID\": \"{id}\",\n  \"LABEL\": \"{label}\",\n  \"STAGE\": \"pass\",\n  \"DESCRIPTION\": \"Imported from Shadertoy\",\n  \"INPUTS\": [\n{}\n  ]\n}}*/\n\n",
        inputs.join(",\n")
    ));
    // 名前の結び直し。Shadertoy は画素の座標(左下原点)、ISF は 0..1 の座標。
    // Shadertoy の座標は下端が 0、wgpu の texture は上端が 0。座標は Shadertoy の作法のまま渡し、
    // 読む時だけ裏返す(macro は自分自身へ展開し直されないので、中の texture は組み込みのまま)。
    out.push_str("#define texture(smp, coord) texture(smp, vec2((coord).x, 1.0 - (coord).y))\n");
    out.push_str("#define iResolution vec3(RENDERSIZE, 1.0)\n");
    // 時計はホストの uniform(ISF の TIME 系)。comp の時刻と fps がそのまま来る。
    out.push_str("#define iTime TIME\n");
    out.push_str("#define iTimeDelta TIMEDELTA\n");
    out.push_str("#define iFrameRate (1.0 / TIMEDELTA)\n");
    out.push_str("#define iFrame FRAMEINDEX\n");
    out.push_str("#define iDate DATE\n");
    out.push_str("#define iSampleRate 44100.0\n");
    if mouse {
        out.push_str("#define iMouse vec4(iMousePoint, 0.0, 0.0)\n");
    }
    out.push('\n');
    out.push_str(source.trim_end());
    out.push_str("\n\nvoid main() {\n    mainImage(gl_FragColor, isf_FragNormCoord * RENDERSIZE);\n}\n");
    Ok(out)
}

/// Shadertoy の書き出し(JSON)を ISF の 1 file へ写す。タブは 1 つの shader に並び、`PASSINDEX` で選ぶ。
///
/// - `Buffer A..D` → `PERSISTENT` な target(Shadertoy の buffer は前のフレームを保つ)。持ち主は host(feedback)
/// - `Image` → 最後の pass(出力)
/// - `Common` → 全部の前に 1 度
/// - 各タブの `iChannelN` は、その入力が buffer ならその target の名前に、texture なら層の絵(`inputImage`)に
///   書き換える(タブごとに違う結び方なので `#define` でなく識別子を置き換える)
/// - keyboard / music / webcam / video / cubemap の入力は写せないので名指しで断る
/// - タブは Shadertoy では別々の翻訳単位: 同じ名前の関数が 2 つのタブに在れば断る(1 つに並べると衝突する)
pub fn isf_from_shadertoy_project(id: &str, label: &str, json: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("Shadertoy の JSON が読めない: {e}"))?;
    let root = value.get("Shader").unwrap_or(&value);
    let passes = root.get("renderpass").and_then(|v| v.as_array()).ok_or("Shadertoy の JSON に renderpass が無い")?;
    let text = |v: &serde_json::Value, key: &str| v.get(key).and_then(|x| x.as_str()).unwrap_or("").to_owned();
    let kind = |v: &serde_json::Value| { let t = text(v, "type"); if t.is_empty() { text(v, "ctype") } else { t } };
    let mut common = String::new();
    let mut buffers: Vec<(String, String, &serde_json::Value)> = Vec::new(); // (target, code, pass)
    let mut image: Option<(String, &serde_json::Value)> = None;
    let mut target_by_output: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for pass in passes {
        let name = text(pass, "name");
        match kind(pass).as_str() {
            "common" => { common.push_str(&text(pass, "code")); common.push('\n'); }
            "buffer" => {
                let letter = name.trim().rsplit(' ').next().unwrap_or("").to_ascii_uppercase();
                let target = if letter.len() == 1 { format!("buffer{letter}") } else { format!("buffer{}", buffers.len()) };
                for output in pass.get("outputs").and_then(|v| v.as_array()).into_iter().flatten() {
                    target_by_output.insert(output.get("id").map(|v| v.to_string()).unwrap_or_default(), target.clone());
                }
                buffers.push((target, text(pass, "code"), pass));
            }
            "image" => image = Some((text(pass, "code"), pass)),
            other => return Err(format!("{name}: {other} のタブは写せない(Image・Buffer・Common だけ)")),
        }
    }
    let (image_code, image_pass) = image.ok_or("Shadertoy の JSON に Image のタブが無い")?;
    let mut ordered: Vec<(Option<String>, String, &serde_json::Value)> = buffers.into_iter().map(|(t, c, p)| (Some(t), c, p)).collect();
    ordered.push((None, image_code, image_pass));

    // タブごとに: iChannelN を結び直し、mainImage に番号を付ける。関数名の衝突を見張る。
    let mut body = String::new();
    let mut defined: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for name in top_level_functions(&strip_comments(&common)) {
        defined.insert(name, "Common".to_owned());
    }
    let mut dispatch = String::new();
    for (k, (target, code, pass)) in ordered.iter().enumerate() {
        let tab = text(pass, "name");
        let stripped = strip_comments(code);
        if let Some(name) = UNSUPPORTED.iter().find(|name| stripped.contains(**name)) {
            return Err(format!("{tab}: {name} はまだ写せない"));
        }
        let mut code = code.clone();
        for n in 0..4 {
            let channel = format!("iChannel{n}");
            if !stripped.contains(channel.as_str()) { continue; }
            let input = pass.get("inputs").and_then(|v| v.as_array()).into_iter().flatten()
                .find(|i| i.get("channel").and_then(|c| c.as_u64()) == Some(n));
            let Some(input) = input else { return Err(format!("{tab}: {channel} に何も繋がっていない")); };
            let bound = match kind(input).as_str() {
                "buffer" => target_by_output.get(&input.get("id").map(|v| v.to_string()).unwrap_or_default()).cloned()
                    .ok_or_else(|| format!("{tab}: {channel} が指す buffer が無い"))?,
                "texture" => "inputImage".to_owned(),
                other => return Err(format!("{tab}: {channel} の {other} は写せない(buffer と texture だけ)")),
            };
            code = replace_word(&code, &channel, &bound);
        }
        let entry = format!("mainImage_{k}");
        code = replace_word(&code, "mainImage", &entry);
        for name in top_level_functions(&strip_comments(&code)) {
            if name == entry { continue; }
            if let Some(where_) = defined.get(&name) {
                return Err(format!("{tab}: 関数 {name} は {where_} にも在る — Shadertoy のタブは別々だが、ここでは 1 つに並ぶ。片方を改名する"));
            }
            defined.insert(name, tab.clone());
        }
        body.push_str(&format!("\n// ---- {tab} ----\n{code}\n"));
        dispatch.push_str(&format!("    {}if (PASSINDEX == {k}) {entry}(gl_FragColor, isf_FragNormCoord * RENDERSIZE);\n", if k == 0 { "" } else { "else " }));
        let _ = target;
    }

    let mouse = body.contains("iMouse");
    let mut inputs = vec!["    { \"NAME\": \"inputImage\", \"TYPE\": \"image\" }".to_owned()];
    if mouse {
        inputs.push("    { \"NAME\": \"iMousePoint\", \"LABEL\": \"Mouse\", \"TYPE\": \"point2D\", \"DEFAULT\": [0.0, 0.0] }".into());
    }
    let passes_json: Vec<String> = ordered.iter().map(|(target, _, _)| match target {
        Some(t) => format!("    {{ \"TARGET\": \"{t}\", \"PERSISTENT\": true, \"FLOAT\": true }}"),
        None => "    { }".to_owned(),
    }).collect();
    let mut out = String::new();
    out.push_str(&format!(
        "/*{{\n  \"ID\": \"{id}\",\n  \"LABEL\": \"{label}\",\n  \"STAGE\": \"pass\",\n  \"DESCRIPTION\": \"Imported from Shadertoy (tabs: {} buffers + Image)\",\n  \"INPUTS\": [\n{}\n  ],\n  \"PASSES\": [\n{}\n  ]\n}}*/\n\n",
        ordered.len() - 1, inputs.join(",\n"), passes_json.join(",\n")
    ));
    out.push_str("#define texture(smp, coord) texture(smp, vec2((coord).x, 1.0 - (coord).y))\n");
    out.push_str("#define iResolution vec3(RENDERSIZE, 1.0)\n");
    out.push_str("#define iTime TIME\n#define iTimeDelta TIMEDELTA\n#define iFrameRate (1.0 / TIMEDELTA)\n#define iFrame FRAMEINDEX\n#define iDate DATE\n#define iSampleRate 44100.0\n");
    if mouse { out.push_str("#define iMouse vec4(iMousePoint, 0.0, 0.0)\n"); }
    out.push_str("\n// ---- Common ----\n");
    out.push_str(&common);
    out.push_str(&body);
    out.push_str(&format!("\nvoid main() {{\n{dispatch}}}\n"));
    Ok(out)
}

/// 識別子を語の境で置き換える(`iChannel0` を `iChannel01` の中では触らない)。
fn replace_word(code: &str, word: &str, with: &str) -> String {
    let bytes = code.as_bytes();
    let mut out = String::with_capacity(code.len());
    let mut i = 0;
    let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    while i < bytes.len() {
        if bytes[i..].starts_with(word.as_bytes())
            && (i == 0 || !is_word(bytes[i - 1]))
            && (i + word.len() >= bytes.len() || !is_word(bytes[i + word.len()]))
        {
            out.push_str(with);
            i += word.len();
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

/// タブの最上位の関数定義の名前(`型 名前(引数) {`)。
fn top_level_functions(stripped: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut line_start = 0usize;
    let bytes = stripped.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'{' => {
                if depth == 0 {
                    let head = &stripped[line_start..i];
                    if let Some(open) = head.find('(') {
                        let before = head[..open].trim_end();
                        if let Some(name) = before.rsplit(|c: char| c.is_whitespace() || c == '*').next() {
                            if !name.is_empty() && before.split_whitespace().count() >= 2 {
                                out.push(name.to_owned());
                            }
                        }
                    }
                }
                depth += 1;
            }
            b'}' => { depth -= 1; if depth == 0 { line_start = i + 1; } }
            b';' if depth == 0 => line_start = i + 1,
            _ => {}
        }
    }
    out
}

/// 注釈を外した本文(ISF 側の時計の判定も使う)。
pub(super) fn strip_comments_pub(source: &str) -> String { strip_comments(source) }

/// 注釈を外した本文。名前を探す時に、注釈の中の語で誤判定しないため。
fn strip_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else if bytes[i..].starts_with(b"/*") {
            i += 2;
            while i < bytes.len() && !bytes[i..].starts_with(b"*/") {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shadertoy の作法そのまま(画素座標・iTime・iResolution)。
    const PLAIN: &str = r#"
void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 uv = fragCoord / iResolution.xy;
    float d = length(uv - 0.5);
    vec3 col = vec3(0.5 + 0.5 * sin(iTime + d * 12.0), uv.x, uv.y);
    fragColor = vec4(col, 1.0);
}
"#;

    const WITH_CHANNEL: &str = r#"
void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 uv = fragCoord / iResolution.xy;
    fragColor = texture(iChannel0, uv).bgra;
}
"#;

    #[test]
    fn the_dialect_is_read_from_the_text() {
        assert_eq!(dialect(PLAIN), Some(Dialect::Shadertoy));
        assert_eq!(dialect("/*{ \"INPUTS\": [] }*/\nvoid main() {}"), Some(Dialect::Isf));
        assert_eq!(dialect("void main() { gl_FragColor = vec4(1.0); }"), Some(Dialect::Glsl));
        // 注釈の中の語では決めない。
        assert_eq!(dialect("// mainImage is not here\nint x;"), None);
    }

    #[test]
    fn a_pasted_shadertoy_compiles_to_wgsl() {
        let isf = isf_from_shadertoy("import.ripple", "Ripple", PLAIN).unwrap();
        let (manifest, _vertex, fragment) = super::super::compiled_stages(&isf).unwrap();
        assert_eq!(manifest.label.as_deref(), Some("Ripple"));
        // iTime は欄ではなくホストの時計。
        assert!(!manifest.inputs.iter().any(|i| i.name == "iTime"));
        assert!(manifest.uses_clock, "iTime を読む効果は時計を読む");
        assert!(fragment.contains("fn main"), "{fragment}");
    }

    #[test]
    fn a_used_channel_becomes_an_image_input() {
        let isf = isf_from_shadertoy("import.swap", "Swap", WITH_CHANNEL).unwrap();
        let (manifest, _vertex, _fragment) = super::super::compiled_stages(&isf).unwrap();
        let images: Vec<_> = manifest.inputs.iter().filter(|i| i.ty == crate::render::compositor::effects::IsfInputType::Image).map(|i| i.name.as_str()).collect();
        assert_eq!(images, ["iChannel0"]);
        // 使われていない channel は欄にしない。
        assert!(!manifest.inputs.iter().any(|i| i.name == "iChannel1"));
    }

    /// 上下の向き。ISF の座標は上端が 1、texture の v は上端が 0 なので、裏返さずに渡すと
    /// 素材を読む位置が上下逆になる(2026-09-12、実写で発覚 — 文字が裏返った)。
    /// 向きが正しいことの証明は恒等の絵(`texture(iChannel0, uv)` が元絵と完全一致)で取った。
    /// ここはその取り決めが消えないための見張り。
    #[test]
    fn the_picture_is_read_right_side_up() {
        let isf = isf_from_shadertoy("import.swap", "Swap", WITH_CHANNEL).unwrap();
        assert!(isf.contains("#define texture(smp, coord)"), "{isf}");
    }

    /// Shadertoy の書き出し(JSON): Buffer A が自分を読み(feedback)、Image が Buffer A を読む。
    const PROJECT: &str = r#"{"Shader":{"info":{"name":"trail"},"renderpass":[
      {"name":"Common","type":"common","code":"float half_of(float x) { return 0.5 * x; }","inputs":[],"outputs":[]},
      {"name":"Buffer A","type":"buffer","code":"void mainImage(out vec4 o, in vec2 c) { vec2 uv = c / iResolution.xy; o = mix(texture(iChannel0, uv), texture(iChannel1, uv), half_of(0.4)); }",
       "inputs":[{"id":257,"channel":0,"type":"buffer"},{"id":1,"channel":1,"type":"texture"}],"outputs":[{"id":257,"channel":0}]},
      {"name":"Image","type":"image","code":"void mainImage(out vec4 o, in vec2 c) { o = texture(iChannel0, c / iResolution.xy); }",
       "inputs":[{"id":257,"channel":0,"type":"buffer"}],"outputs":[]}
    ]}}"#;

    #[test]
    fn a_shadertoy_project_becomes_one_isf_with_persistent_buffers() {
        assert_eq!(dialect(PROJECT), Some(Dialect::Project));
        let isf = isf_from_shadertoy_project("import.trail", "Trail", PROJECT).unwrap();
        let (manifest, _v, fragment) = super::super::compiled_stages(&isf).unwrap();
        assert_eq!(manifest.passes.len(), 2);
        assert_eq!(manifest.passes[0].target.as_deref(), Some("bufferA"));
        assert!(manifest.passes[0].persistent, "Shadertoy の buffer は前のフレームを保つ");
        assert!(manifest.passes[1].target.is_none(), "Image が出力");
        // Buffer A の iChannel0 は自分(bufferA)、iChannel1 は層の絵。Image の iChannel0 は bufferA。
        assert!(isf.contains("texture(bufferA, uv)") && isf.contains("texture(inputImage, uv)"), "{isf}");
        assert!(isf.contains("mainImage_0") && isf.contains("mainImage_1") && isf.contains("PASSINDEX == 1"), "{isf}");
        assert!(fragment.contains("fn main"));
    }

    #[test]
    fn a_function_defined_in_two_tabs_is_refused_by_name() {
        let clashing = PROJECT.replace("void mainImage(out vec4 o, in vec2 c) { o = texture(iChannel0", "float half_of(float x) { return x; } void mainImage(out vec4 o, in vec2 c) { o = texture(iChannel0");
        let error = isf_from_shadertoy_project("import.x", "X", &clashing).unwrap_err();
        assert!(error.contains("half_of") && error.contains("Common"), "{error}");
    }

    #[test]
    fn an_input_that_cannot_be_supplied_is_refused_by_name() {
        let keyboard = PROJECT.replace("\"id\":1,\"channel\":1,\"type\":\"texture\"", "\"id\":1,\"channel\":1,\"type\":\"keyboard\"");
        let error = isf_from_shadertoy_project("import.x", "X", &keyboard).unwrap_err();
        assert!(error.contains("keyboard") && error.contains("iChannel1"), "{error}");
    }

    #[test]
    fn what_cannot_be_written_is_refused_by_name() {
        let source = PLAIN.replace("iTime", "iChannelTime[0]");
        let error = isf_from_shadertoy("import.x", "X", &source).unwrap_err();
        assert!(error.contains("iChannelTime"), "{error}");
    }
}
