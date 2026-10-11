#!/bin/sh
# Compiles the two capture materials that follow each solid's pose (hits.mat / tri.mat come from ../caustic_tri).
set -eu
cd "$(dirname "$0")"
for m in cap_front cap_back grid floor_caustic glass_back; do
  ../third_party/filament/bin/matc -a metal -p desktop -o "$m.filamat" "$m.mat"
done
