#!/bin/zsh
# (libraries: the list dawn_probe/kit_wgpu.sh links)
# A film's picture read by a web effect's own texture_external WGSL on Filament's Dawn device (video_external.cpp).
#   ./dawn_probe/video_external.sh <film> <effect.wgsl> <out folder>
set -e
cd "$(dirname "$0")/.."
SRC=third_party/filament_src
SDK=$SRC/out/wgpu_install
(cd $SRC/out/wgpu && ninja motolii_door third_party/dawn/src/dawn/common/libdawn_common.a > /dev/null)
clang++ -std=c++20 -O2 -fno-rtti -I "$SDK/include" -I $SRC/third_party/dawn/include -I $SRC/third_party/dawn -I $SRC/third_party/dawn/src \
    -I $SRC/out/wgpu/third_party/dawn/gen/include -I $SRC/out/wgpu/third_party/dawn/gen/src -I $SRC/third_party/dawn/third_party/abseil-cpp -I $SRC/third_party/abseil \
    -I dawn_probe dawn_probe/video_external.cpp $(pkg-config --cflags --libs gstreamer-1.0 gstreamer-app-1.0 gstreamer-video-1.0) \
    $SRC/out/wgpu/filament/motolii_probe/libmotolii_door.a $SRC/out/wgpu/third_party/dawn/src/dawn/common/libdawn_common.a -L "$SDK/lib/arm64" \
    -lfilament -lbackend -lwebgpu_dawn -lbluegl -lbluevk -lfilabridge -lfilaflat \
    -lutils -lgeometry -lsmol-v -libl -lktxreader -lstb -luberzlib -lzstd \
    -lshaders -lperfetto -labseil -lmeshoptimizer -lcamutils -limage -lbasis_transcoder -luberarchive -lmatdbg -lfilamat -lcivetweb \
    -framework Cocoa -framework Metal -framework CoreVideo -framework QuartzCore -framework IOSurface -framework IOKit -lz -o out/video_external
./out/video_external "$@"
