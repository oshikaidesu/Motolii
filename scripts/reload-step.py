#!/usr/bin/env python3
"""Record a development command; elapsed targets are advisory, timeout is opt-in."""
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("need", "ruler", "gap", "owner", "class"):
        parser.add_argument("--" + name, required=True)
    parser.add_argument("--timeout", type=float, default=None)
    parser.add_argument("--advisory-seconds", type=float, default=None)
    parser.add_argument("--receipt", required=True)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command or (args.timeout is not None and args.timeout <= 0):
        parser.error("a command and a positive optional timeout are required")
    if args.advisory_seconds is not None and args.advisory_seconds <= 0:
        parser.error("advisory duration must be positive")
    if any(not getattr(args, field).strip() for field in ("need", "ruler", "gap", "owner", "class")):
        parser.error("exception fields must be concrete, nonempty text")
    record = {
        "schema": 1, "event": "development_exception", "execution_class": getattr(args, "class"),
        "build_need": args.need, "external_ruler": args.ruler,
        "hotpatch_or_check_gap": args.gap, "repair_owner": args.owner,
        "command": command, "cwd": os.getcwd(), "unix_ms": int(time.time() * 1000),
        "hard_timeout_seconds": args.timeout,
        "advisory_seconds": args.advisory_seconds or (5 if getattr(args, "class") == "THIN_PATCH" else 60),
    }
    receipt = Path(args.receipt)
    receipt.parent.mkdir(parents=True, exist_ok=True)
    def emit(status):
        row = dict(record, status=status)
        print("MOTOLII_RELOAD " + json.dumps(row, ensure_ascii=False), flush=True)
        with receipt.open("a") as out:
            out.write(json.dumps(row, ensure_ascii=False) + "\n")
    start = time.monotonic()
    try:
        process = subprocess.Popen(command, start_new_session=True)
    except OSError as error:
        record.update(elapsed_ms=round((time.monotonic() - start) * 1000), exit_code=127, error=str(error))
        emit("spawn_failed")
        return 127
    record["pid"] = process.pid
    emit("started")
    status = "finished"
    try:
        code = process.wait(timeout=args.timeout)
    except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
        status = "timeout" if isinstance(error, subprocess.TimeoutExpired) else "interrupted"
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=1)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=1)
        code = 124 if status == "timeout" else 130
    elapsed_ms = round((time.monotonic() - start) * 1000)
    slow = elapsed_ms > record["advisory_seconds"] * 1000
    record.update(elapsed_ms=elapsed_ms, exit_code=code, exceeded_advisory=slow)
    if status == "finished":
        status = "failed" if code else "slow_completed" if slow else "finished"
    emit(status)
    return code


if __name__ == "__main__":
    sys.exit(main())
