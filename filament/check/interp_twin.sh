#!/bin/zsh
# Is interp.h's bezier the same as the past repository's? Set against the ruler rows it holds as exact. Steps, Bounce,
# Elastic, Cyclic, Random and Elastic Steps are AbsentMotion's own (absent_motion.cpp, its easing.cpp), not reckoned here.
#   check/interp_twin.sh
cd "$(dirname "$0")/.."
mkdir -p out/twin
cat > out/twin/twin.cpp <<'CPP'
#include <cstring>
#include <strings.h>
#include "../../interp.h"
#include <cstdio>
int main() {
    const double rows[3][7] = {{1.0 / 3, 0, 2.0 / 3, 1.0 / 3, 0.0625, 0.25, 0.5625}, {1.0 / 3, 1, 2.0 / 3, 1, 0.578125, 0.875, 0.984375}, {1.0 / 3, (1.70158 + 3) / 3, 2.0 / 3, 1, 0.8174096875, 1.0876975, 1.0641365625}};
    double worst = 0;
    for (const auto& r : rows) for (int k = 0; k < 3; ++k) worst = std::fmax(worst, std::fabs(interp_bezier(r[0], r[1], r[2], r[3], 0.25 * (k + 1)) - r[4 + k]));
    std::fprintf(stderr, "Bezier against the ruler's three exact rows: worst off %.1e: %s\n", worst, worst < 1e-9 ? "the same" : "NOT the same");
}
CPP
c++ -std=c++20 -O2 out/twin/twin.cpp -o out/twin/twin_cpp || exit 1
out/twin/twin_cpp 2>&1 | tee out/twin/says.txt
grep -q 'NOT' out/twin/says.txt && exit 1
exit 0
