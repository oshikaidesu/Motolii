#!/bin/zsh
# Two cameras, one after the other (Alight Motion switches between cameras so): the first, zoomed in, on for the first
# second; the second, as it comes, from then on (the cube three seconds long, so the document runs past both). A cube (a thing in the world, as 2.5D and 3D things are: a 2D one keeps to the camera, as stuck to it) is seen larger through the first than through the second (the
# stage looks through the nearest camera that is on: doc_moment.h).
#   ./check/camera_switch.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
printf 'add solid cube\nsay 1 fill ff0000\nset 1 size 0.3\nset 1 duration 3\nadd camera near\nset 2 zoom 2\nset 2 duration 1\nadd camera far\nset 3 start 1\nat 0.5\nstill %s/near.png\nat 1.5\nstill %s/far.png\n' $W $W | ./edit.sh $W/cams.motolii > /dev/null 2>&1
python3 -c "
from PIL import Image
def wide(n):
    im = Image.open('$W/%s.png' % n).convert('RGB'); w, h = im.size
    return sum(1 for x in range(w) if (lambda p: p[0] > p[1] + 60)(im.getpixel((x, h // 2))))   # (its red, lit or in shade)
a, b = wide('near'), wide('far')
print('cameras switch in turn: ' + ('yes' if a > 1.5 * b > 0 else 'NO') + ' (%d points wide through the first, %d through the second)' % (a, b))"
rm -rf $W
