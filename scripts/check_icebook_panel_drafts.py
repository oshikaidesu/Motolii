#!/usr/bin/env python3
"""Icebook向けパネル草案の件数・ID・必須欄を静的に検査する。"""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path.cwd()
DRAFTS = ROOT / "docs/reviews/2026-08-25-icebook-panel-drafts"
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
REQUIRED = ("problem", "hero", "layout", "interaction", "density", "reuse")


def story_key(story_id: str) -> tuple[str, int] | None:
    match = re.fullmatch(r"(?P<prefix>[A-Z]{1,3})-?(?P<number>\d{2})", story_id)
    if match is None:
        return None
    return match.group("prefix"), int(match.group("number"))


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
    errors: list[str] = []
    all_ids: set[str] = set()
    counts: dict[str, int] = {}
    for panel, prefix in PANELS.items():
        path = DRAFTS / f"{panel}.md"
        if not path.is_file():
            errors.append(f"missing: {path.relative_to(ROOT)}")
            continue
        text = path.read_text(encoding="utf-8")
        matches = list(HEADING_RE.finditer(text))
        counts[panel] = len(matches)
        if len(matches) != 30:
            errors.append(f"{panel}: headings={len(matches)} (expected 30)")
        ids: list[str] = []
        for index, match in enumerate(matches):
            story_id = match.group("id")
            ids.append(story_id)
            parsed = story_key(story_id)
            if parsed is None:
                errors.append(f"{panel}: invalid story id {story_id}")
            elif parsed[0] != prefix:
                errors.append(f"{panel}: wrong prefix {story_id}")
            block_end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
            block = text[match.start() : block_end]
            fields = fields_in(block)
            for field in REQUIRED:
                if not fields.get(field):
                    errors.append(f"{panel} {story_id}: missing field {field}")
        if len(set(ids)) != len(ids):
            errors.append(f"{panel}: duplicate story id")
        parsed_ids = {story_key(story_id) for story_id in ids}
        expected = {(prefix, number) for number in range(1, 31)}
        if parsed_ids != expected:
            errors.append(f"{panel}: IDs are not {prefix}01..{prefix}30 (hyphen optional)")
        for story_id in ids:
            key = story_key(story_id)
            normalized = f"{key[0]}-{key[1]:02d}" if key else story_id
            if normalized in all_ids:
                errors.append(f"duplicate across panels: {story_id}")
            all_ids.add(normalized)

    print("icebook panel drafts: " + ", ".join(f"{key}={value}" for key, value in counts.items()))
    print(f"icebook panel drafts: total={len(all_ids)} expected={len(PANELS) * 30}")
    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1
    print("icebook panel drafts: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
