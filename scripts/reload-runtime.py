#!/usr/bin/env python3
"""One pinned warm development process, with recorded baseline exceptions."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import time

REPO = Path(__file__).resolve().parent.parent
PRODUCT = REPO / "motolii"
DESCRIPTOR = PRODUCT / "reference/dioxus-cli-runtime.json"


def binary():
    descriptor = json.loads(DESCRIPTOR.read_text())
    path = Path(os.environ.get("MOTOLII_DX_BIN", str(Path.home() / ".local/bin" / descriptor["binary"])))
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    if digest != descriptor["sha256"]:
        raise RuntimeError("fixed CLI checksum mismatch: " + str(path))
    return path, descriptor


def inventory():
    result = subprocess.run(["ps", "-axo", "pid=,ppid=,state=,command="], capture_output=True, text=True, check=True)
    rows = []
    for line in result.stdout.splitlines():
        fields = line.strip().split(None, 3)
        if len(fields) != 4:
            continue
        pid, parent, state, command = fields
        try:
            args = shlex.split(command)
        except ValueError:
            continue
        if args and state.startswith("T") and Path(args[0]).name in ("cargo", "rustc", "dx"):
            stopped = subprocess.run(["lsof", "-a", "-p", pid, "-d", "cwd", "-Fn"], capture_output=True, text=True, check=True)
            cwd = next((s[1:] for s in stopped.stdout.splitlines() if s.startswith("n")), None)
            if cwd is None:
                raise RuntimeError("cannot inspect stopped compiler: " + pid)
            if Path(cwd).resolve() == PRODUCT.resolve():
                raise RuntimeError("stopped product build process: " + pid)
        if len(args) < 2 or args[1] != "serve":
            continue
        name = Path(args[0]).name
        if name != "dx" and not name.startswith("motolii-dx-"):
            continue
        # The command's working directory determines its project; shell text is never a match.
        cwd_result = subprocess.run(["lsof", "-a", "-p", pid, "-d", "cwd", "-Fn"], capture_output=True, text=True)
        if cwd_result.returncode != 0:
            raise RuntimeError("cannot inspect dev process working directory: " + pid)
        cwd = next((s[1:] for s in cwd_result.stdout.splitlines() if s.startswith("n")), None)
        if cwd is None:
            raise RuntimeError("dev process has no observable working directory: " + pid)
        if Path(cwd).resolve() == PRODUCT.resolve():
            rows.append(dict(pid=int(pid), parent=int(parent), state=state, executable=args[0], cwd=cwd))
    return rows


def state_dir():
    key = hashlib.sha256(str(REPO).encode()).hexdigest()[:16]
    directory = Path.home() / ".local/state/motolii-reload" / key
    directory.mkdir(parents=True, exist_ok=True)
    return directory


def app_inventory():
    executable = (PRODUCT / "target/dx/motolii/debug/macos/Motolii.app/Contents/MacOS/motolii").resolve()
    result = subprocess.run(["ps", "-axo", "pid=,ppid=,state=,comm="], capture_output=True, text=True, check=True)
    apps = []
    for line in result.stdout.splitlines():
        fields = line.strip().split(None, 3)
        if len(fields) == 4 and Path(fields[3]).resolve() == executable:
            apps.append(dict(pid=int(fields[0]), parent=int(fields[1]), state=fields[2], executable=fields[3]))
    return apps


def reusable(rows, path):
    return len(rows) == 1 and Path(rows[0]["executable"]).resolve() == path.resolve() and not rows[0]["state"].startswith("T")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["serve", "doctor", "inventory", "binary-info"])
    parser.add_argument("args", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.action == "inventory":
        print(json.dumps(inventory()))
        return 0
    path, descriptor = binary()
    if args.action == "binary-info":
        print(json.dumps(dict(descriptor, path=str(path))))
        return 0
    rows = inventory()
    if args.action == "doctor":
        apps = app_inventory()
        healthy = not rows or reusable(rows, path)
        status = "unmanaged_app" if apps and not rows else "warm" if rows and healthy else "idle" if healthy else "invalid_server"
        print(json.dumps(dict(binary=str(path), sha256=descriptor["sha256"], processes=rows, apps=apps, status=status, runtime=str(state_dir()))))
        return 0 if healthy and status != "unmanaged_app" else 1
    directory = state_dir()
    with (directory / "serve.lock").open("a+") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            rows = inventory()
            if reusable(rows, path):
                print("MOTOLII_RELOAD " + json.dumps(dict(event="warm_session_reused", **rows[0])))
                return 0
            raise RuntimeError("another launcher owns the lock; inspect doctor before launching")
        if rows:
            if reusable(rows, path):
                print("MOTOLII_RELOAD " + json.dumps(dict(event="warm_session_reused", **rows[0])))
                return 0
            raise RuntimeError("existing unmanaged or duplicate dev process; no second launch: " + json.dumps(rows))
        apps = app_inventory()
        if apps:
            raise RuntimeError("Motolii is open without a reload server; preserve/save the open document and close that app before starting a baseline. No build or second app was started: " + json.dumps(apps))
        fields = {
            "build_need": "MOTOLII_BUILD_NEED", "external_ruler": "MOTOLII_EXTERNAL_RULER",
            "hotpatch_or_check_gap": "MOTOLII_HOTPATCH_OR_CHECK_GAP", "repair_owner": "MOTOLII_REPAIR_OWNER",
        }
        permit = {field: os.environ.get(variable, "").strip() for field, variable in fields.items()}
        defaults = {
            "build_need": "Initial baseline for the requested development session",
            "external_ruler": "Dioxus 0.7.10 requires a Fat baseline before Thin patching",
            "hotpatch_or_check_gap": "No live warm session exists for this workspace",
            "repair_owner": "motolii-runtime",
        }
        permit = {field: value or defaults[field] for field, value in permit.items()}
        permit.update(cause="initial_baseline", unix_ms=int(time.time() * 1000), command=[str(path), "serve", "--hotpatch", "--interactive", "false", *args.args])
        permit_path = directory / "baseline-permit.json"
        permit_path.write_text(json.dumps(permit) + "\n")
        os.environ["MOTOLII_RUNTIME_SOURCE_REGISTRATION"] = str(directory / "runtime-sources.json")
        os.environ["MOTOLII_RELOAD_SESSION"] = str(os.getpid()) + "-" + str(time.time_ns())
        os.environ["MOTOLII_BASELINE_PERMIT"] = str(permit_path)
        os.environ["MOTOLII_RELOAD_EVENTS"] = str(directory / "decisions.jsonl")
        os.chdir(PRODUCT)
        os.set_inheritable(lock.fileno(), True)
        print("MOTOLII_RELOAD " + json.dumps(dict(event="serve_launch", pid=os.getpid(), **permit)), flush=True)
        os.execv(str(path), permit["command"])


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print("MOTOLII_RELOAD " + json.dumps(dict(event="operational_guard", decision="blocked", reason=str(error))), file=sys.stderr)
        sys.exit(1)
