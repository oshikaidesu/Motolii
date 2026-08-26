#!/usr/bin/env python3
"""UIソースのデザイン数値を抽出し、tokens正本との対応を台帳化する。

この検査器は数値をJSONへ勝手に追加しない。UIの見た目に関わるsinkから
raw literalと既存token参照を同じ形式で拾い、どの値が正本へ移っていないかを
file:lineで残す。フレーム数・時間・作品データを誤って拾わないため、対象は
明示したUI sinkと、名前にデザイン語を含むUI定数に限定する。
"""

from __future__ import annotations

import argparse
import csv
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path


NUMBER = r"-?(?:\d+(?:\.\d*)?|\.\d+)"
DESIGN_WORDS = re.compile(
    r"(?:SIZE|WIDTH|HEIGHT|PADDING|MARGIN|GAP|RADIUS|INSET|OFFSET|LINE|"
    r"BORDER|HIT|DIAMOND|HANDLE|ROW|COLUMN|GRID|PLOT|ICON|GLYPH|TARGET|"
    r"ASPECT|THUMB)",
    re.IGNORECASE,
)

# These are still recorded in the ledger, but are deliberately not UI tokens:
# they belong to validation, media sampling, or an off-screen instrument. A
# policy value must remain visible in the ledger so a later UI migration does
# not silently turn it into an unowned visual constant.
POLICY_CONST_NAMES = {
    "MIN_WIDTH",
    "NOMINAL_WAVEFORM_WIDTH_PX",
    "CANVAS_WIDTH",
    "TARGET_PX_PER_BUCKET",
}

# A literal is design-relevant only when it is the direct argument of one of
# these sinks. This intentionally does not include arithmetic in document,
# camera, timing, or geometry code: those values need a semantic owner first.
SINK_RE = re.compile(
    rf"(?P<context>Length::Fixed|size|spacing|padding|with_width|with_height|"
    rf"with_radius|line_width|stroke_width|border_width)\s*\(\s*(?:\[\s*)?(?P<value>{NUMBER})"
)
FIELD_RE = re.compile(
    r"\b(?P<owner>dims|colors)\.(?P<field>[A-Za-z_][A-Za-z0-9_]*)(?![A-Za-z0-9_.])(?!\s*\()"
)
COMPONENT_FIELD_RE = re.compile(
    r"\bdims\.components\.(?P<group>[A-Za-z_][A-Za-z0-9_]*)\."
    r"(?P<field>[A-Za-z_][A-Za-z0-9_]*)(?![A-Za-z0-9_])(?!\s*\()"
)
UTILITY_FIELD_RE = re.compile(
    r"\.theme\(\)\.(?P<namespace>space|text|size|stroke|target)\."
    r"(?P<field>[A-Za-z_][A-Za-z0-9_]*)(?![A-Za-z0-9_])"
)
CONST_RE = re.compile(
    rf"\bconst\s+(?P<name>[A-Z][A-Z0-9_]*)\s*:[^=;]+?=\s*(?P<value>{NUMBER})"
)

UTILITY_FIELDS = {
    "space": {"xs", "s", "m", "l"},
    "text": {"title", "body", "caption", "micro"},
    "size": {"row", "transport", "panel_header", "pane_header"},
    "stroke": {"hairline", "focus"},
    "target": {"minimum"},
}
STRUCT_FIELD_RE = re.compile(r"^\s*pub\s+(?P<field>[A-Za-z_][A-Za-z0-9_]*)\s*:")

HEADERS = (
    "kind",
    "owner",
    "field_or_literal",
    "value",
    "file",
    "line",
    "context",
    "suggested_token",
    "verdict",
)


@dataclass(frozen=True)
class Finding:
    kind: str
    owner: str
    field_or_literal: str
    value: str
    file: str
    line: int
    context: str
    suggested_token: str
    verdict: str


def rust_source_roots(root: Path) -> list[Path]:
    return [root / "next/ui", root / "next/shell/motolii-shell/src"]


def source_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for source_root in rust_source_roots(root):
        if not source_root.exists():
            continue
        for path in sorted(source_root.rglob("*.rs")):
            parts = set(path.parts)
            if (
                "tests" in parts
                or "fixtures" in parts
                or "generated" in parts
                or "vendored" in parts
                or path.name.startswith("lib_tests")
                or path.name.endswith("_tests.rs")
                or "motolii-tokens-rs" in parts
            ):
                continue
            files.append(path)
    return files


def strip_rust_comments(text: str) -> list[str]:
    """Return source lines with // and /* */ comments blanked.

    This is deliberately a small lexical pass, not a Rust parser. Keeping line
    breaks makes every result stable and useful as a source pointer. A comment
    inside a string is rare in the UI sink patterns and is left untouched; the
    sink regex still requires a method call or a named const.
    """

    lines: list[str] = []
    in_block = False
    for original in text.splitlines():
        line = []
        index = 0
        while index < len(original):
            if in_block:
                end = original.find("*/", index)
                if end < 0:
                    index = len(original)
                else:
                    in_block = False
                    index = end + 2
                continue
            if original.startswith("//", index):
                break
            if original.startswith("/*", index):
                in_block = True
                index += 2
                continue
            line.append(original[index])
            index += 1
        lines.append("".join(line))
    return lines


def strip_cfg_test_modules(lines: list[str]) -> list[str]:
    """Blank `#[cfg(test)] mod ... {}` blocks while preserving line numbers."""

    result: list[str] = []
    pending = False
    depth = 0
    for line in lines:
        if depth:
            result.append("")
            depth += line.count("{") - line.count("}")
            if depth <= 0:
                depth = 0
            continue
        if "#[cfg(test)]" in line:
            pending = True
            result.append("")
            continue
        if pending:
            result.append("")
            if re.search(r"\bmod\s+[A-Za-z_][A-Za-z0-9_]*\s*\{", line):
                depth = line.count("{") - line.count("}")
                pending = False
            continue
        result.append(line)
    return result


def dimensions_fields(root: Path) -> set[str]:
    path = root / "next/ui/motolii-tokens-rs/src/dimensions.rs"
    if not path.exists():
        return set()
    fields: set[str] = set()
    in_struct = False
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("pub struct Dimensions"):
            in_struct = True
            continue
        if in_struct and line.startswith("}"):
            break
        if in_struct:
            match = STRUCT_FIELD_RE.match(line)
            if match:
                fields.add(match.group("field"))
    return fields


def colors_fields(root: Path) -> set[str]:
    path = root / "next/ui/motolii-tokens-rs/src/colors.rs"
    if not path.exists():
        return set()
    fields: set[str] = set()
    in_struct = False
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("pub struct Colors"):
            in_struct = True
            continue
        if in_struct and line.startswith("}"):
            break
        if in_struct:
            match = STRUCT_FIELD_RE.match(line)
            if match:
                fields.add(match.group("field"))
    return fields


def component_fields(root: Path) -> set[str]:
    path = root / "next/ui/motolii-tokens-rs/tokens/dimensions.json"
    if not path.exists():
        return set()
    try:
        values = json.loads(path.read_text(encoding="utf-8")).get("components", {})
    except (OSError, json.JSONDecodeError):
        return set()
    fields: set[str] = set()
    for group, group_values in values.items():
        if isinstance(group_values, dict):
            fields.update(
                f"{group}.{field}"
                for field in group_values
                if not field.startswith("_")
            )
    return fields


def token_suggestion(context: str, source: Path, const_name: str | None = None) -> str:
    if const_name:
        return re.sub(r"_+", "_", const_name.lower()).strip("_")
    stem = source.stem.lower().replace("-", "_")
    return f"{stem}_{context.lower()}"


def relpath(path: Path, root: Path) -> str:
    return path.relative_to(root).as_posix()


def extract(root: Path) -> list[Finding]:
    dims = dimensions_fields(root)
    colors = colors_fields(root)
    components = component_fields(root)
    findings: list[Finding] = []
    for path in source_files(root):
        cleaned_lines = strip_cfg_test_modules(
            strip_rust_comments(path.read_text(encoding="utf-8"))
        )
        relative = relpath(path, root)
        for line_number, line in enumerate(cleaned_lines, 1):
            for match in UTILITY_FIELD_RE.finditer(line):
                namespace = match.group("namespace")
                field = match.group("field")
                token = f"theme.{namespace}.{field}"
                findings.append(
                    Finding(
                        kind="utility_ref",
                        owner="theme",
                        field_or_literal=token,
                        value="token",
                        file=relative,
                        line=line_number,
                        context=token,
                        suggested_token=token,
                        verdict=(
                            "GREEN"
                            if field in UTILITY_FIELDS[namespace]
                            else "RED_UNDEFINED_TOKEN"
                        ),
                    )
                )

            for match in COMPONENT_FIELD_RE.finditer(line):
                field = f"{match.group('group')}.{match.group('field')}"
                findings.append(
                    Finding(
                        kind="component_ref",
                        owner="dims.components",
                        field_or_literal=field,
                        value="token",
                        file=relative,
                        line=line_number,
                        context=f"dims.components.{field}",
                        suggested_token=field,
                        verdict="GREEN" if field in components else "RED_UNDEFINED_TOKEN",
                    )
                )

            for match in FIELD_RE.finditer(line):
                owner = match.group("owner")
                field = match.group("field")
                known = field in (dims if owner == "dims" else colors)
                findings.append(
                    Finding(
                        kind="token_ref",
                        owner=owner,
                        field_or_literal=field,
                        value="token",
                        file=relative,
                        line=line_number,
                        context=f"{owner}.{field}",
                        suggested_token=field,
                        verdict="GREEN" if known else "RED_UNDEFINED_TOKEN",
                    )
                )

            for match in SINK_RE.finditer(line):
                value = match.group("value")
                context = match.group("context")
                if float(value) == 0.0:
                    verdict = "IGNORED_ZERO"
                else:
                    verdict = "RED_RAW_LITERAL"
                findings.append(
                    Finding(
                        kind="raw_literal",
                        owner="",
                        field_or_literal=value,
                        value=value,
                        file=relative,
                        line=line_number,
                        context=context,
                        suggested_token=token_suggestion(context, path),
                        verdict=verdict,
                    )
                )

            for match in CONST_RE.finditer(line):
                name = match.group("name")
                if not DESIGN_WORDS.search(name):
                    continue
                value = match.group("value")
                findings.append(
                    Finding(
                        kind="design_const",
                        owner="",
                        field_or_literal=name,
                        value=value,
                        file=relative,
                        line=line_number,
                        context="const",
                        suggested_token=token_suggestion("", path, name),
                        verdict=(
                            "GREEN_POLICY"
                            if name in POLICY_CONST_NAMES
                            else ("RED_RAW_LITERAL" if float(value) != 0.0 else "IGNORED_ZERO")
                        ),
                    )
                )

    return sorted(findings, key=lambda item: (item.file, item.line, item.kind, item.field_or_literal))


def write_tsv(findings: list[Finding], path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as stream:
        writer = csv.writer(stream, delimiter="\t", lineterminator="\n")
        writer.writerow(HEADERS)
        for finding in findings:
            writer.writerow(
                [
                    finding.kind,
                    finding.owner,
                    finding.field_or_literal,
                    finding.value,
                    finding.file,
                    finding.line,
                    finding.context,
                    finding.suggested_token,
                    finding.verdict,
                ]
            )


def print_findings(findings: list[Finding]) -> None:
    for finding in findings:
        print(
            f"{finding.verdict}: {finding.file}:{finding.line} "
            f"{finding.context}={finding.value} -> {finding.suggested_token}"
        )
    counts: dict[str, int] = {}
    for finding in findings:
        counts[finding.verdict] = counts.get(finding.verdict, 0) + 1
    summary = " ".join(f"{key}={counts[key]}" for key in sorted(counts))
    print(f"SUMMARY: {summary or 'GREEN=none'}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    parser.add_argument("--write", action="store_true", help="generated/design-values.tsvへ書く")
    parser.add_argument("--check", action="store_true", help="raw literal/未定義tokenで失敗する")
    args = parser.parse_args(argv)
    root = args.root.resolve()
    findings = extract(root)
    if args.write:
        write_tsv(findings, root / "next/reference/generated/design-values.tsv")
    print_findings(findings)
    if args.check and any(f.verdict.startswith("RED_") for f in findings):
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
