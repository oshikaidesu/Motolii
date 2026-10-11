// Verification only. The CPU cube trace (dispersion 0) supplies the cross.
// HPWater's RGB offset and the single-channel À-trous then run on that map.
// This is not the final illuminance path. out/prism.png is not written.
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
#include <vector>

using namespace filament;
using namespace filament::math;

static const int W = 1920, H = 1080, TEX = 768;
static const float HW = 1.55f, HH = 1.55f, ETA_BASE = 0.75f;

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
static void rotateY(V3& v, float a) {
    float c = std::cos(a), s = std::sin(a);
    float x = v.x * c + v.z * s, z = -v.x * s + v.z * c;
    v.x = x;
    v.z = z;
}
static V3 refract_glsl(V3 I, V3 N, float eta) {
    float d = dot(N, I);
    float k = 1.f - eta * eta * (1.f - d * d);
    if (k < 0.f) return {0, 0, 0};
    return I * eta - N * (eta * d + std::sqrt(k));
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
static void push_vert(std::vector<float>& v, V3 p, V3 T, V3 N) {
    V3 n = norm(N - T * dot(N, T));
    V3 B = norm(cross(n, T));
    quatf q = mat3f::packTangentFrame(mat3f{float3{T.x, T.y, T.z}, float3{B.x, B.y, B.z}, float3{n.x, n.y, n.z}});
    float f[7] = {p.x, p.y, p.z, q.x, q.y, q.z, q.w};
    v.insert(v.end(), f, f + 7);
}
static bool load_f32(const char* path, std::vector<float>& tex) {
    FILE* f = std::fopen(path, "rb");
    if (!f) return false;
    uint32_t n = 0;
    if (std::fread(&n, 4, 1, f) != 1 || n != (uint32_t)TEX) {
        std::fclose(f);
        return false;
    }
    tex.resize((size_t)TEX * TEX * 4);
    bool ok = std::fread(tex.data(), sizeof(float), tex.size(), f) == tex.size();
    std::fclose(f);
    return ok;
}

int main() {
    std::vector<float> mono, spec;
    if (!load_f32("out/prism-mono.f32", mono) || !load_f32("out/prism-spec.f32", spec)) return fail("f32");
    for (float& v : mono) v = std::min(std::max(v, 0.f), 60000.f);
    for (float& v : spec) v = std::min(std::max(v, 0.f), 60000.f);

    const float half = 0.16f;
    const float nd = 1.55f;
    const float dispersion = 36.f;
    const float lux = 22000.f;
    V3 sun = norm({0.92f, -0.42f, 0.22f});
    V3 axis[3] = {{1, 0, 0}, {0, 1, 0}, {0, 0, 1}};
    for (int i = 0; i < 3; ++i) rotateY(axis[i], 0.62f);
    V3 center{0.f, half + 0.001f, 0.f};
    V3 best = axis[1];
    float bestDot = 1.f;
    for (int ax = 0; ax < 3; ++ax)
        for (int sgn = -1; sgn <= 1; sgn += 2) {
            V3 N = axis[ax] * (float)sgn;
            float d = dot(N, sun);
            if (d < bestDot) {
                bestDot = d;
                best = N;
            }
        }
    if (dot(best, sun) > 0.f) best = best * -1.f;
    V3 dR = refract_glsl(sun, best, ETA_BASE + 0.001f);
    V3 dG = refract_glsl(sun, best, ETA_BASE);
    V3 delta = dG - dR;
    float2 off{delta.x / HW * 100.f, delta.z / HH * 100.f};
    std::printf("uv offset %.5f %.5f\n", off.x, off.y);

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    utils::Entity cam_e = utils::EntityManager::get().create();
    Camera* camera = engine->createCamera(cam_e);
    view->setScene(scene);
    view->setCamera(camera);
    view->setDithering(View::Dithering::NONE);
    view->setAntiAliasing(View::AntiAliasing::FXAA);
    view->setPostProcessingEnabled(true);
    view->setScreenSpaceRefractionEnabled(true);
    view->setShadowType(View::ShadowType::VSM);
    VsmShadowOptions vsm;
    vsm.highPrecision = true;
    view->setVsmShadowOptions(vsm);
    BloomOptions bloom{};
    bloom.enabled = true;
    bloom.strength = 0.18f;
    bloom.levels = 6;
    bloom.resolution = 1080;
    bloom.threshold = true;
    bloom.highlight = 10.f;
    bloom.blendMode = BloomOptions::BlendMode::ADD;
    view->setBloomOptions(bloom);
    static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
    view->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.04f).saturation(1.08f).build(*engine));
    scene->setSkybox(Skybox::Builder().color({0.72f, 0.74f, 0.78f, 1}).build(*engine));
    camera->setExposure(2.8f, 1.f / 125.f, 100.f);
    camera->lookAt(float3{0.55f, 0.78f, 0.70f}, float3{0, 0.04f, 0}, float3{0, 1, 0});
    camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});

    auto make_rt = [&](int w, int h, int levels, bool mip) {
        auto usage = Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC;
        if (mip) usage |= Texture::Usage::BLIT_DST | Texture::Usage::GEN_MIPMAPPABLE;
        Texture* color = Texture::Builder().width((uint32_t)w).height((uint32_t)h).levels((uint8_t)levels)
                             .usage(usage).format(Texture::InternalFormat::RGBA16F).build(*engine);
        Texture* depth = Texture::Builder().width((uint32_t)w).height((uint32_t)h).levels(1)
                             .usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH32F).build(*engine);
        RenderTarget* rt = RenderTarget::Builder()
                               .texture(RenderTarget::AttachmentPoint::COLOR, color)
                               .texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
        return std::pair<Texture*, RenderTarget*>{color, rt};
    };
    Texture* srcTex = Texture::Builder().width(TEX).height(TEX).levels(1)
                          .format(Texture::InternalFormat::RGBA16F).build(*engine);
    auto [tempTex, tempRT] = make_rt(TEX, TEX, 10, true);
    auto [filtTex, filtRT] = make_rt(TEX, TEX, 1, false);
    Texture* beauty = Texture::Builder().width(W).height(H).levels(1)
                          .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                          .format(Texture::InternalFormat::RGBA8).build(*engine);
    Texture* beautyDepth = Texture::Builder().width(W).height(H).levels(1)
                               .usage(Texture::Usage::DEPTH_ATTACHMENT)
                               .format(Texture::InternalFormat::DEPTH24).build(*engine);
    RenderTarget* beautyRT = RenderTarget::Builder()
                                 .texture(RenderTarget::AttachmentPoint::COLOR, beauty)
                                 .texture(RenderTarget::AttachmentPoint::DEPTH, beautyDepth).build(*engine);

    auto gpkg = load_file("glass_shadow.filamat");
    auto fpkg = load_file("floor_hp.filamat");
    auto apkg = load_file("caustic_atrous.filamat");
    auto epkg = load_file("emissive.filamat");
    if (gpkg.empty() || fpkg.empty() || apkg.empty() || epkg.empty()) return fail("filamat");
    Material* glassMat = Material::Builder().package(gpkg.data(), gpkg.size()).build(*engine);
    Material* floorMat = Material::Builder().package(fpkg.data(), fpkg.size()).build(*engine);
    Material* atrousMat = Material::Builder().package(apkg.data(), apkg.size()).build(*engine);
    Material* emat = Material::Builder().package(epkg.data(), epkg.size()).build(*engine);
    if (!glassMat || !floorMat || !atrousMat || !emat) return fail("material");

    TextureSampler lin(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
    lin.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    lin.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    TextureSampler mipped(TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR, TextureSampler::MagFilter::LINEAR);
    mipped.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    mipped.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);

    MaterialInstance* floorMi = floorMat->createInstance();
    floorMi->setParameter("color", float4{0.86f, 0.85f, 0.84f, 1});
    floorMi->setParameter("roughness", 0.42f);
    floorMi->setParameter("disperse", 0.f);
    floorMi->setParameter("uvOffset", off);
    floorMi->setParameter("caustic", srcTex, lin);
    MaterialInstance* atrous1 = atrousMat->createInstance();
    atrous1->setParameter("src", srcTex, lin);
    atrous1->setParameter("texel", float2{1.f / TEX, 1.f / TEX});
    atrous1->setParameter("stride", 1.f);
    atrous1->setParameter("lumWeight", 0.5f);
    atrous1->setParameter("mip", 0.f);
    atrous1->setParameter("second", 0.f);
    MaterialInstance* atrous2 = atrousMat->createInstance();
    atrous2->setParameter("src", tempTex, mipped);
    atrous2->setParameter("texel", float2{1.f / TEX, 1.f / TEX});
    atrous2->setParameter("stride", 2.f);
    atrous2->setParameter("lumWeight", 0.5f);
    atrous2->setParameter("mip", 1.f);
    atrous2->setParameter("second", 1.f);
    MaterialInstance* glass = glassMat->createInstance();
    glass->setParameter("ior", nd);
    glass->setParameter("transmission", 1.f);
    glass->setParameter("thickness", half * 2.f);
    glass->setParameter("dispersion", dispersion);
    glass->setParameter("reflectance", 0.5f);
    glass->setParameter("alpha", 1.f);
    glass->setParameter("absorption", float3{0.02f, 0.015f, 0.02f});
    MaterialInstance* panel = emat->createInstance();
    panel->setParameter("color", float4{2200.f, 2300.f, 2600.f, 1});

    std::vector<float> floor_v(4 * 9);
    std::vector<uint32_t> floor_i = {0, 2, 1, 0, 3, 2};
    quatf fq = mat3f::packTangentFrame(mat3f{float3{1, 0, 0}, float3{0, 0, -1}, float3{0, 1, 0}});
    V3 fp[4] = {{-HW, 0, -HH}, {HW, 0, -HH}, {HW, 0, HH}, {-HW, 0, HH}};
    float uv[4][2] = {{0, 0}, {1, 0}, {1, 1}, {0, 1}};
    for (int i = 0; i < 4; ++i) {
        float* o = floor_v.data() + i * 9;
        o[0] = fp[i].x; o[1] = fp[i].y; o[2] = fp[i].z;
        o[3] = uv[i][0]; o[4] = uv[i][1];
        o[5] = fq.x; o[6] = fq.y; o[7] = fq.z; o[8] = fq.w;
    }
    std::vector<float> cube_v;
    std::vector<uint32_t> cube_i;
    for (int face = 0; face < 6; ++face) {
        int ax = face / 2;
        float sgn = (face & 1) ? 1.f : -1.f;
        V3 N = axis[ax] * sgn;
        V3 T = axis[(ax + 1) % 3];
        V3 B = cross(N, T);
        V3 c = center + N * half;
        uint32_t base = (uint32_t)(cube_v.size() / 7);
        push_vert(cube_v, c - T * half - B * half, T, N);
        push_vert(cube_v, c + T * half - B * half, T, N);
        push_vert(cube_v, c + T * half + B * half, T, N);
        push_vert(cube_v, c - T * half + B * half, T, N);
        cube_i.insert(cube_i.end(), {base, base + 1, base + 2, base, base + 2, base + 3});
    }
    std::vector<float> panel_v;
    std::vector<uint32_t> panel_i = {0, 1, 2, 0, 2, 3};
    {
        V3 c{0.0f, 0.55f, -1.15f};
        float hw = 1.6f, hh = 0.7f;
        V3 p[4] = {c + V3{-hw, -hh, 0}, c + V3{hw, -hh, 0}, c + V3{hw, hh, 0}, c + V3{-hw, hh, 0}};
        for (int i = 0; i < 4; ++i) push_vert(panel_v, p[i], {1, 0, 0}, {0, 0, 1});
    }

    auto add = [&](std::vector<float>& verts, std::vector<uint32_t>& idx, int stride, bool hasUv, MaterialInstance* mi,
                   bool cast_s, bool recv) {
        utils::Entity e = utils::EntityManager::get().create();
        uint32_t nv = hasUv ? 4u : (uint32_t)(verts.size() / 7);
        auto b = VertexBuffer::Builder().vertexCount(nv).bufferCount(1);
        b.attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, (uint32_t)stride);
        if (hasUv) {
            b.attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, (uint32_t)stride);
            b.attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, (uint32_t)stride);
        } else b.attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, (uint32_t)stride);
        VertexBuffer* vb = b.build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(verts.data(), verts.size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(idx.data(), idx.size() * sizeof(uint32_t)));
        RenderableManager::Builder(1).boundingBox({{-4, -4, -4}, {4, 4, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, mi).culling(false).castShadows(cast_s).receiveShadows(recv)
            .build(*engine, e);
        scene->addEntity(e);
        return e;
    };
    utils::Entity floor = add(floor_v, floor_i, 36, true, floorMi, false, true);
    utils::Entity cube = add(cube_v, cube_i, 28, false, glass, true, false);
    utils::Entity panelE = add(panel_v, panel_i, 28, false, panel, false, false);
    utils::Entity filter = utils::EntityManager::get().create();
    {
        auto b = VertexBuffer::Builder().vertexCount(4).bufferCount(1);
        b.attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 36);
        b.attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 36);
        b.attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, 36);
        VertexBuffer* vb = b.build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(floor_v.data(), floor_v.size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(floor_i.data(), floor_i.size() * sizeof(uint32_t)));
        RenderableManager::Builder(1).boundingBox({{-4, -4, -4}, {4, 4, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, atrous1).culling(false).castShadows(false).receiveShadows(false)
            .build(*engine, filter);
    }
    utils::Entity sun_e = utils::EntityManager::get().create();
    LightManager::ShadowOptions shopt;
    shopt.mapSize = 2048;
    shopt.vsm.blurWidth = 8.f;
    shopt.vsm.elvsm = true;
    LightManager::Builder(LightManager::Type::SUN)
        .color(float3{1, 0.98f, 0.96f}).intensity(lux).direction(float3{sun.x, sun.y, sun.z})
        .sunAngularRadius(0.35f).sunHaloSize(1.2f).castShadows(true).shadowOptions(shopt)
        .build(*engine, sun_e);
    scene->addEntity(sun_e);

    auto& rm = engine->getRenderableManager();
    auto open = [&]() {
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) return true;
        return false;
    };
    auto upload = [&](std::vector<float>& tex) {
        srcTex->setImage(*engine, 0,
                         Texture::PixelBufferDescriptor(tex.data(), tex.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                        backend::PixelDataType::FLOAT, 1, 0, 0, TEX));
        engine->flushAndWait();
    };
    auto grab = [&](const char* path) -> bool {
        if (!open()) return false;
        view->setRenderTarget(beautyRT);
        renderer->render(view);
        std::vector<uint8_t> frame((size_t)W * H * 4);
        renderer->readPixels(beautyRT, 0, 0, W, H,
                             backend::PixelBufferDescriptor(frame.data(), frame.size(), backend::PixelDataFormat::RGBA,
                                                            backend::PixelDataType::UBYTE));
        renderer->endFrame();
        engine->flushAndWait();
        return write_png(path, W, H, frame.data());
    };
    auto run_atrous = [&]() -> bool {
        bool post = true;
        view->setPostProcessingEnabled(false);
        view->setScreenSpaceRefractionEnabled(false);
        view->setBloomOptions(BloomOptions{});
        scene->setSkybox(nullptr);
        camera->setExposure(1.f);
        camera->lookAt(float3{0, 4, 0}, float3{0, 0, 0}, float3{0, 0, -1});
        camera->setProjection(Camera::Projection::ORTHO, -HW, HW, -HH, HH, 0.01, 10.0);
        view->setViewport({0, 0, (uint32_t)TEX, (uint32_t)TEX});
        scene->remove(floor);
        scene->remove(cube);
        scene->remove(panelE);
        scene->remove(sun_e);
        scene->addEntity(filter);
        rm.setMaterialInstanceAt(rm.getInstance(filter), 0, atrous1);
        Renderer::ClearOptions c;
        c.clear = true;
        c.discard = true;
        c.clearColor = {0, 0, 0, 0};
        renderer->setClearOptions(c);
        if (!open()) return false;
        view->setRenderTarget(tempRT);
        renderer->render(view);
        renderer->endFrame();
        tempTex->generateMipmaps(*engine);
        engine->flushAndWait();
        rm.setMaterialInstanceAt(rm.getInstance(filter), 0, atrous2);
        if (!open()) return false;
        view->setRenderTarget(filtRT);
        renderer->render(view);
        renderer->endFrame();
        engine->flushAndWait();
        scene->remove(filter);
        scene->addEntity(floor);
        scene->addEntity(cube);
        scene->addEntity(panelE);
        scene->addEntity(sun_e);
        scene->setSkybox(Skybox::Builder().color({0.72f, 0.74f, 0.78f, 1}).build(*engine));
        view->setPostProcessingEnabled(post);
        view->setScreenSpaceRefractionEnabled(true);
        view->setBloomOptions(bloom);
        view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
        camera->setExposure(2.8f, 1.f / 125.f, 100.f);
        camera->lookAt(float3{0.55f, 0.78f, 0.70f}, float3{0, 0.04f, 0}, float3{0, 1, 0});
        camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);
        return true;
    };

    upload(mono);
    floorMi->setParameter("disperse", 0.f);
    floorMi->setParameter("caustic", srcTex, lin);
    if (!grab("out/hp-x-mono.png")) return fail("mono");
    floorMi->setParameter("disperse", 1.f);
    if (!grab("out/hp-x-disp.png")) return fail("disp");
    if (!run_atrous()) return fail("atrous");
    floorMi->setParameter("disperse", 0.f);
    floorMi->setParameter("caustic", filtTex, lin);
    if (!grab("out/hp-x-recon.png")) return fail("recon");
    floorMi->setParameter("disperse", 1.f);
    if (!grab("out/hp-x-both.png")) return fail("both");

    upload(spec);
    floorMi->setParameter("disperse", 0.f);
    floorMi->setParameter("caustic", srcTex, lin);
    if (!grab("out/hp-x-spec.png")) return fail("spec");
    std::printf("PASS prism-cross\n");
    return 0;
}
