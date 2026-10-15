#!/bin/zsh
# Memory while many films play live (check/films_memory.c), muted.   check/films_memory.sh [seconds]
cd "$(dirname "$0")/.."
./edit.sh out/films_memory_build.motolii < /dev/null > /dev/null 2>&1; rm -f out/films_memory_build.motolii
rm -f out/films_memory.motolii
cc check/films_memory.c -Lout -lmotolii -Wl,-rpath,$PWD/out -o out/films_memory || exit 1
cd out/wgpu_kit && MOTOLII_MUTE=1 ../films_memory $PWD/../films_memory.motolii $PWD/../vid ${1:-180} 2>&1 | grep -v "Vulkan\|BackendVk\|^ *at \|FEngine\|adapter\|feature\|GStreamer-GL-WARNING\|^$"
