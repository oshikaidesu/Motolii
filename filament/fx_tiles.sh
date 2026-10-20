#!/bin/zsh
# Each whole-screen effect's picture, for its tile on the Effects shelf: the Poster recipe with the effect on, written
# beside the effect as a .png (effects/twirl.wgsl -> effects/twirl.png).   ./fx_tiles.sh [effects/one.wgsl ...]
cd "$(dirname "$0")"
fx=(${@:-effects/*.wgsl})
doc=$(mktemp -d)/tiles.motolii
{
    echo "make Poster"
    echo "at 1"
    for f in $fx; do
        echo "say 0 script $PWD/$f"
        echo "still $PWD/${f:r}.png"
    done
} | SIZE=256x256 ./edit.sh $doc | grep -v "^still: done\|^say: done"
