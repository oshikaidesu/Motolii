#!/usr/bin/env python3
"""Process-inventory regressions for the pinned warm-session guard."""
import importlib.util
import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

spec = importlib.util.spec_from_file_location("runtime", Path(__file__).with_name("reload-runtime.py"))
runtime = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runtime)
step_spec = importlib.util.spec_from_file_location("step", Path(__file__).with_name("reload-step.py"))
step = importlib.util.module_from_spec(step_spec)
step_spec.loader.exec_module(step)


def output(text, code=0):
    return subprocess.CompletedProcess([], code, stdout=text, stderr="")


class Inventory(unittest.TestCase):
    def test_shell_text_is_not_a_running_devserver(self):
        ps = '12 1 S /bin/zsh -c "motolii-dx-0.7.10-guarded serve"\n'
        with patch.object(runtime.subprocess, "run", return_value=output(ps)):
            self.assertEqual(runtime.inventory(), [])

    def test_only_the_actual_product_working_directory_matches(self):
        ps = "12 1 S /tmp/dx serve --hotpatch\n13 1 S /tmp/dx serve\n"
        replies = [output(ps), output("p12\nn" + str(runtime.PRODUCT) + "\n"), output("p13\nn/tmp/another-project\n")]
        with patch.object(runtime.subprocess, "run", side_effect=replies):
            self.assertEqual([r["pid"] for r in runtime.inventory()], [12])

    def test_inspection_failure_never_becomes_zero_processes(self):
        with patch.object(runtime.subprocess, "run", side_effect=subprocess.CalledProcessError(1, "ps")):
            with self.assertRaises(subprocess.CalledProcessError):
                runtime.inventory()
        with patch.object(runtime.subprocess, "run", side_effect=[output("12 1 S /tmp/dx serve\n"), output("", 1)]):
            with self.assertRaises(RuntimeError):
                runtime.inventory()

    def test_launcher_only_announces_registration_path_until_watcher_is_ready(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch.object(runtime, "binary", return_value=(Path("/tmp/test-dx"), {"sha256": "test"})), \
                 patch.object(runtime, "inventory", return_value=[]), \
                 patch.object(runtime, "app_inventory", return_value=[]), \
                 patch.object(runtime, "state_dir", return_value=root), \
                 patch.object(runtime.os, "chdir"), patch.object(runtime.os, "execv") as execute, \
                 patch.dict(os.environ, {}, clear=True), patch.object(sys, "argv", ["runtime", "serve"]), \
                 contextlib.redirect_stdout(io.StringIO()):
                runtime.main()
                self.assertEqual(Path(os.environ["MOTOLII_RUNTIME_SOURCE_REGISTRATION"]), root / "runtime-sources.json")
                self.assertFalse((root / "runtime-sources.json").exists())
                self.assertTrue(os.environ["MOTOLII_RELOAD_SESSION"])
                self.assertEqual(json.loads((root / "baseline-permit.json").read_text())["cause"], "initial_baseline")
                execute.assert_called_once()


class OpenApp(unittest.TestCase):
    def test_reuse_never_accepts_stopped_stock_or_duplicate_servers(self):
        path = Path("/tmp/fixed-dx")
        row = {"pid": 12, "state": "S", "executable": str(path)}
        self.assertTrue(runtime.reusable([row], path))
        for rows in ([], [row, row], [dict(row, state="T")], [dict(row, executable="/tmp/dx")]):
            self.assertFalse(runtime.reusable(rows, path))

    def test_exact_bundle_executable_including_spaces(self):
        with patch.object(runtime, "PRODUCT", Path("/tmp/a project/motolii")):
            executable = runtime.PRODUCT / "target/dx/motolii/debug/macos/Motolii.app/Contents/MacOS/motolii"
            ps = f"12 1 S {executable}\n13 1 S /tmp/other/Motolii.app/Contents/MacOS/motolii\n14 1 S /bin/zsh\n"
            with patch.object(runtime.subprocess, "run", return_value=output(ps)):
                self.assertEqual([app["pid"] for app in runtime.app_inventory()], [12])

    def test_process_inspection_failure_is_not_an_idle_app(self):
        with patch.object(runtime.subprocess, "run", side_effect=subprocess.CalledProcessError(1, "ps")):
            with self.assertRaises(subprocess.CalledProcessError):
                runtime.app_inventory()

    def test_orphan_app_prevents_baseline_and_permit_creation(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch.object(runtime, "binary", return_value=(Path("/tmp/test-dx"), {"sha256": "test"})), \
                 patch.object(runtime, "inventory", return_value=[]), \
                 patch.object(runtime, "app_inventory", return_value=[{"pid": 12}]), \
                 patch.object(runtime, "state_dir", return_value=root), \
                 patch.object(runtime.os, "execv") as execute, \
                 patch.object(sys, "argv", ["runtime", "serve"]):
                with self.assertRaisesRegex(RuntimeError, "open without a reload server"):
                    runtime.main()
                execute.assert_not_called()
                self.assertFalse((root / "baseline-permit.json").exists())

    def test_warm_launch_reuses_server_without_inspecting_app_or_creating_permit(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            row = {"pid": 12, "state": "S", "executable": "/tmp/test-dx"}
            for locked in (False, True):
                with self.subTest(locked=locked), \
                     patch.object(runtime, "binary", return_value=(Path("/tmp/test-dx"), {"sha256": "test"})), \
                     patch.object(runtime, "inventory", return_value=[row]), \
                     patch.object(runtime, "app_inventory") as apps, \
                     patch.object(runtime, "state_dir", return_value=root), \
                     patch.object(runtime.fcntl, "flock", side_effect=BlockingIOError if locked else None), \
                     patch.object(runtime.os, "execv") as execute, \
                     patch.object(sys, "argv", ["runtime", "serve"]), \
                     contextlib.redirect_stdout(io.StringIO()) as output:
                    self.assertEqual(runtime.main(), 0)
                    self.assertEqual(json.loads(output.getvalue().removeprefix("MOTOLII_RELOAD "))["event"], "warm_session_reused")
                    apps.assert_not_called()
                    execute.assert_not_called()
                    self.assertFalse((root / "baseline-permit.json").exists())


class Step(unittest.TestCase):
    def run_step(self, code, clocks):
        with tempfile.TemporaryDirectory() as directory:
            receipt = Path(directory) / "receipt.jsonl"
            args = ["step", "--need", "test", "--ruler", "subprocess wait semantics", "--gap", "test", "--owner", "test", "--class", "THIN_PATCH", "--receipt", str(receipt), "--", "test-command"]
            process = Mock(pid=123, **{"wait.return_value": code})
            with patch.object(sys, "argv", args), patch.object(step.subprocess, "Popen", return_value=process), \
                 patch.object(step.time, "monotonic", side_effect=clocks), patch.object(step.os, "killpg") as kill, \
                 contextlib.redirect_stdout(io.StringIO()):
                result = step.main()
                process.wait.assert_called_once_with(timeout=None)
                kill.assert_not_called()
            return result, [json.loads(line) for line in receipt.read_text().splitlines()]

    def test_slow_default_run_finishes_without_abort(self):
        code, records = self.run_step(0, [0, 120])
        self.assertEqual(code, 0)
        self.assertEqual(records[-1]["status"], "slow_completed")
        self.assertIsNone(records[-1]["hard_timeout_seconds"])
        self.assertTrue(records[-1]["exceeded_advisory"])

    def test_command_failure_keeps_its_exit_code_and_receipt(self):
        code, records = self.run_step(17, [0, 1])
        self.assertEqual(code, 17)
        self.assertEqual(records[-1]["status"], "failed")
        self.assertEqual(records[-1]["exit_code"], 17)


if __name__ == "__main__":
    unittest.main()
