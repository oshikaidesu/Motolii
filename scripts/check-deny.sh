#!/usr/bin/env bash
# cargo-deny, sources only: every crate comes from crates.io or from the two forks Motolii owns (deny.toml).
# Advisories, licences and duplicate versions are observed, not enforced (see docs/reviews/2026-09-30-external-police-spike.md).
set -euo pipefail
cd "$(dirname "$0")/.."
cargo deny check sources
