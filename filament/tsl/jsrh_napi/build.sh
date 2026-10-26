#!/bin/zsh
# out/jsrh_napi/qjs_napi: Motolii's QuickJS-ng 0.17 (Homebrew's libqjs) with BabylonJS/JsRuntimeHost's Node-API for
# QuickJS (fetched at 980cfb0 into third_party/napi/jsruntimehost), jsrh_qjs017_dawn.patch applied to a copy, and host.cc.
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
Q=/opt/homebrew/opt/quickjs-ng
S=$O/src/Node-API
clang++ -std=c++17 -O2 -w -DNAPI_VERSION=8 -I $Q/include -I $S/Include/Shared -I $S/Include/Engine/QuickJS -I $S/Source host.cc \
  $S/Source/js_native_api_quickjs.cc $S/Source/js_native_api_shared.cc $S/Source/env_quickjs.cc \
  -L $Q/lib -lqjs -Wl,-rpath,$Q/lib -Wl,-export_dynamic -o $O/qjs_napi
echo built $O/qjs_napi
