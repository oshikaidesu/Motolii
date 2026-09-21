#!/usr/bin/env python3
from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import unittest


ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts/gpu-test.py"


class GpuTestRunnerTest(unittest.TestCase):
    def run_runner(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, os.fspath(SCRIPT), *args],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )

    def test_sets_requested_backend_and_adapter(self) -> None:
        code = "import os; print(os.environ.get('WGPU_BACKEND')); print(os.environ.get('WGPU_ADAPTER_NAME'))"
        result = self.run_runner(
            "--backend", "metal",
            "--adapter", "probe-adapter",
            "--timeout-seconds", "5",
            "--", sys.executable, "-c", code,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.splitlines(), ["metal", "probe-adapter"])

    def test_timeout_is_a_distinct_failure_instead_of_a_hang(self) -> None:
        result = self.run_runner(
            "--backend", "vulkan",
            "--timeout-seconds", "0.2",
            "--", sys.executable, "-c", "import time; time.sleep(30)",
        )
        self.assertEqual(result.returncode, 124, result.stderr)
        self.assertIn("timeout", result.stderr.lower())


if __name__ == "__main__":
    unittest.main()
