#!/usr/bin/env python3
"""Run the optional pinned Basic Memory index over an HVR-D01 projection."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import subprocess
import sys


BASIC_MEMORY_VERSION = "0.22.1"
EMBEDDING_MODEL = "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2"
PROJECT = "motolii-historical-recovery"
MARKER_NAME = "hvr-index.json"


class RunnerError(Exception):
    """The runner inputs or the immutable projection contract are invalid."""


def resolved(path: Path) -> Path:
    return path.resolve(strict=False)


def is_within(child: Path, parent: Path) -> bool:
    try:
        child.relative_to(parent)
    except ValueError:
        return False
    return True


def validate_boundaries(repo_root: Path, projection: Path | None, state: Path) -> tuple[Path, Path | None, Path]:
    if not repo_root.is_absolute() or not state.is_absolute():
        raise RunnerError("--repo-root and --state must be absolute paths")
    if projection is not None and not projection.is_absolute():
        raise RunnerError("--projection must be an absolute path")
    repo = resolved(repo_root)
    if not repo.is_dir():
        raise RunnerError(f"--repo-root is not a directory: {repo_root}")
    if projection is not None and projection.is_symlink():
        raise RunnerError("--projection must not be a symlink")
    state_path = resolved(state)
    projection_path = resolved(projection) if projection is not None else None
    if is_within(state_path, repo):
        raise RunnerError("--state must be outside --repo-root")
    if projection_path is not None:
        if is_within(projection_path, repo):
            raise RunnerError("--projection must be outside --repo-root")
        if is_within(state_path, projection_path) or is_within(projection_path, state_path):
            raise RunnerError("--projection and --state must not overlap")
    return repo, projection_path, state_path


def projection_tree_hash(projection: Path) -> str:
    if not projection.is_dir() or projection.is_symlink():
        raise RunnerError(f"--projection is not a directory: {projection}")
    for required in (projection / "manifest.tsv", projection / "edges.tsv"):
        if not required.is_file() or required.is_symlink():
            raise RunnerError(f"projection is missing required regular file: {required.name}")
    if not (projection / "nodes").is_dir() or (projection / "nodes").is_symlink():
        raise RunnerError("projection nodes/ must be a directory")

    files: list[tuple[PurePosixPath, Path]] = []
    for directory, names, filenames in os.walk(projection, followlinks=False):
        directory_path = Path(directory)
        for name in names + filenames:
            path = directory_path / name
            if path.is_symlink():
                raise RunnerError(f"projection contains a symlink: {path.relative_to(projection)}")
        for filename in filenames:
            path = directory_path / filename
            if not path.is_file():
                raise RunnerError(f"projection contains a non-regular file: {path.relative_to(projection)}")
            files.append((PurePosixPath(path.relative_to(projection).as_posix()), path))
    digest = hashlib.sha256()
    for relative, path in sorted(files):
        digest.update(os.fsencode(relative.as_posix()))
        digest.update(b"\0")
        with path.open("rb") as source:
            for chunk in iter(lambda: source.read(1024 * 1024), b""):
                digest.update(chunk)
    return digest.hexdigest()


def child_environment(state: Path) -> dict[str, str]:
    return {
        "PATH": os.environ.get("PATH", os.defpath),
        # uvxの依存buildがHOME配下のtoolchain managerを解決できるようにする。
        "HOME": os.environ.get("HOME", os.fspath(state)),
        "BASIC_MEMORY_CONFIG_DIR": os.fspath(state / "config"),
        "BASIC_MEMORY_AUTO_UPDATE": "false",
        "BASIC_MEMORY_SYNC_CHANGES": "false",
        "BASIC_MEMORY_ENSURE_FRONTMATTER_ON_SYNC": "false",
        "BASIC_MEMORY_SEMANTIC_SEARCH_ENABLED": "true",
        "BASIC_MEMORY_SEMANTIC_EMBEDDING_PROVIDER": "fastembed",
        "BASIC_MEMORY_SEMANTIC_EMBEDDING_MODEL": EMBEDDING_MODEL,
        "BASIC_MEMORY_SEMANTIC_EMBEDDING_CACHE_DIR": os.fspath(state / "models"),
        "UV_CACHE_DIR": os.fspath(state / "uv-cache"),
    }


def command_prefix(offline: bool) -> list[str]:
    command = ["uvx"]
    if offline:
        command.append("--offline")
    return command + ["--from", f"basic-memory=={BASIC_MEMORY_VERSION}", "basic-memory"]


def run(command: list[str], environment: dict[str, str], state: Path) -> int:
    try:
        result = subprocess.run(command, env=environment, cwd=state, check=False)
    except FileNotFoundError as error:
        raise RunnerError("uvx was not found on PATH") from error
    return result.returncode


def marker_payload(tree_hash: str) -> dict[str, object]:
    return {
        "schema": 1,
        "basic_memory_version": BASIC_MEMORY_VERSION,
        "embedding_model": EMBEDDING_MODEL,
        "project": PROJECT,
        "projection_tree_sha256": tree_hash,
    }


def index(repo_root: Path, projection: Path, state: Path, offline: bool) -> None:
    _repo, checked_projection, checked_state = validate_boundaries(repo_root, projection, state)
    if checked_projection is None:
        raise RunnerError("--projection is required for indexing")
    before = projection_tree_hash(checked_projection)
    if checked_state.exists() and not checked_state.is_dir():
        raise RunnerError("--state must be a directory or a new path")
    checked_state.mkdir(parents=True, exist_ok=True)
    marker = checked_state / MARKER_NAME
    marker.unlink(missing_ok=True)
    environment = child_environment(checked_state)
    prefix = command_prefix(offline)
    commands = (
        prefix + ["project", "add", PROJECT, os.fspath(checked_projection), "--local", "--default"],
        prefix + ["reindex", "--project", PROJECT, "--full"],
        prefix + ["status", "--project", PROJECT, "--wait", "--timeout", "300", "--json", "--local"],
    )
    for command in commands:
        if run(command, environment, checked_state):
            raise RunnerError("Basic Memory command failed")
    after = projection_tree_hash(checked_projection)
    if after != before:
        raise RunnerError("projection changed while Basic Memory was indexing it")
    marker.write_text(json.dumps(marker_payload(before), sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8")


def load_marker(state: Path) -> None:
    marker = state / MARKER_NAME
    try:
        payload = json.loads(marker.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RunnerError("cannot read a valid index marker") from error
    if not isinstance(payload, dict) or payload != marker_payload(payload.get("projection_tree_sha256")):
        raise RunnerError("index marker does not match the fixed runner contract")
    tree_hash = payload["projection_tree_sha256"]
    if not isinstance(tree_hash, str) or len(tree_hash) != 64 or any(character not in "0123456789abcdef" for character in tree_hash):
        raise RunnerError("index marker has an invalid projection hash")


def search(repo_root: Path, state: Path, query: str, page_size: int, offline: bool) -> int:
    _repo, _projection, checked_state = validate_boundaries(repo_root, None, state)
    if not query:
        raise RunnerError("--query must not be empty")
    if not 1 <= page_size <= 100:
        raise RunnerError("--page-size must be between 1 and 100")
    load_marker(checked_state)
    command = command_prefix(offline) + [
        "tool", "search-notes", query, "--hybrid", "--project", PROJECT, "--local", "--page-size", str(page_size),
    ]
    return run(command, child_environment(checked_state), checked_state)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    subcommands = parser.add_subparsers(dest="command", required=True)
    index_parser = subcommands.add_parser("index")
    index_parser.add_argument("--repo-root", required=True, type=Path)
    index_parser.add_argument("--projection", required=True, type=Path)
    index_parser.add_argument("--state", required=True, type=Path)
    index_parser.add_argument("--offline", action="store_true")
    search_parser = subcommands.add_parser("search")
    search_parser.add_argument("--repo-root", required=True, type=Path)
    search_parser.add_argument("--state", required=True, type=Path)
    search_parser.add_argument("--query", required=True)
    search_parser.add_argument("--page-size", type=int, default=10)
    search_parser.add_argument("--offline", action="store_true")
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    try:
        if args.command == "index":
            index(args.repo_root, args.projection, args.state, args.offline)
            return 0
        return search(args.repo_root, args.state, args.query, args.page_size, args.offline)
    except RunnerError as error:
        print(f"historical semantic index failed: {error}", file=sys.stderr)
        return 1
    except OSError as error:
        print(f"historical semantic index failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
