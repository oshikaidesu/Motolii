#!/usr/bin/env python3
"""Failure-injection fixtures for the cold-replaceable supervision contract."""

from __future__ import annotations

import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest


ROOT = Path(__file__).resolve().parent.parent
OBSERVED = ROOT / "scripts/run-observed-cli.py"
PYTHON = Path(sys.executable).resolve()
GIT = shutil.which("git")


def wait_for(path: Path, timeout: float = 5.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if path.exists():
            return
        time.sleep(0.01)
    raise AssertionError(f"timed out waiting for {path}")


def process_gone(pid: int, timeout: float = 3.0) -> bool:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            return True
        time.sleep(0.02)
    return False


def fingerprint_tree(root: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(candidate for candidate in root.rglob("*") if candidate.is_file()):
        digest.update(path.relative_to(root).as_posix().encode())
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def frontiers_are_disjoint(
    left_paths: set[str],
    left_owner: str,
    right_paths: set[str],
    right_owner: str,
) -> bool:
    return not left_paths.intersection(right_paths) and left_owner != right_owner


class SupervisionFailureContainmentTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.cwd = self.root / "cwd"
        self.cwd.mkdir()
        self.processes: list[subprocess.Popen[bytes]] = []

    def tearDown(self) -> None:
        for process in self.processes:
            if process.poll() is None:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                process.wait()
        self.temp.cleanup()

    def start_lock_holder(self, name: str) -> tuple[subprocess.Popen[bytes], Path, Path]:
        lock_path = self.root / f"{name}.lock"
        ready_path = self.root / f"{name}.ready"
        code = (
            "import fcntl,pathlib,sys,time; "
            "stream=open(sys.argv[1],'a+'); "
            "fcntl.flock(stream.fileno(),fcntl.LOCK_EX); "
            "pathlib.Path(sys.argv[2]).write_text('ready'); time.sleep(60)"
        )
        process = subprocess.Popen(
            [os.fspath(PYTHON), "-c", code, os.fspath(lock_path), os.fspath(ready_path)],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
        )
        self.processes.append(process)
        wait_for(ready_path)
        return process, lock_path, ready_path

    def invoke_observed(
        self,
        name: str,
        code: str,
        *,
        timeout: str = "2",
        heartbeat: str = "0.05",
        child_args: tuple[str, ...] = (),
    ) -> subprocess.CompletedProcess[bytes]:
        return subprocess.run(
            [
                sys.executable,
                os.fspath(OBSERVED),
                "--cwd",
                os.fspath(self.cwd),
                "--log-dir",
                os.fspath(self.root / "logs" / name),
                "--timeout-seconds",
                timeout,
                "--grace-seconds",
                "0.1",
                "--heartbeat-seconds",
                heartbeat,
                "--",
                os.fspath(PYTHON),
                "-c",
                code,
                *child_args,
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )

    def init_git_repo(self, name: str) -> Path:
        if GIT is None:
            self.skipTest("git is required")
        repo = self.root / name
        repo.mkdir()
        subprocess.run([GIT, "init", "-q", os.fspath(repo)], check=True)
        subprocess.run([GIT, "-C", os.fspath(repo), "config", "user.email", "fixture@example.invalid"], check=True)
        subprocess.run([GIT, "-C", os.fspath(repo), "config", "user.name", "Fixture"], check=True)
        return repo

    def test_double_top_seat_and_sigstop_do_not_create_false_takeover(self) -> None:
        holder, lock_path, _ = self.start_lock_holder("stalled")
        os.kill(holder.pid, signal.SIGSTOP)
        launch_marker = self.root / "must-not-launch"

        with lock_path.open("a+") as contender:
            with self.assertRaises(BlockingIOError):
                fcntl.flock(contender.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        self.assertFalse(launch_marker.exists())

        os.kill(holder.pid, signal.SIGCONT)
        os.kill(holder.pid, signal.SIGTERM)
        holder.wait(timeout=3)
        with lock_path.open("a+") as successor:
            fcntl.flock(successor.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)

    def test_sigkill_requires_process_exit_and_current_fingerprint_reconstruction(self) -> None:
        holder, lock_path, _ = self.start_lock_holder("dead")
        worktree = self.root / "active-worktree"
        worktree.mkdir()
        candidate = worktree / "candidate.txt"
        candidate.write_text("before\n", encoding="utf-8")
        prior = fingerprint_tree(worktree)

        os.kill(holder.pid, signal.SIGKILL)
        holder.wait(timeout=3)
        self.assertEqual(holder.returncode, -signal.SIGKILL)
        candidate.write_text("after\n", encoding="utf-8")
        current = fingerprint_tree(worktree)
        self.assertNotEqual(prior, current)

        with lock_path.open("a+") as successor:
            fcntl.flock(successor.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        self.assertEqual(current, fingerprint_tree(worktree))

    def test_base_drift_invalidates_preauthorization_and_candidate_adoption(self) -> None:
        repo = self.init_git_repo("base-drift")
        tracked = repo / "authority.txt"
        tracked.write_text("base\n", encoding="utf-8")
        subprocess.run([GIT, "-C", os.fspath(repo), "add", "authority.txt"], check=True)
        subprocess.run([GIT, "-C", os.fspath(repo), "commit", "-qm", "base"], check=True)
        authorized = subprocess.check_output([GIT, "-C", os.fspath(repo), "rev-parse", "HEAD"]).strip()

        tracked.write_text("advanced\n", encoding="utf-8")
        subprocess.run([GIT, "-C", os.fspath(repo), "commit", "-qam", "advance"], check=True)
        current = subprocess.check_output([GIT, "-C", os.fspath(repo), "rev-parse", "HEAD"]).strip()
        self.assertNotEqual(authorized, current)
        self.assertFalse(authorized == current)

    def test_write_set_semantic_owner_allowlist_and_reviewer_mutation_are_rejected(self) -> None:
        self.assertFalse(frontiers_are_disjoint({"a.rs"}, "document", {"a.rs"}, "render"))
        self.assertFalse(frontiers_are_disjoint({"a.rs"}, "document", {"b.rs"}, "document"))
        self.assertTrue(frontiers_are_disjoint({"a.rs"}, "document", {"b.rs"}, "render"))

        repo = self.init_git_repo("reviewer-mutation")
        (repo / "allowed.txt").write_text("base\n", encoding="utf-8")
        (repo / "forbidden.txt").write_text("base\n", encoding="utf-8")
        subprocess.run([GIT, "-C", os.fspath(repo), "add", "."], check=True)
        subprocess.run([GIT, "-C", os.fspath(repo), "commit", "-qm", "base"], check=True)
        before = fingerprint_tree(repo)
        (repo / "forbidden.txt").write_text("reviewer mutation\n", encoding="utf-8")
        changed = set(
            subprocess.check_output([GIT, "-C", os.fspath(repo), "diff", "--name-only"], text=True).splitlines()
        )
        self.assertFalse(changed.issubset({"allowed.txt"}))
        self.assertNotEqual(before, fingerprint_tree(repo))

    def test_bounded_return_stops_activation_and_preserves_candidates(self) -> None:
        candidate = self.root / "candidate.patch"
        candidate.write_text("unadopted\n", encoding="utf-8")
        return_limit = 3
        activations = [count < return_limit for count in range(5)]
        self.assertEqual(activations, [True, True, True, False, False])
        self.assertEqual(candidate.read_text(encoding="utf-8"), "unadopted\n")

    def test_channel_failure_has_no_silent_fallback_and_campaign_matrix_reconstructs_returns(self) -> None:
        success = self.invoke_observed("success", "print('success')")
        failure = self.invoke_observed("channel-unavailable", "raise SystemExit(69)")
        partial = self.invoke_observed("truncated", "import sys; sys.stdout.write('{partial'); sys.stdout.flush(); raise SystemExit(17)")
        hang = self.invoke_observed("hang", "import time; time.sleep(60)", timeout="0.15")
        silence = self.invoke_observed("silence", "import time; time.sleep(0.18); print('done')")

        self.assertEqual([success.returncode, failure.returncode, partial.returncode, hang.returncode, silence.returncode], [0, 69, 17, 124, 0])
        self.assertFalse((self.root / "logs" / "fallback").exists())
        self.assertEqual((self.root / "logs" / "truncated" / "stdout.log").read_bytes(), b"{partial")

        dispositions: list[str] = []
        for name in ("success", "channel-unavailable", "truncated", "hang", "silence"):
            meta = json.loads((self.root / "logs" / name / "meta.json").read_text(encoding="utf-8"))
            lifecycle = [
                json.loads(line)["event"]
                for line in (self.root / "logs" / name / "lifecycle.jsonl").read_text(encoding="utf-8").splitlines()
            ]
            self.assertEqual(lifecycle[0], "started")
            self.assertEqual(lifecycle[-1], "completed")
            dispositions.append("RETURN(done)" if meta["exit_code"] == 0 and not meta["timed_out"] else "RETURN(fail)")
        self.assertEqual(dispositions, ["RETURN(done)", "RETURN(fail)", "RETURN(fail)", "RETURN(fail)", "RETURN(done)"])
        silence_events = [
            json.loads(line)["event"]
            for line in (self.root / "logs" / "silence" / "lifecycle.jsonl").read_text(encoding="utf-8").splitlines()
        ]
        self.assertIn("heartbeat", silence_events[:-1])

    def test_user_stop_reclaims_the_entire_process_group_without_new_launch(self) -> None:
        child_pid = self.root / "child.pid"
        grandchild_pid = self.root / "grandchild.pid"
        code = (
            "import os,pathlib,subprocess,sys,time; "
            "pathlib.Path(sys.argv[1]).write_text(str(os.getpid())); "
            "child=subprocess.Popen([sys.executable,'-c','import time; time.sleep(60)']); "
            "pathlib.Path(sys.argv[2]).write_text(str(child.pid)); time.sleep(60)"
        )
        harness = subprocess.Popen(
            [
                sys.executable,
                os.fspath(OBSERVED),
                "--cwd",
                os.fspath(self.cwd),
                "--log-dir",
                os.fspath(self.root / "logs" / "user-stop"),
                "--grace-seconds",
                "0.1",
                "--heartbeat-seconds",
                "0.05",
                "--",
                os.fspath(PYTHON),
                "-c",
                code,
                os.fspath(child_pid),
                os.fspath(grandchild_pid),
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
        )
        self.processes.append(harness)
        wait_for(child_pid)
        wait_for(grandchild_pid)
        os.kill(harness.pid, signal.SIGTERM)
        harness.wait(timeout=5)

        meta = json.loads((self.root / "logs" / "user-stop" / "meta.json").read_text(encoding="utf-8"))
        self.assertEqual(meta["received_signal"], "SIGTERM")
        self.assertTrue(process_gone(int(child_pid.read_text())))
        self.assertTrue(process_gone(int(grandchild_pid.read_text())))
        self.assertFalse((self.root / "logs" / "after-user-stop").exists())

    def test_integration_crash_leaves_pre_commit_or_atomic_two_file_commit(self) -> None:
        repo = self.init_git_repo("integration-crash")
        decision = repo / "decision.md"
        ledger = repo / "ledger.md"
        decision.write_text("old decision\n", encoding="utf-8")
        ledger.write_text("old ledger\n", encoding="utf-8")
        subprocess.run([GIT, "-C", os.fspath(repo), "add", "."], check=True)
        subprocess.run([GIT, "-C", os.fspath(repo), "commit", "-qm", "base"], check=True)
        before = subprocess.check_output([GIT, "-C", os.fspath(repo), "rev-parse", "HEAD"]).strip()

        decision.write_text("new decision\n", encoding="utf-8")
        ledger.write_text("new ledger\n", encoding="utf-8")
        hook_ready = self.root / "hook.ready"
        hook = repo / ".git/hooks/pre-commit"
        hook.write_text(f"#!/bin/sh\ntouch {hook_ready}\nsleep 60\n", encoding="utf-8")
        hook.chmod(0o755)
        commit = subprocess.Popen(
            [GIT, "-C", os.fspath(repo), "commit", "-am", "atomic update"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
        )
        self.processes.append(commit)
        wait_for(hook_ready)
        os.killpg(commit.pid, signal.SIGKILL)
        commit.wait(timeout=3)
        self.assertEqual(subprocess.check_output([GIT, "-C", os.fspath(repo), "rev-parse", "HEAD"]).strip(), before)

        hook.unlink()
        index_lock = repo / ".git/index.lock"
        self.assertTrue(index_lock.is_file())
        index_lock.unlink()
        subprocess.run([GIT, "-C", os.fspath(repo), "commit", "-am", "atomic update"], check=True, stdout=subprocess.PIPE)
        after = subprocess.check_output([GIT, "-C", os.fspath(repo), "rev-parse", "HEAD"]).strip()
        self.assertNotEqual(before, after)
        committed = set(
            subprocess.check_output([GIT, "-C", os.fspath(repo), "show", "--format=", "--name-only", "HEAD"], text=True).splitlines()
        )
        self.assertEqual(committed, {"decision.md", "ledger.md"})


if __name__ == "__main__":
    unittest.main()
