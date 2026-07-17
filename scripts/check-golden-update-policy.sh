#!/usr/bin/env bash
# D1i-4 (S16執行): 意味論ゴールデン更新禁止 + 暫定ゴールデンのマーカー必須。
#
# 許可:
#   - 意味論(semantic)への「新規追加」(git status A) — 新variant+新ゴールデン
#   - 台帳への新規分類追加 / provisional→semantic 昇格(厳格化)
#   - 暫定(provisional)の更新 — ファイルに MOTOLII_REGENERATE_WHEN がある場合のみ
#   - 分類外パスの任意変更
# 拒否:
#   - HEADまたはbase台帳で semantic の既存ファイルの変更/削除(マーカー有無を問わない)
#   - HEADまたはbase台帳で provisional の更新で MOTOLII_REGENERATE_WHEN が無い
#   - 台帳から semantic / provisional 行を削る・分類外化する(base 比較時)
#   - semantic → provisional 降格
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
#   MIGRATION_FILE=...      # harness -> oracle 移行台帳
#   GOLDEN_POLICY_BASE_CLASSIFICATION=...  # --files-from 用に base 台帳を注入(回帰テスト)
#   GOLDEN_POLICY_BASE_LOOKUP_ONLY=1       # 注入台帳を effective class 参照のみに使い、削り検査をスキップ(回帰テスト専用)
#   GOLDEN_POLICY_SKIP_CONSISTENCY=1  # 負例フィクスチャ用(本番CIでは設定しない)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

CLASSIFICATION_FILE="${CLASSIFICATION_FILE:-crates/motolii-testkit/golden_policy/classification.tsv}"
MIGRATION_FILE="${MIGRATION_FILE:-crates/motolii-testkit/golden_policy/migrations.tsv}"
CLASS_MARKER_PROVISIONAL='MOTOLII_GOLDEN_CLASS: provisional'
REGENERATE_MARKER='MOTOLII_REGENERATE_WHEN:'
BASE_CLASS_ROWS=""
MIGRATION_ROWS=""
BASE_MIGRATION_ROWS=""

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

load_migrations() {
  local file="$1"
  local input
  if [[ "$file" == "-" ]]; then
    input="/dev/stdin"
  else
    [[ -f "$file" ]] || die "migration file missing: $file"
    input="$file"
  fi
  while IFS= read -r line || [[ -n "${line:-}" ]]; do
    [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]] && continue
    local source target rest
    IFS=$'\t' read -r source target rest <<<"$line"
    if [[ -z "$source" || -z "$target" || -n "${rest:-}" || "$source" == "$target" ]]; then
      die "malformed migration line (need distinct source<TAB>target): $line"
    fi
    printf '%s\t%s\n' "$source" "$target"
  done <"$input"
}

lookup_in_rows() {
  local rows="$1"
  local want="$2"
  local class path
  while IFS=$'\t' read -r class path; do
    [[ -z "${class:-}" ]] && continue
    if [[ "$path" == "$want" ]]; then
      echo "$class"
      return 0
    fi
  done <<<"$rows"
  return 1
}

lookup_class() {
  lookup_in_rows "$CLASS_ROWS" "$1"
}

lookup_base_class() {
  lookup_in_rows "$BASE_CLASS_ROWS" "$1"
}

lookup_migration_target() {
  local want="$1"
  local source target
  while IFS=$'\t' read -r source target; do
    [[ -z "${source:-}" ]] && continue
    if [[ "$source" == "$want" ]]; then
      echo "$target"
      return 0
    fi
  done <<<"$MIGRATION_ROWS"
  return 1
}

semantic_harness_migrated() {
  local source="$1"
  local target
  target="$(lookup_migration_target "$source" || true)"
  [[ -n "$target" ]] || return 1
  [[ -f "$source" ]] || return 1
  [[ -f "$target" ]] || return 1
  [[ "$(lookup_class "$target" || true)" == "semantic" ]]
}

enforce_base_migration_lock() {
  local base_rows="$1"
  BASE_MIGRATION_ROWS="$base_rows"
  local source target now
  while IFS=$'\t' read -r source target; do
    [[ -z "${source:-}" ]] && continue
    now="$(lookup_migration_target "$source" || true)"
    [[ "$now" == "$target" ]] \
      || die "semantic migration ledger entry removed or retargeted: $source -> $target"
  done <<<"$base_rows"
}

load_base_migrations() {
  local base="$1"
  BASE_MIGRATION_ROWS=""
  if ! git cat-file -e "$base:$MIGRATION_FILE" 2>/dev/null; then
    return 0
  fi
  local base_rows
  base_rows="$(git show "$base:$MIGRATION_FILE" | load_migrations -)"
  enforce_base_migration_lock "$base_rows"
}

# HEAD優先。未分類なら base を参照(台帳から外して変更する迂回を塞ぐ)。
effective_class() {
  local path="$1"
  local class
  class="$(lookup_class "$path" || true)"
  if [[ -n "$class" ]]; then
    echo "$class"
    return 0
  fi
  if [[ "$(lookup_base_class "$path" || true)" == "semantic" ]] \
    && semantic_harness_migrated "$path"; then
    return 0
  fi
  lookup_base_class "$path" || true
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

  local source target
  while IFS=$'\t' read -r source target; do
    [[ -z "${source:-}" ]] && continue
    [[ -f "$source" ]] || die "semantic migration source missing: $source"
    [[ -f "$target" ]] || die "semantic migration target missing: $target"
    [[ "$(lookup_class "$target" || true)" == "semantic" ]] \
      || die "semantic migration target is not classified semantic: $source -> $target"
  done <<<"$MIGRATION_ROWS"
}

# base 台帳の semantic/provisional を保護。削り・分類外化・semantic降格を拒否。
# provisional→semantic 昇格のみ許可。
enforce_base_classification_lock() {
  local base_rows="$1"
  BASE_CLASS_ROWS="$base_rows"
  local class path now
  while IFS=$'\t' read -r class path; do
    [[ -z "${class:-}" ]] && continue
    now="$(lookup_class "$path" || true)"
    case "$class" in
      semantic)
        if [[ -z "$now" ]]; then
          if semantic_harness_migrated "$path"; then
            continue
          fi
          die "semantic entry removed without a valid harness-to-oracle migration: $path"
        fi
        if [[ "$now" != "semantic" ]]; then
          die "semantic entry demoted to '$now' (forbidden): $path"
        fi
        ;;
      provisional)
        if [[ -z "$now" ]]; then
          die "provisional entry removed from classification (declassification forbidden): $path"
        fi
        if [[ "$now" != "provisional" && "$now" != "semantic" ]]; then
          die "provisional entry changed to '$now' (forbidden): $path"
        fi
        ;;
    esac
  done <<<"$base_rows"
}

load_base_classification() {
  local base="$1"
  BASE_CLASS_ROWS=""
  if ! git cat-file -e "$base:$CLASSIFICATION_FILE" 2>/dev/null; then
    echo "golden-update-policy: no classification on $base; first registration allowed"
    return 0
  fi
  local base_rows
  base_rows="$(git show "$base:$CLASSIFICATION_FILE" | load_classification -)"
  enforce_base_classification_lock "$base_rows"
}

# 回帰テスト用: ファイルから base 台帳を注入(--files-from と併用)。
load_injected_base_classification() {
  local file="$1"
  [[ -f "$file" ]] || die "GOLDEN_POLICY_BASE_CLASSIFICATION missing: $file"
  local base_rows
  base_rows="$(load_classification "$file")"
  if [[ "${GOLDEN_POLICY_BASE_LOOKUP_ONLY:-}" == "1" ]]; then
    # effective class のみ検証(削り検査は別テスト)。本番CIでは設定しない。
    BASE_CLASS_ROWS="$base_rows"
    return 0
  fi
  enforce_base_classification_lock "$base_rows"
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
MIGRATION_ROWS="$(load_migrations "$MIGRATION_FILE")"
if [[ "${GOLDEN_POLICY_SKIP_CONSISTENCY:-}" != "1" ]]; then
  validate_classification_consistency
fi

MODE_FILES_FROM=0
BASE_CLASS_ROWS=""
if [[ -n "${GOLDEN_POLICY_BASE_CLASSIFICATION:-}" ]]; then
  load_injected_base_classification "$GOLDEN_POLICY_BASE_CLASSIFICATION"
fi

if [[ "${1:-}" == "--files-from" ]]; then
  MODE_FILES_FROM=1
  # 注入が無ければ base 空(ブートストラップ相当)。注入時は上で lock 済み。
else
  # ライブCI: git base 台帳で lock。注入があれば二重適用はしない(注入優先はテスト専用)。
  if [[ -z "${GOLDEN_POLICY_BASE_CLASSIFICATION:-}" ]]; then
    load_base_migrations "${1:-origin/main}"
    load_base_classification "${1:-origin/main}"
  fi
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

  # 台帳パス自体の diff 行はスキップ(削りは enforce_base_classification_lock が審判)。
  if [[ "$path" == "$CLASSIFICATION_FILE" ]]; then
    continue
  fi

  class="$(effective_class "$path")"
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
  echo "  do not drop provisional/semantic rows from ${CLASSIFICATION_FILE} to bypass." >&2
  echo "  classify existing goldens via ${CLASSIFICATION_FILE} only (no body edit)." >&2
  exit 1
fi

echo "D1i-4 golden-update-policy OK (files=$file_count mode_files_from=$MODE_FILES_FROM)"
