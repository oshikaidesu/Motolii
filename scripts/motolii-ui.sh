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
  native|test|profile|check-read-only)
    if [[ -z "${FFMPEG_DIR:-}" ]] && command -v brew >/dev/null; then
      FFMPEG_DIR=$(brew --prefix ffmpeg)
      export FFMPEG_DIR
    fi
    if [[ -z "${LIBCLANG_PATH:-}" ]] && command -v xcrun >/dev/null; then
      clang_bin=$(xcrun --find clang)
      export LIBCLANG_PATH="$(dirname "$(dirname "$clang_bin")")/lib"
    fi
    ;;
esac
case "${1:-dev}" in
  check) exec python3 "$repo/scripts/check-stage5.py" ;;
  native) cd "$repo"; exec cargo build -p motolii-ui ;;
  check-read-only)
    cd "$repo"
    cargo check -p motolii-doc --no-default-features
    exec cargo check -p motolii-render -p motolii-jobs --lib
    ;;
  test-window)
    window_check_dir=$(mktemp -d /tmp/motolii-window-check.XXXXXX)
    trap 'rm -f -- "$window_check_dir/check"; rmdir -- "$window_check_dir"' EXIT
    xcrun swiftc "$ui/macos/Runner/WindowAttachment.swift" "$ui/tool/window_attachment_check.swift" -o "$window_check_dir/check"
    "$window_check_dir/check"
    ;;
  test)
    "$repo/scripts/motolii-ui.sh" why-slow
    "$repo/scripts/motolii-ui.sh" test-window
    cd "$repo"; cargo test -p motolii-script; cargo test -p motolii-doc; cargo test -p motolii-edit; cargo test -p motolii-jobs; cargo test -p motolii-ui --lib; cargo test -p motolii-render --lib
    dart_bin="$(dirname "$flutter_bin")/dart"
    (cd "$ui/tool/motolii_lints" && "$dart_bin" test && "$dart_bin" run bin/check.dart "$ui/lib")
    cd "$ui"; "$flutter_bin" analyze; exec "$flutter_bin" test ;;
  # Why the loop is slow, in 30 ms, without going and looking. rustc enumerates
  # every -L dependency directory once per crate: 10.9 s at 611k entries, 0.12 s
  # at 30k. An APFS directory costs ~32 bytes per entry, so stat answers O(1).
  why-slow)
    deps="$workspace/target/debug/deps"
    files=$(( $(stat -f %z "$deps" 2>/dev/null || echo 0) / 32 ))
    [[ $files -lt 50000 ]] || echo "slow: deps/ holds ~$files files and every rustc rescans it. Prune: find '$deps' -name '*.rcgu.o' -mtime +7 -delete"
    busy=$(pgrep -f 'bin/cargo' | wc -l | tr -d ' ')
    [[ $busy -eq 0 ]] || echo "slow: $busy other cargo run(s) hold the build lock on $workspace/target; yours waits for them."
    swap=$(sysctl -n vm.swapusage | sed -n 's/.*free = \([0-9]*\).*/\1/p')
    [[ ${swap:-9999} -gt 2048 ]] || echo "slow: swap has ${swap}M free; rustc stalls before it computes anything."
    ;;
  restart-ui) [[ -f "$state/flutter.pid" ]] || { echo 'No Stage 5 dev session.'; exit 1; }; kill -USR2 "$(cat "$state/flutter.pid")" ;;
  reload) [[ -f "$state/flutter.pid" ]] || { echo 'No Stage 5 dev session.'; exit 1; }; kill -USR1 "$(cat "$state/flutter.pid")" ;;
  # The real speed: Dart AOT and Rust --release. No hot reload (Flutter's
  # profile mode has none) — use it to judge how fast Motolii actually is,
  # not to work in.
  profile)
    [[ -n "$flutter_bin" ]] || { echo 'Install Flutter and set FLUTTER_BIN or add it to PATH.'; exit 1; }
    cd "$repo"; cargo build --release -p motolii-ui
    export MOTOLII_NATIVE_LIBRARY="$workspace/target/release/libmotolii_ui.dylib"
    cd "$ui"
    if [[ $# -gt 1 ]]; then
      exec "$flutter_bin" run --profile -d macos --dart-define="MOTOLII_DOCUMENT=$2"
    fi
    exec "$flutter_bin" run --profile -d macos
    ;;
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
  *) echo 'Usage: scripts/motolii-ui.sh {check|check-read-only|native|test|test-window|why-slow|dev [document.rrd]|profile [document.rrd]|reload|restart-ui}'; exit 1 ;;
esac
