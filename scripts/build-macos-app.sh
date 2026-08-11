#!/usr/bin/env bash
# macOS desktop appビルダー: JS依存取得 → CocoaPods → arm64 Release xcodebuild を1本に閉じる。
# 対象は唯一のRN製品sourceである ui/motolii-rn/。旧shellへ迂回しない。
# 対象: macOS arm64 Release の1経路のみ。Debug、x86_64、署名付きbuild、配布は非対象。
# 使い方: scripts/build-macos-app.sh [--help]   (どこから呼んでもよい)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RN_DIR="$ROOT/ui/motolii-rn"
MACOS_DIR="$RN_DIR/macos"
WORKSPACE="$MACOS_DIR/MotoliiRn.xcworkspace"
SCHEME="MotoliiRn-macOS"
DESTINATION="generic/platform=macOS,arch=arm64"
# CocoaPods版は固定する。別版で通すとPods生成物が変わり、受入時のbuildを再現できない。
POD_VERSION="1.15.2"

usage() {
  cat <<EOF
usage:
  scripts/build-macos-app.sh          macOS arm64 Release appをビルドする
  scripts/build-macos-app.sh --help   この説明を出す

前提コマンド:
  corepack  xcodebuild  cargo  pod (${POD_VERSION}固定)

出力:
  MotoliiRn.app (Xcodeの既定DerivedData配下)
EOF
}

fail() {
  echo "build-macos-app: $1" >&2
  exit 1
}

step() {
  echo
  echo "build-macos-app: $1"
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "必要なコマンドが見つからない: $1"
}

require_path() {
  [ -e "$1" ] || fail "想定のpathが存在しない: $1"
}

if [ $# -gt 0 ]; then
  case "$1" in
    --help | -h)
      usage
      exit 0
      ;;
    *)
      echo "build-macos-app: 未知の引数: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
fi

step "0/3 前提を確認する"
require_command corepack
require_command pod
require_command xcodebuild
# Rust staticlibはXcodeのbuild phaseが自分で cargo build -p motolii-ui --release --offline を呼ぶ。
# ここで別途叩かないが、cargo不在ならbuild phase側で落ちるので入口で弾く。
require_command cargo
require_path "$RN_DIR"
require_path "$MACOS_DIR"
require_path "$WORKSPACE"
echo "  root=$ROOT"
echo "  workspace=$WORKSPACE scheme=$SCHEME"

step "1/3 JS依存を取得する (corepack yarn install --immutable)"
(cd "$RN_DIR" && corepack yarn install --immutable) \
  || fail "yarn install が失敗した。$RN_DIR の yarn.lock とnetwork到達性を確認すること"

step "2/3 CocoaPods依存を配置する (pod _${POD_VERSION}_ install --deployment)"
# 版が無いときに素の pod install へ落ちない。黙って別版で通すと再現性が壊れるため明示エラーで止める。
if ! pod "_${POD_VERSION}_" --version >/dev/null 2>&1; then
  echo "build-macos-app: CocoaPods ${POD_VERSION} が見つからない。" >&2
  echo "  受入buildは ${POD_VERSION} 固定で再現している。素の pod install へ落とすとPods生成物が" >&2
  echo "  変わり再現性が壊れるため、ここで止める。" >&2
  echo "  導入してから再実行すること: gem install cocoapods -v ${POD_VERSION}" >&2
  exit 1
fi
(cd "$MACOS_DIR" && pod "_${POD_VERSION}_" install --deployment) \
  || fail "pod install が失敗した。$MACOS_DIR/Podfile.lock とstep 1のnode_modulesを確認すること"

step "3/3 arm64 Release appをビルドする (xcodebuild)"
xcodebuild \
  -workspace "$WORKSPACE" \
  -scheme "$SCHEME" \
  -configuration Release \
  -destination "$DESTINATION" \
  ARCHS=arm64 \
  ONLY_ACTIVE_ARCH=NO \
  CODE_SIGNING_ALLOWED=NO \
  build \
  || fail "xcodebuild が失敗した。上のlogでfailした target／build phase を特定すること"

# 出力先はXcodeの既定DerivedData配下なので、build settingsから引き当てて出す。
products_dir="$(
  xcodebuild \
    -workspace "$WORKSPACE" \
    -scheme "$SCHEME" \
    -configuration Release \
    -destination "$DESTINATION" \
    -showBuildSettings 2>/dev/null \
    | awk -F' = ' '/ BUILT_PRODUCTS_DIR = /{print $2; exit}'
)" || products_dir=""

echo
echo "build-macos-app: BUILD SUCCEEDED"
if [ -n "$products_dir" ] && [ -d "$products_dir/MotoliiRn.app" ]; then
  echo "  app: $products_dir/MotoliiRn.app"
else
  echo "  app: MotoliiRn.app (Xcodeの既定DerivedData配下)"
fi
