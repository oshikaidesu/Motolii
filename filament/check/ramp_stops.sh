#!/bin/zsh
# A gradient's colours may say where each stands (ff0000 0000ff@0.2: CSS's colour stop positions, SVG's stop offset,
# ThorVG's ColorStop offset): the blue then begins a fifth of the way along, not halfway, so the picture differs from
# the evenly spaced one; and one said before the one ahead of it stands where that one does (CSS Images 3).
#   ./check/ramp_stops.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
for n in even placed; do
    printf 'add shape rect\nset 1 scale_x 3\nset 1 scale_y 3\n' | ./edit.sh $W/$n.motolii > /dev/null 2>&1
    sqlite3 $W/$n.motolii "insert into fact(thing, name, at, value) values(1, 'fill', 0.0, '$([[ $n == even ]] && print -n 'ff0000 0000ff' || print -n 'ff0000 0000ff@0.2')')"
    printf 'still %s/%s.png\n' $W $n | ./edit.sh $W/$n.motolii > /dev/null 2>&1
done
moved=$(magick compare -metric AE -fuzz 5% $W/even.png $W/placed.png null: 2>&1 | cut -d' ' -f1)
(( moved > 10000 )) && print "a gradient's colours stand where they say: yes ($moved pixels differ from the even one)" || print "a gradient's colours stand where they say: NO ($moved)"
rm -rf $W
