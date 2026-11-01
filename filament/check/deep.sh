#!/bin/zsh
# A flat thing is a thing at Z 0 (the user, 11/1): its Scale Z opens its sheet into a box its picture is pushed through
# (deep.glsl), nothing filmed, read back or cut anew. At 0 not a pixel changes; opened its sides are seen, its front is
# its picture's own colour, and two boxes cross where they really meet (After Effects' 3D layers intersect).
#   ./check/deep.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
shot() { printf "$2\nstill $W/$1.png\n" | ./edit.sh $W/$1.motolii > /dev/null 2>&1 }
star="add shape star\nsay 1 fill e5432d\nsay 1 space 3d\nset 1 rotate_y 40\nset 1 rotate_x -20"
one="add shape rect\nsay 1 fill e5432d\nset 1 scale_z 1\nset 1 rotate_y 50"
two="add shape rect\nsay 2 fill 2a62c9\nset 2 scale_z 1\nset 2 rotate_y -50"
shot flat "$star"; shot zero "$star\nset 1 scale_z 0"; shot deep "$star\nset 1 scale_z 1"
shot one "$one"; shot crossed "$one\n$two"
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
print('two boxes cross: ' + ('yes' if red[:, :mid].sum() > 4 * red[:, mid:].sum() and blue[:, mid:].sum() > 4 * blue[:, :mid].sum() else 'NO') + ' (red %d left, %d right; blue %d left, %d right)' % (red[:, :mid].sum(), red[:, mid:].sum(), blue[:, :mid].sum(), blue[:, mid:].sum()))"
