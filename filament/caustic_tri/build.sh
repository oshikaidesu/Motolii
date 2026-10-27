#!/bin/sh
# Compiles the four caustic materials for the Metal backend. Add `-a vulkan` / `-a opengl` for the others.
set -eu
cd "$(dirname "$0")"
../sdk.sh || exit 1   # (Filament's prebuilt SDK, fetched once: sdk.sh)
for m in cap_front cap_back hits tri; do
  ../third_party/filament/bin/matc -a metal -p desktop -o "$m.filamat" "$m.mat"
done
