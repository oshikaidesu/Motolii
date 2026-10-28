#!/bin/zsh
# A motion blur when written (render_blur: After Effects' 180° shutter, so many moments laid on one picture): a rect
# crossing the picture in a second is smeared along its way, so far more of a picture is between the rect's colour
# and the ground's than when written sharp; a rect standing still is as sharp either way.
#   ./check/motion_blur.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
for mode in sharp blur still; do
    keys='key 1 x 0 -1\nkey 1 x 1 1\n'; [[ $mode == still ]] && keys=''
    extra=''; [[ $mode != sharp ]] && extra='set 0 render_blur 8\n'
    printf "add shape rect\n${keys}${extra}write $W/$mode.mp4\n" | ./edit.sh $W/$mode.motolii > /dev/null 2>&1
    ffmpeg -v error -i $W/$mode.mp4 -vf "select=eq(n\,30)" -frames:v 1 -pix_fmt gray -f rawvideo $W/$mode.raw
done
python3 - $W <<'PY'
import sys, collections
W = sys.argv[1]
def between(name):
    b = open(f"{W}/{name}.raw", "rb").read()
    top = [v for v, _ in collections.Counter(b).most_common(2)]
    return sum(1 for v in b if all(abs(v - t) > 12 for t in top))
sharp, blur, still = between("sharp"), between("blur"), between("still")
ok = blur > 4 * max(sharp, 1) and still <= max(sharp, 1) * 2
print(f"a moving thing is smeared when written with a blur: {'yes' if ok else 'NO'} (between pixels: sharp {sharp}, blurred {blur}, still {still})")
PY
rm -rf $W
