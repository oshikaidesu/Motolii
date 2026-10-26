#!/bin/zsh
# Every WGSL in out/<collection>/ checked by naga (wgpu's WGSL validator): how many pass, and why the others don't.
cd "$(dirname "$0")"
for d in out/*/; do
  c=$(basename $d); pass=0; fail=0
  for f in $d*.wgsl(N); do if naga $f >/dev/null 2>&1; then pass=$((pass+1)); else fail=$((fail+1)); echo "$c/$(basename $f): $(naga $f 2>&1 | grep -v '^\s*$' | sed -n '1p;$p' | tr '\n' ' ' | cut -c1-160)" >> out/naga_fail.txt; fi; done
  echo "$c: $pass of $((pass+fail)) pass naga"
done
