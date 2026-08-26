#!/usr/bin/env python3
"""外部コーパスから UI 作法の候補と判定不能領域を機械抽出する。

この検査器は「普通」を主観スコアへ潰さない。外部の4製品台帳から候補を
列挙し、製品数・矛盾・問いの未収録だけを使って次の4値を返す。

* GREEN: 1候補が2製品以上に現れ、製品数で単独首位
* HOLD: 候補が複数ある、または1製品にしか現れない
* RED: 出典台帳に矛盾が明記されている
* ORACLE_GAP: 問い(特に初期値)を出典が表現していない

GREEN は Motolii の合格ではない。これは「外部候補の収束」だけを示す。
実装の合否は別の black-box probe が出した観測結果で決める。したがって
このスクリプトは Cargo や GUI の代わりではなく、候補選択を人の勘から
切り離す前段である。
"""

from __future__ import annotations

import argparse
import csv
import json
import os
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path


PRODUCT_FILES = {
    "ae": "ae.md",
    "pr": "premiere.md",
    "dr": "resolve.md",
    "cc": "capcut.md",
}
QUESTION_TERMS = {
    "behavior": (),
    "initial-default": ("default", "initial", "既定", "初期", "デフォルト", "開始時"),
}


@dataclass(frozen=True)
class Candidate:
    map_id: str
    canonical: str
    meaning: str
    products: tuple[str, ...]
    product_count: int
    official_source_tags: int
    quality: str
    contradictory: bool
    verdict_in_ledger: str


@dataclass(frozen=True)
class RawHit:
    product: str
    kind: str
    item: str
    meaning: str
    source: str


def normalize(value: str) -> str:
    value = value.casefold()
    value = re.sub(r"[^\w\u3040-\u30ff\u3400-\u9fff]+", " ", value)
    return " ".join(value.split())


def tokens(query: str) -> tuple[str, ...]:
    return tuple(token for token in normalize(query).split() if token)


def matches(query_tokens: tuple[str, ...], *values: str) -> bool:
    haystack = normalize(" ".join(values))
    return all(token in haystack for token in query_tokens)


def split_quality(value: str) -> list[str]:
    return [part.strip() for part in value.split(";") if part.strip()]


def load_candidates(root: Path, query_tokens: tuple[str, ...]) -> list[Candidate]:
    path = root / "next/reference/normal-map.tsv"
    candidates: list[Candidate] = []
    with path.open(encoding="utf-8", newline="") as handle:
        for row in csv.DictReader(handle, delimiter="\t"):
            if not row.get("id", "").isdigit():
                continue
            if not matches(query_tokens, row.get("canonical", ""), row.get("意味", "")):
                continue
            products = tuple(
                product
                for product in PRODUCT_FILES
                if row.get(product, "0") == "1"
            )
            quality = row.get("quality", "")
            candidates.append(
                Candidate(
                    map_id=row["id"],
                    canonical=row.get("canonical", ""),
                    meaning=row.get("意味", ""),
                    products=products,
                    product_count=len(products),
                    official_source_tags=sum(
                        "公式" in tag and "非公式" not in tag
                        for tag in split_quality(quality)
                    ),
                    quality=quality,
                    contradictory="矛盾" in (quality + row.get("理由", "")),
                    verdict_in_ledger=row.get("verdict", ""),
                )
            )
    return candidates


def source_aliases(text: str) -> dict[str, str]:
    aliases: dict[str, str] = {}
    for match in re.finditer(r"(?:^|- )([A-Z][A-Z0-9_]+)\s*=\s*(https?://\S+)", text, re.MULTILINE):
        aliases[match.group(1)] = match.group(2).rstrip(")>")
    return aliases


def load_raw_hits(root: Path, query_tokens: tuple[str, ...]) -> list[RawHit]:
    source_root = root / "docs/reviews/2026-08-21-normal-map-sources"
    hits: list[RawHit] = []
    for product, filename in PRODUCT_FILES.items():
        path = source_root / filename
        text = path.read_text(encoding="utf-8")
        aliases = source_aliases(text)
        for line in text.splitlines():
            columns = line.split("\t")
            if len(columns) < 5 or columns[0] not in {"menu", "shortcut", "panel", "pref"}:
                continue
            if not matches(query_tokens, columns[2], columns[3]):
                continue
            source = aliases.get(columns[4], columns[4])
            hits.append(RawHit(product, columns[0], columns[2], columns[3], source))
    return hits


def code_surface(root: Path, query_tokens: tuple[str, ...]) -> dict[str, object]:
    files: list[str] = []
    test_names: list[str] = []
    token_pattern = re.compile("|".join(re.escape(token) for token in query_tokens), re.IGNORECASE)
    function_pattern = re.compile(r"\b(?:async\s+)?fn\s+([A-Za-z0-9_]+)")
    next_root = root / "next"
    for directory, dirnames, filenames in os.walk(next_root):
        dirnames[:] = [name for name in dirnames if name not in {"target", ".git"}]
        for filename in sorted(filenames):
            if not filename.endswith(".rs"):
                continue
            path = Path(directory) / filename
            text = path.read_text(encoding="utf-8", errors="ignore")
            if not token_pattern.search(text):
                continue
            relative = path.relative_to(root).as_posix()
            files.append(relative)
            for match in function_pattern.finditer(text):
                if token_pattern.search(match.group(1)):
                    test_names.append(f"{relative}:{match.group(1)}")
    return {"files": files, "functions_with_query": test_names}


def assess(
    candidates: list[Candidate],
    raw_hits: list[RawHit],
    question: str,
) -> tuple[str, str]:
    if any(candidate.contradictory for candidate in candidates):
        return "RED", "出典台帳に矛盾が明記されている"

    if question == "initial-default":
        terms = QUESTION_TERMS[question]
        default_hits = [
            hit
            for hit in raw_hits
            if any(term in normalize(hit.item + " " + hit.meaning) for term in terms)
        ]
        if not default_hits:
            return "ORACLE_GAP", "外部コーパスに初期値/既定値を表す行がない"

    if not candidates:
        return "ORACLE_GAP", "外部コーパスに候補がない"
    maximum = max(candidate.product_count for candidate in candidates)
    leaders = [candidate for candidate in candidates if candidate.product_count == maximum]
    if maximum >= 2 and len(leaders) == 1:
        return "GREEN", "製品数で単独首位の候補が2製品以上に存在する"
    return "HOLD", "候補が同率、または1製品にしか存在しない"


def report(root: Path, query: str, question: str) -> dict[str, object]:
    query_tokens = tokens(query)
    if not query_tokens:
        raise ValueError("query が空")
    candidates = load_candidates(root, query_tokens)
    raw_hits = load_raw_hits(root, query_tokens)
    status, reason = assess(candidates, raw_hits, question)
    return {
        "query": query,
        "question": question,
        "status": status,
        "reason": reason,
        "candidates": [asdict(candidate) for candidate in candidates],
        "raw_hits": [asdict(hit) for hit in raw_hits],
        "raw_product_span": sorted({hit.product for hit in raw_hits}),
        "code_surface": code_surface(root, query_tokens),
    }


def print_report(result: dict[str, object]) -> None:
    print(f"query={result['query']} question={result['question']}")
    print(f"外部候補判定: {result['status']} — {result['reason']}")
    candidates = result["candidates"]
    if candidates:
        print("候補(製品数の降順):")
        for candidate in sorted(candidates, key=lambda item: (-item["product_count"], item["canonical"], item["map_id"])):
            print(
                f"  id={candidate['map_id']} {candidate['canonical']} "
                f"products={candidate['products']} official_tags={candidate['official_source_tags']}"
            )
    else:
        print("候補: なし")
    hits = result["raw_hits"]
    print(f"原典ヒット: {len(hits)}件 / 製品={result['raw_product_span']}")
    for hit in hits[:8]:
        print(f"  {hit['product']} {hit['item']} — {hit['source']}")
    surface = result["code_surface"]
    print(f"現行コード面(語彙一致のみ): {len(surface['files'])}ファイル")
    if surface["functions_with_query"]:
        print("関連関数:")
        for function in surface["functions_with_query"][:20]:
            print(f"  {function}")
    print("実行結果: UNEXECUTED — この検査器は実窓/black-box probeを実行しない")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", nargs="?", default=".")
    parser.add_argument("--query", required=True, help="外部コーパスを検索する抽象語")
    parser.add_argument("--question", choices=QUESTION_TERMS, default="behavior")
    parser.add_argument("--json", action="store_true", dest="as_json")
    args = parser.parse_args(argv)
    try:
        result = report(Path(args.root).resolve(), args.query, args.question)
    except (OSError, ValueError, KeyError) as exc:
        print(f"検査不能: {exc}", file=sys.stderr)
        return 2
    if args.as_json:
        print(json.dumps(result, ensure_ascii=False, indent=2))
    else:
        print_report(result)
    return 1 if result["status"] == "RED" else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
