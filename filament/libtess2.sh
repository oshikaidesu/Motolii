#!/bin/zsh
# libtess2 (Mikko Mononen's cleaned-up SGI tessellator, SGI Free Software License B 2.0): a flat shape's outline cut into
# triangles, with holes, crossings and fill rules, as Unity's vector graphics do (LibTessDotNet is its port). Fetched
# once into third_party/libtess2, built as one static library: third_party/libtess2/build/libtess2.a.
#   ./libtess2.sh
set -e
cd "$(dirname "$0")"
L=$PWD/third_party/libtess2
[ -d $L ] || git -c core.hooksPath=/dev/null clone -q --depth 1 https://github.com/memononen/libtess2 $L
mkdir -p $L/build
[ -f $L/build/libtess2.a ] || (cd $L/build && cc -O2 -c -I ../Include ../Source/*.c && ar rcs libtess2.a *.o)
