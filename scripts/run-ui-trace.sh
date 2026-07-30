#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <project.json>" >&2
  exit 2
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
project_path="$1"
trace_dir="${MOTOLII_UI_TRACE_DIR:-$repo_root/target/ui-traces}"
timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
trace_path="$trace_dir/motolii-ui-$timestamp.log"

mkdir -p "$trace_dir"
echo "UI trace: $trace_path"
(
  cd "$repo_root"
  MOTOLII_UI_TRACE=1 \
    cargo run -p motolii-ui --bin motolii_ui_shell -- "$project_path"
) 2>&1 | tee "$trace_path"
