// Four holes, one chart each. Before uses the existing glass.filamat.
// After changes only Filament settings or connects the existing GPU curve
// evaluation to a lit refraction material.
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
#include <filament/ToneMapper.h>
#include <filament/VertexBuffer.h>
#include <filament/View.h>
#include <filament/Viewport.h>

#include <math/mat3.h>
#include <math/vec3.h>
#include <math/vec4.h>

#include <utils/EntityManager.h>

#include <zlib.h>

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>

using namespace filament;
using namespace filament::math;

static const int W = 1600;
static const int H = 900;
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

static Built quad(V3 c, V3 right, V3 up, float hw, float hh) {
    Built m;
    V3 n = norm(cross(right, up));
    V3 p[4] = {c - right * hw - up * hh, c + right * hw - up * hh, c + right * hw + up * hh, c - right * hw + up * hh};
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

struct Hold {
    Built mesh;
    utils::Entity entity;
    VertexBuffer* vb = nullptr;
    IndexBuffer* ib = nullptr;
};

static int fail(const char* m) {
    std::printf("FAIL %s\n", m);
    return 1;
}

int main() {
    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    utils::Entity cam_e = utils::EntityManager::get().create();
    Camera* camera = engine->createCamera(cam_e);
    camera->setExposure(1.0f);
    camera->lookAt({0.15, 0.05, 2.15}, {0, 0, 0}, {0, 1, 0});
    camera->setProjection(30.0, double(W) / double(H), 0.04, 30.0, Camera::Fov::VERTICAL);

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
    static LinearToneMapper linear;
    ColorGrading* grading = ColorGrading::Builder().toneMapper(&linear).build(*engine);
    Skybox* sky = Skybox::Builder().color({0, 0, 0, 1}).build(*engine);
    scene->setSkybox(sky);
    view->setScene(scene);
    view->setCamera(camera);
    view->setRenderTarget(rt);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setPostProcessingEnabled(true);
    view->setScreenSpaceRefractionEnabled(true);
    view->setAntiAliasing(View::AntiAliasing::NONE);
    view->setDithering(View::Dithering::NONE);
    view->setColorGrading(grading);
    Renderer::ClearOptions clear{};
    clear.clearColor = {0, 0, 0, 1};
    clear.clear = true;
    renderer->setClearOptions(clear);

    auto glass_old_pkg = load_file("glass.filamat");
    auto bend_pkg = load_file("glass_bend.filamat");
    auto gpu_pkg = load_file("glass_gpu.filamat");
    auto emit_pkg = load_file("emissive.filamat");
    auto chrome_pkg = load_file("chrome_aniso.filamat");
    if (glass_old_pkg.empty() || bend_pkg.empty() || gpu_pkg.empty() || emit_pkg.empty() || chrome_pkg.empty()) return fail("filamat");
    Material* glass_old_mat = Material::Builder().package(glass_old_pkg.data(), glass_old_pkg.size()).build(*engine);
    Material* bend_mat = Material::Builder().package(bend_pkg.data(), bend_pkg.size()).build(*engine);
    Material* gpu_mat = Material::Builder().package(gpu_pkg.data(), gpu_pkg.size()).build(*engine);
    Material* emit_mat = Material::Builder().package(emit_pkg.data(), emit_pkg.size()).build(*engine);
    Material* chrome_mat = Material::Builder().package(chrome_pkg.data(), chrome_pkg.size()).build(*engine);
    if (!glass_old_mat || !bend_mat || !gpu_mat || !emit_mat || !chrome_mat) return fail("material");

    auto glass_old = glass_old_mat->createInstance();
    glass_old->setParameter("ior", 2.15f);
    glass_old->setParameter("transmission", 1.0f);
    glass_old->setParameter("thickness", 0.62f);
    glass_old->setParameter("dispersion", 0.72f);
    auto tune = [](MaterialInstance* mi, float ior, float thick, float disp, float refl) {
        mi->setParameter("ior", ior);
        mi->setParameter("transmission", 1.0f);
        mi->setParameter("thickness", thick);
        mi->setParameter("dispersion", disp);
        mi->setParameter("reflectance", refl);
    };
    MaterialInstance* bend = bend_mat->createInstance();
    tune(bend, 1.55f, 1.15f, 0.0f, 0.35f);
    MaterialInstance* prism = bend_mat->createInstance();
    tune(prism, 1.55f, 1.15f, 16.0f, 0.35f);
    MaterialInstance* gpu_mi = gpu_mat->createInstance();
    tune(gpu_mi, 1.55f, 1.15f, 0.0f, 0.35f);
    MaterialInstance* bar_mi = emit_mat->createInstance();
    bar_mi->setParameter("color", float4{3.0f, 3.0f, 3.0f, 1});
    MaterialInstance* iso = chrome_mat->createInstance();
    iso->setParameter("anisotropy", 0.0f);
    iso->setParameter("roughness", 0.22f);
    MaterialInstance* aniso = chrome_mat->createInstance();
    aniso->setParameter("anisotropy", 0.92f);
    aniso->setParameter("roughness", 0.22f);

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

    std::vector<Hold> bars;
    for (int i = -7; i <= 7; ++i) {
        float x = (float)i * 0.16f;
        bars.push_back(add(quad({x, 0, -0.85f}, {1, 0, 0}, {0, 1, 0}, 0.028f, 1.15f), bar_mi));
    }

    const int per = 28;
    const int sides = 48;
    const float tube_r = 0.105f;
    auto spans = circle(0.52f, 0.62f);
    auto frames = close_frames(spans, per);
    Built cpu_mesh = tube(frames, sides, tube_r, false);
    Built gpu_mesh = tube(frames, sides, tube_r, true);

    const int tex_w = 64;
    auto tex_rows = [](int n) { return std::max(1, (n + 63) / 64); };
    std::vector<float> controls;
    int ctl_n = (int)spans.size() * 4;
    controls.assign((size_t)tex_rows(ctl_n) * tex_w * 4, 0.f);
    for (int s = 0; s < (int)spans.size(); ++s)
        for (int k = 0; k < 4; ++k) {
            int i = s * 4 + k;
            controls[(size_t)i * 4 + 0] = spans[s].p[k].x;
            controls[(size_t)i * 4 + 1] = spans[s].p[k].y;
            controls[(size_t)i * 4 + 2] = spans[s].p[k].z;
        }
    int frame_n = (int)frames.size() * 2;
    std::vector<float> frame_tex((size_t)tex_rows(frame_n) * tex_w * 4, 0.f);
    for (int i = 0; i < (int)frames.size(); ++i) {
        frame_tex[(size_t)(i * 2) * 4 + 0] = frames[i].N.x;
        frame_tex[(size_t)(i * 2) * 4 + 1] = frames[i].N.y;
        frame_tex[(size_t)(i * 2) * 4 + 2] = frames[i].N.z;
        frame_tex[(size_t)(i * 2 + 1) * 4 + 0] = frames[i].B.x;
        frame_tex[(size_t)(i * 2 + 1) * 4 + 1] = frames[i].B.y;
        frame_tex[(size_t)(i * 2 + 1) * 4 + 2] = frames[i].B.z;
    }
    auto float_tex = [&](int w, int h, const std::vector<float>& data) {
        Texture* tex = Texture::Builder()
                           .width((uint32_t)w)
                           .height((uint32_t)h)
                           .levels(1)
                           .sampler(Texture::Sampler::SAMPLER_2D)
                           .format(Texture::InternalFormat::RGBA32F)
                           .build(*engine);
        tex->setImage(*engine, 0,
                      Texture::PixelBufferDescriptor(data.data(), data.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                     backend::PixelDataType::FLOAT, 1, 0, 0, (uint32_t)w));
        return tex;
    };
    Texture* ctl = float_tex(tex_w, tex_rows(ctl_n), controls);
    Texture* frm = float_tex(tex_w, tex_rows(frame_n), frame_tex);
    TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    gpu_mi->setParameter("controls", ctl, nearest);
    gpu_mi->setParameter("frames", frm, nearest);
    gpu_mi->setParameter("samples", (float)per);
    gpu_mi->setParameter("sides", (float)sides);
    gpu_mi->setParameter("spans", (float)spans.size());
    gpu_mi->setParameter("radius", tube_r);
    gpu_mi->setParameter("texWidth", (float)tex_w);
    gpu_mi->setParameter("texHeight", (float)tex_rows(ctl_n));
    gpu_mi->setParameter("frameHeight", (float)tex_rows(frame_n));

    auto grab = [&](const char* path) {
        std::vector<uint8_t> pic((size_t)W * H * 4);
        for (int i = 0; i < 3; ++i) {
            if (!renderer->beginFrame(swap)) {
                --i;
                continue;
            }
            renderer->render(view);
            if (i == 2)
                renderer->readPixels(rt, 0, 0, (uint32_t)W, (uint32_t)H,
                                     backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA,
                                                                    backend::PixelDataType::UBYTE));
            renderer->endFrame();
        }
        engine->flushAndWait();
        if (!write_png(path, W, H, pic.data())) return false;
        std::printf("wrote %s\n", path);
        return true;
    };

    if (!grab("out/fix-bars.png")) return fail("bars");
    std::vector<Hold> shape;
    shape.push_back(add(std::move(cpu_mesh), glass_old));
    // cpu_mesh was moved. Keep a second copy for later shots.
    Built cpu_again = tube(frames, sides, tube_r, false);
    if (!grab("out/fix-refract-before.png")) return fail("before");
    drop(shape);
    shape.push_back(add(std::move(cpu_again), bend));
    if (!grab("out/fix-refract-after.png")) return fail("after");
    drop(shape);
    Built cpu_prism = tube(frames, sides, tube_r, false);
    shape.push_back(add(std::move(cpu_prism), prism));
    if (!grab("out/fix-dispersion.png")) return fail("dispersion");
    drop(shape);
    Built cpu_ref = tube(frames, sides, tube_r, false);
    shape.push_back(add(std::move(cpu_ref), bend));
    if (!grab("out/fix-cpu.png")) return fail("cpu");
    drop(shape);
    shape.push_back(add(std::move(gpu_mesh), gpu_mi));
    if (!grab("out/fix-gpu.png")) return fail("gpu");
    drop(shape);

    for (Hold& h : bars) scene->remove(h.entity);
    camera->setExposure(4.0f, 1.0f / 125.0f, 100.0f);
    camera->lookAt({0.0, 0.0, 1.85}, {0, 0, 0}, {0, 1, 0});
    utils::Entity sun = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::SUN)
        .color(float3{1, 1, 1})
        .intensity(140000.f)
        .direction(normalize(float3{0.05f, -0.15f, -1.0f}))
        .sunAngularRadius(0.45f)
        .sunHaloSize(1.5f)
        .castShadows(false)
        .build(*engine, sun);
    scene->addEntity(sun);
    auto ring_at = [&](float y) {
        auto sp = circle(0.30f, 0.0f);
        for (Cubic& c : sp)
            for (int k = 0; k < 4; ++k) c.p[k].y += y;
        return close_frames(sp, 24);
    };
    shape.push_back(add(tube(ring_at(0.42f), 36, 0.040f, false), aniso));
    shape.push_back(add(tube(ring_at(-0.42f), 36, 0.040f, false), iso));
    if (!grab("out/fix-aniso.png")) return fail("aniso");

    std::printf("PASS holes\n");
    return 0;
}
