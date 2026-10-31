#!/bin/zsh
# An effect's number typed past its author's range, and what it then draws: the range is where its knob ends, as After
# Effects' slider range is not its valid range (PF_FloatSliderDef); a count stays closed, a range from 0 holds at 0; a
# thing's picture grows by as much, so a ripple comes out of the box it was cut by, and within the range all is
# as it was.
#   ./check/fx_reach.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
F=$PWD/effects/am_circularripple.wgsl
R=js_am_circularripple_radius S=js_am_circularripple_strength
thing="add shape rect\nsay 1 fill ffffff\nlater fx_file 1 $F\nset 1 $S 0.9"
said=$(printf "$thing\nset 1 $R 16\nset 1 $S 10\nset 1 $R -1\nset 1 js_am_circularripple_frequency 90\nstill $W/open.png\nvalue 1 $R 0" | ./edit.sh $W/a.motolii 2>&1)
printf "$thing\nset 1 $R 0.8\nstill $W/in.png" | ./edit.sh $W/b.motolii > /dev/null 2>&1
printf "add shape rect\nsay 1 fill ffffff\nlater fx_file 1 $PWD/effects/chroma.wgsl\nset 1 js_chroma_origin_x 9000" | ./edit.sh $W/c.motolii 2>&1 | grep -q 'NOT done: js_chroma_origin_x' && echo "a number that counts in whole steps stays closed: yes" || echo "a number that counts in whole steps stays closed: NO"
echo "$said" | grep -q '16.0000' && echo "past its range: yes" || echo "past its range: NO"
echo "$said" | grep -q "NOT done: $R is from 0" && echo "never below 0: yes" || echo "never below 0: NO"
python3 - $W <<'P'
import sys
from PIL import Image
w=sys.argv[1]
def box(n): return Image.open(f'{w}/{n}.png').convert('L').point(lambda v:255 if v>24 else 0).getbbox()
a,b=box('open'),box('in')
print('in',b,'open',a)
print('out of the box it was cut by:', 'yes' if a and b and (a[2]-a[0])*(a[3]-a[1])>2*(b[2]-b[0])*(b[3]-b[1]) else 'NO')
P
rm -rf $W
