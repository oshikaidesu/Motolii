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


def stop_process_tree(process: subprocess.Popen[bytes]) -> None:
    if os.name == "nt":
        subprocess.run(
            ["taskkill", "/PID", str(process.pid), "/T", "/F"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
    else:
        # This is a hard wall timeout, not graceful shutdown. The child was
        # started in its own session, so SIGKILL to the process group kills the
        # leader and every descendant even if one ignores SIGTERM.
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    try:
        process.wait(timeout=5.0)
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
