#!/usr/bin/env python3
"""normal-map の意味粒に、技術の委託先とスクラッチ境界を結ぶ。"""

from __future__ import annotations

import csv
import re
import sys
from pathlib import Path


ROOT = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path.cwd()
REFERENCE = ROOT / "next/reference"
NORMAL_MAP = REFERENCE / "normal-map.tsv"
RULES = REFERENCE / "technical-delegation-rules.tsv"
OUTPUT = REFERENCE / "generated/technical-delegation.tsv"

OUTPUT_FIELDS = [
    "map_id",
    "verdict",
    "scope",
    "canonical",
    "meaning",
    "problem",
    "bundle",
    "technical_route",
    "technical_delegate",
    "scratch_policy",
    "scratch_boundary",
    "judgment",
    "evidence",
    "decision_status",
]


def read_tsv(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as stream:
        return list(csv.DictReader(stream, delimiter="\t"))


def segment(reason: str, key: str) -> str:
    match = re.search(
        rf"(?:^|;\s*){re.escape(key)}:(.*?)(?=;\s*(?:CAUSAL|PROBLEM|STRUCTURE|OUTCOME|ABSORB):|$)",
        reason,
    )
    return match.group(1).strip() if match else ""


def first_evidence(text: str) -> str:
    match = re.search(r"next/[^\s;)]+:\d+(?:-\d+)?", text)
    return match.group(0) if match else "-"


def absorbed_record(row: dict[str, str]) -> dict[str, str]:
    reason = row["理由"]
    causal = segment(reason, "CAUSAL") or "因果未記入"
    problem = segment(reason, "PROBLEM") or row["意味"]
    structure = segment(reason, "STRUCTURE") or "既存構造(証拠未記入)"
    outcome = segment(reason, "OUTCOME") or "既存経路の観測結果"
    absorbed = segment(reason, "ABSORB") or "独立粒"
    return {
        "technical_route": "既存構造",
        "technical_delegate": structure,
        "scratch_policy": "不要",
        "scratch_boundary": "独立したowner・状態・検収を増やさない",
        "judgment": f"{causal}: {problem} は {structure} で {outcome} に到達する。{absorbed} は独立実装しない",
        "evidence": first_evidence(structure),
        "decision_status": "構造吸収から導出",
        "problem": problem,
    }


def undecided_record(row: dict[str, str]) -> dict[str, str]:
    return {
        "technical_route": "未判定",
        "technical_delegate": "未判定",
        "scratch_policy": "未判定",
        "scratch_boundary": "未監査",
        "judgment": "意味の在庫判定はあるが、技術の委託先とスクラッチ境界は未監査",
        "evidence": "-",
        "decision_status": "未監査",
        "problem": row["意味"],
    }


def main() -> int:
    normal_rows = read_tsv(NORMAL_MAP)
    rule_rows = read_tsv(RULES)
    normal_by_id = {row["id"]: row for row in normal_rows if row.get("id", "").isdigit()}
    rules_by_id: dict[str, dict[str, str]] = {}
    errors: list[str] = []

    for rule in rule_rows:
        map_id = rule.get("map_id", "")
        if not map_id.isdigit():
            errors.append(f"rules: map_id が整数でない: {map_id!r}")
        if map_id in rules_by_id:
            errors.append(f"rules: map_id が重複: {map_id}")
        rules_by_id[map_id] = rule
        if map_id not in normal_by_id:
            errors.append(f"rules: normal-map に無い map_id: {map_id}")

    active_ids = {
        map_id
        for map_id, row in normal_by_id.items()
        if row["verdict"] in ("採用予定", "結線待ち")
    }
    missing_rules = sorted(active_ids - set(rules_by_id))
    extra_rules = sorted(set(rules_by_id) - active_ids)
    if missing_rules:
        errors.append(f"rules: active粒の技術判定が無い: {','.join(missing_rules)}")
    if extra_rules:
        errors.append(f"rules: activeでないmap_idを直接判定している: {','.join(extra_rules)}")

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    with OUTPUT.open("w", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=OUTPUT_FIELDS, delimiter="\t", lineterminator="\n")
        writer.writeheader()
        for row in normal_rows:
            map_id = row["id"]
            if map_id in rules_by_id:
                technical = {
                    key: rules_by_id[map_id][key]
                    for key in (
                        "technical_route",
                        "technical_delegate",
                        "scratch_policy",
                        "scratch_boundary",
                        "judgment",
                        "evidence",
                    )
                }
                technical["decision_status"] = "技術監査済"
                technical["problem"] = segment(row["理由"], "PROBLEM") or row["意味"]
            elif row["verdict"] == "構造吸収":
                technical = absorbed_record(row)
            else:
                technical = undecided_record(row)
            writer.writerow(
                {
                    "map_id": map_id,
                    "verdict": row["verdict"],
                    "scope": row["scope"],
                    "canonical": row["canonical"],
                    "meaning": row["意味"],
                    "problem": technical["problem"],
                    "bundle": row["bundle"],
                    **{key: technical[key] for key in OUTPUT_FIELDS[7:]},
                }
            )

    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1

    active_audited = sum(1 for map_id in active_ids if map_id in rules_by_id)
    absorbed = sum(1 for row in normal_rows if row["verdict"] == "構造吸収")
    print(f"technical delegation: active={active_audited} absorbed={absorbed} total={len(normal_rows)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
