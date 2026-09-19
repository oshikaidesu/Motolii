"""HTTP contract, fallback policy, and observed-runner integration tests (offline)."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import Mock, patch
import urllib.error

import jev_triage as jev


def response():
    return {"model": jev.MODEL, "answers": {
        name: {"type": "choice", "choice": choice, "confidence": 0.95,
               "probabilities": {option: 0.96 if option == choice else 0.01 for option in q["criteria"]}}
        for (name, q), choice in zip(jev.QUESTIONS.items(), ("test", "document"))
    }}


class JevTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.logs = self.root / "logs"
        self.logs.mkdir()
        (self.logs / "stdout.log").write_text("test edit_transactions FAILED: expected 2, got 1 in motolii-doc")
        (self.logs / "stderr.log").write_text("")
        self.env = patch.dict(os.environ, {"TYPESAFE_API_KEY": "test-key-never-send-raw"})
        self.env.start()
        self.request = Mock(return_value=response())

    def tearDown(self):
        self.env.stop()
        self.temp.cleanup()

    def classify(self, mode="route", **metadata):
        return jev.classify({"exit_code": 101, **metadata}, self.logs, mode, self.request)

    def test_batches_two_typed_questions_and_routes_without_waiving_review(self):
        result = self.classify()
        self.assertEqual(result["next_action"], "inspect_test_failure")
        self.assertEqual(result["owner"], "document")
        self.assertFalse(result["classification_llm_needed"])
        self.assertTrue(result["review_required"])
        self.assertEqual(result["potential_classification_calls_avoided"], 1)
        self.request.assert_called_once()
        payload, key = self.request.call_args.args
        self.assertEqual(set(payload), {"model", "state", "questions"})
        self.assertEqual(set(payload["questions"]), {"failure_kind", "owner"})
        self.assertNotIn(key, json.dumps(payload))

    def test_shadow_keeps_existing_route(self):
        result = self.classify("shadow")
        self.assertEqual(result["next_action"], "existing_llm")
        self.assertEqual(result["suggested_owner"], "document")
        self.assertEqual(result["potential_classification_calls_avoided"], 0)

    def test_no_network_for_disabled_success_and_incomplete_runs(self):
        cases = [("off", {}, "disabled"), ("route", {"exit_code": 0}, "command_succeeded"),
                 ("route", {"timed_out": True, "exit_code": 0}, "incomplete_run"),
                 ("route", {"signal": "SIGTERM"}, "incomplete_run"),
                 ("route", {"received_signal": "SIGINT"}, "incomplete_run"),
                 ("route", {"harness_error": "spawn error"}, "incomplete_run"),
                 ("route", {"exit_code": None}, "invalid_exit_code"),
                 ("route", {"exit_code": False}, "invalid_exit_code")]
        for mode, metadata, reason in cases:
            with self.subTest(reason=reason):
                self.assertEqual(self.classify(mode, **metadata)["reason"], reason)
        self.request.assert_not_called()

    def test_missing_key_and_missing_or_empty_logs_fall_back(self):
        with patch.dict(os.environ, {"TYPESAFE_API_KEY": ""}):
            self.assertEqual(self.classify()["reason"], "missing_api_key")
        (self.logs / "stdout.log").write_text("")
        self.assertEqual(self.classify()["reason"], "empty_evidence")
        (self.logs / "stdout.log").unlink()
        self.assertEqual(self.classify()["next_action"], "existing_llm")
        self.request.assert_not_called()

    def test_malformed_uncertain_and_out_of_contract_answers_never_route(self):
        mutations = [
            lambda r: r.update(model="jev-latest"),
            lambda r: r["answers"].pop("owner"),
            lambda r: r["answers"].update(extra={}),
            lambda r: r["answers"]["owner"].update(choice="run_shell"),
            lambda r: r["answers"]["owner"].update(choice=[]),
            lambda r: r["answers"]["owner"].update(choice="unknown"),
            lambda r: r["answers"]["owner"].update(type="score"),
            lambda r: r["answers"]["owner"].update(confidence=0.84),
            lambda r: r["answers"]["owner"].update(confidence=float("nan")),
            lambda r: r["answers"]["owner"].update(confidence=True),
            lambda r: r["answers"]["owner"].update(probabilities={}),
            lambda r: r["answers"]["owner"]["probabilities"].update(document=0.5),
            lambda r: r["answers"]["owner"]["probabilities"].update(document=float("inf")),
        ]
        for mutate in mutations:
            candidate = response()
            mutate(candidate)
            self.request.return_value = candidate
            with self.subTest(candidate=candidate):
                result = self.classify()
                self.assertEqual(result["next_action"], "existing_llm")
                self.assertTrue(result["review_required"])
                self.assertEqual(result["potential_classification_calls_avoided"], 0)

    def test_rate_limit_auth_and_transport_failures_do_not_retry(self):
        for reason in ("http_401", "http_429", "http_529", "response_too_large", "transport_or_json_error"):
            self.request.reset_mock()
            self.request.return_value = {"transport_error": reason}
            self.assertEqual(self.classify()["reason"], reason)
            self.request.assert_called_once()
        self.request.side_effect = subprocess.TimeoutExpired("hidden", 2)
        self.assertEqual(self.classify()["reason"], "deadline")

    def test_http_rejects_oversized_and_malformed_bodies(self):
        opened = Mock()
        opened.__enter__ = Mock(return_value=opened)
        opened.__exit__ = Mock(return_value=False)
        opener = Mock()
        opener.open.return_value = opened
        with patch.object(jev.urllib.request, "build_opener", return_value=opener):
            opened.read.return_value = b"x" * (jev.MAX_RESPONSE_BYTES + 1)
            self.assertEqual(jev.http_request({}, "key")["transport_error"], "response_too_large")
            opened.read.return_value = b"not json"
            self.assertEqual(jev.http_request({}, "key")["transport_error"], "transport_or_json_error")

    def test_bounded_redacted_input_and_output_dont_persist_log_content(self):
        log = "a" * 10000 + "\nTYPESAFE_API_KEY=test-key-never-send-raw\nAuthorization: Bearer sensitive\n"
        log += "ghp_abcdef123456\ntest edit_transactions FAILED motolii-doc\n"
        (self.logs / "stdout.log").write_text(log)
        result = self.classify()
        payload = self.request.call_args.args[0]
        sent = json.dumps(payload)
        for secret in ("test-key-never-send-raw", "sensitive", "ghp_abcdef123456"):
            self.assertNotIn(secret, sent)
            self.assertNotIn(secret, json.dumps(result))
        self.assertTrue(payload["state"]["stdout"]["truncated"])
        self.assertNotIn("stdout", result)
        self.assertNotIn("edit_transactions", json.dumps(result))

    def test_http_request_contract_and_redirect_refusal(self):
        opened = Mock()
        opened.__enter__ = Mock(return_value=opened)
        opened.__exit__ = Mock(return_value=False)
        opened.read.return_value = json.dumps(response()).encode()
        opener = Mock()
        opener.open.return_value = opened
        with patch.object(jev.urllib.request, "build_opener", return_value=opener):
            self.assertEqual(jev.http_request({"questions": jev.QUESTIONS}, "test-key"), response())
            req = opener.open.call_args.args[0]
            self.assertEqual(req.full_url, "https://api.typesafe.ai/v1/systemone")
            self.assertEqual(req.method, "POST")
            self.assertEqual(req.get_header("Authorization"), "Bearer test-key")
            self.assertEqual(json.loads(req.data)["questions"], jev.QUESTIONS)
            for code in (302, 401, 429, 529):
                opener.open.side_effect = urllib.error.HTTPError(req.full_url, code, "secret body", {}, None)
                self.assertEqual(jev.http_request({}, "key"), {"transport_error": f"http_{code}"})
        self.assertIsNone(jev.NoRedirect().redirect_request(None, None, 302, None, None, "https://elsewhere.invalid"))

    def test_request_has_real_wall_deadline_and_no_secret_in_argv(self):
        run = subprocess.run
        captured = []
        def slow_transport(argv, **kwargs):
            captured.append(argv)
            return run([sys.executable, "-c", "import time; time.sleep(10)"], **kwargs)
        started = time.monotonic()
        with patch.object(jev, "DEADLINE_SECONDS", 0.1), patch.object(jev.subprocess, "run", side_effect=slow_transport):
            with self.assertRaises(subprocess.TimeoutExpired):
                jev.request_jev({}, "key-not-in-argv")
        self.assertLess(time.monotonic() - started, 2)
        self.assertNotIn("key-not-in-argv", str(captured))

    def test_runner_preserves_exit_streams_and_default_artifacts(self):
        runner = Path(__file__).with_name("run-observed-cli.py")
        cwd = self.root / "cwd"
        cwd.mkdir()
        for mode, code in (("off", 7), ("route", 7), ("route", 0)):
            logs = self.root / f"run-{mode}-{code}"
            with patch.dict(os.environ, {"TYPESAFE_API_KEY": ""}):
                result = subprocess.run([
                    sys.executable, str(runner), "--cwd", str(cwd), "--log-dir", str(logs),
                    "--jev-mode", mode, "--", str(Path(sys.executable).resolve()), "-c",
                    f"import sys; print('output'); print('error', file=sys.stderr); sys.exit({code})",
                ], capture_output=True, text=True)
            self.assertEqual(result.returncode, code)
            self.assertEqual(result.stdout, "output\n")
            self.assertEqual(result.stderr, "error\n")
            self.assertEqual(json.loads((logs / "meta.json").read_text())["exit_code"], code)
            if mode == "off":
                self.assertFalse((logs / "jev.json").exists())
            else:
                decision = json.loads((logs / "jev.json").read_text())
                self.assertEqual(decision["next_action"], "continue" if code == 0 else "existing_llm")

    def test_runner_accepts_route_without_turning_failed_command_green(self):
        runner = Path(__file__).with_name("run-observed-cli.py")
        cwd = self.root / "cwd"
        cwd.mkdir()
        logs = self.root / "accepted-run"
        bootstrap = (
            "import json,runpy,sys; import jev_triage; "
            f"jev_triage.request_jev=lambda *args: json.loads({json.dumps(response())!r}); "
            "sys.argv=sys.argv[1:]; runpy.run_path(sys.argv[0],run_name='__main__')"
        )
        env = {**os.environ, "PYTHONPATH": str(runner.parent)}
        result = subprocess.run([
            sys.executable, "-c", bootstrap, str(runner), "--cwd", str(cwd),
            "--log-dir", str(logs), "--jev-mode", "route", "--",
            str(Path(sys.executable).resolve()), "-c", "import sys; print('FAILED motolii-doc'); sys.exit(101)",
        ], capture_output=True, text=True, env=env)
        self.assertEqual(result.returncode, 101, result.stderr)
        report = json.loads((logs / "jev.json").read_text())
        self.assertEqual(report["source"], "jev")
        self.assertEqual(report["owner"], "document")
        self.assertTrue(report["review_required"])


if __name__ == "__main__":
    unittest.main()
