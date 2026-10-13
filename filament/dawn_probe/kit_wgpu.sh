#!/bin/zsh
# One of the kit's pictures (ring, poster, blocks, shapes) on Filament's WebGPU (Dawn) backend, beside the usual Metal
# build (which is not touched). Needs dawn_probe/build.sh first (the WebGPU-enabled Filament and its matc).
#   ./kit_wgpu.sh ring                 builds ../out/<name>_wgpu and runs it on WebGPU:  ../out/wgpu_kit/out/<name>.png
#   BACKEND=metal ./kit_wgpu.sh ring   the same program on Metal, to compare
set -e
cd "$(dirname "$0")/.."
n=${1:-ring}
SRC=third_party/filament_src
SDK=$SRC/out/wgpu_install
TVG=../thorvg/third_party
MATC=$SRC/out/wgpu/tools/matc/matc
mkdir -p out/wgpu_kit/out
for d in lightroom small_studio; do [ -d out/$d ] && ln -sfn ../../$d out/wgpu_kit/out/$d; done
for f in paper paper_copy floor_shade thick_glass far_side far_look glasslight glassbox boxlight bubble poster poster_glass; do
    [ -f $f.mat ] || continue
    [ out/wgpu_kit/$f.filamat -nt $f.mat ] && [ out/wgpu_kit/$f.filamat -nt studio.glsl ] && [ out/wgpu_kit/$f.filamat -nt spectrum.glsl ] && continue
    $MATC -a webgpu -a metal -p desktop -o out/wgpu_kit/$f.filamat $f.mat > out/wgpu_kit/$f.log 2>&1 ||
        $MATC -g -a webgpu -a metal -p desktop -o out/wgpu_kit/$f.filamat $f.mat > out/wgpu_kit/$f.log 2>&1 ||
        echo "no WebGPU material: $f (out/wgpu_kit/$f.log)"
done
clang++ -std=c++20 -fobjc-arc -O2 -I "$SDK/include" -I "$TVG/include/thorvg-1" ${KIT_SRC:-$n.mm} \
    -L "$SDK/lib/arm64" -L "$TVG/lib" -Wl,-rpath,"$PWD/$TVG/lib" \
    -lthorvg-1 -lfilament -lbackend -lwebgpu_dawn -lbluegl -lbluevk -lfilabridge -lfilaflat \
    -lutils -lgeometry -lsmol-v -libl -lktxreader -lstb -luberzlib -lzstd \
    -lshaders -lperfetto -labseil -lmeshoptimizer -lcamutils -limage -lbasis_transcoder -luberarchive \
    -framework Cocoa -framework SceneKit -framework ModelIO -framework Metal -framework CoreVideo -framework QuartzCore -framework IOSurface -framework IOKit \
    -lz -o out/${n}_wgpu
[ -n "$BUILDONLY" ] && exit 0
cd out/wgpu_kit
BACKEND="${BACKEND:-webgpu}" ../${n}_wgpu 2>&1 | grep -v "FrameInfo's circular queue\|libvulkan\|BackendVk"
