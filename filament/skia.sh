#!/bin/zsh
# Skia's paths alone (its path ops, path effects, contour measure, stroker): the official source at the branch we use,
# fetched once into third_party/skia_src, built as fontTools' skia-pathops builds it (no GPU, fonts, images or PDF).
# Cavalry draws its shapes' paths with the same. Out comes third_party/skia_src/out/path/libskia.a.
#   ./skia.sh
set -e
cd "$(dirname "$0")"
K=$PWD/third_party/skia_src
[ -d $K ] || git -c core.hooksPath=/dev/null clone -q --depth 1 --branch chrome/m149 https://skia.googlesource.com/skia $K
[ -x $K/bin/gn ] || (cd $K && python3 bin/fetch-gn > /dev/null)
[ -f $K/out/path/build.ninja ] || (cd $K && bin/gn gen out/path --args='is_official_build=true is_debug=false skia_enable_pdf=false
    skia_enable_discrete_gpu=false skia_enable_ganesh=false skia_enable_graphite=false skia_enable_skottie=false skia_enable_skshaper=false
    skia_use_dng_sdk=false skia_use_expat=false skia_use_freetype=false skia_use_fontconfig=false skia_use_fonthost_mac=false skia_use_gl=false
    skia_use_harfbuzz=false skia_use_icu=false skia_use_libjpeg_turbo_encode=false skia_use_libjpeg_turbo_decode=false skia_use_libpng_encode=false
    skia_use_libpng_decode=false skia_use_libwebp_encode=false skia_use_libwebp_decode=false skia_use_piex=false skia_use_xps=false skia_use_zlib=false
    skia_enable_spirv_validation=false skia_use_lua=false skia_use_wuffs=false skia_enable_fontmgr_empty=true' > /dev/null)
ninja -C $K/out/path skia > /dev/null
