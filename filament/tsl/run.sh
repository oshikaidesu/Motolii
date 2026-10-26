#!/bin/zsh
# One collection's entry (tsl_textures.ts, ...) bundled by esbuild with this folder's three.js (package.json) and run in
# QuickJS: its effects land as WGSL in out/<collection>/, then each is checked by naga.   ./tsl/run.sh tsl_textures
set -e
cd "$(dirname "$0")"
NODE_PATH=$PWD/node_modules npx esbuild $1.ts --tsconfig=../third_party/tsl/basementstudio__shader-lab/tsconfig.json --bundle --format=esm --platform=neutral --main-fields=module,main \
  --outfile=out/$1.mjs --external:qjs:std --external:qjs:os --external:typescript --log-level=error
qjs -m out/$1.mjs
