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
# --- stash そのものを塞ぐ(2026-08-23) ---------------------------------
# 背景: `refs/stash` は worktree 間で共有されるため、並列レーン中の stash は
# 他レーンの WIP を巻き込む(裁定200)。KNOWN.md に禁止と書いてあったが、
# 同日 8本中 2本(SP-3/SP-7)が「分割前と比べる」目的で使った — 文章の禁止は
# 効かなかった。**場所を明示しても通さない**(cd /abs && git stash も塞ぐ)。
# 読み取り専用の `git stash list` だけは通す(被害確認に使う)。
# 破壊的な下位動詞だけを名指しで塞ぐ。`stash list`(読み取り)は通す。
# 「どこかに stash list が在れば通す」形にすると
# `git stash push && git stash list` が素通りするので、
# **stash の直後に来る語**で判定する(2026-08-23 自分の commit で踏んだ)。
if echo "$cmd" | grep -qE '\bgit[[:space:]]+(-C[[:space:]]+\S+[[:space:]]+)?stash([[:space:]]+(push|pop|apply|drop|save|clear|create|store|branch|-|$)|[[:space:]]*[;&|]|[[:space:]]*$)'; then
  cat << 'JSON'
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"git stash は禁止(裁定200)。refs/stash は worktree 間で共有されるので、並列レーンの未コミット作業を巻き込みます。分割前後を比べたいなら非破壊の手段を使ってください: `git show HEAD:<path>` / `git diff HEAD -- <path>` / `git worktree add`。被害確認の `git stash list` だけは通ります"}}
JSON
  exit 0
fi

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
