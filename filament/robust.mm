// Robustness of the 3D path already shown to exist.
// Closure: Wang, Jüttler, Zheng, Liu 2008 — add a tangent rotation, linear in arc length.
// Cusp: the first non-zero derivative (the curve is not regular there).
// Split: de Casteljau until flatness and radius * turning are under one tolerance.
// Many controls: Filament float sampler, textureLod in materialVertex. No SSBO in the public API.
// Intersection: Patrikalakis, Maekawa — local when r * kappa > 1, global when separation < 2r.
// Resolve = shrink to the largest nonsingular radius. Allow = keep the requested radius.
#include <filament/Camera.h>
#include <filament/ColorGrading.h>
#include <filament/Engine.h>
#include <filament/IndexBuffer.h>
#include <filament/Material.h>
#include <filament/MaterialInstance.h>
#include <filament/RenderableManager.h>
#include <filament/Renderer.h>
#include <filament/RenderTarget.h>
#include <filament/Scene.h>
#include <filament/Skybox.h>
#include <filament/SwapChain.h>
#include <filament/Texture.h>
#include <filament/TextureSampler.h>
#include <filament/ToneMapper.h>
#include <filament/VertexBuffer.h>
#include <filament/View.h>
#include <filament/Viewport.h>

#include <math/mat4.h>
#include <math/vec4.h>

#include <utils/EntityManager.h>

#include <zlib.h>

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <vector>

using namespace filament;
using namespace filament::math;

static const int W = 640;
static const int H = 400;
static const float PI = 3.14159265f;

struct V3 {
    float x, y, z;
};
static V3 operator+(V3 a, V3 b) { return {a.x + b.x, a.y + b.y, a.z + b.z}; }
static V3 operator-(V3 a, V3 b) { return {a.x - b.x, a.y - b.y, a.z - b.z}; }
static V3 operator*(V3 a, float s) { return {a.x * s, a.y * s, a.z * s}; }
static float dot(V3 a, V3 b) { return a.x * b.x + a.y * b.y + a.z * b.z; }
static V3 cross(V3 a, V3 b) {
    return {a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x};
}
static float len(V3 a) { return std::sqrt(dot(a, a)); }
static V3 norm(V3 a) {
    float L = len(a);
    return L > 1e-8f ? a * (1.f / L) : V3{0, 0, 0};
}

struct Cubic {
    V3 p[4];
};

struct Sample {
    V3 p, T, N, B;
    float s = 0;
    bool recovered = false;
    bool frame_fail = false;
};

struct Mesh {
    std::vector<float> v;
    std::vector<uint32_t> idx;
};

struct Hit {
    float kappa_max = 0;
    float separation = 1e9f;
    int local = 0;
    int global = 0;
    float safe_r = 0;
};

static V3 bezier(const Cubic& c, float t) {
    float u = 1.f - t;
    return c.p[0] * (u * u * u) + c.p[1] * (3.f * u * u * t) + c.p[2] * (3.f * u * t * t) + c.p[3] * (t * t * t);
}
static V3 bezier_d(const Cubic& c, float t) {
    float u = 1.f - t;
    return (c.p[1] - c.p[0]) * (3.f * u * u) + (c.p[2] - c.p[1]) * (6.f * u * t) + (c.p[3] - c.p[2]) * (3.f * t * t);
}
static V3 bezier_dd(const Cubic& c, float t) {
    float u = 1.f - t;
    return (c.p[2] - c.p[1] * 2.f + c.p[0]) * (6.f * u) + (c.p[3] - c.p[2] * 2.f + c.p[1]) * (6.f * t);
}

static V3 rot_about(V3 v, V3 T, float ang);

static V3 limit_tangent(const Cubic& c, float t, bool& recovered) {
    V3 d = bezier_d(c, t);
    if (len(d) >= 1e-4f) {
        recovered = false;
        return d;
    }
    recovered = true;
    float tin = std::clamp(t + (t < 0.5f ? 1e-3f : -1e-3f), 0.f, 1.f);
    V3 side = bezier_d(c, tin);
    if (len(side) >= 1e-4f) return side;
    V3 chord = c.p[3] - c.p[0];
    if (len(chord) >= 1e-4f) return chord;
    return {1, 0, 0};
}

static V3 at_open(const std::vector<V3>& p, int i) {
    if (i < 0) return p[0] * 2.f - p[1];
    if (i >= (int)p.size()) return p.back() * 2.f - p[p.size() - 2];
    return p[i];
}
static std::vector<Cubic> catmull(const std::vector<V3>& p, bool closed) {
    std::vector<Cubic> out;
    int n = (int)p.size();
    int segs = closed ? n : n - 1;
    auto at = [&](int i) {
        if (closed) return p[(i % n + n) % n];
        return at_open(p, i);
    };
    for (int i = 0; i < segs; ++i) {
        V3 p0 = at(i), p3 = at(i + 1);
        V3 p1 = p0 + (at(i + 1) - at(i - 1)) * (1.f / 6.f);
        V3 p2 = p3 - (at(i + 2) - at(i)) * (1.f / 6.f);
        out.push_back(Cubic{p0, p1, p2, p3});
    }
    return out;
}

static std::vector<Cubic> closed_loop() {
    std::vector<V3> p = {{-0.6f, -0.45f, 0}, {0.65f, -0.25f, 0.45f}, {0.35f, 0.6f, -0.35f}, {-0.55f, 0.25f, 0.5f}};
    return catmull(p, true);
}
static std::vector<Cubic> planar_rect() {
    auto line = [](V3 a, V3 b) { return Cubic{a, a, b, b}; };
    V3 a{-0.7f, -0.4f, 0}, b{0.7f, -0.4f, 0}, c{0.7f, 0.4f, 0}, d{-0.7f, 0.4f, 0};
    return {line(a, b), line(b, c), line(c, d), line(d, a)};
}
static std::vector<Cubic> cusp_curve() {
    return {Cubic{{{0, 0, 0}, {0, 0, 0}, {0.45f, 0.35f, 0.15f}, {0.95f, 0.05f, 0.05f}}}};
}
static Cubic arc_xy(V3 center, float radius, float a0, float a1) {
    V3 p0{center.x + radius * std::cos(a0), center.y + radius * std::sin(a0), center.z};
    V3 p3{center.x + radius * std::cos(a1), center.y + radius * std::sin(a1), center.z};
    float k = radius * 0.55228475f * (a1 - a0) / (PI * 0.5f);
    V3 t0{-std::sin(a0), std::cos(a0), 0};
    V3 t1{-std::sin(a1), std::cos(a1), 0};
    return Cubic{p0, p0 + t0 * k, p3 - t1 * k, p3};
}
static Cubic line_to(V3 a, V3 b) { return Cubic{a, a, b, b}; }

static std::vector<Cubic> local_bend() {
    float R = 0.05f;
    Cubic arc = arc_xy({0, R, 0}, R, -PI * 0.5f, 0);
    return {line_to({-0.9f, 0, 0}, arc.p[0]), arc, line_to(arc.p[3], {R, R + 0.9f, 0})};
}
static std::vector<Cubic> global_pass() {
    float R = 0.35f;
    Cubic c0 = arc_xy({0.8f - R, -0.55f + R, 0}, R, -PI * 0.5f, 0);
    Cubic c1 = arc_xy({0.8f - R, 0.55f - R, 0}, R, 0, PI * 0.5f);
    Cubic c2 = arc_xy({-0.8f + R, 0.55f - R, 0}, R, PI * 0.5f, PI);
    Cubic c3 = arc_xy({-0.8f + R, -0.45f + R, 0}, R, PI, PI * 1.5f);
    return {line_to({-0.2f, -0.55f, 0}, c0.p[0]), c0, line_to(c0.p[3], c1.p[0]), c1, line_to(c1.p[3], c2.p[0]), c2,
            line_to(c2.p[3], c3.p[0]), c3, line_to(c3.p[3], {0.05f, -0.45f, 0})};
}
static std::vector<Cubic> stress_curve(int spans) {
    std::vector<V3> p;
    for (int i = 0; i <= spans; ++i) {
        float a = (float)i / (float)spans * PI * 2.f * 3.f;
        float r = 0.55f + 0.18f * std::sin(a * 2.f);
        p.push_back({r * std::cos(a), r * std::sin(a) * 0.7f, 0.35f * std::sin(a * 3.f)});
    }
    return catmull(p, false);
}
static std::vector<Cubic> vertical_curve() {
    Cubic s0{{{-0.9f, 0, 0}, {-0.65f, 0, 0}, {-0.4f, 0, 0}, {-0.2f, 0, 0}}};
    Cubic s1{{{-0.2f, 0, 0}, {0.05f, 0, 0.05f}, {0.15f, 0, 0.25f}, {0.15f, 0, 0.55f}}};
    Cubic s2{{{0.15f, 0, 0.55f}, {0.15f, 0, 0.9f}, {0.15f, 0, 1.15f}, {0.15f, 0, 1.45f}}};
    Cubic s3{{{0.15f, 0, 1.45f}, {0.15f, 0, 1.75f}, {0.55f, 0.4f, 1.7f}, {0.9f, 0.45f, 1.25f}}};
    return {s0, s1, s2, s3};
}
static std::vector<V3> edit_points() {
    return {{-0.9f, -0.2f, 0.1f}, {-0.3f, 0.45f, 0.7f}, {0.25f, -0.15f, -0.45f}, {0.85f, 0.35f, 0.35f}, {0.15f, -0.55f, 0.9f}};
}

static void push_sample(std::vector<Sample>& out, const Cubic& c, float t, bool has, const Sample& prev) {
    Sample s;
    bool recovered = false;
    V3 d = limit_tangent(c, t, recovered);
    s.p = bezier(c, t);
    s.recovered = recovered;
    s.T = norm(d);
    if (len(s.T) < 0.5f) {
        s.T = has ? prev.T : V3{1, 0, 0};
        s.frame_fail = true;
    }
    V3 up = std::fabs(s.T.z) < 0.9f ? V3{0, 0, 1} : V3{0, 1, 0};
    V3 n = has ? prev.N - s.T * dot(prev.N, s.T) : cross(up, s.T);
    if (len(n) < 1e-4f) {
        if (!has) {
            n = cross(up, s.T);
            if (len(n) < 1e-5f) n = cross(V3{1, 0, 0}, s.T);
        } else {
            V3 axis = cross(prev.T, s.T);
            float axis_len = len(axis);
            float ang = std::atan2(axis_len, std::clamp(dot(prev.T, s.T), -1.f, 1.f));
            if (axis_len < 1e-5f) {
                n = dot(prev.T, s.T) < 0 ? cross(s.T, prev.B) : prev.N;
            } else {
                n = rot_about(prev.N, norm(axis), ang);
                n = n - s.T * dot(n, s.T);
            }
        }
        if (len(n) < 1e-5f) s.frame_fail = true;
    }
    s.N = norm(n);
    s.B = norm(cross(s.T, s.N));
    if (len(s.N) < 0.5f || len(s.B) < 0.5f) s.frame_fail = true;
    s.s = has ? prev.s + len(s.p - prev.p) : 0;
    out.push_back(s);
}

static std::vector<Sample> sample_uniform(const std::vector<Cubic>& spans, int per, bool closed) {
    std::vector<Sample> out;
    Sample prev{};
    for (int s = 0; s < (int)spans.size(); ++s)
        for (int i = 0; i < per; ++i) push_sample(out, spans[s], (float)i / (float)per, !out.empty(), prev), prev = out.back();
    (void)closed;
    return out;
}

static V3 rot_about(V3 v, V3 T, float ang) {
    float c = std::cos(ang), s = std::sin(ang);
    return v * c + cross(T, v) * s;
}
static float signed_about(V3 T, V3 from, V3 to) {
    return std::atan2(dot(T, cross(from, to)), std::clamp(dot(from, to), -1.f, 1.f));
}
static V3 transport_n(V3 n, V3 t_from, V3 t_to) {
    V3 axis = cross(t_from, t_to);
    float axis_len = len(axis);
    if (axis_len < 1e-8f) {
        if (dot(t_from, t_to) < 0.f) n = cross(t_to, cross(t_from, n));
    } else {
        float ang = std::atan2(axis_len, std::clamp(dot(t_from, t_to), -1.f, 1.f));
        n = rot_about(n, norm(axis), ang);
    }
    return n - t_to * dot(n, t_to);
}

static float closure_angle(const std::vector<Sample>& s) {
    if (s.size() < 2) return 0;
    V3 n = transport_n(s.back().N, s.back().T, s.front().T);
    if (len(n) < 1e-6f) return PI;
    return signed_about(s.front().T, s.front().N, norm(n));
}

struct Shut {
    std::vector<Sample> frames;
    float before = 0;
    float join = 0;
    float last_step = 0;
    float edge = 0;
};

static Shut close_arc(const std::vector<Sample>& in) {
    Shut out;
    if (in.size() < 2) {
        out.frames = in;
        return out;
    }
    float alpha = closure_angle(in);
    float L = in.back().s + len(in.front().p - in.back().p);
    out.before = alpha;
    out.frames = in;
    if (L > 1e-6f) {
        for (Sample& a : out.frames) {
            float ang = -alpha * a.s / L;
            a.N = norm(rot_about(a.N, a.T, ang));
            a.B = norm(cross(a.T, a.N));
        }
    }
    V3 arrived = transport_n(in.back().N, in.back().T, in.front().T);
    V3 end_n = rot_about(norm(arrived), in.front().T, -alpha);
    end_n = end_n - in.front().T * dot(end_n, in.front().T);
    Sample end = in.front();
    end.s = L;
    end.N = norm(end_n);
    end.B = norm(cross(end.T, end.N));
    end.recovered = false;
    end.frame_fail = false;
    out.join = signed_about(in.front().T, in.front().N, end.N);
    V3 stepped = transport_n(out.frames.back().N, out.frames.back().T, end.T);
    out.last_step = len(stepped) < 1e-6f ? 0.f : signed_about(end.T, norm(stepped), end.N);
    out.edge = len((end.p + end.N * 0.075f) - (in.front().p + in.front().N * 0.075f));
    out.frames.push_back(end);
    return out;
}

static float dist_chord(V3 p, V3 a, V3 b) {
    V3 ab = b - a;
    float L2 = dot(ab, ab);
    if (L2 < 1e-12f) return len(p - a);
    float u = std::clamp(dot(p - a, ab) / L2, 0.f, 1.f);
    return len(p - (a + ab * u));
}
static void split_cubic(const Cubic& c, Cubic& L, Cubic& R) {
    V3 p01 = (c.p[0] + c.p[1]) * 0.5f;
    V3 p12 = (c.p[1] + c.p[2]) * 0.5f;
    V3 p23 = (c.p[2] + c.p[3]) * 0.5f;
    V3 p012 = (p01 + p12) * 0.5f;
    V3 p123 = (p12 + p23) * 0.5f;
    V3 mid = (p012 + p123) * 0.5f;
    L = Cubic{c.p[0], p01, p012, mid};
    R = Cubic{mid, p123, p23, c.p[3]};
}
static void adapt_span(const Cubic& c, float radius, float tol, int depth, std::vector<Sample>& out) {
    bool rec0 = false, rec1 = false;
    V3 t0 = norm(limit_tangent(c, 0, rec0));
    V3 t1 = norm(limit_tangent(c, 1, rec1));
    float ang = std::acos(std::clamp(dot(t0, t1), -1.f, 1.f));
    float flat = std::max(dist_chord(c.p[1], c.p[0], c.p[3]), dist_chord(c.p[2], c.p[0], c.p[3]));
    float chord = len(c.p[3] - c.p[0]);
    if (depth >= 10 || chord < 1e-4f || (flat < tol && radius * ang < tol)) {
        Sample prev = out.empty() ? Sample{} : out.back();
        if (out.empty()) push_sample(out, c, 0, false, prev);
        push_sample(out, c, 1, true, out.back());
        return;
    }
    Cubic L, R;
    split_cubic(c, L, R);
    adapt_span(L, radius, tol, depth + 1, out);
    adapt_span(R, radius, tol, depth + 1, out);
}
static std::vector<Sample> sample_adaptive(const std::vector<Cubic>& spans, float radius, float tol) {
    std::vector<Sample> out;
    for (const Cubic& c : spans) adapt_span(c, radius, tol, 0, out);
    return out;
}

static float kappa_at(const std::vector<Sample>& s, int i) {
    int n = (int)s.size();
    if (n < 3) return 0;
    int a = std::max(0, i - 1);
    int b = std::min(n - 1, i + 1);
    float ds = s[b].s - s[a].s;
    if (ds < 1e-6f) return 0;
    float ang = std::acos(std::clamp(dot(s[a].T, s[b].T), -1.f, 1.f));
    return ang / ds;
}
static Hit classify(const std::vector<Sample>& s, float radius, float skip) {
    Hit h;
    int n = (int)s.size();
    for (int i = 0; i < n; ++i) h.kappa_max = std::max(h.kappa_max, kappa_at(s, i));
    for (int i = 0; i < n; ++i) {
        for (int j = i + 1; j < n; ++j) {
            if (s[j].s - s[i].s < skip) continue;
            h.separation = std::min(h.separation, len(s[j].p - s[i].p));
        }
    }
    h.local = (radius * h.kappa_max > 1.f) ? 1 : 0;
    h.global = (h.separation < radius * 2.f) ? 1 : 0;
    float local_lim = h.kappa_max > 1e-4f ? 1.f / h.kappa_max : radius;
    float global_lim = h.separation * 0.5f;
    h.safe_r = std::min(radius, 0.95f * std::min(local_lim, global_lim));
    return h;
}

static void push_vert(Mesh& m, V3 p, float r, float g, float b) {
    m.v.insert(m.v.end(), {p.x, p.y, p.z, r, g, b, 1.f});
}
static Mesh ribbon(const std::vector<Sample>& samples, float half, bool closed) {
    Mesh m;
    int rings = (int)samples.size();
    for (int i = 0; i < rings; ++i) {
        const Sample& s = samples[i];
        float r0 = s.frame_fail ? 1.f : 0.92f, g0 = s.frame_fail ? 1.f : 0.28f, b0 = s.frame_fail ? 1.f : 0.22f;
        float r1 = s.frame_fail ? 1.f : 0.22f, g1 = s.frame_fail ? 1.f : 0.38f, b1 = s.frame_fail ? 1.f : 0.92f;
        push_vert(m, s.p + s.N * -half, r0, g0, b0);
        push_vert(m, s.p + s.N * half, r1, g1, b1);
    }
    int limit = closed ? rings : rings - 1;
    for (int i = 0; i < limit; ++i) {
        int i1 = (i + 1) % rings;
        uint32_t a = (uint32_t)(i * 2), b = a + 1, c = (uint32_t)(i1 * 2), d = c + 1;
        m.idx.insert(m.idx.end(), {a, b, c, b, d, c});
    }
    return m;
}
static Mesh tube_colored(const std::vector<Sample>& samples, float radius, int sides, bool zup) {
    Mesh m;
    int rings = (int)samples.size();
    for (int i = 0; i < rings; ++i) {
        Sample s = samples[i];
        if (zup) {
            V3 up = std::fabs(s.T.z) > 0.98f ? V3{0, 1, 0} : V3{0, 0, 1};
            V3 side = norm(cross(s.T, up));
            if (len(side) < 0.5f) side = {1, 0, 0};
            V3 lift = norm(cross(side, s.T));
            s.N = lift;
            s.B = side;
        }
        for (int k = 0; k < sides; ++k) {
            float ang = (float)k / (float)sides * PI * 2.f;
            V3 p = s.p + (s.N * std::cos(ang) + s.B * std::sin(ang)) * radius;
            push_vert(m, p, 0.55f + 0.4f * std::cos(ang), 0.4f + 0.35f * std::sin(ang), 0.72f);
        }
    }
    for (int i = 0; i < rings - 1; ++i) {
        for (int k = 0; k < sides; ++k) {
            uint32_t a = (uint32_t)(i * sides + k);
            uint32_t b = (uint32_t)(i * sides + (k + 1) % sides);
            uint32_t c = (uint32_t)((i + 1) * sides + k);
            uint32_t d = (uint32_t)((i + 1) * sides + (k + 1) % sides);
            m.idx.insert(m.idx.end(), {a, b, c, b, d, c});
        }
    }
    return m;
}

static int count_fail(const std::vector<Sample>& s) {
    int n = 0;
    for (const Sample& a : s)
        if (a.frame_fail) ++n;
    return n;
}
static int count_recovered(const std::vector<Sample>& s) {
    int n = 0;
    for (const Sample& a : s)
        if (a.recovered) ++n;
    return n;
}
static float edge_min(const std::vector<Sample>& s, float half) {
    float m = 1e9f;
    for (const Sample& a : s) m = std::min(m, len((a.p + a.N * half) - a.p));
    return m;
}
static int bend_rings(const std::vector<Sample>& s, float y0, float y1, float x1) {
    int n = 0;
    for (const Sample& a : s)
        if (a.p.x > -0.02f && a.p.x < x1 && a.p.y > y0 && a.p.y < y1) ++n;
    return n;
}


#include "robust_2.h"
#include "robust_3.h"
