#!/usr/bin/env bash
# Motolii の開発窓。asdf の Cargo cache を誤って全走査する stock dx は使わない。
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
fixed_dx="${MOTOLII_DX_BIN:-${HOME}/.local/bin/motolii-dx-0.7.10-fixed}"
expected_sha="25acf557e325b79acc4f1e4e02105cba08094c7476db3b90e1328df35591746a"

fail() {
  echo "motolii-dx: $1" >&2
  exit 1
}

check_fixed_dx() {
  [ -x "$fixed_dx" ] || fail "固定版 dx が無い: $fixed_dx"
  actual_sha="$(shasum -a 256 "$fixed_dx" | awk '{print $1}')"
  [ "$actual_sha" = "$expected_sha" ] \
    || fail "固定版 dx の checksum が違う: $actual_sha"
}

case "${1:-serve}" in
  serve)
    shift 2>/dev/null || true
    check_fixed_dx
    "$repo_root/scripts/check-build-loop.sh" \
      || fail "build-loop doctor が NG。上の原因を直してから起動する"
    cd "$repo_root/motolii"
    exec "$fixed_dx" serve --hotpatch --interactive false "$@"
    ;;
  doctor)
    check_fixed_dx
    "$fixed_dx" --version
    echo "sha256=$expected_sha"
    echo "patch=$repo_root/motolii/reference/dioxus-cli-0.7.10-asdf.patch"
    echo "rule=ui-rust-and-css-hotpatch; workspace-crate-change-restart"
    ;;
  stock)
    shift
    cd "$repo_root/motolii"
    exec dx serve --hotpatch --interactive false "$@"
    ;;
  *)
    echo "usage: scripts/motolii-dx.sh [serve|doctor|stock] [dx serve args...]" >&2
    exit 2
    ;;
esac
