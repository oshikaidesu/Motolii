#!/usr/bin/env python3
"""技術委託台帳の join・判定語彙・証拠を静的に検査する。"""

from __future__ import annotations

import csv
import re
import sys
from collections import Counter
from pathlib import Path


ROOT = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path.cwd()
REFERENCE = ROOT / "next/reference"
NORMAL_MAP = REFERENCE / "normal-map.tsv"
RULES = REFERENCE / "technical-delegation-rules.tsv"
LEDGER = REFERENCE / "generated/technical-delegation.tsv"

ROUTE_KINDS = {"既存構造", "上流", "先例", "移植", "外部依存", "自前最小", "未判定"}
POLICIES = {"不要", "抑制", "許容", "禁止", "未判定"}
EVIDENCE_RE = re.compile(r"(?P<path>next/[^\s;()]+):(?P<start>\d+)(?:-(?P<end>\d+))?")


def read(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as stream:
        return list(csv.DictReader(stream, delimiter="\t"))


def duplicate_ids(rows: list[dict[str, str]], field: str) -> list[str]:
    seen: set[str] = set()
    duplicates: set[str] = set()
    for row in rows:
        value = row.get(field, "")
        if value in seen:
            duplicates.add(value)
        seen.add(value)
    return sorted(duplicates)


def check_evidence(value: str) -> list[str]:
    if value == "-":
        return []
    matches = list(EVIDENCE_RE.finditer(value))
    if not matches:
        return [f"証拠の形式が読めない: {value}"]
    errors: list[str] = []
    for match in matches:
        path = ROOT / match.group("path")
        if not path.is_file():
            errors.append(f"証拠ファイルが無い: {match.group('path')}")
            continue
        with path.open(encoding="utf-8", errors="ignore") as stream:
            line_count = sum(1 for _ in stream)
        start = int(match.group("start"))
        end = int(match.group("end") or start)
        if start < 1 or end < start or end > line_count:
            errors.append(f"証拠行が無い: {match.group(0)}(現在 {line_count} 行)")
    return errors


def main() -> int:
    normal = read(NORMAL_MAP)
    rules = read(RULES)
    ledger = read(LEDGER)
    errors: list[str] = []
    normal_by_id = {row["id"]: row for row in normal}
    rule_by_id = {row["map_id"]: row for row in rules}
    ledger_by_id = {row["map_id"]: row for row in ledger}

    for label, rows, field in (("normal-map", normal, "id"), ("rules", rules, "map_id"), ("ledger", ledger, "map_id")):
        for map_id in duplicate_ids(rows, field):
            errors.append(f"{label}: map_idが重複: {map_id}")

    normal_ids = set(normal_by_id)
    if set(ledger_by_id) != normal_ids:
        errors.append(
            f"ledger join不一致: normal-map={len(normal_ids)} ledger={len(ledger_by_id)} "
            f"missing={sorted(normal_ids - set(ledger_by_id))[:8]} extra={sorted(set(ledger_by_id) - normal_ids)[:8]}"
        )

    active_ids = {
        map_id for map_id, row in normal_by_id.items() if row["verdict"] in ("採用予定", "結線待ち")
    }
    if set(rule_by_id) != active_ids:
        errors.append(f"rules coverage不一致: active={len(active_ids)} rules={len(rule_by_id)}")

    for row in ledger:
        map_id = row["map_id"]
        route = row["technical_route"]
        invalid_route = [token for token in route.split("+") if token not in ROUTE_KINDS]
        if invalid_route:
            errors.append(f"id{map_id}: technical_routeが不明: {invalid_route}")
        if row["scratch_policy"] not in POLICIES:
            errors.append(f"id{map_id}: scratch_policyが不明: {row['scratch_policy']}")
        if row["scratch_policy"] == "許容" and row["scratch_boundary"] in ("", "未監査"):
            errors.append(f"id{map_id}: 許容なのにscratch_boundaryが無い")
        if row["decision_status"] in ("技術監査済", "構造吸収から導出"):
            errors.extend(f"id{map_id}: {error}" for error in check_evidence(row["evidence"]))
        if normal_by_id.get(map_id, {}).get("verdict") == "構造吸収":
            if route != "既存構造" or row["scratch_policy"] != "不要":
                errors.append(f"id{map_id}: 構造吸収なのに既存構造/不要でない")
        if normal_by_id.get(map_id, {}).get("verdict") in ("採用予定", "結線待ち"):
            if row["decision_status"] != "技術監査済":
                errors.append(f"id{map_id}: activeなのに技術監査済でない")

    counts = Counter(row["decision_status"] for row in ledger)
    policies = Counter(row["scratch_policy"] for row in ledger if row["decision_status"] != "未監査")
    routes = Counter(row["technical_route"] for row in ledger if row["decision_status"] != "未監査")
    print(
        f"technical delegation: total={len(ledger)} audited={len(ledger)-counts['未監査']} "
        f"unreviewed={counts['未監査']} active={len(active_ids)} absorbed="
        f"{sum(1 for row in ledger if row['verdict'] == '構造吸収')}"
    )
    print("routes=" + ", ".join(f"{key}:{value}" for key, value in sorted(routes.items())))
    print("scratch=" + ", ".join(f"{key}:{value}" for key, value in sorted(policies.items())))
    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1
    print("technical delegation: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
