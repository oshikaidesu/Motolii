#!/bin/zsh
# motion.h and lottie3d.py against answers worked out by hand and lottie-web's own (motion_test.mm).   ./motion_test.sh
cd "$(dirname "$0")"
./lottie3d.py motion_test3d.src.json motion_test3d.json || exit 1
./lottie3d.py led.src.json led.json || exit 1
[ out/motion_test -nt motion_test.mm ] && [ out/motion_test -nt motion.h ] && [ out/motion_test -nt motion_test3d.inc ] || ./build_one.sh motion_test || exit 1
./out/motion_test | grep -v "^ok"
