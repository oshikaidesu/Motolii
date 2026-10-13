#!/bin/zsh
# The puppet picture (puppet.mm): a film from an .mp4 in the scene, its green screen keyed out.   ./puppet.sh [film | time]
cd "$(dirname "$0")"
mkdir -p out/vid
if [ ! -f out/vid/cat_green.mp4 ]; then   # the clip: an After Effects cat on a green ground, as a plain 30-a-second .mp4
    BUILDONLY=1 ./dawn_probe/kit_wgpu.sh ae || exit 1
    (cd out/wgpu_kit && GROUND="0 1 0" BACKEND=webgpu ../ae_wgpu ../../third_party/thorvg_example/res/lottie/cat_loader.json film > /dev/null 2>&1) || exit 1
    ffmpeg -loglevel error -y -i out/wgpu_kit/out/ae.mp4 -an -vf fps=30 -c:v libx264 -crf 20 -pix_fmt yuv420p out/vid/cat_green.mp4 || exit 1
fi
BUILDONLY=1 ./dawn_probe/kit_wgpu.sh puppet || exit 1
cd out/wgpu_kit && BACKEND="${BACKEND:-webgpu}" ../puppet_wgpu "$@" 2>&1 | grep -v "FrameInfo's circular queue\|libvulkan\|BackendVk"
