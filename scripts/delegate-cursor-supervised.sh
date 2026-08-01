#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
PRIMARY_WORKTREE_RAW="$(git -C "$ROOT_DIR" worktree list --porcelain | awk '/^worktree / && !found { print substr($0, 10); found=1 }')"
PRIMARY_WORKTREE="$(cd "$PRIMARY_WORKTREE_RAW" && pwd -P)"
CURSOR_AGENT_BIN="${CURSOR_AGENT_BIN:-cursor-agent}"
CODEX_AGENT_BIN="${CODEX_AGENT_BIN:-codex}"
CLAUDE_AGENT_BIN="${CLAUDE_AGENT_BIN:-claude}"
CURSOR_GROK_MODEL="cursor-grok-4.5-high"
# 改訂3(2026-08-01): 検収者はmodel名でなく独立性条件で決める。既定はGrokのまま。
# 実装担当・order managerと同一identityのmodelはここに入っていても独立性gateで落ちる
ALLOWED_REVIEW_MODELS=("cursor-grok-4.5-high")
OPUS_MANAGER_MODEL="claude-opus-5"
SPARK_MODEL="gpt-5.3-codex-spark"
LOOP_PROFILE="opus-spark-grok"
SUPERVISOR_TIMEOUT_SECONDS="${CURSOR_SUPERVISED_TIMEOUT_SECONDS:-300}"
SPARK_TIMEOUT_SECONDS="${CODEX_SPARK_TIMEOUT_SECONDS:-1800}"
INSPECTION_TIMEOUT_SECONDS="${CURSOR_INSPECTION_TIMEOUT_SECONDS:-240}"
HEARTBEAT_SECONDS="${CURSOR_SUPERVISED_HEARTBEAT_SECONDS:-30}"
TERMINATION_GRACE_SECONDS="${CURSOR_TERMINATION_GRACE_SECONDS:-2}"
DERIVED_TARGET_ROOT_NAME="target"
DERIVED_TARGET_ENTRIES=(
  "scaffold-plugin-fixture"
  "new-plugin-scaffold-test"
  "d1i4-empty-classification.tsv"
)
MAX_TASK_BYTES=12288
MAX_ORDER_BYTES=32768
MAX_AUTHORITY_COUNT=4
MAX_ALLOWED_FILE_COUNT=8
MAX_READ_FILE_COUNT=12
MAX_READ_FILE_BYTES=131072
MAX_TARGET_COUNT=4
MAX_TARGET_ANCHOR_BYTES=240
MAX_TARGET_CAPSULE_BYTES=49152
MAX_SPARK_GRAIN_BYTES=16384
TARGET_CONTEXT_BEFORE=40
TARGET_CONTEXT_AFTER=80
MAX_OPUS_DELTA_FINDINGS=5
MAX_OPUS_DELTA_TEXT_BYTES=220
MAX_OPUS_DELTA_REASON_BYTES=300
SPARK_GRAIN_REQUIRED_LABELS=(
  Objective
  GRAIN
  ALLOWED_FILE
  CONTEXT_FACT
  READ_FILE
  INTERNAL_TARGET
  TEST_TARGET
  REUSE_TARGET
  NEW_SURFACE
  Non-goal
  STOP
  Test
)
OPUS_DELTA_SCHEMA='{"type":"object","additionalProperties":false,"properties":{"disposition":{"type":"string","enum":["READY","STOP","ESCALATE"]},"findings":{"type":"array","maxItems":5,"items":{"type":"object","additionalProperties":false,"properties":{"kind":{"type":"string","enum":["RISK","NEGATIVE_ORACLE","STOP","CORRECTION"]},"severity":{"type":"string","enum":["P0","P1","P2"]},"delta":{"type":"string","maxLength":220}},"required":["kind","severity","delta"]}},"reason":{"type":"string","maxLength":300}},"required":["disposition","findings","reason"]}'

usage() {
  echo "Usage: $0 prepare <isolated-worktree> <order-file> <task>"
  echo "       $0 execute <isolated-worktree> <approved-order-file> <task>"
  echo "       $0 inspect <isolated-worktree> <approved-order-file> <task>"
  echo "       printf '%s\n' <task> | $0 prepare|execute <isolated-worktree> <order-file>"
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
# gate発火台帳はevidence root直下に置き、prepare/execute/inspectを通して追記する
GATE_LOG="${ORDER_FILE}.evidence/gates.txt"
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
task_bytes="$(LC_ALL=C printf '%s' "$task" | wc -c | tr -d ' ')"
if (( task_bytes > MAX_TASK_BYTES )); then
  echo "delegate-cursor-supervised: taskが文脈予算 ${MAX_TASK_BYTES} bytesを超えています: $task_bytes" >&2
  exit 2
fi

for _ambient_git_var in GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_OBJECT_DIRECTORY \
  GIT_ALTERNATE_OBJECT_DIRECTORIES GIT_COMMON_DIR GIT_NAMESPACE GIT_CEILING_DIRECTORIES \
  GIT_DISCOVERY_ACROSS_FILESYSTEM; do
  if [[ -n "${!_ambient_git_var:-}" ]]; then
    echo "delegate-cursor-supervised: ambient Git addressing is forbidden: $_ambient_git_var" >&2
    exit 2
  fi
done

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
for value in "$SUPERVISOR_TIMEOUT_SECONDS" "$SPARK_TIMEOUT_SECONDS" "$INSPECTION_TIMEOUT_SECONDS" "$HEARTBEAT_SECONDS" "$TERMINATION_GRACE_SECONDS"; do
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
if ! command -v "$CLAUDE_AGENT_BIN" >/dev/null 2>&1; then
  echo "delegate-cursor-supervised: Claude Code CLI '$CLAUDE_AGENT_BIN' が見つかりません" >&2
  exit 127
fi
if ! command -v jq >/dev/null 2>&1; then
  echo "delegate-cursor-supervised: typed delta parser 'jq' が見つかりません" >&2
  exit 127
fi
if ! command -v uuidgen >/dev/null 2>&1; then
  echo "delegate-cursor-supervised: typed delta session ID generator 'uuidgen' が見つかりません" >&2
  exit 127
fi

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/motolii-cursor-supervised.XXXXXX")"
# checkpointはmodel出力ではなくparent(この script)だけが書く。EVIDENCE_ROOT_FOR_TRAPが
# 設定された後、CHECKPOINT_SETTLED=1でexitしない限り、EXIT trapがcheckpointを
# 無効化する。これにより「Spark後のどの経路で抜けても、明示的にpublish/invalidateを
# 済ませていない限りcheckpointは無効」という不変条件を経路網羅なしで保証する
EVIDENCE_ROOT_FOR_TRAP=""
CHECKPOINT_SETTLED=0
cleanup() {
  # EXIT trapから明示的に渡されたstatusを即時退避する。trap内で`$?`を再評価すると、
  # cleanup本体の最初のcommandや展開失敗経路がrunner本来のstatusを上書きし得る。
  # 後始末の成功を呼び出し側へ返さず、最後に元のstatusを明示してfail-closedを維持する
  local status="$1"
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
trap 'cleanup "$?"' EXIT

mark_checkpoint_at_risk() {
  EVIDENCE_ROOT_FOR_TRAP="$1"
  CHECKPOINT_SETTLED=0
}

# 監督ループのwall time・token・costをrunner自身が記録する。従来は外部modelの
# 出力を人が画面から書き写すしかなく、採用差分1件あたりの実コストも、REJECTで
# 捨てた分も機械で追えなかった(2026-08-01計装)。field名はevidence receipt限定で、
# OpenTelemetry GenAI規約が未安定(2026-07時点でDevelopment)のため恒久形式・
# 公開契約へは焼かない。測れないstageはUNKNOWNと明記し、欠測を沈黙させない。
TELEMETRY_VERSION=1
RUN_AGENT_WALL_SECONDS=0

# どのgateが実際に発火したかを残す。通過したgateは従来一切記録されず、
# 4,083行の監督機構のうち何が働いているかを実測で判定できなかった(2026-08-01)。
# gateは失敗時に直接exitするため、ENTERとPASSを対で書き、PASSが無い最後のENTERを
# 拒否したgateとして読む。GATE_LOGはargument解析直後に設定済み(ここで初期化しない)。

record_gate() {
  local phase="$1" name="$2"
  [[ -n "$GATE_LOG" ]] || return 0
  mkdir -p "$(dirname "$GATE_LOG")"
  printf 'GATE_%s: %s\n' "$phase" "$name" >>"$GATE_LOG"
}

gate_run() {
  local name="$1"
  shift
  record_gate ENTER "$name"
  "$@"
  record_gate PASS "$name"
}

# 文脈パッキング層が実際に切り詰めているかを測る。粒がそのまま収まるなら
# capsuleもcompiled grainも節約しておらず、この層は削れるという実測根拠になる。
record_context_economy() {
  local key="$1" produced="$2" baseline="$3"
  [[ -n "$GATE_LOG" ]] || return 0
  mkdir -p "$(dirname "$GATE_LOG")"
  printf 'ECONOMY: %s produced_bytes=%s baseline_bytes=%s\n' "$key" "$produced" "$baseline" >>"$GATE_LOG"
}

# capsuleを作らず`READ_FILE`を丸ごと渡した場合の入力量。capsuleとの差が節約分
read_file_total_bytes() {
  local order_file="$1" worktree="$2" total=0 path bytes
  while IFS= read -r path; do
    [[ -n "$path" && -f "$worktree/$path" ]] || continue
    bytes="$(LC_ALL=C wc -c <"$worktree/$path" | tr -d ' ')"
    total=$((total + bytes))
  done < <(awk '/^READ_FILE: /{sub(/^READ_FILE: /, ""); print}' "$order_file")
  printf '%s' "$total"
}

record_stage_telemetry() {
  local telemetry_file="$1" stage="$2" wall_seconds="$3" result_json="${4:-}"
  local input_tokens="UNKNOWN" output_tokens="UNKNOWN"
  local cache_read_tokens="UNKNOWN" cost_usd="UNKNOWN" api_ms="UNKNOWN"
  mkdir -p "$(dirname "$telemetry_file")"
  if [[ ! -f "$telemetry_file" ]]; then
    printf 'TELEMETRY_VERSION: %s\n' "$TELEMETRY_VERSION" >"$telemetry_file"
  fi
  # Claude CLIの`--output-format json`は既にusageとtotal_cost_usdを返している。
  # 従来はstructured_outputだけを読み、この計測値を捨てていた。
  if [[ -n "$result_json" && -f "$result_json" ]] && jq -e 'type == "object"' "$result_json" >/dev/null 2>&1; then
    # 三CLIでkey表記が違う。Claudeはsnake_case、Cursorはcamel、Codexはcached_input_tokens
    input_tokens="$(jq -r '.usage.input_tokens // .usage.inputTokens // "UNKNOWN"' "$result_json")"
    output_tokens="$(jq -r '.usage.output_tokens // .usage.outputTokens // "UNKNOWN"' "$result_json")"
    cache_read_tokens="$(jq -r '.usage.cache_read_input_tokens // .usage.cacheReadTokens // .usage.cached_input_tokens // "UNKNOWN"' "$result_json")"
    cost_usd="$(jq -r '.total_cost_usd // "UNKNOWN"' "$result_json")"
    api_ms="$(jq -r '.duration_api_ms // "UNKNOWN"' "$result_json")"
  fi
  {
    printf '%s_WALL_SECONDS: %s\n' "$stage" "$wall_seconds"
    printf '%s_INPUT_TOKENS: %s\n' "$stage" "$input_tokens"
    printf '%s_OUTPUT_TOKENS: %s\n' "$stage" "$output_tokens"
    printf '%s_CACHE_READ_TOKENS: %s\n' "$stage" "$cache_read_tokens"
    printf '%s_COST_USD: %s\n' "$stage" "$cost_usd"
    printf '%s_API_MS: %s\n' "$stage" "$api_ms"
  } >>"$telemetry_file"
  # 実行中に何も見えないという運用上の欠陥を、stage完了ごとの1行で塞ぐ
  echo "delegate-cursor-supervised: 計測 ${stage} wall=${wall_seconds}s tokens_in=${input_tokens} tokens_out=${output_tokens} cost_usd=${cost_usd}" >&2
}

# 試行全体の合計。UNKNOWNは合計へ足さず、欠測stage名をそのまま残す
summarize_telemetry() {
  local telemetry_file="$1"
  [[ -f "$telemetry_file" ]] || return 0
  awk '
    /_WALL_SECONDS: / { split($0, f, ": "); if (f[2] ~ /^[0-9]+$/) wall += f[2] }
    /_COST_USD: / { split($0, f, ": "); if (f[2] ~ /^[0-9.]+$/) cost += f[2]; else unknown_cost = unknown_cost " " f[1] }
    /_INPUT_TOKENS: / { split($0, f, ": "); if (f[2] ~ /^[0-9]+$/) tin += f[2]; else unmeasured = unmeasured " " f[1] }
    /_OUTPUT_TOKENS: / { split($0, f, ": "); if (f[2] ~ /^[0-9]+$/) tout += f[2] }
    END {
      printf "TOTAL_WALL_SECONDS: %d\n", wall
      printf "TOTAL_INPUT_TOKENS: %d\n", tin
      printf "TOTAL_OUTPUT_TOKENS: %d\n", tout
      printf "TOTAL_COST_USD: %.6f\n", cost
      printf "UNMEASURED_TOKEN_STAGES:%s\n", (unmeasured == "" ? " NONE" : unmeasured)
      printf "UNMEASURED_COST_STAGES:%s\n", (unknown_cost == "" ? " NONE" : unknown_cost)
    }
  ' "$telemetry_file" >>"$telemetry_file"
  echo "delegate-cursor-supervised: 計測合計 $(awk -F': ' '/^TOTAL_|^UNMEASURED_/ {printf "%s=%s ", $1, $2}' "$telemetry_file")" >&2
}

run_agent() {
  local output="$1"
  local timeout_seconds="$2"
  local start_epoch
  shift 2
  start_epoch="$(date +%s)"
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
  # timeout・失敗した実行も計測対象にする。捨てた時間こそ支配項だったため
  RUN_AGENT_WALL_SECONDS=$(( $(date +%s) - start_epoch ))
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

# 検収を`--output-format text`で受けていた間、thinking delta、usage、失敗理由が
# 全て捨てられ、空出力の原因を後から切り分けられなかった。stream-jsonへ切り替え、
# 生streamを証跡に残す。既存の`VERDICT:` marker契約は最終resultテキストの抽出で保つ
extract_supervisor_result() {
  local stream="$1" output="$2" result_json="$3"
  local plan_text="$stream.plan.txt" chat_text="$stream.chat.txt"
  : >"$output"
  : >"$result_json"
  [[ -f "$stream" ]] || return 0
  jq -c 'select(.type == "result")' "$stream" 2>/dev/null | tail -1 >"$result_json" || true
  # `--mode plan`の検収者は結論をchat messageでなくplan tool callへ書く。textモードが
  # これを捨てていたことが「Grokが60秒動いて空出力」の正体だった(2026-08-01実測)。
  # chat側にはpreambleしか残らないため、plan側を先に見る。
  jq -s -r '
    [.[] | select(.type == "tool_call") | .tool_call.createPlanToolCall.args.plan? // empty]
    | last // empty
  ' "$stream" 2>/dev/null >"$plan_text" || : >"$plan_text"
  : >"$chat_text"
  if [[ -s "$result_json" ]]; then
    jq -r '.result // empty' "$result_json" >"$chat_text" 2>/dev/null || : >"$chat_text"
  fi
  # markerを持つ側を採る。どちらにも無ければ非空の側を残し、原因を診断可能にする
  if [[ -s "$plan_text" ]] && grep -Eq '^(VERDICT|ORDER): ' "$plan_text"; then
    cat "$plan_text" >"$output"
  elif [[ -s "$chat_text" ]] && grep -Eq '^(VERDICT|ORDER): ' "$chat_text"; then
    cat "$chat_text" >"$output"
  elif [[ -s "$plan_text" ]]; then
    cat "$plan_text" >"$output"
  else
    cat "$chat_text" >"$output"
  fi
}

# 施工も`--json` JSONLで受け、生streamを証跡に残す。表示用のspark-stdout.txtへは
# agent messageだけを抽出し、usageはturn.completedを正本にする
extract_spark_result() {
  local attempt_dir="$1"
  local stream="$attempt_dir/spark-stream.jsonl"
  local output="$attempt_dir/spark-stdout.txt"
  local result_json="$attempt_dir/spark-result.json"
  : >"$output"
  : >"$result_json"
  [[ -f "$stream" ]] || return 0
  jq -r 'select(.type == "item.completed") | select(.item.type == "agent_message") | .item.text // empty' \
    "$stream" 2>/dev/null >"$output" || true
  jq -c 'select(.type == "turn.completed")' "$stream" 2>/dev/null | tail -1 >"$result_json" || true
}

run_supervisor() {
  local output="$1"
  local prompt="$2"
  local result_kind="$3"
  local timeout_seconds="${4:-$SUPERVISOR_TIMEOUT_SECONDS}"
  # 起動するmodelはorderの`REVIEW_MODEL`を正本にする。固定定数で起動すると、
  # 独立性gateが検査した宣言と実際に走るmodelが乖離しうる(2026-08-01独立検収P2指摘)
  local review_model="${5:-$CURSOR_GROK_MODEL}"
  # plan modeで編集を禁止しつつ、--forceでread-only shellの非対話実行だけを
  # 可能にする。fingerprint/scope検査は、CLI側のmode退行も検出する多層防御。
  local cursor_mode_args=(--trust --mode plan --force --sandbox enabled)
  local stream="${output%.txt}-stream.jsonl"
  local result_json="${output%.txt}-result.json"
  local agent_status=0
  run_agent "$stream" "$timeout_seconds" \
    env CURSOR_AGENT=1 "$CURSOR_AGENT_BIN" -p "${cursor_mode_args[@]}" \
      --output-format stream-json \
      --model "$review_model" \
      --workspace "$WORKTREE" \
      "$prompt" || agent_status=$?
  # 失敗・timeoutでも抽出する。空出力の理由はstreamにしか残らない
  extract_supervisor_result "$stream" "$output" "$result_json"
  if (( agent_status != 0 )); then
    return 1
  fi
  if ! result_is_valid "$output" "$result_kind"; then
    echo "delegate-cursor-supervised: Grokの結果markerが欠落・曖昧・末尾外です" >&2
    return 1
  fi
}

opus_delta_is_valid() {
  local output="$1"
  jq -e \
    --argjson max_findings "$MAX_OPUS_DELTA_FINDINGS" \
    --argjson max_delta "$MAX_OPUS_DELTA_TEXT_BYTES" \
    --argjson max_reason "$MAX_OPUS_DELTA_REASON_BYTES" '
      .type == "result" and
      .subtype == "success" and
      (.structured_output | type) == "object" and
      (.structured_output as $d |
        (($d | keys | sort) == ["disposition", "findings", "reason"]) and
        ($d.disposition == "READY" or $d.disposition == "STOP" or $d.disposition == "ESCALATE") and
        (($d.findings | type) == "array") and
        (($d.findings | length) <= $max_findings) and
        all($d.findings[];
          ((keys | sort) == ["delta", "kind", "severity"]) and
          (.kind == "RISK" or .kind == "NEGATIVE_ORACLE" or .kind == "STOP" or .kind == "CORRECTION") and
          (.severity == "P0" or .severity == "P1" or .severity == "P2") and
          ((.delta | type) == "string") and
          ((.delta | utf8bytelength) >= 1) and
          ((.delta | utf8bytelength) <= $max_delta) and
          ((.delta | test("[\\r\\n]")) | not)
        ) and
        (($d.reason | type) == "string") and
        (($d.reason | utf8bytelength) >= 1) and
        (($d.reason | utf8bytelength) <= $max_reason) and
        (($d.reason | test("[\\r\\n]")) | not)
      )
    ' "$output" >/dev/null 2>&1
}

run_order_manager_delta() {
  local output="$1"
  local prompt="$2"
  local timeout_seconds="${3:-$SUPERVISOR_TIMEOUT_SECONDS}"
  local session_id retry_prompt
  session_id="$(uuidgen | tr '[:upper:]' '[:lower:]')"
  if ! run_agent "$output" "$timeout_seconds" \
    env CLAUDE_DELEGATED=1 "$CLAUDE_AGENT_BIN" -p \
      --model "$OPUS_MANAGER_MODEL" \
      --effort low \
      --permission-mode default \
      --tools "" \
      --session-id "$session_id" \
      --json-schema "$OPUS_DELTA_SCHEMA" \
      --output-format json \
      "$prompt"; then
    return 1
  fi
  if opus_delta_is_valid "$output"; then
    return 0
  fi

  echo "delegate-cursor-supervised: Opus typed deltaがschema不正。same-sessionで一回だけ差戻します" >&2
  retry_prompt="Your previous response was not a valid typed delta. Return only one JSON object matching the same supplied schema. Do not add prose, markdown, tools, or a full order."
  if run_agent "$output.retry" "$timeout_seconds" \
    env CLAUDE_DELEGATED=1 "$CLAUDE_AGENT_BIN" -p \
      --model "$OPUS_MANAGER_MODEL" \
      --effort low \
      --permission-mode default \
      --tools "" \
      --resume "$session_id" \
      --json-schema "$OPUS_DELTA_SCHEMA" \
      --output-format json \
      "$retry_prompt" &&
      opus_delta_is_valid "$output.retry"; then
    mv "$output.retry" "$output"
    return 0
  fi
  echo "delegate-cursor-supervised: Opus typed deltaが二回ともschema不正。散文fallbackしません" >&2
  return 1
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

git_explicit() {
  local worktree="$1"
  shift
  local git_dir
  git_dir="$(git -C "$worktree" rev-parse --absolute-git-dir)"
  env -u GIT_DIR -u GIT_WORK_TREE -u GIT_INDEX_FILE -u GIT_OBJECT_DIRECTORY \
    -u GIT_ALTERNATE_OBJECT_DIRECTORIES -u GIT_COMMON_DIR -u GIT_NAMESPACE \
    -u GIT_CEILING_DIRECTORIES -u GIT_DISCOVERY_ACROSS_FILESYSTEM \
    git --git-dir="$git_dir" --work-tree="$worktree" "$@"
}

compute_ref_digest() {
  local worktree="$1"
  local head_file show_ref_file sorted_file body_file show_ref_status
  head_file="$(mktemp "$tmp_dir/ref-head.XXXXXX")"
  show_ref_file="$(mktemp "$tmp_dir/ref-show-ref.XXXXXX")"
  sorted_file="$(mktemp "$tmp_dir/ref-sorted.XXXXXX")"
  body_file="$(mktemp "$tmp_dir/ref-body.XXXXXX")"
  if ! git_explicit "$worktree" rev-parse HEAD >"$head_file"; then
    echo "delegate-cursor-supervised: ref digest HEAD capture failed" >&2
    exit 2
  fi
  set +e
  git_explicit "$worktree" show-ref >"$show_ref_file" 2>/dev/null
  show_ref_status=$?
  set -e
  if (( show_ref_status != 0 )); then
    if (( show_ref_status == 1 )) && [[ ! -s "$show_ref_file" ]]; then
      :
    else
      echo "delegate-cursor-supervised: ref digest show-ref capture failed (status $show_ref_status)" >&2
      exit 2
    fi
  fi
  cat "$head_file" "$show_ref_file" >"$body_file"
  LC_ALL=C sort -o "$sorted_file" "$body_file"
  shasum -a 256 "$sorted_file" | awk '{print $1}'
}

derived_target_closure_fail() {
  local stage="$1"
  local msg="SCOPE NG: derived target closure ($stage): $2"
  echo "$msg" >&2
  [[ -z "${CURRENT_ATTEMPT_DIR:-}" ]] || printf '%s\n' "$msg" >>"$CURRENT_ATTEMPT_DIR/stage-result.txt" 2>/dev/null || true
  exit 7
}

derived_target_entry_is_known() {
  local name="$1" known
  for known in "${DERIVED_TARGET_ENTRIES[@]}"; do
    if [[ "$name" == "$known" ]]; then
      return 0
    fi
  done
  return 1
}

derived_target_entry_must_be_directory() {
  local name="$1"
  [[ "$name" == "scaffold-plugin-fixture" || "$name" == "new-plugin-scaffold-test" ]]
}

enforce_derived_target_closure() {
  local worktree="$1" stage_label="$2"
  local root="$worktree/$DERIVED_TARGET_ROOT_NAME"
  local find_enum_file children=() i child base tracked_file desc_file path

  if [[ ! -e "$root" && ! -L "$root" ]]; then
    return 0
  fi
  if [[ -L "$root" ]]; then
    derived_target_closure_fail "$stage_label" "target root is a symlink: $root"
  fi
  if [[ ! -d "$root" ]]; then
    derived_target_closure_fail "$stage_label" "target root is not a directory: $root"
  fi

  find_enum_file="$(mktemp "$tmp_dir/target-enum.XXXXXX")"
  if ! find "$root" -mindepth 1 -maxdepth 1 -print0 >"$find_enum_file"; then
    derived_target_closure_fail "$stage_label" "failed to enumerate target root children"
  fi
  while IFS= read -r -d '' child || [[ -n "${child:-}" ]]; do
    [[ -z "$child" ]] && continue
    children+=("$child")
  done <"$find_enum_file"

  for (( i = 0; i < ${#children[@]}; i++ )); do
    child="${children[i]}"
    if [[ "$(dirname "$child")" != "$root" ]]; then
      derived_target_closure_fail "$stage_label" "path escape: $child"
    fi
    base="$(basename "$child")"
    if [[ -z "$base" ]]; then
      derived_target_closure_fail "$stage_label" "empty basename under target root"
    fi
    if ! derived_target_entry_is_known "$base"; then
      derived_target_closure_fail "$stage_label" "unknown target entry: $base"
    fi
    if [[ -L "$child" ]]; then
      derived_target_closure_fail "$stage_label" "symlink child: $child"
    fi
    if derived_target_entry_must_be_directory "$base"; then
      if [[ ! -d "$child" ]]; then
        derived_target_closure_fail "$stage_label" "expected directory: $child"
      fi
    else
      if [[ ! -f "$child" ]]; then
        derived_target_closure_fail "$stage_label" "expected regular file: $child"
      fi
    fi
    tracked_file="$(mktemp "$tmp_dir/target-tracked.XXXXXX")"
    if ! git_explicit "$worktree" ls-files -z -- "target/$base" >"$tracked_file"; then
      derived_target_closure_fail "$stage_label" "git ls-files failed for target/$base"
    fi
    if [[ -s "$tracked_file" ]]; then
      derived_target_closure_fail "$stage_label" "tracked derived entry: target/$base"
    fi
    if [[ -d "$child" ]]; then
      desc_file="$(mktemp "$tmp_dir/target-desc.XXXXXX")"
      if ! find "$child" -mindepth 1 -print0 >"$desc_file"; then
        derived_target_closure_fail "$stage_label" "failed to enumerate descendants of $child"
      fi
      while IFS= read -r -d '' path || [[ -n "${path:-}" ]]; do
        [[ -z "$path" ]] && continue
        case "$path" in
          "$child"/*) ;;
          *) derived_target_closure_fail "$stage_label" "path escape: $path" ;;
        esac
        if [[ -L "$path" ]]; then
          derived_target_closure_fail "$stage_label" "nested symlink: $path"
        fi
        if [[ -d "$path" ]]; then
          continue
        fi
        if [[ -f "$path" ]]; then
          continue
        fi
        derived_target_closure_fail "$stage_label" "unexpected node type: $path"
      done <"$desc_file"
    fi
  done

  if [[ ! -w "$root" ]]; then
    derived_target_closure_fail "$stage_label" "removal failed: target root is not writable"
  fi

  for (( i = 0; i < ${#children[@]}; i++ )); do
    child="${children[i]}"
    if [[ -z "$child" ]]; then
      derived_target_closure_fail "$stage_label" "empty removal path"
    fi
    if [[ "$child" == "$root" || "$child" == "$root/" ]]; then
      derived_target_closure_fail "$stage_label" "refusing to remove target root as child: $child"
    fi
    if [[ -z "$(basename "$child")" ]]; then
      derived_target_closure_fail "$stage_label" "empty basename for removal: $child"
    fi
    if [[ -d "$child" && ! -L "$child" ]]; then
      if ! rm -rf -- "$child"; then
        derived_target_closure_fail "$stage_label" "removal failed: $child"
      fi
    elif [[ -f "$child" && ! -L "$child" ]]; then
      if ! rm -f -- "$child"; then
        derived_target_closure_fail "$stage_label" "removal failed: $child"
      fi
    else
      derived_target_closure_fail "$stage_label" "unexpected child type at removal: $child"
    fi
  done

  if ! rmdir "$root"; then
    derived_target_closure_fail "$stage_label" "rmdir failed for target root: $root"
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
  local worktree="$1" rel_path="$2" label="${3:-path}"
  local cur="$worktree" part
  local old_ifs="$IFS"
  IFS='/'
  for part in $rel_path; do
    IFS="$old_ifs"
    cur="$cur/$part"
    if [[ -L "$cur" ]]; then
      gate_fail "$label path is a symlink: $rel_path"
    fi
    IFS='/'
  done
  IFS="$old_ifs"
}

gate_ledger_row_state() {
  local ledger="$1" section="$2" id="$3" id_column="$4" state_column="$5"
  awk -v section="$section" -v id="$id" -v id_column="$id_column" -v state_column="$state_column" '
    BEGIN { in_section = 0; count = 0 }
    $0 == "## " section { in_section = 1; next }
    in_section && /^##+ / { in_section = 0 }
    in_section && /^\|/ {
      n = split($0, f, "|")
      if (n <= id_column || n <= state_column) next
      gsub(/^[ \t]+|[ \t]+$/, "", f[id_column])
      if (f[id_column] ~ /^-+$/) next
      if (f[id_column] == id) {
        state = f[state_column]
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
  grain_state="$(gate_ledger_row_state "$ledger" "現在の並列レーン" "$grain" 3 5)"
  case "$grain_state" in
    ABSENT) gate_fail "$grain not found in current-lane ledger" ;;
    AMBIGUOUS) gate_fail "$grain has ambiguous current-lane ledger rows" ;;
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
    dep_state="$(gate_ledger_row_state "$ledger" "発注依存証跡" "$dep_id" 2 3)"
    case "$dep_state" in
      ABSENT) gate_fail "dependency $dep_id not found in dependency-evidence ledger" ;;
      AMBIGUOUS) gate_fail "dependency $dep_id has ambiguous dependency-evidence ledger rows" ;;
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
    gate_reject_symlink_components "$worktree" "$GATE_AUTH_PATH" "AUTHORITY"
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
    if [[ "${af%%/*}" == "target" ]]; then
      gate_fail "ALLOWED_FILE covers derived target output: $af"
    fi
    GATE_ALLOWED_FILES+=("$af")
  done <<<"$allowed_lines"
}

# 外部modelへ同じrepo正本を三重に読ませないため、通常orderを
# Codex検証済みの小さな文脈カプセルへ機械的に閉じる。
gate_check_context_budget() {
  local order_file="$1" worktree="$2"
  local order_bytes read_mode_count fact_count authority_count allowed_count read_count
  local read_lines line read_path read_full read_bytes=0 duplicate_paths

  order_bytes="$(LC_ALL=C wc -c <"$order_file" | tr -d ' ')"
  if (( order_bytes > MAX_ORDER_BYTES )); then
    gate_fail "order exceeds ${MAX_ORDER_BYTES}-byte context budget: $order_bytes"
  fi

  read_mode_count="$(grep -c '^READ_MODE: CAPSULE$' "$order_file" || true)"
  [[ "$read_mode_count" -eq 1 ]] || gate_fail "READ_MODE: CAPSULE must appear exactly once"
  fact_count="$(grep -c '^CONTEXT_FACT: [^[:space:]]' "$order_file" || true)"
  (( fact_count >= 1 )) || gate_fail "missing non-empty CONTEXT_FACT"

  authority_count="$(grep -c '^AUTHORITY:' "$order_file" || true)"
  allowed_count="$(grep -c '^ALLOWED_FILE:' "$order_file" || true)"
  read_count="$(grep -c '^READ_FILE:' "$order_file" || true)"
  (( authority_count >= 1 && authority_count <= MAX_AUTHORITY_COUNT )) \
    || gate_fail "AUTHORITY count must be 1..$MAX_AUTHORITY_COUNT: $authority_count"
  (( allowed_count >= 1 && allowed_count <= MAX_ALLOWED_FILE_COUNT )) \
    || gate_fail "ALLOWED_FILE count must be 1..$MAX_ALLOWED_FILE_COUNT: $allowed_count"
  (( read_count >= 1 && read_count <= MAX_READ_FILE_COUNT )) \
    || gate_fail "READ_FILE count must be 1..$MAX_READ_FILE_COUNT: $read_count"

  read_lines="$(grep -E '^READ_FILE:' "$order_file" || true)"
  duplicate_paths="$(printf '%s\n' "$read_lines" | sort | uniq -d)"
  [[ -z "$duplicate_paths" ]] || gate_fail "READ_FILE duplicated: ${duplicate_paths#READ_FILE: }"
  while IFS= read -r line; do
    if [[ ! "$line" =~ ^READ_FILE:\ ([^[:space:]]+)$ ]]; then
      gate_fail "READ_FILE must be one exact relative path: ${line#READ_FILE: }"
    fi
    read_path="${BASH_REMATCH[1]}"
    if [[ "$read_path" == /* || "$read_path" == *".."* || "$read_path" == *[\*\?\[\]\{\}]* ]]; then
      gate_fail "READ_FILE unsafe or non-exact path: $read_path"
    fi
    gate_reject_symlink_components "$worktree" "$read_path" "READ_FILE"
    read_full="$worktree/$read_path"
    [[ -f "$read_full" ]] || gate_fail "READ_FILE missing: $read_path"
    read_bytes=$((read_bytes + $(LC_ALL=C wc -c <"$read_full" | tr -d ' ')))
  done <<<"$read_lines"
  (( read_bytes <= MAX_READ_FILE_BYTES )) \
    || gate_fail "READ_FILE bytes exceed $MAX_READ_FILE_BYTES: $read_bytes"
}

gate_parse_target_line() {
  local line="$1" label="$2"
  if [[ ! "$line" =~ ^${label}:\ ([^[:space:]]+)\ ::\ (.+)$ ]]; then
    gate_fail "$label must be '<path> :: <unique trimmed line>'"
  fi
  GATE_TARGET_PATH="${BASH_REMATCH[1]}"
  GATE_TARGET_ANCHOR="${BASH_REMATCH[2]}"
  if [[ "$GATE_TARGET_PATH" == /* || "$GATE_TARGET_PATH" == *".."* || \
        "$GATE_TARGET_PATH" == *[\*\?\[\]\{\}]* ]]; then
    gate_fail "$label unsafe path: $GATE_TARGET_PATH"
  fi
  if [[ "$GATE_TARGET_ANCHOR" =~ ^[[:space:]] || "$GATE_TARGET_ANCHOR" =~ [[:space:]]$ || \
        "$GATE_TARGET_ANCHOR" == *$'\t'* ]]; then
    gate_fail "$label anchor must be one trimmed line without tabs"
  fi
  local anchor_bytes
  anchor_bytes="$(LC_ALL=C printf '%s' "$GATE_TARGET_ANCHOR" | wc -c | tr -d ' ')"
  (( anchor_bytes >= 1 && anchor_bytes <= MAX_TARGET_ANCHOR_BYTES )) \
    || gate_fail "$label anchor bytes must be 1..$MAX_TARGET_ANCHOR_BYTES: $anchor_bytes"
}

target_anchor_line_number() {
  local full_path="$1" anchor="$2"
  awk -v needle="$anchor" '
    {
      line = $0
      sub(/^[ \t]+/, "", line)
      sub(/[ \t]+$/, "", line)
      if (line == needle) {
        count++
        line_number = NR
      }
    }
    END {
      if (count == 1) print line_number
      else if (count == 0) print "ABSENT"
      else print "AMBIGUOUS"
    }
  ' "$full_path"
}

gate_check_exact_targets() {
  local order_file="$1" worktree="$2" source_mode="${3:-WORKTREE}"
  local label lines count line line_number read_paths new_surface_count
  local target_source target_tmp base_sha mode

  read_paths="$(grep -E '^READ_FILE:' "$order_file" | sed 's/^READ_FILE: //')"
  if [[ "$source_mode" == "BASE" ]]; then
    base_sha="$(gate_require_single_field "$order_file" "BASE_SHA")"
  elif [[ "$source_mode" != "WORKTREE" ]]; then
    gate_fail "unknown exact-target source mode: $source_mode"
  fi
  for label in INTERNAL_TARGET TEST_TARGET REUSE_TARGET; do
    lines="$(grep -E "^${label}:" "$order_file" || true)"
    count="$(printf '%s\n' "$lines" | grep -c "^${label}:" || true)"
    (( count >= 1 && count <= MAX_TARGET_COUNT )) \
      || gate_fail "$label count must be 1..$MAX_TARGET_COUNT: $count"
    while IFS= read -r line; do
      gate_parse_target_line "$line" "$label"
      printf '%s\n' "$read_paths" | grep -Fqx -- "$GATE_TARGET_PATH" \
        || gate_fail "$label path is not a READ_FILE: $GATE_TARGET_PATH"
      if [[ "$source_mode" == "WORKTREE" ]]; then
        gate_reject_symlink_components "$worktree" "$GATE_TARGET_PATH" "$label"
        [[ -f "$worktree/$GATE_TARGET_PATH" ]] \
          || gate_fail "$label file missing: $GATE_TARGET_PATH"
      fi
      if [[ "$label" == "INTERNAL_TARGET" ]] && ! path_is_allowed "$GATE_TARGET_PATH"; then
        gate_fail "INTERNAL_TARGET is outside ALLOWED_FILE: $GATE_TARGET_PATH"
      fi
      target_source="$worktree/$GATE_TARGET_PATH"
      target_tmp=""
      if [[ "$source_mode" == "BASE" ]]; then
        mode="$(git -C "$worktree" ls-tree "$base_sha" -- "$GATE_TARGET_PATH" | awk '{print $1}')"
        [[ -n "$mode" ]] || gate_fail "$label file missing at BASE_SHA: $GATE_TARGET_PATH"
        [[ "$mode" != "120000" ]] || gate_fail "$label path is a symlink at BASE_SHA: $GATE_TARGET_PATH"
        target_tmp="$(mktemp "$tmp_dir/motolii-target-base.XXXXXX")"
        if ! git -C "$worktree" show "${base_sha}:${GATE_TARGET_PATH}" >"$target_tmp"; then
          gate_fail "$label cannot read BASE_SHA file: $GATE_TARGET_PATH"
        fi
        target_source="$target_tmp"
      fi
      line_number="$(target_anchor_line_number "$target_source" "$GATE_TARGET_ANCHOR")"
      [[ -z "$target_tmp" ]] || rm -f "$target_tmp"
      case "$line_number" in
        ABSENT) gate_fail "$label anchor absent in $GATE_TARGET_PATH: $GATE_TARGET_ANCHOR" ;;
        AMBIGUOUS) gate_fail "$label anchor is not unique in $GATE_TARGET_PATH: $GATE_TARGET_ANCHOR" ;;
      esac
    done <<<"$lines"
  done

  new_surface_count="$(grep -c '^NEW_SURFACE: FORBIDDEN$' "$order_file" || true)"
  [[ "$new_surface_count" -eq 1 ]] \
    || gate_fail "NEW_SURFACE: FORBIDDEN must appear exactly once"
}

build_target_capsule() {
  local order_file="$1" worktree="$2" output="$3"
  local label line line_number start end key seen_file capsule_bytes

  seen_file="$(mktemp "$tmp_dir/motolii-target-capsule-seen.XXXXXX")"
  : >"$output"
  for label in INTERNAL_TARGET TEST_TARGET REUSE_TARGET; do
    while IFS= read -r line; do
      gate_parse_target_line "$line" "$label"
      key="$GATE_TARGET_PATH :: $GATE_TARGET_ANCHOR"
      if grep -Fqx -- "$key" "$seen_file"; then
        continue
      fi
      printf '%s\n' "$key" >>"$seen_file"
      line_number="$(target_anchor_line_number "$worktree/$GATE_TARGET_PATH" "$GATE_TARGET_ANCHOR")"
      start=$((line_number - TARGET_CONTEXT_BEFORE))
      (( start >= 1 )) || start=1
      end=$((line_number + TARGET_CONTEXT_AFTER))
      {
        printf '=== %s line %s anchor: %s ===\n' "$GATE_TARGET_PATH" "$line_number" "$GATE_TARGET_ANCHOR"
        sed -n "${start},${end}p" "$worktree/$GATE_TARGET_PATH"
        echo
      } >>"$output"
    done < <(grep -E "^${label}:" "$order_file")
  done
  rm -f "$seen_file"
  capsule_bytes="$(LC_ALL=C wc -c <"$output" | tr -d ' ')"
  record_context_economy TARGET_CAPSULE "$capsule_bytes" "$(read_file_total_bytes "$order_file" "$worktree")"
  (( capsule_bytes <= MAX_TARGET_CAPSULE_BYTES )) \
    || gate_fail "target capsule exceeds $MAX_TARGET_CAPSULE_BYTES bytes: $capsule_bytes"
}

gate_check_compiled_grain_contract() {
  local order_file="$1" label count
  for label in "${SPARK_GRAIN_REQUIRED_LABELS[@]}"; do
    count="$(grep -c "^${label}:" "$order_file" || true)"
    (( count >= 1 )) || gate_fail "order missing compiled Spark grain field: $label"
  done
}

# 承認済みorderはCodex/Grokの検証正本として保持する一方、Sparkには施工判断へ
# 不要なprovenance、閉鎖判定、model routingを再送しない。未知の製品固有ラベルや
# 自由記述は勝手に捨てず、runner所有の既知metadataだけを除く決定的な射影にする。
build_compiled_spark_grain() {
  local order_file="$1" output="$2"
  local label expected_count actual_count grain_bytes

  awk '
    BEGIN { print "SPARK_GRAIN_VERSION: 1" }
    /^(BASE_REF|BASE_SHA|DEPENDENCY|AUTHORITY|AUTHORITY_SPAN|OWNER_CLOSURE|CAUSE_CLOSURE|CONTRACT_IMPACT|CONTRACT_CLOSURE|CONTRACT_AUTHORITY|ORACLE_CLOSURE|REUSE_CLOSURE|VIEW_PROFILE|HAZARD_TAG|READ_MODE|ORDER|LOOP_PROFILE|ORDER_MANAGER_MODEL|IMPLEMENTER_MODEL|REVIEW_MODEL|TASK_SHA256|CODEX PRECHECK|OPUS_DELTA_REASON):/ {
      next
    }
    { print }
  ' "$order_file" >"$output"

  [[ -s "$output" ]] || gate_fail "compiled Spark grain is empty"
  if grep -Eq '^(BASE_REF|BASE_SHA|DEPENDENCY|AUTHORITY|AUTHORITY_SPAN|OWNER_CLOSURE|CAUSE_CLOSURE|CONTRACT_IMPACT|CONTRACT_CLOSURE|CONTRACT_AUTHORITY|ORACLE_CLOSURE|REUSE_CLOSURE|VIEW_PROFILE|HAZARD_TAG|READ_MODE|ORDER|LOOP_PROFILE|ORDER_MANAGER_MODEL|IMPLEMENTER_MODEL|REVIEW_MODEL|TASK_SHA256|CODEX PRECHECK|OPUS_DELTA_REASON):' "$output"; then
    gate_fail "compiled Spark grain leaked runner-only metadata"
  fi
  [[ "$(grep -c '^SPARK_GRAIN_VERSION: 1$' "$output" || true)" -eq 1 ]] \
    || gate_fail "compiled Spark grain version must appear exactly once"

  # 施工に必要な閉集合は元orderと同数でなければならない。生成器の変更で一行でも
  # 落ちた場合、短いが不完全なpromptをSparkへ送らずmodel起動前に失敗させる。
  for label in "${SPARK_GRAIN_REQUIRED_LABELS[@]}"; do
    expected_count="$(grep -c "^${label}:" "$order_file" || true)"
    actual_count="$(grep -c "^${label}:" "$output" || true)"
    (( expected_count >= 1 )) || gate_fail "order missing compiled Spark grain field: $label"
    [[ "$actual_count" -eq "$expected_count" ]] \
      || gate_fail "compiled Spark grain field count mismatch: $label"
  done

  grain_bytes="$(LC_ALL=C wc -c <"$output" | tr -d ' ')"
  record_context_economy COMPILED_GRAIN "$grain_bytes" \
    "$(LC_ALL=C wc -c <"$order_file" | tr -d ' ')"
  (( grain_bytes <= MAX_SPARK_GRAIN_BYTES )) \
    || gate_fail "compiled Spark grain exceeds $MAX_SPARK_GRAIN_BYTES bytes: $grain_bytes"
}

gate_check_view_profile() {
  local order_file="$1"
  local authority_span owner_closure cause_closure contract_impact contract_closure
  local contract_authority contract_authority_receipt
  local oracle_closure reuse_closure declared_profile computed_profile hazard_tag

  authority_span="$(gate_require_single_field "$order_file" "AUTHORITY_SPAN")"
  owner_closure="$(gate_require_single_field "$order_file" "OWNER_CLOSURE")"
  cause_closure="$(gate_require_single_field "$order_file" "CAUSE_CLOSURE")"
  contract_impact="$(gate_require_single_field "$order_file" "CONTRACT_IMPACT")"
  contract_closure="$(gate_require_single_field "$order_file" "CONTRACT_CLOSURE")"
  contract_authority="$(gate_require_single_field "$order_file" "CONTRACT_AUTHORITY")"
  oracle_closure="$(gate_require_single_field "$order_file" "ORACLE_CLOSURE")"
  reuse_closure="$(gate_require_single_field "$order_file" "REUSE_CLOSURE")"
  declared_profile="$(gate_require_single_field "$order_file" "VIEW_PROFILE")"
  hazard_tag="$(gate_require_single_field "$order_file" "HAZARD_TAG")"

  [[ "$authority_span" == "ONE" || "$authority_span" == "MULTIPLE" || "$authority_span" == "CONFLICTING" ]] \
    || gate_fail "invalid AUTHORITY_SPAN: $authority_span"
  [[ "$owner_closure" == "CLOSED" || "$owner_closure" == "MULTIPLE_KNOWN" || "$owner_closure" == "UNKNOWN" ]] \
    || gate_fail "invalid OWNER_CLOSURE: $owner_closure"
  [[ "$cause_closure" == "LOCALIZED" || "$cause_closure" == "COMPETING" || "$cause_closure" == "UNKNOWN" ]] \
    || gate_fail "invalid CAUSE_CLOSURE: $cause_closure"
  [[ "$contract_impact" == "PRIVATE" || "$contract_impact" == "SHARED" || "$contract_impact" == "PERMANENT" ]] \
    || gate_fail "invalid CONTRACT_IMPACT: $contract_impact"
  [[ "$contract_closure" == "PRIVATE" || "$contract_closure" == "FROZEN" || \
     "$contract_closure" == "UNRESOLVED" ]] \
    || gate_fail "invalid CONTRACT_CLOSURE: $contract_closure"
  [[ "$oracle_closure" == "CLOSED" || "$oracle_closure" == "PARTIAL" || "$oracle_closure" == "ABSENT" ]] \
    || gate_fail "invalid ORACLE_CLOSURE: $oracle_closure"
  [[ "$reuse_closure" == "REUSE" || "$reuse_closure" == "CHOICE" || "$reuse_closure" == "NEW" ]] \
    || gate_fail "invalid REUSE_CLOSURE: $reuse_closure"
  [[ "$declared_profile" == "CLOSED" || "$declared_profile" == "ADJACENT" || "$declared_profile" == "WIDE" ]] \
    || gate_fail "invalid VIEW_PROFILE: $declared_profile"
  [[ "$hazard_tag" == "DESTRUCTIVE_FS" || "$hazard_tag" == "SECURITY" || \
     "$hazard_tag" == "PERSISTENCE" || "$hazard_tag" == "CONCURRENCY" || \
     "$hazard_tag" == "PLATFORM" || "$hazard_tag" == "NONE" ]] \
    || gate_fail "invalid HAZARD_TAG: $hazard_tag"

  case "$contract_impact:$contract_closure" in
    PRIVATE:PRIVATE)
      [[ "$contract_authority" == "NONE" ]] \
        || gate_fail "private contract must use CONTRACT_AUTHORITY: NONE"
      ;;
    SHARED:FROZEN|PERMANENT:FROZEN)
      [[ "$contract_authority" != "NONE" ]] \
        || gate_fail "frozen $contract_impact contract requires CONTRACT_AUTHORITY receipt"
      [[ "$contract_authority" =~ ^([^[:space:]@]+)@SHA256:([0-9a-f]{64})$ ]] \
        || gate_fail "CONTRACT_AUTHORITY must be '<path>@SHA256:<64 lowercase hex>'"
      contract_authority_receipt="${BASH_REMATCH[1]} SHA256:${BASH_REMATCH[2]}"
      grep -Fqx -- "AUTHORITY: $contract_authority_receipt" "$order_file" \
        || gate_fail "CONTRACT_AUTHORITY is not an exact verified AUTHORITY: $contract_authority"
      ;;
    SHARED:UNRESOLVED|PERMANENT:UNRESOLVED)
      [[ "$contract_authority" == "NONE" ]] \
        || gate_fail "unresolved contract must use CONTRACT_AUTHORITY: NONE"
      ;;
    *)
      gate_fail "invalid contract impact/closure pair: $contract_impact/$contract_closure"
      ;;
  esac

  if [[ "$authority_span" == "CONFLICTING" || "$owner_closure" == "UNKNOWN" || \
        "$cause_closure" == "COMPETING" || "$cause_closure" == "UNKNOWN" || \
        "$contract_closure" == "UNRESOLVED" || \
        "$oracle_closure" == "ABSENT" || "$reuse_closure" == "NEW" ]]; then
    computed_profile="WIDE"
  elif [[ "$authority_span" == "ONE" && "$owner_closure" == "CLOSED" && \
          "$cause_closure" == "LOCALIZED" && \
          ( "$contract_closure" == "PRIVATE" || "$contract_closure" == "FROZEN" ) && \
          "$oracle_closure" == "CLOSED" && "$reuse_closure" == "REUSE" ]]; then
    computed_profile="CLOSED"
  else
    computed_profile="ADJACENT"
  fi

  [[ "$declared_profile" == "$computed_profile" ]] \
    || gate_fail "VIEW_PROFILE mismatch: declared=$declared_profile computed=$computed_profile"
  [[ "$computed_profile" != "WIDE" ]] \
    || gate_fail "VIEW_PROFILE WIDE requires read-only exploration and cannot dispatch Spark"
}

prepare_skeleton_reject_reserved_fields() {
  local skeleton_file="$1"
  if grep -Eq '^(ORDER:|CODEX PRECHECK:|LOOP_PROFILE:|ORDER_MANAGER_MODEL:|IMPLEMENTER_MODEL:|REVIEW_MODEL:|TASK_SHA256:|SPARK_GRAIN_VERSION:|OPUS_DELTA_|DELTA_RESOLUTION:|HAZARD_GUARD:|MECHANICAL_GUARD:)' "$skeleton_file"; then
    gate_fail "prepare skeleton contains runner-owned field"
  fi
}

gate_check_delta_resolutions() {
  local order_file="$1"
  local finding_lines resolution_lines line finding_id resolution_id
  local finding_ids="" resolution_ids="" count

  finding_lines="$(grep -E '^OPUS_DELTA_FINDING:' "$order_file" || true)"
  resolution_lines="$(grep -E '^DELTA_RESOLUTION:' "$order_file" || true)"
  while IFS= read -r line; do
    [[ -n "$line" ]] || continue
    if [[ ! "$line" =~ ^OPUS_DELTA_FINDING:\ (F[1-5])\ (RISK|NEGATIVE_ORACLE|STOP|CORRECTION)\ (P0|P1|P2)\ (.+)$ ]]; then
      gate_fail "OPUS_DELTA_FINDING malformed"
    fi
    finding_id="${BASH_REMATCH[1]}"
    finding_ids+="${finding_id}"$'\n'
  done <<<"$finding_lines"

  while IFS= read -r line; do
    [[ -n "$line" ]] || continue
    if [[ ! "$line" =~ ^DELTA_RESOLUTION:\ (F[1-5])\ (.+)$ ]]; then
      gate_fail "DELTA_RESOLUTION malformed"
    fi
    resolution_id="${BASH_REMATCH[1]}"
    resolution_ids+="${resolution_id}"$'\n'
  done <<<"$resolution_lines"

  count="$(printf '%s' "$finding_ids" | grep -c '^F' || true)"
  if (( count > MAX_OPUS_DELTA_FINDINGS )); then
    gate_fail "too many OPUS_DELTA_FINDING entries: $count"
  fi
  [[ -z "$finding_ids" || -z "$(printf '%s' "$finding_ids" | sort | uniq -d)" ]] \
    || gate_fail "duplicate OPUS_DELTA_FINDING ID"
  [[ -z "$resolution_ids" || -z "$(printf '%s' "$resolution_ids" | sort | uniq -d)" ]] \
    || gate_fail "duplicate DELTA_RESOLUTION ID"
  if [[ "$(printf '%s' "$finding_ids" | sort)" != "$(printf '%s' "$resolution_ids" | sort)" ]]; then
    gate_fail "every Opus delta finding requires one matching DELTA_RESOLUTION before precheck"
  fi
}

append_hazard_guard() {
  local order_file="$1" hazard_tag="$2"
  case "$hazard_tag" in
    DESTRUCTIVE_FS)
      {
        echo "HAZARD_GUARD: DESTRUCTIVE_FS requires exact non-empty resolved targets, containment checks, and fail-closed removal."
        printf '%s%s%s\n' \
          'MECHANICAL_GUARD: reject empty collection expansion and the token sequence [@]' \
          ':-' ' on any recursive-delete path.'
        echo "NEGATIVE_ORACLE: zero deletion targets must perform zero recursive removals and must not resolve to the owned root."
      } >>"$order_file"
      ;;
    SECURITY)
      {
        echo "HAZARD_GUARD: SECURITY requires deny-by-default authority, bounded input, and typed rejection evidence."
        # 改訂3(2026-08-01): 最上段は非LLM oracleの併用を必須にする。model間エラーは
        # 60%相関するため、別providerを足しても相関しない軸にならない
        echo "MECHANICAL_GUARD: authority rejection and permission width must be proven by a test or static check, not by review opinion."
        echo "NEGATIVE_ORACLE: missing or forged authority must fail before mutation without widening permissions."
      } >>"$order_file"
      ;;
    PERSISTENCE)
      {
        echo "HAZARD_GUARD: PERSISTENCE requires atomic commit, typed failure, and recovery evidence."
        echo "NEGATIVE_ORACLE: interrupted or partial persistence must not become the accepted canonical state."
      } >>"$order_file"
      ;;
    CONCURRENCY)
      {
        echo "HAZARD_GUARD: CONCURRENCY requires explicit owner, generation or ordering evidence, and bounded cancellation."
        echo "NEGATIVE_ORACLE: stale or reordered work must not overwrite the current owner state."
      } >>"$order_file"
      ;;
    PLATFORM)
      {
        echo "HAZARD_GUARD: PLATFORM requires the named target platform and portable command/tool assumptions."
        echo "NEGATIVE_ORACLE: unsupported platform behavior must fail explicitly rather than silently fall back."
      } >>"$order_file"
      ;;
    NONE)
      echo "HAZARD_GUARD: NONE" >>"$order_file"
      ;;
    *)
      gate_fail "cannot append unknown HAZARD_TAG: $hazard_tag"
      ;;
  esac
}

# 改訂3(2026-08-01): 恒久契約は独立性の最上段であり、非LLM oracleの併用を要求する。
# hazard tagと同じく、runnerが要求するだけでなく機械guardを供給する
append_permanence_guard() {
  local order_file="$1" contract_impact="$2"
  [[ "$contract_impact" == "PERMANENT" ]] || return 0
  echo "MECHANICAL_GUARD: permanent format change must be proven by a rejection test or schema golden, not by reviewer opinion." >>"$order_file"
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

# inspectはSparkを再起動しないため、実装後に必ず汚れているworktreeを
# 通常のclean gateへ通さず、base commit照合とscope/checkpoint検証だけを行う
run_dispatch_gate_for_inspect() {
  local order_file="$1" worktree="$2"
  record_gate ROUND "inspect:$(basename "$order_file")"
  gate_run base gate_check_base "$order_file" "$worktree"
  gate_run grain_and_dependencies gate_check_grain_and_dependencies "$order_file" "$worktree"
  gate_run authorities gate_check_authorities_at_base "$order_file" "$worktree"
  gate_run allowed_files gate_check_allowed_files "$order_file"
  gate_run contract_boundary gate_check_contract_boundary "$order_file"
  gate_run reviewer_independence gate_check_reviewer_independence "$order_file"
  gate_run context_budget gate_check_context_budget "$order_file" "$worktree"
  gate_run compiled_grain_contract gate_check_compiled_grain_contract "$order_file"
  gate_run exact_targets gate_check_exact_targets "$order_file" "$worktree" BASE
  gate_run view_profile gate_check_view_profile "$order_file"
  gate_run react_labels gate_check_react_labels "$order_file"
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

# expected-parent-digestはparent shell変数(Spark起動前にbuild_out_of_scope_manifestの
# 結果をhashした値)のみを権威として使う。永続化したevidence file自体を後から読み直して
# 比較の権威にはしない(Spark/Grokのbash toolがevidence_rootへ書き込み得るため)
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
# hash。スコープ判定は通常の git status(--ignored無し)を使うため、Sparkが
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

# Sparkは許可済みfileの中身を変えてよいが、ignore policyそのものを変えて
# 許可外の変更をgit statusから隠すことは許さない。scope closureより前に
# 独立して判定し、隠蔽が成立する前にfail closedする。
# Sparkのbash toolはevidence_root(worktree外)にも書き込み得るため、
# pre-sparkの永続evidence file自体を後から比較の権威に使うと、Sparkがその
# fileを書き換えて偽装できてしまう。before値はSpark起動前にこの関数の外側で
# parent shell変数として確保させ、afterは(post-spark snapshotの永続fileを
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

# SparkもGrokも承認済みorder本文をargv経由で読めるため、実装/検収stage中に
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

verify_generated_file_integrity() {
  local file="$1" expected_hash="$2" label="$3"
  local actual_hash
  [[ -f "$file" ]] || evidence_fail "$label disappeared during Spark implementation"
  actual_hash="$(shasum -a 256 "$file" | awk '{print $1}')"
  [[ "$actual_hash" == "$expected_hash" ]] \
    || evidence_fail "$label mutated during Spark implementation"
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
# とし、Sparkは元よりGrokも起動しない。
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

  # checkpointが指す試行が実際に"STAGE: spark SUCCESS"をこの試行のstage-result.txtへ
  # 記録済みでない限り、checkpoint fileの中身だけを信用しない
  if [[ ! -d "$evidence_root/$cp_attempt" ]] || \
     ! grep -qx 'STAGE: spark SUCCESS' "$evidence_root/$cp_attempt/stage-result.txt" 2>/dev/null; then
    evidence_fail "checkpoint attempt has no recorded spark success"
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
  local pre_fp post_fp inspection_prompt grok_status=0 grok_start_epoch

  snapshot_worktree "$worktree" "$attempt_dir" "pre-grok"
  pre_fp="$(cat "$attempt_dir/pre-grok-fingerprint.sha256")"
  if [[ "$pre_fp" != "$cp_fingerprint" ]]; then
    invalidate_checkpoint "$evidence_root"
    inspect_fail "worktree fingerprint drifted before grok inspection started"
  fi

  inspection_prompt=$(cat <<EOF
You are the read-only acceptance supervisor for Motolii. Do not edit files,
commit, push, create a PR, spawn subagents, or delegate. Inspect the actual diff
and rerun the exact required evidence now. The binding order is the complete
Codex-approved policy capsule for this grain. Open only READ_FILE paths, changed
ALLOWED_FILE paths, and command output named by the order; do not reread all of
AGENTS.md or whole specs and do not perform repository-wide archaeology. Verify
line-by-line against the binding order. Green tests alone are insufficient. Look for scope drift,
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
  grok_status=0
  grok_start_epoch="$(date +%s)"
  (cd "$worktree" && run_supervisor "$attempt_dir/grok-stdout.txt" "$inspection_prompt" verdict \
    "$inspection_timeout" "$(gate_require_single_field "$order_file" "REVIEW_MODEL")") || grok_status=$?
  # 検収はsubshellで走るためrun_agentのglobalが親へ戻らない。親側で計測する
  record_stage_telemetry "$attempt_dir/telemetry.txt" GROK "$(( $(date +%s) - grok_start_epoch ))" \
    "$attempt_dir/grok-stdout-result.json"
  enforce_derived_target_closure "$worktree" "post-grok"
  if [[ "$grok_status" -ne 0 ]]; then
    [[ ! -f "$attempt_dir/grok-stdout.txt" ]] || cat "$attempt_dir/grok-stdout.txt"
    snapshot_worktree "$worktree" "$attempt_dir" "post-grok"
    echo "STAGE: grok FAILED_OR_TIMEOUT" >>"$attempt_dir/stage-result.txt"
    # timeout/失敗自体はworktreeを汚していない限りcheckpointを潰さない。
    # これにより後続のinspectがSparkを再実行せずに再開できる。ただしfingerprintが
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
    # REJECTで捨てた時間とtokenこそ支配項だったため、ここでも必ず合計を出す
    summarize_telemetry "$attempt_dir/telemetry.txt"
    echo "delegate-cursor-supervised: Grok検収REJECT。差分は隔離したまま採用しません" >&2
    echo "STAGE: grok REJECT" >>"$attempt_dir/stage-result.txt"
    exit 4
  fi
  echo "STAGE: grok ACCEPT" >>"$attempt_dir/stage-result.txt"
  summarize_telemetry "$attempt_dir/telemetry.txt"

  echo "delegate-cursor-supervised: 必須検収ACCEPT。Codex最終レビュー待ちです"
}

# 改訂1(2026-08-01): 粒の上限は行数でなく契約境界数。境界そのものは意味であって
# 機械で導出できないため、(a)`CONTRACT_BOUNDARY:`を一つだけ宣言させ、
# (b)allowlistが単一のtop-level ownerへ収まることを照合する。930行のGR-D3は
# 誰も境界数を宣言しなかったため、4境界であることが見えないまま2回dispatchされた。
# docs/はledger・decision-index登録がworkflow上必須のため、別境界に数えない。
gate_check_contract_boundary() {
  local order_file="$1"
  local af owner seen o
  local owners=()
  gate_require_single_field "$order_file" "CONTRACT_BOUNDARY" >/dev/null
  for af in ${GATE_ALLOWED_FILES[@]+"${GATE_ALLOWED_FILES[@]}"}; do
    case "$af" in
      docs/*) continue ;;
      crates/*) owner="crates/$(printf '%s' "${af#crates/}" | cut -d/ -f1)" ;;
      */*) owner="${af%%/*}" ;;
      *) owner="." ;;
    esac
    seen=0
    for o in ${owners[@]+"${owners[@]}"}; do
      [[ "$o" == "$owner" ]] && seen=1
    done
    (( seen == 1 )) || owners+=("$owner")
  done
  if (( ${#owners[@]} > 1 )); then
    gate_fail "ALLOWED_FILE spans multiple contract owners: ${owners[*]}"
  fi
}

# 改訂3(2026-08-01): 検収者を固定modelでなく独立性条件で定める。
# 不変条件は「Grokを使う」ではなく「実装担当から独立した別LLMの外部視点」。
model_family() {
  case "$1" in
    claude-*) printf 'anthropic' ;;
    gpt-*|codex-*) printf 'openai' ;;
    cursor-grok-*|grok-*) printf 'xai' ;;
    *) printf 'unknown' ;;
  esac
}

gate_check_reviewer_independence() {
  local order_file="$1"
  local reviewer implementer manager impact hazard allowed found=0
  # skeleton段ではrunner所有fieldがまだ無い。orderへ書かれた後だけ検査する。
  # 通過ではなくskipであることを台帳へ残す(PASSと区別できないと監査で誤読される)
  if ! grep -Eq '^REVIEW_MODEL:' "$order_file"; then
    record_gate SKIP "reviewer_independence(no REVIEW_MODEL yet)"
    return 0
  fi
  reviewer="$(gate_require_single_field "$order_file" "REVIEW_MODEL")"
  implementer="$(gate_require_single_field "$order_file" "IMPLEMENTER_MODEL")"
  manager="$(gate_require_single_field "$order_file" "ORDER_MANAGER_MODEL")"
  # 検査順は独立性の強い条件から。allowlistを先に置くと、identity条件とfamily条件が
  # allowlistに吸収されて到達不能になり、負例が空虚になる(2026-08-01独立検収P1指摘)
  [[ "$reviewer" != "$implementer" ]] \
    || gate_fail "reviewer must differ from the implementer: $reviewer"
  # 事前設計へ深く関与したmodelを最終reviewerに選ばない
  [[ "$reviewer" != "$manager" ]] \
    || gate_fail "reviewer must not be the order manager: $reviewer"

  impact="$(gate_require_single_field "$order_file" "CONTRACT_IMPACT")"
  hazard="$(gate_require_single_field "$order_file" "HAZARD_TAG")"
  # 段階化: 影響が広がる粒はmodel familyまで離す
  case "$hazard:$impact" in
    NONE:PRIVATE) : ;;
    *)
      [[ "$(model_family "$reviewer")" != "$(model_family "$implementer")" ]] \
        || gate_fail "reviewer must be from a different model family for $hazard/$impact"
      ;;
  esac
  for allowed in ${ALLOWED_REVIEW_MODELS[@]+"${ALLOWED_REVIEW_MODELS[@]}"}; do
    [[ "$allowed" == "$reviewer" ]] && found=1
  done
  (( found == 1 )) || gate_fail "REVIEW_MODEL is not an approved independent reviewer: $reviewer"
  case "$hazard:$impact" in
    SECURITY:*|DESTRUCTIVE_FS:*|*:PERMANENT)
      # 最上段は非LLM oracleの併用を必須にする(model間エラー60%相関)。
      # 存在だけを見ると空宣言で通るため、中身のある一行を要求する
      grep -Eq '^MECHANICAL_GUARD:[[:space:]]*[^[:space:]]' "$order_file" \
        || gate_fail "$hazard/$impact requires a declared non-LLM oracle (MECHANICAL_GUARD)"
      ;;
  esac
}

run_dispatch_gate() {
  local order_file="$1" worktree="$2"
  record_gate ROUND "$(basename "$order_file")"
  gate_run base gate_check_base "$order_file" "$worktree"
  gate_run grain_and_dependencies gate_check_grain_and_dependencies "$order_file" "$worktree"
  gate_run authorities gate_check_authorities "$order_file" "$worktree"
  gate_run allowed_files gate_check_allowed_files "$order_file"
  gate_run contract_boundary gate_check_contract_boundary "$order_file"
  gate_run reviewer_independence gate_check_reviewer_independence "$order_file"
  gate_run context_budget gate_check_context_budget "$order_file" "$worktree"
  gate_run compiled_grain_contract gate_check_compiled_grain_contract "$order_file"
  gate_run exact_targets gate_check_exact_targets "$order_file" "$worktree"
  gate_run view_profile gate_check_view_profile "$order_file"
  gate_run clean_worktree gate_check_clean_worktree "$worktree"
  gate_run react_labels gate_check_react_labels "$order_file"
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
  skeleton_file="$tmp_dir/order-skeleton.txt"
  printf '%s\n' "$task" >"$skeleton_file"
  prepare_skeleton_reject_reserved_fields "$skeleton_file"
  # Opusへ渡す前に、Codex骨格だけでbase、ledger、authority、scope、視野幅を
  # 全て閉じる。WIDEや不一致はmodelに解釈させずここで差戻す。
  run_dispatch_gate "$skeleton_file" "$WORKTREE"
  hazard_tag="$(gate_require_single_field "$skeleton_file" "HAZARD_TAG")"
  supervisor_prompt=$(cat <<EOF
You are the read-only validator for one Motolii implementation grain. Do not use
tools, edit, commit, push, create a PR, spawn subagents, delegate, inspect the
repository, restate the order, or invent authority. Codex and the runner already
validated BASE_SHA, ledger state, dependencies, authority hashes, allowlist,
read set, exact internal/test/reuse targets, context budgets, VIEW_PROFILE,
NEW_SURFACE, and HAZARD_TAG in the skeleton below.
Those runner-owned fields are immutable.

Return only the supplied JSON Schema. disposition is READY when the skeleton is
sufficient, STOP when it contradicts its own closed contract, and ESCALATE when
the stated facts expose a wider unresolved problem. findings contain only the
highest-severity missing RISK, NEGATIVE_ORACLE, STOP, or CORRECTION. Return at
most $MAX_OPUS_DELTA_FINDINGS findings, each at most
$MAX_OPUS_DELTA_TEXT_BYTES bytes. Prefer fewer findings. Do not repeat facts
already present, propose optional hardening, expand product scope, or write a
full order. Known hazard guards are injected mechanically after validation.

Validated order skeleton:
$task
EOF
  )
  echo "## 1. Claude Opus 5 typed order delta"
  prepare_telemetry_file="${ORDER_FILE}.evidence/prepare-telemetry.txt"
  opus_start_epoch="$(date +%s)"
  if ! (cd "$WORKTREE" && run_order_manager_delta "$tmp_dir/opus-result.json" "$supervisor_prompt"); then
    record_stage_telemetry "$prepare_telemetry_file" OPUS_DELTA \
      "$(( $(date +%s) - opus_start_epoch ))" "$tmp_dir/opus-result.json"
    [[ ! -f "$tmp_dir/opus-result.json" ]] || cat "$tmp_dir/opus-result.json"
    exit 1
  fi
  record_stage_telemetry "$prepare_telemetry_file" OPUS_DELTA \
    "$(( $(date +%s) - opus_start_epoch ))" "$tmp_dir/opus-result.json"
  jq '.structured_output' "$tmp_dir/opus-result.json"
  jq -c '.structured_output' "$tmp_dir/opus-result.json" >"$tmp_dir/opus-delta.json"
  opus_disposition="$(jq -r '.disposition' "$tmp_dir/opus-delta.json")"
  if [[ "$opus_disposition" != "READY" ]]; then
    summarize_telemetry "$prepare_telemetry_file"
    echo "delegate-cursor-supervised: Opus typed delta disposition=${opus_disposition}。Sparkは起動しません" >&2
    exit 3
  fi
  blocking_delta_count="$(jq '[.findings[] | select(.severity == "P0" or .severity == "P1")] | length' \
    "$tmp_dir/opus-delta.json")"
  if (( blocking_delta_count > 0 )); then
    # 430〜586秒のSpark実行を約20秒の骨格差戻しへ置換した経路。その安さを実測で残す
    summarize_telemetry "$prepare_telemetry_file"
    echo "delegate-cursor-supervised: Opus P0/P1 deltaが${blocking_delta_count}件あります。骨格とexact oracleへ織り込み、prepareを再実行してください" >&2
    exit 3
  fi

  cat "$skeleton_file" >"$ORDER_FILE"
  append_hazard_guard "$ORDER_FILE" "$hazard_tag"
  append_permanence_guard "$ORDER_FILE" "$(gate_require_single_field "$skeleton_file" "CONTRACT_IMPACT")"
  jq -r '.findings | to_entries[] | "OPUS_DELTA_FINDING: F\(.key + 1) \(.value.kind) \(.value.severity) \(.value.delta)"' \
    "$tmp_dir/opus-delta.json" >>"$ORDER_FILE"
  {
    printf 'OPUS_DELTA_REASON: %s\n' "$(jq -r '.reason' "$tmp_dir/opus-delta.json")"
    echo "ORDER: READY"
    echo "LOOP_PROFILE: $LOOP_PROFILE"
    echo "ORDER_MANAGER_MODEL: $OPUS_MANAGER_MODEL"
    echo "IMPLEMENTER_MODEL: $SPARK_MODEL"
    echo "REVIEW_MODEL: $CURSOR_GROK_MODEL"
    echo "TASK_SHA256: $task_hash"
  } >>"$ORDER_FILE"
  run_dispatch_gate "$ORDER_FILE" "$WORKTREE"
  summarize_telemetry "$prepare_telemetry_file"
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
ORDER_LOOP_PROFILE="$(order_value "$ORDER_FILE" LOOP_PROFILE)"
ORDER_MANAGER_MODEL="$(order_value "$ORDER_FILE" ORDER_MANAGER_MODEL)"
ORDER_IMPLEMENTER_MODEL="$(order_value "$ORDER_FILE" IMPLEMENTER_MODEL)"
ORDER_REVIEW_MODEL="$(order_value "$ORDER_FILE" REVIEW_MODEL)"
if [[ -z "$ORDER_LOOP_PROFILE" || -z "$ORDER_MANAGER_MODEL" || -z "$ORDER_IMPLEMENTER_MODEL" || -z "$ORDER_REVIEW_MODEL" ]]; then
  echo "delegate-cursor-supervised: 発注書のloop/model指定が欠落または重複しています" >&2
  exit 3
fi
# REVIEW_MODELはここで固定値と突き合わせない。改訂3(2026-08-01)により検収者は
# model名でなく独立性条件で決まり、gate_check_reviewer_independenceが判定する
if [[ "$ORDER_LOOP_PROFILE" != "$LOOP_PROFILE" ]] ||
   [[ "$ORDER_MANAGER_MODEL" != "$OPUS_MANAGER_MODEL" ]] ||
   [[ "$ORDER_IMPLEMENTER_MODEL" != "$SPARK_MODEL" ]]; then
  echo "delegate-cursor-supervised: 発注書のmodel経路がOpus/Spark監督ループと一致しません" >&2
  exit 3
fi
if ! grep -qx 'CODEX PRECHECK: APPROVED' "$ORDER_FILE"; then
  echo "delegate-cursor-supervised: Codex事前承認がありません" >&2
  exit 3
fi
gate_check_delta_resolutions "$ORDER_FILE"

# GR-D2: 発注書ごとのevidence directoryへ、execute/inspectの各試行をappend-onlyで残す
evidence_root="${ORDER_FILE}.evidence"
mkdir -p "$evidence_root"
attempt_dir="$(new_attempt_dir "$evidence_root")"
CURRENT_ATTEMPT_DIR="$attempt_dir"
attempt_name="$(basename "$attempt_dir")"
cp "$ORDER_FILE" "$attempt_dir/order.txt"
# Spark/Grokが起動する前の承認済みorder本文のhash。checkpointへはこの
# pre-model hashだけを刻み、各stage後にこの値との一致を独立に再確認する
approved_order_sha256="$(shasum -a 256 "$attempt_dir/order.txt" | awk '{print $1}')"
printf '%s' "$task" >"$attempt_dir/task.txt"
# receiptに書くmodelは定数でなくorderの宣言。既定から外れた場合は代替として明記し、
# `MODEL_FALLBACK`を実際の値から導出する(literalだと差し替えを隠す)
RECEIPT_REVIEW_MODEL="$(gate_require_single_field "$ORDER_FILE" "REVIEW_MODEL")"
if [[ "$RECEIPT_REVIEW_MODEL" == "$CURSOR_GROK_MODEL" ]]; then
  RECEIPT_MODEL_FALLBACK="NONE"
else
  RECEIPT_MODEL_FALLBACK="REVIEW_MODEL_SUBSTITUTED:$RECEIPT_REVIEW_MODEL"
fi
{
  echo "MODE: $MODE"
  echo "TASK_SHA256: $task_hash"
  echo "WORKTREE: $WORKTREE"
  echo "LOOP_PROFILE: $LOOP_PROFILE"
  echo "ORDER_MANAGER_MODEL: $OPUS_MANAGER_MODEL"
  echo "IMPLEMENTER_MODEL: $SPARK_MODEL"
  # 改訂3: receiptへ**実際に起動するmodel**を残す。定数を書くと、独立性gateが
  # 検査した宣言・実際のargv・receiptの三者が乖離し、受領証の意味が消える
  echo "REVIEW_MODEL: $RECEIPT_REVIEW_MODEL"
  echo "IMPLEMENTER_MODEL_FAMILY: $(model_family "$SPARK_MODEL")"
  echo "REVIEW_MODEL_FAMILY: $(model_family "$RECEIPT_REVIEW_MODEL")"
  echo "MODEL_FALLBACK: $RECEIPT_MODEL_FALLBACK"
} >"$attempt_dir/metadata.txt"

if [[ "$MODE" == "inspect" ]]; then
  enforce_derived_target_closure "$WORKTREE" "inspect-resume"
  # inspectはSparkを再起動しない。実装成功直後のcheckpointに現在の
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
# Spark起動前に既存checkpointを即時無効化する。CHECKPOINT_SETTLEDは0のままにして
# おくことで、Sparkがcheckpoint.txtを自分で偽造してもEXIT trapが後始末する
rm -f "$evidence_root/checkpoint.txt"
mark_checkpoint_at_risk "$evidence_root"
record_base_metadata "$ORDER_FILE" "$attempt_dir"
snapshot_worktree "$WORKTREE" "$attempt_dir" "pre-spark"
# Spark起動前、この試行のevidence fileがまだ書き換えられていないうちに
# ignore policy hashをparent shell変数として確保する(enforce_ignore_policy_unchanged
# 側のコメント参照)
pre_spark_ignore_policy="$(cat "$attempt_dir/pre-spark-ignore-policy.sha256")"
# 同様に、生scope manifestのdigestもSpark起動前にparent shell変数として確保する。
# 永続化したevidence fileはcopyに過ぎず、比較の権威はこの変数だけが持つ
build_out_of_scope_manifest "$WORKTREE" "$attempt_dir/pre-spark-out-of-scope-manifest.nul"
pre_spark_manifest_digest="$(shasum -a 256 "$attempt_dir/pre-spark-out-of-scope-manifest.nul" | awk '{print $1}')"
printf '%s\n' "$pre_spark_manifest_digest" >"$attempt_dir/pre-spark-out-of-scope-manifest.sha256"
pre_spark_ref_digest="$(compute_ref_digest "$WORKTREE")"
printf '%s\n' "$pre_spark_ref_digest" >"$attempt_dir/pre-spark-ref-digest.sha256"

head_before="$(git -C "$WORKTREE" rev-parse HEAD)"
target_capsule_file="$attempt_dir/spark-target-capsule.txt"
compiled_grain_file="$attempt_dir/spark-compiled-grain.txt"
spark_prompt_file="$attempt_dir/spark-prompt.txt"
build_target_capsule "$ORDER_FILE" "$WORKTREE" "$target_capsule_file"
build_compiled_spark_grain "$ORDER_FILE" "$compiled_grain_file"
implementation_prompt=$(cat <<EOF
You are the Spark implementation translator for one Codex-approved Motolii
grain. Implement only the compiled grain below in the isolated worktree. Start
from the target capsule; read only a bounded neighborhood in a named READ_FILE
when a direct dependency requires it. Do not search the repository, reopen
AGENTS.md or whole specs, broaden scope, invent defaults, weaken tests, add a
new command, mode, public API, generic helper, or parallel path, write outside
the worktree, commit, push, delegate, or rerun this runner. If the named targets
cannot satisfy the grain, report the conflicting code fact without improvising.
Run every Test and DELTA_RESOLUTION check before the final response; a failed or
ambiguous check is a local STOP.

TARGET_CAPSULE:
$(cat "$target_capsule_file")

COMPILED_IMPLEMENTATION_GRAIN:
$(cat "$compiled_grain_file")
EOF
)
printf '%s\n' "$implementation_prompt" >"$spark_prompt_file"
target_capsule_sha256="$(shasum -a 256 "$target_capsule_file" | awk '{print $1}')"
compiled_grain_sha256="$(shasum -a 256 "$compiled_grain_file" | awk '{print $1}')"
spark_prompt_sha256="$(shasum -a 256 "$spark_prompt_file" | awk '{print $1}')"
{
  printf 'TARGET_CAPSULE_SHA256: %s\n' "$target_capsule_sha256"
  printf 'SPARK_GRAIN_SHA256: %s\n' "$compiled_grain_sha256"
  printf 'SPARK_PROMPT_SHA256: %s\n' "$spark_prompt_sha256"
} >>"$attempt_dir/metadata.txt"

echo
echo "## 2. Codex Spark implementation"
if ! run_agent "$attempt_dir/spark-stream.jsonl" "$SPARK_TIMEOUT_SECONDS" \
  env CODEX_DELEGATED=1 "$CODEX_AGENT_BIN" --ask-for-approval never exec \
    --json \
    --ephemeral --ignore-user-config --ignore-rules --strict-config \
    --disable memories --disable plugins --disable apps \
    --disable browser_use --disable browser_use_external \
    --disable browser_use_full_cdp_access --disable computer_use \
    --disable multi_agent --disable multi_agent_v2 --disable skill_search \
    --color never --model "$SPARK_MODEL" \
    --sandbox workspace-write --cd "$WORKTREE" \
    "$implementation_prompt"; then
  extract_spark_result "$attempt_dir"
  record_stage_telemetry "$attempt_dir/telemetry.txt" SPARK "$RUN_AGENT_WALL_SECONDS" \
    "$attempt_dir/spark-result.json"
  [[ ! -f "$attempt_dir/spark-stdout.txt" ]] || cat "$attempt_dir/spark-stdout.txt"
  snapshot_worktree "$WORKTREE" "$attempt_dir" "post-spark"
  echo "STAGE: spark FAILED_OR_TIMEOUT" >>"$attempt_dir/stage-result.txt"
  invalidate_checkpoint "$evidence_root"
  exit 1
fi
extract_spark_result "$attempt_dir"
record_stage_telemetry "$attempt_dir/telemetry.txt" SPARK "$RUN_AGENT_WALL_SECONDS" \
  "$attempt_dir/spark-result.json"
cat "$attempt_dir/spark-stdout.txt"
verify_generated_file_integrity "$target_capsule_file" "$target_capsule_sha256" "Spark target capsule"
verify_generated_file_integrity "$compiled_grain_file" "$compiled_grain_sha256" "compiled Spark grain"
verify_generated_file_integrity "$spark_prompt_file" "$spark_prompt_sha256" "Spark prompt"
if [[ "$(git -C "$WORKTREE" rev-parse HEAD)" != "$head_before" ]]; then
  echo "delegate-cursor-supervised: 受注者がcommitを作成したため検収へ進みません" >&2
  snapshot_worktree "$WORKTREE" "$attempt_dir" "post-spark"
  echo "STAGE: spark COMMIT_FORBIDDEN" >>"$attempt_dir/stage-result.txt"
  invalidate_checkpoint "$evidence_root"
  exit 5
fi

post_spark_ref_digest="$(compute_ref_digest "$WORKTREE")"
if [[ "$post_spark_ref_digest" != "$pre_spark_ref_digest" ]]; then
  {
    echo "pre_spark_ref_digest=$pre_spark_ref_digest"
    echo "post_spark_ref_digest=$post_spark_ref_digest"
  } >"$attempt_dir/post-spark-ref-digest-violations.txt"
  echo "REF NG: protected ref digest changed during implementation" >&2
  snapshot_worktree "$WORKTREE" "$attempt_dir" "post-spark"
  echo "STAGE: implementation REF_MUTATION_FORBIDDEN" >>"$attempt_dir/stage-result.txt"
  invalidate_checkpoint "$evidence_root"
  exit 5
fi

enforce_derived_target_closure "$WORKTREE" "post-spark"

# process group reap(run_agent内)後、通常のgit status由来のscope closureより先に、
# parent保持のpre-Spark生manifest digestと直接突き合わせる
build_out_of_scope_manifest "$WORKTREE" "$attempt_dir/post-spark-out-of-scope-manifest.nul"
enforce_out_of_scope_manifest_unchanged "$pre_spark_manifest_digest" \
  "$attempt_dir/post-spark-out-of-scope-manifest.nul" \
  "$attempt_dir/post-spark-out-of-scope-manifest-violations.txt" \
  "$WORKTREE" "$pre_spark_ignore_policy"

snapshot_worktree "$WORKTREE" "$attempt_dir" "post-spark"
enforce_ignore_policy_unchanged "$WORKTREE" "$attempt_dir" "$pre_spark_ignore_policy" "post-spark"
enforce_scope_closure "$WORKTREE" "$attempt_dir/post-spark-scope-violations.txt"
# worktree fingerprintはworktree外のorder fileを見ないため、Spark実装中に
# 承認済みorder本文(外部fileとこの試行のcopyの両方)が変わっていないか独立に確認する
verify_order_integrity "$ORDER_FILE" "$attempt_dir" "$approved_order_sha256" "spark implementation"
echo "STAGE: spark SUCCESS" >>"$attempt_dir/stage-result.txt"

post_impl_fp="$(cat "$attempt_dir/post-spark-fingerprint.sha256")"
base_ref_val="$(gate_require_single_field "$ORDER_FILE" "BASE_REF")"
base_sha_val="$(gate_require_single_field "$ORDER_FILE" "BASE_SHA")"
publish_checkpoint "$evidence_root" "$attempt_name" "$approved_order_sha256" "$task_hash" "$base_ref_val" "$base_sha_val" "$head_before" "$post_impl_fp"

# Grokを起動する直前にもう一度EXIT trapを「無効化がデフォルト」へ倒す。
# run_inspection_stage自身がGrokの結果に応じてpublish/invalidateを確定させる
mark_checkpoint_at_risk "$evidence_root"
run_inspection_stage "$WORKTREE" "$task" "$(cat "$attempt_dir/order.txt")" "$attempt_dir" "$INSPECTION_TIMEOUT_SECONDS" \
  "$ORDER_FILE" "$approved_order_sha256" \
  "$evidence_root" "$attempt_name" "$task_hash" "$base_ref_val" "$base_sha_val" "$head_before" "$post_impl_fp"
