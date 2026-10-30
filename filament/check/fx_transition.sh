#!/bin/zsh
# A transition put on a thing at a moment runs there: its progress keyed 0 then and 1 a second after (After Effects'
# transition effects, Transition Completion 0 to 100%, its presets' keys from the current time; a second, the editors'
# length of a transition); put on at the thing's own start, 1 then 0, it comes in (Premiere Pro's transition on a
# clip's head edge); an effect that is not a transition gets no keys (golf (65): P16 keyed each by hand).
#   ./check/fx_transition.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
printf "add shape rect\nat 2\nlater fx_file 1 $PWD/effects/Circle Open.fs\nlater fx_file 1 $PWD/effects/Glitch Shifter.fs\n" | ./edit.sh $W/t.motolii > /dev/null 2>&1
keys=$(sqlite3 $W/t.motolii "select group_concat(printf('%g=%g', at, value), ' ') from (select at, value from fact where thing = 1 and name like '%progress' order by at);")
other=$(sqlite3 $W/t.motolii "select count(*) from fact where thing = 1 and name like 'js_Glitch_Shifter_%';")
[ "$keys" = "2=0 3=1" ] && [ "$other" = 0 ] && echo "a transition put on runs from the playhead: yes" || echo "a transition put on runs from the playhead: NO (its progress: ${keys:-none}, 0 at 2 and 1 at 3 asked; another effect's facts: $other, 0 asked)"
printf "add shape rect\nset 1 start 2\nat 2\nlater fx_file 1 $PWD/effects/Circle Open.fs\n" | ./edit.sh $W/i.motolii > /dev/null 2>&1
come=$(sqlite3 $W/i.motolii "select group_concat(printf('%g=%g', at, value), ' ') from (select at, value from fact where thing = 1 and name like '%progress' order by at);")
[ "$come" = "2=1 3=0" ] && echo "a transition put on at a thing's start brings it in: yes" || echo "a transition put on at a thing's start brings it in: NO (its progress: ${come:-none}, 1 at 2 and 0 at 3 asked)"
rm -rf $W
