#!/bin/zsh
# The ways a sheet lies on what is behind it (lay.mm).   ./lay.sh [film | time]
cd "$(dirname "$0")"
KIT_EXTRA="$(pkg-config --cflags --libs gstreamer-1.0 gstreamer-app-1.0 gstreamer-video-1.0)" BUILDONLY=1 ./dawn_probe/kit_wgpu.sh lay || exit 1
cd out/wgpu_kit && BACKEND="${BACKEND:-webgpu}" ../lay_wgpu "$@" 2>&1 | grep -v "FrameInfo's circular queue\|libvulkan\|BackendVk"
