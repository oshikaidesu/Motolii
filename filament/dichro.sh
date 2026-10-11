#!/bin/sh
# Glass with colour films in the dark room (DARKROOM=1 in glass_scene.mm: dark table, sun through a blind, the exposure
# opened until the bands of sun burn out, the solids strewn about).
# ./dichro.sh [name] [how many]  ->  out/glass-scene-<name>.png      SEED=n: another throw
cd "$(dirname "$0")"
DARKROOM=1 ONLY=xcut COUNT="${2:-10}" ./out/glass_scene "${1:-dichro}"
