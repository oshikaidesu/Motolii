#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/delegate-cursor-supervised.sh"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/motolii-opus-spark-grok-test.XXXXXX")"

cleanup() {
  rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

fail() {
  echo "test-delegate-cursor-supervised: $*" >&2
  exit 1
}

assert_status() {
  local expected="$1" actual="$2" label="$3"
  [[ "$actual" == "$expected" ]] || fail "$label: expected $expected, got $actual"
}

assert_has() {
  local file="$1" expected="$2" label="$3"
  grep -Fqx -- "$expected" "$file" || fail "$label: missing '$expected'"
}

assert_fragment() {
  local file="$1" expected="$2" label="$3"
  grep -Fq -- "$expected" "$file" || fail "$label: missing fragment '$expected'"
}

sha256_file() {
  shasum -a 256 "$1" | awk '{print $1}'
}

FAKE_BIN="$TMP_ROOT/bin"
CALL_LOG="$TMP_ROOT/calls.log"
mkdir -p "$FAKE_BIN"

cat >"$FAKE_BIN/claude" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo "claude:$*" >>"$FAKE_CALL_LOG"
printf '%s\n' "${FAKE_OPUS_OUTPUT:-ORDER: STOP}"
EOF

cat >"$FAKE_BIN/codex" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo "codex:$*" >>"$FAKE_CALL_LOG"
if [[ -n "${FAKE_SPARK_HOOK:-}" ]]; then
  bash "$FAKE_SPARK_HOOK"
fi
printf '%s\n' "${FAKE_SPARK_OUTPUT:-implementation complete}"
EOF

cat >"$FAKE_BIN/cursor-agent" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo "cursor:$*" >>"$FAKE_CALL_LOG"
if [[ -n "${FAKE_GROK_HOOK:-}" ]]; then
  bash "$FAKE_GROK_HOOK"
fi
printf '%s\n' "${FAKE_GROK_OUTPUT:-VERDICT: ACCEPT}"
EOF
chmod +x "$FAKE_BIN/claude" "$FAKE_BIN/codex" "$FAKE_BIN/cursor-agent"

WT="$TMP_ROOT/worktree"
mkdir -p "$WT/docs"
git -C "$WT" init -q
git -C "$WT" config user.email test@example.com
git -C "$WT" config user.name test
git -C "$WT" checkout -q -b managed-grain
printf 'authority\n' >"$WT/AGENTS.md"
printf 'before\n' >"$WT/src.txt"
grep -Fqx '## 現在の並列レーン' "$ROOT_DIR/docs/implementation-ledger.md" \
  || fail "repository ledger current-lane heading is missing"
grep -Fqx '## 発注依存証跡' "$ROOT_DIR/docs/implementation-ledger.md" \
  || fail "repository ledger dependency-evidence heading is missing"

cat >"$WT/docs/implementation-ledger.md" <<'EOF'
# ledger

## 現在の並列レーン

| lane | 現在粒 | Phase | 状態 | Issue | 依存確認 | 完了後 |
|---|---|---|---|---|---|---|
| PRODUCT | GRAIN-1 | M3 | `DO` | — | DEP-1 | next |
| SPEC | SPEC-1 | Vism | `DO / SPEC` | — | DEP-1 | later |

### 非dispatch補助表

| 優先 | ID | Phase | 状態 | Issue | 依存確認 | 完了後 |
|---|---|---|---|---|---|---|
| 1 | GRAIN-1 | History | `WAIT` | — | blocked | later |

## 発注依存証跡

| ID | 状態 | 完了証拠 |
|---|---|---|
| DEP-1 | `DONE` | fixture |
EOF
git -C "$WT" add -A
git -C "$WT" commit -q -m init

BASE_SHA="$(git -C "$WT" rev-parse HEAD)"
AUTH_HASH="$(sha256_file "$WT/AGENTS.md")"
TASK="managed grain implementation"
TASK_HASH="$(printf '%s' "$TASK" | shasum -a 256 | awk '{print $1}')"
ORDER="$TMP_ROOT/order.md"

OPUS_READY=$(cat <<EOF
Objective: update the allowed fixture.
GRAIN: GRAIN-1
BASE_REF: refs/heads/managed-grain
BASE_SHA: $BASE_SHA
DEPENDENCY: DEP-1
AUTHORITY: AGENTS.md SHA256:$AUTH_HASH
ALLOWED_FILE: src.txt
Non-goal: no adjacent edits.
STOP: authority conflict.
Test: git diff --check.
ORDER: READY
EOF
)

run_script() {
  : >"$CALL_LOG"
  set +e
  env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
    PATH="$FAKE_BIN:/usr/bin:/bin" \
    FAKE_CALL_LOG="$CALL_LOG" \
    CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
    CURSOR_TERMINATION_GRACE_SECONDS=1 \
    "$SCRIPT" "$@" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
  RUN_STATUS=$?
  set -e
}

run_script prepare "$WT" "$ORDER" "$TASK"
assert_status 3 "$RUN_STATUS" "Opus STOP fails closed"
assert_fragment "$CALL_LOG" "claude:-p --model claude-opus-5" "Opus is the order manager"

# Complete READY orderで正規metadataが追加されることを確認する。
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  FAKE_OPUS_OUTPUT="$OPUS_READY" \
  "$SCRIPT" prepare "$WT" "$ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 0 "$RUN_STATUS" "Opus READY prepare"
assert_has "$ORDER" "LOOP_PROFILE: opus-spark-grok" "loop profile"
assert_has "$ORDER" "ORDER_MANAGER_MODEL: claude-opus-5" "manager model"
assert_has "$ORDER" "IMPLEMENTER_MODEL: gpt-5.3-codex-spark" "Spark model"
assert_has "$ORDER" "REVIEW_MODEL: cursor-grok-4.5-high" "Grok model"
assert_has "$ORDER" "TASK_SHA256: $TASK_HASH" "task binding"
printf 'CODEX PRECHECK: APPROVED\n' >>"$ORDER"

# 複合状態をDOへ緩めず、依存を散文や別表から推測しない。
SPEC_ORDER="$TMP_ROOT/spec-order.md"
sed 's/^GRAIN: GRAIN-1$/GRAIN: SPEC-1/' "$ORDER" >"$SPEC_ORDER"
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  "$SCRIPT" execute "$WT" "$SPEC_ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 3 "$RUN_STATUS" "DO / SPEC is not dispatchable"
grep -Fq -- "SPEC-1 is DO / SPEC; dispatch is forbidden" "$TMP_ROOT/stderr.log" \
  || fail "compound state rejection reason is missing"
[[ ! -s "$CALL_LOG" ]] || fail "compound state must fail before model invocation"

MISSING_DEP_ORDER="$TMP_ROOT/missing-dependency-order.md"
sed 's/^DEPENDENCY: DEP-1$/DEPENDENCY: MISSING-DEP/' "$ORDER" >"$MISSING_DEP_ORDER"
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  "$SCRIPT" execute "$WT" "$MISSING_DEP_ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 3 "$RUN_STATUS" "dependency absent from evidence table"
grep -Fq -- "dependency MISSING-DEP not found in dependency-evidence ledger" "$TMP_ROOT/stderr.log" \
  || fail "missing dependency rejection reason is missing"
[[ ! -s "$CALL_LOG" ]] || fail "missing dependency must fail before model invocation"

SPARK_HOOK="$TMP_ROOT/spark-hook.sh"
cat >"$SPARK_HOOK" <<EOF
#!/usr/bin/env bash
printf 'after\n' >>"$WT/src.txt"
EOF
chmod +x "$SPARK_HOOK"

: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  FAKE_SPARK_HOOK="$SPARK_HOOK" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" execute "$WT" "$ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 0 "$RUN_STATUS" "Spark then Grok happy path"
grep -Fq -- "--model gpt-5.3-codex-spark" "$CALL_LOG" || fail "Spark model was not invoked"
grep -Fq -- "--model cursor-grok-4.5-high" "$CALL_LOG" || fail "Grok model was not invoked"
if grep -Fq -- "claude:" "$CALL_LOG"; then
  fail "execute must not rerun Opus or Fable"
fi
codex_line="$(grep -n '^codex:' "$CALL_LOG" | cut -d: -f1)"
grok_line="$(grep -n '^cursor:' "$CALL_LOG" | cut -d: -f1)"
[[ "$codex_line" -lt "$grok_line" ]] || fail "Spark must run before Grok"

GROK_HOOK="$TMP_ROOT/grok-hook.sh"
cat >"$GROK_HOOK" <<EOF
#!/usr/bin/env bash
printf 'reviewer mutation\n' >>"$WT/src.txt"
EOF
chmod +x "$GROK_HOOK"
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  FAKE_GROK_HOOK="$GROK_HOOK" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" inspect "$WT" "$ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 8 "$RUN_STATUS" "read-only Grok mutation rejected"
grep -Fq -- "fingerprint changed during read-only inspection" "$TMP_ROOT/stderr.log" \
  || fail "Grok mutation did not report fingerprint failure"

STALE_ORDER="$TMP_ROOT/stale-order.md"
sed 's/^IMPLEMENTER_MODEL:.*/IMPLEMENTER_MODEL: gpt-5.6-terra/' "$ORDER" >"$STALE_ORDER"
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  "$SCRIPT" execute "$WT" "$STALE_ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 3 "$RUN_STATUS" "archived routing rejected"
[[ ! -s "$CALL_LOG" ]] || fail "stale routing must fail before model invocation"

: >"$CALL_LOG"
set +e
env PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" CLAUDE_DELEGATED=1 \
  "$SCRIPT" prepare "$WT" "$ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 2 "$RUN_STATUS" "recursive Claude dispatch rejected"
[[ ! -s "$CALL_LOG" ]] || fail "recursive dispatch must not invoke a model"

echo "test-delegate-cursor-supervised: PASS"
