#!/bin/sh
# The dark-room picture with colour films (see "Colour films" in glass_scene.mm): one solid with its corners cut and
# four films inside, a band of sun through a slit, the exposure opened until the band burns out.
# ./dichro.sh [name]  ->  out/glass-scene-<name>.png
cd "$(dirname "$0")"
# SKY, GLINT, GLINTDEG: what the glass itself has to show in a dark room, a dim sky and the bright window the sun comes by
# (without them it only shows the dark, and whichever colour happens to see the band of sun).
ONLY=xcut COUNT=1 ROOM=0.012 FLOORGREY=0.3 FLOOR_REFL=0.1 PIECES=16 APERTURE=1.6 BLOOM=0.35 STRIP="0.28 35" \
SKY=30000 GLINTDEG=25 GLINT=30000 \
FILMBANDS="380-490,500-590,600-720,380-490" XFILMS="1,0,1,0.2 1,0,-1,0.2 1,1,0,-0.2 0,1,1,0.25" \
${RUN:-} ./out/glass_scene "${1:-dichro}"
