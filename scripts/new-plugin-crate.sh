#!/usr/bin/env bash
# VSM-A4I: 外部作者crateだけを生成・検査する。in-tree scaffoldとは混ぜない。
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec python3 "${ROOT_DIR}/scripts/new_plugin_crate.py" "$@"
