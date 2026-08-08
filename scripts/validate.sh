#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOCAL_PROFILE_LANES=(docs rust)
cd "$ROOT_DIR"

usage() {
  cat <<'EOF'
usage:
  scripts/validate.sh --list
  scripts/validate.sh --plan local
  scripts/validate.sh local
  scripts/validate.sh <lane> [lane-argument]

lanes:
  docs policy tooling rust web-build web-contract web-visual

profiles:
EOF
  echo "  local: ${LOCAL_PROFILE_LANES[*]}"
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "validate: required command not found: $1" >&2
    exit 127
  }
}

require_no_args() {
  local lane="$1"
  shift
  if [[ $# -ne 0 ]]; then
    echo "validate: $lane lane does not accept arguments" >&2
    exit 2
  fi
}

run_lane() {
  local lane="$1"
  shift

  echo "validate: lane=$lane"
  case "$lane" in
    docs)
      require_no_args "$lane" "$@"
      "$ROOT_DIR/scripts/check-docs.sh"
      ;;
    policy)
      if [[ $# -ne 1 || -z "$1" ]]; then
        echo "validate: policy lane requires exactly one base ref" >&2
        exit 2
      fi
      if [[ "$1" == -* ]]; then
        echo "validate: policy base ref must not begin with '-': $1" >&2
        exit 2
      fi
      local base_ref="$1"
      if ! git rev-parse --verify --quiet --end-of-options "${base_ref}^{commit}" >/dev/null; then
        echo "validate: policy base ref is not a commit: $base_ref" >&2
        exit 1
      fi
      if git cat-file -e "$base_ref:crates/motolii-testkit/src/cpu_reference/mod.rs" 2>/dev/null; then
        "$ROOT_DIR/scripts/check-protected-diff.sh" "$base_ref"
      else
        echo "validate: protected paths absent on $base_ref; skip bootstrap-only live gate"
      fi
      "$ROOT_DIR/scripts/check-golden-update-policy.sh" "$base_ref"
      ;;
    tooling)
      require_no_args "$lane" "$@"
      require_command python3
      "$ROOT_DIR/scripts/check-ui-toolkit-deps.sh"
      python3 -m unittest scripts/test_check_evidence_envelope.py scripts/test_run_observed_cli.py
      "$ROOT_DIR/scripts/test-delegate-cursor-supervised.sh"
      ;;
    rust)
      require_no_args "$lane" "$@"
      require_command cargo
      cargo fmt --all --check
      cargo clippy --workspace --all-targets -- -D warnings
      cargo test --locked --workspace
      ;;
    web-build)
      require_no_args "$lane" "$@"
      require_command npm
      (
        cd "$ROOT_DIR/ui/motolii-web"
        npm ci
        npm run check:host
        npm run build:host
        npm run check:host
      )
      git diff --exit-code -- ui/motolii-web/generated-host
      generated_status="$(git status --porcelain --untracked-files=all -- ui/motolii-web/generated-host)"
      if [[ -n "$generated_status" ]]; then
        echo "validate: generated Host bundle contains uncommitted paths" >&2
        printf '%s\n' "$generated_status" >&2
        exit 1
      fi
      (
        cd "$ROOT_DIR/docs/mocks-ui"
        npm ci
        npm run build
      )
      ;;
    web-contract)
      require_no_args "$lane" "$@"
      require_command npm
      (
        cd "$ROOT_DIR/docs/mocks-ui"
        npm ci
        npm run test:reference-guard
        npm run check-reference
        npm run check-current-route
      )
      ;;
    web-visual)
      require_no_args "$lane" "$@"
      require_command npm
      (
        cd "$ROOT_DIR/docs/mocks-ui"
        npm ci
        npm run test:visual
      )
      ;;
    *)
      echo "validate: unknown lane: $lane" >&2
      usage >&2
      exit 2
      ;;
  esac
  echo "validate: lane=$lane status=PASS"
}

run_profile() {
  local profile="$1"
  case "$profile" in
    local)
      local lane
      for lane in "${LOCAL_PROFILE_LANES[@]}"; do
        run_lane "$lane"
      done
      ;;
    *)
      echo "validate: unknown profile: $profile" >&2
      usage >&2
      exit 2
      ;;
  esac
}

if [[ $# -eq 0 ]]; then
  usage >&2
  exit 2
fi

case "$1" in
  --list)
    [[ $# -eq 1 ]] || {
      usage >&2
      exit 2
    }
    usage
    ;;
  --plan)
    [[ $# -eq 2 ]] || {
      usage >&2
      exit 2
    }
    case "$2" in
      local) echo "local: ${LOCAL_PROFILE_LANES[*]}" ;;
      *)
        echo "validate: unknown profile: $2" >&2
        exit 2
        ;;
    esac
    ;;
  local)
    [[ $# -eq 1 ]] || {
      usage >&2
      exit 2
    }
    run_profile local
    ;;
  *)
    lane="$1"
    shift
    run_lane "$lane" "$@"
    ;;
esac
