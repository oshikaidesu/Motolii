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

static TraceStats trace(const Field& field, V3 sun_dir, float lux, bool same_ior, std::vector<float>& tex) {
    TraceStats st;
    tex.assign((size_t)TEX * TEX * 4, 0.f);
    if (field.segs.empty() || lux <= 0.f) return st;
    auto t0 = std::chrono::steady_clock::now();
    V3 D = norm(sun_dir);
    V3 up = std::fabs(D.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0};
    V3 U = norm(cross(up, D));
    V3 V = cross(D, U);
    V3 C{0, 0, 0};
    for (const Seg& s : field.segs) C = C + s.a;
    C = C * (1.f / (float)field.segs.size());
    const int G = 200;
    const float extent = 1.15f;
    const float cell = (2.f * extent) / (float)G;
    const float texel_area = (FLOOR_HW * 2.f * FLOOR_HH * 2.f) / (float)(TEX * TEX);
    const float flux = lux * cell * cell / texel_area;
    int hits = 0, floors = 0;
    for (int j = 0; j < G; ++j) {
        for (int i = 0; i < G; ++i) {
            float su = -extent + (i + 0.5f) * cell;
            float sv = -extent + (j + 0.5f) * cell;
            V3 ro = C - D * 2.4f + U * su + V * sv;
            for (int w = 0; w < 4; ++w) {
                V3 o = ro, d = D;
                float transmit = 1.f;
                bool live = false;
                for (int pass = 0; pass < 3; ++pass) {
                    float best = 1e9f;
                    int best_seg = -1;
                    for (int si = 0; si < (int)field.segs.size(); ++si) {
                        const Seg& s = field.segs[si];
                        V3 mid = (s.a + s.b) * 0.5f;
                        V3 w = mid - o;
                        float along = dot(w, d);
                        if (along < -(s.rad + s.len) || along > 6.f) continue;
                        V3 perp = w - d * along;
                        if (dot(perp, perp) > (s.rad + s.len * 0.5f + 0.02f) * (s.rad + s.len * 0.5f + 0.02f)) continue;
                        float t = capsule_enter(o, d, s);
                        if (t > 0.006f && t < best) {
                            best = t;
                            best_seg = si;
                        }
                    }
                    if (best_seg < 0) break;
                    const Seg& hit = field.segs[best_seg];
                    V3 hp = o + d * best;
                    float u = std::clamp(dot(hp - hit.a, hit.ax), 0.f, hit.len);
                    V3 axis = hit.a + hit.ax * u;
                    V3 hn = norm(hp - axis);
                    int tube = hit.tube;
                    float ior = hit.ior, disp = hit.disp;
                    float absorb[3] = {hit.absorb[0], hit.absorb[1], hit.absorb[2]};
                    if (dot(d, hn) > 0.f) hn = hn * -1.f;
                    float n = ior;
                    if (!same_ior) n = ior + (disp / 20.f) * (ior - 1.f) * kOff[w];
                    if (n < 1.01f) n = 1.01f;
                    float cosi = std::clamp(-dot(d, hn), 0.f, 1.f);
                    transmit *= 1.f - fresnel(cosi, 1.f, n);
                    V3 inside;
                    if (!refract(d, hn, 1.f / n, inside)) break;
                    auto sdf_tube = [&](V3 p, V3& axis) {
                        float best = 1e9f;
                        for (const Seg& s : field.segs) {
                            if (s.tube != tube) continue;
                            float uu = std::clamp(dot(p - s.a, s.ax), 0.f, s.len);
                            V3 q = s.a + s.ax * uu;
                            float dd = len(p - q) - s.rad;
                            if (dd < best) {
                                best = dd;
                                axis = q;
                            }
                        }
                        return best;
                    };
                    float lo = 0.f, hi = 0.008f;
                    if (sdf_tube(hp + inside * hi, axis) > 0.f) break;
                    for (int step = 0; step < 24 && sdf_tube(hp + inside * hi, axis) < 0.f; ++step) hi *= 1.45f;
                    if (sdf_tube(hp + inside * hi, axis) < 0.f) break;
                    for (int step = 0; step < 12; ++step) {
                        float mid = 0.5f * (lo + hi);
                        if (sdf_tube(hp + inside * mid, axis) < 0.f) lo = mid;
                        else hi = mid;
                    }
                    V3 xp = hp + inside * hi;
                    sdf_tube(xp, axis);
                    V3 xn = norm(xp - axis);
                    float path = len(xp - hp);
                    transmit *= std::exp(-(absorb[0] + absorb[1] + absorb[2]) * 0.15f * path);
                    if (dot(inside, xn) > 0.f) xn = xn * -1.f;
                    float cosi2 = std::clamp(-dot(inside, xn), 0.f, 1.f);
                    transmit *= 1.f - fresnel(cosi2, n, 1.f);
                    V3 out;
                    if (!refract(inside, xn, n, out)) break;
                    o = xp + out * 0.01f;
                    d = out;
                    live = true;
                    hits++;
                }
                if (!live || d.y >= -1e-4f || transmit < 1e-4f) continue;
                float tf = (FLOOR_Y - o.y) / d.y;
                if (tf < 0.f || tf > 8.f) continue;
                V3 hitp = o + d * tf;
                float u, v;
                floor_uv(hitp, u, v);
                if (u < 0.f || u > 1.f || v < 0.f || v > 1.f) continue;
                float x = u * (TEX - 1), y = v * (TEX - 1);
                const int rad = 8;
                const float sig2 = 2.f * 3.1f * 3.1f;
                float accw = 0.f;
                for (int dy = -rad; dy <= rad; ++dy)
                    for (int dx = -rad; dx <= rad; ++dx) accw += std::exp(-(dx * dx + dy * dy) / sig2);
                for (int dy = -rad; dy <= rad; ++dy)
                    for (int dx = -rad; dx <= rad; ++dx) {
                        int xx = (int)std::lround(x) + dx;
                        int yy = (int)std::lround(y) + dy;
                        if (xx < 0 || yy < 0 || xx >= TEX || yy >= TEX) continue;
                        float* p = tex.data() + ((size_t)yy * TEX + xx) * 4;
                        float e = flux * transmit * std::exp(-(dx * dx + dy * dy) / sig2) / accw;
                        p[0] += kW[w][0] * e;
                        p[1] += kW[w][1] * e;
                        p[2] += kW[w][2] * e;
                    }
                floors++;
            }
        }
    }
    float sr = 0, sb = 0, wr = 0, wb = 0, chroma = 0, lum = 0, sum = 0, maxv = 0;
    int nch = 0;
    for (int y = 0; y < TEX; ++y)
        for (int x = 0; x < TEX; ++x) {
            float* p = tex.data() + ((size_t)y * TEX + x) * 4;
            p[0] = std::max(p[0], 0.f);
            p[1] = std::max(p[1], 0.f);
            p[2] = std::max(p[2], 0.f);
            float m = std::max(p[0], std::max(p[1], p[2]));
            sum += m;
            maxv = std::max(maxv, m);
            if (m > 20.f) {
                chroma += std::fabs(p[0] - p[2]);
                lum += m;
                sr += p[0] * x;
                wr += p[0];
                sb += p[2] * x;
                wb += p[2];
                nch++;
            }
        }
    auto t1 = std::chrono::steady_clock::now();
    st.ms = std::chrono::duration<double, std::milli>(t1 - t0).count();
    st.hit = hits;
    st.floor = floors;
    st.sum = sum;
    st.maxv = maxv;
    st.cx_r = wr > 1.f ? sr / wr : 0;
    st.cx_b = wb > 1.f ? sb / wb : 0;
    st.sep = std::fabs(st.cx_r - st.cx_b);
    st.chroma = lum > 1.f ? chroma / lum : 0;
    std::printf("trace hits %d floor %d sum %.3e max %.1f sep %.2f chroma %.3f  %.1f ms\n", st.hit, st.floor, st.sum,
                st.maxv, st.sep, st.chroma, st.ms);
    return st;
}

struct Hold {
    utils::Entity entity{};
    VertexBuffer* vb = nullptr;
    IndexBuffer* ib = nullptr;
    Built mesh;
    std::vector<float> uv;
};

int main() {
    std::vector<Sample> path = close_frames(the_path(), 28);
    Glass back, mid, front;
    back.s = tilted(scaled(path, 1.22f, {0, 0.02f, -0.20f}), 0.62f);
    mid.s = tilted(path, -0.42f);
    front.s = tilted(scaled(path, 0.50f, {0, 0, 0.16f}), 0.22f);
    back.radius = 0.056f;
    mid.radius = 0.042f;
    front.radius = 0.026f;
    back.ior = 1.85f;
    mid.ior = 2.05f;
    front.ior = 2.20f;
    back.dispersion = 9.f;
    mid.dispersion = 13.f;
    front.dispersion = 11.f;
    back.absorb[0] = 0.02f;
    back.absorb[1] = 0.08f;
    back.absorb[2] = 0.55f;
    front.absorb[0] = 0.42f;
    front.absorb[1] = 0.04f;
    front.absorb[2] = 0.02f;
    std::vector<Sample> chrome_s = nudged(mid.s, 0.11f);

    Field field;
    field.build({back, mid, front});
    V3 sun0{0.12f, -0.28f, -1.0f};
    const float lux = 90000.f;
    std::vector<float> tex;
    TraceStats full = trace(field, sun0, lux, false, tex);

    Field none;
    std::vector<float> zero;
    TraceStats empty = trace(none, sun0, lux, false, zero);

    Glass plain = back;
    plain.dispersion = 0;
    Glass plain2 = mid;
    plain2.dispersion = 0;
    Glass plain3 = front;
    plain3.dispersion = 0;
    Field mono;
    mono.build({plain, plain2, plain3});
    std::vector<float> tex0;
    TraceStats same = trace(mono, sun0, lux, true, tex0);

    Glass moved_b = back, moved_m = mid, moved_f = front;
    for (Sample& a : moved_b.s) a.p.x += 0.35f;
    for (Sample& a : moved_m.s) a.p.x += 0.35f;
    for (Sample& a : moved_f.s) a.p.x += 0.35f;
    Field shifted;
    shifted.build({moved_b, moved_m, moved_f});
    std::vector<float> texm;
    TraceStats mv = trace(shifted, sun0, lux, false, texm);

    std::vector<float> texl;
    TraceStats lit = trace(field, {0.55f, -0.28f, -1.0f}, lux, false, texl);
    std::vector<float> tex2;
    TraceStats dbl = trace(field, sun0, lux * 2.f, false, tex2);

    std::printf("CHECK glass-off sum %.3e (want 0)\n", empty.sum);
    std::printf("CHECK same-ior sep %.3f chroma %.3f   dispersed sep %.3f chroma %.3f\n", same.sep, same.chroma, full.sep,
                full.chroma);
    std::printf("CHECK move-glass cxR %.1f -> %.1f\n", full.cx_r, mv.cx_r);
    std::printf("CHECK move-sun cxR %.1f -> %.1f\n", full.cx_r, lit.cx_r);
    std::printf("CHECK lux x2 sum ratio %.3f\n", full.sum > 0 ? dbl.sum / full.sum : 0);

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    utils::Entity cam_e = utils::EntityManager::get().create();
    Camera* camera = engine->createCamera(cam_e);
    camera->setExposure(16.0f, 1.0f / 125.0f, 100.0f);

    auto make_rt = [&](Texture::InternalFormat fmt) {
        Texture* color = Texture::Builder()
                             .width(W).height(H).levels(1)
                             .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                             .format(fmt).build(*engine);
        Texture* depth = Texture::Builder()
                             .width(W).height(H).levels(1)
                             .usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH24).build(*engine);
        RenderTarget* rt = RenderTarget::Builder()
                               .texture(RenderTarget::AttachmentPoint::COLOR, color)
                               .texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
        return std::tuple<Texture*, RenderTarget*>{color, rt};
    };
    auto [cA, rtA] = make_rt(Texture::InternalFormat::RGBA16F);
    auto [cB, rtB] = make_rt(Texture::InternalFormat::RGBA16F);
    auto [c8, rt8] = make_rt(Texture::InternalFormat::RGBA8);
    Texture* cf[2] = {cA, cB};
    RenderTarget* rf[2] = {rtA, rtB};

    Texture* cau = Texture::Builder()
                       .width(TEX).height(TEX).levels(1)
                       .sampler(Texture::Sampler::SAMPLER_2D)
                       .format(Texture::InternalFormat::RGBA16F).build(*engine);
    auto upload = [&](const std::vector<float>& src) {
        cau->setImage(*engine, 0,
                      Texture::PixelBufferDescriptor(src.data(), src.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                     backend::PixelDataType::FLOAT, 1, 0, 0, TEX));
    };
    upload(tex);

    auto ibl_bytes = load_file("out/ibl/ibl_ibl.ktx");
    auto sky_bytes = load_file("out/ibl/ibl_skybox.ktx");
    if (ibl_bytes.empty() || sky_bytes.empty()) return fail("ktx");
    Texture* ibl_tex = ktxreader::Ktx1Reader::createTexture(engine, new image::Ktx1Bundle(ibl_bytes.data(), (uint32_t)ibl_bytes.size()), false);
    Texture* sky_tex = ktxreader::Ktx1Reader::createTexture(engine, new image::Ktx1Bundle(sky_bytes.data(), (uint32_t)sky_bytes.size()), false);
    float3 sh[9];
    if (!load_sh("out/ibl/sh.txt", sh) || !ibl_tex || !sky_tex) return fail("ibl");
    scene->setIndirectLight(IndirectLight::Builder().reflections(ibl_tex).irradiance(3, sh).intensity(9000.f).build(*engine));
    Skybox* sky = Skybox::Builder().environment(sky_tex).showSun(false).build(*engine);
    Skybox* dark = Skybox::Builder().color({0.01f, 0.008f, 0.006f, 1}).build(*engine);

    static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
    view->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.06f).saturation(1.14f).build(*engine));
    view->setScene(scene);
    view->setCamera(camera);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setDithering(View::Dithering::NONE);
    view->setAntiAliasing(View::AntiAliasing::FXAA);
    view->setShadowType(View::ShadowType::VSM);
    VsmShadowOptions vsm;
    vsm.highPrecision = true;
    view->setVsmShadowOptions(vsm);
    ScreenSpaceReflectionsOptions ssr{};
    ssr.enabled = false;
    view->setScreenSpaceReflectionsOptions(ssr);

    auto gpkg = load_file("glass_shadow.filamat");
    auto fpkg = load_file("floor_recv.filamat");
    auto epkg = load_file("emissive.filamat");
    auto cpkg = load_file("chrome_aniso.filamat");
    auto ppkg = load_file("plate.filamat");
    auto lpkg = load_file("floor_lit.filamat");
    if (gpkg.empty() || fpkg.empty() || epkg.empty() || cpkg.empty() || ppkg.empty() || lpkg.empty()) return fail("filamat");
    Material* gmat = Material::Builder().package(gpkg.data(), gpkg.size()).build(*engine);
    Material* fmat = Material::Builder().package(fpkg.data(), fpkg.size()).build(*engine);
    Material* emat = Material::Builder().package(epkg.data(), epkg.size()).build(*engine);
    Material* cmat = Material::Builder().package(cpkg.data(), cpkg.size()).build(*engine);
    Material* pmat = Material::Builder().package(ppkg.data(), ppkg.size()).build(*engine);
    Material* lmat = Material::Builder().package(lpkg.data(), lpkg.size()).build(*engine);
    if (!gmat || !fmat || !emat || !cmat || !pmat || !lmat) return fail("material");

    TextureSampler lin(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
    lin.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    lin.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    MaterialInstance* floor_mi = fmat->createInstance();
    floor_mi->setParameter("color", float4{0.045f, 0.040f, 0.038f, 1});
    floor_mi->setParameter("roughness", 0.55f);
    floor_mi->setParameter("caustic", cau, lin);

    auto emit = [&](float r, float g, float b) {
        MaterialInstance* mi = emat->createInstance();
        mi->setParameter("color", float4{r, g, b, 1});
        return mi;
    };
    auto glass_mi = [&](const Glass& g, float alpha) {
        MaterialInstance* mi = gmat->createInstance();
        mi->setParameter("ior", g.ior);
        mi->setParameter("transmission", 1.f);
        mi->setParameter("thickness", 1.05f);
        mi->setParameter("dispersion", g.dispersion);
        mi->setParameter("reflectance", 0.42f);
        mi->setParameter("alpha", alpha);
        mi->setParameter("absorption", float3{g.absorb[0], g.absorb[1], g.absorb[2]});
        return mi;
    };
    MaterialInstance* window_mi = emit(220000.f, 86000.f, 22000.f);
    MaterialInstance* mullion_mi = lmat->createInstance();
    mullion_mi->setParameter("color", float4{0.012f, 0.008f, 0.006f, 1});
    mullion_mi->setParameter("roughness", 0.9f);
    MaterialInstance* wall_mi = lmat->createInstance();
    wall_mi->setParameter("color", float4{0.03f, 0.027f, 0.034f, 1});
    wall_mi->setParameter("roughness", 0.8f);
    MaterialInstance* chrome_mi = cmat->createInstance();
    chrome_mi->setParameter("anisotropy", 0.92f);
    chrome_mi->setParameter("roughness", 0.20f);

    auto add_floor = [&](MaterialInstance* mi) {
        Hold h;
        quad_uv({0, FLOOR_Y, FLOOR_CZ}, {1, 0, 0}, {0, 0, -1}, FLOOR_HW, FLOOR_HH, h.uv);
        h.mesh.idx = {0, 1, 2, 0, 2, 3};
        h.vb = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 36)
                   .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 36)
                   .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, 36)
                   .build(*engine);
        h.vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(h.uv.data(), h.uv.size() * sizeof(float)));
        h.ib = IndexBuffer::Builder().indexCount((uint32_t)h.mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        h.ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(h.mesh.idx.data(), h.mesh.idx.size() * sizeof(uint32_t)));
        h.entity = utils::EntityManager::get().create();
        RenderableManager::Builder(1)
            .boundingBox({{-8, -8, -8}, {8, 8, 8}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, h.vb, h.ib)
            .material(0, mi).culling(false).castShadows(false).receiveShadows(true)
            .build(*engine, h.entity);
        return h;
    };
    Hold floor_h = add_floor(floor_mi);

    auto add_solid = [&](Built mesh, MaterialInstance* mi, bool cast_s, bool recv) {
        Hold h;
        h.mesh = std::move(mesh);
        uint32_t nv = (uint32_t)(h.mesh.v.size() / 7);
        h.vb = VertexBuffer::Builder().vertexCount(nv).bufferCount(1)
                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                   .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                   .build(*engine);
        h.vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(h.mesh.v.data(), h.mesh.v.size() * sizeof(float)));
        h.ib = IndexBuffer::Builder().indexCount((uint32_t)h.mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        h.ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(h.mesh.idx.data(), h.mesh.idx.size() * sizeof(uint32_t)));
        h.entity = utils::EntityManager::get().create();
        RenderableManager::Builder(1)
            .boundingBox({{-8, -8, -8}, {8, 8, 8}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, h.vb, h.ib)
            .material(0, mi).culling(false).castShadows(cast_s).receiveShadows(recv)
            .build(*engine, h.entity);
        return h;
    };

    Hold window_h = add_solid(quad({-0.42f, 0.34f, -1.22f}, {1, 0, 0}, {0, 1, 0}, 1.15f, 1.05f), window_mi, false, false);
    Hold mull_h = add_solid(quad({-0.42f, 0.34f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 0.016f, 1.05f), mullion_mi, false, false);
    Hold mull2 = add_solid(quad({-0.42f, 0.82f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 1.15f, 0.014f), mullion_mi, false, false);
    Hold wall_h = add_solid(quad({0.0f, 0.15f, -1.35f}, {1, 0, 0}, {0, 1, 0}, 2.2f, 1.3f), wall_mi, false, true);
    Hold chrome_h = add_solid(tube(chrome_s, 28, 0.013f), chrome_mi, true, false);
    MaterialInstance* gb = glass_mi(back, 1.f);
    MaterialInstance* gm = glass_mi(mid, 1.f);
    MaterialInstance* gf = glass_mi(front, 1.f);
    MaterialInstance* gshadow = glass_mi(back, 0.35f);
    Hold hb = add_solid(tube(back.s, 32, back.radius), gb, true, false);
    Hold hm = add_solid(tube(mid.s, 32, mid.radius), gm, true, false);
    Hold hf = add_solid(tube(front.s, 32, front.radius), gf, true, false);

    // Shadow demo: one partial-alpha tube, floor, no caustic contribution.
    std::vector<Sample> demo;
    for (int i = 0; i < 48; ++i) {
        float a = (float)i / 48.f * TAU;
        Sample s;
        s.p = {0.42f * std::cos(a), 0.05f, 0.42f * std::sin(a)};
        s.T = norm({-std::sin(a), 0, std::cos(a)});
        s.N = norm(s.p);
        s.B = norm(cross(s.T, s.N));
        demo.push_back(s);
    }
    Hold demo_h = add_solid(tube(demo, 28, 0.07f), gshadow, true, false);

    utils::Entity sun = utils::EntityManager::get().create();
    LightManager::ShadowOptions shopt;
    shopt.mapSize = 2048;
    shopt.vsm.blurWidth = 16.f;
    shopt.vsm.elvsm = true;
    LightManager::Builder(LightManager::Type::SUN)
        .color(float3{1, 0.96f, 0.9f}).intensity(lux).direction(normalize(float3{sun0.x, sun0.y, sun0.z}))
        .sunAngularRadius(0.55f).sunHaloSize(1.4f).castShadows(true).shadowOptions(shopt)
        .build(*engine, sun);
    scene->addEntity(sun);

    std::vector<utils::Entity> shown;
    auto set_shown = [&](const std::vector<utils::Entity>& next) {
        for (utils::Entity e : shown) scene->remove(e);
        shown = next;
        for (utils::Entity e : shown) scene->addEntity(e);
    };
    auto open_frame = [&]() {
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) return true;
        return false;
    };
    std::vector<uint8_t> pic((size_t)W * H * 4);
    auto grab = [&](RenderTarget* rt, const char* path) {
        if (!open_frame()) return false;
        view->setRenderTarget(rt);
        renderer->render(view);
        renderer->readPixels(rt, 0, 0, W, H,
                             backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA,
                                                            backend::PixelDataType::UBYTE));
        renderer->endFrame();
        engine->flushAndWait();
        if (!write_png(path, W, H, pic.data())) return false;
        std::printf("wrote %s\n", path);
        return true;
    };

    view->setPostProcessingEnabled(true);
    view->setScreenSpaceRefractionEnabled(true);
    auto& lm0 = engine->getLightManager();
    lm0.setDirection(lm0.getInstance(sun), normalize(float3{0.35f, -1.f, 0.25f}));
    engine->getTransformManager().setTransform(engine->getTransformManager().getInstance(demo_h.entity),
                                                mat4f::translation(float3{0, 0.45f, 0}));
    floor_mi->setParameter("color", float4{0.22f, 0.20f, 0.18f, 1});
    camera->lookAt({1.35f, 0.72f, 1.55f}, {0.05f, -0.35f, 0}, {0, 1, 0});
    camera->setProjection(28.0, double(W) / double(H), 0.05, 30.0, Camera::Fov::VERTICAL);
    std::vector<float> black((size_t)TEX * TEX * 4, 0.f);
    upload(black);
    scene->setSkybox(dark);
    set_shown({floor_h.entity, demo_h.entity});
    auto tA0 = std::chrono::steady_clock::now();
    if (!grab(rt8, "out/caustics-shadow.png")) return fail("shadow");
    auto tA1 = std::chrono::steady_clock::now();
    std::printf("shadow frame %.1f ms (flushAndWait, includes readPixels)\n",
                std::chrono::duration<double, std::milli>(tA1 - tA0).count());
    lm0.setDirection(lm0.getInstance(sun), normalize(float3{sun0.x, sun0.y, sun0.z}));
    floor_mi->setParameter("color", float4{0.045f, 0.040f, 0.038f, 1});

    // Caustic only, seen from above so the floor pattern is the picture.
    upload(tex);
    camera->lookAt({0.0f, 1.55f, 0.15f}, {0, FLOOR_Y, FLOOR_CZ}, {0, 0, -1});
    camera->setProjection(48.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);
    scene->remove(sun);
    set_shown({floor_h.entity});
    if (!grab(rt8, "out/caustics-only.png")) return fail("only");
    scene->addEntity(sun);

    // Hero camera, caustics off then on. Opaque room + three refractive passes.
    const float3 eye{0.18f, 0.40f, 1.82f};
    const float3 target{0, 0.02f, 0};
    camera->lookAt(eye, target, {0, 1, 0});
    camera->setProjection(34.0, double(W) / double(H), 0.05, 40.0, Camera::Fov::VERTICAL);

    MaterialInstance* plate_mi[2] = {pmat->createInstance(), pmat->createInstance()};
    plate_mi[0]->setParameter("color", cf[0], lin);
    plate_mi[1]->setParameter("color", cf[1], lin);
    Hold plate;
    std::vector<float> plate_v(4 * 9, 0.f);
    std::vector<uint32_t> plate_idx = {0, 1, 2, 0, 2, 3};
    plate.vb = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 36)
                   .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 36)
                   .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, 36)
                   .build(*engine);
    plate.ib = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    plate.ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(plate_idx.data(), plate_idx.size() * sizeof(uint32_t)));
    plate.entity = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-30, -30, -30}, {30, 30, 30}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, plate.vb, plate.ib)
        .material(0, plate_mi[0]).culling(false).castShadows(false).receiveShadows(false)
        .build(*engine, plate.entity);
    {
        float3 f = normalize(target - eye);
        float3 r = normalize(cross(f, float3{0, 1, 0}));
        float3 u = cross(r, f);
        float dist = 8.f;
        float hh = dist * std::tan(17.f * PI / 180.f);
        float hw = hh * ((float)W / (float)H);
        float3 c = eye + f * dist;
        float3 corner[4] = {c - r * hw - u * hh, c + r * hw - u * hh, c + r * hw + u * hh, c - r * hw + u * hh};
        float uv[4][2] = {{0, 0}, {1, 0}, {1, 1}, {0, 1}};
        quatf q = mat3f::packTangentFrame(mat3f{r, normalize(cross(r, f)), f});
        for (int i = 0; i < 4; ++i) {
            float* o = plate_v.data() + i * 9;
            o[0] = corner[i].x; o[1] = corner[i].y; o[2] = corner[i].z;
            o[3] = uv[i][0]; o[4] = uv[i][1];
            o[5] = q.x; o[6] = q.y; o[7] = q.z; o[8] = q.w;
        }
        plate.vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(plate_v.data(), plate_v.size() * sizeof(float)));
    }

    utils::Entity glasses[3] = {hb.entity, hm.entity, hf.entity};
    std::vector<utils::Entity> opaque = {floor_h.entity, wall_h.entity, window_h.entity, mull_h.entity, mull2.entity, chrome_h.entity};
    auto& rm = engine->getRenderableManager();
    BloomOptions bloom{};
    bloom.enabled = false;
    auto shoot = [&](const char* path) {
        if (!open_frame()) return false;
        auto t0 = std::chrono::steady_clock::now();
        bloom.enabled = false;
        view->setBloomOptions(bloom);
        view->setPostProcessingEnabled(false);
        view->setScreenSpaceRefractionEnabled(false);
        scene->setSkybox(sky);
        set_shown(opaque);
        view->setRenderTarget(rf[0]);
        renderer->render(view);
        scene->setSkybox(nullptr);
        view->setScreenSpaceRefractionEnabled(true);
        int src = 0;
        for (int g = 0; g < 3; ++g) {
            int dst = 1 - src;
            rm.setMaterialInstanceAt(rm.getInstance(plate.entity), 0, plate_mi[src]);
            set_shown({plate.entity, glasses[g]});
            view->setRenderTarget(rf[dst]);
            renderer->render(view);
            src = dst;
        }
        bloom.enabled = true;
        bloom.strength = 0.26f;
        bloom.levels = 7;
        bloom.resolution = 1080;
        bloom.threshold = true;
        bloom.highlight = 12.f;
        bloom.blendMode = BloomOptions::BlendMode::ADD;
        view->setBloomOptions(bloom);
        view->setPostProcessingEnabled(true);
        view->setScreenSpaceRefractionEnabled(false);
        rm.setMaterialInstanceAt(rm.getInstance(plate.entity), 0, plate_mi[src]);
        set_shown({plate.entity});
        view->setRenderTarget(rt8);
        renderer->render(view);
        renderer->readPixels(rt8, 0, 0, W, H,
                             backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA,
                                                            backend::PixelDataType::UBYTE));
        renderer->endFrame();
        engine->flushAndWait();
        auto t1 = std::chrono::steady_clock::now();
        if (!write_png(path, W, H, pic.data())) return false;
        std::printf("frame %s %.1f ms wall (flushAndWait + readPixels, no geometry readback)\n", path,
                    std::chrono::duration<double, std::milli>(t1 - t0).count());
        return true;
    };

    upload(black);
    if (!shoot("out/caustics-off.png")) return fail("off");
    upload(tex);
    if (!shoot("out/caustics-hero.png")) return fail("hero");

    for (int f = 0; f < 6; ++f) {
        float u = (float)f / 5.f;
        float dx = -0.22f + 0.44f * u;
        Glass mb = back, mm = mid, mf = front;
        for (Sample& a : mb.s) a.p.x += dx;
        for (Sample& a : mm.s) a.p.x += dx;
        for (Sample& a : mf.s) a.p.x += dx;
        Field fr;
        fr.build({mb, mm, mf});
        V3 sd{sun0.x + dx * 0.55f, sun0.y, sun0.z};
        std::vector<float> frame_tex;
        trace(fr, sd, lux, false, frame_tex);
        upload(frame_tex);
        auto& tm = engine->getTransformManager();
        tm.setTransform(tm.getInstance(hb.entity), mat4f::translation(float3{dx, 0, 0}));
        tm.setTransform(tm.getInstance(hm.entity), mat4f::translation(float3{dx, 0, 0}));
        tm.setTransform(tm.getInstance(hf.entity), mat4f::translation(float3{dx, 0, 0}));
        tm.setTransform(tm.getInstance(chrome_h.entity), mat4f::translation(float3{dx, 0, 0}));
        auto& lm = engine->getLightManager();
        lm.setDirection(lm.getInstance(sun), normalize(float3{sd.x, sd.y, sd.z}));
        char name[64];
        std::snprintf(name, sizeof(name), "out/caustics-motion-%02d.png", f);
        if (!shoot(name)) return fail("motion");
    }
    std::printf("PASS caustics\n");
    return 0;
}
