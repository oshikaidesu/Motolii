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
// ./build_one.sh band && ./out/band [scene]     scene: ref (default), loops, dark    -> out/band-<scene>.png
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
static Built tube(const std::vector<Sample>& s, int sides, float wide, float thick, float round, float rib, bool inside) {
    Built m;
    int rings = (int)s.size();
    auto at = [&](float ang, float& x, float& y) {
        float c = std::cos(ang), sn = std::sin(ang);
        y = (sn < 0 ? -1.f : 1.f) * std::pow(std::fabs(sn), 2.f / round);
        x = (c < 0 ? -1.f : 1.f) * std::pow(std::fabs(c), 2.f / round) * (1.f + rib * std::exp(-(y * y) / 0.10f));
        x *= thick;
        y *= wide;
    };
    for (int i = 0; i < rings; ++i) {
        for (int k = 0; k < sides; ++k) {
            float ang = (float)k / (float)sides * TAU, x, y, x0, y0, x1, y1;
            at(ang, x, y);
            at(ang - 0.002f, x0, y0);
            at(ang + 0.002f, x1, y1);
            V3 off = s[i].N * x + s[i].B * y;
            V3 nrm = norm(s[i].N * (y1 - y0) - s[i].B * (x1 - x0));   // the outline's tangent turned a quarter, outward
            pack(m.v, s[i].p + off, s[i].T, inside ? nrm * -1.f : nrm);
        }
    }
    for (int i = 0; i < rings; ++i) {
        int i1 = (i + 1) % rings;
        for (int k = 0; k < sides; ++k) {
            int k1 = (k + 1) % sides;
            uint32_t a = (uint32_t)(i * sides + k), b = (uint32_t)(i * sides + k1);
            uint32_t c = (uint32_t)(i1 * sides + k), d = (uint32_t)(i1 * sides + k1);
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
    const GlassLook look;
    const int per = 96, sides = 96;

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
    for (const Band& b : bands) {
        std::vector<Sample> path = close_frames(catmull(b.pts), per);
        for (Sample& q : path) {
            q.N = norm(rot_about(q.N, q.T, b.twist));
            q.B = norm(cross(q.T, q.N));
        }
        farSides.push_back(add(tube(path, sides, b.wide, b.wide * look.slab, look.round, look.rib, true), far_mi, false));
        nearSides.push_back(add(tube(path, sides, b.wide, b.wide * look.slab, look.round, look.rib, false), near_mi, false));
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
        view->setScreenSpaceRefractionEnabled(true);
        rm.setMaterialInstanceAt(rm.getInstance(plate), 0, plate_mi[0]);
        show({plate}, farSides);                           // 2. + the far sides
        view->setRenderTarget(rtF[1]);
        renderer->render(view);
        rm.setMaterialInstanceAt(rm.getInstance(plate), 0, plate_mi[1]);
        show({plate}, nearSides);                          // 3. + the near sides
        view->setRenderTarget(rtF[0]);
        renderer->render(view);
        bloom.enabled = true;
        view->setBloomOptions(bloom);
        view->setPostProcessingEnabled(true);
        view->setAntiAliasing(View::AntiAliasing::FXAA);
        view->setScreenSpaceRefractionEnabled(false);
        rm.setMaterialInstanceAt(rm.getInstance(plate), 0, plate_mi[0]);
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
    const std::string out = "out/band-" + which + ".png";
    if (!write_png(out.c_str(), W, H, pic.data())) return fail("png");
    std::printf("wrote %s  (%zu band(s))\n", out.c_str(), bands.size());

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
        std::printf("TIME: %.2f ms a frame (%d frames, %dx%d, %zu bands, 4 passes)\n", ms / std::max(1, drawn), drawn, W, H, bands.size());
    }
    return 0;
}
