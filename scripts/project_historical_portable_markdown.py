#!/usr/bin/env python3
"""Project the fixed historical Git corpus into portable Markdown nodes."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
import tempfile


EVIDENCE = Path("docs/reviews/evidence/historical-value-recovery")
CORPUS_HEADER = ("blob_sha", "bytes", "observed_path")
RECEIPT_HEADER = (
    "blob_sha",
    "source_scope",
    "disposition_document",
    "publication",
)
MANIFEST_HEADER = (
    "blob_sha",
    "bytes",
    "observed_path",
    "node_path",
    "coverage",
    "receipt_file",
    "source_scope",
    "disposition_document",
    "publication",
)
EDGES_HEADER = ("source_id", "relation", "target_kind", "target_id")
SHA_RE = re.compile(r"^[0-9a-f]{40}$")


class ProjectionError(Exception):
    """An input or output violates the fixed projection contract."""


def reject_control(value: str, label: str) -> None:
    if any(character in value for character in "\t\r\n"):
        raise ProjectionError(f"{label} contains tab, CR, or LF")


def read_tsv(path: Path, header: tuple[str, ...]) -> list[tuple[str, ...]]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as error:
        raise ProjectionError(f"cannot read {path}: {error}") from error
    except UnicodeDecodeError as error:
        raise ProjectionError(f"{path} is not UTF-8: {error}") from error
    if not lines or tuple(lines[0].split("\t")) != header:
        raise ProjectionError(f"invalid header in {path}")
    rows: list[tuple[str, ...]] = []
    for number, line in enumerate(lines[1:], start=2):
        fields = tuple(line.split("\t"))
        if len(fields) != len(header):
            raise ProjectionError(f"invalid column count in {path}:{number}")
        for name, value in zip(header, fields):
            reject_control(value, f"{path}:{number} {name}")
        rows.append(fields)
    return rows


def read_corpus(path: Path) -> dict[str, tuple[int, str]]:
    corpus: dict[str, tuple[int, str]] = {}
    for number, (blob_sha, byte_count, observed_path) in enumerate(
        read_tsv(path, CORPUS_HEADER), start=2
    ):
        if not SHA_RE.fullmatch(blob_sha):
            raise ProjectionError(f"invalid blob SHA in {path}:{number}")
        if not byte_count.isascii() or not byte_count.isdecimal():
            raise ProjectionError(f"invalid byte count in {path}:{number}")
        if blob_sha in corpus:
            raise ProjectionError(f"duplicate blob SHA in corpus: {blob_sha}")
        corpus[blob_sha] = (int(byte_count), observed_path)
    return corpus


def read_receipts(receipts_dir: Path, corpus: dict[str, tuple[int, str]], repo_root: Path) -> dict[str, tuple[str, str, str, str]]:
    if not receipts_dir.is_dir():
        raise ProjectionError(f"missing receipts directory: {receipts_dir}")
    receipts: dict[str, tuple[str, str, str, str]] = {}
    for receipt_path in sorted(receipts_dir.glob("*.tsv"), key=lambda path: path.as_posix()):
        try:
            receipt_file = receipt_path.relative_to(repo_root).as_posix()
        except ValueError as error:
            raise ProjectionError(f"receipt is outside repo: {receipt_path}") from error
        for number, (blob_sha, source_scope, document, publication) in enumerate(
            read_tsv(receipt_path, RECEIPT_HEADER), start=2
        ):
            if not SHA_RE.fullmatch(blob_sha):
                raise ProjectionError(f"invalid receipt SHA in {receipt_path}:{number}")
            if blob_sha not in corpus:
                raise ProjectionError(f"receipt contains unknown corpus SHA: {blob_sha}")
            if blob_sha in receipts:
                raise ProjectionError(f"blob assigned by more than one receipt: {blob_sha}")
            receipts[blob_sha] = (receipt_file, source_scope, document, publication)
    return receipts


def git_blob(repo_root: Path, blob_sha: str) -> bytes:
    result = subprocess.run(
        ["git", "-C", os.fspath(repo_root), "cat-file", "blob", blob_sha],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise ProjectionError(f"cannot read Git blob {blob_sha}: {detail}")
    try:
        result.stdout.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ProjectionError(f"Git blob is not UTF-8: {blob_sha}") from error
    return result.stdout


def quoted(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def node_bytes(
    blob_sha: str,
    byte_count: int,
    observed_path: str,
    cutoff_hash: str,
    receipt: tuple[str, str, str, str] | None,
    body: bytes,
) -> bytes:
    lines = [
        "---",
        f"title: {quoted(f'{observed_path} @ {blob_sha[:12]}')}",
        "type: historical_blob",
        f"permalink: {quoted(f'motolii-history/blob/{blob_sha}')}",
        f"motolii_blob_sha: {quoted(blob_sha)}",
        f"motolii_bytes: {byte_count}",
        f"motolii_observed_path: {quoted(observed_path)}",
        f"motolii_cutoff_manifest_sha256: {quoted(cutoff_hash)}",
    ]
    if receipt is not None:
        receipt_file, source_scope, document, publication = receipt
        lines.extend(
            (
                f"motolii_receipt_file: {quoted(receipt_file)}",
                f"motolii_receipt_source_scope: {quoted(source_scope)}",
                f"motolii_disposition_document: {quoted(document)}",
                f"motolii_publication: {quoted(publication)}",
            )
        )
    return ("\n".join(lines) + "\n---\n").encode("utf-8") + body


def write_tsv(path: Path, header: tuple[str, ...], rows: list[tuple[str, ...]]) -> None:
    for row in rows:
        for value in row:
            reject_control(value, f"output value for {path.name}")
    content = "\t".join(header) + "\n"
    content += "".join("\t".join(row) + "\n" for row in rows)
    path.write_bytes(content.encode("utf-8"))


def validate_paths(repo_root: Path, out: Path) -> None:
    if not out.is_absolute():
        raise ProjectionError("--out must be an absolute path")
    if os.path.lexists(out):
        raise ProjectionError("--out must not exist when projection starts")
    resolved_repo = repo_root.resolve()
    resolved_out = out.resolve(strict=False)
    if os.path.commonpath((os.fspath(resolved_repo), os.fspath(resolved_out))) == os.fspath(resolved_repo):
        raise ProjectionError("--out must be outside --repo-root")
    if not out.parent.is_dir():
        raise ProjectionError(f"output parent does not exist: {out.parent}")


def project(repo_root: Path, out: Path) -> None:
    validate_paths(repo_root, out)
    evidence = repo_root / EVIDENCE
    corpus = read_corpus(evidence / "corpus.tsv")
    receipts = read_receipts(evidence / "disposition-receipts", corpus, repo_root)
    try:
        cutoff_hash = hashlib.sha256((evidence / "cutoff-refs.tsv").read_bytes()).hexdigest()
    except OSError as error:
        raise ProjectionError(f"cannot read cutoff manifest: {error}") from error

    prepared: list[tuple[str, int, str, tuple[str, str, str, str] | None, bytes]] = []
    for blob_sha in sorted(corpus):
        byte_count, observed_path = corpus[blob_sha]
        body = git_blob(repo_root, blob_sha)
        if len(body) != byte_count:
            raise ProjectionError(f"corpus byte count disagrees with Git blob: {blob_sha}")
        prepared.append((blob_sha, byte_count, observed_path, receipts.get(blob_sha), body))

    temporary = Path(tempfile.mkdtemp(prefix=f".{out.name}.tmp-", dir=out.parent))
    try:
        manifest_rows: list[tuple[str, ...]] = []
        edges: list[tuple[str, str, str, str]] = []
        for blob_sha, byte_count, observed_path, receipt, body in prepared:
            node_relative = PurePosixPath("nodes", blob_sha[:2], f"{blob_sha}.md")
            node_path = temporary / Path(*node_relative.parts)
            node_path.parent.mkdir(parents=True, exist_ok=True)
            node_path.write_bytes(node_bytes(blob_sha, byte_count, observed_path, cutoff_hash, receipt, body))
            if receipt is None:
                coverage = "remaining"
                receipt_values = ("", "", "", "")
            else:
                coverage = "disposed"
                receipt_values = receipt
            manifest_rows.append(
                (blob_sha, str(byte_count), observed_path, node_relative.as_posix(), coverage, *receipt_values)
            )
            source = f"blob:{blob_sha}"
            edges.append((source, "observed_path", "path", observed_path))
            if receipt is not None:
                receipt_file, _scope, document, publication = receipt
                edges.extend(
                    (
                        (source, "receipt", "receipt", receipt_file),
                        (source, "disposition_document", "document", document),
                    )
                )
                if publication:
                    edges.append((source, "publication", "publication", publication))
        write_tsv(temporary / "manifest.tsv", MANIFEST_HEADER, manifest_rows)
        edges.sort(key=lambda row: tuple(value.encode("utf-8") for value in row))
        write_tsv(temporary / "edges.tsv", EDGES_HEADER, edges)
        os.replace(temporary, out)
    except Exception:
        shutil.rmtree(temporary, ignore_errors=True)
        raise


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    try:
        project(args.repo_root, args.out)
    except ProjectionError as error:
        print(f"projection failed: {error}", file=sys.stderr)
        return 1
    except OSError as error:
        print(f"projection failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
