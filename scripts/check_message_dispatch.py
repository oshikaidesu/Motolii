#!/usr/bin/env python3
"""`Message` の全 variant が、どこかの dispatch_* に届いているか。

`Shell::update` は 2026-08-23(SP-1)に**一つの網羅 match** から
8領域の chain-of-responsibility へ分割された。読みやすさは上がったが、
**コンパイラの網羅検査を失った** — `dispatch_*` はどれも catch-all で
`Err(message)` を次へ回す形なので、新しい variant を足して腕を書き忘れても
`dispatch_message` 末尾の `Err(_unhandled) => Task::none()` に落ち、
**押しても何も起きない**枝が黙って増える(M13「無反応ゼロ」に反する)。

コンパイル時には戻せない(どの dispatcher も「他が消費した」を証明できない)ので、
**文字列として**検出する。variant 名が Message enum の外に一度も現れなければ落ちる。

落ちたら: その variant を扱う腕を、対応する領域の `dispatch_*` へ足す。
"""
import re, sys, pathlib

root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".")
src = root / "next/shell/motolii-shell/src"
lib = (src / "lib.rs").read_text(encoding="utf-8")

m = re.search(r"pub enum Message \{(.*?)\n\}", lib, re.S)
if not m:
    sys.exit("lib.rs に `pub enum Message` が見つからない(構造が変わった?)")
body = re.sub(r"//[^\n]*", "", m.group(1))
variants = re.findall(r"^\s{4}([A-Z]\w*)", body, re.M)

# enum 定義そのものは干し草に入れない(自己一致してしまう)
hay = lib[m.end():] + "".join(
    p.read_text(encoding="utf-8") for p in sorted(src.rglob("*.rs")) if p.name != "lib.rs"
)
missing = [v for v in variants if not re.search(r"\b%s\b" % v, hay)]

print(f"Message variant {len(variants)}件 / 届いていない {len(missing)}件")
if missing:
    print("\nどの dispatch_* にも現れない(押しても無反応になる):")
    for v in missing:
        print(f"  - Message::{v}")
    sys.exit(1)
