#!/bin/zsh
# Filament's own prebuilt SDK for macOS (Apache-2.0), v1.77.2 as Google releases it, unpacked once into third_party/filament:
# the Metal build the earlier probes use (ring.sh, poster.sh, blocks.sh, ... and its matc and cmgen). The editor builds
# Filament from source with its WebGPU backend instead (dawn_probe/build.sh).
#   ./sdk.sh
set -e
cd "$(dirname "$0")"
[ -d third_party/filament/lib ] && exit 0
mkdir -p third_party
curl -fsSL -o third_party/filament-v1.77.2-mac.tgz https://github.com/google/filament/releases/download/v1.77.2/filament-v1.77.2-mac.tgz
echo "8ed807b398b7a7146a5c3d0800c183b8b7091b0654c405f6a3590e2101a97508  third_party/filament-v1.77.2-mac.tgz" | shasum -a 256 -c - > /dev/null
tar xzf third_party/filament-v1.77.2-mac.tgz -C third_party
