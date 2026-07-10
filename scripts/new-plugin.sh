#!/usr/bin/env bash
# INF-7e: 規約準拠のプラグイン・スケルトンを1発生成する。
# 実体は同ディレクトリの new_plugin.py (テンプレと検証を一箇所に)。
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec python3 "${ROOT_DIR}/scripts/new_plugin.py" "$@"
