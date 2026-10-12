// band.mm: one thick band of clear glass along a closed 3D path, close up, in a pale high-key studio.
// (A picture test after a reference the user gave on 2026-10-12: pale grey ground, a broad curved glass band, soft
// rainbow inside, bright glints.) Everything that draws is Filament as it is: its own screen-space refraction with
// dispersion (glass_layer.mat), the bundled studio light, bloom, depth of field, tone mapping. The path helpers and
// the tube are hero.mm's; the tube here takes a flattened cross-section instead of a circle.
// ./build_one.sh band && ./out/band        -> out/band.png        (knobs: see the getenv lines in main)
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

// A band: the cross-section is a rounded slab, `wide` across (along B) and `thick` through (along N); `round`: 2 is an
// ellipse, more is flatter faces with tighter edges.
static Built tube(const std::vector<Sample>& s, int sides, float wide, float thick, float round) {
    Built m;
    int rings = (int)s.size();
    for (int i = 0; i < rings; ++i) {
        for (int k = 0; k < sides; ++k) {
            float ang = (float)k / (float)sides * TAU;
            float c = std::cos(ang), sn = std::sin(ang);
            float x = (c < 0 ? -1.f : 1.f) * std::pow(std::fabs(c), 2.f / round), y = (sn < 0 ? -1.f : 1.f) * std::pow(std::fabs(sn), 2.f / round);
            V3 off = s[i].N * (x * thick) + s[i].B * (y * wide);
            V3 nrm = s[i].N * ((x < 0 ? -1.f : 1.f) * std::pow(std::fabs(x), round - 1.f) / thick) +
                     s[i].B * ((y < 0 ? -1.f : 1.f) * std::pow(std::fabs(y), round - 1.f) / wide);
            pack(m.v, s[i].p + off, s[i].T, norm(nrm));
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

int main() {
    const int per = 96, sides = 96;
    // the path: one big loop, standing a little tilted; the picture looks at one stretch of it from close by
    const float R = envf("LOOP", 1.6f);
    std::vector<V3> pts;
    for (int i = 0; i < 8; ++i) {
        float a = (float)i / 8.f * TAU;
        pts.push_back(V3{R * std::cos(a), R * 0.92f * std::sin(a) + R * 0.86f, 0.22f * std::sin(2.f * a + 0.6f)});
    }
    std::vector<Sample> path = close_frames(catmull(pts), per);
    const float twist = envf("TWIST", 1.05f);   // the band's flat side turned about the path (radians)
    for (Sample& q : path) {
        q.N = norm(rot_about(q.N, q.T, twist));
        q.B = norm(cross(q.T, q.N));
    }

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    utils::Entity cam_e = utils::EntityManager::get().create();
    Camera* camera = engine->createCamera(cam_e);
    camera->setExposure(envf("FSTOP", 2.8f), 1.0f / envf("SHUTTER", 2200.f), 100.0f);
    const float3 eye{envf("EYEX", -0.35f), envf("EYEY", 0.62f), envf("EYEZ", 1.55f)};
    const float3 target{envf("ATX", -0.62f), envf("ATY", 0.30f), envf("ATZ", 0.0f)};
    camera->lookAt(eye, target, {0, 1, 0});
    camera->setProjection(envf("FOV", 26.f), double(W) / double(H), 0.05, 60.0, Camera::Fov::VERTICAL);
    camera->setFocusDistance(envf("FOCUS", length(target - eye)));

    static AgxToneMapper agx(AgxToneMapper::AgxLook::NONE);
    ColorGrading* grading = ColorGrading::Builder().toneMapper(&agx).contrast(envf("CONTRAST", 1.0f)).saturation(envf("SAT", 1.15f)).build(*engine);
    view->setScene(scene);
    view->setCamera(camera);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setColorGrading(grading);
    view->setPostProcessingEnabled(true);
    view->setScreenSpaceRefractionEnabled(true);
    view->setAntiAliasing(View::AntiAliasing::FXAA);
    view->setDithering(View::Dithering::NONE);
    MultiSampleAntiAliasingOptions msaa{};
    msaa.enabled = true;
    msaa.sampleCount = 4;
    view->setMultiSampleAntiAliasingOptions(msaa);
    BloomOptions bloom{};
    bloom.enabled = true;
    bloom.strength = envf("BLOOM", 0.22f);
    bloom.levels = 8;
    bloom.resolution = 1080;
    bloom.threshold = true;
    bloom.highlight = 20.0f;
    bloom.starburst = true;
    bloom.blendMode = BloomOptions::BlendMode::ADD;
    view->setBloomOptions(bloom);
    DepthOfFieldOptions dof{};
    dof.enabled = envf("DOF", 1.f) > 0.5f;
    dof.cocScale = envf("COC", 6.0f);
    view->setDepthOfFieldOptions(dof);
    Renderer::ClearOptions clear{};
    clear.clearColor = {0.8f, 0.8f, 0.82f, 1};
    clear.clear = true;
    renderer->setClearOptions(clear);

    auto [color8, rt8] = std::tuple<Texture*, RenderTarget*>{nullptr, nullptr};
    {
        Texture* color = Texture::Builder().width(W).height(H).levels(1)
                             .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                             .format(Texture::InternalFormat::RGBA8).build(*engine);
        Texture* depth = Texture::Builder().width(W).height(H).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH24).build(*engine);
        color8 = color;
        rt8 = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, color).texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
    }
    if (!color8 || !rt8) return fail("target");

    // the bundled studio (the one glass_scene.mm uses): its light and its reflections; it is not shown as a background
    auto ibl_bytes = load_file("out/studio/_ibl.ktx");
    if (ibl_bytes.empty()) return fail("out/studio/_ibl.ktx");
    Texture* ibl_tex = ktxreader::Ktx1Reader::createTexture(engine, new image::Ktx1Bundle(ibl_bytes.data(), (uint32_t)ibl_bytes.size()), false);
    float3 sh[9];
    if (!ibl_tex || !load_sh("out/studio/sh.txt", sh)) return fail("studio");
    IndirectLight* ibl = IndirectLight::Builder().reflections(ibl_tex).irradiance(3, sh).intensity(envf("IBL", 42000.f))
                             .rotation(mat3f::rotation(envf("IBLTURN", 0.6f), float3{0, 1, 0})).build(*engine);
    scene->setIndirectLight(ibl);

    auto layer_pkg = load_file("band_glass.filamat");
    auto floor_pkg = load_file("floor_lit.filamat");
    auto back_pkg = load_file("backdrop_grad.filamat");
    auto emit_pkg = load_file("emissive.filamat");
    if (layer_pkg.empty() || floor_pkg.empty() || back_pkg.empty() || emit_pkg.empty()) return fail("filamat");
    Material* layer_mat = Material::Builder().package(layer_pkg.data(), layer_pkg.size()).build(*engine);
    Material* floor_mat = Material::Builder().package(floor_pkg.data(), floor_pkg.size()).build(*engine);
    Material* back_mat = Material::Builder().package(back_pkg.data(), back_pkg.size()).build(*engine);
    Material* emit_mat = Material::Builder().package(emit_pkg.data(), emit_pkg.size()).build(*engine);
    if (!layer_mat || !floor_mat || !back_mat || !emit_mat) return fail("material");

    MaterialInstance* glass_mi = layer_mat->createInstance();
    glass_mi->setParameter("ior", envf("IOR", 1.52f));
    glass_mi->setParameter("transmission", 1.0f);
    glass_mi->setParameter("thickness", envf("THICK", 0.22f));
    glass_mi->setParameter("dispersion", envf("DISP", 6.f));
    glass_mi->setParameter("reflectance", envf("REFL", 0.5f));
    glass_mi->setParameter("sheen", envf("SHEEN", 1.0f));
    glass_mi->setParameter("film", float2{envf("FILM0", 280.f), envf("FILM1", 620.f)});
    glass_mi->setParameter("filmWave", envf("FILMWAVE", 2.2f));
    glass_mi->setParameter("absorption", float3{envf("ABS", 0.02f) * 1.6f, envf("ABS", 0.02f) * 0.9f, envf("ABS", 0.02f) * 0.5f});
    MaterialInstance* floor_mi = floor_mat->createInstance();
    const float fl = envf("FLOORCOL", 0.78f);
    floor_mi->setParameter("color", float4{fl, fl + 0.01f, fl + 0.04f, 1});
    floor_mi->setParameter("roughness", envf("FLOORROUGH", 0.22f));
    MaterialInstance* back_mi = back_mat->createInstance();
    const float lift = envf("BACK", 26000.f);
    const float lowK = envf("BACKLOW", 0.52f);
    back_mi->setParameter("low", float4{lowK * lift, (lowK + 0.01f) * lift, (lowK + 0.04f) * lift, 1});
    back_mi->setParameter("high", float4{1.0f * lift, 1.0f * lift, 1.0f * lift, 1});
    back_mi->setParameter("bend", envf("BACKBEND", 0.8f));

    struct Held { Built mesh; VertexBuffer* vb; IndexBuffer* ib; utils::Entity e; };
    std::vector<Held*> held;
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
            .culling(false).castShadows(false).receiveShadows(false).build(*engine, h->e);
        scene->addEntity(h->e);
        held.push_back(h);
        return h;
    };
    // a quad with uv (0..1 across and up), for the backdrop
    auto quad_uv = [&](V3 center, V3 right, V3 up, float hw, float hh) {
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
    add(quad({0, 0, 0}, {1, 0, 0}, {0, 0, -1}, 40.f, 40.f), floor_mi, false);
    add(quad_uv({0, 6.f, -9.f}, {1, 0, 0}, {0, 1, 0}, 40.f, 9.f), back_mi, true);
    MaterialInstance* clay_mi = floor_mat->createInstance();   // SOLID=1: the band as grey clay, to look at shape and framing
    clay_mi->setParameter("color", float4{0.25f, 0.27f, 0.32f, 1});
    clay_mi->setParameter("roughness", 0.35f);
    add(tube(path, sides, envf("WIDE", 0.34f), envf("SLAB", 0.085f), envf("ROUND", 3.2f)), envf("SOLID", 0.f) > 0.5f ? clay_mi : glass_mi, false);

    auto lamp = [&](float3 color, float lux, float3 dir) {
        utils::Entity e = utils::EntityManager::get().create();
        LightManager::Builder(LightManager::Type::SUN).color(color).intensity(lux).direction(normalize(dir)).castShadows(false)
            .sunAngularRadius(0.6f).sunHaloSize(2.0f).build(*engine, e);
        scene->addEntity(e);
    };
    lamp({1.0f, 0.98f, 0.95f}, envf("SUN", 110000.f), {envf("SUNX", -0.55f), envf("SUNY", -0.70f), envf("SUNZ", -0.45f)});

    std::vector<uint8_t> pic((size_t)W * H * 4);
    for (int frame = 0; frame < 3; ++frame) {
        bool ok = false;
        for (int spin = 0; spin < 4000 && !ok; ++spin) ok = renderer->beginFrame(swap);
        if (!ok) return fail("begin");
        view->setRenderTarget(rt8);
        renderer->render(view);
        if (frame == 2)
            renderer->readPixels(rt8, 0, 0, (uint32_t)W, (uint32_t)H,
                                 backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
        renderer->endFrame();
        engine->flushAndWait();   // (or the next beginFrame may say "still busy" for longer than we ask)
    }
    const char* name = std::getenv("NAME");
    std::string out = std::string("out/band") + (name ? std::string("-") + name : std::string()) + ".png";
    if (!write_png(out.c_str(), W, H, pic.data())) return fail("png");
    std::printf("wrote %s\n", out.c_str());
    return 0;
}
