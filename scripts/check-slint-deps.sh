#!/usr/bin/env bash
# M3E-1: slint 直接依存は motolii-ui のみ許可。spikes/ は workspace 外で対象外。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo test -p motolii-testkit --test slint_dep_policy workspace_has_no_slint_outside_ui_allowlist -- --exact
