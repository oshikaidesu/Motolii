#!/bin/sh
# Builds glass_scene.mm as a library that draws live into pixel buffers: out/libglass_live.dylib (see glass_live.h).
# Built beside the old one and moved over it, so that an app that has the old one open goes on undisturbed.
set -eu
cd "$(dirname "$0")"
./sdk.sh || exit 1   # (Filament's prebuilt SDK, fetched once: sdk.sh)
SDK=third_party/filament
mkdir -p out
clang++ -std=c++20 -fobjc-arc -O2 -DGLASS_LIB -dynamiclib -fvisibility=hidden -I "$SDK/include" glass_scene.mm glass_live.mm glass_door_mac.mm \
  -L "$SDK/lib/arm64" \
  -lfilament -lbackend -lbluegl -lbluevk -lfilabridge -lfilaflat \
  -lutils -lgeometry -lsmol-v -libl -lktxreader -lstb -luberzlib -lzstd \
  -lshaders -lperfetto -labseil -lmeshoptimizer -lcamutils -limage \
  -lbasis_transcoder -luberarchive \
  -framework Cocoa -framework Metal -framework CoreVideo -framework QuartzCore -framework IOSurface -lz \
  -install_name "$PWD/out/libglass_live.dylib" -o out/libglass_live.new.dylib
mv out/libglass_live.new.dylib out/libglass_live.dylib
