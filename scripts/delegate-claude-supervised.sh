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
      --permission-mode plan \
      --allowedTools Read,Glob,Grep,Bash \
      --output-format text \
      "$prompt"; then
    return 1
  fi
  if ! result_is_valid "$output" "$result_kind"; then
    echo "delegate-claude-supervised: Opusの結果markerが欠落・曖昧・末尾外です" >&2
    return 1
  fi
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

End with exactly ORDER: READY if fully specified, otherwise ORDER: STOP.

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
exact final line: VERDICT: ACCEPT or VERDICT: REJECT.

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
