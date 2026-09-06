#!/bin/bash
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
workspace="$repo/motolii"
ui="$workspace/ui"
state="${XDG_STATE_HOME:-$HOME/.local/state}/motolii-stage5"
flutter_bin="${FLUTTER_BIN:-$repo/.tools/flutter/bin/flutter}"
if [[ ! -x "$flutter_bin" ]]; then flutter_bin=$(command -v flutter || true); fi
mkdir -p "$state"
case "${1:-dev}" in
  check) exec python3 "$repo/scripts/check-stage5.py" ;;
  native) cd "$repo"; exec cargo build -p motolii-ui ;;
  test) cd "$repo"; cargo test -p motolii-doc --test edit_transactions; cargo test -p motolii-doc --lib; cargo test -p motolii-ui --lib; cargo test -p motolii-road --test owned_budget; cd "$ui"; exec "$flutter_bin" test ;;
  restart-ui) [[ -f "$state/flutter.pid" ]] || { echo 'No Stage 5 dev session.'; exit 1; }; kill -USR2 "$(cat "$state/flutter.pid")" ;;
  reload) [[ -f "$state/flutter.pid" ]] || { echo 'No Stage 5 dev session.'; exit 1; }; kill -USR1 "$(cat "$state/flutter.pid")" ;;
  dev)
    [[ -n "$flutter_bin" ]] || { echo 'Install Flutter and set FLUTTER_BIN or add it to PATH.'; exit 1; }
    export MOTOLII_NATIVE_LIBRARY="$workspace/target/debug/libmotolii_ui.dylib"
    [[ -f "$MOTOLII_NATIVE_LIBRARY" ]] || { echo 'Run scripts/motolii-ui.sh native once, then dev.'; exit 1; }
    cd "$ui"
    if [[ $# -gt 1 ]]; then
      exec "$flutter_bin" run -d macos --pid-file "$state/flutter.pid" --dart-define="MOTOLII_DOCUMENT=$2"
    fi
    exec "$flutter_bin" run -d macos --pid-file "$state/flutter.pid"
    ;;
  *) echo 'Usage: scripts/motolii-ui.sh {check|native|test|dev [document.rrd]|reload|restart-ui}'; exit 1 ;;
esac
