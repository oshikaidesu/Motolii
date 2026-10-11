// HPWater's single-channel caustic look, on the existing Filament cube.
// The GPU splat in prism_gpu.mm is unchanged. This file only filters that
// illuminance and splits it into RGB at the floor sample.
// CPU reference: out/prism.png. This file does not write it.
//
// Reimplemented from public MPL-2.0 sources (not vendored):
// https://github.com/AshenOneArt/HPWater
//   Core_Source_Preview/HanPiWater/Caustic/HPWaterCausticSampleFunction.hlsl
//   Core_Source_Preview/HanPiWater/Caustic/HPWaterCausticCompute.compute
//   Core_Source_Preview/HanPiWater/Caustic/HPWaterCausticPass.cs
//   Core_Source_Preview/HanPiWater/HPWaterCommon.hlsl
//   Core_Source_Preview/HanPiWater/HPWater.shader
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

static const int W = 1920, H = 1080, CAP = 96, MAP = 512, BANDS = 6;
static const float HW = 1.55f, HH = 1.55f;
static const float ETA_BASE = 0.75f;

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
    float s = std::sqrt(k);
    return I * eta - N * (eta * d + s);
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
static double ms_since(std::chrono::steady_clock::time_point t0) {
    return std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
}
static double median(std::vector<double> v) {
    if (v.empty()) return -1;
    std::sort(v.begin(), v.end());
    return v[v.size() / 2];
}
static void push_vert(std::vector<float>& v, V3 p, V3 T, V3 N) {
    V3 n = norm(N - T * dot(N, T));
    V3 B = norm(cross(n, T));
    quatf q = mat3f::packTangentFrame(mat3f{float3{T.x, T.y, T.z}, float3{B.x, B.y, B.z}, float3{n.x, n.y, n.z}});
    float f[7] = {p.x, p.y, p.z, q.x, q.y, q.z, q.w};
    v.insert(v.end(), f, f + 7);
}

int main(int argc, char** argv) {
    const bool motion = argc > 1 && std::string(argv[1]) == "motion";
    const char* motionMode = argc > 2 ? argv[2] : "d";

    const float half = 0.16f;
    const float nd = 1.55f;
    const float dispersion = 36.f;
    const float lux = 22000.f;
    const float extent = 0.42f;
    const V3 sun0 = norm({0.92f, -0.42f, 0.22f});
    V3 axis[3] = {{1, 0, 0}, {0, 1, 0}, {0, 0, 1}};
    for (int i = 0; i < 3; ++i) rotateY(axis[i], 0.62f);
    V3 center{0.f, half + 0.001f, 0.f};

    auto entry_normal = [&](V3 sun) {
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
        return best;
    };
    auto uv_offset = [&](V3 sun) {
        V3 N = entry_normal(sun);
        if (dot(N, sun) > 0.f) N = N * -1.f;
        V3 dR = refract_glsl(sun, N, ETA_BASE + 0.001f);
        V3 dG = refract_glsl(sun, N, ETA_BASE);
        V3 delta = dG - dR;
        return float2{delta.x / HW * 100.f, delta.z / HH * 100.f};
    };

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
    view->setPostProcessingEnabled(false);
    view->setScreenSpaceRefractionEnabled(false);
    scene->setSkybox(nullptr);

    auto make_color = [&](int w, int h, Texture::InternalFormat fmt, int levels, bool blit) {
        auto usage = Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC;
        if (blit) usage |= Texture::Usage::BLIT_DST | Texture::Usage::GEN_MIPMAPPABLE;
        Texture* color = Texture::Builder()
                             .width((uint32_t)w).height((uint32_t)h).levels((uint8_t)levels)
                             .usage(usage).format(fmt).build(*engine);
        Texture* depth = Texture::Builder()
                             .width((uint32_t)w).height((uint32_t)h).levels(1)
                             .usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH32F).build(*engine);
        RenderTarget* rt = RenderTarget::Builder()
                               .texture(RenderTarget::AttachmentPoint::COLOR, color)
                               .texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
        return std::pair<Texture*, RenderTarget*>{color, rt};
    };
    auto [frontTex, frontRT] = make_color(CAP, CAP, Texture::InternalFormat::RGBA16F, 1, false);
    auto [backTex, backRT] = make_color(CAP, CAP, Texture::InternalFormat::RGBA16F, 1, false);
    auto [mapTex, mapRT] = make_color(MAP, MAP, Texture::InternalFormat::RGBA16F, 1, false);
    auto [tempTex, tempRT] = make_color(MAP, MAP, Texture::InternalFormat::RGBA16F, 8, true);
    auto [filtTex, filtRT] = make_color(MAP, MAP, Texture::InternalFormat::RGBA16F, 1, false);
    auto [beauty, beautyRT] = make_color(W, H, Texture::InternalFormat::RGBA8, 1, false);

    auto capkg = load_file("caustic_capture.filamat");
    auto spkg = load_file("caustic_splat.filamat");
    auto gpkg = load_file("glass_shadow.filamat");
    auto fpkg = load_file("floor_hp.filamat");
    auto apkg = load_file("caustic_atrous.filamat");
    if (capkg.empty() || spkg.empty() || gpkg.empty() || fpkg.empty() || apkg.empty()) return fail("filamat");
    Material* capMat = Material::Builder().package(capkg.data(), capkg.size()).build(*engine);
    Material* splatMat = Material::Builder().package(spkg.data(), spkg.size()).build(*engine);
    Material* glassMat = Material::Builder().package(gpkg.data(), gpkg.size()).build(*engine);
    Material* floorMat = Material::Builder().package(fpkg.data(), fpkg.size()).build(*engine);
    Material* atrousMat = Material::Builder().package(apkg.data(), apkg.size()).build(*engine);
    if (!capMat || !splatMat || !glassMat || !floorMat || !atrousMat) return fail("material");

    V3 sun = sun0;
    V3 up = std::fabs(sun.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0};
    V3 tangent = norm(cross(sun, up));
    V3 bitangent = norm(cross(tangent, sun));
    V3 origin = center - sun * 2.f;

    MaterialInstance* frontMi = capMat->createInstance();
    MaterialInstance* backMi = capMat->createInstance();
    frontMi->setCullingMode(MaterialInstance::CullingMode::BACK);
    backMi->setCullingMode(MaterialInstance::CullingMode::FRONT);
    TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    nearest.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    nearest.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    MaterialInstance* splatMi = splatMat->createInstance();
    auto bind_capture = [&]() {
        up = std::fabs(sun.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0};
        tangent = norm(cross(sun, up));
        bitangent = norm(cross(tangent, sun));
        origin = center - sun * 2.f;
        frontMi->setParameter("origin", float3{origin.x, origin.y, origin.z});
        frontMi->setParameter("dir", float3{sun.x, sun.y, sun.z});
        backMi->setParameter("origin", float3{origin.x, origin.y, origin.z});
        backMi->setParameter("dir", float3{sun.x, sun.y, sun.z});
        splatMi->setParameter("origin", float3{origin.x, origin.y, origin.z});
        splatMi->setParameter("dir", float3{sun.x, sun.y, sun.z});
        splatMi->setParameter("tangent", float3{tangent.x, tangent.y, tangent.z});
        splatMi->setParameter("bitangent", float3{bitangent.x, bitangent.y, bitangent.z});
    };
    splatMi->setParameter("frontFace", frontTex, nearest);
    splatMi->setParameter("backFace", backTex, nearest);
    splatMi->setParameter("captureSize", (float)CAP);
    splatMi->setParameter("extent", extent);
    splatMi->setParameter("ior", nd);
    splatMi->setParameter("dispersion", dispersion);
    splatMi->setParameter("splat", 0.008f);
    splatMi->setParameter("lux", lux);
    bind_capture();

    std::vector<float> cube_v;
    std::vector<uint32_t> cube_i;
    for (int face = 0; face < 6; ++face) {
        int ax = face / 2;
        float sgn = (face & 1) ? 1.f : -1.f;
        V3 N = axis[ax] * sgn;
        V3 T = axis[(ax + 1) % 3];
        V3 c = center + N * half;
        uint32_t base = (uint32_t)(cube_v.size() / 7);
        push_vert(cube_v, c - T * half - norm(cross(N, T)) * half, T, N);
        push_vert(cube_v, c + T * half - norm(cross(N, T)) * half, T, N);
        push_vert(cube_v, c + T * half + norm(cross(N, T)) * half, T, N);
        push_vert(cube_v, c - T * half + norm(cross(N, T)) * half, T, N);
        cube_i.insert(cube_i.end(), {base, base + 1, base + 2, base, base + 2, base + 3});
    }
    VertexBuffer* cubeVB = VertexBuffer::Builder().vertexCount((uint32_t)(cube_v.size() / 7)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                               .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                               .build(*engine);
    cubeVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(cube_v.data(), cube_v.size() * sizeof(float)));
    IndexBuffer* cubeIB = IndexBuffer::Builder().indexCount((uint32_t)cube_i.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    cubeIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(cube_i.data(), cube_i.size() * sizeof(uint32_t)));
    utils::Entity cube = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, cubeVB, cubeIB)
        .material(0, frontMi).culling(false).castShadows(true).receiveShadows(false)
        .build(*engine, cube);

    const int splatVerts = CAP * CAP * BANDS * 6;
    std::vector<float> sv((size_t)splatVerts * 5, 0.f);
    std::vector<uint32_t> si((size_t)splatVerts);
    for (int id = 0; id < CAP * CAP; ++id)
        for (int band = 0; band < BANDS; ++band)
            for (int c = 0; c < 6; ++c) {
                int v = ((id * BANDS + band) * 6 + c);
                sv[(size_t)v * 5 + 3] = (float)band;
                si[(size_t)v] = (uint32_t)v;
            }
    VertexBuffer* splatVB = VertexBuffer::Builder().vertexCount((uint32_t)splatVerts).bufferCount(1)
                                .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                                .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20)
                                .build(*engine);
    splatVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(sv.data(), sv.size() * sizeof(float)));
    IndexBuffer* splatIB = IndexBuffer::Builder().indexCount((uint32_t)si.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    splatIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(si.data(), si.size() * sizeof(uint32_t)));
    utils::Entity splat = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-8, -8, -8}, {8, 8, 8}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, splatVB, splatIB)
        .material(0, splatMi).culling(false).castShadows(false).receiveShadows(false)
        .build(*engine, splat);

    TextureSampler lin(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
    lin.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    lin.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    TextureSampler mipped(TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR, TextureSampler::MagFilter::LINEAR);
    mipped.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    mipped.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);

    float2 off = uv_offset(sun);
    std::printf("uv offset %.5f %.5f  texels %.1f %.1f\n", off.x, off.y, off.x * MAP, off.y * MAP);
    MaterialInstance* floorMi = floorMat->createInstance();
    floorMi->setParameter("color", float4{0.86f, 0.85f, 0.84f, 1});
    floorMi->setParameter("roughness", 0.42f);
    floorMi->setParameter("disperse", 0.f);
    floorMi->setParameter("uvOffset", off);
    floorMi->setParameter("caustic", mapTex, lin);

    MaterialInstance* atrous1 = atrousMat->createInstance();
    atrous1->setParameter("src", mapTex, lin);
    atrous1->setParameter("texel", float2{1.f / MAP, 1.f / MAP});
    atrous1->setParameter("stride", 1.f);
    atrous1->setParameter("lumWeight", 0.5f);
    atrous1->setParameter("mip", 0.f);
    atrous1->setParameter("second", 0.f);
    MaterialInstance* atrous2 = atrousMat->createInstance();
    atrous2->setParameter("src", tempTex, mipped);
    atrous2->setParameter("texel", float2{1.f / MAP, 1.f / MAP});
    atrous2->setParameter("stride", 2.f);
    atrous2->setParameter("lumWeight", 0.5f);
    atrous2->setParameter("mip", 1.f);
    atrous2->setParameter("second", 1.f);

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
    VertexBuffer* floorVB = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                                .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 36)
                                .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 36)
                                .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, 36)
                                .build(*engine);
    floorVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(floor_v.data(), floor_v.size() * sizeof(float)));
    IndexBuffer* floorIB = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    floorIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(floor_i.data(), floor_i.size() * sizeof(uint32_t)));
    utils::Entity floor = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-4, -4, -4}, {4, 4, 4}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, floorVB, floorIB)
        .material(0, floorMi).culling(false).castShadows(false).receiveShadows(true)
        .build(*engine, floor);
    utils::Entity filter = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-4, -4, -4}, {4, 4, 4}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, floorVB, floorIB)
        .material(0, atrous1).culling(false).castShadows(false).receiveShadows(false)
        .build(*engine, filter);

    MaterialInstance* glass = glassMat->createInstance();
    glass->setParameter("ior", nd);
    glass->setParameter("transmission", 1.f);
    glass->setParameter("thickness", half * 2.f);
    glass->setParameter("dispersion", dispersion);
    glass->setParameter("reflectance", 0.5f);
    glass->setParameter("alpha", 1.f);
    glass->setParameter("absorption", float3{0.02f, 0.015f, 0.02f});

    utils::Entity sun_e = utils::EntityManager::get().create();
    LightManager::ShadowOptions shopt;
    shopt.mapSize = 2048;
    shopt.vsm.blurWidth = 8.f;
    shopt.vsm.elvsm = true;
    LightManager::Builder(LightManager::Type::SUN)
        .color(float3{1, 0.98f, 0.96f}).intensity(lux).direction(float3{sun.x, sun.y, sun.z})
        .sunAngularRadius(0.35f).sunHaloSize(1.2f).castShadows(true).shadowOptions(shopt)
        .build(*engine, sun_e);

    auto& rm = engine->getRenderableManager();
    auto open = [&]() {
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) return true;
        return false;
    };
    auto clear_black = [&]() {
        Renderer::ClearOptions c;
        c.clear = true;
        c.discard = true;
        c.clearColor = {0, 0, 0, 0};
        renderer->setClearOptions(c);
    };

    auto draw_caustic = [&]() -> bool {
        camera->setExposure(1.f);
        view->setPostProcessingEnabled(false);
        view->setScreenSpaceRefractionEnabled(false);
        scene->setSkybox(nullptr);
        rm.setMaterialInstanceAt(rm.getInstance(cube), 0, frontMi);
        camera->lookAt(float3{origin.x, origin.y, origin.z}, float3{center.x, center.y, center.z}, float3{bitangent.x, bitangent.y, bitangent.z});
        camera->setProjection(Camera::Projection::ORTHO, -extent, extent, -extent, extent, 0.05, 6.0);
        view->setViewport({0, 0, (uint32_t)CAP, (uint32_t)CAP});
        scene->remove(floor);
        scene->remove(filter);
        scene->remove(sun_e);
        scene->remove(splat);
        scene->addEntity(cube);
        if (!open()) return false;
        clear_black();
        view->setRenderTarget(frontRT);
        renderer->render(view);
        rm.setMaterialInstanceAt(rm.getInstance(cube), 0, backMi);
        clear_black();
        view->setRenderTarget(backRT);
        renderer->render(view);
        renderer->endFrame();
        engine->flushAndWait();
        scene->remove(cube);
        scene->addEntity(splat);
        if (!open()) return false;
        camera->lookAt(float3{0, 4, 0}, float3{0, 0, 0}, float3{0, 0, -1});
        camera->setProjection(Camera::Projection::ORTHO, -HW, HW, -HH, HH, 0.01, 10.0);
        view->setViewport({0, 0, (uint32_t)MAP, (uint32_t)MAP});
        clear_black();
        view->setRenderTarget(mapRT);
        renderer->render(view);
        renderer->endFrame();
        engine->flushAndWait();
        return true;
    };
    auto draw_atrous = [&]() -> bool {
        camera->setExposure(1.f);
        view->setPostProcessingEnabled(false);
        view->setScreenSpaceRefractionEnabled(false);
        scene->setSkybox(nullptr);
        camera->lookAt(float3{0, 4, 0}, float3{0, 0, 0}, float3{0, 0, -1});
        camera->setProjection(Camera::Projection::ORTHO, -HW, HW, -HH, HH, 0.01, 10.0);
        view->setViewport({0, 0, (uint32_t)MAP, (uint32_t)MAP});
        scene->remove(cube);
        scene->remove(splat);
        scene->remove(floor);
        scene->remove(sun_e);
        scene->addEntity(filter);
        rm.setMaterialInstanceAt(rm.getInstance(filter), 0, atrous1);
        if (!open()) return false;
        clear_black();
        view->setRenderTarget(tempRT);
        renderer->render(view);
        renderer->endFrame();
        tempTex->generateMipmaps(*engine);
        engine->flushAndWait();
        rm.setMaterialInstanceAt(rm.getInstance(filter), 0, atrous2);
        if (!open()) return false;
        clear_black();
        view->setRenderTarget(filtRT);
        renderer->render(view);
        renderer->endFrame();
        engine->flushAndWait();
        return true;
    };
    Skybox* sky = nullptr;
    bool beautyReady = false;
    auto arm_beauty = [&]() {
        rm.setMaterialInstanceAt(rm.getInstance(cube), 0, glass);
        if (!beautyReady) {
            static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
            view->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.04f).saturation(1.08f).build(*engine));
            sky = Skybox::Builder().color({0.72f, 0.74f, 0.78f, 1}).build(*engine);
            beautyReady = true;
        }
        view->setPostProcessingEnabled(true);
        view->setScreenSpaceRefractionEnabled(true);
        view->setShadowType(View::ShadowType::VSM);
        view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
        scene->setSkybox(sky);
        scene->remove(splat);
        scene->remove(filter);
        scene->addEntity(floor);
        scene->addEntity(cube);
        scene->addEntity(sun_e);
        camera->setExposure(2.8f, 1.f / 125.f, 100.f);
        camera->lookAt(float3{0.55f, 0.78f, 0.70f}, float3{0, 0.04f, 0}, float3{0, 1, 0});
        camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);
    };
    auto draw_beauty = [&]() -> bool {
        if (!open()) return false;
        view->setRenderTarget(beautyRT);
        renderer->render(view);
        renderer->endFrame();
        engine->flushAndWait();
        return true;
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
    auto time_n = [&](auto&& fn, int n) {
        std::vector<double> v;
        for (int i = 0; i < n; ++i) {
            auto t0 = std::chrono::steady_clock::now();
            if (!fn()) return -1.0;
            v.push_back(ms_since(t0));
        }
        return median(v);
    };

    if (!draw_caustic()) return fail("caustic-warmup");
    double causticMs = time_n(draw_caustic, 5);
    arm_beauty();
    floorMi->setParameter("disperse", 0.f);
    floorMi->setParameter("caustic", mapTex, lin);
    if (!draw_beauty()) return fail("beauty-warmup");
    double beautyA = time_n(draw_beauty, 5);
    if (!grab("out/hp-a.png")) return fail("hp-a");

    floorMi->setParameter("disperse", 1.f);
    floorMi->setParameter("uvOffset", off);
    if (!draw_beauty()) return fail("beauty-b-warmup");
    double beautyB = time_n(draw_beauty, 5);
    if (!grab("out/hp-b.png")) return fail("hp-b");

    if (!draw_atrous()) return fail("atrous-warmup");
    double atrousMs = time_n(draw_atrous, 5);
    arm_beauty();
    floorMi->setParameter("disperse", 0.f);
    floorMi->setParameter("caustic", filtTex, lin);
    if (!draw_beauty()) return fail("beauty-c-warmup");
    double beautyC = time_n(draw_beauty, 5);
    if (!grab("out/hp-c.png")) return fail("hp-c");

    floorMi->setParameter("disperse", 1.f);
    if (!draw_beauty()) return fail("beauty-d-warmup");
    double beautyD = time_n(draw_beauty, 5);
    if (!grab("out/hp-d.png")) return fail("hp-d");

    std::printf("stage wall ms, flushAndWait, readPixels excluded, median of 5\n");
    std::printf("caustic %.2f\n", causticMs);
    std::printf("rgb_dispersion_in_beauty A %.2f  B %.2f\n", beautyA, beautyB);
    std::printf("atrous %.2f\n", atrousMs);
    std::printf("beauty_after_atrous C %.2f  D %.2f\n", beautyC, beautyD);
    std::printf("total A %.2f  B %.2f  C %.2f  D %.2f\n",
                causticMs + beautyA, causticMs + beautyB, causticMs + atrousMs + beautyC, causticMs + atrousMs + beautyD);
    std::printf("60fps budget 16.67 ms. These are CPU waits for GPU completion, not a Metal timestamp.\n");

    if (motion) {
        const int frames = 24;
        bool useFilt = motionMode[0] == 'c' || motionMode[0] == 'd';
        bool useDisp = motionMode[0] == 'b' || motionMode[0] == 'd';
        double motionTotal = 0;
        for (int f = 0; f < frames; ++f) {
            float yaw = -0.18f + 0.36f * (float)f / (float)(frames - 1);
            V3 s = sun0;
            rotateY(s, yaw);
            sun = norm(s);
            bind_capture();
            auto& lm = engine->getLightManager();
            lm.setDirection(lm.getInstance(sun_e), float3{sun.x, sun.y, sun.z});
            floorMi->setParameter("uvOffset", uv_offset(sun));
            floorMi->setParameter("disperse", useDisp ? 1.f : 0.f);
            auto t0 = std::chrono::steady_clock::now();
            if (!draw_caustic()) return fail("motion-caustic");
            if (useFilt && !draw_atrous()) return fail("motion-atrous");
            arm_beauty();
            floorMi->setParameter("caustic", useFilt ? filtTex : mapTex, lin);
            floorMi->setParameter("disperse", useDisp ? 1.f : 0.f);
            if (!draw_beauty()) return fail("motion-beauty");
            if (f == 3) motionTotal = ms_since(t0);
            char path[64];
            std::snprintf(path, sizeof(path), "out/hp-e-%02d.png", f);
            if (!grab(path)) return fail("motion-png");
        }
        std::printf("motion mode %s  one frame wall before readPixels %.2f ms\n", motionMode, motionTotal);
    }
    std::printf("PASS prism-hp\n");
    return 0;
}
