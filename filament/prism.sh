#!/bin/sh
set -eu
cd "$(dirname "$0")"
./sdk.sh || exit 1   # (Filament's prebuilt SDK, fetched once: sdk.sh)
SDK=third_party/filament
clang++ -std=c++20 -fobjc-arc -O2 \
  -I "$SDK/include" \
  prism.mm \
  -L "$SDK/lib/arm64" \
  -lfilament -lbackend -lbluegl -lbluevk -lfilabridge -lfilaflat \
  -lutils -lgeometry -lsmol-v -libl -lktxreader -lstb -luberzlib -lzstd \
  -lshaders -lperfetto -labseil -lmeshoptimizer -lcamutils -limage \
  -lbasis_transcoder -luberarchive \
  -framework Cocoa -framework Metal -framework CoreVideo -framework QuartzCore \
  -lz \
  -o prism
mkdir -p out
./prism
