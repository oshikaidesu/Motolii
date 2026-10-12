#!/bin/zsh
# The poster picture (band.mm, scene "poster"): glass through its REAL far side, eight-wavelength dispersion, by the
# patched matc (third_party/filament_src, filament_glass_reach.patch) and the stock Filament runtime.
# ./poster.sh [extra env as NAME=value ...]   ->  out/band-poster-S.png
cd "$(dirname "$0")"
M=third_party/filament_src/out/wgpu/tools/matc/matc
[ out/band -nt band.mm ] || ./build_one.sh band
[ dirt_glass_f.filamat -nt dirt_glass.mat ] || $M -DMOTOLII_PATCHED -DMOTOLII_FAR_SIDE -DMOTOLII_STEPS=8 -DMOTOLII_HALVE=4 -a metal -p desktop -o dirt_glass_f.filamat dirt_glass.mat
for m in poster band_gbuf; do [ $m.filamat -nt $m.mat ] || third_party/filament/bin/matc -a metal -p desktop -o $m.filamat $m.mat; done
# the room the glass mirrors: the small photo studio that comes with Filament (dark, soft lamps)
[ -f out/small_studio/_ibl.ktx ] || { third_party/filament/bin/cmgen -q -x out/small_studio --format=ktx --size=512 third_party/filament_src/third_party/environments/studio_small_02_2k.hdr && for f in ibl skybox; do mv out/small_studio/small_studio_$f.ktx out/small_studio/_$f.ktx; done; }
env MODEL=S PATCHED=2 THICK=1.26 INNER=0 DISP=2 ROOM=out/small_studio ROOMGAIN=1.0 ROOMTURN=0 BLOOM=0 PDIST=6.6 WAV=0.09 TWISTWAVE=0.2 PWOB=0.15 GPART=2 MESHPER=48 MESHSIDES=64 TONE=filmic SATURATION=0.75 "$@" ./out/band poster
