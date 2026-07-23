#!/usr/bin/env python3
"""Focused contract tests for HVR-D01 portable Markdown projection."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts/project_historical_portable_markdown.py"
EVIDENCE_RELATIVE = Path("docs/reviews/evidence/historical-value-recovery")
FIXTURES = ROOT / "scripts/testdata/hvr-d01"


class ProjectionTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.repo = Path(self.temp.name) / "repo"
        self.repo.mkdir()
        self.git("init", "-q")
        (self.repo / EVIDENCE_RELATIVE / "disposition-receipts").mkdir(parents=True)
        self.write_tsv("corpus.tsv", ("blob_sha", "bytes", "observed_path"), ())
        self.write_tsv("cutoff-refs.tsv", ("ref", "commit"), (("refs/test", "0" * 40),))

    def tearDown(self) -> None:
        self.temp.cleanup()

    def git(self, *args: str, input: bytes | None = None) -> bytes:
        return subprocess.run(
            ["git", "-C", os.fspath(self.repo), *args], input=input, stdout=subprocess.PIPE, check=True
        ).stdout

    def blob(self, body: bytes) -> str:
        return self.git("hash-object", "-w", "--stdin", input=body).decode().strip()

    def write_tsv(self, name: str, header: tuple[str, ...], rows: tuple[tuple[str, ...], ...]) -> None:
        path = self.repo / EVIDENCE_RELATIVE / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("\t".join(header) + "\n" + "".join("\t".join(row) + "\n" for row in rows), encoding="utf-8")

    def project(self, out: Path, expect: int = 0) -> subprocess.CompletedProcess[str]:
        result = subprocess.run(
            [sys.executable, os.fspath(SCRIPT), "--repo-root", os.fspath(self.repo), "--out", os.fspath(out)],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(result.returncode, expect, result.stderr)
        return result

    def fixture(self) -> tuple[str, str, str]:
        japanese = self.blob((FIXTURES / "japanese.md").read_bytes())
        empty = self.blob(b"")
        front_matter = self.blob((FIXTURES / "front-matter-looking.md").read_bytes())
        self.write_tsv(
            "corpus.tsv",
            ("blob_sha", "bytes", "observed_path"),
            (
                (japanese, "16", "docs/日本語.md"),
                (empty, "0", "docs/empty.md"),
                (front_matter, "28", "docs/front.md"),
            ),
        )
        self.write_tsv(
            "disposition-receipts/one.tsv",
            ("blob_sha", "source_scope", "disposition_document", "publication"),
            ((japanese, "日本語 scope", "decision.md", "PR-1"),),
        )
        return japanese, empty, front_matter

    def test_projection_preserves_bodies_receipts_and_closed_edges(self) -> None:
        japanese, empty, front_matter = self.fixture()
        out = Path(self.temp.name) / "out"
        self.project(out)
        for blob_sha in (japanese, empty, front_matter):
            node = (out / "nodes" / blob_sha[:2] / f"{blob_sha}.md").read_bytes()
            self.assertEqual(node.split(b"---\n", 2)[-1], self.git("cat-file", "blob", blob_sha))
        manifest = (out / "manifest.tsv").read_text(encoding="utf-8").splitlines()
        disposed = next(line.split("\t") for line in manifest if line.startswith(japanese))
        remaining = next(line.split("\t") for line in manifest if line.startswith(empty))
        self.assertEqual(disposed[4:], ["disposed", "docs/reviews/evidence/historical-value-recovery/disposition-receipts/one.tsv", "日本語 scope", "decision.md", "PR-1"])
        self.assertEqual(remaining[4:], ["remaining", "", "", "", ""])
        node = (out / "nodes" / japanese[:2] / f"{japanese}.md").read_text(encoding="utf-8")
        self.assertIn('motolii_receipt_source_scope: "日本語 scope"', node)
        self.assertNotIn("motolii_receipt_file", (out / "nodes" / empty[:2] / f"{empty}.md").read_text(encoding="utf-8"))
        edges = (out / "edges.tsv").read_text(encoding="utf-8").splitlines()[1:]
        relations = {line.split("\t")[1] for line in edges}
        self.assertEqual(relations, {"observed_path", "receipt", "disposition_document", "publication"})
        self.assertEqual(edges, sorted(edges, key=lambda line: tuple(value.encode() for value in line.split("\t"))))
        projection_text = (out / "manifest.tsv").read_text(encoding="utf-8") + "\n".join(edges)
        for forbidden in ("Basic Memory", "adopts", "rejects", "supersedes", "implements"):
            self.assertNotIn(forbidden, projection_text)

    def test_same_input_has_identical_tree_hash(self) -> None:
        self.fixture()
        first = Path(self.temp.name) / "first"
        second = Path(self.temp.name) / "second"
        self.project(first)
        self.project(second)
        self.assertEqual(self.tree_hash(first), self.tree_hash(second))

    def test_rejects_non_utf8_blob_without_output(self) -> None:
        blob_sha = self.blob(b"\xff")
        self.write_tsv("corpus.tsv", ("blob_sha", "bytes", "observed_path"), ((blob_sha, "1", "docs/bad.md"),))
        out = Path(self.temp.name) / "out"
        self.project(out, expect=1)
        self.assertFalse(out.exists())

    def test_rejects_existing_and_repository_output_paths(self) -> None:
        self.fixture()
        existing = Path(self.temp.name) / "existing"
        existing.mkdir()
        self.project(existing, expect=1)
        inside = self.repo / "projected"
        self.project(inside, expect=1)
        self.assertFalse(inside.exists())

    def test_rejects_unknown_or_duplicate_receipt_sha_without_output(self) -> None:
        blob_sha, _, _ = self.fixture()
        unknown = "f" * 40
        self.write_tsv(
            "disposition-receipts/invalid.tsv",
            ("blob_sha", "source_scope", "disposition_document", "publication"),
            ((unknown, "scope", "doc", ""),),
        )
        out = Path(self.temp.name) / "out"
        self.project(out, expect=1)
        self.assertFalse(out.exists())
        (self.repo / EVIDENCE_RELATIVE / "disposition-receipts/invalid.tsv").unlink()
        self.write_tsv(
            "disposition-receipts/duplicate.tsv",
            ("blob_sha", "source_scope", "disposition_document", "publication"),
            ((blob_sha, "scope", "doc", ""),),
        )
        self.project(out, expect=1)
        self.assertFalse(out.exists())

    def test_rejects_tsv_control_character_without_output(self) -> None:
        blob_sha = self.blob(b"ok")
        evidence = self.repo / EVIDENCE_RELATIVE
        (evidence / "corpus.tsv").write_bytes(
            b"blob_sha\tbytes\tobserved_path\n" + blob_sha.encode() + b"\t2\tdocs/bad\tpath.md\n"
        )
        out = Path(self.temp.name) / "out"
        self.project(out, expect=1)
        self.assertFalse(out.exists())

    def test_real_corpus_has_fixed_counts_exact_bodies_and_closed_edges(self) -> None:
        with tempfile.TemporaryDirectory(prefix="motolii-hvr-d01-test-") as directory:
            out = Path(directory) / "projection"
            result = subprocess.run(
                [sys.executable, os.fspath(SCRIPT), "--repo-root", os.fspath(ROOT), "--out", os.fspath(out)],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            manifest = (out / "manifest.tsv").read_text(encoding="utf-8").splitlines()[1:]
            self.assertEqual(len(manifest), 1797)
            self.assertEqual(sum(row.split("\t")[4] == "disposed" for row in manifest), 420)
            self.assertEqual(sum(row.split("\t")[4] == "remaining" for row in manifest), 1377)
            self.assertEqual(len({row.split("\t")[0] for row in manifest}), 1797)
            for row in manifest:
                fields = row.split("\t")
                self.assertEqual(
                    (out / fields[3]).read_bytes().split(b"---\n", 2)[-1],
                    subprocess.check_output(["git", "-C", os.fspath(ROOT), "cat-file", "blob", fields[0]]),
                )
            relations = {
                row.split("\t")[1]
                for row in (out / "edges.tsv").read_text(encoding="utf-8").splitlines()[1:]
            }
            self.assertEqual(relations, {"observed_path", "receipt", "disposition_document", "publication"})

    @staticmethod
    def tree_hash(root: Path) -> str:
        digest = hashlib.sha256()
        for path in sorted(root.rglob("*")):
            if path.is_file():
                digest.update(path.relative_to(root).as_posix().encode() + b"\0" + path.read_bytes())
        return digest.hexdigest()


if __name__ == "__main__":
    unittest.main()
