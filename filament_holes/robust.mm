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

static bool write_png(const char* path, int w, int h, const uint8_t* rgba) {
    std::vector<uint8_t> raw((size_t)h * (w * 4 + 1));
    for (int y = 0; y < h; ++y) {
        raw[(size_t)y * (w * 4 + 1)] = 0;
        std::memcpy(raw.data() + (size_t)y * (w * 4 + 1) + 1, rgba + (size_t)y * w * 4, (size_t)w * 4);
    }
    uLongf bound = compressBound((uLong)raw.size());
    std::vector<uint8_t> z(bound);
    if (compress(z.data(), &bound, raw.data(), (uLong)raw.size()) != Z_OK) return false;
    FILE* f = std::fopen(path, "wb");
    if (!f) return false;
    auto chunk = [&](const char* tag, const uint8_t* data, uint32_t n) {
        uint32_t be = __builtin_bswap32(n);
        std::fwrite(&be, 4, 1, f);
        std::fwrite(tag, 1, 4, f);
        if (n) std::fwrite(data, 1, n, f);
        uint32_t crc = crc32(0, (const Bytef*)tag, 4);
        if (n) crc = crc32(crc, data, n);
        crc = __builtin_bswap32(crc);
        std::fwrite(&crc, 4, 1, f);
    };
    const uint8_t sig[8] = {137, 80, 78, 71, 13, 10, 26, 10};
    std::fwrite(sig, 1, 8, f);
    uint8_t ihdr[13];
    uint32_t bw = __builtin_bswap32((uint32_t)w), bh = __builtin_bswap32((uint32_t)h);
    std::memcpy(ihdr, &bw, 4);
    std::memcpy(ihdr + 4, &bh, 4);
    ihdr[8] = 8;
    ihdr[9] = 6;
    ihdr[10] = ihdr[11] = ihdr[12] = 0;
    chunk("IHDR", ihdr, 13);
    chunk("IDAT", z.data(), (uint32_t)bound);
    chunk("IEND", nullptr, 0);
    std::fclose(f);
    return true;
}
static std::vector<uint8_t> load_file(const char* path) {
    std::ifstream in(path, std::ios::binary);
    return std::vector<uint8_t>((std::istreambuf_iterator<char>(in)), {});
}
static std::vector<uint8_t> hstack(const std::vector<uint8_t>& a, const std::vector<uint8_t>& b) {
    std::vector<uint8_t> out((size_t)W * 2 * H * 4);
    for (int y = 0; y < H; ++y) {
        std::memcpy(out.data() + ((size_t)y * W * 2) * 4, a.data() + (size_t)y * W * 4, (size_t)W * 4);
        std::memcpy(out.data() + ((size_t)y * W * 2 + W) * 4, b.data() + (size_t)y * W * 4, (size_t)W * 4);
    }
    return out;
}
static std::vector<uint8_t> grid2(const std::vector<uint8_t>& a, const std::vector<uint8_t>& b, const std::vector<uint8_t>& c,
                                  const std::vector<uint8_t>& d) {
    std::vector<uint8_t> out((size_t)W * 2 * H * 2 * 4);
    const std::vector<uint8_t>* cells[4] = {&a, &b, &c, &d};
    for (int i = 0; i < 4; ++i) {
        int ox = (i % 2) * W;
        int oy = (i / 2) * H;
        for (int y = 0; y < H; ++y)
            std::memcpy(out.data() + ((size_t)(oy + y) * W * 2 + ox) * 4, cells[i]->data() + (size_t)y * W * 4, (size_t)W * 4);
    }
    return out;
}
static double ms_since(std::chrono::steady_clock::time_point t0) {
    return std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
}
static int fail(const char* msg) {
    std::fprintf(stderr, "FAIL %s\n", msg);
    return 1;
}
static double mae_of(const std::vector<uint8_t>& a, const std::vector<uint8_t>& b, int& far) {
    double mae = 0;
    far = 0;
    int n = W * H;
    for (int i = 0; i < n; ++i) {
        int dsum = 0;
        for (int ch = 0; ch < 3; ++ch) {
            int d = std::abs((int)a[(size_t)i * 4 + ch] - (int)b[(size_t)i * 4 + ch]);
            mae += d;
            dsum += d;
        }
        if (dsum > 80) ++far;
    }
    return mae / (double)n / 3.0;
}

struct App {
    Engine* engine = nullptr;
    Renderer* renderer = nullptr;
    Scene* scene = nullptr;
    View* view = nullptr;
    SwapChain* swap = nullptr;
    Camera* camera = nullptr;
    MaterialInstance* mi = nullptr;
    MaterialInstance* gpu = nullptr;
    RenderTarget* rt = nullptr;
};

static bool grab(App& a, std::vector<uint8_t>& picture) {
    picture.resize((size_t)W * H * 4);
    bool drew = false;
    for (int attempt = 0; attempt < 4 && !drew; ++attempt) {
        if (!a.renderer->beginFrame(a.swap)) continue;
        a.renderer->render(a.view);
        a.renderer->readPixels(a.rt, 0, 0, W, H,
                               backend::PixelBufferDescriptor(picture.data(), picture.size(), backend::PixelDataFormat::RGBA,
                                                              backend::PixelDataType::UBYTE));
        a.renderer->endFrame();
        drew = true;
    }
    a.engine->flushAndWait();
    return drew;
}
static utils::Entity upload(App& a, const Mesh& mesh, MaterialInstance* mi) {
    const uint32_t n = (uint32_t)(mesh.v.size() / 7);
    auto* vb = VertexBuffer::Builder()
                   .vertexCount(n)
                   .bufferCount(1)
                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                   .attribute(VertexAttribute::COLOR, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                   .build(*a.engine);
    vb->setBufferAt(*a.engine, 0, VertexBuffer::BufferDescriptor(mesh.v.data(), mesh.v.size() * sizeof(float)));
    auto* ib = IndexBuffer::Builder().indexCount((uint32_t)mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*a.engine);
    ib->setBuffer(*a.engine, IndexBuffer::BufferDescriptor(mesh.idx.data(), mesh.idx.size() * sizeof(uint32_t)));
    utils::Entity e = utils::EntityManager::get().create();
    RenderableManager::Builder(1)
        .boundingBox({{-4, -4, -4}, {4, 4, 4}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
        .material(0, mi)
        .culling(false)
        .castShadows(false)
        .receiveShadows(false)
        .build(*a.engine, e);
    a.scene->addEntity(e);
    return e;
}
static bool shot(App& a, const Mesh& mesh, float3 eye, float3 at, float3 up, std::vector<uint8_t>& rgba) {
    a.camera->lookAt(eye, at, up);
    utils::Entity e = upload(a, mesh, a.mi);
    a.engine->flushAndWait();
    bool ok = grab(a, rgba) && grab(a, rgba);
    a.scene->remove(e);
    return ok;
}

static Mesh gpu_topology(int rings, int sides) {
    Mesh m;
    int n = rings * sides;
    m.v.assign((size_t)n * 7, 0.f);
    for (size_t i = 3; i < m.v.size(); i += 7) m.v[i + 3] = 1.f;
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
static void pack_controls(const std::vector<Cubic>& spans, int tex_w, int tex_h, std::vector<float>& tex) {
    tex.assign((size_t)tex_w * tex_h * 4, 0.f);
    for (int s = 0; s < (int)spans.size(); ++s) {
        for (int k = 0; k < 4; ++k) {
            int index = s * 4 + k;
            tex[(size_t)index * 4 + 0] = spans[s].p[k].x;
            tex[(size_t)index * 4 + 1] = spans[s].p[k].y;
            tex[(size_t)index * 4 + 2] = spans[s].p[k].z;
            tex[(size_t)index * 4 + 3] = 1.f;
        }
    }
}
static void pack_frames(const std::vector<Sample>& samples, int tex_w, int tex_h, std::vector<float>& tex) {
    tex.assign((size_t)tex_w * tex_h * 4, 0.f);
    for (int i = 0; i < (int)samples.size(); ++i) {
        int n = i * 2;
        int b = n + 1;
        const Sample& s = samples[i];
        tex[(size_t)n * 4 + 0] = s.N.x;
        tex[(size_t)n * 4 + 1] = s.N.y;
        tex[(size_t)n * 4 + 2] = s.N.z;
        tex[(size_t)n * 4 + 3] = 0.f;
        tex[(size_t)b * 4 + 0] = s.B.x;
        tex[(size_t)b * 4 + 1] = s.B.y;
        tex[(size_t)b * 4 + 2] = s.B.z;
        tex[(size_t)b * 4 + 3] = 0.f;
    }
}
static int tex_rows(int texels, int tex_w) { return std::max(1, (texels + tex_w - 1) / tex_w); }
static Texture* float_tex(Engine* engine, int w, int h, const std::vector<float>& data) {
    Texture* tex = Texture::Builder()
                       .width((uint32_t)w)
                       .height((uint32_t)h)
                       .levels(1)
                       .sampler(Texture::Sampler::SAMPLER_2D)
                       .format(Texture::InternalFormat::RGBA32F)
                       .build(*engine);
    if (!tex) return nullptr;
    tex->setImage(*engine, 0,
                  Texture::PixelBufferDescriptor(data.data(), data.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                 backend::PixelDataType::FLOAT, 1, 0, 0, (uint32_t)w));
    return tex;
}

int main() {
    auto spatial_pkg = load_file("spatial.filamat");
    auto gpu_pkg = load_file("robust_gpu.filamat");
    if (spatial_pkg.empty() || gpu_pkg.empty()) return fail("filamat");

    App a;
    a.engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    a.renderer = a.engine->createRenderer();
    a.scene = a.engine->createScene();
    a.view = a.engine->createView();
    a.swap = a.engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    a.camera = a.engine->createCamera(utils::EntityManager::get().create());
    a.camera->setExposure(16.f, 1.f / 125.f, 100.f);
    a.camera->setProjection(38.0, double(W) / double(H), 0.05, 40.0, Camera::Fov::VERTICAL);
    Texture* color = Texture::Builder()
                         .width(W)
                         .height(H)
                         .levels(1)
                         .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                         .format(Texture::InternalFormat::RGBA8)
                         .build(*a.engine);
    Texture* depth = Texture::Builder()
                         .width(W)
                         .height(H)
                         .levels(1)
                         .usage(Texture::Usage::DEPTH_ATTACHMENT)
                         .format(Texture::InternalFormat::DEPTH24)
                         .build(*a.engine);
    a.rt = RenderTarget::Builder()
               .texture(RenderTarget::AttachmentPoint::COLOR, color)
               .texture(RenderTarget::AttachmentPoint::DEPTH, depth)
               .build(*a.engine);
    static LinearToneMapper tone;
    a.view->setColorGrading(ColorGrading::Builder().toneMapper(&tone).build(*a.engine));
    a.scene->setSkybox(Skybox::Builder().color({0.04f, 0.045f, 0.055f, 1}).build(*a.engine));
    a.view->setScene(a.scene);
    a.view->setCamera(a.camera);
    a.view->setRenderTarget(a.rt);
    a.view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    a.view->setPostProcessingEnabled(true);
    a.view->setScreenSpaceRefractionEnabled(false);
    a.view->setAntiAliasing(View::AntiAliasing::FXAA);
    a.view->setDithering(View::Dithering::NONE);
    Renderer::ClearOptions clear{};
    clear.clearColor = {0.04f, 0.045f, 0.055f, 1};
    clear.clear = true;
    a.renderer->setClearOptions(clear);
    Material* spatial_mat = Material::Builder().package(spatial_pkg.data(), spatial_pkg.size()).build(*a.engine);
    Material* gpu_mat = Material::Builder().package(gpu_pkg.data(), gpu_pkg.size()).build(*a.engine);
    if (!spatial_mat || !gpu_mat) return fail("material");
    a.mi = spatial_mat->createInstance();
    a.gpu = gpu_mat->createInstance();

    const float3 up{0, 1, 0};
    const float half = 0.075f;
    auto loop = closed_loop();
    auto open_frames = sample_uniform(loop, 16, true);
    Shut shut = close_arc(open_frames);
    Shut planar_shut = close_arc(sample_uniform(planar_rect(), 8, true));
    const float deg = 57.2957795f;
    float before = shut.before * deg;
    float join = shut.join * deg;
    float last_step = shut.last_step * deg;
    float planar_before = planar_shut.before * deg;
    float planar_join = planar_shut.join * deg;

    std::vector<uint8_t> c0, c1, c2, c3;
    const float3 loop_eye{1.55f, 1.25f, 1.55f};
    const float3 loop_at{0, 0, 0.1f};
    if (!shot(a, ribbon(open_frames, half, true), loop_eye, loop_at, up, c0)) return fail("seam before");
    if (!shot(a, ribbon(shut.frames, half, false), loop_eye, loop_at, up, c1)) return fail("seam after");
    auto seam = hstack(c0, c1);
    if (!write_png("out/robust-seam.png", W * 2, H, seam.data())) return fail("png seam");

    auto cusp = sample_uniform(cusp_curve(), 24, false);
    auto corners = sample_uniform(planar_rect(), 8, true);
    if (!shot(a, tube_colored(cusp, 0.045f, 16, false), {1.15f, 0.75f, 1.15f}, {0.4f, 0.12f, 0.04f}, up, c0)) return fail("cusp");
    if (!shot(a, tube_colored(corners, 0.045f, 12, false), {0.2f, 1.35f, 1.5f}, {0, 0, 0}, up, c1)) return fail("corners");
    auto cusp_pic = hstack(c0, c1);
    if (!write_png("out/robust-cusp.png", W * 2, H, cusp_pic.data())) return fail("png cusp");

    auto bend = local_bend();
    const float tol = 0.01f;
    const float adapt_r = 0.04f;
    std::vector<Sample> coarse;
    for (int s = 0; s < (int)bend.size(); ++s) push_sample(coarse, bend[s], 0, !coarse.empty(), coarse.empty() ? Sample{} : coarse.back());
    push_sample(coarse, bend.back(), 1, true, coarse.back());
    auto adapted = sample_adaptive(bend, adapt_r, tol);
    int coarse_bend = bend_rings(coarse, -0.02f, 0.12f, 0.12f);
    int adapt_bend = bend_rings(adapted, -0.02f, 0.12f, 0.12f);
    int coarse_straight = (int)coarse.size() - coarse_bend;
    int adapt_straight = (int)adapted.size() - adapt_bend;
    const float3 bend_eye{1.1f, 0.9f, 1.3f};
    const float3 bend_at{0.0f, 0.15f, 0};
    if (!shot(a, tube_colored(coarse, adapt_r, 12, false), bend_eye, bend_at, up, c0)) return fail("coarse");
    if (!shot(a, tube_colored(adapted, adapt_r, 12, false), bend_eye, bend_at, up, c1)) return fail("adapt");
    auto adapt_pic = hstack(c0, c1);
    if (!write_png("out/robust-adapt.png", W * 2, H, adapt_pic.data())) return fail("png adapt");

    auto local_s = sample_uniform(bend, 32, false);
    Hit local_hit = classify(local_s, 0.09f, 0.25f);
    auto global_s = sample_uniform(global_pass(), 12, false);
    Hit global_hit = classify(global_s, 0.09f, 0.5f);
    const float3 local_eye{0.85f, 0.7f, 1.05f};
    const float3 global_eye{1.6f, 1.5f, 2.2f};
    const float3 global_at{0, 0, 0};
    if (!shot(a, tube_colored(local_s, 0.09f, 16, false), local_eye, bend_at, up, c0)) return fail("local allow");
    if (!shot(a, tube_colored(local_s, local_hit.safe_r, 16, false), local_eye, bend_at, up, c1)) return fail("local resolve");
    if (!shot(a, tube_colored(global_s, 0.09f, 12, false), global_eye, global_at, up, c2)) return fail("global allow");
    if (!shot(a, tube_colored(global_s, global_hit.safe_r, 12, false), global_eye, global_at, up, c3)) return fail("global resolve");
    auto hits = grid2(c0, c1, c2, c3);
    if (!write_png("out/robust-hit.png", W * 2, H * 2, hits.data())) return fail("png hit");

    auto picture_curve = stress_curve(64);
    const int pic_samples = 8;
    const int pic_sides = 12;
    const float pic_r = 0.045f;
    const int tex_w = 256;
    auto picture_frames = sample_uniform(picture_curve, pic_samples, false);
    int tex_h = tex_rows((int)picture_curve.size() * 4, tex_w);
    int frame_h = tex_rows((int)picture_frames.size() * 2, tex_w);
    std::vector<float> texels, frame_texels;
    pack_controls(picture_curve, tex_w, tex_h, texels);
    pack_frames(picture_frames, tex_w, frame_h, frame_texels);
    Texture* controls = float_tex(a.engine, tex_w, tex_h, texels);
    Texture* frames_tex = float_tex(a.engine, tex_w, frame_h, frame_texels);
    if (!controls || !frames_tex) return fail("control texture");
    TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    a.gpu->setParameter("controls", controls, nearest);
    a.gpu->setParameter("frames", frames_tex, nearest);
    a.gpu->setParameter("samples", (float)pic_samples);
    a.gpu->setParameter("sides", (float)pic_sides);
    a.gpu->setParameter("spans", (float)picture_curve.size());
    a.gpu->setParameter("radius", pic_r);
    a.gpu->setParameter("texWidth", (float)tex_w);
    a.gpu->setParameter("texHeight", (float)tex_h);
    a.gpu->setParameter("frameHeight", (float)frame_h);
    Mesh topo = gpu_topology((int)picture_frames.size(), pic_sides);
    const float3 gpu_eye{1.7f, 1.35f, 1.9f};
    a.camera->lookAt(gpu_eye, {0, 0, 0}, up);
    utils::Entity ge = upload(a, topo, a.gpu);
    a.engine->flushAndWait();
    std::vector<uint8_t> gpu_pic, cpu_pic;
    if (!grab(a, gpu_pic) || !grab(a, gpu_pic)) return fail("gpu draw");
    a.scene->remove(ge);
    if (!shot(a, tube_colored(picture_frames, pic_r, pic_sides, false), gpu_eye, {0, 0, 0}, up, cpu_pic)) return fail("cpu parallel");
    int far_px = 0;
    double mae = mae_of(gpu_pic, cpu_pic, far_px);
    if (!write_png("out/robust-gpu.png", W * 2, H, hstack(gpu_pic, cpu_pic).data())) return fail("png gpu");

    auto vertical = vertical_curve();
    auto vertical_frames = sample_uniform(vertical, 16, false);
    int z_tangents = 0;
    for (const Sample& s : vertical_frames)
        if (std::fabs(s.T.z) > 0.98f) ++z_tangents;
    int z_h = tex_rows((int)vertical.size() * 4, tex_w);
    int z_fh = tex_rows((int)vertical_frames.size() * 2, tex_w);
    std::vector<float> z_controls, z_frames;
    pack_controls(vertical, tex_w, z_h, z_controls);
    pack_frames(vertical_frames, tex_w, z_fh, z_frames);
    Texture* z_ct = float_tex(a.engine, tex_w, z_h, z_controls);
    Texture* z_ft = float_tex(a.engine, tex_w, z_fh, z_frames);
    if (!z_ct || !z_ft) return fail("vertical texture");
    a.gpu->setParameter("controls", z_ct, nearest);
    a.gpu->setParameter("frames", z_ft, nearest);
    a.gpu->setParameter("samples", 16.f);
    a.gpu->setParameter("sides", 12.f);
    a.gpu->setParameter("spans", (float)vertical.size());
    a.gpu->setParameter("radius", 0.05f);
    a.gpu->setParameter("texHeight", (float)z_h);
    a.gpu->setParameter("frameHeight", (float)z_fh);
    const float3 z_eye{1.55f, 1.25f, 0.7f};
    const float3 z_at{0.2f, 0.1f, 0.75f};
    a.camera->lookAt(z_eye, z_at, up);
    Mesh z_topo = gpu_topology((int)vertical_frames.size(), 12);
    utils::Entity ze = upload(a, z_topo, a.gpu);
    a.engine->flushAndWait();
    std::vector<uint8_t> z_gpu, z_cpu;
    if (!grab(a, z_gpu) || !grab(a, z_gpu)) return fail("vertical gpu");
    a.scene->remove(ze);
    if (!shot(a, tube_colored(vertical_frames, 0.05f, 12, false), z_eye, z_at, up, z_cpu)) return fail("vertical cpu");
    int z_far = 0;
    double z_mae = mae_of(z_gpu, z_cpu, z_far);
    if (!write_png("out/robust-frame.png", W * 2, H, hstack(z_gpu, z_cpu).data())) return fail("png frame");

    FILE* log = std::fopen("out/robust.txt", "w");
    std::fprintf(log, "3D Path の堅牢性。成立済みの経路の上で、継ぎ目、尖点、分割、制御点の枚数、自己交差を測る。\n\n");
    std::fprintf(log, "継ぎ目\n");
    std::fprintf(log, "補正前の接合 %.2f 度。これは最後のサンプルを始点の接線まで運んだ角で、終点の輪はまだ無い。\n", before);
    std::fprintf(log, "弧長比例の回転を最後の区間まで載せ、終点の輪を始点と同じ位置に置いた。始点と終点の断面角 %.8f 度。縁のずれ %.8f。\n", join,
                 shut.edge);
    std::fprintf(log, "最後の区間に残るねじれは %.2f 度。これは分配の一段で、始点と終点の不一致ではない。\n", last_step);
    std::fprintf(log, "平面の矩形は補正前 %.8f 度、始点と終点 %.8f 度。角の接線は極限で回復した %d サンプル。\n", planar_before, planar_join,
                 count_recovered(corners));
    std::fprintf(log, "out/robust-seam.png 左が補正前、右が終点まで閉じた帯。\n\n");

    std::fprintf(log, "尖点\n");
    std::fprintf(log, "始点で制御点が重なる曲線。回復した接線 %d、枠の落下 %d / %d。帯の半幅の実長 %.4f。\n", count_recovered(cusp),
                 count_fail(cusp), (int)cusp.size(), edge_min(cusp, 0.045f));
    std::fprintf(log, "矩形の角はハンドルが端点にあり、一次導関数が 0。回復 %d、落下 %d / %d。\n", count_recovered(corners), count_fail(corners),
                 (int)corners.size());
    std::fprintf(log, "out/robust-cusp.png 左が尖点の管、右が矩形の角。白は枠が落ちた輪。\n\n");

    std::fprintf(log, "適応分割\n");
    std::fprintf(log, "許容は平坦さと半径×折れ角の両方で %.3f。半径 %.3f。\n", tol, adapt_r);
    std::fprintf(log, "スパンの端だけは輪 %d、曲がり付近 %d、それ以外 %d。\n", (int)coarse.size(), coarse_bend, coarse_straight);
    std::fprintf(log, "適応は輪 %d、曲がり付近 %d、それ以外 %d。\n", (int)adapted.size(), adapt_bend, adapt_straight);
    std::fprintf(log, "out/robust-adapt.png 左が等分割、右が適応。\n\n");

    std::fprintf(log, "自己交差\n");
    std::fprintf(log, "局所: 半径 0.090、最大曲率 %.3f、r*κ %.3f、%s。最近接（弧長 0.25 以上）%.3f。安全半径 %.4f。\n", local_hit.kappa_max,
                 0.09f * local_hit.kappa_max, local_hit.local ? "局所交差" : "局所は半径内", local_hit.separation, local_hit.safe_r);
    std::fprintf(log, "大域: 半径 0.090、最大曲率 %.3f、r*κ %.3f、%s。最近接（弧長 0.50 以上）%.3f、直径 0.180。安全半径 %.4f。\n",
                 global_hit.kappa_max, 0.09f * global_hit.kappa_max, global_hit.global ? "大域交差" : "大域は離れている", global_hit.separation,
                 global_hit.safe_r);
    std::fprintf(log, "許容は指定半径のまま描く。解消は安全半径への縮小で、太さを保った融合ではない。融合は別の形状演算として残す。\n");
    std::fprintf(log, "out/robust-hit.png 左上が局所の許容、右上が解消、左下が大域の許容、右下が解消。\n\n");

    std::fprintf(log, "GPU の断面\n");
    std::fprintf(log, "曲線の位置は頂点シェーダが制御点から計算する。断面の N と B は CPU の Parallel Transport を RGBA32F で渡す。\n");
    std::fprintf(log, "64 スパンの螺旋。GPU と CPU の画素差 mean abs %.2f、差の大きい画素 %d / %d。out/robust-gpu.png 左が GPU、右が CPU。\n", mae,
                 far_px, W * H);
    std::fprintf(log, "Z 方向の直線を含む曲線。|T.z|>0.98 は %d / %d。Parallel の落下 %d。GPU と CPU の画素差 %.2f、差の大きい画素 %d。\n",
                 z_tangents, (int)vertical_frames.size(), count_fail(vertical_frames), z_mae, z_far);
    std::fprintf(log, "out/robust-frame.png 左が GPU、右が CPU。どちらも同じ Parallel の枠。\n");
    std::fprintf(log, "時間はミリ秒。毎フレームは枠の再計算、制御点と枠のパック、テクスチャ転送、描画。トポロジは輪の数が変わらない限り一度。描画に readPixels は含まない。60fps の予算は 16.67 ms。\n");

    const double budget = 16.67;
    double worst = 0;
    for (int spans : {64, 256, 1024, 4096}) {
        auto curve = stress_curve(spans);
        int samples = 6;
        int sides = 8;
        auto t_frame = std::chrono::steady_clock::now();
        auto samp = sample_uniform(curve, samples, false);
        double frame_ms = ms_since(t_frame);
        int th = tex_rows((int)curve.size() * 4, tex_w);
        int fh = tex_rows((int)samp.size() * 2, tex_w);
        auto t_pack = std::chrono::steady_clock::now();
        pack_controls(curve, tex_w, th, texels);
        pack_frames(samp, tex_w, fh, frame_texels);
        double pack_ms = ms_since(t_pack);
        Texture* tex = float_tex(a.engine, tex_w, th, texels);
        Texture* ftex = float_tex(a.engine, tex_w, fh, frame_texels);
        a.engine->flushAndWait();
        auto t_up = std::chrono::steady_clock::now();
        tex->setImage(*a.engine, 0,
                      Texture::PixelBufferDescriptor(texels.data(), texels.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                     backend::PixelDataType::FLOAT, 1, 0, 0, (uint32_t)tex_w));
        ftex->setImage(*a.engine, 0,
                       Texture::PixelBufferDescriptor(frame_texels.data(), frame_texels.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                      backend::PixelDataType::FLOAT, 1, 0, 0, (uint32_t)tex_w));
        a.engine->flushAndWait();
        double upload_ms = ms_since(t_up);
        a.gpu->setParameter("controls", tex, nearest);
        a.gpu->setParameter("frames", ftex, nearest);
        a.gpu->setParameter("samples", (float)samples);
        a.gpu->setParameter("sides", (float)sides);
        a.gpu->setParameter("spans", (float)curve.size());
        a.gpu->setParameter("radius", pic_r);
        a.gpu->setParameter("texWidth", (float)tex_w);
        a.gpu->setParameter("texHeight", (float)th);
        a.gpu->setParameter("frameHeight", (float)fh);
        auto t_topo = std::chrono::steady_clock::now();
        Mesh top = gpu_topology((int)samp.size(), sides);
        double topo_ms = ms_since(t_topo);
        a.camera->lookAt(gpu_eye, {0, 0, 0}, up);
        utils::Entity e = upload(a, top, a.gpu);
        a.engine->flushAndWait();
        auto t_draw = std::chrono::steady_clock::now();
        int drew = 0;
        for (int i = 0; i < 3; ++i) {
            if (!a.renderer->beginFrame(a.swap)) continue;
            a.renderer->render(a.view);
            a.renderer->endFrame();
            ++drew;
        }
        a.engine->flushAndWait();
        double draw_ms = drew ? ms_since(t_draw) / (double)drew : -1;
        a.scene->remove(e);
        double per_frame = frame_ms + pack_ms + upload_ms + draw_ms;
        if (per_frame > worst) worst = per_frame;
        std::fprintf(log, "spans %d  rings %d  verts %d  枠 %.2f  パック %.2f  転送 %.2f  描画 %.2f  毎フレーム %.2f  トポロジ一度 %.2f  %s\n",
                     spans, (int)samp.size(), (int)(top.v.size() / 7), frame_ms, pack_ms, upload_ms, draw_ms, per_frame, topo_ms,
                     per_frame <= budget ? "予算内" : "予算超過");
    }
    std::fprintf(log, "毎フレームの最大は %.2f ms。トポロジの作り直しは含めていない。\n", worst);
    std::fclose(log);

    std::printf("join %.8f deg  edge %.8f  last_step %.2f  ztangents %d  zmae %.2f  gpu mae %.2f  worst %.2f\n", join, shut.edge, last_step,
                z_tangents, z_mae, mae, worst);
    std::printf("PASS robust\n");
    return 0;
}
