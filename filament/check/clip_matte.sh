#!/bin/zsh
# Clip to below (Alight Motion's Mask) and its matte turned over (its Exclude): a red rect over a blue circle, clipped
# to it, is red where the circle is and nothing elsewhere; turned over, red outside the circle and not inside it.
#   ./check/clip_matte.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
for inv in 0 1; do
    printf 'add shape circle\nset 1 size 0.5\nsay 1 fill 0000ff\nadd shape rect\nset 2 size 1.5\nsay 2 fill ff0000\nset 2 clip 1\nset 2 clip_invert %s\nstill %s/c%s.png\n' $inv $W $inv | ./edit.sh $W/c$inv.motolii > /dev/null 2>&1
done
python3 -c "
from PIL import Image
def at(i, down):
    im = Image.open('$W/c%d.png' % i).convert('RGB'); w, h = im.size
    return im.getpixel((w // 2, h // 2 + int(h * down)))
red = lambda p: p[0] > 200 and p[1] < 60 and p[2] < 60
ok = red(at(0, 0)) and not red(at(0, .12)) and not red(at(1, 0)) and red(at(1, .12))
print('clipped to below, and turned over: ' + ('yes' if ok else 'NO (%s %s %s %s)' % (at(0, 0), at(0, .12), at(1, 0), at(1, .12))))"
rm -rf $W
