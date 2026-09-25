#!/usr/bin/env bash
# Fast suite: the render architecture's invariants (one tick, one preparation, one world light,
# shared transmission, plate/reflection ordering), in one test process so the GPU start-up cost
# (~9s, re_renderer's shader file watchers under IS_IN_RERUN_WORKSPACE) is paid once.
# Everything else (pixel parity, export, feedback, video, UI, Glass Garden) is the full suite.
set -euo pipefail
cd "$(dirname "$0")/../motolii"
exec cargo test -q -p motolii-render --lib -- engine::frame_graph::tick_tests
