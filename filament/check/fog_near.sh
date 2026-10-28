#!/bin/zsh
# Where fog begins (the stage's fog_near, Filament's FogOptions distance; Alight Motion's fog near): a red cube under
# thick fog begun at the camera is lost in it; the fog begun far past it, it is red.
#   ./check/fog_near.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
for n in 0 100; do
    printf 'add solid cube\nsay 1 fill ff0000\nset 0 fog 1\nset 0 fog_near %s\nstill %s/f%s.png\n' $n $W $n | ./edit.sh $W/f$n.motolii > /dev/null 2>&1
done
python3 -c "
from PIL import Image
at = lambda n: (lambda im: im.getpixel((im.size[0] // 2, im.size[1] // 2)))(Image.open('$W/f%d.png' % n).convert('RGB'))
ok = at(0)[1] > 200 and at(100)[0] > 200 and at(100)[1] < 120
print('fog begins where it is said: ' + ('yes' if ok else 'NO (%s %s)' % (at(0), at(100))))"
rm -rf $W
