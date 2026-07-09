#!/usr/bin/env bash
# R9スモーク: 合成テスト動画 + project.json を作り verify + GUI まで一気通貫
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="${1:-${TMPDIR:-/tmp}/opencuts-r9-smoke}"
mkdir -p "${DIR}"

INPUT="${DIR}/input.mp4"
OUTPUT="${DIR}/output.mp4"
PROJECT="${DIR}/project.json"

if ! command -v ffmpeg >/dev/null; then
  echo "error: ffmpeg not on PATH" >&2
  exit 1
fi

echo "==> generating 3s testsrc @ 640x360 30fps -> ${INPUT}"
ffmpeg -y -hide_banner -loglevel error \
  -f lavfi -i "testsrc=duration=3:size=640x360:rate=30" \
  -pix_fmt yuv420p -color_range tv -colorspace bt709 \
  "${INPUT}"

cat >"${PROJECT}" <<EOF
{
  "version": 1,
  "input": "input.mp4",
  "output": "output.mp4",
  "start_frame": 0,
  "frame_count": 90,
  "qp0": true,
  "param_drivers": [],
  "overlay": {
    "center": [0.0, 0.0],
    "size": [0.3, 0.3],
    "color": [1.0, 0.2, 0.0, 0.6]
  }
}
EOF

echo "==> project: ${PROJECT}"
export R9_TOLERANCE=24
exec "${ROOT_DIR}/scripts/r9-verify.sh" "${PROJECT}"
