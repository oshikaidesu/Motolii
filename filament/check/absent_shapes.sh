#!/bin/zsh
# AbsentMotion's shapes on the Make shelf (absent_shapes.sh): each of recipes/am_*.js made in a document of its own
# and the stage written as a picture; each one draws something there (more than a few pixels not the ground).
#   ./check/absent_shapes.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d) made=0 empty=()
for f in recipes/am_*.js(N); do
    n=${${(f)"$(<$f)"}[1]#// }; n=${n%%:*}
    printf 'make %s\nstill %s/%s.png\n' "$n" $W ${f:t:r} | ./edit.sh $W/${f:t:r}.motolii > /dev/null 2>&1
    python3 -c "
from PIL import Image
im = Image.open('$W/${f:t:r}.png').convert('RGB').resize((108, 192)); g = im.getpixel((0, 0))
raise SystemExit(sum(im.getpixel((x, y)) != g for x in range(108) for y in range(192)) < 30)" 2>/dev/null && made=$((made + 1)) || empty+=($n)
done
rm -rf $W
(( made >= 54 && ${#empty} == 0 )) && echo "AbsentMotion's shapes each draw: yes ($made)" || echo "AbsentMotion's shapes each draw: NO ($made; not: $empty)"
