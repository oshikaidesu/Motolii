// 3D path probe. ThorVG keeps the 2D curve and the projected guide.
// Filament draws the solid. The 3D control points live in this file.
#include <thorvg.h>

#include <filament/Camera.h>
#include <filament/ColorGrading.h>
#include <filament/Engine.h>
#include <filament/IndexBuffer.h>
#include <filament/LightManager.h>
#include <filament/Material.h>
#include <filament/MaterialInstance.h>
#include <filament/RenderableManager.h>
#include <filament/Renderer.h>
#include <filament/RenderTarget.h>
#include <filament/Scene.h>
#include <filament/Skybox.h>
#include <filament/SwapChain.h>
#include <filament/Texture.h>
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
#include <string>
#include <vector>

using namespace filament;
using namespace filament::math;

static const int W = 640;
static const int H = 400;
static const int SAMPLES = 16;
static const int SIDES = 16;
static const float TUBE_R = 0.055f;
static const float RIBBON_W = 0.09f;

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

enum class FrameKind { ZUp, Frenet, Parallel };
enum class ShapeKind { Tube, Ribbon, Sweep };

struct Sample {
    V3 p, T, N, B;
    bool zero_tangent = false;
    bool frame_fail = false;
};

struct Mesh {
    std::vector<float> v;
    std::vector<uint32_t> idx;
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

static void hsv(float deg, float s, float v, float& r, float& g, float& b) {
    float h = deg / 60.f;
    float c = v * s;
    float x = c * (1.f - std::fabs(std::fmod(h, 2.f) - 1.f));
    float m = v - c;
    r = m;
    g = m;
    b = m;
    int k = (int)std::floor(h);
    float* rgb[6][3] = {};
    (void)rgb;
    switch ((k % 6 + 6) % 6) {
    case 0: r += c; g += x; break;
    case 1: r += x; g += c; break;
    case 2: g += c; b += x; break;
    case 3: g += x; b += c; break;
    case 4: r += x; b += c; break;
    default: r += c; b += x; break;
    }
}

static int fail(const char* msg) {
    std::fprintf(stderr, "FAIL %s\n", msg);
    return 1;
}

static std::vector<Cubic> line_spans(V3 a, V3 b, V3 c, V3 d) {
    auto line = [](V3 p, V3 q) { return Cubic{p, p, q, q}; };
    return {line(a, b), line(b, c), line(c, d), line(d, a)};
}

static std::vector<Cubic> flat_rect() {
    return line_spans({-0.85f, -0.40f, 0}, {0.85f, -0.40f, 0}, {0.85f, 0.40f, 0}, {-0.85f, 0.40f, 0});
}
static std::vector<Cubic> lifted_rect() {
    return line_spans({-0.85f, -0.40f, 0}, {0.85f, -0.40f, 0}, {0.85f, 0.40f, 0.55f}, {-0.85f, 0.40f, 0.55f});
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

static std::vector<V3> edit_points() {
    return {{-0.9f, -0.2f, 0.1f}, {-0.3f, 0.45f, 0.7f}, {0.25f, -0.15f, -0.45f}, {0.85f, 0.35f, 0.35f}, {0.15f, -0.55f, 0.9f}};
}

static std::vector<Cubic> diag_curve() {
    Cubic s0{{{ -0.9f, 0, 0 }, { -0.65f, 0, 0 }, { -0.4f, 0, 0 }, { -0.2f, 0, 0 }}};
    Cubic s1{{{ -0.2f, 0, 0 }, { 0.1f, 0, 0 }, { 0.2f, 0, 0.3f }, { 0.2f, 0, 0.6f }}};
    Cubic s2{{{ 0.2f, 0, 0.6f }, { 0.2f, 0, 0.85f }, { 0.2f, 0, 1.05f }, { 0.2f, 0, 1.2f }}};
    Cubic s3{{{ 0.2f, 0, 1.2f }, { 0.2f, 0, 1.5f }, { 0.7f, 0.45f, 1.45f }, { 0.95f, 0.55f, 1.05f }}};
    return {s0, s1, s2, s3};
}

static std::vector<Cubic> sharp_corner() {
    auto c = diag_curve();
    c[1].p[1] = {-0.2f, 0.55f, 0.15f};
    return c;
}

static std::vector<Cubic> cusp_curve() {
    return {Cubic{{{0, 0, 0}, {0, 0, 0}, {0.45f, 0.25f, 0.2f}, {0.9f, 0, 0.1f}}}};
}

static std::vector<Cubic> cross_curve() {
    return {Cubic{{{ -0.7f, -0.45f, 0 }, { -0.2f, -0.1f, 0 }, { 0.2f, 0.15f, 0 }, { 0.7f, 0.45f, 0 }}},
            Cubic{{{ -0.7f, 0.45f, 0.25f }, { -0.2f, 0.1f, 0.05f }, { 0.2f, -0.15f, -0.05f }, { 0.7f, -0.45f, -0.2f }}}};
}

static std::vector<Cubic> closed_curve() {
    std::vector<V3> p = {{-0.6f, -0.45f, 0}, {0.65f, -0.25f, 0.45f}, {0.35f, 0.6f, -0.35f}, {-0.55f, 0.25f, 0.5f}};
    return catmull(p, true);
}

static float c0_gap(const std::vector<Cubic>& spans) {
    float m = 0;
    for (size_t i = 1; i < spans.size(); ++i) m = std::max(m, len(spans[i].p[0] - spans[i - 1].p[3]));
    return m;
}
static float c1_deg(const std::vector<Cubic>& spans) {
    float m = 0;
    for (size_t i = 1; i < spans.size(); ++i) {
        V3 a = spans[i - 1].p[3] - spans[i - 1].p[2];
        V3 b = spans[i].p[1] - spans[i].p[0];
        if (len(a) < 1e-5f || len(b) < 1e-5f) continue;
        float d = std::clamp(dot(norm(a), norm(b)), -1.f, 1.f);
        m = std::max(m, std::acos(d) * 57.2958f);
    }
    return m;
}
static int zero_handles(const std::vector<Cubic>& spans) {
    int n = 0;
    for (const Cubic& c : spans) {
        if (len(c.p[1] - c.p[0]) < 1e-5f) ++n;
        if (len(c.p[3] - c.p[2]) < 1e-5f) ++n;
    }
    return n;
}

static bool frame_at(const Cubic& c, float t, FrameKind kind, V3 prev_n, bool has_prev, Sample& s) {
    V3 d = bezier_d(c, t);
    s.p = bezier(c, t);
    s.zero_tangent = len(d) < 1e-4f;
    s.T = s.zero_tangent ? (has_prev ? V3{0, 0, 0} : V3{1, 0, 0}) : norm(d);
    if (s.zero_tangent && has_prev) s.T = V3{0, 0, 0};
    s.frame_fail = false;
    if (kind == FrameKind::ZUp) {
        V3 up = std::fabs(s.T.z) > 0.98f ? V3{0, 1, 0} : V3{0, 0, 1};
        if (std::fabs(s.T.z) > 0.98f) s.frame_fail = true;
        V3 side = norm(cross(s.T, up));
        if (len(side) < 0.5f) {
            s.frame_fail = true;
            side = {1, 0, 0};
        }
        s.B = side;
        s.N = norm(cross(side, s.T));
        return true;
    }
    if (kind == FrameKind::Frenet) {
        V3 acc = bezier_dd(c, t);
        V3 bin = cross(s.T, acc);
        if (len(bin) < 1e-4f) {
            s.frame_fail = true;
            if (!has_prev) {
                V3 up = std::fabs(s.T.z) < 0.9f ? V3{0, 0, 1} : V3{0, 1, 0};
                s.N = norm(cross(up, s.T));
                s.B = norm(cross(s.T, s.N));
            } else {
                s.N = norm(prev_n - s.T * dot(prev_n, s.T));
                s.B = norm(cross(s.T, s.N));
            }
            return true;
        }
        s.B = norm(bin);
        s.N = norm(cross(s.B, s.T));
        if (has_prev && dot(s.N, prev_n) < 0) {
            s.N = s.N * -1.f;
            s.B = s.B * -1.f;
        }
        return true;
    }
    V3 up = std::fabs(s.T.z) < 0.9f ? V3{0, 0, 1} : V3{0, 1, 0};
    V3 n = has_prev ? prev_n - s.T * dot(prev_n, s.T) : cross(up, s.T);
    if (len(n) < 1e-5f) {
        s.frame_fail = true;
        n = cross(up, s.T);
    }
    s.N = norm(n);
    s.B = norm(cross(s.T, s.N));
    return true;
}

static std::vector<Sample> sample_curve(const std::vector<Cubic>& spans, FrameKind kind, bool closed, int samples) {
    std::vector<Sample> out;
    Sample prev{};
    bool has = false;
    int segs = (int)spans.size();
    for (int s = 0; s < segs; ++s) {
        for (int i = 0; i < samples; ++i) {
            Sample cur{};
            V3 hold_t = has ? prev.T : V3{1, 0, 0};
            frame_at(spans[s], (float)i / (float)samples, kind, prev.N, has, cur);
            if (cur.zero_tangent) cur.T = hold_t;
            if (len(cur.N) < 0.5f) {
                V3 up = {0, 0, 1};
                cur.N = norm(cross(up, cur.T));
                cur.B = norm(cross(cur.T, cur.N));
                cur.frame_fail = true;
            }
            out.push_back(cur);
            prev = cur;
            has = true;
        }
    }
    if (closed && !out.empty()) {
        Sample extra{};
        frame_at(spans[0], 0, kind, prev.N, true, extra);
        (void)extra;
    }
    return out;
}

static float holonomy_deg(const std::vector<Cubic>& spans, FrameKind kind, int samples) {
    auto s = sample_curve(spans, kind, true, samples);
    if (s.size() < 2) return 0;
    V3 T0 = s[0].T;
    V3 n = s.back().N - T0 * dot(s.back().N, T0);
    if (len(n) < 1e-5f) return 180.f;
    float d = std::clamp(dot(norm(n), s[0].N), -1.f, 1.f);
    return std::acos(d) * 57.2958f;
}

static int count_flips(const std::vector<Sample>& s) {
    int n = 0;
    for (size_t i = 1; i < s.size(); ++i)
        if (dot(s[i].N, s[i - 1].N) < 0.f) ++n;
    return n;
}
static int count_fail(const std::vector<Sample>& s) {
    int n = 0;
    for (const Sample& a : s)
        if (a.frame_fail) ++n;
    return n;
}

static void push_vert(Mesh& m, V3 p, float r, float g, float b) {
    m.v.insert(m.v.end(), {p.x, p.y, p.z, r, g, b, 1.f});
}

static Mesh build_shape(const std::vector<Sample>& samples, ShapeKind shape, bool closed) {
    Mesh m;
    int rings = (int)samples.size();
    int around = shape == ShapeKind::Tube ? SIDES : shape == ShapeKind::Ribbon ? 2 : 4;
    const float sq[4][2] = {{0.10f, 0.045f}, {0.10f, -0.045f}, {-0.10f, -0.045f}, {-0.10f, 0.045f}};
    for (int i = 0; i < rings; ++i) {
        const Sample& s = samples[i];
        float hue = std::atan2(s.N.y, s.N.x) * 57.2958f;
        if (hue < 0) hue += 360.f;
        for (int k = 0; k < around; ++k) {
            V3 p = s.p;
            float r, g, b;
            if (s.frame_fail || s.zero_tangent) {
                r = g = b = 1.f;
            } else if (shape == ShapeKind::Tube) {
                float ang = (float)k / SIDES * 6.2831853f;
                p = s.p + (s.N * std::cos(ang) + s.B * std::sin(ang)) * TUBE_R;
                hsv(hue, 0.55f, 0.45f + 0.55f * (0.5f + 0.5f * std::cos(ang)), r, g, b);
            } else if (shape == ShapeKind::Ribbon) {
                float u = k == 0 ? -RIBBON_W : RIBBON_W;
                p = s.p + s.N * u;
                r = k == 0 ? 0.95f : 0.25f;
                g = 0.35f;
                b = k == 0 ? 0.28f : 0.95f;
            } else {
                p = s.p + s.N * sq[k][0] + s.B * sq[k][1];
                hsv(k * 80.f + hue * 0.15f, 0.65f, 0.9f, r, g, b);
            }
            push_vert(m, p, r, g, b);
        }
    }
    int limit = closed ? rings : rings - 1;
    for (int i = 0; i < limit; ++i) {
        int i1 = (i + 1) % rings;
        for (int k = 0; k < around; ++k) {
            uint32_t a = (uint32_t)(i * around + k);
            uint32_t b = (uint32_t)(i * around + (k + 1) % around);
            uint32_t c = (uint32_t)(i1 * around + k);
            uint32_t d = (uint32_t)(i1 * around + (k + 1) % around);
            m.idx.insert(m.idx.end(), {a, b, c, b, d, c});
        }
    }
    return m;
}

static float spine_error(const std::vector<Cubic>& spans, const std::vector<Sample>& samples, int samples_n) {
    float m = 0;
    for (int i = 0; i < (int)samples.size(); ++i) {
        int span = i / samples_n;
        float t = (float)(i - span * samples_n) / (float)samples_n;
        m = std::max(m, len(samples[i].p - bezier(spans[span], t)));
    }
    return m;
}

static float min_center_clearance(const std::vector<Sample>& s) {
    float m = 1e9f;
    int n = (int)s.size();
    for (int i = 0; i < n; ++i)
        for (int j = i + 8; j < n; ++j) m = std::min(m, len(s[i].p - s[j].p));
    return m;
}

static uint64_t mesh_hash(const Mesh& m) {
    uint64_t h = 1469598103934665603ull;
    for (float f : m.v) {
        uint32_t u;
        std::memcpy(&u, &f, 4);
        h ^= u;
        h *= 1099511628211ull;
    }
    return h;
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

static bool write_xy(tvg::Shape* shape, const std::vector<Cubic>& spans) {
    if (spans.empty()) return false;
    if (shape->moveTo(spans[0].p[0].x, spans[0].p[0].y) != tvg::Result::Success) return false;
    for (const Cubic& c : spans)
        if (shape->cubicTo(c.p[1].x, c.p[1].y, c.p[2].x, c.p[2].y, c.p[3].x, c.p[3].y) != tvg::Result::Success) return false;
    return true;
}


#include "spatial_2.h"
