#!/usr/bin/env bash
# D1i-4 (S16執行): 意味論ゴールデン更新禁止 + 暫定ゴールデンのマーカー必須。
#
# 許可:
#   - 意味論(semantic)への「新規追加」(git status A) — 新variant+新ゴールデン
#   - 台帳(classification.tsv)の追加/更新(分類のみ — 既存semantic本体を触らない)
#   - 暫定(provisional)の更新 — ファイルに MOTOLII_REGENERATE_WHEN がある場合のみ
#   - 分類外パスの任意変更
# 拒否:
#   - HEAD台帳で semantic の既存ファイルの変更/削除(マーカー有無を問わない)
#   - provisional パスの更新で MOTOLII_REGENERATE_WHEN が無い
#   - 台帳から semantic 行を削る(base 比較時)
#   - semantic 集合が空 / 台帳パス不在
#   - merge-base / git diff 失敗(shallow clone等) — fail-closed
#
# 正本は classification.tsv。semantic ファイル内の MOTOLII_GOLDEN_CLASS は任意
# (既存ゴールデン本体を変えずに分類できる)。
#
# Usage:
#   ./scripts/check-golden-update-policy.sh [base_ref]
#   ./scripts/check-golden-update-policy.sh --files-from -
#   CLASSIFICATION_FILE=... ./scripts/check-golden-update-policy.sh --files-from -
#   GOLDEN_POLICY_SKIP_CONSISTENCY=1  # 負例フィクスチャ用(本番CIでは設定しない)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

CLASSIFICATION_FILE="${CLASSIFICATION_FILE:-crates/motolii-testkit/golden_policy/classification.tsv}"
CLASS_MARKER_PROVISIONAL='MOTOLII_GOLDEN_CLASS: provisional'
REGENERATE_MARKER='MOTOLII_REGENERATE_WHEN:'
BASE_SEMANTIC_ROWS=""

die() {
  echo "golden-update-policy FAILED: $*" >&2
  exit 1
}

file_has() {
  local path="$1"
  local needle="$2"
  [[ -f "$path" ]] || return 1
  grep -qF "$needle" "$path"
}

# stdout: "class<TAB>path" (comments/blank stripped)
load_classification() {
  local file="$1"
  local input
  if [[ "$file" == "-" ]]; then
    input="/dev/stdin"
  else
    [[ -f "$file" ]] || die "classification file missing: $file"
    input="$file"
  fi
  while IFS= read -r line || [[ -n "${line:-}" ]]; do
    [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]] && continue
    local class path
    class="${line%%$'\t'*}"
    path="${line#*$'\t'}"
    if [[ "$class" != "semantic" && "$class" != "provisional" ]]; then
      die "unknown class '$class' in $file (line: $line)"
    fi
    if [[ -z "$path" || "$path" == "$line" ]]; then
      die "malformed classification line (need class<TAB>path): $line"
    fi
    printf '%s\t%s\n' "$class" "$path"
  done <"$input"
}

lookup_class() {
  local want="$1"
  local class path
  while IFS=$'\t' read -r class path; do
    if [[ "$path" == "$want" ]]; then
      echo "$class"
      return 0
    fi
  done <<<"$CLASS_ROWS"
  return 1
}

validate_classification_consistency() {
  local class path
  local semantic_count=0
  while IFS=$'\t' read -r class path; do
    [[ -z "${class:-}" ]] && continue
    [[ -f "$path" ]] || die "classified path does not exist: $path ($class)"
    case "$class" in
      semantic)
        semantic_count=$((semantic_count + 1))
        # 正本は台帳。ファイル内マーカーは任意(既存本体を触らず分類できる)。
        ;;
      provisional)
        file_has "$path" "$CLASS_MARKER_PROVISIONAL" \
          || die "provisional path missing '$CLASS_MARKER_PROVISIONAL': $path"
        file_has "$path" "$REGENERATE_MARKER" \
          || die "provisional path missing '$REGENERATE_MARKER': $path"
        ;;
    esac
  done <<<"$CLASS_ROWS"
  if [[ "$semantic_count" -lt 1 ]]; then
    die "semantic classification is empty (refuse vacuous forbid-CI)"
  fi
}

# base 台帳の semantic 集合を読み、削り/降格を拒否。
load_base_semantic() {
  local base="$1"
  BASE_SEMANTIC_ROWS=""
  if ! git cat-file -e "$base:$CLASSIFICATION_FILE" 2>/dev/null; then
    echo "golden-update-policy: no classification on $base; first registration allowed"
    return 0
  fi
  local base_rows
  base_rows="$(git show "$base:$CLASSIFICATION_FILE" | load_classification -)"
  local class path
  while IFS=$'\t' read -r class path; do
    [[ -z "${class:-}" ]] && continue
    [[ "$class" == "semantic" ]] || continue
    BASE_SEMANTIC_ROWS="${BASE_SEMANTIC_ROWS}${BASE_SEMANTIC_ROWS:+$'\n'}${path}"
    local now
    now="$(lookup_class "$path" || true)"
    if [[ -z "$now" ]]; then
      die "semantic entry removed from classification (demotion forbidden): $path"
    fi
    if [[ "$now" != "semantic" ]]; then
      die "semantic entry demoted to '$now' (forbidden): $path"
    fi
  done <<<"$base_rows"
}

collect_changes() {
  if [[ "${1:-}" == "--files-from" ]]; then
    local src="${2:--}"
    if [[ "$src" == "-" ]]; then
      cat
    else
      cat "$src"
    fi
    return 0
  fi

  local base="${1:-origin/main}"
  if ! git rev-parse --verify "$base" >/dev/null 2>&1; then
    die "base ref '$base' not found (fetch the PR base; refuse vacuous OK)"
  fi
  local merge_base
  # process substitution 外で失敗を捕捉する(fail-closed)
  if ! merge_base="$(git merge-base "$base" HEAD 2>/dev/null)"; then
    die "no merge base with $base (shallow clone?). deepen fetch: git fetch --unshallow or fetch-depth: 0"
  fi
  if [[ -z "$merge_base" ]]; then
    die "empty merge base with $base"
  fi
  # `git diff A...B` は merge-base を使うが、失敗時は非ゼロで落ちる
  if ! git diff --name-status "$merge_base" HEAD; then
    die "git diff --name-status $merge_base HEAD failed"
  fi
}

normalize_change_line() {
  local line="$1"
  if [[ "$line" == *$'\t'* ]]; then
    local st="${line%%$'\t'*}"
    local p="${line#*$'\t'}"
    case "$st" in
      A|M|D) printf '%s\t%s\n' "$st" "$p" ;;
      *) die "bad status '$st' in --files-from line: $line" ;;
    esac
  else
    printf 'M\t%s\n' "$line"
  fi
}

CLASS_ROWS="$(load_classification "$CLASSIFICATION_FILE")"
if [[ "${GOLDEN_POLICY_SKIP_CONSISTENCY:-}" != "1" ]]; then
  validate_classification_consistency
fi

MODE_FILES_FROM=0
if [[ "${1:-}" == "--files-from" ]]; then
  MODE_FILES_FROM=1
  BASE_SEMANTIC_ROWS=""
else
  load_base_semantic "${1:-origin/main}"
fi

failures=0
fail_msgs=""
file_count=0

# process substitution だと git 失敗が set -e をすり抜けるため、一旦ファイルへ落としてから読む。
CHANGES_FILE="$(mktemp)"
trap 'rm -f "$CHANGES_FILE"' EXIT
collect_changes "$@" >"$CHANGES_FILE" || die "collect_changes failed (fail-closed)"

while IFS= read -r raw || [[ -n "${raw:-}" ]]; do
  [[ -z "$raw" ]] && continue
  local_line="$(normalize_change_line "$raw")"
  status="${local_line%%$'\t'*}"
  path="${local_line#*$'\t'}"
  file_count=$((file_count + 1))

  # 台帳自体の変更は常に許可(分類のみのブートストラップ経路)
  if [[ "$path" == "$CLASSIFICATION_FILE" ]]; then
    continue
  fi

  class="$(lookup_class "$path" || true)"
  [[ -z "$class" ]] && continue

  case "$class" in
    semantic)
      # 新規 semantic ファイルの追加(A)のみ許可。既存の M/D は例外なし拒否。
      if [[ "$status" == "A" ]]; then
        continue
      fi
      failures=$((failures + 1))
      fail_msgs="${fail_msgs}${fail_msgs:+$'\n'}  - semantic golden modified/deleted (forbidden, no exceptions): ${status} ${path}"
      ;;
    provisional)
      if [[ "$status" == "D" ]]; then
        continue
      fi
      if [[ "$status" == "A" || "$status" == "M" ]]; then
        if ! file_has "$path" "$REGENERATE_MARKER"; then
          failures=$((failures + 1))
          fail_msgs="${fail_msgs}${fail_msgs:+$'\n'}  - provisional golden lacks ${REGENERATE_MARKER}: ${status} ${path}"
        fi
      fi
      ;;
  esac
done <"$CHANGES_FILE"

if [[ "$failures" -gt 0 ]]; then
  echo "D1i-4 golden-update-policy gate FAILED ($failures):" >&2
  echo "$fail_msgs" >&2
  echo >&2
  echo "  semantic: never rewrite; add a new variant + new golden file instead." >&2
  echo "  provisional: require ${REGENERATE_MARKER} in the file (#53)." >&2
  echo "  classify existing goldens via ${CLASSIFICATION_FILE} only (no body edit)." >&2
  exit 1
fi

echo "D1i-4 golden-update-policy OK (files=$file_count mode_files_from=$MODE_FILES_FROM)"
