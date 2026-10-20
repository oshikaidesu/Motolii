#!/bin/zsh
# Each effect's picture, for its tile on the Effects shelf: one sample (a hot air balloon, fx_sample.jpg, drifting slowly,
# and a white Aa over it) with the effect on. A whole-screen effect's is written beside it (effects/twirl.wgsl ->
# effects/twirl.png); a thing's effect (the inside's own, put on the Aa) as effects/thing_<its value>.png.
#   ./fx_tiles.sh [effects/one.wgsl ...]
# fx_sample.jpg: "Mid-Hudson balloon festival 15" by Juliancolton, public domain (PD-self), Wikimedia Commons, cut square.
cd "$(dirname "$0")"
fx=(${@:-effects/*.wgsl})
doc=$(mktemp -d)/tiles.motolii
(( $# )) || rm -f effects/thing_*.png   # (made again, none left from before)
(( $# )) || things=$(echo effects | ./edit.sh $(mktemp -d)/list.motolii | grep "	")   # (the inside's own effects: name, value, ...)
{
    echo "add picture $PWD/fx_sample.jpg"; echo "set 1 size 3.6"; echo "key 1 x 0 -0.1"; echo "key 1 x 2 0.1"
    echo "add text /System/Library/Fonts/Supplemental/Arial Black.ttf"; echo "say 2 text Aa"; echo "say 2 fill ffffff"; echo "set 2 size 1"; echo "set 2 y -0.55"
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
        echo "effect 2 $name"
        echo "at 1.9"   # (half a second on: what it does from where it is put, Trim Paths drawing on)
        echo "still $PWD/effects/thing_$value.png"
        echo "at 1.4"
        echo "effect 2 $name"   # (off again: the next one alone)
    done
} | SIZE=512x512 ./edit.sh $doc | grep -v "^still: done\|^say: done\|^set: done\|^key: done\|^add: done\|^effect: done\|^thing "
for f in effects/thing_*.png(N); do cmp -s $f $doc.png && rm $f; done   # (the same as the sample: that effect is not seen on it, the shelf draws its own)
for f in $fx effects/thing_*.png(N); do sips -Z 256 "${f:r}.png" >/dev/null; done   # (drawn at twice the tile, made small: the letters' edges stay clean)
