#!/bin/bash
# The component explorer on macOS, with hot reload (r / R in this terminal). Needs the native library built once
# (scripts/motolii-ui.sh native). FLUTTER_BIN picks the Flutter to use.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
export MOTOLII_NATIVE_LIBRARY="${MOTOLII_NATIVE_LIBRARY:-$here/../../target/debug/libmotolii_ui.dylib}"
export MOTOLII_EXPLORER_FIXTURES="$here/fixtures"
export MOTOLII_UI_DIR="$(cd "$here/.." && pwd)"
# the explorer keeps its own state (the media catalog, history): a story never touches a person's own
export MOTOLII_STATE_DIR="${MOTOLII_STATE_DIR:-$here/.state}"
cd "$here"
exec "${FLUTTER_BIN:-flutter}" run -d macos "$@"
