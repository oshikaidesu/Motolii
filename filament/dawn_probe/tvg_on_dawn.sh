#!/bin/zsh
# ThorVG's WebGPU engine on the Dawn that Filament's WebGPU backend is built with (dawn_probe/build.sh first).
# ThorVG: its official source at the commit of its main we use, fetched once into third_party/thorvg_src, with thorvg_z.patch
# (a 3D layer's third number kept and said: LottieAnimation::pose, for motion.h; nothing drawn changes) and
# thorvg_over.patch (a texture target of type 2 is drawn over, not replaced: the desk's marks on its picture, live.h) and
# thorvg_matte_clip.patch (an alpha matte of one opaque shape is a clipper, as ThorVG makes a single mask one) and
# thorvg_texture.patch (Picture::load of a texture already on the device, read as it is: a thing's effects in any order,
# ThorVG's taking the picture the step before made, stage_steps) and thorvg_ttc.patch (a collection's first face) and
# thorvg_otf_guard.patch (a real number in a CFF dictionary is stepped over, and a letter that points outside the file stops, not the program), and built
# against Dawn's webgpu.h: its build asks pkg-config for `wgpu_native`, and is given a file of that name that points at Dawn.
#   ./dawn_probe/tvg_on_dawn.sh            builds ThorVG (once) and the probe, runs it
set -e
cd "$(dirname "$0")/.."
F=$PWD/third_party/filament_src
T=$PWD/third_party/thorvg_src
[ -d $T ] || { git init -q $T && git -C $T -c core.hooksPath=/dev/null fetch -q --depth 1 https://github.com/thorvg/thorvg 920324543b0377d14f34ebcd561f27590ddb55de && git -C $T checkout -q FETCH_HEAD; }
git -C $T diff --quiet && git -C $T apply $PWD/dawn_probe/thorvg_z.patch $PWD/dawn_probe/thorvg_wg_upload.patch $PWD/dawn_probe/thorvg_length.patch $PWD/dawn_probe/thorvg_over.patch $PWD/dawn_probe/thorvg_matte_clip.patch $PWD/dawn_probe/thorvg_texture.patch $PWD/dawn_probe/thorvg_ttc.patch $PWD/dawn_probe/thorvg_otf_guard.patch
mkdir -p $T/dawn_pc
cat > $T/dawn_pc/wgpu_native.pc <<PC
Name: wgpu_native
Description: Dawn (from Filament's WebGPU build), under wgpu-native's name
Version: 0.0.0
Cflags: -I$F/third_party/dawn/include -I$F/out/wgpu/third_party/dawn/gen/include
Libs: -L$F/out/wgpu_install/lib/arm64 -lwebgpu_dawn
PC
[ -f $T/build_dawn/build.ninja ] || PKG_CONFIG_PATH=$T/dawn_pc meson setup $T/build_dawn $T -Dengines=cpu,wg -Dloaders=svg,lottie,ttf,otf,png,jpg -Dextra=lottie_exp \
    -Dthreads=true -Dstatic=true -Ddefault_library=static -Dbuildtype=release > /dev/null
PKG_CONFIG_PATH=$T/dawn_pc meson configure $T/build_dawn -Dengines=cpu,wg -Dloaders=svg,lottie,ttf,otf,png,jpg -Dextra=lottie_exp > /dev/null   # (a source whose options changed makes meson set itself up again with ThorVG's own defaults: the CPU engine alone)
ninja -C $T/build_dawn > /dev/null
clang++ -std=c++20 -O2 -I $T/inc -I $F/third_party/dawn/include -I $F/out/wgpu/third_party/dawn/gen/include dawn_probe/tvg_on_dawn.cpp \
    $T/build_dawn/src/libthorvg-1.a -L $F/out/wgpu_install/lib/arm64 -lwebgpu_dawn \
    -framework Cocoa -framework Metal -framework QuartzCore -framework IOSurface -framework IOKit -framework CoreVideo -o out/tvg_on_dawn
./out/tvg_on_dawn
