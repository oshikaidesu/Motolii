#!/usr/bin/env python3
"""Contract tests for the transport-only observed CLI harness."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest


ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts/run-observed-cli.py"
PYTHON = Path(sys.executable).resolve()


class RunObservedCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.cwd = self.root / "cwd"
        self.cwd.mkdir()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def invoke(
        self,
        name: str,
        code: str,
        *child_args: str,
        timeout: str | None = None,
        grace: str = "0.2",
        heartbeat: str = "10",
    ) -> subprocess.CompletedProcess[bytes]:
        command = [
            sys.executable,
            os.fspath(SCRIPT),
            "--cwd",
            os.fspath(self.cwd),
            "--log-dir",
            os.fspath(self.root / name),
            "--grace-seconds",
            grace,
            "--heartbeat-seconds",
            heartbeat,
        ]
        if timeout is not None:
            command.extend(["--timeout-seconds", timeout])
        command.extend(["--", os.fspath(PYTHON), "-c", code, *child_args])
        return subprocess.run(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )

    def metadata(self, name: str) -> dict[str, object]:
        return json.loads((self.root / name / "meta.json").read_text(encoding="utf-8"))

    def test_preserves_exact_argv_raw_streams_status_and_hashes(self) -> None:
        result = self.invoke(
            "success",
            "import os,sys; print(os.getcwd()); print(sys.argv[1]); sys.stderr.buffer.write(b'raw-stderr\\x00\\n')",
            "argument with spaces",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        expected_stdout = (os.fspath(self.cwd.resolve()) + "\nargument with spaces\n").encode()
        expected_stderr = b"raw-stderr\x00\n"
        self.assertEqual(result.stdout, expected_stdout)
        self.assertEqual(result.stderr, expected_stderr)
        self.assertEqual((self.root / "success" / "stdout.log").read_bytes(), expected_stdout)
        self.assertEqual((self.root / "success" / "stderr.log").read_bytes(), expected_stderr)
        metadata = self.metadata("success")
        self.assertEqual(metadata["argv"][-1], "argument with spaces")
        self.assertEqual(metadata["cwd"], os.fspath(self.cwd.resolve()))
        self.assertEqual(metadata["exit_code"], 0)
        self.assertIsNone(metadata["signal"])
        self.assertFalse(metadata["timed_out"])
        self.assertEqual(metadata["stdout_sha256"], hashlib.sha256(expected_stdout).hexdigest())
        self.assertEqual(metadata["stderr_sha256"], hashlib.sha256(expected_stderr).hexdigest())
        events = [json.loads(line)["event"] for line in (self.root / "success" / "lifecycle.jsonl").read_text().splitlines()]
        self.assertEqual(events, ["started", "completed"])

    def test_preserves_nonzero_child_exit(self) -> None:
        result = self.invoke("failure", "import sys; print('failed'); raise SystemExit(23)")
        self.assertEqual(result.returncode, 23)
        self.assertEqual(self.metadata("failure")["exit_code"], 23)

    def test_heartbeat_reports_live_process_and_stream_progress_without_ending_it(self) -> None:
        result = self.invoke(
            "heartbeat",
            "import sys,time; sys.stdout.write('thinking\\n'); sys.stdout.flush(); time.sleep(0.18)",
            heartbeat="0.05",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        lifecycle = [json.loads(line) for line in (self.root / "heartbeat" / "lifecycle.jsonl").read_text().splitlines()]
        heartbeats = [event for event in lifecycle if event["event"] == "heartbeat"]
        self.assertGreaterEqual(len(heartbeats), 1)
        first = heartbeats[0]
        self.assertTrue(first["pid_alive"])
        self.assertGreaterEqual(first["elapsed_ms"], 0)
        self.assertGreaterEqual(first["idle_ms"], 0)
        self.assertGreaterEqual(first["stdout_bytes"], len(b"thinking\n"))
        self.assertEqual(first["stderr_bytes"], 0)
        self.assertEqual(lifecycle[0]["event"], "started")
        self.assertEqual(lifecycle[-1]["event"], "completed")
        self.assertEqual(self.metadata("heartbeat")["heartbeat_seconds"], 0.05)
        self.assertIsNone(self.metadata("heartbeat")["timeout_seconds"])

    def test_timeout_terminates_the_child_process_group(self) -> None:
        pid_file = self.root / "grandchild.pid"
        code = (
            "import pathlib,subprocess,sys,time; "
            "p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(60)']); "
            "pathlib.Path(sys.argv[1]).write_text(str(p.pid)); time.sleep(60)"
        )
        result = self.invoke("timeout", code, os.fspath(pid_file), timeout="0.2")
        self.assertEqual(result.returncode, 124, result.stderr)
        metadata = self.metadata("timeout")
        self.assertTrue(metadata["timed_out"])
        grandchild = int(pid_file.read_text())
        for _ in range(20):
            try:
                os.kill(grandchild, 0)
            except ProcessLookupError:
                break
            time.sleep(0.05)
        else:
            self.fail("grandchild survived process-group timeout cleanup")
        events = [json.loads(line)["event"] for line in (self.root / "timeout" / "lifecycle.jsonl").read_text().splitlines()]
        self.assertIn("timeout", events)
        self.assertIn("signal_sent", events)

    def test_timeout_cleans_up_grandchild_holding_pipe_after_parent_exit(self) -> None:
        pid_file = self.root / "pipe-grandchild.pid"
        code = (
            "import pathlib,subprocess,sys; "
            "p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(60)']); "
            "pathlib.Path(sys.argv[1]).write_text(str(p.pid))"
        )
        result = self.invoke("pipe-timeout", code, os.fspath(pid_file), timeout="0.2")
        self.assertEqual(result.returncode, 124, result.stderr)
        self.assertTrue(self.metadata("pipe-timeout")["timed_out"])

    def test_refuses_existing_log_directory_before_launch(self) -> None:
        existing = self.root / "existing"
        existing.mkdir()
        marker = self.cwd / "must-not-exist"
        result = self.invoke("existing", f"import pathlib; pathlib.Path({os.fspath(marker)!r}).write_text('launched')")
        self.assertEqual(result.returncode, 64)
        self.assertFalse(marker.exists())
        self.assertIn(b"refusing to overwrite", result.stderr)

    def test_requires_absolute_executable_and_positive_bounds(self) -> None:
        base = [sys.executable, os.fspath(SCRIPT), "--cwd", os.fspath(self.cwd), "--log-dir", os.fspath(self.root / "invalid")]
        relative = subprocess.run([*base, "--timeout-seconds", "1", "--", "python3", "-V"], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.assertEqual(relative.returncode, 64)
        self.assertFalse((self.root / "invalid").exists())
        zero = subprocess.run([*base, "--timeout-seconds", "0", "--", os.fspath(PYTHON), "-V"], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.assertEqual(zero.returncode, 64)
        heartbeat_zero = subprocess.run(
            [*base, "--timeout-seconds", "1", "--heartbeat-seconds", "0", "--", os.fspath(PYTHON), "-V"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(heartbeat_zero.returncode, 64)

    def test_rejects_log_tree_overlapping_child_cwd(self) -> None:
        result = subprocess.run(
            [sys.executable, os.fspath(SCRIPT), "--cwd", os.fspath(self.cwd), "--log-dir", os.fspath(self.cwd / "logs"), "--timeout-seconds", "1", "--", os.fspath(PYTHON), "-V"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(result.returncode, 64)
        self.assertFalse((self.cwd / "logs").exists())
        self.assertIn(b"disjoint trees", result.stderr)


if __name__ == "__main__":
    unittest.main()
