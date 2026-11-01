#!/bin/sh
# Hydra as a part apart: hydra-synth (AGPL-3.0) from npm into the maker's own folder, never into this repository or what is
# handed out. Motolii reads it only as text in, text out (hydra_isf.cpp). Delete the folder and Motolii is as it was.
set -e
at="${MOTOLII_OWN:-$HOME/Library/Application Support/Motolii}/extensions/hydra"
mkdir -p "$at"
curl -fsSL https://registry.npmjs.org/hydra-synth/-/hydra-synth-1.4.0.tgz -o "$at/hydra-synth.tgz"
[ "$(shasum "$at/hydra-synth.tgz" | cut -d' ' -f1)" = 48156763d04b338157255fba63b6e79613b8fa54 ]
tar -xzf "$at/hydra-synth.tgz" -C "$at" --strip-components=1
echo "Hydra is in $at"
