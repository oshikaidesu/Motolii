#!/bin/zsh
# Forty stars waving: ThorVG pictures bent as drawn, or Filament meshes bent in their material (check/shape_mesh.mm).
#   check/shape_mesh.sh sheets|meshes [time]
cd "$(dirname "$0")/.."
KIT_SRC=check/shape_mesh.mm BUILDONLY=1 ./dawn_probe/kit_wgpu.sh shape_mesh || exit 1
cd out/wgpu_kit && BACKEND=webgpu ../shape_mesh_wgpu "$@" 2>&1 | grep -v "FrameInfo's circular queue\|libvulkan\|BackendVk"
