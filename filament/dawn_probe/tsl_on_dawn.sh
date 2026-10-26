#!/bin/zsh
# A three.js TSL effect, turned to WGSL by three.js's own builder, drawn on the Dawn that Filament's WebGPU backend is built
# with (dawn_probe/build.sh first). The folder is written by three.js's own code (reports/shaderlab_origins.md).
#   ./dawn_probe/tsl_on_dawn.sh <folder> <width> <height>
set -e
cd "$(dirname "$0")/.."
F=$PWD/third_party/filament_src
clang++ -std=c++20 -O2 -I $F/third_party/dawn/include -I $F/out/wgpu/third_party/dawn/gen/include dawn_probe/tsl_on_dawn.cpp \
    -L $F/out/wgpu_install/lib/arm64 -lwebgpu_dawn \
    -framework Cocoa -framework Metal -framework QuartzCore -framework IOSurface -framework IOKit -framework CoreVideo -o out/tsl_on_dawn
./out/tsl_on_dawn "$@"
