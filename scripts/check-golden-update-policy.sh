#!/usr/bin/env bash
# D1i-4 (S16執行): 意味論ゴールデン更新禁止 + 暫定ゴールデンのマーカー必須。
#
# 許可:
#   - 意味論(semantic)への「新規追加」(git status A) — 新variant+新ゴールデン
#   - 台帳へ初めて載せたPRでのマーカー追記等(base に未登録だった semantic)
#   - 暫定(provisional)の更新 — ファイルに MOTOLII_REGENERATE_WHEN がある場合のみ
#   - 分類外パスの任意変更
# 拒否:
#   - base 時点で既に semantic だったパスの変更/削除/リネーム(マーカー有無を問わない)
#   - provisional パスの更新で MOTOLII_REGENERATE_WHEN が無い
#   - 台帳から semantic 行を削る(base 比較時)
#   - semantic 集合が空 / 台帳パス不在 / クラスマーカー不一致
#
# Usage:
#   ./scripts/check-golden-update-policy.sh [base_ref]
#   ./scripts/check-golden-update-policy.sh --files-from -   # stdin: [STATUS\t]path
#   CLASSIFICATION_FILE=... ./scripts/check-golden-update-policy.sh --files-from -
#   GOLDEN_POLICY_SKIP_CONSISTENCY=1  # 負例フィクスチャ用(本番CIでは設定しない)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

CLASSIFICATION_FILE="${CLASSIFICATION_FILE:-crates/motolii-testkit/golden_policy/classification.tsv}"
CLASS_MARKER_SEMANTIC='MOTOLII_GOLDEN_CLASS: semantic'
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
        file_has "$path" "$CLASS_MARKER_SEMANTIC" \
          || die "semantic path missing '$CLASS_MARKER_SEMANTIC': $path"
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

was_semantic_on_base() {
  local want="$1"
  local p
  while IFS= read -r p; do
    [[ -z "$p" ]] && continue
    if [[ "$p" == "$want" ]]; then
      return 0
    fi
  done <<<"$BASE_SEMANTIC_ROWS"
  return 1
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
  else
    local base="${1:-origin/main}"
    if ! git rev-parse --verify "$base" >/dev/null 2>&1; then
      echo "golden-update-policy: base ref '$base' not found; treating as no changes" >&2
      return 0
    fi
    git diff --name-status "$base"...HEAD | while IFS=$'\t' read -r status a b; do
      [[ -z "${status:-}" ]] && continue
      case "$status" in
        R*)
          printf 'D\t%s\n' "$a"
          printf 'A\t%s\n' "$b"
          ;;
        *)
          printf '%s\t%s\n' "${status:0:1}" "$a"
          ;;
      esac
    done
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
  # 施行テストは「既に semantic 登録済み」前提で変更を拒否する。
  BASE_SEMANTIC_ROWS=""
  while IFS=$'\t' read -r class path; do
    [[ -z "${class:-}" ]] && continue
    if [[ "$class" == "semantic" ]]; then
      BASE_SEMANTIC_ROWS="${BASE_SEMANTIC_ROWS}${BASE_SEMANTIC_ROWS:+$'\n'}${path}"
    fi
  done <<<"$CLASS_ROWS"
else
  load_base_semantic "${1:-origin/main}"
fi

failures=0
fail_msgs=""
file_count=0

while IFS= read -r raw || [[ -n "${raw:-}" ]]; do
  [[ -z "$raw" ]] && continue
  local_line="$(normalize_change_line "$raw")"
  status="${local_line%%$'\t'*}"
  path="${local_line#*$'\t'}"
  file_count=$((file_count + 1))

  class="$(lookup_class "$path" || true)"
  [[ -z "$class" ]] && continue

  case "$class" in
    semantic)
      if [[ "$status" == "A" ]]; then
        continue
      fi
      # base 未登録の初回載せはマーカー追記等を許可。登録後の書き換えは例外なし拒否。
      if ! was_semantic_on_base "$path"; then
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
done < <(collect_changes "$@")

if [[ "$failures" -gt 0 ]]; then
  echo "D1i-4 golden-update-policy gate FAILED ($failures):" >&2
  echo "$fail_msgs" >&2
  echo >&2
  echo "  semantic: never rewrite; add a new variant + new golden file instead." >&2
  echo "  provisional: require ${REGENERATE_MARKER} in the file (#53)." >&2
  exit 1
fi

echo "D1i-4 golden-update-policy OK (files=$file_count mode_files_from=$MODE_FILES_FROM)"
