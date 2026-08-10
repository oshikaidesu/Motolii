#!/usr/bin/env bash
# 迷子成果の観測器: 成果が滞留しがちな5層を1コマンドで報告する。
# 根拠: 2026-08-10回収監査(docs/reviews/2026-08-10-out-of-repo-recovery-and-docs-drift-audit.md)。
#   skia Timeline実装6コミットがローカル専用ブランチに、RN probe 660行がリポ外workdirに滞留し、
#   後続エージェントが「存在しない」前提で再発明しかけた再発防止。
# これはマージ条件ではない(段差撤廃決定 2026-08-10)。事後観測としていつでも実行する。
# 見つけた成果は当日中にmainへ回収するか、棄却をtombstone化する。
# 使い方: scripts/check-stray-work.sh   (リポジトリルートから。要: git fetch済みのorigin/main)
set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
FOUND=0

section() { printf '\n== %s\n' "$1"; }

git fetch origin --prune --quiet 2>/dev/null || true

section "1. ローカル専用ブランチ(origin/mainに無い一意パッチを持つ)"
for b in $(git for-each-ref refs/heads --format='%(refname:short)'); do
  n=$(git rev-list --count "origin/main..$b" 2>/dev/null || echo 0)
  [ "$n" = "0" ] && continue
  plus=$(git cherry origin/main "$b" 2>/dev/null | grep -c '^+' || true)
  [ "$plus" = "0" ] && continue
  d=$(git log -1 --format=%ad --date=short "$b")
  code=$(git diff --name-only "origin/main...$b" 2>/dev/null \
    | grep -cE '^(crates|ui|plugins|spikes|scripts)/' || true)
  echo "STRAY: $b (一意${plus}コミット, 最終${d}, code変更${code}ファイル)"
  FOUND=1
done

section "2. リモート未回収ブランチ(origin/*で一意パッチを持つ)"
for b in $(git branch -r | grep -v HEAD | grep -v 'origin/main$' | sed 's/^ *//'); do
  plus=$(git cherry origin/main "$b" 2>/dev/null | grep -c '^+' || true)
  [ "$plus" = "0" ] && continue
  echo "STRAY: $b (一意${plus}コミット, 最終$(git log -1 --format=%ad --date=short "$b"))"
  FOUND=1
done

section "3. mainに無いHEADを持つworktree"
git worktree list --porcelain | awk '/^worktree /{w=$2} /^HEAD /{print w, $2}' \
| while read -r w h; do
  if ! git merge-base --is-ancestor "$h" origin/main 2>/dev/null; then
    echo "STRAY: $w (${h:0:8})"
  fi
done | grep . && FOUND=1

section "4. リポ外workdirの直近成果 (~/Documents/Codex, 7日以内のmtime)"
if [ -d "$HOME/Documents/Codex" ]; then
  find "$HOME/Documents/Codex" -type f -mtime -7 \
    -not -path '*/node_modules/*' -not -path '*/target/*' -not -path '*/build/*' \
    -not -path '*/Pods/*' -not -name '.DS_Store' -not -name '*.log' \
    -not -name 'meta.json' -not -name 'lifecycle.jsonl' 2>/dev/null | head -20 \
  | sed 's/^/STRAY: /' | grep . && FOUND=1
fi

section "5. docsのリポ外絶対パス参照(移管・棚卸し注記なし)"
grep -rlE 'Documents/Codex|/Users/member_ottoto' docs --include='*.md' 2>/dev/null \
| while read -r f; do
  grep -qE '移管|回収監査|棚卸し' "$f" || echo "STRAY: $f"
done | grep . && FOUND=1

echo
if [ "$FOUND" = "0" ]; then
  echo "OK: 滞留成果なし(5層とも清潔)"
else
  echo "FOUND: 上記STRAYを当日中にmainへ回収するか、棄却をtombstone化すること"
fi
exit 0
