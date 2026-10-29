#!/bin/zsh
# A gradient's colours are mixed as it says (gradient_blend: CSS Color 4's color-interpolation-method): from blue to
# yellow, sRGB goes through grey, OKLab and round-the-hue do not, and held (Blender's Constant) is two flat halves; so
# each draws a picture other than the sRGB one.
#   ./check/ramp_blend.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
for b in rgb oklab hue steps; do
    printf 'add shape rect\nset 1 scale_x 3\nset 1 scale_y 3\nsay 1 fill 0000ff\n' | ./edit.sh $W/$b.motolii > /dev/null 2>&1
    sqlite3 $W/$b.motolii "update fact set value = '0000ff ffff00' where thing = 1 and name = 'fill'; insert into fact(thing, name, at, value) values(1, 'gradient_blend', 0.0, '$b')"
    printf 'still %s/%s.png\n' $W $b | ./edit.sh $W/$b.motolii > /dev/null 2>&1
done
out=""; ok=1
for b in oklab hue steps; do
    n=$(magick compare -metric AE -fuzz 4% $W/rgb.png $W/$b.png null: 2>&1 | cut -d' ' -f1)
    out="$out $b $n"; (( n > 10000 )) || ok=0
done
(( ok )) && print "a gradient's colours are mixed as it says: yes (pixels other than sRGB's:$out)" || print "a gradient's colours are mixed as it says: NO ($out)"
rm -rf $W
