#!/bin/zsh
# AbsentMotion's effects (Apache-2.0, the free Alight Motion alternative: github.com/Comett1225/AbsentMotion) on the Effects
# shelf: its repository at the commit taken, fetched once into third_party/AbsentMotion; Skia's skslc built once from
# third_party/skia_src (skia.sh's checkout, a build of its own: out/sksl); each of its SkSL effects made an effect file
# effects/am_<id>.wgsl by effects/from_absent.py. Prints how many were made and why the rest were not.
#   ./absent.sh   (ninja from Homebrew: Tint's build asks ninja for -t inputs)
cd "$(dirname "$0")"
A=third_party/AbsentMotion K=third_party/skia_src
[ -d $A/app ] || { mkdir -p $A && git init -q $A && git -C $A -c core.hooksPath=/dev/null fetch -q --depth 1 https://github.com/Comett1225/AbsentMotion 0f5d891b7e6fd7260ca6191b4eafef23141230c8 && git -C $A checkout -q FETCH_HEAD; }
[ -x $K/bin/gn ] || ./skia.sh
[ -f $K/out/sksl/build.ninja ] || (cd $K && bin/gn gen out/sksl --args='is_official_build=false is_debug=false skia_compile_sksl_tests=true skia_enable_pdf=false
    skia_enable_skottie=false skia_use_dawn=false skia_use_gl=false skia_use_metal=false skia_enable_ganesh=false skia_enable_graphite=false' > /dev/null)
PATH=/opt/homebrew/bin:$PATH ninja -C $K/out/sksl skslc > /dev/null || exit 1
cp $K/src/sksl/sksl_*.sksl $K/out/sksl/   # (the modules skslc reads beside itself: Skia's BUILD.gn copies them so)
rm -f effects/am_*.wgsl effects/am_*.am
made=0 why=()
for f in $A/app/src/main/assets/effects/*.json(N); do
    n=${f:t:r}
    [[ $n == foreign ]] && continue
    if r=$(python3 effects/from_absent.py $f effects/am_$n.wgsl $K/out/sksl/skslc 2>&1); then made=$((made + 1)); else why+=("$n: ${r##*: }"); fi
    rm -f effects/am_$n.wgsl.frag effects/am_$n.wgsl.raw.wgsl   # (its steps on the way)
done
cp $A/LICENSE effects/AbsentMotion-LICENSE.txt && cp $A/NOTICE effects/AbsentMotion-NOTICE.txt
echo "$made made; not made $#why:"
printf '  %s\n' $why
