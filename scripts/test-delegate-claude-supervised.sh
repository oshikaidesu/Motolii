#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/delegate-claude-supervised.sh"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/motolii-delegate-claude-test.XXXXXX")"

cleanup() {
  rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

fail() {
  echo "test-delegate-claude-supervised: $*" >&2
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

assert_has_fragment() {
  local file="$1"
  local expected="$2"
  local label="$3"
  grep -Fq -- "$expected" "$file" || fail "$label: missing fragment '$expected' in $file"
}

assert_no_claude_calls() {
  local label="$1"
  if [[ -s "$CALL_LOG" ]]; then
    fail "$label: expected zero Claude invocations, got: $(cat "$CALL_LOG")"
  fi
}

sha256_file() {
  shasum -a 256 "$1" | awk '{print $1}'
}

git_init_repo() {
  local dir="$1"
  git -C "$dir" init -q
  git -C "$dir" config user.email test@example.com
  git -C "$dir" config user.name "test"
}

# 呼び出し側がBASE_REF/BASE_SHA/authority hashを組み立てられるよう、戻り値をWT_*globalへ残す
make_worktree() {
  local name="$1" ledger_content="$2"
  local dir="$TMP_ROOT/wt-$name"
  rm -rf "$dir"
  mkdir -p "$dir/docs" "$dir/scripts"
  git_init_repo "$dir"
  git -C "$dir" checkout -q -b grain-branch
  printf 'authority content\n' >"$dir/AGENTS.md"
  printf '%s\n' "$ledger_content" >"$dir/docs/implementation-ledger.md"
  printf '#!/usr/bin/env bash\necho dummy\n' >"$dir/scripts/delegate-claude-supervised.sh"
  git -C "$dir" add -A
  git -C "$dir" commit -q -m init
  WT_DIR="$dir"
  WT_BRANCH="grain-branch"
  WT_BASE_SHA="$(git -C "$dir" rev-parse HEAD)"
  WT_AGENTS_HASH="$(sha256_file "$dir/AGENTS.md")"
  WT_LEDGER_HASH="$(sha256_file "$dir/docs/implementation-ledger.md")"
}

valid_ledger() {
  cat <<'EOF'
# ledger

## 現在選択中の1件

| 優先 | ID | Phase | 状態 | Issue | 依存確認 | 完了後 |
|---|---|---|---|---|---|---|
| 1 | U0e-2R | M3 | `DONE` | — | dep row | next |
| 2 | GR-D1 | M3 guard | `DO` | — | grain row | next |

## 次にIssue化するもの

| 1 | other | M2 | `WAIT` | — | cond | out |
EOF
}

ledger_grain_absent() {
  cat <<'EOF'
# ledger

## 現在選択中の1件

| 優先 | ID | Phase | 状態 | Issue | 依存確認 | 完了後 |
|---|---|---|---|---|---|---|
| 1 | U0e-2R | M3 | `DONE` | — | dep row | next |

## 次にIssue化するもの

| 1 | other | M2 | `WAIT` | — | cond | out |
EOF
}

ledger_grain_wait() {
  cat <<'EOF'
# ledger

## 現在選択中の1件

| 優先 | ID | Phase | 状態 | Issue | 依存確認 | 完了後 |
|---|---|---|---|---|---|---|
| 1 | U0e-2R | M3 | `DONE` | — | dep row | next |
| 2 | GR-D1 | M3 guard | `WAIT` | — | grain row | next |

## 次にIssue化するもの

| 1 | other | M2 | `WAIT` | — | cond | out |
EOF
}

ledger_grain_ambiguous() {
  cat <<'EOF'
# ledger

## 現在選択中の1件

| 優先 | ID | Phase | 状態 | Issue | 依存確認 | 完了後 |
|---|---|---|---|---|---|---|
| 1 | U0e-2R | M3 | `DONE` | — | dep row | next |
| 2 | GR-D1 | M3 guard | `DO` | — | grain row | next |
| 3 | GR-D1 | M3 guard | `WAIT` | — | grain row dup | next |

## 次にIssue化するもの

| 1 | other | M2 | `WAIT` | — | cond | out |
EOF
}

ledger_dep_absent() {
  cat <<'EOF'
# ledger

## 現在選択中の1件

| 優先 | ID | Phase | 状態 | Issue | 依存確認 | 完了後 |
|---|---|---|---|---|---|---|
| 2 | GR-D1 | M3 guard | `DO` | — | grain row | next |

## 次にIssue化するもの

| 1 | other | M2 | `WAIT` | — | cond | out |
EOF
}

ledger_dep_wait() {
  cat <<'EOF'
# ledger

## 現在選択中の1件

| 優先 | ID | Phase | 状態 | Issue | 依存確認 | 完了後 |
|---|---|---|---|---|---|---|
| 1 | U0e-2R | M3 | `WAIT` | — | dep row | next |
| 2 | GR-D1 | M3 guard | `DO` | — | grain row | next |

## 次にIssue化するもの

| 1 | other | M2 | `WAIT` | — | cond | out |
EOF
}

ledger_dep_ambiguous() {
  cat <<'EOF'
# ledger

## 現在選択中の1件

| 優先 | ID | Phase | 状態 | Issue | 依存確認 | 完了後 |
|---|---|---|---|---|---|---|
| 1 | U0e-2R | M3 | `DONE` | — | dep row | next |
| 1b | U0e-2R | M3 | `WAIT` | — | dep row dup | next |
| 2 | GR-D1 | M3 guard | `DO` | — | grain row | next |

## 次にIssue化するもの

| 1 | other | M2 | `WAIT` | — | cond | out |
EOF
}

FAKE_BIN="$TMP_ROOT/bin"
CALL_LOG="$TMP_ROOT/calls.log"
mkdir -p "$FAKE_BIN"

cat >"$FAKE_BIN/claude" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo claude >>"$FAKE_CALL_LOG"
printf 'claude-args:%s\n' "$*" >>"$FAKE_CALL_LOG"
if [[ " $* " == *" --model claude-sonnet-5 "* ]]; then
  printf '%s\n' "${FAKE_SONNET_OUTPUT:-implementation complete}"
  exit "${FAKE_SONNET_STATUS:-0}"
else
  printf '%s\n' "${FAKE_OPUS_OUTPUT:-}"
  exit "${FAKE_OPUS_STATUS:-0}"
fi
EOF
chmod +x "$FAKE_BIN/claude"

task="GR-D1 dispatch gate execution"
task_hash="$(printf '%s' "$task" | shasum -a 256 | awk '{print $1}')"

run_execute() {
  local worktree="$1" order_file="$2"
  shift 2
  : >"$CALL_LOG"
  if env -u CLAUDE_DELEGATED \
      PATH="$FAKE_BIN:/usr/bin:/bin" \
      FAKE_CALL_LOG="$CALL_LOG" \
      CLAUDE_SUPERVISED_HEARTBEAT_SECONDS=1 \
      CLAUDE_SUPERVISED_TIMEOUT_SECONDS=5 \
      CLAUDE_IMPLEMENTER_TIMEOUT_SECONDS=5 \
      "$@" \
      "$SCRIPT" execute "$worktree" "$order_file" "$task" \
      >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
    RUN_STATUS=0
  else
    RUN_STATUS=$?
  fi
}

run_prepare() {
  local order_file="$1"
  shift 1
  : >"$CALL_LOG"
  if env -u CLAUDE_DELEGATED \
      PATH="$FAKE_BIN:/usr/bin:/bin" \
      FAKE_CALL_LOG="$CALL_LOG" \
      CLAUDE_SUPERVISED_HEARTBEAT_SECONDS=1 \
      CLAUDE_SUPERVISED_TIMEOUT_SECONDS=5 \
      "$@" \
      "$SCRIPT" prepare "$WT_DIR" "$order_file" "prepare task" \
      >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
    RUN_STATUS=0
  else
    RUN_STATUS=$?
  fi
}

assert_gate_reject() {
  local label="$1" worktree="$2" order_file="$3" expected_fragment="$4"
  run_execute "$worktree" "$order_file"
  [[ "$RUN_STATUS" -ne 0 ]] || fail "$label: expected nonzero status"
  assert_has_fragment "$TMP_ROOT/stderr.log" "ORDER-GATE NG:" "$label (stable diagnostic prefix)"
  assert_has_fragment "$TMP_ROOT/stderr.log" "$expected_fragment" "$label (diagnostic detail)"
  assert_no_claude_calls "$label"
}

make_worktree "valid" "$(valid_ledger)"
WT_VALID_DIR="$WT_DIR"
WT_VALID_BRANCH="$WT_BRANCH"
WT_VALID_BASE_SHA="$WT_BASE_SHA"
WT_VALID_AGENTS_HASH="$WT_AGENTS_HASH"
WT_VALID_LEDGER_HASH="$WT_LEDGER_HASH"

# BASE_REFが正本と別commitへ解決するケースを再現するための別branch
git -C "$WT_VALID_DIR" branch other-branch
git -C "$WT_VALID_DIR" checkout -q other-branch
printf 'authority content changed\n' >"$WT_VALID_DIR/AGENTS.md"
git -C "$WT_VALID_DIR" commit -q -am other
OTHER_BRANCH_SHA="$(git -C "$WT_VALID_DIR" rev-parse HEAD)"
git -C "$WT_VALID_DIR" checkout -q "$WT_VALID_BRANCH"

valid_order() {
  cat <<EOF
GRAIN: GR-D1
BASE_REF: refs/heads/$WT_VALID_BRANCH
BASE_SHA: $WT_VALID_BASE_SHA
DEPENDENCY: U0e-2R
AUTHORITY: AGENTS.md SHA256:$WT_VALID_AGENTS_HASH
AUTHORITY: docs/implementation-ledger.md SHA256:$WT_VALID_LEDGER_HASH
ALLOWED_FILE: scripts/delegate-claude-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: claude-code
SUPERVISOR_MODEL: claude-opus-4-8
IMPLEMENTER_MODEL: claude-sonnet-5
TASK_SHA256: $task_hash
CODEX PRECHECK: APPROVED
EOF
}

write_order() {
  local file="$1"
  cat >"$file"
}

order_file="$TMP_ROOT/order-base-ref-missing.md"
valid_order | grep -v '^BASE_REF: ' | write_order "$order_file"
assert_gate_reject "BASE_REF missing" "$WT_VALID_DIR" "$order_file" "missing BASE_REF"

order_file="$TMP_ROOT/order-base-ref-shorthand.md"
valid_order | sed -E "s#^BASE_REF: .*#BASE_REF: $WT_VALID_BRANCH#" | write_order "$order_file"
assert_gate_reject "BASE_REF shorthand" "$WT_VALID_DIR" "$order_file" "BASE_REF malformed"

order_file="$TMP_ROOT/order-base-ref-tag.md"
valid_order | sed -E "s#^BASE_REF: .*#BASE_REF: refs/tags/v1#" | write_order "$order_file"
assert_gate_reject "BASE_REF tag" "$WT_VALID_DIR" "$order_file" "BASE_REF malformed"

order_file="$TMP_ROOT/order-base-ref-remote.md"
valid_order | sed -E "s#^BASE_REF: .*#BASE_REF: refs/remotes/origin/$WT_VALID_BRANCH#" | write_order "$order_file"
assert_gate_reject "BASE_REF remote" "$WT_VALID_DIR" "$order_file" "BASE_REF malformed"

order_file="$TMP_ROOT/order-base-ref-traversal.md"
valid_order | sed -E "s#^BASE_REF: .*#BASE_REF: refs/heads/../etc#" | write_order "$order_file"
assert_gate_reject "BASE_REF traversal" "$WT_VALID_DIR" "$order_file" "BASE_REF malformed"

order_file="$TMP_ROOT/order-base-ref-duplicate.md"
{ valid_order | sed -n '1p'; echo "BASE_REF: refs/heads/$WT_VALID_BRANCH"; valid_order | sed -n '2,$p'; } | write_order "$order_file"
assert_gate_reject "BASE_REF duplicate" "$WT_VALID_DIR" "$order_file" "duplicate BASE_REF"

order_file="$TMP_ROOT/order-base-ref-valid-plus-malformed.md"
{ valid_order | sed -n '1p'; echo "BASE_REF: refs/heads/$WT_VALID_BRANCH extra"; valid_order | sed -n '2,$p'; } | write_order "$order_file"
assert_gate_reject "BASE_REF valid line plus malformed line" "$WT_VALID_DIR" "$order_file" "BASE_REF malformed"

order_file="$TMP_ROOT/order-base-ref-unresolved.md"
valid_order | sed -E "s#^BASE_REF: .*#BASE_REF: refs/heads/does-not-exist#" | write_order "$order_file"
assert_gate_reject "BASE_REF unresolved" "$WT_VALID_DIR" "$order_file" "BASE_REF does not resolve:"

order_file="$TMP_ROOT/order-base-ref-elsewhere.md"
valid_order | sed -E "s#^BASE_REF: .*#BASE_REF: refs/heads/other-branch#" | write_order "$order_file"
assert_gate_reject "BASE_REF resolves to another commit" "$WT_VALID_DIR" "$order_file" "BASE_REF does not resolve to BASE_SHA"

order_file="$TMP_ROOT/order-base-sha-mismatch.md"
valid_order | sed -E "s#^BASE_SHA: .*#BASE_SHA: $OTHER_BRANCH_SHA#" | write_order "$order_file"
assert_gate_reject "BASE_SHA mismatch" "$WT_VALID_DIR" "$order_file" "BASE_REF does not resolve to BASE_SHA"

order_file="$TMP_ROOT/order-base-sha-malformed.md"
valid_order | sed -E "s#^BASE_SHA: .*#BASE_SHA: deadbeef#" | write_order "$order_file"
assert_gate_reject "BASE_SHA malformed" "$WT_VALID_DIR" "$order_file" "BASE_SHA malformed"

order_file="$TMP_ROOT/order-base-sha-duplicate.md"
{ valid_order | sed -n '1,2p'; echo "BASE_SHA: $WT_VALID_BASE_SHA"; valid_order | sed -n '3,$p'; } | write_order "$order_file"
assert_gate_reject "BASE_SHA duplicate" "$WT_VALID_DIR" "$order_file" "duplicate BASE_SHA"

# BASE_REF/BASE_SHAは解決に成功するが、worktree HEADだけが先行しているケースを再現する
git -C "$WT_VALID_DIR" checkout -q -b head-drift-branch
printf 'unrelated\n' >"$WT_VALID_DIR/scripts/other.txt"
git -C "$WT_VALID_DIR" add -A
git -C "$WT_VALID_DIR" commit -q -m drift
order_file="$TMP_ROOT/order-head-mismatch.md"
valid_order | write_order "$order_file"
assert_gate_reject "worktree HEAD diverges from BASE_SHA" "$WT_VALID_DIR" "$order_file" "worktree HEAD != BASE_SHA"
git -C "$WT_VALID_DIR" checkout -q "$WT_VALID_BRANCH"
git -C "$WT_VALID_DIR" branch -D head-drift-branch

order_file="$TMP_ROOT/order-authority-missing.md"
valid_order | grep -v '^AUTHORITY: ' | write_order "$order_file"
assert_gate_reject "AUTHORITY missing" "$WT_VALID_DIR" "$order_file" "missing AUTHORITY"

order_file="$TMP_ROOT/order-authority-malformed.md"
valid_order | sed -E "s#^AUTHORITY: AGENTS.md SHA256:.*#AUTHORITY: AGENTS.md nothash#" | write_order "$order_file"
assert_gate_reject "AUTHORITY malformed" "$WT_VALID_DIR" "$order_file" "AUTHORITY malformed"

order_file="$TMP_ROOT/order-authority-absolute.md"
valid_order | sed -E "s#^AUTHORITY: AGENTS.md SHA256:(.*)#AUTHORITY: /etc/AGENTS.md SHA256:\1#" | write_order "$order_file"
assert_gate_reject "AUTHORITY absolute path" "$WT_VALID_DIR" "$order_file" "AUTHORITY absolute path"

order_file="$TMP_ROOT/order-authority-traversal.md"
valid_order | sed -E "s#^AUTHORITY: AGENTS.md SHA256:(.*)#AUTHORITY: ../AGENTS.md SHA256:\1#" | write_order "$order_file"
assert_gate_reject "AUTHORITY path traversal" "$WT_VALID_DIR" "$order_file" "AUTHORITY path traversal"

order_file="$TMP_ROOT/order-authority-missing-file.md"
valid_order | sed -E "s#^AUTHORITY: AGENTS.md SHA256:(.*)#AUTHORITY: docs/does-not-exist.md SHA256:\1#" | write_order "$order_file"
assert_gate_reject "AUTHORITY missing file" "$WT_VALID_DIR" "$order_file" "AUTHORITY file missing"

order_file="$TMP_ROOT/order-authority-hash-mismatch.md"
zero_hash="$(printf '0%.0s' {1..64})"
valid_order | awk -v h="$zero_hash" '{ if ($0 ~ /^AUTHORITY: AGENTS.md SHA256:/) print "AUTHORITY: AGENTS.md SHA256:" h; else print }' | write_order "$order_file"
assert_gate_reject "authority hash mismatch" "$WT_VALID_DIR" "$order_file" "authority hash mismatch: AGENTS.md"

cp -R "$WT_VALID_DIR" "$TMP_ROOT/wt-authority-symlink"
ln -s /etc/passwd "$TMP_ROOT/wt-authority-symlink/authority-link.md"
symlink_hash="$(printf 'irrelevant' | shasum -a 256 | awk '{print $1}')"
order_file="$TMP_ROOT/order-authority-symlink.md"
{ valid_order | sed -n '1,6p'; echo "AUTHORITY: authority-link.md SHA256:$symlink_hash"; valid_order | sed -n '7,$p'; } | write_order "$order_file"
assert_gate_reject "AUTHORITY symlink" "$TMP_ROOT/wt-authority-symlink" "$order_file" "AUTHORITY path is a symlink"

# 最終componentではなく中間directoryがsymlinkでworktree外へ逃げるケースを独立に再現する
mkdir -p "$TMP_ROOT/outside-authority-target"
printf 'outside bytes\n' >"$TMP_ROOT/outside-authority-target/file.md"
cp -R "$WT_VALID_DIR" "$TMP_ROOT/wt-authority-symlink-component"
ln -s "$TMP_ROOT/outside-authority-target" "$TMP_ROOT/wt-authority-symlink-component/linkdir"
outside_hash="$(sha256_file "$TMP_ROOT/outside-authority-target/file.md")"
order_file="$TMP_ROOT/order-authority-symlink-component.md"
{ valid_order | sed -n '1,6p'; echo "AUTHORITY: linkdir/file.md SHA256:$outside_hash"; valid_order | sed -n '7,$p'; } | write_order "$order_file"
assert_gate_reject "AUTHORITY symlink intermediate component" "$TMP_ROOT/wt-authority-symlink-component" "$order_file" "AUTHORITY path is a symlink"

order_file="$TMP_ROOT/order-allowed-missing.md"
valid_order | grep -v '^ALLOWED_FILE: ' | write_order "$order_file"
assert_gate_reject "ALLOWED_FILE missing" "$WT_VALID_DIR" "$order_file" "missing ALLOWED_FILE"

order_file="$TMP_ROOT/order-allowed-absolute.md"
valid_order | sed -E "s#^ALLOWED_FILE: .*#ALLOWED_FILE: /etc/passwd#" | write_order "$order_file"
assert_gate_reject "ALLOWED_FILE absolute" "$WT_VALID_DIR" "$order_file" "ALLOWED_FILE absolute path"

order_file="$TMP_ROOT/order-allowed-traversal.md"
valid_order | sed -E "s#^ALLOWED_FILE: .*#ALLOWED_FILE: ../outside.txt#" | write_order "$order_file"
assert_gate_reject "ALLOWED_FILE traversal" "$WT_VALID_DIR" "$order_file" "ALLOWED_FILE path traversal"

order_file="$TMP_ROOT/order-allowed-empty.md"
valid_order | sed -E "s#^ALLOWED_FILE: .*#ALLOWED_FILE: #" | write_order "$order_file"
assert_gate_reject "ALLOWED_FILE empty" "$WT_VALID_DIR" "$order_file" "ALLOWED_FILE empty"

order_file="$TMP_ROOT/order-allowed-multi-token-space.md"
valid_order | sed -E "s#^ALLOWED_FILE: .*#ALLOWED_FILE: safe.sh /etc/passwd#" | write_order "$order_file"
assert_gate_reject "ALLOWED_FILE space-separated multiple tokens" "$WT_VALID_DIR" "$order_file" "ALLOWED_FILE malformed"

order_file="$TMP_ROOT/order-allowed-multi-token-tab.md"
tab_char="$(printf '\t')"
valid_order | awk -v tab="$tab_char" '{ if ($0 ~ /^ALLOWED_FILE: /) print "ALLOWED_FILE: safe.sh" tab "docs/mocks-ui/App.tsx"; else print }' | write_order "$order_file"
assert_gate_reject "ALLOWED_FILE tab-separated multiple tokens" "$WT_VALID_DIR" "$order_file" "ALLOWED_FILE malformed"

order_file="$TMP_ROOT/order-allowed-two-paths.md"
valid_order | sed -E "s#^ALLOWED_FILE: .*#ALLOWED_FILE: safe.sh docs/mocks-ui/App.tsx#" | write_order "$order_file"
assert_gate_reject "ALLOWED_FILE two space-separated paths" "$WT_VALID_DIR" "$order_file" "ALLOWED_FILE malformed"

order_file="$TMP_ROOT/order-allowed-valid-plus-malformed.md"
valid_order | awk '{ print; if ($0 ~ /^ALLOWED_FILE: /) print "ALLOWED_FILE: safe.sh docs/mocks-ui/App.tsx" }' | write_order "$order_file"
assert_gate_reject "ALLOWED_FILE valid line plus malformed line" "$WT_VALID_DIR" "$order_file" "ALLOWED_FILE malformed"

make_worktree "grain-absent" "$(ledger_grain_absent)"
order_file="$TMP_ROOT/order-grain-absent.md"
cat >"$order_file" <<EOF
GRAIN: GR-D1
BASE_REF: refs/heads/$WT_BRANCH
BASE_SHA: $WT_BASE_SHA
DEPENDENCY: U0e-2R
AUTHORITY: AGENTS.md SHA256:$WT_AGENTS_HASH
AUTHORITY: docs/implementation-ledger.md SHA256:$WT_LEDGER_HASH
ALLOWED_FILE: scripts/delegate-claude-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: claude-code
SUPERVISOR_MODEL: claude-opus-4-8
IMPLEMENTER_MODEL: claude-sonnet-5
TASK_SHA256: $task_hash
CODEX PRECHECK: APPROVED
EOF
assert_gate_reject "grain absent" "$WT_DIR" "$order_file" "not found in selected-work ledger"

make_worktree "grain-wait" "$(ledger_grain_wait)"
order_file="$TMP_ROOT/order-grain-wait.md"
cat >"$order_file" <<EOF
GRAIN: GR-D1
BASE_REF: refs/heads/$WT_BRANCH
BASE_SHA: $WT_BASE_SHA
DEPENDENCY: U0e-2R
AUTHORITY: AGENTS.md SHA256:$WT_AGENTS_HASH
AUTHORITY: docs/implementation-ledger.md SHA256:$WT_LEDGER_HASH
ALLOWED_FILE: scripts/delegate-claude-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: claude-code
SUPERVISOR_MODEL: claude-opus-4-8
IMPLEMENTER_MODEL: claude-sonnet-5
TASK_SHA256: $task_hash
CODEX PRECHECK: APPROVED
EOF
assert_gate_reject "grain WAIT" "$WT_DIR" "$order_file" "GR-D1 is WAIT; dispatch is forbidden"

make_worktree "grain-ambiguous" "$(ledger_grain_ambiguous)"
order_file="$TMP_ROOT/order-grain-ambiguous.md"
cat >"$order_file" <<EOF
GRAIN: GR-D1
BASE_REF: refs/heads/$WT_BRANCH
BASE_SHA: $WT_BASE_SHA
DEPENDENCY: U0e-2R
AUTHORITY: AGENTS.md SHA256:$WT_AGENTS_HASH
AUTHORITY: docs/implementation-ledger.md SHA256:$WT_LEDGER_HASH
ALLOWED_FILE: scripts/delegate-claude-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: claude-code
SUPERVISOR_MODEL: claude-opus-4-8
IMPLEMENTER_MODEL: claude-sonnet-5
TASK_SHA256: $task_hash
CODEX PRECHECK: APPROVED
EOF
assert_gate_reject "duplicate/ambiguous grain row" "$WT_DIR" "$order_file" "ambiguous selected-work ledger rows"

make_worktree "dep-absent" "$(ledger_dep_absent)"
order_file="$TMP_ROOT/order-dep-absent.md"
cat >"$order_file" <<EOF
GRAIN: GR-D1
BASE_REF: refs/heads/$WT_BRANCH
BASE_SHA: $WT_BASE_SHA
DEPENDENCY: U0e-2R
AUTHORITY: AGENTS.md SHA256:$WT_AGENTS_HASH
AUTHORITY: docs/implementation-ledger.md SHA256:$WT_LEDGER_HASH
ALLOWED_FILE: scripts/delegate-claude-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: claude-code
SUPERVISOR_MODEL: claude-opus-4-8
IMPLEMENTER_MODEL: claude-sonnet-5
TASK_SHA256: $task_hash
CODEX PRECHECK: APPROVED
EOF
assert_gate_reject "dependency absent" "$WT_DIR" "$order_file" "dependency U0e-2R not found in selected-work ledger"

make_worktree "dep-wait" "$(ledger_dep_wait)"
order_file="$TMP_ROOT/order-dep-wait.md"
cat >"$order_file" <<EOF
GRAIN: GR-D1
BASE_REF: refs/heads/$WT_BRANCH
BASE_SHA: $WT_BASE_SHA
DEPENDENCY: U0e-2R
AUTHORITY: AGENTS.md SHA256:$WT_AGENTS_HASH
AUTHORITY: docs/implementation-ledger.md SHA256:$WT_LEDGER_HASH
ALLOWED_FILE: scripts/delegate-claude-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: claude-code
SUPERVISOR_MODEL: claude-opus-4-8
IMPLEMENTER_MODEL: claude-sonnet-5
TASK_SHA256: $task_hash
CODEX PRECHECK: APPROVED
EOF
assert_gate_reject "dependency WAIT" "$WT_DIR" "$order_file" "dependency U0e-2R is WAIT; dispatch is forbidden"

make_worktree "dep-ambiguous" "$(ledger_dep_ambiguous)"
order_file="$TMP_ROOT/order-dep-ambiguous.md"
cat >"$order_file" <<EOF
GRAIN: GR-D1
BASE_REF: refs/heads/$WT_BRANCH
BASE_SHA: $WT_BASE_SHA
DEPENDENCY: U0e-2R
AUTHORITY: AGENTS.md SHA256:$WT_AGENTS_HASH
AUTHORITY: docs/implementation-ledger.md SHA256:$WT_LEDGER_HASH
ALLOWED_FILE: scripts/delegate-claude-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: claude-code
SUPERVISOR_MODEL: claude-opus-4-8
IMPLEMENTER_MODEL: claude-sonnet-5
TASK_SHA256: $task_hash
CODEX PRECHECK: APPROVED
EOF
assert_gate_reject "duplicate dependency row" "$WT_DIR" "$order_file" "dependency U0e-2R has ambiguous selected-work ledger rows"

order_file="$TMP_ROOT/order-dependency-valid-plus-malformed.md"
{ valid_order | sed -n '1,3p'; echo "DEPENDENCY: U0e-2R extra"; valid_order | sed -n '4,$p'; } | write_order "$order_file"
assert_gate_reject "DEPENDENCY valid line plus malformed line" "$WT_VALID_DIR" "$order_file" "DEPENDENCY malformed"

order_file="$TMP_ROOT/order-dirty.md"
valid_order | write_order "$order_file"

cp -R "$WT_VALID_DIR" "$TMP_ROOT/wt-dirty-tracked"
echo "# tracked dirty" >>"$TMP_ROOT/wt-dirty-tracked/scripts/delegate-claude-supervised.sh"
assert_gate_reject "dirty tracked file" "$TMP_ROOT/wt-dirty-tracked" "$order_file" "isolated worktree is not clean"

cp -R "$WT_VALID_DIR" "$TMP_ROOT/wt-dirty-staged"
echo "# staged dirty" >>"$TMP_ROOT/wt-dirty-staged/scripts/delegate-claude-supervised.sh"
git -C "$TMP_ROOT/wt-dirty-staged" add scripts/delegate-claude-supervised.sh
assert_gate_reject "dirty staged file" "$TMP_ROOT/wt-dirty-staged" "$order_file" "isolated worktree is not clean"

cp -R "$WT_VALID_DIR" "$TMP_ROOT/wt-dirty-untracked"
echo "untracked" >"$TMP_ROOT/wt-dirty-untracked/untracked.txt"
assert_gate_reject "untracked file" "$TMP_ROOT/wt-dirty-untracked" "$order_file" "isolated worktree is not clean"

react_order_lines() {
  cat <<EOF
GRAIN: GR-D1
BASE_REF: refs/heads/$WT_VALID_BRANCH
BASE_SHA: $WT_VALID_BASE_SHA
DEPENDENCY: U0e-2R
AUTHORITY: AGENTS.md SHA256:$WT_VALID_AGENTS_HASH
AUTHORITY: docs/implementation-ledger.md SHA256:$WT_VALID_LEDGER_HASH
ALLOWED_FILE: docs/mocks-ui/App.jsx
REACT AUTHORITY: fixed react promotion contract
SOURCE ASSET: fixed sha, legacy path, export, closure
PRESERVE: DOM, class, stable id, ARIA
REPLACE: mock state to projection
STATE OWNER: Transient
DIAGNOSTIC ROUTE: separate from product route
NEGATIVE ORACLE: reject double copy
STOP: unresolved meaning halts
ORDER: READY
SUPERVISOR_BACKEND: claude-code
SUPERVISOR_MODEL: claude-opus-4-8
IMPLEMENTER_MODEL: claude-sonnet-5
TASK_SHA256: $task_hash
CODEX PRECHECK: APPROVED
EOF
}

order_file="$TMP_ROOT/order-react-missing-label.md"
react_order_lines | grep -v '^DIAGNOSTIC ROUTE:' | write_order "$order_file"
assert_gate_reject "React label missing" "$WT_VALID_DIR" "$order_file" "React guard label missing or out of order: DIAGNOSTIC ROUTE:"

order_file="$TMP_ROOT/order-react-out-of-order.md"
react_order_lines | awk '
  /^PRESERVE:/ { preserve = $0; next }
  /^REPLACE:/ { print; if (preserve != "") { print preserve; preserve = "" }; next }
  { print }
' | write_order "$order_file"
assert_gate_reject "React labels out of order" "$WT_VALID_DIR" "$order_file" "React guard label missing or out of order: REPLACE:"

order_file="$TMP_ROOT/order-react-duplicate-label.md"
react_order_lines | sed "/^PRESERVE:/a\\
PRESERVE: duplicate line
" | write_order "$order_file"
assert_gate_reject "duplicate React label" "$WT_VALID_DIR" "$order_file" "React guard label duplicated: PRESERVE:"

order_file="$TMP_ROOT/order-task-mismatch.md"
valid_order | sed -E "s#^TASK_SHA256: .*#TASK_SHA256: 0000000000000000000000000000000000000000000000000000000000000000#" | write_order "$order_file"
run_execute "$WT_VALID_DIR" "$order_file"
assert_status 3 "$RUN_STATUS" "task hash mismatch"
assert_has_fragment "$TMP_ROOT/stderr.log" "発注書とtaskが一致しません" "task hash mismatch"
assert_no_claude_calls "task hash mismatch"

order_file="$TMP_ROOT/order-no-codex-approval.md"
valid_order | grep -v '^CODEX PRECHECK: APPROVED' | write_order "$order_file"
run_execute "$WT_VALID_DIR" "$order_file"
assert_status 3 "$RUN_STATUS" "missing Codex approval"
assert_has_fragment "$TMP_ROOT/stderr.log" "Codex事前承認がありません" "missing Codex approval"
assert_no_claude_calls "missing Codex approval"

order_file="$TMP_ROOT/order-recursive.md"
valid_order | write_order "$order_file"
: >"$CALL_LOG"
if env PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" CLAUDE_DELEGATED=1 \
    "$SCRIPT" execute "$WT_VALID_DIR" "$order_file" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
  RUN_STATUS=0
else
  RUN_STATUS=$?
fi
assert_status 2 "$RUN_STATUS" "recursive delegation"
assert_has_fragment "$TMP_ROOT/stderr.log" "再帰実行は禁止です" "recursive delegation"
assert_no_claude_calls "recursive delegation"

PRIMARY_WORKTREE="$(git -C "$ROOT_DIR" worktree list --porcelain | awk '/^worktree / { print substr($0, 10); exit }')"
order_file="$TMP_ROOT/order-primary.md"
valid_order | write_order "$order_file"
: >"$CALL_LOG"
if env -u CLAUDE_DELEGATED PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
    "$SCRIPT" execute "$PRIMARY_WORKTREE" "$order_file" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
  RUN_STATUS=0
else
  RUN_STATUS=$?
fi
assert_status 2 "$RUN_STATUS" "primary worktree execution"
assert_has_fragment "$TMP_ROOT/stderr.log" "主作業ツリーへの実装発注は禁止です" "primary worktree execution"
assert_no_claude_calls "primary worktree execution"

order_file="$TMP_ROOT/order-happy-path.md"
valid_order | write_order "$order_file"
run_execute "$WT_VALID_DIR" "$order_file" \
  FAKE_SONNET_OUTPUT="implementation complete" \
  FAKE_OPUS_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "happy path dispatch"
assert_has_fragment "$CALL_LOG" "--model claude-sonnet-5" "happy path Sonnet invocation"
assert_has_fragment "$CALL_LOG" "--model claude-opus-4-8" "happy path Opus invocation"
sonnet_line="$(grep -n -- '--model claude-sonnet-5' "$CALL_LOG" | head -1 | cut -d: -f1)"
opus_line="$(grep -n -- '--model claude-opus-4-8' "$CALL_LOG" | head -1 | cut -d: -f1)"
[[ "$sonnet_line" -lt "$opus_line" ]] || fail "happy path dispatch: Sonnet must run before Opus inspection"
assert_has_fragment "$TMP_ROOT/stdout.log" "Opus検収ACCEPT" "happy path dispatch"

order_file="$TMP_ROOT/order-react-prose-only.md"
{
  valid_order | sed '/^ORDER: READY/i\
NOTE: this order mentions React in prose only and adds no jsx allowlist entry
'
} | write_order "$order_file"
run_execute "$WT_VALID_DIR" "$order_file" \
  FAKE_SONNET_OUTPUT="implementation complete" \
  FAKE_OPUS_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "React prose without markers"
assert_has_fragment "$CALL_LOG" "--model claude-sonnet-5" "React prose without markers reaches Sonnet"

order_file="$TMP_ROOT/order-react-valid.md"
react_order_lines | write_order "$order_file"
run_execute "$WT_VALID_DIR" "$order_file" \
  FAKE_SONNET_OUTPUT="implementation complete" \
  FAKE_OPUS_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "valid React order"
assert_has_fragment "$CALL_LOG" "--model claude-sonnet-5" "valid React order reaches Sonnet"

order_file="$TMP_ROOT/order-mocks-ui-non-jsx-no-labels.md"
valid_order | sed -E "s#^ALLOWED_FILE: .*#ALLOWED_FILE: docs/mocks-ui/App.tsx#" | write_order "$order_file"
assert_gate_reject "docs/mocks-ui non-jsx path without React labels" "$WT_VALID_DIR" "$order_file" \
  "React guard label missing or out of order: REACT AUTHORITY:"

order_file="$TMP_ROOT/order-mocks-ui-sibling.md"
valid_order | sed -E "s#^ALLOWED_FILE: .*#ALLOWED_FILE: docs/mocks-ui-legacy/README.md#" | write_order "$order_file"
run_execute "$WT_VALID_DIR" "$order_file" \
  FAKE_SONNET_OUTPUT="implementation complete" \
  FAKE_OPUS_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "docs/mocks-ui-legacy sibling does not trigger React labels"
assert_has_fragment "$CALL_LOG" "--model claude-sonnet-5" "docs/mocks-ui-legacy sibling reaches Sonnet"

WT_DIR="$WT_VALID_DIR"

order_file="$TMP_ROOT/prepare-ready.md"
run_prepare "$order_file" FAKE_OPUS_OUTPUT=$'draft order\nORDER: READY'
assert_status 0 "$RUN_STATUS" "prepare ORDER READY"
assert_contains "$order_file" "SUPERVISOR_BACKEND: claude-code" "prepare ORDER READY"
assert_contains "$order_file" "SUPERVISOR_MODEL: claude-opus-4-8" "prepare ORDER READY"
assert_contains "$order_file" "IMPLEMENTER_MODEL: claude-sonnet-5" "prepare ORDER READY"
assert_contains "$order_file" "TASK_SHA256: $(printf '%s' "prepare task" | shasum -a 256 | awk '{print $1}')" "prepare ORDER READY"
assert_has_fragment "$CALL_LOG" "--model claude-opus-4-8" "prepare fixed model id"

order_file="$TMP_ROOT/prepare-stop.md"
run_prepare "$order_file" FAKE_OPUS_OUTPUT=$'blocked by unresolved decision\nORDER: STOP'
assert_status 3 "$RUN_STATUS" "prepare ORDER STOP"
assert_contains "$order_file" "SUPERVISOR_BACKEND: claude-code" "prepare ORDER STOP"

order_file="$TMP_ROOT/prepare-ambiguous.md"
run_prepare "$order_file" FAKE_OPUS_OUTPUT=$'ORDER: STOP\nORDER: READY'
assert_status 1 "$RUN_STATUS" "prepare ambiguous markers"
[[ ! -e "$order_file" ]] || fail "prepare ambiguous markers: order file must not be created"

order_file="$TMP_ROOT/prepare-nonterminal.md"
run_prepare "$order_file" FAKE_OPUS_OUTPUT=$'ORDER: READY\ntrailing text'
assert_status 1 "$RUN_STATUS" "prepare nonterminal marker"
[[ ! -e "$order_file" ]] || fail "prepare nonterminal marker: order file must not be created"

echo "test-delegate-claude-supervised: all tests passed"
