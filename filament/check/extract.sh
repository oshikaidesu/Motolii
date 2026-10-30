#!/bin/zsh
# The work area taken out and the gap closed (After Effects' Edit > Extract Work Area; golf (72), P18): a rect over 0-10 s
# keyed on x, another over 6-12 s, the work area 4-6 s. The first is cut at 4, its part after brought back to 4-8 going on
# as it went (x at 4 what it was at 6) and lying just above the part before it (After Effects' Split Layer); the second
# brought back to 4-10; the document 10 s; one step, undone whole.
#   ./check/extract.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
printf "add shape rect\nset 1 duration 10\nkey 1 x 2 0\nkey 1 x 8 1\nadd shape rect\nset 2 start 6\nset 2 duration 6\n" | ./edit.sh $W/x.motolii > /dev/null 2>&1
sqlite3 $W/x.motolii "insert into fact(thing, name, at, value) values(0, 'work_in', 0, 4), (0, 'work_out', 0, 6);"
cp $W/x.motolii $W/back.motolii
printf "later extract 0\nvalue 3 x 4\n" | ./edit.sh $W/x.motolii > $W/said.txt 2>&1
printf "later extract 0\nundo\n" | ./edit.sh $W/back.motolii > /dev/null 2>&1
bars=$(sqlite3 $W/x.motolii "select group_concat(t, ' ') from (select thing || ':' || printf('%g', coalesce((select value from fact f where f.thing = k.thing and name = 'start'), 0)) || '+' || printf('%g', (select value from fact f where f.thing = k.thing and name = 'duration')) as t from fact k where name = 'kind' and thing <> 0 order by thing);")
x=$(grep -a "^  " $W/said.txt | tail -1 | tr -d ' ')
lie=$(sqlite3 $W/x.motolii "select group_concat(thing, ' ') from (select thing from fact where name = 'order' order by value);")
back=$(sqlite3 $W/back.motolii "select count(*) from fact where name = 'kind' and thing <> 0;")
python3 -c "
ok = '$bars' == '1:0+4 2:4+6 3:4+4' and '$lie' == '1 3 2' and abs(float('${x:-9}') - 4 / 6) < 1e-3 and '$back' == '2'
print('the work area taken out closes the gap: ' + ('yes' if ok else 'NO') + ' (bars $bars, 1:0+4 2:4+6 3:4+4 asked; x at 4 ${x:-none}, 0.6667 asked; laid from the back $lie, 1 3 2 asked; undone: $back things, 2 asked)')"
rm -rf $W
