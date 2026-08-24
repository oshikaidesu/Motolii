#!/usr/bin/env python3
"""コンポーネントの隣に置いた契約から、意味の台帳を導出する。

コンポーネントは実装ファイルの近くに次の形式の Rust コメントを持つ。

    /* motolii-component
    id = "edit.batch_rename"
    kind = "semantic"
    weight = "core_edit"
    maps = [785]
    entry = ["BatchRenameSelectedLayers"]
    meaning = ["apply_selected"]
    evaluation = ["apply_all"]
    render = ["Timeline"]
    observable = ["auto_rename_follows_row_order_and_undoes_as_one_step"]
    */

このブロックだけが手で書く契約で、status/evidence の結果は生成する。
契約ブロック自身を証拠検索から除外するので、名前を書いただけでは緑にならない。
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path


FACETS = ("entry", "meaning", "evaluation", "render", "observable")
WEIGHTS = {
    "truth_safety": 5,
    "core_edit": 4,
    "render_export": 4,
    "fanout": 3,
    "frequency": 2,
    "portability": 2,
    "convenience": 1,
}
CONTRACT_RE = re.compile(
    r"/\*\s*motolii-component\s*\n(?P<body>.*?)\n\s*\*/", re.DOTALL
)
IDENTIFIER_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*$")


@dataclass(frozen=True)
class Contract:
    component_id: str
    kind: str
    weight: str
    maps: tuple[int, ...]
    facets: dict[str, tuple[str, ...]]
    source: str
    line: int


@dataclass(frozen=True)
class Result:
    contracts: tuple[Contract, ...]
    rows: tuple[dict[str, str], ...]
    red: tuple[str, ...]


def _parse_value(raw: str):
    try:
        return json.loads(raw)
    except json.JSONDecodeError as exc:
        raise ValueError(f"JSONとして読めない値: {raw}") from exc


def _parse_contract(match: re.Match[str], source: str, source_line: int) -> Contract:
    values: dict[str, object] = {}
    for raw_line in match.group("body").splitlines():
        stripped = raw_line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        key, separator, raw_value = stripped.partition("=")
        if not separator:
            raise ValueError(f"{source}:{source_line}: `key = value` ではない")
        key = key.strip()
        if key in values:
            raise ValueError(f"{source}: 契約キー `{key}` が重複")
        values[key] = _parse_value(raw_value.strip())

    required = {"id", "kind", "weight", "maps", *FACETS}
    missing = sorted(required - values.keys())
    if missing:
        raise ValueError(f"{source}:{source_line}: 必須キーが無い: {', '.join(missing)}")

    component_id = values["id"]
    kind = values["kind"]
    weight = values["weight"]
    maps = values["maps"]
    if not isinstance(component_id, str) or not re.fullmatch(r"[a-z][a-z0-9_.-]+", component_id):
        raise ValueError(f"{source}:{source_line}: id は kebab/snake の安定 id にする")
    if kind not in {"semantic", "adapter", "surface"}:
        raise ValueError(f"{source}:{source_line}: kind が不明: {kind!r}")
    if weight not in WEIGHTS:
        raise ValueError(f"{source}:{source_line}: weight が不明: {weight!r}")
    if not isinstance(maps, list) or any(not isinstance(item, int) for item in maps):
        raise ValueError(f"{source}:{source_line}: maps は整数配列にする")

    facets: dict[str, tuple[str, ...]] = {}
    for facet in FACETS:
        values_for_facet = values[facet]
        if (
            not isinstance(values_for_facet, list)
            or not values_for_facet
            or any(not isinstance(item, str) or not IDENTIFIER_RE.fullmatch(item) for item in values_for_facet)
        ):
            raise ValueError(f"{source}:{source_line}: {facet} は識別子の非空配列にする")
        facets[facet] = tuple(values_for_facet)

    return Contract(
        component_id=component_id,
        kind=kind,
        weight=weight,
        maps=tuple(maps),
        facets=facets,
        source=source,
        line=source_line,
    )


def _source_files(root: Path) -> list[Path]:
    paths: list[Path] = []
    next_root = root / "next"
    for directory, dirnames, filenames in os.walk(next_root):
        dirnames[:] = [name for name in dirnames if name != "target" and not name.startswith(".")]
        paths.extend(Path(directory) / name for name in filenames if name.endswith(".rs"))
    return sorted(paths)


def _load_contracts(root: Path) -> tuple[list[Contract], list[str], dict[Path, str]]:
    contracts: list[Contract] = []
    errors: list[str] = []
    bodies: dict[Path, str] = {}
    for path in _source_files(root):
        original = path.read_text(encoding="utf-8")
        bodies[path] = original
        relative = path.relative_to(root).as_posix()
        for match in CONTRACT_RE.finditer(original):
            line = original.count("\n", 0, match.start()) + 1
            try:
                contracts.append(_parse_contract(match, relative, line))
            except ValueError as exc:
                errors.append(str(exc))
    by_id: dict[str, Contract] = {}
    for contract in contracts:
        if contract.component_id in by_id:
            errors.append(
                f"component id `{contract.component_id}` が重複: "
                f"{by_id[contract.component_id].source} と {contract.source}"
            )
        by_id[contract.component_id] = contract
    return contracts, errors, bodies


def _normal_map_status(root: Path) -> dict[int, str]:
    result: dict[int, str] = {}
    path = root / "next/reference/normal-map.tsv"
    if not path.exists():
        return result
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        columns = raw_line.split("\t")
        first = columns[0]
        if first.isdigit():
            result[int(first)] = columns[12] if len(columns) > 12 else ""
    return result


def _all_code_without_contracts(bodies: dict[Path, str]) -> str:
    return "\n".join(CONTRACT_RE.sub("", body) for body in bodies.values())


def _contains_identifier(code: str, identifier: str) -> bool:
    return re.search(rf"(?<![A-Za-z0-9_]){re.escape(identifier)}(?![A-Za-z0-9_])", code) is not None


def derive(root: Path) -> Result:
    contracts, errors, bodies = _load_contracts(root)
    code = _all_code_without_contracts(bodies)
    map_status = _normal_map_status(root)
    rows: list[dict[str, str]] = []
    red = list(errors)
    seen_ids: set[str] = set()

    for contract in sorted(contracts, key=lambda item: item.component_id):
        duplicate = contract.component_id in seen_ids
        seen_ids.add(contract.component_id)
        map_reason = "local component (maps=[])"
        map_ok = True
        if contract.maps:
            missing_maps = sorted(set(contract.maps) - map_status.keys())
            pending_maps = sorted(
                map_id for map_id in set(contract.maps) & map_status.keys()
                if map_status[map_id] != "採用済"
            )
            map_ok = not missing_maps and not pending_maps
            if map_ok:
                map_reason = "normal-map が採用済"
            else:
                reasons = []
                if missing_maps:
                    reasons.append(f"無い id: {missing_maps}")
                if pending_maps:
                    reasons.append(f"採用済でない id: {pending_maps}")
                map_reason = "normal-map " + "; ".join(reasons)
        if not map_ok:
            red.append(f"{contract.component_id}: {map_reason}")
        if duplicate:
            red.append(f"{contract.component_id}: duplicate id")

        component_ok = map_ok and not duplicate
        for facet in FACETS:
            for evidence in contract.facets[facet]:
                evidence_ok = _contains_identifier(code, evidence)
                status = "緑" if evidence_ok else "赤"
                reason = "実装コードに実在" if evidence_ok else "実装コードに無い"
                if not evidence_ok:
                    component_ok = False
                    red.append(f"{contract.component_id}/{facet}: `{evidence}` が無い")
                rows.append(
                    {
                        "component": contract.component_id,
                        "kind": contract.kind,
                        "weight": contract.weight,
                        "weight_value": str(WEIGHTS[contract.weight]),
                        "maps": ",".join(str(item) for item in contract.maps),
                        "facet": facet,
                        "evidence": evidence,
                        "status": status,
                        "reason": reason,
                        "source": f"{contract.source}:{contract.line}",
                    }
                )
        if not component_ok:
            red.append(f"{contract.component_id}: component is red")

    return Result(tuple(contracts), tuple(rows), tuple(sorted(set(red))))


def render_tsv(result: Result) -> str:
    headers = [
        "component", "kind", "weight", "weight_value", "maps", "facet",
        "evidence", "status", "reason", "source",
    ]
    lines = ["\t".join(headers)]
    for row in result.rows:
        lines.append("\t".join(row[header] for header in headers))
    return "\n".join(lines) + "\n"


def render_markdown(result: Result) -> str:
    lines = [
        "# コンポーネント台帳(機械導出)",
        "",
        "実装ファイルの `motolii-component` 契約から生成。手で編集しない。",
        "赤 = 契約の粒に対応する証拠が実装コードに無い、または参照する地図行が採用済でない。",
        "",
        "| component | kind | weight | maps | entry | meaning | evaluation | render | observable | 判定 | source |",
        "|---|---|---:|---|---|---|---|---|---|---|---|",
    ]
    by_component: dict[str, list[dict[str, str]]] = {}
    for row in result.rows:
        by_component.setdefault(row["component"], []).append(row)
    for component in sorted(by_component):
        rows = by_component[component]
        values = {facet: [] for facet in FACETS}
        for row in rows:
            values[row["facet"]].append(f"{row['evidence']} {row['status']}")
        status = "緑" if all(row["status"] == "緑" for row in rows) else "赤"
        first = rows[0]
        lines.append(
            "| " + " | ".join([
                component,
                first["kind"],
                f"{first['weight']}({first['weight_value']})",
                first["maps"] or "local",
                "<br>".join(values["entry"]),
                "<br>".join(values["meaning"]),
                "<br>".join(values["evaluation"]),
                "<br>".join(values["render"]),
                "<br>".join(values["observable"]),
                status,
                first["source"],
            ]) + " |"
        )
    if not result.contracts:
        lines.append("| (契約なし) |  |  |  |  |  |  |  |  | 赤 |  |")
    return "\n".join(lines) + "\n"


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", nargs="?", default=".")
    parser.add_argument("--check", action="store_true", help="生成物が最新かだけを検査する")
    args = parser.parse_args(argv)
    root = Path(args.root).resolve()
    result = derive(root)
    generated = root / "next/reference/generated"
    generated.mkdir(parents=True, exist_ok=True)
    files = {
        generated / "components.tsv": render_tsv(result),
        generated / "components.md": render_markdown(result),
    }
    stale = []
    for path, content in files.items():
        if args.check:
            if not path.exists() or path.read_text(encoding="utf-8") != content:
                stale.append(path.relative_to(root).as_posix())
        else:
            path.write_text(content, encoding="utf-8")
    print(f"component {len(result.contracts)}件 / 粒 {len(result.rows)} / 赤 {len(result.red)}")
    for message in result.red:
        print(f"  赤: {message}")
    if stale:
        print("生成物が古い: " + ", ".join(stale))
    return 1 if result.red or stale else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
