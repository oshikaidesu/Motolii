#!/bin/zsh
# Forty-two films put back at once while they play (check/seek_settle.mm): how soon all are right again.
cd "$(dirname "$0")/.."
KIT_SRC=check/seek_settle.mm KIT_EXTRA="$(pkg-config --cflags --libs gstreamer-1.0 gstreamer-app-1.0 gstreamer-video-1.0)" BUILDONLY=1 ./dawn_probe/kit_wgpu.sh seek_settle > /dev/null || exit 1
cd out/wgpu_kit && BACKEND=webgpu FILMS="${FILMS:-../vid}" ../seek_settle_wgpu 2>&1 | grep -E "put back|after:|sought"
