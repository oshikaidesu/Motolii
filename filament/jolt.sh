#!/bin/zsh
# Jolt Physics (Jorrit Rouwe, MIT): the official source at the commit we use, fetched once into third_party/physics/JoltPhysics,
# built by its own CMake (Build/) as a static library, the same everywhere (CROSS_PLATFORM_DETERMINISTIC), without its tests,
# samples and viewer. Out comes third_party/physics/JoltPhysics/jbuild/libJolt.a (physics.cpp; edit.sh and build_lib.sh link it).
#   ./jolt.sh
set -e
cd "$(dirname "$0")"
J=$PWD/third_party/physics/JoltPhysics
[ -d $J ] || { mkdir -p $J && git init -q $J && git -C $J -c core.hooksPath=/dev/null fetch -q --depth 1 https://github.com/jrouwe/JoltPhysics a4b24c3fadd7eade0435f27e9d6a07564273eb5f && git -C $J checkout -q FETCH_HEAD; }
[ -f $J/jbuild/Makefile ] || cmake -S $J/Build -B $J/jbuild -G "Unix Makefiles" -DCMAKE_BUILD_TYPE=Release -DCROSS_PLATFORM_DETERMINISTIC=ON \
    -DTARGET_UNIT_TESTS=OFF -DTARGET_HELLO_WORLD=OFF -DTARGET_PERFORMANCE_TEST=OFF -DTARGET_SAMPLES=OFF -DTARGET_VIEWER=OFF \
    -DJPH_USE_VK=OFF -DJPH_USE_MTL=OFF -DJPH_USE_CPU_COMPUTE=OFF -DENABLE_ALL_WARNINGS=OFF > /dev/null   # (its GPU compute left out: we step it on the CPU)
make -C $J/jbuild -j$(sysctl -n hw.ncpu) Jolt > /dev/null
