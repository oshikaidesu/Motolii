#!/usr/bin/env python3
# A load of shapes whose paths are made again every frame by Skia (skpath.cpp), all moving at once: a set is a
# boolean with a moving cutter, a morph, a rough edge, a resampled zig zag, squares, a trail with a head, a line
# drawn on with heads, and copies sliding round a circle; `sets` of them in a grid. plain: the same things moving
# the same way with none of Skia's makings (what Skia costs is the difference).
#   check/skia_load.py <sets> [plain] | ./edit.sh out/load.motolii
import sys

sets = int(sys.argv[1])
plain = len(sys.argv) > 2 and sys.argv[2] == 'plain'
only = set(sys.argv[2].split(',')) if len(sys.argv) > 2 and sys.argv[2] != 'plain' else None   # (only these of Skia's makings: boolean morph rough resample voxel trail trim copy)
use = lambda what: not plain and (only is None or what in only)
calls = []
n = 0


def add(form):
    global n
    n += 1
    calls.append(f'add shape {form}')
    return n


def say(t, name, word):
    calls.append(f'say {t} {name} {word}')


def set_(t, name, v):
    calls.append(f'set {t} {name} {v}')


def key(t, name, a, b):
    calls.append(f'key {t} {name} 0 {a}')
    calls.append(f'key {t} {name} 2 {b}')


def skia(t, name, v):
    if not plain:
        set_(t, name, v)


cols = max(1, round((sets * 9 / 16) ** 0.5))   # (the stage stands 1080 wide, 1920 high)
for i in range(sets):
    rows = max(1, -(-sets // cols))
    cx, cy = -0.8 + 1.6 * ((i % cols) + 0.5) / cols, 1.4 - 2.8 * ((i // cols) + 0.5) / rows
    s = min(1.6 / cols, 2.8 / rows)   # (a set's cell)
    at = lambda t, dx, dy: (set_(t, 'x', round(cx + dx * s, 4)), set_(t, 'y', round(cy + dy * s, 4)))
    # a plate cut by a moving circle
    a = add('rect'); at(a, -0.35, 0.3); set_(a, 'size', 0.2 * s); say(a, 'fill', 'ff3366')
    b = add('circle'); set_(b, 'size', 0.14 * s); set_(b, 'visible', 0); set_(b, 'y', round(cy + 0.3 * s, 4))
    key(b, 'x', round(cx - 0.45 * s, 4), round(cx - 0.25 * s, 4))
    if use('boolean'): say(a, 'boolean', 'subtract'); set_(a, 'boolean_with', b)
    # a star turning into a squircle
    m = add('star'); at(m, -0.1, 0.3); set_(m, 'size', 0.18 * s); say(m, 'fill', 'ffcc00')
    q = add('squircle'); at(q, -0.1, 0.3); set_(q, 'size', 0.18 * s); set_(q, 'visible', 0)
    if use('morph'): set_(m, 'morph_to', q)
    key(m, 'morph' if use('morph') else 'rotate', 0, 1 if use('morph') else 90)
    # a rough edge
    r = add('circle'); at(r, 0.15, 0.3); set_(r, 'size', 0.18 * s); say(r, 'fill', '22aaff')
    key(r, 'rough' if use('rough') else 'scale_x', 0 if use('rough') else 1, 0.05 if use('rough') else 1.2)
    # a resampled zig zag
    z = add('star'); at(z, 0.4, 0.3); set_(z, 'size', 0.18 * s); say(z, 'fill', '66dd88'); (set_(z, 'zigzag', 0.03), set_(z, 'zigzag_ridges', 1)) if use('resample') else None
    key(z, 'resample' if use('resample') else 'rotate', 3 if use('resample') else 0, 40 if use('resample') else 90)
    # squares
    v = add('circle'); at(v, -0.35, -0.1); set_(v, 'size', 0.2 * s); say(v, 'fill', 'aa66ff')
    key(v, 'voxel' if use('voxel') else 'scale_y', 0.02 if use('voxel') else 1, 0.1 if use('voxel') else 1.2)
    # a waving dot and its trail, with a head
    dot = add('circle'); at(dot, -0.1, -0.1); set_(dot, 'size', 0.04 * s); say(dot, 'fill', 'ffffff'); set_(dot, 'wave_x', round(0.12 * s, 4)); set_(dot, 'wave_speed', 1)
    key(dot, 'y', round(cy - 0.25 * s, 4), round(cy + 0.05 * s, 4))
    tr = add('path'); say(tr, 'fill', 'none'); say(tr, 'stroke', 'ffffff'); set_(tr, 'stroke_width', 4)
    if use('trail'): set_(tr, 'trail', dot); set_(tr, 'trail_length', 1); say(tr, 'heads', 'end')
    else: say(tr, 'path', 'M-0.2 0 L0.2 0'); at(tr, -0.1, -0.1); key(tr, 'rotate', 0, 90)
    # a line drawn on, with heads
    ln = add('path'); at(ln, 0.25, -0.1); say(ln, 'path', 'M-0.5 0.2 C-0.2 -0.4 0.2 0.4 0.5 -0.2'); set_(ln, 'size', 0.3 * s)
    say(ln, 'fill', 'none'); say(ln, 'stroke', 'ffffff'); set_(ln, 'stroke_width', 10)
    if use('trim'): say(ln, 'heads', 'both')
    key(ln, 'trim' if use('trim') else 'rotate', 1 if use('trim') else 0, 0 if use('trim') else 90)
    # copies sliding round the rough circle's place
    ring = add('circle'); at(ring, 0.4, -0.1); set_(ring, 'size', 0.2 * s); set_(ring, 'visible', 0)
    c = add('rect'); at(c, 0.4, -0.1); set_(c, 'size', 0.025 * s); say(c, 'fill', 'ff8800'); set_(c, 'copies', 12)
    if use('copy'): set_(c, 'copy_on', ring)
    else: set_(c, 'copy_x', round(0.03 * s, 4))
    key(c, 'copy_on_offset' if use('copy') else 'rotate', 0, 1 if use('copy') else 90)
print('\n'.join(calls))
print('time')
