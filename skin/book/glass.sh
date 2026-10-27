#!/bin/sh
# Opens the lab's window on Glass > Live: the Filament glass of ../../filament, drawn live and shown as a texture
# (lib/sets/glass_live.dart). Builds the glass library first if it is not there.
# FLUTTER=/path/to/flutter/bin/flutter to use another Flutter.
set -eu
cd "$(dirname "$0")"
GLASS="$(cd ../../filament && pwd)"
[ -f "$GLASS/out/libglass_live.dylib" ] || "$GLASS/build_live.sh"
FLUTTER="${FLUTTER:-flutter}"
exec "$FLUTTER" run -d macos --dart-define=GLASS_DIR="$GLASS" --dart-define=LAB_ROUTE='/?path=glass/glass/live' "$@"
