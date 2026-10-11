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
    echoVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(echo_v.data(), echo_v.size() * sizeof(float)));
    MaterialInstance* echoMi = echoMat->createInstance();
    echoMi->setParameter("eye", float3{0.55f, 0.78f, 0.70f});
    echoMi->setParameter("center", float3{center.x, center.y, center.z});
    echoMi->setParameter("ax", float3{axis[0].x, axis[0].y, axis[0].z});
    echoMi->setParameter("ay", float3{axis[1].x, axis[1].y, axis[1].z});
    echoMi->setParameter("az", float3{axis[2].x, axis[2].y, axis[2].z});
    echoMi->setParameter("halfExtent", half);
    echoMi->setParameter("bevel", kBevel);
    echoMi->setParameter("tilt", kTilt);
    echoMi->setParameter("ior", nd);
    echoMi->setParameter("bounces", 1.f);
    echoMi->setParameter("colorA", float3{0.95f, 0.28f, 0.62f});
    echoMi->setParameter("colorB", float3{0.20f, 0.72f, 0.95f});
    utils::Entity echoE = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, echoVB, bevIB)
        .material(0, echoMi).culling(false).castShadows(false).receiveShadows(false)
        .build(*engine, echoE);
    std::vector<float> panel_v;
    std::vector<uint32_t> panel_i = {0, 1, 2, 0, 2, 3};
    {
        V3 c{0.0f, 0.55f, -1.15f};
        float hw = 1.6f, hh = 0.7f;
        V3 p[4] = {c + V3{-hw, -hh, 0}, c + V3{hw, -hh, 0}, c + V3{hw, hh, 0}, c + V3{-hw, hh, 0}};
        for (int i = 0; i < 4; ++i) push_vert(panel_v, p[i], {1, 0, 0}, {0, 0, 1});
    }
    VertexBuffer* panelVB = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                                .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                                .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                                .build(*engine);
    panelVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(panel_v.data(), panel_v.size() * sizeof(float)));
    IndexBuffer* panelIB = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    panelIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(panel_i.data(), panel_i.size() * sizeof(uint32_t)));
    MaterialInstance* panel = emat->createInstance();
    panel->setParameter("color", float4{2200.f, 2300.f, 2600.f, 1});
    utils::Entity panelE = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-4, -4, -4}, {4, 4, 4}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, panelVB, panelIB)
        .material(0, panel).culling(false).castShadows(false).receiveShadows(false)
        .build(*engine, panelE);

    auto make_film = [&](V3 N0, float tilt, float4 color, float4 glow, Material* mat) {
        N0 = norm(N0);
        V3 up = norm(axis[1]);
        V3 side = norm(cross(up, N0));
        float band = (2.f * half) / 3.f;
        float s = half * 0.90f;
        int quads = tilt == 0.f ? 1 : 3;
        static std::vector<float> store[8];
        static std::vector<uint32_t> index[8];
        static int which = 0;
        std::vector<float>& fv = store[which];
        std::vector<uint32_t>& fi = index[which];
        which += 1;
        fv.clear();
        fi.clear();
        for (int k = 0; k < quads; ++k) {
            int kk = quads == 1 ? 0 : k - 1;
            float ang = (float)kk * tilt;
            V3 N = norm(N0 * std::cos(ang) + side * std::sin(ang));
            V3 T = norm(cross(up, N));
            V3 c0 = center + up * ((float)kk * band);
            float vs = quads == 1 ? s : band * 0.55f;
            V3 q[4] = {c0 - T * s - up * vs, c0 + T * s - up * vs, c0 + T * s + up * vs, c0 - T * s + up * vs};
            uint32_t base = (uint32_t)(fv.size() / 7);
            for (int i = 0; i < 4; ++i) push_vert(fv, q[i], T, N);
            fi.insert(fi.end(), {base, base + 1, base + 2, base, base + 2, base + 3});
        }
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(fv.size() / 7)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                               .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                               .build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(fv.data(), fv.size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)fi.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(fi.data(), fi.size() * sizeof(uint32_t)));
        MaterialInstance* mi = mat->createInstance();
        mi->setParameter("color", color);
        mi->setParameter("glow", glow);
        utils::Entity e = utils::EntityManager::get().create();
        RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, mi).culling(false).castShadows(false).receiveShadows(false)
            .build(*engine, e);
        return e;
    };
    float4 pink{0.95f, 0.28f, 0.62f, 0.42f};
    float4 cyan{0.20f, 0.72f, 0.95f, 0.42f};
    float4 pinkMirror{0.95f, 0.28f, 0.62f, 1.f};
    float4 cyanMirror{0.20f, 0.72f, 0.95f, 1.f};
    utils::Entity filmA = make_film(norm(axis[0] - axis[2]), 0.f, pink, float4{0, 0, 0, 1}, filmMat);
    utils::Entity filmB = make_film(norm(axis[0] + axis[2]), 0.f, cyan, float4{0, 0, 0, 1}, filmMat);
    utils::Entity filmAf = make_film(norm(axis[0] - axis[2]), kTilt, pink, float4{0, 0, 0, 1}, filmMat);
    utils::Entity filmBf = make_film(norm(axis[0] + axis[2]), kTilt, cyan, float4{0, 0, 0, 1}, filmMat);
    utils::Entity filmAl = make_film(norm(axis[0] - axis[2]), kTilt, pinkMirror, float4{0, 0, 0, 1}, filmLitMat);
    utils::Entity filmBl = make_film(norm(axis[0] + axis[2]), kTilt, cyanMirror, float4{0, 0, 0, 1}, filmLitMat);

    utils::Entity sun_e = utils::EntityManager::get().create();
    LightManager::ShadowOptions shopt;
    shopt.mapSize = 512;
    shopt.shadowNearHint = 1.0f;
    shopt.shadowFarHint = 100.0f;
    shopt.constantBias = 0.001f;
    shopt.normalBias = 1.0f;
    LightManager::Builder(LightManager::Type::SUN)
        .color(float3{1, 0.98f, 0.96f}).intensity(lux).direction(float3{sun0.x, sun0.y, sun0.z})
        .sunAngularRadius(0.35f).sunHaloSize(1.2f).castShadows(true).shadowOptions(shopt)
        .build(*engine, sun_e);

    auto& rm = engine->getRenderableManager();
    auto open = [&]() {
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) return true;
        return false;
    };
    Renderer::ClearOptions clearOpt;
    clearOpt.clear = true;
    clearOpt.discard = true;
    clearOpt.clearColor = {0, 0, 0, 0};
    renderer->setClearOptions(clearOpt);
    Skybox* sky = Skybox::Builder().color({0.72f, 0.74f, 0.78f, 1}).build(*engine);
    Skybox* skyDark = Skybox::Builder().color({0.22f, 0.23f, 0.26f, 1}).build(*engine);
    static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
    BloomOptions bloom{};
    bloom.enabled = true;
    bloom.strength = 0.18f;
    bloom.levels = 6;
    bloom.resolution = 1080;
    bloom.threshold = true;
    bloom.highlight = 10.f;
    bloom.blendMode = BloomOptions::BlendMode::ADD;

    auto bind_sun = [&](V3 sun) {
        V3 D = norm(sun);
        V3 up = std::fabs(D.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0};
        V3 U = norm(cross(up, D));
        V3 V = cross(D, U);
        hitMi->setParameter("dir", float3{D.x, D.y, D.z});
        hitMi->setParameter("tangent", float3{U.x, U.y, U.z});
        hitMi->setParameter("bitangent", float3{V.x, V.y, V.z});
        auto& lm = engine->getLightManager();
        lm.setDirection(lm.getInstance(sun_e), float3{D.x, D.y, D.z});
    };
    auto draw_hits = [&]() -> bool {
        view->setPostProcessingEnabled(false);
        view->setScreenSpaceRefractionEnabled(false);
        view->setBloomOptions(BloomOptions{});
        scene->setSkybox(nullptr);
        camera->setExposure(1.f);
        view->setViewport({0, 0, (uint32_t)hitW, (uint32_t)hitH});
        scene->remove(floor);
        scene->remove(cube);
        scene->remove(panelE);
        scene->remove(sun_e);
        scene->remove(filter);
        scene->remove(splat);
        scene->addEntity(hitE);
        if (!open()) return false;
        view->setRenderTarget(hitRT);
        renderer->render(view);
        renderer->endFrame();
        engine->flushAndWait();
        return true;
    };
    auto draw_paths = [&]() -> bool {
        view->setPostProcessingEnabled(false);
        view->setScreenSpaceRefractionEnabled(false);
        view->setBloomOptions(BloomOptions{});
        scene->setSkybox(nullptr);
        camera->setExposure(1.f);
        camera->lookAt(float3{0, 4, 0}, float3{0, 0, 0}, float3{0, 0, -1});
        camera->setProjection(Camera::Projection::ORTHO, -HW, HW, -HW, HW, 0.01, 10.0);
        view->setViewport({0, 0, (uint32_t)MAP, (uint32_t)MAP});
        scene->remove(floor);
        scene->remove(cube);
        scene->remove(panelE);
        scene->remove(sun_e);
        scene->remove(filter);
        scene->remove(hitE);
        scene->addEntity(splat);
        if (!open()) return false;
        view->setRenderTarget(mapRT);
        renderer->render(view);
        renderer->endFrame();
        engine->flushAndWait();
        return true;
    };
    auto draw_atrous = [&]() -> bool {
        scene->remove(splat);
        scene->remove(hitE);
        scene->addEntity(filter);
        rm.setMaterialInstanceAt(rm.getInstance(filter), 0, atrous1);
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
        return true;
    };
    bool graded = false;
    auto arm_beauty = [&]() {
        scene->remove(splat);
        scene->remove(hitE);
        scene->remove(filter);
        scene->addEntity(floor);
        scene->addEntity(cube);
        scene->addEntity(panelE);
        scene->addEntity(sun_e);
        scene->setSkybox(sky);
        if (!graded) {
            view->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.04f).saturation(1.08f).build(*engine));
            graded = true;
        }
        view->setPostProcessingEnabled(true);
        view->setScreenSpaceRefractionEnabled(true);
        view->setShadowType(View::ShadowType::PCF);
        view->setBloomOptions(bloom);
        view->setAntiAliasing(View::AntiAliasing::FXAA);
        view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
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

    bind_sun(sun0);
    if (!draw_hits()) return fail("hits");
    if (!draw_paths()) return fail("paths");
    if (!draw_atrous()) return fail("atrous");
    arm_beauty();
    if (!draw_beauty()) return fail("beauty");
    std::vector<double> hitMs, pathMs, atrousMs, beautyMs, totalMs;
    for (int i = 0; i < 5; ++i) {
        auto t0 = std::chrono::steady_clock::now();
        if (!draw_hits()) return fail("time-hits");
        hitMs.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
        auto tq = std::chrono::steady_clock::now();
        if (!draw_paths()) return fail("time-paths");
        pathMs.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - tq).count());
        auto t1 = std::chrono::steady_clock::now();
        if (!draw_atrous()) return fail("time-atrous");
        atrousMs.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t1).count());
        arm_beauty();
        auto t2 = std::chrono::steady_clock::now();
        if (!draw_beauty()) return fail("time-beauty");
        beautyMs.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t2).count());
        totalMs.push_back(hitMs.back() + pathMs.back() + atrousMs.back() + beautyMs.back());
    }
    auto med = [](std::vector<double> v) {
        std::sort(v.begin(), v.end());
        return v[v.size() / 2];
    };
    std::printf("gpu wall ms, flushAndWait, readPixels excluded, median of 5\n");
    std::printf("hits %.2f  quads %.2f  atrous %.2f  beauty %.2f  total %.2f\n", med(hitMs), med(pathMs), med(atrousMs), med(beautyMs), med(totalMs));
    std::printf("%d wavelengths, grid %d, PCF 512. Not a Metal timestamp.\n", NW, GRID);
    if (tryMode) {
        char tryPath[64];
        std::snprintf(tryPath, sizeof(tryPath), "out/xcube-try-%d-%d.png", NW, GRID);
        arm_beauty();
        if (!grab(tryPath)) return fail("png");
        std::printf("PASS xcube-gpu\n");
        return 0;
    }

    auto& lm = engine->getLightManager();
    auto light = lm.getInstance(sun_e);
    LightManager::ShadowOptions wide = shopt;
    LightManager::ShadowOptions tight = shopt;
    tight.shadowNearHint = 0.2f;
    tight.shadowFarHint = 2.2f;
    tight.constantBias = 0.0006f;
    tight.normalBias = 0.4f;
    tight.polygonOffsetConstant = 0.3f;
    tight.polygonOffsetSlope = 1.0f;
    auto show_films = [&](bool on) {
        scene->remove(filmA);
        scene->remove(filmB);
        if (on) {
            scene->addEntity(filmA);
            scene->addEntity(filmB);
        }
    };
    auto milk = [&]() {
        arm_beauty();
        show_films(false);
        scene->addEntity(panelE);
        scene->setSkybox(sky);
        floorMi->setParameter("color", float4{0.86f, 0.85f, 0.84f, 1});
        glass->setParameter("reflectance", 0.5f);
        glass->setParameter("absorption", float3{0.02f, 0.015f, 0.02f});
        camera->setExposure(2.8f, 1.f / 125.f, 100.f);
        lm.setSunAngularRadius(light, 0.35f);
        lm.setShadowOptions(light, wide);
        view->setShadowType(View::ShadowType::PCF);
        view->setShadowingEnabled(true);
        view->setBloomOptions(bloom);
    };
    auto clear_glass = [&]() {
        arm_beauty();
        scene->remove(panelE);
        scene->setSkybox(sky);
        floorMi->setParameter("color", float4{0.90f, 0.89f, 0.88f, 1});
        glass->setParameter("reflectance", 0.42f);
        glass->setParameter("absorption", float3{0.004f, 0.004f, 0.006f, });
        camera->setExposure(2.8f, 1.f / 180.f, 100.f);
        BloomOptions lean = bloom;
        lean.strength = 0.08f;
        view->setBloomOptions(lean);
        view->setShadowType(View::ShadowType::PCF);
        view->setShadowingEnabled(true);
    };
    milk();
    if (!grab("out/xcube-rw-a.png")) return fail("a");
    clear_glass();
    lm.setShadowOptions(light, wide);
    if (!grab("out/xcube-rw-b.png")) return fail("b");
    clear_glass();
    show_films(true);
    lm.setShadowOptions(light, wide);
    if (!grab("out/xcube-rw-c.png")) return fail("c");
    milk();
    lm.setShadowOptions(light, tight);
    if (!grab("out/xcube-rw-d.png")) return fail("d");
    clear_glass();
    show_films(true);
    lm.setShadowOptions(light, tight);
    lm.setSunAngularRadius(light, 0.02f);
    view->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.2f).saturation(1.4f).build(*engine));
    if (!grab("out/xcube-rw-e.png")) return fail("e");

    auto time_beauty = [&](const char* name) {
        if (!draw_beauty()) return;
        std::vector<double> v;
        for (int i = 0; i < 5; ++i) {
            auto t = std::chrono::steady_clock::now();
            if (!draw_beauty()) return;
            v.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t).count());
        }
        std::printf("beauty %-16s %.2f\n", name, med(v));
    };
    time_beauty("E pcf512 tight");
    tight.mapSize = 1024;
    lm.setShadowOptions(light, tight);
    time_beauty("E pcf1024 tight");
    if (!grab("out/xcube-rw-e-1k.png")) return fail("e1k");
    tight.mapSize = 512;
    lm.setShadowOptions(light, tight);
    view->setShadowingEnabled(false);
    time_beauty("E no shadow");
    view->setShadowingEnabled(true);
    view->setScreenSpaceRefractionEnabled(false);
    time_beauty("E no refraction");
    view->setScreenSpaceRefractionEnabled(true);
    BloomOptions bloomOff{};
    view->setBloomOptions(bloomOff);
    time_beauty("E no bloom");
    view->setBloomOptions(bloom);
    BloomOptions lean = bloom;
    lean.strength = 0.08f;
    view->setBloomOptions(lean);

    auto bake = [&](float bevel, float tilt) {
        hitMi->setParameter("bevel", bevel);
        hitMi->setParameter("tilt", tilt);
        if (!draw_hits()) return false;
        if (!draw_paths()) return false;
        if (!draw_atrous()) return false;
        return true;
    };
    auto facet0 = std::chrono::steady_clock::now();
    if (!bake(kBevel, kTilt)) return fail("facet");
    double facetMs = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - facet0).count();
    clear_glass();
    show_films(false);
    scene->remove(cube);
    scene->addEntity(cubeBevel);
    scene->addEntity(filmAf);
    scene->addEntity(filmBf);
    lm.setShadowOptions(light, tight);
    lm.setSunAngularRadius(light, 0.02f);
    if (!grab("out/xcube-rw-f.png")) return fail("f");
    time_beauty("F pcf512 tight");
    std::printf("facet bake %.2f bevel %.3f tilt %.3f\n", facetMs, kBevel, kTilt);
    scene->addEntity(echoE);
    echoMi->setParameter("bounces", 1.f);
    if (!grab("out/xcube-rw-g1.png")) return fail("g1");
    time_beauty("G1 one bounce");
    echoMi->setParameter("bounces", 2.f);
    if (!grab("out/xcube-rw-g2.png")) return fail("g2");
    time_beauty("G2 two bounces");
    scene->remove(echoE);

    const int kFace = 128;
    const float kIbl = 2200.f;
    auto face_dir = [](int face, float u, float v) {
        switch (face) {
        case 0: return norm({1.f, v, -u});
        case 1: return norm({-1.f, v, u});
        case 2: return norm({u, 1.f, -v});
        case 3: return norm({u, -1.f, v});
        case 4: return norm({u, v, 1.f});
        default: return norm({-u, v, -1.f});
        }
    };
    auto add_strip = [](float3 c, V3 d, V3 n, float rad, float3 col, float gain) {
        float w = std::exp(-(dot(d, norm(n)) * dot(d, norm(n))) / (rad * rad));
        return float3{c.x + col.x * w * gain, c.y + col.y * w * gain, c.z + col.z * w * gain};
    };
    auto add_spot = [](float3 c, V3 d, V3 a, float rad, float3 col, float gain) {
        float cd = std::clamp(dot(d, norm(a)), -1.f, 1.f);
        float ang = std::acos(cd);
        float w = std::exp(-(ang * ang) / (rad * rad));
        return float3{c.x + col.x * w * gain, c.y + col.y * w * gain, c.z + col.z * w * gain};
    };
    auto env_of = [&](int kind, V3 d) {
        if (kind == 0) return float3{0.72f, 0.74f, 0.78f};
        if (kind == 1) {
            float3 c{0.025f, 0.027f, 0.030f};
            c = add_strip(c, d, {0.05f, 1.f, 0.02f}, 0.055f, {1, 1, 1}, 4.5f);
            c = add_strip(c, d, {1.f, 0.08f, 0.15f}, 0.040f, {1, 1, 1}, 3.5f);
            c = add_strip(c, d, {0.2f, 0.15f, 1.f}, 0.045f, {0.95f, 0.97f, 1.f}, 3.0f);
            return c;
        }
        float3 c{0.006f, 0.006f, 0.008f};
        // Spots sit on the reflection directions of the six film facets.
        // Neighbors are 8 degrees apart, so a 0.045 rad spot hits one facet.
        c = add_spot(c, d, {-0.689f, -0.571f, 0.446f}, 0.045f, {1, 1, 1}, 22.f);
        c = add_spot(c, d, {-0.644f, -0.499f, 0.581f}, 0.040f, {1.f, 0.18f, 0.10f}, 16.f);
        c = add_spot(c, d, {0.710f, -0.632f, -0.312f}, 0.040f, {0.12f, 0.82f, 1.f}, 16.f);
        c = add_spot(c, d, {0.689f, -0.571f, -0.446f}, 0.045f, {1, 1, 1}, 22.f);
        c = add_spot(c, d, {0.644f, -0.499f, -0.581f}, 0.040f, {1.f, 0.70f, 0.16f}, 14.f);
        c = add_spot(c, d, {-0.142f, -0.118f, -0.983f}, 0.050f, {1, 1, 1}, 18.f);
        c = add_spot(c, d, {-0.968f, 0.100f, -0.228f}, 0.045f, {1, 1, 1}, 12.f);
        return c;
    };
    static std::vector<float> iblFace[3][6];
    Texture* iblTex[3] = {};
    IndirectLight* ibl[3] = {};
    for (int kind = 0; kind < 3; ++kind) {
        Texture* tex = Texture::Builder().width((uint32_t)kFace).height((uint32_t)kFace).levels(1)
                           .sampler(Texture::Sampler::SAMPLER_CUBEMAP)
                           .format(Texture::InternalFormat::RGBA16F)
                           .usage(Texture::Usage::DEFAULT)
                           .build(*engine);
        for (int face = 0; face < 6; ++face) {
            auto& buf = iblFace[kind][face];
            buf.resize((size_t)kFace * kFace * 4);
            for (int y = 0; y < kFace; ++y) {
                float v = ((y + 0.5f) / kFace) * 2.f - 1.f;
                for (int x = 0; x < kFace; ++x) {
                    float u = ((x + 0.5f) / kFace) * 2.f - 1.f;
                    V3 dir = face_dir(face, u, v);
                    float3 c = env_of(kind, dir);
                    if (kind == 2) {
                        c = float3{0.50f, 0.52f, 0.56f};
                        float az = std::atan2(dir.x, dir.z);
                        int sector = (int)std::floor((az + 3.14159265f) * (10.f / 6.2831853f));
                        if (sector < 0) sector = 0;
                        if (sector > 9) sector = 9;
                        if (std::fabs(dir.y) < 0.55f) {
                            c = (sector % 2 == 0) ? float3{22.f, 22.f, 23.f} : float3{0.f, 0.f, 0.f};
                        } else if (dir.y > 0.75f) {
                            c = float3{16.f, 16.f, 17.f};
                        } else if (dir.y < -0.75f) {
                            c = float3{0.f, 0.f, 0.f};
                        }
                    }
                    size_t i = ((size_t)y * kFace + x) * 4;
                    buf[i] = c.x;
                    buf[i + 1] = c.y;
                    buf[i + 2] = c.z;
                    buf[i + 3] = 1.f;
                }
            }
            tex->setImage(*engine, 0, 0, 0, (uint32_t)face, (uint32_t)kFace, (uint32_t)kFace, 1,
                          Texture::PixelBufferDescriptor(buf.data(), buf.size() * sizeof(float), Texture::Format::RGBA, Texture::Type::FLOAT));
        }
        iblTex[kind] = tex;
        ibl[kind] = IndirectLight::Builder().reflections(tex).intensity(kIbl).build(*engine);
    }
    Skybox* skyIbl = Skybox::Builder().color({0.72f / kIbl, 0.74f / kIbl, 0.78f / kIbl, 1.f}).build(*engine);
    auto use_ibl = [&](int kind) { scene->setIndirectLight(ibl[kind]); scene->setSkybox(skyIbl); };
    auto show_lit = [&](bool on) {
        scene->remove(filmAl);
        scene->remove(filmBl);
        if (on) {
            scene->addEntity(filmAl);
            scene->addEntity(filmBl);
        }
    };
    use_ibl(0);
    if (!grab("out/xcube-ibl-a.png")) return fail("ibl-a");
    use_ibl(1);
    if (!grab("out/xcube-ibl-b.png")) return fail("ibl-b");
    use_ibl(2);
    if (!grab("out/xcube-ibl-c.png")) return fail("ibl-c");
    time_beauty("IBL C unlit");
    scene->remove(filmAf);
    scene->remove(filmBf);
    if (!grab("out/xcube-ibl-c-glass.png")) return fail("ibl-glass");
    time_beauty("IBL C glass");
    show_lit(true);
    if (!grab("out/xcube-ibl-c-film.png")) return fail("ibl-film");
    time_beauty("IBL C lit");
    scene->addEntity(echoE);
    echoMi->setParameter("bounces", 1.f);
    if (!grab("out/xcube-ibl-c-g1.png")) return fail("ibl-g1");
    time_beauty("IBL C G1");
    echoMi->setParameter("bounces", 2.f);
    if (!grab("out/xcube-ibl-c-g2.png")) return fail("ibl-g2");
    time_beauty("IBL C G2");
    scene->remove(echoE);
    V3 focus{0.f, 0.08f, 0.f};
    V3 eye0{0.55f, 0.78f, 0.70f};
    V3 rel = eye0 - focus;
    float camDist = std::sqrt(dot(rel, rel));
    float baseYaw = std::atan2(rel.x, rel.z);
    float basePitch = std::asin(std::clamp(rel.y / camDist, -1.f, 1.f));
    for (int f = 0; f < 24; ++f) {
        float u = (float)f / 23.f;
        float yaw = baseYaw + 0.262f * std::sin(6.283185f * u);
        float pitch = basePitch + 0.140f * std::sin(6.283185f * u);
        float cp = std::cos(pitch), sp = std::sin(pitch);
        V3 eye = focus + V3{camDist * cp * std::sin(yaw), camDist * sp, camDist * cp * std::cos(yaw)};
        camera->lookAt(float3{eye.x, eye.y, eye.z}, float3{focus.x, focus.y, focus.z}, float3{0, 1, 0});
        char path[64];
        std::snprintf(path, sizeof(path), "out/xcube-ibl-spin-%02d.png", f);
        if (!grab(path)) return fail("ibl-spin");
    }
    camera->lookAt(float3{0.55f, 0.78f, 0.70f}, float3{0, 0.04f, 0}, float3{0, 1, 0});
    show_lit(false);
    std::vector<float> gem_v;
    std::vector<uint32_t> gem_i;
    {
        const float s = 0.2f;
        V3 toCam = norm({0.50f, 0.35f, 0.62f});
        V3 side = norm(cross({0.f, 1.f, 0.f}, toCam));
        V3 up2 = cross(toCam, side);
        const V3 c0{0.f, 0.2f, 0.f};
        V3 v[6] = {c0 + toCam * s, c0 - toCam * s, c0 + side * s, c0 - side * s, c0 + up2 * s, c0 - up2 * s};
        int id[8][3] = {{0, 2, 4}, {0, 4, 3}, {0, 3, 5}, {0, 5, 2}, {1, 4, 2}, {1, 3, 4}, {1, 5, 3}, {1, 2, 5}};
        auto tri = [&](V3 a, V3 b, V3 c) {
            V3 n = cross(b - a, c - a);
            V3 mid = (a + b + c) * (1.f / 3.f);
            if (dot(n, mid - c0) < 0.f) {
                std::swap(b, c);
                n = n * -1.f;
            }
            V3 T = norm(b - a);
            uint32_t base = (uint32_t)(gem_v.size() / 7);
            push_vert(gem_v, a, T, n);
            push_vert(gem_v, b, T, n);
            push_vert(gem_v, c, T, n);
            gem_i.insert(gem_i.end(), {base, base + 1, base + 2});
        };
        for (int i = 0; i < 8; ++i) tri(v[id[i][0]], v[id[i][1]], v[id[i][2]]);
    }
    VertexBuffer* gemVB = VertexBuffer::Builder().vertexCount((uint32_t)(gem_v.size() / 7)).bufferCount(1)
                              .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                              .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                              .build(*engine);
    gemVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(gem_v.data(), gem_v.size() * sizeof(float)));
    IndexBuffer* gemIB = IndexBuffer::Builder().indexCount((uint32_t)gem_i.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    gemIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(gem_i.data(), gem_i.size() * sizeof(uint32_t)));
    const uint32_t ntri = (uint32_t)(gem_i.size() / 3);
    static std::vector<float> triBuf;
    triBuf.assign((size_t)ntri * 3 * 4, 0.f);
    for (uint32_t t = 0; t < ntri; ++t) {
        for (int k = 0; k < 3; ++k) {
            uint32_t vi = gem_i[t * 3 + k];
            size_t o = ((size_t)t * 3 + k) * 4;
            triBuf[o] = gem_v[vi * 7];
            triBuf[o + 1] = gem_v[vi * 7 + 1];
            triBuf[o + 2] = gem_v[vi * 7 + 2];
            triBuf[o + 3] = 1.f;
        }
    }
    Texture* triTex = Texture::Builder().width(ntri * 3).height(1).levels(1)
                          .sampler(Texture::Sampler::SAMPLER_2D)
                          .format(Texture::InternalFormat::RGBA16F)
                          .usage(Texture::Usage::DEFAULT)
                          .build(*engine);
    triTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(triBuf.data(), triBuf.size() * sizeof(float), Texture::Format::RGBA, Texture::Type::FLOAT));
    MaterialInstance* dreiMi = dreiMat->createInstance();
    TextureSampler dreiNearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    dreiMi->setParameter("tris", triTex, dreiNearest);
    dreiMi->setParameter("env", iblTex[2], TextureSampler(TextureSampler::MagFilter::LINEAR));
    MaterialInstance* octaMi = octaMat->createInstance();
    octaMi->setParameter("tris", triTex, dreiNearest);
    octaMi->setParameter("center", float3{0.f, 0.2f, 0.f});
    octaMi->setParameter("extent", extent);
    octaMi->setParameter("grid", (float)GRID);
    octaMi->setParameter("bands", (float)NW);
    octaMi->setParameter("ior", 2.4f);
    octaMi->setParameter("dispersion", dispersion);
    octaMi->setParameter("bounces", 3.f);
    octaMi->setParameter("triCount", (float)ntri);
    octaMi->setParameter("hitW", (float)hitW);
    octaMi->setParameter("hitH", (float)hitH);
    dreiMi->setParameter("eye", float3{0.55f, 0.78f, 0.70f});
    dreiMi->setParameter("triCount", (float)ntri);
    dreiMi->setParameter("bounces", 3.f);
    dreiMi->setParameter("ior", 2.4f);
    dreiMi->setParameter("dispersion", dispersion);
    dreiMi->setParameter("aberration", 0.045f);
    Skybox* skyGem = Skybox::Builder().color({0.012f, 0.016f, 0.028f, 1.f}).build(*engine);
    scene->setIndirectLight(nullptr);
    scene->setSkybox(skyGem);
    floorMi->setParameter("color", float4{0.03f, 0.034f, 0.05f, 1.f});
    dreiMi->setParameter("chroma", 1.f);
    dreiMi->setParameter("fresnel", 0.f);
    dreiMi->setParameter("grad", 0.f);
    dreiMi->setParameter("envScale", 1.f);
    utils::Entity cubeDrei = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, gemVB, gemIB)
        .material(0, dreiMi).culling(false).castShadows(true).receiveShadows(false)
        .build(*engine, cubeDrei);
    if (!grab("out/xcube-drei-a.png")) return fail("drei-a");
    time_beauty("drei A glass");
    scene->remove(cubeBevel);
    {
        const int RF = 128;
        Texture* roomCube = Texture::Builder().width((uint32_t)RF).height((uint32_t)RF).levels(1)
                                .sampler(Texture::Sampler::SAMPLER_CUBEMAP)
                                .format(Texture::InternalFormat::RGBA16F)
                                .usage(Texture::Usage::DEFAULT)
                                .build(*engine);
        auto face_dir = [](int face, float u, float v) {
            switch (face) {
            case 0: return norm({1.f, v, -u});
            case 1: return norm({-1.f, v, u});
            case 2: return norm({u, 1.f, -v});
            case 3: return norm({u, -1.f, v});
            case 4: return norm({u, v, 1.f});
            default: return norm({-u, v, -1.f});
            }
        };
        static std::vector<float> roomFace[6];
        auto fill_env = [&](auto color_of) {
            for (int face = 0; face < 6; ++face) {
                roomFace[face].assign((size_t)RF * RF * 4, 0.f);
                for (int y = 0; y < RF; ++y) {
                    float v = ((y + 0.5f) / RF) * 2.f - 1.f;
                    for (int x = 0; x < RF; ++x) {
                        float u = ((x + 0.5f) / RF) * 2.f - 1.f;
                        float3 c = color_of(face_dir(face, u, v));
                        size_t i = ((size_t)y * RF + x) * 4;
                        roomFace[face][i] = c.x;
                        roomFace[face][i + 1] = c.y;
                        roomFace[face][i + 2] = c.z;
                        roomFace[face][i + 3] = 1.f;
                    }
                }
                roomCube->setImage(*engine, 0, 0, 0, (uint32_t)face, (uint32_t)RF, (uint32_t)RF, 1,
                                   Texture::PixelBufferDescriptor(roomFace[face].data(), roomFace[face].size() * sizeof(float), Texture::Format::RGBA, Texture::Type::FLOAT));
            }
        };
        fill_env([](V3 d) { return (d.x > 0.f) ? float3{36.f, 36.f, 38.f} : float3{0.f, 0.f, 0.f}; });
        dreiMi->setParameter("env", roomCube, TextureSampler(TextureSampler::MagFilter::LINEAR));
        scene->remove(floor);
        scene->addEntity(cubeDrei);
        dreiMi->setParameter("chroma", 0.f);
        dreiMi->setParameter("fresnel", 0.f);
        dreiMi->setParameter("bounces", 3.f);
        auto look_gem = [&](float yaw) {
            float cy = std::cos(yaw), sy = std::sin(yaw);
            V3 off{0.62f, 0.22f, 0.38f};
            V3 eye{cy * off.x + sy * off.z, 0.2f + off.y, -sy * off.x + cy * off.z};
            camera->lookAt(float3{eye.x, eye.y, eye.z}, float3{0.f, 0.2f, 0.f}, float3{0, 1, 0});
            dreiMi->setParameter("eye", float3{eye.x, eye.y, eye.z});
        };
        look_gem(0.f);
        if (!grab("out/xcube-octa.png")) return fail("octa");
        look_gem(0.45f);
        if (!grab("out/xcube-octa-l.png")) return fail("octa-l");
        look_gem(-0.45f);
        if (!grab("out/xcube-octa-r.png")) return fail("octa-r");
        look_gem(0.f);
        time_beauty("drei B bounce3");
        dreiMi->setParameter("bounces", 2.f);
        time_beauty("drei B bounce2");
        dreiMi->setParameter("bounces", 1.f);
        time_beauty("drei B bounce1");
        dreiMi->setParameter("bounces", 3.f);
        dreiMi->setParameter("fresnel", 1.f);
        time_beauty("drei fresnel same");
        dreiMi->setParameter("fresnel", 0.f);
        auto wrap_pi = [](float a) {
            const float pi = 3.14159265f;
            while (a > pi) a -= 2.f * pi;
            while (a < -pi) a += 2.f * pi;
            return a;
        };
        fill_env([&](V3 d) {
            float az = std::atan2(d.x, d.z);
            float3 c{0.22f, 0.24f, 0.28f};
            if (d.x > 0.1f && d.y > -0.35f) c = float3{1.4f, 1.5f, 1.7f};
            const float bars[5] = {-2.2f, -1.05f, -0.15f, 0.85f, 2.05f};
            for (float b : bars) {
                if (std::fabs(wrap_pi(az - b)) < 0.045f && d.y > -0.75f && d.y < 0.92f) c = float3{36.f, 38.f, 42.f};
            }
            return c;
        });
        dreiMi->setParameter("fresnel", 1.f);
        if (!grab("out/xcube-octa-studio.png")) return fail("octa-studio");
        time_beauty("drei studio fresnel");
        dreiMi->setParameter("fresnel", 0.f);
    }
    auto shoot_eye = [&](V3 eye, const char* path) {
        camera->lookAt(float3{eye.x, eye.y, eye.z}, float3{0.f, 0.04f, 0.f}, float3{0, 1, 0});
        dreiMi->setParameter("eye", float3{eye.x, eye.y, eye.z});
        return grab(path);
    };
    V3 eyeL = focus + V3{camDist * std::cos(basePitch) * std::sin(baseYaw + 0.26f), camDist * std::sin(basePitch), camDist * std::cos(basePitch) * std::cos(baseYaw + 0.26f)};
    V3 eyeR = focus + V3{camDist * std::cos(basePitch) * std::sin(baseYaw - 0.26f), camDist * std::sin(basePitch), camDist * std::cos(basePitch) * std::cos(baseYaw - 0.26f)};
    if (!shoot_eye(eyeL, "out/xcube-drei-b-l.png")) return fail("drei-bl");
    if (!shoot_eye(eyeR, "out/xcube-drei-b-r.png")) return fail("drei-br");
    V3 eyeFar = focus + (eye0 - focus) * 1.8f;
    V3 eyeNear = focus + (eye0 - focus) * 0.62f;
    camera->lookAt(float3{eyeFar.x, eyeFar.y, eyeFar.z}, float3{0.f, 0.04f, 0.f}, float3{0, 1, 0});
    dreiMi->setParameter("eye", float3{eyeFar.x, eyeFar.y, eyeFar.z});
    time_beauty("drei B far");
    camera->lookAt(float3{eyeNear.x, eyeNear.y, eyeNear.z}, float3{0.f, 0.04f, 0.f}, float3{0, 1, 0});
    dreiMi->setParameter("eye", float3{eyeNear.x, eyeNear.y, eyeNear.z});
    time_beauty("drei B near");
    camera->lookAt(float3{0.55f, 0.78f, 0.70f}, float3{0, 0.04f, 0}, float3{0, 1, 0});
    scene->remove(cubeDrei);
    scene->addEntity(floor);
    scene->addEntity(cubeBevel);
    std::printf("drei tris %u ior 2.4 bounces 3  no usampler BVH\n", ntri);
    scene->setIndirectLight(nullptr);
    scene->setSkybox(sky);
    floorMi->setParameter("color", float4{0.86f, 0.85f, 0.84f, 1});
    show_lit(false);
    {
        const float s = 0.16f;
        const V3 c0{0.f, 0.22f, 0.f};
        auto rot = [](V3 p) {
            float cx = std::cos(0.55f), sx = std::sin(0.55f);
            V3 a{p.x, cx * p.y - sx * p.z, sx * p.y + cx * p.z};
            float cy = std::cos(0.4f), sy = std::sin(0.4f);
            return V3{cy * a.x + sy * a.z, a.y, -sy * a.x + cy * a.z};
        };
        V3 v[6] = {c0 + rot({0, s, 0}), c0 + rot({0, -s, 0}), c0 + rot({s, 0, 0}), c0 + rot({-s, 0, 0}), c0 + rot({0, 0, s}), c0 + rot({0, 0, -s})};
        int id[8][3] = {{0, 2, 4}, {0, 4, 3}, {0, 3, 5}, {0, 5, 2}, {1, 4, 2}, {1, 3, 4}, {1, 5, 3}, {1, 2, 5}};
        std::vector<float> cv;
        std::vector<uint32_t> ci;
        for (int i = 0; i < 8; ++i) {
            V3 a = v[id[i][0]], b = v[id[i][1]], c = v[id[i][2]];
            V3 n = cross(b - a, c - a);
            if (dot(n, (a + b + c) * (1.f / 3.f) - c0) < 0.f) {
                std::swap(b, c);
                n = n * -1.f;
            }
            V3 T = norm(b - a);
            uint32_t base = (uint32_t)(cv.size() / 7);
            push_vert(cv, a, T, n);
            push_vert(cv, b, T, n);
            push_vert(cv, c, T, n);
            ci.insert(ci.end(), {base, base + 1, base + 2});
        }
        VertexBuffer* crystalVB = VertexBuffer::Builder().vertexCount((uint32_t)(cv.size() / 7)).bufferCount(1)
                                      .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                                      .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                                      .build(*engine);
        crystalVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(cv.data(), cv.size() * sizeof(float)));
        IndexBuffer* crystalIB = IndexBuffer::Builder().indexCount((uint32_t)ci.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        crystalIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(ci.data(), ci.size() * sizeof(uint32_t)));
        utils::Entity crystal = utils::EntityManager::get().create();
        RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, crystalVB, crystalIB)
            .material(0, glass).culling(false).castShadows(true).receiveShadows(false)
            .build(*engine, crystal);
        glass->setParameter("thickness", 0.32f);
        glass->setParameter("dispersion", 48.f);
        scene->remove(cube);
        scene->remove(cubeBevel);
        scene->addEntity(crystal);
        view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
        view->setPostProcessingEnabled(true);
        view->setScreenSpaceRefractionEnabled(true);
        camera->setExposure(2.8f, 1.f / 180.f, 100.f);
        camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);
        camera->lookAt(float3{0.42f, 0.5f, 0.48f}, float3{0.f, 0.18f, 0.f}, float3{0, 1, 0});
        if (!grab("out/xcube-crystal.png")) return fail("crystal");
        scene->remove(crystal);
        glass->setParameter("thickness", half * 2.f);
        glass->setParameter("dispersion", dispersion);
        camera->lookAt(float3{0.55f, 0.78f, 0.70f}, float3{0, 0.04f, 0}, float3{0, 1, 0});
    }
    {
        V3 D = sun0;
        V3 up = std::fabs(D.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0};
        V3 U = norm(cross(up, D));
        V3 V = cross(D, U);
        octaMi->setParameter("dir", float3{D.x, D.y, D.z});
        octaMi->setParameter("tangent", float3{U.x, U.y, U.z});
        octaMi->setParameter("bitangent", float3{V.x, V.y, V.z});
        rm.setMaterialInstanceAt(rm.getInstance(hitE), 0, octaMi);
        if (!draw_hits() || !draw_paths() || !draw_atrous()) return fail("octa-caustic");
        rm.setMaterialInstanceAt(rm.getInstance(hitE), 0, hitMi);
        auto n_at = [](float nm, float disp) {
            float offset = 0.70795215f + (nm - 486.1f) / (656.3f - 486.1f) * (-0.29204785f - 0.70795215f);
            float P = disp / 20.f;
            return std::max(1.05f, 2.4f + P * (2.4f - 1.f) * offset);
        };
        const float dispAbbe = 20.f / 55.f;
        std::printf("n ior 2.4 dispersion 36     650 %.4f  550 %.4f  450 %.4f\n", n_at(650.f, 36.f), n_at(550.f, 36.f), n_at(450.f, 36.f));
        std::printf("n ior 2.4 dispersion 20/55  650 %.4f  550 %.4f  450 %.4f\n", n_at(650.f, dispAbbe), n_at(550.f, dispAbbe), n_at(450.f, dispAbbe));
        auto dump_hdr = [&](RenderTarget* rt, int w, int h, const char* path) -> bool {
            std::vector<float> px((size_t)w * h * 4, 0.f);
            if (!open()) return false;
            renderer->readPixels(rt, 0, 0, (uint32_t)w, (uint32_t)h,
                                 backend::PixelBufferDescriptor(px.data(), px.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
            renderer->endFrame();
            engine->flushAndWait();
            int zero = 0;
            float maxv = 0.f;
            for (int i = 0; i < w * h; ++i) {
                float m = std::max(px[(size_t)i * 4], std::max(px[(size_t)i * 4 + 1], px[(size_t)i * 4 + 2]));
                if (m > maxv) maxv = m;
                if (m < 1e-4f) zero++;
            }
            std::printf("%s  max %.4f  zero %d / %d\n", path, maxv, zero, w * h);
            std::vector<uint8_t> img((size_t)w * h * 4);
            float denom = std::log1p(std::max(maxv, 1e-6f));
            for (int i = 0; i < w * h; ++i) {
                float r = px[(size_t)i * 4], g = px[(size_t)i * 4 + 1], b = px[(size_t)i * 4 + 2];
                float m = std::max(r, std::max(g, b));
                uint8_t* o = img.data() + (size_t)i * 4;
                if (m < 1e-4f) {
                    o[0] = 180; o[1] = 0; o[2] = 180; o[3] = 255;
                } else {
                    float s = std::log1p(m) / denom;
                    float L = r + g + b;
                    o[0] = (uint8_t)std::clamp(r / L * s * 255.f, 0.f, 255.f);
                    o[1] = (uint8_t)std::clamp(g / L * s * 255.f, 0.f, 255.f);
                    o[2] = (uint8_t)std::clamp(b / L * s * 255.f, 0.f, 255.f);
                    o[3] = 255;
                }
            }
            return write_png(path, w, h, img.data());
        };
        if (!dump_hdr(mapRT, MAP, MAP, "out/xcube-caustic-map.png")) return fail("caustic-map");
        if (!dump_hdr(filtRT, MAP, MAP, "out/xcube-caustic-filt.png")) return fail("caustic-filt");
        auto face_dir = [](int face, float u, float v) {
            switch (face) {
            case 0: return norm({1.f, v, -u});
            case 1: return norm({-1.f, v, u});
            case 2: return norm({u, 1.f, -v});
            case 3: return norm({u, -1.f, v});
            case 4: return norm({u, v, 1.f});
            default: return norm({-u, v, -1.f});
            }
        };
        auto wrap_pi = [](float a) {
            const float pi = 3.14159265f;
            while (a > pi) a -= 2.f * pi;
            while (a < -pi) a += 2.f * pi;
            return a;
        };
        auto slit = [&](V3 d) {
            float az = std::atan2(d.x, d.z);
            float3 c{0.f, 0.f, 0.f};
            const float azs[12] = {-2.7f, -2.2f, -1.7f, -1.2f, -0.75f, -0.3f, 0.15f, 0.55f, 1.0f, 1.45f, 1.9f, 2.4f};
            for (int k = 0; k < 12; ++k) {
                float w = (k % 2 == 0) ? 0.028f : 0.04f;
                if (std::fabs(wrap_pi(az - azs[k])) < w && d.y > -0.85f && d.y < 0.92f) {
                    if (k % 4 == 1) c = float3{46.f, 16.f, 8.f};
                    else if (k % 4 == 3) c = float3{8.f, 34.f, 42.f};
                    else c = float3{42.f, 44.f, 48.f};
                }
            }
            return c;
        };
        static std::vector<std::vector<float>> envKeep;
        envKeep.clear();
        auto upload_env = [&](int RF, bool withMips) -> Texture* {
            int levels = 1;
            if (withMips) {
                for (int n = RF; n > 1; n /= 2) levels++;
            }
            Texture* tex = Texture::Builder().width((uint32_t)RF).height((uint32_t)RF).levels((uint8_t)levels)
                               .sampler(Texture::Sampler::SAMPLER_CUBEMAP)
                               .format(Texture::InternalFormat::RGBA16F)
                               .usage(Texture::Usage::DEFAULT)
                               .build(*engine);
            for (int face = 0; face < 6; ++face) {
                envKeep.emplace_back((size_t)RF * RF * 4);
                auto& buf = envKeep.back();
                for (int y = 0; y < RF; ++y) {
                    float fv = ((y + 0.5f) / RF) * 2.f - 1.f;
                    for (int x = 0; x < RF; ++x) {
                        float fu = ((x + 0.5f) / RF) * 2.f - 1.f;
                        float3 c = slit(face_dir(face, fu, fv));
                        size_t i = ((size_t)y * RF + x) * 4;
                        buf[i] = c.x; buf[i + 1] = c.y; buf[i + 2] = c.z; buf[i + 3] = 1.f;
                    }
                }
                tex->setImage(*engine, 0, 0, 0, (uint32_t)face, (uint32_t)RF, (uint32_t)RF, 1,
                              Texture::PixelBufferDescriptor(buf.data(), buf.size() * sizeof(float), Texture::Format::RGBA, Texture::Type::FLOAT));
                if (!withMips) continue;
                int n = RF;
                for (int level = 1; level < levels; ++level) {
                    int m = n / 2;
                    envKeep.emplace_back((size_t)m * m * 4);
                    auto& src = envKeep[envKeep.size() - 2];
                    auto& dst = envKeep.back();
                    for (int y = 0; y < m; ++y) {
                        for (int x = 0; x < m; ++x) {
                            for (int c = 0; c < 4; ++c) {
                                float s = src[((size_t)(y * 2) * n + x * 2) * 4 + c];
                                s += src[((size_t)(y * 2) * n + x * 2 + 1) * 4 + c];
                                s += src[((size_t)(y * 2 + 1) * n + x * 2) * 4 + c];
                                s += src[((size_t)(y * 2 + 1) * n + x * 2 + 1) * 4 + c];
                                dst[((size_t)y * m + x) * 4 + c] = s * 0.25f;
                            }
                        }
                    }
                    tex->setImage(*engine, (size_t)level, 0, 0, (uint32_t)face, (uint32_t)m, (uint32_t)m, 1,
                                  Texture::PixelBufferDescriptor(dst.data(), dst.size() * sizeof(float), Texture::Format::RGBA, Texture::Type::FLOAT));
                    n = m;
                }
            }
            engine->flushAndWait();
            return tex;
        };
        Texture* env128 = upload_env(128, false);
        Texture* env1024 = upload_env(1024, false);
        Texture* envMip = upload_env(1024, true);
        TextureSampler envLinear(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
        TextureSampler envMipSamp(TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR, TextureSampler::MagFilter::LINEAR);
        dreiMi->setParameter("env", env128, envLinear);
        dreiMi->setParameter("chroma", 1.f);
        dreiMi->setParameter("dispersion", dispersion);
        dreiMi->setParameter("fresnel", 0.f);
        dreiMi->setParameter("grad", 0.f);
        dreiMi->setParameter("bounces", 3.f);
        scene->remove(filter);
        scene->remove(splat);
        scene->remove(hitE);
        scene->remove(cube);
        scene->remove(cubeBevel);
        scene->addEntity(floor);
        scene->addEntity(cubeDrei);
        scene->addEntity(sun_e);
        scene->setIndirectLight(nullptr);
        scene->setSkybox(skyGem);
        floorMi->setParameter("color", float4{0.90f, 0.89f, 0.88f, 1});
        view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
        view->setPostProcessingEnabled(true);
        view->setScreenSpaceRefractionEnabled(false);
        view->setShadowType(View::ShadowType::PCF);
        view->setShadowingEnabled(true);
        BloomOptions lean = bloom;
        lean.strength = 0.08f;
        view->setBloomOptions(lean);
        camera->setExposure(2.8f, 1.f / 180.f, 100.f);
        camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);
        float3 eye{0.62f, 0.78f, 0.70f};
        camera->lookAt(eye, float3{0.f, 0.12f, 0.f}, float3{0, 1, 0});
        dreiMi->setParameter("eye", eye);
        if (!grab("out/xcube-gem.png")) return fail("gem");
        view->setShadowingEnabled(false);
        if (!grab("out/xcube-gem-noshadow.png")) return fail("gem-noshadow");
        view->setShadowingEnabled(true);
        dreiMi->setParameter("env", env1024, envLinear);
        if (!grab("out/xcube-gem-a.png")) return fail("gem-a");
        dreiMi->setParameter("env", env128, envLinear);
        dreiMi->setParameter("dispersion", dispAbbe);
        if (!grab("out/xcube-gem-b.png")) return fail("gem-b");
        dreiMi->setParameter("dispersion", dispersion);
        dreiMi->setParameter("env", envMip, envMipSamp);
        dreiMi->setParameter("grad", 1.f);
        if (!grab("out/xcube-gem-c.png")) return fail("gem-c");
        dreiMi->setParameter("grad", 0.f);
        dreiMi->setParameter("env", env128, envLinear);
        scene->remove(floor);
        time_beauty("gem only");
        scene->addEntity(floor);
        scene->remove(cubeDrei);
        scene->setSkybox(sky);
    }
    std::printf("ibl intensity %.0f  reflection cameras 0\n", kIbl);
    scene->remove(cubeBevel);
    scene->remove(filmAf);
    scene->remove(filmBf);
    scene->addEntity(cube);
    show_films(true);
    if (!bake(0.f, 0.f)) return fail("flat");

    float dist = std::sqrt(0.55f * 0.55f + 0.70f * 0.70f);
    for (int f = 0; f < 12; ++f) {
        float yaw = 6.283185f * (float)f / 12.f;
        camera->lookAt(float3{dist * std::sin(yaw), 0.78f, dist * std::cos(yaw)}, float3{0, 0.08f, 0}, float3{0, 1, 0});
        char path[64];
        std::snprintf(path, sizeof(path), "out/xcube-rw-orbit-%02d.png", f);
        if (!grab(path)) return fail("orbit");
    }
    std::printf("PASS xcube-gpu\n");
    return 0;
}
