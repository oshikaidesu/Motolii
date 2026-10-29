#!/usr/bin/env python3
# A Sashimono Edit effect (MIT, Copyright (c) 2026 Sashimono Edit contributors, github.com/kagemorikosame/sashimono-edit at 761b6e4ba247:
# src/sashimono/effects/builtin.py, an EffectDefinition: its params and a GLSL fragment shader after its PRELUDE) made
# an ISF effect file (.fs) for Motolii's ISF door (isf.cpp): the shader's own lines and the PRELUDE's own functions kept
# as they are; PRELUDE's declarations said again in ISF's words (its picture `u_texture` and `u_source` the picture in,
# `v_uv` ISF's place, `u_size` the picture's size, `u_pass` the pass), the clip's own (`u_object`, `u_origin`,
# `u_content`) the whole picture, a pixel read by texelFetch turned over as ISF's places are (isf.cpp: up from the
# bottom left). Its params ISF's inputs by their own names: a track a float, a check a bool (a check read as a number a
# float of 0 or 1), a colour a colour, a choice a long of its choices' numbers. Sashimono draws in linear light; Motolii
# gives an effect sRGB (screen.h screen_tone): its colours are read as given.
# usage (Python 3.12 or later, numpy and packaging: what Sashimono's effects import):
#   uv run --no-project --python 3.13 --with numpy --with packaging python -I from_sashimono.py <Sashimono's src> <out dir> [kind ...]
# (an effect of more than one pass, of pieces, of a picture file, a gradient, a value of its own kind, or a sound: refused)
import json, os, re, sys

src, out, wanted = sys.argv[1], sys.argv[2], sys.argv[3:]
sys.path.insert(0, os.path.join(src, 'src'))
from sashimono.effects.builtin import PRELUDE, register_builtin_effects   # noqa: E402
from sashimono.effects import definition, spec   # noqa: E402

register_builtin_effects()
declared = PRELUDE[:PRELUDE.index('const vec3 LUMA')]   # (its declarations: said again below)
kept = PRELUDE[len(declared):]
words = '''#define u_texture inputImage
#define u_source inputImage
#define v_uv isf_FragNormCoord
#define frag_color gl_FragColor
#define u_size RENDERSIZE
#define u_pass PASSINDEX
#define u_time TIME
#define u_frame (TIME * 60.0)
#define u_fps 60.0
#define u_duration 1.0e9
#define u_object vec4(0.0, 0.0, RENDERSIZE)
#define u_origin (RENDERSIZE * 0.5)
#define u_content vec4(0.0, 0.0, RENDERSIZE)
#define u_pixel_scale 1.0
#define texelFetch(t, p, l) texelFetch(t, ivec2((p).x, textureSize(t, 0).y - 1 - (p).y), l)
'''


def args_at(code, open_at):   # (a call's or a header's arguments from its '(' on: each one's start and end, and where it closes)
    depth, start, spans = 0, open_at + 1, []
    for i in range(open_at, len(code)):
        c = code[i]
        if c in '([':
            depth += 1
        elif c in ')]':
            depth -= 1
            if depth == 0:
                spans.append((start, i))
                return spans, i
        elif c == ',' and depth == 1:
            spans.append((start, i))
            start = i + 1
    raise ValueError('unclosed')


def one_picture(code):
    # (Vulkan's GLSL makes a picture's sampler where it is read, not handed to a function, so a function taking a
    #  sampler2D takes none: it reads the picture in by its name (inputImage), and each call to it drops that argument;
    #  every picture Sashimono hands one, for an effect of one pass, is the picture in)
    header = re.compile(r'\b(?:void|float|int|bool|[iu]?vec[234]|mat[234])\s+(\w+)\s*\(')
    takes = {}
    for m in header.finditer(code):
        spans, _ = args_at(code, m.end() - 1)
        for k, (a, b) in enumerate(spans):
            p = code[a:b].split()
            if len(p) == 2 and p[0] == 'sampler2D':
                takes[m.group(1)] = (k, p[1])
    while True:   # (each definition: its sampler said no more, its name the picture in within it)
        for m in header.finditer(code):
            if m.group(1) not in takes:
                continue
            spans, close = args_at(code, m.end() - 1)
            k, name = takes[m.group(1)]
            if len(spans) <= k or code[spans[k][0]:spans[k][1]].split() != ['sampler2D', name]:
                continue
            body_open = code.index('{', close)
            depth, end = 0, body_open
            for end in range(body_open, len(code)):
                depth += {'{': 1, '}': -1}.get(code[end], 0)
                if depth == 0:
                    break
            params = [code[a:b] for j, (a, b) in enumerate(spans) if j != k]
            body = re.sub(rf'\b{name}\b', 'inputImage', code[body_open:end + 1])
            code = code[:m.start()] + code[m.start():m.end() - 1] + '(' + ','.join(params).strip() + ')' + code[close + 1:body_open] + body + code[end + 1:]
            break
        else:
            break
    for fn, (k, _) in takes.items():   # (each call: that argument dropped)
        at = 0
        while (m := re.compile(rf'\b{fn}\s*\(').search(code, at)):
            before = code[max(0, m.start() - 12):m.start()]
            spans, close = args_at(code, m.end() - 1)
            if re.search(r'\b(?:void|float|int|bool|[iu]?vec[234]|mat[234])\s+$', before) or len(spans) <= k:
                at = m.end()
                continue
            kept_args = [code[a:b] for j, (a, b) in enumerate(spans) if j != k]
            code = code[:m.end()] + ','.join(kept_args).strip() + code[close:]
            at = m.end()
    return code


def inputs(d):
    said = [{'NAME': 'inputImage', 'TYPE': 'image'}]
    for p in d.parameters:
        if isinstance(p, spec.TrackSpec):
            said.append({'NAME': p.name, 'LABEL': p.label, 'TYPE': 'float', 'DEFAULT': p.default, 'MIN': p.minimum, 'MAX': p.maximum})
        elif isinstance(p, spec.CheckSpec):
            said.append({'NAME': p.name, 'LABEL': p.label, 'TYPE': 'float', 'DEFAULT': float(p.default), 'MIN': 0, 'MAX': 1} if p.as_number else {'NAME': p.name, 'LABEL': p.label, 'TYPE': 'bool', 'DEFAULT': bool(p.default)})
        elif isinstance(p, spec.ColorSpec):
            said.append({'NAME': p.name, 'LABEL': p.label, 'TYPE': 'color', 'DEFAULT': list(p.default)})
        elif isinstance(p, spec.SelectSpec):
            values = [v for v, _ in p.choices]
            said.append({'NAME': p.name, 'LABEL': p.label, 'TYPE': 'long', 'VALUES': list(range(len(values))), 'LABELS': [l for _, l in p.choices], 'DEFAULT': values.index(p.default) if p.default in values else 0})
        else:
            return None
    return said


shelf = {'変形': 'Distort', '装飾': 'Stylize', '登場・退場': 'In & Out', '色': 'Color', '動き': 'Move', 'ぼかし': 'Blur', '形': 'Shape', '合成': 'Key'}   # (its categories as the shelf's own labels say them)


def title(kind):   # (its name: its own word for it, in words; in and out, its In-Out ones)
    words = kind.split('_')
    return ' '.join(['In-Out'] + [w.capitalize() for w in words[1:]]) if words[0] == 'inout' else ' '.join(w.capitalize() for w in words)


made, refused = [], []
for d in definition.registry.all():
    if wanted and d.kind not in wanted:
        continue
    said = inputs(d)
    if said is None or d.audio_process or d.pieces or d.scopes_following or not d.fragment_shader or d.passes != 1:
        refused.append(d.kind)
        continue
    head = {'DESCRIPTION': f"Sashimono Edit's {d.label} ({d.kind}, {d.category}; MIT, src/sashimono/effects/builtin.py), made an ISF effect by filament/effects/from_sashimono.py", 'CREDIT': 'Sashimono Edit (MIT)', 'CATEGORIES': [shelf.get(d.category, d.category)], 'ISFVSN': '2', 'INPUTS': said}
    body = d.fragment_shader[len(PRELUDE):] if d.fragment_shader.startswith(PRELUDE) else d.fragment_shader
    body = re.sub(r'^[ \t]*uniform[ \t]+\w+[ \t]+\w+[ \t]*;.*$', '', body, flags=re.M)   # (its params: ISF's inputs, declared by the door)
    with open(os.path.join(out, f'{title(d.kind)} (Sashimono).fs'), 'w') as f:   # (its name on the shelf the file's: Sashimono's beside ISF's own of a name, Emboss, Mosaic)
        f.write('/*' + json.dumps(head, ensure_ascii=False, indent=1) + '*/\n' + words + one_picture(kept + body))
    made.append(d.kind)
print(len(made), 'made:', ' '.join(made))
print(len(refused), 'refused:', ' '.join(refused))
