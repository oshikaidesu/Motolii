#!/bin/zsh
# The led picture (led.mm): glass things on paper, their motion read from led.json (After Effects motion, worked out by ThorVG).
#   ./led.sh  ->  out/wgpu_kit/out/led.png        ./led.sh film  ->  out/wgpu_kit/out/led.mp4
cd "$(dirname "$0")"
./sdk.sh || exit 1   # (Filament's prebuilt SDK, fetched once: sdk.sh)
[ -f out/lightroom/lightroom_ibl.ktx ] || third_party/filament/bin/cmgen -q -x out/lightroom --format=ktx --size=512 third_party/filament_src/third_party/environments/lightroom_14b.hdr || exit 1
BUILDONLY=1 ./dawn_probe/kit_wgpu.sh led || exit 1
ln -sf ../../led.json out/wgpu_kit/led.json; ln -sf ../../led_card.json out/wgpu_kit/led_card.json
cd out/wgpu_kit && BACKEND="${BACKEND:-webgpu}" ../led_wgpu "$@" 2>&1 | grep -v "FrameInfo's circular queue\|libvulkan\|BackendVk"
