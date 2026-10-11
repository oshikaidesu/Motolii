// A plain glass block, dropped on a floor, throwing sharp light.
//   * The floor light is computed by beam_glass.h (exact polygons per wavelength, nothing blurred) and drawn as
//     plain coloured triangles with the picture's own camera into a float buffer (beam_light.mat).
//   * The floor reads that buffer by screen position (floor_beam.mat). The shadow is Filament's own.
//   * The block is the same chamfered solid the light was traced through; each pixel follows its sight line
//     through that solid (glass_solid.mat) and looks at a picture of the scene without the block (backdrop).
// The only per-pose CPU work is beam::trace and one vertex upload; nothing is read back.
//
// Usage: ./out/glass_beam [name]        writes out/glass-beam-<name>.png
// Timing: PACE=1 [PACE_HZ=60] [PACE_FRAMES=600] moves the block and the sun every frame and counts dropped frames.
// REF=1 traces every band in full (the slow reference). DETAIL scales the colour step (1 = one pixel).
// Environment: CHAMFER (fraction of the edge, default 0.015), DISP (dispersion exaggeration, default 4),
//   BANDS (default 64), SUN="x y z", YAW, HALF, SS (light buffer scale, default 2), IBL (studio light, default 1500), APERTURE (default 5.0),
//   CAM="ex ey ez tx ty tz fov", NOGLASS, NOLIGHT.
#include "beam_glass.h"

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
#include <filament/TransformManager.h>
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

static const int W = 1920, H = 1080;
static const uint32_t MAX_LIGHT_VERTS = 600000;

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
static float env_f(const char* name, float d) { return std::getenv(name) ? (float)std::atof(std::getenv(name)) : d; }
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
    float3 center;
    float yaw;
    float half;
};

int main(int argc, char** argv) {
    const std::string name = argc > 1 ? argv[1] : "first";
    const float chamfer = env_f("CHAMFER", 0.03f);   // flat width as a fraction of the edge (real optical cubes: about 0.012)
    const float disp = env_f("DISP", 10.f);          // how much further than real glass the colours spread
    const float sunRadiusDeg = env_f("SUNR", 0.5f);  // half the sun's width in degrees (the real sun: 0.27)
    const int bands = (int)env_f("BANDS", 64.f);
    const int SS = std::max(1, (int)env_f("SS", 1.f));
    const bool filamentShadow = std::getenv("FILSHADOW") != nullptr;  // debug: Filament's own shadow map instead of the softened outline
    const float lux = 22000.f;
    const float3 sunColor{1.f, 0.98f, 0.96f};

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    auto& em = utils::EntityManager::get();
    auto& tm = engine->getTransformManager();
    auto& rm = engine->getRenderableManager();

    auto mat = [&](const char* path) -> Material* {
        auto b = load_file(path);
        return b.empty() ? nullptr : Material::Builder().package(b.data(), b.size()).build(*engine);
    };
    Material* lightMat = mat("beam_light.filamat");
    Material* floorMat = mat("floor_beam.filamat");
    Material* glassMat = mat("glass_solid.filamat");
    Material* shadowMat = mat("shadow_only.filamat");
    Material* softenMat = mat("beam_soften.filamat");
    if (!lightMat || !floorMat || !glassMat || !shadowMat || !softenMat) return fail("material");

    auto make_rt = [&](int w, int h, Texture::InternalFormat fmt, bool mips = false) {
        auto usage = Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC;
        if (mips) usage |= Texture::Usage::BLIT_DST | Texture::Usage::GEN_MIPMAPPABLE;
        const int levels = mips ? (int)std::floor(std::log2((double)std::max(w, h))) + 1 : 1;
        Texture* color = Texture::Builder().width((uint32_t)w).height((uint32_t)h).levels((uint8_t)levels).usage(usage).format(fmt).build(*engine);
        Texture* depth = Texture::Builder().width((uint32_t)w).height((uint32_t)h).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH32F).build(*engine);
        RenderTarget* rt = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, color)
                               .texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
        return Target{color, rt};
    };
    Target light = make_rt(W * SS, H * SS, Texture::InternalFormat::RGBA16F, true);   // the block's light on the floor
    Target shade = make_rt(W * SS, H * SS, Texture::InternalFormat::RGBA16F, true);   // the block's outline against the sun
    Target soft = make_rt(W, H, Texture::InternalFormat::RGBA16F);      // the light after the sun's width has softened it
    Target backdrop = make_rt(W, H, Texture::InternalFormat::RGBA16F);  // the scene without the block, for the block to look at
    Target beauty = make_rt(W, H, Texture::InternalFormat::RGBA8);
    TextureSampler lin(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE);
    TextureSampler mipped(TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE);

    auto renderable = [&](VertexBuffer* vb, IndexBuffer* ib, MaterialInstance* mi, bool cast, bool recv, float box = 8.f) {
        utils::Entity e = em.create();
        RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {box, box, box}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, mi).culling(false).castShadows(cast).receiveShadows(recv).build(*engine, e);
        return e;
    };
    auto lit_buffers = [&](std::vector<float>& v, std::vector<uint32_t>& i) {
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(v.size() / 7)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                               .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28).build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(v.data(), v.size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)i.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(i.data(), i.size() * sizeof(uint32_t)));
        return std::pair<VertexBuffer*, IndexBuffer*>{vb, ib};
    };

    // ---- the floor light: triangles from beam::trace, refilled when the pose or the sun changes
    VertexBuffer* lightVB = VertexBuffer::Builder().vertexCount(MAX_LIGHT_VERTS).bufferCount(1)
                                .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                                .attribute(VertexAttribute::COLOR, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28).build(*engine);
    static std::vector<uint32_t> light_i(MAX_LIGHT_VERTS);
    for (uint32_t i = 0; i < MAX_LIGHT_VERTS; ++i) light_i[i] = i;
    IndexBuffer* lightIB = IndexBuffer::Builder().indexCount(MAX_LIGHT_VERTS).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    lightIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(light_i.data(), light_i.size() * sizeof(uint32_t)));
    {
        // something valid in the buffer before the first trace
        auto* zero = new std::vector<float>(21, 0.f);
        lightVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(zero->data(), zero->size() * sizeof(float),
                                                                         [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, zero));
    }
    MaterialInstance* lightMi = lightMat->createInstance();
    utils::Entity lightE = renderable(lightVB, lightIB, lightMi, false, false, 100.f);
    Scene* sLight = engine->createScene();
    sLight->addEntity(lightE);
    utils::Entity shadeE = renderable(lightVB, lightIB, lightMat->createInstance(), false, false, 100.f);  // same buffer, the outline's part of it
    Scene* sShade = engine->createScene();
    sShade->addEntity(shadeE);

    // ---- softening pass (full-screen quad)
    MaterialInstance* softenMi = softenMat->createInstance();
    softenMi->setParameter("light", light.color, mipped);
    softenMi->setParameter("shade", shade.color, mipped);
    softenMi->setParameter("floorY", 0.f);
    softenMi->setParameter("sunTan", std::tan(sunRadiusDeg * 3.14159265f / 180.f));
    softenMi->setParameter("texel", float2{1.f / (W * SS), 1.f / (H * SS)});
    static float sq[] = {-1, -1, 0, 0, 0, 1, -1, 0, 1, 0, 1, 1, 0, 1, 1, -1, 1, 0, 0, 1};
    static uint32_t sqi[] = {0, 1, 2, 0, 2, 3};
    VertexBuffer* sqVB = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                             .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                             .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*engine);
    sqVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(sq, sizeof(sq)));
    IndexBuffer* sqIB = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    sqIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(sqi, sizeof(sqi)));
    Scene* sSoft = engine->createScene();
    sSoft->addEntity(renderable(sqVB, sqIB, softenMi, false, false));

    // ---- the picture
    Scene* scene = engine->createScene();
    MaterialInstance* floorMi = floorMat->createInstance();
    floorMi->setParameter("color", float4{0.86f, 0.85f, 0.84f, 1});
    floorMi->setParameter("roughness", env_f("FLOOR_ROUGH", 0.42f));
    floorMi->setParameter("reflectance", env_f("FLOOR_REFL", 0.35f));
    floorMi->setParameter("gain", sunColor * (lux / 3.14159265f));
    floorMi->setParameter("light", soft.color, lin);
    static std::vector<float> floor_v;
    static std::vector<uint32_t> floor_i = {0, 2, 1, 0, 3, 2};
    {
        const float F = env_f("FLOOR", 6.f);
        const float3 fp[4] = {{-F, 0, -F}, {F, 0, -F}, {F, 0, F}, {-F, 0, F}};
        for (int i = 0; i < 4; ++i) push_vert(floor_v, fp[i], {1, 0, 0}, {0, 1, 0});
    }
    auto [floorVB, floorIB] = lit_buffers(floor_v, floor_i);
    scene->addEntity(renderable(floorVB, floorIB, floorMi, false, true, 8.f));

    // the block: the same chamfered solid the light is traced through, as a unit mesh (+-0.5); the pose is its transform
    static std::vector<float> cube_v;
    static std::vector<uint32_t> cube_i;
    {
        beam::Solid unit = beam::chamfered_box({0, 0, 0}, 0.0, 0.5, chamfer);
        for (size_t f = 0; f < unit.faces.size(); ++f) {
            const beam::Poly& P = unit.faces[f];
            if (P.n < 3) continue;
            float3 N{(float)unit.planes[f].n.x, (float)unit.planes[f].n.y, (float)unit.planes[f].n.z};
            float3 e0{(float)(P.p[1].x - P.p[0].x), (float)(P.p[1].y - P.p[0].y), (float)(P.p[1].z - P.p[0].z)};
            float3 T = normalize(e0);
            // outward winding for Filament's back-face culling (the glass is single sided)
            beam::V3 nn = beam::cross(P.p[1] - P.p[0], P.p[2] - P.p[0]);
            const bool flip = beam::dot(nn, unit.planes[f].n) < 0;
            uint32_t base = (uint32_t)(cube_v.size() / 7);
            for (int i = 0; i < P.n; ++i) push_vert(cube_v, float3{(float)P.p[(size_t)i].x, (float)P.p[(size_t)i].y, (float)P.p[(size_t)i].z}, T, N);
            for (int i = 1; i + 1 < P.n; ++i) {
                if (flip) cube_i.insert(cube_i.end(), {base, base + (uint32_t)i + 1, base + (uint32_t)i});
                else cube_i.insert(cube_i.end(), {base, base + (uint32_t)i, base + (uint32_t)i + 1});
            }
        }
    }
    auto [cubeVB, cubeIB] = lit_buffers(cube_v, cube_i);
    const float nd = 1.5168f;
    MaterialInstance* glass = glassMat->createInstance();
    glass->setParameter("ior", nd);
    glass->setParameter("abbe", 64.17f / disp);
    {
        // the faces of the unit mesh, for the block's own material
        beam::Solid unit = beam::chamfered_box({0, 0, 0}, 0.0, 0.5, chamfer);
        std::vector<float4> planes;
        for (size_t f = 0; f < unit.planes.size(); ++f)
            if (unit.faces[f].n >= 3) planes.push_back(float4{(float)unit.planes[f].n.x, (float)unit.planes[f].n.y, (float)unit.planes[f].n.z, (float)unit.planes[f].o});
        planes.resize(40, float4{0, 1, 0, 1e6f});
        glass->setParameter("planes", planes.data(), 40);
        glass->setParameter("planeCount", (int32_t)std::min<size_t>(40, unit.planes.size()));
        glass->setParameter("kind", (int32_t)0);
        glass->setParameter("dims", float4{0, 0, 0, 0});
    }
    {
        // no other glass in this picture: an empty scene table
        static float none[4] = {0, 0, 0, 0};
        Texture* sceneTex = Texture::Builder().width(1).height(1).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
        sceneTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(none, sizeof(none), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
        glass->setParameter("scene", sceneTex, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST));
        glass->setParameter("solidCount", (int32_t)0);
        glass->setParameter("self", (int32_t)0);
        glass->setParameter("spread", env_f("SPREAD", 2.f));  // colours part only at the last face; this makes up for the others
    }
    glass->setParameter("floorY", 0.f);
    glass->setParameter("absorption", float3{0.9f, 0.35f, 0.5f} * env_f("ABSORB", 0.15f));  // per world unit: a faint green-blue in thick glass
    glass->setParameter("backdrop", backdrop.color, lin);
    // The block is drawn by its own material and left out of the backdrop picture (layer 1); a twin that draws
    // nothing casts its shadow in every view.
    utils::Entity cube = em.create();
    RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {0.5f, 0.5f, 0.5f}}).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, cubeVB, cubeIB)
        .material(0, glass).culling(false).castShadows(false).receiveShadows(false).layerMask(0xFF, 0x02).build(*engine, cube);
    tm.create(cube);
    if (!std::getenv("NOGLASS")) scene->addEntity(cube);
    // The block's shadow is its outline drawn into `shade` and softened with the light. (FILSHADOW=1: a twin that
    // draws nothing casts Filament's own shadow instead.)
    utils::Entity caster = renderable(cubeVB, cubeIB, shadowMat->createInstance(), true, false, 0.5f);
    tm.create(caster);
    if (filamentShadow) scene->addEntity(caster);

    utils::Entity sunE = em.create();
    LightManager::ShadowOptions shopt;
    shopt.mapSize = 2048;
    shopt.shadowNearHint = 0.5f;
    shopt.shadowFarHint = 8.0f;
    shopt.constantBias = 0.001f;
    shopt.normalBias = 0.5f;
    LightManager::Builder(LightManager::Type::SUN).color(sunColor).intensity(lux).direction(float3{0, -1, 0})
        .sunAngularRadius(sunRadiusDeg).sunHaloSize(1.2f).castShadows(true).shadowOptions(shopt).build(*engine, sunE);
    scene->addEntity(sunE);
    scene->setSkybox(Skybox::Builder().color({0.72f, 0.74f, 0.78f, 1}).build(*engine));
    {
        // Studio environment (out/studio): gives the block something to mirror and keeps the shadow from being black.
        auto ib = load_file("out/studio/_ibl.ktx");
        std::ifstream shf("out/studio/sh.txt");
        std::vector<float3> shv;
        std::string line;
        while (std::getline(shf, line)) {
            size_t l = line.find('('), r = line.find(')');
            if (l == std::string::npos || r == std::string::npos) continue;
            float x, y, z;
            if (std::sscanf(line.c_str() + l + 1, "%f,%f,%f", &x, &y, &z) == 3) shv.push_back({x, y, z});
        }
        if (ib.empty() || shv.size() < 9) return fail("studio environment (out/studio)");
        Texture* ibl = ktxreader::Ktx1Reader::createTexture(engine, new image::Ktx1Bundle(ib.data(), (uint32_t)ib.size()), false);
        const float iblLux = env_f("IBL", 900.f);
        scene->setIndirectLight(IndirectLight::Builder().reflections(ibl).irradiance(3, shv.data()).intensity(iblLux).build(*engine));
        TextureSampler cubeS(TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE);
        glass->setParameter("env", ibl, cubeS);
        glass->setParameter("envGain", iblLux * env_f("ENVGAIN", 0.f));
        glass->setParameter("sky", float3{0.95f, 0.97f, 1.0f} * env_f("SKY", 1800.f));
    }

    // ---- camera and views
    Camera* camera = engine->createCamera(em.create());
    camera->setExposure(env_f("APERTURE", 4.0f), 1.f / 125.f, 100.f);
    float cam[7] = {0.95f, 0.62f, 1.05f, 0.28f, 0.03f, -0.02f, 30.f};
    if (const char* c = std::getenv("CAM")) std::sscanf(c, "%f %f %f %f %f %f %f", &cam[0], &cam[1], &cam[2], &cam[3], &cam[4], &cam[5], &cam[6]);
    camera->lookAt(float3{cam[0], cam[1], cam[2]}, float3{cam[3], cam[4], cam[5]}, float3{0, 1, 0});
    camera->setProjection((double)cam[6], double(W) / double(H), 0.05, 40.0, Camera::Fov::VERTICAL);

    View* vLight = engine->createView();
    vLight->setScene(sLight);
    vLight->setCamera(camera);
    vLight->setRenderTarget(light.rt);
    vLight->setViewport({0, 0, (uint32_t)(W * SS), (uint32_t)(H * SS)});
    vLight->setDithering(View::Dithering::NONE);
    vLight->setPostProcessingEnabled(false);
    vLight->setShadowingEnabled(false);

    {
        const mat4 clipFromWorld = camera->getProjectionMatrix() * camera->getViewMatrix();
        softenMi->setParameter("worldFromClip", mat4f(inverse(clipFromWorld)));
        softenMi->setParameter("eye", float3{cam[0], cam[1], cam[2]});
        softenMi->setParameter("focal", (float)(H * SS / (2.0 * std::tan(cam[6] * 0.5 * 3.14159265 / 180.0))));
    }
    View* vShade = engine->createView();
    vShade->setScene(sShade);
    vShade->setCamera(camera);
    vShade->setRenderTarget(shade.rt);
    vShade->setViewport({0, 0, (uint32_t)(W * SS), (uint32_t)(H * SS)});
    vShade->setDithering(View::Dithering::NONE);
    vShade->setPostProcessingEnabled(false);
    vShade->setShadowingEnabled(false);
    View* vSoft = engine->createView();
    vSoft->setScene(sSoft);
    vSoft->setCamera(camera);
    vSoft->setRenderTarget(soft.rt);
    vSoft->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    vSoft->setDithering(View::Dithering::NONE);
    vSoft->setPostProcessingEnabled(false);
    vSoft->setShadowingEnabled(false);

    const View::ShadowType shadowType = std::getenv("HARDSHADOW") ? View::ShadowType::PCF : View::ShadowType::PCSS;
    SoftShadowOptions softShadow;
    softShadow.penumbraScale = env_f("PENUMBRA", 1.f);
    View* vBackdrop = engine->createView();
    vBackdrop->setScene(scene);
    vBackdrop->setCamera(camera);
    vBackdrop->setRenderTarget(backdrop.rt);
    vBackdrop->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    vBackdrop->setVisibleLayers(0xFF, 0x01);
    vBackdrop->setDithering(View::Dithering::NONE);
    vBackdrop->setPostProcessingEnabled(false);
    vBackdrop->setShadowType(shadowType);
    vBackdrop->setSoftShadowOptions(softShadow);

    View* vBeauty = engine->createView();
    vBeauty->setVisibleLayers(0xFF, 0x03);
    vBeauty->setScene(scene);
    vBeauty->setCamera(camera);
    vBeauty->setRenderTarget(beauty.rt);
    vBeauty->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    vBeauty->setDithering(View::Dithering::NONE);
    static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
    vBeauty->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.04f).saturation(1.08f).build(*engine));
    vBeauty->setPostProcessingEnabled(true);
    vBeauty->setShadowType(shadowType);
    vBeauty->setSoftShadowOptions(softShadow);
    BloomOptions bloom{};
    bloom.enabled = !std::getenv("NOBLOOM");
    bloom.strength = env_f("BLOOM", 0.10f);
    bloom.levels = 6;
    bloom.resolution = 1080;
    bloom.threshold = true;
    bloom.highlight = 10.f;
    bloom.blendMode = BloomOptions::BlendMode::ADD;
    vBeauty->setBloomOptions(bloom);
    vBeauty->setAntiAliasing(std::getenv("NOFXAA") ? View::AntiAliasing::NONE : View::AntiAliasing::FXAA);

    Renderer::ClearOptions clearOpt;
    clearOpt.clear = true;
    clearOpt.discard = true;
    clearOpt.clearColor = {0, 0, 0, 0};
    renderer->setClearOptions(clearOpt);

    // ---- pose and sun: the only inputs
    float3 sunDir = normalize(float3{0.92f, -0.42f, 0.22f});
    Pose pose{{0.f, 0.16f, 0.f}, 0.62f, 0.16f};
    double traceMs = 0;
    size_t lightTris = 0;
    auto update = [&]() {
        const float e = 2.f * pose.half;
        const float c = std::cos(pose.yaw), s = std::sin(pose.yaw);
        float3 ax{c, 0, -s}, ay{0, 1, 0}, az{s, 0, c};
        const mat4f xf(float4{ax * e, 0}, float4{ay * e, 0}, float4{az * e, 0}, float4{pose.center, 1});
        tm.setTransform(tm.getInstance(cube), xf);
        tm.setTransform(tm.getInstance(caster), xf);
        auto& lm = engine->getLightManager();
        lm.setDirection(lm.getInstance(sunE), sunDir);

        glass->setParameter("sunTo", -sunDir);
        glass->setParameter("sunGlint", sunColor * env_f("GLINT", 4.0e5f));
        glass->setParameter("sunCos", std::cos(env_f("GLINTDEG", 2.0f) * 3.14159265f / 180.f));
        auto t0 = std::chrono::steady_clock::now();
        beam::Solid solid = beam::chamfered_box({pose.center.x, pose.center.y, pose.center.z}, pose.yaw, pose.half, chamfer * e);
        beam::Params par;
        par.sun = {sunDir.x, sunDir.y, sunDir.z};
        par.bands = bands;
        par.glass.nd = nd;
        par.glass.exaggerate = disp;
        par.cutoff = env_f("CUTOFF", 0.02f);
        par.maxBounces = (int)env_f("BOUNCES", 4.f);
        static std::vector<float> tri;
        if (std::getenv("REF")) {
            // reference: every band traced in full (slow, exact per band)
            par.maxBounces = 6;
            par.cutoff = 0.002;
            beam::to_triangles(beam::trace(solid, par), tri);
        } else {
            // one pixel on the floor near the block, in world units
            const float dist = length(float3{cam[0], cam[1], cam[2]} - pose.center);
            const double pixel = 2.0 * std::tan(cam[6] * 0.5 * 3.14159265 / 180.0) * dist / H;
            beam::trace_fast(solid, par, pixel * env_f("DETAIL", 1.f), tri);
        }
        traceMs = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
        const size_t TS = beam::TRI;
        size_t nLight = std::min<size_t>(tri.size() / TS, MAX_LIGHT_VERTS - 3000);
        nLight -= nLight % 3;
        tri.resize(nLight * TS);
        if (!filamentShadow) beam::silhouette_triangles(solid, par.sun, 0.0, tri);
        size_t nv = tri.size() / TS;
        lightTris = nLight / 3;
        softenMi->setParameter("sunDown", -sunDir.y);
        auto* buf = new std::vector<float>(std::max<size_t>(nv, 3) * 7, 0.f);
        const bool off = std::getenv("NOLIGHT") != nullptr;
        for (size_t i = 0; i < nv; ++i) {
            float* o = buf->data() + i * 7;
            o[0] = tri[i * TS];
            o[1] = 0.f;
            o[2] = tri[i * TS + 1];
            o[3] = (off && i < nLight) ? 0.f : tri[i * TS + 2];
            o[4] = (off && i < nLight) ? 0.f : tri[i * TS + 3];
            o[5] = (off && i < nLight) ? 0.f : tri[i * TS + 4];
            o[6] = tri[i * TS + 5];  // distance flown, for the softening pass
        }
        lightVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(buf->data(), buf->size() * sizeof(float),
                                                                         [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, buf));
        rm.setGeometryAt(rm.getInstance(lightE), 0, RenderableManager::PrimitiveType::TRIANGLES, lightVB, lightIB, 0, std::max<size_t>(nLight, 3));
        // nothing to draw when there is no outline: point at one corner three times
        rm.setGeometryAt(rm.getInstance(shadeE), 0, RenderableManager::PrimitiveType::TRIANGLES, lightVB, lightIB, nv > nLight ? nLight : 0, nv > nLight ? nv - nLight : 0);
    };
    if (const char* sv = std::getenv("SUN")) {
        float x, y, z;
        if (std::sscanf(sv, "%f %f %f", &x, &y, &z) == 3) sunDir = normalize(float3{x, y, z});
    }
    pose.yaw = env_f("YAW", pose.yaw);
    pose.half = env_f("HALF", pose.half);
    pose.center.y = pose.half + 0.0005f;

    auto draw_light = [&]() {
        renderer->render(vLight);
        renderer->render(vShade);
        light.color->generateMipmaps(*engine);
        shade.color->generateMipmaps(*engine);
        renderer->render(vSoft);
    };
    auto open = [&]() {
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) return true;
        return false;
    };
    std::vector<uint8_t> frame((size_t)W * H * 4);
    auto grab = [&](const char* path) -> bool {
        if (!open()) return false;
        draw_light();
        renderer->render(vBackdrop);
        renderer->render(vBeauty);
        renderer->readPixels(beauty.rt, 0, 0, W, H,
                             backend::PixelBufferDescriptor(frame.data(), frame.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
        renderer->endFrame();
        engine->flushAndWait();
        return write_png(path, W, H, frame.data());
    };
    std::system("mkdir -p out");
    update();
    for (int i = 0; i < 3; ++i) {
        if (!open()) return fail("warm");
        draw_light();
        renderer->render(vBackdrop);
        renderer->render(vBeauty);
        renderer->endFrame();
        engine->flushAndWait();
    }
    const std::string out = "out/glass-beam-" + name + ".png";
    if (!grab(out.c_str())) return fail("grab");

    // ---- paced like an app: every tick the block turns and slides and the sun swings, so the light is traced
    // again and uploaded again each frame. beginFrame() returning false is Filament saying it is behind.
    if (std::getenv("PACE")) {
        const int n = (int)env_f("PACE_FRAMES", 600.f);
        const double hz = env_f("PACE_HZ", 60.f);
        const double period = 1000.0 / hz;
        const Pose pose0 = pose;
        const float3 sun0 = sunDir;
        std::vector<double> cpu, g, lat, upd;
        int skipped = 0, drawn = 0;
        size_t maxTris = 0;
        auto pct = [](std::vector<double> v, double p) {
            if (v.empty()) return -1.0;
            std::sort(v.begin(), v.end());
            return v[std::min(v.size() - 1, (size_t)(v.size() * p))];
        };
        auto start = std::chrono::steady_clock::now();
        for (int i = 0; i < n; ++i) {
            auto due = start + std::chrono::duration<double, std::milli>(period * i);
            while (std::chrono::steady_clock::now() < due) {}
            const float t = (float)i / 120.f * 6.2831853f;
            pose = pose0;
            pose.center.x += 0.10f * std::cos(t);
            pose.center.z += 0.08f * std::sin(t);
            pose.yaw = pose0.yaw + t;
            sunDir = normalize(float3{sun0.x * std::cos(0.3f * std::sin(t)), sun0.y, sun0.z + 0.4f * std::sin(t)});
            auto u0 = std::chrono::steady_clock::now();
            update();
            upd.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - u0).count());
            cpu.push_back(traceMs);
            maxTris = std::max(maxTris, lightTris);
            if (!renderer->beginFrame(swap)) { ++skipped; continue; }
            draw_light();
            renderer->render(vBackdrop);
            renderer->render(vBeauty);
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
        const double total = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - start).count();
        size_t late = 0;
        for (double v : lat) if (v > period) ++late;
        std::printf("PACE %.0f Hz for %d frames (%.1f s, target %.1f s), block and sun moving every frame, %dx%d, light buffer x%d\n", hz, n, total / 1000.0,
                    n * period / 1000.0, W, H, SS);
        auto line = [&](const char* nm, std::vector<double>& v) {
            std::printf("     %-40s n=%3zu  min %6.2f  median %6.2f  95%% %6.2f  99%% %6.2f  max %6.2f ms\n", nm, v.size(), pct(v, 0.0), pct(v, 0.5), pct(v, 0.95), pct(v, 0.99), pct(v, 1.0));
        };
        line("CPU: trace the light", cpu);
        line("CPU: trace + fill + hand to Filament", upd);
        line("GPU time per frame", g);
        line("beginFrame -> GPU finished", lat);
        std::printf("     most light triangles in a frame: %zu\n", maxTris);
        std::printf("     finished after their %.2f ms slot: %zu of %zu measured (%.1f%%)\n", period, late, lat.size(), lat.empty() ? 0.0 : 100.0 * late / lat.size());
        std::printf("     DROPPED (Filament asked to skip): %d of %d ticks (%.1f%%)  -> delivered %.1f fps of %.0f\n", skipped, n, 100.0 * skipped / n, drawn / (total / 1000.0), hz);
    }
    std::printf("wrote %s  (chamfer %.3f, dispersion x%.1f, %d bands, trace %.2f ms, %zu light triangles, light buffer %dx%d)\n", out.c_str(), chamfer, disp,
                bands, traceMs, lightTris, W * SS, H * SS);
    return 0;
}
