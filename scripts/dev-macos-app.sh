#!/usr/bin/env bash
# macOS開発入口: Rust Debug増分build → React Native Debug app + Metro。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RN_DIR="$ROOT/ui/motolii-rn"
MACOS_DIR="$RN_DIR/macos"
MANIFEST="$RN_DIR/native-renderer/Cargo.toml"
CLI="$RN_DIR/node_modules/.bin/react-native"

fail() {
  echo "dev-macos-app: $1" >&2
  exit 1
}

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  echo "usage: scripts/dev-macos-app.sh"
  echo "Rust Debug増分build後、MotoliiRn Debug appとMetroを起動する。"
  exit 0
fi
[ $# -eq 0 ] || fail "引数は指定できない。--help を参照すること"

command -v cargo >/dev/null 2>&1 || fail "cargoが見つからない"
command -v curl >/dev/null 2>&1 || fail "curlが見つからない"
[ -x "$CLI" ] \
  || fail "node_modulesがない。先に ui/motolii-rn で corepack yarn install --immutable を実行すること"
[ -f "$MACOS_DIR/Pods/Manifest.lock" ] && cmp -s "$MACOS_DIR/Podfile.lock" "$MACOS_DIR/Pods/Manifest.lock" \
  || fail "Podsが未配置またはlockと不一致。macosで pod _1.15.2_ install --deployment を実行すること"

cargo build --manifest-path "$MANIFEST" --locked
cd "$RN_DIR"

if curl -fsS http://127.0.0.1:8081/status >/dev/null 2>&1; then
  exec "$CLI" run-macos --scheme MotoliiRn-macOS --project-path macos --no-packager
fi

"$CLI" start --port 8081 &
metro_pid=$!
trap 'kill "$metro_pid" 2>/dev/null || true' EXIT INT TERM
for _ in {1..100}; do
  curl -fsS http://127.0.0.1:8081/status >/dev/null 2>&1 && break
  kill -0 "$metro_pid" 2>/dev/null || fail "Metroの起動に失敗した"
  sleep 0.1
done
curl -fsS http://127.0.0.1:8081/status >/dev/null 2>&1 || fail "Metroが10秒以内に起動しなかった"

"$CLI" run-macos --scheme MotoliiRn-macOS --project-path macos --no-packager
wait "$metro_pid"
