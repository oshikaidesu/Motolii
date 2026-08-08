#!/usr/bin/env python3
"""Contract tests for deterministic blind evidence envelopes."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts/check-evidence-envelope.py"


class CheckEvidenceEnvelopeTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.source = self.repo / "authority.md"
        self.selection = self.root / "selection.json"
        self.envelope = self.root / "envelope.md"

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write_selection(
        self,
        *,
        path: str = "authority.md",
        ranges: list[list[int]] | None = None,
        queries: list[dict[str, object]] | None = None,
    ) -> None:
        value = {
            "schema_version": 1,
            "sources": [
                {
                    "path": path,
                    "queries": queries or [{"literal": "top seat", "scope": [1, 3]}],
                    "ranges": ranges or [[1, 2]],
                }
            ],
        }
        self.selection.write_text(json.dumps(value, ensure_ascii=False), encoding="utf-8")

    def invoke(self, action: str, envelope: Path | None = None) -> subprocess.CompletedProcess[bytes]:
        return subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
                "--repo-root",
                str(self.repo),
                "--selection",
                str(self.selection),
                action,
                str(envelope or self.envelope),
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )

    def test_builds_computed_manifest_and_checks_exact_envelope(self) -> None:
        source_bytes = b"alpha top seat\nbeta top seat top seat\nomega\n"
        self.source.write_bytes(source_bytes)
        self.write_selection()

        built = self.invoke("--write-envelope")
        self.assertEqual(built.returncode, 0, built.stderr)
        envelope_bytes = self.envelope.read_bytes()
        self.assertIn(hashlib.sha256(source_bytes).hexdigest().encode(), envelope_bytes)
        self.assertIn(b'"columns": [\n                7', envelope_bytes)
        self.assertIn(b'"columns": [\n                6,\n                15', envelope_bytes)
        self.assertIn(source_bytes.splitlines(keepends=True)[0], envelope_bytes)
        self.assertIn(source_bytes.splitlines(keepends=True)[1], envelope_bytes)

        checked = self.invoke("--check-envelope")
        self.assertEqual(checked.returncode, 0, checked.stderr)
        self.assertIn(hashlib.sha256(envelope_bytes).hexdigest().encode(), checked.stdout)

    def test_rejects_literal_hit_outside_included_ranges(self) -> None:
        self.source.write_text("top seat one\ntop seat two\nno hit\n", encoding="utf-8")
        self.write_selection(ranges=[[1, 1]])

        result = self.invoke("--write-envelope")
        self.assertEqual(result.returncode, 65)
        self.assertIn(b"authority.md:2", result.stderr)
        self.assertFalse(self.envelope.exists())

    def test_check_rejects_source_drift(self) -> None:
        self.source.write_text("alpha top seat\nbeta top seat\nomega\n", encoding="utf-8")
        self.write_selection()
        self.assertEqual(self.invoke("--write-envelope").returncode, 0)
        self.source.write_text("ALPHA top seat\nbeta top seat\nomega\n", encoding="utf-8")

        result = self.invoke("--check-envelope")
        self.assertEqual(result.returncode, 65)
        self.assertIn(b"envelope differs from current source/selection", result.stderr)

    def test_rejects_repository_escape(self) -> None:
        (self.root / "outside.md").write_text("top seat\n", encoding="utf-8")
        self.write_selection(path="../outside.md", ranges=[[1, 1]], queries=[{"literal": "top seat", "scope": [1, 1]}])

        result = self.invoke("--write-envelope")
        self.assertEqual(result.returncode, 65)
        self.assertIn(b"normalized repository-relative", result.stderr)

    def test_rejects_symlink_source_even_when_target_stays_inside_repository(self) -> None:
        actual = self.repo / "actual.md"
        actual.write_text("top seat\n", encoding="utf-8")
        self.source.symlink_to(actual.name)
        self.write_selection(ranges=[[1, 1]], queries=[{"literal": "top seat", "scope": [1, 1]}])

        result = self.invoke("--write-envelope")
        self.assertEqual(result.returncode, 65)
        self.assertIn(b"regular non-symlink", result.stderr)

    def test_rejects_overlapping_or_unsorted_ranges(self) -> None:
        self.source.write_text("top seat\ncontext\ntop seat\n", encoding="utf-8")
        self.write_selection(ranges=[[1, 2], [2, 3]], queries=[{"literal": "top seat", "scope": [1, 3]}])

        result = self.invoke("--write-envelope")
        self.assertEqual(result.returncode, 65)
        self.assertIn(b"sorted and non-overlapping", result.stderr)

    def test_refuses_to_overwrite_an_existing_envelope(self) -> None:
        self.source.write_text("alpha top seat\nbeta top seat\nomega\n", encoding="utf-8")
        self.write_selection()
        self.assertEqual(self.invoke("--write-envelope").returncode, 0)

        result = self.invoke("--write-envelope")
        self.assertEqual(result.returncode, 73)
        self.assertIn(b"refusing to overwrite", result.stderr)


if __name__ == "__main__":
    unittest.main()
