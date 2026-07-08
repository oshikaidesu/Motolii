#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLS_DIR="${ROOT_DIR}/.tools"
FFMPEG_DIR="${TOOLS_DIR}/ffmpeg"
BIN_DIR="${FFMPEG_DIR}/bin"
TMP_DIR="${TOOLS_DIR}/tmp"

mkdir -p "${BIN_DIR}" "${TMP_DIR}"

OS="$(uname -s)"
ARCH="$(uname -m)"

download_ffmpeg_macos() {
  local name="$1"
  local url="$2"
  local zip_path="${TMP_DIR}/${name}.zip"
  local unpack_dir="${TMP_DIR}/${name}-unpack"

  curl -fsSL "${url}" -o "${zip_path}"
  rm -rf "${unpack_dir}"
  mkdir -p "${unpack_dir}"
  unzip -qo "${zip_path}" -d "${unpack_dir}"
  install -m 755 "${unpack_dir}/${name}" "${BIN_DIR}/${name}"
}

case "${OS}-${ARCH}" in
  Darwin-arm64|Darwin-x86_64)
    download_ffmpeg_macos "ffmpeg" "https://evermeet.cx/ffmpeg/getrelease/zip"
    if command -v ffprobe >/dev/null 2>&1; then
      install -m 755 "$(command -v ffprobe)" "${BIN_DIR}/ffprobe"
    else
      echo "ffprobe is not available on this machine." >&2
      echo "Install it once (e.g. brew install ffmpeg), then rerun this script." >&2
      exit 1
    fi
    ;;
  *)
    echo "Unsupported platform for auto ffmpeg bundle: ${OS}-${ARCH}" >&2
    echo "Please place ffmpeg/ffprobe binaries into ${BIN_DIR}" >&2
    exit 1
    ;;
esac

cat > "${TOOLS_DIR}/env.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="${ROOT_DIR}/.tools/ffmpeg/bin:${PATH}"
EOF
chmod +x "${TOOLS_DIR}/env.sh"

rustup toolchain install stable >/dev/null
cargo +stable fetch --locked

echo "Local dependencies are ready."
echo "Use: source ./.tools/env.sh"
echo "ffmpeg: ${BIN_DIR}/ffmpeg"
