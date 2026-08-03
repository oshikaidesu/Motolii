#!/usr/bin/env python3
"""Compare one lexical expansion with 3-5 read-only search hypotheses."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
import time
from typing import Iterable


EX_USAGE = 64
NON_CURRENT = ("ARCHIVED", "STOPPED", "履歴snapshot", "観察", "比較中", "棄却", "撤回", "停止")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def document_state(text: str) -> str:
    for line in text.splitlines()[:20]:
        stripped = line.strip()
        if stripped.startswith(("状態:", "状態：", "ステータス:", "ステータス：", "Status:", "STATUS:")) or "ARCHIVED" in stripped:
            value = stripped.split(":", 1)[-1].split("：", 1)[-1].replace("*", "").strip()
            for marker in NON_CURRENT:
                if marker in ("ARCHIVED", "STOPPED") and marker.casefold() in value.casefold():
                    return marker
                if value == marker or value.startswith(f"{marker} /") or f"/ {marker} /" in value:
                    return marker
            return "CURRENT"
    return "UNSPECIFIED"


def normalize_hypotheses(value: object) -> list[list[str]]:
    if not isinstance(value, list) or not 1 <= len(value) <= 5:
        raise ValueError("hypotheses must contain 1-5 term lists")
    normalized: list[list[str]] = []
    for hypothesis in value:
        if not isinstance(hypothesis, list) or not hypothesis:
            raise ValueError("each hypothesis must be a non-empty term list")
        terms = [term.strip().casefold() for term in hypothesis if isinstance(term, str) and term.strip()]
        if len(terms) != len(hypothesis):
            raise ValueError("hypothesis terms must be non-empty strings")
        normalized.append(terms)
    return normalized


def first_span(text: str, terms: Iterable[str]) -> tuple[int, str]:
    term_list = list(terms)
    best = (0, 1, "")
    for line_number, line in enumerate(text.splitlines(), 1):
        folded = line.casefold()
        hits = sum(term in folded for term in term_list)
        if hits > best[0]:
            best = (hits, line_number, line.strip())
    return best[1], best[2][:240]


def term_window(text: str, terms: list[str]) -> int:
    occurrences: list[tuple[int, str]] = []
    for line_number, line in enumerate(text.splitlines(), 1):
        folded = line.casefold()
        occurrences.extend((line_number, term) for term in terms if term in folded)
    left = 0
    counts: dict[str, int] = {}
    best = len(text.splitlines()) + 1
    for right, (line_number, term) in enumerate(occurrences):
        counts[term] = counts.get(term, 0) + 1
        while len(counts) == len(terms):
            best = min(best, line_number - occurrences[left][0])
            left_term = occurrences[left][1]
            counts[left_term] -= 1
            if counts[left_term] == 0:
                del counts[left_term]
            left += 1
    return best


def decision_links(root: Path, hypotheses: list[list[str]]) -> set[Path]:
    index = root / "docs/decision-index.md"
    if not index.is_file():
        return set()
    links: set[Path] = set()
    for line in index.read_text(encoding="utf-8").splitlines():
        folded = line.casefold()
        if not any(all(term in folded for term in hypothesis) for hypothesis in hypotheses):
            continue
        for target in re.findall(r"\[[^]]+\]\(([^)#]+)(?:#[^)]*)?\)", line):
            resolved = (index.parent / target).resolve()
            if resolved.is_file() and root.resolve() in resolved.parents:
                links.add(resolved)
    return links


def route(root: Path, hypotheses: list[list[str]], top_k: int) -> dict[str, object]:
    started = time.perf_counter()
    authority_links = decision_links(root, hypotheses)
    candidates: list[dict[str, object]] = []
    for path in sorted((root / "docs").rglob("*.md")):
        text = path.read_text(encoding="utf-8", errors="replace")
        folded = text.casefold()
        matched = [hypothesis for hypothesis in hypotheses if all(term in folded for term in hypothesis)]
        if not matched:
            continue
        state = document_state(text)
        terms = sorted({term for hypothesis in matched for term in hypothesis})
        line, excerpt = first_span(text, terms)
        current = state not in NON_CURRENT
        proximity = sum(max(0, 80 - term_window(text, hypothesis)) for hypothesis in matched)
        header = "\n".join(text.splitlines()[:12]).casefold()
        header_matches = sum(all(term in header for term in hypothesis) for hypothesis in matched)
        score = (
            (1000 if path.resolve() in authority_links else 0)
            + 150 * header_matches
            + (20 if current else 0)
            + proximity
            + 10 * len(matched)
            + len(terms)
        )
        candidates.append(
            {
                "excerpt": excerpt,
                "line": line,
                "matched_hypotheses": len(matched),
                "path": path.relative_to(root).as_posix(),
                "score": score,
                "sha256": sha256(path),
                "state": state,
            }
        )
    candidates.sort(key=lambda item: (-int(item["score"]), str(item["path"])))
    selected = candidates[:top_k]
    encoded = json.dumps(selected, ensure_ascii=False, sort_keys=True).encode()
    return {
        "candidate_count": len(candidates),
        "capsule_bytes": len(encoded),
        "elapsed_ms": round((time.perf_counter() - started) * 1000, 3),
        "results": selected,
        "state_pollution": sum(item["state"] in NON_CURRENT for item in selected),
    }


def recall(result: dict[str, object], gold: list[str]) -> tuple[int, int]:
    paths = {str(item["path"]) for item in result["results"]}  # type: ignore[index]
    return sum(path in paths for path in gold), len(gold)


def benchmark(root: Path, fixture_path: Path, top_k: int) -> dict[str, object]:
    fixture = json.loads(fixture_path.read_text(encoding="utf-8"))
    queries = fixture.get("queries") if isinstance(fixture, dict) else None
    if not isinstance(queries, list) or not queries:
        raise ValueError("fixture must contain a non-empty queries list")
    rows: list[dict[str, object]] = []
    baseline_hits = baseline_gold = multi_hits = multi_gold = 0
    for query in queries:
        if not isinstance(query, dict) or not isinstance(query.get("id"), str):
            raise ValueError("each query needs a string id")
        hypotheses = normalize_hypotheses(query.get("hypotheses"))
        if len(hypotheses) < 3:
            raise ValueError("benchmark queries need 3-5 hypotheses")
        gold = query.get("gold")
        if not isinstance(gold, list) or not gold or not all(isinstance(path, str) for path in gold):
            raise ValueError("each query needs non-empty string gold paths")
        missing_gold = [path for path in gold if not (root / path).is_file()]
        if missing_gold:
            raise ValueError(f"gold paths are missing: {missing_gold}")
        baseline = route(root, hypotheses[:1], top_k)
        multi = route(root, hypotheses, top_k)
        base_recall = recall(baseline, gold)
        multi_recall = recall(multi, gold)
        baseline_hits += base_recall[0]
        baseline_gold += base_recall[1]
        multi_hits += multi_recall[0]
        multi_gold += multi_recall[1]
        rows.append(
            {
                "baseline": {**baseline, "recall": list(base_recall)},
                "id": query["id"],
                "multi": {**multi, "recall": list(multi_recall)},
            }
        )
    return {
        "queries": rows,
        "summary": {
            "baseline_recall": [baseline_hits, baseline_gold],
            "multi_recall": [multi_hits, multi_gold],
            "query_count": len(rows),
            "top_k": top_k,
        },
    }


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--top-k", type=int, default=10)
    args = parser.parse_args(argv)
    if args.top_k <= 0:
        parser.error("--top-k must be positive")
    return args


def main(argv: list[str]) -> int:
    try:
        args = parse_args(argv)
        root = args.root.resolve()
        if not (root / "docs").is_dir():
            raise ValueError(f"docs directory is missing under {root}")
        result = benchmark(root, args.fixture.resolve(), args.top_k)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"context-route-shadow: {error}", file=sys.stderr)
        return EX_USAGE
    print(json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
