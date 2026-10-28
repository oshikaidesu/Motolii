#!/bin/zsh
# Is interp.h the same as the past repository's own where it still is (Steps, and the bezier; Bounce, Elastic, Cyclic,
# Random and Elastic Steps are AbsentMotion's since 10/29, absent_motion.cpp)? The formulas of its track.rs (lines 264 to
# 432: nothing but the formulas) are compiled as they stand, in out/ (the past repository is only read), run over 41 points beside the C++
# copy, and the two sets of numbers compared; the bezier is set against the ruler rows the past repository holds as exact.
#   check/interp_twin.sh [the past repository's track.rs]
cd "$(dirname "$0")/.."
mkdir -p out/twin
src=${1:-out/twin/track.rs}   # (by default the past repository's own, github.com/oshikaidesu/rust_ae at main 8d2a0ebd, fetched once)
[ -f $src ] || curl -fsSL -o $src https://raw.githubusercontent.com/oshikaidesu/rust_ae/8d2a0ebdf7261bc9b77262766560f26fc1b3c464/motolii/crates/motolii-doc/src/eval/track.rs || { echo "the past repository's track.rs could not be fetched: not checked"; exit 0 }
{ sed -n 264,432p $src; cat <<'RS'
fn main() {
    for i in 0..=40 {
        let u = i as f64 / 40.0;
        println!("{:.12} {:.12}", steps_ease(0.178, 0.0, u), steps_ease(0.25, 0.1, u));
    }
}
RS
} > out/twin/twin.rs
rustc -O -A warnings out/twin/twin.rs -o out/twin/twin_rs || exit 1
cat > out/twin/twin.cpp <<'CPP'
#include <cstring>
#include <strings.h>
#include "../../interp.h"
#include <cstdio>
int main() {
    for (int i = 0; i <= 40; ++i) {
        const double u = i / 40.0;
        std::printf("%.12f %.12f\n", interp_steps(0.178, 0.0, u), interp_steps(0.25, 0.1, u));
    }
    const double rows[3][7] = {{1.0 / 3, 0, 2.0 / 3, 1.0 / 3, 0.0625, 0.25, 0.5625}, {1.0 / 3, 1, 2.0 / 3, 1, 0.578125, 0.875, 0.984375}, {1.0 / 3, (1.70158 + 3) / 3, 2.0 / 3, 1, 0.8174096875, 1.0876975, 1.0641365625}};
    double worst = 0;
    for (const auto& r : rows) for (int k = 0; k < 3; ++k) worst = std::fmax(worst, std::fabs(interp_bezier(r[0], r[1], r[2], r[3], 0.25 * (k + 1)) - r[4 + k]));
    std::fprintf(stderr, "Bezier against the ruler's three exact rows: worst off %.1e: %s\n", worst, worst < 1e-9 ? "the same" : "NOT the same");
}
CPP
c++ -std=c++20 -O2 out/twin/twin.cpp -o out/twin/twin_cpp || exit 1
out/twin/twin_rs > out/twin/rs.txt; out/twin/twin_cpp > out/twin/cpp.txt
python3 -c "
import numpy as np
a, b = np.loadtxt('out/twin/rs.txt'), np.loadtxt('out/twin/cpp.txt')
worst = np.abs(a - b).max()
print('Steps (two settings, 41 points): worst difference from the past repository %.1e: %s' % (worst, 'the same' if worst < 1e-9 else 'NOT the same'))"
