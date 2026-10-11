// One closed 3D path, five stills. ThorVG is not in this cut: the curve is the
// control points already proven against ThorVG. Filament lights, IBL from cmgen,
// and the existing glass materials make the picture.
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

#include <clocale>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <functional>
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
    V3 operator-() const { return {-x, -y, -z}; }
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
static std::vector<Sample> nudged(std::vector<Sample> s, float along_n, float along_b) {
    for (Sample& a : s) a.p = a.p + a.N * along_n + a.B * along_b;
    return s;
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

static float closed_len(const std::vector<Sample>& s) {
    return s.back().s + len(s.front().p - s.back().p);
}

static Built tube(const std::vector<Sample>& s, int sides, const std::function<float(float)>& radius) {
    Built m;
    int rings = (int)s.size();
    float L = closed_len(s);
    for (int i = 0; i < rings; ++i) {
        float u = s[i].s / L;
        float r = radius(u);
        for (int k = 0; k < sides; ++k) {
            float ang = (float)k / (float)sides * TAU;
            V3 radial = norm(s[i].N * std::cos(ang) + s[i].B * std::sin(ang));
            pack(m.v, s[i].p + radial * r, s[i].T, radial);
        }
    }
    for (int i = 0; i < rings; ++i) {
        int i1 = (i + 1) % rings;
        for (int k = 0; k < sides; ++k) {
            int k1 = (k + 1) % sides;
            uint32_t a = (uint32_t)(i * sides + k);
            uint32_t b = (uint32_t)(i * sides + k1);
            uint32_t c = (uint32_t)(i1 * sides + k);
            uint32_t d = (uint32_t)(i1 * sides + k1);
            m.idx.insert(m.idx.end(), {a, b, c, b, d, c});
        }
    }
    return m;
}

static Built ribbon(const std::vector<Sample>& s, float turns, const std::function<float(float)>& half) {
    Built m;
    int rings = (int)s.size();
    float L = closed_len(s);
    for (int i = 0; i < rings; ++i) {
        float u = s[i].s / L;
        float ang = turns * TAU * u;
        V3 width = norm(s[i].N * std::cos(ang) + s[i].B * std::sin(ang));
        V3 face = norm(cross(s[i].T, width));
        float h = half(u);
        pack(m.v, s[i].p - width * h, s[i].T, face);
        pack(m.v, s[i].p + width * h, s[i].T, face);
    }
    for (int i = 0; i < rings; ++i) {
        int i1 = (i + 1) % rings;
        uint32_t a = (uint32_t)(i * 2), b = a + 1, c = (uint32_t)(i1 * 2), d = c + 1;
        m.idx.insert(m.idx.end(), {a, b, c, b, d, c});
    }
    return m;
}

static Built sweep_rect(const std::vector<Sample>& s, float turns, const std::function<float(float)>& width,
                        const std::function<float(float)>& height) {
    Built m;
    int rings = (int)s.size();
    float L = closed_len(s);
    for (int i = 0; i < rings; ++i) {
        float u = s[i].s / L;
        float ang = turns * TAU * u;
        V3 Nr = norm(s[i].N * std::cos(ang) + s[i].B * std::sin(ang));
        V3 Br = norm(cross(s[i].T, Nr));
        float hw = width(u) * 0.5f, hh = height(u) * 0.5f;
        V3 corner[4] = {s[i].p + Nr * hw + Br * hh, s[i].p - Nr * hw + Br * hh, s[i].p - Nr * hw - Br * hh,
                        s[i].p + Nr * hw - Br * hh};
        V3 face[4] = {Br, -Nr, -Br, Nr};
        for (int k = 0; k < 4; ++k) pack(m.v, corner[k], s[i].T, face[k]);
    }
    for (int i = 0; i < rings; ++i) {
        int i1 = (i + 1) % rings;
        for (int k = 0; k < 4; ++k) {
            int k1 = (k + 1) % 4;
            uint32_t a = (uint32_t)(i * 4 + k);
            uint32_t b = (uint32_t)(i * 4 + k1);
            uint32_t c = (uint32_t)(i1 * 4 + k);
            uint32_t d = (uint32_t)(i1 * 4 + k1);
            m.idx.insert(m.idx.end(), {a, c, b, b, c, d});
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

struct Hold {
    Built mesh;
    utils::Entity entity;
    VertexBuffer* vb = nullptr;
    IndexBuffer* ib = nullptr;
};

static int fail(const char* msg) {
    std::printf("FAIL %s\n", msg);
    return 1;
}

int main() {
    std::vector<Sample> path = close_frames(the_path(), 32);
    float L = closed_len(path);
    float kappa = 0, sep = 1e9f;
    int n = (int)path.size();
    for (int i = 0; i < n; ++i) {
        int a = (i + n - 1) % n, b = (i + 1) % n;
        float ds = path[b].s - path[a].s;
        if (ds < 0) ds += L;
        if (ds > 1e-6f) kappa = std::max(kappa, std::acos(std::clamp(dot(path[a].T, path[b].T), -1.f, 1.f)) / ds);
    }
    for (int i = 0; i < n; ++i)
        for (int j = i + 1; j < n; ++j) {
            float ds = path[j].s - path[i].s;
            if (std::min(ds, L - ds) < 0.35f) continue;
            sep = std::min(sep, len(path[j].p - path[i].p));
        }
    const float glass_r = 0.050f;
    std::printf("path rings %d  length %.3f  kappa %.3f  r*kappa %.3f  separation %.3f  diameter %.3f\n", n, L, kappa,
                glass_r * kappa, sep, glass_r * 2.f);

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    utils::Entity cam_e = utils::EntityManager::get().create();
    Camera* camera = engine->createCamera(cam_e);
    camera->setExposure(16.0f, 1.0f / 125.0f, 100.0f);
    camera->lookAt({0.18, 0.40, 1.82}, {0.0, 0.02, 0.0}, {0, 1, 0});
    camera->setProjection(34.0, double(W) / double(H), 0.05, 40.0, Camera::Fov::VERTICAL);

    Texture* color = Texture::Builder()
                         .width(W)
                         .height(H)
                         .levels(1)
                         .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                         .format(Texture::InternalFormat::RGBA8)
                         .build(*engine);
    Texture* depth = Texture::Builder()
                         .width(W)
                         .height(H)
                         .levels(1)
                         .usage(Texture::Usage::DEPTH_ATTACHMENT)
                         .format(Texture::InternalFormat::DEPTH24)
                         .build(*engine);
    RenderTarget* rt = RenderTarget::Builder()
                           .texture(RenderTarget::AttachmentPoint::COLOR, color)
                           .texture(RenderTarget::AttachmentPoint::DEPTH, depth)
                           .build(*engine);

    static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
    ColorGrading* grading = ColorGrading::Builder().toneMapper(&agx).contrast(1.06f).saturation(1.14f).build(*engine);

    auto ibl_bytes = load_file("out/ibl/ibl_ibl.ktx");
    auto sky_bytes = load_file("out/ibl/ibl_skybox.ktx");
    if (ibl_bytes.empty() || sky_bytes.empty()) return fail("ktx");
    Texture* ibl_tex = ktxreader::Ktx1Reader::createTexture(engine, new image::Ktx1Bundle(ibl_bytes.data(), (uint32_t)ibl_bytes.size()), false);
    Texture* sky_tex = ktxreader::Ktx1Reader::createTexture(engine, new image::Ktx1Bundle(sky_bytes.data(), (uint32_t)sky_bytes.size()), false);
    if (!ibl_tex || !sky_tex) return fail("cubemap");
    float3 sh[9];
    if (!load_sh("out/ibl/sh.txt", sh)) return fail("sh");
    const float ibl_intensity = 9000.f;
    IndirectLight* ibl = IndirectLight::Builder().reflections(ibl_tex).irradiance(3, sh).intensity(ibl_intensity).build(*engine);
    Skybox* sky = Skybox::Builder().environment(sky_tex).showSun(false).build(*engine);
    scene->setIndirectLight(ibl);
    scene->setSkybox(sky);

    view->setScene(scene);
    view->setCamera(camera);
    view->setRenderTarget(rt);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setPostProcessingEnabled(true);
    view->setScreenSpaceRefractionEnabled(true);
    view->setAntiAliasing(View::AntiAliasing::FXAA);
    view->setDithering(View::Dithering::NONE);
    view->setColorGrading(grading);
    ScreenSpaceReflectionsOptions ssr;
    ssr.enabled = true;
    ssr.thickness = 0.12f;
    ssr.bias = 0.02f;
    ssr.maxDistance = 6.0f;
    ssr.stride = 2.0f;
    view->setScreenSpaceReflectionsOptions(ssr);
    GuardBandOptions guard;
    guard.enabled = true;
    view->setGuardBandOptions(guard);
    Renderer::ClearOptions clear{};
    clear.clearColor = {0.01f, 0.012f, 0.016f, 1};
    clear.clear = true;
    renderer->setClearOptions(clear);

    auto pkg = [](const char* p) { return load_file(p); };
    auto glass_pkg = pkg("glass.filamat");
    auto pop_pkg = pkg("glass_pop.filamat");
    auto chrome_pkg = pkg("chrome_aniso.filamat");
    auto emit_pkg = pkg("emissive.filamat");
    auto floor_pkg = pkg("floor_lit.filamat");
    if (glass_pkg.empty() || pop_pkg.empty() || chrome_pkg.empty() || emit_pkg.empty() || floor_pkg.empty()) return fail("filamat");
    Material* glass_mat = Material::Builder().package(glass_pkg.data(), glass_pkg.size()).build(*engine);
    Material* pop_mat = Material::Builder().package(pop_pkg.data(), pop_pkg.size()).build(*engine);
    Material* chrome_mat = Material::Builder().package(chrome_pkg.data(), chrome_pkg.size()).build(*engine);
    Material* emit_mat = Material::Builder().package(emit_pkg.data(), emit_pkg.size()).build(*engine);
    Material* floor_mat = Material::Builder().package(floor_pkg.data(), floor_pkg.size()).build(*engine);
    if (!glass_mat || !pop_mat || !chrome_mat || !emit_mat || !floor_mat) return fail("material");

    auto glass_of = [&](Material* mat, float ior, float thick, float dispersion) {
        MaterialInstance* mi = mat->createInstance();
        mi->setParameter("ior", ior);
        mi->setParameter("transmission", 1.0f);
        mi->setParameter("thickness", thick);
        mi->setParameter("dispersion", dispersion);
        return mi;
    };
    MaterialInstance* glass = glass_of(glass_mat, 2.15f, 0.85f, 1.0f);
    MaterialInstance* amber = glass_of(pop_mat, 1.85f, 0.48f, 0.55f);
    MaterialInstance* chrome = chrome_mat->createInstance();
    chrome->setParameter("anisotropy", 0.9f);
    chrome->setParameter("roughness", 0.28f);
    MaterialInstance* chrome_sharp = chrome_mat->createInstance();
    chrome_sharp->setParameter("anisotropy", 0.0f);
    chrome_sharp->setParameter("roughness", 0.28f);
    MaterialInstance* brushed = chrome_mat->createInstance();
    brushed->setParameter("anisotropy", 0.92f);
    brushed->setParameter("roughness", 0.32f);
    auto emit = [&](float r, float g, float b) {
        MaterialInstance* mi = emit_mat->createInstance();
        mi->setParameter("color", float4{r, g, b, 1});
        return mi;
    };
    MaterialInstance* core = emit(140000.f, 42000.f, 9000.f);
    MaterialInstance* filament_w = emit(90000.f, 98000.f, 70000.f);
    MaterialInstance* filament_c = emit(6000.f, 70000.f, 160000.f);
    MaterialInstance* bulb_mi = emit(480000.f, 420000.f, 320000.f);
    MaterialInstance* window_mi = emit(220000.f, 86000.f, 22000.f);
    MaterialInstance* slit_mi = emit(4000.f, 80000.f, 150000.f);
    auto floor_of = [&](float4 color, float rough) {
        MaterialInstance* mi = floor_mat->createInstance();
        mi->setParameter("color", color);
        mi->setParameter("roughness", rough);
        return mi;
    };
    MaterialInstance* floor_mi = floor_of({0.11f, 0.10f, 0.115f, 1}, 0.12f);
    MaterialInstance* panel_mi = floor_of({0.045f, 0.042f, 0.055f, 1}, 0.55f);
    MaterialInstance* wall_mi = floor_of({0.030f, 0.027f, 0.034f, 1}, 0.8f);
    MaterialInstance* mullion_mi = floor_of({0.012f, 0.008f, 0.006f, 1}, 0.9f);

    auto add = [&](Built mesh, MaterialInstance* mi) {
        Hold h;
        h.mesh = std::move(mesh);
        uint32_t nv = (uint32_t)(h.mesh.v.size() / 7);
        h.vb = VertexBuffer::Builder()
                   .vertexCount(nv)
                   .bufferCount(1)
                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                   .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                   .build(*engine);
        h.vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(h.mesh.v.data(), h.mesh.v.size() * sizeof(float)));
        h.ib = IndexBuffer::Builder().indexCount((uint32_t)h.mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        h.ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(h.mesh.idx.data(), h.mesh.idx.size() * sizeof(uint32_t)));
        h.entity = utils::EntityManager::get().create();
        RenderableManager::Builder(1)
            .boundingBox({{-4, -4, -4}, {4, 4, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, h.vb, h.ib)
            .material(0, mi)
            .culling(false)
            .castShadows(false)
            .receiveShadows(false)
            .build(*engine, h.entity);
        scene->addEntity(h.entity);
        return h;
    };
    auto drop = [&](std::vector<Hold>& hs) {
        for (Hold& h : hs) {
            scene->remove(h.entity);
            engine->destroy(h.entity);
            engine->destroy(h.vb);
            engine->destroy(h.ib);
            utils::EntityManager::get().destroy(h.entity);
        }
        hs.clear();
    };

    std::vector<Hold> room;
    room.push_back(add(quad({0, -0.64f, 0.15f}, {1, 0, 0}, {0, 0, -1}, 2.8f, 2.2f), floor_mi));
    room.push_back(add(quad({0.0f, 0.35f, -1.35f}, {1, 0, 0}, {0, 1, 0}, 2.4f, 1.5f), wall_mi));
    room.push_back(add(quad({-1.05f, 0.25f, 0.05f}, {0, 0, -1}, {0, 1, 0}, 1.15f, 1.2f), panel_mi));
    room.push_back(add(quad({1.28f, 0.22f, -0.05f}, {0, 0, 1}, {0, 1, 0}, 1.15f, 1.2f), panel_mi));
    room.push_back(add(quad({-0.42f, 0.34f, -1.22f}, {1, 0, 0}, {0, 1, 0}, 1.15f, 1.05f), window_mi));
    room.push_back(add(quad({-0.42f, 0.34f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 0.014f, 1.05f), mullion_mi));
    room.push_back(add(quad({-0.95f, 0.34f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 0.012f, 1.05f), mullion_mi));
    room.push_back(add(quad({0.12f, 0.34f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 0.012f, 1.05f), mullion_mi));
    room.push_back(add(quad({-0.42f, 0.82f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 1.15f, 0.012f), mullion_mi));
    room.push_back(add(quad({-0.42f, -0.08f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 1.15f, 0.012f), mullion_mi));
    room.push_back(add(quad({1.05f, 0.15f, 0.55f}, norm(V3{-0.15f, 0, 0.2f}), {0, 1, 0}, 0.028f, 0.85f), slit_mi));
    room.push_back(add(quad({0.28f, 0.72f, 0.42f}, {1, 0, 0}, {0, 1, 0}, 0.04f, 0.04f), bulb_mi));

    auto lamp = [&](float3 color, float lux, float3 dir) {
        utils::Entity e = utils::EntityManager::get().create();
        LightManager::Builder(LightManager::Type::DIRECTIONAL)
            .color(color)
            .intensity(lux)
            .direction(normalize(dir))
            .castShadows(false)
            .build(*engine, e);
        scene->addEntity(e);
        return e;
    };
    utils::Entity key = lamp({1.0f, 0.78f, 0.48f}, 4200.f, {0.42f, -0.22f, 0.88f});
    utils::Entity fill = lamp({0.45f, 0.62f, 1.0f}, 900.f, {-0.75f, -0.18f, -0.35f});
    (void)key;
    (void)fill;

    auto constant = [](float r) { return [r](float) { return r; }; };
    auto vary = [](float a, float b, float freq, float phase) {
        return [=](float u) { return a + (b - a) * (0.5f + 0.5f * std::sin(u * TAU * freq + phase)); };
    };

    struct Shot {
        const char* name;
        float bloom;
        float highlight;
        std::function<void(std::vector<Hold>&)> build;
    };
    Shot shots[5] = {
        {"glass", 0.22f, 8.0f,
         [&](std::vector<Hold>& hs) {
             hs.push_back(add(tube(path, 48, constant(0.078f)), glass));
             hs.push_back(add(tube(scaled(path, 0.58f, {0.04f, 0.03f, 0.05f}), 40, constant(0.036f)), amber));
         }},
        {"chrome", 0.16f, 10.0f,
         [&](std::vector<Hold>& hs) {
             hs.push_back(add(tube(nudged(path, -0.09f, 0), 40, constant(0.024f)), chrome));
             hs.push_back(add(tube(nudged(path, 0.09f, 0), 40, constant(0.024f)), chrome_sharp));
         }},
        {"emissive", 0.55f, 14.0f,
         [&](std::vector<Hold>& hs) {
             hs.push_back(add(ribbon(path, 0.0f, vary(0.004f, 0.012f, 3.f, 0.f)), core));
             hs.push_back(add(ribbon(nudged(path, 0.028f, 0.01f), 1.5f, vary(0.003f, 0.008f, 2.f, 1.f)), filament_w));
             hs.push_back(add(ribbon(nudged(path, -0.02f, -0.02f), 2.0f, vary(0.003f, 0.007f, 2.f, 2.f)), filament_c));
         }},
        {"ribbon", 0.20f, 8.0f,
         [&](std::vector<Hold>& hs) {
             hs.push_back(add(ribbon(nudged(path, 0.0f, 0.16f), 2.5f, vary(0.012f, 0.072f, 2.f, 0.4f)), brushed));
             hs.push_back(add(sweep_rect(nudged(path, 0.0f, -0.16f), 0.75f, vary(0.03f, 0.11f, 1.f, 0.2f),
                                         vary(0.012f, 0.045f, 2.f, 1.1f)),
                              glass));
         }},
        {"composite", 0.42f, 12.0f,
         [&](std::vector<Hold>& hs) {
             hs.push_back(add(sweep_rect(scaled(path, 1.18f, {0, -0.02f, 0}), 0.35f, vary(0.02f, 0.055f, 1.f, 0.f),
                                         constant(0.012f)),
                              glass));
             hs.push_back(add(tube(path, 48, constant(0.064f)), glass));
             hs.push_back(add(tube(scaled(path, 0.60f, {0.04f, 0.03f, 0.05f}), 36, constant(0.026f)), amber));
             hs.push_back(add(tube(nudged(path, 0.075f, 0.02f), 28, constant(0.012f)), chrome));
             hs.push_back(add(ribbon(path, 1.0f, vary(0.0035f, 0.009f, 3.f, 0.5f)), core));
         }},
    };

    FILE* log = std::fopen("out/limit.txt", "w");
    std::fprintf(log, "1920x1080  exposure f/16 1/125 ISO100  ibl %.0f lux  AgX punchy\n", ibl_intensity);
    std::fprintf(log, "path rings %d length %.3f kappa %.3f r*kappa %.3f separation %.3f\n", n, L, kappa, glass_r * kappa, sep);
    auto ms = [](std::chrono::steady_clock::duration d) {
        return std::chrono::duration<double, std::milli>(d).count();
    };
    bool png_ok = true;
    for (Shot& shot : shots) {
        BloomOptions bloom;
        bloom.enabled = true;
        bloom.strength = shot.bloom;
        bloom.levels = 7;
        bloom.resolution = 1080;
        bloom.threshold = true;
        bloom.highlight = shot.highlight;
        bloom.blendMode = BloomOptions::BlendMode::ADD;
        view->setBloomOptions(bloom);
        std::vector<Hold> hs;
        shot.build(hs);
        for (int i = 0; i < 2; ++i) {
            if (!renderer->beginFrame(swap)) {
                --i;
                continue;
            }
            renderer->render(view);
            renderer->endFrame();
        }
        engine->flushAndWait();
        std::vector<double> bare;
        for (int i = 0; i < 5; ++i) {
            auto t0 = std::chrono::steady_clock::now();
            while (!renderer->beginFrame(swap)) {}
            renderer->render(view);
            renderer->endFrame();
            engine->flushAndWait();
            bare.push_back(ms(std::chrono::steady_clock::now() - t0));
        }
        std::vector<uint8_t> pic((size_t)W * H * 4);
        std::vector<double> full;
        for (int i = 0; i < 3; ++i) {
            auto t1 = std::chrono::steady_clock::now();
            while (!renderer->beginFrame(swap)) {}
            renderer->render(view);
            renderer->readPixels(rt, 0, 0, (uint32_t)W, (uint32_t)H,
                                 backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA,
                                                                backend::PixelDataType::UBYTE));
            renderer->endFrame();
            engine->flushAndWait();
            full.push_back(ms(std::chrono::steady_clock::now() - t1));
        }
        std::printf("  bare");
        for (double t : bare) std::printf(" %.2f", t);
        std::printf("  full");
        for (double t : full) std::printf(" %.2f", t);
        std::printf("\n");
        std::sort(bare.begin(), bare.end());
        std::sort(full.begin(), full.end());
        double draw = bare[bare.size() / 2];
        double with_read = full[full.size() / 2];
        double sum = 0;
        int hot = 0, dead = 0, peak = 0;
        for (int i = 0; i < W * H; ++i) {
            int r = pic[i * 4], g = pic[i * 4 + 1], b = pic[i * 4 + 2];
            sum += (r + g + b) / 3.0;
            peak = std::max(peak, std::max(r, std::max(g, b)));
            if (r > 248 && g > 248 && b > 248) ++hot;
            if (r < 3 && g < 3 && b < 3) ++dead;
        }
        std::string file = std::string("out/limit-") + shot.name + ".png";
        if (!write_png(file.c_str(), W, H, pic.data())) png_ok = false;
        std::printf("%-10s  draw %.2f ms  with-readback %.2f ms  readback %.2f ms  mean %.1f  peak %d  hot %.2f%%  dead %.1f%%\n",
                    shot.name, draw, with_read, with_read - draw, sum / (W * H), peak, 100.0 * hot / (W * H), 100.0 * dead / (W * H));
        std::fprintf(log, "%s draw_ms %.3f with_readback_ms %.3f readback_ms %.3f mean %.2f peak %d hot_pct %.3f dead_pct %.2f\n",
                     shot.name, draw, with_read, with_read - draw, sum / (W * H), peak, 100.0 * hot / (W * H), 100.0 * dead / (W * H));
        drop(hs);
    }
    std::fclose(log);
    if (!png_ok) return fail("png");
    std::printf("PASS beauty\n");
    return 0;
}
