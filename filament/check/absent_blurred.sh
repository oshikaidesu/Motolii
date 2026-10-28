#!/bin/zsh
# AbsentMotion's effects that read the picture blurred (Inner Glow, Feather, Soft Glow...: AbsentMotion's effects.cpp
# gives them a Gaussian of sigma blur/3; here two passes across and down, effects/from_absent.py): each, put on a photo
# as its own effect, draws without trouble and changes the photo from the photo without it (compared in RGB: Pillow's
# getbbox of an RGBA difference reads its alpha alone, all 0 between two opaque pictures).
#   ./check/absent_blurred.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
names=(${(f)"$(grep -l 'pass@blurPicture' effects/am_*.wgsl)"})
cmds="add picture $PWD/fx_sample.jpg\nat 0\nstill $W/none.png\n"
for f in $names; do cmds+="say 1 fx $PWD/$f\nat 0\nstill $W/${f:t:r}.png\nsay 1 fx none\n"; done
printf "$cmds" | ./edit.sh $W/b.motolii > $W/said.txt 2>&1
python3 - $W ${names:t:r} <<'PY'
import sys
from PIL import Image, ImageChops
W, names = sys.argv[1], sys.argv[2:]
base = Image.open(f"{W}/none.png").convert('RGB')
apart = {}
for n in names:
    try: apart[n] = sum(1 for v in ImageChops.difference(base, Image.open(f"{W}/{n}.png").convert('RGB')).convert('L').getdata() if v > 12)
    except OSError: apart[n] = -1
same = [n for n, k in apart.items() if k <= 1000]
print(f"AbsentMotion's blurred-picture effects draw: {'yes' if not same else 'NO'} ({len(names) - len(same)} of {len(names)} change the photo, the least by {min(apart.values())} pixels; not: {same})")
PY
grep -a "is not taken" $W/said.txt | head -3
rm -rf $W
