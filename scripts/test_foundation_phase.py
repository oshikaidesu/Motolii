import json
import tempfile
import unittest
from pathlib import Path

from foundation_phase import FoundationPhaseError, load_phase


def manifest_root(root: Path) -> Path:
    path = root / "next/reference/foundation"
    path.mkdir(parents=True)
    return path / "phase.json"


class FoundationPhaseTest(unittest.TestCase):
    def base_manifest(self) -> dict:
        return {
            "schema_version": 1,
            "current_stage": "serial_a",
            "mode": "serial",
            "parallel_components": "locked",
            "parallel_stage": "parallel",
            "parallel_unlock_requires_closed": ["serial_a"],
            "stages": [
                {
                    "id": "serial_a",
                    "kind": "serial",
                    "status": "open",
                    "depends_on": [],
                },
                {
                    "id": "parallel",
                    "kind": "parallel",
                    "status": "locked",
                    "depends_on": ["serial_a"],
                },
            ],
        }

    def test_current_serial_manifest_is_valid(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest_root(root).write_text(json.dumps(self.base_manifest()), encoding="utf-8")
            phase = load_phase(root)
            self.assertEqual(phase["parallel_components"], "locked")

    def test_unlocked_parallel_requires_closed_prerequisite(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = self.base_manifest()
            manifest["current_stage"] = "parallel"
            manifest["mode"] = "parallel"
            manifest["parallel_components"] = "unlocked"
            manifest["stages"][1]["status"] = "open"
            manifest_root(root).write_text(json.dumps(manifest), encoding="utf-8")
            with self.assertRaises(FoundationPhaseError):
                load_phase(root)

    def test_dependency_cycle_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = self.base_manifest()
            manifest["stages"][0]["depends_on"] = ["parallel"]
            manifest_root(root).write_text(json.dumps(manifest), encoding="utf-8")
            with self.assertRaises(FoundationPhaseError):
                load_phase(root)


if __name__ == "__main__":
    unittest.main()
