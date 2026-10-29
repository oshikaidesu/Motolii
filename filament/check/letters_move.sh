#!/bin/zsh
# A text split into its letters moves letter by letter (animators/bounce_in.js, Cavalry's Bounce In): its letters fall in,
# so a moment early and one late differ. With a Return after the words too (a last line left empty is no line: the golf's
# MOTOLII typed and Returned did not move at all).
#   ./check/letters_move.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
for n in 1 2; do
    printf 'add text %s/out/arial_black.ttf\nsay 1 split letters\nsay 1 script %s/animators/bounce_in.js\n' $PWD $PWD | ./edit.sh $W/t$n.motolii > /dev/null 2>&1
    sqlite3 $W/t$n.motolii "update fact set value = 'MOTOLII'$([[ $n == 2 ]] && print -n " || char(10)") where thing = 1 and name = 'text'"
    printf 'at 0.1\nstill %s/a%s.png\nat 1.5\nstill %s/b%s.png\n' $W $n $W $n | ./edit.sh $W/t$n.motolii > /dev/null 2>&1
done
one=$(magick compare -metric AE -fuzz 2% $W/a1.png $W/b1.png null: 2>&1 | cut -d' ' -f1)
two=$(magick compare -metric AE -fuzz 2% $W/a2.png $W/b2.png null: 2>&1 | cut -d' ' -f1)
(( one > 10000 && two > 10000 )) && print "a text's letters move: yes (between 0.1 s and 1.5 s: $one pixels, with a Return after it $two)" || print "a text's letters move: NO ($one, with a Return after it $two)"
rm -rf $W
