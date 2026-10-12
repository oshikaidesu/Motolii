#!/bin/zsh
# The shapes picture (shapes.mm): a glass ring, a twenty-faced solid and letters on pale paper.  ./shapes.sh  ->  out/shapes.png
cd "$(dirname "$0")"
F=third_party/filament
[ out/shapes -nt shapes.mm ] && [ out/shapes -nt kit.h ] || ./build_one.sh shapes || exit 1
for m in floor_lit poster_glass glasslight shadow_only; do [ $m.filamat -nt $m.mat ] && [ $m.filamat -nt spectrum.glsl ] || $F/bin/matc -a metal -p desktop -o $m.filamat $m.mat || exit 1; done
[ -f out/lightroom/lightroom_ibl.ktx ] || $F/bin/cmgen -q -x out/lightroom --format=ktx --size=512 third_party/filament_src/third_party/environments/lightroom_14b.hdr || exit 1
./out/shapes
