#!/usr/bin/env python3
"""コードが宣言したWIRE境界に意味の書き込みが漏れていないか検査する。"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path


WIRE_RE = re.compile(r"^\s*//!\s*responsibility:\s*wire\s*$", re.MULTILINE)
FORBIDDEN_RE = re.compile(
    r"\b(?:self\.)?doc\s*\.\s*(?:apply|apply_all|set_transient|clear_transient)\s*\(|"
    r"\bDocument\s*::\s*(?:load|save)\s*\("
)
RENDER_WRITE_RE = re.compile(
    r"\b(?:self\.)?doc\s*\.\s*(?:apply|apply_all|set_transient|clear_transient)\s*\("
)


def source_files(root: Path):
    next_root = root / "next"
    for directory, dirnames, filenames in os.walk(next_root):
        dirnames[:] = [name for name in dirnames if name != "target" and not name.startswith(".")]
        for filename in filenames:
            if filename.endswith(".rs"):
                yield Path(directory) / filename


def code_lines(text: str):
    in_block = False
    for number, original in enumerate(text.splitlines(), 1):
        line = original
        if in_block:
            end = line.find("*/")
            if end < 0:
                continue
            line = line[end + 2 :]
            in_block = False
        while "/*" in line:
            start = line.find("/*")
            end = line.find("*/", start + 2)
            if end < 0:
                line = line[:start]
                in_block = True
                break
            line = line[:start] + line[end + 2 :]
        line = line.split("//", 1)[0]
        if line.strip():
            yield number, line


def without_cfg_test_modules(text: str) -> str:
    """本体の柵から同じファイル内の cfg(test) module だけを外す。"""
    lines = text.splitlines()
    kept = []
    skipping = False
    depth = 0
    for line in lines:
        if not skipping and "#[cfg(test)]" in line:
            skipping = True
            depth = line.count("{") - line.count("}")
            continue
        if skipping:
            depth += line.count("{") - line.count("}")
            if depth <= 0:
                skipping = False
            continue
        kept.append(line)
    return "\n".join(kept)


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    wire = []
    errors = []
    for path in source_files(root):
        text = path.read_text(encoding="utf-8", errors="ignore")
        if not WIRE_RE.search(text):
            continue
        wire.append(path)
        relative = path.relative_to(root).as_posix()
        for number, line in code_lines(text):
            if FORBIDDEN_RE.search(line):
                errors.append(f"{relative}:{number}: WIREに意味の書き込みがある")

    expected = root / "next/shell/motolii-shell/src/lib.rs"
    if expected not in wire:
        errors.append("next/shell/motolii-shell/src/lib.rs: WIRE宣言が無い")
    render = root / "next/shell/motolii-shell/src/render.rs"
    if render.exists():
        production = without_cfg_test_modules(render.read_text(encoding="utf-8", errors="ignore"))
        if RENDER_WRITE_RE.search(production):
            errors.append("next/shell/motolii-shell/src/render.rs: 描画moduleがDocumentへ書き込む")
    for error in errors:
        print(error)
    print(f"WIRE files {len(wire)} / 違反 {len(errors)}")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
