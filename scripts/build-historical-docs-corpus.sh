#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
    echo "usage: $0 <cutoff-refs.tsv>" >&2
    exit 2
fi

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
refs_file=$1

if [ ! -f "$refs_file" ]; then
    echo "missing cutoff refs: $refs_file" >&2
    exit 2
fi

tip_count=$(awk -F '\t' 'NR > 1 { print $2 }' "$refs_file" | sort -u | wc -l | tr -d ' ')
if [ "$tip_count" -eq 0 ]; then
    echo "cutoff refs contain no commit tips" >&2
    exit 2
fi

printf 'blob_sha\tbytes\tobserved_path\n'
awk -F '\t' 'NR > 1 { print $2 }' "$refs_file" \
    | sort -u \
    | git -C "$repo_root" rev-list --objects --stdin -- docs \
    | git -C "$repo_root" cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' \
    | awk '$1 == "blob" && $4 ~ /^docs\/.*\.md$/ { print $2 "\t" $3 "\t" $4 }' \
    | LC_ALL=C sort -t "$(printf '\t')" -k3,3 -k1,1
