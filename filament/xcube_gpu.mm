// GPU X-cube. Neighboring rays of one path become a quad, same as paint_cell.
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

#include <math/mat3.h>
#include <math/vec3.h>
#include <math/vec4.h>

#include <utils/EntityManager.h>

#include <zlib.h>

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdlib>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <vector>

using namespace filament;
using namespace filament::math;

static const int W = 1920, H = 1080, MAP = 512, PATHS = 8;
static int GRID = 84, NW = 21;
static const float HW = 1.55f;

struct V3 {
    float x, y, z;
    V3 operator+(V3 o) const { return {x + o.x, y + o.y, z + o.z}; }
    V3 operator-(V3 o) const { return {x - o.x, y - o.y, z - o.z}; }
    V3 operator*(float s) const { return {x * s, y * s, z * s}; }
};
static float dot(V3 a, V3 b) { return a.x * b.x + a.y * b.y + a.z * b.z; }
static V3 cross(V3 a, V3 b) { return {a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x}; }
static V3 norm(V3 a) {
    float L = std::sqrt(dot(a, a));
    return L > 1e-8f ? a * (1.f / L) : V3{0, 1, 0};
}
static void rotateY(V3& v, float a) {
    float c = std::cos(a), s = std::sin(a);
    float x = v.x * c + v.z * s;
    v.z = -v.x * s + v.z * c;
    v.x = x;
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

int main(int argc, char** argv) {
    const bool motion = argc > 1 && std::strcmp(argv[1], "motion") == 0;
    const bool tryMode = argc > 3 && std::strcmp(argv[1], "try") == 0;
    if (tryMode) {
        NW = std::max(1, std::atoi(argv[2]));
        GRID = std::max(2, std::atoi(argv[3]));
    }
    const float half = 0.16f;
    const float nd = 1.55f;
    const float dispersion = 36.f;
    const float lux = 22000.f;
    const float extent = half * 3.1f;
    V3 sun0 = norm({0.92f, -0.42f, 0.22f});
    V3 axis[3] = {{1, 0, 0}, {0, 1, 0}, {0, 0, 1}};
    for (int i = 0; i < 3; ++i) rotateY(axis[i], 0.62f);
    V3 center{0.f, half + 0.001f, 0.f};

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
    scene->setSkybox(nullptr);

    auto make_rt = [&](int w, int h, int levels, bool mip, Texture::InternalFormat fmt) {
        auto usage = Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC;
        if (mip) usage |= Texture::Usage::BLIT_DST | Texture::Usage::GEN_MIPMAPPABLE;
        Texture* color = Texture::Builder().width((uint32_t)w).height((uint32_t)h).levels((uint8_t)levels)
                             .usage(usage).format(fmt).build(*engine);
        Texture* depth = Texture::Builder().width((uint32_t)w).height((uint32_t)h).levels(1)
                             .usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH32F).build(*engine);
        RenderTarget* rt = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, color)
                               .texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
        return std::tuple<Texture*, Texture*, RenderTarget*>{color, depth, rt};
    };
    const int hitW = GRID * NW;
    const int hitH = GRID * PATHS;
    auto [hitTex, hitDepth, hitRT] = make_rt(hitW, hitH, 1, false, Texture::InternalFormat::RGBA16F);
    auto [mapTex, mapDepth, mapRT] = make_rt(MAP, MAP, 1, false, Texture::InternalFormat::RGBA16F);
    auto [tempTex, tempDepth, tempRT] = make_rt(MAP, MAP, 10, true, Texture::InternalFormat::RGBA16F);
    auto [filtTex, filtDepth, filtRT] = make_rt(MAP, MAP, 1, false, Texture::InternalFormat::RGBA16F);
    auto [beauty, beautyDepth, beautyRT] = make_rt(W, H, 1, false, Texture::InternalFormat::RGBA8);
    (void)hitDepth;
    (void)mapDepth;
    (void)tempDepth;
    (void)filtDepth;
    (void)beautyDepth;

    auto hpkg = load_file("xcube_hit.filamat");
    auto spkg = load_file("xcube_splat.filamat");
    auto apkg = load_file("caustic_atrous.filamat");
    auto gpkg = load_file("glass_shadow.filamat");
    auto fpkg = load_file("floor_recv.filamat");
    auto epkg = load_file("emissive.filamat");
    auto filmPkg = load_file("xcube_film.filamat");
    auto filmLitPkg = load_file("xcube_film_lit.filamat");
    auto echoPkg = load_file("xcube_echo.filamat");
    auto dreiPkg = load_file("xcube_drei.filamat");
    auto octaPkg = load_file("xcube_octa_hit.filamat");
    if (hpkg.empty() || spkg.empty() || apkg.empty() || gpkg.empty() || fpkg.empty() || epkg.empty() || filmPkg.empty() || filmLitPkg.empty() || echoPkg.empty() || dreiPkg.empty() || octaPkg.empty()) return fail("filamat");
    Material* hitMat = Material::Builder().package(hpkg.data(), hpkg.size()).build(*engine);
    Material* splatMat = Material::Builder().package(spkg.data(), spkg.size()).build(*engine);
    Material* atrousMat = Material::Builder().package(apkg.data(), apkg.size()).build(*engine);
    Material* glassMat = Material::Builder().package(gpkg.data(), gpkg.size()).build(*engine);
    Material* floorMat = Material::Builder().package(fpkg.data(), fpkg.size()).build(*engine);
    Material* emat = Material::Builder().package(epkg.data(), epkg.size()).build(*engine);
    Material* filmMat = Material::Builder().package(filmPkg.data(), filmPkg.size()).build(*engine);
    Material* filmLitMat = Material::Builder().package(filmLitPkg.data(), filmLitPkg.size()).build(*engine);
    Material* echoMat = Material::Builder().package(echoPkg.data(), echoPkg.size()).build(*engine);
    Material* dreiMat = Material::Builder().package(dreiPkg.data(), dreiPkg.size()).build(*engine);
    Material* octaMat = Material::Builder().package(octaPkg.data(), octaPkg.size()).build(*engine);
    if (!hitMat || !splatMat || !atrousMat || !glassMat || !floorMat || !emat || !filmMat || !filmLitMat || !echoMat || !dreiMat || !octaMat) return fail("material");

    MaterialInstance* hitMi = hitMat->createInstance();
    hitMi->setParameter("center", float3{center.x, center.y, center.z});
    hitMi->setParameter("ax", float3{axis[0].x, axis[0].y, axis[0].z});
    hitMi->setParameter("ay", float3{axis[1].x, axis[1].y, axis[1].z});
    hitMi->setParameter("az", float3{axis[2].x, axis[2].y, axis[2].z});
    hitMi->setParameter("halfExtent", half);
    hitMi->setParameter("extent", extent);
    hitMi->setParameter("grid", (float)GRID);
    hitMi->setParameter("bands", (float)NW);
    hitMi->setParameter("ior", nd);
    hitMi->setParameter("dispersion", dispersion);
    hitMi->setParameter("hitW", (float)hitW);
    hitMi->setParameter("hitH", (float)hitH);
    hitMi->setParameter("bevel", 0.f);
    hitMi->setParameter("tilt", 0.f);
    float hq[] = {-1, -1, 0, 0, 0, 1, -1, 0, 1, 0, 1, 1, 0, 1, 1, -1, 1, 0, 0, 1};
    uint32_t hi[] = {0, 1, 2, 0, 2, 3};
    VertexBuffer* hitVB = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                              .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                              .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20)
                              .build(*engine);
    hitVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(hq, sizeof(hq)));
    IndexBuffer* hitIB = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    hitIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(hi, sizeof(hi)));
    utils::Entity hitE = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, hitVB, hitIB)
        .material(0, hitMi).culling(false).castShadows(false).receiveShadows(false)
        .build(*engine, hitE);

    TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    nearest.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    nearest.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    MaterialInstance* splatMi = splatMat->createInstance();
    splatMi->setParameter("hits", hitTex, nearest);
    splatMi->setParameter("hitW", (float)hitW);
    splatMi->setParameter("hitH", (float)hitH);
    splatMi->setParameter("extent", extent);
    splatMi->setParameter("grid", (float)GRID);
    splatMi->setParameter("bands", (float)NW);
    splatMi->setParameter("lux", lux);

    const int cells = GRID - 1;
    const int nvert = cells * cells * NW * PATHS * 6;
    std::vector<float> sv((size_t)nvert * 5, 0.f);
    std::vector<uint32_t> si((size_t)nvert);
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
                                .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20)
                                .build(*engine);
    splatVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(sv.data(), sv.size() * sizeof(float)));
    IndexBuffer* splatIB = IndexBuffer::Builder().indexCount((uint32_t)nvert).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
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
    MaterialInstance* atrous1 = atrousMat->createInstance();
    atrous1->setParameter("src", mapTex, lin);
    atrous1->setParameter("texel", float2{1.f / MAP, 1.f / MAP});
    atrous1->setParameter("stride", 1.f);
    atrous1->setParameter("lumWeight", 0.f);
    atrous1->setParameter("mip", 0.f);
    atrous1->setParameter("second", 0.f);
    MaterialInstance* atrous2 = atrousMat->createInstance();
    atrous2->setParameter("src", tempTex, mipped);
    atrous2->setParameter("texel", float2{1.f / MAP, 1.f / MAP});
    atrous2->setParameter("stride", 2.f);
    atrous2->setParameter("lumWeight", 0.f);
    atrous2->setParameter("mip", 1.f);
    atrous2->setParameter("second", 1.f);

    std::vector<float> floor_v(4 * 9);
    std::vector<uint32_t> floor_i = {0, 2, 1, 0, 3, 2};
    quatf fq = mat3f::packTangentFrame(mat3f{float3{1, 0, 0}, float3{0, 0, -1}, float3{0, 1, 0}});
    V3 fp[4] = {{-HW, 0, -HW}, {HW, 0, -HW}, {HW, 0, HW}, {-HW, 0, HW}};
    float uv[4][2] = {{0, 0}, {1, 0}, {1, 1}, {0, 1}};
    for (int i = 0; i < 4; ++i) {
        float* o = floor_v.data() + i * 9;
        o[0] = fp[i].x; o[1] = 0; o[2] = fp[i].z;
        o[3] = uv[i][0]; o[4] = uv[i][1];
        o[5] = fq.x; o[6] = fq.y; o[7] = fq.z; o[8] = fq.w;
    }
    MaterialInstance* floorMi = floorMat->createInstance();
    floorMi->setParameter("color", float4{0.86f, 0.85f, 0.84f, 1});
    floorMi->setParameter("roughness", 0.42f);
    floorMi->setParameter("caustic", filtTex, lin);
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
    VertexBuffer* cubeVB = VertexBuffer::Builder().vertexCount((uint32_t)(cube_v.size() / 7)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                               .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                               .build(*engine);
    cubeVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(cube_v.data(), cube_v.size() * sizeof(float)));
    IndexBuffer* cubeIB = IndexBuffer::Builder().indexCount((uint32_t)cube_i.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    cubeIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(cube_i.data(), cube_i.size() * sizeof(uint32_t)));
    MaterialInstance* glass = glassMat->createInstance();
    glass->setParameter("ior", nd);
    glass->setParameter("transmission", 1.f);
    glass->setParameter("thickness", half * 2.f);
    glass->setParameter("dispersion", dispersion);
    glass->setParameter("reflectance", 0.5f);
    glass->setParameter("alpha", 1.f);
    glass->setParameter("absorption", float3{0.02f, 0.015f, 0.02f});
    utils::Entity cube = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, cubeVB, cubeIB)
        .material(0, glass).culling(false).castShadows(true).receiveShadows(false)
        .build(*engine, cube);
    const float kBevel = 0.012f;
    const float kTilt = 0.08f;
    std::vector<float> bev_v;
    std::vector<uint32_t> bev_i;
    {
        float m = half - kBevel;
        auto P = [&](float x, float y, float z) { return center + axis[0] * x + axis[1] * y + axis[2] * z; };
        auto face = [&](V3 a, V3 b, V3 c, V3 d, V3 want) {
            if (dot(norm(cross(b - a, d - a)), want) < 0.f) {
                V3 t = b;
                b = d;
                d = t;
            }
            V3 T = norm(b - a);
            uint32_t base = (uint32_t)(bev_v.size() / 7);
            push_vert(bev_v, a, T, want);
            push_vert(bev_v, b, T, want);
            push_vert(bev_v, c, T, want);
            push_vert(bev_v, d, T, want);
            bev_i.insert(bev_i.end(), {base, base + 1, base + 2, base, base + 2, base + 3});
        };
        for (int i = 0; i < 3; ++i) {
            int u = (i + 1) % 3;
            int v = (i + 2) % 3;
            for (int s = -1; s <= 1; s += 2) {
                float ss = (float)s;
                V3 N = axis[i] * ss;
                float c[3] = {};
                c[i] = ss * half;
                c[u] = -m;
                c[v] = -m;
                V3 p00 = P(c[0], c[1], c[2]);
                c[u] = m;
                V3 p10 = P(c[0], c[1], c[2]);
                c[v] = m;
                V3 p11 = P(c[0], c[1], c[2]);
                c[u] = -m;
                V3 p01 = P(c[0], c[1], c[2]);
                face(p00, p10, p11, p01, N);
            }
        }
        for (int sx = -1; sx <= 1; sx += 2)
            for (int sy = -1; sy <= 1; sy += 2) {
                float x = (float)sx, y = (float)sy;
                face(P(x * half, y * m, -m), P(x * half, y * m, m), P(x * m, y * half, m), P(x * m, y * half, -m),
                     norm(axis[0] * x + axis[1] * y));
            }
        for (int sx = -1; sx <= 1; sx += 2)
            for (int sz = -1; sz <= 1; sz += 2) {
                float x = (float)sx, z = (float)sz;
                face(P(x * half, -m, z * m), P(x * half, m, z * m), P(x * m, m, z * half), P(x * m, -m, z * half),
                     norm(axis[0] * x + axis[2] * z));
            }
        for (int sy = -1; sy <= 1; sy += 2)
            for (int sz = -1; sz <= 1; sz += 2) {
                float y = (float)sy, z = (float)sz;
                face(P(-m, y * half, z * m), P(m, y * half, z * m), P(m, y * m, z * half), P(-m, y * m, z * half),
                     norm(axis[1] * y + axis[2] * z));
            }
        for (int sx = -1; sx <= 1; sx += 2)
            for (int sy = -1; sy <= 1; sy += 2)
                for (int sz = -1; sz <= 1; sz += 2) {
                    float x = (float)sx, y = (float)sy, z = (float)sz;
                    V3 a = P(x * half, y * m, z * m);
                    V3 b = P(x * m, y * half, z * m);
                    V3 c = P(x * m, y * m, z * half);
                    V3 N = norm(axis[0] * x + axis[1] * y + axis[2] * z);
                    if (dot(norm(cross(b - a, c - a)), N) < 0.f) {
                        V3 t = b;
                        b = c;
                        c = t;
                    }
                    V3 T = norm(b - a);
                    uint32_t base = (uint32_t)(bev_v.size() / 7);
                    push_vert(bev_v, a, T, N);
                    push_vert(bev_v, b, T, N);
                    push_vert(bev_v, c, T, N);
                    bev_i.insert(bev_i.end(), {base, base + 1, base + 2});
                }
    }
    VertexBuffer* bevVB = VertexBuffer::Builder().vertexCount((uint32_t)(bev_v.size() / 7)).bufferCount(1)
                              .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                              .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                              .build(*engine);
    bevVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(bev_v.data(), bev_v.size() * sizeof(float)));
    IndexBuffer* bevIB = IndexBuffer::Builder().indexCount((uint32_t)bev_i.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    bevIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(bev_i.data(), bev_i.size() * sizeof(uint32_t)));
    utils::Entity cubeBevel = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, bevVB, bevIB)
        .material(0, glass).culling(false).castShadows(true).receiveShadows(false)
        .build(*engine, cubeBevel);
    std::vector<float> echo_v = bev_v;
    for (size_t i = 0; i + 6 < echo_v.size(); i += 7) {
        V3 p{echo_v[i], echo_v[i + 1], echo_v[i + 2]};
        p = center + (p - center) * 1.012f;
        echo_v[i] = p.x;
        echo_v[i + 1] = p.y;
        echo_v[i + 2] = p.z;
    }
    VertexBuffer* echoVB = VertexBuffer::Builder().vertexCount((uint32_t)(echo_v.size() / 7)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                               .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                               .build(*engine);
#include "xcube_gpu_2.inc"
#include "xcube_gpu_3.inc"
#include "xcube_gpu_4.inc"
