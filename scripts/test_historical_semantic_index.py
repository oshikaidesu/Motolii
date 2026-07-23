#!/usr/bin/env python3
"""Focused contract tests for the optional HVR-D02 Basic Memory runner."""

from __future__ import annotations

import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts/historical_semantic_index.py"
MODEL = "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2"
PROJECT = "motolii-historical-recovery"


FAKE_UVX = "#!" + sys.executable + """
import json
import os
from pathlib import Path
import sys

fixture_dir = Path(__file__).resolve().parent
record = fixture_dir / "uvx-record.json"
entries = json.loads(record.read_text()) if record.exists() else []
entries.append({"argv": sys.argv[1:], "cwd": os.getcwd(), "env": dict(os.environ)})
record.write_text(json.dumps(entries, sort_keys=True))
mutation = fixture_dir / "mutate-path"
if mutation.exists():
    Path(mutation.read_text(encoding="utf-8")).write_text("mutated", encoding="utf-8")
failure = fixture_dir / "fail-command"
if failure.exists() and failure.read_text(encoding="utf-8") in " ".join(sys.argv[1:]):
    raise SystemExit(23)
if "search-notes" in sys.argv:
    print("fake multilingual result")
"""


class HistoricalSemanticIndexTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.repo = self.root / "repo"
        self.projection = self.root / "projection"
        self.state = self.root / "state"
        self.repo.mkdir()
        (self.projection / "nodes" / "aa").mkdir(parents=True)
        (self.projection / "manifest.tsv").write_text("manifest\n", encoding="utf-8")
        (self.projection / "edges.tsv").write_text("edges\n", encoding="utf-8")
        (self.projection / "nodes" / "aa" / "node.md").write_text("日本語\n", encoding="utf-8")
        self.bin = self.root / "bin"
        self.bin.mkdir()
        fake = self.bin / "uvx"
        fake.write_text(FAKE_UVX, encoding="utf-8")
        fake.chmod(fake.stat().st_mode | stat.S_IXUSR)
        self.record = self.bin / "uvx-record.json"

    def tearDown(self) -> None:
        self.temp.cleanup()

    def invoke(self, *args: str, expect: int = 0, fake: bool = True, **extra: str) -> subprocess.CompletedProcess[str]:
        environment = {
            "PATH": os.fspath(self.bin) if fake else os.defpath,
            "INHERITED_SECRET": "must-not-reach-uvx",
            **extra,
        }
        result = subprocess.run([sys.executable, os.fspath(SCRIPT), *args], text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=environment)
        self.assertEqual(result.returncode, expect, result.stderr)
        return result

    def index(self, *extra: str, expect: int = 0, **environment: str) -> subprocess.CompletedProcess[str]:
        return self.invoke("index", "--repo-root", os.fspath(self.repo), "--projection", os.fspath(self.projection), "--state", os.fspath(self.state), *extra, expect=expect, **environment)

    def records(self) -> list[dict[str, object]]:
        return json.loads(self.record.read_text(encoding="utf-8"))

    def test_index_has_exact_commands_isolated_environment_and_deterministic_marker(self) -> None:
        tool_home = self.root / "tool-home"
        self.index(HOME=os.fspath(tool_home))
        prefix = ["--from", "basic-memory==0.22.1", "basic-memory"]
        self.assertEqual([entry["argv"] for entry in self.records()], [
            prefix + ["project", "add", PROJECT, os.fspath(self.projection.resolve()), "--local", "--default"],
            prefix + ["reindex", "--project", PROJECT, "--full"],
            prefix + ["status", "--project", PROJECT, "--wait", "--timeout", "300", "--json", "--local"],
        ])
        child_env = self.records()[0]["env"]
        self.assertEqual(self.records()[0]["cwd"], os.fspath(self.state.resolve()))
        self.assertNotIn("INHERITED_SECRET", child_env)
        self.assertEqual(child_env["HOME"], os.fspath(tool_home))
        self.assertEqual(child_env["BASIC_MEMORY_CONFIG_DIR"], os.fspath(self.state.resolve() / "config"))
        self.assertEqual(child_env["BASIC_MEMORY_AUTO_UPDATE"], "false")
        self.assertEqual(child_env["BASIC_MEMORY_SYNC_CHANGES"], "false")
        self.assertEqual(child_env["BASIC_MEMORY_ENSURE_FRONTMATTER_ON_SYNC"], "false")
        self.assertEqual(child_env["BASIC_MEMORY_SEMANTIC_SEARCH_ENABLED"], "true")
        self.assertEqual(child_env["BASIC_MEMORY_SEMANTIC_EMBEDDING_PROVIDER"], "fastembed")
        self.assertEqual(child_env["BASIC_MEMORY_SEMANTIC_EMBEDDING_MODEL"], MODEL)
        self.assertEqual(child_env["BASIC_MEMORY_SEMANTIC_EMBEDDING_CACHE_DIR"], os.fspath(self.state.resolve() / "models"))
        self.assertEqual(child_env["UV_CACHE_DIR"], os.fspath(self.state.resolve() / "uv-cache"))
        marker_text = (self.state / "hvr-index.json").read_text(encoding="utf-8")
        self.assertTrue(marker_text.endswith("\n"))
        marker = json.loads(marker_text)
        self.assertEqual(marker, {"basic_memory_version": "0.22.1", "embedding_model": MODEL, "project": PROJECT, "projection_tree_sha256": marker["projection_tree_sha256"], "schema": 1})
        self.assertEqual(list(marker), sorted(marker))
        self.assertNotIn(os.fspath(self.repo), marker_text)
        self.assertNotIn("time", marker_text)

    def test_offline_is_immediately_after_uvx(self) -> None:
        self.index("--offline")
        for entry in self.records():
            self.assertEqual(entry["argv"][:2], ["--offline", "--from"])

    def test_search_exact_passthrough_and_validation(self) -> None:
        self.index()
        result = self.invoke("search", "--repo-root", os.fspath(self.repo), "--state", os.fspath(self.state), "--query", "日本語", "--page-size", "7")
        self.assertEqual(result.stdout, "fake multilingual result\n")
        self.assertEqual(self.records()[-1]["argv"], ["--from", "basic-memory==0.22.1", "basic-memory", "tool", "search-notes", "日本語", "--hybrid", "--project", PROJECT, "--local", "--page-size", "7"])
        self.invoke("search", "--repo-root", os.fspath(self.repo), "--state", os.fspath(self.state), "--query", "", expect=1)
        self.invoke("search", "--repo-root", os.fspath(self.repo), "--state", os.fspath(self.state), "--query", "x", "--page-size", "0", expect=1)
        self.invoke("search", "--repo-root", os.fspath(self.repo), "--state", os.fspath(self.state), "--query", "x", "--page-size", "101", expect=1)
        (self.bin / "fail-command").write_text("search-notes", encoding="utf-8")
        self.invoke("search", "--repo-root", os.fspath(self.repo), "--state", os.fspath(self.state), "--query", "x", expect=23)
        (self.bin / "fail-command").unlink()
        marker = self.state / "hvr-index.json"
        marker.write_text('{"schema": 1}\n', encoding="utf-8")
        self.invoke("search", "--repo-root", os.fspath(self.repo), "--state", os.fspath(self.state), "--query", "x", expect=1)

    def test_rejects_missing_uvx_and_invalid_projection_or_boundaries(self) -> None:
        self.invoke("index", "--repo-root", os.fspath(self.repo), "--projection", os.fspath(self.projection), "--state", os.fspath(self.state), expect=1, fake=False)
        (self.projection / "manifest.tsv").unlink()
        self.index(expect=1)
        self.assertFalse((self.state / "hvr-index.json").exists())
        (self.projection / "manifest.tsv").write_text("manifest\n", encoding="utf-8")
        (self.projection / "edges.tsv").unlink()
        self.index(expect=1)
        self.assertFalse((self.state / "hvr-index.json").exists())
        (self.projection / "edges.tsv").write_text("edges\n", encoding="utf-8")
        nodes = self.projection / "nodes"
        nodes.rename(self.projection / "nodes-away")
        self.index(expect=1)
        self.assertFalse((self.state / "hvr-index.json").exists())
        (self.projection / "nodes-away").rename(nodes)
        self.invoke("index", "--repo-root", os.fspath(self.repo), "--projection", os.fspath(self.projection), "--state", os.fspath(self.repo / "state"), expect=1)
        self.assertFalse((self.repo / "state" / "hvr-index.json").exists())
        self.invoke("index", "--repo-root", os.fspath(self.repo), "--projection", os.fspath(self.projection), "--state", os.fspath(self.projection / "state"), expect=1)
        self.assertFalse((self.projection / "state" / "hvr-index.json").exists())
        repo_projection = self.repo / "projection"
        repo_projection.mkdir()
        (repo_projection / "manifest.tsv").write_text("manifest\n", encoding="utf-8")
        (repo_projection / "edges.tsv").write_text("edges\n", encoding="utf-8")
        (repo_projection / "nodes").mkdir()
        self.invoke("index", "--repo-root", os.fspath(self.repo), "--projection", os.fspath(repo_projection), "--state", os.fspath(self.state), expect=1)
        self.assertFalse((self.state / "hvr-index.json").exists())
        (self.projection / "link").symlink_to(self.projection / "manifest.tsv")
        self.index(expect=1)
        self.assertFalse((self.state / "hvr-index.json").exists())

    def test_child_failure_and_projection_mutation_leave_no_marker(self) -> None:
        (self.bin / "fail-command").write_text("reindex", encoding="utf-8")
        self.index(expect=1)
        self.assertFalse((self.state / "hvr-index.json").exists())
        (self.bin / "fail-command").unlink()
        (self.bin / "mutate-path").write_text(os.fspath(self.projection / "manifest.tsv"), encoding="utf-8")
        self.index(expect=1)
        self.assertFalse((self.state / "hvr-index.json").exists())


if __name__ == "__main__":
    unittest.main()
