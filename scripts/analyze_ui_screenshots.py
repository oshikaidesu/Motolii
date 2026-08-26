#!/usr/bin/env python3
"""実窓観測PNGを機械的に測り、シナリオ単位の赤/緑とスコアを出す。

この検査器は「美しい」を採点しない。空白化、構造の消失、状態操作が画面へ
反映されないことを、画像から再現可能な数値で検出する。意味(どの Message を
送ったか)はシナリオ台帳、画素(どんな結果になったか)はここが担当する。
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path
from typing import Any

try:
    import numpy as np
    from PIL import Image
except ImportError as error:  # pragma: no cover - exercised by the CLI environment
    raise SystemExit(
        "RED: 画像解析には Pillow と numpy が必要です "
        f"({error})"
    ) from error


SCHEMA_VERSION = 1
MAX_ANALYSIS_EDGE = 640


def _fail(message: str) -> ValueError:
    return ValueError(f"ui-observation manifest: {message}")


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        manifest = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise _fail(str(error)) from error
    if not isinstance(manifest, dict):
        raise _fail("root must be an object")
    if manifest.get("version") != SCHEMA_VERSION:
        raise _fail(f"version must be {SCHEMA_VERSION}")
    if manifest.get("source") not in {"real-window", "synthetic"}:
        raise _fail("source must be real-window or synthetic")
    scenarios = manifest.get("scenarios")
    if not isinstance(scenarios, list) or not scenarios:
        raise _fail("scenarios must be a non-empty array")
    ids: set[str] = set()
    for scenario in scenarios:
        if not isinstance(scenario, dict):
            raise _fail("each scenario must be an object")
        scenario_id = scenario.get("id")
        if not isinstance(scenario_id, str) or not scenario_id:
            raise _fail("scenario id must be a non-empty string")
        if scenario_id in ids:
            raise _fail(f"duplicate scenario id: {scenario_id}")
        ids.add(scenario_id)
        argv = scenario.get("argv", [])
        if not isinstance(argv, list) or not all(isinstance(value, str) for value in argv):
            raise _fail(f"{scenario_id}: argv must be an array of strings")
        if "--screenshot" in argv:
            raise _fail(
                f"{scenario_id}: --screenshot is the headless instrument, not real-window evidence"
            )

    analysis = manifest.get("analysis")
    if not isinstance(analysis, dict):
        raise _fail("analysis must be an object")
    regions = analysis.get("regions")
    if not isinstance(regions, dict) or "window" not in regions:
        raise _fail("analysis.regions.window is required")
    for name, box in regions.items():
        if (
            not isinstance(box, list)
            or len(box) != 4
            or not all(isinstance(value, (int, float)) and math.isfinite(value) for value in box)
            or not (0 <= box[0] <= 1 and 0 <= box[1] <= 1)
            or not (0 < box[2] <= 1 and 0 < box[3] <= 1)
            or box[0] + box[2] > 1
            or box[1] + box[3] > 1
        ):
            raise _fail(f"region {name} must be normalized [x, y, width, height]")
    checks = analysis.get("checks")
    if not isinstance(checks, list) or not checks:
        raise _fail("analysis.checks must be a non-empty array")
    for check in checks:
        if not isinstance(check, dict):
            raise _fail("each analysis check must be an object")
        if not all(isinstance(check.get(key), str) and check.get(key) for key in ("id", "metric", "region", "op")):
            raise _fail("each check needs id, metric, region, and op")
        if check["region"] not in regions:
            raise _fail(f"check {check['id']} refers to unknown region {check['region']}")
        if check["op"] == "between":
            value = check.get("value")
            if not isinstance(value, list) or len(value) != 2:
                raise _fail(f"check {check['id']} between value must have two numbers")
        elif not isinstance(check.get("value"), (int, float)):
            raise _fail(f"check {check['id']} value must be numeric")
    return manifest


def _image_array(path: Path) -> np.ndarray:
    try:
        image = Image.open(path).convert("RGBA")
    except (OSError, ValueError) as error:
        raise ValueError(f"cannot read PNG {path}: {error}") from error
    scale = min(1.0, MAX_ANALYSIS_EDGE / max(image.width, image.height))
    if scale < 1.0:
        image = image.resize(
            (max(1, round(image.width * scale)), max(1, round(image.height * scale))),
            Image.Resampling.BILINEAR,
        )
    return np.asarray(image, dtype=np.float32) / 255.0


def _region(array: np.ndarray, box: list[float]) -> np.ndarray:
    height, width = array.shape[:2]
    x, y, region_width, region_height = box
    left = min(width - 1, max(0, round(x * width)))
    top = min(height - 1, max(0, round(y * height)))
    right = min(width, max(left + 1, round((x + region_width) * width)))
    bottom = min(height, max(top + 1, round((y + region_height) * height)))
    return array[top:bottom, left:right]


def _metrics(array: np.ndarray) -> dict[str, float]:
    rgb = array[..., :3]
    alpha = array[..., 3]
    luma = 0.2126 * rgb[..., 0] + 0.7152 * rgb[..., 1] + 0.0722 * rgb[..., 2]
    horizontal = np.abs(luma[:, 1:] - luma[:, :-1])
    vertical = np.abs(luma[1:, :] - luma[:-1, :])
    if horizontal.size and vertical.size:
        edge = max(float(horizontal.mean()), float(vertical.mean()))
        edge_density = float(
            (
                (horizontal > 0.035).mean()
                + (vertical > 0.035).mean()
            )
            / 2.0
        )
    else:
        edge = 0.0
        edge_density = 0.0
    median = float(np.median(luma))
    activity = float(np.abs(luma - median).mean())
    color_std = float(np.std(rgb, axis=(0, 1)).mean())
    return {
        "width": float(array.shape[1]),
        "height": float(array.shape[0]),
        "mean_luma": float(luma.mean()),
        "luma_p95_p05": float(np.quantile(luma, 0.95) - np.quantile(luma, 0.05)),
        "luma_std": float(luma.std()),
        "color_std": color_std,
        "edge_mean": edge,
        "edge_density": edge_density,
        "activity": activity,
        "alpha_coverage": float((alpha > 0.01).mean()),
    }


def _delta(left: np.ndarray, right: np.ndarray) -> dict[str, float]:
    height = min(left.shape[0], right.shape[0])
    width = min(left.shape[1], right.shape[1])
    left_rgb = left[:height, :width, :3]
    right_rgb = right[:height, :width, :3]
    difference = np.abs(left_rgb - right_rgb)
    return {
        "mean_abs_rgb": float(difference.mean()),
        "changed_fraction": float((difference.max(axis=2) > 0.02).mean()),
    }


def _passes(actual: float, operator: str, expected: Any) -> bool:
    if operator == ">=":
        return actual + 1e-12 >= float(expected)
    if operator == ">":
        return actual > float(expected)
    if operator == "<=":
        return actual - 1e-12 <= float(expected)
    if operator == "<":
        return actual < float(expected)
    if operator == "==":
        return abs(actual - float(expected)) <= 1e-12
    if operator == "between":
        low, high = expected
        return float(low) <= actual <= float(high)
    raise _fail(f"unsupported operator: {operator}")


def _check_result(check: dict[str, Any], metrics: dict[str, dict[str, float]]) -> dict[str, Any]:
    region = check["region"]
    metric = check["metric"]
    if metric not in metrics[region]:
        raise _fail(f"metric {metric} is not available for region {region}")
    actual = metrics[region][metric]
    expected = check["value"]
    return {
        "id": check["id"],
        "region": region,
        "metric": metric,
        "operator": check["op"],
        "expected": expected,
        "actual": round(actual, 8),
        "pass": _passes(actual, check["op"], expected),
    }


def _scenario_by_id(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    return {scenario["id"]: scenario for scenario in manifest["scenarios"]}


def analyze_manifest(manifest_path: Path, artifact_dir: Path) -> dict[str, Any]:
    manifest = load_manifest(manifest_path)
    regions = manifest["analysis"]["regions"]
    checks = manifest["analysis"]["checks"]
    scenario_results: list[dict[str, Any]] = []
    scenario_lookup = _scenario_by_id(manifest)

    for scenario in manifest["scenarios"]:
        scenario_id = scenario["id"]
        scenario_dir = artifact_dir / "scenarios" / scenario_id
        image_path = scenario_dir / "window.png"
        result: dict[str, Any] = {
            "id": scenario_id,
            "label": scenario.get("label", scenario_id),
            "operation": scenario.get("operation", "unspecified"),
            "image": str(image_path.relative_to(artifact_dir)) if image_path.exists() else None,
            "status": "RED",
            "score": 0.0,
            "checks": [],
            "regions": {},
        }
        if manifest["source"] == "real-window":
            capture_meta = scenario_dir / "capture.json"
            if capture_meta.exists():
                try:
                    meta = json.loads(capture_meta.read_text(encoding="utf-8"))
                except (OSError, json.JSONDecodeError):
                    meta = {}
                if meta.get("source") != "real-window":
                    result["checks"].append(
                        {
                            "id": "capture-source-is-real-window",
                            "expected": "real-window",
                            "actual": meta.get("source"),
                            "pass": False,
                        }
                    )
            else:
                result["checks"].append(
                    {
                        "id": "capture-source-metadata-exists",
                        "expected": "capture.json",
                        "actual": None,
                        "pass": False,
                    }
                )

        if not image_path.exists():
            result["checks"].append(
                {
                    "id": "screenshot-exists",
                    "expected": str(image_path),
                    "actual": None,
                    "pass": False,
                }
            )
            scenario_results.append(result)
            continue

        try:
            array = _image_array(image_path)
            metrics = {
                name: _metrics(_region(array, box)) for name, box in regions.items()
            }
            result["regions"] = metrics
            result["checks"].extend(_check_result(check, metrics) for check in checks)

            reference_id = scenario.get("reference")
            delta_policy = scenario.get("delta")
            if reference_id and delta_policy:
                reference = scenario_lookup.get(reference_id)
                reference_path = (
                    artifact_dir / "scenarios" / reference_id / "window.png"
                    if reference
                    else None
                )
                if reference_path is None or not reference_path.exists():
                    result["checks"].append(
                        {
                            "id": "reference-screenshot-exists",
                            "expected": reference_id,
                            "actual": None,
                            "pass": False,
                        }
                    )
                else:
                    difference = _delta(array, _image_array(reference_path))
                    result["delta"] = difference
                    for metric, expected in delta_policy.items():
                        if metric not in difference:
                            raise _fail(f"delta metric {metric} is not available")
                        result["checks"].append(
                            {
                                "id": f"delta-{metric}-from-{reference_id}",
                                "metric": metric,
                                "reference": reference_id,
                                "operator": ">=",
                                "expected": expected,
                                "actual": round(difference[metric], 8),
                                "pass": difference[metric] + 1e-12 >= float(expected),
                            }
                        )
        except (OSError, ValueError) as error:
            result["checks"].append(
                {"id": "image-analysis-completes", "expected": "readable PNG", "actual": str(error), "pass": False}
            )

        passed = sum(1 for check in result["checks"] if check.get("pass") is True)
        total = len(result["checks"])
        result["passed"] = passed
        result["total"] = total
        result["score"] = round(100.0 * passed / total, 2) if total else 0.0
        result["status"] = "GREEN" if total > 0 and passed == total else "RED"
        scenario_results.append(result)

    passed_scenarios = sum(1 for result in scenario_results if result["status"] == "GREEN")
    overall = {
        "schema": SCHEMA_VERSION,
        "source": manifest["source"],
        "manifest": str(manifest_path),
        "artifact_dir": str(artifact_dir),
        "status": "GREEN" if passed_scenarios == len(scenario_results) else "RED",
        "score": round(
            sum(result["score"] for result in scenario_results) / len(scenario_results), 2
        ),
        "passed_scenarios": passed_scenarios,
        "scenario_count": len(scenario_results),
        "scenarios": scenario_results,
    }
    (artifact_dir / "scores.json").write_text(
        json.dumps(overall, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    with (artifact_dir / "scores.tsv").open("w", encoding="utf-8") as stream:
        stream.write("scenario\tstatus\tscore\tpassed\ttotal\timage\n")
        for result in scenario_results:
            stream.write(
                "\t".join(
                    str(result.get(key, ""))
                    for key in ("id", "status", "score", "passed", "total", "image")
                )
                + "\n"
            )
    return overall


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("artifact_dir", type=Path)
    args = parser.parse_args()
    try:
        result = analyze_manifest(args.manifest.resolve(), args.artifact_dir.resolve())
    except (OSError, ValueError) as error:
        print(f"RED: {error}")
        return 1
    print(
        f"UI_OBSERVATION source={result['source']} status={result['status']} "
        f"score={result['score']:.2f} "
        f"scenarios={result['passed_scenarios']}/{result['scenario_count']}"
    )
    for scenario in result["scenarios"]:
        print(
            f"{scenario['status']}: {scenario['id']} "
            f"score={scenario['score']:.2f} "
            f"checks={scenario.get('passed', 0)}/{scenario.get('total', len(scenario['checks']))}"
        )
    return 0 if result["status"] == "GREEN" else 1


if __name__ == "__main__":
    raise SystemExit(main())
