#!/usr/bin/env python3
"""Contract tests for the read-only context routing shadow."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts/context-route-shadow.py"
SPEC = importlib.util.spec_from_file_location("context_route_shadow", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class ContextRouteShadowTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        (self.root / "docs").mkdir()
        (self.root / "docs/decision-index.md").write_text(
            "| alpha current | summary | 決定 | [alpha](alpha.md) | target |\n",
            encoding="utf-8",
        )
        (self.root / "docs/alpha.md").write_text(
            "# Alpha\n\n状態: **決定**\n\nalpha current owner\n",
            encoding="utf-8",
        )
        (self.root / "docs/beta.md").write_text(
            "# Beta\n\n状態: **決定**\n\nbeta alternate test\n",
            encoding="utf-8",
        )
        (self.root / "docs/old.md").write_text(
            "# Old\n\n状態: **ARCHIVED**\n\nalpha current old\n",
            encoding="utf-8",
        )

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_multiple_hypotheses_recover_gold_and_emit_provenance(self) -> None:
        fixture = self.root / "fixture.json"
        fixture.write_text(
            json.dumps(
                {
                    "queries": [
                        {
                            "id": "q1",
                            "hypotheses": [
                                ["alpha", "current"],
                                ["beta", "alternate"],
                                ["missing", "third"],
                            ],
                            "gold": ["docs/alpha.md", "docs/beta.md"],
                        }
                    ]
                }
            ),
            encoding="utf-8",
        )
        before = {path: path.read_bytes() for path in self.root.rglob("*.md")}
        result = MODULE.benchmark(self.root, fixture, 3)
        row = result["queries"][0]
        self.assertEqual(row["baseline"]["recall"], [1, 2])
        self.assertEqual(row["multi"]["recall"], [2, 2])
        alpha = next(item for item in row["multi"]["results"] if item["path"] == "docs/alpha.md")
        self.assertEqual(alpha["state"], "CURRENT")
        self.assertGreater(alpha["line"], 0)
        self.assertEqual(len(alpha["sha256"]), 64)
        self.assertEqual(before, {path: path.read_bytes() for path in self.root.rglob("*.md")})

    def test_non_current_candidate_is_flagged_and_ranked_after_current_authority(self) -> None:
        result = MODULE.route(self.root, [["alpha", "current"]], 3)
        paths = [item["path"] for item in result["results"]]
        self.assertLess(paths.index("docs/alpha.md"), paths.index("docs/old.md"))
        old = next(item for item in result["results"] if item["path"] == "docs/old.md")
        self.assertEqual(old["state"], "ARCHIVED")
        self.assertEqual(result["state_pollution"], 1)

    def test_rejects_invalid_hypothesis_shape(self) -> None:
        with self.assertRaisesRegex(ValueError, "1-5"):
            MODULE.normalize_hypotheses([])
        with self.assertRaisesRegex(ValueError, "non-empty"):
            MODULE.normalize_hypotheses([[""]])

    def test_term_window_prefers_clustered_evidence(self) -> None:
        self.assertEqual(MODULE.term_window("alpha beta\nother\n", ["alpha", "beta"]), 0)
        self.assertEqual(MODULE.term_window("alpha\nother\nbeta\n", ["alpha", "beta"]), 2)

    def test_status_word_inside_current_status_is_not_reclassified(self) -> None:
        text = "# Map\n\n状態: **施工前コンパイル正本 / 停止simulation反映**\n"
        self.assertEqual(MODULE.document_state(text), "CURRENT")

    def test_historical_snapshot_is_flagged(self) -> None:
        text = "# Old map\n\n状態: **履歴snapshot / oracle来歴**\n"
        self.assertEqual(MODULE.document_state(text), "履歴snapshot")


if __name__ == "__main__":
    unittest.main()
