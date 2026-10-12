#!/bin/zsh
# Glass bands along 3D paths, dropped in (band.mm). ./band.sh [ref|loops|dark]  ->  out/band-<scene>.png
# The glass's look is fixed inside band.mm (GlassLook); a scene only gives paths, a ground and a camera.
cd "$(dirname "$0")"
[ out/band -nt band.mm ] || ./build_one.sh band
for m in band_glass backdrop_grad plate floor_lit poster dirt_glass; do [ $m.filamat -nt $m.mat ] || third_party/filament/bin/matc -a metal -p desktop -o $m.filamat $m.mat; done
./out/band "${1:-ref}"
