#!/bin/zsh
# The poster picture (band.mm, scene "poster"): glass through its REAL far side, eight-wavelength dispersion, by the
# patched matc (third_party/filament_src, filament_glass_reach.patch) and the stock Filament runtime.
# ./poster.sh [extra env as NAME=value ...]   ->  out/band-poster-S.png
cd "$(dirname "$0")"
M=third_party/filament_src/out/wgpu/tools/matc/matc
[ out/band -nt band.mm ] || ./build_one.sh band
[ dirt_glass_f.filamat -nt dirt_glass.mat ] || $M -DMOTOLII_PATCHED -DMOTOLII_FAR_SIDE -DMOTOLII_STEPS=8 -DMOTOLII_HALVE=4 -a metal -p desktop -o dirt_glass_f.filamat dirt_glass.mat
for m in poster band_gbuf; do [ $m.filamat -nt $m.mat ] || third_party/filament/bin/matc -a metal -p desktop -o $m.filamat $m.mat; done
env MODEL=S PATCHED=2 THICK=1.26 INNER=0 DISP=5 ROOM=out/dark_room ROOMGAIN=0.5 ROOMTURN=0 BLOOM=0.12 BLOOMRES=360 BLOOMLEVELS=6 GPART=2 MESHPER=48 MESHSIDES=64 TONE=filmic SATURATION=0.75 "$@" ./out/band poster
