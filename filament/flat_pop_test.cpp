// Desk check: how much the floor light of ONE tumbling flat-faced solid jumps from one small turn to the next.
// Same placement and the same trace_cheap that glass_scene uses (PIECES cut per face, one ray per piece).
// The solid is turned about one tilted axis in small steps; each step's floor picture is drawn from the triangles
// and compared with the step before. A smooth picture gives frame differences that all look alike; a pop shows up
// as a step far above the typical step. Prints the summary and writes out/flat-pop-curve.csv (one line per step).
// Build and run: clang++ -std=c++20 -O2 flat_pop_test.cpp -o out/flat_pop_test && ./out/flat_pop_test [pieces] [stepDeg]
#include "beam_glass.h"

#include <algorithm>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <chrono>
#include <string>
#include <vector>

using namespace beam;

// Rodrigues: rotation about unit axis n by angle a.
static void rot(V3 n, double a, V3 R[3]) {
    const double c = std::cos(a), s = std::sin(a), t = 1 - c;
    const double x = n.x, y = n.y, z = n.z;
    const V3 cols[3] = {
        {t * x * x + c, t * x * y + s * z, t * x * z - s * y},
        {t * x * y - s * z, t * y * y + c, t * y * z + s * x},
        {t * x * z + s * y, t * y * z - s * x, t * z * z + c},
    };
    for (int i = 0; i < 3; ++i) R[i] = cols[i];
}

// Floor picture: sums the flat-coloured triangles into a W x W grid (one sample at each pixel centre, plus the
// corners). Returns the three channels interleaved.
static std::vector<double> draw(const std::vector<float>& tc, int W, double cx, double cz, double R) {
    std::vector<double> img((size_t)W * W * 3, 0.0);
    const double px = 2.0 * R / W;
    for (size_t i = 0; i + 3 * TRI <= tc.size(); i += 3 * TRI) {
        const double r = tc[i + 2], g = tc[i + 3], b = tc[i + 4];
        double x[3], z[3];
        for (int k = 0; k < 3; ++k) {
            x[k] = tc[i + (size_t)k * TRI];
            z[k] = tc[i + (size_t)k * TRI + 1];
        }
        const double area2 = (x[1] - x[0]) * (z[2] - z[0]) - (x[2] - x[0]) * (z[1] - z[0]);
        if (std::fabs(area2) < 1e-18) continue;
        const double minx = std::min({x[0], x[1], x[2]}), maxx = std::max({x[0], x[1], x[2]});
        const double minz = std::min({z[0], z[1], z[2]}), maxz = std::max({z[0], z[1], z[2]});
        const int ix0 = std::max(0, (int)std::floor((minx - (cx - R)) / px)), ix1 = std::min(W - 1, (int)std::floor((maxx - (cx - R)) / px));
        const int iz0 = std::max(0, (int)std::floor((minz - (cz - R)) / px)), iz1 = std::min(W - 1, (int)std::floor((maxz - (cz - R)) / px));
        for (int iz = iz0; iz <= iz1; ++iz)
            for (int ix = ix0; ix <= ix1; ++ix) {
                const double px_ = cx - R + (ix + 0.5) * px, pz = cz - R + (iz + 0.5) * px;
                const double w1 = ((px_ - x[0]) * (z[2] - z[0]) - (x[2] - x[0]) * (pz - z[0])) / area2;
                const double w2 = ((x[1] - x[0]) * (pz - z[0]) - (px_ - x[0]) * (z[1] - z[0])) / area2;
                if (w1 < 0 || w2 < 0 || w1 + w2 > 1) continue;
                double* o = &img[((size_t)iz * W + ix) * 3];
                o[0] += r;
                o[1] += g;
                o[2] += b;
            }
    }
    return img;
}

int main(int argc, char** argv) {
    const int pieces = argc > 1 ? std::atoi(argv[1]) : 4;
    const double stepDeg = argc > 2 ? std::atof(argv[2]) : 0.5;
    const int steps = (int)std::lround(360.0 / stepDeg);
    const int W = 360;
    const double Rv = 2.2, cx = 0.6, cz = 0.0;  // view on the floor around the solid

    Params par;
    par.sun = V3{0.62, -0.62, 0.28};
    par.glass.exaggerate = 10;
    const double detail = 2.0 * Rv / W;

    const Solid base = chamfered(box(0.5, 0.5, 0.5), 0.03);
    V3 axis = normalize(V3{0.3, 1.0, 0.2});
    if (const char* ax = std::getenv("AXIS")) std::sscanf(ax, "%lf %lf %lf", &axis.x, &axis.y, &axis.z), axis = normalize(axis);

    std::vector<std::vector<double>> frames, exact;
    frames.reserve((size_t)steps + 1);
    std::vector<float> tc;
    CheapStats cs;
    const bool withExact = std::getenv("EXACT") != nullptr;  // also the exact trace, to see what the cheap one adds
    for (int k = 0; k <= steps; ++k) {
        V3 R[3];
        rot(axis, k * stepDeg * 3.14159265358979 / 180.0, R);
        Scene sc;
        sc.solids = {placed(base, R, 1.0, V3{0, 0.5, 0})};
        sc.bound();
        trace_cheap(sc, par, std::vector<double>(sc.solids.size(), detail), pieces, tc, &cs);
        frames.push_back(draw(tc, W, cx, cz, Rv));
        if (withExact) {
            trace_fast(sc, par, std::vector<double>(sc.solids.size(), detail), tc);
            exact.push_back(draw(tc, W, cx, cz, Rv));
        }
    }

    // difference between each step and the one before (mean per pixel over channels, and the largest single pixel)
    std::vector<double> mean(steps), peak(steps);
    for (int k = 1; k <= steps; ++k) {
        double sum = 0, mx = 0;
        const auto& a = frames[(size_t)k];
        const auto& b = frames[(size_t)k - 1];
        for (size_t i = 0; i < a.size(); ++i) {
            const double d = std::fabs(a[i] - b[i]);
            sum += d;
            mx = std::max(mx, d);
        }
        mean[(size_t)k - 1] = sum / (double)(W * W * 3);
        peak[(size_t)k - 1] = mx;
    }
    if (withExact) {
        // what the cheap picture changes by from one step to the next, minus what the exact one changes by
        std::vector<double> gap(steps);
        for (int k = 1; k <= steps; ++k) {
            double sum = 0;
            for (size_t i = 0; i < frames[(size_t)k].size(); ++i) {
                const double dc = frames[(size_t)k][i] - frames[(size_t)k - 1][i];
                const double de = exact[(size_t)k][i] - exact[(size_t)k - 1][i];
                sum += std::fabs(dc - de);
            }
            gap[(size_t)k - 1] = sum / (double)(W * W * 3);
        }
        std::vector<double> gs = gap;
        std::sort(gs.begin(), gs.end());
        const double gm = gs[gs.size() / 2];
        std::printf("cheap minus exact, frame change: median %.6f  max %.6f (max / median %.2f)\n", gm, gs.back(), gs.back() / std::max(gm, 1e-12));
        std::printf("steps where the cheap change is off by more than 3x its median:");
        int n = 0;
        for (int k = 0; k < steps; ++k)
            if (gap[(size_t)k] > 3.0 * gm) {
                if (n < 16) std::printf(" %d(%.1f deg, %.2fx)", k + 1, (k + 1) * stepDeg, gap[(size_t)k] / gm);
                ++n;
            }
        std::printf("  total %d\n", n);
        for (int k = 0; k < steps; ++k) mean[(size_t)k] = gap[(size_t)k];  // the CSV and the spike list below use this
    }
    std::vector<double> sorted = mean;
    std::sort(sorted.begin(), sorted.end());
    const double med = sorted[sorted.size() / 2];
    const double p95 = sorted[(size_t)(sorted.size() * 0.95)];
    const double top = sorted.back();

    // how many steps stand far above the typical step
    std::vector<int> spikes;
    for (int k = 0; k < steps; ++k)
        if (mean[(size_t)k] > 3.0 * med + 1e-12) spikes.push_back(k + 1);

    double light = 0, lightExact = 0;
    for (const auto& f : frames) for (double v : f) light += v;
    for (const auto& f : exact) for (double v : f) lightExact += v;
    if (withExact) std::printf("light drawn, cheap / exact: %.3f\n", light / std::max(lightExact, 1e-30));
    std::printf("pieces %d, step %.2f deg, %d steps\n", pieces, stepDeg, steps);
    std::printf("mean frame difference: median %.6f  p95 %.6f  max %.6f  (max / median %.2f)\n", med, p95, top, top / std::max(med, 1e-12));
    std::printf("steps above 3x median: %zu", spikes.size());
    for (size_t i = 0; i < spikes.size() && i < 12; ++i) std::printf(" %d(%.2fx)", spikes[i], mean[(size_t)spikes[i] - 1] / med);
    std::printf("\n");

    // DUMP=a,b,c: write those steps' floor pictures as raw float32 (W x W x 3) for looking at
    if (const char* dump = std::getenv("DUMP")) {
        std::string list = dump;
        size_t at = 0;
        while (at < list.size()) {
            const int k = std::atoi(list.c_str() + at);
            if (k >= 0 && k <= steps) {
                std::vector<float> f32(frames[(size_t)k].begin(), frames[(size_t)k].end());
                std::string name = "out/flat-pop-frame-p" + std::to_string(pieces) + (std::getenv("TAG") ? std::string("-") + std::getenv("TAG") : std::string()) + "-" + std::to_string(k) + ".f32";
                if (FILE* f = std::fopen(name.c_str(), "wb")) {
                    std::fwrite(f32.data(), sizeof(float), f32.size(), f);
                    std::fclose(f);
                }
                if (withExact) {
                    std::vector<float> e32(exact[(size_t)k].begin(), exact[(size_t)k].end());
                    std::string ename = "out/flat-pop-exact-" + std::to_string(k) + ".f32";
                    if (FILE* g = std::fopen(ename.c_str(), "wb")) {
                        std::fwrite(e32.data(), sizeof(float), e32.size(), g);
                        std::fclose(g);
                    }
                }
                if (withExact && k >= 1) {  // signed (cheap change - exact change) for this step, same layout
                    std::vector<float> gapImg(f32.size());
                    for (size_t i = 0; i < f32.size(); ++i)
                        gapImg[i] = (float)((frames[(size_t)k][i] - frames[(size_t)k - 1][i]) - (exact[(size_t)k][i] - exact[(size_t)k - 1][i]));
                    std::string gname = "out/flat-pop-gap-p" + std::to_string(pieces) + "-" + std::to_string(k) + ".f32";
                    if (FILE* g = std::fopen(gname.c_str(), "wb")) {
                        std::fwrite(gapImg.data(), sizeof(float), gapImg.size(), g);
                        std::fclose(g);
                    }
                }
            }
            const size_t comma = list.find(',', at);
            if (comma == std::string::npos) break;
            at = comma + 1;
        }
    }

    std::string file = "out/flat-pop-curve-p" + std::to_string(pieces) + ".csv";
    if (FILE* f = std::fopen(file.c_str(), "w")) {
        std::fprintf(f, "step,deg,mean_diff,peak_diff\n");
        for (int k = 0; k < steps; ++k) std::fprintf(f, "%d,%.3f,%.8f,%.8f\n", k + 1, (k + 1) * stepDeg, mean[(size_t)k], peak[(size_t)k]);
        std::fclose(f);
        std::printf("wrote %s\n", file.c_str());
    }
    return 0;
}
