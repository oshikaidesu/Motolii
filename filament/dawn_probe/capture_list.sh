#!/bin/zsh
# Lists a Dawn capture one line per root command (capture_list.cpp), with Dawn's own replay library from out/wgpu.
#   ./dawn_probe/capture_list.sh <capture>
set -e
cd "$(dirname "$0")/.."
F=$PWD/third_party/filament_src
B=$F/out/wgpu
D=$F/third_party/dawn
clang++ -std=c++20 -O2 -fno-rtti -I $D -I $D/include -I $D/src -I $B/third_party/dawn/gen/include -I $B/third_party/dawn/gen/src \
    -I $F/third_party/abseil -I $D/third_party/abseil-cpp dawn_probe/capture_list.cpp \
    $B/third_party/dawn/src/dawn/replay/libreplay.a $B/third_party/dawn/src/dawn/common/libdawn_common.a \
    -L $F/out/wgpu_install/lib/arm64 -lwebgpu_dawn -labseil \
    -framework Cocoa -framework Metal -framework QuartzCore -framework IOSurface -framework IOKit -framework CoreVideo -o out/capture_list
./out/capture_list "$@"
