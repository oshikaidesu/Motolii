// luma.gl's spectral caustic, on Filament draw passes.
// Light-view front and back surfaces go to RGBA16F.
// A vertex shader splats six wavelength bands onto the floor.
// The existing CPU cube trace in prism.mm is the picture to compare, not this file's input.
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
#include <vector>

using namespace filament;
using namespace filament::math;

static const int W = 1920, H = 1080, CAP = 96, MAP = 512, BANDS = 6;
static const float HW = 1.55f, HH = 1.55f;

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
static float half_to_float(uint16_t h) {
    uint32_t s = (uint32_t)(h & 0x8000) << 16;
    uint32_t e = (h >> 10) & 0x1f;
    uint32_t m = h & 0x3ff;
    uint32_t f;
    if (e == 0) f = s;
    else if (e == 31) f = s | 0x7f800000u | (m << 13);
    else f = s | ((e + 112) << 23) | (m << 13);
    float o;
    std::memcpy(&o, &f, 4);
    return o;
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

int main() {
    const float half = 0.16f;
    const float nd = 1.55f;
    const float dispersion = 36.f;
    const float lux = 22000.f;
    const float extent = 0.42f;
    V3 sun = norm({0.92f, -0.42f, 0.22f});
    V3 axis[3] = {{1, 0, 0}, {0, 1, 0}, {0, 0, 1}};
    for (int i = 0; i < 3; ++i) rotateY(axis[i], 0.62f);
    V3 center{0.f, half + 0.001f, 0.f};
    V3 up = std::fabs(sun.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0};
    V3 tangent = norm(cross(sun, up));
    V3 bitangent = norm(cross(tangent, sun));
    V3 origin = center - sun * 2.f;

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

    auto make_color = [&](int w, int h, Texture::InternalFormat fmt) {
        Texture* color = Texture::Builder()
                             .width((uint32_t)w).height((uint32_t)h).levels(1)
                             .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                             .format(fmt).build(*engine);
        Texture* depth = Texture::Builder()
                             .width((uint32_t)w).height((uint32_t)h).levels(1)
                             .usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH32F).build(*engine);
        RenderTarget* rt = RenderTarget::Builder()
                               .texture(RenderTarget::AttachmentPoint::COLOR, color)
                               .texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
        return std::pair<Texture*, RenderTarget*>{color, rt};
    };
    auto [frontTex, frontRT] = make_color(CAP, CAP, Texture::InternalFormat::RGBA16F);
    auto [backTex, backRT] = make_color(CAP, CAP, Texture::InternalFormat::RGBA16F);
    auto [mapTex, mapRT] = make_color(MAP, MAP, Texture::InternalFormat::RGBA16F);
    auto [beauty, beautyRT] = make_color(W, H, Texture::InternalFormat::RGBA8);

    auto capkg = load_file("caustic_capture.filamat");
    auto spkg = load_file("caustic_splat.filamat");
    auto gpkg = load_file("glass_shadow.filamat");
    auto fpkg = load_file("floor_recv.filamat");
    if (capkg.empty() || spkg.empty() || gpkg.empty() || fpkg.empty()) return fail("filamat");
    Material* capMat = Material::Builder().package(capkg.data(), capkg.size()).build(*engine);
    Material* splatMat = Material::Builder().package(spkg.data(), spkg.size()).build(*engine);
    Material* glassMat = Material::Builder().package(gpkg.data(), gpkg.size()).build(*engine);
    Material* floorMat = Material::Builder().package(fpkg.data(), fpkg.size()).build(*engine);
    if (!capMat || !splatMat || !glassMat || !floorMat) return fail("material");

    auto setup_cap = [&](MaterialInstance* mi) {
        mi->setParameter("origin", float3{origin.x, origin.y, origin.z});
        mi->setParameter("dir", float3{sun.x, sun.y, sun.z});
    };
    MaterialInstance* frontMi = capMat->createInstance();
    MaterialInstance* backMi = capMat->createInstance();
    setup_cap(frontMi);
    setup_cap(backMi);
    frontMi->setCullingMode(MaterialInstance::CullingMode::BACK);
    backMi->setCullingMode(MaterialInstance::CullingMode::FRONT);

    TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    nearest.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    nearest.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    MaterialInstance* splatMi = splatMat->createInstance();
    splatMi->setParameter("frontFace", frontTex, nearest);
    splatMi->setParameter("backFace", backTex, nearest);
    splatMi->setParameter("origin", float3{origin.x, origin.y, origin.z});
    splatMi->setParameter("dir", float3{sun.x, sun.y, sun.z});
    splatMi->setParameter("tangent", float3{tangent.x, tangent.y, tangent.z});
    splatMi->setParameter("bitangent", float3{bitangent.x, bitangent.y, bitangent.z});
    splatMi->setParameter("captureSize", (float)CAP);
    splatMi->setParameter("extent", extent);
    splatMi->setParameter("ior", nd);
    splatMi->setParameter("dispersion", dispersion);
    splatMi->setParameter("splat", 0.008f);
    splatMi->setParameter("lux", lux);
    MaterialInstance* splatMono = splatMat->createInstance();
    splatMono->setParameter("frontFace", frontTex, nearest);
    splatMono->setParameter("backFace", backTex, nearest);
    splatMono->setParameter("origin", float3{origin.x, origin.y, origin.z});
    splatMono->setParameter("dir", float3{sun.x, sun.y, sun.z});
    splatMono->setParameter("tangent", float3{tangent.x, tangent.y, tangent.z});
    splatMono->setParameter("bitangent", float3{bitangent.x, bitangent.y, bitangent.z});
    splatMono->setParameter("captureSize", (float)CAP);
    splatMono->setParameter("extent", extent);
    splatMono->setParameter("ior", nd);
    splatMono->setParameter("dispersion", 0.f);
    splatMono->setParameter("splat", 0.008f);
    splatMono->setParameter("lux", lux);

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

    const int quads = CAP * CAP * BANDS;
    const int splatVerts = quads * 6;
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
    camera->setExposure(1.f);
    camera->lookAt(float3{origin.x, origin.y, origin.z}, float3{center.x, center.y, center.z}, float3{bitangent.x, bitangent.y, bitangent.z});
    camera->setProjection(Camera::Projection::ORTHO, -extent, extent, -extent, extent, 0.05, 6.0);
    view->setViewport({0, 0, (uint32_t)CAP, (uint32_t)CAP});
    scene->addEntity(cube);

    auto t0 = std::chrono::steady_clock::now();
    if (!open()) return fail("frame");
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
    if (!open()) return fail("splat-frame");
    camera->lookAt(float3{0, 4, 0}, float3{0, 0, 0}, float3{0, 0, -1});
    camera->setProjection(Camera::Projection::ORTHO, -HW, HW, -HH, HH, 0.01, 10.0);
    view->setViewport({0, 0, (uint32_t)MAP, (uint32_t)MAP});
    clear_black();
    view->setRenderTarget(mapRT);
    renderer->render(view);
    std::vector<uint16_t> mapHalf((size_t)MAP * MAP * 4);
    std::vector<uint16_t> cap((size_t)CAP * CAP * 4);
    std::vector<uint16_t> capBack((size_t)CAP * CAP * 4);
    renderer->readPixels(mapRT, 0, 0, MAP, MAP,
                         backend::PixelBufferDescriptor(mapHalf.data(), mapHalf.size() * 2, backend::PixelDataFormat::RGBA,
                                                        backend::PixelDataType::HALF));
    renderer->readPixels(frontRT, 0, 0, CAP, CAP,
                         backend::PixelBufferDescriptor(cap.data(), cap.size() * 2, backend::PixelDataFormat::RGBA,
                                                        backend::PixelDataType::HALF));
    renderer->readPixels(backRT, 0, 0, CAP, CAP,
                         backend::PixelBufferDescriptor(capBack.data(), capBack.size() * 2, backend::PixelDataFormat::RGBA,
                                                        backend::PixelDataType::HALF));
    renderer->endFrame();
    engine->flushAndWait();
    auto t1 = std::chrono::steady_clock::now();

    float maxDepth = 0, sum = 0, maxv = 0, chroma = 0, lum = 0;
    int hits = 0, capHit = 0;
    float capMax[4] = {};
    for (int i = 0; i < CAP * CAP; ++i) {
        float ch[4];
        for (int k = 0; k < 4; ++k) {
            ch[k] = half_to_float(cap[(size_t)i * 4 + k]);
            capMax[k] = std::max(capMax[k], ch[k]);
        }
        if (ch[2] > 0.05f) capHit++;
        maxDepth = std::max(maxDepth, ch[2]);
    }
    int backHit = 0;
    float backMax = 0, backMin = 1e9f;
    float frontMin = 1e9f;
    for (int i = 0; i < CAP * CAP; ++i) {
        float b = half_to_float(capBack[(size_t)i * 4 + 2]);
        if (b > 0.05f) {
            backHit++;
            backMax = std::max(backMax, b);
            backMin = std::min(backMin, b);
        }
        float f = half_to_float(cap[(size_t)i * 4 + 2]);
        if (f > 0.05f) frontMin = std::min(frontMin, f);
    }
    std::printf("front pixels %d depth %.3f..%.3f  rgba max %.3f %.3f %.3f %.3f\n", capHit, frontMin, maxDepth, capMax[0], capMax[1], capMax[2], capMax[3]);
    std::printf("back pixels %d depth %.3f..%.3f\n", backHit, backHit ? backMin : 0.f, backMax);
    std::vector<uint8_t> pic((size_t)MAP * MAP * 4);
    double sr = 0, sg = 0, sb = 0;
    int blueish = 0;
    for (int i = 0; i < MAP * MAP; ++i) {
        float r = std::max(half_to_float(mapHalf[(size_t)i * 4 + 0]), 0.f);
        float g = std::max(half_to_float(mapHalf[(size_t)i * 4 + 1]), 0.f);
        float b = std::max(half_to_float(mapHalf[(size_t)i * 4 + 2]), 0.f);
        sr += r;
        sg += g;
        sb += b;
        if (b > r + 20.f && b > 30.f) blueish++;
        float m = std::max(r, std::max(g, b));
        sum += m;
        maxv = std::max(maxv, m);
        if (m > 30.f) {
            chroma += std::fabs(r - b);
            lum += m;
            hits++;
        }
        auto enc = [](float v) {
            float x = v / (v + 3500.f);
            return (uint8_t)std::clamp(int(std::pow(std::max(x, 0.f), 0.85f) * 255.f), 0, 255);
        };
        pic[(size_t)i * 4 + 0] = enc(r);
        pic[(size_t)i * 4 + 1] = enc(g);
        pic[(size_t)i * 4 + 2] = enc(b);
        pic[(size_t)i * 4 + 3] = 255;
    }
    write_png("out/prism-gpu-map.png", MAP, MAP, pic.data());
    std::printf("gpu map hits %d sum %.3e max %.1f chroma %.3f  rgb %.3e %.3e %.3e blueish %d  %.1f ms wall\n", hits, sum, maxv,
                lum > 1.f ? chroma / lum : 0.f, sr, sg, sb, blueish, std::chrono::duration<double, std::milli>(t1 - t0).count());

    if (!open()) return fail("mono");
    rm.setMaterialInstanceAt(rm.getInstance(splat), 0, splatMono);
    view->setRenderTarget(mapRT);
    renderer->render(view);
    renderer->readPixels(mapRT, 0, 0, MAP, MAP,
                         backend::PixelBufferDescriptor(mapHalf.data(), mapHalf.size() * 2, backend::PixelDataFormat::RGBA,
                                                        backend::PixelDataType::HALF));
    renderer->endFrame();
    engine->flushAndWait();
    float chroma0 = 0, lum0 = 0;
    for (int i = 0; i < MAP * MAP; ++i) {
        float r = std::max(half_to_float(mapHalf[(size_t)i * 4 + 0]), 0.f);
        float g = std::max(half_to_float(mapHalf[(size_t)i * 4 + 1]), 0.f);
        float b = std::max(half_to_float(mapHalf[(size_t)i * 4 + 2]), 0.f);
        float m = std::max(r, std::max(g, b));
        if (m > 30.f) {
            chroma0 += std::fabs(r - b);
            lum0 += m;
        }
    }
    std::printf("gpu same-ior chroma %.3f   dispersed chroma %.3f\n", lum0 > 1.f ? chroma0 / lum0 : 0.f, lum > 1.f ? chroma / lum : 0.f);

    rm.setMaterialInstanceAt(rm.getInstance(splat), 0, splatMi);
    if (!open()) return fail("redraw");
    view->setRenderTarget(mapRT);
    renderer->render(view);
    renderer->endFrame();
    engine->flushAndWait();

    TextureSampler lin(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
    lin.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    lin.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    MaterialInstance* floorMi = floorMat->createInstance();
    floorMi->setParameter("color", float4{0.86f, 0.85f, 0.84f, 1});
    floorMi->setParameter("roughness", 0.42f);
    floorMi->setParameter("caustic", mapTex, lin);
    MaterialInstance* glass = glassMat->createInstance();
    glass->setParameter("ior", nd);
    glass->setParameter("transmission", 1.f);
    glass->setParameter("thickness", half * 2.f);
    glass->setParameter("dispersion", dispersion);
    glass->setParameter("reflectance", 0.5f);
    glass->setParameter("alpha", 1.f);
    glass->setParameter("absorption", float3{0.02f, 0.015f, 0.02f});
    rm.setMaterialInstanceAt(rm.getInstance(cube), 0, glass);

    std::vector<float> floor_v(4 * 9);
    std::vector<uint32_t> floor_i = {0, 2, 1, 0, 3, 2};
    quatf fq = mat3f::packTangentFrame(mat3f{float3{1, 0, 0}, float3{0, 0, -1}, float3{0, 1, 0}});
    V3 fp[4] = {{-HW, 0, -HH}, {HW, 0, -HH}, {HW, 0, HH}, {-HW, 0, HH}};
    float uv[4][2] = {{0, 0}, {1, 0}, {1, 1}, {0, 1}};
    for (int i = 0; i < 4; ++i) {
        float* o = floor_v.data() + i * 9;
        o[0] = fp[i].x;
        o[1] = fp[i].y;
        o[2] = fp[i].z;
        o[3] = uv[i][0];
        o[4] = uv[i][1];
        o[5] = fq.x;
        o[6] = fq.y;
        o[7] = fq.z;
        o[8] = fq.w;
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

    utils::Entity sun_e = utils::EntityManager::get().create();
    LightManager::ShadowOptions shopt;
    shopt.mapSize = 2048;
    shopt.vsm.blurWidth = 8.f;
    shopt.vsm.elvsm = true;
    LightManager::Builder(LightManager::Type::SUN)
        .color(float3{1, 0.98f, 0.96f}).intensity(lux).direction(float3{sun.x, sun.y, sun.z})
        .sunAngularRadius(0.35f).sunHaloSize(1.2f).castShadows(true).shadowOptions(shopt)
        .build(*engine, sun_e);

    static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
    view->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.04f).saturation(1.08f).build(*engine));
    view->setPostProcessingEnabled(true);
    view->setScreenSpaceRefractionEnabled(true);
    view->setShadowType(View::ShadowType::VSM);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    scene->setSkybox(Skybox::Builder().color({0.72f, 0.74f, 0.78f, 1}).build(*engine));
    scene->remove(splat);
    scene->addEntity(floor);
    scene->addEntity(cube);
    scene->addEntity(sun_e);
    camera->setExposure(2.8f, 1.f / 125.f, 100.f);
    camera->lookAt(float3{0.55f, 0.78f, 0.70f}, float3{0, 0.04f, 0}, float3{0, 1, 0});
    camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);
    if (!open()) return fail("beauty");
    view->setRenderTarget(beautyRT);
    renderer->render(view);
    std::vector<uint8_t> frame((size_t)W * H * 4);
    renderer->readPixels(beautyRT, 0, 0, W, H,
                         backend::PixelBufferDescriptor(frame.data(), frame.size(), backend::PixelDataFormat::RGBA,
                                                        backend::PixelDataType::UBYTE));
    renderer->endFrame();
    engine->flushAndWait();
    if (!write_png("out/prism-gpu.png", W, H, frame.data())) return fail("png");
    std::printf("PASS prism-gpu\n");
    return 0;
}
