#!/usr/bin/env python3
"""Fast static inventory for the FrameGraph cutover.

This deliberately does not try to prove semantic parity.  It makes the remaining
legacy seams cheap to find and classifies references by cluster.  A cluster is
only deletion-ready after its FrameGraph replacement/evidence is recorded in
docs/stage5/node-migration-manifest.md and product-path references reach zero.
"""
from __future__ import annotations
import argparse, pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_ROOTS = ("motolii/crates",)
CLUSTERS = (
    ("legacy-scene-owner", r"\blayers_from_resolved\b|\bbuild_layers\b", "SceneValue / prepare_gpu_scene"),
    ("compat-resolved-projection", r"\bresolved_layers_from_scene\b|\bresolved_with_analysis\b", "SceneValue direct consumers"),
    ("resolved-layer-type", r"\bResolvedLayer\b", "SceneLayerValue / lower-level adapters"),
    ("legacy-transform-resolve", r"\b(?:local_transform3d|world_transform3d|world_transforms3d|world_transform3d_chain|world_affine|resolve_(?:local_)?transform|resolved_transform)\b", "TransformProgram / world transform nodes"),
)
TEXT_SUFFIXES={".rs",".md",".toml",".py",".sh",".yml",".yaml"}

def files(roots):
    for root in roots:
        p=ROOT/root
        if not p.exists(): continue
        for f in p.rglob("*"):
            if f.is_file() and f.suffix in TEXT_SUFFIXES and ".git" not in f.parts and "target" not in f.parts:
                yield f

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--root", action="append", dest="roots")
    ap.add_argument("--fail-on-product-owner", action="store_true",
                    help="fail when legacy scene-owner symbols occur outside tests/docs")
    ns=ap.parse_args()
    roots=tuple(ns.roots or DEFAULT_ROOTS)
    hits={name:[] for name,_,_ in CLUSTERS}
    for f in files(roots):
        rel=f.relative_to(ROOT).as_posix()
        try: lines=f.read_text(encoding="utf-8").splitlines()
        except UnicodeDecodeError: continue
        for no,line in enumerate(lines,1):
            for name,pattern,replacement in CLUSTERS:
                if re.search(pattern,line):
                    kind="oracle/test" if ("/tests/" in rel or rel.endswith("_test.rs") or "#[test]" in line) else "product/compat"
                    hits[name].append((rel,no,line.strip(),kind,replacement))
    print("# FrameGraph migration inventory")
    print()
    total=0
    for name,_,replacement in CLUSTERS:
        rows=hits[name]; total+=len(rows)
        product=sum(k=="product/compat" for *_,k,_ in rows)
        print(f"## {name}: {len(rows)} refs ({product} product/compat)")
        print(f"replacement: {replacement}")
        for rel,no,line,kind,_ in rows:
            print(f"- {kind} {rel}:{no}: {line[:180]}")
        print()
    print(f"TOTAL {total}")
    if ns.fail_on_product_owner:
        bad=[r for r in hits["legacy-scene-owner"] if r[3]=="product/compat"]
        if bad:
            print(f"legacy scene-owner still has {len(bad)} product/compat refs", file=sys.stderr)
            return 2
    return 0
if __name__=="__main__":
    raise SystemExit(main())
