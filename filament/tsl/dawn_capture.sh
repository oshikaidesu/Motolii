#!/bin/zsh
# A second dawn.node that can record: Dawn's own WebGPU-on-WebGPU backend (src/dawn/native/webgpu, which keeps a
# capture of every call: CaptureContext) switched on, and dawn_capture.patch (the capture=<path> flag, the way
# Dawn's DawnTest.cpp Recorder starts one). Built beside Motolii's out/wgpu, which stays as it is; same Filament
# and Dawn sources, same options as dawn_probe/build.sh + dawn_node.sh. Also builds Dawn's replay library.
#   ./tsl/dawn_capture.sh   -> filament_src/out/wgpu_capture/third_party/dawn/dawn.node
set -e
cd "$(dirname "$0")/.."
F=$PWD/third_party/filament_src
./tsl/dawn_node.sh > /dev/null   # the three inputs dawn.node needs (and Go)
(cd $F && git apply --check $PWD/../../tsl/dawn_capture.patch 2> /dev/null && git apply $PWD/../../tsl/dawn_capture.patch || true)
mkdir -p $F/out/wgpu_capture
[ -f $F/out/wgpu_capture/build.ninja ] || cmake -S $F -B $F/out/wgpu_capture -G Ninja -DCMAKE_BUILD_TYPE=Release \
    -DFILAMENT_SUPPORTS_WEBGPU=ON -DFILAMENT_SKIP_SAMPLES=ON -DFILAMENT_BUILD_TESTING=OFF -DDAWN_ENABLE_PIC=ON \
    -DDAWN_BUILD_NODE_BINDINGS=ON -DGO_EXECUTABLE=$(command -v go) -DDAWN_ENABLE_WEBGPU_ON_WEBGPU=ON \
    -DCMAKE_CXX_FLAGS=-DMOTOLII_DAWN_CAPTURE > /dev/null
ninja -C $F/out/wgpu_capture third_party/dawn/dawn.node
