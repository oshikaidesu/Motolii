#!/usr/bin/env python3
"""基盤ゲートの機械可読状態を読む共通部品。"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any


MANIFEST_RELATIVE = Path("next/reference/foundation/phase.json")
VALID_MODES = {"serial", "parallel"}
VALID_PARALLEL_STATES = {"locked", "unlocked"}
VALID_STAGE_KINDS = {"serial", "parallel"}
VALID_STAGE_STATUSES = {"open", "blocked", "closed", "locked"}


class FoundationPhaseError(ValueError):
    """マニフェストが段階ゲートとして矛盾している。"""


def _require_string(value: Any, name: str) -> str:
    if not isinstance(value, str) or not value:
        raise FoundationPhaseError(f"{name} must be a non-empty string")
    return value


def _require_string_list(value: Any, name: str) -> list[str]:
    if not isinstance(value, list) or any(not isinstance(item, str) or not item for item in value):
        raise FoundationPhaseError(f"{name} must be a string list")
    return value


def _check_acyclic(stages: dict[str, dict[str, Any]]) -> None:
    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(stage_id: str) -> None:
        if stage_id in visiting:
            raise FoundationPhaseError(f"stage dependency cycle at {stage_id}")
        if stage_id in visited:
            return
        visiting.add(stage_id)
        for dependency in stages[stage_id]["depends_on"]:
            visit(dependency)
        visiting.remove(stage_id)
        visited.add(stage_id)

    for stage_id in stages:
        visit(stage_id)


def validate_manifest(data: Any) -> dict[str, Any]:
    if not isinstance(data, dict):
        raise FoundationPhaseError("manifest root must be an object")
    if data.get("schema_version") != 1:
        raise FoundationPhaseError("unsupported schema_version")

    current_stage = _require_string(data.get("current_stage"), "current_stage")
    mode = _require_string(data.get("mode"), "mode")
    parallel_components = _require_string(
        data.get("parallel_components"), "parallel_components"
    )
    parallel_stage = _require_string(data.get("parallel_stage"), "parallel_stage")
    if mode not in VALID_MODES:
        raise FoundationPhaseError(f"unknown mode: {mode}")
    if parallel_components not in VALID_PARALLEL_STATES:
        raise FoundationPhaseError(f"unknown parallel_components: {parallel_components}")

    raw_stages = data.get("stages")
    if not isinstance(raw_stages, list) or not raw_stages:
        raise FoundationPhaseError("stages must be a non-empty list")
    stages: dict[str, dict[str, Any]] = {}
    for raw_stage in raw_stages:
        if not isinstance(raw_stage, dict):
            raise FoundationPhaseError("each stage must be an object")
        stage_id = _require_string(raw_stage.get("id"), "stage.id")
        if stage_id in stages:
            raise FoundationPhaseError(f"duplicate stage id: {stage_id}")
        kind = _require_string(raw_stage.get("kind"), f"{stage_id}.kind")
        status = _require_string(raw_stage.get("status"), f"{stage_id}.status")
        if kind not in VALID_STAGE_KINDS:
            raise FoundationPhaseError(f"unknown stage kind: {kind}")
        if status not in VALID_STAGE_STATUSES:
            raise FoundationPhaseError(f"unknown stage status: {status}")
        stages[stage_id] = {
            **raw_stage,
            "depends_on": _require_string_list(
                raw_stage.get("depends_on"), f"{stage_id}.depends_on"
            ),
        }

    if current_stage not in stages:
        raise FoundationPhaseError(f"current_stage is not declared: {current_stage}")
    if parallel_stage not in stages:
        raise FoundationPhaseError(f"parallel_stage is not declared: {parallel_stage}")

    for stage_id, stage in stages.items():
        for dependency in stage["depends_on"]:
            if dependency not in stages:
                raise FoundationPhaseError(f"{stage_id} depends on unknown stage: {dependency}")
    _check_acyclic(stages)

    required_closed = _require_string_list(
        data.get("parallel_unlock_requires_closed"),
        "parallel_unlock_requires_closed",
    )
    for stage_id in required_closed:
        if stage_id not in stages:
            raise FoundationPhaseError(f"parallel unlock requires unknown stage: {stage_id}")

    if parallel_components == "unlocked":
        if mode != "parallel":
            raise FoundationPhaseError("parallel_components=unlocked requires mode=parallel")
        if current_stage != parallel_stage:
            raise FoundationPhaseError("unlocked parallel state must be at parallel_stage")
        not_closed = [stage_id for stage_id in required_closed if stages[stage_id]["status"] != "closed"]
        if not_closed:
            raise FoundationPhaseError(
                "parallel unlock prerequisites are not closed: " + ", ".join(not_closed)
            )
    else:
        if mode != "serial":
            raise FoundationPhaseError("parallel_components=locked requires mode=serial")
        if stages[parallel_stage]["status"] != "locked":
            raise FoundationPhaseError("locked parallel state requires parallel stage status=locked")

    return {**data, "stages": stages}


def load_phase(root: Path, *, required: bool = True) -> dict[str, Any] | None:
    path = root / MANIFEST_RELATIVE
    if not path.exists():
        if required:
            raise FoundationPhaseError(f"missing phase manifest: {path}")
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise FoundationPhaseError(f"invalid JSON in {path}: {error}") from error
    return validate_manifest(data)


def summary(data: dict[str, Any]) -> str:
    return (
        f"current_stage={data['current_stage']} "
        f"mode={data['mode']} "
        f"parallel_components={data['parallel_components']}"
    )
