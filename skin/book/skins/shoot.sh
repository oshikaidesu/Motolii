#!/bin/zsh
# A skin's picture: the whole window wearing it, written as preview.png in the skin's folder, to look at after each change.
#   skins/shoot.sh ~/.motolii/skins/mine      (several folders at once; none: every skin in skins/)
# With no folder named, the window's own skin is shot too, as reports/skins/own.png; a named folder alone is written in.
cd "$(dirname "$0")/.."
FLUTTER="${FLUTTER:-/Users/motolii_maker/rust_ae/Motolii/.tools/flutter/bin/flutter}"
dirs=("$@"); own=0; (( $#dirs )) || { dirs=(skins/*/); own=1; }
abs=(); for d in $dirs; do abs+=("$(cd "$d" && pwd)"); done
"$FLUTTER" drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/skin_test.dart --dart-define=SKINS="${(j:,:)abs}" --dart-define=OWN=$own 2>&1 | grep -E "shot:|All tests passed|Some tests failed|Error|error:" 
