#!/bin/sh
# Desk check of beam_glass.h (no renderer). Writes out/beam-top-*.png.
set -eu
cd "$(dirname "$0")"
mkdir -p out
clang++ -std=c++20 -O2 -Wall -Wextra beam_glass_test.cpp -lz -o out/beam_glass_test
./out/beam_glass_test
