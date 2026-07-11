#!/usr/bin/env bash
# M2E-2: 保護テスト資産を含むPRは保護領域のみに触れなければならない。
#
# 許可される形:
#   - 保護領域のみ変更(=テスト更新専用PR)
#   - 保護領域に触れない任意の変更(=通常の実装/TDD PR)
# 拒否:
#   - 保護領域と、それ以外(src/tests/Cargo.toml/CI等)の同時変更
#
# Usage:
#   ./scripts/check-protected-diff.sh [base_ref]          # git diff --name-only
#   ./scripts/check-protected-diff.sh --files-from -      # stdinのパス一覧で判定(負例テスト用)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

is_protected() {
  case "$1" in
    crates/motolii-testkit/golden/*) return 0 ;;
    crates/motolii-testkit/src/cpu_reference/*) return 0 ;;
    crates/motolii-testkit/src/tol/*) return 0 ;;
    *) return 1 ;;
  esac
}

collect_files() {
  if [[ "${1:-}" == "--files-from" ]]; then
    local src="${2:--}"
    if [[ "$src" == "-" ]]; then
      cat
    else
      cat "$src"
    fi
  else
    local base="${1:-origin/main}"
    if git rev-parse --verify "$base" >/dev/null 2>&1; then
      git diff --name-only "$base"...HEAD
    else
      echo "check-protected-diff: base ref '$base' not found; treating as no changes" >&2
      true
    fi
  fi
}

hit_protected=0
hit_other=0
protected_hits=""
other_hits=""
file_count=0

while IFS= read -r f || [[ -n "${f:-}" ]]; do
  [[ -z "$f" ]] && continue
  file_count=$((file_count + 1))
  if is_protected "$f"; then
    hit_protected=1
    protected_hits="${protected_hits}${protected_hits:+$'\n'}    ${f}"
  else
    hit_other=1
    other_hits="${other_hits}${other_hits:+$'\n'}    ${f}"
  fi
done < <(collect_files "$@")

if [[ "$hit_protected" -eq 1 && "$hit_other" -eq 1 ]]; then
  echo "M2E-2 protected-diff gate FAILED:" >&2
  echo "  protected assets changed together with non-protected paths." >&2
  echo "  Test-update PRs must touch ONLY protected paths:" >&2
  echo "    golden/**, src/cpu_reference/**, src/tol/**" >&2
  echo "  Split implementation/TDD changes into a separate PR that does not" >&2
  echo "  touch protected assets." >&2
  echo >&2
  echo "  protected hits:" >&2
  echo "$protected_hits" >&2
  echo "  other hits:" >&2
  echo "$other_hits" >&2
  exit 1
fi

echo "M2E-2 protected-diff gate OK (protected=$hit_protected other=$hit_other files=$file_count)"
