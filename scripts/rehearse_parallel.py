#!/usr/bin/env python3
"""write-set の責任分離を同時実行でリハーサルする。

実装は変更しない。各意味レーンが所有ファイルを同時に読み、専用の一時領域へ
完了マーカーを書くことで、(1) write-set の交差が無いこと、(2) WIRE が意味
レーンへ漏れていないこと、(3) 全レーンが同時に完了できることを検査する。
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import re
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path


SPECIAL_LANES = {"(外部依存)", "(責任ファイル未記入)"}
PATH_SEPARATOR = re.compile(r"[;,]")


def parse_paths(value: str) -> set[str]:
    return {
        path.strip()
        for path in PATH_SEPARATOR.split(value)
        if path.strip() and path.strip() != "-"
    }


def resolve_path(root: Path, relative: str) -> Path:
    for candidate in (root / relative, root / "next" / relative):
        if candidate.is_file():
            return candidate
    raise FileNotFoundError(relative)


def load_lanes(root: Path) -> dict[str, set[str]]:
    worklist = root / "next/reference/generated/worklist.tsv"
    with worklist.open(newline="") as stream:
        rows = csv.DictReader(stream, delimiter="\t")
        lanes: dict[str, set[str]] = {}
        for row in rows:
            lane = row["lane"]
            if lane in SPECIAL_LANES:
                continue
            lanes.setdefault(lane, set()).update(parse_paths(row["semantic-write-set"]))
    return lanes


def load_wire_files(root: Path) -> set[str]:
    next_root = root / "next"
    return {
        str(path.relative_to(root))
        for path in next_root.rglob("*.rs")
        if "//! responsibility: wire" in path.read_text(errors="replace")
    }


def assert_disjoint(lanes: dict[str, set[str]], wire_files: set[str]) -> None:
    names = sorted(lanes)
    collisions: list[tuple[str, str, list[str]]] = []
    for index, left in enumerate(names):
        for right in names[index + 1 :]:
            overlap = sorted(lanes[left] & lanes[right])
            if overlap:
                collisions.append((left, right, overlap))
    if collisions:
        raise RuntimeError(f"semantic write-set collision(s): {collisions}")

    leaked = sorted(path for owned in lanes.values() for path in owned if path in wire_files)
    if leaked:
        raise RuntimeError(f"WIRE file leaked into semantic write-set: {leaked}")


def rehearse(root: Path) -> dict[str, int]:
    lanes = load_lanes(root)
    wire_files = load_wire_files(root)
    assert_disjoint(lanes, wire_files)
    names = sorted(lanes)

    with tempfile.TemporaryDirectory(prefix="motolii-parallel-rehearsal-") as temporary:
        marker_root = Path(temporary)

        def run_lane(lane: str) -> tuple[str, int]:
            for relative in sorted(lanes[lane]):
                hashlib.sha256(resolve_path(root, relative).read_bytes()).digest()
            (marker_root / f"{names.index(lane)}.ok").write_text(f"{lane}\n")
            return lane, len(lanes[lane])

        with ThreadPoolExecutor(max_workers=len(names)) as pool:
            results = list(pool.map(run_lane, names))
        markers = list(marker_root.glob("*.ok"))

    return {
        "parallel_workers": len(names),
        "lanes_completed": len(results),
        "owned_files": sum(count for _, count in results),
        "write_set_pair_collisions": 0,
        "wire_files_excluded": len(wire_files),
        "isolated_markers": len(markers),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    for key, value in rehearse(args.root.resolve()).items():
        print(f"{key}={value}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
