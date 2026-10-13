#!/bin/zsh
# motion.h against answers worked out by hand and lottie-web's own (motion_test.mm), with the ThorVG that the kit uses.   ./motion_test.sh
cd "$(dirname "$0")"
BUILDONLY=1 ./dawn_probe/kit_wgpu.sh motion_test || exit 1
./out/motion_test_wgpu | grep -v "^ok"
