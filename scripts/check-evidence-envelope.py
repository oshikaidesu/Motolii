#!/usr/bin/env python3
"""Build or verify one deterministic blind evidence envelope."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import sys
from typing import NoReturn


EX_USAGE = 64
EX_DATAERR = 65
EX_CANTCREAT = 73
SCHEMA_VERSION = 1


class EvidenceError(Exception):
    """The selection or evidence does not satisfy the closed envelope contract."""


def fail(message: str) -> NoReturn:
    raise EvidenceError(message)


def sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_object(value: object, context: str, keys: set[str]) -> dict[str, object]:
    if not isinstance(value, dict):
        fail(f"{context} must be an object")
    actual = set(value)
    if actual != keys:
        fail(f"{context} keys must be {sorted(keys)}; got {sorted(actual)}")
    return value


def require_positive_int(value: object, context: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value <= 0:
        fail(f"{context} must be a positive integer")
    return value


def require_line_pair(value: object, context: str) -> tuple[int, int]:
    if not isinstance(value, list) or len(value) != 2:
        fail(f"{context} must be [start_line, end_line]")
    start = require_positive_int(value[0], f"{context}[0]")
    end = require_positive_int(value[1], f"{context}[1]")
    if start > end:
        fail(f"{context} start_line exceeds end_line")
    return start, end


def safe_source_path(value: object, context: str) -> str:
    if not isinstance(value, str) or not value:
        fail(f"{context} must be a non-empty repository-relative path")
    path = PurePosixPath(value)
    if path.is_absolute() or path.as_posix() != value or any(part in {"", ".", ".."} for part in path.parts):
        fail(f"{context} must be a normalized repository-relative POSIX path: {value!r}")
    return value


def literal_columns(line: str, literal: str) -> list[int]:
    columns: list[int] = []
    offset = 0
    while True:
        found = line.find(literal, offset)
        if found < 0:
            return columns
        columns.append(found + 1)
        offset = found + 1


def source_file(repo_root: Path, relative: str) -> Path:
    candidate = repo_root.joinpath(*PurePosixPath(relative).parts)
    try:
        resolved = candidate.resolve(strict=True)
        resolved.relative_to(repo_root)
    except (FileNotFoundError, RuntimeError, ValueError):
        fail(f"source escapes the repository or does not exist: {relative}")
    if resolved != candidate or candidate.is_symlink() or not resolved.is_file():
        fail(f"source must be a regular non-symlink file at its exact repository path: {relative}")
    return resolved


def build_manifest(repo_root: Path, selection_path: Path) -> tuple[dict[str, object], list[tuple[str, int, int, bytes]]]:
    try:
        selection_bytes = selection_path.read_bytes()
        selection_value = json.loads(selection_bytes.decode("utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"cannot read UTF-8 JSON selection: {error}")

    selection = require_object(selection_value, "selection", {"schema_version", "sources"})
    if selection["schema_version"] != SCHEMA_VERSION:
        fail(f"selection.schema_version must be {SCHEMA_VERSION}")
    sources_value = selection["sources"]
    if not isinstance(sources_value, list) or not sources_value:
        fail("selection.sources must be a non-empty array")

    seen_paths: set[str] = set()
    manifest_sources: list[dict[str, object]] = []
    excerpts: list[tuple[str, int, int, bytes]] = []

    for source_index, source_value in enumerate(sources_value):
        context = f"selection.sources[{source_index}]"
        source = require_object(source_value, context, {"path", "queries", "ranges"})
        relative = safe_source_path(source["path"], f"{context}.path")
        if relative in seen_paths:
            fail(f"duplicate source path: {relative}")
        seen_paths.add(relative)

        path = source_file(repo_root, relative)
        raw = path.read_bytes()
        try:
            text_lines = [line.decode("utf-8") for line in raw.splitlines(keepends=True)]
        except UnicodeDecodeError as error:
            fail(f"source is not UTF-8: {relative}: {error}")
        raw_lines = raw.splitlines(keepends=True)
        line_count = len(raw_lines)

        ranges_value = source["ranges"]
        if not isinstance(ranges_value, list) or not ranges_value:
            fail(f"{context}.ranges must be a non-empty array")
        ranges: list[tuple[int, int]] = []
        manifest_ranges: list[dict[str, object]] = []
        previous_end = 0
        for range_index, range_value in enumerate(ranges_value):
            start, end = require_line_pair(range_value, f"{context}.ranges[{range_index}]")
            if end > line_count:
                fail(f"{context}.ranges[{range_index}] exceeds {relative} line count {line_count}")
            if start <= previous_end:
                fail(f"{context}.ranges must be sorted and non-overlapping")
            previous_end = end
            ranges.append((start, end))
            excerpt = b"".join(raw_lines[start - 1 : end])
            excerpts.append((relative, start, end, excerpt))
            manifest_ranges.append(
                {
                    "end_line": end,
                    "ends_with_line_break": excerpt.endswith((b"\n", b"\r")),
                    "sha256": sha256(excerpt),
                    "start_line": start,
                }
            )

        queries_value = source["queries"]
        if not isinstance(queries_value, list) or not queries_value:
            fail(f"{context}.queries must be a non-empty array")
        seen_queries: set[tuple[str, int, int]] = set()
        manifest_queries: list[dict[str, object]] = []
        for query_index, query_value in enumerate(queries_value):
            query_context = f"{context}.queries[{query_index}]"
            query = require_object(query_value, query_context, {"literal", "scope"})
            literal = query["literal"]
            if not isinstance(literal, str) or not literal or "\n" in literal or "\r" in literal:
                fail(f"{query_context}.literal must be one non-empty line")
            scope_start, scope_end = require_line_pair(query["scope"], f"{query_context}.scope")
            if scope_end > line_count:
                fail(f"{query_context}.scope exceeds {relative} line count {line_count}")
            query_key = (literal, scope_start, scope_end)
            if query_key in seen_queries:
                fail(f"duplicate literal query in {relative}: {query_key!r}")
            seen_queries.add(query_key)

            hits: list[dict[str, object]] = []
            for line_number in range(scope_start, scope_end + 1):
                columns = literal_columns(text_lines[line_number - 1], literal)
                if not columns:
                    continue
                if not any(start <= line_number <= end for start, end in ranges):
                    fail(
                        f"literal hit is not covered by an included range: "
                        f"{relative}:{line_number}: {literal!r}"
                    )
                hits.append({"columns": columns, "line": line_number})
            manifest_queries.append(
                {
                    "hits": hits,
                    "literal": literal,
                    "scope": {"end_line": scope_end, "start_line": scope_start},
                }
            )

        manifest_sources.append(
            {
                "line_count": line_count,
                "path": relative,
                "queries": manifest_queries,
                "ranges": manifest_ranges,
                "sha256": sha256(raw),
            }
        )

    manifest: dict[str, object] = {
        "schema_version": SCHEMA_VERSION,
        "selection_sha256": sha256(selection_bytes),
        "sources": manifest_sources,
    }
    return manifest, excerpts


def build_envelope(manifest: dict[str, object], excerpts: list[tuple[str, int, int, bytes]]) -> bytes:
    manifest_json = json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True).encode("utf-8")
    output = bytearray(
        b"# Blind evidence envelope\n\n"
        b"This artifact is generated from repository bytes. The manifest proves only the recorded literal-query scopes.\n\n"
        b"## Manifest\n\n```json\n"
    )
    output.extend(manifest_json)
    output.extend(b"\n```\n\n## Exact source ranges\n\n")
    for relative, start, end, excerpt in excerpts:
        excerpt_hash = sha256(excerpt)
        marker = f"path={relative} lines={start}-{end} sha256={excerpt_hash}".encode("utf-8")
        output.extend(b"<!-- BEGIN EXACT SOURCE ")
        output.extend(marker)
        output.extend(b" -->\n")
        output.extend(excerpt)
        if not excerpt.endswith((b"\n", b"\r")):
            output.extend(b"\n")
        output.extend(b"<!-- END EXACT SOURCE ")
        output.extend(marker)
        output.extend(b" -->\n\n")
    return bytes(output)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", required=True, type=Path, help="Repository root containing selected sources")
    parser.add_argument("--selection", required=True, type=Path, help="UTF-8 JSON source/range/literal-query selection")
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--write-envelope", type=Path, help="Create a new deterministic envelope; refuses overwrite")
    action.add_argument("--check-envelope", type=Path, help="Verify an existing envelope byte-for-byte")
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    try:
        args = parse_args(argv)
    except SystemExit as error:
        return EX_USAGE if error.code else 0

    try:
        repo_root = args.repo_root.resolve(strict=True)
        if not repo_root.is_dir():
            fail(f"repo root is not a directory: {repo_root}")
        manifest, excerpts = build_manifest(repo_root, args.selection)
        expected = build_envelope(manifest, excerpts)

        if args.write_envelope is not None:
            output_path: Path = args.write_envelope
            if not output_path.parent.is_dir():
                fail(f"envelope parent directory does not exist: {output_path.parent}")
            try:
                with output_path.open("xb") as stream:
                    stream.write(expected)
            except FileExistsError:
                print(f"check-evidence-envelope: refusing to overwrite: {output_path}", file=sys.stderr)
                return EX_CANTCREAT
            print(f"OK: envelope_sha256={sha256(expected)} bytes={len(expected)} path={output_path}")
            return 0

        envelope_path: Path = args.check_envelope
        try:
            actual = envelope_path.read_bytes()
        except OSError as error:
            fail(f"cannot read envelope: {error}")
        if actual != expected:
            mismatch = next(
                (index for index, (left, right) in enumerate(zip(actual, expected)) if left != right),
                min(len(actual), len(expected)),
            )
            fail(
                f"envelope differs from current source/selection at byte {mismatch}; "
                f"expected_bytes={len(expected)} actual_bytes={len(actual)}"
            )
        print(f"OK: envelope_sha256={sha256(actual)} bytes={len(actual)} path={envelope_path}")
        return 0
    except EvidenceError as error:
        print(f"check-evidence-envelope: {error}", file=sys.stderr)
        return EX_DATAERR
    except OSError as error:
        print(f"check-evidence-envelope: filesystem error: {error}", file=sys.stderr)
        return EX_DATAERR


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
