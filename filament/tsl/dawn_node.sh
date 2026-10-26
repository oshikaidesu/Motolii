#!/bin/zsh
# Dawn's own WebGPU binding for JavaScript (dawn.node, src/dawn/node) built from Motolii's own Dawn: the Filament build
# dawn_probe/build.sh makes (out/wgpu, already DAWN_ENABLE_PIC), with DAWN_BUILD_NODE_BINDINGS on, as Dawn's
# src/dawn/node/README.md builds it. Its three inputs fetched at the commits Dawn's own DEPS names (dawn_node), and Go
# for its code generator (idlgen), from Homebrew.   ./tsl/dawn_node.sh   -> filament_src/out/wgpu/third_party/dawn/dawn.node
set -e
cd "$(dirname "$0")/.."
F=$PWD/third_party/filament_src
T=$F/third_party/dawn/third_party
for spec in "node-api-headers https://github.com/nodejs/node-api-headers d5cfe19da8b974ca35764dd1c73b91d57cd3c4ce" \
            "node-addon-api https://github.com/nodejs/node-addon-api 1e26dcb52829a74260ec262edb41fc22998669b6" \
            "gpuweb https://github.com/gpuweb/gpuweb e95743d3940e0ff3c267ab55ced9ae6120c7d416"; do
  set -- ${=spec}
  [ -d $T/$1/.git ] || { git init -q $T/$1 && git -C $T/$1 fetch -q --depth 1 $2 $3 && git -C $T/$1 checkout -q FETCH_HEAD; }
done
command -v go > /dev/null || brew install go
cmake -S $F -B $F/out/wgpu -DDAWN_BUILD_NODE_BINDINGS=ON -DGO_EXECUTABLE=$(command -v go) > /dev/null
ninja -C $F/out/wgpu third_party/dawn/dawn.node
