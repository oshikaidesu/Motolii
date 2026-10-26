#!/bin/zsh
# out/jsrh_napi/src: BabylonJS/JsRuntimeHost's Node-API for QuickJS (fetched at 980cfb0 into third_party/napi/jsruntimehost)
# with jsrh_qjs017_dawn.patch applied to a copy, for Motolii's QuickJS-ng 0.17 (Homebrew's libqjs): dawn_probe/js_effects.sh builds it in.
set -e
cd "$(dirname "$0")"
R=../../third_party/napi/jsruntimehost
if [ ! -d $R/.git ]; then
  git init -q $R && git -C $R remote add origin https://github.com/BabylonJS/JsRuntimeHost.git
  git -C $R sparse-checkout set --no-cone 'Core/Node-API/' LICENSE.md
  git -C $R fetch -q --depth 1 origin 980cfb0197f7fda661d362df4b275cff42d1e7f2 && git -C $R checkout -q FETCH_HEAD
fi
O=../out/jsrh_napi
rm -rf $O/src && mkdir -p $O/src && cp -R $R/Core/Node-API $O/src/
(cd $O/src && patch -s -p2 < ../../../jsrh_napi/jsrh_qjs017_dawn.patch)
echo patched $O/src
