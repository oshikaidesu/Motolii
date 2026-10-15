#!/usr/bin/env python3
# An ISF effect (Vidvox's Interactive Shader Format: a GLSL fragment shader with a JSON head of its knobs, ISF-Files is
# MIT) made an effect file in vgpu's manner (screen.h), by naga: the shader's own lines are kept; around them ISF's
# words (IMG_NORM_PIXEL, IMG_THIS_PIXEL, isf_FragNormCoord, TIME, RENDERSIZE...) are said in plain GLSL, the picture
# split into a texture and a sampler, the knobs gathered into one struct and written in the head.
# usage: from_isf.py Effect.fs out.wgsl   (naga on PATH)
# Refused: one with PASSES (pictures kept between passes or pictures: not yet), a vertex shader of its own, a second
# picture used, and those whose source is not MIT (v002's, Shadertoy's, VHS Glitch: research_notes/webgpu_looks).
import json, os, re, subprocess, sys, tempfile

src, out = sys.argv[1], sys.argv[2]
text = open(src).read()
head = json.loads(re.match(r"\s*/\*\s*(\{.*?\})\s*\*/", text, re.S).group(1))
body = text[text.index("*/") + 2:]
name = os.path.basename(src)[:-3]
credit = head.get("CREDIT", "")

if re.search(r"v002|shadertoy", name + " " + credit + " " + body[:400], re.I) or name in ("VHS Glitch", "Optical Flow Distort", "Optical Flow Generator", "Glow", "ASCII Art", "Smoke Screen"):
    sys.exit(f"{name}: not MIT at its source ({credit})")
if head.get("PASSES") or os.path.exists(src[:-3] + ".vs"):
    sys.exit(f"{name}: passes or a vertex shader of its own: not yet")

pictures = [i["NAME"] for i in head.get("INPUTS", []) if i["TYPE"] == "image"]
picture = "inputImage" if "inputImage" in pictures else (pictures[0] if pictures else None)
if not picture:
    sys.exit(f"{name}: makes a picture of its own (no image in): not an effect over the stage")
for other in pictures:
    if other != picture and re.search(rf"\b{other}\b", body):
        sys.exit(f"{name}: reads a second picture ({other})")

kinds = {"float": "float", "bool": "int", "long": "int", "event": "int", "point2D": "vec2", "color": "vec4"}
knobs = [i for i in head.get("INPUTS", []) if i["TYPE"] in kinds]
glsl = "#version 450\n"
glsl += "layout(set = 0, binding = 0) uniform texture2D isf_picture;\nlayout(set = 0, binding = 1) uniform sampler isf_sampler;\n"
glsl += "layout(set = 0, binding = 2) uniform Knobs {\n  float time;\n  vec2 resolution;\n  float frame;\n"
glsl += "".join(f"  {kinds[k['TYPE']]} {k['NAME']}{'_on' if k['TYPE'] in ('bool', 'event') else ''};\n" for k in knobs) + "};\n"
glsl += "".join(f"#define {k['NAME']} ({k['NAME']}_on != 0)\n" for k in knobs if k["TYPE"] in ("bool", "event"))
glsl += "layout(location = 0) in vec2 isf_FragNormCoord;\nlayout(location = 0) out vec4 isf_FragColor;\n"
glsl += f"#define {picture} sampler2D(isf_picture, isf_sampler)\n"
glsl += "#define TIME time\n#define RENDERSIZE resolution\n#define TIMEDELTA (1.0 / 60.0)\n#define FRAMEINDEX int(frame)\n#define PASSINDEX 0\n#define DATE vec4(0.0)\n"
glsl += "#define IMG_NORM_PIXEL(i, c) texture(i, c)\n#define IMG_PIXEL(i, c) texture(i, (c) / RENDERSIZE)\n"
glsl += "#define IMG_THIS_NORM_PIXEL(i) texture(i, isf_FragNormCoord)\n#define IMG_THIS_PIXEL(i) texture(i, isf_FragNormCoord)\n#define IMG_SIZE(i) RENDERSIZE\n"
glsl += "#define gl_FragColor isf_FragColor\n#define texture2D texture\n#define varying //\n"
glsl += "#define texture(i, c) textureLod(i, c, 0.0)\n"   # (WGSL samples with derivatives only where every pixel goes the same way: these read the top level anywhere)
glsl += re.sub(r"^\s*#version.*$", "", body, flags=re.M)

with tempfile.TemporaryDirectory() as d:
    f, w = os.path.join(d, "shader.frag"), os.path.join(d, "shader.wgsl")
    open(f, "w").write(glsl)
    run = subprocess.run(["naga", "--input-kind", "glsl", "--shader-stage", "frag", f, w], capture_output=True, text=True)
    if run.returncode:
        sys.exit(f"{name}: naga: {run.stderr.strip()[:600]}")
    wgsl = open(w).read()

lines = [f"// {name}: ISF-Files' {name}.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files{'; ' + credit if credit else ''}), its GLSL made WGSL by naga (filament/effects/from_isf.py)"]
for k in knobs:
    n, t, label = k["NAME"], k["TYPE"], k.get("LABEL", k["NAME"]).replace(",", " ")
    if t in ("bool", "event"):
        lines.append(f"// --check@{n}_on:{label},{int(bool(k.get('DEFAULT', 0)))}")
        continue
    usual, low, high = k.get("DEFAULT", 0), k.get("MIN", 0), k.get("MAX", 1)
    if t in ("float", "long"):
        lines.append(f"// --track@{n}:{label},{low},{high},{usual},{1 if t == 'long' else 0.001}")
    else:
        for i in range(2 if t == "point2D" else 4):
            pick = lambda v, f: v[i] if isinstance(v, list) and i < len(v) else f
            lines.append(f"// --track@{n}_{'xyzw'[i]}:{label} {'xyzw'[i] if t == 'point2D' else 'rgba'[i]},{pick(low, 0)},{pick(high, 1)},{pick(usual, 0)},0.001")
open(out, "w").write("\n".join(lines) + "\n\n" + wgsl)
print(f"wrote {out}: knobs {[k['NAME'] for k in knobs]}")
