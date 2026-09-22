#!/usr/bin/env python3
from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
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

    @unittest.skipIf(os.name == "nt", "POSIX process-group regression")
    def test_timeout_kills_sigterm_ignoring_grandchild(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            pid_file = Path(temp) / "child.pid"
            child_script = (
                "trap '' TERM; "
                f"echo $$ > {subprocess.list2cmdline([os.fspath(pid_file)])}; "
                "exec sleep 30"
            )
            parent_code = (
                "import pathlib,subprocess,time;"
                f"subprocess.Popen(['/bin/sh','-c',{child_script!r}]);"
                f"path=pathlib.Path({os.fspath(pid_file)!r});"
                "deadline=time.monotonic()+5;"
                "exec('while not path.exists() and time.monotonic()<deadline:\\n time.sleep(0.01)');"
                "time.sleep(30)"
            )
            result = self.run_runner(
                "--backend", "vulkan",
                "--timeout-seconds", "1.0",
                "--", sys.executable, "-c", parent_code,
            )
            self.assertEqual(result.returncode, 124, result.stderr)
            self.assertTrue(pid_file.exists(), result.stderr)
            pid = int(pid_file.read_text())
            deadline = time.monotonic() + 2.0
            while time.monotonic() < deadline:
                try:
                    os.kill(pid, 0)
                except ProcessLookupError:
                    break
                time.sleep(0.05)
            else:
                self.fail(f"SIGTERM-ignoring child {pid} survived runner timeout")

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
