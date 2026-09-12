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
    /// `void main()` を持つ素の fragment shader。
    Glsl,
}

pub fn dialect(source: &str) -> Option<Dialect> {
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
    inputs.push("    { \"NAME\": \"iTime\", \"LABEL\": \"Time\", \"TYPE\": \"float\", \"DEFAULT\": 0.0, \"SUBTYPE\": \"TIME\" }".into());
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
    out.push_str("#define iTimeDelta (1.0 / 60.0)\n");
    out.push_str("#define iFrameRate 60.0\n");
    out.push_str("#define iFrame int(iTime * 60.0)\n");
    out.push_str("#define iDate vec4(0.0)\n");
    out.push_str("#define iSampleRate 44100.0\n");
    if mouse {
        out.push_str("#define iMouse vec4(iMousePoint, 0.0, 0.0)\n");
    }
    out.push('\n');
    out.push_str(source.trim_end());
    out.push_str("\n\nvoid main() {\n    mainImage(gl_FragColor, isf_FragNormCoord * RENDERSIZE);\n}\n");
    Ok(out)
}

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
        // iTime は時刻の欄になる。
        let time = manifest.inputs.iter().find(|i| i.name == "iTime").expect("iTime input");
        assert_eq!(time.subtype.as_deref(), Some("TIME"));
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

    #[test]
    fn what_cannot_be_written_is_refused_by_name() {
        let source = PLAIN.replace("iTime", "iChannelTime[0]");
        let error = isf_from_shadertoy("import.x", "X", &source).unwrap_err();
        assert!(error.contains("iChannelTime"), "{error}");
    }
}
