#!/bin/zsh
# The poster picture (poster.mm): a glass band over white letters, by Filament as it is.  ./poster.sh  ->  out/poster.png
cd "$(dirname "$0")"
F=third_party/filament
[ out/poster -nt poster.mm ] && [ out/poster -nt kit.h ] || ./build_one.sh poster || exit 1
for m in poster poster_glass glassbox; do [ $m.filamat -nt $m.mat ] || $F/bin/matc -a metal -p desktop -o $m.filamat $m.mat || exit 1; done
# the room the glass mirrors: the small photo studio that comes with Filament (dark, soft lamps)
[ -f out/small_studio/small_studio_ibl.ktx ] || $F/bin/cmgen -q -x out/small_studio --format=ktx --size=512 third_party/filament_src/third_party/environments/studio_small_02_2k.hdr || exit 1
./out/poster
