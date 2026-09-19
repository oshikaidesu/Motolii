"""Measure offline fallback overhead, or explicitly opt into a live synthetic smoke benchmark."""
from __future__ import annotations

import argparse
import json
import math
import os
from pathlib import Path
import statistics
import tempfile

from jev_triage import MODEL, classify

# Synthetic smoke labels, not evidence of accuracy on real Motolii failures.
CASES = [
    ("compile", "document", "error[E0308]: mismatched types in motolii/crates/motolii-doc/src/store.rs: expected LayerId, found u32"),
    ("test", "ui", "motolii/ui/test/inspector_test.dart: Expected: 42 Actual: 0. Some tests failed."),
    ("check", "tooling", "scripts/check-docs.sh: NG: broken documentation link docs/README.md -> missing.md. FAILED"),
    ("environment", "render", "motolii/crates/motolii-render: failed to download dependency: Could not resolve host index.crates.io"),
]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--live", action="store_true", help="Send synthetic cases to TypeSafe; requires TYPESAFE_API_KEY")
    parser.add_argument("--repeats", type=int, default=5)
    args = parser.parse_args()
    if not 1 <= args.repeats <= 100:
        parser.error("repeats must be between 1 and 100")
    if args.live and not os.environ.get("TYPESAFE_API_KEY"):
        parser.error("live benchmark requires TYPESAFE_API_KEY")
    if not args.live:
        os.environ.pop("TYPESAFE_API_KEY", None)
    results = []
    with tempfile.TemporaryDirectory() as directory:
        logs = Path(directory)
        (logs / "stderr.log").write_text("")
        for _ in range(args.repeats):
            for kind, owner, log in CASES:
                (logs / "stdout.log").write_text(log)
                result = classify({"exit_code": 1}, logs, "route")
                result["matches_synthetic_label"] = result["failure_kind"] == kind and result["owner"] == owner
                results.append(result)
    latencies = sorted(r["triage_latency_ms"] for r in results)
    accepted = [r for r in results if r["source"] == "jev"]
    print(json.dumps({
        "measurement": "live_synthetic_smoke" if args.live else "offline_missing_key_fallback",
        "model": MODEL, "samples": len(results),
        "triage_p50_ms": statistics.median(latencies),
        "triage_p95_ms": latencies[math.ceil(len(latencies) * 0.95) - 1],
        "jev_requests_attempted": sum(r["jev_requests_attempted"] for r in results),
        "accepted_routes": len(accepted), "fallbacks": len(results) - len(accepted),
        "accepted_label_matches": sum(r["matches_synthetic_label"] for r in accepted),
        "potential_classification_calls_avoided": sum(r["potential_classification_calls_avoided"] for r in results),
        "production_llm_calls_saved": None, "end_to_end_speedup": None,
        "results": results,
    }, indent=2))


if __name__ == "__main__":
    main()
