#!/bin/zsh
# An effect taken off a thing takes its knobs' values and keys with it, as After Effects' and Alight Motion's effect
# deleted takes its keyframes (golf P15: put on again, the old keys came back; doc_recipes.h doc_fx_file); one step,
# undone together. Its eye (fx_off) is not this: it keeps them.
#   ./check/fx_out_keys.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
F="$PWD/effects/Glitch Shifter.fs"
printf "add shape rect\nsay 1 fx $F\nkey 1 js_Glitch_Shifter_glitch_horizontal 0.5 0.3\nkey 1 js_Glitch_Shifter_glitch_horizontal 1 0\nlater fx_out 1 Glitch Shifter\n" | ./edit.sh $W/out.motolii > /dev/null 2>&1
gone=$(sqlite3 $W/out.motolii "select count(*) from fact where thing = 1 and name like 'js_Glitch_Shifter_%';")
printf "add shape rect\nsay 1 fx $F\nkey 1 js_Glitch_Shifter_glitch_horizontal 0.5 0.3\nkey 1 js_Glitch_Shifter_glitch_horizontal 1 0\nlater fx_out 1 Glitch Shifter\nundo\n" | ./edit.sh $W/back.motolii > /dev/null 2>&1
back=$(sqlite3 $W/back.motolii "select count(*) from fact where thing = 1 and name like 'js_Glitch_Shifter_%';")
[ "$gone" = 0 ] && [ "$back" = 2 ] && echo "an effect taken off takes its keys: yes" || echo "an effect taken off takes its keys: NO (after: $gone of its facts, undone: $back, 0 and 2 asked)"
rm -rf $W
