#!/usr/bin/env bash
# レーンの検収を1コマンドに固定する(2026-08-23、背景待ちの自己停止9連発の根治)。
#
# なぜこれが在るか
# ----------------
# レーン(subagent)が cargo を**背景で起動して自分の完了通知を待ち、来ない signal を
# 待ち続けて止まる**事故が、このセッションだけで9回起きた。発注書に「背景禁止」と
# 書く対処を9回繰り返して9回とも失敗している。文章で禁じるのをやめ、`AGENTS.md`
# 「頻出コマンド(コピペ用 — 記憶から組み立てない)」と同じ手を採る:
# **組み立てる余地を消す**。
#
# !! 未検証 !! (2026-08-23)
# この script は**一度も実行していない**(書いた直後にセッションのリセットが入ったため)。
# 発注書で使う前に、必ず自分で `bash next/lane-check.sh check motolii-core` 等を通して
# 動作を確かめること。動かなければ直すか捨てる — 未検証の物をレーンへ渡さない。
#
# 使い方(これだけ。オプションは無い)
#   bash next/lane-check.sh check  <crate...>   # cargo check --tests
#   bash next/lane-check.sh test   <crate...>   # cargo test
#   bash next/lane-check.sh full                # ワークスペース全体の test
#
# 例:
#   bash next/lane-check.sh test motolii-store motolii-eval
#
# この script が引き受けること(レーンが判断しなくてよくなること)
#   - **前景で走る**(背景・監視・待ち合わせを構造的に作れない)
#   - `--manifest-path` を `$(git rev-parse --show-toplevel)/next` へ解決
#     (自分の worktree で走る。絶対パス直書きで main を誤ビルドしない)
#   - **asdf shim を迂回して toolchain を直呼び**+レーンごとの `CARGO_HOME`
#     (裁定204: shim の exec-env が `CARGO_HOME` を無条件に上書きするため、
#      呼び手が設定しても効かない。直呼びでロック待ち 3/3 → 0/0)
#   - **合否をパイプ越しに殺さない**(AGENTS.md: log へ落として `$?` を直取り)
#   - 出力を絞って返す(全文を待たない・エラーと test result だけ)
set -u

MODE="${1:-}"
shift || true

ROOT="$(git rev-parse --show-toplevel)"
MANIFEST="$ROOT/next/Cargo.toml"
CARGO_BIN="$HOME/.asdf/installs/rust/stable/bin/cargo"
[ -x "$CARGO_BIN" ] || CARGO_BIN="cargo"   # asdf が無い環境では素の cargo

# レーンごとに CARGO_HOME を分ける(裁定204 — 共有ロックの直列化を避ける)。
# worktree のパスを鍵にするので、レーン同士で衝突しない。
export CARGO_HOME="${CARGO_HOME_OVERRIDE:-/tmp/motolii-cargo-home-$(echo "$ROOT" | shasum | cut -c1-12)}"

LOG="/tmp/motolii-lane-check-$$.log"

case "$MODE" in
  check) set -- check --tests --manifest-path "$MANIFEST" $(printf -- '-p %s ' "$@") -j 4 ;;
  test)  set -- test  --manifest-path "$MANIFEST" $(printf -- '-p %s ' "$@") -j 4 --no-fail-fast ;;
  full)  set -- test  --manifest-path "$MANIFEST" --workspace --locked --no-fail-fast -j 4 ;;
  *)
    echo "使い方: bash next/lane-check.sh {check|test} <crate...>   /   bash next/lane-check.sh full" >&2
    exit 2
    ;;
esac

echo "== $CARGO_BIN $* =="
echo "== CARGO_HOME=$CARGO_HOME =="
"$CARGO_BIN" "$@" > "$LOG" 2>&1
STATUS=$?

# 合否は $? を直取り済み。以降の絞り込みは表示のためだけ(exit code を殺さない)。
grep -E '^(error|error\[|warning: unused|test result|failures:)' "$LOG" | head -40
echo "----"
echo "EXIT=$STATUS   (全文: $LOG)"
exit "$STATUS"
