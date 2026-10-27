#!/bin/zsh
# bladeSk's PianoRollSvg.js (MIT as its package.json says; its repository has no licence file, so it is fetched, not
# bundled), at the commit we use, into third_party/midi_roll: the MIDI view draws its piano roll with it (midi_roll.cpp).
#   ./midi_roll.sh
set -e
cd "$(dirname "$0")"
F=third_party/midi_roll/PianoRollSvg.js
[ -f $F ] && exit 0
curl -fsSL -o $F https://raw.githubusercontent.com/bladeSk/midi-to-piano-roll/205854ee374d1410eb4bc8a21066c222b7996859/PianoRollSvg.js
echo "4beb99a00a9a72a3e2728ca9b65daf7710f28656d99c9490d59e3d50b0adc564  $F" | shasum -a 256 -c - > /dev/null
