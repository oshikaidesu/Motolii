#!/bin/zsh
# A camera given AbsentMotion's Auto-Shake shakes the view: its place, which it looks from, is moved by the motion, as
# AbsentMotion's camera is placed by its own motions (camera.cpp LayerView, styles.cpp OwnAt: ApplyMotion); still again
# with it taken off. An effect with a picture to lay it on is refused: a camera has none (golf ㊴: shaken only by a
# Null made its parent). A cube (a thing in the world) is what the camera is seen by.
#   ./check/camera_motion.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
A=$PWD/effects/am_autoshake.am   # (put on and taken off as the shelf does: fx_file)
printf "add solid cube\nset 1 duration 5\nadd camera eye\nset 2 duration 5\nlater fx_file 2 $A\nat 0.3\nstill $W/s3.png\nat 0.6\nstill $W/s6.png\nlater fx_file 2 $A\nat 0.3\nstill $W/n3.png\nat 0.6\nstill $W/n6.png\nlater fx_file 2 $PWD/effects/Shake.fs\n" | ./edit.sh $W/cam.motolii > $W/said.txt 2>&1
taken=$(grep -c "^later: done" $W/said.txt) refused=$(grep -c "^later: NOT done: a camera takes motions only" $W/said.txt)
python3 -c "
from PIL import Image, ImageChops
def moved(a, b): return ImageChops.difference(Image.open('$W/%s.png' % a).convert('RGB'), Image.open('$W/%s.png' % b).convert('RGB')).getbbox() is not None
shaken, still = moved('s3', 's6'), not moved('n3', 'n6')
print('a camera is shaken by a motion on it: ' + ('yes' if $taken == 2 and $refused == 1 and shaken and still else 'NO') + ' (taken %d of 2, an effect refused %d of 1, shaken %s, still without %s)' % ($taken, $refused, shaken, still))"
rm -rf $W
