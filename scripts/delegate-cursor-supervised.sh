#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GROK_BUILD_MODEL="${GROK_BUILD_MODEL:-grok-4.5}"
CURSOR_GROK_MODEL="${CURSOR_GROK_MODEL:-cursor-grok-4.5-high-fast}"
CURSOR_AGENT_BIN="${CURSOR_AGENT_BIN:-cursor-agent}"
COMPOSER_MODEL="${CURSOR_COMPOSER_MODEL:-composer-2.5}"
TIMEOUT_SECONDS="${GROK_SUPERVISED_TIMEOUT_SECONDS:-${CURSOR_SUPERVISED_TIMEOUT_SECONDS:-300}}"
GROK_BUILD_FALLBACK_TO_CURSOR="${GROK_BUILD_FALLBACK_TO_CURSOR:-1}"
HEARTBEAT_SECONDS="${CURSOR_SUPERVISED_HEARTBEAT_SECONDS:-30}"

usage() {
  echo "Usage: $0 prepare <isolated-worktree> <order-file> <task>"
  echo "       $0 execute <isolated-worktree> <approved-order-file> <task>"
  echo "       printf '%s\n' <task> | $0 prepare|execute <isolated-worktree> <order-file>"
}

if [[ -n "${CURSOR_AGENT:-}" ]]; then
  echo "delegate-cursor-supervised: Cursor子エージェントからの再帰実行は禁止です" >&2
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
if [[ "$WORKTREE" == "$ROOT_DIR" ]]; then
  echo "delegate-cursor-supervised: 主作業ツリーへの実装発注は禁止です。隔離worktreeを指定してください" >&2
  exit 2
fi
if ! git -C "$WORKTREE" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "delegate-cursor-supervised: git worktreeではありません: $WORKTREE" >&2
  exit 2
fi
if [[ ! "$TIMEOUT_SECONDS" =~ ^[1-9][0-9]*$ ]]; then
  echo "delegate-cursor-supervised: timeoutは正の整数で指定してください" >&2
  exit 2
fi
if [[ ! "$HEARTBEAT_SECONDS" =~ ^[1-9][0-9]*$ ]]; then
  echo "delegate-cursor-supervised: heartbeat間隔は正の整数で指定してください" >&2
  exit 2
fi
if [[ "$GROK_BUILD_FALLBACK_TO_CURSOR" != "0" && "$GROK_BUILD_FALLBACK_TO_CURSOR" != "1" ]]; then
  echo "delegate-cursor-supervised: GROK_BUILD_FALLBACK_TO_CURSOR は0か1で指定してください" >&2
  exit 2
fi
if [[ "$MODE" == "execute" ]] && ! command -v "$CURSOR_AGENT_BIN" >/dev/null 2>&1; then
  echo "delegate-cursor-supervised: Composer用のCursor Agent CLI '$CURSOR_AGENT_BIN' が見つかりません" >&2
  exit 127
fi

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/motolii-cursor-supervised.XXXXXX")"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

run_agent() {
  local output="$1"
  shift
  echo "delegate-cursor-supervised: 起動: $1 (timeout=${TIMEOUT_SECONDS}s)" >&2
  "$@" >"$output" 2>"$output.err" &
  local pid=$!
  (
    local elapsed=0
    local interval
    while (( elapsed < TIMEOUT_SECONDS )); do
      interval="$HEARTBEAT_SECONDS"
      if (( elapsed + interval > TIMEOUT_SECONDS )); then
        interval=$((TIMEOUT_SECONDS - elapsed))
      fi
      sleep "$interval"
      elapsed=$((elapsed + interval))
      if ! kill -0 "$pid" 2>/dev/null; then
        exit 0
      fi
      if (( elapsed < TIMEOUT_SECONDS )); then
        echo "delegate-cursor-supervised: 実行継続中 (${elapsed}s)" >&2
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
    echo "delegate-cursor-supervised: ${TIMEOUT_SECONDS}秒でタイムアウトしました" >&2
    status=124
  fi
  if [[ -s "$output.err" ]]; then
    cat "$output.err" >&2
  fi
  return "$status"
}

supervisor_result_is_valid() {
  local output="$1"
  local result_kind="$2"

  awk -v result_kind="$result_kind" '
    NF { last_nonempty = $0 }
    $0 == "ORDER: READY" || $0 == "ORDER: STOP" {
      order_markers++
    }
    $0 == "VERDICT: ACCEPT" || $0 == "VERDICT: REJECT" {
      verdict_markers++
    }
    END {
      if (result_kind == "order") {
        valid_terminal = last_nonempty == "ORDER: READY" || last_nonempty == "ORDER: STOP"
        exit !(order_markers == 1 && verdict_markers == 0 && valid_terminal)
      }
      if (result_kind == "verdict") {
        valid_terminal = last_nonempty == "VERDICT: ACCEPT" || last_nonempty == "VERDICT: REJECT"
        exit !(verdict_markers == 1 && order_markers == 0 && valid_terminal)
      }
      exit 1
    }
  ' "$output"
}

run_supervisor() {
  local output="$1"
  local prompt="$2"
  local result_kind="$3"
  local cursor_mode="ask"
  if [[ "$result_kind" == "verdict" ]]; then
    cursor_mode="plan"
  fi
  prompt="Do not spawn subagents or delegate any part of this task. Complete the requested read-only work yourself in this run and return the required terminal marker.

$prompt"

  if command -v grok >/dev/null 2>&1; then
    if run_agent "$output.grok-build" grok -p "$prompt" \
      --cwd "$WORKTREE" \
      --model "$GROK_BUILD_MODEL" \
      --output-format plain \
      --permission-mode plan \
      --no-subagents \
      --no-memory \
      --disable-web-search; then
      if grep -Eqi '402 Payment Required|spending-limit|run out of credits|responses API error' \
        "$output.grok-build" "$output.grok-build.err"; then
        echo "delegate-cursor-supervised: Grok Buildが利用上限を返しました" >&2
      elif supervisor_result_is_valid "$output.grok-build" "$result_kind"; then
        cp "$output.grok-build" "$output"
        SUPERVISOR_BACKEND_USED="grok-build"
        return 0
      else
        cp "$output.grok-build" "$output"
        echo "delegate-cursor-supervised: Grok Buildの結果マーカーが欠落・曖昧・末尾外です" >&2
      fi
    else
      echo "delegate-cursor-supervised: Grok Buildの実行に失敗しました" >&2
    fi
  else
    echo "delegate-cursor-supervised: Grok Build CLI 'grok' が見つかりません" >&2
  fi

  if [[ "$GROK_BUILD_FALLBACK_TO_CURSOR" != "1" ]]; then
    return 1
  fi
  if ! command -v "$CURSOR_AGENT_BIN" >/dev/null 2>&1; then
    echo "delegate-cursor-supervised: フォールバック用のCursor Agent CLI '$CURSOR_AGENT_BIN' が見つかりません" >&2
    return 127
  fi

  echo "delegate-cursor-supervised: Cursor版Grokへフォールバックします" >&2
  if ! run_agent "$output.cursor-grok" "$CURSOR_AGENT_BIN" -p --trust --mode "$cursor_mode" \
    --output-format text --model "$CURSOR_GROK_MODEL" --workspace "$WORKTREE" "$prompt"; then
    return 1
  fi
  if ! supervisor_result_is_valid "$output.cursor-grok" "$result_kind"; then
    cp "$output.cursor-grok" "$output"
    echo "delegate-cursor-supervised: Cursor版Grokの結果マーカーが欠落・曖昧・末尾外です" >&2
    return 1
  fi
  cp "$output.cursor-grok" "$output"
  SUPERVISOR_BACKEND_USED="cursor-grok"
}

if [[ "$MODE" == "prepare" ]]; then
  supervisor_prompt=$(cat <<EOF
You are the on-site supervisor for Motolii. Work read-only. Read AGENTS.md and every required spec/review completely. Inspect the current worktree and existing diff. Translate the user intent into a binding implementation order for Composer 2.5; do not implement.

The order must contain: objective and user intent, current state and already-completed work, authoritative spec/task IDs, exact allowed files, explicit non-goals, existing helpers to reuse, invariants and atomicity, STOP conditions, required positive and negative tests, exact verification commands, and known integration gates. Do not permit allow/ignore/lint suppression, expected-value or golden rewrites, fixture special-cases, raw JSON/string scanners that bypass typed boundaries, public raw allocation/mutation APIs, serde defaults inventing durable meaning, duplicate planners/helpers, implicit migration, partial mutation, TODO stubs, or expansion into adjacent tasks.

If the task is ready and fully specified, end with exactly: ORDER: READY
If any unresolved decision or dependency blocks implementation, end with exactly: ORDER: STOP

User task:
$task
EOF
  )

  echo "## 1. Grok supervisor order draft"
  if ! run_supervisor "$tmp_dir/order.txt" "$supervisor_prompt" order; then
    [[ ! -f "$tmp_dir/order.txt" ]] || cat "$tmp_dir/order.txt"
    exit 1
  fi
  cat "$tmp_dir/order.txt"
  {
    cat "$tmp_dir/order.txt"
    echo "SUPERVISOR_BACKEND: $SUPERVISOR_BACKEND_USED"
    echo "TASK_SHA256: $task_hash"
  } >"$ORDER_FILE"
  if ! grep -qx 'ORDER: READY' "$tmp_dir/order.txt"; then
    echo "delegate-cursor-supervised: Grok監督がREADYを出していません。Codex審査へ進めず差し戻してください" >&2
    exit 3
  fi
  echo "delegate-cursor-supervised: 発注書案を保存しました: $ORDER_FILE" >&2
  echo "delegate-cursor-supervised: Codex事前審査後にのみ CODEX PRECHECK: APPROVED を追記してください" >&2
  exit 0
fi

if [[ ! -f "$ORDER_FILE" ]]; then
  echo "delegate-cursor-supervised: 承認対象の発注書がありません: $ORDER_FILE" >&2
  exit 2
fi
if ! grep -qx 'ORDER: READY' "$ORDER_FILE"; then
  echo "delegate-cursor-supervised: ORDER: READY のない発注書は実行できません" >&2
  exit 3
fi
if ! grep -qx "TASK_SHA256: $task_hash" "$ORDER_FILE"; then
  echo "delegate-cursor-supervised: 発注書とtaskが一致しません" >&2
  exit 3
fi
if ! grep -qx 'CODEX PRECHECK: APPROVED' "$ORDER_FILE"; then
  echo "delegate-cursor-supervised: Codex事前承認がないためComposerを起動しません" >&2
  exit 3
fi

cp "$ORDER_FILE" "$tmp_dir/order.txt"

composer_prompt=$(cat <<EOF
You are the implementation contractor for Motolii. The order from the Grok supervisor below is binding. Read AGENTS.md and all sources named by the order. Implement only the allowed scope. You may not reinterpret requirements, broaden file scope, invent defaults, or substitute a local workaround. If the order cannot be implemented exactly, do not improvise: stop and report the conflicting spec/file evidence. Do not commit, push, or create a PR.

Original user task:
$task

Binding Grok order:
$(cat "$tmp_dir/order.txt")
EOF
)

echo
echo "## 2. Composer 2.5 implementation (Codex-prechecked order)"
if ! run_agent "$tmp_dir/implementation.txt" "$CURSOR_AGENT_BIN" -p --force --trust --output-format text \
  --model "$COMPOSER_MODEL" --workspace "$WORKTREE" "$composer_prompt"; then
  cat "$tmp_dir/implementation.txt"
  exit 1
fi
cat "$tmp_dir/implementation.txt"

inspection_prompt=$(cat <<EOF
You are the same on-site supervisor for Motolii. Work read-only. Do not create a plan, spawn subagents, or wait for another agent. Use read-only shell/tools now to inspect the actual git diff and rerun the required test evidence in the worktree. Verify it line-by-line against your binding order below and the authoritative specs. A green test suite is not sufficient. Look specifically for contract-avoidance hacks, scope/file drift, weakened tests, missing negative cases, duplicated logic, public raw APIs, implicit migration, non-atomic failure paths, unbounded work or allocation, wire incompatibility, and unfinished integration gates.

Classify findings P0/P1/P2 with file and line evidence. P0 or P1, missing required tests, edits outside the allowlist, or unverifiable command output requires rejection. End with exactly one line:
VERDICT: ACCEPT
or
VERDICT: REJECT

Original user task:
$task

Binding order:
$(cat "$tmp_dir/order.txt")
EOF
)

echo
echo "## 3. Grok supervisor inspection"
if ! run_supervisor "$tmp_dir/inspection.txt" "$inspection_prompt" verdict; then
  [[ ! -f "$tmp_dir/inspection.txt" ]] || cat "$tmp_dir/inspection.txt"
  exit 1
fi
cat "$tmp_dir/inspection.txt"
if ! grep -qx 'VERDICT: ACCEPT' "$tmp_dir/inspection.txt"; then
  echo "delegate-cursor-supervised: Grok検収REJECT。差分は隔離したまま採用しません" >&2
  exit 4
fi

echo "delegate-cursor-supervised: Grok検収ACCEPT。主担当の最終レビュー待ちです"
