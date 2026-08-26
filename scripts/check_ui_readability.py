#!/usr/bin/env python3
"""機械可読の色契約とUI寸法の外出しを検査する。"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


def color_at(root: dict, path: str) -> list[float]:
    node = root["color"]
    for part in path.split("."):
        node = node[part]
    return [float(value) for value in node["$value"]["components"][:3]]


def luminance(rgb: list[float]) -> float:
    linear = []
    for component in rgb:
        linear.append(
            component / 12.92
            if component <= 0.04045
            else ((component + 0.055) / 1.055) ** 2.4
        )
    return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2]


def contrast(foreground: list[float], background: list[float]) -> float:
    fg = luminance(foreground)
    bg = luminance(background)
    return (max(fg, bg) + 0.05) / (min(fg, bg) + 0.05)


def source_violations(root: Path) -> list[str]:
    """UI chromeの非ゼロLength::Fixedを見つける。

    padding(0)や作品データの色は意味上の値なので対象外。寸法トークンに
    できるのは、画面レイアウトの固定寸法だけに限定する。
    """

    pattern = re.compile(r"Length::Fixed\(\s*([0-9]+(?:\.[0-9]+)?)\s*\)")
    violations: list[str] = []
    roots = [root / "next/ui", root / "next/shell/motolii-shell/src"]
    for source_root in roots:
        if not source_root.exists():
            continue
        for path in sorted(source_root.rglob("*.rs")):
            if "tests" in path.parts or "vendored" in path.parts:
                continue
            for line_number, line in enumerate(path.read_text().splitlines(), 1):
                for match in pattern.finditer(line):
                    if float(match.group(1)) != 0.0:
                        violations.append(
                            f"{path.relative_to(root)}:{line_number}: "
                            f"Length::Fixed({match.group(1)}) は dimensions.json へ移す"
                        )
    return violations


def interaction_violations(policy: dict, dimensions: dict) -> list[str]:
    """共通の操作対象床とフォーカス境界を機械的に検査する。"""
    interaction = policy["interaction"]
    minimum = float(interaction["minimum_target_px"])
    focus_minimum = float(interaction["focus_indicator_width_px"])
    target_min = float(dimensions["interactive_target_min"])
    focus_width = float(dimensions["focus_indicator_width"])
    failures: list[str] = []

    if target_min + 1e-9 < minimum:
        failures.append(f"interactive_target_min: {target_min:.1f}px < {minimum:.1f}px")
    else:
        print(f"GREEN: interactive_target_min = {target_min:.1f}px")
    if focus_width + 1e-9 < focus_minimum:
        failures.append(f"focus_indicator_width: {focus_width:.1f}px < {focus_minimum:.1f}px")
    else:
        print(f"GREEN: focus_indicator_width = {focus_width:.1f}px")

    targets = {
        "inspector_glyph": (
            float(dimensions["inspector_glyph_width"]),
            max(float(dimensions["inspector_row_height"]) - float(dimensions["spacing_xs"]), target_min),
        ),
        "browser_view_mode": (target_min, target_min),
        "timeline_transport": (
            float(dimensions["timeline_transport_button_width"]),
            float(dimensions["timeline_transport_height"]),
        ),
    }
    for name in interaction["targets"]:
        width, height = targets[name]
        if min(width, height) + 1e-9 < minimum:
            failures.append(f"{name}: {width:.1f}x{height:.1f}px < {minimum:.1f}px target floor")
        else:
            print(f"GREEN: {name} = {width:.1f}x{height:.1f}px")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    root = args.root.resolve()
    colors_path = root / "ui/motolii-tokens/sources/motolii-dark.json"
    policy_path = root / "next/ui/motolii-tokens-rs/tokens/readability.json"
    dimensions_path = root / "next/ui/motolii-tokens-rs/tokens/dimensions.json"

    try:
        colors = json.loads(colors_path.read_text())
        policy = json.loads(policy_path.read_text())
        dimensions = json.loads(dimensions_path.read_text())
    except (OSError, json.JSONDecodeError, KeyError) as error:
        print(f"RED: token JSON を読めない: {error}")
        return 1

    failures: list[str] = []
    thresholds = policy["thresholds"]
    for role, pair in policy["roles"].items():
        threshold_key = (
            "normal_text_contrast_min"
            if role == "normal_text"
            else "non_text_contrast_min"
        )
        threshold = float(thresholds[threshold_key])
        for foreground_name in pair["foreground"]:
            for background_name in pair["background"]:
                value = contrast(
                    color_at(colors, foreground_name), color_at(colors, background_name)
                )
                label = f"{foreground_name} on {background_name}"
                if value + 1e-9 < threshold:
                    failures.append(
                        f"{label}: {value:.3f}:1 < {threshold:.1f}:1 ({role})"
                    )
                else:
                    print(f"GREEN: {label} = {value:.3f}:1 ({role})")

    failures.extend(source_violations(root))
    failures.extend(interaction_violations(policy, dimensions))
    for failure in failures:
        print(f"RED: {failure}")
    if failures:
        print(f"SUMMARY: RED={len(failures)}")
        return 1
    print("SUMMARY: GREEN=all readability pairs and layout dimensions")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
