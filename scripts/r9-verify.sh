#!/usr/bin/env bash
# R9: 実素材での書き出し + B-4(プレビュー/書き出し一致)自動検証 + GUIプレビュー起動
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="${ROOT_DIR}/.tools/env.sh"

if [[ -f "${ENV_FILE}" ]]; then
  # shellcheck source=/dev/null
  source "${ENV_FILE}"
fi

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <project.json> [--no-preview]" >&2
  echo "  project.json: paths relative to the json file directory" >&2
  echo "  --no-preview: skip GUI (default: launch spikes/r9-preview after verify)" >&2
  exit 1
fi

PROJECT="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
shift || true
PREVIEW=true
if [[ "${1:-}" == "--no-preview" ]]; then
  PREVIEW=false
fi

if ! command -v ffmpeg >/dev/null; then
  echo "error: ffmpeg not on PATH (run scripts/setup-local-deps.sh)" >&2
  exit 1
fi

cd "${ROOT_DIR}"
TOLERANCE="${R9_TOLERANCE:-8}"
echo "==> export + B-4 verify (tolerance ${TOLERANCE}, qp0 recommended in project json)"
set +e
VERIFY_OUT="$(cargo run -q -p motoly-cli -- verify-b4 --project "${PROJECT}" --export --tolerance "${TOLERANCE}" 2>&1)"
VERIFY_RC=$?
set -e
echo "${VERIFY_OUT}"
if [[ "${VERIFY_RC}" -ne 0 ]]; then
  echo ""
  echo "WARNING: B-4 verify failed — opening preview anyway for manual check"
fi

echo ""
echo "==> R9 manual checklist (docs/reviews/2026-07-10-R9-real-material-checklist.md)"
echo "  - Subjective: watch output mp4 full screen — MV quality OK?"

if [[ "${PREVIEW}" == true ]]; then
  echo "==> launching GUI preview (close window to finish)..."
  (cd spikes/r9-preview && cargo run -q --release -- "${PROJECT}")
else
  echo "  (GUI skipped; run without --no-preview to open preview)"
fi

exit "${VERIFY_RC}"
