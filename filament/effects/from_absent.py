#!/usr/bin/env python3
# An AbsentMotion effect (app/src/main/assets/effects/<id>.json: its params, and an SkSL runtime shader reading the layer
# as `image`) made an effect file in vgpu's manner (screen.h), by Skia's own skslc: the shader's own lines are kept;
# around them the picture is a texture and a sampler (Skia's WGSL tests' way of writing one: resources/sksl/wgsl/
# Sample.sksl), the loose uniforms one struct, `size` read as the stage's `resolution`, its main called at each pixel;
# the knobs written in the head (a point two, a colour four, as screen.h reads a vector's numbers).
# usage: from_absent.py <id>.json out.wgsl skslc   (an effect reading a second picture, or native, is refused)
import json, re, subprocess, sys

src, out, skslc = sys.argv[1], sys.argv[2], sys.argv[3]
d = json.load(open(src))
if 'sksl' not in d:
    sys.exit(f"{src}: drawn by AbsentMotion's own C++, no shader")
s = d['sksl']
pictures = re.findall(r'^\s*uniform\s+shader\s+(\w+)\s*;', s, re.M)
if pictures != ['image']:
    sys.exit(f"{src}: reads {pictures}: an effect reads one picture")
s = re.sub(r'\bsize\b', 'resolution', s)
typed = r'^\s*(?:layout\(color\)\s+)?uniform\s+(float[234]?|half[234]?|int)\s+(\w+)\s*;'
loose = re.findall(typed, s, re.M)
s = re.sub(typed + r'\s*$', '', s, flags=re.M)
s = re.sub(r'^\s*uniform\s+shader\s+image\s*;', '', s, flags=re.M)
head = '/*#pragma settings ForceHighPrecision AllowNarrowingConversions*/\n'   # (as a runtime effect: floats throughout, a half taken from a float)
head += 'layout(webgpu, set=0, texture=0, sampler=1) sampler2D image;\n'
head += 'layout(set=0, binding=2) uniform Knobs {\n' + ''.join(f'  {t} {n};\n' for t, n in loose) + '};\n'
head += 'half4 imageAt(float2 p) { return sample(image, p / resolution); }\n'   # (a child shader read at a pixel: the picture at that place)
s = head + re.sub(r'\bimage\.eval\(', 'imageAt(', s).replace('half4 main(float2 coord)', 'half4 effect_(float2 coord)')
s += '\nvoid main() { sk_FragColor = effect_(sk_FragCoord.xy); }\n'
open(out + '.frag', 'w').write(s)
made = subprocess.run([skslc, out + '.frag', out + '.raw.wgsl'], capture_output=True, text=True)
if made.returncode:
    sys.exit(f"{src}: {made.stdout.strip().splitlines()[0] if made.stdout.strip() else made.stderr}")
wgsl = open(out + '.raw.wgsl').read()
fields = re.findall(r'^\s*(\w+)\s*:', re.search(r'struct Knobs\s*\{([^}]*)\}', wgsl).group(1), re.M)
named = {re.sub(r'^R_', '', n): n for n in fields}   # (skslc's R_ before a name WGSL keeps for itself: center)
lines = [f"// {d['name']}: AbsentMotion's {d['id']} (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)"]
lines.append(f"// --label:{d.get('category', '').capitalize()}")
if d.get('description'):
    lines.append(f"// --about:{' '.join(d['description'].split())}")   # (what it does, in AbsentMotion's words: the shelf finds it by them)
for p in d.get('params', []):
    n = named.get(p['id'])
    if not n:
        continue
    lo, hi, step = p.get('min', 0), p.get('max', 1), p.get('step', 0.01)
    if p['type'] in ('float', 'int', 'bool', 'choice'):
        lines.append(f"// --track@{n}:{p['label']},{lo},{hi},{float(p['default'])},{step}")
    elif p['type'] == 'point':
        lines += [f"// --track@{n}_{a}:{p['label']} {a},{lo},{hi},{p['default'][i]},{step}" for i, a in enumerate('xy')]
    elif p['type'] == 'color':
        c = p['default']
        rgba = [int(c[i:i + 2], 16) / 255 for i in (3, 5, 7, 1)] if len(c) == 9 else [int(c[i:i + 2], 16) / 255 for i in (1, 3, 5)] + [1]   # (#AARRGGBB or #RRGGBB)
        lines += [f"// --track@{n}_{a}:{p['label']} {'RGBA'[i]},0,1,{rgba[i]:.4g},0.01" for i, a in enumerate('xyzw')]
open(out, 'w').write('\n'.join(lines) + '\n' + wgsl)
