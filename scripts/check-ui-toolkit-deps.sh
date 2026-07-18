#!/usr/bin/env bash
# M3E-1: UI toolkit直接依存はmotolii-uiのみ許可。spikes/はworkspace外で対象外。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo test -p motolii-testkit --test ui_toolkit_dep_policy workspace_has_no_ui_toolkit_outside_ui_allowlist -- --exact
