#!/usr/bin/env bash
# Motolii 固有の開発ループ診断。読むだけで、build・clean・process停止はしない。
set -u

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
product_root="$repo_root/motolii"
budget_file="$product_root/reference/hygiene-budget.tsv"
loop_budget="$product_root/reference/build-loop-budget.tsv"
runtime="$repo_root/scripts/reload-runtime.py"
target_limit_gib="$(awk -F '\t' '$1=="target_gb" {print $2}' "$budget_file")"
libdeps_advisory="$(awk -F '\t' '$1=="libdeps_generations" {print $2}' "$loop_budget")"

fail=0
warn=0
pass() { echo "PASS $1 $2"; }
advice() { echo "WARN $1 $2"; warn=$((warn + 1)); }
ng() { echo "NG $1 $2"; fail=$((fail + 1)); }
size_kib() { [ -e "$1" ] && du -sk "$1" 2>/dev/null | awk '{print $1}' || echo 0; }

if binary_info="$(python3 "$runtime" binary-info)"; then
  fixed_dx="$(printf '%s' "$binary_info" | python3 -c 'import json,sys; print(json.load(sys.stdin)["path"])')"
  actual_sha="$(printf '%s' "$binary_info" | python3 -c 'import json,sys; print(json.load(sys.stdin)["sha256"])')"
  pass fixed_dx "sha256=$actual_sha path=$fixed_dx"
else
  fixed_dx="unverified"
  ng fixed_dx "runtime descriptor or pinned executable unavailable"
fi

if inventory="$(python3 "$runtime" inventory)"; then
  counts="$(printf '%s' "$inventory" | python3 -c '
import json,sys
from pathlib import Path
rows=json.load(sys.stdin); fixed=Path(sys.argv[1]).resolve()
ok=[r for r in rows if Path(r["executable"]).resolve()==fixed]
print(len(rows)-len(ok), len(ok), sum(r["state"].startswith("T") for r in rows))
' "$fixed_dx")"
  read -r stock_count fixed_count stopped_builds <<< "$counts"
  [ "$stock_count" -eq 0 ] && pass stock_dx_process "count=0" || ng stock_dx_process "count=$stock_count"
  if [ "$fixed_count" -eq 1 ]; then pass fixed_dx_process "count=1; reuse warm process"; elif [ "$fixed_count" -eq 0 ]; then advice fixed_dx_process "count=0; initial baseline requires exception record"; else ng fixed_dx_process "count=$fixed_count"; fi
  [ "$stopped_builds" -eq 0 ] && pass stopped_build_processes "count=0" || ng stopped_build_processes "count=$stopped_builds"
else
  ng process_inventory "inspection failed; unknown is not zero"
fi

target_kib="$(size_kib "$product_root/target")"
target_gib=$((target_kib / 1048576))
[ "$target_gib" -le "$target_limit_gib" ] \
  && pass target_gib "value=$target_gib advisory=$target_limit_gib" \
  || advice target_gib "value=$target_gib advisory=$target_limit_gib; classify by reference/build-artifacts.tsv"

debug_kib="$(size_kib "$product_root/target/debug")"
triple_kib="$(size_kib "$product_root/target/aarch64-apple-darwin")"
host_profile_kib="$(size_kib "$product_root/target/desktop-dev")"
libdeps_count="$(find "$product_root/target/aarch64-apple-darwin/desktop-dev" -maxdepth 1 -type f -name 'libdeps-*.a' 2>/dev/null | wc -l | tr -d ' ')"
debug_incremental_count="$(find "$product_root/target/debug/incremental" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l | tr -d ' ')"
dx_incremental_count="$(find "$product_root/target/aarch64-apple-darwin/desktop-dev/incremental" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l | tr -d ' ')"
echo "INFO target_axes debug_kib=$debug_kib triple_kib=$triple_kib host_profile_kib=$host_profile_kib"
echo "INFO hotpatch_cache libdeps_generations=$libdeps_count debug_incremental=$debug_incremental_count dx_incremental=$dx_incremental_count"
[ "$libdeps_count" -le "$libdeps_advisory" ] \
  && pass libdeps_generations "value=$libdeps_count advisory=$libdeps_advisory" \
  || advice libdeps_generations "value=$libdeps_count advisory=$libdeps_advisory; do-not-clean-live-cache"

cargo_bin="$(asdf which cargo 2>/dev/null || command -v cargo 2>/dev/null || true)"
cargo_home=""
if [ -n "$cargo_bin" ]; then cargo_home="$(dirname "$(dirname "$cargo_bin")")"; fi
nested_count=0
nested_kib=0
if [ -d "$cargo_home/git/checkouts" ]; then
  while IFS= read -r nested_target; do
    nested_count=$((nested_count + 1))
    one_kib="$(size_kib "$nested_target")"
    nested_kib=$((nested_kib + one_kib))
    echo "INFO nested_checkout_target path=$nested_target kib=$one_kib"
  done < <(find "$cargo_home/git/checkouts" -type d -name target -prune -print 2>/dev/null)
fi
[ "$nested_count" -eq 0 ] \
  && pass nested_checkout_targets "count=0" \
  || ng nested_checkout_targets "count=$nested_count kib=$nested_kib"

sccache_requests="$(sccache --show-stats 2>/dev/null | awk '/^Compile requests[[:space:]]/ {print $NF; exit}')"
if [ -z "$sccache_requests" ] || [ "$sccache_requests" -eq 0 ]; then
  advice sccache "compile_requests=${sccache_requests:-unknown}; cold-build-probe-only"
else
  pass sccache "compile_requests=$sccache_requests"
fi

echo "SUMMARY failures=$fail warnings=$warn"
[ "$fail" -eq 0 ]
