#!/usr/bin/env python3
"""Run one exact CLI argv while preserving raw output and process lifecycle facts."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import threading
import time
from typing import BinaryIO


EX_USAGE = 64
EX_SOFTWARE = 70
EX_TIMEOUT = 124
SCHEMA_VERSION = 1


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="milliseconds").replace("+00:00", "Z")


def write_json_line(stream: BinaryIO, value: dict[str, object]) -> None:
    stream.write((json.dumps(value, ensure_ascii=False, sort_keys=True) + "\n").encode())
    stream.flush()


def atomic_json(path: Path, value: dict[str, object]) -> None:
    temporary = path.with_name(f".{path.name}.tmp-{os.getpid()}")
    temporary.write_text(json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(temporary, path)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def tee(
    source: BinaryIO,
    log: BinaryIO,
    destination: BinaryIO,
    progress: dict[str, float | int],
    progress_lock: threading.Lock,
    byte_key: str,
) -> None:
    try:
        try:
            read_chunk = getattr(source, "read1", source.read)
            while chunk := read_chunk(65536):
                log.write(chunk)
                log.flush()
                with progress_lock:
                    progress[byte_key] += len(chunk)
                    progress["last_output_monotonic"] = time.monotonic()
                try:
                    destination.write(chunk)
                    destination.flush()
                except BrokenPipeError:
                    # 親への表示が閉じても、証跡ファイルの保存は最後まで続ける。
                    destination = open(os.devnull, "wb")
        except (OSError, ValueError):
            # 回収不能な子孫がpipeを保持した場合、main threadが期限後にread endを閉じる。
            pass
    finally:
        source.close()


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cwd", required=True, type=Path, help="Existing child working directory")
    parser.add_argument("--log-dir", required=True, type=Path, help="New one-run evidence directory")
    parser.add_argument("--timeout-seconds", required=True, type=float, help="Positive wall timeout")
    parser.add_argument("--grace-seconds", type=float, default=5.0, help="Positive TERM-to-KILL grace")
    parser.add_argument("--heartbeat-seconds", type=float, default=10.0, help="Positive lifecycle heartbeat interval")
    parser.add_argument("command", nargs=argparse.REMAINDER, help="Exact child argv after --")
    args = parser.parse_args(argv)
    if args.command and args.command[0] == "--":
        args.command = args.command[1:]
    if not args.command:
        parser.error("an exact child argv is required after --")
    if args.timeout_seconds <= 0 or args.grace_seconds <= 0 or args.heartbeat_seconds <= 0:
        parser.error("timeout, grace, and heartbeat must be positive")
    if not args.command[0].startswith(os.sep):
        parser.error("the child executable must be an absolute path")
    return args


def child_return_code(returncode: int) -> int:
    if returncode < 0:
        return min(255, 128 + abs(returncode))
    return returncode if returncode <= 255 else 255


def main(argv: list[str]) -> int:
    try:
        args = parse_args(argv)
    except SystemExit as error:
        return EX_USAGE if error.code else 0

    cwd = args.cwd.resolve()
    log_dir = args.log_dir.resolve()
    executable = Path(args.command[0])
    if not cwd.is_dir():
        print(f"run-observed-cli: cwd is not a directory: {cwd}", file=sys.stderr)
        return EX_USAGE
    if not executable.is_file() or not os.access(executable, os.X_OK):
        print(f"run-observed-cli: executable is not runnable: {executable}", file=sys.stderr)
        return EX_USAGE
    if cwd == log_dir or cwd in log_dir.parents or log_dir in cwd.parents:
        print("run-observed-cli: cwd and log directory must be disjoint trees", file=sys.stderr)
        return EX_USAGE
    try:
        log_dir.mkdir(parents=True, exist_ok=False)
    except FileExistsError:
        print(f"run-observed-cli: refusing to overwrite log directory: {log_dir}", file=sys.stderr)
        return EX_USAGE

    stdout_path = log_dir / "stdout.log"
    stderr_path = log_dir / "stderr.log"
    lifecycle_path = log_dir / "lifecycle.jsonl"
    metadata_path = log_dir / "meta.json"
    started_at = utc_now()
    started_monotonic = time.monotonic()
    process: subprocess.Popen[bytes] | None = None
    timed_out = False
    received_signal: int | None = None
    sent_kill = False
    spawn_error: str | None = None
    progress: dict[str, float | int] = {
        "last_output_monotonic": started_monotonic,
        "stderr_bytes": 0,
        "stdout_bytes": 0,
    }
    progress_lock = threading.Lock()

    def remember_signal(signum: int, _frame: object) -> None:
        nonlocal received_signal
        if received_signal is None:
            received_signal = signum

    old_handlers = {number: signal.signal(number, remember_signal) for number in (signal.SIGINT, signal.SIGTERM)}
    with stdout_path.open("wb") as stdout_log, stderr_path.open("wb") as stderr_log, lifecycle_path.open("wb") as lifecycle:
        try:
            process = subprocess.Popen(
                args.command,
                cwd=cwd,
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                start_new_session=True,
            )
            assert process.stdout is not None and process.stderr is not None
            write_json_line(lifecycle, {"at": started_at, "event": "started", "pid": process.pid})
            stdout_thread = threading.Thread(
                target=tee,
                args=(process.stdout, stdout_log, sys.stdout.buffer, progress, progress_lock, "stdout_bytes"),
                daemon=True,
            )
            stderr_thread = threading.Thread(
                target=tee,
                args=(process.stderr, stderr_log, sys.stderr.buffer, progress, progress_lock, "stderr_bytes"),
                daemon=True,
            )
            stdout_thread.start()
            stderr_thread.start()

            deadline = started_monotonic + args.timeout_seconds
            next_heartbeat = started_monotonic + args.heartbeat_seconds
            while process.poll() is None:
                if received_signal is not None:
                    write_json_line(lifecycle, {"at": utc_now(), "event": "parent_signal", "signal": signal.Signals(received_signal).name})
                    break
                now = time.monotonic()
                if now >= deadline:
                    timed_out = True
                    write_json_line(lifecycle, {"at": utc_now(), "event": "timeout"})
                    break
                if now >= next_heartbeat:
                    if process.poll() is not None:
                        continue
                    with progress_lock:
                        stdout_bytes = int(progress["stdout_bytes"])
                        stderr_bytes = int(progress["stderr_bytes"])
                        last_output_monotonic = float(progress["last_output_monotonic"])
                    write_json_line(
                        lifecycle,
                        {
                            "at": utc_now(),
                            "elapsed_ms": round((now - started_monotonic) * 1000),
                            "event": "heartbeat",
                            "idle_ms": max(0, round((now - last_output_monotonic) * 1000)),
                            "pid": process.pid,
                            "pid_alive": True,
                            "stderr_bytes": stderr_bytes,
                            "stdout_bytes": stdout_bytes,
                        },
                    )
                    while next_heartbeat <= now:
                        next_heartbeat += args.heartbeat_seconds
                time.sleep(min(0.05, max(0.0, deadline - now), max(0.0, next_heartbeat - now)))

            if process.poll() is None and (timed_out or received_signal is not None):
                forwarded = received_signal if received_signal is not None else signal.SIGTERM
                os.killpg(process.pid, forwarded)
                write_json_line(lifecycle, {"at": utc_now(), "event": "signal_sent", "signal": signal.Signals(forwarded).name})
                try:
                    process.wait(timeout=args.grace_seconds)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    sent_kill = True
                    write_json_line(lifecycle, {"at": utc_now(), "event": "signal_sent", "signal": "SIGKILL"})
            process.wait()
            for thread in (stdout_thread, stderr_thread):
                thread.join(max(0.0, deadline - time.monotonic()))
            if stdout_thread.is_alive() or stderr_thread.is_alive():
                timed_out = True
                write_json_line(lifecycle, {"at": utc_now(), "event": "timeout", "reason": "open_pipe_after_child_exit"})
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                    write_json_line(lifecycle, {"at": utc_now(), "event": "signal_sent", "signal": "SIGTERM"})
                except ProcessLookupError:
                    pass
                for thread in (stdout_thread, stderr_thread):
                    thread.join(args.grace_seconds)
                if stdout_thread.is_alive() or stderr_thread.is_alive():
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                        sent_kill = True
                        write_json_line(lifecycle, {"at": utc_now(), "event": "signal_sent", "signal": "SIGKILL"})
                    except ProcessLookupError:
                        pass
                    process.stdout.close()
                    process.stderr.close()
                    stdout_thread.join(args.grace_seconds)
                    stderr_thread.join(args.grace_seconds)
            write_json_line(lifecycle, {"at": utc_now(), "event": "completed", "returncode": process.returncode})
        except (OSError, subprocess.SubprocessError) as error:
            spawn_error = f"{type(error).__name__}: {error}"
            write_json_line(lifecycle, {"at": utc_now(), "error": spawn_error, "event": "harness_error"})
        finally:
            for number, handler in old_handlers.items():
                signal.signal(number, handler)

    completed_at = utc_now()
    returncode = process.returncode if process is not None else None
    terminating_signal = signal.Signals(-returncode).name if returncode is not None and returncode < 0 else None
    metadata: dict[str, object] = {
        "argv": args.command,
        "completed_at": completed_at,
        "cwd": os.fspath(cwd),
        "duration_ms": round((time.monotonic() - started_monotonic) * 1000),
        "exit_code": returncode if returncode is not None and returncode >= 0 else None,
        "grace_seconds": args.grace_seconds,
        "heartbeat_seconds": args.heartbeat_seconds,
        "harness_error": spawn_error,
        "pid": process.pid if process is not None else None,
        "received_signal": signal.Signals(received_signal).name if received_signal is not None else None,
        "schema_version": SCHEMA_VERSION,
        "sent_sigkill": sent_kill,
        "signal": terminating_signal,
        "started_at": started_at,
        "stderr_bytes": stderr_path.stat().st_size,
        "stderr_sha256": sha256_file(stderr_path),
        "stdout_bytes": stdout_path.stat().st_size,
        "stdout_sha256": sha256_file(stdout_path),
        "timed_out": timed_out,
        "timeout_seconds": args.timeout_seconds,
    }
    atomic_json(metadata_path, metadata)
    if spawn_error is not None:
        print(f"run-observed-cli: {spawn_error}", file=sys.stderr)
        return EX_SOFTWARE
    if timed_out:
        return EX_TIMEOUT
    assert returncode is not None
    return child_return_code(returncode)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
