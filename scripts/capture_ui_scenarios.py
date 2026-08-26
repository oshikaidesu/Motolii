#!/usr/bin/env python3
"""シナリオごとに Motolii の実ウィンドウを撮り、機械採点まで実行する。

`--screenshot` のオフスクリーン器具は意図的に拒否する。ここで作る証拠は
「起動した PID の on-screen window」を macOS の `screencapture -l` で撮った物だけ。
"""

from __future__ import annotations

import argparse
import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


def _load_manifest(path: Path) -> dict[str, Any]:
    analyzer_path = Path(__file__).with_name("analyze_ui_screenshots.py")
    import importlib.util

    spec = importlib.util.spec_from_file_location("ui_observation_analyzer", analyzer_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"解析器をロードできない: {analyzer_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.load_manifest(path)


def _compile_window_helper(root: Path, artifact_dir: Path) -> Path:
    if sys.platform != "darwin":
        raise RuntimeError("real-window capture は macOS の CoreGraphics に限定している")
    helper_source = root / "scripts/macos_window_id.swift"
    helper = artifact_dir / ".bin/macos-window-id"
    helper.parent.mkdir(parents=True, exist_ok=True)
    if not helper.exists() or helper.stat().st_mtime < helper_source.stat().st_mtime:
        result = subprocess.run(
            ["/usr/bin/swiftc", str(helper_source), "-o", str(helper)],
            cwd=root,
            text=True,
            capture_output=True,
            check=False,
        )
        if result.returncode != 0:
            raise RuntimeError(f"window helper のコンパイルに失敗: {result.stderr.strip()}")
    return helper


def _window_id(helper: Path, pid: int) -> int | None:
    result = subprocess.run(
        [str(helper), str(pid)], capture_output=True, text=True, check=False
    )
    if result.returncode != 0:
        return None
    try:
        return int(result.stdout.strip().splitlines()[-1])
    except (ValueError, IndexError):
        return None


def _wait_for_window(helper: Path, pid: int, timeout: float) -> int:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        window_id = _window_id(helper, pid)
        if window_id is not None:
            return window_id
        time.sleep(0.1)
    raise TimeoutError(f"PID {pid} の実ウィンドウが {timeout:.1f}s 内に見つからない")


def _stop(process: subprocess.Popen[Any]) -> None:
    if process.poll() is not None:
        return
    process.send_signal(signal.SIGTERM)
    try:
        process.wait(timeout=4.0)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=2.0)


def _scenario_selection(manifest: dict[str, Any], requested: list[str]) -> list[dict[str, Any]]:
    scenarios = manifest["scenarios"]
    if not requested:
        return scenarios
    lookup = {scenario["id"]: scenario for scenario in scenarios}
    missing = [scenario_id for scenario_id in requested if scenario_id not in lookup]
    if missing:
        raise ValueError(f"unknown scenario: {', '.join(missing)}")
    selected = [lookup[scenario_id] for scenario_id in requested]
    for scenario in selected:
        reference = scenario.get("reference")
        if reference and reference not in {item["id"] for item in selected}:
            raise ValueError(
                f"scenario {scenario['id']} needs reference {reference}; select both or run all"
            )
    return selected


def _write_json(path: Path, value: Any) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("--manifest", type=Path, default=Path("next/reference/ui-observation-scenarios.json"))
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--scenario", action="append", default=[])
    parser.add_argument("--settle-ms", type=int)
    parser.add_argument("--window-timeout", type=float)
    parser.add_argument("--dry-run", action="store_true", help="実行せず、シナリオとコマンドだけ検証する")
    args = parser.parse_args()

    root = args.root.resolve()
    manifest_path = (root / args.manifest).resolve() if not args.manifest.is_absolute() else args.manifest.resolve()
    try:
        manifest = _load_manifest(manifest_path)
        scenarios = _scenario_selection(manifest, args.scenario)
    except (OSError, ValueError, RuntimeError) as error:
        print(f"RED: {error}")
        return 1

    configured_binary = manifest.get("binary", "next/target/preview/motolii-shell")
    binary = args.binary or Path(configured_binary)
    binary = (root / binary).resolve() if not binary.is_absolute() else binary.resolve()
    settle_ms = args.settle_ms if args.settle_ms is not None else int(manifest["capture"].get("settle_ms", 1200))
    window_timeout = args.window_timeout if args.window_timeout is not None else float(manifest["capture"].get("window_timeout_s", 15.0))

    print(f"SOURCE: real-window (macOS window capture; headless --screenshot is rejected)")
    print(f"BINARY: {binary}")
    for scenario in scenarios:
        print(f"PLAN: {scenario['id']} operation={scenario.get('operation', 'unspecified')} argv={scenario.get('argv', [])}")
    if args.dry_run:
        return 0

    if not binary.is_file() or not os.access(binary, os.X_OK):
        print(f"RED: executable not found or not executable: {binary}")
        print("build once first: cargo build --manifest-path next/Cargo.toml --profile preview -p motolii-shell -j 4")
        return 1

    timestamp = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())
    artifact_dir = args.output.resolve() if args.output else root / "target/ui-observations" / timestamp
    artifact_dir.mkdir(parents=True, exist_ok=True)
    _write_json(artifact_dir / "manifest.json", manifest)
    run: dict[str, Any] = {
        "schema": 1,
        "source": "real-window",
        "started_at_utc": timestamp,
        "binary": str(binary),
        "manifest": str(manifest_path),
        "settle_ms": settle_ms,
        "window_timeout_s": window_timeout,
        "scenarios": [],
    }
    errors = 0
    helper: Path | None = None
    try:
        helper = _compile_window_helper(root, artifact_dir)
    except (OSError, RuntimeError) as error:
        print(f"RED: {error}")
        run["error"] = str(error)
        _write_json(artifact_dir / "run.json", run)
        return 1

    for scenario in scenarios:
        scenario_id = scenario["id"]
        scenario_dir = artifact_dir / "scenarios" / scenario_id
        scenario_dir.mkdir(parents=True, exist_ok=True)
        log_path = scenario_dir / "process.log"
        metadata: dict[str, Any] = {
            "source": "real-window",
            "capture_method": "macOS screencapture -l <window-id>",
            "scenario": scenario_id,
            "operation": scenario.get("operation", "unspecified"),
            "argv": scenario.get("argv", []),
            "image": "window.png",
        }
        process: subprocess.Popen[Any] | None = None
        try:
            argv = [str(value) for value in scenario.get("argv", [])]
            if "--screenshot" in argv:
                raise ValueError("--screenshot is forbidden in real-window capture")
            with log_path.open("w", encoding="utf-8") as log:
                process = subprocess.Popen(
                    [str(binary), *argv],
                    cwd=root,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                    start_new_session=True,
                )
                metadata["pid"] = process.pid
                window_id = _wait_for_window(helper, process.pid, window_timeout)
                metadata["window_id"] = window_id
                time.sleep(max(0, settle_ms) / 1000.0)
                capture = subprocess.run(
                    # `-o` excludes the macOS shadow. Without it the artifact has a
                    # transparent/black margin outside the actual window, which
                    # makes a normalized footer region look falsely empty.
                    ["/usr/sbin/screencapture", "-x", "-o", "-l", str(window_id), str(scenario_dir / "window.png")],
                    cwd=root,
                    text=True,
                    capture_output=True,
                    check=False,
                )
                if capture.returncode != 0:
                    raise RuntimeError(
                        f"screencapture failed ({capture.returncode}): {capture.stderr.strip()}"
                    )
            metadata["status"] = "captured"
            print(f"GREEN: captured {scenario_id} window={metadata['window_id']} pid={metadata['pid']}")
        except (OSError, RuntimeError, TimeoutError, ValueError) as error:
            errors += 1
            metadata["status"] = "RED"
            metadata["error"] = str(error)
            print(f"RED: {scenario_id}: {error}")
        finally:
            if process is not None:
                _stop(process)
            _write_json(scenario_dir / "capture.json", metadata)
            run["scenarios"].append(metadata)

    _write_json(artifact_dir / "run.json", run)
    analyzer = Path(__file__).with_name("analyze_ui_screenshots.py")
    analyzed = subprocess.run(
        [sys.executable, str(analyzer), str(manifest_path), str(artifact_dir)],
        cwd=root,
        text=True,
        check=False,
    )
    return 1 if errors or analyzed.returncode else 0


if __name__ == "__main__":
    raise SystemExit(main())
