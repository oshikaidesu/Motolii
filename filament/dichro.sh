#!/bin/sh
# The dark-room picture with colour films (see "Colour films" in glass_scene.mm): one solid with its corners cut and
# four films inside, a band of sun through a slit, the exposure opened until the band burns out.
# ./dichro.sh [name]  ->  out/glass-scene-<name>.png
cd "$(dirname "$0")"
ONLY=xcut COUNT=1 ROOM=0.012 FLOORGREY=0.3 FLOOR_REFL=0.1 PIECES=16 APERTURE=1.6 BLOOM=0.35 STRIP="0.28 35" \
FILMBANDS="380-490,500-590,600-720,380-490" XFILMS="1,0,1,0.2 1,0,-1,0.2 1,1,0,-0.2 0,1,1,0.25" \
./out/glass_scene "${1:-dichro}"
