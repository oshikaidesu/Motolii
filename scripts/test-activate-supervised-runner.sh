#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
SCRIPT="$ROOT_DIR/scripts/activate-supervised-runner.sh"
set +e
RETIRED_OUTPUT="$("$SCRIPT" status 2>&1)"
RETIRED_STATUS=$?
set -e
[[ "$RETIRED_STATUS" -eq 64 ]] || { echo "test-activate-supervised-runner: expected retired exit 64, got $RETIRED_STATUS" >&2; exit 1; }
[[ "$RETIRED_OUTPUT" == *"RETIRED 2026-08-02"* ]] || { echo "test-activate-supervised-runner: retirement marker missing" >&2; exit 1; }
COMMON_DIR="$(git -C "$ROOT_DIR" rev-parse --path-format=absolute --git-common-dir)"
INSTALLED_LAUNCHER="$COMMON_DIR/motolii-supervised-runner/run"
if [[ -e "$INSTALLED_LAUNCHER" ]]; then
  [[ -f "$INSTALLED_LAUNCHER" && ! -L "$INSTALLED_LAUNCHER" ]] || { echo "test-activate-supervised-runner: installed launcher is not a regular file" >&2; exit 1; }
  grep -Fq "RETIRED 2026-08-02" "$INSTALLED_LAUNCHER" || { echo "test-activate-supervised-runner: live canonical launcher bypass remains" >&2; exit 1; }
fi
echo "test-activate-supervised-runner: RETIRED PASS"
exit 0

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/motolii-canonical-runner-test.XXXXXX")"

cleanup() {
  rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

fail() {
  echo "test-activate-supervised-runner: $*" >&2
  exit 1
}

assert_fragment() {
  local file="$1" fragment="$2" label="$3"
  grep -Fq -- "$fragment" "$file" || fail "$label: missing '$fragment'"
}

REPO="$TMP_ROOT/repo"
TARGET="$TMP_ROOT/target"
mkdir -p "$REPO/scripts"
git -C "$REPO" init -q
git -C "$REPO" config user.email test@example.com
git -C "$REPO" config user.name test
cp "$ROOT_DIR/scripts/delegate-cursor-supervised.sh" "$REPO/scripts/"
cp "$ROOT_DIR/scripts/activate-supervised-runner.sh" "$REPO/scripts/"
chmod 755 "$REPO/scripts/"*.sh
git -C "$REPO" add scripts
git -C "$REPO" commit -q -m canonical-v1
SOURCE_COMMIT="$(git -C "$REPO" rev-parse HEAD)"

"$REPO/scripts/activate-supervised-runner.sh" activate "$SOURCE_COMMIT" >"$TMP_ROOT/activate.log"
LAUNCHER="$("$REPO/scripts/activate-supervised-runner.sh" path)"
[[ -x "$LAUNCHER" ]] || fail "canonical launcher was not installed"
RUNNER_SHA256="$(awk -F': ' '$1=="RUNNER_SHA256" { print $2 }' "$REPO/.git/motolii-supervised-runner/active.txt")"
[[ "$RUNNER_SHA256" =~ ^[0-9a-f]{64}$ ]] || fail "active runner hash is invalid"

git -C "$REPO" worktree add -q -b target-branch "$TARGET" HEAD
ORDER="$TMP_ROOT/order.md"
printf 'ORDER: READY\n' >"$ORDER"
mkdir -p "$ORDER.evidence"
printf 'ATTEMPT: attempt-0001\n' >"$ORDER.evidence/checkpoint.txt"

# activate後にbranch側runnerが変わっても、launcherは固定bundleを使う。
printf '\n# branch-local drift\n' >>"$REPO/scripts/delegate-cursor-supervised.sh"
git -C "$REPO" add scripts/delegate-cursor-supervised.sh
git -C "$REPO" commit -q -m branch-drift
"$LAUNCHER" cancel "$TARGET" "$ORDER" WRONG_STAGE >"$TMP_ROOT/cancel.stdout" 2>"$TMP_ROOT/cancel.stderr"
RECEIPT="$ORDER.evidence/cancellations/cancel-0001/receipt.txt"
assert_fragment "$RECEIPT" "STATUS: CANCELLED" "canonical cancel status"
assert_fragment "$RECEIPT" "RUNNER_SHA256: $RUNNER_SHA256" "canonical receipt runner hash"
[[ ! -e "$ORDER.evidence/checkpoint.txt" ]] || fail "canonical cancel left checkpoint"

set +e
"$REPO/scripts/delegate-cursor-supervised.sh" cancel "$TARGET" "$ORDER" WRONG_STAGE \
  >"$TMP_ROOT/direct.stdout" 2>"$TMP_ROOT/direct.stderr"
DIRECT_STATUS=$?
set -e
[[ "$DIRECT_STATUS" == 2 ]] || fail "branch-local runner returned $DIRECT_STATUS"
assert_fragment "$TMP_ROOT/direct.stderr" "canonical launcher" "branch-local runner rejection"

# active bundleのbyte driftはmodelやcancel処理より前に拒否する。
ACTIVE_RUNNER="$REPO/.git/motolii-supervised-runner/bundles/$RUNNER_SHA256/delegate-cursor-supervised.sh"
printf '\n# tampered\n' >>"$ACTIVE_RUNNER"
SECOND_ORDER="$TMP_ROOT/order-2.md"
printf 'ORDER: READY\n' >"$SECOND_ORDER"
mkdir -p "$SECOND_ORDER.evidence"
set +e
"$LAUNCHER" cancel "$TARGET" "$SECOND_ORDER" WRONG_STAGE \
  >"$TMP_ROOT/tamper.stdout" 2>"$TMP_ROOT/tamper.stderr"
TAMPER_STATUS=$?
set -e
[[ "$TAMPER_STATUS" == 2 ]] || fail "tampered bundle returned $TAMPER_STATUS"
assert_fragment "$TMP_ROOT/tamper.stderr" "bytes do not match manifest" "tampered bundle rejection"

echo "test-activate-supervised-runner: PASS"
