#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PRIMARY_WORKTREE="$(git -C "$ROOT_DIR" worktree list --porcelain | awk '/^worktree / { print substr($0, 10); exit }')"
CLAUDE_AGENT_BIN="${CLAUDE_AGENT_BIN:-claude}"
CLAUDE_SUPERVISOR_MODEL="${CLAUDE_SUPERVISOR_MODEL:-claude-opus-4-8}"
CLAUDE_IMPLEMENTER_MODEL="${CLAUDE_IMPLEMENTER_MODEL:-claude-sonnet-5}"
SUPERVISOR_TIMEOUT_SECONDS="${CLAUDE_SUPERVISED_TIMEOUT_SECONDS:-600}"
IMPLEMENTER_TIMEOUT_SECONDS="${CLAUDE_IMPLEMENTER_TIMEOUT_SECONDS:-1800}"
HEARTBEAT_SECONDS="${CLAUDE_SUPERVISED_HEARTBEAT_SECONDS:-30}"

usage() {
  echo "Usage: $0 prepare <isolated-worktree> <order-file> <task>"
  echo "       $0 execute <isolated-worktree> <approved-order-file> <task>"
  echo "       printf '%s\n' <task> | $0 prepare|execute <isolated-worktree> <order-file>"
}

if [[ -n "${CLAUDE_DELEGATED:-}" ]]; then
  echo "delegate-claude-supervised: Claude子エージェントからの再帰実行は禁止です" >&2
  exit 2
fi

if [[ "$#" -lt 3 || "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

MODE="$1"
WORKTREE="$(cd "$2" && pwd)"
ORDER_FILE="$3"
shift 3
if [[ "$MODE" != "prepare" && "$MODE" != "execute" ]]; then
  usage >&2
  exit 2
fi
if [[ "$#" -gt 0 ]]; then
  task="$*"
else
  task="$(cat)"
fi
if [[ -z "${task//[[:space:]]/}" ]]; then
  usage >&2
  exit 2
fi

task_hash="$(printf '%s' "$task" | shasum -a 256 | awk '{print $1}')"
if [[ "$WORKTREE" == "$PRIMARY_WORKTREE" ]]; then
  echo "delegate-claude-supervised: 主作業ツリーへの実装発注は禁止です" >&2
  exit 2
fi
if ! git -C "$WORKTREE" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "delegate-claude-supervised: git worktreeではありません: $WORKTREE" >&2
  exit 2
fi
for value in "$SUPERVISOR_TIMEOUT_SECONDS" "$IMPLEMENTER_TIMEOUT_SECONDS" "$HEARTBEAT_SECONDS"; do
  if [[ ! "$value" =~ ^[1-9][0-9]*$ ]]; then
    echo "delegate-claude-supervised: timeout/heartbeatは正の整数で指定してください" >&2
    exit 2
  fi
done
if ! command -v "$CLAUDE_AGENT_BIN" >/dev/null 2>&1; then
  echo "delegate-claude-supervised: Claude Code '$CLAUDE_AGENT_BIN' が見つかりません" >&2
  exit 127
fi

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/motolii-claude-supervised.XXXXXX")"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

run_agent() {
  local output="$1"
  local timeout_seconds="$2"
  shift 2
  echo "delegate-claude-supervised: 起動: $1 (timeout=${timeout_seconds}s)" >&2
  "$@" >"$output" 2>"$output.err" &
  local pid=$!
  (
    local elapsed=0
    local interval
    while (( elapsed < timeout_seconds )); do
      interval="$HEARTBEAT_SECONDS"
      if (( elapsed + interval > timeout_seconds )); then
        interval=$((timeout_seconds - elapsed))
      fi
      sleep "$interval"
      elapsed=$((elapsed + interval))
      if ! kill -0 "$pid" 2>/dev/null; then
        exit 0
      fi
      if (( elapsed < timeout_seconds )); then
        echo "delegate-claude-supervised: 実行継続中 (${elapsed}s)" >&2
      fi
    done
    touch "$output.timeout"
    kill -TERM "$pid" 2>/dev/null || true
  ) &
  local watchdog=$!
  set +e
  wait "$pid"
  local status=$?
  set -e
  kill "$watchdog" 2>/dev/null || true
  wait "$watchdog" 2>/dev/null || true
  if [[ -f "$output.timeout" ]]; then
    echo "delegate-claude-supervised: ${timeout_seconds}秒でタイムアウトしました" >&2
    status=124
  fi
  if [[ -s "$output.err" ]]; then
    cat "$output.err" >&2
  fi
  return "$status"
}

result_is_valid() {
  local output="$1"
  local result_kind="$2"
  awk -v result_kind="$result_kind" '
    NF { last_nonempty = $0 }
    $0 == "ORDER: READY" || $0 == "ORDER: STOP" { order_markers++ }
    $0 == "VERDICT: ACCEPT" || $0 == "VERDICT: REJECT" { verdict_markers++ }
    END {
      if (result_kind == "order") {
        exit !(order_markers == 1 && verdict_markers == 0 &&
          (last_nonempty == "ORDER: READY" || last_nonempty == "ORDER: STOP"))
      }
      if (result_kind == "verdict") {
        exit !(verdict_markers == 1 && order_markers == 0 &&
          (last_nonempty == "VERDICT: ACCEPT" || last_nonempty == "VERDICT: REJECT"))
      }
      exit 1
    }
  ' "$output"
}

run_supervisor() {
  local output="$1"
  local prompt="$2"
  local result_kind="$3"
  if ! run_agent "$output" "$SUPERVISOR_TIMEOUT_SECONDS" \
    env CLAUDE_DELEGATED=1 "$CLAUDE_AGENT_BIN" -p \
      --model "$CLAUDE_SUPERVISOR_MODEL" \
      --permission-mode default \
      --allowedTools Read,Glob,Grep,Bash \
      --disallowedTools Edit,Write \
      --output-format text \
      "$prompt"; then
    return 1
  fi
  if ! result_is_valid "$output" "$result_kind"; then
    echo "delegate-claude-supervised: Opusの結果markerが欠落・曖昧・末尾外です" >&2
    return 1
  fi
}

# U0e-2の却下原因(発注書と正本の未照合)を再発させないためのgate。詳細:
# docs/reviews/2026-07-22-u0e-2-delegation-guardrails.md
REACT_LABELS_ORDERED=(
  "REACT AUTHORITY:"
  "SOURCE ASSET:"
  "PRESERVE:"
  "REPLACE:"
  "STATE OWNER:"
  "DIAGNOSTIC ROUTE:"
  "NEGATIVE ORACLE:"
  "STOP:"
)

gate_fail() {
  echo "ORDER-GATE NG: $*" >&2
  exit 3
}

gate_require_single_field() {
  # 同じprefixの行が1つでも正規文法を外れたら、他に正しい行があっても採用しない
  local file="$1" label="$2"
  local lines line count=0 value=""
  lines="$(grep -E "^${label}:" "$file" || true)"
  if [[ -n "$lines" ]]; then
    while IFS= read -r line; do
      if [[ "$line" =~ ^${label}:[[:space:]]*$ ]]; then
        gate_fail "$label empty"
      fi
      if [[ ! "$line" =~ ^${label}:\ ([^[:space:]]+)$ ]]; then
        gate_fail "$label malformed: ${line#${label}: }"
      fi
      value="${BASH_REMATCH[1]}"
      count=$((count + 1))
    done <<<"$lines"
  fi
  if [[ "$count" -eq 0 ]]; then
    gate_fail "missing $label"
  fi
  if [[ "$count" -gt 1 ]]; then
    gate_fail "duplicate $label"
  fi
  printf '%s' "$value"
}

gate_reject_symlink_components() {
  # linkdir -> /outside のような中間componentの逃げ道も、最終componentのみの
  # -Lや文字列上の".."判定では検出できないため、経路全componentを実体で歩いて確認する
  local worktree="$1" rel_path="$2"
  local cur="$worktree" part
  local old_ifs="$IFS"
  IFS='/'
  for part in $rel_path; do
    IFS="$old_ifs"
    cur="$cur/$part"
    if [[ -L "$cur" ]]; then
      gate_fail "AUTHORITY path is a symlink: $rel_path"
    fi
    IFS='/'
  done
  IFS="$old_ifs"
}

gate_ledger_row_state() {
  local ledger="$1" id="$2"
  awk -v id="$id" '
    BEGIN { in_section = 0; count = 0 }
    /^## 現在選択中の1件/ { in_section = 1; next }
    in_section && /^## / { in_section = 0 }
    in_section && /^\|/ {
      n = split($0, f, "|")
      if (n < 5) next
      gsub(/^[ \t]+|[ \t]+$/, "", f[3])
      if (f[3] ~ /^-+$/) next
      if (f[3] == id) {
        state = f[5]
        gsub(/^[ \t]+|[ \t]+$/, "", state)
        gsub(/`/, "", state)
        count++
        result = state
      }
    }
    END {
      if (count == 0) { print "ABSENT"; exit }
      if (count > 1) { print "AMBIGUOUS"; exit }
      print result
    }
  ' "$ledger"
}

gate_check_base() {
  local order_file="$1" worktree="$2"
  local base_ref base_sha ref_name resolved_sha worktree_head

  base_ref="$(gate_require_single_field "$order_file" "BASE_REF")"
  if [[ ! "$base_ref" =~ ^refs/heads/[A-Za-z0-9._/-]+$ ]]; then
    gate_fail "BASE_REF malformed: $base_ref"
  fi
  ref_name="${base_ref#refs/heads/}"
  if [[ -z "$ref_name" || "$ref_name" == */ || "$ref_name" == *"//"* || \
        "$ref_name" == *".."* || "$ref_name" == .* || "$ref_name" == */.* || \
        "$ref_name" == *".lock" ]]; then
    gate_fail "BASE_REF malformed: $base_ref"
  fi

  base_sha="$(gate_require_single_field "$order_file" "BASE_SHA")"
  if [[ ! "$base_sha" =~ ^[0-9a-f]{40}$ ]]; then
    gate_fail "BASE_SHA malformed: $base_sha"
  fi

  if ! resolved_sha="$(git -C "$worktree" rev-parse --verify --quiet "$base_ref" 2>/dev/null)"; then
    gate_fail "BASE_REF does not resolve: $base_ref"
  fi
  if [[ "$resolved_sha" != "$base_sha" ]]; then
    gate_fail "BASE_REF does not resolve to BASE_SHA"
  fi

  worktree_head="$(git -C "$worktree" rev-parse HEAD)"
  if [[ "$worktree_head" != "$base_sha" ]]; then
    gate_fail "worktree HEAD != BASE_SHA"
  fi
}

gate_check_grain_and_dependencies() {
  local order_file="$1" worktree="$2"
  local ledger="$worktree/docs/implementation-ledger.md"
  local grain grain_state dep_lines dep_id dep_state

  if [[ ! -f "$ledger" ]]; then
    gate_fail "docs/implementation-ledger.md missing in worktree"
  fi

  grain="$(gate_require_single_field "$order_file" "GRAIN")"
  grain_state="$(gate_ledger_row_state "$ledger" "$grain")"
  case "$grain_state" in
    ABSENT) gate_fail "$grain not found in selected-work ledger" ;;
    AMBIGUOUS) gate_fail "$grain has ambiguous selected-work ledger rows" ;;
    DO) ;;
    *) gate_fail "$grain is $grain_state; dispatch is forbidden" ;;
  esac

  dep_lines="$(grep -E '^DEPENDENCY:' "$order_file" || true)"
  if [[ -z "$dep_lines" ]]; then
    gate_fail "missing DEPENDENCY"
  fi
  while IFS= read -r dep_id; do
    if [[ "$dep_id" =~ ^DEPENDENCY:[[:space:]]*$ ]]; then
      gate_fail "DEPENDENCY empty"
    fi
    if [[ ! "$dep_id" =~ ^DEPENDENCY:\ ([^[:space:]]+)$ ]]; then
      gate_fail "DEPENDENCY malformed: ${dep_id#DEPENDENCY: }"
    fi
    dep_id="${BASH_REMATCH[1]}"
    dep_state="$(gate_ledger_row_state "$ledger" "$dep_id")"
    case "$dep_state" in
      ABSENT) gate_fail "dependency $dep_id not found in selected-work ledger" ;;
      AMBIGUOUS) gate_fail "dependency $dep_id has ambiguous selected-work ledger rows" ;;
      DONE) ;;
      *) gate_fail "dependency $dep_id is $dep_state; dispatch is forbidden" ;;
    esac
  done <<<"$dep_lines"
}

gate_check_authorities() {
  local order_file="$1" worktree="$2"
  local authority_lines line auth_path auth_hash auth_full actual_hash

  authority_lines="$(grep -E '^AUTHORITY:' "$order_file" || true)"
  if [[ -z "$authority_lines" ]]; then
    gate_fail "missing AUTHORITY"
  fi
  while IFS= read -r line; do
    if [[ ! "$line" =~ ^AUTHORITY:\ ([^[:space:]]+)\ SHA256:([0-9a-f]{64})$ ]]; then
      gate_fail "AUTHORITY malformed: ${line#AUTHORITY: }"
    fi
    auth_path="${BASH_REMATCH[1]}"
    auth_hash="${BASH_REMATCH[2]}"
    if [[ "$auth_path" == /* ]]; then
      gate_fail "AUTHORITY absolute path: $auth_path"
    fi
    if [[ "$auth_path" == *".."* ]]; then
      gate_fail "AUTHORITY path traversal: $auth_path"
    fi
    auth_full="$worktree/$auth_path"
    # symlinkはworktree外への逃げ道になり得るため、経路や存在確認より先に拒否する
    gate_reject_symlink_components "$worktree" "$auth_path"
    if [[ ! -f "$auth_full" ]]; then
      gate_fail "AUTHORITY file missing: $auth_path"
    fi
    actual_hash="$(shasum -a 256 "$auth_full" | awk '{print $1}')"
    if [[ "$actual_hash" != "$auth_hash" ]]; then
      gate_fail "authority hash mismatch: $auth_path"
    fi
  done <<<"$authority_lines"
}

gate_check_allowed_files() {
  local order_file="$1"
  local allowed_lines af

  allowed_lines="$(grep -E '^ALLOWED_FILE:' "$order_file" || true)"
  if [[ -z "$allowed_lines" ]]; then
    gate_fail "missing ALLOWED_FILE"
  fi
  GATE_ALLOWED_FILES=()
  while IFS= read -r af; do
    if [[ "$af" =~ ^ALLOWED_FILE:[[:space:]]*$ ]]; then
      gate_fail "ALLOWED_FILE empty"
    fi
    if [[ ! "$af" =~ ^ALLOWED_FILE:\ ([^[:space:]]+)$ ]]; then
      gate_fail "ALLOWED_FILE malformed: ${af#ALLOWED_FILE: }"
    fi
    af="${BASH_REMATCH[1]}"
    if [[ "$af" == /* ]]; then
      gate_fail "ALLOWED_FILE absolute path: $af"
    fi
    if [[ "$af" == *".."* ]]; then
      gate_fail "ALLOWED_FILE path traversal: $af"
    fi
    GATE_ALLOWED_FILES+=("$af")
  done <<<"$allowed_lines"
}

gate_check_clean_worktree() {
  local worktree="$1"
  if [[ -n "$(git -C "$worktree" status --porcelain)" ]]; then
    gate_fail "isolated worktree is not clean"
  fi
}

gate_check_react_labels() {
  local order_file="$1"
  local is_react=0 af label matches count line_no last_line=0

  if grep -qx 'REACT TASK: YES' "$order_file"; then
    is_react=1
  fi
  for af in "${GATE_ALLOWED_FILES[@]}"; do
    # docs/mocks-ui自身/直下の子孫だけを対象とし、docs/mocks-ui-legacy等の兄弟名を誤検知しない
    if [[ "$af" == "docs/mocks-ui" || "$af" == docs/mocks-ui/* || "$af" == *.jsx ]]; then
      is_react=1
    fi
  done
  if [[ "$is_react" -eq 0 ]]; then
    return
  fi

  for label in "${REACT_LABELS_ORDERED[@]}"; do
    matches="$(grep -nE "^${label}" "$order_file" | cut -d: -f1 || true)"
    count=0
    [[ -z "$matches" ]] || count="$(printf '%s\n' "$matches" | wc -l | tr -d ' ')"
    if [[ "$count" -eq 0 ]]; then
      gate_fail "React guard label missing or out of order: $label"
    fi
    if [[ "$count" -gt 1 ]]; then
      gate_fail "React guard label duplicated: $label"
    fi
    line_no="$matches"
    if (( line_no <= last_line )); then
      gate_fail "React guard label missing or out of order: $label"
    fi
    last_line="$line_no"
  done
}

run_dispatch_gate() {
  local order_file="$1" worktree="$2"
  gate_check_base "$order_file" "$worktree"
  gate_check_grain_and_dependencies "$order_file" "$worktree"
  gate_check_authorities "$order_file" "$worktree"
  gate_check_allowed_files "$order_file"
  gate_check_clean_worktree "$worktree"
  gate_check_react_labels "$order_file"
}

if [[ "$MODE" == "prepare" ]]; then
  supervisor_prompt=$(cat <<EOF
You are the read-only on-site supervisor for Motolii. Do not edit files, commit,
push, create a PR, spawn subagents, or delegate. Read AGENTS.md and every required
authority completely. Inspect the current worktree and existing diff. Turn the
user task into a binding implementation order for Claude Sonnet 5. Do not invent
unresolved product meaning or public contracts.

The order must contain objective, current code facts, authoritative spec/task IDs,
an exact closed file allowlist, non-goals, helpers to reuse, invariants, STOP
conditions, positive and negative tests, exact commands, and integration gates.
Forbid suppressions, expected-value or golden rewrites, fixture special-cases,
raw scanners that bypass typed boundaries, public raw mutation APIs, invented
serde defaults, duplicate planners/helpers, partial mutation, TODO stubs, and
adjacent-ticket expansion.

The order must also emit the fields the dispatch gate checks mechanically before
Sonnet is started: exactly one \`GRAIN: <id>\`, exactly one
\`BASE_REF: refs/heads/<full-branch-name>\`, exactly one full 40-hex
\`BASE_SHA: <sha>\` that BASE_REF resolves to and that equals the isolated
worktree HEAD, one or more \`DEPENDENCY: <id>\` lines, one or more
\`AUTHORITY: <worktree-relative-path> SHA256:<64-hex>\` lines, and one or more
\`ALLOWED_FILE: <worktree-relative-path-or-glob>\` lines. Before writing GRAIN or
DEPENDENCY, read the target worktree's docs/implementation-ledger.md
selected-work table and confirm GRAIN's own row states exactly \`DO\` and every
DEPENDENCY row states exactly \`DONE\`; never infer these states from prose or
from a different worktree. Before writing an AUTHORITY line, hash the file
inside the target worktree and copy that exact hash. If the order touches a
React surface (exact \`REACT TASK: YES\`, an ALLOWED_FILE under docs/mocks-ui, or
an ALLOWED_FILE ending in .jsx), also include, exactly once and in this order:
REACT AUTHORITY:, SOURCE ASSET:, PRESERVE:, REPLACE:, STATE OWNER:,
DIAGNOSTIC ROUTE:, NEGATIVE ORACLE:, STOP:. Merely mentioning React in prose
does not require these labels.

The last non-empty line must be exactly plain text ORDER: READY only if every
ledger, authority, and label fact above is mechanically true; otherwise end with
plain text ORDER: STOP. Do not bold it, quote it, or append text.

User task:
$task
EOF
  )
  echo "## 1. Claude Opus 4.8 supervisor order draft"
  if ! (cd "$WORKTREE" && run_supervisor "$tmp_dir/order.txt" "$supervisor_prompt" order); then
    [[ ! -f "$tmp_dir/order.txt" ]] || cat "$tmp_dir/order.txt"
    exit 1
  fi
  cat "$tmp_dir/order.txt"
  {
    cat "$tmp_dir/order.txt"
    echo "SUPERVISOR_BACKEND: claude-code"
    echo "SUPERVISOR_MODEL: $CLAUDE_SUPERVISOR_MODEL"
    echo "IMPLEMENTER_MODEL: $CLAUDE_IMPLEMENTER_MODEL"
    echo "TASK_SHA256: $task_hash"
  } >"$ORDER_FILE"
  if ! grep -qx 'ORDER: READY' "$tmp_dir/order.txt"; then
    echo "delegate-claude-supervised: OpusがREADYを出していません" >&2
    exit 3
  fi
  echo "delegate-claude-supervised: 発注書案を保存しました: $ORDER_FILE" >&2
  echo "delegate-claude-supervised: Codex審査後に CODEX PRECHECK: APPROVED を追記してください" >&2
  exit 0
fi

if [[ ! -f "$ORDER_FILE" ]]; then
  echo "delegate-claude-supervised: 承認対象の発注書がありません" >&2
  exit 2
fi
if ! grep -qx 'ORDER: READY' "$ORDER_FILE"; then
  echo "delegate-claude-supervised: ORDER: READY がありません" >&2
  exit 3
fi
if ! grep -qx "TASK_SHA256: $task_hash" "$ORDER_FILE"; then
  echo "delegate-claude-supervised: 発注書とtaskが一致しません" >&2
  exit 3
fi
if ! grep -qx 'CODEX PRECHECK: APPROVED' "$ORDER_FILE"; then
  echo "delegate-claude-supervised: Codex事前承認がありません" >&2
  exit 3
fi

run_dispatch_gate "$ORDER_FILE" "$WORKTREE"

cp "$ORDER_FILE" "$tmp_dir/order.txt"
head_before="$(git -C "$WORKTREE" rev-parse HEAD)"
implementation_prompt=$(cat <<EOF
You are the implementation contractor for Motolii. The binding order below was
written by Claude Opus 4.8 and approved by Codex. Read AGENTS.md and every source
named by the order. Implement only the allowed scope in the current isolated
worktree. Do not write outside this worktree, reinterpret requirements, broaden
file scope, invent defaults, weaken tests, commit, push, or create a PR. Do not
run this delegation script recursively. If exact implementation is blocked, stop
and report the conflicting authority and code evidence instead of improvising.

Original user task:
$task

Binding order:
$(cat "$tmp_dir/order.txt")
EOF
)

echo
echo "## 2. Claude Sonnet 5 implementation"
if ! (cd "$WORKTREE" && run_agent "$tmp_dir/implementation.txt" "$IMPLEMENTER_TIMEOUT_SECONDS" \
  env CLAUDE_DELEGATED=1 "$CLAUDE_AGENT_BIN" -p \
    --model "$CLAUDE_IMPLEMENTER_MODEL" \
    --permission-mode acceptEdits \
    --allowedTools Read,Glob,Grep,Edit,Write,Bash \
    --output-format text \
    "$implementation_prompt"); then
  [[ ! -f "$tmp_dir/implementation.txt" ]] || cat "$tmp_dir/implementation.txt"
  exit 1
fi
cat "$tmp_dir/implementation.txt"
if [[ "$(git -C "$WORKTREE" rev-parse HEAD)" != "$head_before" ]]; then
  echo "delegate-claude-supervised: 受注者がcommitを作成したため検収へ進みません" >&2
  exit 5
fi

inspection_prompt=$(cat <<EOF
You are the read-only acceptance supervisor for Motolii. Do not edit files,
commit, push, create a PR, spawn subagents, or delegate. Inspect the actual diff
and rerun required evidence now. Verify line-by-line against the binding order
and authorities. Green tests alone are insufficient. Look for scope drift,
contract-avoidance, weakened tests, missing negative cases, duplicate state or
logic, raw public APIs, non-atomic failure, unbounded work, and unfinished gates.

Classify P0/P1/P2 with file and line evidence. Any P0/P1, missing required test,
out-of-allowlist edit, or unverifiable command requires rejection. End with one
exact plain-text final line: VERDICT: ACCEPT or VERDICT: REJECT. Do not bold it,
quote it, or append text.

Original user task:
$task

Binding order:
$(cat "$tmp_dir/order.txt")
EOF
)

echo
echo "## 3. Claude Opus 4.8 read-only inspection"
if ! (cd "$WORKTREE" && run_supervisor "$tmp_dir/inspection.txt" "$inspection_prompt" verdict); then
  [[ ! -f "$tmp_dir/inspection.txt" ]] || cat "$tmp_dir/inspection.txt"
  exit 1
fi
cat "$tmp_dir/inspection.txt"
if ! grep -qx 'VERDICT: ACCEPT' "$tmp_dir/inspection.txt"; then
  echo "delegate-claude-supervised: Opus検収REJECT。差分は隔離したまま採用しません" >&2
  exit 4
fi
echo "delegate-claude-supervised: Opus検収ACCEPT。Codex最終レビュー待ちです"
