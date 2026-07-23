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

assert_exists() {
  local path="$1"
  local label="$2"
  [[ -e "$path" ]] || fail "$label: missing $path"
}

FAKE_BIN="$TMP_ROOT/bin"
WORKTREE="$TMP_ROOT/worktree"
CALL_LOG="$TMP_ROOT/calls.log"
mkdir -p "$FAKE_BIN" "$WORKTREE"
git -C "$WORKTREE" init -q
mkdir -p "$WORKTREE/docs"
printf '%s\n' 'test authority' >"$WORKTREE/docs/authority.md"
git -C "$WORKTREE" add docs/authority.md
git -C "$WORKTREE" -c user.name=Test -c user.email=test@example.invalid \
  commit -qm initial
base_sha="$(git -C "$WORKTREE" rev-parse HEAD)"
authority_sha="$(shasum -a 256 "$WORKTREE/docs/authority.md" | awk '{print $1}')"

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
workspace=""
while [[ "$#" -gt 0 ]]; do
  if [[ "$1" == "--workspace" ]]; then workspace="$2"; shift 2; else shift; fi
done
if [[ -n "${FAKE_INSPECTION_WRITE:-}" ]]; then
  mkdir -p "$workspace/$(dirname "$FAKE_INSPECTION_WRITE")"
  printf '%s\n' reviewer-write >"$workspace/$FAKE_INSPECTION_WRITE"
fi
if [[ -n "${FAKE_CURSOR_SLEEP:-}" ]]; then sleep "$FAKE_CURSOR_SLEEP"; fi
printf '%s\n' "${FAKE_CURSOR_OUTPUT:-}"
exit "${FAKE_CURSOR_STATUS:-0}"
EOF

cat >"$FAKE_BIN/codex" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo codex >>"$FAKE_CALL_LOG"
printf 'codex-args:%s\n' "$*" >>"$FAKE_CALL_LOG"
worktree=""
while [[ "$#" -gt 0 ]]; do
  if [[ "$1" == "--cd" ]]; then worktree="$2"; shift 2; else shift; fi
done
if [[ -n "${FAKE_TERRA_WRITE:-}" ]]; then
  mkdir -p "$worktree/$(dirname "$FAKE_TERRA_WRITE")"
  printf '%s\n' terra-write >"$worktree/$FAKE_TERRA_WRITE"
fi
printf '%s\n' "${FAKE_TERRA_OUTPUT:-implementation complete}"
exit "${FAKE_TERRA_STATUS:-0}"
EOF

cat >"$FAKE_BIN/agent" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo agent >>"$FAKE_CALL_LOG"
exit 99
EOF

chmod +x "$FAKE_BIN/grok" "$FAKE_BIN/cursor-agent" "$FAKE_BIN/codex" "$FAKE_BIN/agent"

run_prepare() {
  local order_file="$1"
  shift
  : >"$CALL_LOG"
  if env -u CURSOR_AGENT -u CURSOR_AGENT_BIN -u CODEX_DELEGATED -u CODEX_AGENT_BIN \
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

order_file="$TMP_ROOT/ready-order.md"
run_prepare "$order_file" \
  FAKE_GROK_OUTPUT=$'must not be used\nORDER: STOP' \
  FAKE_CURSOR_OUTPUT=$'cursor order\nORDER: READY'
status="$RUN_STATUS"
assert_status 0 "$status" "Cursor-only order"
assert_contains "$order_file" "SUPERVISOR_BACKEND: cursor-grok" "Cursor-only order"
assert_contains "$order_file" "SUPERVISOR_MODEL: cursor-grok-4.5-high" "fixed Grok model"
assert_contains "$order_file" "IMPLEMENTER_MODEL: gpt-5.6-terra" "fixed Terra model"
assert_contains "$CALL_LOG" "cursor-agent" "explicit Cursor binary"
assert_not_contains "$CALL_LOG" "grok" "Grok Build bypass"
assert_has_fragment "$CALL_LOG" "--mode ask" "read-only order supervisor mode"
assert_not_contains "$CALL_LOG" "agent" "generic agent collision"

order_file="$TMP_ROOT/stop-order.md"
run_prepare "$order_file" \
  FAKE_CURSOR_OUTPUT=$'blocked by a real specification decision\nORDER: STOP'
status="$RUN_STATUS"
assert_status 3 "$status" "ORDER STOP preservation"
assert_contains "$order_file" "SUPERVISOR_BACKEND: cursor-grok" "ORDER STOP preservation"
assert_contains "$CALL_LOG" "cursor-agent" "ORDER STOP preservation"
assert_not_contains "$CALL_LOG" "grok" "ORDER STOP backend preservation"

order_file="$TMP_ROOT/ambiguous-order.md"
run_prepare "$order_file" \
  FAKE_CURSOR_OUTPUT=$'ORDER: STOP\nORDER: READY'
status="$RUN_STATUS"
assert_status 1 "$status" "ambiguous Cursor rejection"
[[ ! -e "$order_file" ]] || fail "ambiguous Cursor rejection: order file must not be created"

order_file="$TMP_ROOT/nonterminal-order.md"
run_prepare "$order_file" \
  FAKE_CURSOR_OUTPUT=$'ORDER: READY\ntrailing text'
status="$RUN_STATUS"
assert_status 1 "$status" "nonterminal Cursor rejection"
[[ ! -e "$order_file" ]] || fail "nonterminal Cursor rejection: order file must not be created"

order_file="$TMP_ROOT/invalid-cursor-order.md"
run_prepare "$order_file" \
  FAKE_CURSOR_OUTPUT="still no marker"
status="$RUN_STATUS"
assert_status 1 "$status" "invalid Cursor supervisor result"
[[ ! -e "$order_file" ]] || fail "invalid Cursor supervisor result: order file must not be created"
assert_has_fragment "$TMP_ROOT/stdout.log" "still no marker" "invalid Cursor output visibility"

task="execute task"
task_hash="$(printf '%s' "$task" | shasum -a 256 | awk '{print $1}')"
approved_order="$TMP_ROOT/approved-order.md"
cat >"$approved_order" <<EOF
GRAIN: CU-0A01
BASE_SHA: $base_sha
AUTHORITY: docs/authority.md SHA256:$authority_sha
ALLOWED_FILE: docs/authority.md
ORDER: READY
SUPERVISOR_BACKEND: cursor-grok
SUPERVISOR_MODEL: cursor-grok-4.5-high
IMPLEMENTER_MODEL: gpt-5.6-terra
TASK_SHA256: $task_hash
CODEX PRECHECK: APPROVED
EOF
: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CURSOR_AGENT_BIN -u CODEX_DELEGATED -u CODEX_AGENT_BIN \
    PATH="$FAKE_BIN:/usr/bin:/bin" \
    FAKE_CALL_LOG="$CALL_LOG" \
    FAKE_CURSOR_OUTPUT=$'contract defect\nVERDICT: REJECT' \
    CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
    CURSOR_SUPERVISED_TIMEOUT_SECONDS=5 \
    "$SCRIPT" execute "$WORKTREE" "$approved_order" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
  status=0
else
  status=$?
fi
assert_status 4 "$status" "VERDICT REJECT preservation"
assert_contains "$CALL_LOG" "codex" "Terra explicit Codex binary"
assert_has_fragment "$CALL_LOG" "--model gpt-5.6-terra" "Terra fixed model"
assert_has_fragment "$CALL_LOG" "--sandbox danger-full-access" "Terra isolated worktree permissions"
assert_has_fragment "$CALL_LOG" "--ask-for-approval never" "Terra noninteractive approvals"
assert_not_contains "$CALL_LOG" "grok" "Grok Build inspection bypass"
assert_not_contains "$CALL_LOG" "agent" "execute generic agent collision"
assert_exists "$approved_order.evidence/order.txt" "persistent order evidence"
assert_exists "$approved_order.evidence/implementation.txt" "persistent implementation evidence"
assert_exists "$approved_order.evidence/inspection.txt" "persistent inspection evidence"
assert_exists "$approved_order.evidence/before-inspection.status" "persistent pre-inspection status"
assert_exists "$approved_order.evidence/after-inspection.status" "persistent post-inspection status"

bad_base_order="$TMP_ROOT/bad-base-order.md"
sed "s/^BASE_SHA:.*/BASE_SHA: 0000000000000000000000000000000000000000/" \
  "$approved_order" >"$bad_base_order"
: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CURSOR_AGENT_BIN -u CODEX_DELEGATED -u CODEX_AGENT_BIN \
    PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
    "$SCRIPT" execute "$WORKTREE" "$bad_base_order" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then status=0; else status=$?; fi
assert_status 3 "$status" "BASE_SHA mismatch rejection"
assert_has_fragment "$TMP_ROOT/stderr.log" "ORDER-GATE NG: worktree HEAD != BASE_SHA" "BASE_SHA mismatch message"
assert_not_contains "$CALL_LOG" "codex" "BASE_SHA mismatch blocks Terra"

bad_authority_order="$TMP_ROOT/bad-authority-order.md"
sed "s/^AUTHORITY:.*/AUTHORITY: docs\/authority.md SHA256:0000000000000000000000000000000000000000000000000000000000000000/" \
  "$approved_order" >"$bad_authority_order"
: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CURSOR_AGENT_BIN -u CODEX_DELEGATED -u CODEX_AGENT_BIN \
    PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
    "$SCRIPT" execute "$WORKTREE" "$bad_authority_order" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then status=0; else status=$?; fi
assert_status 3 "$status" "authority hash rejection"
assert_has_fragment "$TMP_ROOT/stderr.log" "ORDER-GATE NG: authority hash mismatch: docs/authority.md" "authority mismatch message"
assert_not_contains "$CALL_LOG" "codex" "authority mismatch blocks Terra"

wait_order="$TMP_ROOT/wait-order.md"
sed 's/^GRAIN:.*/GRAIN: CU-0A02/' "$approved_order" >"$wait_order"
: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CURSOR_AGENT_BIN -u CODEX_DELEGATED -u CODEX_AGENT_BIN \
    PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
    "$SCRIPT" execute "$WORKTREE" "$wait_order" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then status=0; else status=$?; fi
assert_status 3 "$status" "WAIT grain rejection"
assert_has_fragment "$TMP_ROOT/stderr.log" "ORDER-GATE NG: CU-0A02 is not DO" "WAIT grain message"
assert_not_contains "$CALL_LOG" "codex" "WAIT grain blocks Terra"

missing_react_labels_order="$TMP_ROOT/missing-react-labels-order.md"
{
  printf '%s\n' 'REACT TASK: YES'
  cat "$approved_order"
} >"$missing_react_labels_order"
: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CURSOR_AGENT_BIN -u CODEX_DELEGATED -u CODEX_AGENT_BIN \
    PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
    "$SCRIPT" execute "$WORKTREE" "$missing_react_labels_order" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then status=0; else status=$?; fi
assert_status 3 "$status" "React label rejection"
assert_has_fragment "$TMP_ROOT/stderr.log" "ORDER-GATE NG: React guard label missing or out of order: REACT AUTHORITY" "React label message"
assert_not_contains "$CALL_LOG" "codex" "missing React labels block Terra"

: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CURSOR_AGENT_BIN -u CODEX_DELEGATED -u CODEX_AGENT_BIN \
    PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
    FAKE_TERRA_WRITE="outside.txt" \
    FAKE_CURSOR_OUTPUT=$'must not inspect\nVERDICT: ACCEPT' \
    "$SCRIPT" execute "$WORKTREE" "$approved_order" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then status=0; else status=$?; fi
assert_status 6 "$status" "unallowed untracked rejection"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG: 変更許可外path: outside.txt" "scope rejection message"
assert_not_contains "$CALL_LOG" "cursor-agent" "scope rejection blocks inspection"
rm -f "$WORKTREE/outside.txt"

: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CURSOR_AGENT_BIN -u CODEX_DELEGATED -u CODEX_AGENT_BIN \
    PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
    FAKE_INSPECTION_WRITE="reviewer-write.txt" \
    FAKE_CURSOR_OUTPUT=$'inspection changed worktree\nVERDICT: ACCEPT' \
    "$SCRIPT" execute "$WORKTREE" "$approved_order" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then status=0; else status=$?; fi
assert_status 7 "$status" "inspection mutation rejection"
assert_has_fragment "$TMP_ROOT/stderr.log" "INSPECT NG: 検収中にworktreeが変更された" "inspection mutation message"
rm -f "$WORKTREE/reviewer-write.txt"

: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CURSOR_AGENT_BIN -u CODEX_DELEGATED -u CODEX_AGENT_BIN \
    PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
    FAKE_CURSOR_SLEEP=2 \
    FAKE_CURSOR_OUTPUT=$'late inspection\nVERDICT: ACCEPT' \
    CURSOR_INSPECTION_TIMEOUT_SECONDS=1 \
    CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
    "$SCRIPT" execute "$WORKTREE" "$approved_order" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then status=0; else status=$?; fi
assert_status 1 "$status" "inspection timeout rejection"
assert_exists "$approved_order.evidence/inspection.timeout" "persistent timeout evidence"

: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CURSOR_AGENT_BIN -u CODEX_DELEGATED -u CODEX_AGENT_BIN \
    PATH="$FAKE_BIN:/usr/bin:/bin" \
    FAKE_CALL_LOG="$CALL_LOG" \
    FAKE_CURSOR_OUTPUT=$'read-only inspection complete\nVERDICT: ACCEPT' \
    CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
    CURSOR_SUPERVISED_TIMEOUT_SECONDS=5 \
    "$SCRIPT" execute "$WORKTREE" "$approved_order" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
  status=0
else
  status=$?
fi
assert_status 0 "$status" "Cursor inspection"
inspection_args="$(grep -F -- '--model cursor-grok-4.5-high' "$CALL_LOG")"
[[ -n "$inspection_args" ]] || fail "inspection supervisor model: cursor-grok args missing"
[[ "$inspection_args" != *"--mode plan"* ]] || fail "inspection shell-capable standard mode: plan mode present"
[[ "$inspection_args" == *"--force"* ]] || fail "inspection shell approval: force missing"

echo "test-delegate-cursor-supervised: all tests passed"
