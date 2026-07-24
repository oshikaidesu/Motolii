#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
PRIMARY_WORKTREE_RAW="$(git -C "$ROOT_DIR" worktree list --porcelain | awk '/^worktree / && !found { print substr($0, 10); found=1 }')"
PRIMARY_WORKTREE="$(cd "$PRIMARY_WORKTREE_RAW" && pwd -P)"
CURSOR_AGENT_BIN="${CURSOR_AGENT_BIN:-cursor-agent}"
CODEX_AGENT_BIN="${CODEX_AGENT_BIN:-codex}"
CLAUDE_AGENT_BIN="${CLAUDE_AGENT_BIN:-claude}"
CURSOR_GROK_MODEL="cursor-grok-4.5-high"
FABLE_MODEL="${CLAUDE_FABLE_MODEL:-claude-fable-5}"
TERRA_MODEL="gpt-5.6-terra"
SUPERVISOR_TIMEOUT_SECONDS="${CURSOR_SUPERVISED_TIMEOUT_SECONDS:-600}"
TERRA_TIMEOUT_SECONDS="${CODEX_TERRA_TIMEOUT_SECONDS:-1800}"
INSPECTION_TIMEOUT_SECONDS="${CURSOR_INSPECTION_TIMEOUT_SECONDS:-300}"
HEARTBEAT_SECONDS="${CURSOR_SUPERVISED_HEARTBEAT_SECONDS:-30}"
TERMINATION_GRACE_SECONDS="${CURSOR_TERMINATION_GRACE_SECONDS:-2}"

usage() {
  echo "Usage: $0 prepare <isolated-worktree> <order-file> <task>"
  echo "       $0 execute <isolated-worktree> <approved-order-file> <task>"
  echo "       $0 inspect <isolated-worktree> <approved-order-file> <task>"
  echo "       printf '%s\n' <task> | $0 prepare|execute <isolated-worktree> <order-file>"
  echo "       prepare routing: DELEGATION_TASK_CLASS=mechanical|standard|rapid|complex|cross-boundary"
}

select_routing() {
  local task_class="$1"
  case "$task_class" in
    mechanical)
      IMPLEMENTER_MODEL="gpt-5.3-codex-spark"
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

if [[ -n "${CURSOR_AGENT:-}" || -n "${CODEX_DELEGATED:-}" ]]; then
  echo "delegate-cursor-supervised: 外部子エージェントからの再帰実行は禁止です" >&2
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
# macOSの/varと/private/varのようなsymlink alias差でtoplevel比較が誤爆しないよう、
# 物理path(pwd -P)へ正規化してから比較する
WORKTREE="$(cd "$2" && pwd -P)"
ORDER_FILE="$3"
# 呼び出し側の相対pathのままだと、この後のexistence確認・hash・evidence-root
# ($ORDER_FILE.evidence)・modelへ渡すargvが、後続処理でcdが起きた場合に
# 別の場所を指してしまう。argument解析直後、他の処理より前に絶対pathへ固定する
case "$ORDER_FILE" in
  /*) : ;;
  *) ORDER_FILE="$(cd "$(dirname "$ORDER_FILE")" && pwd)/$(basename "$ORDER_FILE")" ;;
esac
shift 3
if [[ "$MODE" != "prepare" && "$MODE" != "execute" && "$MODE" != "inspect" ]]; then
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
if [[ "$WORKTREE" == "$PRIMARY_WORKTREE" ]]; then
  echo "delegate-cursor-supervised: 主作業ツリーへの実装発注は禁止です" >&2
  exit 2
fi
if ! git -C "$WORKTREE" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "delegate-cursor-supervised: git worktreeではありません: $WORKTREE" >&2
  exit 2
fi
# サブディレクトリはisolated worktreeそのものではない。scope判定はworktree
# toplevel基準で行うため、渡されたWORKTREEがtoplevel自身であることを物理path
# (pwd -P)同士の完全一致で要求する
worktree_toplevel_raw="$(git -C "$WORKTREE" rev-parse --show-toplevel 2>/dev/null)" || {
  echo "delegate-cursor-supervised: worktree toplevelを解決できません: $WORKTREE" >&2
  exit 2
}
worktree_toplevel="$(cd "$worktree_toplevel_raw" && pwd -P)"
if [[ "$WORKTREE" != "$worktree_toplevel" ]]; then
  echo "delegate-cursor-supervised: WORKTREEはworktree toplevelではありません: $WORKTREE" >&2
  exit 2
fi
for value in "$SUPERVISOR_TIMEOUT_SECONDS" "$TERRA_TIMEOUT_SECONDS" "$INSPECTION_TIMEOUT_SECONDS" "$HEARTBEAT_SECONDS" "$TERMINATION_GRACE_SECONDS"; do
  if [[ ! "$value" =~ ^[1-9][0-9]*$ ]]; then
    echo "delegate-cursor-supervised: timeout/heartbeatは正の整数で指定してください" >&2
    exit 2
  fi
done
if ! command -v "$CURSOR_AGENT_BIN" >/dev/null 2>&1; then
  echo "delegate-cursor-supervised: Cursor Agent CLI '$CURSOR_AGENT_BIN' が見つかりません" >&2
  exit 127
fi
if ! command -v "$CODEX_AGENT_BIN" >/dev/null 2>&1; then
  echo "delegate-cursor-supervised: Codex CLI '$CODEX_AGENT_BIN' が見つかりません" >&2
  exit 127
fi

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/motolii-cursor-supervised.XXXXXX")"
# checkpointはmodel出力ではなくparent(この script)だけが書く。EVIDENCE_ROOT_FOR_TRAPが
# 設定された後、CHECKPOINT_SETTLED=1でexitしない限り、EXIT trapがcheckpointを
# 無効化する。これにより「Terra後のどの経路で抜けても、明示的にpublish/invalidateを
# 済ませていない限りcheckpointは無効」という不変条件を経路網羅なしで保証する
EVIDENCE_ROOT_FOR_TRAP=""
CHECKPOINT_SETTLED=0
cleanup() {
  # $?をここで即時退避しないと、後続コマンドがrunnerの本来の終了statusを上書きしてしまう。
  # 後始末の成功を呼び出し側へ返さず、最後に元のstatusを明示してfail-closedを維持する
  local status=$?
  if [[ -n "${CURRENT_ATTEMPT_DIR:-}" ]]; then
    printf 'EXIT_STATUS: %s\n' "$status" >>"$CURRENT_ATTEMPT_DIR/stage-result.txt" 2>/dev/null || true
  fi
  if [[ -n "$EVIDENCE_ROOT_FOR_TRAP" && "$CHECKPOINT_SETTLED" != "1" ]]; then
    rm -f "$EVIDENCE_ROOT_FOR_TRAP/checkpoint.txt" 2>/dev/null || true
  fi
  rm -rf "$tmp_dir"
  trap - EXIT
  exit "$status"
}
trap cleanup EXIT

mark_checkpoint_at_risk() {
  EVIDENCE_ROOT_FOR_TRAP="$1"
  CHECKPOINT_SETTLED=0
}

run_agent() {
  local output="$1"
  local timeout_seconds="$2"
  shift 2
  echo "delegate-cursor-supervised: 起動: $1 (timeout=${timeout_seconds}s)" >&2
  # set -mでbackground jobを独立process group(pgid==pid)に置く。外部モデル側の
  # bash孫プロセスがtimeout/returnを生き延びてsnapshot後に書き換えることを防ぐため、
  # leader pidだけでなくgroup全体をkill/reapする
  set -m
  # 親のstdinがpipe/FIFOのまま開いていても、prompt引数を受け取ったCLIが追加入力を
  # 待ち続けないよう明示的に閉じる。外部agentは対話入力を必要としない。
  "$@" </dev/null >"$output" 2>"$output.err" &
  local pid=$!
  set +m
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
    kill -TERM -- "-$pid" 2>/dev/null || kill -TERM "$pid" 2>/dev/null || true
    # TERMを無視するmodel/孫processでもwaitを永久停止させない。短い猶予後は
    # watchdog自身がprocess groupへKILLを送り、main側のwaitを必ず解放する。
    sleep "$TERMINATION_GRACE_SECONDS"
    kill -KILL -- "-$pid" 2>/dev/null || kill -KILL "$pid" 2>/dev/null || true
  ) &
  local watchdog=$!
  set +e
  wait "$pid"
  local status=$?
  set -e
  kill "$watchdog" 2>/dev/null || true
  wait "$watchdog" 2>/dev/null || true
  # leader終了後も生き残り得るgroup内の子孫を確実に回収する
  kill -KILL -- "-$pid" 2>/dev/null || true
  if [[ -f "$output.timeout" ]]; then
    echo "delegate-cursor-supervised: ${timeout_seconds}秒でタイムアウトしました" >&2
    status=124
  fi
  if [[ -s "$output.err" ]]; then
    cat "$output.err" >&2
  fi
  return "$status"
}

result_is_valid() {
  local output="$1"
  local result_kind="$2"
  awk -v result_kind="$result_kind" '
    NF { last_nonempty = $0 }
    $0 == "ORDER: READY" || $0 == "ORDER: STOP" { order_markers++ }
    $0 == "VERDICT: ACCEPT" || $0 == "VERDICT: REJECT" { verdict_markers++ }
    END {
      if (result_kind == "order") {
        exit !(order_markers == 1 && verdict_markers == 0 &&
          (last_nonempty == "ORDER: READY" || last_nonempty == "ORDER: STOP"))
      }
      if (result_kind == "verdict") {
        exit !(verdict_markers == 1 && order_markers == 0 &&
          (last_nonempty == "VERDICT: ACCEPT" || last_nonempty == "VERDICT: REJECT"))
      }
      exit 1
    }
  ' "$output"
}

run_supervisor() {
  local output="$1"
  local prompt="$2"
  local result_kind="$3"
  local timeout_seconds="${4:-$SUPERVISOR_TIMEOUT_SECONDS}"
  # plan modeで編集を禁止しつつ、--forceでread-only shellの非対話実行だけを
  # 可能にする。fingerprint/scope検査は、CLI側のmode退行も検出する多層防御。
  local cursor_mode_args=(--trust --mode plan --force --sandbox enabled)
  if ! run_agent "$output" "$timeout_seconds" \
    env CURSOR_AGENT=1 "$CURSOR_AGENT_BIN" -p "${cursor_mode_args[@]}" \
      --output-format text \
      --model "$CURSOR_GROK_MODEL" \
      --workspace "$WORKTREE" \
      "$prompt"; then
    return 1
  fi
  if ! result_is_valid "$output" "$result_kind"; then
    echo "delegate-cursor-supervised: Grokの結果markerが欠落・曖昧・末尾外です" >&2
    return 1
  fi
}

# U0e-2の却下原因(発注書と正本の未照合)を再発させないためのgate。詳細:
# docs/reviews/2026-07-22-u0e-2-delegation-guardrails.md
REACT_LABELS_ORDERED=(
  "REACT AUTHORITY:"
  "SOURCE ASSET:"
  "PRESERVE:"
  "REPLACE:"
  "STATE OWNER:"
  "DIAGNOSTIC ROUTE:"
  "NEGATIVE ORACLE:"
  "STOP:"
)

gate_fail() {
  local msg="ORDER-GATE NG: $*"
  echo "$msg" >&2
  [[ -z "${CURRENT_ATTEMPT_DIR:-}" ]] || printf '%s\n' "$msg" >>"$CURRENT_ATTEMPT_DIR/stage-result.txt" 2>/dev/null || true
  exit 3
}

evidence_fail() {
  local msg="EVIDENCE NG: $*"
  echo "$msg" >&2
  [[ -z "${CURRENT_ATTEMPT_DIR:-}" ]] || printf '%s\n' "$msg" >>"$CURRENT_ATTEMPT_DIR/stage-result.txt" 2>/dev/null || true
  exit 6
}

inspect_fail() {
  local msg="INSPECT NG: $*"
  echo "$msg" >&2
  [[ -z "${CURRENT_ATTEMPT_DIR:-}" ]] || printf '%s\n' "$msg" >>"$CURRENT_ATTEMPT_DIR/stage-result.txt" 2>/dev/null || true
  exit 8
}

# git列挙が失敗した場合、素通り(空の成功集合)させず必ずfail closedする。
# process substitution(< <(cmd))はcmdの終了statusを呼び出し元へ伝えないため、
# 安全判定に使うgit呼び出しは必ずfileへ実体化してstatusを明示確認してから読む
scope_enum_fail() {
  local msg="SCOPE NG: git enumeration failed: $*"
  echo "$msg" >&2
  [[ -z "${CURRENT_ATTEMPT_DIR:-}" ]] || printf '%s\n' "$msg" >>"$CURRENT_ATTEMPT_DIR/stage-result.txt" 2>/dev/null || true
  exit 7
}

git_capture_or_fail() {
  local outfile="$1" worktree="$2"
  shift 2
  if ! git -C "$worktree" "$@" >"$outfile" 2>"$outfile.err"; then
    scope_enum_fail "git $*"
  fi
}

gate_require_single_field() {
  # 同じprefixの行が1つでも正規文法を外れたら、他に正しい行があっても採用しない
  local file="$1" label="$2"
  local lines line count=0 value=""
  lines="$(grep -E "^${label}:" "$file" || true)"
  if [[ -n "$lines" ]]; then
    while IFS= read -r line; do
      if [[ "$line" =~ ^${label}:[[:space:]]*$ ]]; then
        gate_fail "$label empty"
      fi
      if [[ ! "$line" =~ ^${label}:\ ([^[:space:]]+)$ ]]; then
        gate_fail "$label malformed: ${line#${label}: }"
      fi
      value="${BASH_REMATCH[1]}"
      count=$((count + 1))
    done <<<"$lines"
  fi
  if [[ "$count" -eq 0 ]]; then
    gate_fail "missing $label"
  fi
  if [[ "$count" -gt 1 ]]; then
    gate_fail "duplicate $label"
  fi
  printf '%s' "$value"
}

gate_reject_symlink_components() {
  # linkdir -> /outside のような中間componentの逃げ道も、最終componentのみの
  # -Lや文字列上の".."判定では検出できないため、経路全componentを実体で歩いて確認する
  local worktree="$1" rel_path="$2"
  local cur="$worktree" part
  local old_ifs="$IFS"
  IFS='/'
  for part in $rel_path; do
    IFS="$old_ifs"
    cur="$cur/$part"
    if [[ -L "$cur" ]]; then
      gate_fail "AUTHORITY path is a symlink: $rel_path"
    fi
    IFS='/'
  done
  IFS="$old_ifs"
}

gate_ledger_row_state() {
  local ledger="$1" id="$2"
  awk -v id="$id" '
    BEGIN { in_section = 0; count = 0 }
    /^## 現在選択中の1件/ { in_section = 1; next }
    in_section && /^## / { in_section = 0 }
    in_section && /^\|/ {
      n = split($0, f, "|")
      if (n < 5) next
      gsub(/^[ \t]+|[ \t]+$/, "", f[3])
      if (f[3] ~ /^-+$/) next
      if (f[3] == id) {
        state = f[5]
        gsub(/^[ \t]+|[ \t]+$/, "", state)
        gsub(/`/, "", state)
        count++
        result = state
      }
    }
    END {
      if (count == 0) { print "ABSENT"; exit }
      if (count > 1) { print "AMBIGUOUS"; exit }
      print result
    }
  ' "$ledger"
}

gate_check_base() {
  local order_file="$1" worktree="$2"
  local base_ref base_sha ref_name resolved_sha worktree_head

  base_ref="$(gate_require_single_field "$order_file" "BASE_REF")"
  if [[ ! "$base_ref" =~ ^refs/heads/[A-Za-z0-9._/-]+$ ]]; then
    gate_fail "BASE_REF malformed: $base_ref"
  fi
  ref_name="${base_ref#refs/heads/}"
  if [[ -z "$ref_name" || "$ref_name" == */ || "$ref_name" == *"//"* || \
        "$ref_name" == *".."* || "$ref_name" == .* || "$ref_name" == */.* || \
        "$ref_name" == *".lock" ]]; then
    gate_fail "BASE_REF malformed: $base_ref"
  fi

  base_sha="$(gate_require_single_field "$order_file" "BASE_SHA")"
  if [[ ! "$base_sha" =~ ^[0-9a-f]{40}$ ]]; then
    gate_fail "BASE_SHA malformed: $base_sha"
  fi

  if ! resolved_sha="$(git -C "$worktree" rev-parse --verify --quiet "$base_ref" 2>/dev/null)"; then
    gate_fail "BASE_REF does not resolve: $base_ref"
  fi
  if [[ "$resolved_sha" != "$base_sha" ]]; then
    gate_fail "BASE_REF does not resolve to BASE_SHA"
  fi

  worktree_head="$(git -C "$worktree" rev-parse HEAD)"
  if [[ "$worktree_head" != "$base_sha" ]]; then
    gate_fail "worktree HEAD != BASE_SHA"
  fi
}

gate_check_grain_and_dependencies() {
  local order_file="$1" worktree="$2"
  local ledger="$worktree/docs/implementation-ledger.md"
  local grain grain_state dep_lines dep_id dep_state

  if [[ ! -f "$ledger" ]]; then
    gate_fail "docs/implementation-ledger.md missing in worktree"
  fi

  grain="$(gate_require_single_field "$order_file" "GRAIN")"
  grain_state="$(gate_ledger_row_state "$ledger" "$grain")"
  case "$grain_state" in
    ABSENT) gate_fail "$grain not found in selected-work ledger" ;;
    AMBIGUOUS) gate_fail "$grain has ambiguous selected-work ledger rows" ;;
    DO) ;;
    *) gate_fail "$grain is $grain_state; dispatch is forbidden" ;;
  esac

  dep_lines="$(grep -E '^DEPENDENCY:' "$order_file" || true)"
  if [[ -z "$dep_lines" ]]; then
    gate_fail "missing DEPENDENCY"
  fi
  while IFS= read -r dep_id; do
    if [[ "$dep_id" =~ ^DEPENDENCY:[[:space:]]*$ ]]; then
      gate_fail "DEPENDENCY empty"
    fi
    if [[ ! "$dep_id" =~ ^DEPENDENCY:\ ([^[:space:]]+)$ ]]; then
      gate_fail "DEPENDENCY malformed: ${dep_id#DEPENDENCY: }"
    fi
    dep_id="${BASH_REMATCH[1]}"
    dep_state="$(gate_ledger_row_state "$ledger" "$dep_id")"
    case "$dep_state" in
      ABSENT) gate_fail "dependency $dep_id not found in selected-work ledger" ;;
      AMBIGUOUS) gate_fail "dependency $dep_id has ambiguous selected-work ledger rows" ;;
      DONE) ;;
      *) gate_fail "dependency $dep_id is $dep_state; dispatch is forbidden" ;;
    esac
  done <<<"$dep_lines"
}

# AUTHORITY行の文法・path安全性検証を、worktree実体照合(gate_check_authorities)と
# BASE_SHA blob照合(gate_check_authorities_at_base)の両方で共有する単一parser。
# 検証結果はGATE_AUTH_PATH/GATE_AUTH_HASHへ返し、hashの取得元(working tree
# ファイル vs BASE_SHA commit blob)という本質的に異なる後段だけを呼び出し側に残す
gate_parse_authority_line() {
  local line="$1"
  if [[ ! "$line" =~ ^AUTHORITY:\ ([^[:space:]]+)\ SHA256:([0-9a-f]{64})$ ]]; then
    gate_fail "AUTHORITY malformed: ${line#AUTHORITY: }"
  fi
  GATE_AUTH_PATH="${BASH_REMATCH[1]}"
  GATE_AUTH_HASH="${BASH_REMATCH[2]}"
  if [[ "$GATE_AUTH_PATH" == /* ]]; then
    gate_fail "AUTHORITY absolute path: $GATE_AUTH_PATH"
  fi
  if [[ "$GATE_AUTH_PATH" == *".."* ]]; then
    gate_fail "AUTHORITY path traversal: $GATE_AUTH_PATH"
  fi
}

gate_check_authorities() {
  local order_file="$1" worktree="$2"
  local authority_lines line auth_full actual_hash

  authority_lines="$(grep -E '^AUTHORITY:' "$order_file" || true)"
  if [[ -z "$authority_lines" ]]; then
    gate_fail "missing AUTHORITY"
  fi
  while IFS= read -r line; do
    gate_parse_authority_line "$line"
    auth_full="$worktree/$GATE_AUTH_PATH"
    # symlinkはworktree外への逃げ道になり得るため、経路や存在確認より先に拒否する
    gate_reject_symlink_components "$worktree" "$GATE_AUTH_PATH"
    if [[ ! -f "$auth_full" ]]; then
      gate_fail "AUTHORITY file missing: $GATE_AUTH_PATH"
    fi
    actual_hash="$(shasum -a 256 "$auth_full" | awk '{print $1}')"
    if [[ "$actual_hash" != "$GATE_AUTH_HASH" ]]; then
      gate_fail "authority hash mismatch: $GATE_AUTH_PATH"
    fi
  done <<<"$authority_lines"
}

gate_check_allowed_files() {
  local order_file="$1"
  local allowed_lines af

  allowed_lines="$(grep -E '^ALLOWED_FILE:' "$order_file" || true)"
  if [[ -z "$allowed_lines" ]]; then
    gate_fail "missing ALLOWED_FILE"
  fi
  GATE_ALLOWED_FILES=()
  while IFS= read -r af; do
    if [[ "$af" =~ ^ALLOWED_FILE:[[:space:]]*$ ]]; then
      gate_fail "ALLOWED_FILE empty"
    fi
    if [[ ! "$af" =~ ^ALLOWED_FILE:\ ([^[:space:]]+)$ ]]; then
      gate_fail "ALLOWED_FILE malformed: ${af#ALLOWED_FILE: }"
    fi
    af="${BASH_REMATCH[1]}"
    if [[ "$af" == /* ]]; then
      gate_fail "ALLOWED_FILE absolute path: $af"
    fi
    if [[ "$af" == *".."* ]]; then
      gate_fail "ALLOWED_FILE path traversal: $af"
    fi
    GATE_ALLOWED_FILES+=("$af")
  done <<<"$allowed_lines"
}

gate_check_clean_worktree() {
  local worktree="$1" ignored_file hidden_file
  if [[ -n "$(git -C "$worktree" status --porcelain)" ]]; then
    gate_fail "isolated worktree is not clean"
  fi
  hidden_file="$(mktemp "$tmp_dir/motolii-gate-hidden.XXXXXX")"
  scope_violations_hidden_by_index "$worktree" 1 >"$hidden_file"
  if [[ -s "$hidden_file" ]]; then
    gate_fail "isolated worktree contains tracked changes hidden by index bits: $(tr '\0' ' ' <"$hidden_file")"
  fi
  rm -f "$hidden_file"
  ignored_file="$(mktemp "$tmp_dir/motolii-gate-ignored.XXXXXX")"
  scope_violations_from_ignored "$worktree" >"$ignored_file"
  if [[ -s "$ignored_file" ]]; then
    gate_fail "isolated worktree contains ignored paths outside ALLOWED_FILE"
  fi
  rm -f "$ignored_file"
}

gate_check_react_labels() {
  local order_file="$1"
  local is_react=0 af label matches count line_no last_line=0

  if grep -qx 'REACT TASK: YES' "$order_file"; then
    is_react=1
  fi
  for af in "${GATE_ALLOWED_FILES[@]}"; do
    # docs/mocks-ui自身/直下の子孫だけを対象とし、docs/mocks-ui-legacy等の兄弟名を誤検知しない
    if [[ "$af" == "docs/mocks-ui" || "$af" == docs/mocks-ui/* || "$af" == *.jsx ]]; then
      is_react=1
    fi
  done
  if [[ "$is_react" -eq 0 ]]; then
    return
  fi

  for label in "${REACT_LABELS_ORDERED[@]}"; do
    matches="$(grep -nE "^${label}" "$order_file" | cut -d: -f1 || true)"
    count=0
    [[ -z "$matches" ]] || count="$(printf '%s\n' "$matches" | wc -l | tr -d ' ')"
    if [[ "$count" -eq 0 ]]; then
      gate_fail "React guard label missing or out of order: $label"
    fi
    if [[ "$count" -gt 1 ]]; then
      gate_fail "React guard label duplicated: $label"
    fi
    line_no="$matches"
    if (( line_no <= last_line )); then
      gate_fail "React guard label missing or out of order: $label"
    fi
    last_line="$line_no"
  done
}

# GR-D2: worktree外(authority hash相当)のBASE_SHA commit bytesに対する検証。
# inspectは実装が許可ファイルへ行った変更(このrunner自身を含み得る)を
# 汚損として誤検知しないよう、working treeではなくcommit blobを照合する。
gate_check_authorities_at_base() {
  local order_file="$1" worktree="$2"
  local base_sha authority_lines line actual_hash mode

  base_sha="$(gate_require_single_field "$order_file" "BASE_SHA")"
  authority_lines="$(grep -E '^AUTHORITY:' "$order_file" || true)"
  if [[ -z "$authority_lines" ]]; then
    gate_fail "missing AUTHORITY"
  fi
  while IFS= read -r line; do
    gate_parse_authority_line "$line"
    mode="$(git -C "$worktree" ls-tree "$base_sha" -- "$GATE_AUTH_PATH" | awk '{print $1}')"
    if [[ -z "$mode" ]]; then
      gate_fail "AUTHORITY file missing at BASE_SHA: $GATE_AUTH_PATH"
    fi
    if [[ "$mode" == "120000" ]]; then
      gate_fail "AUTHORITY path is a symlink at BASE_SHA: $GATE_AUTH_PATH"
    fi
    actual_hash="$(git -C "$worktree" show "${base_sha}:${GATE_AUTH_PATH}" | shasum -a 256 | awk '{print $1}')"
    if [[ "$actual_hash" != "$GATE_AUTH_HASH" ]]; then
      gate_fail "authority hash mismatch: $GATE_AUTH_PATH"
    fi
  done <<<"$authority_lines"
}

# inspectはTerraを再起動しないため、実装後に必ず汚れているworktreeを
# 通常のclean gateへ通さず、base commit照合とscope/checkpoint検証だけを行う
run_dispatch_gate_for_inspect() {
  local order_file="$1" worktree="$2"
  gate_check_base "$order_file" "$worktree"
  gate_check_grain_and_dependencies "$order_file" "$worktree"
  gate_check_authorities_at_base "$order_file" "$worktree"
  gate_check_allowed_files "$order_file"
  gate_check_react_labels "$order_file"
}

# GR-D2: 変更許可閉集合とcontent fingerprintの永続証跡。
# 詳細: docs/reviews/2026-07-22-u0e-2-delegation-guardrails.md

new_attempt_dir() {
  local root="$1"
  mkdir -p "$root"
  local max=0 d n
  for d in "$root"/attempt-*; do
    [[ -d "$d" ]] || continue
    n="${d##*/attempt-}"
    case "$n" in
      ''|*[!0-9]*) continue ;;
    esac
    n=$((10#$n))
    if (( n > max )); then
      max=$n
    fi
  done
  local next=$((max + 1))
  local name path
  name="$(printf 'attempt-%04d' "$next")"
  path="$root/$name"
  mkdir "$path"
  printf '%s' "$path"
}

# ALLOWED_FILEのshell-style glob(*, ?, [...])はpath component単位でのみ照合する。
# bashの[[ == ]]はfnmatchにFNM_PATHNAMEを渡さないため素の*は"/"を跨いで一致してしまい、
# 例えば"scripts/*.sh"が"scripts/sub/a.sh"にも一致する誤検出を起こす。
# ここでは"/"で分割したcomponent数を一致させたうえで、component単位でpattern照合する
path_matches_pattern() {
  local path="$1" pattern="$2"
  # `dir/**`だけは明示した閉じたsubtreeとして再帰一致を許す。通常の`*`は
  # 従来どおりcomponent境界を跨がせない。
  if [[ "$pattern" == */'**' ]]; then
    local subtree="${pattern%/**}"
    [[ "$path" == "$subtree" || "$path" == "$subtree/"* ]]
    return
  fi
  local path_parts=() pattern_parts=()
  local old_ifs="$IFS" part
  IFS='/'
  set -f
  for part in $path; do path_parts+=("$part"); done
  for part in $pattern; do pattern_parts+=("$part"); done
  set +f
  IFS="$old_ifs"
  if [[ "${#path_parts[@]}" -ne "${#pattern_parts[@]}" ]]; then
    return 1
  fi
  local i
  for (( i = 0; i < ${#path_parts[@]}; i++ )); do
    if [[ "${path_parts[$i]}" != ${pattern_parts[$i]} ]]; then
      return 1
    fi
  done
  return 0
}

# GATE_ALLOWED_FILESの各patternに対する単一判定を、複数のscope_violations系
# generatorで共有する
path_is_allowed() {
  local path="$1" pattern
  for pattern in "${GATE_ALLOWED_FILES[@]}"; do
    if path_matches_pattern "$path" "$pattern"; then
      return 0
    fi
  done
  return 1
}

# 変更許可外のtracked/staged/deleted/untracked pathをNUL-safeに列挙する。
# renameは検出せず(--no-renames)delete+addの二レコードへ分解させ、旧/新両方の
# pathを独立に評価する。git status -zの各recordはNULで終端されるため、
# 空白/改行を含むファイル名でも record 境界を誤認しない
scope_violations_from_status() {
  local worktree="$1"
  local record prefix2 rest path i status_file
  status_file="$(mktemp "$tmp_dir/motolii-status.XXXXXX")"
  git_capture_or_fail "$status_file" "$worktree" status --porcelain=v2 -z --untracked-files=all --no-renames
  while IFS= read -r -d '' record; do
    path=""
    prefix2="${record:0:2}"
    case "$prefix2" in
      "1 ")
        rest="$record"
        for i in 1 2 3 4 5 6 7 8; do rest="${rest#* }"; done
        path="$rest"
        ;;
      "2 ")
        rest="$record"
        for i in 1 2 3 4 5 6 7 8 9; do rest="${rest#* }"; done
        path="$rest"
        ;;
      "u ")
        rest="$record"
        for i in 1 2 3 4 5 6 7 8 9 10; do rest="${rest#* }"; done
        path="$rest"
        ;;
      *)
        if [[ "${record:0:1}" == "?" ]]; then
          path="${record:2}"
        fi
        ;;
    esac
    [[ -n "$path" ]] || continue
    if ! path_is_allowed "$path"; then
      printf '%s\0' "$path"
    fi
  done <"$status_file"
  rm -f "$status_file" "$status_file.err"
}

# git index modeの三種(100644/100755/120000)のうち、regular fileの実行bitだけは
# blob shaに含まれないため、内容が同一でもchmodだけで実質的な変更になり得る。
# 現worktree実体(lstat)から実効modeを求め、index記録modeと直接比較する
actual_git_mode_of() {
  local full="$1"
  if [[ -L "$full" ]]; then
    printf '120000'
  elif [[ -f "$full" ]]; then
    if [[ -x "$full" ]]; then
      printf '100755'
    else
      printf '100644'
    fi
  else
    printf ''
  fi
}

# BSD readlink(1)(および$(readlink ...)のtrailing newline剥ぎ取り)はsymlink
# targetの埋め込み/末尾LF byteを失う。perlのreadlink()はsyscallの生byteを
# そのまま返し、printも改行を付加しないため、これを唯一のsymlink target読み取り
# 経路として生manifest・hidden-index比較・ignore/control hash・全体fingerprintの
# 全箇所で共有する。呼び出し側はpipeで直接消費し、$(...)へは最終hash値だけを渡す
raw_symlink_target() {
  perl -e '
    my $t = readlink($ARGV[0]);
    defined($t) or die "readlink failed: $!\n";
    print $t or die "write failed: $!\n";
  ' "$1"
}

# raw_symlink_targetの失敗(perl die等)をpipeline末尾のcommand substitutionへ
# 委ね、set -eの暗黙exit(errno由来のstatus、生のstderr)任せにしない。ここで
# 必ずtmp_dirへ実体化してstatusを明示確認し、失敗はstable「SCOPE NG:」/exit 7
# 経由のscope_enum_failへ正規化する。空inputのSHA-256は非空文字列になるため、
# 呼び出し側の`[[ -n ... ]]`は失敗の証明にならず、この一本化で置き換える
raw_symlink_target_to_file() {
  local path="$1" outfile="$2"
  if ! raw_symlink_target "$path" >"$outfile" 2>/dev/null; then
    scope_enum_fail "readlink $path"
  fi
}

raw_symlink_target_sha256() {
  local path="$1"
  local tmp
  tmp="$(mktemp "$tmp_dir/motolii-readlink.XXXXXX")"
  raw_symlink_target_to_file "$path" "$tmp"
  shasum -a 256 "$tmp" | awk '{print $1}'
  rm -f "$tmp"
}

raw_symlink_target_blob_sha() {
  local worktree="$1" path="$2"
  local tmp
  tmp="$(mktemp "$tmp_dir/motolii-readlink.XXXXXX")"
  raw_symlink_target_to_file "$path" "$tmp"
  git -C "$worktree" hash-object -t blob --stdin <"$tmp" || scope_enum_fail "hash-object symlink target $path"
  rm -f "$tmp"
}

# gitlink配下は親repositoryのstatusがskip-worktree/assume-unchangedや
# submodule ignore設定の影響を受け得る。submodule自身の.git metadata本体へは
# 降りず、worktreeの生byte・symlink target・modeをNUL区切りで決定的にhashする。
gitlink_tree_sha256() {
  local root="$1"
  local paths_file records_file entry rel state raw_hash actual_mode target_hash
  paths_file="$(mktemp "$tmp_dir/motolii-gitlink-paths.XXXXXX")"
  records_file="$(mktemp "$tmp_dir/motolii-gitlink-records.XXXXXX")"
  : >"$records_file"
  if [[ ! -d "$root" ]]; then
    printf 'MISSING\0' >"$records_file"
  else
    if ! find "$root" -mindepth 1 \
      \( -path "$root/.git" -o -path "$root/.git/*" \) -prune -o -print0 \
      >"$paths_file"; then
      scope_enum_fail "find gitlink tree $root"
    fi
    LC_ALL=C sort -z "$paths_file" -o "$paths_file" || scope_enum_fail "sort gitlink tree $root"
    while IFS= read -r -d '' entry; do
      rel="${entry#"$root"/}"
      if [[ -L "$entry" ]]; then
        target_hash="$(raw_symlink_target_sha256 "$entry")"
        state="SYMLINK:120000:${target_hash}"
      elif [[ -f "$entry" ]]; then
        raw_hash="$(shasum -a 256 "$entry" | awk '{print $1}')"
        actual_mode="$(actual_git_mode_of "$entry")"
        state="REGULAR:${actual_mode}:${raw_hash}"
      elif [[ -d "$entry" ]]; then
        actual_mode="$(actual_git_mode_of "$entry")"
        state="DIR:${actual_mode}"
      elif [[ -e "$entry" ]]; then
        state="OTHER"
      else
        state="MISSING"
      fi
      printf '%s\x1f%s\0' "$state" "$rel" >>"$records_file"
    done <"$paths_file"
    # 通常のsubmoduleでは.gitは外部metadataへのpointer fileである。pointerの
    # 差し替えも閉集合の変更なので、その生内容だけをhashへ含める。
    if [[ -L "$root/.git" ]]; then
      target_hash="$(raw_symlink_target_sha256 "$root/.git")"
      printf 'GIT_META_SYMLINK\x1f%s\0' "$target_hash" >>"$records_file"
    elif [[ -f "$root/.git" ]]; then
      raw_hash="$(shasum -a 256 "$root/.git" | awk '{print $1}')"
      printf 'GIT_META_FILE\x1f%s\0' "$raw_hash" >>"$records_file"
    elif [[ -d "$root/.git" ]]; then
      printf 'GIT_META_DIR\0' >>"$records_file"
    fi
  fi
  shasum -a 256 "$records_file" | awk '{print $1}'
  rm -f "$paths_file" "$records_file"
}

# 親indexのgitlink SHAだけでなく、submoduleのHEAD/index/raw worktreeを直接照合する。
# inner indexのhide bitを信用せず、stage-0 blobと生内容を一件ずつ比較する。
gitlink_matches_index() {
  local full="$1" expected_sha="$2"
  local top_raw top head nested_file other_file record mode rest sha stage path current_sha actual_mode
  [[ -d "$full" ]] || return 1
  top_raw="$(git -C "$full" rev-parse --show-toplevel 2>/dev/null || true)"
  if [[ -z "$top_raw" || "$(cd "$top_raw" 2>/dev/null && pwd -P || true)" != "$(cd "$full" && pwd -P)" ]]; then
    # 未初期化gitlinkの空directoryはGit上clean。payloadが一つでもあれば拒否する。
    other_file="$(mktemp "$tmp_dir/motolii-gitlink-uninit.XXXXXX")"
    find "$full" -mindepth 1 -print -quit >"$other_file" 2>/dev/null || return 1
    [[ ! -s "$other_file" ]]
    local empty_status=$?
    rm -f "$other_file"
    return "$empty_status"
  fi
  top="$(cd "$top_raw" && pwd -P)"
  head="$(git -C "$top" rev-parse HEAD 2>/dev/null || true)"
  [[ "$head" == "$expected_sha" ]] || return 1
  git -C "$top" diff-index --cached --quiet "$expected_sha" -- 2>/dev/null || return 1

  nested_file="$(mktemp "$tmp_dir/motolii-gitlink-index.XXXXXX")"
  git_capture_or_fail "$nested_file" "$top" ls-files -z -s
  while IFS= read -r -d '' record; do
    mode="${record%% *}"
    rest="${record#* }"
    sha="${rest%% *}"
    rest="${rest#* }"
    stage="${rest%%$'\t'*}"
    path="${rest#*$'\t'}"
    [[ "$stage" == "0" ]] || { rm -f "$nested_file" "$nested_file.err"; return 1; }
    if [[ "$mode" == "160000" ]]; then
      gitlink_matches_index "$top/$path" "$sha" || { rm -f "$nested_file" "$nested_file.err"; return 1; }
      continue
    fi
    if [[ -L "$top/$path" ]]; then
      current_sha="$(raw_symlink_target_blob_sha "$top" "$top/$path")"
    elif [[ -f "$top/$path" ]]; then
      current_sha="$(git -C "$top" hash-object -t blob --no-filters -- "$path")"
    else
      current_sha=""
    fi
    actual_mode="$(actual_git_mode_of "$top/$path")"
    if [[ "$current_sha" != "$sha" || ( "$mode" != "120000" && "$actual_mode" != "$mode" ) ]]; then
      rm -f "$nested_file" "$nested_file.err"
      return 1
    fi
  done <"$nested_file"
  rm -f "$nested_file" "$nested_file.err"

  other_file="$(mktemp "$tmp_dir/motolii-gitlink-others.XXXXXX")"
  git_capture_or_fail "$other_file" "$top" ls-files -z --others --exclude-standard
  if [[ -s "$other_file" ]]; then rm -f "$other_file" "$other_file.err"; return 1; fi
  # 親worktreeと同じdirectory-root + filesystem walkを使い、submodule内の
  # target/・node_modules/のようなdirectory-class ignoreも子孫まで閉じる。
  capture_ignored_paths "$top" "$other_file"
  if [[ -s "$other_file" ]]; then rm -f "$other_file" "$other_file.err"; return 1; fi
  rm -f "$other_file" "$other_file.err"
  return 0
}

# assume-unchanged/skip-worktree bitはgit status/diffのworktree比較そのものを
# 省略させるため、bitを立てたまま許可外trackedファイルへ手を入れると
# scope_violations_from_statusを素通りする。bitの有無に関わらず全stage-0
# trackedパスの現worktree内容をindex blobと直接hash比較し、変更も独立に拾う。
# 内容が同一でもchmod +x/-xだけの変更(blob shaには現れない)を見逃さないよう、
# 実効modeもindex記録modeと直接比較する
scope_violations_hidden_by_index() {
  local worktree="$1" include_allowed="${2:-0}"
  local record mode rest sha stage path full current_sha actual_mode ls_file
  ls_file="$(mktemp "$tmp_dir/motolii-lsfiles.XXXXXX")"
  git_capture_or_fail "$ls_file" "$worktree" ls-files -z -s
  while IFS= read -r -d '' record; do
    mode="${record%% *}"
    rest="${record#* }"
    sha="${rest%% *}"
    rest="${rest#* }"
    stage="${rest%%$'\t'*}"
    path="${rest#*$'\t'}"
    [[ "$stage" == "0" ]] || continue
    if [[ "$mode" == "160000" ]]; then
      if ! gitlink_matches_index "$worktree/$path" "$sha" && \
         { [[ "$include_allowed" == "1" ]] || ! path_is_allowed "$path"; }; then
        printf '%s\0' "$path"
      fi
      continue
    fi
    full="$worktree/$path"
    if [[ -L "$full" ]]; then
      # symlink blobはtarget文字列そのもの(末尾改行なし)
      current_sha="$(raw_symlink_target_blob_sha "$worktree" "$full")"
    elif [[ -f "$full" ]]; then
      # --no-filtersを付けないと、attackerが仕込んだclean filter/gitattributesが
      # 適用され、index blobを返すよう偽装した内容がそのまま一致してしまう
      current_sha="$(git -C "$worktree" hash-object -t blob --no-filters -- "$path")"
    else
      current_sha=""
    fi
    actual_mode="$(actual_git_mode_of "$full")"
    if [[ "$current_sha" == "$sha" && ( "$mode" == "120000" || "$actual_mode" == "$mode" ) ]]; then
      continue
    fi
    if [[ "$include_allowed" == "1" ]] || ! path_is_allowed "$path"; then
      printf '%s\0' "$path"
    fi
  done <"$ls_file"
  rm -f "$ls_file" "$ls_file.err"
}

# Git実装やignore patternがignored directoryの子孫を省略しても閉集合を狭めない。
# まずignored rootを--directoryで列挙し、directoryはfilesystem walkで全子孫へ
# 展開する。.git metadataはroot列挙に現れないためwalk対象にならない。
capture_ignored_paths() {
  local worktree="$1" outfile="$2"
  local roots_file expanded_file found_file path normalized full entry rel
  roots_file="$(mktemp "$tmp_dir/motolii-ignored-roots.XXXXXX")"
  expanded_file="$(mktemp "$tmp_dir/motolii-ignored-expanded.XXXXXX")"
  : >"$expanded_file"
  git_capture_or_fail "$roots_file" "$worktree" ls-files -z --others --ignored --exclude-standard --directory
  while IFS= read -r -d '' path; do
    normalized="${path%/}"
    full="$worktree/$normalized"
    if [[ -d "$full" && ! -L "$full" ]]; then
      found_file="$(mktemp "$tmp_dir/motolii-ignored-find.XXXXXX")"
      if ! find "$full" -print0 >"$found_file"; then
        scope_enum_fail "find ignored tree $normalized"
      fi
      while IFS= read -r -d '' entry; do
        rel="${entry#"$worktree"/}"
        printf '%s\0' "$rel" >>"$expanded_file"
      done <"$found_file"
      rm -f "$found_file"
    else
      printf '%s\0' "$normalized" >>"$expanded_file"
    fi
  done <"$roots_file"
  LC_ALL=C sort -zu "$expanded_file" >"$outfile" || scope_enum_fail "sort expanded ignored paths"
  rm -f "$roots_file" "$roots_file.err" "$expanded_file"
}

# git statusと通常のls-filesは既存ignore配下を表示しない。build出力も含め、
# worktree内へ書くignored pathは発注書のALLOWED_FILEへ明示させ、それ以外を
# 通常のuntracked pathと同じ閉集合違反として扱う。
scope_violations_from_ignored() {
  local worktree="$1" path ignored_file
  ignored_file="$(mktemp "$tmp_dir/motolii-ignored.XXXXXX")"
  capture_ignored_paths "$worktree" "$ignored_file"
  while IFS= read -r -d '' path; do
    if ! path_is_allowed "$path"; then
      printf '%s\0' "$path"
    fi
  done <"$ignored_file"
  rm -f "$ignored_file" "$ignored_file.err"
}

# parent保持のraw manifestへ、Gitがignoreした許可外fileも内容・mode・symlink
# target込みで加える。これによりignore policyを変えずに成立するtarget/や
# node_modules/配下への持込みもpre/post digestで検出する。
append_ignored_out_of_scope_manifest() {
  local worktree="$1" records_file="$2"
  local path full state raw_hash actual_mode target_hash ignored_file
  ignored_file="$(mktemp "$tmp_dir/motolii-manifest-ignored.XXXXXX")"
  capture_ignored_paths "$worktree" "$ignored_file"
  while IFS= read -r -d '' path; do
    if path_is_allowed "$path"; then
      continue
    fi
    full="$worktree/$path"
    if [[ -L "$full" ]]; then
      target_hash="$(raw_symlink_target_sha256 "$full")"
      state="SYMLINK:120000:${target_hash}"
    elif [[ -f "$full" ]]; then
      raw_hash="$(shasum -a 256 "$full" | awk '{print $1}')"
      actual_mode="$(actual_git_mode_of "$full")"
      state="REGULAR:${actual_mode}:${raw_hash}"
    elif [[ -d "$full" ]]; then
      state="DIR"
    elif [[ -e "$full" ]]; then
      state="OTHER"
    else
      state="MISSING"
    fi
    printf 'I\x1f-\x1f-\x1f0\x1f%s\x1f%s\0' "$state" "$path" >>"$records_file"
  done <"$ignored_file"
  rm -f "$ignored_file" "$ignored_file.err"
}

# parent shellが直接保持するpre/post生scope baseline。第六〜八次修正で追加した
# git status/index比較(scope_violations_from_status/scope_violations_hidden_by_index)は
# porcelainやfilter/assume-unchanged/skip-worktreeの解釈を経由するため、それら自身が
# 偽装され得る余地を完全には塞げない。ここではGATE_ALLOWED_FILES外の全indexレコードを
# stage別に列挙し、attributes/clean filterを経由しない生byte(またはsymlink target/型)を
# 直接hashして、mode(chmod)・index追加/削除・型変化・生content変化を単一digestで捕捉する
build_out_of_scope_manifest() {
  local worktree="$1" outfile="$2"
  local ls_file records_file record rest tag mode sha stage path full state raw_hash
  local actual_mode target_hash
  ls_file="$(mktemp "$tmp_dir/motolii-manifest-ls.XXXXXX")"
  git_capture_or_fail "$ls_file" "$worktree" ls-files -z -s -v
  records_file="$(mktemp "$tmp_dir/motolii-manifest-records.XXXXXX")"
  : >"$records_file"
  while IFS= read -r -d '' record; do
    tag="${record:0:1}"
    rest="${record:2}"
    mode="${rest%% *}"
    rest="${rest#* }"
    sha="${rest%% *}"
    rest="${rest#* }"
    stage="${rest%%$'\t'*}"
    path="${rest#*$'\t'}"
    if path_is_allowed "$path"; then
      continue
    fi
    full="$worktree/$path"
    if [[ "$mode" == "160000" ]]; then
      state="GITLINK:${mode}:${sha}:$(gitlink_tree_sha256 "$full")"
    elif [[ -L "$full" ]]; then
      target_hash="$(raw_symlink_target_sha256 "$full")"
      # symlinkのmodeは常に120000固定(index modeがGitの記録値と食い違うことは
      # ないが、regular fileと同じ「生worktree実効mode」の原則をここでも揃える)
      state="SYMLINK:120000:${target_hash}"
    elif [[ -f "$full" ]]; then
      # --no-filtersでclean filter/gitattributesを経由しない生byteをhashする
      raw_hash="$(git -C "$worktree" hash-object -t blob --no-filters -- "$path" 2>/dev/null)" || scope_enum_fail "hash-object $path"
      # indexが記録するmodeではなく、現worktreeの実効mode(lstatベース)を使う。
      # そうしないと、chmodだけの変更(blob shaに現れない)がmanifest digestへ
      # 反映されない
      actual_mode="$(actual_git_mode_of "$full")"
      state="REGULAR:${actual_mode}:${raw_hash}"
    elif [[ -d "$full" ]]; then
      state="DIR:${mode}:DIR"
    elif [[ -e "$full" ]]; then
      state="OTHER:${mode}:OTHER"
    else
      state="MISSING"
    fi
    # metadata-first/path-lastにすることで、path中の空白/改行がfieldsとの境界誤認を
    # 起こさない(pathは常に最後のfield)。fieldsの区切りにはpath/hash/modeへ現れない
    # US(0x1f)を使う
    printf '%s\x1f%s\x1f%s\x1f%s\x1f%s\x1f%s\0' "$tag" "$mode" "$sha" "$stage" "$state" "$path" >>"$records_file"
  done <"$ls_file"
  rm -f "$ls_file" "$ls_file.err"
  append_ignored_out_of_scope_manifest "$worktree" "$records_file"
  LC_ALL=C sort -z "$records_file" >"$outfile" || scope_enum_fail "sort out-of-scope manifest"
  rm -f "$records_file"
}

# expected-parent-digestはparent shell変数(Terra起動前にbuild_out_of_scope_manifestの
# 結果をhashした値)のみを権威として使う。永続化したevidence file自体を後から読み直して
# 比較の権威にはしない(Terra/Grokのbash toolがevidence_rootへ書き込み得るため)
enforce_out_of_scope_manifest_unchanged() {
  local expected_digest="$1" post_manifest_file="$2" violations_file="$3" worktree="$4" pre_ignore_policy="$5"
  local post_digest named_violations_file post_ignore_policy
  post_digest="$(shasum -a 256 "$post_manifest_file" | awk '{print $1}')"
  : >"$violations_file"
  if [[ "$post_digest" != "$expected_digest" ]]; then
    printf 'SCOPE NG: out-of-allowlist raw manifest changed during implementation\n' >>"$violations_file"
    # manifestのdigest不一致自体はpath名を持たないため、既存のnamed diagnostics
    # (git status/index比較由来、およびignore policy比較)を追加でevidence/stderrへ
    # 載せる。record_scope_violations自体のenumeration失敗はscope_enum_fail内で
    # exitするため、ここへは到達しない
    named_violations_file="$(mktemp "$tmp_dir/motolii-manifest-named.XXXXXX")"
    record_scope_violations "$worktree" "$named_violations_file" || true
    cat "$named_violations_file" >>"$violations_file"
    rm -f "$named_violations_file"
    post_ignore_policy="$(compute_ignore_policy_hash "$worktree")"
    if [[ "$post_ignore_policy" != "$pre_ignore_policy" ]]; then
      printf 'SCOPE NG: ignore policy changed during implementation\n' >>"$violations_file"
    fi
    cat "$violations_file" >&2
    [[ -z "${CURRENT_ATTEMPT_DIR:-}" ]] || cat "$violations_file" >>"$CURRENT_ATTEMPT_DIR/stage-result.txt" 2>/dev/null || true
    exit 7
  fi
}

scope_violations() {
  local worktree="$1"
  local list_file
  list_file="$(mktemp "$tmp_dir/motolii-scope.XXXXXX")"
  : >"$list_file"
  scope_violations_from_status "$worktree" >>"$list_file"
  scope_violations_hidden_by_index "$worktree" >>"$list_file"
  scope_violations_from_ignored "$worktree" >>"$list_file"
  LC_ALL=C sort -z -u "$list_file"
  rm -f "$list_file"
}

# violationsをoutfileへ記録するだけで、exitはしない(検収後の再照合など、
# 呼び出し側が別のprimary診断を先に出したい場面で使う)。
# process substitution(< <(cmd))でscope_violationsを消費すると、内部の
# scope_enum_fail(exit)がsubshell内で止まり、呼び出し元へ伝播しない
# (enumeration失敗を検知できないまま空集合成功として通ってしまう)。
# ここではredirectのみ(subshellを作らない)でtmp fileへ実体化し、その後で読む
record_scope_violations() {
  local worktree="$1" outfile="$2"
  local path found=0 violations_file
  violations_file="$(mktemp "$tmp_dir/motolii-scope-violations.XXXXXX")"
  scope_violations "$worktree" >"$violations_file"
  : >"$outfile"
  while IFS= read -r -d '' path; do
    found=1
    printf 'SCOPE NG: out-of-allowlist path: %s\n' "$path" >>"$outfile"
  done <"$violations_file"
  rm -f "$violations_file"
  [[ "$found" -eq 0 ]]
}

# violationsがあればSCOPE NG:を出してGrok起動前にfail closedする
enforce_scope_closure() {
  local worktree="$1" outfile="$2"
  if ! record_scope_violations "$worktree" "$outfile"; then
    cat "$outfile" >&2
    [[ -z "${CURRENT_ATTEMPT_DIR:-}" ]] || cat "$outfile" >>"$CURRENT_ATTEMPT_DIR/stage-result.txt" 2>/dev/null || true
    exit 7
  fi
}

# ignore policy(.gitignore/.git/info/exclude/core.excludesFile)そのものの
# hash。スコープ判定は通常の git status(--ignored無し)を使うため、Terraが
# .gitignoreへ"*"を書いてから許可外fileを作ると、そのfileも.gitignore自身も
# git statusから消え、スコープ違反として一切検知できなくなる。
# .gitignore自身の列挙にはgit statusが使う除外規則(--exclude-standard)を
# 使わない: 測定対象のignore policyを、その存在有無の判定基準に使う
# 自己参照を避けるため
hash_path_or_empty() {
  local path="$1"
  if [[ -L "$path" ]]; then
    raw_symlink_target_sha256 "$path"
  elif [[ -f "$path" ]]; then
    shasum -a 256 "$path" | awk '{print $1}'
  else
    printf '' | shasum -a 256 | awk '{print $1}'
  fi
}

# core.excludesFile/core.attributesFileの相対pathを、実行時cwdではなく
# 対象worktreeから解決する(Gitの実際の解決基準に合わせる)
resolve_worktree_relative_config_path() {
  local worktree="$1" raw="$2"
  case "$raw" in
    "~/"*) printf '%s' "$HOME/${raw#\~/}" ;;
    /*) printf '%s' "$raw" ;;
    *) printf '%s' "$worktree/$raw" ;;
  esac
}

compute_ignore_policy_hash() {
  local worktree="$1"
  local list_file ls_file ls_sorted path full h common_dir info_exclude info_attrs
  local excludes_file attrs_file resolved global_excludes
  list_file="$(mktemp "$tmp_dir/motolii-ignore.XXXXXX")"
  ls_file="$(mktemp "$tmp_dir/motolii-ignore-ls.XXXXXX")"
  ls_sorted="$(mktemp "$tmp_dir/motolii-ignore-ls-sorted.XXXXXX")"
  : >"$list_file"
  git_capture_or_fail "$ls_file" "$worktree" ls-files -z --cached --others
  # process substitution(< <(cmd))はcmdの終了statusを呼び出し元へ伝えないため、
  # sort失敗が空の成功集合へ化けないよう、明示的にstatusを確認してから読む
  LC_ALL=C sort -z "$ls_file" >"$ls_sorted" || scope_enum_fail "sort ignore-policy file list"
  while IFS= read -r -d '' path; do
    case "$path" in
      .gitignore|*/.gitignore|.gitattributes|*/.gitattributes) ;;
      *) continue ;;
    esac
    full="$worktree/$path"
    h="$(hash_path_or_empty "$full")"
    printf 'control-file:%s:%s\0' "$path" "$h" >>"$list_file"
  done <"$ls_sorted"
  rm -f "$ls_file" "$ls_file.err" "$ls_sorted"

  # info/exclude, info/attributesはlinked worktreeでは共有git dir配下にあるため、
  # $worktree/.gitではなくgit-common-dirから解決する
  common_dir="$(git -C "$worktree" rev-parse --git-common-dir)" || scope_enum_fail "git rev-parse --git-common-dir"
  case "$common_dir" in
    /*) : ;;
    *) common_dir="$worktree/$common_dir" ;;
  esac
  info_exclude="$common_dir/info/exclude"
  printf 'info-exclude:%s\0' "$(hash_path_or_empty "$info_exclude")" >>"$list_file"
  info_attrs="$common_dir/info/attributes"
  printf 'info-attributes:%s\0' "$(hash_path_or_empty "$info_attrs")" >>"$list_file"

  excludes_file="$(git -C "$worktree" config --get core.excludesFile 2>/dev/null || true)"
  if [[ -n "$excludes_file" ]]; then
    resolved="$(resolve_worktree_relative_config_path "$worktree" "$excludes_file")"
    printf 'core-excludesFile:%s:%s\0' "$excludes_file" "$(hash_path_or_empty "$resolved")" >>"$list_file"
  else
    # core.excludesFile未設定時、Gitは既定でXDG configの下のglobal excludesを使う
    global_excludes="${XDG_CONFIG_HOME:-$HOME/.config}/git/ignore"
    printf 'default-global-excludes:%s:%s\0' "$global_excludes" "$(hash_path_or_empty "$global_excludes")" >>"$list_file"
  fi

  attrs_file="$(git -C "$worktree" config --get core.attributesFile 2>/dev/null || true)"
  if [[ -n "$attrs_file" ]]; then
    resolved="$(resolve_worktree_relative_config_path "$worktree" "$attrs_file")"
    printf 'core-attributesFile:%s:%s\0' "$attrs_file" "$(hash_path_or_empty "$resolved")" >>"$list_file"
  else
    printf 'core-attributesFile:unset\0' >>"$list_file"
  fi

  # fileMode/autocrlf/safecrlfやfilter.*.clean/smudgeはstatus/index/属性/フィルタの
  # 解釈そのものを変え得るため、repository/worktree scopeのconfig全体をcontrol
  # policyへ混ぜる(globalなuser設定は対象外、対象worktreeのlocal configのみ)
  printf 'local-config:%s\0' "$(git -C "$worktree" config --local --list 2>/dev/null | LC_ALL=C sort | shasum -a 256 | awk '{print $1}')" >>"$list_file"
  if git -C "$worktree" config --worktree --list >/dev/null 2>&1; then
    printf 'worktree-config:%s\0' "$(git -C "$worktree" config --worktree --list 2>/dev/null | LC_ALL=C sort | shasum -a 256 | awk '{print $1}')" >>"$list_file"
  fi

  shasum -a 256 "$list_file" | awk '{print $1}'
  rm -f "$list_file"
}

# tracked/staged/untracked(非ignore)の全pathをcontent単位でhashし、
# git status文言が同じでも中身が変わった場合を検知できる単一fingerprintにする。
# symlinkはtarget文字列を、通常fileはbytesをhashする。ignore policy hashも
# 混ぜ込み、Grok検収や再開待ち中の書き換えをfingerprintの一致判定だけで
# 検知できるようにする
compute_fingerprint() {
  local worktree="$1"
  local list_file path full h ignore_hash ls_file ignored_file ls_sorted status_file bits_file gitlinks_file
  local record mode rest sha stage
  # tmp_dirの下に置くことで、この関数を離れるどの経路で失敗してもEXIT trapの
  # rm -rf "$tmp_dir" が必ず後始末する(TMPDIR直下だとtrap対象外で残り得る)
  list_file="$(mktemp "$tmp_dir/motolii-fp.XXXXXX")"
  ls_file="$(mktemp "$tmp_dir/motolii-fp-ls.XXXXXX")"
  ignored_file="$(mktemp "$tmp_dir/motolii-fp-ignored.XXXXXX")"
  ls_sorted="$(mktemp "$tmp_dir/motolii-fp-ls-sorted.XXXXXX")"
  status_file="$(mktemp "$tmp_dir/motolii-fp-status.XXXXXX")"
  bits_file="$(mktemp "$tmp_dir/motolii-fp-bits.XXXXXX")"
  gitlinks_file="$(mktemp "$tmp_dir/motolii-fp-gitlinks.XXXXXX")"
  : >"$list_file"
  git_capture_or_fail "$ls_file" "$worktree" ls-files -z --cached --others --exclude-standard
  capture_ignored_paths "$worktree" "$ignored_file"
  cat "$ignored_file" >>"$ls_file"
  # process substitution(< <(cmd))はcmdの終了statusを呼び出し元へ伝えないため、
  # sort失敗が空の成功集合へ化けないよう、明示的にstatusを確認してから読む
  LC_ALL=C sort -z "$ls_file" >"$ls_sorted" || scope_enum_fail "sort fingerprint file list"
  while IFS= read -r -d '' path; do
    full="$worktree/$path"
    if [[ -L "$full" ]]; then
      h="$(raw_symlink_target_sha256 "$full")"
    elif [[ -f "$full" ]]; then
      h="$(shasum -a 256 "$full" | awk '{print $1}')"
    else
      h="$(printf '' | shasum -a 256 | awk '{print $1}')"
    fi
    # chmod +x/-xはbyte内容を変えないため、実効modeも別途fingerprintへ混ぜる
    printf '%s%s%s\0' "$path" "$h" "$(actual_git_mode_of "$full")" >>"$list_file"
  done <"$ls_sorted"
  # directoryとして列挙されるgitlinkは通常file branchの空hashだけでは内容変異を
  # 表せないため、親index SHAとsubmodule raw tree hashを別recordで混ぜる。
  git_capture_or_fail "$gitlinks_file" "$worktree" ls-files -z -s
  while IFS= read -r -d '' record; do
    mode="${record%% *}"
    rest="${record#* }"
    sha="${rest%% *}"
    rest="${rest#* }"
    stage="${rest%%$'\t'*}"
    path="${rest#*$'\t'}"
    if [[ "$mode" == "160000" ]]; then
      printf 'gitlink:%s:%s:%s:%s\0' "$stage" "$sha" "$path" "$(gitlink_tree_sha256 "$worktree/$path")" >>"$list_file"
    fi
  done <"$gitlinks_file"
  # content bytesだけではgit add/git resetのようなindex-only操作(bytesは不変でも
  # staged/unstagedやtracked modeが変わる)を検知できないため、porcelain v2の生record
  # (XY, mode三種, HEAD/index object id)もfingerprintへ混ぜる
  git_capture_or_fail "$status_file" "$worktree" status --porcelain=v2 -z --untracked-files=all --no-renames
  LC_ALL=C sort -z "$status_file" >>"$list_file"
  # assume-unchanged/skip-worktree bitはgit status/diffの表示から変更を隠すため、
  # bitの状態自体もfingerprintへ混ぜ、bitだけを立て直す変異も検知できるようにする
  git_capture_or_fail "$bits_file" "$worktree" ls-files -z -v
  LC_ALL=C sort -z "$bits_file" >>"$list_file"
  ignore_hash="$(compute_ignore_policy_hash "$worktree")"
  printf 'ignore-policy:%s\0' "$ignore_hash" >>"$list_file"
  shasum -a 256 "$list_file" | awk '{print $1}'
  rm -f "$list_file" "$ls_file" "$ls_file.err" "$ignored_file" "$ignored_file.err" "$ls_sorted" "$status_file" "$status_file.err" "$bits_file" "$bits_file.err" "$gitlinks_file" "$gitlinks_file.err"
}

# 発注書に対応するevidence directoryへ、stage到達分のbefore/after証跡
# (人間可読なstatus/diffと機械判定用fingerprint)を残す
snapshot_worktree() {
  local worktree="$1" outdir="$2" prefix="$3"
  git -C "$worktree" status --porcelain=v2 --untracked-files=all --ignored=matching --no-renames \
    >"$outdir/${prefix}-status.txt" 2>&1 || true
  {
    git -C "$worktree" diff
    git -C "$worktree" diff --cached
  } >"$outdir/${prefix}-diff.txt" 2>&1 || true
  compute_ignore_policy_hash "$worktree" >"$outdir/${prefix}-ignore-policy.sha256"
  compute_fingerprint "$worktree" >"$outdir/${prefix}-fingerprint.sha256"
}

# Terraは許可済みfileの中身を変えてよいが、ignore policyそのものを変えて
# 許可外の変更をgit statusから隠すことは許さない。scope closureより前に
# 独立して判定し、隠蔽が成立する前にfail closedする。
# Terraのbash toolはevidence_root(worktree外)にも書き込み得るため、
# pre-terraの永続evidence file自体を後から比較の権威に使うと、Terraがその
# fileを書き換えて偽装できてしまう。before値はTerra起動前にこの関数の外側で
# parent shell変数として確保させ、afterは(post-terra snapshotの永続fileを
# 経由せず)ここで直接再計算した値を使う
enforce_ignore_policy_unchanged() {
  local worktree="$1" attempt_dir="$2" before="$3" after_prefix="$4"
  local after outfile msg
  after="$(compute_ignore_policy_hash "$worktree")"
  if [[ "$before" != "$after" ]]; then
    outfile="$attempt_dir/${after_prefix}-scope-violations.txt"
    msg="SCOPE NG: ignore policy changed during implementation"
    : >"$outfile"
    printf '%s\n' "$msg" >>"$outfile"
    echo "$msg" >&2
    [[ -z "${CURRENT_ATTEMPT_DIR:-}" ]] || printf '%s\n' "$msg" >>"$CURRENT_ATTEMPT_DIR/stage-result.txt" 2>/dev/null || true
    exit 7
  fi
}

# TerraもGrokも承認済みorder本文をargv経由で読めるため、実装/検収stage中に
# 外部order fileまたはこの試行がcopyしたorder.txtのどちらかが変わっていないか
# 直接hash比較する。fingerprintはworktree内しか見ないため、worktree外に置かれる
# order fileの改変はこの独立チェックでしか捕まえられない
verify_order_integrity() {
  local order_file="$1" attempt_dir="$2" expected_hash="$3" stage_label="$4"
  local now_external now_copy
  now_external="$(shasum -a 256 "$order_file" | awk '{print $1}')"
  now_copy="$(shasum -a 256 "$attempt_dir/order.txt" | awk '{print $1}')"
  if [[ "$now_external" != "$expected_hash" || "$now_copy" != "$expected_hash" ]]; then
    evidence_fail "approved order mutated during $stage_label"
  fi
}

# checkpointの発行(publish)/無効化(invalidate)は、この2関数を通じてのみ行う。
# どちらもCHECKPOINT_SETTLED=1を立てて、EXIT trapによる無効化上書きを止める
# ("settled"=現在のディスク状態が意図した最終状態である、という意味)
publish_checkpoint() {
  local evidence_root="$1" attempt_name="$2" order_sha256="$3" task_hash="$4" base_ref="$5" base_sha="$6" head="$7" fingerprint="$8"
  local tmp_cp
  # checkpoint.txtと同じevidence_root配下でtmpを作り、mvを同一filesystem内の
  # atomic renameにする(tmp_dir配下だとmvがdevice間copy+削除になり得て非atomic)
  tmp_cp="$(mktemp "$evidence_root/checkpoint.tmp.XXXXXX")"
  {
    echo "ATTEMPT: $attempt_name"
    echo "ORDER_SHA256: $order_sha256"
    echo "TASK_SHA256: $task_hash"
    echo "BASE_REF: $base_ref"
    echo "BASE_SHA: $base_sha"
    echo "HEAD: $head"
    echo "FINGERPRINT: $fingerprint"
  } >"$tmp_cp"
  mv -f "$tmp_cp" "$evidence_root/checkpoint.txt"
  CHECKPOINT_SETTLED=1
}

invalidate_checkpoint() {
  local evidence_root="$1"
  rm -f "$evidence_root/checkpoint.txt"
  CHECKPOINT_SETTLED=1
}

# inspectは、実装成功直後に残したcheckpointが現在のorder/task/base/head/
# worktree fingerprintと完全一致する時だけ進む。driftや証跡欠落はEVIDENCE NG
# とし、Terraは元よりGrokも起動しない。
# 比較は必ずこの関数がindependentに再計算した値(order_sha256/base_ref/base_sha/
# head_now/fp_now)を基準に行い、checkpoint file自体を比較の権威にはしない。
# 一致した値はVALIDATED_*globalへ残し、以降の再publishがcheckpoint fileの
# 中身を再度信用せずに済むようにする
validate_checkpoint() {
  local evidence_root="$1" order_file="$2" task_hash="$3" worktree="$4"
  local checkpoint="$evidence_root/checkpoint.txt"
  if [[ ! -f "$checkpoint" ]]; then
    evidence_fail "missing checkpoint"
  fi
  local order_sha256 base_ref base_sha head_now fp_now
  local cp_attempt cp_order cp_task cp_base_ref cp_base_sha cp_head cp_fp
  order_sha256="$(shasum -a 256 "$order_file" | awk '{print $1}')"
  base_ref="$(gate_require_single_field "$order_file" "BASE_REF")"
  base_sha="$(gate_require_single_field "$order_file" "BASE_SHA")"
  head_now="$(git -C "$worktree" rev-parse HEAD)"
  fp_now="$(compute_fingerprint "$worktree")"

  cp_attempt="$(awk -F': ' '$1=="ATTEMPT"{print $2}' "$checkpoint")"
  cp_order="$(awk -F': ' '$1=="ORDER_SHA256"{print $2}' "$checkpoint")"
  cp_task="$(awk -F': ' '$1=="TASK_SHA256"{print $2}' "$checkpoint")"
  cp_base_ref="$(awk -F': ' '$1=="BASE_REF"{print $2}' "$checkpoint")"
  cp_base_sha="$(awk -F': ' '$1=="BASE_SHA"{print $2}' "$checkpoint")"
  cp_head="$(awk -F': ' '$1=="HEAD"{print $2}' "$checkpoint")"
  cp_fp="$(awk -F': ' '$1=="FINGERPRINT"{print $2}' "$checkpoint")"

  [[ -n "$cp_attempt" && "$cp_attempt" =~ ^attempt-[0-9]+$ ]] || evidence_fail "checkpoint missing attempt binding"
  [[ -n "$cp_order" && "$cp_order" == "$order_sha256" ]] || evidence_fail "approved order drifted from checkpoint"
  [[ -n "$cp_task" && "$cp_task" == "$task_hash" ]] || evidence_fail "task drifted from checkpoint"
  [[ -n "$cp_base_ref" && "$cp_base_ref" == "$base_ref" ]] || evidence_fail "BASE_REF drifted from checkpoint"
  [[ -n "$cp_base_sha" && "$cp_base_sha" == "$base_sha" ]] || evidence_fail "BASE_SHA drifted from checkpoint"
  [[ -n "$cp_head" && "$cp_head" == "$head_now" ]] || evidence_fail "worktree HEAD drifted from checkpoint"
  [[ -n "$cp_fp" && "$cp_fp" == "$fp_now" ]] || evidence_fail "worktree fingerprint drifted from checkpoint"

  # checkpointが指す試行が実際に"STAGE: terra SUCCESS"をこの試行のstage-result.txtへ
  # 記録済みでない限り、checkpoint fileの中身だけを信用しない
  if [[ ! -d "$evidence_root/$cp_attempt" ]] || \
     ! grep -qx 'STAGE: terra SUCCESS' "$evidence_root/$cp_attempt/stage-result.txt" 2>/dev/null; then
    evidence_fail "checkpoint attempt has no recorded terra success"
  fi

  VALIDATED_ATTEMPT="$cp_attempt"
  VALIDATED_ORDER_SHA256="$order_sha256"
  VALIDATED_TASK_SHA256="$task_hash"
  VALIDATED_BASE_REF="$base_ref"
  VALIDATED_BASE_SHA="$base_sha"
  VALIDATED_HEAD="$head_now"
  VALIDATED_FINGERPRINT="$fp_now"
}

# Grok検収の起動から終了までを一つのstageとして、直前/直後のfingerprintを
# 比較する。read-only検収者がworktreeを変えていればACCEPTでも無効化する。
# checkpointの発行判断はここで完結させる: 呼び出し側がmark_checkpoint_at_riskで
# 既にEXIT trapを「無効化がデフォルト」に倒した後、fingerprint/scope/order
# integrityが保たれていることを確認できた時だけpublish_checkpointし、
# 崩れていればinvalidate_checkpointする。どちらもmodel出力ではなくこの関数
# (parent)が最後に確定させた値だけを使う
run_inspection_stage() {
  local worktree="$1" task="$2" order_txt="$3" attempt_dir="$4" inspection_timeout="$5"
  local order_file="$6" expected_order_sha256="$7"
  local evidence_root="$8" cp_attempt_name="$9" cp_task_hash="${10}" cp_base_ref="${11}" cp_base_sha="${12}" cp_head="${13}" cp_fingerprint="${14}"
  local pre_fp post_fp inspection_prompt

  snapshot_worktree "$worktree" "$attempt_dir" "pre-grok"
  pre_fp="$(cat "$attempt_dir/pre-grok-fingerprint.sha256")"
  if [[ "$pre_fp" != "$cp_fingerprint" ]]; then
    invalidate_checkpoint "$evidence_root"
    inspect_fail "worktree fingerprint drifted before grok inspection started"
  fi

  inspection_prompt=$(cat <<EOF
You are the read-only acceptance supervisor for Motolii. Do not edit files,
commit, push, create a PR, spawn subagents, or delegate. Inspect the actual diff
and rerun required evidence now. Verify line-by-line against the binding order
and authorities. Green tests alone are insufficient. Look for scope drift,
contract-avoidance, weakened tests, missing negative cases, duplicate state or
logic, raw public APIs, non-atomic failure, unbounded work, and unfinished gates.
Do not search outside the selected worktree, run broad filesystem find commands,
or launch background commands. Complete and reap every command before deciding.

Classify P0/P1/P2 with file and line evidence. Any P0/P1, missing required test,
out-of-allowlist edit, or unverifiable command requires rejection. End with one
exact plain-text final line: VERDICT: ACCEPT or VERDICT: REJECT. Do not bold it,
quote it, append text, run another tool, or report background command status
after that line.

Original user task:
$task

Binding order:
$order_txt
EOF
  )

  echo
  echo "## 3. Cursor Grok 4.5 High read-only inspection"
  if ! (cd "$worktree" && run_supervisor "$attempt_dir/grok-stdout.txt" "$inspection_prompt" verdict "$inspection_timeout"); then
    [[ ! -f "$attempt_dir/grok-stdout.txt" ]] || cat "$attempt_dir/grok-stdout.txt"
    snapshot_worktree "$worktree" "$attempt_dir" "post-grok"
    echo "STAGE: grok FAILED_OR_TIMEOUT" >>"$attempt_dir/stage-result.txt"
    # timeout/失敗自体はworktreeを汚していない限りcheckpointを潰さない。
    # これにより後続のinspectがTerraを再実行せずに再開できる。ただしfingerprintが
    # 保たれていても、Grokのbash toolはworktree外の承認済みorder(外部fileとこの
    # 試行のcopyの両方)を書き換え得るため、republishする前に独立して確認する
    post_fp="$(cat "$attempt_dir/post-grok-fingerprint.sha256")"
    if [[ "$(shasum -a 256 "$order_file" | awk '{print $1}')" != "$expected_order_sha256" || \
          "$(shasum -a 256 "$attempt_dir/order.txt" | awk '{print $1}')" != "$expected_order_sha256" ]]; then
      invalidate_checkpoint "$evidence_root"
      evidence_fail "approved order mutated during grok inspection"
    fi
    if [[ "$post_fp" == "$pre_fp" ]]; then
      publish_checkpoint "$evidence_root" "$cp_attempt_name" "$expected_order_sha256" "$cp_task_hash" "$cp_base_ref" "$cp_base_sha" "$cp_head" "$pre_fp"
    else
      invalidate_checkpoint "$evidence_root"
    fi
    exit 1
  fi
  cat "$attempt_dir/grok-stdout.txt"

  snapshot_worktree "$worktree" "$attempt_dir" "post-grok"
  post_fp="$(cat "$attempt_dir/post-grok-fingerprint.sha256")"

  if [[ "$post_fp" != "$pre_fp" ]]; then
    invalidate_checkpoint "$evidence_root"
    record_scope_violations "$worktree" "$attempt_dir/post-grok-scope-violations.txt" || true
    [[ ! -s "$attempt_dir/post-grok-scope-violations.txt" ]] || cat "$attempt_dir/post-grok-scope-violations.txt" >&2
    inspect_fail "worktree fingerprint changed during read-only inspection"
  fi

  # fingerprintが変わっていなくても、再検証として scope closure を独立に再確認する
  if ! record_scope_violations "$worktree" "$attempt_dir/post-grok-scope-violations.txt"; then
    invalidate_checkpoint "$evidence_root"
    cat "$attempt_dir/post-grok-scope-violations.txt" >&2
    [[ -z "${CURRENT_ATTEMPT_DIR:-}" ]] || cat "$attempt_dir/post-grok-scope-violations.txt" >>"$CURRENT_ATTEMPT_DIR/stage-result.txt" 2>/dev/null || true
    exit 7
  fi

  # worktree fingerprintはworktree外のorder fileを見ないため、Grok起動前後で
  # 承認済みorder本文(外部fileとこの試行のcopyの両方)が変わっていないか独立に確認する
  if [[ "$(shasum -a 256 "$order_file" | awk '{print $1}')" != "$expected_order_sha256" || \
        "$(shasum -a 256 "$attempt_dir/order.txt" | awk '{print $1}')" != "$expected_order_sha256" ]]; then
    invalidate_checkpoint "$evidence_root"
    evidence_fail "approved order mutated during grok inspection"
  fi

  # ここまでintegrityが保たれているため、ACCEPT/REJECTいずれの結果でも
  # checkpointを(parent保持値で)再発行し、後続inspectの再開余地を残す
  publish_checkpoint "$evidence_root" "$cp_attempt_name" "$expected_order_sha256" "$cp_task_hash" "$cp_base_ref" "$cp_base_sha" "$cp_head" "$pre_fp"

  if ! grep -qx 'VERDICT: ACCEPT' "$attempt_dir/grok-stdout.txt"; then
    echo "delegate-cursor-supervised: Grok検収REJECT。差分は隔離したまま採用しません" >&2
    echo "STAGE: grok REJECT" >>"$attempt_dir/stage-result.txt"
    exit 4
  fi
  echo "STAGE: grok ACCEPT" >>"$attempt_dir/stage-result.txt"

  if [[ "$REVIEW_PROFILE" == "grok+fable" ]]; then
    local fable_prompt fable_pre_fp fable_post_fp
    snapshot_worktree "$worktree" "$attempt_dir" "pre-fable"
    fable_pre_fp="$(cat "$attempt_dir/pre-fable-fingerprint.sha256")"
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
one exact plain-text final line: VERDICT: ACCEPT or VERDICT: REJECT.

Original user task:
$task

Binding order:
$order_txt
EOF
    )
    echo
    echo "## 4. Claude Code Fable read-only inspection"
    if ! (cd "$worktree" && run_agent "$attempt_dir/fable-inspection.txt" "$inspection_timeout" \
      env CLAUDE_DELEGATED=1 "$CLAUDE_AGENT_BIN" -p \
        --model "$FABLE_MODEL" \
        --permission-mode default \
        --allowedTools Read,Glob,Grep,Bash \
        --disallowedTools Edit,Write \
        --output-format text \
        "$fable_prompt"); then
      [[ ! -f "$attempt_dir/fable-inspection.txt" ]] || cat "$attempt_dir/fable-inspection.txt"
      snapshot_worktree "$worktree" "$attempt_dir" "post-fable"
      echo "STAGE: fable FAILED_OR_TIMEOUT" >>"$attempt_dir/stage-result.txt"
      exit 1
    fi
    cat "$attempt_dir/fable-inspection.txt"
    snapshot_worktree "$worktree" "$attempt_dir" "post-fable"
    fable_post_fp="$(cat "$attempt_dir/post-fable-fingerprint.sha256")"
    if [[ "$fable_post_fp" != "$fable_pre_fp" ]]; then
      invalidate_checkpoint "$evidence_root"
      inspect_fail "worktree fingerprint changed during Fable read-only inspection"
    fi
    if [[ "$(shasum -a 256 "$order_file" | awk '{print $1}')" != "$expected_order_sha256" || \
          "$(shasum -a 256 "$attempt_dir/order.txt" | awk '{print $1}')" != "$expected_order_sha256" ]]; then
      invalidate_checkpoint "$evidence_root"
      evidence_fail "approved order mutated during Fable inspection"
    fi
    if ! result_is_valid "$attempt_dir/fable-inspection.txt" verdict; then
      echo "delegate-cursor-supervised: Fableの結果markerが欠落・曖昧・末尾外です" >&2
      echo "STAGE: fable INVALID" >>"$attempt_dir/stage-result.txt"
      exit 1
    fi
    if ! grep -qx 'VERDICT: ACCEPT' "$attempt_dir/fable-inspection.txt"; then
      echo "delegate-cursor-supervised: Fable検収REJECT。差分は隔離したまま採用しません" >&2
      echo "STAGE: fable REJECT" >>"$attempt_dir/stage-result.txt"
      exit 4
    fi
    echo "STAGE: fable ACCEPT" >>"$attempt_dir/stage-result.txt"
  fi
  echo "delegate-cursor-supervised: 必須検収ACCEPT。Codex最終レビュー待ちです"
}

run_dispatch_gate() {
  local order_file="$1" worktree="$2"
  gate_check_base "$order_file" "$worktree"
  gate_check_grain_and_dependencies "$order_file" "$worktree"
  gate_check_authorities "$order_file" "$worktree"
  gate_check_allowed_files "$order_file"
  gate_check_clean_worktree "$worktree"
  gate_check_react_labels "$order_file"
}

# gate_check_baseがBASE_REF/BASE_SHAを検証した後だけ呼ぶ。発注書本文の間接的な
# コピーだけでなく、到達した試行のmetadataへ直接BASE_REF/BASE_SHAを残す
record_base_metadata() {
  local order_file="$1" attempt_dir="$2"
  local base_ref base_sha
  base_ref="$(gate_require_single_field "$order_file" "BASE_REF")"
  base_sha="$(gate_require_single_field "$order_file" "BASE_SHA")"
  {
    echo "BASE_REF: $base_ref"
    echo "BASE_SHA: $base_sha"
  } >>"$attempt_dir/metadata.txt"
}

if [[ "$MODE" == "prepare" ]]; then
  TASK_CLASS="${DELEGATION_TASK_CLASS:-standard}"
  if ! select_routing "$TASK_CLASS"; then
    exit 2
  fi
  supervisor_prompt=$(cat <<EOF
You are the read-only on-site supervisor for Motolii. Do not edit files, commit,
push, create a PR, spawn subagents, or delegate. Read AGENTS.md and every required
authority completely. Inspect the current worktree and existing diff. Turn the
user task into a binding implementation order for GPT-5.6 Terra. Do not invent
unresolved product meaning or public contracts.

The order must contain objective, current code facts, authoritative spec/task IDs,
an exact closed file allowlist, non-goals, helpers to reuse, invariants, STOP
conditions, positive and negative tests, exact commands, and integration gates.
Forbid suppressions, expected-value or golden rewrites, fixture special-cases,
raw scanners that bypass typed boundaries, public raw mutation APIs, invented
serde defaults, duplicate planners/helpers, partial mutation, TODO stubs, and
adjacent-ticket expansion.

The order must also emit the fields the dispatch gate checks mechanically before
Terra is started: exactly one \`GRAIN: <id>\`, exactly one
\`BASE_REF: refs/heads/<full-branch-name>\`, exactly one full 40-hex
\`BASE_SHA: <sha>\` that BASE_REF resolves to and that equals the isolated
worktree HEAD, one or more \`DEPENDENCY: <id>\` lines, one or more
\`AUTHORITY: <worktree-relative-path> SHA256:<64-hex>\` lines, and one or more
\`ALLOWED_FILE: <worktree-relative-path-or-glob>\` lines. Before writing GRAIN or
DEPENDENCY, read the target worktree's docs/implementation-ledger.md
selected-work table and confirm GRAIN's own row states exactly \`DO\` and every
DEPENDENCY row states exactly \`DONE\`; never infer these states from prose or
from a different worktree. Before writing an AUTHORITY line, hash the file
inside the target worktree and copy that exact hash. If the order touches a
React surface (exact \`REACT TASK: YES\`, an ALLOWED_FILE under docs/mocks-ui, or
an ALLOWED_FILE ending in .jsx), also include, exactly once and in this order:
REACT AUTHORITY:, SOURCE ASSET:, PRESERVE:, REPLACE:, STATE OWNER:,
DIAGNOSTIC ROUTE:, NEGATIVE ORACLE:, STOP:. Merely mentioning React in prose
does not require these labels.

For a mechanical/Spark order, make the packet self-contained from the closed
order, allowed files, and explicitly named authorities. Do not require inherited
conversation history, repository-wide archaeology, multiple-spec meaning
judgment, or unspecified public-boundary discovery. If those are necessary,
return ORDER: STOP so Codex can select a higher task class.

Do not repeat the Codex-owned routing labels in the draft; the dispatcher appends
them after validating the terminal marker.

The last non-empty line must be exactly plain text ORDER: READY only if every
ledger, authority, and label fact above is mechanically true; otherwise end with
plain text ORDER: STOP. Do not bold it, quote it, or append text.

User task:
$task
EOF
  )
  echo "## 1. Cursor Grok 4.5 High supervisor order draft"
  if ! (cd "$WORKTREE" && run_supervisor "$tmp_dir/order.txt" "$supervisor_prompt" order); then
    [[ ! -f "$tmp_dir/order.txt" ]] || cat "$tmp_dir/order.txt"
    exit 1
  fi
  cat "$tmp_dir/order.txt"
  {
    cat "$tmp_dir/order.txt"
    echo "SUPERVISOR_BACKEND: cursor-grok"
    echo "SUPERVISOR_MODEL: $CURSOR_GROK_MODEL"
    echo "TASK_CLASS: $TASK_CLASS"
    echo "IMPLEMENTER_MODEL: $IMPLEMENTER_MODEL"
    echo "REVIEW_PROFILE: $REVIEW_PROFILE"
    echo "FABLE_MODEL: $FABLE_MODEL"
    echo "TASK_SHA256: $task_hash"
  } >"$ORDER_FILE"
  if ! grep -qx 'ORDER: READY' "$tmp_dir/order.txt"; then
    echo "delegate-cursor-supervised: GrokがREADYを出していません" >&2
    exit 3
  fi
  echo "delegate-cursor-supervised: 発注書案を保存しました: $ORDER_FILE" >&2
  echo "delegate-cursor-supervised: Codex審査後に CODEX PRECHECK: APPROVED を追記してください" >&2
  exit 0
fi

if [[ ! -f "$ORDER_FILE" ]]; then
  echo "delegate-cursor-supervised: 承認対象の発注書がありません" >&2
  exit 2
fi
if ! grep -qx 'ORDER: READY' "$ORDER_FILE"; then
  echo "delegate-cursor-supervised: ORDER: READY がありません" >&2
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
TERRA_MODEL="$IMPLEMENTER_MODEL"
if ! grep -qx 'SUPERVISOR_BACKEND: cursor-grok' "$ORDER_FILE" ||
   ! grep -qx "SUPERVISOR_MODEL: $CURSOR_GROK_MODEL" "$ORDER_FILE" ||
   [[ "$ORDER_IMPLEMENTER_MODEL" != "$IMPLEMENTER_MODEL" ]] ||
   [[ "$ORDER_REVIEW_PROFILE" != "$REVIEW_PROFILE" ]] ||
   [[ "$ORDER_FABLE_MODEL" != "$FABLE_MODEL" ]]; then
  echo "delegate-cursor-supervised: 発注書のモデル経路がTASK_CLASS対応表と一致しません" >&2
  exit 3
fi
if ! grep -qx 'CODEX PRECHECK: APPROVED' "$ORDER_FILE"; then
  echo "delegate-cursor-supervised: Codex事前承認がありません" >&2
  exit 3
fi
if [[ "$REVIEW_PROFILE" == "grok+fable" ]] && ! command -v "$CLAUDE_AGENT_BIN" >/dev/null 2>&1; then
  echo "delegate-cursor-supervised: Fable必須クラスですがClaude Code '$CLAUDE_AGENT_BIN' が見つかりません" >&2
  exit 127
fi

# GR-D2: 発注書ごとのevidence directoryへ、execute/inspectの各試行をappend-onlyで残す
evidence_root="${ORDER_FILE}.evidence"
mkdir -p "$evidence_root"
attempt_dir="$(new_attempt_dir "$evidence_root")"
CURRENT_ATTEMPT_DIR="$attempt_dir"
attempt_name="$(basename "$attempt_dir")"
cp "$ORDER_FILE" "$attempt_dir/order.txt"
# Terra/Grokが起動する前の承認済みorder本文のhash。checkpointへはこの
# pre-model hashだけを刻み、各stage後にこの値との一致を独立に再確認する
approved_order_sha256="$(shasum -a 256 "$attempt_dir/order.txt" | awk '{print $1}')"
printf '%s' "$task" >"$attempt_dir/task.txt"
{
  echo "MODE: $MODE"
  echo "TASK_SHA256: $task_hash"
  echo "WORKTREE: $WORKTREE"
  echo "SUPERVISOR_MODEL: $CURSOR_GROK_MODEL"
  echo "TASK_CLASS: $TASK_CLASS"
  echo "IMPLEMENTER_MODEL: $TERRA_MODEL"
  echo "REVIEW_PROFILE: $REVIEW_PROFILE"
  echo "FABLE_MODEL: $FABLE_MODEL"
} >"$attempt_dir/metadata.txt"

if [[ "$MODE" == "inspect" ]]; then
  # inspectはTerraを再起動しない。実装成功直後のcheckpointに現在の
  # order/task/base/head/worktree fingerprintが一致する時だけ、scope closureを
  # 再確認してGrokだけを起動する
  validate_checkpoint "$evidence_root" "$ORDER_FILE" "$task_hash" "$WORKTREE"
  run_dispatch_gate_for_inspect "$ORDER_FILE" "$WORKTREE"
  record_base_metadata "$ORDER_FILE" "$attempt_dir"
  enforce_scope_closure "$WORKTREE" "$attempt_dir/pre-grok-scope-violations.txt"
  # ここまでの検証はcheckpointを変更しない(先行するinspect失敗の証跡を破壊しない)。
  # Grokを起動する直前にだけEXIT trapを「無効化がデフォルト」へ倒す
  mark_checkpoint_at_risk "$evidence_root"
  run_inspection_stage "$WORKTREE" "$task" "$(cat "$attempt_dir/order.txt")" "$attempt_dir" "$INSPECTION_TIMEOUT_SECONDS" \
    "$ORDER_FILE" "$approved_order_sha256" \
    "$evidence_root" "$VALIDATED_ATTEMPT" "$VALIDATED_TASK_SHA256" "$VALIDATED_BASE_REF" "$VALIDATED_BASE_SHA" "$VALIDATED_HEAD" "$VALIDATED_FINGERPRINT"
  exit 0
fi

run_dispatch_gate "$ORDER_FILE" "$WORKTREE"
# Terra起動前に既存checkpointを即時無効化する。CHECKPOINT_SETTLEDは0のままにして
# おくことで、Terraがcheckpoint.txtを自分で偽造してもEXIT trapが後始末する
rm -f "$evidence_root/checkpoint.txt"
mark_checkpoint_at_risk "$evidence_root"
record_base_metadata "$ORDER_FILE" "$attempt_dir"
snapshot_worktree "$WORKTREE" "$attempt_dir" "pre-terra"
# Terra起動前、この試行のevidence fileがまだ書き換えられていないうちに
# ignore policy hashをparent shell変数として確保する(enforce_ignore_policy_unchanged
# 側のコメント参照)
pre_terra_ignore_policy="$(cat "$attempt_dir/pre-terra-ignore-policy.sha256")"
# 同様に、生scope manifestのdigestもTerra起動前にparent shell変数として確保する。
# 永続化したevidence fileはcopyに過ぎず、比較の権威はこの変数だけが持つ
build_out_of_scope_manifest "$WORKTREE" "$attempt_dir/pre-terra-out-of-scope-manifest.nul"
pre_terra_manifest_digest="$(shasum -a 256 "$attempt_dir/pre-terra-out-of-scope-manifest.nul" | awk '{print $1}')"
printf '%s\n' "$pre_terra_manifest_digest" >"$attempt_dir/pre-terra-out-of-scope-manifest.sha256"

head_before="$(git -C "$WORKTREE" rev-parse HEAD)"
implementation_prompt=$(cat <<EOF
You are the implementation contractor for Motolii. The binding order below was
approved by Codex. Read AGENTS.md and every source
named by the order. Implement only the allowed scope in the current isolated
worktree. Do not write outside this worktree, reinterpret requirements, broaden
file scope, invent defaults, weaken tests, commit, push, or create a PR. Do not
run this delegation script recursively. If exact implementation is blocked, stop
and report the conflicting authority and code evidence instead of improvising.

Original user task:
$task

Binding order:
$(cat "$attempt_dir/order.txt")
EOF
)

echo
echo "## 2. GPT-5.6 Terra implementation"
if ! run_agent "$attempt_dir/terra-stdout.txt" "$TERRA_TIMEOUT_SECONDS" \
  env CODEX_DELEGATED=1 "$CODEX_AGENT_BIN" --ask-for-approval never exec \
    --ephemeral --color never --model "$TERRA_MODEL" \
    --sandbox danger-full-access --cd "$WORKTREE" \
    "$implementation_prompt"; then
  [[ ! -f "$attempt_dir/terra-stdout.txt" ]] || cat "$attempt_dir/terra-stdout.txt"
  snapshot_worktree "$WORKTREE" "$attempt_dir" "post-terra"
  echo "STAGE: terra FAILED_OR_TIMEOUT" >>"$attempt_dir/stage-result.txt"
  invalidate_checkpoint "$evidence_root"
  exit 1
fi
cat "$attempt_dir/terra-stdout.txt"
if [[ "$(git -C "$WORKTREE" rev-parse HEAD)" != "$head_before" ]]; then
  echo "delegate-cursor-supervised: 受注者がcommitを作成したため検収へ進みません" >&2
  snapshot_worktree "$WORKTREE" "$attempt_dir" "post-terra"
  echo "STAGE: terra COMMIT_FORBIDDEN" >>"$attempt_dir/stage-result.txt"
  invalidate_checkpoint "$evidence_root"
  exit 5
fi

# process group reap(run_agent内)後、通常のgit status由来のscope closureより先に、
# parent保持のpre-Terra生manifest digestと直接突き合わせる
build_out_of_scope_manifest "$WORKTREE" "$attempt_dir/post-terra-out-of-scope-manifest.nul"
enforce_out_of_scope_manifest_unchanged "$pre_terra_manifest_digest" \
  "$attempt_dir/post-terra-out-of-scope-manifest.nul" \
  "$attempt_dir/post-terra-out-of-scope-manifest-violations.txt" \
  "$WORKTREE" "$pre_terra_ignore_policy"

snapshot_worktree "$WORKTREE" "$attempt_dir" "post-terra"
enforce_ignore_policy_unchanged "$WORKTREE" "$attempt_dir" "$pre_terra_ignore_policy" "post-terra"
enforce_scope_closure "$WORKTREE" "$attempt_dir/post-terra-scope-violations.txt"
# worktree fingerprintはworktree外のorder fileを見ないため、Terra実装中に
# 承認済みorder本文(外部fileとこの試行のcopyの両方)が変わっていないか独立に確認する
verify_order_integrity "$ORDER_FILE" "$attempt_dir" "$approved_order_sha256" "terra implementation"
echo "STAGE: terra SUCCESS" >>"$attempt_dir/stage-result.txt"

post_impl_fp="$(cat "$attempt_dir/post-terra-fingerprint.sha256")"
base_ref_val="$(gate_require_single_field "$ORDER_FILE" "BASE_REF")"
base_sha_val="$(gate_require_single_field "$ORDER_FILE" "BASE_SHA")"
publish_checkpoint "$evidence_root" "$attempt_name" "$approved_order_sha256" "$task_hash" "$base_ref_val" "$base_sha_val" "$head_before" "$post_impl_fp"

# Grokを起動する直前にもう一度EXIT trapを「無効化がデフォルト」へ倒す。
# run_inspection_stage自身がGrokの結果に応じてpublish/invalidateを確定させる
mark_checkpoint_at_risk "$evidence_root"
run_inspection_stage "$WORKTREE" "$task" "$(cat "$attempt_dir/order.txt")" "$attempt_dir" "$INSPECTION_TIMEOUT_SECONDS" \
  "$ORDER_FILE" "$approved_order_sha256" \
  "$evidence_root" "$attempt_name" "$task_hash" "$base_ref_val" "$base_sha_val" "$head_before" "$post_impl_fp"
