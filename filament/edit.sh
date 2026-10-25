#!/bin/zsh
# A document edited by calls, a line each on its input.   ./edit.sh out/first.motolii < calls.txt
# Two parts: the editing inside as a library with C calls (motolii.h, motolii.mm: films by GStreamer, the document's
# file, its undo and the working out of a stretch between keys by SQLite, 3D files by Filament's gltfio), and a shell that calls it
# (edit.cpp), as a shell on a screen would.
cd "$(dirname "$0")"
doc=${1:a}; shift
# (built again only if something it is made of has changed since)
if [ ! -x out/edit_wgpu ] || [ -n "$(find . dawn_probe check -maxdepth 1 \( -name '*.c' -o -name '*.h' -o -name '*.mm' -o -name '*.cpp' -o -name '*.mat' -o -name '*.glsl' -o -name '*.sh' \) -newer out/edit_wgpu | head -1)" ]; then
    F=third_party/filament_src   # (an ISF effect made WGSL by Filament's own glslang and Dawn's Tint: isf.cpp, its headers heavy, built apart)
    [ out/isf.o -nt isf.cpp ] && [ out/isf.o -nt isf.h ] || clang++ -c -std=c++20 -O2 -I$F/third_party/libassimp/contrib/rapidjson/include -I/opt/homebrew/opt/sqlite/include -I$F/third_party/glslang -I$F/libs/filamat/src -I$F/third_party/dawn -I$F/third_party/dawn/include -I$F/out/wgpu/third_party/dawn/gen -DTINT_BUILD_SPV_READER=1 -DTINT_BUILD_WGSL_WRITER=1 -DTINT_BUILD_WGSL_READER=1 isf.cpp -o out/isf.o || exit 1
    KIT_EXTRA="-dynamiclib -install_name @rpath/libmotolii.dylib $(pkg-config --cflags --libs gstreamer-1.0 gstreamer-app-1.0 gstreamer-video-1.0 gstreamer-pbutils-1.0) -lgltfio -lgltfio_core -ldracodec -DSQLITE_ENABLE_SESSION -DSQLITE_ENABLE_PREUPDATE_HOOK -I/opt/homebrew/opt/sqlite/include /opt/homebrew/opt/sqlite/lib/libsqlite3.dylib $PWD/physics.cpp $PWD/script.cpp -I$PWD/third_party/thermion_frame/include $PWD/third_party/thermion_frame/FrameScheduler.cpp -framework CoreVideo $PWD/splatfx.cpp $PWD/three_isf.cpp $PWD/solid_three.cpp $PWD/midi_roll.cpp -lmatp $PWD/out/isf.o -I/opt/homebrew/opt/quickjs-ng/include -L/opt/homebrew/opt/quickjs-ng/lib -lqjs -Wl,-rpath,/opt/homebrew/opt/quickjs-ng/lib -I$PWD/third_party/physics/JoltPhysics -DJPH_CROSS_PLATFORM_DETERMINISTIC -DJPH_DEBUG_RENDERER -DJPH_OBJECT_LAYER_BITS=16 -DJPH_OBJECT_STREAM -DJPH_PROFILE_ENABLED -ffp-contract=off $PWD/third_party/physics/JoltPhysics/jbuild/libJolt.a" BUILDONLY=1 ./dawn_probe/kit_wgpu.sh motolii || exit 1
    mv out/motolii_wgpu out/libmotolii.dylib
    clang++ -std=c++20 -O2 edit.cpp -Lout -lmotolii -Wl,-rpath,@executable_path -o out/edit_wgpu || exit 1
    cc check/api.c -Lout -lmotolii -Wl,-rpath,@executable_path -o out/api_check || exit 1   # (the same calls from plain C)
    clang++ -std=c++20 -fobjc-arc check/live.mm -Lout -lmotolii -Wl,-rpath,@executable_path -framework Foundation -framework CoreVideo -framework CoreGraphics -framework ImageIO -framework UniformTypeIdentifiers -o out/live_check || exit 1   # (the live picture, without a shell)
fi
cd out/wgpu_kit && BACKEND="${BACKEND:-webgpu}" ../edit_wgpu $doc "$@" 2>&1 | grep -v "FrameInfo's circular queue\|libvulkan\|BackendVk\|FEngine\|WebGPU adapter\|Backend feature"
