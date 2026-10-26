#!/bin/zsh
# A web effect's own renderer inside this process on Filament's Dawn device (js_effects.cpp): Motolii's QuickJS with
# JsRuntimeHost's Node-API (tsl/jsrh_napi/build.sh made its sources), Dawn's dawn.node with dawn_lend.patch (out/wgpu).
# (libraries: the list dawn_probe/kit_wgpu.sh links)
#   ./dawn_probe/js_effects.sh <preset.json> <w> <h> <frames> <out.rgba>
set -e
cd "$(dirname "$0")/.."
SRC=third_party/filament_src
SDK=$SRC/out/wgpu_install
J=tsl/out/jsrh_napi/src/Node-API
Q=/opt/homebrew/opt/quickjs-ng
(cd $SRC/out/wgpu && ninja motolii_door third_party/dawn/dawn.node > /dev/null)
[ -d $J ] || ./tsl/jsrh_napi/build.sh > /dev/null
clang++ -std=c++20 -O2 -w -DNAPI_VERSION=8 -I "$SDK/include" -I $SRC/third_party/dawn/include -I $SRC/out/wgpu/third_party/dawn/gen/include \
    -I $Q/include -I $J/Include/Shared -I $J/Include/Engine/QuickJS -I $J/Source -I dawn_probe dawn_probe/js_effects.cpp \
    $J/Source/js_native_api_quickjs.cc $J/Source/js_native_api_shared.cc $J/Source/env_quickjs.cc -L $Q/lib -lqjs -Wl,-rpath,$Q/lib \
    $SRC/out/wgpu/filament/motolii_probe/libmotolii_door.a -Wl,-export_dynamic -L "$SDK/lib/arm64" \
    -lfilament -lbackend -lwebgpu_dawn -lbluegl -lbluevk -lfilabridge -lfilaflat \
    -lutils -lgeometry -lsmol-v -libl -lktxreader -lstb -luberzlib -lzstd \
    -lshaders -lperfetto -labseil -lmeshoptimizer -lcamutils -limage -lbasis_transcoder -luberarchive -lmatdbg -lfilamat -lcivetweb \
    -framework Cocoa -framework Metal -framework CoreVideo -framework QuartzCore -framework IOSurface -framework IOKit -lz -o out/js_effects
DAWN_NODE=$PWD/$SRC/out/wgpu/third_party/dawn/dawn.node EFFECT_JS=$PWD/tsl/out/jsrh_napi/shaders.mjs ./out/js_effects "$@"
