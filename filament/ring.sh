#!/bin/zsh
# The ring picture (ring.mm): a twisted glass ring on pale paper.  ./ring.sh -> out/ring.png   ./ring.sh close -> out/ring_close.png
cd "$(dirname "$0")"
F=third_party/filament
[ out/ring -nt ring.mm ] && [ out/ring -nt kit.h ] || ./build_one.sh ring || exit 1
for m in paper floor_shade poster_glass room_glass glasslight; do [ $m.filamat -nt $m.mat ] && [ $m.filamat -nt spectrum.glsl ] || $F/bin/matc -a metal -p desktop -o $m.filamat $m.mat || exit 1; done
[ -f out/lightroom/lightroom_ibl.ktx ] || $F/bin/cmgen -q -x out/lightroom --format=ktx --size=512 third_party/filament_src/third_party/environments/lightroom_14b.hdr || exit 1
[ -f out/small_studio/small_studio_ibl.ktx ] || $F/bin/cmgen -q -x out/small_studio --format=ktx --size=512 third_party/filament_src/third_party/environments/studio_small_02_2k.hdr || exit 1
./out/ring "$@"
