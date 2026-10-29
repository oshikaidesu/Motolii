#!/bin/zsh
# A camera given a parent, as the inspector's Target > Layer gives it (After Effects parents a camera to a Null and
# moves the Null; golf P08): taken (it was refused, "a camera has no size": doc_parent set a size a camera has not),
# and the view goes where its parent goes, keyed along x or shaken by AbsentMotion's Auto-Shake on it (its motion is
# the thing's place, motion.cpp ApplyMotion); still again with the shake taken off. A cube (a thing in the world) is
# what the camera is seen by.
#   ./check/camera_parent.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
printf "add shape null\nset 1 duration 5\nadd solid cube\nset 2 duration 5\nadd camera eye\nset 3 duration 5\nset 3 parent 1\nvalue 3 parent 0\nkey 1 x 0 0\nkey 1 x 1 1\nat 0.3\nstill $W/k3.png\nat 0.6\nstill $W/k6.png\nunkey 1 x 0\nunkey 1 x 1\nsay 1 fx $PWD/effects/am_autoshake.am\nat 0.3\nstill $W/s3.png\nat 0.6\nstill $W/s6.png\nsay 1 fx none\nat 0.3\nstill $W/n3.png\nat 0.6\nstill $W/n6.png\n" | ./edit.sh $W/cam.motolii > $W/said.txt 2>&1
taken=$(grep -A1 "^set: done" $W/said.txt | grep -c "1.0000")
python3 -c "
from PIL import Image, ImageChops
def moved(a, b): return ImageChops.difference(Image.open('$W/%s.png' % a).convert('RGB'), Image.open('$W/%s.png' % b).convert('RGB')).getbbox() is not None
keyed, shaken, still = moved('k3', 'k6'), moved('s3', 's6'), not moved('n3', 'n6')
print('a camera takes a parent and goes with it: ' + ('yes' if $taken and keyed and shaken and still else 'NO') + ' (taken %s, keyed %s, shaken %s, still without %s)' % (bool($taken), keyed, shaken, still))"
rm -rf $W
