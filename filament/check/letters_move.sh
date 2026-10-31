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
printf 'add text %s/out/arial_black.ttf\nsay 1 split letters\nsay 1 script %s/animators/bounce_in.js\nset 1 duration 8\nat 6\nstill %s/whole.png\nsplit 1 3\nat 3.05\nstill %s/cut.png\n' $PWD $PWD $W $W | ./edit.sh $W/c.motolii > /dev/null 2>&1
cut=$(magick compare -metric AE -fuzz 2% $W/whole.png $W/cut.png null: 2>&1 | cut -d' ' -f1)
(( cut < 2000 )) && print "letters at rest stay at rest past a cut: yes ($cut pixels)" || print "letters at rest stay at rest past a cut: NO ($cut pixels)"
(( one > 10000 && two > 10000 )) && print "a text's letters move: yes (between 0.1 s and 1.5 s: $one pixels, with a Return after it $two)" || print "a text's letters move: NO ($one, with a Return after it $two)"
# Split into words, its Opacity keyed and staggered: a word fades as late as it moves (After Effects' Based On Words, GSAP's stagger).
printf 'add text %s/out/arial_black.ttf\nsay 1 fill ffffff\nsay 1 split words\nset 1 stagger 1\nset 1 size 0.1\nkeys 1 opacity 0 0 0.2 1\n' $PWD | ./edit.sh $W/w.motolii > /dev/null 2>&1
sqlite3 $W/w.motolii "update fact set value = 'ONE TWO' where thing = 1 and name = 'text'"
printf 'at 0.3\nstill %s/w.png\nat 1.3\nstill %s/w2.png\n' $W $W | ./edit.sh $W/w.motolii > /dev/null 2>&1
early=$(magick $W/w.png -alpha off -gravity East -crop 50%x100%+0+0 -format '%[fx:mean]' info:)
first=$(magick $W/w.png -alpha off -gravity West -crop 50%x100%+0+0 -format '%[fx:mean]' info:)
late=$(magick $W/w2.png -alpha off -gravity East -crop 50%x100%+0+0 -format '%[fx:mean]' info:)
(( first > 0.003 && early < first / 20 && late > 0.003 )) && print "a word fades as late as it moves: yes" || print "a word fades as late as it moves: NO (first $first, last early $early, last late $late)"
rm -rf $W
