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
for d in lightroom small_studio graffiti_shelter_2k; do [ -d out/$d ] && ln -sfn ../../$d out/wgpu_kit/out/$d 2> /dev/null; done; true
for f in paper paper_copy floor_shade thick_glass far_side far_look glasslight glassbox boxlight bubble poster own_glass flat blade matte floor_lit; do
    [ -f $f.mat ] || continue
    [ out/wgpu_kit/$f.filamat -nt $f.mat ] && [ out/wgpu_kit/$f.filamat -nt studio.glsl ] && [ out/wgpu_kit/$f.filamat -nt spectrum.glsl ] && [ out/wgpu_kit/$f.filamat -nt matte.glsl ] && continue
    $MATC -a webgpu -a metal -p desktop -TBLENDING=transparent -o out/wgpu_kit/$f.filamat $f.mat > out/wgpu_kit/$f.log 2>&1 ||
        $MATC -g -a webgpu -a metal -p desktop -TBLENDING=transparent -o out/wgpu_kit/$f.filamat $f.mat > out/wgpu_kit/$f.log 2>&1 ||
        echo "no WebGPU material: $f (out/wgpu_kit/$f.log)"
    [ $f = flat ] && for way in add multiply screen; do   # (the ways a sheet lies on what is behind it: Filament's own blending)
        $MATC -a webgpu -a metal -p desktop -TBLENDING=$way -DLAY_$way -o out/wgpu_kit/flat_$way.filamat flat.mat > out/wgpu_kit/flat_$way.log 2>&1 || echo "no WebGPU material: flat_$way"
    done
done
# ThorVG here is the one built against Dawn (dawn_probe/tvg_on_dawn.sh makes it): it can draw on Filament's own device
# (drawn.h), through the door built inside Filament's build (gpu_door.h).
THOR=third_party/thorvg_src
[ -f $THOR/build_dawn/src/libthorvg-1.a ] || ./dawn_probe/tvg_on_dawn.sh > /dev/null
ninja -C $THOR/build_dawn > /dev/null
(cd $SRC/out/wgpu && ninja motolii_door > /dev/null)
clang++ -std=c++20 -fobjc-arc -O2 -I "$SDK/include" -I $THOR/inc -I $THOR/src/loaders/lottie -I $SRC/third_party/dawn/include -I $SRC/out/wgpu/third_party/dawn/gen/include ${KIT_SRC:-$n.mm} ${=KIT_EXTRA} \
    $THOR/build_dawn/src/libthorvg-1.a $SRC/out/wgpu/filament/motolii_probe/libmotolii_door.a -L "$SDK/lib/arm64" \
    -lfilament -lbackend -lwebgpu_dawn -lbluegl -lbluevk -lfilabridge -lfilaflat \
    -lutils -lgeometry -lsmol-v -libl -lktxreader -lstb -luberzlib -lzstd \
    -lshaders -lperfetto -labseil -lmeshoptimizer -lcamutils -limage -lbasis_transcoder -luberarchive \
    -framework Cocoa -framework SceneKit -framework ModelIO -framework Metal -framework CoreVideo -framework QuartzCore -framework IOSurface -framework IOKit \
    -lz -o out/${n}_wgpu
[ -n "$BUILDONLY" ] && exit 0
cd out/wgpu_kit
BACKEND="${BACKEND:-webgpu}" ../${n}_wgpu 2>&1 | grep -v "FrameInfo's circular queue\|libvulkan\|BackendVk"
