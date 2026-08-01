#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/delegate-cursor-supervised.sh"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/motolii-opus-spark-grok-test.XXXXXX")"

cleanup() {
  if [[ "${MOTOLII_KEEP_TEST_TMP:-0}" == "1" ]]; then
    echo "test-delegate-cursor-supervised: kept $TMP_ROOT" >&2
    return
  fi
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

GIT_UNSET=( -u GIT_DIR -u GIT_WORK_TREE -u GIT_INDEX_FILE -u GIT_OBJECT_DIRECTORY
  -u GIT_ALTERNATE_OBJECT_DIRECTORIES -u GIT_COMMON_DIR -u GIT_NAMESPACE
  -u GIT_CEILING_DIRECTORIES -u GIT_DISCOVERY_ACROSS_FILESYSTEM )

git_fixture() {
  local wt="$1"
  shift
  env "${GIT_UNSET[@]}" git --git-dir="$wt/.git" --work-tree="$wt" "$@"
}

canonical_ref_digest() {
  local wt="$1"
  local head_file show_ref_file combined_file sorted_file show_ref_status
  head_file="$(mktemp "${TMP_ROOT}/ref-head.XXXXXX")"
  show_ref_file="$(mktemp "${TMP_ROOT}/ref-show-ref.XXXXXX")"
  combined_file="$(mktemp "${TMP_ROOT}/ref-combined.XXXXXX")"
  sorted_file="$(mktemp "${TMP_ROOT}/ref-sorted.XXXXXX")"
  if ! git_fixture "$wt" rev-parse HEAD >"$head_file"; then
    fail "canonical ref digest: rev-parse HEAD failed"
  fi
  set +e
  git_fixture "$wt" show-ref >"$show_ref_file" 2>/dev/null
  show_ref_status=$?
  set -e
  if (( show_ref_status != 0 )); then
    if (( show_ref_status == 1 )) && [[ ! -s "$show_ref_file" ]]; then
      :
    else
      fail "canonical ref digest: show-ref failed (status $show_ref_status)"
    fi
  fi
  cat "$head_file" "$show_ref_file" >"$combined_file"
  LC_ALL=C sort -o "$sorted_file" "$combined_file"
  shasum -a 256 "$sorted_file" | awk '{print $1}'
}

root_repo_git() {
  env "${GIT_UNSET[@]}" git --git-dir="$ROOT_GIT_DIR" --work-tree="$ROOT_DIR" "$@"
}

canonical_root_ref_digest() {
  local head_file show_ref_file combined_file sorted_file show_ref_status
  head_file="$(mktemp "${TMP_ROOT}/root-ref-head.XXXXXX")"
  show_ref_file="$(mktemp "${TMP_ROOT}/root-ref-show-ref.XXXXXX")"
  combined_file="$(mktemp "${TMP_ROOT}/root-ref-combined.XXXXXX")"
  sorted_file="$(mktemp "${TMP_ROOT}/root-ref-sorted.XXXXXX")"
  if ! root_repo_git rev-parse HEAD >"$head_file"; then
    fail "canonical root ref digest: rev-parse HEAD failed"
  fi
  set +e
  root_repo_git show-ref >"$show_ref_file" 2>/dev/null
  show_ref_status=$?
  set -e
  if (( show_ref_status != 0 )); then
    if (( show_ref_status == 1 )) && [[ ! -s "$show_ref_file" ]]; then
      :
    else
      fail "canonical root ref digest: show-ref failed (status $show_ref_status)"
    fi
  fi
  cat "$head_file" "$show_ref_file" >"$combined_file"
  LC_ALL=C sort -o "$sorted_file" "$combined_file"
  shasum -a 256 "$sorted_file" | awk '{print $1}'
}

ROOT_GIT_DIR="$(git -C "$ROOT_DIR" rev-parse --absolute-git-dir)"
ROOT_HEAD_BEFORE="$(root_repo_git rev-parse HEAD)"
ROOT_REF_DIGEST_BEFORE="$(canonical_root_ref_digest)"

FAKE_BIN="$TMP_ROOT/bin"
CALL_LOG="$TMP_ROOT/calls.log"
RM_ARGS_LOG="$TMP_ROOT/rm-args.log"
RMDIR_ARGS_LOG="$TMP_ROOT/rmdir-args.log"
export FAKE_TMP_ROOT="$TMP_ROOT"
mkdir -p "$FAKE_BIN"

cat >"$FAKE_BIN/claude" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo "claude:$*" >>"$FAKE_CALL_LOG"
if [[ "${FAKE_OPUS_STATUS:-0}" != "0" ]]; then
  exit "$FAKE_OPUS_STATUS"
fi
default_output='{"type":"result","subtype":"success","structured_output":{"disposition":"STOP","findings":[],"reason":"fixture stop"}}'
if [[ "$*" == *"--resume "* ]]; then
  printf '%s\n' "${FAKE_OPUS_RETRY_OUTPUT:-${FAKE_OPUS_OUTPUT:-$default_output}}"
else
  printf '%s\n' "${FAKE_OPUS_OUTPUT:-$default_output}"
fi
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

cat >"$FAKE_BIN/rm" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
{
  for arg in "$@"; do
    printf '%s\0' "$arg"
  done
} >>"${FAKE_TMP_ROOT}/rm-args.log"
/bin/rm "$@"
EOF

cat >"$FAKE_BIN/rmdir" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
{
  for arg in "$@"; do
    printf '%s\0' "$arg"
  done
} >>"${FAKE_TMP_ROOT}/rmdir-args.log"
/bin/rmdir "$@"
EOF
chmod +x "$FAKE_BIN/rm" "$FAKE_BIN/rmdir"

gr_d3_write_ledger() {
  local wt="$1"
  cat >"$wt/docs/implementation-ledger.md" <<'EOF'
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
}

gr_d3_init_worktree() {
  local wt="$1"
  mkdir -p "$wt/docs"
  git_fixture "$wt" init -q
  git_fixture "$wt" config user.email test@example.com
  git_fixture "$wt" config user.name test
  git_fixture "$wt" checkout -q -b managed-grain
  printf 'authority\n' >"$wt/AGENTS.md"
  printf 'before\n' >"$wt/src.txt"
  gr_d3_write_ledger "$wt"
  git_fixture "$wt" add -A
  git_fixture "$wt" commit -q -m init
  printf '%s' "$(cd "$wt" && pwd -P)"
}

gr_d3_ready_order() {
  local wt="$1" dest="$2"
  local base_sha auth_hash task_hash
  base_sha="$(git_fixture "$wt" rev-parse HEAD)"
  auth_hash="$(sha256_file "$wt/AGENTS.md")"
  task_hash="$(printf '%s' "$TASK" | shasum -a 256 | awk '{print $1}')"
  cat >"$dest" <<EOF
Objective: update the allowed fixture.
GRAIN: GRAIN-1
BASE_REF: refs/heads/managed-grain
BASE_SHA: $base_sha
DEPENDENCY: DEP-1
AUTHORITY: AGENTS.md SHA256:$auth_hash
ALLOWED_FILE: src.txt
READ_MODE: CAPSULE
CONTEXT_FACT: src.txt is the only implementation fixture.
READ_FILE: src.txt
INTERNAL_TARGET: src.txt :: before
TEST_TARGET: src.txt :: before
REUSE_TARGET: src.txt :: before
NEW_SURFACE: FORBIDDEN
AUTHORITY_SPAN: ONE
OWNER_CLOSURE: CLOSED
CAUSE_CLOSURE: LOCALIZED
CONTRACT_IMPACT: PRIVATE
CONTRACT_CLOSURE: PRIVATE
CONTRACT_AUTHORITY: NONE
ORACLE_CLOSURE: CLOSED
REUSE_CLOSURE: REUSE
VIEW_PROFILE: CLOSED
HAZARD_TAG: NONE
Non-goal: no adjacent edits.
STOP: authority conflict.
Test: git diff --check.
ORDER: READY
LOOP_PROFILE: opus-spark-grok
ORDER_MANAGER_MODEL: claude-opus-5
IMPLEMENTER_MODEL: gpt-5.3-codex-spark
REVIEW_MODEL: cursor-grok-4.5-high
TASK_SHA256: $task_hash
CODEX PRECHECK: APPROVED
EOF
}

assert_rm_never_removed_target_root() {
  local wt="$1" label="$2"
  local wt_real arg base
  wt_real="$(cd "$wt" && pwd -P)"
  [[ -f "$RM_ARGS_LOG" ]] || return 0
  while IFS= read -r -d '' arg || [[ -n "${arg:-}" ]]; do
    [[ -z "$arg" ]] && continue
    [[ "$arg" == "--" ]] && continue
    [[ "$arg" == -* ]] && continue
    [[ "$arg" == "$wt_real/target" || "$arg" == "$wt_real/target/" ]] \
      && fail "$label: rm invoked on target root: $arg"
    if [[ "$arg" == */ ]]; then
      base="$(basename "${arg%/}")"
    else
      base="$(basename "$arg")"
    fi
    [[ -n "$base" ]] || fail "$label: rm argument with empty basename: $arg"
  done <"$RM_ARGS_LOG"
}

assert_rmdir_removed_target_root() {
  local wt="$1" label="$2"
  local wt_real target_root found=0 arg
  wt_real="$(cd "$wt" && pwd -P)"
  target_root="$wt_real/target"
  [[ -f "$RMDIR_ARGS_LOG" ]] || fail "$label: missing rmdir log"
  while IFS= read -r -d '' arg || [[ -n "${arg:-}" ]]; do
    [[ -z "$arg" ]] && continue
    if [[ "$arg" == "$target_root" ]]; then
      found=1
    fi
  done <"$RMDIR_ARGS_LOG"
  [[ "$found" -eq 1 ]] || fail "$label: rmdir never received target root (expected $target_root)"
}

create_three_known_target_entries() {
  local wt="$1"
  mkdir -p "$wt/target/scaffold-plugin-fixture/nested"
  printf 'marker-scaffold\n' >"$wt/target/scaffold-plugin-fixture/nested/marker.txt"
  mkdir -p "$wt/target/new-plugin-scaffold-test"
  printf 'marker-scaffold-test\n' >"$wt/target/new-plugin-scaffold-test/marker.txt"
  printf 'marker-tsv\n' >"$wt/target/d1i4-empty-classification.tsv"
}

assert_three_known_target_entries() {
  local wt="$1" label="$2"
  grep -Fqx 'marker-scaffold' "$wt/target/scaffold-plugin-fixture/nested/marker.txt" \
    || fail "$label: scaffold marker missing"
  grep -Fqx 'marker-scaffold-test' "$wt/target/new-plugin-scaffold-test/marker.txt" \
    || fail "$label: scaffold-test marker missing"
  grep -Fqx 'marker-tsv' "$wt/target/d1i4-empty-classification.tsv" \
    || fail "$label: tsv marker missing"
}

WT="$TMP_ROOT/worktree"
mkdir -p "$WT/docs"
git -C "$WT" init -q
git -C "$WT" config user.email test@example.com
git -C "$WT" config user.name test
git -C "$WT" checkout -q -b managed-grain
printf 'authority\n' >"$WT/AGENTS.md"
printf 'before\n' >"$WT/src.txt"
printf 'fixture oracle\n' >"$WT/test.txt"
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
TASK=$(cat <<EOF
Objective: update the allowed fixture.
GRAIN: GRAIN-1
BASE_REF: refs/heads/managed-grain
BASE_SHA: $BASE_SHA
DEPENDENCY: DEP-1
AUTHORITY: AGENTS.md SHA256:$AUTH_HASH
ALLOWED_FILE: src.txt
ALLOWED_FILE: test.txt
READ_MODE: CAPSULE
CONTEXT_FACT: src.txt is the only implementation fixture.
READ_FILE: src.txt
READ_FILE: test.txt
INTERNAL_TARGET: src.txt :: before
TEST_TARGET: test.txt :: fixture oracle
REUSE_TARGET: src.txt :: before
NEW_SURFACE: FORBIDDEN
AUTHORITY_SPAN: ONE
OWNER_CLOSURE: CLOSED
CAUSE_CLOSURE: LOCALIZED
CONTRACT_IMPACT: PRIVATE
CONTRACT_CLOSURE: PRIVATE
CONTRACT_AUTHORITY: NONE
ORACLE_CLOSURE: CLOSED
REUSE_CLOSURE: REUSE
VIEW_PROFILE: CLOSED
HAZARD_TAG: NONE
Non-goal: no adjacent edits.
STOP: authority conflict.
Test: git diff --check.
EOF
)
TASK_HASH="$(printf '%s' "$TASK" | shasum -a 256 | awk '{print $1}')"
ORDER="$TMP_ROOT/order.md"
# 実CLIの`--output-format json`はstructured_outputと同じ階層でusageとtotal_cost_usdを
# 返す。fixtureも同形にして、全prepare試験が計装の抽出経路を通るようにする
OPUS_USAGE='"total_cost_usd":0.116,"duration_api_ms":21000,"usage":{"input_tokens":8084,"output_tokens":2206,"cache_read_input_tokens":2944}'
OPUS_READY='{"type":"result","subtype":"success",'"$OPUS_USAGE"',"structured_output":{"disposition":"READY","findings":[{"kind":"NEGATIVE_ORACLE","severity":"P2","delta":"src.txt must remain unchanged when authority validation fails."}],"reason":"fixture skeleton is closed"}}'
OPUS_BLOCKING='{"type":"result","subtype":"success",'"$OPUS_USAGE"',"structured_output":{"disposition":"READY","findings":[{"kind":"NEGATIVE_ORACLE","severity":"P1","delta":"Add an exact unchanged-file rejection oracle before implementation."}],"reason":"one blocking oracle gap remains"}}'

run_script() {
  : >"$CALL_LOG"
  set +e
  env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
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
assert_fragment "$CALL_LOG" "--effort low" "Opus fast delta option"
assert_fragment "$CALL_LOG" "--json-schema" "Opus typed delta schema"
assert_fragment "$CALL_LOG" "--output-format json" "Opus JSON output"
if grep -Fq -- "--no-session-persistence" "$CALL_LOG"; then
  fail "typed delta must retain its session only for one schema retry"
fi

# P0/P1 deltaはSparkへの散文handoffにせず、Codex骨格のrevisionへ戻す。
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  FAKE_OPUS_OUTPUT="$OPUS_BLOCKING" \
  "$SCRIPT" prepare "$WT" "$TMP_ROOT/blocking-order.md" "$TASK" \
  >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 3 "$RUN_STATUS" "P0/P1 delta requires skeleton revision"
grep -Fq -- "骨格とexact oracleへ織り込み" "$TMP_ROOT/stderr.log" \
  || fail "blocking delta did not request skeleton revision"
[[ ! -f "$TMP_ROOT/blocking-order.md" ]] || fail "blocking delta must not produce a dispatchable order"

# 骨格差戻しはSpark実行を置換する最も安い経路であり、その安さを実測で残す。
# Opus JSONのusage/costを捨てず、欠測stageも沈黙させないことを固定する。
BLOCKING_TELEMETRY="$TMP_ROOT/blocking-order.md.evidence/prepare-telemetry.txt"
[[ -f "$BLOCKING_TELEMETRY" ]] || fail "prepare must record telemetry even when it fails closed"
assert_has "$BLOCKING_TELEMETRY" "TELEMETRY_VERSION: 1" "telemetry version"
assert_has "$BLOCKING_TELEMETRY" "OPUS_DELTA_INPUT_TOKENS: 8084" "Opus input tokens captured"
assert_has "$BLOCKING_TELEMETRY" "OPUS_DELTA_OUTPUT_TOKENS: 2206" "Opus output tokens captured"
assert_has "$BLOCKING_TELEMETRY" "OPUS_DELTA_CACHE_READ_TOKENS: 2944" "Opus cache read tokens captured"
assert_has "$BLOCKING_TELEMETRY" "OPUS_DELTA_COST_USD: 0.116" "Opus cost captured"
assert_has "$BLOCKING_TELEMETRY" "TOTAL_INPUT_TOKENS: 8084" "prepare telemetry totals"
assert_has "$BLOCKING_TELEMETRY" "UNMEASURED_TOKEN_STAGES: NONE" "prepare has no unmeasured stage"
grep -Eq '^OPUS_DELTA_WALL_SECONDS: [0-9]+$' "$BLOCKING_TELEMETRY" \
  || fail "Opus wall seconds must be numeric"
grep -Eq '^TOTAL_WALL_SECONDS: [0-9]+$' "$BLOCKING_TELEMETRY" \
  || fail "prepare must total wall seconds"

# WIDEはOpusを起動せずread-only探索へ戻す。
WIDE_TASK="$(printf '%s\n' "$TASK" |
  sed 's/^CAUSE_CLOSURE: LOCALIZED$/CAUSE_CLOSURE: COMPETING/; s/^VIEW_PROFILE: CLOSED$/VIEW_PROFILE: WIDE/')"
: >"$CALL_LOG"
run_script prepare "$WT" "$ORDER" "$WIDE_TASK"
assert_status 3 "$RUN_STATUS" "WIDE prepare rejected before Opus"
grep -Fq -- "VIEW_PROFILE WIDE requires read-only exploration" "$TMP_ROOT/stderr.log" \
  || fail "WIDE rejection reason is missing"
[[ ! -s "$CALL_LOG" ]] || fail "WIDE must fail before model invocation"

# 狭いprofileへの手動overrideもOpus起動前に拒否する。
MISMATCH_TASK="$(printf '%s\n' "$TASK" | sed 's/^OWNER_CLOSURE: CLOSED$/OWNER_CLOSURE: MULTIPLE_KNOWN/')"
: >"$CALL_LOG"
run_script prepare "$WT" "$ORDER" "$MISMATCH_TASK"
assert_status 3 "$RUN_STATUS" "VIEW_PROFILE mismatch rejected"
grep -Fq -- "VIEW_PROFILE mismatch: declared=CLOSED computed=ADJACENT" "$TMP_ROOT/stderr.log" \
  || fail "profile mismatch reason is missing"
[[ ! -s "$CALL_LOG" ]] || fail "profile mismatch must fail before model invocation"

# 未決の恒久契約はWIDEとしてOpus起動前に止める。
UNRESOLVED_PERMANENT_TASK="$(printf '%s\n' "$TASK" |
  sed 's/^CONTRACT_IMPACT: PRIVATE$/CONTRACT_IMPACT: PERMANENT/;
       s/^CONTRACT_CLOSURE: PRIVATE$/CONTRACT_CLOSURE: UNRESOLVED/;
       s/^VIEW_PROFILE: CLOSED$/VIEW_PROFILE: WIDE/')"
: >"$CALL_LOG"
run_script prepare "$WT" "$ORDER" "$UNRESOLVED_PERMANENT_TASK"
assert_status 3 "$RUN_STATUS" "unresolved permanent contract rejected"
grep -Fq -- "VIEW_PROFILE WIDE requires read-only exploration" "$TMP_ROOT/stderr.log" \
  || fail "unresolved permanent contract rejection reason is missing"
[[ ! -s "$CALL_LOG" ]] || fail "unresolved permanent contract must fail before model invocation"

# 決定済み恒久契約は、検証済みAUTHORITYと一致するreceiptがある場合だけ施工へ進める。
FROZEN_PERMANENT_TASK="$(printf '%s\n' "$TASK" |
  sed "s/^CONTRACT_IMPACT: PRIVATE$/CONTRACT_IMPACT: PERMANENT/;
       s/^CONTRACT_CLOSURE: PRIVATE$/CONTRACT_CLOSURE: FROZEN/;
       s/^CONTRACT_AUTHORITY: NONE$/CONTRACT_AUTHORITY: AGENTS.md@SHA256:$AUTH_HASH/;
       s/^HAZARD_TAG: NONE$/HAZARD_TAG: PERSISTENCE/")"
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  FAKE_OPUS_OUTPUT="$OPUS_READY" \
  "$SCRIPT" prepare "$WT" "$TMP_ROOT/frozen-permanent-order.md" "$FROZEN_PERMANENT_TASK" \
  >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 0 "$RUN_STATUS" "frozen permanent contract prepare"
assert_has "$TMP_ROOT/frozen-permanent-order.md" "CONTRACT_CLOSURE: FROZEN" \
  "frozen permanent contract closure"
assert_has "$TMP_ROOT/frozen-permanent-order.md" \
  "CONTRACT_AUTHORITY: AGENTS.md@SHA256:$AUTH_HASH" "frozen contract authority receipt"

# receiptがAUTHORITYのpath/hashと完全一致しなければmodel起動前に拒否する。
BAD_FROZEN_TASK="$(printf '%s\n' "$FROZEN_PERMANENT_TASK" |
  sed 's/^CONTRACT_AUTHORITY: .*$/CONTRACT_AUTHORITY: AGENTS.md@SHA256:0000000000000000000000000000000000000000000000000000000000000000/')"
: >"$CALL_LOG"
run_script prepare "$WT" "$ORDER" "$BAD_FROZEN_TASK"
assert_status 3 "$RUN_STATUS" "mismatched frozen contract receipt rejected"
grep -Fq -- "CONTRACT_AUTHORITY is not an exact verified AUTHORITY" "$TMP_ROOT/stderr.log" \
  || fail "mismatched frozen contract receipt reason is missing"
[[ ! -s "$CALL_LOG" ]] || fail "mismatched frozen receipt must fail before model invocation"

# exact targetが不在ならOpus起動前に拒否する。
MISSING_TARGET_TASK="$(printf '%s\n' "$TASK" |
  sed 's/^INTERNAL_TARGET: src.txt :: before$/INTERNAL_TARGET: src.txt :: missing implementation anchor/')"
: >"$CALL_LOG"
run_script prepare "$WT" "$ORDER" "$MISSING_TARGET_TASK"
assert_status 3 "$RUN_STATUS" "missing exact target rejected"
grep -Fq -- "INTERNAL_TARGET anchor absent in src.txt" "$TMP_ROOT/stderr.log" \
  || fail "missing target rejection reason is absent"
[[ ! -s "$CALL_LOG" ]] || fail "missing exact target must fail before model invocation"

# 通常粒は既存targetへ接続し、新しいcommand/mode/public surfaceを許可しない。
NEW_SURFACE_TASK="$(printf '%s\n' "$TASK" |
  sed 's/^NEW_SURFACE: FORBIDDEN$/NEW_SURFACE: ALLOWED/')"
: >"$CALL_LOG"
run_script prepare "$WT" "$ORDER" "$NEW_SURFACE_TASK"
assert_status 3 "$RUN_STATUS" "new surface rejected"
grep -Fq -- "NEW_SURFACE: FORBIDDEN must appear exactly once" "$TMP_ROOT/stderr.log" \
  || fail "new surface rejection reason is absent"
[[ ! -s "$CALL_LOG" ]] || fail "new surface must fail before model invocation"

# 隣接軸はWIDEへ過剰拡大せず、Codexが隣接事実をcapsuleへ閉じた後のdelta検証を許す。
ADJACENT_TASK="$(printf '%s\n' "$TASK" |
  sed 's/^OWNER_CLOSURE: CLOSED$/OWNER_CLOSURE: MULTIPLE_KNOWN/; s/^VIEW_PROFILE: CLOSED$/VIEW_PROFILE: ADJACENT/')"
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  FAKE_OPUS_OUTPUT="$OPUS_READY" \
  "$SCRIPT" prepare "$WT" "$TMP_ROOT/adjacent-order.md" "$ADJACENT_TASK" \
  >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 0 "$RUN_STATUS" "ADJACENT prepare"
assert_has "$TMP_ROOT/adjacent-order.md" "VIEW_PROFILE: ADJACENT" "adjacent profile"
[[ "$(grep -c '^claude:' "$CALL_LOG")" -eq 1 ]] || fail "ADJACENT must invoke one typed delta validation"

# Complete READY orderで正規metadataが追加されることを確認する。
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
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
assert_has "$ORDER" "READ_MODE: CAPSULE" "bounded context mode"
assert_has "$ORDER" "VIEW_PROFILE: CLOSED" "computed view profile"
assert_has "$ORDER" "HAZARD_GUARD: NONE" "runner-owned hazard guard"
assert_has "$ORDER" \
  "OPUS_DELTA_FINDING: F1 NEGATIVE_ORACLE P2 src.txt must remain unchanged when authority validation fails." \
  "typed delta finding"
assert_has "$ORDER" "OPUS_DELTA_REASON: fixture skeleton is closed" "typed delta reason"

# destructive filesystem grainは、Opusの気分に依存せず既知の機械guardを注入する。
HAZARD_ORDER="$TMP_ROOT/hazard-order.md"
HAZARD_TASK="$(printf '%s\n' "$TASK" | sed 's/^HAZARD_TAG: NONE$/HAZARD_TAG: DESTRUCTIVE_FS/')"
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  FAKE_OPUS_OUTPUT="$OPUS_READY" \
  "$SCRIPT" prepare "$WT" "$HAZARD_ORDER" "$HAZARD_TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 0 "$RUN_STATUS" "destructive hazard prepare"
assert_fragment "$HAZARD_ORDER" \
  "HAZARD_GUARD: DESTRUCTIVE_FS requires exact non-empty resolved targets" \
  "destructive filesystem guard"
assert_has "$HAZARD_ORDER" \
  "MECHANICAL_GUARD: reject empty collection expansion and the token sequence [@]:- on any recursive-delete path." \
  "empty-array expansion guard"
assert_fragment "$HAZARD_ORDER" \
  "NEGATIVE_ORACLE: zero deletion targets must perform zero recursive removals" \
  "zero-target negative oracle"

# CLI自体の失敗はschema不正ではないためsame-session retryを行わない。
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  FAKE_OPUS_STATUS=42 \
  "$SCRIPT" prepare "$WT" "$TMP_ROOT/cli-failure-order.md" "$TASK" \
  >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 1 "$RUN_STATUS" "Opus CLI failure"
[[ "$(grep -c '^claude:' "$CALL_LOG")" -eq 1 ]] || fail "CLI failure must not trigger schema retry"

# schema不正は同じsessionへ一度だけ差戻し、二回目が正しければ採用する。
RETRY_ORDER="$TMP_ROOT/retry-order.md"
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  FAKE_OPUS_OUTPUT='not-json' \
  FAKE_OPUS_RETRY_OUTPUT="$OPUS_READY" \
  "$SCRIPT" prepare "$WT" "$RETRY_ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 0 "$RUN_STATUS" "same-session schema retry succeeds"
[[ "$(grep -c '^claude:' "$CALL_LOG")" -eq 2 ]] || fail "schema retry must invoke Claude exactly twice"
assert_fragment "$CALL_LOG" "--resume " "schema retry resumes the same session"

# 二回とも不正なら散文fallbackせず終了する。
DOUBLE_BAD_ORDER="$TMP_ROOT/double-bad-order.md"
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  FAKE_OPUS_OUTPUT='not-json' \
  FAKE_OPUS_RETRY_OUTPUT='still-not-json' \
  "$SCRIPT" prepare "$WT" "$DOUBLE_BAD_ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 1 "$RUN_STATUS" "double schema failure"
[[ "$(grep -c '^claude:' "$CALL_LOG")" -eq 2 ]] || fail "double schema failure must stop after two calls"
grep -Fq -- "散文fallbackしません" "$TMP_ROOT/stderr.log" \
  || fail "double schema failure did not report fail-closed fallback policy"

# Codexが各deltaを実装／試験へどう閉じるか明記しない限り、Sparkを起動しない。
UNRESOLVED_ORDER="$TMP_ROOT/unresolved-order.md"
cp "$ORDER" "$UNRESOLVED_ORDER"
printf 'CODEX PRECHECK: APPROVED\n' >>"$UNRESOLVED_ORDER"
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  "$SCRIPT" execute "$WT" "$UNRESOLVED_ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 3 "$RUN_STATUS" "unresolved Opus delta"
grep -Fq -- "every Opus delta finding requires one matching DELTA_RESOLUTION" "$TMP_ROOT/stderr.log" \
  || fail "missing delta resolution reason is absent"
[[ ! -s "$CALL_LOG" ]] || fail "unresolved delta must fail before Spark"

printf '%s\n' \
  'DELTA_RESOLUTION: F1 Preserve the authority-failure fixture as an explicit unchanged-file negative test.' \
  >>"$ORDER"
printf 'CODEX PRECHECK: APPROVED\n' >>"$ORDER"

# 読込予算超過は外部modelを起動する前にfail closedする。
OVER_BUDGET_ORDER="$TMP_ROOT/over-budget-order.md"
cp "$ORDER" "$OVER_BUDGET_ORDER"
for n in $(seq 1 13); do
  printf 'READ_FILE: read-%s.txt\n' "$n" >>"$OVER_BUDGET_ORDER"
  printf 'read\n' >"$WT/read-$n.txt"
done
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  "$SCRIPT" execute "$WT" "$OVER_BUDGET_ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 3 "$RUN_STATUS" "read budget overflow"
grep -Fq -- "READ_FILE count must be 1..12" "$TMP_ROOT/stderr.log" \
  || fail "read budget rejection reason is missing"
[[ ! -s "$CALL_LOG" ]] || fail "read budget overflow must fail before model invocation"
rm -f "$WT"/read-*.txt

# 複合状態をDOへ緩めず、依存を散文や別表から推測しない。
SPEC_ORDER="$TMP_ROOT/spec-order.md"
sed 's/^GRAIN: GRAIN-1$/GRAIN: SPEC-1/' "$ORDER" >"$SPEC_ORDER"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
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
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  "$SCRIPT" execute "$WT" "$MISSING_DEP_ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 3 "$RUN_STATUS" "dependency absent from evidence table"
grep -Fq -- "dependency MISSING-DEP not found in dependency-evidence ledger" "$TMP_ROOT/stderr.log" \
  || fail "missing dependency rejection reason is missing"
[[ ! -s "$CALL_LOG" ]] || fail "missing dependency must fail before model invocation"

# compiled grainの必須施工fieldが欠けたorderは、Spark起動前にfail closedする。
MISSING_GRAIN_FIELD_ORDER="$TMP_ROOT/missing-grain-field-order.md"
sed '/^Objective:/d' "$ORDER" >"$MISSING_GRAIN_FIELD_ORDER"
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  "$SCRIPT" execute "$WT" "$MISSING_GRAIN_FIELD_ORDER" "$TASK" \
  >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 3 "$RUN_STATUS" "compiled grain missing required field"
grep -Fq -- "order missing compiled Spark grain field: Objective" "$TMP_ROOT/stderr.log" \
  || fail "compiled grain missing-field reason is absent"
[[ ! -s "$CALL_LOG" ]] || fail "incomplete compiled grain must fail before Spark"

# Sparkがworktree外の生成grainを改変しても、parent保持hashとの不一致で検収へ進めない。
TAMPER_GRAIN_HOOK="$TMP_ROOT/tamper-grain-hook.sh"
cat >"$TAMPER_GRAIN_HOOK" <<EOF
#!/usr/bin/env bash
attempt_dir="\$(find "${ORDER}.evidence" -maxdepth 1 -type d -name 'attempt-*' | sort | tail -n 1)"
printf 'tampered\n' >>"\$attempt_dir/spark-compiled-grain.txt"
printf 'after\n' >"$WT/src.txt"
EOF
chmod +x "$TAMPER_GRAIN_HOOK"
: >"$CALL_LOG"
set +e
env -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  FAKE_SPARK_HOOK="$TAMPER_GRAIN_HOOK" \
  "$SCRIPT" execute "$WT" "$ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 6 "$RUN_STATUS" "compiled grain mutation rejected"
grep -Fq -- "compiled Spark grain mutated during Spark implementation" "$TMP_ROOT/stderr.log" \
  || fail "compiled grain mutation reason is absent"
printf 'before\n' >"$WT/src.txt"

SPARK_HOOK="$TMP_ROOT/spark-hook.sh"
cat >"$SPARK_HOOK" <<EOF
#!/usr/bin/env bash
printf 'after\n' >"$WT/src.txt"
EOF
chmod +x "$SPARK_HOOK"

: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
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
assert_fragment "$CALL_LOG" "--ignore-user-config" "Spark ignores ambient user config"
assert_fragment "$CALL_LOG" "--disable memories" "Spark memory is disabled"
assert_fragment "$CALL_LOG" "--disable plugins" "Spark plugins are disabled"
assert_fragment "$CALL_LOG" "--disable multi_agent" "Spark delegation is disabled"
assert_fragment "$CALL_LOG" "--sandbox workspace-write" "Spark write scope is sandboxed"
assert_fragment "$CALL_LOG" "TARGET_CAPSULE:" "Spark receives target capsule"
assert_fragment "$CALL_LOG" "COMPILED_IMPLEMENTATION_GRAIN:" "Spark receives compiled grain"
assert_fragment "$CALL_LOG" "=== src.txt line 1 anchor: before ===" "internal target neighborhood"
assert_fragment "$CALL_LOG" "=== test.txt line 1 anchor: fixture oracle ===" "test target neighborhood"
assert_fragment "$CALL_LOG" "new command, mode, public API" "new surface prohibition"
assert_fragment "$CALL_LOG" "Run every Test and DELTA_RESOLUTION check" "Spark must run required tests"
if grep -Fq -- "claude:" "$CALL_LOG"; then
  fail "execute must not rerun Opus or Fable"
fi
codex_line="$(grep -n '^codex:' "$CALL_LOG" | cut -d: -f1)"
grok_line="$(grep -n '^cursor:' "$CALL_LOG" | cut -d: -f1)"
[[ "$codex_line" -lt "$grok_line" ]] || fail "Spark must run before Grok"

happy_attempt_name="$(awk -F': ' '$1=="ATTEMPT"{print $2}' "${ORDER}.evidence/checkpoint.txt")"
happy_attempt="${ORDER}.evidence/$happy_attempt_name"
assert_has "$happy_attempt/spark-compiled-grain.txt" \
  "SPARK_GRAIN_VERSION: 1" "compiled grain version"
assert_has "$happy_attempt/spark-compiled-grain.txt" \
  "Objective: update the allowed fixture." "compiled objective"
assert_has "$happy_attempt/spark-compiled-grain.txt" \
  "ALLOWED_FILE: src.txt" "compiled allowlist"
assert_has "$happy_attempt/spark-compiled-grain.txt" \
  "DELTA_RESOLUTION: F1 Preserve the authority-failure fixture as an explicit unchanged-file negative test." \
  "compiled delta resolution"
assert_has "$happy_attempt/spark-compiled-grain.txt" \
  "Test: git diff --check." "compiled test"
if grep -Eq '^(AUTHORITY|BASE_SHA|VIEW_PROFILE|TASK_SHA256|CODEX PRECHECK|OPUS_DELTA_REASON):' \
  "$happy_attempt/spark-compiled-grain.txt"; then
  fail "compiled grain retained runner-only metadata"
fi
if grep -Fq -- "Original user task:" "$happy_attempt/spark-prompt.txt" ||
   grep -Fq -- "Binding order:" "$happy_attempt/spark-prompt.txt"; then
  fail "Spark prompt retained duplicated task or full order"
fi
[[ "$(grep -c '^Objective:' "$happy_attempt/spark-prompt.txt")" -eq 1 ]] \
  || fail "Spark prompt must contain Objective exactly once"
[[ "$(grep -c '^Test:' "$happy_attempt/spark-prompt.txt")" -eq 1 ]] \
  || fail "Spark prompt must contain Test exactly once"
assert_has "$happy_attempt/metadata.txt" \
  "TARGET_CAPSULE_SHA256: $(sha256_file "$happy_attempt/spark-target-capsule.txt")" \
  "target capsule hash evidence"
assert_has "$happy_attempt/metadata.txt" \
  "SPARK_GRAIN_SHA256: $(sha256_file "$happy_attempt/spark-compiled-grain.txt")" \
  "compiled grain hash evidence"
assert_has "$happy_attempt/metadata.txt" \
  "SPARK_PROMPT_SHA256: $(sha256_file "$happy_attempt/spark-prompt.txt")" \
  "Spark prompt hash evidence"

# execute側はSpark/Grokのwall timeを必ず残す。両CLIはusageを返さないため、
# tokenは推測せずUNKNOWNとし、欠測stage名を合計行へ明示する
HAPPY_TELEMETRY="$happy_attempt/telemetry.txt"
[[ -f "$HAPPY_TELEMETRY" ]] || fail "execute must record telemetry"
grep -Eq '^SPARK_WALL_SECONDS: [0-9]+$' "$HAPPY_TELEMETRY" || fail "Spark wall seconds must be numeric"
grep -Eq '^GROK_WALL_SECONDS: [0-9]+$' "$HAPPY_TELEMETRY" || fail "Grok wall seconds must be numeric"
assert_has "$HAPPY_TELEMETRY" "SPARK_INPUT_TOKENS: UNKNOWN" "Spark tokens are unmeasured, not guessed"
assert_has "$HAPPY_TELEMETRY" "GROK_INPUT_TOKENS: UNKNOWN" "Grok tokens are unmeasured, not guessed"
grep -Eq '^UNMEASURED_TOKEN_STAGES:.*SPARK_INPUT_TOKENS' "$HAPPY_TELEMETRY" \
  || fail "unmeasured Spark tokens must be named in the summary"
grep -Eq '^UNMEASURED_TOKEN_STAGES:.*GROK_INPUT_TOKENS' "$HAPPY_TELEMETRY" \
  || fail "unmeasured Grok tokens must be named in the summary"
grep -Eq '^TOTAL_WALL_SECONDS: [0-9]+$' "$HAPPY_TELEMETRY" || fail "execute must total wall seconds"

GROK_HOOK="$TMP_ROOT/grok-hook.sh"
cat >"$GROK_HOOK" <<EOF
#!/usr/bin/env bash
printf 'reviewer mutation\n' >>"$WT/src.txt"
EOF
chmod +x "$GROK_HOOK"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
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
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" \
  FAKE_CALL_LOG="$CALL_LOG" \
  "$SCRIPT" execute "$WT" "$STALE_ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 3 "$RUN_STATUS" "archived routing rejected"
[[ ! -s "$CALL_LOG" ]] || fail "stale routing must fail before model invocation"

: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" CLAUDE_DELEGATED=1 \
  "$SCRIPT" prepare "$WT" "$ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 2 "$RUN_STATUS" "recursive Claude dispatch rejected"
[[ ! -s "$CALL_LOG" ]] || fail "recursive dispatch must not invoke a model"

# --- GR-D3 derived target closure oracles ---

WT_D3="$TMP_ROOT/gr-d3-base"
WT_D3="$(gr_d3_init_worktree "$WT_D3")"
ORDER_D3="$TMP_ROOT/gr-d3-order.md"
gr_d3_ready_order "$WT_D3" "$ORDER_D3"

SPARK_POSITIVE="$TMP_ROOT/gr-d3-spark-positive.sh"
cat >"$SPARK_POSITIVE" <<EOF
#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$WT_D3/target/scaffold-plugin-fixture/nested"
printf 'nested\n' >"$WT_D3/target/scaffold-plugin-fixture/nested/file.txt"
mkdir -p "$WT_D3/target/new-plugin-scaffold-test"
printf 'spark-test\n' >"$WT_D3/target/new-plugin-scaffold-test/inside.txt"
printf 'tsv-body\n' >"$WT_D3/target/d1i4-empty-classification.tsv"
printf 'after\n' >>"$WT_D3/src.txt"
EOF
chmod +x "$SPARK_POSITIVE"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" FAKE_SPARK_HOOK="$SPARK_POSITIVE" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" execute "$WT_D3" "$ORDER_D3" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 0 "$RUN_STATUS" "GR-D3 positive post-Spark closure"
[[ ! -e "$WT_D3/target" ]] || fail "GR-D3 positive: target root still present"
grep -Fqx 'after' "$WT_D3/src.txt" || fail "GR-D3 positive: allowed edit missing"
latest_attempt="$(find "$ORDER_D3.evidence" -mindepth 1 -maxdepth 1 -type d | LC_ALL=C sort | tail -n 1)"
[[ -n "$latest_attempt" ]] || fail "GR-D3 ref digest oracle: missing execute attempt evidence"
runner_ref_digest="$(tr -d '[:space:]' <"$latest_attempt/pre-spark-ref-digest.sha256")"
canonical_ref_digest_value="$(canonical_ref_digest "$WT_D3")"
[[ "$runner_ref_digest" == "$canonical_ref_digest_value" ]] \
  || fail "GR-D3 ref digest recipe: runner=$runner_ref_digest canonical=$canonical_ref_digest_value"

WT_EMPTY="$TMP_ROOT/gr-d3-empty-target"
WT_EMPTY="$(gr_d3_init_worktree "$WT_EMPTY")"
ORDER_EMPTY="$TMP_ROOT/gr-d3-order-empty.md"
gr_d3_ready_order "$WT_EMPTY" "$ORDER_EMPTY"
SPARK_EMPTY="$TMP_ROOT/gr-d3-spark-empty.sh"
cat >"$SPARK_EMPTY" <<EOF
#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$WT_EMPTY/target"
printf 'after\n' >>"$WT_EMPTY/src.txt"
EOF
chmod +x "$SPARK_EMPTY"
: >"$RM_ARGS_LOG"
: >"$RMDIR_ARGS_LOG"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" FAKE_SPARK_HOOK="$SPARK_EMPTY" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" execute "$WT_EMPTY" "$ORDER_EMPTY" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 0 "$RUN_STATUS" "GR-D3 empty target oracle"
[[ ! -e "$WT_EMPTY/target" ]] || fail "GR-D3 empty target: directory still present"
assert_rm_never_removed_target_root "$WT_EMPTY" "GR-D3 empty target rm oracle"
assert_rmdir_removed_target_root "$WT_EMPTY" "GR-D3 empty target rmdir oracle"

WT_UNKNOWN="$TMP_ROOT/gr-d3-unknown"
WT_UNKNOWN="$(gr_d3_init_worktree "$WT_UNKNOWN")"
ORDER_UNKNOWN="$TMP_ROOT/gr-d3-order-unknown.md"
gr_d3_ready_order "$WT_UNKNOWN" "$ORDER_UNKNOWN"
SPARK_UNKNOWN="$TMP_ROOT/gr-d3-spark-unknown.sh"
cat >"$SPARK_UNKNOWN" <<EOF
#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$WT_UNKNOWN/target/scaffold-plugin-fixture/nested"
printf 'marker-scaffold\n' >"$WT_UNKNOWN/target/scaffold-plugin-fixture/nested/marker.txt"
mkdir -p "$WT_UNKNOWN/target/new-plugin-scaffold-test"
printf 'marker-scaffold-test\n' >"$WT_UNKNOWN/target/new-plugin-scaffold-test/marker.txt"
printf 'marker-tsv\n' >"$WT_UNKNOWN/target/d1i4-empty-classification.tsv"
printf 'unknown-marker\n' >"$WT_UNKNOWN/target/unknown-thing"
printf 'after\n' >>"$WT_UNKNOWN/src.txt"
EOF
chmod +x "$SPARK_UNKNOWN"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" FAKE_SPARK_HOOK="$SPARK_UNKNOWN" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" execute "$WT_UNKNOWN" "$ORDER_UNKNOWN" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 7 "$RUN_STATUS" "GR-D3 unknown child"
assert_fragment "$TMP_ROOT/stderr.log" "unknown-thing" "GR-D3 unknown stderr"
assert_three_known_target_entries "$WT_UNKNOWN" "GR-D3 unknown preservation"
grep -Fqx 'unknown-marker' "$WT_UNKNOWN/target/unknown-thing" \
  || fail "GR-D3 unknown: marker file missing"

WT_SYMLINK_ROOT="$TMP_ROOT/gr-d3-symlink-root"
WT_SYMLINK_ROOT="$(gr_d3_init_worktree "$WT_SYMLINK_ROOT")"
ORDER_SYMLINK_ROOT="$TMP_ROOT/gr-d3-order-symlink-root.md"
gr_d3_ready_order "$WT_SYMLINK_ROOT" "$ORDER_SYMLINK_ROOT"
OUTSIDE_DIR="$TMP_ROOT/gr-d3-outside-marker"
mkdir -p "$OUTSIDE_DIR"
printf 'outside-marker\n' >"$OUTSIDE_DIR/marker.txt"
SPARK_SYMLINK_ROOT="$TMP_ROOT/gr-d3-spark-symlink-root.sh"
cat >"$SPARK_SYMLINK_ROOT" <<EOF
#!/usr/bin/env bash
set -euo pipefail
ln -s "$OUTSIDE_DIR" "$WT_SYMLINK_ROOT/target"
printf 'after\n' >>"$WT_SYMLINK_ROOT/src.txt"
EOF
chmod +x "$SPARK_SYMLINK_ROOT"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" FAKE_SPARK_HOOK="$SPARK_SYMLINK_ROOT" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" execute "$WT_SYMLINK_ROOT" "$ORDER_SYMLINK_ROOT" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 7 "$RUN_STATUS" "GR-D3 symlink target root"
[[ -L "$WT_SYMLINK_ROOT/target" ]] || fail "GR-D3 symlink root: link removed"
grep -Fqx 'outside-marker' "$OUTSIDE_DIR/marker.txt" || fail "GR-D3 symlink root: outside marker missing"

WT_DANGLING="$TMP_ROOT/gr-d3-dangling-child"
WT_DANGLING="$(gr_d3_init_worktree "$WT_DANGLING")"
ORDER_DANGLING="$TMP_ROOT/gr-d3-order-dangling.md"
gr_d3_ready_order "$WT_DANGLING" "$ORDER_DANGLING"
SPARK_DANGLING="$TMP_ROOT/gr-d3-spark-dangling.sh"
cat >"$SPARK_DANGLING" <<EOF
#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$WT_DANGLING/target/scaffold-plugin-fixture"
printf 'real-scaffold\n' >"$WT_DANGLING/target/scaffold-plugin-fixture/marker.txt"
ln -s "$WT_DANGLING/target/missing" "$WT_DANGLING/target/new-plugin-scaffold-test"
printf 'marker-tsv\n' >"$WT_DANGLING/target/d1i4-empty-classification.tsv"
printf 'after\n' >>"$WT_DANGLING/src.txt"
EOF
chmod +x "$SPARK_DANGLING"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" FAKE_SPARK_HOOK="$SPARK_DANGLING" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" execute "$WT_DANGLING" "$ORDER_DANGLING" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 7 "$RUN_STATUS" "GR-D3 dangling known child"
[[ -L "$WT_DANGLING/target/new-plugin-scaffold-test" ]] || fail "GR-D3 dangling: symlink removed"
grep -Fqx 'real-scaffold' "$WT_DANGLING/target/scaffold-plugin-fixture/marker.txt" \
  || fail "GR-D3 dangling: real sibling missing"

WT_NESTED_LIVE="$TMP_ROOT/gr-d3-nested-live"
WT_NESTED_LIVE="$(gr_d3_init_worktree "$WT_NESTED_LIVE")"
ORDER_NESTED_LIVE="$TMP_ROOT/gr-d3-order-nested-live.md"
gr_d3_ready_order "$WT_NESTED_LIVE" "$ORDER_NESTED_LIVE"
SPARK_NESTED_LIVE="$TMP_ROOT/gr-d3-spark-nested-live.sh"
cat >"$SPARK_NESTED_LIVE" <<EOF
#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$WT_NESTED_LIVE/target/scaffold-plugin-fixture"
printf 'sibling\n' >"$WT_NESTED_LIVE/target/scaffold-plugin-fixture/sibling.txt"
ln -s "$WT_NESTED_LIVE/target/scaffold-plugin-fixture/sibling.txt" \
  "$WT_NESTED_LIVE/target/scaffold-plugin-fixture/live-link"
mkdir -p "$WT_NESTED_LIVE/target/new-plugin-scaffold-test"
printf 'marker-tsv\n' >"$WT_NESTED_LIVE/target/d1i4-empty-classification.tsv"
printf 'after\n' >>"$WT_NESTED_LIVE/src.txt"
EOF
chmod +x "$SPARK_NESTED_LIVE"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" FAKE_SPARK_HOOK="$SPARK_NESTED_LIVE" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" execute "$WT_NESTED_LIVE" "$ORDER_NESTED_LIVE" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 7 "$RUN_STATUS" "GR-D3 nested live symlink"
[[ -L "$WT_NESTED_LIVE/target/scaffold-plugin-fixture/live-link" ]] \
  || fail "GR-D3 nested live: symlink removed"
grep -Fqx 'sibling' "$WT_NESTED_LIVE/target/scaffold-plugin-fixture/sibling.txt" \
  || fail "GR-D3 nested live: sibling missing"

WT_NESTED_DANGLING="$TMP_ROOT/gr-d3-nested-dangling"
WT_NESTED_DANGLING="$(gr_d3_init_worktree "$WT_NESTED_DANGLING")"
ORDER_NESTED_DANGLING="$TMP_ROOT/gr-d3-order-nested-dangling.md"
gr_d3_ready_order "$WT_NESTED_DANGLING" "$ORDER_NESTED_DANGLING"
SPARK_NESTED_DANGLING="$TMP_ROOT/gr-d3-spark-nested-dangling.sh"
cat >"$SPARK_NESTED_DANGLING" <<EOF
#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$WT_NESTED_DANGLING/target/new-plugin-scaffold-test"
printf 'keep-me\n' >"$WT_NESTED_DANGLING/target/new-plugin-scaffold-test/keep.txt"
ln -s "$WT_NESTED_DANGLING/target/missing-inside" \
  "$WT_NESTED_DANGLING/target/new-plugin-scaffold-test/dangle"
mkdir -p "$WT_NESTED_DANGLING/target/scaffold-plugin-fixture"
printf 'marker-tsv\n' >"$WT_NESTED_DANGLING/target/d1i4-empty-classification.tsv"
printf 'after\n' >>"$WT_NESTED_DANGLING/src.txt"
EOF
chmod +x "$SPARK_NESTED_DANGLING"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" FAKE_SPARK_HOOK="$SPARK_NESTED_DANGLING" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" execute "$WT_NESTED_DANGLING" "$ORDER_NESTED_DANGLING" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 7 "$RUN_STATUS" "GR-D3 nested dangling symlink"
[[ -L "$WT_NESTED_DANGLING/target/new-plugin-scaffold-test/dangle" ]] \
  || fail "GR-D3 nested dangling: symlink removed"
grep -Fqx 'keep-me' "$WT_NESTED_DANGLING/target/new-plugin-scaffold-test/keep.txt" \
  || fail "GR-D3 nested dangling: sibling missing"

WT_TRACKED="$TMP_ROOT/gr-d3-tracked"
WT_TRACKED="$(gr_d3_init_worktree "$WT_TRACKED")"
ORDER_TRACKED="$TMP_ROOT/gr-d3-order-tracked.md"
gr_d3_ready_order "$WT_TRACKED" "$ORDER_TRACKED"
SPARK_TRACKED_SETUP="$TMP_ROOT/gr-d3-spark-tracked-setup.sh"
cat >"$SPARK_TRACKED_SETUP" <<EOF
#!/usr/bin/env bash
set -euo pipefail
printf 'after\n' >>"$WT_TRACKED/src.txt"
EOF
chmod +x "$SPARK_TRACKED_SETUP"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" FAKE_SPARK_HOOK="$SPARK_TRACKED_SETUP" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" execute "$WT_TRACKED" "$ORDER_TRACKED" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 0 "$RUN_STATUS" "GR-D3 tracked setup execute"
mkdir -p "$WT_TRACKED/target"
printf 'tracked-body\n' >"$WT_TRACKED/target/d1i4-empty-classification.tsv"
git_fixture "$WT_TRACKED" add -f target/d1i4-empty-classification.tsv
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" inspect "$WT_TRACKED" "$ORDER_TRACKED" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 7 "$RUN_STATUS" "GR-D3 tracked known path"
assert_fragment "$TMP_ROOT/stderr.log" "tracked derived entry" "GR-D3 tracked message"
grep -Fqx 'tracked-body' "$WT_TRACKED/target/d1i4-empty-classification.tsv" \
  || fail "GR-D3 tracked: file content changed"
git_fixture "$WT_TRACKED" rm --cached -f target/d1i4-empty-classification.tsv

if [[ "$(id -u)" -eq 0 ]]; then
  fail "GR-D3 removal failure test requires a non-root user"
fi
WT_REMOVAL="$TMP_ROOT/gr-d3-removal-fail"
WT_REMOVAL="$(gr_d3_init_worktree "$WT_REMOVAL")"
ORDER_REMOVAL="$TMP_ROOT/gr-d3-order-removal.md"
gr_d3_ready_order "$WT_REMOVAL" "$ORDER_REMOVAL"
SPARK_REMOVAL="$TMP_ROOT/gr-d3-spark-removal.sh"
cat >"$SPARK_REMOVAL" <<EOF
#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$WT_REMOVAL/target/scaffold-plugin-fixture"
printf 'm1\n' >"$WT_REMOVAL/target/scaffold-plugin-fixture/m.txt"
mkdir -p "$WT_REMOVAL/target/new-plugin-scaffold-test"
printf 'm2\n' >"$WT_REMOVAL/target/new-plugin-scaffold-test/m.txt"
printf 'm3\n' >"$WT_REMOVAL/target/d1i4-empty-classification.tsv"
chmod 555 "$WT_REMOVAL/target"
if ( : >"$WT_REMOVAL/target/.probe" ) 2>/dev/null; then
  echo "target directory remained writable after chmod 555" >&2
  exit 1
fi
printf 'after\n' >>"$WT_REMOVAL/src.txt"
EOF
chmod +x "$SPARK_REMOVAL"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" FAKE_SPARK_HOOK="$SPARK_REMOVAL" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" execute "$WT_REMOVAL" "$ORDER_REMOVAL" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
chmod u+w "$WT_REMOVAL/target" 2>/dev/null || true
assert_status 7 "$RUN_STATUS" "GR-D3 deterministic removal failure"
assert_fragment "$TMP_ROOT/stderr.log" "removal failed" "GR-D3 removal failure message"
grep -Fqx 'm1' "$WT_REMOVAL/target/scaffold-plugin-fixture/m.txt" \
  || fail "GR-D3 removal failure: content lost"

WT_INSPECT_RESUME="$TMP_ROOT/gr-d3-inspect-resume"
WT_INSPECT_RESUME="$(gr_d3_init_worktree "$WT_INSPECT_RESUME")"
ORDER_INSPECT_RESUME="$TMP_ROOT/gr-d3-order-inspect-resume.md"
gr_d3_ready_order "$WT_INSPECT_RESUME" "$ORDER_INSPECT_RESUME"
SPARK_INSPECT_RESUME="$TMP_ROOT/gr-d3-spark-inspect-resume.sh"
cat >"$SPARK_INSPECT_RESUME" <<EOF
#!/usr/bin/env bash
set -euo pipefail
printf 'after\n' >>"$WT_INSPECT_RESUME/src.txt"
EOF
chmod +x "$SPARK_INSPECT_RESUME"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" FAKE_SPARK_HOOK="$SPARK_INSPECT_RESUME" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" execute "$WT_INSPECT_RESUME" "$ORDER_INSPECT_RESUME" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 0 "$RUN_STATUS" "GR-D3 inspect-resume setup execute"
create_three_known_target_entries "$WT_INSPECT_RESUME"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" inspect "$WT_INSPECT_RESUME" "$ORDER_INSPECT_RESUME" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 0 "$RUN_STATUS" "GR-D3 inspect-resume closure"
[[ ! -e "$WT_INSPECT_RESUME/target" ]] || fail "GR-D3 inspect-resume: target still present"

WT_POST_GROK="$TMP_ROOT/gr-d3-post-grok"
WT_POST_GROK="$(gr_d3_init_worktree "$WT_POST_GROK")"
ORDER_POST_GROK="$TMP_ROOT/gr-d3-order-post-grok.md"
gr_d3_ready_order "$WT_POST_GROK" "$ORDER_POST_GROK"
SPARK_POST_GROK="$TMP_ROOT/gr-d3-spark-post-grok.sh"
cat >"$SPARK_POST_GROK" <<EOF
#!/usr/bin/env bash
set -euo pipefail
printf 'after\n' >>"$WT_POST_GROK/src.txt"
EOF
chmod +x "$SPARK_POST_GROK"
GROK_POST_GROK="$TMP_ROOT/gr-d3-grok-post-grok.sh"
cat >"$GROK_POST_GROK" <<EOF
#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$WT_POST_GROK/target/scaffold-plugin-fixture"
printf 'grok\n' >"$WT_POST_GROK/target/scaffold-plugin-fixture/g.txt"
mkdir -p "$WT_POST_GROK/target/new-plugin-scaffold-test"
printf 'grok2\n' >"$WT_POST_GROK/target/new-plugin-scaffold-test/g.txt"
printf 'grok-tsv\n' >"$WT_POST_GROK/target/d1i4-empty-classification.tsv"
EOF
chmod +x "$GROK_POST_GROK"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
  FAKE_SPARK_HOOK="$SPARK_POST_GROK" FAKE_GROK_HOOK="$GROK_POST_GROK" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" execute "$WT_POST_GROK" "$ORDER_POST_GROK" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 0 "$RUN_STATUS" "GR-D3 post-reaped-Grok closure"
[[ ! -e "$WT_POST_GROK/target" ]] || fail "GR-D3 post-grok: target still present"

WT_GROK_UNKNOWN="$TMP_ROOT/gr-d3-grok-unknown"
WT_GROK_UNKNOWN="$(gr_d3_init_worktree "$WT_GROK_UNKNOWN")"
ORDER_GROK_UNKNOWN="$TMP_ROOT/gr-d3-order-grok-unknown.md"
gr_d3_ready_order "$WT_GROK_UNKNOWN" "$ORDER_GROK_UNKNOWN"
SPARK_GROK_UNKNOWN="$TMP_ROOT/gr-d3-spark-grok-unknown.sh"
cat >"$SPARK_GROK_UNKNOWN" <<EOF
#!/usr/bin/env bash
set -euo pipefail
printf 'after\n' >>"$WT_GROK_UNKNOWN/src.txt"
EOF
chmod +x "$SPARK_GROK_UNKNOWN"
GROK_UNKNOWN="$TMP_ROOT/gr-d3-grok-unknown-hook.sh"
cat >"$GROK_UNKNOWN" <<EOF
#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$WT_GROK_UNKNOWN/target"
printf 'grok-unknown-marker\n' >"$WT_GROK_UNKNOWN/target/grok-unknown"
EOF
chmod +x "$GROK_UNKNOWN"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
  FAKE_SPARK_HOOK="$SPARK_GROK_UNKNOWN" FAKE_GROK_HOOK="$GROK_UNKNOWN" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" execute "$WT_GROK_UNKNOWN" "$ORDER_GROK_UNKNOWN" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 7 "$RUN_STATUS" "GR-D3 post-grok unknown mutation"
grep -Fqx 'grok-unknown-marker' "$WT_GROK_UNKNOWN/target/grok-unknown" \
  || fail "GR-D3 grok unknown: mutation removed"
[[ -f "${ORDER_GROK_UNKNOWN}.evidence/checkpoint.txt" ]] \
  && fail "GR-D3 grok unknown: checkpoint must not remain after scope failure"

WT_REF="$TMP_ROOT/gr-d3-ref-drift"
WT_REF="$(gr_d3_init_worktree "$WT_REF")"
ORDER_REF="$TMP_ROOT/gr-d3-order-ref.md"
gr_d3_ready_order "$WT_REF" "$ORDER_REF"
SPARK_REF="$TMP_ROOT/gr-d3-spark-ref.sh"
cat >"$SPARK_REF" <<EOF
#!/usr/bin/env bash
set -euo pipefail
head="\$(env -u GIT_DIR -u GIT_WORK_TREE -u GIT_INDEX_FILE -u GIT_OBJECT_DIRECTORY \
  -u GIT_ALTERNATE_OBJECT_DIRECTORIES -u GIT_COMMON_DIR -u GIT_NAMESPACE \
  -u GIT_CEILING_DIRECTORIES -u GIT_DISCOVERY_ACROSS_FILESYSTEM \
  git --git-dir="$WT_REF/.git" --work-tree="$WT_REF" rev-parse HEAD)"
env -u GIT_DIR -u GIT_WORK_TREE -u GIT_INDEX_FILE -u GIT_OBJECT_DIRECTORY \
  -u GIT_ALTERNATE_OBJECT_DIRECTORIES -u GIT_COMMON_DIR -u GIT_NAMESPACE \
  -u GIT_CEILING_DIRECTORIES -u GIT_DISCOVERY_ACROSS_FILESYSTEM \
  git --git-dir="$WT_REF/.git" --work-tree="$WT_REF" update-ref refs/heads/sneaky "\$head"
printf 'after\n' >>"$WT_REF/src.txt"
EOF
chmod +x "$SPARK_REF"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" FAKE_SPARK_HOOK="$SPARK_REF" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" execute "$WT_REF" "$ORDER_REF" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 5 "$RUN_STATUS" "GR-D3 ref digest drift"
assert_fragment "$TMP_ROOT/stderr.log" "REF NG: protected ref digest changed during implementation" \
  "GR-D3 ref stderr"

WT_COMMIT="$TMP_ROOT/gr-d3-commit"
WT_COMMIT="$(gr_d3_init_worktree "$WT_COMMIT")"
ORDER_COMMIT="$TMP_ROOT/gr-d3-order-commit.md"
gr_d3_ready_order "$WT_COMMIT" "$ORDER_COMMIT"
SPARK_COMMIT="$TMP_ROOT/gr-d3-spark-commit.sh"
cat >"$SPARK_COMMIT" <<EOF
#!/usr/bin/env bash
set -euo pipefail
printf 'after\n' >>"$WT_COMMIT/src.txt"
env -u GIT_DIR -u GIT_WORK_TREE -u GIT_INDEX_FILE -u GIT_OBJECT_DIRECTORY \
  -u GIT_ALTERNATE_OBJECT_DIRECTORIES -u GIT_COMMON_DIR -u GIT_NAMESPACE \
  -u GIT_CEILING_DIRECTORIES -u GIT_DISCOVERY_ACROSS_FILESYSTEM \
  git --git-dir="$WT_COMMIT/.git" --work-tree="$WT_COMMIT" add src.txt
env -u GIT_DIR -u GIT_WORK_TREE -u GIT_INDEX_FILE -u GIT_OBJECT_DIRECTORY \
  -u GIT_ALTERNATE_OBJECT_DIRECTORIES -u GIT_COMMON_DIR -u GIT_NAMESPACE \
  -u GIT_CEILING_DIRECTORIES -u GIT_DISCOVERY_ACROSS_FILESYSTEM \
  git --git-dir="$WT_COMMIT/.git" --work-tree="$WT_COMMIT" \
  -c user.email=test@example.com -c user.name=test commit -q -m forbidden
EOF
chmod +x "$SPARK_COMMIT"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" FAKE_SPARK_HOOK="$SPARK_COMMIT" \
  FAKE_GROK_OUTPUT="VERDICT: ACCEPT" \
  "$SCRIPT" execute "$WT_COMMIT" "$ORDER_COMMIT" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 5 "$RUN_STATUS" "GR-D3 commit rejection"
grep -Fq -- "受注者がcommitを作成したため検収へ進みません" "$TMP_ROOT/stderr.log" \
  || fail "GR-D3 commit forbidden message missing"

: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  GIT_DIR="$WT_D3/.git" PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
  "$SCRIPT" execute "$WT_D3" "$ORDER_D3" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 2 "$RUN_STATUS" "GR-D3 ambient GIT_DIR rejection"
assert_fragment "$TMP_ROOT/stderr.log" "GIT_DIR" "GR-D3 ambient stderr"
[[ ! -s "$CALL_LOG" ]] || fail "GR-D3 ambient: models must not run"

TARGET_ALLOW_ORDER="$TMP_ROOT/gr-d3-target-allow-order.md"
cp "$ORDER_D3" "$TARGET_ALLOW_ORDER"
printf 'ALLOWED_FILE: target/scaffold-plugin-fixture\n' >>"$TARGET_ALLOW_ORDER"
: >"$CALL_LOG"
set +e
env "${GIT_UNSET[@]}" -u CURSOR_AGENT -u CODEX_DELEGATED -u CLAUDE_DELEGATED \
  PATH="$FAKE_BIN:/usr/bin:/bin" FAKE_CALL_LOG="$CALL_LOG" \
  "$SCRIPT" execute "$WT_D3" "$TARGET_ALLOW_ORDER" "$TASK" >"$TMP_ROOT/stdout.log" 2>"$TMP_ROOT/stderr.log"
RUN_STATUS=$?
set -e
assert_status 3 "$RUN_STATUS" "GR-D3 target allowlist rejection"
assert_fragment "$TMP_ROOT/stderr.log" "ALLOWED_FILE covers derived target output" \
  "GR-D3 target allowlist stderr"
[[ ! -s "$CALL_LOG" ]] || fail "GR-D3 target allowlist: models must not run"

grep -Fq -- '[@]:-' "$SCRIPT" && fail "GR-D3 static oracle: forbidden [@]:- token present in runner"

ROOT_HEAD_AFTER="$(root_repo_git rev-parse HEAD)"
ROOT_REF_DIGEST_AFTER="$(canonical_root_ref_digest)"
[[ "$ROOT_HEAD_BEFORE" == "$ROOT_HEAD_AFTER" ]] \
  || fail "implementation worktree ROOT HEAD changed during dedicated test run"
[[ "$ROOT_REF_DIGEST_BEFORE" == "$ROOT_REF_DIGEST_AFTER" ]] \
  || fail "implementation worktree ROOT ref digest changed during dedicated test run"
echo "ROOT_HEAD_BEFORE=$ROOT_HEAD_BEFORE"
echo "ROOT_REF_DIGEST_BEFORE=$ROOT_REF_DIGEST_BEFORE"
echo "ROOT_HEAD_AFTER=$ROOT_HEAD_AFTER"
echo "ROOT_REF_DIGEST_AFTER=$ROOT_REF_DIGEST_AFTER"

echo "test-delegate-cursor-supervised: PASS"
