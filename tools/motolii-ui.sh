#!/usr/bin/env bash
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
ui="$repo/app/ui"
flutter_bin="${FLUTTER_BIN:-$repo/.tools/flutter/bin/flutter}"
if [[ ! -x "$flutter_bin" ]]; then flutter_bin=$(command -v flutter || true); fi
case "${1:-help}" in
 check) exec python3 "$repo/tools/check-repository.py" ;;
 native) cd "$repo"; exec cargo build -p motolii-ui ;;
 dev) [[ -n "$flutter_bin" ]] || { echo 'Install Flutter or set FLUTTER_BIN.'; exit 1; }; export MOTOLII_NATIVE_LIBRARY="$repo/target/debug/libmotolii_ui.dylib"; cd "$ui"; exec "$flutter_bin" run -d macos ;;
 *) echo 'Motolii: check | native | dev'; exit 2 ;;
esac
