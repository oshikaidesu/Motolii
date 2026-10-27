#!/bin/zsh
# The README's 8 seconds: the calls of mv.txt (a recipe, a 3D ring, text, a glow), an RGB split on the whole picture,
# written as ~/Movies/Motolii/mv.motolii and mv.mp4. Needs the inside built (README, "Build and open").
#   docs/examples/mv.sh
cd "$(dirname "$0")/../.."
out=$HOME/Movies/Motolii
mkdir -p $out && rm -f $out/mv.motolii
{ cat docs/examples/mv.txt; print "say 0 script $PWD/filament/effects/rgbshift.wgsl"; print "write $out/mv.mp4"; } | filament/edit.sh $out/mv.motolii | grep -E 'NOT|write'
