#!/usr/bin/env bash
# Timeline widget の視覚調整だけを速く確認するためのローカル開発補助。
set -euo pipefail

if [[ ${1:-} == --help ]]; then
  echo "usage: $0 [out/timeline.png]"
  exit 0
fi

if [[ $# -gt 1 ]]; then
  echo "usage: $0 [out/timeline.png]" >&2
  exit 2
fi

out=${1:-out/timeline-widget.png}
source_dir=crates/motolii-ui/src/timeline_blitz
dump_source=crates/motolii-ui/src/blitz_dump

fingerprint() {
  find "$source_dir" "$dump_source" -type f -print0 | sort -z | xargs -0 shasum
}

render() {
  cargo build --profile fast -p motolii-ui --bin motolii-blitz-dump
  mkdir -p "$(dirname "$out")"
  target/fast/motolii-blitz-dump timeline "$out"
  echo "updated $out"
}

last=$(fingerprint)
render
echo "watching $source_dir (Ctrl-C to stop)"

while :; do
  sleep 0.25
  current=$(fingerprint)
  if [[ "$current" != "$last" ]]; then
    last=$current
    render
  fi
done
