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
MATC=$SRC/out/wgpu/tools/matc/matc
mkdir -p out/wgpu_kit/out
# (the rooms' light, made once by Filament's own cmgen from its environments, as poster.sh and blocks.sh make them with the SDK's)
[ -f out/small_studio/small_studio_ibl.ktx ] && [ -f out/lightroom/lightroom_ibl.ktx ] || (cd $SRC/out/wgpu && ninja cmgen > /dev/null)
[ -f out/small_studio/small_studio_ibl.ktx ] || $SRC/out/wgpu/tools/cmgen/cmgen -q -x out/small_studio --format=ktx --size=512 $SRC/third_party/environments/studio_small_02_2k.hdr
[ -f out/lightroom/lightroom_ibl.ktx ] || $SRC/out/wgpu/tools/cmgen/cmgen -q -x out/lightroom --format=ktx --size=512 $SRC/third_party/environments/lightroom_14b.hdr
for d in lightroom small_studio graffiti_shelter_2k studio_small_02_2k venetian_crossroads_2k parking_garage_2k pillars_2k; do [ -d out/$d ] && ln -sfn ../../$d out/wgpu_kit/out/$d 2> /dev/null; done; true
for f in paper paper_copy floor_shade effected thick_glass far_side far_look glasslight glassbox boxlight bubble poster own_glass flat blade matte floor_lit solid_cloth solid_subsurface shape points solid_image; do
    [ -f $f.mat ] || continue
    [ out/wgpu_kit/$f.filamat -nt $f.mat ] && [ out/wgpu_kit/$f.filamat -nt studio.glsl ] && [ out/wgpu_kit/$f.filamat -nt spectrum.glsl ] && [ out/wgpu_kit/$f.filamat -nt matte.glsl ] && continue
    $MATC -a webgpu -a metal -p desktop -TBLENDING=transparent -TCUT=opaque -TSHADE=lit -o out/wgpu_kit/$f.filamat $f.mat > out/wgpu_kit/$f.log 2>&1 ||
        $MATC -g -a webgpu -a metal -p desktop -TBLENDING=transparent -TCUT=opaque -o out/wgpu_kit/$f.filamat $f.mat > out/wgpu_kit/$f.log 2>&1 ||
        echo "no WebGPU material: $f (out/wgpu_kit/$f.log)"
    [ $f = floor_lit ] && { $MATC -a webgpu -a metal -p desktop -TBLENDING=transparent -TCUT=masked -o out/wgpu_kit/floor_lit_matte.filamat floor_lit.mat > out/wgpu_kit/floor_lit_matte.log 2>&1 ||
        echo "no WebGPU material: floor_lit_matte"; }   # (the same, cut by a matte: what the matte leaves out is not drawn, matte.glsl)
    [ $f = solid_image ] && { $MATC -a webgpu -a metal -p desktop -TSHADE=unlit -DFLAT -o out/wgpu_kit/solid_image_flat.filamat solid_image.mat > out/wgpu_kit/solid_image_flat.log 2>&1 ||
        echo "no WebGPU material: solid_image_flat"; }   # (the same, unlit)
    [ $f = flat ] && for way in add multiply screen opaque; do   # (the ways a sheet lies on what is behind it: Filament's own blending; opaque: the colour behind everything, drawn before the solid things)
        $MATC -a webgpu -a metal -p desktop -TBLENDING=$way -DLAY_$way -o out/wgpu_kit/flat_$way.filamat flat.mat > out/wgpu_kit/flat_$way.log 2>&1 || echo "no WebGPU material: flat_$way"
    done
done
# ThorVG here is the one built against Dawn (dawn_probe/tvg_on_dawn.sh makes it): it can draw on Filament's own device
# (drawn.h), through the door built inside Filament's build (gpu_door.h).
THOR=third_party/thorvg_src
[ -f $THOR/build_dawn/src/libthorvg-1.a ] || ./dawn_probe/tvg_on_dawn.sh > /dev/null
meson configure $THOR/build_dawn -Dengines=cpu,wg -Dextra=lottie_exp > /dev/null   # (kept on: meson set up again by a changed source takes ThorVG's defaults, the CPU engine alone)
ninja -C $THOR/build_dawn > /dev/null
# (bend.cpp: ThorVG's own path operators, built as ThorVG builds them; libtess2: a flat shape cut into triangles, flat_mesh)
[ out/bend.o -nt bend.cpp ] || clang++ -c -std=c++17 -O2 -fno-rtti -fno-exceptions -DTVG_STATIC -I $THOR/build_dawn -I $THOR/inc -I $THOR/src/common -I $THOR/src/renderer -I $THOR/src/renderer/gpu_engine -I $THOR/src/renderer/gpu_engine/wg -I $THOR/src/loaders/lottie -I $SRC/third_party/dawn/include -I $SRC/out/wgpu/third_party/dawn/gen/include bend.cpp -o out/bend.o
./libtess2.sh
./jolt.sh   # (physics.cpp: Jolt, which edit.sh and build_lib.sh link)
./midi_roll.sh   # (midi_roll.cpp: bladeSk's PianoRollSvg.js, read as the MIDI view is drawn)
# (skpath.cpp: Skia's own path operations, its Boolean: skia.sh builds it)
SKIA=third_party/skia_src
[ -f $SKIA/out/path/libpathops.a ] || ./skia.sh
[ out/skpath.o -nt skpath.cpp ] || clang++ -c -std=c++20 -O2 -DNDEBUG -fno-rtti -fno-exceptions -I $SKIA -I $THOR/inc skpath.cpp -o out/skpath.o
# (absent_motion.cpp: AbsentMotion's own motion and text effects, its motion.cpp, scene.cpp, easing.cpp and textanim.cpp as they are, with
#  Skia's headers; what of scene.cpp draws or reads a whole scene is not reached and is stripped: hidden, -dead_strip below)
AM=third_party/AbsentMotion/app/src/main/cpp
for f in motion scene easing textanim; do [ out/am_$f.o -nt $AM/$f.cpp ] || clang++ -c -std=c++20 -O2 -DNDEBUG -fno-rtti -fvisibility=hidden -I $SKIA -I $AM $AM/$f.cpp -o out/am_$f.o; done
[ out/absent_motion.o -nt absent_motion.cpp ] && [ out/absent_motion.o -nt absent_motion.h ] || clang++ -c -std=c++20 -O2 -DNDEBUG -fno-rtti -fvisibility=hidden -I $SKIA -I $AM absent_motion.cpp -o out/absent_motion.o
[ out/absent_drawing.o -nt absent_drawing.cpp ] && [ out/absent_drawing.o -nt $AM/drawing.cpp ] || clang++ -c -std=c++20 -O2 -DNDEBUG -fno-rtti -fvisibility=hidden -I $SKIA -I $AM absent_drawing.cpp -o out/absent_drawing.o   # (its drawing.cpp read whole: absent_drawing.cpp)
[ out/absent_ends.o -nt absent_ends.cpp ] && [ out/absent_ends.o -nt $AM/styles.cpp ] || clang++ -c -std=c++20 -O2 -DNDEBUG -fno-rtti -fvisibility=hidden -I $SKIA -I $AM absent_ends.cpp -o out/absent_ends.o   # (its styles.cpp read whole: absent_ends.cpp)
(cd $SRC/out/wgpu && ninja motolii_door > /dev/null)
clang++ -std=c++20 -fobjc-arc -O2 -I "$SDK/include" -I $THOR/inc -I $THOR/src/loaders/lottie -I $SRC/third_party/dawn/include -I $SRC/out/wgpu/third_party/dawn/gen/include ${KIT_SRC:-$n.mm} ${=KIT_EXTRA} \
    $THOR/build_dawn/src/libthorvg-1.a out/bend.o out/skpath.o out/absent_motion.o out/absent_ends.o out/absent_drawing.o out/am_motion.o out/am_scene.o out/am_easing.o out/am_textanim.o $SKIA/out/path/libpathops.a $SKIA/out/path/libskia.a -I third_party/libtess2/Include third_party/libtess2/build/libtess2.a $SRC/out/wgpu/filament/motolii_probe/libmotolii_door.a -L "$SDK/lib/arm64" \
    -lfilament -lbackend -lwebgpu_dawn -lbluegl -lbluevk -lfilabridge -lfilaflat \
    -lutils -lgeometry -lsmol-v -libl -lktxreader -lstb -luberzlib -lzstd \
    -lshaders -lperfetto -labseil -lmeshoptimizer -lcamutils -limage -lbasis_transcoder -luberarchive -lmatdbg -lfilamat -lcivetweb -lviewer \
    -framework Cocoa -framework SceneKit -framework ModelIO -framework Metal -framework CoreVideo -framework QuartzCore -framework IOSurface -framework IOKit \
    -lz -Wl,-dead_strip -o out/${n}_wgpu
[ -n "$BUILDONLY" ] && exit 0
cd out/wgpu_kit
BACKEND="${BACKEND:-webgpu}" ../${n}_wgpu 2>&1 | grep -v "FrameInfo's circular queue\|libvulkan\|BackendVk"
