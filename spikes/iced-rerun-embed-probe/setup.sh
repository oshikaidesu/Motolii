#!/usr/bin/env bash
# iced master を `vendor/iced` に取ってきて、**1行だけ**直す。
#
# なぜ git 依存のままにできないのか
# --------------------------------
# iced master の workspace は `web-sys = "=0.3.85"` と**完全一致**で釘を打っている
# (`Cargo.toml`)。web-sys 0.3.85 は `js-sys = "=0.3.85"` を要求する。一方 Rerun fork の
# `re_renderer` は `js-sys ^0.3.94` を要求する。どちらも wasm32 でしか使われない依存だが、
# cargo の解決はターゲットを問わず全部を1つのグラフに載せるので、両者は同居できない:
#
#     error: failed to select a version for `js-sys`.
#         ... required by package `re_renderer`
#       previously selected package `js-sys v0.3.85`
#         ... which satisfies dependency `js-sys = "=0.3.85"` of package `web-sys v0.3.85`
#         ... which satisfies dependency `web-sys = "=0.3.85"` of package `iced_winit`
#
# 直し方は `=0.3.85` を `0.3.85` にするだけ(上流に投げれば済む種類の話であって、
# 設計上の壁ではない)。native ビルドには一切影響しない。
# この spike ではその1行だけを当てて `path` 依存で使う。rev は下の変数が正本。
set -euo pipefail

ICED_REV="3de451447bd28217bb535632867550908e29d5d0"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEST="$HERE/vendor/iced"

if [ ! -d "$DEST/.git" ]; then
    mkdir -p "$HERE/vendor"
    git clone https://github.com/iced-rs/iced.git "$DEST"
fi

git -C "$DEST" fetch --all --tags
git -C "$DEST" checkout --force "$ICED_REV"
git -C "$DEST" checkout -- Cargo.toml

# 唯一の変更。
perl -pi -e 's/^web-sys = "=0\.3\.85"$/web-sys = "0.3.85"/' "$DEST/Cargo.toml"

echo "vendored iced @ $ICED_REV into $DEST"
git -C "$DEST" --no-pager diff --stat
