#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
COMMON_DIR="$(git -C "$ROOT_DIR" rev-parse --path-format=absolute --git-common-dir)"
INSTALL_ROOT="$COMMON_DIR/motolii-supervised-runner"
ACTIVE_FILE="$INSTALL_ROOT/active.txt"
LAUNCHER="$INSTALL_ROOT/run"

usage() {
  echo "Usage: $0 activate <commit-or-ref>"
  echo "       $0 status"
  echo "       $0 path"
}

field() {
  local file="$1" key="$2"
  awk -F': ' -v key="$key" '$1 == key { count++; value=$2 } END { if (count == 1) print value }' "$file"
}

write_launcher() {
  local tmp_launcher="$1"
  cat >"$tmp_launcher" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

INSTALL_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ACTIVE_FILE="$INSTALL_ROOT/active.txt"

fail() {
  echo "motolii-supervised-runner: $*" >&2
  exit 2
}

field() {
  local key="$1"
  awk -F': ' -v key="$key" '$1 == key { count++; value=$2 } END { if (count == 1) print value }' "$ACTIVE_FILE"
}

[[ -f "$ACTIVE_FILE" && ! -L "$ACTIVE_FILE" ]] || fail "canonical runner is not activated"
runner_sha256="$(field RUNNER_SHA256)"
source_commit="$(field SOURCE_COMMIT)"
route_version="$(field ROUTE_CONTRACT_VERSION)"
loop_profile="$(field LOOP_PROFILE)"
[[ "$runner_sha256" =~ ^[0-9a-f]{64}$ ]] || fail "invalid RUNNER_SHA256"
[[ "$source_commit" =~ ^[0-9a-f]{40}$ ]] || fail "invalid SOURCE_COMMIT"
[[ "$route_version" == "2" ]] || fail "unsupported route contract: $route_version"
[[ "$loop_profile" == "grok-spark-opus" ]] || fail "unsupported loop profile: $loop_profile"

runner="$INSTALL_ROOT/bundles/$runner_sha256/delegate-cursor-supervised.sh"
[[ -f "$runner" && ! -L "$runner" ]] || fail "active runner bundle is missing"
actual_sha256="$(shasum -a 256 "$runner" | awk '{print $1}')"
[[ "$actual_sha256" == "$runner_sha256" ]] || fail "active runner bytes do not match manifest"
[[ "$#" -ge 3 ]] || fail "runner arguments are missing"
target_worktree="$2"
[[ -d "$target_worktree" ]] || fail "target worktree does not exist: $target_worktree"
primary_raw="$(git -C "$target_worktree" worktree list --porcelain | awk '/^worktree / && !found { print substr($0, 10); found=1 }')"
[[ -n "$primary_raw" && -d "$primary_raw" ]] || fail "primary worktree cannot be resolved"
primary_worktree="$(cd "$primary_raw" && pwd -P)"

exec env \
  MOTOLII_CANONICAL_RUNNER_SHA256="$runner_sha256" \
  MOTOLII_CANONICAL_RUNNER_ACTIVE_FILE="$ACTIVE_FILE" \
  MOTOLII_CANONICAL_RUNNER_SOURCE_COMMIT="$source_commit" \
  MOTOLII_RUNNER_ROOT_DIR="$primary_worktree" \
  "$runner" "$@"
EOF
  chmod 755 "$tmp_launcher"
}

case "${1:-}" in
  activate)
    [[ "$#" == 2 ]] || { usage >&2; exit 2; }
    source_commit="$(git -C "$ROOT_DIR" rev-parse --verify "$2^{commit}")"
    mkdir -p "$INSTALL_ROOT/bundles"
    runner_tmp="$(mktemp "$INSTALL_ROOT/runner.XXXXXX")"
    launcher_tmp="$(mktemp "$INSTALL_ROOT/launcher.XXXXXX")"
    active_tmp="$(mktemp "$INSTALL_ROOT/active.XXXXXX")"
    cleanup() {
      rm -f "$runner_tmp" "$launcher_tmp" "$active_tmp"
    }
    trap cleanup EXIT
    git -C "$ROOT_DIR" show "$source_commit:scripts/delegate-cursor-supervised.sh" >"$runner_tmp"
    grep -Fqx 'ROUTE_CONTRACT_VERSION="2"' "$runner_tmp" \
      || { echo "activate-supervised-runner: route contract version 2ではありません" >&2; exit 2; }
    grep -Fqx 'LOOP_PROFILE="grok-spark-opus"' "$runner_tmp" \
      || { echo "activate-supervised-runner: grok-spark-opus profileではありません" >&2; exit 2; }
    runner_sha256="$(shasum -a 256 "$runner_tmp" | awk '{print $1}')"
    bundle_dir="$INSTALL_ROOT/bundles/$runner_sha256"
    mkdir -p "$bundle_dir"
    if [[ -e "$bundle_dir/delegate-cursor-supervised.sh" ]]; then
      existing_sha256="$(shasum -a 256 "$bundle_dir/delegate-cursor-supervised.sh" | awk '{print $1}')"
      [[ "$existing_sha256" == "$runner_sha256" ]] \
        || { echo "activate-supervised-runner: immutable bundle collision" >&2; exit 2; }
    else
      chmod 755 "$runner_tmp"
      mv "$runner_tmp" "$bundle_dir/delegate-cursor-supervised.sh"
    fi
    write_launcher "$launcher_tmp"
    {
      echo "FORMAT: 1"
      echo "SOURCE_COMMIT: $source_commit"
      echo "RUNNER_SHA256: $runner_sha256"
      echo "ROUTE_CONTRACT_VERSION: 2"
      echo "LOOP_PROFILE: grok-spark-opus"
    } >"$active_tmp"
    mv -f "$launcher_tmp" "$LAUNCHER"
    mv -f "$active_tmp" "$ACTIVE_FILE"
    echo "activate-supervised-runner: activated $source_commit"
    echo "RUNNER_SHA256: $runner_sha256"
    echo "LAUNCHER: $LAUNCHER"
    ;;
  status)
    [[ "$#" == 1 ]] || { usage >&2; exit 2; }
    [[ -f "$ACTIVE_FILE" && ! -L "$ACTIVE_FILE" ]] \
      || { echo "activate-supervised-runner: not activated" >&2; exit 1; }
    cat "$ACTIVE_FILE"
    ;;
  path)
    [[ "$#" == 1 ]] || { usage >&2; exit 2; }
    [[ -x "$LAUNCHER" ]] \
      || { echo "activate-supervised-runner: launcher is not installed" >&2; exit 1; }
    printf '%s\n' "$LAUNCHER"
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
