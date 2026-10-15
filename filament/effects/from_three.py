#!/usr/bin/env python3
# A three.js screen shader (examples/jsm/shaders/*.js: uniforms, a GLSL fragmentShader reading tDiffuse at vUv) made an
# effect file in vgpu's manner (screen.h), by naga: the shader's own lines are kept; around them the picture is split
# into a texture and a sampler, the loose uniforms gathered into one struct, and the knobs written in the head.
# usage: from_three.py Shader.js out.wgsl ["licence line"]   (naga on PATH; a shader reading a second picture is refused)
import re, subprocess, sys, tempfile, os

src, out = sys.argv[1], sys.argv[2]
js = open(src).read()
frag = re.search(r"fragmentShader:\s*/\*\s*glsl\s*\*/\s*`(.*?)`", js, re.S).group(1)
usual = {m.group(1): [float(m.group(2))] for m in re.finditer(r"'(\w+)':\s*\{\s*value:\s*(-?[\d.]+)\s*\}", js)}
usual.update({m.group(1): [float(x) for x in m.group(2).split(",")] for m in re.finditer(r"'(\w+)':\s*\{\s*value:\s*new Vector[234]\(\s*([-\d., ]+?)\s*\)\s*\}", js)})
chunks = os.path.join(os.path.dirname(src), "../../../src/renderers/shaders/ShaderChunk")   # (three's own pieces a shader names by #include)
def chunk(name):
    return re.search(r"`(.*?)`", open(os.path.join(chunks, name + ".glsl.js")).read(), re.S).group(1)
common = re.search(r"^\s*#include\s*<common>", frag, re.M) is not None
frag = re.sub(r"^\s*#include\s*<(\w+)>\s*$", lambda m: "" if m.group(1) == "common" else chunk(m.group(1)), frag, flags=re.M)

loose = re.findall(r"^\s*uniform\s+(float|int|bool|vec2|vec3|vec4)\s+(\w+)\s*;", frag, re.M)
pictures = re.findall(r"^\s*uniform\s+sampler2D\s+(\w+)\s*;", frag, re.M)
if pictures != ['tDiffuse']:
    sys.exit(f"{src}: reads {pictures}: an effect reads one picture")
body = re.sub(r"^\s*uniform\s+\w+\s+\w+\s*;\s*$", "", frag, flags=re.M)
at = re.search(r"^\s*varying\s+vec2\s+(\w+)\s*;", body, re.M).group(1)   # (where on the picture: three's vUv, by whatever name)
body = re.sub(r"^\s*varying\s+vec2\s+\w+\s*;\s*$", "", body, flags=re.M)
glsl = "#version 450\n"
glsl += "layout(set = 0, binding = 0) uniform texture2D tDiffuse_picture;\n"
glsl += "layout(set = 0, binding = 1) uniform sampler tDiffuse_sampler;\n"
if loose:
    glsl += "layout(set = 0, binding = 2) uniform Knobs {\n" + "".join(f"  int {n}_on;\n" if t == "bool" else f"  {t} {n};\n" for t, n in loose) + "};\n"
    glsl += "".join(f"#define {n} ({n}_on != 0)\n" for t, n in loose if t == "bool")   # (a uniform holds no bool: an int, read as one)
glsl += f"layout(location = 0) in vec2 {at};\nlayout(location = 0) out vec4 three_FragColor;\n"
glsl += "#define tDiffuse sampler2D(tDiffuse_picture, tDiffuse_sampler)\n#define texture2D texture\n#define gl_FragColor three_FragColor\n"
glsl += "#define texture(i, c) textureLod(i, c, 0.0)\n"   # (WGSL samples with derivatives only where every pixel goes the same way: these read the top level anywhere)
if common:   # (the shader asks for three's common piece: rand, PI, saturate...)
    glsl += re.sub(r"^\s*#include.*$", "", chunk("common"), flags=re.M) + "\n"
if "luminance(" in body and "float luminance" not in body:   # (three puts this before every shader: WebGLProgram.js getLuminanceFunction, linear sRGB's weights)
    glsl += "float luminance(vec3 rgb) { return dot(vec3(0.2126, 0.7152, 0.0722), rgb); }\n"
glsl += body
glsl = glsl.replace("const in ", "in ")   # (naga's GLSL reader takes no const on an argument)

with tempfile.TemporaryDirectory() as d:
    f = os.path.join(d, "shader.frag")
    open(f, "w").write(glsl)
    w = os.path.join(d, "shader.wgsl")
    run = subprocess.run(["naga", "--input-kind", "glsl", "--shader-stage", "frag", f, w], capture_output=True, text=True)
    if run.returncode:
        sys.exit(f"{src}: naga: {run.stderr.strip()}\n--- what it was given:\n{glsl}")
    wgsl = open(w).read()

name = os.path.basename(src).replace("Shader.js", "").replace(".js", "")
head = f"// {name}: three.js's {os.path.basename(src)} (MIT, Copyright (c) 2010-2026 three.js authors, github.com/mrdoob/three.js), its GLSL made WGSL by naga (filament/effects/from_three.py)\n"
if len(sys.argv) > 3:
    head += f"// {sys.argv[3]}\n"
for t, n in loose:
    if n in ("time",):
        continue
    vs = usual.get(n, [0.0])
    for i, v in enumerate(vs):
        k = n if len(vs) == 1 else f"{n}_{'xyzw'[i]}"   # (a vector's numbers each a knob: screen.h gathers them back)
        if t == "bool":
            head += f"// --check@{n}_on:{n},{int(v)}\n"
        elif t == "int":
            head += f"// --track@{k}:{k},0,{max(1, int(v) * 4)},{int(v)},1\n"
        else:
            head += f"// --track@{k}:{k},{min(0.0, v * 4)},{max(1.0, v * 4)},{v},0.001\n"
open(out, "w").write(head + "\n" + wgsl)
print(f"wrote {out}: knobs {[n for _, n in loose]}")
