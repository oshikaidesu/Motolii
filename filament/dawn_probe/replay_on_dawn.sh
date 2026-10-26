#!/bin/zsh
# Plays back a recording of a three.js effect (tsl/dawn_capture.sh made it) on Motolii's Dawn with Dawn's own replay
# library (out/wgpu builds it: src/dawn/replay), no three.js and no JavaScript.
#   ./dawn_probe/replay_on_dawn.sh <capture> <label> <width> <height> <out.rgba>
set -e
cd "$(dirname "$0")/.."
F=$PWD/third_party/filament_src
B=$F/out/wgpu
ninja -C $B third_party/dawn/src/dawn/replay/libreplay.a > /dev/null
clang++ -std=c++20 -O2 -I $F/third_party/dawn/include -I $B/third_party/dawn/gen/include dawn_probe/replay_on_dawn.cpp \
    $B/third_party/dawn/src/dawn/replay/libreplay.a $B/third_party/dawn/src/dawn/common/libdawn_common.a \
    -L $F/out/wgpu_install/lib/arm64 -lwebgpu_dawn -labseil \
    -framework Cocoa -framework Metal -framework QuartzCore -framework IOSurface -framework IOKit -framework CoreVideo -o out/replay_on_dawn
./out/replay_on_dawn "$@"
