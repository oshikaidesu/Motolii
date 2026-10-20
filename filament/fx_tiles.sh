#!/bin/zsh
# Each effect's picture, for its tile on the Effects shelf: one plain sample (Aa, a red circle passing, a blue square, on
# paper) with the effect on. A whole-screen effect's is written beside it (effects/twirl.wgsl -> effects/twirl.png); a
# thing's effect (the inside's own, put on all three) as effects/thing_<its value>.png.   ./fx_tiles.sh [effects/one.wgsl ...]
cd "$(dirname "$0")"
fx=(${@:-effects/*.wgsl})
doc=$(mktemp -d)/tiles.motolii
(( $# )) || things=$(echo effects | ./edit.sh $(mktemp -d)/list.motolii | grep "	")   # (the inside's own effects: name, value, ...)
{
    echo "say 0 behind colour"
    echo "say 0 background f1ece2"
    echo "add shape circle"; echo "say 1 fill e5432d"; echo "set 1 size 0.9"; echo "set 1 y 0.35"; echo "key 1 x 0 -0.75"; echo "key 1 x 2 0.75"
    echo "add shape rect"; echo "say 2 fill 2a62c9"; echo "set 2 size 0.9"; echo "set 2 x -0.45"; echo "set 2 y -0.4"; echo "set 2 rotate 15"
    echo "add text /System/Library/Fonts/Supplemental/Arial Black.ttf"; echo "say 3 text Aa"; echo "say 3 fill 151515"; echo "set 3 size 1"
    echo "at 1.4"
    for f in $fx; do
        echo "say 0 script $PWD/$f"
        n=$(sed -n 's/.*--history@\([0-9]*\).*/\1/p' $f | head -1)   # (an effect of what was: as many pictures before as its head says it keeps, one a frame)
        for (( k = ${n:-0}; k > 0; k-- )); do echo "at $(( 1.4 - k / 60.0 ))"; echo "still $doc.warm.png"; done
        echo "at 1.4"
        echo "still $PWD/${f:r}.png"
    done
    echo "say 0 script none"
    echo "at 1.9"; echo "still $doc.png"; echo "at 1.4"   # (the sample as it is, to tell a picture that shows nothing done)
    print -r -- "$things" | while IFS=$'\t' read -r name value _; do
        [[ -n $name ]] || continue
        for t in 1 2 3; do echo "effect $t $name"; done
        echo "at 1.9"   # (half a second on: what it does from where it is put, Trim Paths drawing on)
        echo "still $PWD/effects/thing_$value.png"
        echo "at 1.4"
        for t in 1 2 3; do echo "effect $t $name"; done   # (off again: the next one alone)
    done
} | SIZE=512x512 ./edit.sh $doc | grep -v "^still: done\|^say: done\|^set: done\|^key: done\|^add: done\|^effect: done\|^thing "
for f in effects/thing_*.png(N); do cmp -s $f $doc.png && rm $f; done   # (the same as the sample: that effect is not seen on it, the shelf draws its own)
for f in $fx effects/thing_*.png(N); do sips -Z 256 "${f:r}.png" >/dev/null; done   # (drawn at twice the tile, made small: the letters' edges stay clean)
