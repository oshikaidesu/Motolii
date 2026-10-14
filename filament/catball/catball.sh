#!/bin/zsh
# The dancing cat knocking balls about (2D, Box2D): its keyed film's outline baked (bake.py, OpenCV), the balls
# simulated against it and baked to keys (sim.c), the document made of the film and the balls, and written.
#   ./catball/catball.sh [balls] [seed]
cd "$(dirname "$0")/.."
mkdir -p out/catball ../reports/show/catball
[ -f out/catball/happy.outline ] || python3 -I catball/bake.py out/vid/happy.mp4 out/catball/happy.outline || exit 1
cc -std=c17 -O2 catball/sim.c -I/opt/homebrew/include -L/opt/homebrew/lib -lbox2d -o out/catball/sim || exit 1
out/catball/sim out/catball/happy.outline ${1:-40} ${2:-7} > out/catball/balls.txt || exit 1
rm -f out/catball/catball.motolii
{
    printf "say 0 background ffffff\nsay 0 behind colour\nadd film $PWD/out/vid/happy.mp4\nset 1 size 3.2154\nset 1 keyed 1\n"
    grep "^add\|^set" out/catball/balls.txt
    n=$(grep -c "^add" out/catball/balls.txt)
    for i in $(seq 2 $((n + 1))); do echo "say $i fill $(echo "6437ff ff24e0 000000 4ffd7a" | cut -d' ' -f$(( (i % 4) + 1 )))"; done
    grep "^key\|^way" out/catball/balls.txt
    echo "write $PWD/../reports/show/catball/catball.mp4"
} | SIZE=1920x1080 ./edit.sh out/catball/catball.motolii | grep "NOT\|write"
