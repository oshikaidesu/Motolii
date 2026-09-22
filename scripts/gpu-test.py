#!/usr/bin/env python3
"""Run one GPU test command with an explicit backend and a hard wall timeout.

This is a process boundary on purpose. A native GPU driver may block inside
adapter/device initialization, where an in-process Rust timeout cannot safely
cancel the call. Exit 124 means the GPU test host timed out.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time


EX_TIMEOUT = 124


def default_backend() -> str:
    if sys.platform == "darwin":
        return "metal"
    if os.name == "nt":
        return "dx12"
    return "vulkan"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--backend", default=default_backend(), help="WGPU_BACKEND value (default: platform primary backend)")
    parser.add_argument("--adapter", help="Optional WGPU_ADAPTER_NAME substring")
    parser.add_argument("--timeout-seconds", type=float, default=90.0, help="Hard wall timeout for the whole child process tree")
    parser.add_argument("command", nargs=argparse.REMAINDER, help="Command after --, for example: -- cargo test -p motolii-render ...")
    args = parser.parse_args()
    if args.command and args.command[0] == "--":
        args.command = args.command[1:]
    if not args.command:
        parser.error("a child command is required after --")
    if args.timeout_seconds <= 0:
        parser.error("--timeout-seconds must be positive")
    return args


def command_argv(raw: list[str]) -> list[str]:
    executable = shutil.which(raw[0])
    if executable is None:
        raise SystemExit(f"gpu-test: executable not found: {raw[0]}")
    return [str(Path(executable).resolve()), *raw[1:]]


def process_group_alive(pgid: int) -> bool:
    try:
        os.killpg(pgid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def wait_for_process_group_exit(pgid: int, timeout: float) -> bool:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if not process_group_alive(pgid):
            return True
        time.sleep(0.05)
    return not process_group_alive(pgid)


def stop_process_tree(process: subprocess.Popen[bytes]) -> None:
    if os.name == "nt":
        subprocess.run(
            ["taskkill", "/PID", str(process.pid), "/T", "/F"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
        try:
            process.wait(timeout=5.0)
        except subprocess.TimeoutExpired:
            pass
        return

    # start_new_session=True makes the child's pid the process-group id. The
    # group can outlive its leader, so waiting only for the parent is wrong:
    # a grandchild may ignore SIGTERM and remain after the runner returns 124.
    pgid = process.pid
    try:
        os.killpg(pgid, signal.SIGTERM)
    except ProcessLookupError:
        return

    # Reap a leader that exits promptly. Otherwise its zombie keeps the process
    # group observable and can hide the real question: are descendants alive?
    try:
        process.wait(timeout=0.1)
    except subprocess.TimeoutExpired:
        pass

    if not wait_for_process_group_exit(pgid, 3.0):
        try:
            os.killpg(pgid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        wait_for_process_group_exit(pgid, 3.0)

    # Reap the direct child independently of group lifetime.
    try:
        process.wait(timeout=1.0)
    except subprocess.TimeoutExpired:
        try:
            process.kill()
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=1.0)
        except subprocess.TimeoutExpired:
            pass


def main() -> int:
    args = parse_args()
    argv = command_argv(args.command)
    env = os.environ.copy()
    env["WGPU_BACKEND"] = args.backend
    if args.adapter:
        env["WGPU_ADAPTER_NAME"] = args.adapter
    else:
        env.pop("WGPU_ADAPTER_NAME", None)

    print(
        "MOTOLII_GPU_TEST "
        f"backend={args.backend!r} adapter={args.adapter or '<any>'!r} "
        f"timeout={args.timeout_seconds:.1f}s command={argv!r}",
        file=sys.stderr,
        flush=True,
    )
    popen_args: dict[str, object] = {}
    if os.name == "nt":
        popen_args["creationflags"] = subprocess.CREATE_NEW_PROCESS_GROUP
    else:
        popen_args["start_new_session"] = True
    process = subprocess.Popen(argv, env=env, **popen_args)
    try:
        return process.wait(timeout=args.timeout_seconds)
    except subprocess.TimeoutExpired:
        print(
            "MOTOLII_GPU_TEST timeout: terminating the whole test process tree "
            f"after {args.timeout_seconds:.1f}s",
            file=sys.stderr,
            flush=True,
        )
        stop_process_tree(process)
        return EX_TIMEOUT


if __name__ == "__main__":
    raise SystemExit(main())
