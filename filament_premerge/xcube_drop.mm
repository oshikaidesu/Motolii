// Drop-in X-cube: the keeper look of xcube_gpu.mm (`try 11 56`), unchanged materials, restructured so that
//   * the cube is ONE entity with a transform; position / yaw / size are given as a pose,
//   * the sun direction is a value,
//   * the light map follows the cube (it is centred on the cube's x/z), and the floor reads it by world position,
//   * the whole thing is drawn in ONE frame: hits -> bands -> blur -> blur -> picture (no flush between passes,
//     no per-frame readback, nothing computed on the CPU except the pose uniforms).
// Materials reused as they are: xcube_hit, xcube_splat, caustic_atrous, glass_shadow, emissive.
// New: floor_recv_world (floor_recv with a world-space lookup).
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
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>

using namespace filament;
using namespace filament::math;

static const int W = 1920, H = 1080, MAP = 512, PATHS = 8;
static int GRID = 56, NW = 11;
static const float HW = 1.55f;  // half size of the light map, world units

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
static void push_vert(std::vector<float>& v, float3 p, float3 T, float3 N) {
    float3 n = normalize(N - T * dot(N, T));
    float3 B = normalize(cross(n, T));
    quatf q = mat3f::packTangentFrame(mat3f{T, B, n});
    float f[7] = {p.x, p.y, p.z, q.x, q.y, q.z, q.w};
    v.insert(v.end(), f, f + 7);
}

struct Target {
    Texture* color;
    RenderTarget* rt;
};
struct Pose {
    float3 center;   // world position of the cube's centre
    float yaw;       // rotation about +Y, radians (same convention as xcube_gpu.mm's rotateY)
    float half;      // half edge length
};

int main(int argc, char** argv) {
    if (argc > 2) {
        NW = std::max(1, std::atoi(argv[1]));
        GRID = std::max(2, std::atoi(argv[2]));
    }
    const float nd = 1.55f, dispersion = 36.f, lux = 22000.f;

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    auto& em = utils::EntityManager::get();
    auto& tm = engine->getTransformManager();

    auto mat = [&](const char* path) -> Material* {
        auto b = load_file(path);
        return b.empty() ? nullptr : Material::Builder().package(b.data(), b.size()).build(*engine);
    };
    Material* hitMat = mat("xcube_hit.filamat");
    Material* splatMat = mat("xcube_splat.filamat");
    Material* atrousMat = mat("caustic_atrous.filamat");
    Material* glassMat = mat("glass_shadow.filamat");
    Material* floorMat = mat("floor_recv_world.filamat");
    Material* emat = mat("emissive.filamat");
    if (!hitMat || !splatMat || !atrousMat || !glassMat || !floorMat || !emat) return fail("material");

    auto make_rt = [&](int w, int h, int levels, bool mip, Texture::InternalFormat fmt) {
        auto usage = Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC;
        if (mip) usage |= Texture::Usage::BLIT_DST | Texture::Usage::GEN_MIPMAPPABLE;
        Texture* color = Texture::Builder().width((uint32_t)w).height((uint32_t)h).levels((uint8_t)levels).usage(usage).format(fmt).build(*engine);
        Texture* depth = Texture::Builder().width((uint32_t)w).height((uint32_t)h).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH32F).build(*engine);
        RenderTarget* rt = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, color)
                               .texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
        return Target{color, rt};
    };
    const int hitW = GRID * NW, hitH = GRID * PATHS;
    Target hit = make_rt(hitW, hitH, 1, false, Texture::InternalFormat::RGBA16F);
    Target map = make_rt(MAP, MAP, 1, false, Texture::InternalFormat::RGBA16F);
    Target temp = make_rt(MAP, MAP, 10, true, Texture::InternalFormat::RGBA16F);
    Target filt = make_rt(MAP, MAP, 1, false, Texture::InternalFormat::RGBA16F);
    Target beauty = make_rt(W, H, 1, false, Texture::InternalFormat::RGBA8);

    TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST, TextureSampler::WrapMode::CLAMP_TO_EDGE);
    TextureSampler lin(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE);
    TextureSampler mipped(TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE);

    auto renderable = [&](VertexBuffer* vb, IndexBuffer* ib, MaterialInstance* mi, bool cast, bool recv, float box = 8.f) {
        utils::Entity e = em.create();
        RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {box, box, box}})  // centre, half extent
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, mi).culling(false).castShadows(cast).receiveShadows(recv).build(*engine, e);
        return e;
    };

    // ---- pass 1: one pixel, one ray (xcube_hit, device-space quad)
    MaterialInstance* hitMi = hitMat->createInstance();
    hitMi->setParameter("grid", (float)GRID);
    hitMi->setParameter("bands", (float)NW);
    hitMi->setParameter("ior", nd);
    hitMi->setParameter("dispersion", dispersion);
    hitMi->setParameter("hitW", (float)hitW);
    hitMi->setParameter("hitH", (float)hitH);
    hitMi->setParameter("bevel", 0.f);
    hitMi->setParameter("tilt", 0.f);
    static float hq[] = {-1, -1, 0, 0, 0, 1, -1, 0, 1, 0, 1, 1, 0, 1, 1, -1, 1, 0, 0, 1};
    static uint32_t hi[] = {0, 1, 2, 0, 2, 3};
    VertexBuffer* hitVB = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                              .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                              .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*engine);
    hitVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(hq, sizeof(hq)));
    IndexBuffer* hitIB = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    hitIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(hi, sizeof(hi)));
    Scene* sHit = engine->createScene();
    sHit->addEntity(renderable(hitVB, hitIB, hitMi, false, false));

    // ---- pass 2: neighbouring rays of one path become a band (xcube_splat)
    MaterialInstance* splatMi = splatMat->createInstance();
    splatMi->setParameter("hits", hit.color, nearest);
    splatMi->setParameter("hitW", (float)hitW);
    splatMi->setParameter("hitH", (float)hitH);
    splatMi->setParameter("grid", (float)GRID);
    splatMi->setParameter("bands", (float)NW);
    splatMi->setParameter("lux", lux);
    const int cells = GRID - 1;
    const int nvert = cells * cells * NW * PATHS * 6;
    static std::vector<float> sv;
    static std::vector<uint32_t> si;
    sv.assign((size_t)nvert * 5, 0.f);
    si.resize((size_t)nvert);
    for (int id = 0; id < cells * cells; ++id)
        for (int band = 0; band < NW; ++band)
            for (int path = 0; path < PATHS; ++path)
                for (int c = 0; c < 6; ++c) {
                    int v = ((id * NW + band) * PATHS + path) * 6 + c;
                    sv[(size_t)v * 5 + 3] = (float)band;
                    si[(size_t)v] = (uint32_t)v;
                }
    VertexBuffer* splatVB = VertexBuffer::Builder().vertexCount((uint32_t)nvert).bufferCount(1)
                                .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                                .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*engine);
    splatVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(sv.data(), sv.size() * sizeof(float)));
    IndexBuffer* splatIB = IndexBuffer::Builder().indexCount((uint32_t)nvert).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    splatIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(si.data(), si.size() * sizeof(uint32_t)));
    Scene* sPath = engine->createScene();
    sPath->addEntity(renderable(splatVB, splatIB, splatMi, false, false));

    // ---- pass 3+4: a-trous blur of the map (caustic_atrous), on a quad that follows the map
    static std::vector<float> quad_v(4 * 9);
    static std::vector<uint32_t> quad_i = {0, 2, 1, 0, 3, 2};
    {
        quatf fq = mat3f::packTangentFrame(mat3f{float3{1, 0, 0}, float3{0, 0, -1}, float3{0, 1, 0}});
        const float3 fp[4] = {{-HW, 0, -HW}, {HW, 0, -HW}, {HW, 0, HW}, {-HW, 0, HW}};
        const float uv[4][2] = {{0, 0}, {1, 0}, {1, 1}, {0, 1}};
        for (int i = 0; i < 4; ++i) {
            float* o = quad_v.data() + i * 9;
            o[0] = fp[i].x; o[1] = 0; o[2] = fp[i].z;
            o[3] = uv[i][0]; o[4] = uv[i][1];
            o[5] = fq.x; o[6] = fq.y; o[7] = fq.z; o[8] = fq.w;
        }
    }
    VertexBuffer* quadVB = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 36)
                               .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 36)
                               .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, 36).build(*engine);
    quadVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(quad_v.data(), quad_v.size() * sizeof(float)));
    IndexBuffer* quadIB = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    quadIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(quad_i.data(), quad_i.size() * sizeof(uint32_t)));
    auto atrous = [&](Texture* src, const TextureSampler& s, float stride, float mip, float second) {
        MaterialInstance* mi = atrousMat->createInstance();
        mi->setParameter("src", src, s);
        mi->setParameter("texel", float2{1.f / MAP, 1.f / MAP});
        mi->setParameter("stride", stride);
        mi->setParameter("lumWeight", 0.f);
        mi->setParameter("mip", mip);
        mi->setParameter("second", second);
        return mi;
    };
    utils::Entity blur1 = renderable(quadVB, quadIB, atrous(map.color, lin, 1.f, 0.f, 0.f), false, false);
    utils::Entity blur2 = renderable(quadVB, quadIB, atrous(temp.color, mipped, 2.f, 1.f, 1.f), false, false);
    tm.create(blur1);
    tm.create(blur2);
    Scene* sBlur1 = engine->createScene();
    Scene* sBlur2 = engine->createScene();
    sBlur1->addEntity(blur1);
    sBlur2->addEntity(blur2);

    // ---- the picture: floor (any size, no UVs needed), the cube entity, a bright wall panel, the sun
    Scene* scene = engine->createScene();
    MaterialInstance* floorMi = floorMat->createInstance();
    floorMi->setParameter("color", float4{0.86f, 0.85f, 0.84f, 1});
    floorMi->setParameter("roughness", 0.42f);
    floorMi->setParameter("mapSize", 2.f * HW);
    floorMi->setParameter("caustic", filt.color, lin);
    const float env_floor = std::getenv("FLOOR") ? (float)std::atof(std::getenv("FLOOR")) : HW;
    static std::vector<float> floor_v;
    static std::vector<uint32_t> floor_i = {0, 2, 1, 0, 3, 2};
    {
        const float F = env_floor;  // may be bigger than the light map: the floor is not tied to it
        const float3 fp[4] = {{-F, 0, -F}, {F, 0, -F}, {F, 0, F}, {-F, 0, F}};
        for (int i = 0; i < 4; ++i) push_vert(floor_v, fp[i], {1, 0, 0}, {0, 1, 0});
    }
    auto lit_buffers = [&](std::vector<float>& v, std::vector<uint32_t>& i) {
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(v.size() / 7)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                               .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28).build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(v.data(), v.size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)i.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(i.data(), i.size() * sizeof(uint32_t)));
        return std::pair<VertexBuffer*, IndexBuffer*>{vb, ib};
    };
    auto [floorVB, floorIB] = lit_buffers(floor_v, floor_i);
    scene->addEntity(renderable(floorVB, floorIB, floorMi, false, true, 4.f));

    // unit cube (+-0.5); its pose is the entity transform
    static std::vector<float> cube_v;
    static std::vector<uint32_t> cube_i;
    {
        const float3 ax[3] = {{1, 0, 0}, {0, 1, 0}, {0, 0, 1}};
        for (int face = 0; face < 6; ++face) {
            int a = face / 2;
            float sgn = (face & 1) ? 1.f : -1.f;
            float3 N = ax[a] * sgn, T = ax[(a + 1) % 3], B = cross(N, T), c = N * 0.5f;
            uint32_t base = (uint32_t)(cube_v.size() / 7);
            push_vert(cube_v, c - T * 0.5f - B * 0.5f, T, N);
            push_vert(cube_v, c + T * 0.5f - B * 0.5f, T, N);
            push_vert(cube_v, c + T * 0.5f + B * 0.5f, T, N);
            push_vert(cube_v, c - T * 0.5f + B * 0.5f, T, N);
            cube_i.insert(cube_i.end(), {base, base + 1, base + 2, base, base + 2, base + 3});
        }
    }
    auto [cubeVB, cubeIB] = lit_buffers(cube_v, cube_i);
    MaterialInstance* glass = glassMat->createInstance();
    glass->setParameter("ior", nd);
    glass->setParameter("transmission", 1.f);
    glass->setParameter("dispersion", dispersion);
    glass->setParameter("reflectance", 0.5f);
    glass->setParameter("alpha", 1.f);
    glass->setParameter("absorption", float3{0.02f, 0.015f, 0.02f});
    utils::Entity cube = renderable(cubeVB, cubeIB, glass, true, false, 0.5f);  // the unit cube itself; the transform scales it
    tm.create(cube);
    scene->addEntity(cube);

    static std::vector<float> panel_v;
    static std::vector<uint32_t> panel_i = {0, 1, 2, 0, 2, 3};
    {
        const float3 c{0.0f, 0.55f, -1.15f};
        const float hw = 1.6f, hh = 0.7f;
        const float3 p[4] = {c + float3{-hw, -hh, 0}, c + float3{hw, -hh, 0}, c + float3{hw, hh, 0}, c + float3{-hw, hh, 0}};
        for (int i = 0; i < 4; ++i) push_vert(panel_v, p[i], {1, 0, 0}, {0, 0, 1});
    }
    auto [panelVB, panelIB] = lit_buffers(panel_v, panel_i);
    MaterialInstance* panel = emat->createInstance();
    panel->setParameter("color", float4{2200.f, 2300.f, 2600.f, 1});
    scene->addEntity(renderable(panelVB, panelIB, panel, false, false, 4.f));

    utils::Entity sunE = em.create();
    LightManager::ShadowOptions shopt;
    shopt.mapSize = 512;
    shopt.shadowNearHint = 1.0f;
    shopt.shadowFarHint = 100.0f;
    shopt.constantBias = 0.001f;
    shopt.normalBias = 1.0f;
    LightManager::Builder(LightManager::Type::SUN).color(float3{1, 0.98f, 0.96f}).intensity(lux).direction(float3{0, -1, 0})
        .sunAngularRadius(0.35f).sunHaloSize(1.2f).castShadows(true).shadowOptions(shopt).build(*engine, sunE);
    scene->addEntity(sunE);
    scene->setSkybox(Skybox::Builder().color({0.72f, 0.74f, 0.78f, 1}).build(*engine));

    // ---- cameras and views (one View per pass, all in the same frame)
    Camera* camFlat = engine->createCamera(em.create());   // unused by the device-space quad, needed by the View
    camFlat->setExposure(1.f);
    Camera* camTop = engine->createCamera(em.create());    // looks straight down at the light map
    camTop->setExposure(1.f);
    camTop->setProjection(Camera::Projection::ORTHO, -HW, HW, -HW, HW, 0.01, 10.0);
    Camera* camera = engine->createCamera(em.create());
    camera->setExposure(2.8f, 1.f / 125.f, 100.f);
    camera->lookAt(float3{0.55f, 0.78f, 0.70f}, float3{0, 0.04f, 0}, float3{0, 1, 0});
    camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);

    auto pass_view = [&](Scene* s, Camera* c, RenderTarget* rt, int w, int h) {
        View* v = engine->createView();
        v->setScene(s);
        v->setCamera(c);
        v->setRenderTarget(rt);
        v->setViewport({0, 0, (uint32_t)w, (uint32_t)h});
        v->setDithering(View::Dithering::NONE);
        v->setPostProcessingEnabled(false);
        v->setShadowingEnabled(false);
        return v;
    };
    View* vHit = pass_view(sHit, camFlat, hit.rt, hitW, hitH);
    View* vPath = pass_view(sPath, camTop, map.rt, MAP, MAP);
    View* vBlur1 = pass_view(sBlur1, camTop, temp.rt, MAP, MAP);
    View* vBlur2 = pass_view(sBlur2, camTop, filt.rt, MAP, MAP);
    View* vBeauty = engine->createView();
    vBeauty->setScene(scene);
    vBeauty->setCamera(camera);
    vBeauty->setRenderTarget(beauty.rt);
    vBeauty->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    vBeauty->setDithering(View::Dithering::NONE);
    static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
    vBeauty->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.04f).saturation(1.08f).build(*engine));
    vBeauty->setPostProcessingEnabled(true);
    vBeauty->setScreenSpaceRefractionEnabled(true);
    vBeauty->setShadowType(View::ShadowType::PCF);
    BloomOptions bloom{};
    bloom.enabled = true;
    bloom.strength = 0.18f;
    bloom.levels = 6;
    bloom.resolution = std::getenv("BLOOM_RES") ? (uint32_t)std::atoi(std::getenv("BLOOM_RES")) : 1080;
    if (std::getenv("NOBLOOM")) bloom.enabled = false;
    bloom.threshold = true;
    bloom.highlight = 10.f;
    bloom.blendMode = BloomOptions::BlendMode::ADD;
    vBeauty->setBloomOptions(bloom);
    vBeauty->setAntiAliasing(std::getenv("NOFXAA") ? View::AntiAliasing::NONE : View::AntiAliasing::FXAA);
    if (std::getenv("NOREFR")) vBeauty->setScreenSpaceRefractionEnabled(false);
    if (std::getenv("NOSHADOW")) vBeauty->setShadowingEnabled(false);

    Renderer::ClearOptions clearOpt;
    clearOpt.clear = true;
    clearOpt.discard = true;
    clearOpt.clearColor = {0, 0, 0, 0};
    renderer->setClearOptions(clearOpt);

    // ---- the only per-frame CPU work: a handful of uniforms and two transforms
    auto set_sun = [&](float3 sun) {
        float3 D = normalize(sun);
        float3 up = std::fabs(D.y) < 0.9f ? float3{0, 1, 0} : float3{1, 0, 0};
        float3 U = normalize(cross(up, D));
        float3 V = cross(D, U);
        hitMi->setParameter("dir", D);
        hitMi->setParameter("tangent", U);
        hitMi->setParameter("bitangent", V);
        auto& lm = engine->getLightManager();
        lm.setDirection(lm.getInstance(sunE), D);
    };
    auto set_pose = [&](const Pose& p) {
        float c = std::cos(p.yaw), s = std::sin(p.yaw);
        float3 ax{c, 0, -s}, ay{0, 1, 0}, az{s, 0, c};  // rotateY applied to the unit axes
        hitMi->setParameter("center", p.center);
        hitMi->setParameter("ax", ax);
        hitMi->setParameter("ay", ay);
        hitMi->setParameter("az", az);
        hitMi->setParameter("halfExtent", p.half);
        hitMi->setParameter("extent", p.half * 3.1f);
        splatMi->setParameter("extent", p.half * 3.1f);
        glass->setParameter("thickness", p.half * 2.f);
        const float e = 2.f * p.half;
        tm.setTransform(tm.getInstance(cube), mat4f(float4{ax * e, 0}, float4{ay * e, 0}, float4{az * e, 0}, float4{p.center, 1}));
        // the light map rides under the cube
        const float3 m{p.center.x, 0, p.center.z};
        camTop->lookAt(m + float3{0, 4, 0}, m, float3{0, 0, -1});
        tm.setTransform(tm.getInstance(blur1), mat4f::translation(m));
        tm.setTransform(tm.getInstance(blur2), mat4f::translation(m));
        floorMi->setParameter("mapCenter", float2{m.x, m.z});
    };
    auto open = [&]() {
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) return true;
        return false;
    };
    const bool split = std::getenv("SPLIT") != nullptr;  // debug: one frame per pass, like xcube_gpu.mm
    auto step = [&]() {
        if (!split) return;
        renderer->endFrame();
        engine->flushAndWait();
        open();
    };
    auto draw = [&]() {
        renderer->render(vHit);
        step();
        renderer->render(vPath);
        step();
        renderer->render(vBlur1);
        if (split) { renderer->endFrame(); temp.color->generateMipmaps(*engine); engine->flushAndWait(); open(); }
        else temp.color->generateMipmaps(*engine);
        renderer->render(vBlur2);
        step();
        renderer->render(vBeauty);
    };
    std::vector<uint8_t> frame((size_t)W * H * 4);
    auto grab = [&](const char* path) -> bool {
        if (!open()) return false;
        draw();
        renderer->readPixels(beauty.rt, 0, 0, W, H,
                             backend::PixelBufferDescriptor(frame.data(), frame.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
        renderer->endFrame();
        engine->flushAndWait();
        return write_png(path, W, H, frame.data());
    };
    std::system("mkdir -p out");

    // ---- 1. the keeper pose: must match xcube_gpu.mm `try 11 56`
    const float3 sun0 = normalize(float3{0.92f, -0.42f, 0.22f});
    const Pose keeper{{0.f, 0.16f + 0.001f, 0.f}, 0.62f, 0.16f};
    set_sun(sun0);
    set_pose(keeper);
    for (int i = 0; i < 3; ++i) {  // warm up (shader compilation, first-use mip chain)
        if (!open()) return fail("warm");
        draw();
        renderer->endFrame();
        engine->flushAndWait();
    }
    if (!grab("out/xcube-drop-first.png")) return fail("keeper");
    std::vector<uint8_t> keeperPic = frame;

    // ---- 2. drop it somewhere else, turn it, resize it, move the sun: everything follows
    if (true) {
        set_pose({{0.30f, 0.11f + 0.001f, -0.22f}, 0.15f, 0.11f});
        if (!grab("out/xcube-drop-moved.png")) return fail("moved");
        set_pose({{-0.22f, 0.20f + 0.001f, 0.10f}, 1.10f, 0.20f});
        set_sun(normalize(float3{0.55f, -0.50f, -0.45f}));
        if (!grab("out/xcube-drop-turned.png")) return fail("turned");
    }

    // ---- 3. timing. Wall clock per frame, plus Filament's own FrameInfo (GPU duration, main thread, backend thread).
    const bool still = std::getenv("STILL") != nullptr;      // do not move the cube / sun
    const bool nowait = std::getenv("NOWAIT") != nullptr;    // do not flushAndWait every frame (as a real app would not)
    const int skip = std::getenv("SKIP") ? std::atoi(std::getenv("SKIP")) : 0;  // 1 = picture only, 2 = light only
    auto draw_timed = [&]() {
        if (skip != 1) {
            renderer->render(vHit);
            renderer->render(vPath);
            renderer->render(vBlur1);
            temp.color->generateMipmaps(*engine);
            renderer->render(vBlur2);
        }
        if (skip != 2) renderer->render(vBeauty);
    };
    const int frames = 240;
    std::vector<double> wall, gpu, mainT, backT;
    auto pct = [](std::vector<double> v, double p) {
        if (v.empty()) return -1.0;
        std::sort(v.begin(), v.end());
        return v[std::min(v.size() - 1, (size_t)(v.size() * p))];
    };
    auto tAll = std::chrono::steady_clock::now();
    for (int i = 0; i < frames; ++i) {
        if (!still) {
            float t = (float)i / 120.f * 6.2831853f;
            set_pose({{0.25f * std::cos(t), 0.161f, 0.20f * std::sin(t)}, 0.62f + t, 0.16f});
            set_sun(normalize(float3{0.92f * std::cos(0.3f * std::sin(t)), -0.42f, 0.22f + 0.4f * std::sin(t)}));
        }
        auto t0 = std::chrono::steady_clock::now();
        if (!open()) return fail("frame");
        draw_timed();
        renderer->endFrame();
        if (!nowait) engine->flushAndWait();
        wall.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
        if (i % 16 == 15 || i == frames - 1) {
            auto hist = renderer->getFrameInfoHistory(16);
            for (auto const& f : hist) {
                if (f.gpuFrameDuration > 0) gpu.push_back(f.gpuFrameDuration * 1e-6);
                if (f.endFrame > 0 && f.beginFrame > 0) mainT.push_back((f.endFrame - f.beginFrame) * 1e-6);
                if (f.backendEndFrame > 0 && f.backendBeginFrame > 0) backT.push_back((f.backendEndFrame - f.backendBeginFrame) * 1e-6);
            }
        }
    }
    engine->flushAndWait();
    double total = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - tAll).count();
    std::printf("TIME %dx%d, %d wavelengths, grid %d, PCF 512%s%s%s: %d frames in %.0f ms = %.2f ms/frame average\n", W, H, NW, GRID,
                still ? ", still" : ", cube+sun moving", nowait ? ", no per-frame wait" : ", flushAndWait each frame",
                skip == 1 ? ", PICTURE ONLY" : (skip == 2 ? ", LIGHT ONLY" : ""), frames, total, total / frames);
    auto line = [&](const char* name, std::vector<double>& v) {
        std::printf("     %-34s n=%3zu  min %6.2f  median %6.2f  95%% %6.2f  max %6.2f ms\n", name, v.size(), pct(v, 0.0), pct(v, 0.5), pct(v, 0.95), pct(v, 1.0));
    };
    line("wall clock per frame", wall);
    line("GPU (Filament gpuFrameDuration)", gpu);
    line("main thread beginFrame..endFrame", mainT);
    line("backend thread", backT);
    std::printf("     max history %zu frames; 60fps budget 16.67 ms\n", renderer->getMaxFrameHistorySize());

    // ---- 3b. the real question: paced at 60 Hz like an app, does every frame finish inside its 16.67 ms slot?
    if (std::getenv("PACE")) {
        const int n = std::getenv("PACE_FRAMES") ? std::atoi(std::getenv("PACE_FRAMES")) : 600;
        const double hz = std::getenv("PACE_HZ") ? std::atof(std::getenv("PACE_HZ")) : 60.0;
        const double period = 1000.0 / hz;
        std::vector<double> g, lat;
        int skipped = 0, drawn = 0;
        auto start = std::chrono::steady_clock::now();
        for (int i = 0; i < n; ++i) {
            auto due = start + std::chrono::duration<double, std::milli>(period * i);
            while (std::chrono::steady_clock::now() < due) {}
            float t = (float)i / 120.f * 6.2831853f;
            if (!still) {
                set_pose({{0.25f * std::cos(t), 0.161f, 0.20f * std::sin(t)}, 0.62f + t, 0.16f});
                set_sun(normalize(float3{0.92f * std::cos(0.3f * std::sin(t)), -0.42f, 0.22f + 0.4f * std::sin(t)}));
            }
            // One attempt per tick. beginFrame() returning false is Filament saying "skip this frame, I am behind".
            if (!renderer->beginFrame(swap)) { ++skipped; continue; }
            draw_timed();
            renderer->endFrame();
            ++drawn;
            if (drawn % 16 == 0 || i == n - 1) {
                engine->flush();
                auto hist = renderer->getFrameInfoHistory(16);
                for (auto const& f : hist) {
                    if (f.gpuFrameDuration > 0) g.push_back(f.gpuFrameDuration * 1e-6);
                    if (f.gpuFrameComplete > 0 && f.beginFrame > 0) lat.push_back((f.gpuFrameComplete - f.beginFrame) * 1e-6);
                }
            }
        }
        engine->flushAndWait();
        double total2 = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - start).count();
        size_t late = 0;
        for (double v : lat) if (v > period) ++late;
        std::printf("PACE %.0f Hz for %d frames (%.1f s, target %.1f s)%s\n", hz, n, total2 / 1000.0, n * period / 1000.0, still ? ", still" : ", cube+sun moving");
        std::printf("     %-34s n=%3zu  min %6.2f  median %6.2f  95%% %6.2f  99%% %6.2f  max %6.2f ms\n", "GPU time per frame", g.size(), pct(g, 0.0), pct(g, 0.5), pct(g, 0.95), pct(g, 0.99), pct(g, 1.0));
        std::printf("     %-34s n=%3zu  min %6.2f  median %6.2f  95%% %6.2f  99%% %6.2f  max %6.2f ms\n", "beginFrame -> GPU finished", lat.size(), pct(lat, 0.0), pct(lat, 0.5), pct(lat, 0.95), pct(lat, 0.99), pct(lat, 1.0));
        std::printf("     finished after their %.2f ms slot: %zu of %zu measured (%.1f%%)\n", period, late, lat.size(), lat.empty() ? 0.0 : 100.0 * late / lat.size());
        std::printf("     DROPPED (Filament asked to skip): %d of %d ticks (%.1f%%)  -> delivered %.1f fps of %.0f\n", skipped, n, 100.0 * skipped / n, drawn / (total2 / 1000.0), hz);
    }

    // ---- 4. compare the keeper pose with the picture xcube_gpu.mm made
    {
        // re-render the keeper pose last so the saved file is what was compared
        set_sun(sun0);
        set_pose(keeper);
        if (!grab("out/xcube-drop-keeper.png")) return fail("keeper2");
        (void)keeperPic;
        std::printf("wrote out/xcube-drop-keeper.png, out/xcube-drop-moved.png, out/xcube-drop-turned.png\n");
    }
    return 0;
}
