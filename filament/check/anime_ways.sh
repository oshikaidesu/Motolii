#!/bin/zsh
# anime.js's easings as a key's stretch (third_party/anime, MIT, run in QuickJS by script.cpp's anime_ease): ways
# defined with anime('...', u) are read back at moments (Elastic, a spring, linear() by points, irregular() twice: the
# same words, the same curve), and one that is not an ease of anime.js is refused. The replies are as check/anime_ways.expect.
cd "$(dirname "$0")/.."
rm -f out/anime_ways.motolii
./edit.sh out/anime_ways.motolii < check/anime_ways.txt 2>/dev/null | grep -E '^  |NOT done' | sed 's/, [0-9.]* ms$//' > out/anime_ways.txt
diff out/anime_ways.txt check/anime_ways.expect > /dev/null && echo "anime.js's eases go as expected: yes ($(grep -c '^  ' out/anime_ways.txt) values read)" || echo "anime.js's eases go as expected: NO"
