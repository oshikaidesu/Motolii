#!/usr/bin/env python3
"""Static ownership inventory for the FrameGraph/GPU cutover.

This scanner is deliberately syntax-light.  It does not prove semantic parity;
it finds places where product code still appears to *discover work* that should
already have been expressed as nodes/resources/edges.
"""
from __future__ import annotations

import argparse
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_ROOTS = ("motolii/crates",)
TEXT_SUFFIXES = {".rs", ".md", ".toml", ".py", ".sh", ".yml", ".yaml"}

# Symbol clusters: cheap inventory of known legacy seams.
SYMBOL_CLUSTERS = (
    ("legacy-scene-owner", r"\blayers_from_resolved\b|\bbuild_layers\b|\bprepare_gpu_scene\b|\bgpu_executable_scene(?:_with_solver)?\b",
     "per-contribution GPU resources + relation lowering + scene root"),
    ("compat-resolved-projection", r"\bresolved_layers_from_scene\b|\bresolved_with_analysis\b",
     "SceneValue direct consumers"),
    ("resolved-layer-type", r"\bResolvedLayer\b", "SceneLayerValue / lower-level adapters"),
    ("legacy-transform-resolve",
     r"\blocal_transform3d\b|\bworld_transform3d\b|\bworld_transforms3d\b|\bworld_transform3d_chain\b|\bworld_affine\b|\bresolve_(?:local_)?transform\b|\bresolved_transform\b",
     "TransformProgram local/world values"),
    ("gpu-scene-boundary", r"\bGpuSceneValue\b|\bLayerWithPasses\b",
     "logical GPU resources; LayerWithPasses only below explicit execution primitives"),
)

# Behavioral smells: these are stronger than a legacy symbol because they detect
# a second planner even after names have changed.  They are warnings by default.
BEHAVIOR_CLUSTERS = (
    ("semantic-relation-rediscovery",
     r"\.(?:matte|clip_to_below|masks|effects|after_effects)\b",
     "relation/effect meaning should be consumed by semantic adapter/lowering, not rediscovered by execution owners"),
    ("layer-index-rediscovery",
     r"HashMap\s*<[^>]*LayerId|HashSet\s*<[^>]*LayerId|by_id|base_index|source_index",
     "cross-contribution dependencies should already be resource edges"),
    ("whole-scene-vector-mutation",
     r"Vec\s*<\s*LayerWithPasses\s*>|layers\s*\[.*\]\s*=|removed\s*=\s*vec!\[false",
     "executor should consume a plan, not mutate a scene vector to discover survivors"),
    ("backend-semantic-read",
     r"SceneValue|SceneLayerValue",
     "concrete gpu_exec backends should prefer resource-keyed payloads"),
)

# Places allowed to interpret semantic values.  Everything below gpu_exec is
# intentionally stricter: adapter/lowerer may read semantics; concrete backends
# should execute already-decided work.
SEMANTIC_OWNER_ALLOW = (
    "motolii/crates/motolii-render/src/gpu_exec/semantic_adapter.rs",
    "motolii/crates/motolii-render/src/gpu_exec/logical_lowerer.rs",
    "motolii/crates/motolii-render/src/gpu_exec/lowerer.rs",
)
LOW_LEVEL_ALLOW = (
    "motolii/crates/motolii-render/src/gpu_exec/content_backend.rs",
    "motolii/crates/motolii-render/src/gpu_exec/placement_backend.rs",
    "motolii/crates/motolii-render/src/gpu_exec/effect_backend.rs",
)

def files(roots):
    for root in roots:
        path = ROOT / root
        if not path.exists():
            continue
        for file in path.rglob("*"):
            if (
                file.is_file()
                and file.suffix in TEXT_SUFFIXES
                and ".git" not in file.parts
                and "target" not in file.parts
            ):
                yield file

def is_test_or_doc(rel: str) -> bool:
    return (
        "/tests/" in rel
        or rel.endswith("_test.rs")
        or rel.endswith("_tests.rs")
        or rel.startswith("docs/")
        or rel.endswith(".md")
    )

def classify(rel: str, cluster: str) -> str:
    if is_test_or_doc(rel):
        return "oracle/test"
    if rel in SEMANTIC_OWNER_ALLOW:
        return "semantic-owner"
    if rel in LOW_LEVEL_ALLOW:
        return "allowed-adapter"
    if rel.startswith("motolii/crates/motolii-render/src/gpu_exec/"):
        if cluster in {
            "semantic-relation-rediscovery",
            "layer-index-rediscovery",
            "whole-scene-vector-mutation",
            "backend-semantic-read",
        }:
            return "product-owner-smell"
    return "product/compat"

def scan(roots):
    clusters = SYMBOL_CLUSTERS + BEHAVIOR_CLUSTERS
    hits = {name: [] for name, _, _ in clusters}
    for file in files(roots):
        rel = file.relative_to(ROOT).as_posix()
        try:
            lines = file.read_text(encoding="utf-8").splitlines()
        except UnicodeDecodeError:
            continue
        for no, line in enumerate(lines, 1):
            # Skip comments for behavioral patterns; symbol inventory still
            # includes them because migration docs/comments can reveal stale seams.
            code = line.split("//", 1)[0]
            for name, pattern, replacement in SYMBOL_CLUSTERS:
                if re.search(pattern, line):
                    hits[name].append((rel, no, line.strip(), classify(rel, name), replacement))
            for name, pattern, replacement in BEHAVIOR_CLUSTERS:
                if code and re.search(pattern, code):
                    hits[name].append((rel, no, line.strip(), classify(rel, name), replacement))
    return hits

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", action="append", dest="roots")
    parser.add_argument(
        "--fail-on-product-owner",
        action="store_true",
        help="fail when the known legacy whole-scene owner remains in product code",
    )
    parser.add_argument(
        "--fail-on-gpu-planner-smell",
        action="store_true",
        help="fail when concrete gpu_exec code rediscovers semantic/cross-layer work",
    )
    args = parser.parse_args()
    roots = tuple(args.roots or DEFAULT_ROOTS)
    hits = scan(roots)

    print("# FrameGraph migration inventory\n")
    total = 0
    for name, _, replacement in SYMBOL_CLUSTERS + BEHAVIOR_CLUSTERS:
        rows = hits[name]
        total += len(rows)
        product = sum(kind in {"product/compat", "product-owner-smell"} for *_, kind, _ in rows)
        print(f"## {name}: {len(rows)} refs ({product} product)")
        print(f"replacement: {replacement}")
        for rel, no, line, kind, _ in rows:
            print(f"- {kind} {rel}:{no}: {line[:180]}")
        print()
    print(f"TOTAL {total}")

    failed = False
    if args.fail_on_product_owner:
        bad = [
            row for row in hits["legacy-scene-owner"]
            if row[3] in {"product/compat", "product-owner-smell"}
        ]
        if bad:
            print(f"legacy scene-owner still has {len(bad)} product refs", file=sys.stderr)
            failed = True

    if args.fail_on_gpu_planner_smell:
        smell_clusters = {
            "semantic-relation-rediscovery",
            "layer-index-rediscovery",
            "whole-scene-vector-mutation",
            "backend-semantic-read",
        }
        bad = [
            (name, row)
            for name in smell_clusters
            for row in hits[name]
            if row[3] == "product-owner-smell"
        ]
        if bad:
            print(f"gpu execution layer still has {len(bad)} planner/discovery smells", file=sys.stderr)
            for name, row in bad:
                print(f"- {name}: {row[0]}:{row[1]}: {row[2][:160]}", file=sys.stderr)
            failed = True

    return 2 if failed else 0

if __name__ == "__main__":
    raise SystemExit(main())
