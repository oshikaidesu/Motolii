#!/bin/zsh
# Does glass read what is behind it where it is (no shift sideways)? A glass cube that does not bend light (ior 1) over
# a white sheet's sharp edge: the edge seen through it against beside it, to a hundredth of a point.
cd "$(dirname "$0")/.."
rm -f out/glass_shift.motolii out/glass_shift.png
printf 'add shape rect\nsay 1 fill ffffff\nset 1 x -1.5\nset 1 z -1.5\nset 1 size 3\nadd solid cube\nsay 2 fill glass\nset 2 ior 1\nset 2 dispersion 0\nset 2 size 1\nstill %s/out/glass_shift.png\n' $PWD | SIZE=1920x1080 ./edit.sh out/glass_shift.motolii > /dev/null 2>&1
python3 -I - $PWD/out/glass_shift.png <<'PY'
import sys
from PIL import Image
im = Image.open(sys.argv[1]).convert('L'); px = im.load()
def edge(y):
    for x in range(900, 1100):
        a, b = px[x - 1, y], px[x, y]
        if a > 128 >= b: return x - 1 + (a - 128) / max(a - b, 1)
through = [e for e in (edge(y) for y in range(420, 661, 10)) if e]
beside = [e for e in (edge(y) for y in list(range(180, 361, 10)) + list(range(740, 901, 10))) if e]
off = sum(through) / len(through) - sum(beside) / len(beside)
print("glass reads what is behind it where it is: %s (off by %.2f points)" % ("yes" if abs(off) < 0.25 else "NO", off))
PY
