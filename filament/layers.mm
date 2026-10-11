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

int main(int argc, char** argv) {
    const bool hdr_only = argc > 1 && std::strcmp(argv[1], "hdr") == 0;
    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    utils::Entity cam_e = utils::EntityManager::get().create();
    Camera* camera = engine->createCamera(cam_e);
    camera->setExposure(1.0f);

    auto make_target = [&](Texture::InternalFormat format) {
        Texture* color = Texture::Builder()
                             .width(W)
                             .height(H)
                             .levels(1)
                             .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                             .format(format)
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
        return std::tuple<Texture*, Texture*, RenderTarget*>{color, depth, rt};
    };
    auto [colorA, depthA, rtA] = make_target(Texture::InternalFormat::RGBA8);
    auto [colorB, depthB, rtB] = make_target(Texture::InternalFormat::RGBA8);
    Texture* color8[2] = {colorA, colorB};
    RenderTarget* rt8[2] = {rtA, rtB};
    auto [colorAF, depthAF, rtAF] = make_target(Texture::InternalFormat::RGBA16F);
    auto [colorBF, depthBF, rtBF] = make_target(Texture::InternalFormat::RGBA16F);
    Texture* colorF[2] = {colorAF, colorBF};
    RenderTarget* rtF[2] = {rtAF, rtBF};
    Texture** colors = color8;
    RenderTarget** rts = rt8;
    if (!colorA || !colorB || !rtA || !rtB || !colorAF || !colorBF || !rtAF || !rtBF) return fail("target");

    Skybox* sky = Skybox::Builder().color({0, 0, 0, 1}).build(*engine);
    view->setScene(scene);
    view->setCamera(camera);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setPostProcessingEnabled(false);
    view->setScreenSpaceRefractionEnabled(true);
    ScreenSpaceReflectionsOptions ssr{};
    ssr.enabled = false;
    view->setScreenSpaceReflectionsOptions(ssr);
    view->setAntiAliasing(View::AntiAliasing::NONE);
    view->setDithering(View::Dithering::NONE);
    BloomOptions bloom{};
    bloom.enabled = false;
    view->setBloomOptions(bloom);
    Renderer::ClearOptions clear{};
    clear.clearColor = {0, 0, 0, 1};
    clear.clear = true;
    renderer->setClearOptions(clear);

    auto layer_pkg = load_file("glass_layer.filamat");
    auto plate_pkg = load_file("plate.filamat");
    auto emit_pkg = load_file("emissive.filamat");
    auto chrome_pkg = load_file("chrome.filamat");
    if (layer_pkg.empty() || plate_pkg.empty() || emit_pkg.empty() || chrome_pkg.empty()) return fail("filamat");
    Material* layer_mat = Material::Builder().package(layer_pkg.data(), layer_pkg.size()).build(*engine);
    Material* plate_mat = Material::Builder().package(plate_pkg.data(), plate_pkg.size()).build(*engine);
    Material* emit_mat = Material::Builder().package(emit_pkg.data(), emit_pkg.size()).build(*engine);
    Material* chrome_mat = Material::Builder().package(chrome_pkg.data(), chrome_pkg.size()).build(*engine);
    if (!layer_mat || !plate_mat || !emit_mat || !chrome_mat) return fail("material");

    MaterialInstance* bar_mi = emit_mat->createInstance();
    bar_mi->setParameter("color", float4{3, 3, 3, 1});
    MaterialInstance* chrome_mi = chrome_mat->createInstance();
    TextureSampler plate_sampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
    plate_sampler.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    plate_sampler.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    MaterialInstance* plate_mi[2] = {plate_mat->createInstance(), plate_mat->createInstance()};
    plate_mi[0]->setParameter("color", colorA, plate_sampler);
    plate_mi[1]->setParameter("color", colorB, plate_sampler);

    struct GlassSpec {
        const char* name;
        float radius;
        float tube;
        float tilt;
        float3 at;
        float ior, thick, disp, refl;
        float3 absorb;
        MaterialInstance* mi = nullptr;
        Hold hold;
    };
    GlassSpec glasses[3] = {
        {"back", 0.62f, 0.115f, 0.85f, {0.00f, 0.00f, -0.20f}, 1.70f, 1.05f, 10.f, 0.42f, {0.04f, 0.16f, 0.85f}, nullptr, {}},
        {"mid", 0.48f, 0.100f, -0.65f, {-0.20f, 0.06f, 0.12f}, 1.95f, 0.85f, 14.f, 0.38f, {0.15f, 0.75f, 0.08f}, nullptr, {}},
        {"front", 0.38f, 0.092f, 0.55f, {0.16f, -0.04f, 0.46f}, 2.25f, 1.20f, 12.f, 0.46f, {0.90f, 0.10f, 0.35f}, nullptr, {}},
    };

    auto add_mesh = [&](Built mesh, MaterialInstance* mi, bool uv) {
        Hold h;
        h.mesh = std::move(mesh);
        uint32_t stride = uv ? 36u : 28u;
        uint32_t nv = (uint32_t)(h.mesh.v.size() / (stride / 4));
        auto b = VertexBuffer::Builder().vertexCount(nv).bufferCount(1);
        b.attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, stride);
        if (uv) {
            b.attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, stride);
            b.attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, stride);
        } else {
            b.attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, stride);
        }
        h.vb = b.build(*engine);
        h.vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(h.mesh.v.data(), h.mesh.v.size() * sizeof(float)));
        h.ib = IndexBuffer::Builder().indexCount((uint32_t)h.mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        h.ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(h.mesh.idx.data(), h.mesh.idx.size() * sizeof(uint32_t)));
        h.entity = utils::EntityManager::get().create();
        RenderableManager::Builder(1)
            .boundingBox({{-8, -8, -8}, {8, 8, 8}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, h.vb, h.ib)
            .material(0, mi)
            .culling(false)
            .castShadows(false)
            .receiveShadows(false)
            .build(*engine, h.entity);
        return h;
    };
    auto place = [&](utils::Entity e, float3 p) {
        auto& tm = engine->getTransformManager();
        tm.setTransform(tm.getInstance(e), mat4f::translation(p));
    };

    std::vector<Hold> bars;
    for (int i = -8; i <= 8; ++i)
        bars.push_back(add_mesh(quad_bars({(float)i * 0.18f, 0, -1.05f}, 0.030f, 1.35f), bar_mi, false));
    Hold chrome = add_mesh(sphere(0.20f, 24, 32), chrome_mi, false);
    place(chrome.entity, {0.36f, -0.10f, -0.30f});

    for (GlassSpec& g : glasses) {
        g.mi = layer_mat->createInstance();
        g.mi->setParameter("ior", g.ior);
        g.mi->setParameter("transmission", 1.0f);
        g.mi->setParameter("thickness", g.thick);
        g.mi->setParameter("dispersion", g.disp);
        g.mi->setParameter("reflectance", g.refl);
        g.mi->setParameter("absorption", g.absorb);
        g.hold = add_mesh(tube(close_frames(circle(g.radius, g.tilt), 32), 40, g.tube), g.mi, false);
        place(g.hold.entity, g.at);
    }

    // Frustum plate. UVs run with the projected rectangle. Rebuilt when the camera moves.
    Hold plate;
    std::vector<float> plate_v(4 * 9, 0.f);
    std::vector<uint32_t> plate_idx = {0, 1, 2, 0, 2, 3};
    {
        plate.vb = VertexBuffer::Builder()
                       .vertexCount(4)
                       .bufferCount(1)
                       .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 36)
                       .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 36)
                       .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, 36)
                       .build(*engine);
        plate.ib = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        plate.ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(plate_idx.data(), plate_idx.size() * sizeof(uint32_t)));
        plate.entity = utils::EntityManager::get().create();
        RenderableManager::Builder(1)
            .boundingBox({{-30, -30, -30}, {30, 30, 30}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, plate.vb, plate.ib)
            .material(0, plate_mi[0])
            .culling(false)
            .castShadows(false)
            .receiveShadows(false)
            .build(*engine, plate.entity);
    }
    auto update_plate = [&](Eye eye) {
        float3 f = normalize(eye.target - eye.eye);
        float3 r = normalize(cross(f, float3{0, 1, 0}));
        float3 u = cross(r, f);
        float dist = 8.0f;
        float hh = dist * std::tan(13.0f * PI / 180.f);
        float hw = hh * ((float)W / (float)H);
        float3 c = eye.eye + f * dist;
        float3 corner[4] = {c - r * hw - u * hh, c + r * hw - u * hh, c + r * hw + u * hh, c - r * hw + u * hh};
        float uv[4][2] = {{0, 0}, {1, 0}, {1, 1}, {0, 1}};
        V3 T{r.x, r.y, r.z};
        V3 N{f.x, f.y, f.z};
        V3 Nn = norm(N - T * dot(N, T));
        V3 B = norm(cross(T, Nn));
        quatf q = mat3f::packTangentFrame(mat3f{float3{T.x, T.y, T.z}, float3{B.x, B.y, B.z}, float3{Nn.x, Nn.y, Nn.z}});
        for (int i = 0; i < 4; ++i) {
            float* o = plate_v.data() + i * 9;
            o[0] = corner[i].x;
            o[1] = corner[i].y;
            o[2] = corner[i].z;
            o[3] = uv[i][0];
            o[4] = uv[i][1];
            o[5] = q.x;
            o[6] = q.y;
            o[7] = q.z;
            o[8] = q.w;
        }
        plate.vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(plate_v.data(), plate_v.size() * sizeof(float)));
    };

    utils::Entity sun = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::SUN)
        .color(float3{1.0f, 0.98f, 0.94f})
        .intensity(110000.f)
        .direction(normalize(float3{0.15f, -0.35f, -1.0f}))
        .sunAngularRadius(1.2f)
        .sunHaloSize(2.0f)
        .castShadows(false)
        .build(*engine, sun);
    scene->addEntity(sun);

    std::vector<utils::Entity> shown;
    auto set_shown = [&](const std::vector<utils::Entity>& next) {
        for (utils::Entity e : shown) scene->remove(e);
        shown = next;
        for (utils::Entity e : shown) scene->addEntity(e);
    };
    auto look = [&](Eye eye) {
        camera->lookAt(eye.eye, eye.target, {0, 1, 0});
        camera->setProjection(26.0, double(W) / double(H), 0.05, 40.0, Camera::Fov::VERTICAL);
        update_plate(eye);
    };

    std::vector<uint8_t> pic((size_t)W * H * 4);
    std::vector<uint16_t> half;
    bool capture_half = false;
    auto open_frame = [&]() -> bool {
        for (int spin = 0; spin < 4000; ++spin)
            if (renderer->beginFrame(swap)) return true;
        std::printf("beginFrame failed\n");
        return false;
    };
    auto render_to = [&](RenderTarget* rt, bool read) {
        view->setRenderTarget(rt);
        renderer->render(view);
        if (read && capture_half) {
            renderer->readPixels(rt, 0, 0, (uint32_t)W, (uint32_t)H,
                                 backend::PixelBufferDescriptor(half.data(), half.size() * sizeof(uint16_t),
                                                                backend::PixelDataFormat::RGBA, backend::PixelDataType::HALF));
        } else if (read) {
            renderer->readPixels(rt, 0, 0, (uint32_t)W, (uint32_t)H,
                                 backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA,
                                                                backend::PixelDataType::UBYTE));
        }
    };

    // One beginFrame for every glass. readPixels runs only on the last pass when asked.
    auto draw = [&](const int* which, int n, bool single, Eye eye, bool read) -> Texture* {
        look(eye);
        std::vector<int> order(which, which + n);
        if (!open_frame()) return nullptr;
        if (!single && n > 1) {
            std::sort(order.begin(), order.end(), [&](int a, int b) {
                float3 fa = glasses[a].at - eye.eye;
                float3 fb = glasses[b].at - eye.eye;
                return dot(fa, fa) > dot(fb, fb);
            });
        }
        if (single) {
            scene->setSkybox(sky);
            std::vector<utils::Entity> es;
            for (Hold& h : bars) es.push_back(h.entity);
            es.push_back(chrome.entity);
            for (int i : order) es.push_back(glasses[i].hold.entity);
            set_shown(es);
            render_to(rts[0], read);
            renderer->endFrame();
            return colors[0];
        }
        scene->setSkybox(sky);
        std::vector<utils::Entity> bg;
        for (Hold& h : bars) bg.push_back(h.entity);
        bg.push_back(chrome.entity);
        set_shown(bg);
        render_to(rts[0], read && n == 0);
        int written = 0;
        int next = 1;
        scene->setSkybox(nullptr);
        for (int k = 0; k < n; ++k) {
            auto& rm = engine->getRenderableManager();
            rm.setMaterialInstanceAt(rm.getInstance(plate.entity), 0, plate_mi[written]);
            set_shown({plate.entity, glasses[order[k]].hold.entity});
            render_to(rts[next], read && k == n - 1);
            written = next;
            next = 1 - next;
        }
        renderer->endFrame();
        return colors[written];
    };

    auto shoot = [&](const int* which, int n, bool single, Eye eye, const char* path) -> bool {
        if (!draw(which, n, single, eye, false)) return false;
        if (!draw(which, n, single, eye, true)) return false;
        engine->flushAndWait();
        if (!write_png(path, W, H, pic.data())) return false;
        std::printf("wrote %s\n", path);
        return true;
    };

    Eye still{{0.10f, 0.16f, 3.55f}, {0, 0, 0}};
    const int back[] = {0};
    const int two[] = {0, 1};
    const int all[] = {0, 1, 2};
    const int front[] = {2};
    const int none[] = {0};
    if (!hdr_only) {
    if (!shoot(none, 0, false, still, "out/layers-bg.png")) return fail("bg");
    if (!shoot(back, 1, false, still, "out/layers-1.png")) return fail("one");
    if (!shoot(two, 2, false, still, "out/layers-2.png")) return fail("two");
    if (!shoot(all, 3, false, still, "out/layers-3.png")) return fail("three");
    if (!shoot(front, 1, false, still, "out/layers-front.png")) return fail("front");
    if (!shoot(all, 3, true, still, "out/layers-single.png")) return fail("single");

    for (int f = 0; f < 6; ++f) {
        float yaw = -0.42f + f * (0.84f / 5.f);
        Eye eye{{std::sin(yaw) * 3.55f, 0.18f, std::cos(yaw) * 3.55f}, {0, 0, 0}};
        glasses[2].at.x = 0.16f + 0.22f * std::sin(yaw * 1.7f);
        place(glasses[2].hold.entity, glasses[2].at);
        char path[64];
        std::snprintf(path, sizeof(path), "out/layers-m%02d.png", f);
        if (!shoot(all, 3, false, eye, path)) return fail("motion");
    }

    // GPU time of the nested path. flushAndWait is the sync used to read the clock.
    // The frames above already called it after readPixels; this loop does not read pixels.
    std::vector<double> ms;
    for (int i = 0; i < 12; ++i) {
        auto t0 = std::chrono::steady_clock::now();
        if (!draw(all, 3, false, still, false)) return fail("time");
        engine->flushAndWait();
        auto t1 = std::chrono::steady_clock::now();
        ms.push_back(std::chrono::duration<double, std::milli>(t1 - t0).count());
    }
    std::vector<double> single_ms;
    for (int i = 0; i < 8; ++i) {
        auto t0 = std::chrono::steady_clock::now();
        if (!draw(all, 3, true, still, false)) return fail("time-single");
        engine->flushAndWait();
        auto t1 = std::chrono::steady_clock::now();
        single_ms.push_back(std::chrono::duration<double, std::milli>(t1 - t0).count());
    }
    FILE* log = std::fopen("out/layers-time.txt", "w");
    if (!log) return fail("log");
    std::fprintf(log, "nested ms (flushAndWait, no readPixels)\n");
    for (double v : ms) std::fprintf(log, "%.3f\n", v);
    std::fprintf(log, "single-view ms\n");
    for (double v : single_ms) std::fprintf(log, "%.3f\n", v);
    std::fclose(log);
    }

    // Same three glasses, but the View targets are RGBA16F. Filament still chooses
    // the texture format it copies into this target; the measurement is whether a
    // value above 1 survives. The bars are emissive 3 at exposure 1.
    colors = colorF;
    rts = rtF;
    plate_mi[0]->setParameter("color", colorF[0], plate_sampler);
    plate_mi[1]->setParameter("color", colorF[1], plate_sampler);
    half.assign((size_t)W * H * 4, 0);
    capture_half = true;
    auto atf = [&](int x, int y) {
        size_t i = ((size_t)y * W + x) * 4;
        return float3{half_to_float(half[i]), half_to_float(half[i + 1]), half_to_float(half[i + 2])};
    };
    auto report = [&](const char* label, Texture* color) {
        if (!color) {
            std::printf("HDR %s draw failed\n", label);
            return;
        }
        engine->flushAndWait();
        float3 bar = atf(249, 540);
        float maxv = 0;
        int mx = 0, my = 0;
        double sum = 0;
        int n = 0;
        for (int y = 0; y < H; y += 2) {
            for (int x = 0; x < W; x += 2) {
                float3 c = atf(x, y);
                float m = std::max(c.x, std::max(c.y, c.z));
                sum += m;
                n++;
                if (m > maxv) {
                    maxv = m;
                    mx = x;
                    my = y;
                }
            }
        }
        std::printf("HDR %s bar249,540 (%.4f %.4f %.4f) max %.4f at %d,%d mean %.4f\n",
                    label, bar.x, bar.y, bar.z, maxv, mx, my, sum / n);
    };

    view->setScreenSpaceRefractionEnabled(false);
    if (!draw(none, 0, false, still, false)) return fail("hdr-plain");
    Texture* plain = draw(none, 0, false, still, true);
    report("no-refraction", plain);

    view->setScreenSpaceRefractionEnabled(true);
    if (!draw(all, 3, false, still, false)) return fail("hdr-nested");
    Texture* nested = draw(all, 3, false, still, true);
    report("three-glass", nested);

    colors = color8;
    rts = rt8;
    plate_mi[0]->setParameter("color", color8[0], plate_sampler);
    plate_mi[1]->setParameter("color", color8[1], plate_sampler);
    capture_half = false;
    if (!draw(all, 3, false, still, true)) return fail("ldr-nested");
    engine->flushAndWait();
    auto at8 = [&](int x, int y) {
        size_t i = ((size_t)y * W + x) * 4;
        return int3{(int)pic[i], (int)pic[i + 1], (int)pic[i + 2]};
    };
    int3 bar8 = at8(249, 540);
    int3 hot8 = at8(1068, 382);
    std::printf("LDR three-glass bar249,540 (%d %d %d) hot1068,382 (%d %d %d)\n",
                bar8.x, bar8.y, bar8.z, hot8.x, hot8.y, hot8.z);

    FILE* hdr = std::fopen("out/layers-hdr.txt", "w");
    if (!hdr) return fail("hdr-log");
    std::fprintf(hdr, "see stdout\n");
    std::fclose(hdr);
    std::printf("PASS hdr\n");
    return 0;
}
