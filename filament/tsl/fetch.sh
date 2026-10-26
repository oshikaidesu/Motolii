#!/bin/zsh
# The TSL collections in sources.tsv (name, repository, commit, licence, size), each fetched whole at its commit into
# filament/third_party/tsl/<name>, as tvg_on_dawn.sh fetches ThorVG: git init, fetch that one commit, check it out.
# Nothing in them is changed by the fetch. The host seams are applied after, from tsl/*.patch.
#   ./tsl/fetch.sh            all of them        ./tsl/fetch.sh <name>   one
set -e
cd "$(dirname "$0")/.."
mkdir -p third_party/tsl
while IFS=$'\t' read -r name url commit licence size; do
  [ -n "$1" ] && [ "$1" != "$name" ] && continue
  T=third_party/tsl/$name
  [ -d $T/.git ] && continue
  git init -q $T && git -C $T -c core.hooksPath=/dev/null fetch -q --depth 1 $url $commit && git -C $T checkout -q FETCH_HEAD
  echo "$name ($licence, $size)"
done < tsl/sources.tsv
# A host's point where the browser had a mouse, and tilt-shift's mix taken off the cube scene.
# -N: already applied is fine.
for spec in \
  "basementstudio__shader-lab|tsl/slab_target.patch" \
  "ics-creative__250916_threejs_fluids|tsl/ics_target.patch" \
  "ics-creative__250423_vite_threejs_tsl|tsl/ics_tilt.patch"
do
  dir=${spec%%|*}
  p=${spec#*|}
  [ -d third_party/tsl/$dir/.git ] || continue
  patch -p1 -N -r - -d third_party/tsl/$dir < $p >/dev/null || true
done
