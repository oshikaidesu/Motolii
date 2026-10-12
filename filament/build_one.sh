#!/bin/sh
# Builds one .mm into out/<name> without running it (link.sh also runs it).
set -eu
cd "$(dirname "$0")"
SDK=third_party/filament
TVG=../thorvg/third_party
mkdir -p out
clang++ -std=c++20 -fobjc-arc -O2 -I "$SDK/include" -I "$TVG/include/thorvg-1" "$1.mm" \
  -L "$SDK/lib/arm64" -L "$TVG/lib" -Wl,-rpath,"$PWD/$TVG/lib" \
  -lthorvg-1 -lfilament -lbackend -lbluegl -lbluevk -lfilabridge -lfilaflat \
  -lutils -lgeometry -lsmol-v -libl -lktxreader -lstb -luberzlib -lzstd \
  -lshaders -lperfetto -labseil -lmeshoptimizer -lcamutils -limage \
  -lbasis_transcoder -luberarchive \
  -framework Cocoa -framework SceneKit -framework ModelIO -framework Metal -framework CoreVideo -framework QuartzCore -lz -o "out/$1"
