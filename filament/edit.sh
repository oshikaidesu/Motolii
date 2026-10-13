#!/bin/zsh
# A document edited by calls, a line each on its input (edit.mm, doc.h).   ./edit.sh out/first.motolii < calls.txt
# (films by GStreamer, keys by its controller, the document's file and undo by SQLite: Homebrew's, whose header has the session calls)
cd "$(dirname "$0")"
doc=${1:a}; shift
KIT_EXTRA="$(pkg-config --cflags --libs gstreamer-1.0 gstreamer-app-1.0 gstreamer-video-1.0 gstreamer-controller-1.0) -DSQLITE_ENABLE_SESSION -DSQLITE_ENABLE_PREUPDATE_HOOK -I/opt/homebrew/opt/sqlite/include /opt/homebrew/opt/sqlite/lib/libsqlite3.dylib" BUILDONLY=1 ./dawn_probe/kit_wgpu.sh edit || exit 1
cd out/wgpu_kit && BACKEND="${BACKEND:-webgpu}" ../edit_wgpu $doc "$@" 2>&1 | grep -v "FrameInfo's circular queue\|libvulkan\|BackendVk\|FEngine\|WebGPU adapter\|Backend feature"
