#!/bin/bash
# git 変更系コマンドに明示の作業場所を強制する柵(2026-08-21 利用者指示)。
# 背景: セッションが lane worktree へ cd したまま main 向けの git を打つ事故が
# 4回再発(規律でなく構造で塞ぐ)。
# 規則: add/commit/merge/push/reset/rebase/checkout/switch/restore/stash/
#       clean/rm/tag/revert/cherry-pick を含む Bash コマンドは、
#       (a) 同一コマンド内に絶対パスへの cd(`cd /…`)がある、または
#       (b) `git -C /…` で場所を明示している、場合のみ通す。
cmd=$(jq -r '.tool_input.command // empty')
[ -z "$cmd" ] && exit 0
if echo "$cmd" | grep -qE '\bgit[[:space:]]+(add|commit|merge|push|reset|rebase|checkout|switch|restore|stash|clean|rm|tag|revert|cherry-pick)\b'; then
  if echo "$cmd" | grep -qE '(^|[;&|][[:space:]]*)cd[[:space:]]+(/|"/|~\/)' || echo "$cmd" | grep -qE '\bgit[[:space:]]+-C[[:space:]]+(/|"/|~\/)'; then
    exit 0
  fi
  cat << 'JSON'
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"git変更系コマンドに明示の作業場所がありません。`cd /絶対パス && git …` か `git -C /絶対パス …` の形で場所を明示してください(worktree誤爆防止の柵)"}}
JSON
  exit 0
fi
exit 0
