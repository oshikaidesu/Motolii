#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSER_MODEL="${CURSOR_COMPOSER_MODEL:-composer-2.5}"
GROK_MODEL="${CURSOR_GROK_MODEL:-cursor-grok-4.5-high-fast}"
TIMEOUT_SECONDS="${CURSOR_DELEGATE_TIMEOUT_SECONDS:-180}"

usage() {
  echo "Usage: $0 <task>"
  echo "       printf '%s\n' <task> | $0"
}

if [[ -n "${CURSOR_AGENT:-}" ]]; then
  echo "delegate-cursor-review: Cursor子エージェントからの再帰実行は禁止です" >&2
  exit 2
fi

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

if ! command -v agent >/dev/null 2>&1; then
  echo "delegate-cursor-review: Cursor Agent CLI 'agent' が見つかりません" >&2
  exit 127
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

if [[ ! "$TIMEOUT_SECONDS" =~ ^[1-9][0-9]*$ ]]; then
  echo "delegate-cursor-review: CURSOR_DELEGATE_TIMEOUT_SECONDS は正の整数で指定してください" >&2
  exit 2
fi

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/motolii-cursor-review.XXXXXX")"
cleanup() {
  local pid
  for pid in "${composer_pid:-}" "${grok_pid:-}" "${composer_watchdog_pid:-}" "${grok_watchdog_pid:-}"; do
    if [[ -n "$pid" ]]; then
      kill -TERM "$pid" 2>/dev/null || true
    fi
  done
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

common_prompt=$(cat <<'EOF'
You are an external read-only consultant for the Motolii repository. Read and obey AGENTS.md and the documents it requires for the task. Do not create, edit, rename, or delete files. Do not run commands that change repository or external state. Do not delegate to another agent. Give concrete evidence with file paths and task/spec IDs. Clearly label unresolved specification decisions that should stop implementation.

Treat a green test suite as insufficient evidence. Explicitly look for contract-avoidance hacks: new allow/ignore exceptions or lint suppression outside a spec-authorized closed scope, expected-value or golden rewrites, test deletion, special-cases for fixtures, raw JSON/string scanners duplicating typed boundaries, public raw allocation/mutation APIs, serde defaults inventing missing durable meaning, duplicated planners/helpers, implicit migration, partial mutation before validation, and comments that defer a required invariant. Classify findings as P0/P1/P2. P0 or P1 means DO NOT INTEGRATE. If a shortcut seems necessary, identify the existing canonical boundary or declare STOP; do not recommend a local workaround.
EOF
)

composer_prompt="${common_prompt}

Role: implementation planner. Analyze the task, locate the smallest valid implementation boundary, identify reusable code, dependencies, acceptance tests, and an ordered implementation plan.

Task:
${task}"

grok_prompt="${common_prompt}

Role: adversarial design reviewer. Independently challenge the task and likely implementation, looking for contract violations, hidden state, permanence/UI boundary contamination, concurrency hazards, missing tests, and reasons implementation must wait.

Task:
${task}"

cd "$ROOT_DIR"

agent -p --trust --mode ask --output-format text \
  --model "$COMPOSER_MODEL" "$composer_prompt" \
  >"$tmp_dir/composer.txt" 2>"$tmp_dir/composer.err" &
composer_pid=$!

agent -p --trust --mode ask --output-format text \
  --model "$GROK_MODEL" "$grok_prompt" \
  >"$tmp_dir/grok.txt" 2>"$tmp_dir/grok.err" &
grok_pid=$!

(
  sleep "$TIMEOUT_SECONDS"
  if kill -0 "$composer_pid" 2>/dev/null; then
    touch "$tmp_dir/composer.timeout"
    kill -TERM "$composer_pid" 2>/dev/null
  fi
) &
composer_watchdog_pid=$!

(
  sleep "$TIMEOUT_SECONDS"
  if kill -0 "$grok_pid" 2>/dev/null; then
    touch "$tmp_dir/grok.timeout"
    kill -TERM "$grok_pid" 2>/dev/null
  fi
) &
grok_watchdog_pid=$!

set +e
wait "$composer_pid"
composer_status=$?
kill "$composer_watchdog_pid" 2>/dev/null
wait "$composer_watchdog_pid" 2>/dev/null
set -e

echo "## Composer 2.5 (${COMPOSER_MODEL})"
cat "$tmp_dir/composer.txt"
if [[ -s "$tmp_dir/composer.err" ]]; then
  echo "[stderr]" >&2
  cat "$tmp_dir/composer.err" >&2
fi
if [[ -f "$tmp_dir/composer.timeout" ]]; then
  echo "delegate-cursor-review: Composer 2.5は${TIMEOUT_SECONDS}秒でタイムアウトしました" >&2
  composer_status=124
fi

set +e
wait "$grok_pid"
grok_status=$?
kill "$grok_watchdog_pid" 2>/dev/null
wait "$grok_watchdog_pid" 2>/dev/null
set -e

echo
echo "## Grok 4.5 Fast (${GROK_MODEL})"
cat "$tmp_dir/grok.txt"
if [[ -s "$tmp_dir/grok.err" ]]; then
  echo "[stderr]" >&2
  cat "$tmp_dir/grok.err" >&2
fi
if [[ -f "$tmp_dir/grok.timeout" ]]; then
  echo "delegate-cursor-review: Grok 4.5 Fastは${TIMEOUT_SECONDS}秒でタイムアウトしました" >&2
  grok_status=124
fi

if [[ "$composer_status" -ne 0 || "$grok_status" -ne 0 ]]; then
  echo "delegate-cursor-review: composer=$composer_status grok=$grok_status" >&2
  exit 1
fi
