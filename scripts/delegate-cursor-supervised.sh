#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CURSOR_GROK_MODEL="${CURSOR_GROK_MODEL:-cursor-grok-4.5-high}"
CURSOR_AGENT_BIN="${CURSOR_AGENT_BIN:-cursor-agent}"
CODEX_AGENT_BIN="${CODEX_AGENT_BIN:-codex}"
CLAUDE_AGENT_BIN="${CLAUDE_AGENT_BIN:-claude}"
FABLE_MODEL="${CLAUDE_FABLE_MODEL:-claude-fable-5}"
SUPERVISOR_TIMEOUT_SECONDS="${CURSOR_SUPERVISED_TIMEOUT_SECONDS:-300}"
INSPECTION_TIMEOUT_SECONDS="${CURSOR_INSPECTION_TIMEOUT_SECONDS:-900}"
IMPLEMENTER_TIMEOUT_SECONDS="${CODEX_IMPLEMENTER_TIMEOUT_SECONDS:-900}"
FABLE_TIMEOUT_SECONDS="${CLAUDE_FABLE_TIMEOUT_SECONDS:-900}"
HEARTBEAT_SECONDS="${CURSOR_SUPERVISED_HEARTBEAT_SECONDS:-30}"
GRAIN_LEDGER="$ROOT_DIR/docs/reviews/2026-07-22-m3-comfortable-use-granulation.md"

usage() {
  echo "Usage: $0 prepare <isolated-worktree> <order-file> <task>"
  echo "       $0 execute <isolated-worktree> <approved-order-file> <task>"
  echo "       printf '%s\n' <task> | $0 prepare|execute <isolated-worktree> <order-file>"
  echo "       prepare routing: DELEGATION_TASK_CLASS=mechanical|standard|rapid|complex|cross-boundary"
}

select_routing() {
  local task_class="$1"
  case "$task_class" in
    mechanical)
      IMPLEMENTER_MODEL="gpt-5.4-mini-none"
      REVIEW_PROFILE="grok"
      ;;
    standard)
      IMPLEMENTER_MODEL="gpt-5.6-luna-none-fast"
      REVIEW_PROFILE="grok"
      ;;
    rapid)
      IMPLEMENTER_MODEL="gpt-5.6-terra"
      REVIEW_PROFILE="grok"
      ;;
    complex|cross-boundary)
      IMPLEMENTER_MODEL="gpt-5.6-sol-none-fast"
      REVIEW_PROFILE="grok+fable"
      ;;
    *)
      echo "delegate-cursor-supervised: TASK_CLASSはmechanical/standard/rapid/complex/cross-boundaryから選んでください" >&2
      return 1
      ;;
  esac
}

order_value() {
  local order_file="$1"
  local key="$2"
  awk -v prefix="$key: " '
    index($0, prefix) == 1 { count++; value = substr($0, length(prefix) + 1) }
    END { if (count == 1) print value }
  ' "$order_file"
}

if [[ -n "${CURSOR_AGENT:-}" ]]; then
  echo "delegate-cursor-supervised: Cursor子エージェントからの再帰実行は禁止です" >&2
  exit 2
fi
if [[ -n "${CODEX_DELEGATED:-}" ]]; then
  echo "delegate-cursor-supervised: Codex子エージェントからの再帰実行は禁止です" >&2
  exit 2
fi
if [[ -n "${CLAUDE_DELEGATED:-}" ]]; then
  echo "delegate-cursor-supervised: Claude子エージェントからの再帰実行は禁止です" >&2
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
if [[ ! "$SUPERVISOR_TIMEOUT_SECONDS" =~ ^[1-9][0-9]*$ ]]; then
  echo "delegate-cursor-supervised: CURSOR_SUPERVISED_TIMEOUT_SECONDSは正の整数で指定してください" >&2
  exit 2
fi
if [[ ! "$INSPECTION_TIMEOUT_SECONDS" =~ ^[1-9][0-9]*$ ]]; then
  echo "delegate-cursor-supervised: CURSOR_INSPECTION_TIMEOUT_SECONDSは正の整数で指定してください" >&2
  exit 2
fi
if [[ ! "$IMPLEMENTER_TIMEOUT_SECONDS" =~ ^[1-9][0-9]*$ ]]; then
  echo "delegate-cursor-supervised: CODEX_IMPLEMENTER_TIMEOUT_SECONDSは正の整数で指定してください" >&2
  exit 2
fi
if [[ ! "$FABLE_TIMEOUT_SECONDS" =~ ^[1-9][0-9]*$ ]]; then
  echo "delegate-cursor-supervised: CLAUDE_FABLE_TIMEOUT_SECONDSは正の整数で指定してください" >&2
  exit 2
fi
if [[ ! "$HEARTBEAT_SECONDS" =~ ^[1-9][0-9]*$ ]]; then
  echo "delegate-cursor-supervised: heartbeat間隔は正の整数で指定してください" >&2
  exit 2
fi
if ! command -v "$CURSOR_AGENT_BIN" >/dev/null 2>&1; then
  echo "delegate-cursor-supervised: Cursor Agent CLI '$CURSOR_AGENT_BIN' が見つかりません" >&2
  exit 127
fi
if ! command -v "$CODEX_AGENT_BIN" >/dev/null 2>&1; then
  echo "delegate-cursor-supervised: Codex CLI '$CODEX_AGENT_BIN' が見つかりません" >&2
  exit 127
fi

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/motolii-cursor-supervised.XXXXXX")"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

run_agent() {
  local output="$1"
  local timeout_seconds="$2"
  shift 2
  echo "delegate-cursor-supervised: 起動: $1 (timeout=${timeout_seconds}s)" >&2
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
    echo "delegate-cursor-supervised: ${timeout_seconds}秒でタイムアウトしました" >&2
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

order_gate_fail() {
  echo "ORDER-GATE NG: $*" >&2
  return 1
}

order_gate() {
  local order_file="$1"
  local worktree="$2"
  local base_sha grain actual_head authority_count=0
  local authority_path authority_hash actual_hash allowed_count=0 allowed_path

  base_sha="$(sed -n 's/^BASE_SHA: \([0-9a-f]\{40,64\}\)$/\1/p' "$order_file")"
  if [[ -z "$base_sha" || "$(grep -c '^BASE_SHA:' "$order_file")" -ne 1 ]]; then
    order_gate_fail "BASE_SHA must appear exactly once as a full commit id"
    return 1
  fi
  actual_head="$(git -C "$worktree" rev-parse HEAD)"
  if [[ "$actual_head" != "$base_sha" ]]; then
    order_gate_fail "worktree HEAD != BASE_SHA ($actual_head != $base_sha)"
    return 1
  fi

  grain="$(awk '/^GRAIN: / { count++; value = substr($0, 8) } END { if (count == 1) print value }' "$order_file")"
  if [[ -z "$grain" || "$(grep -c '^GRAIN:' "$order_file")" -ne 1 ]]; then
    order_gate_fail "GRAIN must appear exactly once"
    return 1
  fi
  if [[ ! -f "$GRAIN_LEDGER" ]]; then
    order_gate_fail "main grain ledger is missing: $GRAIN_LEDGER"
    return 1
  fi
  if ! awk -F'|' -v grain="$grain" '
      $2 ~ "^[[:space:]]*" grain "[[:space:]]*$" {
        found = 1
        state = $3
        gsub(/`|[[:space:]]/, "", state)
        if (state ~ /\/DO$/) ready = 1
      }
      END { exit !(found && ready) }
    ' "$GRAIN_LEDGER"; then
    order_gate_fail "$grain is not DO in main grain ledger; dispatch is forbidden"
    return 1
  fi

  while IFS=$'\t' read -r authority_path authority_hash; do
    [[ -n "$authority_path" ]] || continue
    authority_count=$((authority_count + 1))
    if [[ "$authority_path" = /* || "$authority_path" == ".." || "$authority_path" == ../* || "$authority_path" == */../* ]]; then
      order_gate_fail "authority path must stay inside worktree: $authority_path"
      return 1
    fi
    if [[ ! -f "$worktree/$authority_path" ]]; then
      order_gate_fail "authority is missing from worktree: $authority_path"
      return 1
    fi
    actual_hash="$(shasum -a 256 "$worktree/$authority_path" | awk '{print $1}')"
    if [[ "$actual_hash" != "$authority_hash" ]]; then
      order_gate_fail "authority hash mismatch: $authority_path"
      return 1
    fi
  done < <(awk '
    $1 == "AUTHORITY:" && NF == 3 && $3 ~ /^SHA256:/ {
      hash = substr($3, 8)
      if (length(hash) == 64 && hash !~ /[^0-9a-f]/) print $2 "\t" hash
    }
  ' "$order_file")
  if [[ "$authority_count" -eq 0 || "$(grep -c '^AUTHORITY:' "$order_file")" -ne "$authority_count" ]]; then
    order_gate_fail "AUTHORITY lines must use: AUTHORITY: <worktree-relative-path> SHA256:<64 hex>"
    return 1
  fi

  while IFS= read -r allowed_path; do
    [[ -n "$allowed_path" ]] || continue
    allowed_count=$((allowed_count + 1))
    if [[ "$allowed_path" = /* || "$allowed_path" == ".." || "$allowed_path" == ../* || "$allowed_path" == */../* ]]; then
      order_gate_fail "allowed path must stay inside worktree: $allowed_path"
      return 1
    fi
  done < <(awk '$1 == "ALLOWED_FILE:" && NF == 2 { print $2 }' "$order_file")
  if [[ "$allowed_count" -eq 0 || "$(grep -c '^ALLOWED_FILE:' "$order_file")" -ne "$allowed_count" ]]; then
    order_gate_fail "ALLOWED_FILE lines must use one worktree-relative glob per line"
    return 1
  fi

  if [[ -n "$(git -C "$worktree" status --porcelain=v1 --untracked-files=all)" ]]; then
    order_gate_fail "isolated worktree is not clean before implementation"
    return 1
  fi

  if grep -qx 'REACT TASK: YES' "$order_file" ||
     grep -Eq '^ALLOWED_FILE: (docs/mocks-ui(/|$)|.*\.jsx$)' "$order_file"; then
    local labels=(
      "REACT AUTHORITY"
      "SOURCE ASSET"
      "PRESERVE"
      "REPLACE"
      "STATE OWNER"
      "DIAGNOSTIC ROUTE"
      "NEGATIVE ORACLE"
      "STOP"
    )
    local previous=0 label line
    for label in "${labels[@]}"; do
      line="$(awk -v label="$label" '$0 == label ":" || $0 == "`" label "`:" { print NR; exit }' "$order_file")"
      if [[ -z "$line" || "$line" -le "$previous" ]]; then
        order_gate_fail "React guard label missing or out of order: $label"
        return 1
      fi
      previous="$line"
    done
  fi
}

path_is_allowed() {
  local order_file="$1"
  local changed_path="$2"
  local pattern
  while IFS= read -r pattern; do
    [[ -n "$pattern" ]] || continue
    if [[ "$changed_path" == $pattern ]]; then
      return 0
    fi
  done < <(awk '$1 == "ALLOWED_FILE:" && NF == 2 { print $2 }' "$order_file")
  return 1
}

scope_closure() {
  local order_file="$1"
  local worktree="$2"
  local changed_path
  while IFS= read -r changed_path; do
    [[ -n "$changed_path" ]] || continue
    if ! path_is_allowed "$order_file" "$changed_path"; then
      echo "SCOPE NG: 変更許可外path: $changed_path" >&2
      return 1
    fi
  done < <(
    {
      git -C "$worktree" diff --name-only
      git -C "$worktree" diff --cached --name-only
      git -C "$worktree" ls-files --others --exclude-standard
    } | LC_ALL=C sort -u
  )
}

persist_evidence() {
  local stage="$1"
  local output="$2"
  [[ -d "$EVIDENCE_DIR" ]] || mkdir -p "$EVIDENCE_DIR"
  [[ ! -f "$output" ]] || cp "$output" "$EVIDENCE_DIR/$stage.txt"
  [[ ! -f "$output.err" ]] || cp "$output.err" "$EVIDENCE_DIR/$stage.err"
  [[ ! -f "$output.timeout" ]] || cp "$output.timeout" "$EVIDENCE_DIR/$stage.timeout"
}

snapshot_worktree() {
  local prefix="$1"
  git -C "$WORKTREE" status --porcelain=v1 --untracked-files=all >"$EVIDENCE_DIR/$prefix.status"
  git -C "$WORKTREE" diff --binary >"$EVIDENCE_DIR/$prefix.diff"
}

run_supervisor() {
  local output="$1"
  local prompt="$2"
  local result_kind="$3"
  local cursor_mode_args=(--trust)
  local timeout_seconds="$SUPERVISOR_TIMEOUT_SECONDS"
  if [[ "$result_kind" == "order" ]]; then
    cursor_mode_args+=(--mode ask)
  else
    # headlessはread-only shellにも承認が要る。検収promptと事後差分審査で書込みを拒否する。
    cursor_mode_args+=(--force)
    timeout_seconds="$INSPECTION_TIMEOUT_SECONDS"
  fi
  prompt="Do not spawn subagents or delegate any part of this task. Complete the requested read-only work yourself in this run and return the required terminal marker.

$prompt"

  if ! run_agent "$output.cursor-grok" "$timeout_seconds" \
    "$CURSOR_AGENT_BIN" -p "${cursor_mode_args[@]}" \
    --output-format text --model "$CURSOR_GROK_MODEL" --workspace "$WORKTREE" "$prompt"; then
    [[ ! -f "$output.cursor-grok" ]] || cp "$output.cursor-grok" "$output"
    [[ ! -f "$output.cursor-grok.err" ]] || cp "$output.cursor-grok.err" "$output.err"
    [[ ! -f "$output.cursor-grok.timeout" ]] || cp "$output.cursor-grok.timeout" "$output.timeout"
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
  TASK_CLASS="${DELEGATION_TASK_CLASS:-standard}"
  if ! select_routing "$TASK_CLASS"; then
    exit 2
  fi
  supervisor_prompt=$(cat <<EOF
You are the on-site supervisor for Motolii. Work read-only. Read AGENTS.md and every required spec/review completely. Inspect the current worktree and existing diff. Translate the user intent into a binding implementation order for the Codex implementer selected by Codex; do not implement or change the selected routing.

Codex-selected routing:
TASK_CLASS: $TASK_CLASS
IMPLEMENTER_MODEL: $IMPLEMENTER_MODEL
REVIEW_PROFILE: $REVIEW_PROFILE

Do not repeat these routing labels in the draft; the dispatcher appends the
Codex-owned values after validating your terminal marker.

The order must contain: objective and user intent, current state and already-completed work, authoritative spec/task IDs, exact allowed files, explicit non-goals, existing helpers to reuse, invariants and atomicity, STOP conditions, required positive and negative tests, exact verification commands, and known integration gates. It must also contain exactly one GRAIN line, exactly one BASE_SHA line equal to the isolated worktree HEAD, one or more AUTHORITY lines in the exact form AUTHORITY: <worktree-relative-path> SHA256:<64 lowercase hex>, and one ALLOWED_FILE: <worktree-relative-glob> line for every allowed path or closed subtree. Add REACT TASK: YES only when the implementation changes a React source asset or docs/mocks-ui/JSX runtime path; prose that merely discusses React does not make an infrastructure or documentation grain a React task. A grain may be READY only when its row is DO in the main comfortable-use granulation ledger and every authority exists inside the target worktree at that hash. Do not permit allow/ignore/lint suppression, expected-value or golden rewrites, fixture special-cases, raw JSON/string scanners that bypass typed boundaries, public raw allocation/mutation APIs, serde defaults inventing durable meaning, duplicate planners/helpers, implicit migration, partial mutation, TODO stubs, or expansion into adjacent tasks.

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
    echo "SUPERVISOR_MODEL: $CURSOR_GROK_MODEL"
    echo "TASK_CLASS: $TASK_CLASS"
    echo "IMPLEMENTER_MODEL: $IMPLEMENTER_MODEL"
    echo "REVIEW_PROFILE: $REVIEW_PROFILE"
    echo "FABLE_MODEL: $FABLE_MODEL"
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
TASK_CLASS="$(order_value "$ORDER_FILE" TASK_CLASS)"
ORDER_IMPLEMENTER_MODEL="$(order_value "$ORDER_FILE" IMPLEMENTER_MODEL)"
ORDER_REVIEW_PROFILE="$(order_value "$ORDER_FILE" REVIEW_PROFILE)"
ORDER_FABLE_MODEL="$(order_value "$ORDER_FILE" FABLE_MODEL)"
if [[ -z "$TASK_CLASS" || -z "$ORDER_IMPLEMENTER_MODEL" || -z "$ORDER_REVIEW_PROFILE" || -z "$ORDER_FABLE_MODEL" ]]; then
  echo "delegate-cursor-supervised: 発注書のTASK_CLASS/model/review指定が欠落または重複しています" >&2
  exit 3
fi
if ! select_routing "$TASK_CLASS"; then
  exit 3
fi
if ! grep -qx "SUPERVISOR_MODEL: $CURSOR_GROK_MODEL" "$ORDER_FILE" ||
   [[ "$ORDER_IMPLEMENTER_MODEL" != "$IMPLEMENTER_MODEL" ]] ||
   [[ "$ORDER_REVIEW_PROFILE" != "$REVIEW_PROFILE" ]] ||
   [[ "$ORDER_FABLE_MODEL" != "$FABLE_MODEL" ]]; then
  echo "delegate-cursor-supervised: 発注書のモデル経路がTASK_CLASS対応表と一致しません" >&2
  exit 3
fi
if ! grep -qx 'CODEX PRECHECK: APPROVED' "$ORDER_FILE"; then
  echo "delegate-cursor-supervised: Codex事前承認がないため実装担当を起動しません" >&2
  exit 3
fi
if ! order_gate "$ORDER_FILE" "$WORKTREE"; then
  echo "delegate-cursor-supervised: 発注正本・粒状態・worktreeが一致しないため実装担当を起動しません" >&2
  exit 3
fi
if [[ "$REVIEW_PROFILE" == "grok+fable" ]] && ! command -v "$CLAUDE_AGENT_BIN" >/dev/null 2>&1; then
  echo "delegate-cursor-supervised: Fable必須クラスですがClaude Code '$CLAUDE_AGENT_BIN' が見つかりません" >&2
  exit 127
fi

EVIDENCE_DIR="${ORDER_FILE}.evidence"
mkdir -p "$EVIDENCE_DIR"
rm -f "$EVIDENCE_DIR"/{order,implementation,inspection,fable-inspection}.{txt,err,timeout} \
  "$EVIDENCE_DIR"/{before-implementation,after-implementation,before-inspection,after-inspection,before-fable-inspection,after-fable-inspection}.{status,diff}
cp "$ORDER_FILE" "$tmp_dir/order.txt"
cp "$ORDER_FILE" "$EVIDENCE_DIR/order.txt"
snapshot_worktree before-implementation

implementation_prompt=$(cat <<EOF
You are the implementation contractor for Motolii. The order from the Grok supervisor below is binding. Read AGENTS.md and all sources named by the order. Implement only the allowed scope. You may not reinterpret requirements, broaden file scope, invent defaults, or substitute a local workaround. If the order cannot be implemented exactly, do not improvise: stop and report the conflicting spec/file evidence. Do not commit, push, or create a PR.

Original user task:
$task

Binding Grok order:
$(cat "$tmp_dir/order.txt")
EOF
)

echo
echo "## 2. $IMPLEMENTER_MODEL implementation (Codex-prechecked order)"
head_before="$(git -C "$WORKTREE" rev-parse HEAD)"
if ! run_agent "$tmp_dir/implementation.txt" "$IMPLEMENTER_TIMEOUT_SECONDS" \
  env CODEX_DELEGATED=1 "$CODEX_AGENT_BIN" --ask-for-approval never exec \
  --ephemeral --color never --model "$IMPLEMENTER_MODEL" --sandbox danger-full-access \
  --cd "$WORKTREE" "$implementation_prompt"; then
  persist_evidence implementation "$tmp_dir/implementation.txt"
  snapshot_worktree after-implementation
  cat "$tmp_dir/implementation.txt"
  exit 1
fi
persist_evidence implementation "$tmp_dir/implementation.txt"
snapshot_worktree after-implementation
cat "$tmp_dir/implementation.txt"
if [[ "$(git -C "$WORKTREE" rev-parse HEAD)" != "$head_before" ]]; then
  echo "delegate-cursor-supervised: 実装担当がcommitを作成したため検収へ進みません" >&2
  exit 5
fi
if ! scope_closure "$ORDER_FILE" "$WORKTREE"; then
  echo "delegate-cursor-supervised: 変更許可閉集合に違反したため検収へ進みません" >&2
  exit 6
fi

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
snapshot_worktree before-inspection
if ! run_supervisor "$tmp_dir/inspection.txt" "$inspection_prompt" verdict; then
  persist_evidence inspection "$tmp_dir/inspection.txt"
  snapshot_worktree after-inspection
  [[ ! -f "$tmp_dir/inspection.txt" ]] || cat "$tmp_dir/inspection.txt"
  exit 1
fi
persist_evidence inspection "$tmp_dir/inspection.txt"
snapshot_worktree after-inspection
if ! cmp -s "$EVIDENCE_DIR/before-inspection.status" "$EVIDENCE_DIR/after-inspection.status" ||
   ! cmp -s "$EVIDENCE_DIR/before-inspection.diff" "$EVIDENCE_DIR/after-inspection.diff"; then
  echo "INSPECT NG: 検収中にworktreeが変更されたためverdictを無効化します" >&2
  exit 7
fi
cat "$tmp_dir/inspection.txt"
if ! grep -qx 'VERDICT: ACCEPT' "$tmp_dir/inspection.txt"; then
  echo "delegate-cursor-supervised: Grok検収REJECT。差分は隔離したまま採用しません" >&2
  exit 4
fi

if [[ "$REVIEW_PROFILE" == "grok+fable" ]]; then
  fable_prompt=$(cat <<EOF
You are the independent final counter-reviewer for Motolii. Work read-only in
the current worktree. Do not edit files, commit, push, create a PR, spawn
subagents, or delegate. Read AGENTS.md, the binding order, every named authority,
the actual diff, and the required test evidence. Review the whole change for
cross-file invariants, atomic failure behavior, contract drift, hidden public or
durable meaning, missed negative cases, and locally-correct changes that violate
the wider architecture. Do not accept merely because Grok accepted or tests are
green.

Classify findings P0/P1/P2 with file and line evidence. Any P0/P1, missing
required evidence, or unresolved contract conflict requires rejection. End with
exactly one line:
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
  echo "## 4. Claude Code Fable independent inspection"
  snapshot_worktree before-fable-inspection
  if ! (cd "$WORKTREE" && run_agent "$tmp_dir/fable-inspection.txt" "$FABLE_TIMEOUT_SECONDS" \
    env CLAUDE_DELEGATED=1 "$CLAUDE_AGENT_BIN" -p \
      --model "$FABLE_MODEL" \
      --permission-mode default \
      --allowedTools Read,Glob,Grep,Bash \
      --disallowedTools Edit,Write \
      --output-format text \
      "$fable_prompt"); then
    persist_evidence fable-inspection "$tmp_dir/fable-inspection.txt"
    snapshot_worktree after-fable-inspection
    [[ ! -f "$tmp_dir/fable-inspection.txt" ]] || cat "$tmp_dir/fable-inspection.txt"
    exit 1
  fi
  persist_evidence fable-inspection "$tmp_dir/fable-inspection.txt"
  snapshot_worktree after-fable-inspection
  if ! cmp -s "$EVIDENCE_DIR/before-fable-inspection.status" "$EVIDENCE_DIR/after-fable-inspection.status" ||
     ! cmp -s "$EVIDENCE_DIR/before-fable-inspection.diff" "$EVIDENCE_DIR/after-fable-inspection.diff"; then
    echo "INSPECT NG: Fable検収中にworktreeが変更されたためverdictを無効化します" >&2
    exit 7
  fi
  cat "$tmp_dir/fable-inspection.txt"
  if ! supervisor_result_is_valid "$tmp_dir/fable-inspection.txt" verdict; then
    echo "delegate-cursor-supervised: Fableの結果マーカーが欠落・曖昧・末尾外です" >&2
    exit 1
  fi
  if ! grep -qx 'VERDICT: ACCEPT' "$tmp_dir/fable-inspection.txt"; then
    echo "delegate-cursor-supervised: Fable検収REJECT。差分は隔離したまま採用しません" >&2
    exit 4
  fi
fi

echo "delegate-cursor-supervised: 必須検収ACCEPT。主担当の最終レビュー待ちです"
