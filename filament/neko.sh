#!/bin/zsh
# The cat populace (neko.mm): films from out/vid, many times over, keyed, as a crowd.   ./neko.sh [film | time]
cd "$(dirname "$0")"
mkdir -p out/vid
if [ ! -f out/vid/cat_green.mp4 ]; then   # the clip: an After Effects cat on a green ground, as a plain 30-a-second .mp4
    BUILDONLY=1 ./dawn_probe/kit_wgpu.sh ae || exit 1
    (cd out/wgpu_kit && GROUND="0 1 0" BACKEND=webgpu ../ae_wgpu ../../third_party/thorvg_example/res/lottie/cat_loader.json film > /dev/null 2>&1) || exit 1
    ffmpeg -loglevel error -y -i out/wgpu_kit/out/ae.mp4 -an -vf fps=30 -c:v libx264 -crf 20 -pix_fmt yuv420p out/vid/cat_green.mp4 || exit 1
fi
# (films are read by GStreamer: Homebrew's, 1.28)
KIT_EXTRA="$(pkg-config --cflags --libs gstreamer-1.0 gstreamer-app-1.0 gstreamer-video-1.0 gstreamer-pbutils-1.0)" BUILDONLY=1 ./dawn_probe/kit_wgpu.sh neko || exit 1
cd out/wgpu_kit && BACKEND="${BACKEND:-webgpu}" ../neko_wgpu "$@" 2>&1 | grep -v "FrameInfo's circular queue\|libvulkan\|BackendVk"
