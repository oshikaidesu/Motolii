#!/bin/zsh
# The ring picture (ring.mm): a twisted glass ring on pale paper.  ./ring.sh -> out/ring.png
cd "$(dirname "$0")"
./sdk.sh || exit 1   # (Filament's prebuilt SDK, fetched once: sdk.sh)
F=third_party/filament
[ out/ring -nt ring.mm ] && [ out/ring -nt kit.h ] || ./build_one.sh ring || exit 1
for m in paper paper_copy floor_shade thick_glass far_side far_look glasslight; do [ $m.filamat -nt $m.mat ] && [ $m.filamat -nt spectrum.glsl ] && [ $m.filamat -nt studio.glsl ] || $F/bin/matc -a metal -p desktop -o $m.filamat $m.mat || exit 1; done
[ -f out/lightroom/lightroom_ibl.ktx ] || $F/bin/cmgen -q -x out/lightroom --format=ktx --size=512 third_party/filament_src/third_party/environments/lightroom_14b.hdr || exit 1
./out/ring "$@"
