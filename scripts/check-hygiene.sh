#!/usr/bin/env bash
# 人なら即座に「変だ」と感じる数を、赤緑にする。閾値は reference/hygiene-budget.tsv。
# build しない。数えるだけなので数秒で終わる。
set -u
cd "$(dirname "$0")/../motolii" || exit 2
budget=reference/hygiene-budget.tsv
fail=0
val() { awk -F'\t' -v k="$1" '$1==k {print $2}' "$budget"; }
check() { # name actual
    local limit; limit=$(val "$1")
    if [ -z "$limit" ]; then echo "NG: $1 の天井が $budget に無い"; fail=1; return; fi
    if [ "$2" -gt "$limit" ]; then echo "NG: $1 = $2 > $limit"; fail=1; else echo "ok: $1 = $2 (<= $limit)"; fi
}
target_gb=$( [ -d target ] && du -sk target | awk '{print int($1/1048576)}' || echo 0 )
target_limit=$(val target_gb)
if [ "$target_gb" -gt "$target_limit" ]; then
    echo "WARN: target_gb = $target_gb > $target_limit (live cache は reference/build-artifacts.tsv で分類。総量だけでは消さない)"
else
    echo "ok: target_gb = $target_gb (<= $target_limit)"
fi
incr=$( [ -d target/debug/incremental ] && ls target/debug/incremental | wc -l | tr -d ' ' || echo 0 )
check incremental_sessions "$incr"
longest=$(find src crates/*/src -name '*.rs' -exec wc -l {} + | grep -v ' total$' | sort -rn | head -1 | awk '{print $1}')
check longest_file_lines "$longest"
files_over=$(find src crates/*/src -name '*.rs' -exec wc -l {} + | grep -v ' total$' | awk -v l="$(val file_lines_soft)" '$1>l' | wc -l | tr -d ' ')
check files_over_soft "$files_over"
[ "$fail" -eq 0 ] && echo "OK: hygiene 全項目通過" || { echo "FAILED"; exit 1; }
