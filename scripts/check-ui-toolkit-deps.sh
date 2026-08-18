#!/usr/bin/env bash
# M3E-1: UI toolkit直接依存はmotolii-uiのみ許可。slintはcrates/でゼロ。spikes/はworkspace外で対象外。
# 2026-08-18 M-0: iced系は motolii-shell-iced のみ許可(同crateはegui系のallowlistに載せない)。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo test -p motolii-testkit --test ui_toolkit_dep_policy workspace_has_no_ui_toolkit_outside_ui_allowlist -- --exact
cargo test -p motolii-testkit --test ui_toolkit_dep_policy workspace_has_no_slint_in_crates -- --exact
cargo test -p motolii-testkit --test ui_toolkit_dep_policy workspace_has_no_iced_outside_shell_allowlist -- --exact
cargo test -p motolii-testkit --test ui_toolkit_dep_policy the_iced_shell_is_not_on_the_egui_allowlist -- --exact
