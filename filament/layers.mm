// Nested screen-space refraction.
// Filament copies the opaque color once per render() and every refractive
// object in that render samples the copy. Each glass is therefore its own
// render(), back to front. The previous color is an opaque plate, so the
// existing solid refraction shader (IOR, thickness, dispersion, absorption)
// bends an image that already contains the glasses behind it.
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
#include <filament/TextureSampler.h>
#include <filament/TransformManager.h>
#include <filament/VertexBuffer.h>
#include <filament/View.h>
#include <filament/Viewport.h>

#include <math/mat3.h>
#include <math/mat4.h>
#include <math/vec3.h>
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
static V3 cross(V3 a, V3 b) { return {a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x}; }
static float len(V3 a) { return std::sqrt(dot(a, a)); }
static V3 norm(V3 a) {
    float L = len(a);
    return L > 1e-8f ? a * (1.f / L) : V3{0, 0, 0};
}
static void rotY(V3& p, float a) {
    float c = std::cos(a), s = std::sin(a);
    float x = p.x * c + p.z * s;
    float z = -p.x * s + p.z * c;
    p.x = x;
    p.z = z;
}

struct Cubic {
    V3 p[4];
};
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
static V3 rot_about(V3 v, V3 T, float ang) { return v * std::cos(ang) + cross(T, v) * std::sin(ang); }
static V3 transport_n(V3 n, V3 t_from, V3 t_to) {
    V3 axis = cross(t_from, t_to);
    float axis_len = len(axis);
    if (axis_len < 1e-8f) {
        if (dot(t_from, t_to) < 0.f) n = cross(t_to, cross(t_from, n));
    } else {
        n = rot_about(n, norm(axis), std::atan2(axis_len, std::clamp(dot(t_from, t_to), -1.f, 1.f)));
    }
    return n - t_to * dot(n, t_to);
}
static Cubic arc(float R, float a0, float a1) {
    V3 p0{R * std::cos(a0), R * std::sin(a0), 0};
    V3 p3{R * std::cos(a1), R * std::sin(a1), 0};
    float k = R * 0.55228475f * (a1 - a0) / (PI * 0.5f);
    V3 t0{-std::sin(a0), std::cos(a0), 0};
    V3 t1{-std::sin(a1), std::cos(a1), 0};
    return Cubic{p0, p0 + t0 * k, p3 - t1 * k, p3};
}
static std::vector<Cubic> circle(float R, float tiltY) {
    std::vector<Cubic> c;
    for (int i = 0; i < 4; ++i) c.push_back(arc(R, i * PI * 0.5f, (i + 1) * PI * 0.5f));
    for (Cubic& s : c)
        for (int k = 0; k < 4; ++k) rotY(s.p[k], tiltY);
    return c;
}
static void push_sample(std::vector<Sample>& out, const Cubic& c, float t) {
    Sample s;
    bool has = !out.empty();
    Sample prev = has ? out.back() : Sample{};
    V3 d = bezier_d(c, t);
    s.p = bezier(c, t);
    s.T = norm(len(d) > 1e-5f ? d : (has ? prev.T : V3{1, 0, 0}));
    V3 up = std::fabs(s.T.z) < 0.9f ? V3{0, 0, 1} : V3{0, 1, 0};
    V3 n = has ? prev.N - s.T * dot(prev.N, s.T) : cross(up, s.T);
    if (len(n) < 1e-4f) n = cross(up, s.T);
    s.N = norm(n);
    s.B = norm(cross(s.T, s.N));
    s.s = has ? prev.s + len(s.p - prev.p) : 0;
    out.push_back(s);
}
static std::vector<Sample> close_frames(const std::vector<Cubic>& spans, int per) {
    std::vector<Sample> in;
    for (const Cubic& c : spans)
        for (int i = 0; i < per; ++i) push_sample(in, c, (float)i / (float)per);
    V3 arrived = transport_n(in.back().N, in.back().T, in.front().T);
    float alpha = std::atan2(dot(in.front().T, cross(in.front().N, norm(arrived))),
                              std::clamp(dot(in.front().N, norm(arrived)), -1.f, 1.f));
    float L = in.back().s + len(in.front().p - in.back().p);
    for (Sample& a : in) {
        a.N = norm(rot_about(a.N, a.T, -alpha * a.s / L));
        a.B = norm(cross(a.T, a.N));
    }
    return in;
}
static void pack(std::vector<float>& v, V3 p, V3 T, V3 normal) {
    V3 N = norm(normal - T * dot(normal, T));
    if (len(N) < 0.5f) N = norm(cross(std::fabs(T.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0}, T));
    V3 B = norm(cross(T, N));
    quatf q = mat3f::packTangentFrame(mat3f{float3{T.x, T.y, T.z}, float3{B.x, B.y, B.z}, float3{N.x, N.y, N.z}});
    float f[7] = {p.x, p.y, p.z, q.x, q.y, q.z, q.w};
    v.insert(v.end(), f, f + 7);
}
static Built tube(const std::vector<Sample>& s, int sides, float radius) {
    Built m;
    int rings = (int)s.size();
    for (int i = 0; i < rings; ++i) {
        for (int k = 0; k < sides; ++k) {
            float ang = (float)k / (float)sides * TAU;
            V3 radial = norm(s[i].N * std::cos(ang) + s[i].B * std::sin(ang));
            pack(m.v, s[i].p + radial * radius, s[i].T, radial);
        }
    }
    for (int i = 0; i < rings; ++i) {
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
static Built quad_bars(V3 c, float hw, float hh) {
    Built m;
    V3 p[4] = {{c.x - hw, c.y - hh, c.z}, {c.x + hw, c.y - hh, c.z}, {c.x + hw, c.y + hh, c.z}, {c.x - hw, c.y + hh, c.z}};
    for (int i = 0; i < 4; ++i) pack(m.v, p[i], {1, 0, 0}, {0, 0, 1});
    m.idx = {0, 1, 2, 0, 2, 3};
    return m;
}
static Built sphere(float radius, int stacks, int slices) {
    Built m;
    for (int iy = 0; iy <= stacks; ++iy) {
        float v = (float)iy / (float)stacks;
        float phi = v * PI;
        for (int ix = 0; ix <= slices; ++ix) {
            float u = (float)ix / (float)slices;
            float th = u * TAU;
            V3 n{std::sin(phi) * std::cos(th), std::cos(phi), std::sin(phi) * std::sin(th)};
            V3 t{-std::sin(th), 0, std::cos(th)};
            pack(m.v, n * radius, len(t) < 1e-4f ? V3{1, 0, 0} : norm(t), n);
        }
    }
    int stride = slices + 1;
    for (int iy = 0; iy < stacks; ++iy) {
        for (int ix = 0; ix < slices; ++ix) {
            uint32_t a = (uint32_t)(iy * stride + ix);
            uint32_t b = a + 1;
            uint32_t c = a + (uint32_t)stride;
            uint32_t d = c + 1;
            m.idx.insert(m.idx.end(), {a, c, b, b, c, d});
        }
    }
    return m;
}

static std::vector<uint8_t> load_file(const char* path) {
    std::ifstream in(path, std::ios::binary);
    return std::vector<uint8_t>((std::istreambuf_iterator<char>(in)), {});
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

struct Hold {
    Built mesh;
    utils::Entity entity = {};
    VertexBuffer* vb = nullptr;
    IndexBuffer* ib = nullptr;
};

static int fail(const char* m) {
    std::printf("FAIL %s\n", m);
    return 1;
}

struct Eye {
    float3 eye;
    float3 target;
};

static float half_to_float(uint16_t h) {
    uint32_t s = (uint32_t)(h >> 15) & 1u;
    int e = (h >> 10) & 0x1f;
    uint32_t m = h & 0x3ff;
    uint32_t bits;
    if (e == 0) {
        if (m == 0) bits = s << 31;
        else {
            while ((m & 0x400) == 0) {
                m <<= 1;
                e--;
            }
            m &= 0x3ff;
            e = e + 1 + (127 - 15);
            bits = (s << 31) | ((uint32_t)e << 23) | (m << 13);
        }
    } else if (e == 31) {
        bits = (s << 31) | 0x7f800000u | (m << 13);
    } else {
        bits = (s << 31) | ((uint32_t)(e + (127 - 15)) << 23) | (m << 13);
    }
    float f;
    std::memcpy(&f, &bits, 4);
    return f;
}


#include "layers_2.h"
