#!/bin/zsh
# The window on Workspace > Live with Flutter's own hot reload: a .dart under lib/ saved shows in the open window in a
# moment, its state kept (flutter run reloads when sent SIGUSR1, restarts on SIGUSR2; r and R still work here too).
# The inside's own files reload by themselves (effects .wgsl, scripts .js). Ends when the window is closed.
#   ./live.sh                      (opens ~/Movies/Motolii/Untitled.motolii; MOTOLII_DOC=/path/to.motolii for another document)
cd "$(dirname "$0")"
FLUTTER="${FLUTTER:-flutter}"
pid="build/live.pid" seen="build/live.seen"
: ${MOTOLII_DOC:=$HOME/Movies/Motolii/Untitled.motolii}   # (the document, and beside it what Render writes and File › New makes; MOTOLII_DOC=… for another)
mkdir -p "${MOTOLII_DOC:h}"
mkdir -p build && rm -f "$pid" && touch "$seen"
"$FLUTTER" run -d macos --pid-file "$pid" --dart-define=LAB_ROUTE='/?path=workspace/workspace/live' --dart-define=MOTOLII_DIR="${MOTOLII_DIR:-${PWD:h:h}/filament}" ${MOTOLII_DOC:+--dart-define=MOTOLII_DOC="$MOTOLII_DOC"} "$@" &
run=$!
trap 'kill $run 2>/dev/null' EXIT INT TERM
while kill -0 $run 2>/dev/null; do   # (once a second: a .dart newer than the last reload, then reload)
    sleep 1
    if [ -f "$pid" ] && [ -n "$(find lib -name '*.dart' -newer "$seen" | head -1)" ]; then
        touch "$seen"
        kill -USR1 "$(cat "$pid")" 2>/dev/null && echo "live.sh: reloaded"
    fi
done
