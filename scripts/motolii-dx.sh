#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
case "${1:-serve}" in
  serve)
    shift 2>/dev/null || true
    exec python3 "$repo_root/scripts/reload-runtime.py" serve "$@"
    ;;
  doctor)
    python3 "$repo_root/scripts/reload-runtime.py" doctor
    exec "$repo_root/scripts/check-build-loop.sh"
    ;;
  *)
    echo "usage: scripts/motolii-dx.sh [serve|doctor]; stock bypass is retired" >&2
    exit 2
    ;;
esac
