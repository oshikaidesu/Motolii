#!/bin/zsh
# AbsentMotion's shapes (Apache-2.0: github.com/Comett1225/AbsentMotion, fetched by absent.sh) on the Make shelf as
# recipes, recipes/am_<name>.js: its own shapes.cpp, read as it is, built with Skia (skia.sh's libskia) beside the little
# of its scene it reads (recipes/absent_shapes), each of its shapes.xml a path (recipes/from_absent_shapes.py).
#   ./absent_shapes.sh
cd "$(dirname "$0")"
A=third_party/AbsentMotion K=third_party/skia_src W=$(mktemp -d)
[ -d $A/app ] || ./absent.sh
[ -f $K/out/path/libskia.a ] || ./skia.sh
cp $A/app/src/main/cpp/shapes.cpp $W/   # (apart from its own scene.h: the one beside the tool is read)
clang++ -std=c++20 -O1 -I recipes/absent_shapes -I $K recipes/absent_shapes/main.cpp $W/shapes.cpp $K/out/path/libskia.a -framework CoreFoundation -o $W/shapes || exit 1
rm -f recipes/am_*.js(N)
python3 recipes/from_absent_shapes.py $A/app/src/main/assets/shapes.xml $W/shapes recipes
rm -rf $W
