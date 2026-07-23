#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/delegate-cursor-supervised.sh"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/motolii-delegate-cursor-test.XXXXXX")"

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

assert_has_fragment() {
  local file="$1"
  local expected="$2"
  local label="$3"
  grep -Fq -- "$expected" "$file" || fail "$label: missing fragment '$expected' in $file"
}

assert_no_external_calls() {
  local label="$1"
  if [[ -s "$CALL_LOG" ]]; then
    fail "$label: expected zero external invocations, got: $(cat "$CALL_LOG")"
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
  printf '#!/usr/bin/env bash\necho dummy\n' >"$dir/scripts/delegate-cursor-supervised.sh"
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

cat >"$FAKE_BIN/cursor-agent" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo cursor-agent >>"$FAKE_CALL_LOG"
printf 'cursor-agent-args:%s\n' "$*" >>"$FAKE_CALL_LOG"
if [[ -n "${FAKE_GROK_REQUIRE_DEV_NULL:-}" ]] && [[ ! /dev/stdin -ef /dev/null ]]; then
  echo "Grok stdin was not closed" >&2
  exit 91
fi
if [[ -n "${FAKE_GROK_SLEEP:-}" ]]; then
  sleep "${FAKE_GROK_SLEEP}"
fi
if [[ -n "${FAKE_GROK_HOOK:-}" ]]; then
  bash "${FAKE_GROK_HOOK}"
fi
printf '%s\n' "${FAKE_GROK_OUTPUT:-}"
exit "${FAKE_GROK_STATUS:-0}"
EOF

cat >"$FAKE_BIN/codex" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo codex >>"$FAKE_CALL_LOG"
printf 'codex-args:%s\n' "$*" >>"$FAKE_CALL_LOG"
if [[ -n "${FAKE_TERRA_REQUIRE_DEV_NULL:-}" ]] && [[ ! /dev/stdin -ef /dev/null ]]; then
  echo "Terra stdin was not closed" >&2
  exit 91
fi
if [[ -n "${FAKE_TERRA_IGNORE_TERM:-}" ]]; then
  trap '' TERM
fi
if [[ -n "${FAKE_TERRA_SLEEP:-}" ]]; then
  sleep "${FAKE_TERRA_SLEEP}"
fi
if [[ -n "${FAKE_TERRA_HOOK:-}" ]]; then
  bash "${FAKE_TERRA_HOOK}"
fi
printf '%s\n' "${FAKE_TERRA_OUTPUT:-implementation complete}"
exit "${FAKE_TERRA_STATUS:-0}"
EOF
chmod +x "$FAKE_BIN/cursor-agent" "$FAKE_BIN/codex"

task="GR-D1 dispatch gate execution"
task_hash="$(printf '%s' "$task" | shasum -a 256 | awk '{print $1}')"

run_execute() {
  local worktree="$1" order_file="$2"
  shift 2
  : >"$CALL_LOG"
  if env -u CURSOR_AGENT -u CODEX_DELEGATED \
      PATH="$FAKE_BIN:/usr/bin:/bin" \
      FAKE_CALL_LOG="$CALL_LOG" \
      CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
      CURSOR_SUPERVISED_TIMEOUT_SECONDS=5 \
      CODEX_TERRA_TIMEOUT_SECONDS=5 \
      CURSOR_INSPECTION_TIMEOUT_SECONDS=5 \
      "$@" \
      "$SCRIPT" execute "$worktree" "$order_file" "$task" \
      >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
    RUN_STATUS=0
  else
    RUN_STATUS=$?
  fi
}

run_inspect() {
  local worktree="$1" order_file="$2"
  shift 2
  : >"$CALL_LOG"
  if env -u CURSOR_AGENT -u CODEX_DELEGATED \
      PATH="$FAKE_BIN:/usr/bin:/bin" \
      FAKE_CALL_LOG="$CALL_LOG" \
      CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
      CURSOR_INSPECTION_TIMEOUT_SECONDS=5 \
      "$@" \
      "$SCRIPT" inspect "$worktree" "$order_file" "$task" \
      >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
    RUN_STATUS=0
  else
    RUN_STATUS=$?
  fi
}

# WT_VALID_DIRはこの時点でclean/WT_VALID_BRANCHのままである前提の独立copyを作る。
# 各GR-D2試験はこのcopyへ書き込み/commitを行うため、共有stateを汚さない
fresh_valid_worktree() {
  local name="$1"
  local dir="$TMP_ROOT/wt-gr-d2-$name"
  rm -rf "$dir"
  cp -R "$WT_VALID_DIR" "$dir"
  printf '%s' "$dir"
}

evidence_root_for() {
  printf '%s.evidence' "$1"
}

latest_attempt_dir() {
  local root="$1" d best=""
  for d in "$root"/attempt-*; do
    [[ -d "$d" ]] || continue
    best="$d"
  done
  printf '%s' "$best"
}

assert_file_exists() {
  local path="$1" label="$2"
  [[ -e "$path" ]] || fail "$label: expected file to exist: $path"
}

run_prepare() {
  local order_file="$1"
  shift 1
  : >"$CALL_LOG"
  if env -u CURSOR_AGENT -u CODEX_DELEGATED \
      PATH="$FAKE_BIN:/usr/bin:/bin" \
      FAKE_CALL_LOG="$CALL_LOG" \
      CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
      CURSOR_SUPERVISED_TIMEOUT_SECONDS=5 \
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
  assert_no_external_calls "$label"
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
ALLOWED_FILE: scripts/delegate-cursor-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: cursor-grok
SUPERVISOR_MODEL: cursor-grok-4.5-high
IMPLEMENTER_MODEL: gpt-5.6-terra
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
ALLOWED_FILE: scripts/delegate-cursor-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: cursor-grok
SUPERVISOR_MODEL: cursor-grok-4.5-high
IMPLEMENTER_MODEL: gpt-5.6-terra
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
ALLOWED_FILE: scripts/delegate-cursor-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: cursor-grok
SUPERVISOR_MODEL: cursor-grok-4.5-high
IMPLEMENTER_MODEL: gpt-5.6-terra
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
ALLOWED_FILE: scripts/delegate-cursor-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: cursor-grok
SUPERVISOR_MODEL: cursor-grok-4.5-high
IMPLEMENTER_MODEL: gpt-5.6-terra
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
ALLOWED_FILE: scripts/delegate-cursor-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: cursor-grok
SUPERVISOR_MODEL: cursor-grok-4.5-high
IMPLEMENTER_MODEL: gpt-5.6-terra
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
ALLOWED_FILE: scripts/delegate-cursor-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: cursor-grok
SUPERVISOR_MODEL: cursor-grok-4.5-high
IMPLEMENTER_MODEL: gpt-5.6-terra
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
ALLOWED_FILE: scripts/delegate-cursor-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: cursor-grok
SUPERVISOR_MODEL: cursor-grok-4.5-high
IMPLEMENTER_MODEL: gpt-5.6-terra
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
echo "# tracked dirty" >>"$TMP_ROOT/wt-dirty-tracked/scripts/delegate-cursor-supervised.sh"
assert_gate_reject "dirty tracked file" "$TMP_ROOT/wt-dirty-tracked" "$order_file" "isolated worktree is not clean"

cp -R "$WT_VALID_DIR" "$TMP_ROOT/wt-dirty-staged"
echo "# staged dirty" >>"$TMP_ROOT/wt-dirty-staged/scripts/delegate-cursor-supervised.sh"
git -C "$TMP_ROOT/wt-dirty-staged" add scripts/delegate-cursor-supervised.sh
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
SUPERVISOR_BACKEND: cursor-grok
SUPERVISOR_MODEL: cursor-grok-4.5-high
IMPLEMENTER_MODEL: gpt-5.6-terra
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
assert_no_external_calls "task hash mismatch"

order_file="$TMP_ROOT/order-no-codex-approval.md"
valid_order | grep -v '^CODEX PRECHECK: APPROVED' | write_order "$order_file"
run_execute "$WT_VALID_DIR" "$order_file"
assert_status 3 "$RUN_STATUS" "missing Codex approval"
assert_has_fragment "$TMP_ROOT/stderr.log" "Codex事前承認がありません" "missing Codex approval"
assert_no_external_calls "missing Codex approval"

order_file="$TMP_ROOT/order-stale-model.md"
valid_order | sed 's/^IMPLEMENTER_MODEL:.*/IMPLEMENTER_MODEL: composer-2.5/' | write_order "$order_file"
run_execute "$WT_VALID_DIR" "$order_file"
assert_status 3 "$RUN_STATUS" "stale implementer model"
assert_has_fragment "$TMP_ROOT/stderr.log" "backend/model固定が現行のTerra + Grok運用と一致しません" "stale implementer model"
assert_no_external_calls "stale implementer model"

order_file="$TMP_ROOT/order-recursive.md"
valid_order | write_order "$order_file"
: >"$CALL_LOG"
if env PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" CURSOR_AGENT=1 \
    "$SCRIPT" execute "$WT_VALID_DIR" "$order_file" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
  RUN_STATUS=0
else
  RUN_STATUS=$?
fi
assert_status 2 "$RUN_STATUS" "recursive delegation"
assert_has_fragment "$TMP_ROOT/stderr.log" "再帰実行は禁止です" "recursive delegation"
assert_no_external_calls "recursive delegation"

PRIMARY_WORKTREE="$(git -C "$ROOT_DIR" worktree list --porcelain | awk '/^worktree / && !found { print substr($0, 10); found=1 }')"
order_file="$TMP_ROOT/order-primary.md"
valid_order | write_order "$order_file"
: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CODEX_DELEGATED PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
    "$SCRIPT" execute "$PRIMARY_WORKTREE" "$order_file" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
  RUN_STATUS=0
else
  RUN_STATUS=$?
fi
assert_status 2 "$RUN_STATUS" "primary worktree execution"
assert_has_fragment "$TMP_ROOT/stderr.log" "主作業ツリーへの実装発注は禁止です" "primary worktree execution"
assert_no_external_calls "primary worktree execution"

order_file="$TMP_ROOT/order-happy-path.md"
valid_order | write_order "$order_file"
run_execute "$WT_VALID_DIR" "$order_file" \
  FAKE_TERRA_OUTPUT="implementation complete" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "happy path dispatch"
assert_has_fragment "$CALL_LOG" "--model gpt-5.6-terra" "happy path Terra invocation"
assert_has_fragment "$CALL_LOG" "--model cursor-grok-4.5-high" "happy path Grok invocation"
assert_has_fragment "$CALL_LOG" "--ask-for-approval never" "happy path Terra is noninteractive"
assert_has_fragment "$CALL_LOG" "--sandbox danger-full-access" "happy path Terra uses the isolated worktree sandbox"
assert_has_fragment "$CALL_LOG" "--mode ask" "happy path Grok inspection is read-only"
if grep -q -- "--force" "$CALL_LOG"; then
  fail "happy path dispatch: Grok inspection must not receive write-capable --force"
fi
terra_line="$(grep -n -- '--model gpt-5.6-terra' "$CALL_LOG" | head -1 | cut -d: -f1)"
grok_line="$(grep -n -- '--model cursor-grok-4.5-high' "$CALL_LOG" | head -1 | cut -d: -f1)"
[[ "$terra_line" -lt "$grok_line" ]] || fail "happy path dispatch: Terra must run before Grok inspection"
assert_has_fragment "$TMP_ROOT/stdout.log" "Grok検収ACCEPT" "happy path dispatch"

run_execute "$WT_VALID_DIR" "$order_file" \
  FAKE_TERRA_REQUIRE_DEV_NULL=1 \
  FAKE_GROK_REQUIRE_DEV_NULL=1 \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "external agent stdin is closed"

run_execute "$WT_VALID_DIR" "$order_file" \
  CODEX_TERRA_MODEL=composer-2.5 \
  CURSOR_GROK_MODEL=cursor-grok-fast \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "model environment overrides are ignored"
assert_has_fragment "$CALL_LOG" "--model gpt-5.6-terra" "fixed Terra model ignores environment"
assert_has_fragment "$CALL_LOG" "--model cursor-grok-4.5-high" "fixed Grok model ignores environment"
if grep -Eq -- '--model (composer-2.5|cursor-grok-fast)' "$CALL_LOG"; then
  fail "model environment overrides must not reach external CLIs"
fi

order_file="$TMP_ROOT/order-react-prose-only.md"
{
  valid_order | sed '/^ORDER: READY/i\
NOTE: this order mentions React in prose only and adds no jsx allowlist entry
'
} | write_order "$order_file"
run_execute "$WT_VALID_DIR" "$order_file" \
  FAKE_TERRA_OUTPUT="implementation complete" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "React prose without markers"
assert_has_fragment "$CALL_LOG" "--model gpt-5.6-terra" "React prose without markers reaches Terra"

order_file="$TMP_ROOT/order-react-valid.md"
react_order_lines | write_order "$order_file"
run_execute "$WT_VALID_DIR" "$order_file" \
  FAKE_TERRA_OUTPUT="implementation complete" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "valid React order"
assert_has_fragment "$CALL_LOG" "--model gpt-5.6-terra" "valid React order reaches Terra"

order_file="$TMP_ROOT/order-mocks-ui-non-jsx-no-labels.md"
valid_order | sed -E "s#^ALLOWED_FILE: .*#ALLOWED_FILE: docs/mocks-ui/App.tsx#" | write_order "$order_file"
assert_gate_reject "docs/mocks-ui non-jsx path without React labels" "$WT_VALID_DIR" "$order_file" \
  "React guard label missing or out of order: REACT AUTHORITY:"

order_file="$TMP_ROOT/order-mocks-ui-sibling.md"
valid_order | sed -E "s#^ALLOWED_FILE: .*#ALLOWED_FILE: docs/mocks-ui-legacy/README.md#" | write_order "$order_file"
run_execute "$WT_VALID_DIR" "$order_file" \
  FAKE_TERRA_OUTPUT="implementation complete" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "docs/mocks-ui-legacy sibling does not trigger React labels"
assert_has_fragment "$CALL_LOG" "--model gpt-5.6-terra" "docs/mocks-ui-legacy sibling reaches Terra"

WT_DIR="$WT_VALID_DIR"

order_file="$TMP_ROOT/prepare-ready.md"
run_prepare "$order_file" FAKE_GROK_OUTPUT=$'draft order\nORDER: READY'
assert_status 0 "$RUN_STATUS" "prepare ORDER READY"
assert_contains "$order_file" "SUPERVISOR_BACKEND: cursor-grok" "prepare ORDER READY"
assert_contains "$order_file" "SUPERVISOR_MODEL: cursor-grok-4.5-high" "prepare ORDER READY"
assert_contains "$order_file" "IMPLEMENTER_MODEL: gpt-5.6-terra" "prepare ORDER READY"
assert_contains "$order_file" "TASK_SHA256: $(printf '%s' "prepare task" | shasum -a 256 | awk '{print $1}')" "prepare ORDER READY"
assert_has_fragment "$CALL_LOG" "--model cursor-grok-4.5-high" "prepare fixed model id"
assert_has_fragment "$CALL_LOG" "--mode ask" "prepare uses the non-editing Cursor mode"

order_file="$TMP_ROOT/prepare-stop.md"
run_prepare "$order_file" FAKE_GROK_OUTPUT=$'blocked by unresolved decision\nORDER: STOP'
assert_status 3 "$RUN_STATUS" "prepare ORDER STOP"
assert_contains "$order_file" "SUPERVISOR_BACKEND: cursor-grok" "prepare ORDER STOP"

order_file="$TMP_ROOT/prepare-ambiguous.md"
run_prepare "$order_file" FAKE_GROK_OUTPUT=$'ORDER: STOP\nORDER: READY'
assert_status 1 "$RUN_STATUS" "prepare ambiguous markers"
[[ ! -e "$order_file" ]] || fail "prepare ambiguous markers: order file must not be created"

order_file="$TMP_ROOT/prepare-nonterminal.md"
run_prepare "$order_file" FAKE_GROK_OUTPUT=$'ORDER: READY\ntrailing text'
assert_status 1 "$RUN_STATUS" "prepare nonterminal marker"
[[ ! -e "$order_file" ]] || fail "prepare nonterminal marker: order file must not be created"

##############################################################################
# GR-D2: 変更許可閉集合・永続証跡・検収再開
##############################################################################

order_with_allowed() {
  # valid_order()の1行だけALLOWED_FILEを差し替える。複数行を渡すと複数ALLOWED_FILEになる
  local allowed_lines="$1"
  valid_order | grep -v '^ALLOWED_FILE:' | awk -v extra="$allowed_lines" '
    /^ORDER: READY/ { print extra; print; next }
    { print }
  '
}

hook_touch() {
  # Terra/Grok hookから使う、1行1コマンドのスクリプトファイルを作る
  local file="$1"
  shift
  printf '%s\n' "$@" >"$file"
}

# --- 1. 許可されたtracked変更とuntracked新規ファイルはGrokへ到達する(literal) ---
wt="$(fresh_valid_worktree allowed-literal)"
order_file="$TMP_ROOT/order-gr-d2-allowed-literal.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-allowed-literal.sh"
hook_touch "$hook" "echo '# allowed literal edit' >> \"$wt/scripts/delegate-cursor-supervised.sh\""
run_execute "$wt" "$order_file" \
  FAKE_TERRA_HOOK="$hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "GR-D2 allowed literal tracked edit reaches Grok"
assert_has_fragment "$CALL_LOG" "--model cursor-grok-4.5-high" "GR-D2 allowed literal tracked edit reaches Grok"

# --- 2. 許可された(glob)untracked新規ファイルはGrokへ到達するが、glob非対象の
#        literalだけの発注書では同じ変更がSCOPE NGになる ---
wt="$(fresh_valid_worktree allowed-glob)"
order_file="$TMP_ROOT/order-gr-d2-allowed-glob.md"
order_with_allowed "ALLOWED_FILE: scripts/*.sh" | write_order "$order_file"
hook="$TMP_ROOT/hook-allowed-glob.sh"
hook_touch "$hook" "echo generated > \"$wt/scripts/generated.sh\""
run_execute "$wt" "$order_file" \
  FAKE_TERRA_HOOK="$hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "GR-D2 explicit glob ALLOWED_FILE reaches Grok"
assert_has_fragment "$CALL_LOG" "--model cursor-grok-4.5-high" "GR-D2 explicit glob ALLOWED_FILE reaches Grok"

wt="$(fresh_valid_worktree literal-only-vs-glob-path)"
order_file="$TMP_ROOT/order-gr-d2-literal-only.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-literal-only.sh"
hook_touch "$hook" "echo generated > \"$wt/scripts/generated.sh\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "GR-D2 same new file blocked without explicit glob"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "GR-D2 same new file blocked without explicit glob"
assert_has_fragment "$TMP_ROOT/stderr.log" "scripts/generated.sh" "GR-D2 same new file blocked without explicit glob"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "GR-D2 same new file blocked without explicit glob: Grok must not be called"
fi

# --- 2b. globはpath component単位でのみ照合されるべきで、"scripts/*.sh"が
#         "scripts/sub/a.sh"まで拾ってしまう誤検出がないことを確認する ---
wt="$(fresh_valid_worktree glob-same-dir)"
order_file="$TMP_ROOT/order-gr-d2-glob-same-dir.md"
order_with_allowed "ALLOWED_FILE: scripts/*.sh" | write_order "$order_file"
hook="$TMP_ROOT/hook-glob-same-dir.sh"
hook_touch "$hook" "echo generated > \"$wt/scripts/a.sh\""
run_execute "$wt" "$order_file" \
  FAKE_TERRA_HOOK="$hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "GR-D2 glob matches file in the same directory component"
assert_has_fragment "$CALL_LOG" "--model cursor-grok-4.5-high" "GR-D2 glob matches file in the same directory component"

wt="$(fresh_valid_worktree glob-deeper-dir)"
order_file="$TMP_ROOT/order-gr-d2-glob-deeper-dir.md"
order_with_allowed "ALLOWED_FILE: scripts/*.sh" | write_order "$order_file"
hook="$TMP_ROOT/hook-glob-deeper-dir.sh"
hook_touch "$hook" "mkdir -p \"$wt/scripts/sub\"" "echo generated > \"$wt/scripts/sub/a.sh\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "GR-D2 glob must not cross a path separator into a subdirectory"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "GR-D2 glob must not cross a path separator into a subdirectory"
assert_has_fragment "$TMP_ROOT/stderr.log" "scripts/sub/a.sh" "GR-D2 glob must not cross a path separator into a subdirectory"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "GR-D2 glob must not cross a path separator into a subdirectory: Grok must not be called"
fi

wt="$(fresh_valid_worktree glob-deeper-dir-approved)"
order_file="$TMP_ROOT/order-gr-d2-glob-deeper-dir-approved.md"
order_with_allowed "ALLOWED_FILE: scripts/sub/*.sh" | write_order "$order_file"
hook="$TMP_ROOT/hook-glob-deeper-dir-approved.sh"
hook_touch "$hook" "mkdir -p \"$wt/scripts/sub\"" "echo generated > \"$wt/scripts/sub/a.sh\""
run_execute "$wt" "$order_file" \
  FAKE_TERRA_HOOK="$hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "GR-D2 an explicitly deeper glob shape is approved"
assert_has_fragment "$CALL_LOG" "--model cursor-grok-4.5-high" "GR-D2 an explicitly deeper glob shape is approved"

# --- 2c. `dir/**`はbuild cache等の閉じたsubtreeだけを再帰許可する ---
wt="$(fresh_valid_worktree recursive-subtree-approved)"
printf 'target/\n' >"$wt/.gitignore"
git -C "$wt" add .gitignore
git -C "$wt" commit -qm "add ignored build subtree"
recursive_base_sha="$(git -C "$wt" rev-parse HEAD)"
order_file="$TMP_ROOT/order-gr-d2-recursive-subtree.md"
valid_order |
  sed -e '/^ORDER: READY/i\
ALLOWED_FILE: target/**
' -e "s#^BASE_SHA: .*#BASE_SHA: $recursive_base_sha#" | write_order "$order_file"
hook="$TMP_ROOT/hook-recursive-subtree.sh"
hook_touch "$hook" "mkdir -p \"$wt/target/deep/cache\"" "echo generated > \"$wt/target/deep/cache/item.bin\""
run_execute "$wt" "$order_file" \
  FAKE_TERRA_HOOK="$hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "GR-D2 explicit recursive subtree reaches Grok"

# --- 3. worktreeが無変更でもACCEPTし、到達した全stage分のevidenceが残ることを確認する ---
wt="$(fresh_valid_worktree unchanged-evidence)"
order_file="$TMP_ROOT/order-gr-d2-unchanged.md"
valid_order | write_order "$order_file"
run_execute "$wt" "$order_file" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "GR-D2 unchanged worktree ACCEPT"
evroot="$(evidence_root_for "$order_file")"
attempt="$(latest_attempt_dir "$evroot")"
[[ -n "$attempt" ]] || fail "GR-D2 unchanged worktree ACCEPT: attempt dir missing"
for f in order.txt metadata.txt task.txt \
         pre-terra-status.txt pre-terra-diff.txt pre-terra-fingerprint.sha256 \
         terra-stdout.txt terra-stdout.txt.err \
         post-terra-status.txt post-terra-diff.txt post-terra-fingerprint.sha256 \
         pre-grok-status.txt pre-grok-diff.txt pre-grok-fingerprint.sha256 \
         grok-stdout.txt grok-stdout.txt.err \
         post-grok-status.txt post-grok-diff.txt post-grok-fingerprint.sha256 \
         stage-result.txt; do
  assert_file_exists "$attempt/$f" "GR-D2 unchanged worktree ACCEPT evidence"
done
assert_file_exists "$evroot/checkpoint.txt" "GR-D2 unchanged worktree ACCEPT checkpoint"
assert_has_fragment "$attempt/metadata.txt" "BASE_REF: refs/heads/$WT_VALID_BRANCH" \
  "GR-D2 unchanged worktree ACCEPT records validated BASE_REF"
assert_has_fragment "$attempt/metadata.txt" "BASE_SHA: $WT_VALID_BASE_SHA" \
  "GR-D2 unchanged worktree ACCEPT records validated BASE_SHA"
assert_has_fragment "$attempt/stage-result.txt" "EXIT_STATUS: 0" \
  "GR-D2 unchanged worktree ACCEPT records numeric exit status"

# --- 4. 検収timeoutは証跡として残り、再開したinspectはfake Grokだけを呼びACCEPTできる ---
wt="$(fresh_valid_worktree inspect-resume)"
order_file="$TMP_ROOT/order-gr-d2-inspect-resume.md"
valid_order | write_order "$order_file"
run_execute "$wt" "$order_file" \
  CURSOR_INSPECTION_TIMEOUT_SECONDS=1 \
  FAKE_GROK_SLEEP=3
assert_status 1 "$RUN_STATUS" "GR-D2 inspection timeout on first execute"
assert_has_fragment "$TMP_ROOT/stderr.log" "1秒でタイムアウトしました" "GR-D2 inspection timeout uses inspection timeout"
evroot="$(evidence_root_for "$order_file")"
first_attempt="$(latest_attempt_dir "$evroot")"
assert_file_exists "$first_attempt/grok-stdout.txt.timeout" "GR-D2 inspection timeout evidence"
assert_file_exists "$evroot/checkpoint.txt" "GR-D2 inspection timeout preserves checkpoint from successful implementation"

run_inspect "$wt" "$order_file" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "GR-D2 resumed inspect ACCEPTs without Terra"
if grep -q -- "--model gpt-5.6-terra" "$CALL_LOG"; then
  fail "GR-D2 resumed inspect ACCEPTs without Terra: Terra must not be invoked"
fi
assert_has_fragment "$CALL_LOG" "--model cursor-grok-4.5-high" "GR-D2 resumed inspect ACCEPTs without Terra"
second_attempt="$(latest_attempt_dir "$evroot")"
[[ "$second_attempt" != "$first_attempt" ]] || fail "GR-D2 resumed inspect: expected a new attempt directory"
assert_file_exists "$first_attempt/grok-stdout.txt.timeout" "GR-D2 resumed inspect: earlier timeout attempt preserved"

# --- 5. 複数回試行しても、先行するattempt directoryはbyte単位で不変のままである ---
before_snapshot="$TMP_ROOT/first-attempt-snapshot"
cp -R "$first_attempt" "$before_snapshot"
run_inspect "$wt" "$order_file" FAKE_GROK_STATUS=1 FAKE_GROK_OUTPUT="rejected"
assert_status 1 "$RUN_STATUS" "GR-D2 extra inspect attempt after resume"
if ! diff -r "$before_snapshot" "$first_attempt" >/dev/null 2>&1; then
  fail "GR-D2 multiple attempts: earlier attempt directory mutated"
fi

# --- 6. 既にuntrackedな許可済みfileでも、git statusの文言ではなく中身でfingerprintされる ---
wt="$(fresh_valid_worktree untracked-content-fingerprint)"
order_file="$TMP_ROOT/order-gr-d2-untracked-fp.md"
order_with_allowed "ALLOWED_FILE: scripts/generated.sh" | write_order "$order_file"
hook="$TMP_ROOT/hook-untracked-fp.sh"
hook_touch "$hook" "echo one > \"$wt/scripts/generated.sh\""
grok_hook="$TMP_ROOT/hook-untracked-fp-grok.sh"
hook_touch "$grok_hook" "echo two > \"$wt/scripts/generated.sh\""
run_execute "$wt" "$order_file" \
  FAKE_TERRA_HOOK="$hook" \
  FAKE_GROK_HOOK="$grok_hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 8 "$RUN_STATUS" "GR-D2 untracked-allowed content mutation invalidates ACCEPT"
assert_has_fragment "$TMP_ROOT/stderr.log" "INSPECT NG:" "GR-D2 untracked-allowed content mutation invalidates ACCEPT"

# --- 6b. Terraのtimeoutでも、timeout marker・stdout/stderr・stage result・
#         数値exit status・先行attemptの保全がcleanup後も全て残ることを確認する ---
wt="$(fresh_valid_worktree terra-timeout)"
order_file="$TMP_ROOT/order-gr-d2-terra-timeout.md"
valid_order | write_order "$order_file"
run_execute "$wt" "$order_file" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "execute before Terra-timeout attempt"
evroot="$(evidence_root_for "$order_file")"
terra_timeout_prior_attempt="$(latest_attempt_dir "$evroot")"
terra_timeout_prior_snapshot="$TMP_ROOT/terra-timeout-prior-snapshot"
cp -R "$terra_timeout_prior_attempt" "$terra_timeout_prior_snapshot"

run_execute "$wt" "$order_file" \
  CODEX_TERRA_TIMEOUT_SECONDS=1 \
  CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
  FAKE_TERRA_SLEEP=3
assert_status 1 "$RUN_STATUS" "Terra timeout preserves durable evidence"
assert_has_fragment "$TMP_ROOT/stderr.log" "1秒でタイムアウトしました" "Terra timeout uses the implementer timeout"
terra_timeout_attempt="$(latest_attempt_dir "$evroot")"
[[ "$terra_timeout_attempt" != "$terra_timeout_prior_attempt" ]] || \
  fail "Terra timeout: expected a new attempt directory"
assert_file_exists "$terra_timeout_attempt/terra-stdout.txt.timeout" "Terra timeout marker persists after cleanup"
assert_file_exists "$terra_timeout_attempt/terra-stdout.txt" "Terra timeout stdout persists after cleanup"
assert_file_exists "$terra_timeout_attempt/terra-stdout.txt.err" "Terra timeout stderr persists after cleanup"
assert_has_fragment "$terra_timeout_attempt/stage-result.txt" "STAGE: terra FAILED_OR_TIMEOUT" \
  "Terra timeout records stage result"
assert_has_fragment "$terra_timeout_attempt/stage-result.txt" "EXIT_STATUS:" \
  "Terra timeout records numeric exit status"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "Terra timeout: Grok must not be called"
fi
if ! diff -r "$terra_timeout_prior_snapshot" "$terra_timeout_prior_attempt" >/dev/null 2>&1; then
  fail "Terra timeout: earlier attempt directory mutated"
fi

# TERMを無視するleader/子孫でもgrace後のKILLで有界に復帰する。
wt="$(fresh_valid_worktree terra-timeout-term-ignore)"
order_file="$TMP_ROOT/order-gr-d2-terra-timeout-term-ignore.md"
valid_order | write_order "$order_file"
SECONDS=0
run_execute "$wt" "$order_file" \
  CODEX_TERRA_TIMEOUT_SECONDS=1 \
  CURSOR_TERMINATION_GRACE_SECONDS=1 \
  FAKE_TERRA_IGNORE_TERM=1 \
  FAKE_TERRA_SLEEP=30
elapsed="$SECONDS"
assert_status 1 "$RUN_STATUS" "TERM-ignoring Terra is KILLed after grace"
(( elapsed <= 6 )) || fail "TERM-ignoring Terra timeout exceeded bound: ${elapsed}s"
assert_has_fragment "$TMP_ROOT/stderr.log" "1秒でタイムアウトしました" "TERM-ignoring Terra timeout marker"

##############################################################################
# GR-D2 negative: 許可閉集合外のあらゆる変更categoryがSCOPE NGでGrok起動を阻む
##############################################################################

wt="$(fresh_valid_worktree scope-tracked)"
order_file="$TMP_ROOT/order-gr-d2-scope-tracked.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-scope-tracked.sh"
hook_touch "$hook" "echo extra >> \"$wt/docs/implementation-ledger.md\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "tracked out-of-allowlist change"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "tracked out-of-allowlist change"
assert_has_fragment "$TMP_ROOT/stderr.log" "docs/implementation-ledger.md" "tracked out-of-allowlist change"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "tracked out-of-allowlist change: Grok must not be called"
fi

wt="$(fresh_valid_worktree scope-staged)"
order_file="$TMP_ROOT/order-gr-d2-scope-staged.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-scope-staged.sh"
hook_touch "$hook" "echo extra >> \"$wt/AGENTS.md\"" "git -C \"$wt\" add AGENTS.md"
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "staged out-of-allowlist change"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "staged out-of-allowlist change"
assert_has_fragment "$TMP_ROOT/stderr.log" "AGENTS.md" "staged out-of-allowlist change"

wt="$(fresh_valid_worktree scope-deleted)"
order_file="$TMP_ROOT/order-gr-d2-scope-deleted.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-scope-deleted.sh"
hook_touch "$hook" "rm \"$wt/docs/implementation-ledger.md\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "deleted out-of-allowlist tracked file"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "deleted out-of-allowlist tracked file"
assert_has_fragment "$TMP_ROOT/stderr.log" "docs/implementation-ledger.md" "deleted out-of-allowlist tracked file"

wt="$(fresh_valid_worktree scope-rename)"
order_file="$TMP_ROOT/order-gr-d2-scope-rename.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-scope-rename.sh"
hook_touch "$hook" "git -C \"$wt\" mv docs/implementation-ledger.md docs/implementation-ledger-renamed.md"
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "renamed out-of-allowlist tracked file"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "renamed out-of-allowlist tracked file (old side)"
assert_has_fragment "$TMP_ROOT/stderr.log" "docs/implementation-ledger.md" "renamed out-of-allowlist tracked file (old side)"
assert_has_fragment "$TMP_ROOT/stderr.log" "docs/implementation-ledger-renamed.md" "renamed out-of-allowlist tracked file (new side)"

wt="$(fresh_valid_worktree scope-untracked)"
order_file="$TMP_ROOT/order-gr-d2-scope-untracked.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-scope-untracked.sh"
hook_touch "$hook" "echo new > \"$wt/oos.txt\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "untracked out-of-allowlist file"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "untracked out-of-allowlist file"
assert_has_fragment "$TMP_ROOT/stderr.log" "oos.txt" "untracked out-of-allowlist file"

wt="$(fresh_valid_worktree scope-ignored)"
printf 'target/\n' >"$wt/.gitignore"
git -C "$wt" add .gitignore
git -C "$wt" commit -qm "add ignored subtree"
ignored_base_sha="$(git -C "$wt" rev-parse HEAD)"
order_file="$TMP_ROOT/order-gr-d2-scope-ignored.md"
valid_order | sed -E "s#^BASE_SHA: .*#BASE_SHA: $ignored_base_sha#" | write_order "$order_file"
hook="$TMP_ROOT/hook-scope-ignored.sh"
hook_touch "$hook" "mkdir -p \"$wt/target/hidden\"" "echo hidden > \"$wt/target/hidden/oos.bin\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "ignored out-of-allowlist file"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "ignored out-of-allowlist file"
assert_has_fragment "$TMP_ROOT/stderr.log" "target/hidden/oos.bin" "ignored out-of-allowlist file"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "ignored out-of-allowlist file: Grok must not be called"
fi

wt="$(fresh_valid_worktree scope-whitespace)"
order_file="$TMP_ROOT/order-gr-d2-scope-whitespace.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-scope-whitespace.sh"
hook_touch "$hook" "echo new > \"$wt/oos with spaces.txt\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "whitespace-name out-of-allowlist file"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "whitespace-name out-of-allowlist file"
assert_has_fragment "$TMP_ROOT/stderr.log" "oos with spaces.txt" "whitespace-name out-of-allowlist file"

wt="$(fresh_valid_worktree scope-newline)"
order_file="$TMP_ROOT/order-gr-d2-scope-newline.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-scope-newline.sh"
# ファイル名へ実際のLFバイトを埋め込む(バックスラッシュ+nの2文字ではない)。
# $newline_pathの構築自体に実LFを含め、それをそのままhookスクリプトの引用符内へ書く
newline_path="$wt/oos-with"$'\n'"newline.txt"
printf 'printf %s > "%s"\n' "new" "$newline_path" >"$hook"
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "newline-name out-of-allowlist file"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "newline-name out-of-allowlist file"
assert_has_fragment "$TMP_ROOT/stderr.log" "oos-with" "newline-name out-of-allowlist file"
# byte単位の証明: 診断行は"oos-with"の直後で終わり(見た目のbackslash+nではない)、
# 次の行が"newline.txt"から始まることで、backslash+nの2文字ではなく実LFバイトが
# 埋め込まれていることを示す
newline_diag_line_no="$(grep -n 'oos-with' "$TMP_ROOT/stderr.log" | head -1 | cut -d: -f1)"
[[ -n "$newline_diag_line_no" ]] || fail "newline-name out-of-allowlist file: diagnostic line not found"
newline_diag_line="$(sed -n "${newline_diag_line_no}p" "$TMP_ROOT/stderr.log")"
case "$newline_diag_line" in
  *'\n'*) fail "newline-name out-of-allowlist file: diagnostic contains literal backslash-n instead of a real newline byte" ;;
esac
[[ "$newline_diag_line" == *"oos-with" ]] || fail "newline-name out-of-allowlist file: diagnostic line must end at the LF boundary"
newline_diag_next_line="$(sed -n "$((newline_diag_line_no + 1))p" "$TMP_ROOT/stderr.log")"
[[ "$newline_diag_next_line" == newline.txt* ]] || fail "newline-name out-of-allowlist file: expected an actual LF byte before newline.txt"

##############################################################################
# GR-D2 negative: ignore policyの書き換えで許可外変更を隠すe2e経路を塞ぐ
##############################################################################

wt="$(fresh_valid_worktree gitignore-root-star)"
order_file="$TMP_ROOT/order-gr-d2-gitignore-root-star.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-gitignore-root-star.sh"
hook_touch "$hook" \
  "printf '*\n' > \"$wt/.gitignore\"" \
  "echo hidden > \"$wt/hidden-oos.txt\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "root .gitignore rewritten to * to hide an out-of-allowlist file"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "root .gitignore rewritten to * to hide an out-of-allowlist file"
assert_has_fragment "$TMP_ROOT/stderr.log" "ignore policy changed" \
  "root .gitignore rewritten to * to hide an out-of-allowlist file"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "root .gitignore rewritten to *: Grok must not be called"
fi

wt="$(fresh_valid_worktree gitignore-nested-star)"
order_file="$TMP_ROOT/order-gr-d2-gitignore-nested-star.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-gitignore-nested-star.sh"
hook_touch "$hook" \
  "printf '*\n' > \"$wt/scripts/.gitignore\"" \
  "echo hidden > \"$wt/scripts/hidden-oos.txt\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "nested .gitignore rewritten to * to hide an out-of-allowlist file"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "nested .gitignore rewritten to * to hide an out-of-allowlist file"
assert_has_fragment "$TMP_ROOT/stderr.log" "ignore policy changed" \
  "nested .gitignore rewritten to * to hide an out-of-allowlist file"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "nested .gitignore rewritten to *: Grok must not be called"
fi

wt="$(fresh_valid_worktree info-exclude-star)"
order_file="$TMP_ROOT/order-gr-d2-info-exclude-star.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-info-exclude-star.sh"
hook_touch "$hook" \
  "printf '*\n' >> \"$wt/.git/info/exclude\"" \
  "echo hidden > \"$wt/hidden-oos.txt\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" ".git/info/exclude rewritten to * to hide an out-of-allowlist file"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" ".git/info/exclude rewritten to * to hide an out-of-allowlist file"
assert_has_fragment "$TMP_ROOT/stderr.log" "ignore policy changed" \
  ".git/info/exclude rewritten to * to hide an out-of-allowlist file"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail ".git/info/exclude rewritten to *: Grok must not be called"
fi

wt="$(fresh_valid_worktree reviewer-gitignore-star)"
order_file="$TMP_ROOT/order-gr-d2-reviewer-gitignore-star.md"
valid_order | write_order "$order_file"
grok_hook="$TMP_ROOT/hook-reviewer-gitignore-star.sh"
hook_touch "$grok_hook" \
  "printf '*\n' > \"$wt/.gitignore\"" \
  "echo hidden > \"$wt/hidden-oos.txt\""
run_execute "$wt" "$order_file" \
  FAKE_GROK_HOOK="$grok_hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 8 "$RUN_STATUS" "reviewer rewrites .gitignore to * during inspection invalidates ACCEPT"
assert_has_fragment "$TMP_ROOT/stderr.log" "INSPECT NG:" \
  "reviewer rewrites .gitignore to * during inspection invalidates ACCEPT"

##############################################################################
# GR-D2 negative: read-only検収者による変更はどのACCEPTも無効化する
##############################################################################

wt="$(fresh_valid_worktree reviewer-new-file)"
order_file="$TMP_ROOT/order-gr-d2-reviewer-new-file.md"
valid_order | write_order "$order_file"
grok_hook="$TMP_ROOT/hook-reviewer-new-file.sh"
hook_touch "$grok_hook" "echo reviewer > \"$wt/reviewer-created.txt\""
run_execute "$wt" "$order_file" \
  FAKE_GROK_HOOK="$grok_hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 8 "$RUN_STATUS" "reviewer creates a new file invalidates ACCEPT"
assert_has_fragment "$TMP_ROOT/stderr.log" "INSPECT NG:" "reviewer creates a new file invalidates ACCEPT"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "reviewer creates a new file also evidenced as scope violation"
assert_has_fragment "$TMP_ROOT/stderr.log" "reviewer-created.txt" "reviewer creates a new file also evidenced as scope violation"

wt="$(fresh_valid_worktree reviewer-ignored-file)"
printf 'target/\n' >"$wt/.gitignore"
git -C "$wt" add .gitignore
git -C "$wt" commit -qm "add ignored subtree"
reviewer_ignored_base_sha="$(git -C "$wt" rev-parse HEAD)"
order_file="$TMP_ROOT/order-gr-d2-reviewer-ignored.md"
valid_order | sed -E "s#^BASE_SHA: .*#BASE_SHA: $reviewer_ignored_base_sha#" | write_order "$order_file"
grok_hook="$TMP_ROOT/hook-reviewer-ignored.sh"
hook_touch "$grok_hook" "mkdir -p \"$wt/target/reviewer\"" "echo reviewer > \"$wt/target/reviewer/write.bin\""
run_execute "$wt" "$order_file" \
  FAKE_GROK_HOOK="$grok_hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 8 "$RUN_STATUS" "reviewer ignored-path write invalidates ACCEPT"
assert_has_fragment "$TMP_ROOT/stderr.log" "INSPECT NG:" "reviewer ignored-path write invalidates ACCEPT"

wt="$(fresh_valid_worktree reviewer-edit-tracked)"
order_file="$TMP_ROOT/order-gr-d2-reviewer-edit-tracked.md"
valid_order | write_order "$order_file"
terra_hook="$TMP_ROOT/hook-reviewer-edit-tracked-terra.sh"
hook_touch "$terra_hook" "echo '# terra edit' >> \"$wt/scripts/delegate-cursor-supervised.sh\""
grok_hook="$TMP_ROOT/hook-reviewer-edit-tracked-grok.sh"
hook_touch "$grok_hook" "echo '# reviewer edit' >> \"$wt/scripts/delegate-cursor-supervised.sh\""
run_execute "$wt" "$order_file" \
  FAKE_TERRA_HOOK="$terra_hook" \
  FAKE_GROK_HOOK="$grok_hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 8 "$RUN_STATUS" "reviewer edits an already-modified tracked file invalidates ACCEPT"
assert_has_fragment "$TMP_ROOT/stderr.log" "INSPECT NG:" "reviewer edits an already-modified tracked file invalidates ACCEPT"

wt="$(fresh_valid_worktree reviewer-symlink)"
ln -s original-target "$wt/tracked-link"
git -C "$wt" add tracked-link
git -C "$wt" commit -qm "add tracked symlink"
WT_SYMLINK_AGENTS_HASH="$(sha256_file "$wt/AGENTS.md")"
WT_SYMLINK_LEDGER_HASH="$(sha256_file "$wt/docs/implementation-ledger.md")"
WT_SYMLINK_BASE_SHA="$(git -C "$wt" rev-parse HEAD)"
order_file="$TMP_ROOT/order-gr-d2-reviewer-symlink.md"
cat >"$order_file" <<EOF
GRAIN: GR-D1
BASE_REF: refs/heads/$WT_VALID_BRANCH
BASE_SHA: $WT_SYMLINK_BASE_SHA
DEPENDENCY: U0e-2R
AUTHORITY: AGENTS.md SHA256:$WT_SYMLINK_AGENTS_HASH
AUTHORITY: docs/implementation-ledger.md SHA256:$WT_SYMLINK_LEDGER_HASH
ALLOWED_FILE: scripts/delegate-cursor-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: cursor-grok
SUPERVISOR_MODEL: cursor-grok-4.5-high
IMPLEMENTER_MODEL: gpt-5.6-terra
TASK_SHA256: $task_hash
CODEX PRECHECK: APPROVED
EOF
grok_hook="$TMP_ROOT/hook-reviewer-symlink.sh"
hook_touch "$grok_hook" "rm \"$wt/tracked-link\"" "ln -s changed-target \"$wt/tracked-link\""
run_execute "$wt" "$order_file" \
  FAKE_GROK_HOOK="$grok_hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 8 "$RUN_STATUS" "reviewer changes a symlink target invalidates ACCEPT"
assert_has_fragment "$TMP_ROOT/stderr.log" "INSPECT NG:" "reviewer changes a symlink target invalidates ACCEPT"

wt="$(fresh_valid_worktree reviewer-index-only)"
order_file="$TMP_ROOT/order-gr-d2-reviewer-index-only.md"
valid_order | write_order "$order_file"
terra_hook="$TMP_ROOT/hook-reviewer-index-only-terra.sh"
hook_touch "$terra_hook" "echo '# terra edit' >> \"$wt/scripts/delegate-cursor-supervised.sh\""
grok_hook="$TMP_ROOT/hook-reviewer-index-only-grok.sh"
hook_touch "$grok_hook" "git -C \"$wt\" add scripts/delegate-cursor-supervised.sh"
run_execute "$wt" "$order_file" \
  FAKE_TERRA_HOOK="$terra_hook" \
  FAKE_GROK_HOOK="$grok_hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 8 "$RUN_STATUS" "reviewer stages an already-modified allowed file without changing bytes invalidates ACCEPT"
assert_has_fragment "$TMP_ROOT/stderr.log" "INSPECT NG:" "reviewer stages an already-modified allowed file without changing bytes invalidates ACCEPT"

##############################################################################
# GR-D2 negative: inspect再開時のgate群
##############################################################################

wt="$(fresh_valid_worktree inspect-no-checkpoint)"
order_file="$TMP_ROOT/order-gr-d2-inspect-no-checkpoint.md"
valid_order | write_order "$order_file"
run_inspect "$wt" "$order_file"
assert_status 6 "$RUN_STATUS" "inspect with no prior execute has no checkpoint"
assert_has_fragment "$TMP_ROOT/stderr.log" "EVIDENCE NG:" "inspect with no prior execute has no checkpoint"
assert_no_external_calls "inspect with no prior execute has no checkpoint"

wt="$(fresh_valid_worktree inspect-failed-terra)"
order_file="$TMP_ROOT/order-gr-d2-inspect-failed-terra.md"
valid_order | write_order "$order_file"
run_execute "$wt" "$order_file" FAKE_TERRA_STATUS=1 FAKE_TERRA_OUTPUT="broken"
assert_status 1 "$RUN_STATUS" "execute with failing Terra preserves evidence"
evroot="$(evidence_root_for "$order_file")"
failed_attempt="$(latest_attempt_dir "$evroot")"
assert_file_exists "$failed_attempt/terra-stdout.txt" "execute with failing Terra preserves evidence"
[[ ! -f "$evroot/checkpoint.txt" ]] || fail "execute with failing Terra preserves evidence: checkpoint must not be written"
assert_has_fragment "$failed_attempt/stage-result.txt" "EXIT_STATUS: 1" \
  "execute with failing Terra records numeric exit status"
run_inspect "$wt" "$order_file"
assert_status 6 "$RUN_STATUS" "inspect after failed Terra has no checkpoint"
assert_has_fragment "$TMP_ROOT/stderr.log" "EVIDENCE NG:" "inspect after failed Terra has no checkpoint"
assert_no_external_calls "inspect after failed Terra has no checkpoint"

wt="$(fresh_valid_worktree inspect-terra-commit)"
order_file="$TMP_ROOT/order-gr-d2-inspect-terra-commit.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-terra-commit.sh"
hook_touch "$hook" \
  "echo committed >> \"$wt/scripts/delegate-cursor-supervised.sh\"" \
  "git -C \"$wt\" commit -qam terra-committed"
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 5 "$RUN_STATUS" "Terra-created commit preserves evidence and skips inspection"
evroot="$(evidence_root_for "$order_file")"
commit_attempt="$(latest_attempt_dir "$evroot")"
assert_file_exists "$commit_attempt/terra-stdout.txt" "Terra-created commit preserves evidence and skips inspection"
assert_file_exists "$commit_attempt/post-terra-status.txt" "Terra-created commit preserves evidence and skips inspection"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "Terra-created commit preserves evidence and skips inspection: Grok must not be called"
fi

wt="$(fresh_valid_worktree inspect-task-mismatch)"
order_file="$TMP_ROOT/order-gr-d2-inspect-task-mismatch.md"
valid_order | write_order "$order_file"
run_execute "$wt" "$order_file" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "execute before task-mismatch inspect"
: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CODEX_DELEGATED PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
    CURSOR_INSPECTION_TIMEOUT_SECONDS=5 \
    "$SCRIPT" inspect "$wt" "$order_file" "a different task string" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
  RUN_STATUS=0
else
  RUN_STATUS=$?
fi
assert_status 3 "$RUN_STATUS" "inspect with mismatched task"
assert_has_fragment "$TMP_ROOT/stderr.log" "発注書とtaskが一致しません" "inspect with mismatched task"
assert_no_external_calls "inspect with mismatched task"

wt="$(fresh_valid_worktree inspect-head-drift)"
order_file="$TMP_ROOT/order-gr-d2-inspect-head-drift.md"
valid_order | write_order "$order_file"
run_execute "$wt" "$order_file" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "execute before head-drift inspect"
echo unrelated > "$wt/unrelated-untracked.txt"
git -C "$wt" add unrelated-untracked.txt
git -C "$wt" commit -qm "unrelated head drift"
run_inspect "$wt" "$order_file"
assert_status 6 "$RUN_STATUS" "inspect after worktree HEAD drifted"
assert_has_fragment "$TMP_ROOT/stderr.log" "EVIDENCE NG:" "inspect after worktree HEAD drifted"
assert_no_external_calls "inspect after worktree HEAD drifted"

wt="$(fresh_valid_worktree inspect-diff-changed)"
order_file="$TMP_ROOT/order-gr-d2-inspect-diff-changed.md"
valid_order | write_order "$order_file"
run_execute "$wt" "$order_file" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "execute before diff-changed inspect"
echo "# post-checkpoint edit" >> "$wt/scripts/delegate-cursor-supervised.sh"
run_inspect "$wt" "$order_file"
assert_status 6 "$RUN_STATUS" "inspect after implementation diff changed post-checkpoint"
assert_has_fragment "$TMP_ROOT/stderr.log" "EVIDENCE NG:" "inspect after implementation diff changed post-checkpoint"
assert_no_external_calls "inspect after implementation diff changed post-checkpoint"

# 承認済みorder本文がexecute成功後に(markerを保ったまま)書き換わった場合、checkpointの
# ORDER_SHA256と一致しなくなることを独立に確認する
wt="$(fresh_valid_worktree inspect-order-drift)"
order_file="$TMP_ROOT/order-gr-d2-inspect-order-drift.md"
valid_order | write_order "$order_file"
run_execute "$wt" "$order_file" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "execute before order-drift inspect"
{
  valid_order | sed '/^ORDER: READY/i\
NOTE: order text mutated after approval
'
} | write_order "$order_file"
run_inspect "$wt" "$order_file"
assert_status 6 "$RUN_STATUS" "inspect after approved order text drifted"
assert_has_fragment "$TMP_ROOT/stderr.log" "EVIDENCE NG:" "inspect after approved order text drifted"
assert_has_fragment "$TMP_ROOT/stderr.log" "approved order drifted from checkpoint" "inspect after approved order text drifted"
assert_no_external_calls "inspect after approved order text drifted"

# BASE_REF/BASE_SHA checkpoint検証の分岐を、order hash一致を経由してでも独立に踏むための
# 試験: checkpointのORDER_SHA256だけを変更後のorderへ合わせ、BASE_SHAは古いまま残す
wt="$(fresh_valid_worktree inspect-base-drift)"
order_file="$TMP_ROOT/order-gr-d2-inspect-base-drift.md"
valid_order | write_order "$order_file"
run_execute "$wt" "$order_file" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "execute before base-drift inspect"
valid_order | sed -E "s#^BASE_SHA: .*#BASE_SHA: $OTHER_BRANCH_SHA#" | write_order "$order_file"
evroot="$(evidence_root_for "$order_file")"
mutated_order_sha256="$(shasum -a 256 "$order_file" | awk '{print $1}')"
sed -E "s#^ORDER_SHA256: .*#ORDER_SHA256: $mutated_order_sha256#" "$evroot/checkpoint.txt" \
  >"$evroot/checkpoint.txt.new"
mv "$evroot/checkpoint.txt.new" "$evroot/checkpoint.txt"
run_inspect "$wt" "$order_file"
assert_status 6 "$RUN_STATUS" "inspect after BASE_SHA drifted while checkpoint order hash was realigned"
assert_has_fragment "$TMP_ROOT/stderr.log" "EVIDENCE NG:" "inspect after BASE_SHA drifted while checkpoint order hash was realigned"
assert_has_fragment "$TMP_ROOT/stderr.log" "BASE_SHA drifted from checkpoint" "inspect after BASE_SHA drifted while checkpoint order hash was realigned"
assert_no_external_calls "inspect after BASE_SHA drifted while checkpoint order hash was realigned"

# checkpointが(仮に)一致しても、scope closureがinspect側でも独立に再確認されることを
# 確認する防御的試験: 先に正規のexecuteでATTEMPT紐付き済みcheckpointを作らせ、
# その後checkpointのFINGERPRINTだけを許可外untracked込みの値へ手動で合わせる。
# 許可外untrackedを残した状態でinspectしてもSCOPE NGでGrok未起動になる
wt="$(fresh_valid_worktree inspect-scope-defense)"
order_file="$TMP_ROOT/order-gr-d2-inspect-scope-defense.md"
valid_order | write_order "$order_file"
run_execute "$wt" "$order_file" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "execute before inspect scope-defense forgery"
evroot="$(evidence_root_for "$order_file")"
defense_attempt="$(basename "$(latest_attempt_dir "$evroot")")"

echo out-of-scope > "$wt/oos-defense.txt"
manual_fp_list="$TMP_ROOT/manual-fp-list"
: >"$manual_fp_list"
while IFS= read -r -d '' p; do
  full="$wt/$p"
  if [[ -L "$full" ]]; then h="$(readlink "$full" | shasum -a 256 | awk '{print $1}')"; m="120000"
  elif [[ -f "$full" ]]; then
    h="$(shasum -a 256 "$full" | awk '{print $1}')"
    if [[ -x "$full" ]]; then m="100755"; else m="100644"; fi
  else h="$(printf '' | shasum -a 256 | awk '{print $1}')"; m=""
  fi
  printf '%s%s%s\0' "$p" "$h" "$m" >>"$manual_fp_list"
done < <(git -C "$wt" ls-files -z --cached --others --exclude-standard | LC_ALL=C sort -z)
git -C "$wt" status --porcelain=v2 -z --untracked-files=all --no-renames | LC_ALL=C sort -z >>"$manual_fp_list"
git -C "$wt" ls-files -z -v | LC_ALL=C sort -z >>"$manual_fp_list"
# compute_ignore_policy_hashと同じ手順を再現する(fresh worktreeには.gitignore/
# .gitattributes/info/exclude/info/attributes/core.excludesFile/core.attributesFileが
# 無いため、既定global excludesとlocal configだけを反映した値になる)
manual_ignore_list="$TMP_ROOT/manual-ignore-list"
: >"$manual_ignore_list"
while IFS= read -r -d '' p; do
  case "$p" in
    .gitignore|*/.gitignore|.gitattributes|*/.gitattributes) ;;
    *) continue ;;
  esac
  full="$wt/$p"
  if [[ -L "$full" ]]; then ih="$(readlink "$full" | shasum -a 256 | awk '{print $1}')"
  elif [[ -f "$full" ]]; then ih="$(shasum -a 256 "$full" | awk '{print $1}')"
  else ih="$(printf '' | shasum -a 256 | awk '{print $1}')"
  fi
  printf 'control-file:%s:%s\0' "$p" "$ih" >>"$manual_ignore_list"
done < <(git -C "$wt" ls-files -z --cached --others | LC_ALL=C sort -z)
manual_common_dir="$(git -C "$wt" rev-parse --git-common-dir)"
case "$manual_common_dir" in
  /*) : ;;
  *) manual_common_dir="$wt/$manual_common_dir" ;;
esac
manual_info_exclude="$manual_common_dir/info/exclude"
if [[ -f "$manual_info_exclude" ]]; then
  ih="$(shasum -a 256 "$manual_info_exclude" | awk '{print $1}')"
else
  ih="$(printf '' | shasum -a 256 | awk '{print $1}')"
fi
printf 'info-exclude:%s\0' "$ih" >>"$manual_ignore_list"
manual_info_attrs="$manual_common_dir/info/attributes"
if [[ -f "$manual_info_attrs" ]]; then
  ih="$(shasum -a 256 "$manual_info_attrs" | awk '{print $1}')"
else
  ih="$(printf '' | shasum -a 256 | awk '{print $1}')"
fi
printf 'info-attributes:%s\0' "$ih" >>"$manual_ignore_list"
manual_excludes_file="$(git -C "$wt" config --get core.excludesFile 2>/dev/null || true)"
if [[ -n "$manual_excludes_file" ]]; then
  manual_resolved="$manual_excludes_file"
  case "$manual_resolved" in
    "~/"*) manual_resolved="$HOME/${manual_resolved#\~/}" ;;
    /*) : ;;
    *) manual_resolved="$wt/$manual_resolved" ;;
  esac
  if [[ -f "$manual_resolved" ]]; then
    ih="$(shasum -a 256 "$manual_resolved" | awk '{print $1}')"
  else
    ih="$(printf '' | shasum -a 256 | awk '{print $1}')"
  fi
  printf 'core-excludesFile:%s:%s\0' "$manual_excludes_file" "$ih" >>"$manual_ignore_list"
else
  manual_global_excludes="${XDG_CONFIG_HOME:-$HOME/.config}/git/ignore"
  if [[ -f "$manual_global_excludes" ]]; then
    ih="$(shasum -a 256 "$manual_global_excludes" | awk '{print $1}')"
  else
    ih="$(printf '' | shasum -a 256 | awk '{print $1}')"
  fi
  printf 'default-global-excludes:%s:%s\0' "$manual_global_excludes" "$ih" >>"$manual_ignore_list"
fi
manual_attrs_file="$(git -C "$wt" config --get core.attributesFile 2>/dev/null || true)"
if [[ -n "$manual_attrs_file" ]]; then
  manual_resolved="$manual_attrs_file"
  case "$manual_resolved" in
    "~/"*) manual_resolved="$HOME/${manual_resolved#\~/}" ;;
    /*) : ;;
    *) manual_resolved="$wt/$manual_resolved" ;;
  esac
  if [[ -f "$manual_resolved" ]]; then
    ih="$(shasum -a 256 "$manual_resolved" | awk '{print $1}')"
  else
    ih="$(printf '' | shasum -a 256 | awk '{print $1}')"
  fi
  printf 'core-attributesFile:%s:%s\0' "$manual_attrs_file" "$ih" >>"$manual_ignore_list"
else
  printf 'core-attributesFile:unset\0' >>"$manual_ignore_list"
fi
printf 'local-config:%s\0' "$(git -C "$wt" config --local --list 2>/dev/null | LC_ALL=C sort | shasum -a 256 | awk '{print $1}')" >>"$manual_ignore_list"
if git -C "$wt" config --worktree --list >/dev/null 2>&1; then
  printf 'worktree-config:%s\0' "$(git -C "$wt" config --worktree --list 2>/dev/null | LC_ALL=C sort | shasum -a 256 | awk '{print $1}')" >>"$manual_ignore_list"
fi
manual_ignore_hash="$(shasum -a 256 "$manual_ignore_list" | awk '{print $1}')"
printf 'ignore-policy:%s\0' "$manual_ignore_hash" >>"$manual_fp_list"
manual_fp="$(shasum -a 256 "$manual_fp_list" | awk '{print $1}')"
order_sha256="$(shasum -a 256 "$order_file" | awk '{print $1}')"
wt_head="$(git -C "$wt" rev-parse HEAD)"
{
  echo "ATTEMPT: $defense_attempt"
  echo "ORDER_SHA256: $order_sha256"
  echo "TASK_SHA256: $task_hash"
  echo "BASE_REF: refs/heads/$WT_VALID_BRANCH"
  echo "BASE_SHA: $WT_VALID_BASE_SHA"
  echo "HEAD: $wt_head"
  echo "FINGERPRINT: $manual_fp"
} >"$evroot/checkpoint.txt"
run_inspect "$wt" "$order_file"
assert_status 7 "$RUN_STATUS" "inspect independently re-checks scope closure even when checkpoint matches"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "inspect independently re-checks scope closure even when checkpoint matches"
assert_has_fragment "$TMP_ROOT/stderr.log" "oos-defense.txt" "inspect independently re-checks scope closure even when checkpoint matches"
assert_no_external_calls "inspect independently re-checks scope closure even when checkpoint matches"

##############################################################################
# GR-D2 negative: model側による承認済みorder file自体の改変は、evidenceを残した上で
# fail closedし、後続stageの起動/採用より前に止める。order fileは通常worktree外に
# あるためworktree fingerprintだけでは検知できず、pre-model hashとの独立照合が要る
##############################################################################

wt="$(fresh_valid_worktree terra-order-mutate)"
order_file="$TMP_ROOT/order-gr-d2-terra-order-mutate.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-terra-order-mutate.sh"
hook_touch "$hook" "echo '# terra tampered with the approved order' >> \"$order_file\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 6 "$RUN_STATUS" "Terra mutating the approved order fails closed"
assert_has_fragment "$TMP_ROOT/stderr.log" "EVIDENCE NG:" "Terra mutating the approved order fails closed"
assert_has_fragment "$TMP_ROOT/stderr.log" "approved order mutated during terra implementation" \
  "Terra mutating the approved order fails closed"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "Terra mutating the approved order fails closed: Grok must not be called"
fi

wt="$(fresh_valid_worktree grok-order-mutate)"
order_file="$TMP_ROOT/order-gr-d2-grok-order-mutate.md"
valid_order | write_order "$order_file"
grok_hook="$TMP_ROOT/hook-grok-order-mutate.sh"
hook_touch "$grok_hook" "echo '# grok tampered with the approved order' >> \"$order_file\""
run_execute "$wt" "$order_file" \
  FAKE_GROK_HOOK="$grok_hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 6 "$RUN_STATUS" "Grok mutating the approved order invalidates acceptance"
assert_has_fragment "$TMP_ROOT/stderr.log" "EVIDENCE NG:" "Grok mutating the approved order invalidates acceptance"
assert_has_fragment "$TMP_ROOT/stderr.log" "approved order mutated during grok inspection" \
  "Grok mutating the approved order invalidates acceptance"

##############################################################################
# GR-D2 negative: 不正なinspection timeoutはExternal呼び出しより前に失敗する
##############################################################################

wt="$(fresh_valid_worktree bad-inspection-timeout-zero)"
order_file="$TMP_ROOT/order-gr-d2-bad-timeout-zero.md"
valid_order | write_order "$order_file"
run_execute "$wt" "$order_file" CURSOR_INSPECTION_TIMEOUT_SECONDS=0
assert_status 2 "$RUN_STATUS" "zero inspection timeout rejected before any External call"
assert_has_fragment "$TMP_ROOT/stderr.log" "timeout/heartbeatは正の整数で指定してください" "zero inspection timeout rejected before any External call"
assert_no_external_calls "zero inspection timeout rejected before any External call"

wt="$(fresh_valid_worktree bad-inspection-timeout-word)"
order_file="$TMP_ROOT/order-gr-d2-bad-timeout-word.md"
valid_order | write_order "$order_file"
run_execute "$wt" "$order_file" CURSOR_INSPECTION_TIMEOUT_SECONDS=abc
assert_status 2 "$RUN_STATUS" "non-numeric inspection timeout rejected before any External call"
assert_has_fragment "$TMP_ROOT/stderr.log" "timeout/heartbeatは正の整数で指定してください" "non-numeric inspection timeout rejected before any External call"
assert_no_external_calls "non-numeric inspection timeout rejected before any External call"

##############################################################################
# GR-D2: 呼び出し側相対のorder pathでも、evidenceはそのorder fileの絶対path脇に残り、
# worktree内には作られない
##############################################################################

wt="$(fresh_valid_worktree relative-order-path)"
rel_dir="$TMP_ROOT/relative-order-dir"
mkdir -p "$rel_dir"
valid_order | write_order "$rel_dir/order-gr-d2-relative.md"
: >"$CALL_LOG"
if (cd "$rel_dir" && env -u CURSOR_AGENT -u CODEX_DELEGATED \
      PATH="$FAKE_BIN:/usr/bin:/bin" \
      FAKE_CALL_LOG="$CALL_LOG" \
      CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
      CURSOR_SUPERVISED_TIMEOUT_SECONDS=5 \
      CODEX_TERRA_TIMEOUT_SECONDS=5 \
      CURSOR_INSPECTION_TIMEOUT_SECONDS=5 \
      FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT' \
      "$SCRIPT" execute "$wt" "order-gr-d2-relative.md" "$task" \
      >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"); then
  RUN_STATUS=0
else
  RUN_STATUS=$?
fi
assert_status 0 "$RUN_STATUS" "relative order path dispatch"
assert_has_fragment "$CALL_LOG" "--model cursor-grok-4.5-high" "relative order path dispatch reaches Grok"
rel_evroot="$(evidence_root_for "$rel_dir/order-gr-d2-relative.md")"
assert_file_exists "$rel_evroot/checkpoint.txt" "relative order path evidence resolves beside the absolute order path"
rel_attempt="$(latest_attempt_dir "$rel_evroot")"
[[ -n "$rel_attempt" ]] || fail "relative order path evidence: attempt dir missing"
assert_file_exists "$rel_attempt/order.txt" "relative order path evidence attempt contents"
if [[ -e "$wt/order-gr-d2-relative.md.evidence" ]]; then
  fail "relative order path evidence: must not be created inside the target worktree"
fi

##############################################################################
# GR-D2 negative: assume-unchanged/skip-worktree bitはgit status由来のscope
# 検知を迂回できない。dispatch前にbitを立て、bitを残したままTerraが許可外
# tracked authority fileへ手を入れてもSCOPE NGでGrok未起動になることを確認する
##############################################################################

wt="$(fresh_valid_worktree scope-assume-unchanged)"
git -C "$wt" update-index --assume-unchanged AGENTS.md
order_file="$TMP_ROOT/order-gr-d2-scope-assume-unchanged.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-scope-assume-unchanged.sh"
hook_touch "$hook" "echo extra >> \"$wt/AGENTS.md\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "assume-unchanged bit cannot hide an out-of-allowlist tracked edit"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "assume-unchanged bit cannot hide an out-of-allowlist tracked edit"
assert_has_fragment "$TMP_ROOT/stderr.log" "AGENTS.md" "assume-unchanged bit cannot hide an out-of-allowlist tracked edit"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "assume-unchanged bit cannot hide an out-of-allowlist tracked edit: Grok must not be called"
fi

wt="$(fresh_valid_worktree scope-skip-worktree)"
git -C "$wt" update-index --skip-worktree AGENTS.md
order_file="$TMP_ROOT/order-gr-d2-scope-skip-worktree.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-scope-skip-worktree.sh"
hook_touch "$hook" "echo extra >> \"$wt/AGENTS.md\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "skip-worktree bit cannot hide an out-of-allowlist tracked edit"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "skip-worktree bit cannot hide an out-of-allowlist tracked edit"
assert_has_fragment "$TMP_ROOT/stderr.log" "AGENTS.md" "skip-worktree bit cannot hide an out-of-allowlist tracked edit"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "skip-worktree bit cannot hide an out-of-allowlist tracked edit: Grok must not be called"
fi

# 親indexのgitlinkへhide bitがあっても、submodule内部のraw tracked editを直接拾う。
submodule_source="$TMP_ROOT/gitlink-source"
mkdir -p "$submodule_source"
git_init_repo "$submodule_source"
printf 'submodule baseline\n' >"$submodule_source/tracked.txt"
git -C "$submodule_source" add tracked.txt
git -C "$submodule_source" commit -qm init

wt="$(fresh_valid_worktree scope-gitlink-clean)"
git -C "$wt" -c protocol.file.allow=always submodule add -q "$submodule_source" vendor/sub
git -C "$wt" commit -qm "add clean gitlink"
gitlink_base_sha="$(git -C "$wt" rev-parse HEAD)"
order_file="$TMP_ROOT/order-gr-d2-scope-gitlink-clean.md"
valid_order | sed -E "s#^BASE_SHA: .*#BASE_SHA: $gitlink_base_sha#" | write_order "$order_file"
run_execute "$wt" "$order_file" FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "clean out-of-allowlist gitlink remains admissible"

wt="$(fresh_valid_worktree scope-gitlink-hidden-dirty)"
git -C "$wt" -c protocol.file.allow=always submodule add -q "$submodule_source" vendor/sub
git -C "$wt" commit -qm "add clean gitlink"
gitlink_base_sha="$(git -C "$wt" rev-parse HEAD)"
git -C "$wt" update-index --skip-worktree vendor/sub
printf 'hidden mutation\n' >>"$wt/vendor/sub/tracked.txt"
order_file="$TMP_ROOT/order-gr-d2-scope-gitlink-hidden-dirty.md"
valid_order | sed -E "s#^BASE_SHA: .*#BASE_SHA: $gitlink_base_sha#" | write_order "$order_file"
run_execute "$wt" "$order_file"
assert_status 3 "$RUN_STATUS" "gitlink hide bit cannot conceal a pre-existing nested edit"
assert_has_fragment "$TMP_ROOT/stderr.log" "ORDER-GATE NG:" "gitlink hide bit nested edit gate"
assert_has_fragment "$TMP_ROOT/stderr.log" "vendor/sub" "gitlink hide bit nested edit path"
assert_no_external_calls "gitlink hide bit nested edit"

wt="$(fresh_valid_worktree scope-gitlink-terra-mutation)"
git -C "$wt" -c protocol.file.allow=always submodule add -q "$submodule_source" vendor/sub
git -C "$wt" commit -qm "add clean gitlink"
gitlink_base_sha="$(git -C "$wt" rev-parse HEAD)"
order_file="$TMP_ROOT/order-gr-d2-scope-gitlink-terra-mutation.md"
valid_order | sed -E "s#^BASE_SHA: .*#BASE_SHA: $gitlink_base_sha#" | write_order "$order_file"
hook="$TMP_ROOT/hook-scope-gitlink-terra-mutation.sh"
hook_touch "$hook" "printf 'terra mutation\\n' >> \"$wt/vendor/sub/tracked.txt\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "Terra cannot mutate an out-of-allowlist gitlink subtree"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "Terra gitlink mutation scope closure"
assert_has_fragment "$TMP_ROOT/stderr.log" "vendor/sub" "Terra gitlink mutation path"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "Terra gitlink mutation: Grok must not be called"
fi

printf 'target/\n' >"$submodule_source/.gitignore"
git -C "$submodule_source" add .gitignore
git -C "$submodule_source" commit -qm "ignore generated subtree"
wt="$(fresh_valid_worktree scope-gitlink-hidden-ignored)"
git -C "$wt" -c protocol.file.allow=always submodule add -q "$submodule_source" vendor/sub
git -C "$wt" commit -qm "add gitlink with ignored subtree"
gitlink_base_sha="$(git -C "$wt" rev-parse HEAD)"
git -C "$wt" update-index --skip-worktree vendor/sub
mkdir -p "$wt/vendor/sub/target/private"
printf 'ignored payload\n' >"$wt/vendor/sub/target/private/payload.bin"
order_file="$TMP_ROOT/order-gr-d2-scope-gitlink-hidden-ignored.md"
valid_order | sed -E "s#^BASE_SHA: .*#BASE_SHA: $gitlink_base_sha#" | write_order "$order_file"
run_execute "$wt" "$order_file"
assert_status 3 "$RUN_STATUS" "gitlink ignored directory cannot conceal a pre-existing payload"
assert_has_fragment "$TMP_ROOT/stderr.log" "ORDER-GATE NG:" "gitlink ignored directory gate"
assert_has_fragment "$TMP_ROOT/stderr.log" "vendor/sub" "gitlink ignored directory path"
assert_no_external_calls "gitlink ignored directory payload"

##############################################################################
# GR-D2負担分再修正: assume-unchanged/skip-worktree bitはblob shaに現れない
# chmodだけの変更(内容は同一)も隠せない。実効modeをindex記録modeと直接
# 比較していることを確認する
##############################################################################

wt="$(fresh_valid_worktree scope-assume-unchanged-chmod)"
git -C "$wt" update-index --assume-unchanged AGENTS.md
order_file="$TMP_ROOT/order-gr-d2-scope-assume-unchanged-chmod.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-scope-assume-unchanged-chmod.sh"
hook_touch "$hook" "chmod +x \"$wt/AGENTS.md\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "assume-unchanged bit cannot hide a mode-only out-of-allowlist edit"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "assume-unchanged bit cannot hide a mode-only out-of-allowlist edit"
assert_has_fragment "$TMP_ROOT/stderr.log" "AGENTS.md" "assume-unchanged bit cannot hide a mode-only out-of-allowlist edit"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "assume-unchanged bit cannot hide a mode-only out-of-allowlist edit: Grok must not be called"
fi

wt="$(fresh_valid_worktree scope-skip-worktree-chmod)"
git -C "$wt" update-index --skip-worktree AGENTS.md
order_file="$TMP_ROOT/order-gr-d2-scope-skip-worktree-chmod.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-scope-skip-worktree-chmod.sh"
hook_touch "$hook" "chmod +x \"$wt/AGENTS.md\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "skip-worktree bit cannot hide a mode-only out-of-allowlist edit"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "skip-worktree bit cannot hide a mode-only out-of-allowlist edit"
assert_has_fragment "$TMP_ROOT/stderr.log" "AGENTS.md" "skip-worktree bit cannot hide a mode-only out-of-allowlist edit"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "skip-worktree bit cannot hide a mode-only out-of-allowlist edit: Grok must not be called"
fi

wt="$(fresh_valid_worktree reviewer-chmod-only)"
order_file="$TMP_ROOT/order-gr-d2-reviewer-chmod-only.md"
valid_order | write_order "$order_file"
terra_hook="$TMP_ROOT/hook-reviewer-chmod-only-terra.sh"
hook_touch "$terra_hook" "echo '# terra edit' >> \"$wt/scripts/delegate-cursor-supervised.sh\""
grok_hook="$TMP_ROOT/hook-reviewer-chmod-only-grok.sh"
hook_touch "$grok_hook" "chmod +x \"$wt/scripts/delegate-cursor-supervised.sh\""
run_execute "$wt" "$order_file" \
  FAKE_TERRA_HOOK="$terra_hook" \
  FAKE_GROK_HOOK="$grok_hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 8 "$RUN_STATUS" "reviewer chmod-only mutation of an allowed file invalidates ACCEPT"
assert_has_fragment "$TMP_ROOT/stderr.log" "INSPECT NG:" "reviewer chmod-only mutation of an allowed file invalidates ACCEPT"

##############################################################################
# GR-D2負担分再修正: ignore policyのbefore値は、Terra起動前にparent shell
# 変数として確保する。Terraのbash toolがevidence_root配下のpre-terra
# 証跡fileを削除/改変しても、判定はその変数だけを基準にするため揺らがない
##############################################################################

wt="$(fresh_valid_worktree ignore-policy-evidence-tamper)"
order_file="$TMP_ROOT/order-gr-d2-ignore-policy-evidence-tamper.md"
valid_order | write_order "$order_file"
ipet_evroot="$(evidence_root_for "$order_file")"
hook="$TMP_ROOT/hook-ignore-policy-evidence-tamper.sh"
hook_touch "$hook" \
  "printf '*\\n' > \"$wt/.gitignore\"" \
  "echo hidden > \"$wt/hidden-oos.txt\"" \
  "rm -f \"$ipet_evroot\"/attempt-*/pre-terra-ignore-policy.sha256"
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" \
  "Terra cannot defeat the ignore-policy guard by deleting the persisted pre-terra evidence file"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" \
  "Terra cannot defeat the ignore-policy guard by deleting the persisted pre-terra evidence file"
assert_has_fragment "$TMP_ROOT/stderr.log" "ignore policy changed" \
  "Terra cannot defeat the ignore-policy guard by deleting the persisted pre-terra evidence file"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "Terra cannot defeat the ignore-policy guard by deleting the persisted pre-terra evidence file: Grok must not be called"
fi

##############################################################################
# GR-D2負担分再修正: scope判定に使うgit列挙の失敗(process substitutionで
# statusを握りつぶさない)は、空集合成功へ素通りさせずSCOPE NGでfail closedする
##############################################################################

REAL_GIT_BIN="$(command -v git)"
GIT_FAIL_BIN="$TMP_ROOT/bin-git-fail"
mkdir -p "$GIT_FAIL_BIN"
cat >"$GIT_FAIL_BIN/git" <<EOF
#!/usr/bin/env bash
set -euo pipefail
for a in "\$@"; do
  if [[ "\$a" == "-s" ]]; then
    echo "injected git enumeration failure" >&2
    exit 1
  fi
done
exec "$REAL_GIT_BIN" "\$@"
EOF
chmod +x "$GIT_FAIL_BIN/git"

wt="$(fresh_valid_worktree scope-enumeration-failure)"
order_file="$TMP_ROOT/order-gr-d2-scope-enum-fail.md"
valid_order | write_order "$order_file"
: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CODEX_DELEGATED \
    PATH="$GIT_FAIL_BIN:$FAKE_BIN:/usr/bin:/bin" \
    FAKE_CALL_LOG="$CALL_LOG" \
    CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
    CURSOR_SUPERVISED_TIMEOUT_SECONDS=5 \
    CODEX_TERRA_TIMEOUT_SECONDS=5 \
    CURSOR_INSPECTION_TIMEOUT_SECONDS=5 \
    "$SCRIPT" execute "$wt" "$order_file" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
  RUN_STATUS=0
else
  RUN_STATUS=$?
fi
assert_status 7 "$RUN_STATUS" "injected git ls-files -s enumeration failure fails closed rather than an empty successful scope set"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "injected git ls-files -s enumeration failure fails closed"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "injected git ls-files -s enumeration failure: Grok must not be called"
fi

wt="$(fresh_valid_worktree reviewer-index-bit-only)"
order_file="$TMP_ROOT/order-gr-d2-reviewer-index-bit-only.md"
valid_order | write_order "$order_file"
terra_hook="$TMP_ROOT/hook-reviewer-index-bit-only-terra.sh"
hook_touch "$terra_hook" "echo '# terra edit' >> \"$wt/scripts/delegate-cursor-supervised.sh\""
grok_hook="$TMP_ROOT/hook-reviewer-index-bit-only-grok.sh"
hook_touch "$grok_hook" "git -C \"$wt\" update-index --assume-unchanged scripts/delegate-cursor-supervised.sh"
run_execute "$wt" "$order_file" \
  FAKE_TERRA_HOOK="$terra_hook" \
  FAKE_GROK_HOOK="$grok_hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 8 "$RUN_STATUS" "reviewer sets an index bit without changing bytes invalidates ACCEPT"
assert_has_fragment "$TMP_ROOT/stderr.log" "INSPECT NG:" "reviewer sets an index bit without changing bytes invalidates ACCEPT"

##############################################################################
# GR-D2 negative: 相対core.excludesFileはrunnerの起動cwdではなく対象worktreeから
# 解決する。worktree外から起動し、既存の相対excludes fileを書き換えて自分自身と
# 許可外pathの両方を隠そうとしてもSCOPE NGでGrok未起動になることを確認する
##############################################################################

wt="$(fresh_valid_worktree relative-excludes)"
printf 'hide-oos.txt\n' >"$wt/myexcludes"
git -C "$wt" add myexcludes
git -C "$wt" commit -qm "add relative excludes file"
git -C "$wt" config core.excludesFile myexcludes
RE_BASE_SHA="$(git -C "$wt" rev-parse HEAD)"
order_file="$TMP_ROOT/order-gr-d2-relative-excludes.md"
cat >"$order_file" <<EOF
GRAIN: GR-D1
BASE_REF: refs/heads/$WT_VALID_BRANCH
BASE_SHA: $RE_BASE_SHA
DEPENDENCY: U0e-2R
AUTHORITY: AGENTS.md SHA256:$WT_VALID_AGENTS_HASH
AUTHORITY: docs/implementation-ledger.md SHA256:$WT_VALID_LEDGER_HASH
ALLOWED_FILE: scripts/delegate-cursor-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: cursor-grok
SUPERVISOR_MODEL: cursor-grok-4.5-high
IMPLEMENTER_MODEL: gpt-5.6-terra
TASK_SHA256: $task_hash
CODEX PRECHECK: APPROVED
EOF
hook="$TMP_ROOT/hook-relative-excludes.sh"
hook_touch "$hook" \
  "printf 'hide-oos.txt\\nmyexcludes\\n' > \"$wt/myexcludes\"" \
  "echo hidden > \"$wt/hide-oos.txt\""
: >"$CALL_LOG"
if (cd "$ROOT_DIR" && env -u CURSOR_AGENT -u CODEX_DELEGATED \
      PATH="$FAKE_BIN:/usr/bin:/bin" \
      FAKE_CALL_LOG="$CALL_LOG" \
      FAKE_TERRA_HOOK="$hook" \
      CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
      CURSOR_SUPERVISED_TIMEOUT_SECONDS=5 \
      CODEX_TERRA_TIMEOUT_SECONDS=5 \
      CURSOR_INSPECTION_TIMEOUT_SECONDS=5 \
      "$SCRIPT" execute "$wt" "$order_file" "$task" \
      >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"); then
  RUN_STATUS=0
else
  RUN_STATUS=$?
fi
assert_status 7 "$RUN_STATUS" \
  "Terra rewriting a relative core.excludesFile to hide itself and an out-of-allowlist path fails closed"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" \
  "Terra rewriting a relative core.excludesFile to hide itself and an out-of-allowlist path fails closed"
assert_has_fragment "$TMP_ROOT/stderr.log" "ignore policy changed" \
  "Terra rewriting a relative core.excludesFile to hide itself and an out-of-allowlist path fails closed"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "Terra rewriting a relative core.excludesFile: Grok must not be called"
fi

##############################################################################
# GR-D2十次修正: raw manifestのmode/symlink target hash具体的欠陥修正を、
# manifest evidence file自体のhash比較で直接検証する(古いfilter/status/mode
# defenseがgreenでも、manifestが壊れていれば見逃す設計にしない)
##############################################################################

# chmodのみの許可外変更は、index記録modeではなく実効worktree modeを見ないと
# manifest digestへ反映されない
wt="$(fresh_valid_worktree manifest-mode-only)"
order_file="$TMP_ROOT/order-gr-d2-manifest-mode-only.md"
valid_order | write_order "$order_file"
hook="$TMP_ROOT/hook-manifest-mode-only.sh"
hook_touch "$hook" "chmod +x \"$wt/AGENTS.md\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "manifest mode-only chmod attack fails closed"
manifest_mode_evroot="$(evidence_root_for "$order_file")"
manifest_mode_attempt="$(latest_attempt_dir "$manifest_mode_evroot")"
[[ -f "$manifest_mode_attempt/pre-terra-out-of-scope-manifest.nul" ]] \
  || fail "manifest mode-only: pre manifest evidence missing"
[[ -f "$manifest_mode_attempt/post-terra-out-of-scope-manifest.nul" ]] \
  || fail "manifest mode-only: post manifest evidence missing"
manifest_mode_pre_hash="$(shasum -a 256 "$manifest_mode_attempt/pre-terra-out-of-scope-manifest.nul" | awk '{print $1}')"
manifest_mode_post_hash="$(shasum -a 256 "$manifest_mode_attempt/post-terra-out-of-scope-manifest.nul" | awk '{print $1}')"
[[ "$manifest_mode_pre_hash" != "$manifest_mode_post_hash" ]] \
  || fail "manifest mode-only: chmod-only out-of-allowlist change must change the raw manifest digest"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "manifest mode-only: Grok must not be called"
fi

# symlink targetが末尾に実LFを1byte追加しただけの変更でも、$(readlink ...)の
# trailing newline剥ぎ取りに埋もれずmanifest digestが変わることを確認する
wt="$(fresh_valid_worktree manifest-symlink-lf)"
ln -s same-target "$wt/tracked-link"
git -C "$wt" add tracked-link
git -C "$wt" commit -qm "add tracked symlink for manifest LF test"
manifest_lf_base_sha="$(git -C "$wt" rev-parse HEAD)"
order_file="$TMP_ROOT/order-gr-d2-manifest-symlink-lf.md"
valid_order | sed -E "s#^BASE_SHA: .*#BASE_SHA: $manifest_lf_base_sha#" | write_order "$order_file"
hook="$TMP_ROOT/hook-manifest-symlink-lf.sh"
hook_touch "$hook" "rm \"$wt/tracked-link\"" "ln -s \$'same-target\\n' \"$wt/tracked-link\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "manifest symlink trailing-LF attack fails closed"
manifest_lf_evroot="$(evidence_root_for "$order_file")"
manifest_lf_attempt="$(latest_attempt_dir "$manifest_lf_evroot")"
[[ -f "$manifest_lf_attempt/pre-terra-out-of-scope-manifest.nul" ]] \
  || fail "manifest symlink LF: pre manifest evidence missing"
[[ -f "$manifest_lf_attempt/post-terra-out-of-scope-manifest.nul" ]] \
  || fail "manifest symlink LF: post manifest evidence missing"
manifest_lf_pre_hash="$(shasum -a 256 "$manifest_lf_attempt/pre-terra-out-of-scope-manifest.nul" | awk '{print $1}')"
manifest_lf_post_hash="$(shasum -a 256 "$manifest_lf_attempt/post-terra-out-of-scope-manifest.nul" | awk '{print $1}')"
[[ "$manifest_lf_pre_hash" != "$manifest_lf_post_hash" ]] \
  || fail "manifest symlink LF: appending a trailing LF byte to a symlink target must change the raw manifest digest"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "manifest symlink LF: Grok must not be called"
fi

# clean filterがindex blobを装って返し、かつassume-unchangedでstatusからも
# 隠す二重の偽装を、--no-filtersで生byteを直接hashするmanifestが検出できる
# ことを直接確認する
wt="$(fresh_valid_worktree manifest-clean-filter)"
cat >"$wt/.gitattributes" <<'EOF'
AGENTS.md filter=motolii-fake
EOF
git -C "$wt" add .gitattributes
git -C "$wt" commit -qm "add clean filter fixture for manifest test"
git -C "$wt" config filter.motolii-fake.clean "git show HEAD:AGENTS.md"
git -C "$wt" config filter.motolii-fake.smudge cat
git -C "$wt" update-index --assume-unchanged AGENTS.md
manifest_cf_base_sha="$(git -C "$wt" rev-parse HEAD)"
order_file="$TMP_ROOT/order-gr-d2-manifest-clean-filter.md"
valid_order | sed -E "s#^BASE_SHA: .*#BASE_SHA: $manifest_cf_base_sha#" | write_order "$order_file"
hook="$TMP_ROOT/hook-manifest-clean-filter.sh"
hook_touch "$hook" "echo '# clean-filter raw content attack' >> \"$wt/AGENTS.md\""
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "manifest clean-filter + assume-unchanged raw content attack fails closed"
manifest_cf_evroot="$(evidence_root_for "$order_file")"
manifest_cf_attempt="$(latest_attempt_dir "$manifest_cf_evroot")"
[[ -f "$manifest_cf_attempt/pre-terra-out-of-scope-manifest.nul" ]] \
  || fail "manifest clean-filter: pre manifest evidence missing"
[[ -f "$manifest_cf_attempt/post-terra-out-of-scope-manifest.nul" ]] \
  || fail "manifest clean-filter: post manifest evidence missing"
manifest_cf_pre_hash="$(shasum -a 256 "$manifest_cf_attempt/pre-terra-out-of-scope-manifest.nul" | awk '{print $1}')"
manifest_cf_post_hash="$(shasum -a 256 "$manifest_cf_attempt/post-terra-out-of-scope-manifest.nul" | awk '{print $1}')"
[[ "$manifest_cf_pre_hash" != "$manifest_cf_post_hash" ]] \
  || fail "manifest clean-filter: a clean-filter-masked raw content change must change the raw manifest digest"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "manifest clean-filter: Grok must not be called"
fi

# 実LF byteを含むtracked pathでも、metadata-first/path-lastのNUL区切り framingが
# 壊れず、その1record内でbyte単位の変化を検出できることを確認する
wt="$(fresh_valid_worktree manifest-newline-path)"
manifest_lf_path="oos-lf-"$'\n'"tail.txt"
printf 'printf %s > "%s"\n' "original" "$wt/$manifest_lf_path" | bash
(cd "$wt" && git add -- "$manifest_lf_path")
git -C "$wt" commit -qm "add newline-named tracked file for manifest test"
manifest_np_base_sha="$(git -C "$wt" rev-parse HEAD)"
order_file="$TMP_ROOT/order-gr-d2-manifest-newline-path.md"
valid_order | sed -E "s#^BASE_SHA: .*#BASE_SHA: $manifest_np_base_sha#" | write_order "$order_file"
hook="$TMP_ROOT/hook-manifest-newline-path.sh"
printf 'printf %s > "%s"\n' "changed" "$wt/$manifest_lf_path" >"$hook"
run_execute "$wt" "$order_file" FAKE_TERRA_HOOK="$hook"
assert_status 7 "$RUN_STATUS" "manifest actual-LF tracked path fails closed"
manifest_np_evroot="$(evidence_root_for "$order_file")"
manifest_np_attempt="$(latest_attempt_dir "$manifest_np_evroot")"
[[ -f "$manifest_np_attempt/pre-terra-out-of-scope-manifest.nul" ]] \
  || fail "manifest actual-LF path: pre manifest evidence missing"
[[ -f "$manifest_np_attempt/post-terra-out-of-scope-manifest.nul" ]] \
  || fail "manifest actual-LF path: post manifest evidence missing"
manifest_np_pre_hash="$(shasum -a 256 "$manifest_np_attempt/pre-terra-out-of-scope-manifest.nul" | awk '{print $1}')"
manifest_np_post_hash="$(shasum -a 256 "$manifest_np_attempt/post-terra-out-of-scope-manifest.nul" | awk '{print $1}')"
[[ "$manifest_np_pre_hash" != "$manifest_np_post_hash" ]] \
  || fail "manifest actual-LF path: a content change to an actual-LF tracked path must change the raw manifest digest"
grep -zqF -- "$manifest_lf_path" "$manifest_np_attempt/post-terra-out-of-scope-manifest.nul" \
  || fail "manifest actual-LF path: the exact LF byte in the path must survive manifest framing"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "manifest actual-LF path: Grok must not be called"
fi

##############################################################################
# GR-D2十一次修正: byte-preserving symlink target hash・checkpoint一時fileの
# evidence_root配下化・Grok失敗/timeout時のorder整合性再確認・
# ignore-policy/fingerprint sortのprocess substitution除去を直接検証する
##############################################################################

# 許可済みの追跡symlinkでも、targetへ実LFを1byte追加しただけの変更で
# 全体fingerprintが変わり、ACCEPTが無効化されることを確認する
# ($(readlink ...)のtrailing newline剥ぎ取りに埋もれないことの直接証明)
wt="$(fresh_valid_worktree grok-symlink-lf-fingerprint)"
ln -s same-target "$wt/tracked-link"
git -C "$wt" add tracked-link
git -C "$wt" commit -qm "add allowed tracked symlink for grok LF fingerprint test"
GROK_LF_AGENTS_HASH="$(sha256_file "$wt/AGENTS.md")"
GROK_LF_LEDGER_HASH="$(sha256_file "$wt/docs/implementation-ledger.md")"
GROK_LF_BASE_SHA="$(git -C "$wt" rev-parse HEAD)"
order_file="$TMP_ROOT/order-gr-d2-grok-symlink-lf.md"
cat >"$order_file" <<EOF
GRAIN: GR-D1
BASE_REF: refs/heads/$WT_VALID_BRANCH
BASE_SHA: $GROK_LF_BASE_SHA
DEPENDENCY: U0e-2R
AUTHORITY: AGENTS.md SHA256:$GROK_LF_AGENTS_HASH
AUTHORITY: docs/implementation-ledger.md SHA256:$GROK_LF_LEDGER_HASH
ALLOWED_FILE: scripts/delegate-cursor-supervised.sh
ALLOWED_FILE: tracked-link
ORDER: READY
SUPERVISOR_BACKEND: cursor-grok
SUPERVISOR_MODEL: cursor-grok-4.5-high
IMPLEMENTER_MODEL: gpt-5.6-terra
TASK_SHA256: $task_hash
CODEX PRECHECK: APPROVED
EOF
grok_hook="$TMP_ROOT/hook-grok-symlink-lf.sh"
hook_touch "$grok_hook" "rm \"$wt/tracked-link\"" "ln -s \$'same-target\\n' \"$wt/tracked-link\""
run_execute "$wt" "$order_file" \
  FAKE_GROK_HOOK="$grok_hook" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 8 "$RUN_STATUS" "reviewer appends a trailing LF byte to an allowed symlink target invalidates ACCEPT"
assert_has_fragment "$TMP_ROOT/stderr.log" "INSPECT NG:" \
  "reviewer appends a trailing LF byte to an allowed symlink target invalidates ACCEPT"
grok_lf_evroot="$(evidence_root_for "$order_file")"
grok_lf_attempt="$(latest_attempt_dir "$grok_lf_evroot")"
assert_file_exists "$grok_lf_attempt/pre-grok-fingerprint.sha256" "reviewer symlink LF: pre-grok fingerprint evidence missing"
assert_file_exists "$grok_lf_attempt/post-grok-fingerprint.sha256" "reviewer symlink LF: post-grok fingerprint evidence missing"
grok_lf_pre_fp="$(cat "$grok_lf_attempt/pre-grok-fingerprint.sha256")"
grok_lf_post_fp="$(cat "$grok_lf_attempt/post-grok-fingerprint.sha256")"
[[ "$grok_lf_pre_fp" != "$grok_lf_post_fp" ]] \
  || fail "reviewer symlink LF: appending a trailing LF byte to a symlink target must change the general fingerprint"

# publish_checkpointの一時fileはevidence_root配下に作られ、mvは同一ディレクトリ内の
# atomic renameになる。BSD mktemp(macOS)はXXXXXXが末尾run以外だと置換せず、
# 旧テンプレート"checkpoint.XXXXXX.tmp"は固定名の残留物を作っていた。ここでは
# その固定名をsentinelとして事前に置き、publishがそれを一切開閉/上書きしないこと、
# かつ本物のcheckpoint.txtがACCEPTされることを直接証明する
wt="$(fresh_valid_worktree checkpoint-tmp-location)"
order_file="$TMP_ROOT/order-gr-d2-checkpoint-tmp-location.md"
valid_order | write_order "$order_file"
ckpt_evroot="$(evidence_root_for "$order_file")"
mkdir -p "$ckpt_evroot"
ckpt_sentinel="$ckpt_evroot/checkpoint.XXXXXX.tmp"
printf 'sentinel-untouched\n' >"$ckpt_sentinel"
ckpt_sentinel_before="$(sha256_file "$ckpt_sentinel")"
run_execute "$wt" "$order_file" \
  FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT'
assert_status 0 "$RUN_STATUS" "checkpoint tmp location: ACCEPT with unchanged worktree"
assert_file_exists "$ckpt_evroot/checkpoint.txt" "checkpoint tmp location: checkpoint published"
[[ -f "$ckpt_sentinel" ]] \
  || fail "checkpoint tmp location: precreated checkpoint.XXXXXX.tmp sentinel must survive a successful publish"
ckpt_sentinel_after="$(sha256_file "$ckpt_sentinel")"
[[ "$ckpt_sentinel_before" == "$ckpt_sentinel_after" ]] \
  || fail "checkpoint tmp location: precreated checkpoint.XXXXXX.tmp sentinel must not be opened or overwritten by publish"
for leftover in "$ckpt_evroot"/checkpoint.tmp.*; do
  [[ -e "$leftover" ]] || continue
  fail "checkpoint tmp location: no checkpoint.tmp.XXXXXX temp file may survive under evidence_root after a successful publish"
done

# Grokがtimeoutする直前に承認済みorder(worktree外の外部fileとattempt copyの両方)を
# 書き換えても、worktree fingerprintが不変のまま republish されてはならない。
# republish前にorder整合性を独立に再確認し、EVIDENCE NGでfail closedして
# checkpointを無効化したままにする
wt="$(fresh_valid_worktree grok-timeout-order-mutation)"
order_file="$TMP_ROOT/order-gr-d2-grok-timeout-order-mutation.md"
valid_order | write_order "$order_file"
grok_hook="$TMP_ROOT/hook-grok-timeout-order-mutation.sh"
hook_touch "$grok_hook" \
  "printf '# mutated during timeout\\n' >> \"$order_file\"" \
  "sleep 5"
run_execute "$wt" "$order_file" \
  CURSOR_INSPECTION_TIMEOUT_SECONDS=1 \
  FAKE_GROK_HOOK="$grok_hook"
assert_status 6 "$RUN_STATUS" "reviewer mutates the approved order during an inspection timeout fails closed"
assert_has_fragment "$TMP_ROOT/stderr.log" "EVIDENCE NG:" \
  "reviewer mutates the approved order during an inspection timeout fails closed"
assert_has_fragment "$TMP_ROOT/stderr.log" "order mutated during grok inspection" \
  "reviewer mutates the approved order during an inspection timeout fails closed"
otm_evroot="$(evidence_root_for "$order_file")"
[[ ! -f "$otm_evroot/checkpoint.txt" ]] \
  || fail "reviewer mutates the approved order during an inspection timeout: checkpoint must not remain valid"
otm_first_attempt="$(latest_attempt_dir "$otm_evroot")"
run_inspect "$wt" "$order_file"
assert_status 6 "$RUN_STATUS" "resumed inspect after an grok-timeout order mutation has no checkpoint"
assert_no_external_calls "resumed inspect after an grok-timeout order mutation has no checkpoint"
otm_second_attempt="$(latest_attempt_dir "$otm_evroot")"
[[ "$otm_second_attempt" != "$otm_first_attempt" ]] \
  || fail "resumed inspect after an grok-timeout order mutation: expected a new attempt directory"

# compute_ignore_policy_hash/compute_fingerprintがGit列挙のsort結果を
# process substitution(< <(...))経由で読むと、sort失敗が呼び出し元へ伝わらず
# 空の成功集合へ素通りし得る。sortを注入して失敗させ、SCOPE NGでfail closed
# することを確認する
SORT_FAIL_BIN="$TMP_ROOT/bin-sort-fail"
mkdir -p "$SORT_FAIL_BIN"
cat >"$SORT_FAIL_BIN/sort" <<EOF
#!/usr/bin/env bash
echo "injected sort failure" >&2
exit 1
EOF
chmod +x "$SORT_FAIL_BIN/sort"

wt="$(fresh_valid_worktree sort-enumeration-failure)"
order_file="$TMP_ROOT/order-gr-d2-sort-enum-fail.md"
valid_order | write_order "$order_file"
: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CODEX_DELEGATED \
    PATH="$SORT_FAIL_BIN:$FAKE_BIN:/usr/bin:/bin" \
    FAKE_CALL_LOG="$CALL_LOG" \
    CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
    CURSOR_SUPERVISED_TIMEOUT_SECONDS=5 \
    CODEX_TERRA_TIMEOUT_SECONDS=5 \
    CURSOR_INSPECTION_TIMEOUT_SECONDS=5 \
    "$SCRIPT" execute "$wt" "$order_file" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
  RUN_STATUS=0
else
  RUN_STATUS=$?
fi
assert_status 7 "$RUN_STATUS" "injected sort failure in ignore-policy/fingerprint enumeration fails closed rather than an empty successful set"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "injected sort failure fails closed"
if grep -q -- "--model cursor-grok-4.5-high" "$CALL_LOG"; then
  fail "injected sort failure: Grok must not be called"
fi

##############################################################################
# GR-D2十三次修正: raw_symlink_targetの失敗をpipeline末尾のcommand
# substitutionへ委ねたset -eの暗黙exitへ戻さず、実体化してstatusを確認して
# からSCOPE NG:/exit 7へ正規化したことを、追跡済みの許可外symlinkに対し
# perlのreadlinkが失敗する状況で直接確認する。この失敗はTerra起動前の
# build_out_of_scope_manifestで起きるため、External呼び出しは一切発生しない
##############################################################################

PERL_FAIL_BIN="$TMP_ROOT/bin-perl-fail"
mkdir -p "$PERL_FAIL_BIN"
cat >"$PERL_FAIL_BIN/perl" <<EOF
#!/usr/bin/env bash
echo "injected perl readlink failure" >&2
exit 1
EOF
chmod +x "$PERL_FAIL_BIN/perl"

wt="$(fresh_valid_worktree perl-readlink-failure)"
ln -s original-target "$wt/oos-symlink"
git -C "$wt" add oos-symlink
git -C "$wt" commit -qm "add tracked out-of-allowlist symlink"
PRF_AGENTS_HASH="$(sha256_file "$wt/AGENTS.md")"
PRF_LEDGER_HASH="$(sha256_file "$wt/docs/implementation-ledger.md")"
PRF_BASE_SHA="$(git -C "$wt" rev-parse HEAD)"
order_file="$TMP_ROOT/order-gr-d2-perl-readlink-failure.md"
cat >"$order_file" <<EOF
GRAIN: GR-D1
BASE_REF: refs/heads/$WT_VALID_BRANCH
BASE_SHA: $PRF_BASE_SHA
DEPENDENCY: U0e-2R
AUTHORITY: AGENTS.md SHA256:$PRF_AGENTS_HASH
AUTHORITY: docs/implementation-ledger.md SHA256:$PRF_LEDGER_HASH
ALLOWED_FILE: scripts/delegate-cursor-supervised.sh
ORDER: READY
SUPERVISOR_BACKEND: cursor-grok
SUPERVISOR_MODEL: cursor-grok-4.5-high
IMPLEMENTER_MODEL: gpt-5.6-terra
TASK_SHA256: $task_hash
CODEX PRECHECK: APPROVED
EOF
: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CODEX_DELEGATED \
    PATH="$PERL_FAIL_BIN:$FAKE_BIN:/usr/bin:/bin" \
    FAKE_CALL_LOG="$CALL_LOG" \
    CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
    CURSOR_SUPERVISED_TIMEOUT_SECONDS=5 \
    CODEX_TERRA_TIMEOUT_SECONDS=5 \
    CURSOR_INSPECTION_TIMEOUT_SECONDS=5 \
    "$SCRIPT" execute "$wt" "$order_file" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
  RUN_STATUS=0
else
  RUN_STATUS=$?
fi
assert_status 7 "$RUN_STATUS" "injected perl readlink failure on a tracked out-of-allowlist symlink fails closed before any External call"
assert_has_fragment "$TMP_ROOT/stderr.log" "SCOPE NG:" "injected perl readlink failure fails closed"
assert_no_external_calls "injected perl readlink failure on a tracked out-of-allowlist symlink"
prf_evroot="$(evidence_root_for "$order_file")"
[[ ! -f "$prf_evroot/checkpoint.txt" ]] \
  || fail "injected perl readlink failure: checkpoint must not be written"
prf_attempt="$(latest_attempt_dir "$prf_evroot")"
assert_has_fragment "$prf_attempt/stage-result.txt" "SCOPE NG:" \
  "injected perl readlink failure: stage evidence must record the failure"

##############################################################################
# GR-D2十四次修正: 起動時のPRIMARY_WORKTREE解決がgit worktree list --porcelainを
# 早期examitするawkへpipeすると、pipe bufferを超える出力でGitがSIGPIPEを受け
# exit 141になり得る。実際の先頭recordを出す前提でpipe bufferを超える大量の
# 追加porcelain recordを返すgit shimを注入し、正常な単独worktree実行が引き続き
# status 0でACCEPTし、実際の先頭worktree recordを選択できることを確認する
##############################################################################

REAL_GIT_BIN="$(command -v git)"
WT_FLOOD_BIN="$TMP_ROOT/bin-worktree-list-flood"
mkdir -p "$WT_FLOOD_BIN"
cat >"$WT_FLOOD_BIN/git" <<EOF
#!/usr/bin/env bash
set -euo pipefail
args=("\$@")
is_worktree_list_porcelain=0
has_worktree=0
has_list=0
has_porcelain=0
for a in "\${args[@]}"; do
  case "\$a" in
    worktree) has_worktree=1 ;;
    list) has_list=1 ;;
    --porcelain) has_porcelain=1 ;;
  esac
done
if [[ "\$has_worktree" == 1 && "\$has_list" == 1 && "\$has_porcelain" == 1 ]]; then
  is_worktree_list_porcelain=1
fi
if [[ "\$is_worktree_list_porcelain" == 1 ]]; then
  "$REAL_GIT_BIN" "\${args[@]}"
  i=0
  while [[ "\$i" -lt 20000 ]]; do
    printf 'worktree /tmp/bogus-flood-worktree-%d\nbare\n\n' "\$i"
    i=\$((i + 1))
  done
  exit 0
fi
exec "$REAL_GIT_BIN" "\${args[@]}"
EOF
chmod +x "$WT_FLOOD_BIN/git"

wt="$(fresh_valid_worktree worktree-list-flood)"
order_file="$TMP_ROOT/order-gr-d2-worktree-list-flood.md"
valid_order | write_order "$order_file"
: >"$CALL_LOG"
if env -u CURSOR_AGENT -u CODEX_DELEGATED \
    PATH="$WT_FLOOD_BIN:$FAKE_BIN:/usr/bin:/bin" \
    FAKE_CALL_LOG="$CALL_LOG" \
    FAKE_GROK_OUTPUT=$'inspection complete\nVERDICT: ACCEPT' \
    CURSOR_SUPERVISED_HEARTBEAT_SECONDS=1 \
    CURSOR_SUPERVISED_TIMEOUT_SECONDS=5 \
    CODEX_TERRA_TIMEOUT_SECONDS=5 \
    CURSOR_INSPECTION_TIMEOUT_SECONDS=5 \
    "$SCRIPT" execute "$wt" "$order_file" "$task" \
    >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"; then
  RUN_STATUS=0
else
  RUN_STATUS=$?
fi
assert_status 0 "$RUN_STATUS" \
  "a pipe-buffer-exceeding git worktree list --porcelain output must not turn into SIGPIPE/exit 141"
assert_has_fragment "$CALL_LOG" "--model cursor-grok-4.5-high" \
  "a pipe-buffer-exceeding git worktree list --porcelain output still reaches Grok on a valid isolated worktree"

echo "test-delegate-cursor-supervised: all tests passed"
