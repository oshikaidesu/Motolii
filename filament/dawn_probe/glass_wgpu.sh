#!/bin/zsh
# The existing glass scene on Filament's WebGPU (Dawn) backend, beside the usual Metal build (which is not touched).
# Needs ./build.sh first (it builds the WebGPU-enabled Filament and its matc).
#   ./glass_wgpu.sh [name]            one picture on WebGPU:  ../out/wgpu_mat/out/glass-scene-<name>.png
#   BACKEND=metal ./glass_wgpu.sh x   the same build on Metal, to compare
# Materials: all of ../*.mat compiled for webgpu + metal into ../out/wgpu_mat. Four of them (flat_walk, flat_walk1,
# extrude_find, extrude_walk) only pass Dawn's shader converter (Tint) unoptimised (-g): optimised, it stops with an
# internal error. glass_vert does not pass at all (an experiment's material; the usual pictures do not use it).
set -e
cd "$(dirname "$0")/.."
SRC=third_party/filament_src
SDK=$SRC/out/wgpu_install
TVG=../thorvg/third_party
MATC=$SRC/out/wgpu/tools/matc/matc
(cd $SRC/out/wgpu && ninja install > /dev/null)
mkdir -p out/wgpu_mat/out
ln -sfn ../../studio out/wgpu_mat/out/studio
for f in *.mat; do
    n=${f%.mat}
    [ out/wgpu_mat/$n.filamat -nt $f ] && continue
    $MATC -a webgpu -a metal -p desktop -o out/wgpu_mat/$n.filamat $f > out/wgpu_mat/$n.log 2>&1 ||
        $MATC -g -a webgpu -a metal -p desktop -o out/wgpu_mat/$n.filamat $f > out/wgpu_mat/$n.log 2>&1 ||
        echo "no WebGPU material: $n"
done
clang++ -std=c++20 -fobjc-arc -O2 -I "$SDK/include" -I "$TVG/include/thorvg-1" glass_scene.mm \
    -L "$SDK/lib/arm64" -L "$TVG/lib" -Wl,-rpath,"$PWD/$TVG/lib" \
    -lthorvg-1 -lfilament -lbackend -lwebgpu_dawn -lbluegl -lbluevk -lfilabridge -lfilaflat \
    -lutils -lgeometry -lsmol-v -libl -lktxreader -lstb -luberzlib -lzstd \
    -lshaders -lperfetto -labseil -lmeshoptimizer -lcamutils -limage -lbasis_transcoder -luberarchive \
    -framework Cocoa -framework Metal -framework CoreVideo -framework QuartzCore -framework IOSurface -framework IOKit \
    -lz -o out/glass_scene_wgpu
cd out/wgpu_mat
BACKEND="${BACKEND:-webgpu}" ../glass_scene_wgpu "${1:-dawn}" 2>&1 | grep -v "FrameInfo's circular queue\|libvulkan\|BackendVk"
