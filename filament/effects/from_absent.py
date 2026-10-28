#!/usr/bin/env python3
# An AbsentMotion effect (app/src/main/assets/effects/<id>.json: its params, and an SkSL runtime shader reading the layer
# as `image`) made an effect file in vgpu's manner (screen.h), by Skia's own skslc: the shader's own lines are kept;
# around them the picture is a texture and a sampler (Skia's WGSL tests' way of writing one: resources/sksl/wgsl/
# Sample.sksl), the loose uniforms one struct, `size` read as the stage's `resolution`, its main called at each pixel;
# the knobs written in the head (a point two, a colour four, as screen.h reads a vector's numbers).
# One reading the picture blurred (`blurred`), or its bright part blurred (`bright`), is given it as AbsentMotion's
# effects.cpp gives it: a Gaussian blur of sigma blur/3 (bright: luma less its threshold first), here in two passes
# (screen.h's ISF passes: across into `h`, then down into the blurred picture), as Skia's own blur goes, then its own.
# One AbsentMotion moves by its own C++ (its motion.cpp ApplyMotion: a native effect it names below) made a file of its
# knobs alone, out with .am for .wgsl: run as AbsentMotion runs it (absent_motion.cpp), its knobs written as below.
# usage: from_absent.py <id>.json out.wgsl skslc   (an effect reading another picture, or another native, is refused)
import json, os, re, subprocess, sys

src, out, skslc = sys.argv[1], sys.argv[2], sys.argv[3]
d = json.load(open(src))
moves = ('autoshake', 'randomdisplace', 'randomjitter', 'oscillate', 'swing', 'spin', 'pulsesize', 'pulseopacity')   # (AbsentMotion's motion.cpp ApplyMotion: what it moves the layer by; MotionOpacity: its opacity)
moves += ('repeat', 'linearrepeat', 'gridrepeat', 'radialrepeat')   # (its RepeatCopies: copies of the layer; along a path needs the layer beneath, a scatter field its size and camera: not yet)


def knob_lines(named, numbers):   # (each param a knob or more, by the name the effect reads it by; numbers: the kinds one number each)
    lines = []
    for p in d.get('params', []):
        n = named.get(p['id'])
        if not n:
            continue
        lo, hi, step = p.get('min', 0), p.get('max', 1), p.get('step', 0.01)
        if p['type'] == 'choice' and 'switch' in numbers:   # (a native one's choice is the number of its option, as its C++ reads it)
            lo, hi, step = 0, len(p.get('options', [])) - 1, 1
        if p['type'] in numbers:   # (an angle is degrees, as the shader's radians() reads it; a switch 0 or 1)
            lines.append(f"// --track@{n}:{p['label']},{lo},{hi},{float(p['default'])},{step}")
        elif p['type'] == 'point':
            lines += [f"// --track@{n}_{a}:{p['label']} {a},{lo},{hi},{p['default'][i]},{step}" for i, a in enumerate('xy')]
        elif p['type'] == 'color':
            c = p['default']
            rgba = [int(c[i:i + 2], 16) / 255 for i in (3, 5, 7, 1)] if len(c) == 9 else [int(c[i:i + 2], 16) / 255 for i in (1, 3, 5)] + [1]   # (#AARRGGBB or #RRGGBB)
            lines += [f"// --track@{n}_{a}:{p['label']} {'RGBA'[i]},0,1,{rgba[i]:.4g},0.01" for i, a in enumerate('xyzw')]
    return lines


if 'sksl' not in d:
    if not d.get('native') or d['id'] not in moves:
        sys.exit(f"{src}: drawn by AbsentMotion's own C++, no shader")
    head = [f"// {d['name']}: AbsentMotion's {d['id']} (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), moved by its own motion.cpp (filament/absent_motion.cpp)",
            f"// --label:{d.get('category', '').capitalize()}", f"// --absent:{d['id']}"]
    if d.get('description'):
        head.append(f"// --about:{' '.join(d['description'].split())}")
    open(os.path.splitext(out)[0] + '.am', 'w').write('\n'.join(head + knob_lines({p['id']: p['id'] for p in d.get('params', [])}, ('float', 'int', 'bool', 'switch', 'choice', 'angle'))) + '\n')
    sys.exit(0)
s = d['sksl']
pictures = re.findall(r'^\s*uniform\s+shader\s+(\w+)\s*;', s, re.M)
blur = next((p for p in pictures if p in ('blurred', 'bright')), None)
across = 'h_' if re.search(r'\bh\b', s) else 'h'   # (the blur's pass across: a name the effect's own lines do not use)
back = 'backdrop' in pictures   # (what lies behind the layer: the stage's picture without it, screen.h's behind, read where the layer's box lies on it)
if sorted(pictures) != sorted(['image'] + ([blur] if blur else []) + (['backdrop'] if back else [])):
    sys.exit(f"{src}: reads {pictures}: an effect reads the picture, it blurred, and what is behind it at most")
s = re.sub(r'(uniform\s+float2\s+)size\b', r'\1resolution', s)   # (the picture's size, as screen.h tells it)
s = re.sub(r'\bsize\b', 'layerSize_()', s)   # (AbsentMotion's size where it is read: the layer's own, not the picture's with room round it)
typed = r'^\s*(?:layout\(color\)\s+)?uniform\s+(float[234]?|half[234]?|int)\s+(\w+)\s*;'
loose = re.findall(typed, s, re.M)
s = re.sub(typed + r'\s*$', '', s, flags=re.M)
s = re.sub(r'^\s*uniform\s+shader\s+(image|blurred|bright|backdrop)\s*;', '', s, flags=re.M)
loose.append(('float4', 'layerRect'))   # (the layer's own rectangle on the picture, in its pixels: corner and size, doc_moment_fx.h; none said, the whole picture)
if back:
    loose.append(('float4', 'backdropBox'))   # (the layer's box on the stage: left, top, width, height, each a share of it: doc_moment_fx.h)
if blur:
    for n in ('blur', 'threshold') if blur == 'bright' else ('blur',):
        if not any(m == n for _, m in loose): loose.append(('float', n))   # (what the blur is made by, a knob of the effect's own in AbsentMotion)
    loose.append(('float', 'passindex'))   # (which pass: screen.h's ISF PASSINDEX)
head = '/*#pragma settings ForceHighPrecision AllowNarrowingConversions*/\n'   # (as a runtime effect: floats throughout, a half taken from a float)
head += 'layout(webgpu, set=0, texture=0, sampler=1) sampler2D image;\n'
if blur:
    head += f'layout(webgpu, set=0, texture=2, sampler=3) sampler2D {across};\nlayout(webgpu, set=0, texture=4, sampler=5) sampler2D blurPicture;\n'
if back:
    head += f"layout(webgpu, set=0, texture={6 if blur else 2}, sampler={7 if blur else 3}) sampler2D backdropPic;\n"   # (read after its own and its passes' pictures: screen.h's others)
head += f"layout(set=0, binding={(6 if blur else 2) + (2 if back else 0)}) uniform Knobs {{\n" + ''.join(f'  {t} {n};\n' for t, n in loose) + '};\n'
head += 'float2 layerAt_() { return layerRect.z > 0 ? layerRect.xy : float2(0); }\nfloat2 layerSize_() { return layerRect.z > 0 ? layerRect.zw : resolution; }\n'   # (where AbsentMotion's layer space begins on the picture, and how large it is)
head += 'half4 imageAt(float2 p) { return sample(image, (p + layerAt_()) / resolution); }\n'   # (a child shader read at a pixel: the picture at that place)
if back:
    head += 'half4 backdropAt(float2 p) { return sample(backdropPic, backdropBox.xy + (p + layerAt_()) / resolution * backdropBox.zw); }\n'   # (what is behind that place)
if blur:   # (a Gaussian across then down, sigma blur/3 in pixels, three sigmas each side, as Skia's blur reaches)
    head += 'half4 ' + blur + 'At(float2 p) { return sample(blurPicture, (p + layerAt_()) / resolution); }\n'
    head += ('half4 gauss_(float2 p, float2 by, int from) {\n  float sigma = max(blur, 0) / 3; if (sigma < 0.5) return from == 0 ? sample(image, p / resolution) : sample(' + across + ', p / resolution);\n'
             '  half4 all = half4(0); float weight = 0;\n  for (int i = -96; i <= 96; i++) { float x = float(i); if (abs(x) > 3 * sigma) continue; float w = exp(-x * x / (2 * sigma * sigma));\n'
             '    half4 c = from == 0 ? sample(image, (p + by * x) / resolution) : sample(' + across + ', (p + by * x) / resolution);\n'
             + ('    if (from == 0) c = half4(clamp(dot(c.rgb, half3(0.2126, 0.7152, 0.0722)) - threshold, 0, 1).xxx, c.a);\n' if blur == 'bright' else '') +
             '    all += c * w; weight += w; }\n  return all / weight;\n}\n')
s = re.sub(r'\bbackdrop\.eval\(', 'backdropAt(', s)
s = head + re.sub(r'\b(blurred|bright)\.eval\(', blur + 'At(' if blur else r'\1.eval(', re.sub(r'\bimage\.eval\(', 'imageAt(', s))
s = re.sub(r'\bhalf4 main\(float2 (\w+)\)', r'half4 effect_(float2 \1)', s)   # (the file's own pixel function, whatever it names the pixel: the wrapper's main calls it)
s += ('\nvoid main() { sk_FragColor = passindex < 0.5 ? gauss_(sk_FragCoord.xy, float2(1, 0), 0) : passindex < 1.5 ? gauss_(sk_FragCoord.xy, float2(0, 1), 1) : effect_(sk_FragCoord.xy - layerAt_()); }\n' if blur
      else '\nvoid main() { sk_FragColor = effect_(sk_FragCoord.xy - layerAt_()); }\n')
open(out + '.frag', 'w').write(s)
made = subprocess.run([skslc, out + '.frag', out + '.raw.wgsl'], capture_output=True, text=True)
if made.returncode:
    sys.exit(f"{src}: {made.stdout.strip().splitlines()[0] if made.stdout.strip() else made.stderr}")
wgsl = open(out + '.raw.wgsl').read()
fields = re.findall(r'^\s*(\w+)\s*:', re.search(r'struct Knobs\s*\{([^}]*)\}', wgsl).group(1), re.M)
named = {re.sub(r'^R_', '', n): n for n in fields}   # (skslc's R_ before a name WGSL keeps for itself: center)
lines = [f"// {d['name']}: AbsentMotion's {d['id']} (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)"]
lines.append(f"// --label:{d.get('category', '').capitalize()}")
if blur:
    lines += [f'// --pass@{across}:0,1,s1,s1', '// --pass@blurPicture:0,1,s1,s1', '// --pass@:0,0,s1,s1']   # (across, down, then the effect itself)
if back:
    lines.append('// --picture@backdrop:Behind')   # (screen.h gives it the stage's picture of what is behind, not a thing chosen)
if d.get('description'):
    lines.append(f"// --about:{' '.join(d['description'].split())}")   # (what it does, in AbsentMotion's words: the shelf finds it by them)
lines += knob_lines(named, ('float', 'int', 'bool', 'choice', 'angle'))
open(out, 'w').write('\n'.join(lines) + '\n' + wgsl)
