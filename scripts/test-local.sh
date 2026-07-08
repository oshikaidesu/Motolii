#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="${ROOT_DIR}/.tools/env.sh"

if [[ ! -f "${ENV_FILE}" ]]; then
  echo "Missing ${ENV_FILE}. Run scripts/setup-local-deps.sh first." >&2
  exit 1
fi

# shellcheck source=/dev/null
source "${ENV_FILE}"

echo "Using ffmpeg from: $(command -v ffmpeg)"
echo "Using ffprobe from: $(command -v ffprobe)"

cargo +stable test --workspace
