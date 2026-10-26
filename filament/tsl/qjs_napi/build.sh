#!/bin/zsh
# out/qjs_napi/qjs_napi: NativeScript/runtimes' Node-API over its QuickJS (fetched at 5b697b3 into
# third_party/napi/ns_runtimes), ns_finalize_once.patch applied to a copy, and host.c; mimalloc as runtimes uses it (its
# own copy, mimalloc-dev, built from its single-file src/static.c).
set -e
cd "$(dirname "$0")"
R=../../third_party/napi/ns_runtimes
if [ ! -d $R/.git ]; then
  git init -q $R && git -C $R remote add origin https://github.com/NativeScript/runtimes.git
  git -C $R sparse-checkout set --no-cone 'NativeScript/napi/quickjs/' 'NativeScript/napi/common/'
  git -C $R fetch -q --depth 1 origin 5b697b393152f973dd392851fd72dacf03a7a0c9 && git -C $R checkout -q FETCH_HEAD
fi
O=../out/qjs_napi
rm -rf $O/src && mkdir -p $O/src && cp -R $R/NativeScript/napi/common $R/NativeScript/napi/quickjs $O/src/
(cd $O/src && for p in ns_finalize_once ns_coerce_to_object ns_typedarray_info; do patch -s -p1 < ../../../qjs_napi/$p.patch; done)
Q=$O/src/quickjs/source
clang -O2 -std=gnu11 -D_GNU_SOURCE -DNAPI_VERSION=8 -DCONFIG_VERSION='"ns"' -w -I $Q -I $O/src/common -I $O/src/quickjs host.c \
  -I $O/src/quickjs/mimalloc-dev/include $O/src/quickjs/mimalloc-dev/src/static.c $Q/cutils.c $Q/libbf.c $Q/libregexp.c $Q/libunicode.c $Q/quickjs.c $Q/quickjs-libc.c -Wl,-export_dynamic -lm -o $O/qjs_napi
echo built $O/qjs_napi
