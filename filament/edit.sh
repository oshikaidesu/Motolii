#!/bin/zsh
# A document edited by calls, a line each on its input (edit.mm, doc.h).   ./edit.sh out/first.motolii < calls.txt
# (films by GStreamer, keys by its controller, the document's file and undo by SQLite: Homebrew's, whose header has the session calls)
cd "$(dirname "$0")"
doc=${1:a}; shift
# (built again only if something it is made of has changed since)
if [ ! -x out/edit_wgpu ] || [ -n "$(find . dawn_probe -maxdepth 1 \( -name '*.h' -o -name '*.mm' -o -name '*.mat' -o -name '*.glsl' -o -name '*.sh' \) -newer out/edit_wgpu | head -1)" ]; then
    KIT_EXTRA="$(pkg-config --cflags --libs gstreamer-1.0 gstreamer-app-1.0 gstreamer-video-1.0 gstreamer-controller-1.0) -lgltfio -lgltfio_core -ldracodec -DSQLITE_ENABLE_SESSION -DSQLITE_ENABLE_PREUPDATE_HOOK -I/opt/homebrew/opt/sqlite/include /opt/homebrew/opt/sqlite/lib/libsqlite3.dylib" BUILDONLY=1 ./dawn_probe/kit_wgpu.sh edit || exit 1
fi
cd out/wgpu_kit && BACKEND="${BACKEND:-webgpu}" ../edit_wgpu $doc "$@" 2>&1 | grep -v "FrameInfo's circular queue\|libvulkan\|BackendVk\|FEngine\|WebGPU adapter\|Backend feature"
