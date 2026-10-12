#!/bin/zsh
# The blocks picture (blocks.mm): glass blocks on pale paper in hard sun.  ./blocks.sh  ->  out/blocks.png
cd "$(dirname "$0")"
F=third_party/filament
[ out/blocks -nt blocks.mm ] && [ out/blocks -nt kit.h ] || ./build_one.sh blocks || exit 1
for m in floor_lit glassbox glasslight shadow_only; do [ $m.filamat -nt $m.mat ] && [ $m.filamat -nt spectrum.glsl ] || $F/bin/matc -a metal -p desktop -o $m.filamat $m.mat || exit 1; done
# the sky that fills the shadows: the bright studio that comes with Filament
[ -f out/lightroom/lightroom_ibl.ktx ] || $F/bin/cmgen -q -x out/lightroom --format=ktx --size=512 third_party/filament_src/third_party/environments/lightroom_14b.hdr || exit 1
./out/blocks
