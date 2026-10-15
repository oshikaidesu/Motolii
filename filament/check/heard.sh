#!/bin/zsh
# A film's own sound heard where the live stage puts it (check/heard.c), muted.   check/heard.sh
cd "$(dirname "$0")/.."
./edit.sh out/heard_build.motolii < /dev/null > /dev/null 2>&1   # (the library built, if anything it is made of changed)
rm -f out/heard_check.motolii out/heard_check
cc check/heard.c -Lout -lmotolii -Wl,-rpath,$PWD/out -o out/heard_check || exit 1
cd out/wgpu_kit && MOTOLII_MUTE=1 ../heard_check $PWD/../heard_check.motolii $PWD/../vid/happy.mp4 2>&1 | grep -v "Vulkan\|BackendVk\|^ *at \|FEngine\|adapter\|feature\|^$"
