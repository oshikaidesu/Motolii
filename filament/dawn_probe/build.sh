#!/bin/zsh
# dawn_probe: Filament 1.77.2 built from source WITH its WebGPU (Dawn) backend, plus filament_dawn_compute.patch
# (compute; a shorter wait; the device lent out and Texture::Builder::import filled in, for drawn.h).
# The prebuilt SDK in ../third_party/filament has no WebGPU backend; this builds a second Filament beside it and
# leaves that one alone. First build: about 5 minutes on an M4 (4 min 55 s from a fresh clone, 10/27; Dawn is most of it).
#   ./build.sh          build, then run once with pictures into out/shots
set -e
cd "$(dirname "$0")"
HERE="$PWD"
SRC="$HERE/../third_party/filament_src"
if [ ! -d "$SRC" ]; then
    git clone --depth 1 --branch v1.77.2 https://github.com/google/filament.git "$SRC"
    (cd "$SRC" && git apply "$HERE/filament_dawn_compute.patch" && git apply "$HERE/glslang_hlsl.patch")   # (glslang_hlsl: its HLSL front end built in, AviUtl2's shaders: ../aviutl.cpp)
fi
# (Filament's WebGPU viewport turned to WebGPU's top-left origin, as its own scissor() does: upstream's TODO in
#  beginRenderPass, still there on main 10/27. Put on a tree made before it too, once)
(cd "$SRC" && { git apply --reverse --check "$HERE/filament_webgpu_viewport.patch" 2>/dev/null || git apply "$HERE/filament_webgpu_viewport.patch"; })
# (a depth-only pass keeps a material's fragment stage: without it a shadow is its whole quad, whatever it cuts away)
(cd "$SRC" && { git apply --reverse --check "$HERE/filament_webgpu_depth_fragment.patch" 2>/dev/null || git apply "$HERE/filament_webgpu_depth_fragment.patch"; })
(cd "$SRC" && { git apply --reverse --check "$HERE/tint_msl_let.patch" 2>/dev/null || git apply "$HERE/tint_msl_let.patch"; })   # (a first open of a 立体 took 76 s in Apple's compiler, on one shader, for a `const`)
mkdir -p "$SRC/out/wgpu" out/shots
cd "$SRC/out/wgpu"
[ -f build.ninja ] || cmake -G Ninja -DCMAKE_BUILD_TYPE=Release -DFILAMENT_SUPPORTS_WEBGPU=ON -DFILAMENT_SKIP_SAMPLES=ON \
    -DFILAMENT_BUILD_TESTING=OFF -DCMAKE_INSTALL_PREFIX=../wgpu_install -DMOTOLII_PROBE_DIR="$HERE" ../..
ninja matc dawn_probe
ninja install > /dev/null   # (the headers and libraries the kit builds against: ../wgpu_install, CMAKE_INSTALL_PREFIX above)
cd "$HERE"
"$SRC/out/wgpu/tools/matc/matc" -a webgpu -p desktop -o out/dawn_probe.filamat dawn_probe.mat
"$SRC/out/wgpu/filament/motolii_probe/dawn_probe" out/dawn_probe.filamat out/shots 2>&1 | grep -v "FrameInfo's circular queue\|libvulkan\|BackendVk"
