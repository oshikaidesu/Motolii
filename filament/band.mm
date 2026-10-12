// band.mm: bands of clear glass along closed 3D paths, to be DROPPED IN: a path and a width are all a band is given.
// (After a reference the user gave on 2026-10-12: pale ground, a broad curved glass band, pale rainbow sheen, sharp
// colour at the edges, lines of light inside.) The look is the glass's own (GlassLook, one set of numbers for every
// band and every scene); a scene only says where the path is, what stands behind it, and where the camera is.
// Everything that draws is Filament as it is: screen-space refraction with dispersion, thin-film iridescence
// (band_glass.mat), the bundled studio light, bloom, tone mapping. Nothing follows light.
// The frame, for any number of bands (hero.mm's way of letting glass see glass):
//   1. the scene without glass                      -> a picture
//   2. that picture + the bands' FAR sides          -> a picture   (the far surface mirrors the studio, bends once)
//   3. that picture + the bands' NEAR sides         -> a picture   (bends again; the colours split at what 2 drew)
//   4. that picture, bloom, tone mapping            -> the screen
// MODEL=A (default) is the above. MODEL=B: the near side's pixels follow their sight line through the REAL far
// side (band_two.mat; the far side's normals and depth drawn first by band_gbuf.mat), two true crossings of the
// surface per colour step. MODEL=C: B and one mirroring inside (the near side drawn as numbers too).
// ./build_one.sh band && ./out/band [scene]     scene: ref (default), loops, dark    -> out/band-<scene>[-B|-C].png
// TIME=n: n frames one after another, then the mean time of a frame.
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
#include <chrono>
#include <clocale>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>

using namespace filament;
using namespace filament::math;

static const int W = 1080;
static const int H = 1920;
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
    std::vector<V3> pos, nrm;   // (band.mm's checks: the same corners and normals, kept for the CPU)
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
static V3 limit_tangent(const Cubic& c, float t) {
    V3 d = bezier_d(c, t);
    if (len(d) >= 1e-4f) return d;
    float tin = std::clamp(t + (t < 0.5f ? 1e-3f : -1e-3f), 0.f, 1.f);
    V3 side = bezier_d(c, tin);
    if (len(side) >= 1e-4f) return side;
    V3 chord = c.p[3] - c.p[0];
    return len(chord) >= 1e-4f ? chord : V3{1, 0, 0};
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
static float signed_about(V3 T, V3 from, V3 to) {
    return std::atan2(dot(T, cross(from, to)), std::clamp(dot(from, to), -1.f, 1.f));
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
    return catmull({{-0.78f, -0.22f, 0.12f},
                    {-0.22f, -0.48f, 0.38f},
                    {0.55f, -0.12f, 0.22f},
                    {0.42f, 0.46f, -0.22f},
                    {-0.18f, 0.38f, -0.48f},
                    {-0.68f, 0.08f, -0.12f}});
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
    if (len(n) < 1e-4f) {
        V3 axis = cross(has ? prev.T : up, s.T);
        float axis_len = len(axis);
        if (!has || axis_len < 1e-5f) n = cross(up, s.T);
        else n = rot_about(prev.N, norm(axis), std::atan2(axis_len, std::clamp(dot(prev.T, s.T), -1.f, 1.f)));
        n = n - s.T * dot(n, s.T);
    }
    s.N = norm(n);
    s.B = norm(cross(s.T, s.N));
    s.s = has ? prev.s + len(s.p - prev.p) : 0;
    out.push_back(s);
}

static std::vector<Sample> sample_uniform(const std::vector<Cubic>& spans, int per) {
    std::vector<Sample> out;
    for (const Cubic& c : spans)
        for (int i = 0; i < per; ++i) push_sample(out, c, (float)i / (float)per);
    return out;
}

static std::vector<Sample> close_frames(const std::vector<Cubic>& spans, int per) {
    std::vector<Sample> in = sample_uniform(spans, per);
    float alpha = 0;
    {
        V3 n = transport_n(in.back().N, in.back().T, in.front().T);
        alpha = signed_about(in.front().T, in.front().N, norm(n));
    }
    float L = in.back().s + len(in.front().p - in.back().p);
    for (Sample& a : in) {
        a.N = norm(rot_about(a.N, a.T, -alpha * a.s / L));
        a.B = norm(cross(a.T, a.N));
    }
    return in;
}

static std::vector<Sample> scaled(std::vector<Sample> s, float k, V3 shift) {
    V3 c{0, 0, 0};
    for (const Sample& a : s) c = c + a.p;
    c = c * (1.f / (float)s.size());
    for (Sample& a : s) a.p = c + (a.p - c) * k + shift;
    return s;
}
static std::vector<Sample> nudged(std::vector<Sample> s, float along_n) {
    for (Sample& a : s) a.p = a.p + a.N * along_n;
    return s;
}

static void rotX(V3& v, float ca, float sa) {
    float y = v.y * ca - v.z * sa;
    float z = v.y * sa + v.z * ca;
    v.y = y;
    v.z = z;
}
static V3 centroid(const std::vector<Sample>& s) {
    V3 c{0, 0, 0};
    for (const Sample& a : s) c = c + a.p;
    return c * (1.f / (float)s.size());
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
static std::vector<Cubic> tilt_spans(std::vector<Cubic> spans, V3 c, float ang) {
    float ca = std::cos(ang), sa = std::sin(ang);
    for (Cubic& cu : spans)
        for (int k = 0; k < 4; ++k) {
            cu.p[k] = cu.p[k] - c;
            rotX(cu.p[k], ca, sa);
            cu.p[k] = cu.p[k] + c;
        }
    return spans;
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
    float packf[7] = {p.x, p.y, p.z, q.x, q.y, q.z, q.w};
    v.insert(v.end(), packf, packf + 7);
}

// A band of glass along a path. The cross-section is a rounded slab, `wide` across (along B) and `thick` through
// (along N), `round`: 2 is an ellipse, more is flatter faces with tighter edges; `rib` swells the middle of the two
// flat faces (0: none), which gives a third line of light along the band besides the two edges.
// inside: the same surface seen from within (triangles turned round, normals inward): the band's far side.
// Experiments on the shape (band.mm, PROFILE): facets > 0: the outline is that many flat faces with sharp creases;
// grooves > 0: that many furrows run along the band (the outline waves in and out by `deep`).
static Built tube(const std::vector<Sample>& s, int sides, float wide, float thick, float round, float rib, bool inside,
                  int facets = 0, int grooves = 0, float deep = 0.f) {
    Built m;
    int rings = (int)s.size();
    auto at = [&](float ang, float& x, float& y) {
        float c = std::cos(ang), sn = std::sin(ang);
        y = (sn < 0 ? -1.f : 1.f) * std::pow(std::fabs(sn), 2.f / round);
        x = (c < 0 ? -1.f : 1.f) * std::pow(std::fabs(c), 2.f / round) * (1.f + rib * std::exp(-(y * y) / 0.10f));
        float wave = grooves > 0 ? 1.f + deep * std::cos((float)grooves * ang) : 1.f;
        x *= thick * wave;
        y *= wide * wave;
    };
    const int per = facets > 0 ? 2 * facets : sides;   // corners in a ring (a facet has its own two)
    for (int i = 0; i < rings; ++i) {
        for (int k = 0; k < per; ++k) {
            float x, y;
            V3 nrm;
            if (facets > 0) {
                int face = k / 2, end = k % 2;
                float a0 = (float)face / (float)facets * TAU, a1 = (float)(face + 1) / (float)facets * TAU, x0, y0, x1, y1;
                at(a0, x0, y0);
                at(a1, x1, y1);
                x = end ? x1 : x0;
                y = end ? y1 : y0;
                nrm = norm(s[i].N * (y1 - y0) - s[i].B * (x1 - x0));
            } else {
                float ang = (float)k / (float)sides * TAU, x0, y0, x1, y1;
                at(ang, x, y);
                at(ang - 0.002f, x0, y0);
                at(ang + 0.002f, x1, y1);
                nrm = norm(s[i].N * (y1 - y0) - s[i].B * (x1 - x0));   // the outline's tangent turned a quarter, outward
            }
            V3 p = s[i].p + s[i].N * x + s[i].B * y;
            pack(m.v, p, s[i].T, inside ? nrm * -1.f : nrm);
            m.pos.push_back(p);
            m.nrm.push_back(nrm);
        }
    }
    for (int i = 0; i < rings; ++i) {
        int i1 = (i + 1) % rings;
        for (int k = 0; k < per; ++k) {
            if (facets > 0 && k % 2 == 1) continue;          // (a facet: its two corners here and in the next ring)
            int k1 = facets > 0 ? k + 1 : (k + 1) % per;
            uint32_t a = (uint32_t)(i * per + k), b = (uint32_t)(i * per + k1);
            uint32_t c = (uint32_t)(i1 * per + k), d = (uint32_t)(i1 * per + k1);
            if (inside) m.idx.insert(m.idx.end(), {a, c, b, b, c, d});
            else m.idx.insert(m.idx.end(), {a, b, c, b, d, c});
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

static float gap(const std::vector<Sample>& a, const std::vector<Sample>& b) {
    float m = 1e9f;
    for (const Sample& p : a)
        for (const Sample& q : b) m = std::min(m, len(p.p - q.p));
    return m;
}

struct Hold {
    Built mesh;
    utils::Entity entity = {};
    VertexBuffer* vb = nullptr;
    IndexBuffer* ib = nullptr;
};

static int fail(const char* msg) {
    std::printf("FAIL %s\n", msg);
    return 1;
}

static float envf(const char* name, float d) { return std::getenv(name) ? (float)std::atof(std::getenv(name)) : d; }

// The look of the glass. Not per scene, not per band.
struct GlassLook {
    float ior = 1.7f, thickness = 0.9f, dispersion = 45.f, reflectance = 1.0f;
    float absorb = 0.25f;                       // (x 1.6, 0.9, 0.5 for red, green, blue: a breath of blue)
    float sheen = 1.0f, film0 = 280.f, film1 = 620.f, filmWave = 3.5f;   // the thin film: how much, thickness from..to (nm), drift
    float farReflectance = 1.0f, farSheen = 1.0f, farThickness = 0.35f;   // the far side: mirrors more than it bends
    float slab = 0.41f;                         // how thick the band is, as a part of its half width
    float round = 5.f, rib = 0.10f;
};

int main(int argc, char** argv) {
    const std::string which = argc > 1 ? argv[1] : "ref";
    const std::string model = std::getenv("MODEL") ? std::getenv("MODEL") : "A";
    if (model != "A" && model != "B" && model != "C" && model != "D") return fail("MODEL: A, B, C or D");
    const GlassLook look;
    const int per = (int)envf("MESHPER", 96.f), sides = (int)envf("MESHSIDES", 96.f);   // (how finely the band's mesh is cut: along a span, around)

    // ---- a scene: paths (points of a closed loop, half width), what is behind, the camera
    struct Band { std::vector<V3> pts; float wide; float twist; };
    std::vector<Band> bands;
    float3 eye, target;
    float fov = 30.f;
    float backLow = 0.62f, backHigh = 1.25f, floorCol = 0.055f, floorRough = 0.6f;   // (the floor is the scene's: a glossy one mirrors the studio and goes white)
    auto loop = [](float R, V3 c, float tiltX, float wob) {
        std::vector<V3> p;
        for (int i = 0; i < 8; ++i) {
            float a = (float)i / 8.f * TAU;
            V3 q{R * std::cos(a), R * 0.92f * std::sin(a), wob * std::sin(2.f * a + 0.6f)};
            rotX(q, std::cos(tiltX), std::sin(tiltX));
            p.push_back(q + c);
        }
        return p;
    };
    if (which == "ref") {            // the reference: one big standing loop, seen from close by
        bands.push_back({loop(1.6f, {0, 1.376f, 0}, 0.f, 0.22f), 0.22f, -0.6f});
        eye = {0.9f, 0.55f, 2.0f};
        target = {1.25f, 0.75f, 0.f};
    } else if (which == "loops") {   // three loops of different sizes leaning on each other, seen whole
        bands.push_back({loop(1.0f, {0, 0.95f, 0}, 0.25f, 0.15f), 0.16f, 0.4f});
        bands.push_back({loop(0.62f, {0.75f, 0.60f, 0.45f}, -0.5f, 0.10f), 0.11f, 1.2f});
        bands.push_back({loop(0.45f, {-0.85f, 0.44f, 0.55f}, 0.9f, 0.06f), 0.08f, -0.3f});
        eye = {0.4f, 1.1f, 4.2f};
        target = {0.f, 0.75f, 0.f};
        fov = 34.f;
    } else if (which == "dark") {    // the same loops on a dark ground
        bands.push_back({loop(1.0f, {0, 0.95f, 0}, 0.25f, 0.15f), 0.16f, 0.4f});
        bands.push_back({loop(0.62f, {0.75f, 0.60f, 0.45f}, -0.5f, 0.10f), 0.11f, 1.2f});
        eye = {-0.9f, 0.8f, 3.4f};
        target = {0.1f, 0.8f, 0.f};
        fov = 34.f;
        backLow = 0.012f; backHigh = 0.05f; floorCol = 0.02f; floorRough = 0.45f;
    } else return fail("scene: ref, loops or dark");

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    utils::Entity cam_e = utils::EntityManager::get().create();
    Camera* camera = engine->createCamera(cam_e);
    camera->setExposure(16.0f, 1.0f / 125.0f, 100.0f);   // a plain sunny-day exposure: nothing here is tuned to the look
    camera->lookAt(eye, target, {0, 1, 0});
    camera->setProjection(fov, double(W) / double(H), 0.05, 60.0, Camera::Fov::VERTICAL);

    static AgxToneMapper agx(AgxToneMapper::AgxLook::NONE);
    ColorGrading* grading = ColorGrading::Builder().toneMapper(&agx).saturation(1.4f).build(*engine);
    view->setScene(scene);
    view->setCamera(camera);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setColorGrading(grading);
    view->setDithering(View::Dithering::NONE);
    Renderer::ClearOptions clear{};
    clear.clearColor = {0, 0, 0, 1};
    clear.clear = true;
    renderer->setClearOptions(clear);

    auto make_target = [&](Texture::InternalFormat format) {
        Texture* color = Texture::Builder().width(W).height(H).levels(1)
                             .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC).format(format).build(*engine);
        Texture* depth = Texture::Builder().width(W).height(H).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH24).build(*engine);
        RenderTarget* rt = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, color).texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
        return std::tuple<Texture*, RenderTarget*>{color, rt};
    };
    auto [colorA, rtA] = make_target(Texture::InternalFormat::RGBA16F);
    auto [colorB, rtB] = make_target(Texture::InternalFormat::RGBA16F);
    auto [color8, rt8] = make_target(Texture::InternalFormat::RGBA8);
    if (!rtA || !rtB || !rt8) return fail("target");
    auto [gFar, rtGFar] = make_target(Texture::InternalFormat::RGBA32F);     // models B, C: the far sides as numbers
    auto [gNear, rtGNear] = make_target(Texture::InternalFormat::RGBA32F);   // model C: the near sides
    if (!rtGFar || !rtGNear) return fail("target");
    Texture* colorF[2] = {colorA, colorB};
    RenderTarget* rtF[2] = {rtA, rtB};

    // the bundled studio (the one glass_scene.mm uses): the glass's light and what it mirrors; never shown as a background
    auto ibl_bytes = load_file("out/studio/_ibl.ktx");
    if (ibl_bytes.empty()) return fail("out/studio/_ibl.ktx");
    Texture* ibl_tex = ktxreader::Ktx1Reader::createTexture(engine, new image::Ktx1Bundle(ibl_bytes.data(), (uint32_t)ibl_bytes.size()), false);
    float3 sh[9];
    if (!ibl_tex || !load_sh("out/studio/sh.txt", sh)) return fail("studio");
    IndirectLight* ibl = IndirectLight::Builder().reflections(ibl_tex).irradiance(3, sh).intensity(60000.f)
                             .rotation(mat3f::rotation(2.2f, float3{0, 1, 0})).build(*engine);
    scene->setIndirectLight(ibl);

    auto glass_pkg = load_file("band_glass.filamat");
    auto floor_pkg = load_file("floor_lit.filamat");
    auto back_pkg = load_file("backdrop_grad.filamat");
    auto plate_pkg = load_file("plate.filamat");
    if (glass_pkg.empty() || floor_pkg.empty() || back_pkg.empty() || plate_pkg.empty()) return fail("filamat");
    Material* glass_mat = Material::Builder().package(glass_pkg.data(), glass_pkg.size()).build(*engine);
    Material* floor_mat = Material::Builder().package(floor_pkg.data(), floor_pkg.size()).build(*engine);
    Material* back_mat = Material::Builder().package(back_pkg.data(), back_pkg.size()).build(*engine);
    Material* plate_mat = Material::Builder().package(plate_pkg.data(), plate_pkg.size()).build(*engine);
    if (!glass_mat || !floor_mat || !back_mat || !plate_mat) return fail("material");

    auto glass_of = [&](bool farSide) {
        MaterialInstance* mi = glass_mat->createInstance();
        mi->setParameter("ior", look.ior);
        mi->setParameter("transmission", 1.0f);
        mi->setParameter("thickness", farSide ? look.farThickness : look.thickness);
        mi->setParameter("dispersion", look.dispersion);
        mi->setParameter("reflectance", farSide ? look.farReflectance : look.reflectance);
        mi->setParameter("absorption", float3{look.absorb * 1.6f, look.absorb * 0.9f, look.absorb * 0.5f});
        mi->setParameter("sheen", farSide ? look.farSheen : look.sheen);
        mi->setParameter("film", float2{look.film0, look.film1});
        mi->setParameter("filmWave", look.filmWave);
        return mi;
    };
    // models B and C
    auto gbuf_pkg = load_file("band_gbuf.filamat");
    auto two_pkg = load_file("band_two.filamat");
    auto sky_bytes = load_file("out/studio/_skybox.ktx");
    if (gbuf_pkg.empty() || two_pkg.empty() || sky_bytes.empty()) return fail("band_gbuf / band_two .filamat, out/studio/_skybox.ktx");
    Material* gbuf_mat = Material::Builder().package(gbuf_pkg.data(), gbuf_pkg.size()).build(*engine);
    Material* two_mat = Material::Builder().package(two_pkg.data(), two_pkg.size()).build(*engine);
    Texture* studio_tex = ktxreader::Ktx1Reader::createTexture(engine, new image::Ktx1Bundle(sky_bytes.data(), (uint32_t)sky_bytes.size()), false);
    if (!gbuf_mat || !two_mat || !studio_tex) return fail("material");
    MaterialInstance* gbuf_far = gbuf_mat->createInstance();
    gbuf_far->setParameter("flip", -1.0f);
    MaterialInstance* gbuf_near = gbuf_mat->createInstance();
    gbuf_near->setParameter("flip", 1.0f);
    MaterialInstance* two_mi = two_mat->createInstance();
    {
        TextureSampler lin(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
        lin.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
        lin.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
        TextureSampler near1(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
        near1.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
        near1.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
        float widest = 0;
        for (const Band& b : bands) widest = std::max(widest, b.wide);
        two_mi->setParameter("scene", colorA, lin);
        two_mi->setParameter("farSide", gFar, near1);
        two_mi->setParameter("nearSide", gNear, near1);
        two_mi->setParameter("studio", studio_tex, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
        const float exposure = 1.0f / (1.2f * 256.f * 125.f);   // (f/16, 1/125 s, ISO 100)
        two_mi->setParameter("studioGain", 60000.f * exposure * envf("STUDIO", 1.0f));
        two_mi->setParameter("studioTurn", envf("STUDIOTURN", 0.f));   // (0: the studio as the scene is lit by it)
        two_mi->setParameter("sunTo", normalize(float3{-0.35f, 0.6f, 0.7f}));
        two_mi->setParameter("sunGain", envf("SUNGLINT", 60.f));
        two_mi->setParameter("ior", envf("IOR", 1.5f));
        two_mi->setParameter("spread", envf("SPREAD", 0.04f));
        two_mi->setParameter("absorb", float3{0.5f, 0.28f, 0.16f} * envf("ABSORB", 1.0f));
        two_mi->setParameter("reach", widest * envf("REACH", 3.0f));
        two_mi->setParameter("beyond", envf("BEYOND", 3.0f));
        two_mi->setParameter("inner", model == "C" ? 1.0f : 0.0f);
        two_mi->setParameter("show", 0.0f);
        two_mi->setParameter("exact", envf("EXACT", 0.f));
        two_mi->setParameter("plain", envf("PLAIN", 0.f));
        two_mi->setParameter("turned", envf("TURNED", 1.f));   // (1 is right; 0 kept to show the mistake)
    }
    MaterialInstance* near_mi = glass_of(false);
    MaterialInstance* far_mi = glass_of(true);
    MaterialInstance* floor_mi = floor_mat->createInstance();
    floor_mi->setParameter("color", float4{floorCol, floorCol + 0.003f, floorCol + 0.012f, 1});
    floor_mi->setParameter("roughness", floorRough);
    MaterialInstance* back_mi = back_mat->createInstance();
    back_mi->setParameter("low", float4{backLow, backLow * 1.01f, backLow * 1.05f, 1});
    back_mi->setParameter("high", float4{backHigh, backHigh, backHigh, 1});
    back_mi->setParameter("bend", 1.2f);

    struct Held { Built mesh; VertexBuffer* vb; IndexBuffer* ib; utils::Entity e; };
    auto add = [&](Built mesh, MaterialInstance* mi, bool uv) {
        Held* h = new Held{std::move(mesh), nullptr, nullptr, {}};
        const uint32_t stride = uv ? 36 : 28, nv = (uint32_t)(h->mesh.v.size() * 4 / stride);
        VertexBuffer::Builder b;
        b.vertexCount(nv).bufferCount(1).attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, (uint8_t)stride);
        if (uv) b.attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, (uint8_t)stride).attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, (uint8_t)stride);
        else b.attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, (uint8_t)stride);
        h->vb = b.build(*engine);
        h->vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(h->mesh.v.data(), h->mesh.v.size() * sizeof(float)));
        h->ib = IndexBuffer::Builder().indexCount((uint32_t)h->mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        h->ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(h->mesh.idx.data(), h->mesh.idx.size() * sizeof(uint32_t)));
        h->e = utils::EntityManager::get().create();
        RenderableManager::Builder(1).boundingBox({{-60, -60, -60}, {60, 60, 60}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, h->vb, h->ib).material(0, mi)
            .culling(true).castShadows(false).receiveShadows(false).build(*engine, h->e);
        return h->e;
    };
    auto quad_uv = [&](V3 center, V3 right, V3 up, float hw, float hh) {   // (uv 0..1 across and up)
        Built m;
        V3 n = norm(cross(right, up));
        const float c[4][2] = {{-1, -1}, {1, -1}, {1, 1}, {-1, 1}};
        quatf q = mat3f::packTangentFrame(mat3f{float3{right.x, right.y, right.z}, float3{up.x, up.y, up.z}, float3{n.x, n.y, n.z}});
        for (int i = 0; i < 4; ++i) {
            V3 p = center + right * (hw * c[i][0]) + up * (hh * c[i][1]);
            float row[9] = {p.x, p.y, p.z, 0.5f + 0.5f * c[i][0], 0.5f + 0.5f * c[i][1], q.x, q.y, q.z, q.w};
            m.v.insert(m.v.end(), row, row + 9);
        }
        m.idx = {0, 1, 2, 0, 2, 3};
        return m;
    };

    // what is behind (the scene's own business)
    std::vector<utils::Entity> opaque;
    opaque.push_back(add(quad({0, 0, 0}, {1, 0, 0}, {0, 0, -1}, 40.f, 40.f), floor_mi, false));
    opaque.push_back(add(quad_uv({0, 6.f, -9.f}, {1, 0, 0}, {0, 1, 0}, 40.f, 9.f), back_mi, true));

    // DROPPING A BAND IN: its path and its half width; everything else is the glass's own
    std::vector<utils::Entity> farSides, nearSides;
    std::vector<Built> cpuNear;   // (checks)
    // PROFILE (experiments on the shape, not part of the look): facet<n> or groove<n>, e.g. facet10, groove12
    const std::string profile = std::getenv("PROFILE") ? std::getenv("PROFILE") : "";
    const int facets = profile.rfind("facet", 0) == 0 ? std::atoi(profile.c_str() + 5) : 0;
    const int grooves = profile.rfind("groove", 0) == 0 ? std::atoi(profile.c_str() + 6) : 0;
    const float deep = envf("DEEP", 0.06f);
    for (const Band& b : bands) {
        std::vector<Sample> path = close_frames(catmull(b.pts), per);
        for (Sample& q : path) {
            q.N = norm(rot_about(q.N, q.T, b.twist));
            q.B = norm(cross(q.T, q.N));
        }
        farSides.push_back(add(tube(path, sides, b.wide, b.wide * look.slab, look.round, look.rib, true, facets, grooves, deep), far_mi, false));
        Built nearMesh = tube(path, sides, b.wide, b.wide * look.slab, look.round, look.rib, false, facets, grooves, deep);
        cpuNear.push_back(Built{{}, nearMesh.idx, nearMesh.pos, nearMesh.nrm});
        nearSides.push_back(add(std::move(nearMesh), near_mi, false));
    }

    // model D (a probe): the bands' mesh in a tree three-mesh-bvh builds and packs itself (bvh_probe/build_bvh.mjs);
    // band_bvh.mat walks it with three-mesh-bvh's own GLSL. Done once, here; a frame only draws.
    MaterialInstance* bvh_mi = nullptr;
    std::vector<V3> allPos;
    std::vector<uint32_t> allIdx;
    static std::vector<uint8_t> bvhBytes;
    if (model == "D") {
        for (const Built& m : cpuNear) {
            const uint32_t base = (uint32_t)allPos.size();
            allPos.insert(allPos.end(), m.pos.begin(), m.pos.end());
            for (uint32_t i : m.idx) allIdx.push_back(base + i);
        }
        {
            FILE* f = std::fopen("out/band-mesh.bin", "wb");
            if (!f) return fail("out/band-mesh.bin");
            const uint32_t head[2] = {(uint32_t)allPos.size(), (uint32_t)allIdx.size()};
            std::fwrite(head, 4, 2, f);
            std::fwrite(allPos.data(), sizeof(V3), allPos.size(), f);
            std::fwrite(allIdx.data(), 4, allIdx.size(), f);
            for (const Built& m : cpuNear) std::fwrite(m.nrm.data(), sizeof(V3), m.nrm.size(), f);   // (smooth normals, after what three-mesh-bvh reads: for ../rayq)
            std::fclose(f);
        }
        if (std::system("node bvh_probe/build_bvh.mjs out/band-mesh.bin out/band-bvh.bin bvh_probe/three_mesh_bvh.glsl") != 0) return fail("bvh_probe/build_bvh.mjs (node, three-mesh-bvh)");
        bvhBytes = load_file("out/band-bvh.bin");
        auto bvh_pkg = load_file("band_bvh.filamat");
        if (bvhBytes.empty() || bvh_pkg.empty()) return fail("out/band-bvh.bin / band_bvh.filamat");
        Material* bvh_mat = Material::Builder().package(bvh_pkg.data(), bvh_pkg.size()).build(*engine);
        if (!bvh_mat) return fail("band_bvh material");
        bvh_mi = bvh_mat->createInstance();
        TextureSampler exact1(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
        const char* names[4] = {"bvhPosition", "bvhIndex", "bvhBounds", "bvhContents"};
        size_t at = 0;
        for (int k = 0; k < 4; ++k) {   // (the file: three-mesh-bvh's four textures, as it packed them)
            uint32_t h[4];
            std::memcpy(h, bvhBytes.data() + at, 16);
            at += 16;
            const size_t bytes = (size_t)h[0] * h[1] * h[2] * 4;
            const bool u = h[3] != 0;
            if ((h[2] != 4 && h[2] != 2) || (h[2] == 2 && !u)) return fail("a texture layout this probe does not know");
            Texture* t = Texture::Builder().width(h[0]).height(h[1]).levels(1).sampler(Texture::Sampler::SAMPLER_2D)
                .format(h[2] == 2 ? Texture::InternalFormat::RG32UI : u ? Texture::InternalFormat::RGBA32UI : Texture::InternalFormat::RGBA32F).build(*engine);
            t->setImage(*engine, 0, Texture::PixelBufferDescriptor(bvhBytes.data() + at, bytes,
                h[2] == 2 ? Texture::Format::RG_INTEGER : u ? Texture::Format::RGBA_INTEGER : Texture::Format::RGBA, u ? Texture::Type::UINT : Texture::Type::FLOAT));
            bvh_mi->setParameter(names[k], t, exact1);
            at += bytes;
        }
        const float exposure = 1.0f / (1.2f * 256.f * 125.f);
        bvh_mi->setParameter("studio", studio_tex, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
        bvh_mi->setParameter("studioGain", 60000.f * exposure);
        bvh_mi->setParameter("ior", envf("IOR", 1.5f));
        bvh_mi->setParameter("most", envf("MOST", 5.f));
        bvh_mi->setParameter("show", 0.0f);
    }

    // the picture so far, held up in front of the camera for the next pass to bend (hero.mm's plate)
    TextureSampler plate_sampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
    plate_sampler.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    plate_sampler.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    MaterialInstance* plate_mi[2] = {plate_mat->createInstance(), plate_mat->createInstance()};
    plate_mi[0]->setParameter("color", colorF[0], plate_sampler);
    plate_mi[1]->setParameter("color", colorF[1], plate_sampler);
    utils::Entity plate;
    {
        float3 f = normalize(target - eye), r = normalize(cross(f, float3{0, 1, 0})), u = cross(r, f);
        const float dist = 30.0f, hh = dist * std::tan(0.5f * fov * PI / 180.f), hw = hh * ((float)W / (float)H);
        float3 c = eye + f * dist;
        plate = add(quad_uv(V3{c.x, c.y, c.z}, V3{r.x, r.y, r.z}, V3{u.x, u.y, u.z}, hw, hh), plate_mi[0], true);
    }
    auto& rm = engine->getRenderableManager();
    rm.setCulling(rm.getInstance(plate), false);

    utils::Entity sun = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::SUN).color({1.0f, 0.98f, 0.95f}).intensity(57000.f).direction(normalize(float3{0.35f, -0.6f, -0.7f}))
        .castShadows(false).sunAngularRadius(0.6f).sunHaloSize(2.0f).build(*engine, sun);
    scene->addEntity(sun);

    std::vector<utils::Entity> shown;
    auto show = [&](const std::vector<utils::Entity>& a, const std::vector<utils::Entity>& b) {
        for (utils::Entity e : shown) scene->remove(e);
        shown = a;
        shown.insert(shown.end(), b.begin(), b.end());
        for (utils::Entity e : shown) scene->addEntity(e);
    };
    BloomOptions bloom{};
    bloom.strength = 0.7f;
    bloom.levels = 8;
    bloom.resolution = 1080;
    bloom.threshold = true;
    bloom.highlight = 20.0f;
    bloom.blendMode = BloomOptions::BlendMode::ADD;
    std::vector<uint8_t> pic((size_t)W * H * 4);
    std::vector<uint16_t> half((size_t)W * H * 4);
    bool readGlass = false;   // (checks: the glass pass's own numbers)
    std::vector<float> full((size_t)W * H * 4);   // (model D's checks: whole numbers must come back whole)
    auto frame = [&](bool read) {
        bool ok = false;
        for (int spin = 0; spin < 4000 && !ok; ++spin) ok = renderer->beginFrame(swap);
        if (!ok) return false;
        bloom.enabled = false;
        view->setBloomOptions(bloom);
        view->setPostProcessingEnabled(false);
        view->setAntiAliasing(View::AntiAliasing::NONE);
        view->setScreenSpaceRefractionEnabled(false);
        show(opaque, {});                                  // 1. the scene without glass
        view->setRenderTarget(rtF[0]);
        renderer->render(view);
        auto dress = [&](const std::vector<utils::Entity>& es, MaterialInstance* mi) {
            for (utils::Entity e : es) rm.setMaterialInstanceAt(rm.getInstance(e), 0, mi);
        };
        if (model == "A") {
            view->setScreenSpaceRefractionEnabled(true);
            dress(farSides, far_mi);
            dress(nearSides, near_mi);
            rm.setMaterialInstanceAt(rm.getInstance(plate), 0, plate_mi[0]);
            show({plate}, farSides);                           // 2. + the far sides
            view->setRenderTarget(rtF[1]);
            renderer->render(view);
            rm.setMaterialInstanceAt(rm.getInstance(plate), 0, plate_mi[1]);
            show({plate}, nearSides);                          // 3. + the near sides
            view->setRenderTarget(rtF[0]);
            renderer->render(view);
        } else if (model == "D") {
            dress(nearSides, bvh_mi);
            if (readGlass) {                                   // (checks) the glass alone, as numbers
                show(nearSides, {});
                view->setRenderTarget(rtGNear);
                renderer->render(view);
                renderer->readPixels(rtGNear, 0, 0, (uint32_t)W, (uint32_t)H,
                                     backend::PixelBufferDescriptor(full.data(), full.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
            }
            rm.setMaterialInstanceAt(rm.getInstance(plate), 0, plate_mi[0]);
            show({plate}, nearSides);                          // 2. the glass: every pixel asks the mesh's tree for its way
            view->setRenderTarget(rtF[1]);
            renderer->render(view);
        } else {
            dress(farSides, gbuf_far);
            show(farSides, {});                                // 2. the far sides as numbers
            view->setRenderTarget(rtGFar);
            renderer->render(view);
            if (model == "C") {
                dress(nearSides, gbuf_near);
                show(nearSides, {});                           // (C) the near sides as numbers
                view->setRenderTarget(rtGNear);
                renderer->render(view);
            }
            dress(nearSides, two_mi);
            rm.setMaterialInstanceAt(rm.getInstance(plate), 0, plate_mi[0]);
            show({plate}, nearSides);                          // 3. the glass: every pixel follows its line through the solid
            view->setRenderTarget(rtF[1]);
            renderer->render(view);
            if (readGlass)
                renderer->readPixels(rtF[1], 0, 0, (uint32_t)W, (uint32_t)H,
                                     backend::PixelBufferDescriptor(half.data(), half.size() * sizeof(uint16_t), backend::PixelDataFormat::RGBA, backend::PixelDataType::HALF));
        }
        bloom.enabled = true;
        view->setBloomOptions(bloom);
        view->setPostProcessingEnabled(true);
        view->setAntiAliasing(View::AntiAliasing::FXAA);
        view->setScreenSpaceRefractionEnabled(false);
        rm.setMaterialInstanceAt(rm.getInstance(plate), 0, plate_mi[model == "A" ? 0 : 1]);
        show({plate}, {});                                 // 4. bloom, tone mapping
        view->setRenderTarget(rt8);
        renderer->render(view);
        if (read)
            renderer->readPixels(rt8, 0, 0, (uint32_t)W, (uint32_t)H,
                                 backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
        renderer->endFrame();
        return true;
    };
    for (int i = 0; i < 3; ++i) {
        if (!frame(i == 2)) return fail("begin");
        engine->flushAndWait();
    }
    const char* tag = std::getenv("NAME");
    const std::string out = "out/band-" + which + (model == "A" ? "" : "-" + model) + (tag ? std::string("-") + tag : std::string()) + ".png";
    if (!write_png(out.c_str(), W, H, pic.data())) return fail("png");
    std::printf("wrote %s  (%zu band(s))\n", out.c_str(), bands.size());

    // ---- CHECK=1 (models B, C): is the way through the glass the right one?
    if (std::getenv("CHECK") && (model == "B" || model == "C")) {
        auto glass_numbers = [&](float showMode, float exactMode, std::vector<float>& outv) {
            two_mi->setParameter("show", showMode);
            two_mi->setParameter("exact", exactMode);
            readGlass = true;
            for (int i = 0; i < 2; ++i) { frame(false); engine->flushAndWait(); }
            readGlass = false;
            outv.resize((size_t)W * H * 3);
            for (size_t i = 0; i < (size_t)W * H; ++i)
                for (int c = 0; c < 3; ++c) outv[i * 3 + c] = half_to_float(half[i * 4 + c]);
        };
        std::vector<float> cls, num, uvNear, uvExact;
        std::vector<uint8_t> leaves((size_t)W * H, 0);   // glass pixels whose line leaves through the far side
        glass_numbers(1.f, envf("EXACT", 0.f), cls);
        glass_numbers(2.f, 0.f, num);
        glass_numbers(4.f, 0.f, uvNear);
        glass_numbers(4.f, 1.f, uvExact);
        two_mi->setParameter("show", 0.0f);
        two_mi->setParameter("exact", envf("EXACT", 0.f));

        // 1. why each glass pixel shows what it shows (the green colour step), as a picture and as shares
        {
            std::vector<uint8_t> img((size_t)W * H * 4, 255);
            const char* names[8] = {"left through the far side, sees the scene's picture, bright", "... dark there", "left, off the screen: sees the studio, bright", "... dark there",
                                    "mirrored at the far side (past the critical angle), then bright", "... then dark", "far side NOT found: the line left that side's picture (the solid's outline)",
                                    "far side NOT found: it lay under the line but was never crossed"};
            const float key[8][3] = {{0, 0.9f, 0}, {0, 0.35f, 0}, {0, 0.85f, 1}, {0, 0, 0.8f}, {1, 0.9f, 0}, {0.45f, 0.3f, 0}, {0.45f, 0, 0}, {1, 0, 0}};
            long count[8] = {}, glass = 0;
            for (size_t i = 0; i < (size_t)W * H; ++i) {
                const float* c = &cls[i * 3];
                for (int k = 0; k < 3; ++k) img[i * 4 + k] = (uint8_t)std::lround(std::clamp(c[k], 0.f, 1.f) * 255.f);
                int best = -1;
                for (int k = 0; k < 8; ++k)
                    if (std::fabs(c[0] - key[k][0]) + std::fabs(c[1] - key[k][1]) + std::fabs(c[2] - key[k][2]) < 0.06f) best = k;
                if (best >= 0) { ++count[best]; ++glass; }
                if (best >= 0 && best < 4) leaves[i] = 1;
            }
            const std::string cp = "out/band-" + which + "-" + model + (tag ? std::string("-") + tag : std::string()) + "-why.png";
            write_png(cp.c_str(), W, H, img.data());
            std::printf("CHECK 1: %ld glass pixels (%s). Wrote %s\n", glass, profile.empty() ? "smooth outline" : profile.c_str(), cp.c_str());
            for (int k = 0; k < 8; ++k) std::printf("    %5.1f%%  %s\n", 100.0 * (double)count[k] / (double)std::max(1L, glass), names[k]);
        }

        // 2. the way to the far side against the same way worked out on the CPU from the mesh itself
        {
            struct Hit { float t; V3 n; };
            auto first_hit = [&](V3 o, V3 d, float tMin) {
                Hit best{1e30f, {0, 0, 0}};
                for (const Built& m : cpuNear)
                    for (size_t i = 0; i + 2 < m.idx.size(); i += 3) {
                        const V3 a = m.pos[m.idx[i]], e1 = m.pos[m.idx[i + 1]] - a, e2 = m.pos[m.idx[i + 2]] - a;
                        const V3 pv = cross(d, e2);
                        const float det = dot(e1, pv);
                        if (std::fabs(det) < 1e-12f) continue;
                        const V3 tv = o - a;
                        const float u = dot(tv, pv) / det;
                        if (u < 0 || u > 1) continue;
                        const V3 qv = cross(tv, e1);
                        const float v = dot(d, qv) / det;
                        if (v < 0 || u + v > 1) continue;
                        const float t = dot(e2, qv) / det;
                        if (t > tMin && t < best.t)
                            best = Hit{t, norm(m.nrm[m.idx[i]] * (1 - u - v) + m.nrm[m.idx[i + 1]] * u + m.nrm[m.idx[i + 2]] * v)};
                    }
                return best;
            };
            const float3 f3 = normalize(target - eye), r3 = normalize(cross(f3, float3{0, 1, 0})), u3 = cross(r3, f3);
            const V3 F{f3.x, f3.y, f3.z}, Rr{r3.x, r3.y, r3.z}, U{u3.x, u3.y, u3.z}, E{eye.x, eye.y, eye.z};
            const float th = std::tan(0.5f * fov * PI / 180.f), asp = (float)W / (float)H;
            const float n = envf("IOR", 1.5f) + envf("SPREAD", 0.04f) * (2.f / 5.f - 0.5f);
            for (int flip = 0; flip < 2; ++flip) {   // (the picture may come back upside down: the way with the more agreement is the right one)
                long tried = 0, bothHit = 0, gpuMissed = 0;
                std::vector<float> lenErr, angErr;
                for (int y = 12; y < H; y += 24)
                    for (int x = 12; x < W; x += 24) {
                        const float ndx = ((float)x + 0.5f) / (float)W * 2.f - 1.f, ndy = ((float)y + 0.5f) / (float)H * 2.f - 1.f;
                        const V3 d = norm(F + Rr * (ndx * th * asp) + U * (ndy * th));
                        const Hit h1 = first_hit(E, d, 1e-4f);
                        if (h1.t > 1e29f) continue;
                        V3 N = h1.n;
                        if (dot(N, d) > 0) N = N * -1.f;
                        const float ci = -dot(N, d), k = 1.f - (1.f - ci * ci) / (n * n);
                        const V3 T1 = norm(d * (1.f / n) + N * (ci / n - std::sqrt(std::max(k, 0.f))));
                        const V3 P1 = E + d * h1.t;
                        const Hit h2 = first_hit(P1, T1, 1e-3f);
                        if (h2.t > 1e29f) continue;
                        ++tried;
                        const float* g = &num[((size_t)(flip ? H - 1 - y : y) * W + x) * 3];
                        if (!(g[0] > 0)) { ++gpuMissed; continue; }
                        ++bothHit;
                        float gx = g[1], gy = g[2], sz = 1.f;
                        if (gx > 2.f) { gx -= 4.f; sz = -1.f; }
                        V3 gn{gx, gy, sz * std::sqrt(std::max(0.f, 1.f - gx * gx - gy * gy))};
                        V3 tn = h2.n;
                        if (dot(tn, T1) < 0) tn = tn * -1.f;
                        if (flip == 1 && std::getenv("CHECKDUMP") && lenErr.size() < 8)
                            std::printf("      sample: gpu normal %.2f %.2f %.2f | cpu normal %.2f %.2f %.2f | raw %.3f %.3f %.3f | T1 %.2f %.2f %.2f\n", gn.x, gn.y, gn.z, tn.x, tn.y, tn.z, g[0], g[1], g[2], T1.x, T1.y, T1.z);
                        lenErr.push_back(std::fabs(g[0] - h2.t) / h2.t);
                        angErr.push_back(std::acos(std::clamp(dot(norm(gn), tn), -1.f, 1.f)) * 180.f / PI);
                    }
                std::sort(lenErr.begin(), lenErr.end());
                std::sort(angErr.begin(), angErr.end());
                auto q = [](const std::vector<float>& v, double at) { return v.empty() ? 0.f : v[(size_t)(at * (double)(v.size() - 1))]; };
                std::printf("CHECK 2 (%s): %ld sight lines that truly cross the solid; the picture's search found the far side for %ld (%.1f%%), missed %ld. Where found: length off by %.1f%% (median), %.1f%% (9 in 10 within); far side's normal off by %.1f deg (median), %.1f deg (9 in 10 within)\n",
                            flip ? "picture upside down" : "picture as is", tried, bothHit, 100.0 * (double)bothHit / (double)std::max(1L, tried), gpuMissed, 100.f * q(lenErr, 0.5), 100.f * q(lenErr, 0.9),
                            q(angErr, 0.5), q(angErr, 0.9));
            }
            // 4. (CPU only, geometry only) a sight line mirrored inside at the far side: how many mirrorings until it meets
            //    a surface it can leave through? Nothing here is drawn; it says what a method would have to follow.
            {
                const int MOST = 32;
                std::vector<long> after((size_t)MOST + 2, 0);
                std::vector<float> way;
                long lines = 0;
                for (int y = 12; y < H; y += 24)
                    for (int x = 12; x < W; x += 24) {
                        const float ndx = ((float)x + 0.5f) / (float)W * 2.f - 1.f, ndy = ((float)y + 0.5f) / (float)H * 2.f - 1.f;
                        const V3 d = norm(F + Rr * (ndx * th * asp) + U * (ndy * th));
                        const Hit h1 = first_hit(E, d, 1e-4f);
                        if (h1.t > 1e29f) continue;
                        V3 N = h1.n;
                        if (dot(N, d) > 0) N = N * -1.f;
                        const float ci = -dot(N, d), k = 1.f - (1.f - ci * ci) / (n * n);
                        V3 T = norm(d * (1.f / n) + N * (ci / n - std::sqrt(std::max(k, 0.f))));
                        V3 P = E + d * h1.t;
                        int b = 0;
                        float len = 0;
                        bool lost = false;
                        for (; b <= MOST; ++b) {
                            const Hit h = first_hit(P, T, 1e-3f);
                            if (h.t > 1e29f) { lost = true; break; }
                            len += h.t;
                            V3 M = h.n;
                            if (dot(M, T) < 0) M = M * -1.f;   // (M: out of the glass, along the line)
                            const float co = dot(M, T);
                            if (1.f - n * n * (1.f - co * co) >= 0) break;   // can leave here
                            P = P + T * h.t;
                            T = norm(T - M * (2.f * co));
                        }
                        ++lines;
                        ++after[(size_t)(lost ? MOST + 1 : std::min(b, MOST))];
                        if (!lost && b < MOST) way.push_back(len);
                    }
                std::sort(way.begin(), way.end());
                std::printf("CHECK 4 (CPU, geometry only): %ld sight lines into the glass. Mirrorings inside before a surface lets the line out:\n   ", lines);
                long sum = 0;
                for (int b = 0; b <= MOST + 1; ++b) {
                    if (!after[(size_t)b]) continue;
                    sum += after[(size_t)b];
                    if (b == MOST + 1) std::printf("  lost (no surface ahead): %.1f%%", 100.0 * (double)after[(size_t)b] / (double)lines);
                    else std::printf("  %d%s: %.1f%% (up to here %.1f%%)", b, b == MOST ? " or more" : "", 100.0 * (double)after[(size_t)b] / (double)lines, 100.0 * (double)sum / (double)lines);
                }
                std::printf("\n    way inside the glass: %.2f (median), %.2f (9 in 10 within), %.2f (longest)\n", way.empty() ? 0.f : way[way.size() / 2],
                            way.empty() ? 0.f : way[(size_t)(0.9 * (double)(way.size() - 1))], way.empty() ? 0.f : way.back());
            }
        }

        // 3. where the line that left is looked up: `beyond` units along it, against the true floor and backdrop
        {
            long both = 0, nearOn = 0, exactOn = 0;
            std::vector<float> px;
            for (size_t i = 0; i < (size_t)W * H; ++i) {
                const float *a = &uvNear[i * 3], *b = &uvExact[i * 3];
                if (!leaves[i]) continue;
                ++both;
                const bool an = std::fmod(a[2], 2.f) > 0.5f, bn = std::fmod(b[2], 2.f) > 0.5f;
                nearOn += an; exactOn += bn;
                if (an && bn) px.push_back(std::hypot((a[0] - b[0]) * (float)W, (a[1] - b[1]) * (float)H));
            }
            std::sort(px.begin(), px.end());
            auto q = [&](double at) { return px.empty() ? 0.f : px[(size_t)(at * (double)(px.size() - 1))]; };
            std::printf("CHECK 3: of %ld glass pixels whose line leaves through the far side, it is looked up on screen for %.1f%% (taken %.1f units away) and %.1f%% (true floor and backdrop). Where both are on screen (%zu): the two places are %.0f pixels apart (median), %.0f (9 in 10 within), %.0f (worst)\n",
                        both, 100.0 * (double)nearOn / (double)std::max(1L, both), envf("BEYOND", 3.0f), 100.0 * (double)exactOn / (double)std::max(1L, both), px.size(), q(0.5), q(0.9), q(1.0));
        }
    }

    // ---- CHECK=1 (model D): does three-mesh-bvh's tree, walked in a Filament material, give what the mesh itself gives?
    // The CPU side below is a test twin: every triangle tried, the same rule as band_bvh.mat (flat normal of the
    // triangle met, start 0.001 along the line). Geometry only: it says nothing about how the picture should look.
    if (std::getenv("CHECK") && model == "D") {
        const float cap = 32.f;
        bvh_mi->setParameter("most", cap);
        auto numbers = [&](float showMode, std::vector<float>& outv) {
            bvh_mi->setParameter("show", showMode);
            readGlass = true;
            for (int i = 0; i < 2; ++i) { frame(false); engine->flushAndWait(); }
            readGlass = false;
            outv = full;
        };
        std::vector<float> g1, g2, g3;
        numbers(1.f, g1);
        numbers(2.f, g2);
        numbers(3.f, g3);
        bvh_mi->setParameter("show", 0.f);
        bvh_mi->setParameter("most", envf("MOST", 5.f));

        struct Hit { float t; V3 n; uint32_t c[3]; V3 smooth; };
        auto first_hit = [&](V3 o, V3 d) {
            Hit best{1e30f, {0, 0, 0}, {0, 0, 0}, {0, 0, 0}};
            for (size_t i = 0; i + 2 < allIdx.size(); i += 3) {
                const V3 a = allPos[allIdx[i]], e1 = allPos[allIdx[i + 1]] - a, e2 = allPos[allIdx[i + 2]] - a;
                const V3 pv = cross(d, e2);
                const float det = dot(e1, pv);
                if (std::fabs(det) < 1e-20f) continue;
                const V3 tv = o - a;
                const float u = dot(tv, pv) / det;
                if (u < -1e-5f || u > 1.00001f) continue;
                const V3 qv = cross(tv, e1);
                const float v = dot(d, qv) / det;
                if (v < -1e-5f || u + v > 1.00001f) continue;
                const float t = dot(e2, qv) / det;
                if (t >= -1e-5f && t < best.t) {
                    V3 fn = norm(cross(e1, e2));
                    if (dot(fn, d) > 0) fn = fn * -1.f;   // (facing the line, as three-mesh-bvh gives it)
                    best = Hit{t, fn, {allIdx[i], allIdx[i + 1], allIdx[i + 2]}, {u, v, 0}};
                }
            }
            return best;
        };
        std::vector<V3> allNrm;
        for (const Built& m : cpuNear) allNrm.insert(allNrm.end(), m.nrm.begin(), m.nrm.end());
        const float3 f3 = normalize(target - eye), r3 = normalize(cross(f3, float3{0, 1, 0})), u3 = cross(r3, f3);
        const V3 F{f3.x, f3.y, f3.z}, Rr{r3.x, r3.y, r3.z}, U{u3.x, u3.y, u3.z}, E{eye.x, eye.y, eye.z};
        const float th = std::tan(0.5f * fov * PI / 180.f), asp = (float)W / (float)H, n = envf("IOR", 1.5f);
        const int MOST = (int)cap, every = (int)envf("CHECKEVERY", 12.f);
        for (int flip = 0; flip < 2; ++flip) {   // (the numbers may come back upside down: the way that agrees is the right one)
            long lines = 0, gpuNone = 0, sameTri = 0, sameCount = 0, cpuIn = 0, gpuIn = 0;
            std::vector<float> lenErr, angErr, wayErr;
            std::vector<long> cpuAfter((size_t)MOST + 3, 0), gpuAfter((size_t)MOST + 3, 0);
            auto slot = [&](float c) { return c < -0.5f ? (size_t)MOST + 2 : c > 98.f ? (size_t)MOST + 1 : (size_t)std::lround(c); };
            for (int y = every / 2; y < H; y += every)
                for (int x = every / 2; x < W; x += every) {
                    const float ndx = ((float)x + 0.5f) / (float)W * 2.f - 1.f, ndy = ((float)y + 0.5f) / (float)H * 2.f - 1.f;
                    const V3 d = norm(F + Rr * (ndx * th * asp) + U * (ndy * th));
                    const Hit h1 = first_hit(E, d);
                    if (h1.t > 1e29f) continue;
                    // (the way in: the smooth normal, as the drawn surface has it)
                    V3 N = norm(allNrm[h1.c[0]] * (1 - h1.smooth.x - h1.smooth.y) + allNrm[h1.c[1]] * h1.smooth.x + allNrm[h1.c[2]] * h1.smooth.y);
                    if (dot(N, d) > 0) N = N * -1.f;
                    const float ci = -dot(N, d), k = 1.f - (1.f - ci * ci) / (n * n);
                    V3 T = norm(d * (1.f / n) + N * (ci / n - std::sqrt(std::max(k, 0.f))));
                    V3 O = E + d * h1.t + T * 0.001f;
                    float count = 99.f, way = 0, firstT = -1;
                    V3 firstN{0, 0, 0};
                    uint32_t firstC[3] = {0, 0, 0};
                    for (int b = 0; b <= MOST; ++b) {
                        const Hit h = first_hit(O, T);
                        if (h.t > 1e29f) { count = -1.f; break; }
                        way += h.t + 0.001f;
                        if (b == 0) { firstT = h.t + 0.001f; firstN = h.n; std::memcpy(firstC, h.c, sizeof firstC); }
                        const float co = -dot(h.n, T), kk = 1.f - n * n * (1.f - co * co);
                        if (kk >= 0) { count = (float)b; break; }
                        const V3 at = O + T * h.t;
                        T = norm(T + h.n * (2.f * co));
                        O = at + T * 0.001f;
                    }
                    ++lines;
                    const size_t gi = ((size_t)(flip ? H - 1 - y : y) * W + x) * 4;
                    if (!(g1[gi] > 0) && !(g3[gi] < -0.5f)) { ++gpuNone; continue; }   // (the picture has no glass in this pixel)
                    ++cpuAfter[slot(count)];
                    ++gpuAfter[slot(g3[gi])];
                    if (count == g3[gi]) ++sameCount;
                    if (count > 98.f) ++cpuIn;
                    if (g3[gi] > 98.f) ++gpuIn;
                    if (firstT < 0 || !(g1[gi] > 0)) continue;
                    uint32_t a[3] = {firstC[0], firstC[1], firstC[2]}, c[3] = {(uint32_t)std::lround(g2[gi]), (uint32_t)std::lround(g2[gi + 1]), (uint32_t)std::lround(g2[gi + 2])};
                    std::sort(a, a + 3);
                    std::sort(c, c + 3);
                    if (a[0] == c[0] && a[1] == c[1] && a[2] == c[2]) ++sameTri;
                    lenErr.push_back(std::fabs(g1[gi] - firstT) / firstT);
                    const V3 gn{g1[gi + 1], g1[gi + 2], g3[gi + 2]};
                    angErr.push_back(std::acos(std::clamp(dot(norm(gn), firstN), -1.f, 1.f)) * 180.f / PI);
                    if (count == g3[gi] && count >= 0 && count < 98.f) wayErr.push_back(std::fabs(g3[gi + 1] - way) / way);
                }
            std::sort(lenErr.begin(), lenErr.end());
            std::sort(angErr.begin(), angErr.end());
            std::sort(wayErr.begin(), wayErr.end());
            auto q = [](const std::vector<float>& v, double at) { return v.empty() ? 0.f : v[(size_t)(at * (double)(v.size() - 1))]; };
            const long both = lines - gpuNone;
            std::printf("CHECK D (%s): %ld sight lines into the glass (CPU, every triangle tried), %ld of them not glass in the picture.\n", flip ? "numbers upside down" : "numbers as they are", lines, gpuNone);
            std::printf("    first surface inside: the same triangle for %.2f%% (%ld of %zu); way to it off by %.4f%% (median), %.4f%% (9 in 10), %.3f%% (worst); its normal off by %.3f deg (median), %.3f (9 in 10), %.2f (worst)\n",
                        100.0 * (double)sameTri / (double)std::max<size_t>(1, lenErr.size()), sameTri, lenErr.size(), 100.f * q(lenErr, 0.5), 100.f * q(lenErr, 0.9), 100.f * q(lenErr, 1.0), q(angErr, 0.5), q(angErr, 0.9), q(angErr, 1.0));
            std::printf("    mirrorings before leaving: the same number for %.2f%% of %ld lines; whole way inside off by %.4f%% (median), %.3f%% (9 in 10) where the number is the same\n",
                        100.0 * (double)sameCount / (double)std::max(1L, both), both, 100.f * q(wayErr, 0.5), 100.f * q(wayErr, 0.9));
            std::printf("    mirrorings   CPU      GPU (three-mesh-bvh in Filament)\n");
            for (int b = 0; b <= MOST + 2; ++b) {
                if (!cpuAfter[(size_t)b] && !gpuAfter[(size_t)b]) continue;
                char name[40];
                if (b == MOST + 1) std::snprintf(name, sizeof name, "still inside after %d", MOST);
                else if (b == MOST + 2) std::snprintf(name, sizeof name, "no surface ahead");
                else std::snprintf(name, sizeof name, "%d", b);
                std::printf("    %-22s %6.2f%%  %6.2f%%\n", name, 100.0 * (double)cpuAfter[(size_t)b] / (double)std::max(1L, both), 100.0 * (double)gpuAfter[(size_t)b] / (double)std::max(1L, both));
            }
        }
        // every glass pixel of the picture (GPU only): how many are still inside at each limit of mirrorings
        {
            std::vector<long> after((size_t)MOST + 3, 0);
            long glass = 0;
            for (size_t i = 0; i < (size_t)W * H; ++i) {
                const float c = g3[i * 4];
                if (!(g1[i * 4] > 0) && !(c < -0.5f)) continue;
                ++glass;
                ++after[c < -0.5f ? (size_t)MOST + 2 : c > 98.f ? (size_t)MOST + 1 : (size_t)std::lround(c)];
            }
            std::printf("CHECK D, every glass pixel (%ld, GPU): out within N mirrorings (the rest is NOT out):", glass);
            long sum = 0;
            for (int b = 0; b <= MOST; ++b) {
                sum += after[(size_t)b];
                if (b <= 5 || b == 8 || b == 16 || b == MOST) std::printf("  %d: %.2f%%", b, 100.0 * (double)sum / (double)std::max(1L, glass));
            }
            std::printf("  | still inside after %d: %.2f%%, no surface ahead: %.2f%%\n", MOST, 100.0 * (double)after[(size_t)MOST + 1] / (double)std::max(1L, glass), 100.0 * (double)after[(size_t)MOST + 2] / (double)std::max(1L, glass));
        }
    }

    if (const char* t = std::getenv("TIME")) {   // n frames, each waited for to its end
        const int n = std::atoi(t);
        int drawn = 0;
        const auto t0 = std::chrono::steady_clock::now();
        for (int i = 0; i < n; ++i) {
            if (frame(false)) ++drawn;
            engine->flushAndWait();   // (every frame waited for to its end: CPU and GPU together, nothing queued behind)
        }
        engine->flushAndWait();
        const double ms = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
        std::printf("TIME: model %s, %.2f ms a frame (%d frames, %dx%d, %zu bands)\n", model.c_str(), ms / std::max(1, drawn), drawn, W, H, bands.size());
    }
    return 0;
}
