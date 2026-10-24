// Transmission caustics on the hero tubes.
// Filament still draws the glass, the shadows, and the grade.
// This file only builds the missing light: sun -> tube -> floor, per wavelength.
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
#include <filament/TransformManager.h>
#include <filament/Texture.h>
#include <filament/TextureSampler.h>
#include <filament/ToneMapper.h>
#include <filament/VertexBuffer.h>
#include <filament/View.h>
#include <filament/Viewport.h>

#include <image/Ktx1Bundle.h>
#include <ktxreader/Ktx1Reader.h>

#include <math/mat3.h>
#include <math/mat4.h>
#include <math/vec3.h>
#include <math/vec4.h>

#include <utils/EntityManager.h>

#include <zlib.h>

#include <algorithm>
#include <chrono>
#include <clocale>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <string>
#include <unordered_map>
#include <vector>

using namespace filament;
using namespace filament::math;

static const int W = 1920;
static const int H = 1080;
static const int TEX = 512;
static const float PI = 3.14159265f;
static const float TAU = PI * 2.f;
static const float FLOOR_Y = -0.64f;
static const float FLOOR_HW = 2.8f;
static const float FLOOR_HH = 2.2f;
static const float FLOOR_CZ = 0.15f;

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
    return L > 1e-8f ? a * (1.f / L) : V3{0, 1, 0};
}
struct Cubic { V3 p[4]; };
struct Sample { V3 p, T, N, B; float s = 0; };
struct Built { std::vector<float> v; std::vector<uint32_t> idx; };

static V3 bezier(const Cubic& c, float t) {
    float u = 1.f - t;
    return c.p[0] * (u * u * u) + c.p[1] * (3.f * u * u * t) + c.p[2] * (3.f * u * t * t) + c.p[3] * (t * t * t);
}
static V3 bezier_d(const Cubic& c, float t) {
    float u = 1.f - t;
    return (c.p[1] - c.p[0]) * (3.f * u * u) + (c.p[2] - c.p[1]) * (6.f * u * t) + (c.p[3] - c.p[2]) * (3.f * t * t);
}
static V3 rot_about(V3 v, V3 T, float ang) { return v * std::cos(ang) + cross(T, v) * std::sin(ang); }
static void rotX(V3& v, float ca, float sa) {
    float y = v.y * ca - v.z * sa, z = v.y * sa + v.z * ca;
    v.y = y;
    v.z = z;
}
static V3 limit_tangent(const Cubic& c, float t) {
    V3 d = bezier_d(c, t);
    if (len(d) >= 1e-4f) return d;
    return c.p[3] - c.p[0];
}
static std::vector<Cubic> catmull(const std::vector<V3>& p) {
    std::vector<Cubic> out;
    int n = (int)p.size();
    auto at = [&](int i) { return p[(i % n + n) % n]; };
    for (int i = 0; i < n; ++i) {
        V3 p0 = at(i), p3 = at(i + 1);
        V3 p1 = p0 + (at(i + 1) - at(i - 1)) * (1.f / 6.f);
        V3 p2 = p3 - (at(i + 2) - at(i)) * (1.f / 6.f);
        out.push_back(Cubic{p0, p1, p2, p3});
    }
    return out;
}
static std::vector<Cubic> the_path() {
    return catmull({{-0.78f, -0.22f, 0.12f}, {-0.22f, -0.48f, 0.38f}, {0.55f, -0.12f, 0.22f},
                    {0.42f, 0.46f, -0.22f}, {-0.18f, 0.38f, -0.48f}, {-0.68f, 0.08f, -0.12f}});
}
static void push_sample(std::vector<Sample>& out, const Cubic& c, float t) {
    Sample s;
    bool has = !out.empty();
    Sample prev = has ? out.back() : Sample{};
    s.p = bezier(c, t);
    s.T = norm(limit_tangent(c, t));
    if (len(s.T) < 0.5f) s.T = has ? prev.T : V3{1, 0, 0};
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
    V3 n = in.back().N;
    V3 axis = cross(in.back().T, in.front().T);
    float al = len(axis);
    if (al < 1e-8f) {
        if (dot(in.back().T, in.front().T) < 0.f) n = cross(in.front().T, cross(in.back().T, n));
    } else n = rot_about(n, norm(axis), std::atan2(al, std::clamp(dot(in.back().T, in.front().T), -1.f, 1.f)));
    n = norm(n - in.front().T * dot(n, in.front().T));
    float alpha = std::atan2(dot(in.front().T, cross(in.front().N, n)), std::clamp(dot(in.front().N, n), -1.f, 1.f));
    float L = in.back().s + len(in.front().p - in.back().p);
    for (Sample& a : in) {
        a.N = norm(rot_about(a.N, a.T, -alpha * a.s / L));
        a.B = norm(cross(a.T, a.N));
    }
    return in;
}
static V3 centroid(const std::vector<Sample>& s) {
    V3 c{0, 0, 0};
    for (const Sample& a : s) c = c + a.p;
    return c * (1.f / (float)s.size());
}
static std::vector<Sample> scaled(std::vector<Sample> s, float k, V3 shift) {
    V3 c = centroid(s);
    for (Sample& a : s) a.p = c + (a.p - c) * k + shift;
    return s;
}
static std::vector<Sample> tilted(std::vector<Sample> s, float ang) {
    V3 c = centroid(s);
    float ca = std::cos(ang), sa = std::sin(ang);
    for (Sample& a : s) {
        a.p = a.p - c;
        rotX(a.p, ca, sa);
        a.p = a.p + c;
        rotX(a.T, ca, sa);
        rotX(a.N, ca, sa);
        rotX(a.B, ca, sa);
        a.T = norm(a.T);
        a.N = norm(a.N);
        a.B = norm(a.B);
    }
    return s;
}
static std::vector<Sample> nudged(std::vector<Sample> s, float d) {
    for (Sample& a : s) a.p = a.p + a.N * d;
    return s;
}
static void pack(std::vector<float>& v, V3 p, V3 T, V3 normal) {
    V3 N = normal - T * dot(normal, T);
    if (len(N) < 1e-5f) N = cross(std::fabs(T.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0}, T);
    N = norm(N);
    V3 B = norm(cross(T, N));
    quatf q = mat3f::packTangentFrame(mat3f{float3{T.x, T.y, T.z}, float3{B.x, B.y, B.z}, float3{N.x, N.y, N.z}});
    float f[7] = {p.x, p.y, p.z, q.x, q.y, q.z, q.w};
    v.insert(v.end(), f, f + 7);
}
static Built tube(const std::vector<Sample>& s, int sides, float radius) {
    Built m;
    int rings = (int)s.size();
    for (int i = 0; i < rings; ++i)
        for (int k = 0; k < sides; ++k) {
            float ang = (float)k / (float)sides * TAU;
            V3 radial = norm(s[i].N * std::cos(ang) + s[i].B * std::sin(ang));
            pack(m.v, s[i].p + radial * radius, s[i].T, radial);
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
static Built quad_uv(V3 center, V3 right, V3 up, float hw, float hh, std::vector<float>& uv_pack) {
    Built m;
    V3 n = norm(cross(right, up));
    V3 p[4] = {center - right * hw - up * hh, center + right * hw - up * hh, center + right * hw + up * hh,
               center - right * hw + up * hh};
    float uv[4][2] = {{0, 0}, {1, 0}, {1, 1}, {0, 1}};
    V3 T = norm(right);
    V3 N = norm(n - T * dot(n, T));
    V3 B = norm(cross(T, N));
    quatf q = mat3f::packTangentFrame(mat3f{float3{T.x, T.y, T.z}, float3{B.x, B.y, B.z}, float3{N.x, N.y, N.z}});
    uv_pack.resize(4 * 9);
    for (int i = 0; i < 4; ++i) {
        float* o = uv_pack.data() + i * 9;
        o[0] = p[i].x;
        o[1] = p[i].y;
        o[2] = p[i].z;
        o[3] = uv[i][0];
        o[4] = uv[i][1];
        o[5] = q.x;
        o[6] = q.y;
        o[7] = q.z;
        o[8] = q.w;
    }
    m.idx = {0, 1, 2, 0, 2, 3};
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
static int fail(const char* m) {
    std::printf("FAIL %s\n", m);
    return 1;
}

struct Glass {
    std::vector<Sample> s;
    float radius = 0.05f;
    float ior = 1.5f;
    float dispersion = 0.f;
    float absorb[3] = {0.02f, 0.02f, 0.02f};
};

struct Seg {
    V3 a, b, ab, ax;
    float len = 0, rad = 0, ior = 1.5f, disp = 0;
    float absorb[3] = {};
    int tube = 0;
};

struct Field {
    float cell = 0.07f;
    std::vector<Seg> segs;
    std::unordered_map<int, std::vector<int>> bins;
    static int key(int x, int y, int z) { return ((x & 1023) << 20) ^ ((y & 1023) << 10) ^ (z & 1023); }
    int cell_of(float v) const { return (int)std::floor(v / cell); }
    void build(const std::vector<Glass>& tubes) {
        segs.clear();
        bins.clear();
        for (int ti = 0; ti < (int)tubes.size(); ++ti) {
            const Glass& g = tubes[ti];
            int n = (int)g.s.size();
            for (int i = 0; i < n; ++i) {
                Seg s;
                s.a = g.s[i].p;
                s.b = g.s[(i + 1) % n].p;
                s.ab = s.b - s.a;
                s.len = len(s.ab);
                if (s.len < 1e-5f) continue;
                s.ax = s.ab * (1.f / s.len);
                s.rad = g.radius;
                s.ior = g.ior;
                s.disp = g.dispersion;
                s.absorb[0] = g.absorb[0];
                s.absorb[1] = g.absorb[1];
                s.absorb[2] = g.absorb[2];
                s.tube = ti;
                int id = (int)segs.size();
                segs.push_back(s);
                float reach = g.radius + cell;
                V3 mn{std::min(s.a.x, s.b.x) - reach, std::min(s.a.y, s.b.y) - reach, std::min(s.a.z, s.b.z) - reach};
                V3 mx{std::max(s.a.x, s.b.x) + reach, std::max(s.a.y, s.b.y) + reach, std::max(s.a.z, s.b.z) + reach};
                for (int z = cell_of(mn.z); z <= cell_of(mx.z); ++z)
                    for (int y = cell_of(mn.y); y <= cell_of(mx.y); ++y)
                        for (int x = cell_of(mn.x); x <= cell_of(mx.x); ++x) bins[key(x, y, z)].push_back(id);
            }
        }
    }
    float sample(V3 p, int only, V3& axis, float& rad, int& tube, float absorb[3], float& ior, float& disp) const {
        float best = 1e9f;
        int cx = cell_of(p.x), cy = cell_of(p.y), cz = cell_of(p.z);
        for (int dz = -1; dz <= 1; ++dz)
            for (int dy = -1; dy <= 1; ++dy)
                for (int dx = -1; dx <= 1; ++dx) {
                    auto it = bins.find(key(cx + dx, cy + dy, cz + dz));
                    if (it == bins.end()) continue;
                    for (int id : it->second) {
                        const Seg& s = segs[id];
                        if (only >= 0 && s.tube != only) continue;
                        float u = std::clamp(dot(p - s.a, s.ax), 0.f, s.len);
                        V3 q = s.a + s.ax * u;
                        float d = len(p - q);
                        if (d < best) {
                            best = d;
                            axis = q;
                            rad = s.rad;
                            tube = s.tube;
                            absorb[0] = s.absorb[0];
                            absorb[1] = s.absorb[1];
                            absorb[2] = s.absorb[2];
                            ior = s.ior;
                            disp = s.disp;
                        }
                    }
                }
        return best;
    }
};

static bool refract(V3 I, V3 N, float eta, V3& T) {
    float cosi = std::clamp(-dot(I, N), -1.f, 1.f);
    float k = 1.f - eta * eta * (1.f - cosi * cosi);
    if (k < 0.f) return false;
    T = norm(I * eta + N * (eta * cosi - std::sqrt(k)));
    return true;
}
static float capsule_enter(V3 ro, V3 rd, const Seg& s) {
    V3 ba = s.ab;
    V3 oa = ro - s.a;
    float baba = dot(ba, ba);
    float bard = dot(ba, rd);
    float baoa = dot(ba, oa);
    float rdoa = dot(rd, oa);
    float oaoa = dot(oa, oa);
    float a = baba - bard * bard;
    float b = baba * rdoa - baoa * bard;
    float c = baba * oaoa - baoa * baoa - s.rad * s.rad * baba;
    float h = b * b - a * c;
    if (h < 0.f) return -1.f;
    float t = a > 1e-6f ? (-b - std::sqrt(h)) / a : 1e9f;
    float y = baoa + t * bard;
    if (y > 0.f && y < baba) return t;
    V3 oc = (y <= 0.f) ? oa : ro - s.b;
    b = dot(rd, oc);
    c = dot(oc, oc) - s.rad * s.rad;
    h = b * b - c;
    if (h > 0.f) return -b - std::sqrt(h);
    return -1.f;
}

static float fresnel(float cosi, float n1, float n2) {
    float r0 = (n1 - n2) / (n1 + n2);
    r0 *= r0;
    float c = std::clamp(1.f - cosi, 0.f, 1.f);
    return r0 + (1.f - r0) * c * c * c * c * c;
}

static const float kOff[4] = {0.70795215f, 0.24790980f, 0.f, -0.29204785f};
static const float kW[4][3] = {{-0.5662f, 0.1812f, 1.1104f}, {-0.0643f, 0.7084f, -0.0637f},
                               {1.3154f, 0.1486f, -0.0451f}, {0.3152f, -0.0383f, -0.0016f}};

struct TraceStats {
    int hit = 0, floor = 0;
    double ms = 0;
    float sum = 0, maxv = 0;
    float cx_r = 0, cx_b = 0, sep = 0, chroma = 0;
};

static void floor_uv(V3 p, float& u, float& v) {
    u = (p.x / FLOOR_HW + 1.f) * 0.5f;
    v = ((FLOOR_CZ - p.z) / FLOOR_HH + 1.f) * 0.5f;
}


#include "caustics_2.h"
#include "caustics_3.h"
