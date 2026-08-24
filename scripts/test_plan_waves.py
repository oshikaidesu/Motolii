import csv
import subprocess
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("plan_waves.py")


class PlanWavesWireTest(unittest.TestCase):
    def test_wire_file_does_not_join_semantic_lanes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            axis = root / "next/reference/axis"
            generated = root / "next/reference/generated"
            wire = root / "next/shell/motolii-shell/src/lib.rs"
            semantic_a = root / "next/ui/example-a/src/lib.rs"
            semantic_b = root / "next/ui/example-b/src/lib.rs"
            semantic_c = root / "next/ui/example-c/src/lib.rs"
            axis.mkdir(parents=True)
            generated.mkdir(parents=True)
            wire.parent.mkdir(parents=True)
            for path in (semantic_a, semantic_b, semantic_c):
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("// semantic\n", encoding="utf-8")
            wire.write_text("//! responsibility: wire\n", encoding="utf-8")

            (root / "next/reference/normal-map.tsv").write_text(
                "id\tcanonical\tmeaning\textra\n", encoding="utf-8"
            )
            (axis / "A01-entry.tsv").write_text(
                "axis\tmap_id\ttarget\tverdict\t理由\t判定\t責任\n"
                "A01\t-\twire only\t未確認\t\t穴\tnext/shell/motolii-shell/src/lib.rs\n"
                "A01\t-\tsemantic a\t未確認\t\t穴\tnext/ui/example-a/src/lib.rs;next/shell/motolii-shell/src/lib.rs\n"
                "A01\t-\tsemantic b\t未確認\t\t穴\tnext/ui/example-b/src/lib.rs\n"
                "A01\t-\tsemantic c\t未確認\t\t穴\tnext/ui/example-b/src/lib.rs;next/ui/example-c/src/lib.rs\n"
                "A01\t-\texternal\t未確認\t\t穴\tfork側の上流APIに依存\n",
                encoding="utf-8",
            )

            result = subprocess.run(
                ["python3", str(SCRIPT), str(root)],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("意味レーン 2", result.stdout)
            self.assertIn("WIRE結線 1", result.stdout)

            with (generated / "worklist.tsv").open(encoding="utf-8", newline="") as stream:
                rows = list(csv.DictReader(stream, delimiter="\t"))
            by_target = {row["対象"]: row for row in rows}
            self.assertEqual(by_target["wire only"]["lane"], "(WIRE結線)")
            self.assertEqual(by_target["semantic a"]["lane"], "ui/example-a/src/lib.rs")
            self.assertEqual(
                by_target["semantic a"]["wire-set"],
                "shell/motolii-shell/src/lib.rs",
            )
            self.assertEqual(by_target["semantic b"]["lane"], "ui/example-b/src/lib.rs")
            self.assertEqual(by_target["semantic c"]["lane"], "ui/example-b/src/lib.rs")
            self.assertEqual(by_target["external"]["lane"], "(外部依存)")


if __name__ == "__main__":
    unittest.main()
