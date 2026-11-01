#!/bin/zsh
# A flat thing is a thing at Z 0 (the user, 11/1): its Scale Z opens its sheet into a box its picture is pushed through
# (deep.glsl), nothing filmed, read back or cut anew. At 0 not a pixel changes; opened its sides are seen, its front is
# its picture's own colour, and two boxes cross where they really meet (After Effects' 3D layers intersect).
# With a lamp in the document it takes its light, flat or thick, as an After Effects 3D layer; what it throws on a wall
# is the shape drawn, not its box (a depth-only pass keeps its fragment stage: dawn_probe/filament_webgpu_depth_fragment.patch).
# A bevel cuts its rim by how far each point is from its edge (drawn_edge.h); no bevel, or none deep, is the box as it was.
# Metal is a solid's own three numbers, and needs no lamp: it mirrors the room. With an effect of its own a flat thing is
# filmed apart, and its sheet, standing where it lies, throws the shadow. Its bevel, side and back each take a colour of
# their own (After Effects' Bevel, Side and Back Color), and its front stays as it is. A word's further strokes, drawn
# with it, stand in its depth, and a Step bevel makes each a stair.
#   ./check/deep.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
shot() { printf "$2\nstill $W/$1.png\n" | ./edit.sh $W/$1.motolii > /dev/null 2>&1 }
star="add shape star\nsay 1 fill e5432d\nsay 1 space 3d\nset 1 rotate_y 40\nset 1 rotate_x -20"
one="add shape rect\nsay 1 fill e5432d\nset 1 scale_z 1\nset 1 rotate_y 50"
two="add shape rect\nsay 2 fill 2a62c9\nset 2 scale_z 1\nset 2 rotate_y -50"
shot flat "$star"; shot zero "$star\nset 1 scale_z 0"; shot deep "$star\nset 1 scale_z 1"
shot one "$one"; shot crossed "$one\n$two"
wall="add solid cube\nsay 2 fill bfbfbf\nset 2 scale_x 14\nset 2 scale_y 8\nset 2 scale_z 0.1\nset 2 z -0.8"
sun="add light sun\nset 3 shadows 1\nset 3 intensity 0.3\nset 3 rotate_x -30\nset 3 rotate_y 25"
shot bare "$star\nset 1 scale_z 1\n$wall"; shot sunlit "$star\nset 1 scale_z 1\n$wall\n$sun"; shot flatlit "$star\n$wall\n$sun"; shot unshaded "$star\nset 1 scale_z 1\n$wall\n$sun\nset 3 shadows 0"
bevel="$star\nset 1 scale_z 1\n$wall\n$sun"
shot plain "$bevel"; shot bevel0 "$bevel\nsay 1 bevel none"; shot bevelled "$bevel\nsay 1 bevel convex\nset 1 bevel_depth 30"; shot bevelno "$bevel\nsay 1 bevel convex\nset 1 bevel_depth 0"
shot metal "$star\nset 1 scale_z 1\nset 1 metallic 1\nset 1 roughness 0.2"
shot partside "$star\nset 1 scale_z 1\nsay 1 side_color 2fd04a"; shot partback "$star\nset 1 scale_z 1\nset 1 rotate_y 160\nsay 1 back_color 2fd04a"; shot partbevel "$star\nset 1 scale_z 1\nsay 1 bevel convex\nset 1 bevel_depth 20\nsay 1 bevel_color 2fd04a"
word="add text $PWD/blobtrack/kit/JetBrainsMono-Regular.ttf\nsay 1 text Hi\nset 1 size 2\nset 1 scale_z 1\nset 1 rotate_y 40\nsay 1 fill e5432d\nsay 1 stroke ffffff\nset 1 stroke_width 4"
shot rim0 "$word"; shot rim2 "$word\nsay 1 stroke_2 2fd04a\nset 1 stroke_2_width 8\nsay 1 stroke_3 2a62c9\nset 1 stroke_3_width 8"
shot stair "$word\nsay 1 stroke_2 2fd04a\nset 1 stroke_2_width 8\nsay 1 stroke_3 2a62c9\nset 1 stroke_3_width 8\nsay 1 bevel step\nset 1 bevel_depth 30"
fxd="say 1 fx $PWD/effects/am_huesaturation.wgsl"
shot fxlit "$star\n$fxd\n$wall\n$sun"; shot fxshaded "$star\n$fxd\n$wall\n${sun/shadows 1/shadows 0}"
python3 -c "
from PIL import Image, ImageChops
import numpy as np
L = lambda n: np.asarray(Image.open('$W/' + n + '.png').convert('RGB'), dtype=int)
flat, zero, deep = L('flat'), L('zero'), L('deep')
n = (flat != zero).any(axis=2).sum(); print('at Z 0 it is the flat thing: ' + ('yes' if n == 0 else 'NO') + ' (%d pixels changed)' % n)
seen = lambda a: a.max(axis=2) > 8
print('opened, it takes more room: ' + ('yes' if seen(deep).sum() > 1.1 * seen(flat).sum() else 'NO') + ' (%d and %d pixels)' % (seen(flat).sum(), seen(deep).sum()))
front = (abs(deep - [0xe5, 0x43, 0x2d]).max(axis=2) < 6).sum(); print('its front is its own colour: ' + ('yes' if front > .5 * seen(flat).sum() else 'NO') + ' (%d pixels)' % front)
side = (seen(deep) & (abs(deep - [0xe5, 0x43, 0x2d]).max(axis=2) > 24)).sum(); print('its sides are shaded: ' + ('yes' if side > .05 * seen(flat).sum() else 'NO') + ' (%d pixels)' % side)
c, o = L('crossed'), L('one')
red = abs(c - [0xe5, 0x43, 0x2d]).max(axis=2) < 40; blue = abs(c - [0x2a, 0x62, 0xc9]).max(axis=2) < 40
x = np.nonzero(seen(c).any(axis=0))[0]; mid = (x[0] + x[-1]) // 2
print('two boxes cross: ' + ('yes' if red[:, :mid].sum() > 4 * red[:, mid:].sum() and blue[:, mid:].sum() > 4 * blue[:, :mid].sum() else 'NO') + ' (red %d left, %d right; blue %d left, %d right)' % (red[:, :mid].sum(), red[:, mid:].sum(), blue[:, :mid].sum(), blue[:, mid:].sum()))
b, l, f = L('bare'), L('sunlit'), L('flatlit')
front = abs(b - [0xe5, 0x43, 0x2d]).max(axis=2) < 6; same = (abs(b - l)[front].max(axis=1) > 8).mean(); print('a lamp lights it: ' + ('yes' if same > .9 else 'NO') + ' (%.0f%% of its front changed)' % (100 * same))
print('flat, it takes the lamp too: ' + ('yes' if (abs(L('flat') - f).max(axis=2) > 8)[abs(L('flat') - [0xe5, 0x43, 0x2d]).max(axis=2) < 6].mean() > .9 else 'NO'))
dark = (L('unshaded') - l).sum(axis=2) > 8
y, x = np.nonzero(dark); box = (y.max() - y.min()) * (x.max() - x.min()); print('its shadow is its shape, not its box: ' + ('yes' if 2000 < dark.sum() < .75 * box else 'NO') + ' (%d of %d pixels)' % (dark.sum(), box))
p, e = L('plain'), L('bevelled')
n = (p != L('bevel0')).any(axis=2).sum() + (p != L('bevelno')).any(axis=2).sum(); print('no bevel, or none deep, is the box as it was: ' + ('yes' if n == 0 else 'NO') + ' (%d pixels changed)' % n)
ring = abs(p - e).max(axis=2) > 8; star = abs(p - p[0, 0]).max(axis=2) > 8; print('a bevel shades its rim, not its middle: ' + ('yes' if 2000 < ring.sum() < .75 * star.sum() else 'NO') + ' (%d of %d pixels)' % (ring.sum(), star.sum()))
front = abs(deep - [0xe5, 0x43, 0x2d]).max(axis=2) < 6; m = abs(deep - L('metal')).max(axis=2) > 8; print('said to be metal, it mirrors the room with no lamp: ' + ('yes' if m[front].mean() > .9 else 'NO') + ' (%.0f%% of its front changed)' % (100 * m[front].mean()))
green = lambda a: (a[:, :, 1] > a[:, :, 0] + 30).sum()
front = abs(deep - [0xe5, 0x43, 0x2d]).max(axis=2) < 6; s = L('partside'); n = (abs(s - deep).max(axis=2) > 8)[front].sum(); print('a side of its own colour, its front as it was: ' + ('yes' if green(s) > 2000 and n < .03 * front.sum() else 'NO') + ' (%d pixels; %d of its front, its last points to the edge, changed)' % (green(s), n))
print('a back of its own colour: ' + ('yes' if green(L('partback')) > 2000 else 'NO') + ' (%d pixels)' % green(L('partback')))
r = L('rim2'); blue = (r[:, :, 2] > r[:, :, 0] + 40).sum(); print('a word\'s further strokes stand in its depth: ' + ('yes' if green(r) > 500 and blue > 500 and seen(r).sum() > 1.1 * seen(L('rim0')).sum() else 'NO') + ' (%d green, %d blue pixels)' % (green(r), blue))
t = L('stair'); print('a Step bevel makes a stair of each, their colours kept: ' + ('yes' if (abs(t - r).max(axis=2) > 8).sum() > 2000 and green(t) > .5 * green(r) else 'NO') + ' (%d pixels changed, %d green)' % ((abs(t - r).max(axis=2) > 8).sum(), green(t)))
print('a bevel of its own colour: ' + ('yes' if green(L('partbevel')) > 2000 else 'NO') + ' (%d pixels)' % green(L('partbevel')))
n = ((L('fxshaded') - L('fxlit')).sum(axis=2) > 8).sum(); print('with an effect on, a flat thing throws its shadow too: ' + ('yes' if n > 2000 else 'NO') + ' (%d pixels)' % n)
"
