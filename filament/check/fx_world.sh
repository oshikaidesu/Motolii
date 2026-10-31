#!/bin/zsh
# A thing with effects stays in the world: a flat one stood there (thing_fx_stand, After Effects' layer: effects on its own
# picture, then its place), a solid showing its picture itself (doc_thing_shell, three.js' webgpu_portal.html). An effect
# that changes nothing leaves depth, fog and the thing's way up as they were, and what an effect makes keeps to a flat
# thing as it goes to the stage's edge.
#   ./check/fx_world.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
cat > $W/nothing.fs <<'FS'
/*{ "INPUTS": [{ "NAME": "inputImage", "TYPE": "image" }] }*/
void main() { gl_FragColor = IMG_THIS_PIXEL(inputImage); }
FS
cat > $W/bars.fs <<'FS'
/*{ "INPUTS": [{ "NAME": "inputImage", "TYPE": "image" }] }*/
void main() { vec4 c = IMG_THIS_PIXEL(inputImage); gl_FragColor = vec4(vec3(step(.5, fract(isf_FragNormCoord.x * 20.))), c.a); }
FS
shot() { printf "$2\nstill $W/$1.png\n" | ./edit.sh $W/$1.motolii > /dev/null 2>&1 }
behind="add solid cube\nsay 1 fill 3366ff\nadd shape rect\nsay 2 fill ff3366\nsay 2 space 2.5d\nset 2 z -3\nset 2 x 0.9"
fog="add shape rect\nsay 1 fill ff3366\nsay 1 space 2.5d\nset 1 z -8\nset 0 fog 1\nset 0 fog_near 0\nset 0 fog_far 20"
star="add shape star\nsay 1 fill ffffff\nset 1 rotate 11"
shot behind "$behind"; shot behind_fx "$behind\nlater fx_file 2 $W/nothing.fs"
shot fog "$fog"; shot fog_fx "$fog\nlater fx_file 1 $W/nothing.fs"
shot star "$star"; shot star_fx "$star\nlater fx_file 1 $W/nothing.fs"
crossed="add solid cube\nsay 1 fill 3366ff\nadd solid cube\nsay 2 fill ff3366\nset 2 x 0.5\nset 2 rotate_y 45\nset 2 rotate_x 30"
foggy="add solid cube\nsay 1 fill 3366ff\nset 1 rotate_y 35\nset 1 z -8\nset 0 fog 1\nset 0 fog_near 0\nset 0 fog_far 20"
shot crossed "$crossed"; shot crossed_fx "$crossed\nlater fx_file 2 $W/nothing.fs"
shot foggy "$foggy"; shot foggy_fx "$foggy\nlater fx_file 1 $W/nothing.fs"
shot middle "add shape rect\nsay 1 fill ffffff\nlater fx_file 1 $W/bars.fs"
shot edge "add shape rect\nsay 1 fill ffffff\nset 1 x 0.8\nlater fx_file 1 $W/bars.fs"
python3 -c "
from PIL import Image, ImageChops
def changed(a, b):
    h = ImageChops.difference(Image.open('$W/' + a + '.png').convert('L'), Image.open('$W/' + b + '.png').convert('L')).histogram()
    return sum(h[64:])
def bars(name):
    im = Image.open('$W/' + name + '.png').convert('L'); w, h = im.size
    v = [im.getpixel((x, h // 2)) for x in range(w)]
    at = [i for i in range(w - 1) if (v[i] - 127.5) * (v[i + 1] - 127.5) < 0]
    return (at[-1] - at[0]) / (len(at) - 1) if len(at) > 2 else 0
for what, a, b in (('behind a solid it stays behind', 'behind', 'behind_fx'), ('fog stays on it', 'fog', 'fog_fx')):
    n = changed(a, b); print(what + ': ' + ('yes' if n == 0 else 'NO') + ' (%d pixels changed)' % n)
for what, a, b in (('two solids stay crossed', 'crossed', 'crossed_fx'), ('fog stays on a solid', 'foggy', 'foggy_fx')):
    n = changed(a, b); print(what + ': ' + ('yes' if n < 50 else 'NO') + ' (%d pixels changed)' % n)
n = changed('star', 'star_fx'); print('it stays the way up it was: ' + ('yes' if n < 2000 else 'NO') + ' (%d pixels changed)' % n)
a, b = bars('middle'), bars('edge'); print('its pattern keeps to it at the edge: ' + ('yes' if abs(a - b) < .5 else 'NO') + ' (%.1f and %.1f pixels a bar)' % (a, b))"
rm -rf $W
