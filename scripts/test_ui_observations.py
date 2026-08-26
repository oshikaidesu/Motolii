#!/usr/bin/env python3
"""画像解析器の決定論的な小さな柵。実窓は開かず、画像の赤/緑だけを見る。"""

from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

from PIL import Image, ImageDraw


SCRIPT = Path(__file__).with_name("analyze_ui_screenshots.py")
SPEC = importlib.util.spec_from_file_location("analyze_ui_screenshots", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class UiObservationTests(unittest.TestCase):
    def make_manifest(self, root: Path) -> Path:
        manifest = {
            "version": 1,
            "source": "synthetic",
            "analysis": {
                "regions": {
                    "window": [0, 0, 1, 1],
                    "header": [0, 0, 1, 0.2],
                },
                "checks": [
                    {"id": "window-range", "metric": "luma_p95_p05", "region": "window", "op": ">=", "value": 0.1},
                    {"id": "header-edges", "metric": "edge_density", "region": "header", "op": ">=", "value": 0.01},
                ],
            },
            "scenarios": [
                {"id": "before", "argv": [], "operation": "boot"},
                {
                    "id": "after",
                    "argv": [],
                    "operation": "toggle",
                    "reference": "before",
                    "delta": {"changed_fraction": 0.01, "mean_abs_rgb": 0.005},
                },
            ],
        }
        path = root / "manifest.json"
        path.write_text(json.dumps(manifest), encoding="utf-8")
        return path

    def draw_editor(self, path: Path, changed: bool) -> None:
        image = Image.new("RGBA", (320, 200), (28, 30, 34, 255))
        draw = ImageDraw.Draw(image)
        draw.rectangle((0, 0, 319, 38), fill=(48, 51, 58, 255))
        draw.line((12, 19, 180, 19), fill=(210, 214, 220, 255), width=2)
        draw.rectangle((12, 50, 120, 180), fill=(57, 60, 68, 255), outline=(115, 120, 130, 255))
        draw.rectangle((126, 50, 307, 145), fill=(42, 45, 51, 255), outline=(115, 120, 130, 255))
        draw.rectangle((126, 151, 307, 180), fill=(57, 60, 68, 255), outline=(115, 120, 130, 255))
        if changed:
            draw.rectangle((190, 76, 250, 120), fill=(190, 94, 80, 255))
        image.save(path)

    def test_realistic_synthetic_run_is_green_and_delta_is_measured(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = self.make_manifest(root)
            artifact = root / "run"
            (artifact / "scenarios/before").mkdir(parents=True)
            (artifact / "scenarios/after").mkdir(parents=True)
            self.draw_editor(artifact / "scenarios/before/window.png", changed=False)
            self.draw_editor(artifact / "scenarios/after/window.png", changed=True)

            result = MODULE.analyze_manifest(manifest, artifact)

            self.assertEqual(result["status"], "GREEN")
            self.assertEqual(result["passed_scenarios"], 2)
            self.assertGreater(result["scenarios"][1]["delta"]["changed_fraction"], 0.01)
            self.assertTrue((artifact / "scores.json").exists())
            self.assertTrue((artifact / "scores.tsv").exists())

    def test_flat_image_is_red(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = self.make_manifest(root)
            artifact = root / "run"
            for scenario_id in ("before", "after"):
                directory_path = artifact / "scenarios" / scenario_id
                directory_path.mkdir(parents=True)
                Image.new("RGBA", (320, 200), (30, 30, 30, 255)).save(directory_path / "window.png")

            result = MODULE.analyze_manifest(manifest, artifact)

            self.assertEqual(result["status"], "RED")
            self.assertTrue(any(not check["pass"] for check in result["scenarios"][0]["checks"]))

    def test_real_window_requires_capture_provenance(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = self.make_manifest(root)
            raw = json.loads(manifest.read_text(encoding="utf-8"))
            raw["source"] = "real-window"
            manifest.write_text(json.dumps(raw), encoding="utf-8")
            artifact = root / "run"
            directory_path = artifact / "scenarios/before"
            directory_path.mkdir(parents=True)
            self.draw_editor(directory_path / "window.png", changed=False)

            result = MODULE.analyze_manifest(manifest, artifact)

            self.assertEqual(result["status"], "RED")
            self.assertTrue(any(check["id"] == "capture-source-metadata-exists" for check in result["scenarios"][0]["checks"]))


if __name__ == "__main__":
    unittest.main()
