#!/bin/zsh
# An effect's colour, ISF's one colour input held as three numbers and an alpha (_x _y _z _w), said as one colour as the
# Colors shelf says it: the three put, the alpha as it was (After Effects' and Alight Motion's effect colour is one
# colour; golf (83): white took three number fields).
#   ./check/fx_colour.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
printf "add shape rect\nlater fx_file 1 $PWD/effects/Fade Color.fs\nsay 1 js_Fade_Color_color ff8000\n" | ./edit.sh $W/c.motolii > /dev/null 2>&1
said=$(sqlite3 $W/c.motolii "select group_concat(printf('%.3f', value), ' ') from (select value from fact where thing = 1 and name in ('js_Fade_Color_color_x', 'js_Fade_Color_color_y', 'js_Fade_Color_color_z') order by name);")
alpha=$(sqlite3 $W/c.motolii "select count(*) from fact where thing = 1 and name = 'js_Fade_Color_color_w';")
[ "$said" = "1.000 0.502 0.000" ] && [ "$alpha" = 0 ] && echo "an effect's colour is said as one colour: yes" || echo "an effect's colour is said as one colour: NO (r g b ${said:-none}, 1 0.502 0 asked; the alpha said $alpha times, 0 asked)"
rm -rf $W
