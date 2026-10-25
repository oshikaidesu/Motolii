#!/bin/zsh
# dawn_probe: Filament 1.77.2 built from source WITH its WebGPU (Dawn) backend, plus filament_dawn_compute.patch
# (compute; a shorter wait; the device lent out and Texture::Builder::import filled in, for drawn.h).
# The prebuilt SDK in ../third_party/filament has no WebGPU backend; this builds a second Filament beside it and
# leaves that one alone. First build: about 15 minutes on an M4 (Dawn is most of it).
#   ./build.sh          build, then run once with pictures into out/shots
set -e
cd "$(dirname "$0")"
HERE="$PWD"
SRC="$HERE/../third_party/filament_src"
if [ ! -d "$SRC" ]; then
    git clone --depth 1 --branch v1.77.2 https://github.com/google/filament.git "$SRC"
    (cd "$SRC" && git apply "$HERE/filament_dawn_compute.patch" && git apply "$HERE/glslang_hlsl.patch")   # (glslang_hlsl: its HLSL front end built in, AviUtl2's shaders: ../aviutl.cpp)
fi
mkdir -p "$SRC/out/wgpu" out/shots
cd "$SRC/out/wgpu"
[ -f build.ninja ] || cmake -G Ninja -DCMAKE_BUILD_TYPE=Release -DFILAMENT_SUPPORTS_WEBGPU=ON -DFILAMENT_SKIP_SAMPLES=ON \
    -DFILAMENT_BUILD_TESTING=OFF -DCMAKE_INSTALL_PREFIX=../wgpu_install -DMOTOLII_PROBE_DIR="$HERE" ../..
ninja matc dawn_probe
cd "$HERE"
"$SRC/out/wgpu/tools/matc/matc" -a webgpu -p desktop -o out/dawn_probe.filamat dawn_probe.mat
"$SRC/out/wgpu/filament/motolii_probe/dawn_probe" out/dawn_probe.filamat out/shots 2>&1 | grep -v "FrameInfo's circular queue\|libvulkan\|BackendVk"
