// Three stills from the same path and the same Filament path as hero.mm.
// clear: dark ground, thick clear glass. prism: striped ground, strong dispersion.
// chrome: a built cubemap, two metals, glass in front of them.
// Three glasses, back to front, each in its own render() inside one frame.
// The previous color is an opaque plate on an RGBA16F target, so Filament's
// solid refraction bends an image that already holds the glasses behind it.
// AgX and bloom run once, on the last pass.
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

static Built tube(const std::vector<Sample>& s, int sides, float radius, bool zero_pos) {
    Built m;
    int rings = (int)s.size();
    for (int i = 0; i < rings; ++i) {
        for (int k = 0; k < sides; ++k) {
            float ang = (float)k / (float)sides * TAU;
            V3 radial = norm(s[i].N * std::cos(ang) + s[i].B * std::sin(ang));
            pack(m.v, zero_pos ? V3{0, 0, 0} : s[i].p + radial * radius, s[i].T, radial);
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

static Built ribbon(const std::vector<Sample>& s, float half_width) {
    Built m;
    int rings = (int)s.size();
    for (int i = 0; i < rings; ++i) {
        V3 width = s[i].N;
        V3 face = norm(cross(s[i].T, width));
        pack(m.v, s[i].p - width * half_width, s[i].T, face);
        pack(m.v, s[i].p + width * half_width, s[i].T, face);
    }
    for (int i = 0; i < rings; ++i) {
        int i1 = (i + 1) % rings;
        uint32_t a = (uint32_t)(i * 2), b = a + 1, c = (uint32_t)(i1 * 2), d = c + 1;
        m.idx.insert(m.idx.end(), {a, b, c, b, d, c, a, c, b, b, c, d});
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

int main(int argc, char** argv) {
    const char* which = argc > 1 ? argv[1] : "clear";
    const bool prism = std::strcmp(which, "prism") == 0;
    const bool chrome_shot = std::strcmp(which, "chrome") == 0;
    const bool clear_shot = !prism && !chrome_shot;
    const char* out_path = clear_shot ? "out/look-clear.png" : prism ? "out/look-prism.png" : "out/look-chrome.png";

    const int per = 32;
    const int sides = 36;
    std::vector<Cubic> spans0 = the_path();
    std::vector<Sample> path = close_frames(spans0, per);
    float back_scale = 1.30f, back_z = -0.26f, back_tilt = 0.58f, back_r = 0.070f;
    float mid_tilt = -0.36f, mid_r = 0.052f;
    float front_scale = 0.46f, front_z = 0.20f, front_tilt = 0.20f, front_r = 0.036f;
    float chrome_nudge = 0.15f, chrome_r = 0.012f;
    float chrome_aniso = 0.55f, chrome_rough = 0.45f;
    float back_ior = 1.95f, back_thick = 1.55f, back_disp = 0.4f, back_refl = 0.62f;
    float3 back_abs{0.012f, 0.012f, 0.018f};
    float front_ior = 2.25f, front_thick = 1.35f, front_disp = 0.3f, front_refl = 0.58f;
    float3 front_abs{0.012f, 0.014f, 0.012f};
    float gpu_ior = 2.10f, gpu_thick = 1.45f, gpu_disp = 0.5f, gpu_refl = 0.55f;
    float bloom_strength = 0.10f;
    float sun_lux = 28000.f;
    float sun_radius = 0.35f;
    float ibl_intensity = 3500.f;
    bool front_is_ribbon = false;
    int nglass = 3;
    if (prism) {
        back_scale = 1.24f; back_z = -0.22f; back_tilt = 0.72f; back_r = 0.046f;
        mid_tilt = -0.50f; mid_r = 0.034f;
        front_scale = 0.55f; front_z = 0.14f; front_tilt = 0.34f; front_r = 0.055f;
        chrome_nudge = 0.11f; chrome_r = 0.010f;
        chrome_aniso = 0.2f; chrome_rough = 0.55f;
        back_ior = 1.75f; back_thick = 1.25f; back_disp = 16.f; back_refl = 0.28f;
        back_abs = {0.015f, 0.10f, 0.62f};
        front_ior = 2.35f; front_thick = 1.15f; front_disp = 18.f; front_refl = 0.26f;
        front_abs = {0.55f, 0.03f, 0.02f};
        gpu_ior = 2.05f; gpu_thick = 1.20f; gpu_disp = 20.f; gpu_refl = 0.22f;
        bloom_strength = 0.16f;
        sun_lux = 42000.f;
        ibl_intensity = 2800.f;
        front_is_ribbon = true;
    } else if (chrome_shot) {
        back_scale = 1.20f; back_z = -0.22f; back_tilt = 0.48f; back_r = 0.046f;
        mid_tilt = -0.18f; mid_r = 0.040f;
        front_scale = 0.44f; front_z = 0.24f; front_tilt = 0.36f; front_r = 0.020f;
        chrome_nudge = 0.0f; chrome_r = 0.016f;
        chrome_aniso = 0.88f; chrome_rough = 0.28f;
        back_ior = 1.6f; back_thick = 0.4f; back_disp = 1.f; back_refl = 0.2f;
        gpu_ior = 1.85f; gpu_thick = 1.10f; gpu_disp = 3.5f; gpu_refl = 0.34f;
        front_ior = 2.05f; front_thick = 0.85f; front_disp = 5.f; front_refl = 0.30f;
        front_abs = {0.20f, 0.04f, 0.02f};
        bloom_strength = 0.20f;
        sun_lux = 70000.f;
        sun_radius = 0.40f;
        ibl_intensity = 5000.f;
        nglass = 2;
    }

#include "trio_2.inc"
#include "trio_3.inc"
