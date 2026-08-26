#!/usr/bin/env python3
"""基盤ゲートの状態と依存関係を検査する。"""

from __future__ import annotations

import sys
from pathlib import Path

from foundation_phase import FoundationPhaseError, load_phase, summary


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    try:
        phase = load_phase(root)
    except FoundationPhaseError as error:
        print(f"FOUNDATION_PHASE=RED {error}")
        return 1

    assert phase is not None
    print(f"FOUNDATION_PHASE=GREEN {summary(phase)}")
    print(f"PARALLEL_AUTHORIZATION={phase['parallel_components'].upper()}")
    for stage_id, stage in phase["stages"].items():
        dependencies = ",".join(stage["depends_on"]) or "-"
        print(f"STAGE {stage_id} status={stage['status']} depends_on={dependencies}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
