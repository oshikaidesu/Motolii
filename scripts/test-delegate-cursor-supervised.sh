#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/delegate-cursor-supervised.sh"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/motolii-delegate-test.XXXXXX")"

cleanup() {
  rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

fail() {
  echo "test-delegate-cursor-supervised: $*" >&2
  exit 1
}

assert_status() {
  local expected="$1"
  local actual="$2"
  local label="$3"
  [[ "$actual" == "$expected" ]] || fail "$label: expected status $expected, got $actual"
}

assert_contains() {
  local file="$1"
  local expected="$2"
  local label="$3"
  grep -Fqx "$expected" "$file" || fail "$label: missing '$expected' in $file"
}

assert_not_contains() {
  local file="$1"
  local unexpected="$2"
  local label="$3"
  if grep -Fqx "$unexpected" "$file"; then
    fail "$label: unexpected '$unexpected' in $file"
  fi
}

assert_has_fragment() {
  local file="$1"
  local expected="$2"
  local label="$3"
  grep -Fq -- "$expected" "$file" || fail "$label: missing fragment '$expected' in $file"
}

FAKE_BIN="$TMP_ROOT/bin"
WORKTREE="$TMP_ROOT/worktree"
CALL_LOG="$TMP_ROOT/calls.log"
mkdir -p "$FAKE_BIN" "$WORKTREE"
git -C "$WORKTREE" init -q

cat >"$FAKE_BIN/grok" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo grok >>"$FAKE_CALL_LOG"
printf '%s\n' "${FAKE_GROK_OUTPUT:-}"
exit "${FAKE_GROK_STATUS:-0}"
EOF

cat >"$FAKE_BIN/cursor-agent" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo cursor-agent >>"$FAKE_CALL_LOG"
printf 'cursor-agent-args:%s\n' "$*" >>"$FAKE_CALL_LOG"
if [[ " $* " == *" --model composer-2.5 "* ]]; then
  printf '%s\n' "${FAKE_COMPOSER_OUTPUT:-implementation complete}"
else
  printf '%s\n' "${FAKE_CURSOR_OUTPUT:-}"
fi
exit "${FAKE_CURSOR_STATUS:-0}"
EOF

cat >"$FAKE_BIN/agent" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo agent >>"$FAKE_CALL_LOG"
exit 99
EOF

chmod +x "$FAKE_BIN/grok" "$FAKE_BIN/cursor-agent" "$FAKE_BIN/agent"

run_prepare() {
  local order_file="$1"
  shift
  : >"$CALL_LOG"
  if env -u CURSOR_AGENT -u CURSOR_AGENT_BIN \
      PATH="$FAKE_BIN:/usr/bin:/bin" \
      FAKE_CALL_LOG="$CALL_LOG" \
      CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
      CURSOR_SUPERVISED_TIMEOUT_SECONDS=5 \
      "$@" \
      "$SCRIPT" prepare "$WORKTREE" "$order_file" "test task" \
      >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
    RUN_STATUS=0
  else
    RUN_STATUS=$?
  fi
}

order_file="$TMP_ROOT/fallback-order.md"
run_prepare "$order_file" \
  FAKE_GROK_OUTPUT="quota response without a contract marker" \
  FAKE_CURSOR_OUTPUT=$'fallback order\nORDER: READY'
status="$RUN_STATUS"
assert_status 0 "$status" "markerless Grok fallback"
assert_contains "$order_file" "SUPERVISOR_BACKEND: cursor-grok" "markerless Grok fallback"
assert_contains "$CALL_LOG" "cursor-agent" "explicit Cursor binary"
assert_has_fragment "$CALL_LOG" "--mode ask" "read-only order supervisor mode"
assert_not_contains "$CALL_LOG" "agent" "generic agent collision"

order_file="$TMP_ROOT/stop-order.md"
run_prepare "$order_file" \
  FAKE_GROK_OUTPUT=$'blocked by a real specification decision\nORDER: STOP' \
  FAKE_CURSOR_OUTPUT=$'must not be used\nORDER: READY'
status="$RUN_STATUS"
assert_status 3 "$status" "ORDER STOP preservation"
assert_contains "$order_file" "SUPERVISOR_BACKEND: grok-build" "ORDER STOP preservation"
assert_not_contains "$CALL_LOG" "cursor-agent" "ORDER STOP preservation"

order_file="$TMP_ROOT/ambiguous-order.md"
run_prepare "$order_file" \
  FAKE_GROK_OUTPUT=$'ORDER: STOP\nORDER: READY' \
  FAKE_CURSOR_OUTPUT=$'unambiguous fallback\nORDER: READY'
status="$RUN_STATUS"
assert_status 0 "$status" "ambiguous Grok fallback"
assert_contains "$order_file" "SUPERVISOR_BACKEND: cursor-grok" "ambiguous Grok fallback"

order_file="$TMP_ROOT/nonterminal-order.md"
run_prepare "$order_file" \
  FAKE_GROK_OUTPUT=$'ORDER: READY\ntrailing text' \
  FAKE_CURSOR_OUTPUT=$'terminal fallback\nORDER: READY'
status="$RUN_STATUS"
assert_status 0 "$status" "nonterminal Grok fallback"
assert_contains "$order_file" "SUPERVISOR_BACKEND: cursor-grok" "nonterminal Grok fallback"

order_file="$TMP_ROOT/invalid-cursor-order.md"
run_prepare "$order_file" \
  FAKE_GROK_OUTPUT="no marker" \
  FAKE_CURSOR_OUTPUT="still no marker"
status="$RUN_STATUS"
assert_status 1 "$status" "invalid Cursor supervisor result"
[[ ! -e "$order_file" ]] || fail "invalid Cursor supervisor result: order file must not be created"
assert_has_fragment "$TMP_ROOT/stdout.log" "still no marker" "invalid Cursor output visibility"

task="execute task"
task_hash="$(printf '%s' "$task" | shasum -a 256 | awk '{print $1}')"
approved_order="$TMP_ROOT/approved-order.md"
cat >"$approved_order" <<EOF
ORDER: READY
SUPERVISOR_BACKEND: grok-build
TASK_SHA256: $task_hash
CODEX PRECHECK: APPROVED
EOF
: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CURSOR_AGENT_BIN \
    PATH="$FAKE_BIN:/usr/bin:/bin" \
    FAKE_CALL_LOG="$CALL_LOG" \
    FAKE_GROK_OUTPUT=$'contract defect\nVERDICT: REJECT' \
    CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
    CURSOR_SUPERVISED_TIMEOUT_SECONDS=5 \
    "$SCRIPT" execute "$WORKTREE" "$approved_order" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
  status=0
else
  status=$?
fi
assert_status 4 "$status" "VERDICT REJECT preservation"
assert_contains "$CALL_LOG" "cursor-agent" "Composer explicit Cursor binary"
assert_contains "$CALL_LOG" "grok" "Grok inspection"
assert_not_contains "$CALL_LOG" "agent" "execute generic agent collision"

: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CURSOR_AGENT_BIN \
    PATH="$FAKE_BIN:/usr/bin:/bin" \
    FAKE_CALL_LOG="$CALL_LOG" \
    FAKE_GROK_OUTPUT="inspection transport failure without verdict" \
    FAKE_CURSOR_OUTPUT=$'read-only inspection complete\nVERDICT: ACCEPT' \
    CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
    CURSOR_SUPERVISED_TIMEOUT_SECONDS=5 \
    "$SCRIPT" execute "$WORKTREE" "$approved_order" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
  status=0
else
  status=$?
fi
assert_status 0 "$status" "Cursor inspection fallback"
inspection_args="$(grep -F -- '--model cursor-grok-4.5-high-fast' "$CALL_LOG")"
[[ -n "$inspection_args" ]] || fail "inspection supervisor model: cursor-grok args missing"
[[ "$inspection_args" != *"--mode plan"* ]] || fail "inspection shell-capable standard mode: plan mode present"
[[ "$inspection_args" != *"--force"* ]] || fail "read-only inspection autonomy: force present"

echo "test-delegate-cursor-supervised: all tests passed"
