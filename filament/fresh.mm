// A new still. Not the beauty path and not the window.
// A vertical glass helix, a chrome arc beside it, a tilted prism ring in front.
// Same Filament path: RGBA16F plates, one glass per render(), AgX once at the end.
#include <filament/Camera.h>
#include <filament/ColorGrading.h>
#include <filament/Engine.h>
#include <filament/IndexBuffer.h>
#include <filament/IndirectLight.h>
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
#include <filament/TextureSampler.h>
#include <filament/ToneMapper.h>
#include <filament/VertexBuffer.h>
#include <filament/View.h>
#include <filament/Viewport.h>

#include <image/Ktx1Bundle.h>
#include <ktxreader/Ktx1Reader.h>

#include <math/mat3.h>
#include <math/vec3.h>
#include <math/vec4.h>

#include <utils/EntityManager.h>

#include <zlib.h>

#include <algorithm>
#include <clocale>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <fstream>
#include <string>
#include <vector>

using namespace filament;
using namespace filament::math;

static const int W = 1920;
static const int H = 1080;
static const float PI = 3.14159265f;
static const float TAU = PI * 2.f;

struct V3 {
    float x, y, z;
    V3 operator+(V3 o) const { return {x + o.x, y + o.y, z + o.z}; }
    V3 operator-(V3 o) const { return {x - o.x, y - o.y, z - o.z}; }
    V3 operator*(float s) const { return {x * s, y * s, z * s}; }
};
static float dot(V3 a, V3 b) { return a.x * b.x + a.y * b.y + a.z * b.z; }
static V3 cross(V3 a, V3 b) {
    return {a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x};
}
static float len(V3 a) { return std::sqrt(dot(a, a)); }
static V3 norm(V3 a) {
    float L = len(a);
    return L > 1e-8f ? a * (1.f / L) : V3{0, 0, 0};
}

struct Cubic { V3 p[4]; };
struct Sample {
    V3 p, T, N, B;
    float s = 0;
};
struct Built {
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
static V3 rot_about(V3 v, V3 T, float ang) {
    return v * std::cos(ang) + cross(T, v) * std::sin(ang);
}
static void rotX(V3& v, float ca, float sa) {
    float y = v.y * ca - v.z * sa;
    float z = v.y * sa + v.z * ca;
    v.y = y;
    v.z = z;
}

static void push_sample(std::vector<Sample>& out, V3 p, V3 T) {
    Sample s;
    bool has = !out.empty();
    Sample prev = has ? out.back() : Sample{};
    s.p = p;
    s.T = norm(T);
    if (len(s.T) < 0.5f) s.T = has ? prev.T : V3{0, 1, 0};
    V3 up = std::fabs(s.T.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0};
    V3 n = has ? prev.N - s.T * dot(prev.N, s.T) : cross(up, s.T);
    if (len(n) < 1e-4f) n = cross(up, s.T);
    s.N = norm(n);
    s.B = norm(cross(s.T, s.N));
    s.s = has ? prev.s + len(s.p - prev.p) : 0;
    out.push_back(s);
}

static std::vector<Sample> helix(int n, float R, float turns, float y0, float y1, V3 c) {
    std::vector<Sample> out;
    for (int i = 0; i < n; ++i) {
        float u = (float)i / (float)(n - 1);
        float a = turns * TAU * u;
        float y = y0 + (y1 - y0) * u;
        V3 d{-R * std::sin(a) * turns * TAU, y1 - y0, R * std::cos(a) * turns * TAU};
        push_sample(out, c + V3{R * std::cos(a), y, R * std::sin(a)}, d);
    }
    return out;
}

static Cubic arc(float R, float a0, float a1) {
    V3 p0{R * std::cos(a0), R * std::sin(a0), 0};
    V3 p3{R * std::cos(a1), R * std::sin(a1), 0};
    float k = R * 0.55228475f * (a1 - a0) / (PI * 0.5f);
    V3 t0{-std::sin(a0), std::cos(a0), 0};
    V3 t1{-std::sin(a1), std::cos(a1), 0};
    return Cubic{p0, p0 + t0 * k, p3 - t1 * k, p3};
}

static void push_on_cubic(std::vector<Sample>& out, const Cubic& c, float t) {
    push_sample(out, bezier(c, t), bezier_d(c, t));
}

static std::vector<Sample> close_frames(const std::vector<Cubic>& spans, int per) {
    std::vector<Sample> in;
    for (const Cubic& c : spans)
        for (int i = 0; i < per; ++i) push_on_cubic(in, c, (float)i / (float)per);
    V3 n = in.back().N;
    V3 axis = cross(in.back().T, in.front().T);
    float axis_len = len(axis);
    if (axis_len < 1e-8f) {
        if (dot(in.back().T, in.front().T) < 0.f) n = cross(in.front().T, cross(in.back().T, n));
    } else {
        n = rot_about(n, norm(axis), std::atan2(axis_len, std::clamp(dot(in.back().T, in.front().T), -1.f, 1.f)));
    }
    n = n - in.front().T * dot(n, in.front().T);
    float alpha = std::atan2(dot(in.front().T, cross(in.front().N, norm(n))),
                              std::clamp(dot(in.front().N, norm(n)), -1.f, 1.f));
    float L = in.back().s + len(in.front().p - in.back().p);
    for (Sample& a : in) {
        a.N = norm(rot_about(a.N, a.T, -alpha * a.s / L));
        a.B = norm(cross(a.T, a.N));
    }
    return in;
}

static void pack(std::vector<float>& v, V3 p, V3 T, V3 normal) {
    V3 N = normal - T * dot(normal, T);
    if (len(N) < 1e-5f) {
        V3 up = std::fabs(T.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0};
        N = cross(up, T);
    }
    N = norm(N);
    V3 B = norm(cross(T, N));
    quatf q = mat3f::packTangentFrame(mat3f{float3{T.x, T.y, T.z}, float3{B.x, B.y, B.z}, float3{N.x, N.y, N.z}});
    float f[7] = {p.x, p.y, p.z, q.x, q.y, q.z, q.w};
    v.insert(v.end(), f, f + 7);
}

static Built tube(const std::vector<Sample>& s, int sides, float radius, bool closed, bool zero_pos) {
    Built m;
    int rings = (int)s.size();
    for (int i = 0; i < rings; ++i) {
        for (int k = 0; k < sides; ++k) {
            float ang = (float)k / (float)sides * TAU;
            V3 radial = norm(s[i].N * std::cos(ang) + s[i].B * std::sin(ang));
            pack(m.v, zero_pos ? V3{0, 0, 0} : s[i].p + radial * radius, s[i].T, radial);
        }
    }
    int links = closed ? rings : rings - 1;
    for (int i = 0; i < links; ++i) {
        int i1 = (i + 1) % rings;
        for (int k = 0; k < sides; ++k) {
            int k1 = (k + 1) % sides;
            uint32_t a = (uint32_t)(i * sides + k), b = (uint32_t)(i * sides + k1);
            uint32_t c = (uint32_t)(i1 * sides + k), d = (uint32_t)(i1 * sides + k1);
            m.idx.insert(m.idx.end(), {a, b, c, b, d, c});
        }
    }
    return m;
}

static Built quad(V3 center, V3 right, V3 up, float hw, float hh) {
    Built m;
    V3 n = norm(cross(right, up));
    V3 p[4] = {center - right * hw - up * hh, center + right * hw - up * hh, center + right * hw + up * hh,
               center - right * hw + up * hh};
    for (int i = 0; i < 4; ++i) pack(m.v, p[i], norm(right), n);
    m.idx = {0, 1, 2, 0, 2, 3};
    return m;
}

static float gap(const std::vector<Sample>& a, const std::vector<Sample>& b) {
    float m = 1e9f;
    for (const Sample& p : a)
        for (const Sample& q : b) m = std::min(m, len(p.p - q.p));
    return m;
}

static std::vector<uint8_t> load_file(const char* path) {
    std::ifstream in(path, std::ios::binary);
    return std::vector<uint8_t>((std::istreambuf_iterator<char>(in)), {});
}
static bool load_sh(const char* path, float3 sh[9]) {
    std::setlocale(LC_NUMERIC, "C");
    std::ifstream in(path);
    if (!in) return false;
    std::string line;
    for (int i = 0; i < 9; ++i) {
        if (!std::getline(in, line)) return false;
        float a, b, c;
        if (std::sscanf(line.c_str(), " (%f , %f , %f )", &a, &b, &c) != 3) return false;
        sh[i] = float3{a, b, c};
    }
    return true;
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
static int fail(const char* msg) {
    std::printf("FAIL %s\n", msg);
    return 1;
}

struct Hold {
    Built mesh;
    utils::Entity entity = {};
    VertexBuffer* vb = nullptr;
    IndexBuffer* ib = nullptr;
};


#include "fresh_2.h"
