#!/bin/zsh
# Each whole-screen effect's picture, for its tile on the Effects shelf: one plain sample (Aa, a red circle passing, a
# blue square, on paper) with the effect on, written beside the effect as a .png (effects/twirl.wgsl -> effects/twirl.png).
#   ./fx_tiles.sh [effects/one.wgsl ...]
cd "$(dirname "$0")"
fx=(${@:-effects/*.wgsl})
doc=$(mktemp -d)/tiles.motolii
{
    echo "say 0 behind colour"
    echo "say 0 background f1ece2"
    echo "add shape circle"; echo "say 1 fill e5432d"; echo "set 1 size 0.9"; echo "set 1 y 0.35"; echo "key 1 x 0 -0.75"; echo "key 1 x 2 0.75"
    echo "add shape rect"; echo "say 2 fill 2a62c9"; echo "set 2 size 0.9"; echo "set 2 x -0.45"; echo "set 2 y -0.4"; echo "set 2 rotate 15"
    echo "add text /System/Library/Fonts/Supplemental/Arial Black.ttf"; echo "say 3 text Aa"; echo "say 3 fill 151515"; echo "set 3 size 1"
    echo "at 1.4"
    for f in $fx; do
        echo "say 0 script $PWD/$f"
        echo "still $PWD/${f:r}.png"
    done
} | SIZE=512x512 ./edit.sh $doc | grep -v "^still: done\|^say: done\|^set: done\|^key: done\|^add: done\|^thing "
for f in $fx; do sips -Z 256 "${f:r}.png" >/dev/null; done   # (drawn at twice the tile, made small: the letters' edges stay clean)
