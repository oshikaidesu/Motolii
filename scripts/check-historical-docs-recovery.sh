#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
evidence_dir="$repo_root/docs/reviews/evidence/historical-value-recovery"
manifest="$evidence_dir/corpus.tsv"
receipts_dir="$evidence_dir/disposition-receipts"
read_receipts_dir="$evidence_dir/read-receipts"
require_complete=false

if [ "${1:-}" = "--complete" ]; then
    require_complete=true
elif [ "$#" -ne 0 ]; then
    echo "usage: $0 [--complete]" >&2
    exit 2
fi

if [ ! -f "$manifest" ]; then
    echo "missing corpus manifest: $manifest" >&2
    exit 1
fi

tmp_dir=$(mktemp -d "${TMPDIR:-/tmp}/motolii-history-coverage.XXXXXX")
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

awk -F '\t' 'NR > 1 { print $1 }' "$manifest" | LC_ALL=C sort > "$tmp_dir/corpus"
if [ -n "$(uniq -d "$tmp_dir/corpus")" ]; then
    echo "duplicate blob in corpus manifest" >&2
    uniq -d "$tmp_dir/corpus" >&2
    exit 1
fi

find "$receipts_dir" -type f -name '*.tsv' -print0 \
    | LC_ALL=C sort -z \
    | xargs -0 awk -F '\t' 'FNR > 1 { print $1 }' \
    | LC_ALL=C sort > "$tmp_dir/covered"

if [ -n "$(uniq -d "$tmp_dir/covered")" ]; then
    echo "blob assigned by more than one disposition receipt" >&2
    uniq -d "$tmp_dir/covered" >&2
    exit 1
fi

comm -13 "$tmp_dir/corpus" "$tmp_dir/covered" > "$tmp_dir/unknown"
if [ -s "$tmp_dir/unknown" ]; then
    echo "receipt contains blob outside cutoff corpus" >&2
    sed -n '1,20p' "$tmp_dir/unknown" >&2
    exit 1
fi

comm -23 "$tmp_dir/corpus" "$tmp_dir/covered" > "$tmp_dir/remaining"

find "$read_receipts_dir" -type f -name '*.tsv' -print0 \
    | LC_ALL=C sort -z \
    | xargs -0 awk -F '\t' 'FNR > 1 { print $1 }' \
    | LC_ALL=C sort > "$tmp_dir/read"

if [ -n "$(uniq -d "$tmp_dir/read")" ]; then
    echo "blob assigned by more than one read receipt" >&2
    uniq -d "$tmp_dir/read" >&2
    exit 1
fi

comm -13 "$tmp_dir/corpus" "$tmp_dir/read" > "$tmp_dir/unknown-read"
if [ -s "$tmp_dir/unknown-read" ]; then
    echo "read receipt contains blob outside cutoff corpus" >&2
    sed -n '1,20p' "$tmp_dir/unknown-read" >&2
    exit 1
fi

corpus_count=$(wc -l < "$tmp_dir/corpus" | tr -d ' ')
covered_count=$(wc -l < "$tmp_dir/covered" | tr -d ' ')
remaining_count=$(wc -l < "$tmp_dir/remaining" | tr -d ' ')
read_count=$(wc -l < "$tmp_dir/read" | tr -d ' ')

echo "historical docs recovery coverage: read=$read_count; dispositioned=$covered_count/$corpus_count; remaining=$remaining_count"

if [ "$require_complete" = true ] && [ "$remaining_count" -ne 0 ]; then
    exit 1
fi
