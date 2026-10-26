#!/bin/zsh
# shader-effects-inc/shaders effects written out as WGSL by shaderport's own exporter (tools/export-wgsl), the way its
# run.mjs runs it (the test copied into the upstream's tests, vitest), with shaderport_frames.patch: the child under
# each effect chosen (SP_BASE, e.g. VideoTexture), only the picked effects (SP_PICK), and each frame's uniform bytes
# (SP_FRAMES at SP_FPS, SP_W x SP_H) packed by TypeGPU from the upstream's own store; composed as the upstream renderer
# composes for the screen (no Y flip, premultiplied tail).
#   SP_BASE=VideoTexture SP_PICK=Glitch,VHS SP_FRAMES=120 ./tsl/shaderport_export.sh <out folder>
set -e
cd "$(dirname "$0")/.."
./tsl/fetch.sh shader-effects-inc__shaders > /dev/null && ./tsl/fetch.sh quizzyjoy__shaderport > /dev/null
U=$PWD/third_party/tsl/shader-effects-inc__shaders
P=$PWD/third_party/tsl/quizzyjoy__shaderport
[ -d $U/node_modules ] || (cd $U && corepack pnpm install --filter shaders-core... --ignore-scripts > /dev/null)
T=$(mktemp -d)
cp $P/tools/export-wgsl/export.test.ts $T/ && (cd $T && patch -s -p3 < $OLDPWD/tsl/shaderport_frames.patch)
mkdir -p "$1"
OUT=$(cd "$1" && pwd)
cp $T/export.test.ts $U/packages/core/src/__tests__/gpu/_shaderport_export.test.ts
(cd $U/packages/core && SP_OUT=$OUT corepack pnpm exec vitest run src/__tests__/gpu/_shaderport_export.test.ts 2>&1 | grep -E "export-wgsl|Tests|FAIL|Error" ) || true
rm -f $U/packages/core/src/__tests__/gpu/_shaderport_export.test.ts
rm -rf $T
