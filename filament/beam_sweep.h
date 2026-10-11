// A profile pushed along a path in space and turned as it goes: a twisted bar, a ring, a knot of glass.
// The idea the renderer works by (sweep_glass.mat, sweep_light.mat): at any place along the path the bar is taken
// to be a STRAIGHT bar with that place's profile, turned as it is turned there. A ray inside is followed through
// that straight bar; that the path bends and the profile turns on while the ray is inside is left out. Plausible,
// not exact, and good while the path bends gently compared with the profile's width.
// Built once per shape. Per frame the CPU says where the whole thing stands (one pose), nothing more.
#pragma once
#include "beam_glass.h"

#include <cmath>
#include <cstdint>
#include <functional>
#include <vector>

namespace beam {

struct Sweep {
    int sides = 3;
    float profile[8][4] = {};    // per side of the profile: outward normal (x, y) in the profile's own plane, distance from the middle, 0
    std::vector<float> v;        // per corner 13 numbers: the path's point there (3), the profile's x axis there (3), its y axis (3),
                                 // the corner in the profile's plane (2), its side's outward normal there (2)
    std::vector<uint32_t> idx;   // three per triangle
    double reach = 0;            // nothing of it is further than this from its own origin
    double radius = 0;           // the profile's corners are this far from the path
};

// A closed path (point(u), u in 0..1, coming back to where it started) with a regular `sides`-sided profile whose
// corners are `radius` from it. The profile is carried along without turning of its own (so it does not spin where
// the path merely bends) and then turned evenly by `turns` whole turns in all, plus whatever makes it meet itself
// where the path closes. along: places along the path; across: parts per side.
inline Sweep sweep_closed(const std::function<V3(double)>& point, int sides, double radius, double turns, int along = 240, int across = 3) {
    Sweep sw;
    sw.sides = std::min(8, std::max(3, sides));
    sw.radius = radius;
    const double pi = 3.14159265358979323846;
    const int K = sw.sides, M = along;
    std::vector<V3> P((size_t)M), T((size_t)M), U((size_t)M);
    for (int i = 0; i < M; ++i) P[(size_t)i] = point((double)i / M);
    for (int i = 0; i < M; ++i) T[(size_t)i] = normalize(P[(size_t)((i + 1) % M)] - P[(size_t)((i + M - 1) % M)]);
    // carried along without turning: each axis is the one before, laid back into the plane across the path
    const V3 up = std::fabs(T[0].y) < 0.9 ? V3{0, 1, 0} : V3{1, 0, 0};
    U[0] = normalize(cross(up, T[0]));
    for (int i = 1; i < M; ++i) U[(size_t)i] = normalize(U[(size_t)i - 1] - T[(size_t)i] * dot(U[(size_t)i - 1], T[(size_t)i]));
    // once round, the axis comes back turned by some angle: take that out evenly, so that the profile meets itself
    const V3 back = normalize(U[(size_t)M - 1] - T[0] * dot(U[(size_t)M - 1], T[0]));
    const double off = std::atan2(dot(cross(U[0], back), T[0]), dot(U[0], back));
    const double total = -off + turns * 2.0 * pi;
    for (int k = 0; k < K; ++k) {
        const double mid = 2.0 * pi * (k + 0.5) / K;
        sw.profile[k][0] = (float)std::cos(mid);
        sw.profile[k][1] = (float)std::sin(mid);
        sw.profile[k][2] = (float)(radius * std::cos(pi / K));
        sw.profile[k][3] = 0.f;
    }
    const int A = std::max(1, across);
    for (int i = 0; i < M; ++i) {
        const double ph = total * i / M;
        const V3 u0 = U[(size_t)i], v0 = cross(T[(size_t)i], u0);
        const V3 u = u0 * std::cos(ph) + v0 * std::sin(ph), w = v0 * std::cos(ph) - u0 * std::sin(ph);
        sw.reach = std::max(sw.reach, length(P[(size_t)i]) + radius);
        for (int k = 0; k < K; ++k) {
            const double a0 = 2.0 * pi * k / K, a1 = 2.0 * pi * (k + 1) / K;
            for (int a = 0; a <= A; ++a) {
                const double f = (double)a / A;
                const double x = radius * (std::cos(a0) * (1 - f) + std::cos(a1) * f), y = radius * (std::sin(a0) * (1 - f) + std::sin(a1) * f);
                sw.v.insert(sw.v.end(), {(float)P[(size_t)i].x, (float)P[(size_t)i].y, (float)P[(size_t)i].z, (float)u.x, (float)u.y, (float)u.z, (float)w.x, (float)w.y,
                                         (float)w.z, (float)x, (float)y, sw.profile[k][0], sw.profile[k][1]});
            }
        }
    }
    const uint32_t per = (uint32_t)(K * (A + 1));
    for (int i = 0; i < M; ++i) {
        const uint32_t r0 = (uint32_t)i * per, r1 = (uint32_t)((i + 1) % M) * per;
        for (int k = 0; k < K; ++k)
            for (int a = 0; a < A; ++a) {
                const uint32_t q = (uint32_t)(k * (A + 1) + a);
                sw.idx.insert(sw.idx.end(), {r0 + q, r0 + q + 1, r1 + q + 1, r0 + q, r1 + q + 1, r1 + q});
            }
    }
    return sw;
}

// A knot with three lobes (a trefoil), about `size` across.
inline V3 knot_point(double u, double size) {
    const double t = 2.0 * 3.14159265358979323846 * u, s = size / 6.0;
    return V3{(std::sin(t) + 2.0 * std::sin(2.0 * t)) * s, -std::sin(3.0 * t) * s * 1.1, (std::cos(t) - 2.0 * std::cos(2.0 * t)) * s};
}
// A ring that rises and falls as it goes round (so that it is a path in space, not in a plane), about `size` across.
inline V3 wave_ring_point(double u, double size, double lift, int waves) {
    const double t = 2.0 * 3.14159265358979323846 * u, r = 0.5 * size;
    return V3{std::cos(t) * r, std::sin(waves * t) * lift, std::sin(t) * r};
}

}  // namespace beam
