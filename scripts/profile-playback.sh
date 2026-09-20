#!/bin/bash
# 再生中に**実際に呼ばれた物を全部**持ってきて、持ち主ごとに畳む。
#
# 手で時計を置かない — OS の標本器(`sample`、Xcode 同梱)が 1ms ごとに全 thread の
# 呼び出し木を取り、scripts/fold-sample.py が self / total へ畳む。
# 関数が増えても、この script も畳む側も直さない。
#
#   scripts/profile-playback.sh [秒 = 10] [上位 = 25]
#
# **先にこれを走らせて、数えている間に ▶ を押す。** 再生は数秒で終わるので、
# 押してから走らせると止まった窓を測ってしまう(眠っている thread しか出ない)。
# Linux の枝: `perf record -p <pid> -F 1000 -g` + `perf script`(畳む側は入力の形だけ替える)。
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
seconds="${1:-10}"
top="${2:-25}"
pid=$(pgrep -x motolii_stage5 | head -1 || true)
[[ -n "$pid" ]] || { echo '窓が居ません。scripts/motolii-ui.sh dev で起こしてから。'; exit 1; }
out="/tmp/motolii-sample-$(date +%Y%m%d-%H%M%S).txt"
echo "pid=$pid を ${seconds}s 標本します。"
echo ">>> いま窓へ行って ▶ を押してください(2 秒後から数え始めます) <<<"
sleep 2
/usr/bin/sample "$pid" "$seconds" 1 -f "$out" >/dev/null
python3 "$repo/scripts/fold-sample.py" "$out" "$top"
echo
echo "生の木: $out"
