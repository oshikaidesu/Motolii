#!/bin/zsh
# ThorVG's WebGPU engine on the Dawn that Filament's WebGPU backend is built with (dawn_probe/build.sh first).
# ThorVG: its official source at the version we use, fetched once into third_party/thorvg_src, with thorvg_z.patch
# (a 3D layer's third number kept and said: LottieAnimation::pose, for motion.h; nothing drawn changes) and thorvg_bend.patch
# (WgCanvas::bend: all a canvas draws bent in its vertex shaders, on the GPU, nothing made again; drawn.h warp_at), and built
# against Dawn's webgpu.h: its build asks pkg-config for `wgpu_native`, and is given a file of that name that points at Dawn.
#   ./dawn_probe/tvg_on_dawn.sh            builds ThorVG (once) and the probe, runs it
set -e
cd "$(dirname "$0")/.."
F=$PWD/third_party/filament_src
T=$PWD/third_party/thorvg_src
[ -d $T ] || git -c core.hooksPath=/dev/null clone -q --depth 1 --branch v1.1.2 https://github.com/thorvg/thorvg $T
git -C $T diff --quiet && git -C $T apply $PWD/dawn_probe/thorvg_z.patch $PWD/dawn_probe/thorvg_wg_upload.patch $PWD/dawn_probe/thorvg_bend.patch $PWD/dawn_probe/thorvg_length.patch
mkdir -p $T/dawn_pc
cat > $T/dawn_pc/wgpu_native.pc <<PC
Name: wgpu_native
Description: Dawn (from Filament's WebGPU build), under wgpu-native's name
Version: 0.0.0
Cflags: -I$F/third_party/dawn/include -I$F/out/wgpu/third_party/dawn/gen/include
Libs: -L$F/out/wgpu_install/lib/arm64 -lwebgpu_dawn
PC
[ -f $T/build_dawn/build.ninja ] || PKG_CONFIG_PATH=$T/dawn_pc meson setup $T/build_dawn $T -Dengines=cpu,wg -Dloaders=svg,lottie,ttf,png,jpg -Dextra=lottie_exp \
    -Dthreads=true -Dstatic=true -Ddefault_library=static -Dbuildtype=release > /dev/null
ninja -C $T/build_dawn > /dev/null
clang++ -std=c++20 -O2 -I $T/inc -I $F/third_party/dawn/include -I $F/out/wgpu/third_party/dawn/gen/include dawn_probe/tvg_on_dawn.cpp \
    $T/build_dawn/src/libthorvg-1.a -L $F/out/wgpu_install/lib/arm64 -lwebgpu_dawn \
    -framework Cocoa -framework Metal -framework QuartzCore -framework IOSurface -framework IOKit -framework CoreVideo -o out/tvg_on_dawn
./out/tvg_on_dawn
