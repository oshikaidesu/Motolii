// Several glass solids of different shapes on one floor, each throwing its own sharp light.
// Same machinery as glass_beam.mm (see BEAM_GLASS.md), with:
//   * shapes: any convex flat-faced solid from beam_glass.h (boxes of any proportion, prisms, pyramids ...),
//     each with the chamfer real cut glass has, each drawn by glass_solid.mat from its own list of faces;
//   * many of them: every solid is traced on its own thread and all the light goes into the same buffers.
//   * together: light that leaves one solid and meets another goes on through it, a solid standing in another's
//     sun takes that light first, and glass is seen through glass (glass_solid.mat follows each sight line on
//     through the other solids, from a small table of all their faces that is refilled every frame).
// The light is beam::trace_cheap: plausible, not exact (one ray per piece of each lit face decides where the
// whole piece goes). EXACT=1 uses the exact, slow trace instead, to compare.
//
// Usage: ./out/glass_scene [name]       writes out/glass-scene-<name>.png
// Timing: PACE=1 [PACE_HZ=60] [PACE_FRAMES=600] turns and slides every solid and swings the sun every frame.
// TUMBLE=1 throws the solids about (any turn, off the floor), T=<time> picks the moment of the still picture.
// Environment: CHAMFER, DISP, SUNR, BANDS, SUN="x y z", IBL, APERTURE, CAM="ex ey ez tx ty tz fov", ONLY=<kind>, COUNT=<n>, THREADS=<n>.
// Balls are drawn from one shared shape on the GPU (ball_light.mat); BALLCPU=1 works the same light out on the CPU, to compare.
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
#include <thread>
#include <vector>

using namespace filament;
using namespace filament::math;

static const int W = 1920, H = 1080;
static const uint32_t MAX_LIGHT_VERTS = 1500000;

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
struct Item {
    const char* kind;
    beam::Solid local;                 // the solid in its own space (chamfer included); the mesh has the same shape
    beam::V3 down;                     // which of its directions points at the floor (how it rests)
    float size;                        // uniform scale
    float x, z, yaw;                   // where it stands, and its turn about the vertical
    float spin, drift, phase;          // only for the timing test
    MaterialInstance* glass = nullptr;
    utils::Entity entity;
    std::vector<float> tri, outline;   // this frame's light and shadow outline
    double ms = 0;
};
// Where the solid's own x, y, z axes go: first `down` is brought to the floor, then the whole is turned by yaw.
static void item_axes(const Item& it, float yaw, beam::V3 R[3]) {
    using beam::V3;
    const V3 a = beam::normalize(it.down), b{0, -1, 0};
    V3 axis = beam::cross(a, b);
    const double sn = beam::length(axis), cs = beam::dot(a, b);
    V3 e[3] = {{1, 0, 0}, {0, 1, 0}, {0, 0, 1}};
    if (sn > 1e-9) {
        axis = axis * (1.0 / sn);
        for (V3& v : e) v = v * cs + beam::cross(axis, v) * sn + axis * (beam::dot(axis, v) * (1.0 - cs));  // Rodrigues
    } else if (cs < 0) {
        for (V3& v : e) v = {v.x, -v.y, -v.z};  // upside down: half a turn about x
    }
    const double c = std::cos(yaw), s = std::sin(yaw);
    for (int k = 0; k < 3; ++k) R[k] = {c * e[k].x + s * e[k].z, e[k].y, -s * e[k].x + c * e[k].z};
}

int main(int argc, char** argv) {
    const std::string name = argc > 1 ? argv[1] : "first";
    const float nd = 1.5168f;
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
    Material* ballMat = mat("ball_light.filamat");
    Material* coneMat = mat("cone_light.filamat");
    Material* coneWalkMat = mat("cone_walk.filamat");
    Material* flatMat = mat("flat_light.filamat");
    Material* flatWalkMat = mat("flat_walk.filamat");
    Material* outlineMat = mat("outline.filamat");
    if (!lightMat || !floorMat || !glassMat || !shadowMat || !softenMat || !ballMat || !coneMat || !coneWalkMat || !flatMat || !flatWalkMat || !outlineMat)
        return fail("material");

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
    // ---- the light of every ball: ONE mesh, drawn once per ball and colour step with the ball's middle and radius
    // read from a small table (ball_light.mat). Renderables of 1, 2, 4 ... balls are switched on and off so that
    // exactly as many are drawn as there are balls.
    static std::vector<float> discRS;  // ring, sector per corner
    static std::vector<uint32_t> discI;
    beam::ball_disc(discRS, discI);
    static std::vector<float> discV;
    for (size_t i = 0; i + 1 < discRS.size(); i += 2) discV.insert(discV.end(), {discRS[i], discRS[i + 1], 0.f});
    VertexBuffer* discVB = VertexBuffer::Builder().vertexCount((uint32_t)(discV.size() / 3)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12).build(*engine);
    discVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(discV.data(), discV.size() * sizeof(float)));
    IndexBuffer* discIB = IndexBuffer::Builder().indexCount((uint32_t)discI.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    discIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(discI.data(), discI.size() * sizeof(uint32_t)));
    const int BALL_TABLE = 128, BALL_SETS = 7, BALL_STEPS_MOST = 12;
    Texture* ballTex = Texture::Builder().width(BALL_TABLE).height(1).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
    int ballStepCount = 0;  // colour steps per ball; fixed once the glass is known
    Texture* ballRows = nullptr;  // what a ray does inside a ball, per step and ring: the same for every ball, and it depends on the glass alone
    {
        beam::Params par0;
        par0.bands = bands;
        par0.glass.nd = nd;
        par0.glass.exaggerate = disp;
        const beam::BallShape shape0 = beam::ball_shape(par0);
        ballStepCount = (int)shape0.steps.size();
        if (ballStepCount > BALL_STEPS_MOST) return fail("too many colour steps for ball_light.mat");
        static std::vector<float> rowTable;
        int tw = 0, th = 0;
        beam::ball_table(shape0, rowTable, tw, th);
        ballRows = Texture::Builder().width((uint32_t)tw).height((uint32_t)th).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
        ballRows->setImage(*engine, 0, Texture::PixelBufferDescriptor(rowTable.data(), rowTable.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
    }
    MaterialInstance* ballMi[BALL_SETS];
    utils::Entity ballE[BALL_SETS];
    for (int b = 0; b < BALL_SETS; ++b) {
        ballMi[b] = ballMat->createInstance();
        ballMi[b]->setParameter("balls", ballTex, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST));
        ballMi[b]->setParameter("floorY", 0.f);
        ballMi[b]->setParameter("rows", ballRows, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST));
        ballMi[b]->setParameter("flatter", (float)beam::BALL_FLAT);
        ballMi[b]->setParameter("reach", (float)beam::BALL_REACH);
        ballMi[b]->setParameter("cap", (float)beam::BALL_CAP);
        ballMi[b]->setParameter("stepCount", (float)ballStepCount);
        ballE[b] = em.create();
        RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, discVB, discIB)
            .instances((size_t)(1 << b) * (size_t)ballStepCount)
            .material(0, ballMi[b]).culling(false).castShadows(false).receiveShadows(false).layerMask(0xFF, 0x00).build(*engine, ballE[b]);
        sLight->addEntity(ballE[b]);
    }
    // ---- the light of every cone: likewise one mesh, drawn once per cone, colour step and way (cone_light.mat).
    // What becomes of each corner is worked out once, in a pass of its own, into a small float texture (cone_walk.mat).
    // (set up further down, once the solids are known: the texture has one line per cone, colour step and way)
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

    // ---- the solids
    std::vector<Item> items;
    {
        using namespace beam;
        const double c = chamfer;  // flat width, in the solids' own units (they are about 1 across)
        auto add = [&](const char* kind, Solid s, V3 down, float size) {
            Item it{};
            it.kind = kind;
            it.local = std::move(s);
            it.down = down;
            it.size = size;
            items.push_back(std::move(it));
        };
        Solid tri3 = chamfered(prism(3, 0.55, 0.60), c), hex6 = chamfered(prism(6, 0.50, 0.38), c);
        add("cube", chamfered(box(0.5, 0.5, 0.5), c), {0, -1, 0}, 0.30f);
        add("slab", chamfered(box(0.80, 0.16, 0.55), c), {0, -1, 0}, 0.30f);
        add("prism", tri3, tri3.planes[0].n, 0.30f);                     // lying on a side: the classic prism
        add("pillar", chamfered(box(0.26, 0.75, 0.26), c), {0, -1, 0}, 0.30f);
        add("hexagon", hex6, {0, -1, 0}, 0.30f);
        add("bar", chamfered(box(0.95, 0.20, 0.20), c), {0, -1, 0}, 0.30f);
        add("pyramid", chamfered(pyramid(4, 0.62, 0.70), c), {0, -1, 0}, 0.30f);
        add("wedge", chamfered(prism(3, 0.50, 0.30), c), {0, -1, 0}, 0.30f);  // a standing triangular prism
        Solid oct = chamfered(octahedron(0.62), c);
        add("octahedron", oct, oct.planes[0].n, 0.30f);
        add("ball", ball(0.5), {0, -1, 0}, 0.30f);
        add("rod", rod(0.30, 0.70), {1, 0, 0}, 0.30f);       // a round bar lying on its side
        add("column", rod(0.34, 0.55), {0, -1, 0}, 0.30f);   // the same shape standing
        add("cone", cone(0.45, 0.95), {0, -1, 0}, 0.30f);
        add("conelying", cone(0.45, 0.95), {1, 0.45 / 0.95, 0}, 0.30f);  // the same shape on its side
        if (const char* only = std::getenv("ONLY")) {
            std::vector<Item> keep;
            for (Item& it : items)
                if (std::strcmp(it.kind, only) == 0) keep.push_back(std::move(it));
            items = std::move(keep);
        }
        const int want = (int)env_f("COUNT", (float)items.size());
        const size_t kinds = items.size();
        if (kinds == 0) return fail("no such kind");
        for (int i = (int)kinds; i < want; ++i) items.push_back(items[(size_t)i % kinds]);  // more of the same kinds
        if ((int)items.size() > want) items.resize((size_t)std::max(1, want));
        // a loose grid, each turned differently (fixed numbers: the same scene every run)
        const int n = (int)items.size(), cols = (int)std::ceil(std::sqrt((double)n * 1.4));
        for (int i = 0; i < n; ++i) {
            Item& it = items[(size_t)i];
            const int cx = i % cols, cz = i / cols;
            it.x = (cx - 0.5f * (cols - 1)) * 0.62f + ((cz & 1) ? 0.18f : 0.f);
            it.z = (cz - 0.5f * ((n - 1) / cols)) * 0.62f;
            it.yaw = 0.35f + 0.83f * (float)i;
            it.spin = (i & 1) ? 1.0f : -0.7f;
            it.drift = 0.05f;
            it.phase = 1.7f * (float)i;
        }
    }
    // ---- the cones' light (see above)
    static std::vector<float> coneV;  // row, column, patch per corner
    static std::vector<uint32_t> coneI, coneITip;  // for a rod, and for a cone with its tip (nothing to draw on its top)
    beam::cone_mesh(coneV, coneI, true);
    {
        std::vector<float> same;
        beam::cone_mesh(same, coneITip, false);
    }
    VertexBuffer* coneVB = VertexBuffer::Builder().vertexCount((uint32_t)(coneV.size() / 3)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12).build(*engine);
    coneVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(coneV.data(), coneV.size() * sizeof(float)));
    IndexBuffer* coneIB = IndexBuffer::Builder().indexCount((uint32_t)coneI.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    coneIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(coneI.data(), coneI.size() * sizeof(uint32_t)));
    IndexBuffer* coneIBTip = IndexBuffer::Builder().indexCount((uint32_t)coneITip.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    coneIBTip->setBuffer(*engine, IndexBuffer::BufferDescriptor(coneITip.data(), coneITip.size() * sizeof(uint32_t)));
    const int CONE_SETS = 2, CONE_STEPS_MOST = 32;  // two renderables: the cones, then the rods (a rod is drawn as a cone without its tip)
    size_t coneRoom = 0, conesWithTip = 0;  // cones and rods in this scene; how many of them are cones
    for (const Item& it : items) {
        coneRoom += it.local.kind == beam::Solid::CONE || it.local.kind == beam::Solid::ROD;
        conesWithTip += it.local.kind == beam::Solid::CONE;
    }
    const int coneStepCount = std::min(CONE_STEPS_MOST, std::max(1, (int)env_f("CONESTEPS", (float)beam::Params().coneSteps)));
    const int coneWays = std::min(beam::CONE_WAYS, std::max(1, (int)env_f("CONEWAYS", (float)beam::CONE_WAYS)));
    const int coneLines = (int)std::max<size_t>(1, coneRoom) * coneStepCount * coneWays;
    Texture* coneTex = Texture::Builder().width((uint32_t)std::max<size_t>(1, coneRoom)).height(4).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
    Target coneWalk = make_rt((int)(coneV.size() / 3) * 2, coneLines, Texture::InternalFormat::RGBA32F);
    const float4 coneRows{(float)beam::CONE_ROWS[0], (float)beam::CONE_ROWS[1], (float)beam::CONE_ROWS[2], (float)beam::CONE_ROWS[3]};
    const float4 coneCols{(float)beam::CONE_COLS[0], (float)beam::CONE_COLS[1], (float)beam::CONE_COLS[2], (float)beam::CONE_COLS[3]};
    MaterialInstance* coneWalkMi = coneWalkMat->createInstance();
    coneWalkMi->setParameter("cones", coneTex, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST));
    coneWalkMi->setParameter("stepCount", (float)coneStepCount);
    coneWalkMi->setParameter("wayCount", (float)coneWays);
    coneWalkMi->setParameter("rows", coneRows);
    coneWalkMi->setParameter("cols", coneCols);
    coneWalkMi->setParameter("floorY", 0.f);
    coneWalkMi->setParameter("tip", (float)beam::CONE_TIP);
    coneWalkMi->setParameter("inset", (float)beam::CONE_INSET);
    coneWalkMi->setParameter("edge", (float)beam::CONE_EDGE);
    coneWalkMi->setParameter("flatter", (float)beam::BALL_FLAT);
    coneWalkMi->setParameter("reach", (float)beam::BALL_REACH);
    Scene* sConeWalk = engine->createScene();
    sConeWalk->addEntity(renderable(sqVB, sqIB, coneWalkMi, false, false));
    MaterialInstance* coneMi[CONE_SETS];
    utils::Entity coneE[CONE_SETS];
    for (int b = 0; b < CONE_SETS; ++b) {
        coneMi[b] = coneMat->createInstance();
        coneMi[b]->setParameter("walk", coneWalk.color, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST));
        coneMi[b]->setParameter("floorY", 0.f);
        coneMi[b]->setParameter("sunTan", std::tan(sunRadiusDeg * 3.14159265f / 180.f));
        coneMi[b]->setParameter("cap", (float)beam::CONE_CAP);
        coneMi[b]->setParameter("stepCount", (float)coneStepCount);
        coneMi[b]->setParameter("wayCount", (float)coneWays);
        coneMi[b]->setParameter("rows", coneRows);
        coneMi[b]->setParameter("cols", coneCols);
        coneE[b] = em.create();
        const size_t count = b == 0 ? conesWithTip : coneRoom - conesWithTip;
        coneMi[b]->setParameter("first", (float)(b == 0 ? 0 : conesWithTip));
        RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, coneVB, b == 0 ? coneIBTip : coneIB)
            .instances(std::max<size_t>(1, count) * (size_t)(coneStepCount * coneWays))
            .material(0, coneMi[b]).culling(false).castShadows(false).receiveShadows(false).layerMask(0xFF, 0x00).build(*engine, coneE[b]);
        sLight->addEntity(coneE[b]);
    }

    // the table of all faces of all solids, in world space, for glass seen through glass
    static const int SCENE_W = 48, SCENE_H = 64;
    if (items.size() > (size_t)SCENE_H) return fail("more than 64 solids");
    Texture* sceneTex = Texture::Builder().width(SCENE_W).height(SCENE_H).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);

    // ---- the light of the flat-faced solids, on the GPU (flat_walk.mat + flat_light.mat, "Flat faces on the GPU" in
    // beam_glass.h). Built once: per shape a table of its pieces and a mesh. Per frame the CPU only says where each
    // solid stands (the pose table). FLATCPU=1 works the same light out on the CPU instead, as before, to compare.
    const bool flatOnCpu = std::getenv("FLATCPU") != nullptr;
    const int piecesPerSide = (int)env_f("PIECES", 4.f);
    const int flatSteps = std::min(15, std::max(1, (int)env_f("FLATSTEPS", 8.f))) + 1;  // colour steps, and the white light mirrored off the outside
    struct FlatKind {
        std::string name;
        beam::FlatShape shape;
        std::vector<int> solids;  // the items of this shape
        MaterialInstance* mi = nullptr;
        utils::Entity entity;
    };
    static std::vector<FlatKind> flatKinds;
    std::vector<int> flatKindOf(items.size(), -1);
    for (size_t k = 0; k < items.size(); ++k) {
        if (items[k].local.kind != beam::Solid::FLAT) continue;
        int found = -1;
        for (size_t q = 0; q < flatKinds.size(); ++q)
            if (flatKinds[q].name == items[k].kind) found = (int)q;
        if (found < 0) {
            flatKinds.push_back(FlatKind{items[k].kind, beam::flat_shape(items[k].local, piecesPerSide), {}, nullptr, {}});
            found = (int)flatKinds.size() - 1;
        }
        flatKinds[(size_t)found].solids.push_back((int)k);
        flatKindOf[k] = found;
    }
    int flatMostPieces = 1;
    for (const FlatKind& fk : flatKinds) flatMostPieces = std::max(flatMostPieces, fk.shape.pieces);
    Texture* flatPieces = Texture::Builder().width((uint32_t)flatMostPieces * 5).height((uint32_t)std::max<size_t>(1, flatKinds.size())).levels(1)
                              .format(Texture::InternalFormat::RGBA32F).build(*engine);
    {
        static std::vector<float> texels;
        texels.assign((size_t)flatMostPieces * 5 * std::max<size_t>(1, flatKinds.size()) * 4, 0.f);
        for (size_t q = 0; q < flatKinds.size(); ++q) std::copy(flatKinds[q].shape.table.begin(), flatKinds[q].shape.table.end(), texels.begin() + (long)(q * (size_t)flatMostPieces * 5 * 4));
        flatPieces->setImage(*engine, 0, Texture::PixelBufferDescriptor(texels.data(), texels.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
    }
    Texture* flatHull = Texture::Builder().width((uint32_t)flatMostPieces * 5).height((uint32_t)std::max<size_t>(1, flatKinds.size())).levels(1)
                            .format(Texture::InternalFormat::RGBA32F).build(*engine);
    {
        static std::vector<float> texels;
        texels.assign((size_t)flatMostPieces * 5 * std::max<size_t>(1, flatKinds.size()) * 4, 0.f);
        for (size_t q = 0; q < flatKinds.size(); ++q) std::copy(flatKinds[q].shape.hull.begin(), flatKinds[q].shape.hull.end(), texels.begin() + (long)(q * (size_t)flatMostPieces * 5 * 4));
        flatHull->setImage(*engine, 0, Texture::PixelBufferDescriptor(texels.data(), texels.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
    }
    Texture* poseTex = Texture::Builder().width(SCENE_H).height(4).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
    Target flatWalk = make_rt(flatMostPieces * 3, (int)items.size() * flatSteps, Texture::InternalFormat::RGBA32F);
    const TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    MaterialInstance* flatWalkMi = flatWalkMat->createInstance();
    flatWalkMi->setParameter("pose", poseTex, nearest);
    flatWalkMi->setParameter("pieces", flatPieces, nearest);
    flatWalkMi->setParameter("scene", sceneTex, nearest);
    flatWalkMi->setParameter("solidCount", (int32_t)items.size());
    flatWalkMi->setParameter("stepCount", (float)flatSteps);
    flatWalkMi->setParameter("floorY", 0.f);
    Scene* sFlatWalk = engine->createScene();
    sFlatWalk->addEntity(renderable(sqVB, sqIB, flatWalkMi, false, false));
    for (FlatKind& fk : flatKinds) {
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(fk.shape.corners.size() / 5)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                               .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(fk.shape.corners.data(), fk.shape.corners.size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)fk.shape.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(fk.shape.idx.data(), fk.shape.idx.size() * sizeof(uint32_t)));
        fk.mi = flatMat->createInstance();
        fk.mi->setParameter("walk", flatWalk.color, nearest);
        fk.mi->setParameter("stepCount", (float)flatSteps);
        fk.mi->setParameter("hull", flatHull, nearest);
        fk.mi->setParameter("shape", (float)(&fk - flatKinds.data()));
        fk.mi->setParameter("floorY", 0.f);
        float4 which[16] = {};
        for (size_t i = 0; i < fk.solids.size() && i < 64; ++i) which[i / 4][i % 4] = (float)fk.solids[i];
        fk.mi->setParameter("solids", which, 16);
        fk.entity = em.create();
        RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .instances(fk.solids.size() * (size_t)flatSteps)
            .material(0, fk.mi).culling(false).castShadows(false).receiveShadows(false)
            .layerMask(0xFF, (flatOnCpu || std::getenv("NOLIGHT")) ? 0x00 : 0x01).build(*engine, fk.entity);
        sLight->addEntity(fk.entity);
    }
    static std::vector<std::vector<float>> mesh_v(items.size());
    static std::vector<std::vector<uint32_t>> mesh_i(items.size());
    for (size_t k = 0; k < items.size(); ++k) {
        Item& it = items[k];
        const beam::Solid& unit = it.local;
        float reach = 0;
        if (unit.kind != beam::Solid::FLAT) {
            // a round solid: a fine enough mesh of its surface (the material works out the true shape per pixel)
            const float r = (float)unit.r, h = (float)unit.h, pi = 3.14159265f;
            auto quad = [&](float3 a, float3 b, float3 c, float3 d, float3 N) {  // a, b, c, d counter-clockwise seen from outside
                const uint32_t base = (uint32_t)(mesh_v[k].size() / 7);
                float3 T = normalize(b - a + float3{1e-6f, 2e-6f, 3e-6f});
                for (float3 p : {a, b, c, d}) push_vert(mesh_v[k], p, T, N);
                mesh_i[k].insert(mesh_i[k].end(), {base, base + 1, base + 2, base, base + 2, base + 3});
            };
            const int ns = 64;
            if (unit.kind == beam::Solid::BALL) {
                const int nr = 32;
                auto at = [&](int i, int j) {
                    const float th = pi * i / nr, ph = 2 * pi * j / ns;
                    return float3{r * std::sin(th) * std::cos(ph), r * std::cos(th), r * std::sin(th) * std::sin(ph)};
                };
                for (int i = 0; i < nr; ++i)
                    for (int j = 0; j < ns; ++j) quad(at(i, j), at(i, j + 1), at(i + 1, j + 1), at(i + 1, j), normalize(at(i, j) + at(i + 1, j + 1)));
                reach = r;
            } else if (unit.kind == beam::Solid::CONE) {
                auto ring = [&](int j, float y, float rad) {
                    const float ph = 2 * pi * j / ns;
                    return float3{rad * std::cos(ph), y, rad * std::sin(ph)};
                };
                const float k = r / h;
                for (int j = 0; j < ns; ++j) {
                    quad(float3{0, 0.5f * h, 0}, float3{0, 0.5f * h, 0}, ring(j + 1, -0.5f * h, r), ring(j, -0.5f * h, r), normalize(ring(j, k, 1) + ring(j + 1, k, 1)));
                    quad(float3{0, -0.5f * h, 0}, ring(j, -0.5f * h, r), ring(j + 1, -0.5f * h, r), float3{0, -0.5f * h, 0}, float3{0, -1, 0});
                }
                reach = std::sqrt(r * r + 0.25f * h * h);
            } else {
                auto ring = [&](int j, float y, float rad) {
                    const float ph = 2 * pi * j / ns;
                    return float3{rad * std::cos(ph), y, rad * std::sin(ph)};
                };
                for (int j = 0; j < ns; ++j) {
                    quad(ring(j, h, r), ring(j + 1, h, r), ring(j + 1, -h, r), ring(j, -h, r), normalize(ring(j, 0, 1) + ring(j + 1, 0, 1)));
                    quad(float3{0, h, 0}, ring(j + 1, h, r), ring(j, h, r), float3{0, h, 0}, float3{0, 1, 0});
                    quad(float3{0, -h, 0}, ring(j, -h, r), ring(j + 1, -h, r), float3{0, -h, 0}, float3{0, -1, 0});
                }
                reach = std::sqrt(r * r + h * h);
            }
        }
        for (size_t f = 0; f < unit.faces.size(); ++f) {
            const beam::Poly& P = unit.faces[f];
            if (P.n < 3) continue;
            float3 N{(float)unit.planes[f].n.x, (float)unit.planes[f].n.y, (float)unit.planes[f].n.z};
            float3 T = normalize(float3{(float)(P.p[1].x - P.p[0].x), (float)(P.p[1].y - P.p[0].y), (float)(P.p[1].z - P.p[0].z)});
            // outward winding for back-face culling (the glass is single sided)
            const bool flip = beam::dot(beam::cross(P.p[1] - P.p[0], P.p[2] - P.p[0]), unit.planes[f].n) < 0;
            uint32_t base = (uint32_t)(mesh_v[k].size() / 7);
            for (int i = 0; i < P.n; ++i) {
                const float3 p{(float)P.p[(size_t)i].x, (float)P.p[(size_t)i].y, (float)P.p[(size_t)i].z};
                reach = std::max(reach, length(p));
                push_vert(mesh_v[k], p, T, N);
            }
            for (int i = 1; i + 1 < P.n; ++i) {
                if (flip) mesh_i[k].insert(mesh_i[k].end(), {base, base + (uint32_t)i + 1, base + (uint32_t)i});
                else mesh_i[k].insert(mesh_i[k].end(), {base, base + (uint32_t)i, base + (uint32_t)i + 1});
            }
        }
        auto [vb, ib] = lit_buffers(mesh_v[k], mesh_i[k]);
        it.glass = glassMat->createInstance();
        it.glass->setParameter("ior", nd);
        it.glass->setParameter("abbe", 64.17f / disp);
        std::vector<float4> planes;
        for (size_t f = 0; f < unit.planes.size(); ++f)
            if (unit.faces[f].n >= 3) planes.push_back(float4{(float)unit.planes[f].n.x, (float)unit.planes[f].n.y, (float)unit.planes[f].n.z, (float)unit.planes[f].o});
        if (planes.size() > 40) return fail("a solid has more than 40 faces");
        const int32_t count = (int32_t)planes.size();
        planes.resize(40, float4{0, 1, 0, 1e6f});
        it.glass->setParameter("planes", planes.data(), 40);
        it.glass->setParameter("planeCount", count);
        it.glass->setParameter("kind", (int32_t)unit.kind);
        it.glass->setParameter("dims", float4{(float)unit.r, (float)unit.h, 0, 0});
        it.glass->setParameter("scene", sceneTex, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST));
        it.glass->setParameter("solidCount", (int32_t)(std::getenv("ALONE") ? 0 : items.size()));
        it.glass->setParameter("self", (int32_t)k);
        it.glass->setParameter("spread", env_f("SPREAD", 2.f));  // colours part only at the last face; this makes up for the others
        it.glass->setParameter("floorY", 0.f);
        it.glass->setParameter("absorption", float3{0.9f, 0.35f, 0.5f} * env_f("ABSORB", 0.15f));
        it.glass->setParameter("backdrop", backdrop.color, lin);
        // drawn by its own material and left out of the backdrop picture (layer 1)
        it.entity = em.create();
        RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {reach, reach, reach}}).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, it.glass).culling(false).castShadows(false).receiveShadows(false).layerMask(0xFF, 0x02).build(*engine, it.entity);
        tm.create(it.entity);
        if (!std::getenv("NOGLASS")) scene->addEntity(it.entity);
    }
    (void)shadowMat;
    (void)filamentShadow;

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
        for (Item& it : items) {
            it.glass->setParameter("env", ibl, cubeS);
            it.glass->setParameter("envGain", iblLux * env_f("ENVGAIN", 0.f));
            it.glass->setParameter("sky", float3{0.95f, 0.97f, 1.0f} * env_f("SKY", 1800.f));
        }
    }

    // ---- camera and views
    Camera* camera = engine->createCamera(em.create());
    camera->setExposure(env_f("APERTURE", 4.0f), 1.f / 125.f, 100.f);
    float cam[7] = {1.55f, 1.45f, 2.15f, 0.10f, 0.0f, -0.05f, 32.f};
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
    // ---- the shadow outlines, on the GPU (outline.mat): per shape a mesh of its surface, built once; the vertex shader
    // slides the sunlit part along the sun onto the floor. OUTLINECPU=1 makes them on the CPU instead, as before.
    const bool outlineOnCpu = std::getenv("OUTLINECPU") != nullptr;
    struct OutlineKind {
        std::string name;
        std::vector<int> solids;
        MaterialInstance* mi = nullptr;
    };
    static std::vector<OutlineKind> outlineKinds;
    for (size_t k = 0; k < items.size(); ++k) {
        int found = -1;
        for (size_t q = 0; q < outlineKinds.size(); ++q)
            if (outlineKinds[q].name == items[k].kind) found = (int)q;
        if (found < 0) {
            outlineKinds.push_back(OutlineKind{items[k].kind, {}, nullptr});
            found = (int)outlineKinds.size() - 1;
        }
        outlineKinds[(size_t)found].solids.push_back((int)k);
    }
    for (OutlineKind& ok : outlineKinds) {
        const beam::Solid& unit = items[(size_t)ok.solids[0]].local;
        auto* v = new std::vector<float>();   // position xyz, then four more numbers (the face's normal, for flat faces)
        auto* ix = new std::vector<uint32_t>();
        auto put = [&](float x, float y, float z, float nx = 0, float ny = 0, float nz = 0) {
            v->insert(v->end(), {x, y, z, nx, ny, nz, 0.f});
            return (uint32_t)(v->size() / 7 - 1);
        };
        const float pi = 3.14159265f;
        if (unit.kind == beam::Solid::FLAT) {
            for (size_t f = 0; f < unit.faces.size(); ++f) {
                const beam::Poly& P = unit.faces[f];
                if (P.n < 3) continue;
                const beam::V3 n = unit.planes[f].n;
                uint32_t first = 0;
                for (int i = 0; i < P.n; ++i) {
                    const uint32_t id = put((float)P.p[(size_t)i].x, (float)P.p[(size_t)i].y, (float)P.p[(size_t)i].z, (float)n.x, (float)n.y, (float)n.z);
                    if (i == 0) first = id;
                }
                for (int i = 1; i + 1 < P.n; ++i) ix->insert(ix->end(), {first, first + (uint32_t)i, first + (uint32_t)i + 1});
            }
        } else if (unit.kind == beam::Solid::BALL) {
            const int n = 56;
            const uint32_t mid = put(0, 0, 0);
            for (int j = 0; j < n; ++j) put(std::cos(2 * pi * j / n), std::sin(2 * pi * j / n), 0);
            for (int j = 0; j < n; ++j) ix->insert(ix->end(), {mid, mid + 1 + (uint32_t)j, mid + 1 + (uint32_t)((j + 1) % n)});
        } else if (unit.kind == beam::Solid::ROD) {
            const int n = 28, m = 32;
            for (int j = 0; j < n; ++j) {
                const float a = -1.f + 2.f * j / n, b = -1.f + 2.f * (j + 1) / n;
                const uint32_t q = put(0, a, 0);
                put(1, a, 0);
                put(1, b, 0);
                put(0, b, 0);
                ix->insert(ix->end(), {q, q + 1, q + 2, q, q + 2, q + 3});
            }
            for (int end = 1; end <= 2; ++end) {
                const uint32_t mid = put(0, 0, (float)end);
                for (int j = 0; j < m; ++j) put(std::cos(2 * pi * j / m), std::sin(2 * pi * j / m), (float)end);
                for (int j = 0; j < m; ++j) ix->insert(ix->end(), {mid, mid + 1 + (uint32_t)j, mid + 1 + (uint32_t)((j + 1) % m)});
            }
        } else {
            const int n = 48;
            for (int part = 0; part < 2; ++part)
                for (int j = 0; j < n; ++j) {
                    const uint32_t q = put(0, (float)j / n, (float)part);
                    put(1, (float)j / n, (float)part);
                    put(1, (float)(j + 1) / n, (float)part);
                    ix->insert(ix->end(), {q, q + 1, q + 2});
                }
        }
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(v->size() / 7)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                               .attribute(VertexAttribute::COLOR, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28).build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(v->data(), v->size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)ix->size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(ix->data(), ix->size() * sizeof(uint32_t)));
        ok.mi = outlineMat->createInstance();
        ok.mi->setParameter("pose", poseTex, nearest);
        ok.mi->setParameter("kind", (float)unit.kind);
        ok.mi->setParameter("dims", float4{(float)unit.r, (float)unit.h, 0, 0});
        ok.mi->setParameter("floorY", 0.f);
        float4 which[16] = {};
        for (size_t i = 0; i < ok.solids.size() && i < 64; ++i) which[i / 4][i % 4] = (float)ok.solids[i];
        ok.mi->setParameter("solids", which, 16);
        utils::Entity e = em.create();
        RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .instances(ok.solids.size())
            .material(0, ok.mi).culling(false).castShadows(false).receiveShadows(false).layerMask(0xFF, outlineOnCpu ? 0x00 : 0x01).build(*engine, e);
        sShade->addEntity(e);
    }

    View* vFlatWalk = engine->createView();
    vFlatWalk->setScene(sFlatWalk);
    vFlatWalk->setCamera(camera);
    vFlatWalk->setRenderTarget(flatWalk.rt);
    vFlatWalk->setViewport({0, 0, (uint32_t)flatMostPieces * 3, (uint32_t)(items.size() * (size_t)flatSteps)});
    vFlatWalk->setDithering(View::Dithering::NONE);
    vFlatWalk->setPostProcessingEnabled(false);
    vFlatWalk->setShadowingEnabled(false);
    View* vConeWalk = engine->createView();
    vConeWalk->setScene(sConeWalk);
    vConeWalk->setCamera(camera);
    vConeWalk->setRenderTarget(coneWalk.rt);
    vConeWalk->setViewport({0, 0, (uint32_t)(coneV.size() / 3) * 2, (uint32_t)coneLines});
    vConeWalk->setDithering(View::Dithering::NONE);
    vConeWalk->setPostProcessingEnabled(false);
    vConeWalk->setShadowingEnabled(false);
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

    // ---- every frame: place the solids, trace their light (one thread each), hand the triangles to the GPU
    float3 sunDir = normalize(float3{0.92f, -0.42f, 0.22f});
    if (const char* sv = std::getenv("SUN")) {
        float x, y, z;
        if (std::sscanf(sv, "%f %f %f", &x, &y, &z) == 3) sunDir = normalize(float3{x, y, z});
    }
    const unsigned threads = (unsigned)std::max(1.f, env_f("THREADS", (float)std::max(1u, std::thread::hardware_concurrency())));
    const int pieces = (int)env_f("PIECES", 4.f);       // each lit face is cut into pieces x pieces parts
    const bool exact = std::getenv("EXACT") != nullptr; // the slow exact trace, to compare
    const bool tumble = std::getenv("TUMBLE") != nullptr; // every solid turns about its own tilted axis and leaves the floor
    const bool ballsOnCpu = std::getenv("BALLCPU") != nullptr;  // work every ball's light out on the CPU (to compare)
    const bool conesOnCpu = std::getenv("CONECPU") != nullptr;  // the same for the cones
    size_t conesDrawn = 0;
    double traceMs = 0;
    size_t lightTris = 0, ballsAlike = 0;
    beam::Scene world;
    static std::vector<float> tri, outline;
    auto update = [&](float t) {
        auto& lm = engine->getLightManager();
        lm.setDirection(lm.getInstance(sunE), sunDir);
        softenMi->setParameter("sunDown", -sunDir.y);
        for (Item& it : items) {
            it.glass->setParameter("sunTo", -sunDir);
            it.glass->setParameter("sunGlint", sunColor * env_f("GLINT", 4.0e5f));
            it.glass->setParameter("sunCos", std::cos(env_f("GLINTDEG", 2.0f) * 3.14159265f / 180.f));
        }
        auto t0 = std::chrono::steady_clock::now();
        // where every solid stands now
        world.solids.resize(items.size());
        std::vector<double> detail(items.size());
        auto* pose = new std::vector<float>((size_t)SCENE_H * 4 * 4, 0.f);  // where every solid stands: all the CPU tells the flat-face light
        for (size_t k = 0; k < items.size(); ++k) {
            Item& it = items[k];
            const float yaw = it.yaw + it.spin * t;
            const float x = it.x + it.drift * std::cos(t + it.phase), z = it.z + it.drift * std::sin(t + it.phase);
            beam::V3 R[3];
            item_axes(it, yaw, R);
            double lift = 0;
            if (tumble) {
                // thrown about: turning about its own tilted axis, and off the floor
                const beam::V3 ax = beam::normalize(beam::V3{std::sin(1.3 * k + 0.4), 0.6 + 0.4 * std::cos(2.1 * k), std::cos(1.7 * k + 1.0)});
                const double ang = it.phase + (0.9 + 0.35 * (k % 3)) * t * it.spin, cs = std::cos(ang), sn = std::sin(ang);
                for (beam::V3& v : R) v = v * cs + beam::cross(ax, v) * sn + ax * (beam::dot(ax, v) * (1.0 - cs));
                lift = 0.22 * (0.5 + 0.5 * std::sin(1.7 * t + 2.3 * k));
            }
            double low = 1e30;
            for (const beam::Poly& P : it.local.faces)
                for (int i = 0; i < P.n; ++i) {
                    const beam::V3 v = P.p[(size_t)i];
                    low = std::min(low, (R[0].y * v.x + R[1].y * v.y + R[2].y * v.z) * it.size);
                }
            if (it.local.kind == beam::Solid::BALL) low = -it.local.r * it.size;
            if (it.local.kind == beam::Solid::CONE)  // the tip, or the lowest point of the rim
                low = std::min(0.5 * it.local.h * R[1].y, -0.5 * it.local.h * R[1].y - it.local.r * std::sqrt(std::max(0.0, 1.0 - R[1].y * R[1].y))) * it.size;
            if (it.local.kind == beam::Solid::ROD) low = -(it.local.h * std::fabs(R[1].y) + it.local.r * std::sqrt(std::max(0.0, 1.0 - R[1].y * R[1].y))) * it.size;
            const beam::V3 at{x, -low + 0.0005 + lift, z};
            world.solids[k] = beam::placed(it.local, R, it.size, at);
            const float s = it.size;
            tm.setTransform(tm.getInstance(it.entity),
                            mat4f(float4{(float)R[0].x * s, (float)R[0].y * s, (float)R[0].z * s, 0}, float4{(float)R[1].x * s, (float)R[1].y * s, (float)R[1].z * s, 0},
                                  float4{(float)R[2].x * s, (float)R[2].y * s, (float)R[2].z * s, 0}, float4{(float)at.x, (float)at.y, (float)at.z, 1}));
            for (int c = 0; c < 3; ++c) {
                float* o = pose->data() + ((size_t)c * SCENE_H + k) * 4;
                o[0] = (float)R[c].x; o[1] = (float)R[c].y; o[2] = (float)R[c].z;
            }
            (*pose)[(0 * (size_t)SCENE_H + k) * 4 + 3] = s;
            (*pose)[(1 * (size_t)SCENE_H + k) * 4 + 3] = (float)flatKindOf[k];
            (*pose)[(2 * (size_t)SCENE_H + k) * 4 + 3] = flatKindOf[k] >= 0 ? (float)flatKinds[(size_t)flatKindOf[k]].shape.pieces : 0.f;
            {
                float* o = pose->data() + (3 * (size_t)SCENE_H + k) * 4;
                o[0] = (float)at.x; o[1] = (float)at.y; o[2] = (float)at.z;
            }
            const float dist = length(float3{cam[0], cam[1], cam[2]} - float3{(float)at.x, (float)at.y, (float)at.z});
            detail[k] = 2.0 * std::tan(cam[6] * 0.5 * 3.14159265 / 180.0) * dist / H * env_f("DETAIL", 1.f);  // one pixel on the floor there
        }
        world.bound();
        poseTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(pose->data(), pose->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                     [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, pose));
        beam::Params par;
        par.sun = {sunDir.x, sunDir.y, sunDir.z};
        par.bands = bands;
        par.glass.nd = nd;
        par.glass.exaggerate = disp;
        par.threads = threads;
        par.coneSteps = coneStepCount;
        par.sunTan = std::tan(sunRadiusDeg * 3.14159265 / 180.0);
        std::vector<int> alike;  // the balls and cones: drawn from their shared shapes
        if (exact) {
            par.minFlux = 30.0 * detail[0] * detail[0];
            beam::trace_fast(world, par, detail, tri);
        } else {
            const unsigned leave = (1u << beam::Solid::BALL) | (1u << beam::Solid::CONE) | (1u << beam::Solid::ROD) | (flatOnCpu ? 0u : 1u << beam::Solid::FLAT);
            beam::trace_cheap(world, par, detail, pieces, tri, nullptr, ballsOnCpu ? nullptr : &alike, leave);
        }
        outline.clear();
        if (outlineOnCpu) beam::silhouette_triangles(world, par.sun, 0.0, outline);
        for (OutlineKind& ok : outlineKinds) ok.mi->setParameter("sunDir", sunDir);
        traceMs = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();

        // the cones: where they are and how they stand to the sun into their table, and as many instances switched on
        {
            std::vector<int> balls, cones;
            for (int i : alike) {
                if (world.solids[(size_t)i].kind == beam::Solid::CONE || world.solids[(size_t)i].kind == beam::Solid::ROD) cones.push_back(i);
                else if (world.solids[(size_t)i].kind == beam::Solid::BALL) balls.push_back(i);
            }
            {
                // the flat-faced solids: their colour steps (the pose table is already on its way)
                const std::vector<beam::BallStep> fsteps = beam::ball_steps(par, flatSteps - 1);
                float4 table[16] = {};
                for (int i = 0; i + 1 < flatSteps && i < (int)fsteps.size(); ++i)
                    table[i] = float4{(float)fsteps[(size_t)i].rgb.x, (float)fsteps[(size_t)i].rgb.y, (float)fsteps[(size_t)i].rgb.z, (float)fsteps[(size_t)i].n};
                table[flatSteps - 1] = table[(flatSteps - 1) / 2];
                flatWalkMi->setParameter("steps", table, 16);
                flatWalkMi->setParameter("sunDir", sunDir);
                for (FlatKind& fk : flatKinds) fk.mi->setParameter("steps", table, 16);
            }
            alike.swap(balls);
            // the cones first, then the rods (as their two renderables expect)
            std::stable_sort(cones.begin(), cones.end(), [&](int a, int b) { return world.solids[(size_t)a].kind == beam::Solid::CONE && world.solids[(size_t)b].kind != beam::Solid::CONE; });
            if (ballsOnCpu) {  // (then trace_cheap has already written everything, the cones' light too)
                cones.clear();
            } else if (conesOnCpu) {
                for (int i : cones) beam::cone_light(world.solids[(size_t)i], par, tri);
                cones.clear();
            }
            conesDrawn = std::getenv("NOLIGHT") ? 0 : std::min(cones.size(), coneRoom);
            const size_t CONE_TABLE = std::max<size_t>(1, coneRoom);
            if (conesDrawn) {
                auto* ct = new std::vector<float>((size_t)CONE_TABLE * 4 * 4, 0.f);
                for (size_t i = 0; i < conesDrawn; ++i) {
                    const beam::Taper so = beam::taper_of(world.solids[(size_t)cones[i]]);
                    const beam::ConeFrame cf = beam::cone_frame(so, par.sun);
                    float* a = ct->data() + i * 4;
                    float* b = a + (size_t)CONE_TABLE * 4;
                    float* c = b + (size_t)CONE_TABLE * 4;
                    c[(size_t)CONE_TABLE * 4] = (float)so.rt;
                    a[0] = (float)so.c.x; a[1] = (float)so.c.y; a[2] = (float)so.c.z; a[3] = (float)so.r;
                    b[0] = (float)so.ax.x; b[1] = (float)so.ax.y; b[2] = (float)so.ax.z; b[3] = (float)so.h;
                    c[0] = (float)cf.e1.x; c[1] = (float)cf.e1.y; c[2] = (float)cf.e1.z; c[3] = (float)cf.half;
                }
                coneTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(ct->data(), ct->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                             [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, ct));
            }
            const std::vector<beam::BallStep> steps = beam::ball_steps(par, par.coneSteps);
            float4 stepTable[32] = {};
            for (int i = 0; i < coneStepCount && i < (int)steps.size(); ++i)
                stepTable[i] = float4{(float)steps[(size_t)i].rgb.x, (float)steps[(size_t)i].rgb.y, (float)steps[(size_t)i].rgb.z, (float)steps[(size_t)i].n};
            for (int b = 0; b < CONE_SETS; ++b) {
                const bool on = conesDrawn > 0 && (b == 0 ? conesWithTip : coneRoom - conesWithTip) > 0;
                rm.setLayerMask(rm.getInstance(coneE[b]), 0xFF, on ? 0x01 : 0x00);
                if (!on) continue;
                coneMi[b]->setParameter("steps", stepTable, 32);
                coneMi[b]->setParameter("sunDir", sunDir);
            }
            coneWalkMi->setParameter("steps", stepTable, 32);
            coneWalkMi->setParameter("sunDir", sunDir);
        }
        // the balls: middle and radius into their table, and as many instances switched on
        ballsAlike = std::min<size_t>(alike.size(), (size_t)BALL_TABLE - 1);
        if (std::getenv("NOLIGHT")) ballsAlike = 0;
        if (ballsAlike) {
            auto* bt = new std::vector<float>((size_t)BALL_TABLE * 4, 0.f);
            for (size_t i = 0; i < ballsAlike; ++i) {
                const beam::Solid& so = world.solids[(size_t)alike[i]];
                float* o = bt->data() + i * 4;
                o[0] = (float)so.c.x; o[1] = (float)so.c.y; o[2] = (float)so.c.z; o[3] = (float)so.r;
            }
            ballTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(bt->data(), bt->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                         [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, bt));
        }
        {
            const std::vector<beam::BallStep> steps = beam::ball_steps(par);
            struct { beam::V3 w, e1, e2; } bf;
            beam::sun_frame(par.sun, bf.w, bf.e1, bf.e2);
            float4 stepTable[12] = {};
            for (int i = 0; i < ballStepCount; ++i)
                stepTable[i] = float4{(float)steps[(size_t)i].rgb.x, (float)steps[(size_t)i].rgb.y, (float)steps[(size_t)i].rgb.z, (float)steps[(size_t)i].n};
            size_t first = 0;
            for (int b = 0; b < BALL_SETS; ++b) {
                const bool on = (ballsAlike >> b) & 1u;
                rm.setLayerMask(rm.getInstance(ballE[b]), 0xFF, on ? 0x01 : 0x00);
                if (!on) continue;
                ballMi[b]->setParameter("steps", stepTable, 12);
                ballMi[b]->setParameter("first", (float)first);
                ballMi[b]->setParameter("sunTo", float3{(float)bf.w.x, (float)bf.w.y, (float)bf.w.z});
                ballMi[b]->setParameter("e1", float3{(float)bf.e1.x, (float)bf.e1.y, (float)bf.e1.z});
                ballMi[b]->setParameter("e2", float3{(float)bf.e2.x, (float)bf.e2.y, (float)bf.e2.z});
                first += (size_t)1 << b;
            }
        }

        // the table of faces for glass seen through glass
        auto* table = new std::vector<float>((size_t)SCENE_W * SCENE_H * 4, 0.f);
        for (size_t k = 0; k < items.size(); ++k) {
            float* row = table->data() + k * SCENE_W * 4;
            row[0] = (float)world.centre[k].x; row[1] = (float)world.centre[k].y; row[2] = (float)world.centre[k].z; row[3] = (float)world.radius[k];
            int count = 0;
            row[5] = (float)world.solids[k].kind; row[6] = (float)world.solids[k].r; row[7] = (float)world.solids[k].h;
            row[8] = (float)world.solids[k].ax.x; row[9] = (float)world.solids[k].ax.y; row[10] = (float)world.solids[k].ax.z;
            if (world.solids[k].kind == beam::Solid::CONE) {  // the middle of its base
                row[12] = (float)world.solids[k].c.x; row[13] = (float)world.solids[k].c.y; row[14] = (float)world.solids[k].c.z;
            }
            for (size_t f = 0; f < world.solids[k].planes.size() && count < SCENE_W - 3; ++f) {
                if (world.solids[k].faces[f].n < 3) continue;
                float* o = row + (3 + count) * 4;
                o[0] = (float)world.solids[k].planes[f].n.x; o[1] = (float)world.solids[k].planes[f].n.y; o[2] = (float)world.solids[k].planes[f].n.z;
                o[3] = (float)world.solids[k].planes[f].o;
                ++count;
            }
            row[4] = (float)count;
        }
        {
            float4 spheres[64] = {};
            for (size_t k = 0; k < items.size() && k < 64; ++k)
                spheres[k] = float4{(float)world.centre[k].x, (float)world.centre[k].y, (float)world.centre[k].z, (float)world.radius[k]};
            flatWalkMi->setParameter("spheres", spheres, 64);
            for (Item& it : items) it.glass->setParameter("spheres", spheres, 64);
        }
        sceneTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(table->data(), table->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                      [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, table));

        const size_t TS = beam::TRI;
        size_t nLight = tri.size() / TS, nOut = outline.size() / TS;
        if (nLight + nOut > MAX_LIGHT_VERTS) nLight = (MAX_LIGHT_VERTS - nOut) / 3 * 3;  // never in practice
        lightTris = nLight / 3;
        const size_t nv = nLight + nOut;
        auto* buf = new std::vector<float>(std::max<size_t>(nv, 3) * 7, 0.f);
        const bool off = std::getenv("NOLIGHT") != nullptr;
        for (size_t i = 0; i < nv; ++i) {
            const float* src = i < nLight ? tri.data() + i * TS : outline.data() + (i - nLight) * TS;
            float* o = buf->data() + i * 7;
            o[0] = src[0];
            o[1] = 0.f;
            o[2] = src[1];
            o[3] = (off && i < nLight) ? 0.f : src[2];
            o[4] = (off && i < nLight) ? 0.f : src[3];
            o[5] = (off && i < nLight) ? 0.f : src[4];
            o[6] = src[5];  // distance flown, for the softening pass
        }
        lightVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(buf->data(), std::max<size_t>(nv, 3) * 7 * sizeof(float),
                                                                         [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, buf));
        rm.setGeometryAt(rm.getInstance(lightE), 0, RenderableManager::PrimitiveType::TRIANGLES, lightVB, lightIB, 0, std::max<size_t>(nLight, 3));
        rm.setLayerMask(rm.getInstance(lightE), 0xFF, nLight ? 0x01 : 0x00);  // no CPU light at all (only balls alike): the buffer starts with the outline, do not draw it as light
        rm.setGeometryAt(rm.getInstance(shadeE), 0, RenderableManager::PrimitiveType::TRIANGLES, lightVB, lightIB, nOut ? nLight : 0, std::max<size_t>(nOut, 3));
        rm.setLayerMask(rm.getInstance(shadeE), 0xFF, nOut ? 0x01 : 0x00);
    };

    auto draw_light = [&]() {
        if (!flatOnCpu && !flatKinds.empty()) renderer->render(vFlatWalk);
        if (conesDrawn) renderer->render(vConeWalk);
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
    const float t0 = env_f("T", 0.f);
    update(t0);
    for (int i = 0; i < 3; ++i) {
        if (!open()) return fail("warm");
        draw_light();
        renderer->render(vBackdrop);
        renderer->render(vBeauty);
        renderer->endFrame();
        engine->flushAndWait();
    }
    const std::string out = "out/glass-scene-" + name + ".png";
    if (!grab(out.c_str())) return fail("grab");
    update(t0);
    std::printf("wrote %s  (%zu solids, chamfer %.3f, dispersion x%.1f, %s, %u threads: light %.2f ms, %zu light triangles)\n", out.c_str(), items.size(), chamfer, disp,
                exact ? "EXACT trace" : "cheap trace", threads, traceMs, lightTris);
    std::printf("      %zu ball(s) drawn from the one shared shape (%zu triangles on the GPU, none from the CPU)\n", ballsAlike, ballsAlike * (size_t)ballStepCount * discI.size() / 3);
    std::printf("      %zu cone(s) drawn from the one shared mesh (%zu triangles on the GPU, none from the CPU)\n", conesDrawn, conesDrawn * (size_t)(coneStepCount * coneWays) * coneI.size() / 3);

    // ---- a run of frames for a movie: FRAMES=n writes out/seq/0000.png ... at 60 per second of the same motion
    if (std::getenv("FRAMES")) {
        const int n = (int)env_f("FRAMES", 120.f);
        std::system("mkdir -p out/seq && rm -f out/seq/*.png");
        const float3 sun0 = sunDir;
        for (int i = 0; i < n; ++i) {
            const float t = t0 + (float)i / 120.f * 6.2831853f * env_f("SPEED", 0.5f);
            if (!std::getenv("FIXSUN")) sunDir = normalize(float3{sun0.x * std::cos(0.3f * std::sin(t)), sun0.y, sun0.z + 0.4f * std::sin(t)});
            update(t);
            char path[64];
            std::snprintf(path, sizeof(path), "out/seq/%04d.png", i);
            if (!grab(path)) return fail("frame");
        }
        std::printf("wrote %d frames to out/seq\n", n);
        return 0;
    }

    // ---- paced like an app: every tick every solid turns and slides and the sun swings, so all the light is
    // traced again and uploaded again each frame. beginFrame() returning false is Filament saying it is behind.
    if (std::getenv("PACE")) {
        const int n = (int)env_f("PACE_FRAMES", 600.f);
        const double hz = env_f("PACE_HZ", 60.f);
        const double period = 1000.0 / hz;
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
            sunDir = normalize(float3{sun0.x * std::cos(0.3f * std::sin(t)), sun0.y, sun0.z + 0.4f * std::sin(t)});
            auto u0 = std::chrono::steady_clock::now();
            update(t);
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
        std::printf("PACE %.0f Hz for %d frames (%.1f s, target %.1f s), %zu solids and the sun moving every frame, %dx%d, %u threads\n", hz, n, total / 1000.0,
                    n * period / 1000.0, items.size(), W, H, threads);
        auto line = [&](const char* nm, std::vector<double>& v) {
            std::printf("     %-44s n=%3zu  min %6.2f  median %6.2f  95%% %6.2f  99%% %6.2f  max %6.2f ms\n", nm, v.size(), pct(v, 0.0), pct(v, 0.5), pct(v, 0.95), pct(v, 0.99), pct(v, 1.0));
        };
        line("CPU: the light of all solids", cpu);
        line("CPU: trace + fill + hand to Filament", upd);
        line("GPU time per frame", g);
        line("beginFrame -> GPU finished", lat);
        std::printf("     most light triangles in a frame: %zu from the CPU; balls drawn from the one shared shape: %zu, cones: %zu\n", maxTris, ballsAlike, conesDrawn);
        std::printf("     finished after their %.2f ms slot: %zu of %zu measured (%.1f%%)\n", period, late, lat.size(), lat.empty() ? 0.0 : 100.0 * late / lat.size());
        std::printf("     DROPPED (Filament asked to skip): %d of %d ticks (%.1f%%)  -> delivered %.1f fps of %.0f\n", skipped, n, 100.0 * skipped / n, drawn / (total / 1000.0), hz);
    }
    return 0;
}
