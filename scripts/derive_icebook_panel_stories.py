#!/usr/bin/env python3
"""パネル草案MarkdownをIcebook story索引へ変換する。"""

from __future__ import annotations

import csv
import re
import sys
from pathlib import Path


ROOT = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path.cwd()
DRAFTS = ROOT / "docs/reviews/2026-08-25-icebook-panel-drafts"
OUTPUT = ROOT / "next/reference/generated/icebook-panel-stories.tsv"
PANELS = {
    "browser": "B",
    "inspector": "I",
    "stage": "ST",
    "timeline": "T",
    "export": "E",
    "settings": "S",
}
HEADING_RE = re.compile(
    r"^##+\s+(?P<id>[A-Z]{1,3}-?\d{2})"
    r"(?:\s+[—-]\s*|\s+)(?P<name>.+?)\s*$",
    re.MULTILINE,
)
FIELD_RE = re.compile(
    r"^[ \t]*(?:[-*][ \t]+)?(?P<label>[^:：\r\n]+?)"
    r"[ \t]*[:：][ \t]*(?P<value>[^\r\n]+?)\s*$",
    re.MULTILINE,
)
FIELDS = ("problem", "hero", "layout", "interaction", "density", "reuse")


def field_name(label: str) -> str | None:
    normalized = label.strip().strip("*`").strip().casefold()
    normalized = re.sub(r"\s+", " ", normalized)
    if normalized in {"problem", "problem solved", "解決する問題"}:
        return "problem"
    if normalized.startswith("hero"):
        return "hero"
    if normalized.startswith("layout") or normalized.startswith("レイアウト"):
        return "layout"
    if normalized.startswith("interaction"):
        return "interaction"
    if normalized.startswith("density"):
        return "density"
    if normalized.startswith("reuse"):
        return "reuse"
    return None


def fields_in(block: str) -> dict[str, str]:
    fields: dict[str, str] = {}
    for match in FIELD_RE.finditer(block):
        name = field_name(match.group("label"))
        if name is not None:
            fields.setdefault(name, match.group("value").strip())
    return fields


def main() -> int:
    rows: list[dict[str, str]] = []
    errors: list[str] = []
    for panel, prefix in PANELS.items():
        path = DRAFTS / f"{panel}.md"
        if not path.is_file():
            errors.append(f"missing: {path.relative_to(ROOT)}")
            continue
        text = path.read_text(encoding="utf-8")
        matches = list(HEADING_RE.finditer(text))
        for index, match in enumerate(matches):
            end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
            block = text[match.start():end]
            values: dict[str, str] = {}
            for match_field in FIELD_RE.finditer(block):
                name = field_name(match_field.group("label"))
                if name is not None:
                    values.setdefault(name, match_field.group("value").strip())
            story_id = match.group("id")
            parsed_prefix = re.match(r"[A-Z]{1,3}", story_id)
            if parsed_prefix is None or parsed_prefix.group(0) != prefix:
                errors.append(f"{panel}: wrong id {story_id}")
            missing = [field for field in FIELDS if not values.get(field)]
            if missing:
                errors.append(f"{story_id}: missing {','.join(missing)}")
            rows.append({"story_id": story_id, "panel": panel, "name": match.group("name"), **values})

    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = ["story_id", "panel", "name", *FIELDS]
    with OUTPUT.open("w", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=fieldnames, delimiter="\t", lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
    print(f"icebook stories: {len(rows)} -> {OUTPUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
