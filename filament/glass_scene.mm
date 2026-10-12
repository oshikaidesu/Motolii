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
// Solids with dents and holes (star, washer, letters: beam_extrude.h) likewise have their light and their shadow worked
// out on the GPU (extrude_find/walk/strip/cap.mat); EXTRUDECPU=1 draws the CPU's copy of the same instead, to compare.
// Light on things other than the floor: WALLS=1|2 (a wall behind the solids, or two meeting in a corner; WALLGAP, WALLTOP,
// WALLYAW), PROPS=1..4 (matte blocks among the solids; or PROPAT="x z halfx halfy halfz yaw; ..."), STAGE=1 (the same
// machinery with neither, to compare with the plain picture). See "the stage" below and "Receivers" in beam_glass.h.
#include "beam_glass.h"
#include "beam_split.h"

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

// As a library (-DGLASS_LIB, see glass_live.h) the files it loads are looked for in the host's folder; on its own, here.
#ifdef GLASS_LIB
#include "glass_door.h"
#include "glass_live.h"
#include <atomic>
#define GLASS_PATH(p) (g_live.base + (p))
#else
#define GLASS_PATH(p) std::string(p)
#endif

static int W = 1920, H = 1080;   // RES=2 / RES=4: 3840x2160 / 7680x4320 (set first thing in main)
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
    MaterialInstance* cheap = nullptr; // VBODY: the cheap layer in front of it (glass_vert.mat)
    bool vbody = false;                // VBODY: its four-sided faces are also drawn as a net from the table of glass_nodes.mat
    utils::Entity entity;
    std::vector<float> tri, outline;   // this frame's light and shadow outline
    double ms = 0;
    int exColumn = -1;                 // solids with dents and holes: its column in their table of poses
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

#ifdef GLASS_LIB
int glass_scene_main(int argc, char** argv) {
#else
int main(int argc, char** argv) {
#endif
    // DARKROOM=1: the dark room in one switch (each of these can still be set on its own): a dark matte table, sun
    // through a blind (bands of sun across the floor), the exposure opened until those bands burn out, a dim sky and
    // the bright window for the glass to show, the solids strewn about. Meant for solids with colour films (xcut, xcube).
    if (std::getenv("DARKROOM")) {
        static const char* dark[][2] = {{"ROOM", "0.012"}, {"FLOORGREY", "0.3"}, {"FLOOR_REFL", "0.1"}, {"APERTURE", "1.6"}, {"BLOOM", "0.35"},
                                        {"SKY", "30000"}, {"GLINT", "30000"}, {"GLINTDEG", "25"}, {"STRIP", "0.2 35 0 0.95"}, {"PIECES", "4"}, {"BEAMS", "1"},
                                        {"SCATTER", "1"}};
        for (const auto& kv : dark) setenv(kv[0], kv[1], 0);
    }
    if (const char* res = std::getenv("RES")) {
        const double k = std::min(4.0, std::max(0.25, std::atof(res)));
        W = (int)std::lround(1920 * k);
        H = (int)std::lround(1080 * k);
    }
#ifdef GLASS_LIB
    W = g_live.w;
    H = g_live.h;
#endif
    const std::string name = argc > 1 ? argv[1] : "first";
    const float nd = 1.5168f;
    const float chamfer = env_f("CHAMFER", 0.03f);   // flat width as a fraction of the edge (real optical cubes: about 0.012)
    const float disp = env_f("DISP", 10.f);          // how much further than real glass the colours spread
    const float sunRadiusDeg = env_f("SUNR", 0.5f);  // half the sun's width in degrees (the real sun: 0.27)
    const int bands = (int)env_f("BANDS", 64.f);
    // The light and the outline are drawn into pictures SS times the screen's size each way (1: pixel for pixel; 0.5: a
    // quarter of the pixels, and the softening pass, which reads them, hands the floor a full-size picture all the same).
    // LIGHTH=n instead: the light's pictures are n lines high whatever the size of the final picture (so their cost does not
    // grow with it), and so is the softened picture the floor reads (SOFTFULL=1 keeps that one at the picture's own size).
    const float SS = std::getenv("LIGHTH") ? std::max(64.f, env_f("LIGHTH", 1080.f)) / (float)H : std::min(4.f, std::max(0.125f, env_f("SS", 1.f)));
    const int LW = std::max(16, (int)std::lround(W * SS)), LH = std::max(16, (int)std::lround(H * SS));
    const bool softSmall = std::getenv("LIGHTH") && !std::getenv("SOFTFULL");
    const int SW = softSmall ? LW : W, SH = softSmall ? LH : H;
    const bool filamentShadow = std::getenv("FILSHADOW") != nullptr;  // debug: Filament's own shadow map instead of the softened outline
    const float lux = 22000.f;
    const float3 sunColor{1.f, 0.98f, 0.96f};

    // ---- the stage: what light can land on besides the floor (walls, matte blocks). With none of these set nothing
    // of it is in use and the picture is the plain one.
    // Everything about it is settled here, once: the receivers are a small table for the materials (see "Receivers" in
    // beam_glass.h), and which receiver a piece of light lands on is decided in the shaders. Nothing is added to a frame's CPU work.
    struct PropSpec {
        float x, z, hx, hy, hz, yaw;   // a matte block: where it stands, half sizes, turn about the vertical
    };
    std::vector<PropSpec> props;
    if (const char* pa = std::getenv("PROPAT")) {
        // "x z halfx halfy halfz yaw; x z ..." (yaw in degrees)
        for (const char* c = pa; c && *c && props.size() < 4;) {
            PropSpec ps{};
            if (std::sscanf(c, "%f %f %f %f %f %f", &ps.x, &ps.z, &ps.hx, &ps.hy, &ps.hz, &ps.yaw) == 6) {
                ps.yaw *= 3.14159265f / 180.f;
                props.push_back(ps);
            }
            c = std::strchr(c, ';');
            if (c) ++c;
        }
    }
    const int wallCount = std::min(2, std::max(0, (int)env_f("WALLS", 0.f)));
    const int propWant = std::getenv("PROPAT") ? (int)props.size() : std::min(4, std::max(0, (int)env_f("PROPS", 0.f)));
    const bool stageOn = wallCount > 0 || propWant > 0 || std::getenv("STAGE") != nullptr;
    const int roomPlanes = 1 + wallCount;        // the floor and the walls
    // A material that puts light or an outline on the floor draws it once per receiver on a stage (recv_drop.glsl):
    // the floor, each wall, and three faces of each block. `on_stage` notes it down (it is told about the stage once
    // that is set up, further down); its renderable gets `recvTotal` times the instances.
    // A material whose light is a fine mesh lets every corner choose what it lands on instead, and is drawn once for the
    // room and, if there are blocks, once more for the blocks: `recvKinds` times (see recv_drop.glsl, recvChooses).
    const size_t recvTotal = stageOn ? (size_t)roomPlanes + 3 * (size_t)propWant : 1;
    const size_t recvKinds = stageOn && propWant > 0 ? 2 : 1;
    std::vector<MaterialInstance*> stageLights;
    auto on_stage = [&](MaterialInstance* mi, bool chooses) {
        mi->setParameter("recvCount", (float)(chooses ? recvKinds : recvTotal));
        mi->setParameter("recvChooses", chooses ? 1.f : 0.f);
        stageLights.push_back(mi);
    };

#ifdef GLASS_LIB
    Engine* engine = glass_door_engine();
#else
    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
#endif
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    auto& em = utils::EntityManager::get();
    auto& tm = engine->getTransformManager();
    auto& rm = engine->getRenderableManager();

    auto mat = [&](const char* path) -> Material* {
        auto b = load_file(GLASS_PATH(path).c_str());
        return b.empty() ? nullptr : Material::Builder().package(b.data(), b.size()).build(*engine);
    };
    Material* lightMat = mat("beam_light.filamat");
    // POLYTEST=n (experiment): the floor works out n soft-edged patches of light per pixel from a table (floor_poly.mat)
    static const int polyTest = std::getenv("POLYTEST") ? std::max(0, std::min(64, std::atoi(std::getenv("POLYTEST")))) : -1;
    Material* floorMat = mat(polyTest >= 0 ? "floor_poly.filamat" : "floor_beam.filamat");
    // SSRBACK=1 (experiment): the bodies look at Filament's own copy of the opaque scene; no backdrop view (glass_solid_ssr.mat)
    static const bool noBackdrop = std::getenv("SSRBACK") || std::getenv("XBACKDROP");
    Material* glassMat = mat(std::getenv("SSRBACK") ? "glass_solid_ssr.filamat" : "glass_solid.filamat");
    Material* shadowMat = mat("shadow_only.filamat");
    Material* softenMat = mat("beam_soften.filamat");
    Material* ballMat = mat("ball_light.filamat");
    Material* coneMat = mat("cone_light.filamat");
    Material* coneWalkMat = mat("cone_walk.filamat");
    // RAINBOW1=1 (experiment): every piece of a flat-faced solid's light is drawn once, between where the red and the blue
    // end of the spectrum land, and the pixels work the colours out (flat_walk1.mat, flat_light1.mat), instead of once per colour step
    static const bool rainbowOnce = std::getenv("RAINBOW1") != nullptr;
    Material* flatMat = mat(rainbowOnce ? "flat_light1.filamat" : "flat_light.filamat");
    Material* flatWalkMat = mat(rainbowOnce ? "flat_walk1.filamat" : "flat_walk.filamat");
    Material* outlineMat = mat("outline.filamat");
    Material* exFindMat = mat("extrude_find.filamat");
    Material* exWalkMat = mat("extrude_walk.filamat");
    Material* exStripMat = mat("extrude_strip.filamat");
    Material* exCapMat = mat("extrude_cap.filamat");
    if (!lightMat || !floorMat || !glassMat || !shadowMat || !softenMat || !ballMat || !coneMat || !coneWalkMat || !flatMat || !flatWalkMat || !outlineMat)
        return fail("material");
    if (!exFindMat || !exWalkMat || !exStripMat || !exCapMat) return fail("material (extrude_*)");
    Material* recvLightMat = stageOn ? mat("recv_light.filamat") : nullptr;
    Material* recvMaskMat = stageOn ? mat("recv_mask.filamat") : nullptr;
    Material* recvBeamMat = stageOn ? mat("recv_beam.filamat") : nullptr;
    if (stageOn && (!recvLightMat || !recvMaskMat || !recvBeamMat)) return fail("material (recv_*)");

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
    Target light = make_rt(LW, LH, Texture::InternalFormat::RGBA16F, true);   // the block's light on the floor
    Target shade = make_rt(LW, LH, Texture::InternalFormat::RGBA16F, true);   // the block's outline against the sun
    Target soft = make_rt(SW, SH, Texture::InternalFormat::RGBA16F);    // the light after the sun's width has softened it
    // the scene without the glass, for the glass to look at: BACK times the screen's size each way, BACKFMT=11 for a
    // picture of half the bytes (no fourth number, fewer bits)
    const float backScale = std::min(1.f, std::max(0.125f, env_f("BACK", 1.f)));
    const int BW = std::max(16, (int)std::lround(W * backScale)), BH = std::max(16, (int)std::lround(H * backScale));
    Target backdrop = make_rt(BW, BH, env_f("BACKFMT", 16.f) < 12.f ? Texture::InternalFormat::R11F_G11F_B10F : Texture::InternalFormat::RGBA16F);
    Target beauty = make_rt(W, H, Texture::InternalFormat::RGBA8);
    const bool blocksOn = propWant > 0;
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
    if (!stageOn) sLight->addEntity(lightE);   // (on a stage the same buffer is drawn once per receiver: see "the stage's own drawing")
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
        on_stage(ballMi[b], true);
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
            .instances((size_t)(1 << b) * (size_t)ballStepCount * recvKinds)
            .material(0, ballMi[b]).culling(false).castShadows(false).receiveShadows(false).layerMask(0xFF, 0x00).build(*engine, ballE[b]);
        sLight->addEntity(ballE[b]);
    }
    // ---- the light of every cone: likewise one mesh, drawn once per cone, colour step and way (cone_light.mat).
    // What becomes of each corner is worked out once, in a pass of its own, into a small float texture (cone_walk.mat).
    // (set up further down, once the solids are known: the texture has one line per cone, colour step and way)
    utils::Entity shadeE = renderable(lightVB, lightIB, lightMat->createInstance(), false, false, 100.f);  // same buffer, the outline's part of it
    Scene* sShade = engine->createScene();
    if (!stageOn) sShade->addEntity(shadeE);

    // ---- softening pass (full-screen quad)
    MaterialInstance* softenMi = softenMat->createInstance();
    softenMi->setParameter("light", light.color, mipped);
    softenMi->setParameter("shade", shade.color, mipped);
    softenMi->setParameter("floorY", 0.f);
    softenMi->setParameter("sunTan", std::tan(sunRadiusDeg * 3.14159265f / 180.f));
    softenMi->setParameter("texel", float2{1.f / LW, 1.f / LH});
    softenMi->setParameter("least", env_f("LEAST", LH < H ? 1.2f : 0.f));   // (see beam_soften.mat: only when the light's picture is the smaller one)
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
    // ROOM (1: the pale studio; towards 0: a dark room, sky and fill light go down together), FLOORGREY (how light the floor is)
    const float roomLight = env_f("ROOM", 1.f), floorGrey = env_f("FLOORGREY", 1.f);
    floorMi->setParameter("color", float4{0.86f * floorGrey, 0.85f * floorGrey, 0.84f * floorGrey, 1});
    floorMi->setParameter("roughness", env_f("FLOOR_ROUGH", 0.42f));
    floorMi->setParameter("reflectance", env_f("FLOOR_REFL", 0.35f));
    floorMi->setParameter("gain", sunColor * (lux / 3.14159265f));
    floorMi->setParameter("light", soft.color, lin);
    if (polyTest >= 0) {
        static std::vector<float> polys(64 * 2 * 4);
        for (int i = 0; i < 64; ++i) {   // fixed made-up patches: a spiral of slanted four-sided ones round the middle
            const float an = 0.7f * i, rr = 0.25f + 0.035f * i, tilt = 1.3f * i;
            const float cx = rr * std::cos(an), cz = rr * std::sin(an), hw = 0.10f + 0.004f * (i % 7), hh = 0.05f + 0.006f * (i % 5);
            const float e[8] = {cx, cz, hw * std::cos(tilt), hw * std::sin(tilt), -hh * std::sin(tilt + 0.3f), hh * std::cos(tilt + 0.3f), 0.004f + 0.002f * (i % 9), 0.25f};
            std::copy(e, e + 8, polys.begin() + i * 8);
        }
        Texture* polyTex = Texture::Builder().width(128).height(1).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
        polyTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(polys.data(), polys.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
        floorMi->setParameter("polys", polyTex, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST));
        floorMi->setParameter("polyCount", (int32_t)polyTest);
    }
    static std::vector<float> floor_v;
    static std::vector<uint32_t> floor_i = {0, 2, 1, 0, 3, 2};
    {
        const float F = env_f("FLOOR", 6.f);
        const float3 fp[4] = {{-F, 0, -F}, {F, 0, -F}, {F, 0, F}, {-F, 0, F}};
        for (int i = 0; i < 4; ++i) push_vert(floor_v, fp[i], {1, 0, 0}, {0, 1, 0});
    }
    auto [floorVB, floorIB] = lit_buffers(floor_v, floor_i);
    // on a stage the floor, the walls and the blocks are all the same kind of surface (recv_beam.mat)
    std::vector<MaterialInstance*> recvSurfaces;   // they are told where the sun is
    auto recv_surface = [&](float4 color, float rough, Texture* from) {
        MaterialInstance* mi = recvBeamMat->createInstance();
        mi->setParameter("color", color);
        mi->setParameter("roughness", rough);
        mi->setParameter("reflectance", env_f("FLOOR_REFL", 0.35f));
        mi->setParameter("gain", sunColor * (lux / 3.14159265f));
        mi->setParameter("light", from, lin);
        recvSurfaces.push_back(mi);
        return mi;
    };
    scene->addEntity(renderable(floorVB, floorIB, stageOn ? recv_surface(float4{0.86f, 0.85f, 0.84f, 1}, env_f("FLOOR_ROUGH", 0.42f), soft.color) : floorMi, false, true, 8.f));

    // ---- the solids
    // FILMBANDS: the colours (nanometres, from-to) that the films inside a solid mirror, in the order of beam::Solid::films
    static float4 filmNm[4] = {{380, 490, 0, 0}, {500, 590, 0, 0}, {600, 720, 0, 0}, {380, 490, 0, 0}};
    if (const char* fb = std::getenv("FILMBANDS"))
        std::sscanf(fb, "%f-%f,%f-%f,%f-%f,%f-%f", &filmNm[0].x, &filmNm[0].y, &filmNm[1].x, &filmNm[1].y, &filmNm[2].x, &filmNm[2].y, &filmNm[3].x, &filmNm[3].y);
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
        {   // glass with colour films inside (beam::Solid::films): the colour-splitting cube, and the same with its corners cut
            const double q = std::sqrt(0.5), t = std::sqrt(1.0 / 3.0);
            const std::vector<Plane> cross = {{{q, 0, q}, 0}, {{q, 0, -q}, 0}};
            Solid xc = chamfered(box(0.5, 0.5, 0.5), c);
            xc.films = cross;
            add("xcube", xc, {0, -1, 0}, 0.30f);
            std::vector<Plane> cut = box(0.5, 0.5, 0.5).planes;
            for (int i = 0; i < 8; ++i) cut.push_back({{(i & 1 ? t : -t), (i & 2 ? t : -t), (i & 4 ? t : -t)}, (double)env_f("XCUT", 0.62f)});
            Solid xk = from_planes(cut, 4.0);
            // four films, each aslant and none through the middle: much of the light then meets one film only and
            // comes out as white less one colour (yellow, pink, pale blue) instead of as red, green or blue
            xk.films = {{{q, 0, q}, 0.2}, {{q, 0, -q}, 0.2}, {{q, q, 0}, -0.2}, {{0, q, q}, 0.25}};
            if (const char* xf = std::getenv("XFILMS")) {   // XFILMS="nx,ny,nz,o nx,ny,nz,o ...": other films for this solid (up to 4)
                xk.films.clear();
                float a, b, d, o;
                int used = 0;
                while (xk.films.size() < 4 && std::sscanf(xf, " %f,%f,%f,%f%n", &a, &b, &d, &o, &used) == 4) {
                    xk.films.push_back({normalize(V3{a, b, d}), o});
                    xf += used;
                }
            }
            add("xcut", xk, {0, -1, 0}, 0.30f);
        }
        Solid oct = chamfered(octahedron(0.62), c);
        add("octahedron", oct, oct.planes[0].n, 0.30f);
        add("ball", ball(0.5), {0, -1, 0}, 0.30f);
        add("rod", rod(0.30, 0.70), {1, 0, 0}, 0.30f);       // a round bar lying on its side
        add("column", rod(0.34, 0.55), {0, -1, 0}, 0.30f);   // the same shape standing
        add("cone", cone(0.45, 0.95), {0, -1, 0}, 0.30f);
        add("conelying", cone(0.45, 0.95), {1, 0.45 / 0.95, 0}, 0.30f);  // the same shape on its side
        // with dents and holes (the outline lies in the solid's x, y; z is its thickness)
        const Solid starS = star(5, 0.5, 0.225, 0.12, c), washerS = washer(0.5, 0.25, 0.11, c);
        add("star", starS, {0, 0, -1}, 0.30f);           // lying flat
        add("washer", washerS, {0, 0, -1}, 0.30f);
        add("letterA", letter('A', 0.15, c), {0, -1, 0}, 0.30f);   // letters stand
        add("letterB", letter('B', 0.15, c), {0, -1, 0}, 0.30f);
        add("letterO", letter('O', 0.15, c), {0, -1, 0}, 0.30f);
        add("letterM", letter('M', 0.15, c), {0, -1, 0}, 0.30f);
        add("letterR", letter('R', 0.15, c), {0, -1, 0}, 0.30f);
        add("starup", starS, {0, -1, 0}, 0.30f);         // standing on two tips
        add("ring", washerS, {0, -1, 0}, 0.30f);         // the washer on its rim
        if (const char* only = std::getenv("ONLY")) {
            std::vector<Item> keep;
            // one kind, or several with commas between; "holes" = every solid with dents and holes
            const std::string list = std::string(",") + only + ",";
            for (Item& it : items)
                if (list.find(std::string(",") + it.kind + ",") != std::string::npos || (list.find(",holes,") != std::string::npos && it.local.kind == beam::Solid::EXTRUDE))
                    keep.push_back(std::move(it));
            items = std::move(keep);
        }
        const int want = (int)env_f("COUNT", (float)items.size());
        const size_t kinds = items.size();
        if (kinds == 0) return fail("no such kind");
        for (int i = (int)kinds; i < want; ++i) items.push_back(items[(size_t)i % kinds]);  // more of the same kinds
        if ((int)items.size() > want) items.resize((size_t)std::max(1, want));
        // a loose grid, each turned differently (fixed numbers: the same scene every run)
        // (PROPS: matte blocks take some of the places, spread through the grid; without them nothing changes here)
        const int propPlaces = std::getenv("PROPAT") ? 0 : propWant;
        const int n = (int)items.size() + propPlaces, cols = (int)std::ceil(std::sqrt((double)n * 1.4));
        auto place = [&](int at, float& x, float& z) {
            const int cx = at % cols, cz = at / cols;
            x = (cx - 0.5f * (cols - 1)) * 0.62f + ((cz & 1) ? 0.18f : 0.f);
            z = (cz - 0.5f * ((n - 1) / cols)) * 0.62f;
        };
        std::vector<int> propAt;
        for (int j = 0; j < propPlaces; ++j) {
            static const float half[4][3] = {{0.17f, 0.06f, 0.17f}, {0.12f, 0.20f, 0.12f}, {0.22f, 0.09f, 0.10f}, {0.13f, 0.13f, 0.13f}};
            static const float turn[4] = {0.3f, -0.5f, 0.9f, 0.2f};
            // spread over the rows, and never in the last place of a row: a solid then stands beside it on the side the
            // default sun of a stage comes from, and throws its light on it
            const int rows = (n + cols - 1) / cols;
            int cell = std::min(n - 2, ((2 * j + 1) * rows / (2 * propPlaces)) * cols + std::max(0, cols - 2 - j % 2));
            while (cell > 0 && std::find(propAt.begin(), propAt.end(), cell) != propAt.end()) --cell;
            propAt.push_back(cell);
            PropSpec ps{0, 0, half[j][0], half[j][1], half[j][2], turn[j]};
            place(propAt.back(), ps.x, ps.z);
            props.push_back(ps);
        }
        int at = 0;
        for (int i = 0; i < (int)items.size(); ++i, ++at) {
            Item& it = items[(size_t)i];
            while (std::find(propAt.begin(), propAt.end(), at) != propAt.end()) ++at;
            place(at, it.x, it.z);
            it.yaw = 0.35f + 0.83f * (float)i;
            it.spin = (i & 1) ? 1.0f : -0.7f;
            it.drift = 0.05f;
            it.phase = 1.7f * (float)i;
        }
        if (std::getenv("SCATTER")) {
            // strewn about like dice instead of the grid: anywhere in a patch of floor that grows with their number, not
            // touching, each of its own size and turn, flat-faced ones on any of their faces (the same every run; SEED: another throw)
            uint32_t seed = 0x9E3779B9u * (uint32_t)(1 + (int)env_f("SEED", 0.f));
            auto rnd = [&seed]() { seed = seed * 1664525u + 1013904223u; return (float)(seed >> 8) / 16777216.f; };
            const float side = 0.85f * std::sqrt((float)items.size());
            for (int i = 0; i < (int)items.size(); ++i) {
                Item& it = items[(size_t)i];
                it.size = 0.22f + 0.14f * rnd();
                it.yaw = 6.2832f * rnd();
                if (it.local.kind == beam::Solid::FLAT && !it.local.planes.empty()) {
                    const beam::V3 nrm = it.local.planes[(size_t)(rnd() * (float)it.local.planes.size()) % it.local.planes.size()].n;
                    it.down = nrm;
                }
                for (int tries = 0; tries < 200; ++tries) {
                    it.x = (rnd() - 0.5f) * side;
                    it.z = (rnd() - 0.5f) * side;
                    bool clear = true;
                    for (int j = 0; j < i && clear; ++j)
                        clear = std::hypot(it.x - items[(size_t)j].x, it.z - items[(size_t)j].z) > 1.25f * (it.size + items[(size_t)j].size);
                    if (clear) break;
                }
            }
        }
    }
    // ---- the stage (see the top): the room's planes and the blocks. They stand still for the whole run.
    beam::Stage stage;
    std::vector<beam::Block> blocks;
    std::vector<beam::Half> walls;
    const float wallTop = env_f("WALLTOP", 3.f);
    static float4 roomT[6], blocksT[16];
    if (stageOn) {
        float lowX = 1e9f, lowZ = 1e9f;
        for (const Item& it : items) lowX = std::min(lowX, it.x), lowZ = std::min(lowZ, it.z);
        for (const PropSpec& ps : props) {
            beam::Block B;
            B.c = {ps.x, ps.hy, ps.z};
            B.ax[0] = {std::cos(ps.yaw), 0, -std::sin(ps.yaw)};
            B.ax[1] = {0, 1, 0};
            B.ax[2] = {std::sin(ps.yaw), 0, std::cos(ps.yaw)};
            B.h[0] = ps.hx; B.h[1] = ps.hy; B.h[2] = ps.hz;
            blocks.push_back(B);
            lowX = std::min(lowX, ps.x), lowZ = std::min(lowZ, ps.z);
        }
        // the walls stand behind the solids as the default camera sees them: one across (z), one along (x), meeting in
        // a corner that WALLYAW turns about the vertical
        const float gap = env_f("WALLGAP", 0.40f), turn = env_f("WALLYAW", 0.f) * 3.14159265f / 180.f;
        const beam::V3 corner{env_f("WALLX", lowX - gap), 0, env_f("WALLZ", lowZ - gap)};
        const beam::V3 inward[2] = {{std::sin(turn), 0, std::cos(turn)}, {std::cos(turn), 0, -std::sin(turn)}};
        for (int w = 0; w < wallCount; ++w) walls.push_back(beam::Half{inward[w], -beam::dot(inward[w], corner)});
        stage = beam::make_stage(0.0, walls, wallTop, blocks);
        if (stage.room.size() > 6) return fail("too many walls");
        for (size_t k = 0; k < stage.room.size(); ++k)
            roomT[k] = float4{(float)stage.room[k].a.x, (float)stage.room[k].a.y, (float)stage.room[k].a.z, (float)stage.room[k].b};
        for (size_t b = 0; b < blocks.size(); ++b) {
            blocksT[4 * b] = float4{(float)blocks[b].c.x, (float)blocks[b].c.y, (float)blocks[b].c.z, 0};
            for (int k = 0; k < 3; ++k)
                blocksT[4 * b + 1 + (size_t)k] = float4{(float)blocks[b].ax[k].x, (float)blocks[b].ax[k].y, (float)blocks[b].ax[k].z, (float)blocks[b].h[k]};
        }
    }
    // what the materials that drop light on a stage need to know (recv_plane.glsl, recv_lands.glsl)
    std::vector<MaterialInstance*> landers;   // they are told every frame which way the sunlight travels
    auto set_stage = [&](MaterialInstance* mi, bool lands) {
        mi->setParameter("room", roomT, 6);
        mi->setParameter("roomCount", (float)stage.room.size());
        mi->setParameter("blocks", blocksT, 16);
        mi->setParameter("blockCount", (float)blocks.size());
        if (lands) landers.push_back(mi);
    };


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
        on_stage(coneMi[b], true);
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
            .instances(std::max<size_t>(1, count) * (size_t)(coneStepCount * coneWays) * recvKinds)
            .material(0, coneMi[b]).culling(false).castShadows(false).receiveShadows(false).layerMask(0xFF, 0x00).build(*engine, coneE[b]);
        sLight->addEntity(coneE[b]);
    }

    // ---- solids with dents and holes (beam_extrude.h). Built once: the table of shapes (their edges, the stretches
    // their walls are divided into, their triangles) and two meshes that serve every shape. Per frame the CPU only
    // writes where each of these solids is (three texels); two fragment passes (extrude_find.mat, extrude_walk.mat)
    // work out what becomes of the light along every wall, and the strips and outlines are drawn from that
    // (extrude_strip.mat, extrude_cap.mat), instanced per solid and colour step.
    struct ExShapeUse {
        const beam::Outline* ol;
        int line;        // its first line in the table of shapes
        int first = 0;   // the column of its first solid in the table of poses
        int count = 0;   // how many solids have this shape
    };
    std::vector<ExShapeUse> exShapes;
    size_t exCount = 0;
    for (Item& it : items) {
        if (it.local.kind != beam::Solid::EXTRUDE) continue;
        size_t at = 0;
        while (at < exShapes.size() && exShapes[at].ol != it.local.shape.get()) ++at;
        if (at == exShapes.size()) exShapes.push_back(ExShapeUse{it.local.shape.get(), (int)(3 * at)});
        ++exShapes[at].count;
        ++exCount;
    }
    {
        int col = 0;
        for (ExShapeUse& sh : exShapes) {
            sh.first = col;
            int k = 0;
            for (Item& it : items)
                if (it.local.kind == beam::Solid::EXTRUDE && it.local.shape.get() == sh.ol) it.exColumn = col + k++;
            col += sh.count;
        }
    }
    const bool exOnCpu = std::getenv("EXTRUDECPU") != nullptr;  // draw the CPU's copy of their light and shadow instead (to compare)
    const size_t exRoom = std::max<size_t>(1, exCount), exShapeRoom = std::max<size_t>(1, exShapes.size());
    static std::vector<float> exTable(exShapeRoom * 3 * beam::EXT_TABLE * 4, 0.f);
    for (size_t i = 0; i < exShapes.size(); ++i)
        if (!beam::ex_shape_table(*exShapes[i].ol, exTable.data() + i * 3 * beam::EXT_TABLE * 4)) return fail("an outline is too large for the table of shapes");
    Texture* exShapeTex = Texture::Builder().width(beam::EXT_TABLE).height((uint32_t)(3 * exShapeRoom)).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
    exShapeTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(exTable.data(), exTable.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
    Texture* exPoseTex = Texture::Builder().width((uint32_t)exRoom).height(3).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
    Target exFind = make_rt(beam::EXT_COLUMNS, (int)exRoom * beam::EXT_FIND_ROWS, Texture::InternalFormat::RGBA32F);
    Target exWalk = make_rt(beam::EXT_COLUMNS, (int)exRoom * beam::EXT_ROWS, Texture::InternalFormat::RGBA32F);
    const TextureSampler exNearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    std::vector<MaterialInstance*> exMis;  // every instance of the four materials: they all take the sun and the colour steps
    auto ex_instance = [&](Material* m) {
        MaterialInstance* mi = m->createInstance();
        mi->setParameter("shapes", exShapeTex, exNearest);
        mi->setParameter("poses", exPoseTex, exNearest);
        mi->setParameter("floorY", 0.f);
        exMis.push_back(mi);
        return mi;
    };
    MaterialInstance* exFindMi = ex_instance(exFindMat);
    MaterialInstance* exWalkMi = ex_instance(exWalkMat);
    exWalkMi->setParameter("find", exFind.color, exNearest);
    Scene* sExFind = engine->createScene();
    sExFind->addEntity(renderable(sqVB, sqIB, exFindMi, false, false));
    Scene* sExWalk = engine->createScene();
    sExWalk->addEntity(renderable(sqVB, sqIB, exWalkMi, false, false));
    static std::vector<float> exStripV, exCapV;
    static std::vector<uint32_t> exSlimI, exPairI, exFullI, exCapI;
    beam::ex_strip_mesh(exStripV, exSlimI, exPairI, exFullI);
    beam::ex_cap_mesh(exCapV, exCapI);
    auto corner_buffer = [&](std::vector<float>& v) {
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(v.size() / 3)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12).build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(v.data(), v.size() * sizeof(float)));
        return vb;
    };
    auto index_buffer = [&](std::vector<uint32_t>& i) {
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)i.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(i.data(), i.size() * sizeof(uint32_t)));
        return ib;
    };
    VertexBuffer* exStripVB = corner_buffer(exStripV);
    VertexBuffer* exCapVB = corner_buffer(exCapV);
    IndexBuffer* exSlimIB = index_buffer(exSlimI);
    IndexBuffer* exPairIB = index_buffer(exPairI);
    IndexBuffer* exFullIB = index_buffer(exFullI);
    IndexBuffer* exCapIB = index_buffer(exCapI);
    std::vector<utils::Entity> exEntities;
    std::vector<int> exEntityBit;  // which kind of light each one draws (for EXSHOW)
    int exBit = 0;
    const int exShow = (int)env_f("EXSHOW", 255.f);  // debug: a bit per kind of light (1 mirrored, 2 foot, 4 through walls, 8 glint, 16 chamfers, 32 window, 64 shadow)
    size_t exTriangles = 0;  // drawn per frame for these solids (most of them folded up to nothing)
    {
        const bool mirrors = true;  // (beam::Params::outerReflection)
        size_t exPer = 1;  // times as many instances on a stage (set by `cap` and `strip` below)
        auto draw = [&](Scene* into, VertexBuffer* vb, IndexBuffer* ib, size_t indices, MaterialInstance* mi, size_t instances) {
            utils::Entity e = em.create();
            RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
                .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib, 0, indices).instances(instances * exPer)
                .material(0, mi).culling(false).castShadows(false).receiveShadows(false).build(*engine, e);
            into->addEntity(e);
            exEntities.push_back(e);
            exEntityBit.push_back(exBit);
            exTriangles += indices / 3 * instances;
        };
        for (const ExShapeUse& sh : exShapes) {
            const size_t stretches = sh.ol->stretches.size(), tris = sh.ol->tris.size() / 3;
            auto cap = [&](Scene* into, int roleBase, int per) {
                MaterialInstance* mi = ex_instance(exCapMat);
                on_stage(mi, false);
                exPer = recvTotal;
                mi->setParameter("first", (float)sh.first);
                mi->setParameter("per", (float)per);
                mi->setParameter("roleBase", (float)roleBase);
                draw(into, exCapVB, exCapIB, tris * 3, mi, (size_t)sh.count * (size_t)per);
            };
            auto strip = [&](Scene* into, int family, int stepBase, int per) {
                MaterialInstance* mi = ex_instance(exStripMat);
                on_stage(mi, true);
                exPer = recvKinds;
                mi->setParameter("find", exFind.color, exNearest);
                mi->setParameter("walk", exWalk.color, exNearest);
                mi->setParameter("sunTan", std::tan(sunRadiusDeg * 3.14159265f / 180.f));
                mi->setParameter("first", (float)sh.first);
                mi->setParameter("per", (float)per);
                mi->setParameter("stepBase", (float)stepBase);
                mi->setParameter("family", (float)family);
                mi->setParameter("chamfer", (float)sh.ol->chamfer);
                // (slot 0 of the mirrored light is what the walls take out of the window: the window's own shader sees to that)
                const int slots = family == beam::detail::EX_MIRROR ? 2 : beam::detail::ex_slots(family);
                draw(into, exStripVB, slots == 3 ? exFullIB : (slots == 2 ? exPairIB : exSlimIB), stretches * 18 * (size_t)slots, mi, (size_t)sh.count * (size_t)per);
            };
            using namespace beam::detail;
            exBit = 64;
            cap(sShade, 0, 1);
            strip(sShade, EX_SHADOW, 0, 1);
            exBit = 32;
            cap(sLight, 1, mirrors ? 2 : 1);  // the window, and the sun mirrored off the cap
            exBit = 1;
            strip(sLight, EX_MIRROR, 0, beam::EXT_CAP_STEPS);
            exBit = 2;
            strip(sLight, EX_FOOT, 0, beam::EXT_STEPS);
            exBit = 4;
            strip(sLight, EX_WALL, 0, beam::EXT_STEPS);
            exBit = 8;
            if (mirrors) strip(sLight, EX_GLINT, beam::EXT_STEPS / 2, 1);
            exBit = 16;
            strip(sLight, EX_CUT, 0, 2 * beam::EXT_STEPS);
        }
    }

    // ---- the stage's own drawing. Every material that puts light or an outline on the floor draws it once per receiver
    // (recv_drop.glsl: the floor, each wall, three faces per block), see `on_stage` above. What the CPU still makes
    // as triangles (only with FLATCPU, OUTLINECPU, ...) and the blocks' own outlines go through recv_light.mat, which
    // does the same for a vertex buffer. All of it goes into the same two pictures as before (light, shade).
    utils::Entity stageLightE[2], stageShadeE[2];   // the CPU's triangles, light and outlines: [0] on the room's planes, [1] on the blocks' faces
    if (stageOn) {
        auto recv_tris = [&](VertexBuffer* vb, IndexBuffer* ib, float source, float cover, int first, int count) {
            MaterialInstance* mi = recvLightMat->createInstance();
            mi->setParameter("source", source);
            mi->setParameter("cover", cover);
            mi->setParameter("floorY", 0.f);
            mi->setParameter("recvFirst", (float)first);
            set_stage(mi, true);
            if (first == 0 && blocksOn) mi->setDepthCulling(true);
            utils::Entity e = em.create();
            RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}}).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
                .instances((size_t)count).material(0, mi).culling(false).castShadows(false).receiveShadows(false).build(*engine, e);
            return e;
        };
        const int faces = 3 * (int)blocks.size();   // (recv_light.mat: a block shows the sun three faces at most)
        stageLightE[0] = recv_tris(lightVB, lightIB, 0.f, 0.f, 0, roomPlanes);
        stageShadeE[0] = recv_tris(lightVB, lightIB, 1.f, 1.f, 0, roomPlanes);
        sLight->addEntity(stageLightE[0]);
        sShade->addEntity(stageShadeE[0]);
        if (blocksOn) {
            stageLightE[1] = recv_tris(lightVB, lightIB, 0.f, 0.f, roomPlanes, faces);
            stageShadeE[1] = recv_tris(lightVB, lightIB, 1.f, 1.f, roomPlanes, faces);
            sLight->addEntity(stageLightE[1]);
            sShade->addEntity(stageShadeE[1]);
        }
        // the softening pass: told the spread in pixels by whoever drew the light, and it hands light and shade on apart
        softenMi->setParameter("unit", 1.f);
        softenMi->setParameter("split", 1.f);
        softenMi->setParameter("marks", blocksOn ? 1.f : 0.f);

        // the walls
        for (const beam::Half& w : walls) {
            auto* v = new std::vector<float>();
            auto* idx = new std::vector<uint32_t>{0, 2, 1, 0, 3, 2};   // (its front is the room's side)
            const float F = env_f("FLOOR", 6.f);
            const beam::V3 along = beam::normalize(beam::cross(w.a, beam::V3{0, 1, 0})), foot = w.a * -w.b;
            const float3 n{(float)w.a.x, (float)w.a.y, (float)w.a.z}, u{(float)along.x, (float)along.y, (float)along.z}, p0{(float)foot.x, 0.f, (float)foot.z};
            for (float3 p : {p0 - u * F, p0 + u * F, p0 + u * F + float3{0, wallTop, 0}, p0 - u * F + float3{0, wallTop, 0}}) push_vert(*v, p, u, n);
            auto [vb, ib] = lit_buffers(*v, *idx);
            scene->addEntity(renderable(vb, ib, recv_surface(float4{0.86f, 0.85f, 0.84f, 1}, env_f("WALL_ROUGH", 0.55f), soft.color), false, true, 12.f));
        }
        // the blocks: as surfaces in the picture, and flat in the picture of shade, each with its own mark
        // (recv_mask.mat), so that the softening pass can tell a block from what is seen behind it
        static const float4 tint[4] = {{0.90f, 0.86f, 0.80f, 1}, {0.78f, 0.83f, 0.88f, 1}, {0.82f, 0.87f, 0.80f, 1}, {0.89f, 0.81f, 0.81f, 1}};
        auto* outlineV = new std::vector<float>();
        for (size_t b = 0; b < blocks.size(); ++b) {
            const beam::Block& B = blocks[b];
            auto* v = new std::vector<float>();
            auto* idx = new std::vector<uint32_t>();
            for (int i = 0; i < 3; ++i)
                for (int sgn = -1; sgn <= 1; sgn += 2) {
                    const int j = (i + 1) % 3, k = (i + 2) % 3;
                    const beam::V3 n = B.ax[i] * (double)sgn, mid = B.c + n * B.h[i], a = B.ax[j] * B.h[j], c = B.ax[k] * B.h[k];
                    const beam::V3 q[4] = {mid - a - c, mid + a - c, mid + a + c, mid - a + c};
                    const uint32_t base = (uint32_t)(v->size() / 7);
                    for (beam::V3 p : q)
                        push_vert(*v, float3{(float)p.x, (float)p.y, (float)p.z}, float3{(float)B.ax[j].x, (float)B.ax[j].y, (float)B.ax[j].z}, float3{(float)n.x, (float)n.y, (float)n.z});
                    if (sgn > 0) idx->insert(idx->end(), {base, base + 1, base + 2, base, base + 2, base + 3});
                    else idx->insert(idx->end(), {base, base + 2, base + 1, base, base + 3, base + 2});   // (the front is the outside)
                    // and for the block's outline against the sun (see below): position; the face's normal, the block's number
                    for (int t : {0, 1, 2, 0, 2, 3}) outlineV->insert(outlineV->end(), {(float)q[t].x, (float)q[t].y, (float)q[t].z, (float)n.x, (float)n.y, (float)n.z, (float)b});
                }
            auto [vb, ib] = lit_buffers(*v, *idx);
            scene->addEntity(renderable(vb, ib, recv_surface(tint[b % 4], env_f("PROP_ROUGH", 0.7f), soft.color), false, true, 12.f));
            MaterialInstance* mark = recvMaskMat->createInstance();
            mark->setParameter("mark", float4{0, (float)(b + 1) / 8.f, 0, 0});
            sShade->addEntity(renderable(vb, ib, mark, false, false, 12.f));
            MaterialInstance* depthOnly = recvMaskMat->createInstance();   // (in the picture of light: nothing but depth)
            depthOnly->setParameter("mark", float4{0, 0, 0, 0});
            depthOnly->setColorWrite(false);
            sLight->addEntity(renderable(vb, ib, depthOnly, false, false, 12.f));
        }
        // the blocks' own outlines against the sun: their faces as they stand, made once; recv_light.mat slides the
        // ones that face the sun along it onto every receiver
        if (!blocks.empty()) {
            const uint32_t count = (uint32_t)(outlineV->size() / 7);
            VertexBuffer* vb = VertexBuffer::Builder().vertexCount(count).bufferCount(1)
                                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                                   .attribute(VertexAttribute::COLOR, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28).build(*engine);
            vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(outlineV->data(), outlineV->size() * sizeof(float)));
            IndexBuffer* ib = IndexBuffer::Builder().indexCount(count).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
            ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(light_i.data(), count * sizeof(uint32_t)));
            sShade->addEntity(recv_tris(vb, ib, 2.f, 1.f, 0, roomPlanes));
            sShade->addEntity(recv_tris(vb, ib, 2.f, 1.f, roomPlanes, faces));
        }
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
    const int rainbowParts = rainbowOnce ? std::min(7, std::max(1, (int)env_f("RAINBOW1", 1.f))) : 0;   // RAINBOW1=k: the spectrum in k stretches
    const int flatLines = rainbowOnce ? 2 * rainbowParts + 1 : flatSteps;   // lines per solid in the walk table (RAINBOW1: two ends per stretch, mirrored)
    const int flatDraws = rainbowOnce ? rainbowParts + 1 : flatSteps;       // times every piece is drawn (RAINBOW1: once per stretch, and once mirrored)
    struct FlatKind {
        std::string name;
        beam::FlatShape shape;
        std::vector<int> solids;  // the items of this shape
        MaterialInstance* mi = nullptr;
        MaterialInstance* mi2 = nullptr;   // EDGES: the pieces of the walk's second turn
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
    // EDGES=1: the pieces of light are cut where their light starts to go another way (flat_walk.mat, clip), so their
    // edges are true lines however coarse the pieces; the walk is then done twice (the second time for the far side
    // of such a line), into two tables of four texels per piece.
    // ROUTES=1 (experiment, route_light.mat): one shape per way the light goes. The pieces only serve to find the ways
    // (a few per face are enough: PIECES=4); nothing is cut along their grid, so EDGES has no part in it.
    // BEAMS=1: the light of the flat-faced solids by splitting beams exactly, on the CPU (beam_split.h). Every patch
    // comes out as its own outline; nothing is found by rays and nothing is checked per pixel. BEAMSTAT=1 counts.
    const bool flatBeams = env_f("BEAMS", 0.f) > 0.5f;
    static std::vector<std::array<beam::SplitShape, 16>> beamShapes;   // per shape, per set of films
    struct BeamPose {
        beam::V3 R[3], at;
        double s;
    };
    static std::vector<BeamPose> beamPose;
    const bool flatRoutes = env_f("ROUTES", 0.f) > 0.5f && !rainbowOnce && !flatBeams;
    const bool flatClip = env_f("EDGES", 0.f) > 0.5f && !rainbowOnce && !flatRoutes;
    const int flatPer = (flatClip || flatRoutes) ? 4 : 3;
    std::vector<MaterialInstance*> routeMis;
    // ROUTES=2: the ways are not found by rays but worked out from where the faces and films are, level by level
    // (route_tree.mat), and every leaf is drawn by route_draw.mat. ROUTESTAT=1 counts the leaves and their light.
    const bool routeTree = flatRoutes && env_f("ROUTES", 0.f) > 1.5f;
    struct RouteLevel {
        Target table;
        View* view;
        int slots, rows;
    };
    static std::vector<RouteLevel> routeLevels;
    if (flatMostPieces * flatPer > 16384) return fail("too many pieces for one line of the walk table (PIECES is too high)");
    Target flatWalk = make_rt(flatMostPieces * flatPer, (int)items.size() * flatLines, Texture::InternalFormat::RGBA32F);
    Target flatWalk2 = flatClip ? make_rt(flatMostPieces * flatPer, (int)items.size() * flatLines, Texture::InternalFormat::RGBA32F) : Target{nullptr, nullptr};
    const TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    MaterialInstance* flatWalkMi = flatWalkMat->createInstance();
    MaterialInstance* flatWalk2Mi = flatClip ? flatWalkMat->createInstance() : nullptr;
// what the walk is told goes to both of its turns
#define FLAT_WALK_SET(...) do { flatWalkMi->setParameter(__VA_ARGS__); if (flatWalk2Mi) flatWalk2Mi->setParameter(__VA_ARGS__); } while (0)
    if (!rainbowOnce) {
        flatWalkMi->setParameter("clip", flatRoutes ? 3.f : (flatClip ? 1.f : 0.f));
        flatWalkMi->setParameter("first", flatPieces, nearest);   // (not read on the first turn)
        if (flatWalk2Mi) {
            flatWalk2Mi->setParameter("clip", 2.f);
            flatWalk2Mi->setParameter("first", flatWalk.color, nearest);
        }
    }
    FLAT_WALK_SET("pose", poseTex, nearest);
    FLAT_WALK_SET("pieces", flatPieces, nearest);
    FLAT_WALK_SET("scene", sceneTex, nearest);
    FLAT_WALK_SET("solidCount", (int32_t)items.size());
    FLAT_WALK_SET("stepCount", (float)flatLines);
    FLAT_WALK_SET("floorY", 0.f);
    Scene* sFlatWalk = engine->createScene();
    sFlatWalk->addEntity(renderable(sqVB, sqIB, flatWalkMi, false, false));
    Scene* sFlatWalk2 = engine->createScene();
    if (flatWalk2Mi) sFlatWalk2->addEntity(renderable(sqVB, sqIB, flatWalk2Mi, false, false));
    for (FlatKind& fk : flatKinds) {
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(fk.shape.corners.size() / 5)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                               .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(fk.shape.corners.data(), fk.shape.corners.size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)fk.shape.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(fk.shape.idx.data(), fk.shape.idx.size() * sizeof(uint32_t)));
        float4 which[16] = {};
        for (size_t i = 0; i < fk.solids.size() && i < 64; ++i) which[i / 4][i % 4] = (float)fk.solids[i];
        for (int turn = 0; turn < (flatClip ? 2 : 1); ++turn) {   // (EDGES: once more, from the walk's second table)
            MaterialInstance* mi = flatMat->createInstance();
            on_stage(mi, true);
            mi->setParameter("walk", turn == 0 ? flatWalk.color : flatWalk2.color, nearest);
            mi->setParameter("stepCount", (float)flatLines);
            mi->setParameter("hull", flatHull, nearest);
            mi->setParameter("shape", (float)(&fk - flatKinds.data()));
            mi->setParameter("floorY", 0.f);
            if (!rainbowOnce) mi->setParameter("clip", flatClip ? 1.f : 0.f);
            mi->setParameter("solids", which, 16);
            const utils::Entity e = em.create();
            RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
                .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
                .instances(fk.solids.size() * (size_t)flatDraws * recvKinds)
                .material(0, mi).culling(false).castShadows(false).receiveShadows(false)
                .layerMask(0xFF, (flatOnCpu || flatRoutes || flatBeams || std::getenv("NOLIGHT")) ? 0x00 : 0x01).build(*engine, e);
            sLight->addEntity(e);
            if (turn == 0) { fk.mi = mi; fk.entity = e; } else fk.mi2 = mi;
            if (turn == 1 && std::getenv("XDRAW2")) rm.setLayerMask(rm.getInstance(e), 0xFF, 0x00);   // (XDRAW2: timing only, its pieces not drawn)
            if (turn == 0 && std::getenv("XDRAW1")) rm.setLayerMask(rm.getInstance(e), 0xFF, 0x00);   // (XDRAW1: to look at the second turn alone)
        }
    }
    if (flatBeams) {
        for (FlatKind& fk : flatKinds) {
            beamShapes.emplace_back();
            for (int use = 0; use < 16; ++use) beamShapes.back()[(size_t)use] = beam::split_shape(items[(size_t)fk.solids[0]].local, use);
            std::printf("  beams: %s, all its films together cut it into %zu cells\n", fk.name.c_str(), beamShapes.back()[15].cells.size());
        }
        beamPose.resize(items.size());
    }
    if (routeTree) {
        Material* treeMat = mat("route_tree.filamat");
        Material* drawMat = mat("route_draw.filamat");
        if (!treeMat || !drawMat) return fail("route_tree.filamat / route_draw.filamat");
        static const int fans[] = {6, 4, 3, 2, 2};   // how many things an entry of the level before may go on to meet
        const int levels = std::min(5, std::max(1, (int)env_f("ROUTEDEPTH", 5.f)));
        for (FlatKind& fk : flatKinds) {
            const beam::Solid& loc = items[(size_t)fk.solids[0]].local;
            float4 planes[40] = {}, films[4] = {};
            int planeCount = 0;
            auto* cornerTexels = new std::vector<float>((size_t)8 * 40 * 4, 0.f);
            for (size_t f = 0; f < loc.planes.size() && planeCount < 40; ++f) {   // (the order of the table of faces)
                if (loc.faces[f].n < 3) continue;
                planes[planeCount] = float4{(float)loc.planes[f].n.x, (float)loc.planes[f].n.y, (float)loc.planes[f].n.z, (float)loc.planes[f].o};
                const int cn = std::min(8, loc.faces[f].n);
                for (int k = 0; k < cn; ++k) {
                    float* o = cornerTexels->data() + ((size_t)planeCount * 8 + (size_t)k) * 4;
                    o[0] = (float)loc.faces[f].p[(size_t)k].x; o[1] = (float)loc.faces[f].p[(size_t)k].y; o[2] = (float)loc.faces[f].p[(size_t)k].z;
                    o[3] = (float)cn;
                }
                ++planeCount;
            }
            const int filmCount = (int)std::min<size_t>(4, loc.films.size());
            for (int f = 0; f < filmCount; ++f) films[f] = float4{(float)loc.films[(size_t)f].n.x, (float)loc.films[(size_t)f].n.y, (float)loc.films[(size_t)f].n.z, (float)loc.films[(size_t)f].o};
            Texture* cornerTex = Texture::Builder().width(8).height(40).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
            cornerTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(cornerTexels->data(), cornerTexels->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                           [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, cornerTexels));
            float4 which[16] = {};
            for (size_t i = 0; i < fk.solids.size() && i < 64; ++i) which[i / 4][i % 4] = (float)fk.solids[i];
            const int rows = (int)fk.solids.size() * flatLines * planeCount;
            if (rows > 16384) return fail("ROUTES=2: too many solids of one shape for one table");
            auto common = [&](MaterialInstance* mi) {
                mi->setParameter("pose", poseTex, nearest);
                mi->setParameter("corners", cornerTex, nearest);
                mi->setParameter("planes", planes, 40);
                mi->setParameter("planeCount", (int32_t)planeCount);
                mi->setParameter("films", films, 4);
                mi->setParameter("filmCount", (int32_t)filmCount);
                mi->setParameter("stepCount", (float)flatLines);
                mi->setParameter("solids", which, 16);
                routeMis.push_back(mi);
            };
            int slots = 1;
            Texture* before = cornerTex;
            for (int lv = 1; lv <= levels; ++lv) {
                slots *= fans[lv - 1];
                RouteLevel rl{make_rt(slots * 2, rows, Texture::InternalFormat::RGBA32F), engine->createView(), slots, rows};
                MaterialInstance* tmi = treeMat->createInstance();
                common(tmi);
                tmi->setParameter("prev", before, nearest);
                tmi->setParameter("solidsHere", (float)fk.solids.size());
                tmi->setParameter("level", (float)lv);
                tmi->setParameter("fan", (float)fans[lv - 1]);
                tmi->setParameter("last", lv == levels ? 1.f : 0.f);
                Scene* sc = engine->createScene();
                sc->addEntity(renderable(sqVB, sqIB, tmi, false, false));
                rl.view->setScene(sc);
                rl.view->setRenderTarget(rl.table.rt);
                rl.view->setViewport({0, 0, (uint32_t)(slots * 2), (uint32_t)rows});
                rl.view->setDithering(View::Dithering::NONE);
                rl.view->setPostProcessingEnabled(false);
                rl.view->setShadowingEnabled(false);
                routeLevels.push_back(rl);
                before = rl.table.color;
                // its leaves: four corners per entry
                auto* v = new std::vector<float>();
                auto* ix = new std::vector<uint32_t>();
                for (int i = 0; i < slots; ++i) {
                    const uint32_t base = (uint32_t)(v->size() / 3);
                    for (int k = 0; k < 4; ++k) v->insert(v->end(), {(float)i, (float)(k & 1), (float)(k >> 1)});
                    ix->insert(ix->end(), {base, base + 1, base + 3, base, base + 3, base + 2});
                }
                VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(v->size() / 3)).bufferCount(1)
                                       .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12).build(*engine);
                vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(v->data(), v->size() * sizeof(float)));
                IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)ix->size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
                ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(ix->data(), ix->size() * sizeof(uint32_t)));
                MaterialInstance* dmi = drawMat->createInstance();
                common(dmi);
                dmi->setParameter("tree", rl.table.color, nearest);
                dmi->setParameter("floorY", 0.f);
                const utils::Entity e = em.create();
                RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
                    .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
                    .instances((size_t)rows)
                    .material(0, dmi).culling(false).castShadows(false).receiveShadows(false)
                    .layerMask(0xFF, std::getenv("NOLIGHT") ? 0x00 : 0x01).build(*engine, e);
                sLight->addEntity(e);
            }
            std::printf("  routes: %s, %d faces, %d films, %d levels, %d entries in the last, %d lines\n", fk.name.c_str(), planeCount, filmCount, levels, slots, rows);
        }
    } else if (flatRoutes) {
        Material* routeMat = mat("route_light.filamat");
        if (!routeMat) return fail("route_light.filamat");
        for (FlatKind& fk : flatKinds) {
            using beam::V3;
            const beam::Solid& loc = items[(size_t)fk.solids[0]].local;
            float4 planes[40] = {}, films[4] = {};
            int planeCount = 0;
            for (size_t f = 0; f < loc.planes.size() && planeCount < 40; ++f)   // (the order of the table of faces)
                if (loc.faces[f].n >= 3) planes[planeCount++] = float4{(float)loc.planes[f].n.x, (float)loc.planes[f].n.y, (float)loc.planes[f].n.z, (float)loc.planes[f].o};
            const int filmCount = (int)std::min<size_t>(4, loc.films.size());
            for (int f = 0; f < filmCount; ++f) films[f] = float4{(float)loc.films[(size_t)f].n.x, (float)loc.films[(size_t)f].n.y, (float)loc.films[(size_t)f].n.z, (float)loc.films[(size_t)f].o};
            // the corners of every face, for route_light.mat
            std::vector<int> tableOf(loc.planes.size(), -1);
            auto* cornerTexels = new std::vector<float>((size_t)8 * 40 * 4, 0.f);
            for (size_t f = 0, row = 0; f < loc.planes.size() && row < 40; ++f) {
                if (loc.faces[f].n < 3) continue;
                tableOf[f] = (int)row;
                const int cn = std::min(8, loc.faces[f].n);
                for (int k = 0; k < cn; ++k) {
                    float* o = cornerTexels->data() + (row * 8 + (size_t)k) * 4;
                    o[0] = (float)loc.faces[f].p[(size_t)k].x; o[1] = (float)loc.faces[f].p[(size_t)k].y; o[2] = (float)loc.faces[f].p[(size_t)k].z;
                    o[3] = (float)cn;
                }
                ++row;
            }
            Texture* cornerTex = Texture::Builder().width(8).height(40).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
            cornerTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(cornerTexels->data(), cornerTexels->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                           [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, cornerTexels));
            // per piece four corners (the material puts them round the light of the way that piece finds)
            auto* v = new std::vector<float>();
            auto* ix = new std::vector<uint32_t>();
            int firstOfFace = 0, lastFace = -2;
            for (int i = 0; i < fk.shape.pieces; ++i) {
                const float* tb = fk.shape.table.data() + (size_t)i * 20;
                const V3 nf{tb[4], tb[5], tb[6]}, c0{tb[8], tb[9], tb[10]};
                int face = -1;
                for (size_t f = 0; f < loc.planes.size(); ++f)
                    if (loc.faces[f].n >= 3 && beam::dot(loc.planes[f].n, nf) > 0.99999 && std::fabs(beam::dot(loc.planes[f].n, c0) - loc.planes[f].o) < 1e-5) face = (int)f;
                if (face != lastFace) { firstOfFace = i; lastFace = face; }
                if (face < 0) continue;
                const uint32_t base = (uint32_t)(v->size() / 5);
                for (int k = 0; k < 4; ++k) v->insert(v->end(), {(float)i, (float)(k & 1), (float)(k >> 1), (float)firstOfFace, (float)tableOf[(size_t)face]});
                ix->insert(ix->end(), {base, base + 1, base + 3, base, base + 3, base + 2});
            }
            VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(v->size() / 5)).bufferCount(1)
                                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                                   .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*engine);
            vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(v->data(), v->size() * sizeof(float)));
            IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)ix->size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
            ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(ix->data(), ix->size() * sizeof(uint32_t)));
            MaterialInstance* mi = routeMat->createInstance();
            mi->setParameter("walk", flatWalk.color, nearest);
            mi->setParameter("pieces", flatPieces, nearest);
            mi->setParameter("pose", poseTex, nearest);
            mi->setParameter("corners", cornerTex, nearest);
            mi->setParameter("loose", env_f("ROUTELOOSE", 0.f));
            mi->setParameter("shape", (float)(&fk - flatKinds.data()));
            mi->setParameter("stepCount", (float)flatLines);
            mi->setParameter("planes", planes, 40);
            mi->setParameter("planeCount", (int32_t)planeCount);
            mi->setParameter("films", films, 4);
            mi->setParameter("filmCount", (int32_t)filmCount);
            mi->setParameter("floorY", 0.f);
            float4 which[16] = {};
            for (size_t i = 0; i < fk.solids.size() && i < 64; ++i) which[i / 4][i % 4] = (float)fk.solids[i];
            mi->setParameter("solids", which, 16);
            const utils::Entity e = em.create();
            RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
                .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
                .instances(fk.solids.size() * (size_t)flatLines)
                .material(0, mi).culling(false).castShadows(false).receiveShadows(false)
                .layerMask(0xFF, std::getenv("NOLIGHT") ? 0x00 : 0x01).build(*engine, e);
            sLight->addEntity(e);
            routeMis.push_back(mi);
            std::printf("  routes: %s, %d pieces to find the ways, %zu corners in the mesh\n", fk.name.c_str(), fk.shape.pieces, v->size() / 5);
        }
    }
    static std::vector<std::vector<float>> mesh_v(items.size());
    static std::vector<std::vector<uint32_t>> mesh_i(items.size());
    // ---- VBODY=1 (experiment): the four-sided faces of flat-faced solids are not followed pixel by pixel. A table pass
    // (glass_nodes.mat) works out, at 17 x 17 points per face, where the sight line through the glass ends up; the faces are
    // drawn as a net of cells from that (glass_vert.mat), and only the cells the table cannot vouch for are followed per pixel.
    static const bool vbodyOn = std::getenv("VBODY") != nullptr && !stageOn;
    Material* glassVertMat = vbodyOn ? mat("glass_vert.filamat") : nullptr;
    Material* glassNodesMat = vbodyOn ? mat("glass_nodes.filamat") : nullptr;
    Material* glassCellsMat = vbodyOn ? mat("glass_cells.filamat") : nullptr;
    if (vbodyOn && (!glassVertMat || !glassNodesMat || !glassCellsMat)) return fail("VBODY materials");
    static std::vector<float> faceTable((size_t)24 * SCENE_H * 4, 0.f);   // per solid (line), per face: a corner (+1: in use), its two neighbours, the normal
    Texture* faceTex = Texture::Builder().width(24).height(SCENE_H).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
    Target nodesRT = make_rt(289, std::max(1, (int)items.size()) * 6, Texture::InternalFormat::RGBA32F);
    Target cellsRT = make_rt(80, std::max(1, (int)items.size()) * 6, Texture::InternalFormat::RGBA16F);   // which cells pass (glass_cells.mat)
    Scene* sCells = engine->createScene();
    MaterialInstance* nodesMi = nullptr;
    Scene* sNodes = engine->createScene();
    static std::vector<float> gridSoup;   // the net, the same for every solid: (face, level, cell), which corner
    if (vbodyOn) {
        for (int face = 0; face < 6; ++face)
            for (int level = 0; level < 7; ++level) {
                const int per = level == 0 ? 4 : 8;
                for (int cell = 0; cell < per * per; ++cell)
                    for (const int* c : {(const int[]){0, 0}, (const int[]){1, 0}, (const int[]){1, 1}, (const int[]){0, 1}})   // four corners a cell
                        gridSoup.insert(gridSoup.end(), {(float)face, (float)level, (float)cell, (float)c[0], (float)c[1]});
            }
    }
    for (size_t k = 0; k < items.size(); ++k) {
        Item& it = items[k];
        const beam::Solid& unit = it.local;
        float reach = 0;
        if (unit.kind != beam::Solid::FLAT && unit.kind != beam::Solid::EXTRUDE) {
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
        if (unit.kind == beam::Solid::EXTRUDE) {
            // caps, walls and chamfers as flat triangles (the material puts the walls of arcs on their true curve)
            std::vector<beam::V3> pos, nrm;
            beam::extrude_mesh(*unit.shape, pos, nrm);
            for (size_t i = 0; i + 2 < pos.size(); i += 3) {
                const float3 a{(float)pos[i].x, (float)pos[i].y, (float)pos[i].z}, b{(float)pos[i + 1].x, (float)pos[i + 1].y, (float)pos[i + 1].z},
                    c{(float)pos[i + 2].x, (float)pos[i + 2].y, (float)pos[i + 2].z}, N{(float)nrm[i].x, (float)nrm[i].y, (float)nrm[i].z};
                const uint32_t base = (uint32_t)(mesh_v[k].size() / 7);
                const float3 T = normalize(b - a);
                for (float3 p : {a, b, c}) {
                    push_vert(mesh_v[k], p, T, N);
                    reach = std::max(reach, length(p));
                }
                mesh_i[k].insert(mesh_i[k].end(), {base, base + 1, base + 2});
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
        if (vbodyOn && unit.kind == beam::Solid::FLAT) {
            // its four-sided faces (up to six of them) go into the table the cheap layer is drawn from
            int used = 0;
            for (size_t f = 0; f < unit.faces.size(); ++f) {
                const beam::Poly& P = unit.faces[f];
                if (P.n != 4 || used >= 6) continue;
                if (beam::length(P.p[0] + P.p[2] - P.p[1] - P.p[3]) > 1e-6 * (beam::length(P.p[2] - P.p[0]) + 1e-30)) continue;
                const beam::V3 nf = unit.planes[f].n;
                beam::V3 c0 = P.p[0], c1 = P.p[1], c2 = P.p[3];
                if (beam::dot(beam::cross(c1 - c0, c2 - c0), nf) < 0) std::swap(c1, c2);   // counter-clockwise seen from outside
                float* o = faceTable.data() + (k * 24 + (size_t)used * 4) * 4;
                const float e[16] = {(float)c0.x, (float)c0.y, (float)c0.z, 1.f, (float)c1.x, (float)c1.y, (float)c1.z, 0.f,
                                     (float)c2.x, (float)c2.y, (float)c2.z, 0.f, (float)nf.x, (float)nf.y, (float)nf.z, 0.f};
                std::copy(e, e + 16, o);
                ++used;
            }
            it.vbody = used > 0;
        }
        // STDGLASS=1, measuring only: draw the body with Filament's own screen-space refraction (std_glass.mat) instead
        static Material* stdGlassMat = std::getenv("STDGLASS") ? mat("std_glass.filamat") : nullptr;
        MaterialInstance* drawn = it.glass;
        if (it.vbody && (std::getenv("VSHOW") || std::getenv("VREDOLD"))) {   // (VREDOLD: timing only, the old way behind costs nothing)
            static Material* redMat = mat("flat_red.filamat");
            if (redMat) drawn = redMat->createInstance();
        }
        if (stdGlassMat) {
            drawn = stdGlassMat->createInstance();
            drawn->setParameter("ior", nd);
            drawn->setParameter("thickness", (float)it.size * env_f("STDTHICK", 1.f));
            drawn->setParameter("dispersion", env_f("STDDISP", 20.f / 64.17f * disp));
        }
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
        {
            float4 films[4] = {};
            const int nf = it.local.kind == beam::Solid::FLAT ? (int)std::min<size_t>(4, it.local.films.size()) : 0;
            for (int f = 0; f < nf; ++f) films[f] = float4{(float)it.local.films[(size_t)f].n.x, (float)it.local.films[(size_t)f].n.y, (float)it.local.films[(size_t)f].n.z, (float)it.local.films[(size_t)f].o};
            it.glass->setParameter("films", films, 4);
            it.glass->setParameter("filmNm", filmNm, 4);
            it.glass->setParameter("filmCount", (int32_t)nf);
        }
        it.glass->setParameter("kind", (int32_t)unit.kind);
        it.glass->setParameter("dims", float4{(float)unit.r, (float)unit.h, 0, 0});
        it.glass->setParameter("scene", sceneTex, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST));
        it.glass->setParameter("shapes", exShapeTex, exNearest);
        it.glass->setParameter("solidCount", (int32_t)(std::getenv("ALONE") ? 0 : items.size()));
        it.glass->setParameter("self", (int32_t)k);
        it.glass->setParameter("nearCone", env_f("NEARCONE", 0.8f));  // which solids are asked about being seen through this one (glass_solid.mat, `near`; 0: all)
        it.glass->setParameter("spread", env_f("SPREAD", 2.f));  // colours part only at the last face; this makes up for the others
        it.glass->setParameter("floorY", 0.f);
        it.glass->setParameter("absorption", float3{0.9f, 0.35f, 0.5f} * env_f("ABSORB", 0.15f));
        it.glass->setParameter("backdrop", backdrop.color, lin);
        if (stageOn) {  // what else a sight line through the glass can end on
            it.glass->setParameter("room", roomT, 6);
            it.glass->setParameter("roomCount", (float)stage.room.size());
            it.glass->setParameter("blocks", blocksT, 16);
            it.glass->setParameter("blockCount", (float)blocks.size());
        }
        // drawn by its own material and left out of the backdrop picture (layer 1)
        it.entity = em.create();
        RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {reach, reach, reach}}).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, drawn).culling(false).castShadows(false).receiveShadows(false).layerMask(0xFF, 0x02).build(*engine, it.entity);
        tm.create(it.entity);
        if (!std::getenv("NOGLASS")) scene->addEntity(it.entity);
        if (it.vbody) {
            static VertexBuffer* gridVB = nullptr;
            static IndexBuffer* gridIB = nullptr;
            if (!gridVB) {
                const uint32_t count = (uint32_t)(gridSoup.size() / 5);
                gridVB = VertexBuffer::Builder().vertexCount(count).bufferCount(1)
                             .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                             .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*engine);
                gridVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(gridSoup.data(), gridSoup.size() * sizeof(float)));
                static std::vector<uint32_t> gridIdx;
                for (uint32_t q = 0; q + 3 < count; q += 4) gridIdx.insert(gridIdx.end(), {q, q + 1, q + 2, q, q + 2, q + 3});
                gridIB = IndexBuffer::Builder().indexCount((uint32_t)gridIdx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
                gridIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(gridIdx.data(), gridIdx.size() * sizeof(uint32_t)));
            }
            const TextureSampler ns(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
            MaterialInstance* c = it.cheap = glassVertMat->createInstance();
            c->setParameter("ior", nd);
            c->setParameter("abbe", 64.17f / disp);
            c->setParameter("planes", planes.data(), 40);
            c->setParameter("planeCount", count);
            c->setParameter("kind", (int32_t)0);
            c->setParameter("dims", float4{0, 0, 0, 0});
            c->setParameter("scene", sceneTex, ns);
            c->setParameter("shapes", exShapeTex, exNearest);
            c->setParameter("solidCount", (int32_t)0);
            c->setParameter("self", (int32_t)k);
            c->setParameter("nearCone", 0.f);
            c->setParameter("spread", env_f("SPREAD", 2.f));
            c->setParameter("floorY", 0.f);
            c->setParameter("absorption", float3{0.9f, 0.35f, 0.5f} * env_f("ABSORB", 0.15f));
            c->setParameter("backdrop", backdrop.color, lin);
            c->setParameter("nodes", nodesRT.color, ns);
            c->setParameter("faces", faceTex, ns);
            c->setParameter("tol", env_f("VTOL", 1.f));
            c->setParameter("levels", env_f("VLEVELS", 6.f));
            c->setParameter("cells", cellsRT.color, ns);
            c->setParameter("slop", math::float2{env_f("VSLOP", 0.f), env_f("VSLOPT", 0.f)});
            c->setParameter("showCells", env_f("VSHOW", 0.f));
            utils::Entity ce = em.create();
            RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}}).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, gridVB, gridIB)
                .material(0, c).culling(false).castShadows(false).receiveShadows(false).priority(2).layerMask(0xFF, 0x02).build(*engine, ce);
            if (!std::getenv("NOGLASS") && !std::getenv("VNOCHEAP")) scene->addEntity(ce);
        }
    }
    (void)shadowMat;
    (void)filamentShadow;
    if (vbodyOn) {
        faceTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(faceTable.data(), faceTable.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
        const TextureSampler ns(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
        nodesMi = glassNodesMat->createInstance();
        nodesMi->setParameter("pose", poseTex, ns);
        nodesMi->setParameter("faces", faceTex, ns);
        nodesMi->setParameter("scene", sceneTex, ns);
        nodesMi->setParameter("solidCount", (int32_t)items.size());
        nodesMi->setParameter("search", std::getenv("ALONE") ? 0.f : 1.f);
        nodesMi->setParameter("mode", env_f("VNODEMODE", 0.f));   // timing only (glass_nodes.mat)
        nodesMi->setParameter("ior", nd);
        nodesMi->setParameter("abbe", 64.17f / disp);
        nodesMi->setParameter("spread", env_f("SPREAD", 2.f));
        nodesMi->setParameter("floorY", 0.f);
        sNodes->addEntity(renderable(sqVB, sqIB, nodesMi, false, false));
        MaterialInstance* cellsMi = glassCellsMat->createInstance();
        cellsMi->setParameter("nodes", nodesRT.color, ns);
        cellsMi->setParameter("tol", env_f("VTOL", 1.f));
        sCells->addEntity(renderable(sqVB, sqIB, cellsMi, false, false));
    }

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
    scene->setSkybox(Skybox::Builder().color({0.72f * roomLight, 0.74f * roomLight, 0.78f * roomLight, 1}).build(*engine));
    {
        // Studio environment (out/studio): gives the block something to mirror and keeps the shadow from being black.
        auto ib = load_file(GLASS_PATH("out/studio/_ibl.ktx").c_str());
        std::ifstream shf(GLASS_PATH("out/studio/sh.txt"));
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
        const float iblLux = env_f("IBL", 900.f) * roomLight;
        scene->setIndirectLight(IndirectLight::Builder().reflections(ibl).irradiance(3, shv.data()).intensity(iblLux).build(*engine));
        TextureSampler cubeS(TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE);
        for (Item& it : items) {
            it.glass->setParameter("env", ibl, cubeS);
            it.glass->setParameter("envGain", iblLux * env_f("ENVGAIN", 0.f));
            it.glass->setParameter("sky", float3{0.95f, 0.97f, 1.0f} * (env_f("SKY", 1800.f) * roomLight));
            if (it.cheap) {
                it.cheap->setParameter("env", ibl, cubeS);
                it.cheap->setParameter("envGain", iblLux * env_f("ENVGAIN", 0.f));
                it.cheap->setParameter("sky", float3{0.95f, 0.97f, 1.0f} * (env_f("SKY", 1800.f) * roomLight));
            }
        }
    }

    // ---- camera and views
    Camera* camera = engine->createCamera(em.create());
    camera->setExposure(env_f("APERTURE", 4.0f), 1.f / 125.f, 100.f);
    float cam[7] = {1.55f, 1.45f, 2.15f, 0.10f, 0.0f, -0.05f, 32.f};
    if (wallCount > 0) {  // a little further back and higher, so that the walls are in the picture
        const float room[7] = {1.9f, 1.55f, 2.5f, -0.3f, 0.3f, -0.5f, 36.f};
        std::memcpy(cam, room, sizeof(cam));
    }
    if (std::getenv("SCATTER")) {   // strewn solids take more floor than the grid: stand back as far as it takes to see them all
        const float back = std::max(1.f, 0.85f * std::sqrt((float)items.size()) / 1.25f);
        for (int i = 0; i < 3; ++i) cam[i] = cam[3 + i] + (cam[i] - cam[3 + i]) * back;
    }
    if (const char* c = std::getenv("CAM")) std::sscanf(c, "%f %f %f %f %f %f %f", &cam[0], &cam[1], &cam[2], &cam[3], &cam[4], &cam[5], &cam[6]);
    camera->lookAt(float3{cam[0], cam[1], cam[2]}, float3{cam[3], cam[4], cam[5]}, float3{0, 1, 0});
    camera->setProjection((double)cam[6], double(W) / double(H), 0.05, 40.0, Camera::Fov::VERTICAL);

    View* vLight = engine->createView();
    vLight->setScene(sLight);
    vLight->setCamera(camera);
    vLight->setRenderTarget(light.rt);
    vLight->setViewport({0, 0, (uint32_t)LW, (uint32_t)LH});
    vLight->setDithering(View::Dithering::NONE);
    vLight->setPostProcessingEnabled(false);
    vLight->setShadowingEnabled(false);

    {
        const mat4 clipFromWorld = camera->getProjectionMatrix() * camera->getViewMatrix();
        softenMi->setParameter("worldFromClip", mat4f(inverse(clipFromWorld)));
        softenMi->setParameter("eye", float3{cam[0], cam[1], cam[2]});
        softenMi->setParameter("focal", (float)(LH / (2.0 * std::tan(cam[6] * 0.5 * 3.14159265 / 180.0))));
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
        if (items[k].local.kind == beam::Solid::EXTRUDE) continue;  // (solids with dents and holes draw their own shadow)
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
        on_stage(ok.mi, false);
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
            .instances(ok.solids.size() * recvTotal)
            .material(0, ok.mi).culling(false).castShadows(false).receiveShadows(false).layerMask(0xFF, outlineOnCpu ? 0x00 : 0x01).build(*engine, e);
        sShade->addEntity(e);
    }

    // ---- EXPERIMENT, STREAK=1: the bright lines of the solids with an outline, drawn from the outline itself with no
    // table pass (edge_streak.mat): one four-cornered piece per straight bit of the outline, per solid and colour step.
    std::vector<MaterialInstance*> streakMis;
    // EXNEW=1: all the light and shadow of those solids drawn this way (window and shadow of the caps: edge_fill.mat),
    // the old table passes and their drawing switched off.
    const bool exNew = std::getenv("EXNEW") != nullptr;
    std::vector<MaterialInstance*> fillMis;
    if (std::getenv("STREAK") || exNew) {
        Material* streakMat = mat("edge_streak.filamat");
        Material* fillMat = mat("edge_fill.filamat");
        if (!streakMat || !fillMat) return fail("edge_streak.filamat / edge_fill.filamat");
        for (const ExShapeUse& sh : exShapes) {
            auto* v = new std::vector<float>();   // a.x, a.y, corner; b.x, b.y; the way out of the glass at a, at b
            auto* ix = new std::vector<uint32_t>();
            for (const std::vector<int>& loop : sh.ol->loops)
                for (size_t i = 0; i < loop.size(); ++i) {
                    const int ja = loop[i];
                    const beam::V2 a = sh.ol->pts[(size_t)ja], b = sh.ol->pts[(size_t)loop[(i + 1) % loop.size()]];
                    // straight: the same way out all along; on an arc: away from (or towards) the middle of its circle
                    beam::V2 na{b.y - a.y, a.x - b.x}, nb = na;
                    for (const beam::OutlineEdge& e : sh.ol->edges)
                        if (e.r != 0 && ja >= e.first && ja < e.first + e.chords) {
                            const double sg = e.r > 0 ? 1.0 : -1.0;
                            na = (a - e.c) * sg;
                            nb = (b - e.c) * sg;
                        }
                    na = na * (1.0 / beam::length(na));
                    nb = nb * (1.0 / beam::length(nb));
                    const uint32_t base = (uint32_t)(v->size() / 9);
                    for (int c = 0; c < 4; ++c) v->insert(v->end(), {(float)a.x, (float)a.y, (float)c, (float)b.x, (float)b.y, (float)na.x, (float)na.y, (float)nb.x, (float)nb.y});
                    ix->insert(ix->end(), {base, base + 1, base + 2, base, base + 2, base + 3});
                }
            VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(v->size() / 9)).bufferCount(1)
                                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 36)
                                   .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 36)
                                   .attribute(VertexAttribute::CUSTOM0, 0, VertexBuffer::AttributeType::FLOAT4, 20, 36).build(*engine);
            vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(v->data(), v->size() * sizeof(float)));
            IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)ix->size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
            ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(ix->data(), ix->size() * sizeof(uint32_t)));
            float4 which[16] = {};
            int n = 0;
            for (size_t k = 0; k < items.size() && n < 64; ++k)
                if (items[k].local.kind == beam::Solid::EXTRUDE && items[k].local.shape.get() == sh.ol) { which[n / 4][n % 4] = (float)k; ++n; }
            if (exNew) {   // the caps: window, shadow of the near cap, shadow of the far cap
                auto* cv = new std::vector<float>();
                for (const beam::V2& q : sh.ol->pts) cv->insert(cv->end(), {(float)q.x, (float)q.y, 0.f});
                auto* ci = new std::vector<uint32_t>(sh.ol->tris.begin(), sh.ol->tris.end());
                VertexBuffer* cvb = VertexBuffer::Builder().vertexCount((uint32_t)(cv->size() / 3)).bufferCount(1)
                                        .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12).build(*engine);
                cvb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(cv->data(), cv->size() * sizeof(float)));
                IndexBuffer* cib = IndexBuffer::Builder().indexCount((uint32_t)ci->size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
                cib->setBuffer(*engine, IndexBuffer::BufferDescriptor(ci->data(), ci->size() * sizeof(uint32_t)));
                for (int mode = 0; mode < 3; ++mode) {
                    MaterialInstance* mi = fillMat->createInstance();
                    mi->setParameter("pose", poseTex, nearest);
                    mi->setParameter("solids", which, 16);
                    mi->setParameter("halfThick", (float)sh.ol->h);
                    mi->setParameter("index", nd);
                    mi->setParameter("mode", (float)mode);
                    mi->setParameter("floorY", 0.f);
                    utils::Entity e = em.create();
                    RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
                        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, cvb, cib).instances((size_t)n)
                        .material(0, mi).culling(false).castShadows(false).receiveShadows(false).build(*engine, e);
                    (mode == 0 ? sLight : sShade)->addEntity(e);
                    fillMis.push_back(mi);
                }
            }
            for (int mode = 0; mode < (exNew ? 4 : 2); ++mode) {   // mirrored by the wall; out through the cut on the rim (the colours);
                                                                   // EXNEW: the rim the window lacks; the walls' shadow
                const float strength = mode >= 2 ? 1.f : env_f(mode == 0 ? "STREAK" : "STREAKRAINBOW", 1.f);
                if (strength <= 0.f) continue;
                MaterialInstance* mi = streakMat->createInstance();
                mi->setParameter("pose", poseTex, nearest);
                mi->setParameter("solids", which, 16);
                mi->setParameter("stepCount", (float)(mode >= 2 ? 1 : beam::EXT_STEPS));
                mi->setParameter("halfThick", (float)sh.ol->h);
                mi->setParameter("mode", (float)mode);
                mi->setParameter("chamfer", (float)sh.ol->chamfer);
                mi->setParameter("wide", env_f("STREAKWIDE", 0.f));   // extra width for every colour's band (edge_streak.mat); 0: as the cut gives it
                mi->setParameter("gain", strength);
                mi->setParameter("cap", env_f("STREAKCAP", 3.f));
                mi->setParameter("floorY", 0.f);
                utils::Entity e = em.create();
                RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
                    .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib).instances((size_t)n * (size_t)(mode >= 2 ? 1 : beam::EXT_STEPS))
                    .material(0, mi).culling(false).castShadows(false).receiveShadows(false).build(*engine, e);
                (mode == 3 ? sShade : sLight)->addEntity(e);
                streakMis.push_back(mi);
            }
        }
    }

    View* vFlatWalk = engine->createView();
    vFlatWalk->setScene(sFlatWalk);
    vFlatWalk->setCamera(camera);
    for (const RouteLevel& rl : routeLevels) rl.view->setCamera(camera);
    vFlatWalk->setRenderTarget(flatWalk.rt);
    vFlatWalk->setViewport({0, 0, (uint32_t)(flatMostPieces * flatPer), (uint32_t)(items.size() * (size_t)flatLines)});
    vFlatWalk->setDithering(View::Dithering::NONE);
    vFlatWalk->setPostProcessingEnabled(false);
    vFlatWalk->setShadowingEnabled(false);
    View* vFlatWalk2 = engine->createView();
    if (flatClip) {
        vFlatWalk2->setScene(sFlatWalk2);
        vFlatWalk2->setCamera(camera);
        vFlatWalk2->setRenderTarget(flatWalk2.rt);
        vFlatWalk2->setViewport({0, 0, (uint32_t)(flatMostPieces * flatPer), (uint32_t)(items.size() * (size_t)flatLines)});
        vFlatWalk2->setDithering(View::Dithering::NONE);
        vFlatWalk2->setPostProcessingEnabled(false);
        vFlatWalk2->setShadowingEnabled(false);
    }
    View* vNodes = engine->createView();
    vNodes->setScene(sNodes);
    vNodes->setCamera(camera);
    vNodes->setRenderTarget(nodesRT.rt);
    vNodes->setViewport({0, 0, 289, (uint32_t)std::max<size_t>(1, items.size()) * 6});
    View* vCells = engine->createView();
    vCells->setScene(sCells);
    vCells->setCamera(camera);
    vCells->setRenderTarget(cellsRT.rt);
    vCells->setViewport({0, 0, 80, (uint32_t)std::max<size_t>(1, items.size()) * 6});
    vCells->setDithering(View::Dithering::NONE);
    vCells->setPostProcessingEnabled(false);
    vCells->setShadowingEnabled(false);
    vNodes->setDithering(View::Dithering::NONE);
    vNodes->setPostProcessingEnabled(false);
    vNodes->setShadowingEnabled(false);
    View* vConeWalk = engine->createView();
    vConeWalk->setScene(sConeWalk);
    vConeWalk->setCamera(camera);
    vConeWalk->setRenderTarget(coneWalk.rt);
    vConeWalk->setViewport({0, 0, (uint32_t)(coneV.size() / 3) * 2, (uint32_t)coneLines});
    vConeWalk->setDithering(View::Dithering::NONE);
    vConeWalk->setPostProcessingEnabled(false);
    vConeWalk->setShadowingEnabled(false);
    View* vExFind = engine->createView();
    View* vExWalk = engine->createView();
    for (View* v : {vExFind, vExWalk}) {
        const bool findPass = v == vExFind;
        v->setScene(findPass ? sExFind : sExWalk);
        v->setCamera(camera);
        v->setRenderTarget(findPass ? exFind.rt : exWalk.rt);
        v->setViewport({0, 0, (uint32_t)beam::EXT_COLUMNS, (uint32_t)(exRoom * (size_t)(findPass ? beam::EXT_FIND_ROWS : beam::EXT_ROWS))});
        v->setDithering(View::Dithering::NONE);
        v->setPostProcessingEnabled(false);
        v->setShadowingEnabled(false);
    }
    View* vShade = engine->createView();
    vShade->setScene(sShade);
    vShade->setCamera(camera);
    vShade->setRenderTarget(shade.rt);
    vShade->setViewport({0, 0, (uint32_t)LW, (uint32_t)LH});
    vShade->setDithering(View::Dithering::NONE);
    vShade->setPostProcessingEnabled(false);
    vShade->setShadowingEnabled(false);
    View* vSoft = engine->createView();
    vSoft->setScene(sSoft);
    vSoft->setCamera(camera);
    vSoft->setRenderTarget(soft.rt);
    vSoft->setViewport({0, 0, (uint32_t)SW, (uint32_t)SH});
    vSoft->setDithering(View::Dithering::NONE);
    vSoft->setPostProcessingEnabled(false);
    vSoft->setShadowingEnabled(false);

    // the stage: every material that drops light is told how to turn a distance flown into a spread in pixels (see
    // beam_soften.mat, `unit`)
    if (stageOn) {
        const float pxScale = std::tan(sunRadiusDeg * 3.14159265f / 180.f) * (float)(LH / (2.0 * std::tan(cam[6] * 0.5 * 3.14159265 / 180.0)));
        for (MaterialInstance* mi : stageLights) {
            set_stage(mi, true);
            if (blocksOn) mi->setDepthCulling(true);  // (the blocks are in the depth buffer of both pictures: what they hide is not drawn)
        }
        for (MaterialInstance* mi : landers) mi->setParameter("pxScale", pxScale);
    }

    const View::ShadowType shadowType = std::getenv("HARDSHADOW") ? View::ShadowType::PCF : View::ShadowType::PCSS;
    SoftShadowOptions softShadow;
    softShadow.penumbraScale = env_f("PENUMBRA", 1.f);
    View* vBackdrop = engine->createView();
    vBackdrop->setScene(scene);
    vBackdrop->setCamera(camera);
    vBackdrop->setRenderTarget(backdrop.rt);
    vBackdrop->setViewport({0, 0, (uint32_t)BW, (uint32_t)BH});
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
    // Every shadow here is an outline drawn as missing light (and on a stage Filament's own shadow maps would take the
    // sun away twice). Nothing casts a Filament shadow, but while shadowing was left on the floor still worked out
    // a soft shadow at every pixel of both pictures, from an empty map: 2.5 ms of GPU a frame for nothing.
    vBeauty->setShadowingEnabled(false);
    vBackdrop->setShadowingEnabled(false);
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
    if (wallCount > 0) sunDir = normalize(float3{-0.85f, -0.40f, -0.35f});   // towards the walls, or nothing would land on them
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
            if (it.cheap) {
                it.cheap->setParameter("sunTo", -sunDir);
                it.cheap->setParameter("sunGlint", sunColor * env_f("GLINT", 4.0e5f));
                it.cheap->setParameter("sunCos", std::cos(env_f("GLINTDEG", 2.0f) * 3.14159265f / 180.f));
            }
        }
        if (stageOn) {  // (all the stage costs the CPU in a frame: these few numbers, and switching its sets on and off below)
            for (MaterialInstance* mi : landers) mi->setParameter("recvSun", sunDir);
            for (MaterialInstance* mi : recvSurfaces) mi->setParameter("sunTo", -sunDir);
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
            if (it.local.kind == beam::Solid::EXTRUDE) {  // the lowest point of its outline, at whichever cap is lower
                low = 1e30;
                for (const beam::V2& p : it.local.shape->pts) low = std::min(low, R[0].y * p.x + R[1].y * p.y);
                low = (low - std::fabs(R[2].y) * it.local.shape->h) * it.size;
            }
            const beam::V3 at{x, -low + 0.0005 + lift, z};
            world.solids[k] = beam::placed(it.local, R, it.size, at);
            if (flatBeams) beamPose[k] = BeamPose{{R[0], R[1], R[2]}, at, (double)it.size};
            const float s = it.size;
            tm.setTransform(tm.getInstance(it.entity),
                            mat4f(float4{(float)R[0].x * s, (float)R[0].y * s, (float)R[0].z * s, 0}, float4{(float)R[1].x * s, (float)R[1].y * s, (float)R[1].z * s, 0},
                                  float4{(float)R[2].x * s, (float)R[2].y * s, (float)R[2].z * s, 0}, float4{(float)at.x, (float)at.y, (float)at.z, 1}));
            for (int c = 0; c < 3; ++c) {
                float* o = pose->data() + ((size_t)c * SCENE_H + k) * 4;
                o[0] = (float)R[c].x; o[1] = (float)R[c].y; o[2] = (float)R[c].z;
            }
            (*pose)[(0 * (size_t)SCENE_H + k) * 4 + 3] = s;
            if (it.cheap) {
                const float4 where[4] = {{(float)R[0].x, (float)R[0].y, (float)R[0].z, s}, {(float)R[1].x, (float)R[1].y, (float)R[1].z, 0.f},
                                         {(float)R[2].x, (float)R[2].y, (float)R[2].z, 0.f}, {(float)at.x, (float)at.y, (float)at.z, 0.f}};
                it.cheap->setParameter("pose", where, 4);
            }
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
            const unsigned leave = (1u << beam::Solid::BALL) | (1u << beam::Solid::CONE) | (1u << beam::Solid::ROD) | (1u << beam::Solid::EXTRUDE) |
                                   (flatOnCpu ? 0u : 1u << beam::Solid::FLAT);
            beam::trace_cheap(world, par, detail, pieces, tri, nullptr, ballsOnCpu ? nullptr : &alike, leave);
        }
        if (flatBeams && !std::getenv("NOLIGHT")) {
            const auto b0 = std::chrono::steady_clock::now();
            const std::vector<beam::BallStep> bsteps = beam::ball_steps(par, flatSteps - 1);
            double filmN[4][2];
            for (int f = 0; f < 4; ++f) { filmN[f][0] = par.glass.n(filmNm[f].y); filmN[f][1] = par.glass.n(filmNm[f].x); }
            beam::SplitBand band;
            if (const char* sv = std::getenv("STRIP")) {   // (the same band of sun the floor gets: beam_soften.mat)
                float half = 0.3f, deg = 35.f, side = 0.f, every = 0.f;
                std::sscanf(sv, "%f %f %f %f", &half, &deg, &side, &every);
                const float a = deg * 3.14159265f / 180.f;
                const float3 down = float3{(float)world.centre[0].x, (float)world.centre[0].y, (float)world.centre[0].z} + sunDir * ((float)world.centre[0].y / -sunDir.y);
                band = beam::SplitBand{std::cos(a), std::sin(a), std::cos(a) * down.x + std::sin(a) * down.z + side, half, every};
            }
            static std::vector<std::vector<float>> each;
            static std::vector<beam::SplitStats> stats;
            each.resize(items.size());
            stats.assign(items.size(), beam::SplitStats{});
            const double least = env_f("BEAMCUT", 2e-5f);
            const int mostFaces = (int)env_f("BEAMFACES", 8.f);
            beam::parallel_for(items.size(), threads, [&](size_t k) {
                each[k].clear();
                if (flatKindOf[k] < 0) return;
                beam::split_light(beamShapes[(size_t)flatKindOf[k]].data(), beamPose[k].R, beamPose[k].s, beamPose[k].at, par.sun, bsteps, filmN, band, 0.0, least, mostFaces, true, each[k], &stats[k]);
            });
            for (const std::vector<float>& e : each) tri.insert(tri.end(), e.begin(), e.end());
            static double msSum = 0;
            static int frames = 0;
            msSum += std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - b0).count();
            if (std::getenv("BEAMSTAT") && (++frames == 1 || frames % 300 == 0)) {
                beam::SplitStats all;
                for (const beam::SplitStats& st : stats) {
                    all.beams += st.beams; all.leaves += st.leaves; all.corners += st.corners;
                    all.deepest = std::max(all.deepest, st.deepest);
                    for (int i = 0; i < 12; ++i) all.light[i] += st.light[i];
                }
                double sum = 0;
                for (double l : all.light) sum += l;
                std::printf("  BEAMSTAT frame %d: %.2f ms a frame on %u threads (mean so far), %ld beams followed, %ld patches drawn (%ld corners), deepest %d steps\n", frames,
                            msSum / frames, threads, all.beams, all.leaves, all.corners, all.deepest);
                std::printf("    light by how many faces it met inside:");
                for (int i = 1; i < 12; ++i)
                    if (all.light[i] > 0) std::printf("  %d: %.1f%%", i, 100.0 * all.light[i] / sum);
                std::printf("\n");
            } else if (!std::getenv("BEAMSTAT")) ++frames;
        }
        outline.clear();
        for (const beam::Solid& so : world.solids)  // (on the CPU only to compare: solids with dents and holes have their own switch)
            if (so.kind == beam::Solid::EXTRUDE ? exOnCpu : outlineOnCpu) beam::silhouette_triangles(so, par.sun, 0.0, outline);
        for (OutlineKind& ok : outlineKinds) ok.mi->setParameter("sunDir", sunDir);
        traceMs = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();

        // the cones: where they are and how they stand to the sun into their table, and as many instances switched on
        {
            std::vector<int> balls, cones;
            for (int i : alike) {
                const int kind = world.solids[(size_t)i].kind;
                if (kind == beam::Solid::EXTRUDE) {
                    if (exOnCpu) beam::extrude_light(world.solids[(size_t)i], par, tri, nullptr, -1);
                    continue;
                }
                if (kind == beam::Solid::CONE || kind == beam::Solid::ROD) cones.push_back(i);
                else if (kind == beam::Solid::BALL) balls.push_back(i);
            }
            {
                // the flat-faced solids: their colour steps (the pose table is already on its way)
                if (rainbowOnce) {
                    // the two ends of the spectrum and its middle, and the spectrum summed up from the red end to the
                    // blue in 32 equal steps of the index of refraction (all of it: white)
                    const double nRed = par.glass.n(par.nmMax), nBlue = par.glass.n(par.nmMin);
                    const std::vector<beam::Band> fine = beam::make_bands(2048, par.nmMin, par.nmMax);
                    float4 rainbow[32] = {};
                    for (const beam::Band& b : fine) {
                        const double sAt = (par.glass.n(b.nm) - nRed) / (nBlue - nRed) * 31.0;
                        for (int i = 0; i < 32; ++i)
                            if ((double)i >= sAt) rainbow[i] += float4{(float)b.rgb.x, (float)b.rgb.y, (float)b.rgb.z, 0.f};
                    }
                    FLAT_WALK_SET("ends", float4{(float)(0.5 * (nRed + nBlue)), (float)nRed, (float)nBlue, 0.f});
                    FLAT_WALK_SET("sunDir", sunDir);
                    for (FlatKind& fk : flatKinds) fk.mi->setParameter("rainbow", rainbow, 32);
                } else {
                    const std::vector<beam::BallStep> fsteps = beam::ball_steps(par, flatSteps - 1);
                    float4 table[16] = {};
                    for (int i = 0; i + 1 < flatSteps && i < (int)fsteps.size(); ++i)
                        table[i] = float4{(float)fsteps[(size_t)i].rgb.x, (float)fsteps[(size_t)i].rgb.y, (float)fsteps[(size_t)i].rgb.z, (float)fsteps[(size_t)i].n};
                    table[flatSteps - 1] = table[(flatSteps - 1) / 2];
                    FLAT_WALK_SET("steps", table, 16);
                    FLAT_WALK_SET("sunDir", sunDir);
                    {   // which colours each film mirrors, as indices of refraction (red has the lowest)
                        float4 band[4] = {};
                        for (int f = 0; f < 4; ++f) band[f] = float4{(float)par.glass.n(filmNm[f].y), (float)par.glass.n(filmNm[f].x), 0.f, 0.f};
                        FLAT_WALK_SET("filmBand", band, 4);
                        for (MaterialInstance* mi : routeMis) mi->setParameter("filmBand", band, 4);
                    }
                    for (FlatKind& fk : flatKinds) {
                        fk.mi->setParameter("steps", table, 16);
                        if (fk.mi2) fk.mi2->setParameter("steps", table, 16);
                    }
                    for (MaterialInstance* mi : routeMis) {
                        mi->setParameter("steps", table, 16);
                        mi->setParameter("sunDir", sunDir);
                    }
                }
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
        // the solids with dents and holes: where each one is, into their table; the sun and the colour steps
        if (exCount) {
            auto* pt = new std::vector<float>(exRoom * 3 * 4, 0.f);
            for (size_t k = 0; k < items.size(); ++k)
                if (items[k].exColumn >= 0) {
                    float* a = pt->data() + (size_t)items[k].exColumn * 4;
                    beam::ex_pose(world.solids[k], 0, a, a + exRoom * 4, a + 2 * exRoom * 4);
                    for (const ExShapeUse& sh : exShapes)
                        if (sh.ol == world.solids[k].shape.get()) a[exRoom * 4 + 3] = (float)sh.line;
                }
            exPoseTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(pt->data(), pt->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                           [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, pt));
            const std::vector<beam::BallStep> ws = beam::ball_steps(par, beam::EXT_STEPS), cs = beam::ball_steps(par, beam::EXT_CAP_STEPS);
            float4 wall[8] = {}, cap[4] = {};
            for (size_t i = 0; i < 8; ++i) {
                const beam::BallStep& b = ws[std::min(i, ws.size() - 1)];
                wall[i] = i < ws.size() ? float4{(float)b.rgb.x, (float)b.rgb.y, (float)b.rgb.z, (float)b.n} : float4{0, 0, 0, (float)b.n};
            }
            for (size_t i = 0; i < 4; ++i) {
                const beam::BallStep& b = cs[std::min(i, cs.size() - 1)];
                cap[i] = i < cs.size() ? float4{(float)b.rgb.x, (float)b.rgb.y, (float)b.rgb.z, (float)b.n} : float4{0, 0, 0, (float)b.n};
            }
            for (MaterialInstance* mi : exMis) {
                mi->setParameter("steps", wall, 8);
                mi->setParameter("capSteps", cap, 4);
                mi->setParameter("sunDir", sunDir);
            }
            for (MaterialInstance* mi : streakMis) {
                mi->setParameter("steps", wall, 8);
                mi->setParameter("sunDir", sunDir);
            }
            for (MaterialInstance* mi : fillMis) mi->setParameter("sunDir", sunDir);
            const bool on = !exOnCpu && !exNew && !std::getenv("NOLIGHT");
            for (size_t i = 0; i < exEntities.size(); ++i) rm.setLayerMask(rm.getInstance(exEntities[i]), 0xFF, on && (exEntityBit[i] & exShow) ? 0x01 : 0x00);
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
            if (k == 0 && std::getenv("STRIP")) {
                // STRIP="half width [degrees round [off to the side]]": the sun comes through a slit and lights a band of the floor
                // only, which passes under the first solid (beam_soften.mat, flat_walk.mat)
                float half = 0.3f, deg = 35.f, side = 0.f, every = 0.f;   // (every: a blind, the band again and again this far apart)
                std::sscanf(std::getenv("STRIP"), "%f %f %f %f", &half, &deg, &side, &every);
                const float a = deg * 3.14159265f / 180.f;
                const float3 down = float3{(float)world.centre[0].x, (float)world.centre[0].y, (float)world.centre[0].z} + sunDir * ((float)world.centre[0].y / -sunDir.y);
                const float4 strip{std::cos(a), std::sin(a), std::cos(a) * down.x + std::sin(a) * down.z + side, half};
                softenMi->setParameter("strip", strip);
                softenMi->setParameter("stripSoft", float2{env_f("STRIPSOFT", 0.02f), every});
                FLAT_WALK_SET("strip", strip);
                FLAT_WALK_SET("stripSoft", float2{env_f("STRIPSOFT", 0.02f), every});
                for (MaterialInstance* mi : routeMis) {
                    mi->setParameter("strip", strip);
                    mi->setParameter("stripSoft", float2{env_f("STRIPSOFT", 0.02f), every});
                }
            }
            int count = 0;
            row[5] = (float)world.solids[k].kind; row[6] = (float)world.solids[k].r; row[7] = (float)world.solids[k].h;
            row[8] = (float)world.solids[k].ax.x; row[9] = (float)world.solids[k].ax.y; row[10] = (float)world.solids[k].ax.z;
            if (world.solids[k].kind == beam::Solid::CONE) {  // the middle of its base
                row[12] = (float)world.solids[k].c.x; row[13] = (float)world.solids[k].c.y; row[14] = (float)world.solids[k].c.z;
            }
            if (world.solids[k].kind == beam::Solid::EXTRUDE) {  // where its outline's origin is and its size; where its x axis points and the line of its shape
                const beam::Solid& so = world.solids[k];
                row[12] = (float)so.c.x; row[13] = (float)so.c.y; row[14] = (float)so.c.z; row[15] = (float)so.sc;
                row[16] = (float)so.ux.x; row[17] = (float)so.ux.y; row[18] = (float)so.ux.z;
                for (const ExShapeUse& sh : exShapes)
                    if (sh.ol == so.shape.get()) row[19] = (float)sh.line;
            }
            for (size_t f = 0; f < world.solids[k].planes.size() && count < SCENE_W - 3; ++f) {
                if (world.solids[k].faces[f].n < 3) continue;
                float* o = row + (3 + count) * 4;
                o[0] = (float)world.solids[k].planes[f].n.x; o[1] = (float)world.solids[k].planes[f].n.y; o[2] = (float)world.solids[k].planes[f].n.z;
                o[3] = (float)world.solids[k].planes[f].o;
                ++count;
            }
            row[4] = (float)count;
            if (world.solids[k].kind == beam::Solid::FLAT) {   // its colour films, after its faces
                int films = 0;
                for (const beam::Plane& fl : world.solids[k].films) {
                    if (films >= 4 || 3 + count + films >= SCENE_W) break;
                    float* o = row + (3 + count + films) * 4;
                    o[0] = (float)fl.n.x; o[1] = (float)fl.n.y; o[2] = (float)fl.n.z; o[3] = (float)fl.o;
                    ++films;
                }
                row[6] = (float)films;
            }
        }
        {
            float4 spheres[64] = {};
            for (size_t k = 0; k < items.size() && k < 64; ++k)
                spheres[k] = float4{(float)world.centre[k].x, (float)world.centre[k].y, (float)world.centre[k].z, (float)world.radius[k]};
            FLAT_WALK_SET("spheres", spheres, 64);
            if (nodesMi) nodesMi->setParameter("spheres", spheres, 64);
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
        for (int l = 0; stageOn && l < (blocksOn ? 2 : 1); ++l) {  // the stage draws the same buffer, once per receiver
            rm.setGeometryAt(rm.getInstance(stageLightE[l]), 0, RenderableManager::PrimitiveType::TRIANGLES, lightVB, lightIB, 0, std::max<size_t>(nLight, 3));
            rm.setLayerMask(rm.getInstance(stageLightE[l]), 0xFF, nLight ? 0x01 : 0x00);
            rm.setGeometryAt(rm.getInstance(stageShadeE[l]), 0, RenderableManager::PrimitiveType::TRIANGLES, lightVB, lightIB, nOut ? nLight : 0, std::max<size_t>(nOut, 3));
            rm.setLayerMask(rm.getInstance(stageShadeE[l]), 0xFF, nOut ? 0x01 : 0x00);
        }
    };

    auto draw_light = [&]() {
        if (vbodyOn && !std::getenv("VNODESOFF")) {
            renderer->render(vNodes);
            renderer->render(vCells);
        }
        // XWALK (timing only): 1 leaves out every table pass, or any of the letters f (flat), e (dents and holes: find),
        // w (dents and holes: walk), c (cones and rods)
        static const char* xw = std::getenv("XWALK");
        static const bool xF = xw && (std::strchr(xw, '1') || std::strchr(xw, 'f')), xE = xw && (std::strchr(xw, '1') || std::strchr(xw, 'e')),
                          xW = xw && (std::strchr(xw, '1') || std::strchr(xw, 'w')), xC = xw && (std::strchr(xw, '1') || std::strchr(xw, 'c'));
        if (routeTree) {
            for (const RouteLevel& rl : routeLevels) renderer->render(rl.view);
        } else if (!xF && !flatOnCpu && !flatBeams && !flatKinds.empty()) {
            renderer->render(vFlatWalk);
            if (flatClip && !std::getenv("XWALK2")) renderer->render(vFlatWalk2);   // (XWALK2: timing only, the second turn left out)
        }
        if (exCount && !exOnCpu && !exNew) {
            if (!xE) renderer->render(vExFind);
            if (!xW) renderer->render(vExWalk);
        }
        if (!xC && conesDrawn) renderer->render(vConeWalk);
        static const bool xLight = std::getenv("XLIGHT"), xShade = std::getenv("XSHADE"), xMips = std::getenv("XMIPS"), xSoft = std::getenv("XSOFT");  // timing only
        if (!xLight) renderer->render(vLight);
        if (!xShade) renderer->render(vShade);
        if (!xMips) {
            light.color->generateMipmaps(*engine);
            shade.color->generateMipmaps(*engine);
        }
        if (!xSoft) renderer->render(vSoft);
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
        if (!noBackdrop) renderer->render(vBackdrop);
        renderer->render(vBeauty);
        renderer->readPixels(beauty.rt, 0, 0, W, H,
                             backend::PixelBufferDescriptor(frame.data(), frame.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
        // VDUMP=file (to count ways): the table of glass_nodes.mat as it is, 289 * 3 x (solids * 6) texels of four floats
        static std::vector<float> nodeDump;
        if (vbodyOn && std::getenv("VDUMP")) {
            nodeDump.assign((size_t)289 * items.size() * 6 * 4, 0.f);
            renderer->readPixels(nodesRT.rt, 0, 0, 289, (uint32_t)items.size() * 6,
                                 backend::PixelBufferDescriptor(nodeDump.data(), nodeDump.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
        }
        // ROUTESTAT=1: how many leaves each level of route_tree.mat holds, and how much light (read back only to count)
        static std::vector<std::vector<float>> routeDump;
        if (routeTree && std::getenv("ROUTESTAT")) {
            routeDump.resize(routeLevels.size());
            for (size_t i = 0; i < routeLevels.size(); ++i) {
                const RouteLevel& rl = routeLevels[i];
                routeDump[i].assign((size_t)rl.slots * 2 * (size_t)rl.rows * 4, 0.f);
                renderer->readPixels(rl.table.rt, 0, 0, (uint32_t)(rl.slots * 2), (uint32_t)rl.rows,
                                     backend::PixelBufferDescriptor(routeDump[i].data(), routeDump[i].size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
            }
        }
        renderer->endFrame();
        engine->flushAndWait();
        if (!routeDump.empty()) {
            double all = 0;
            std::vector<double> light(routeDump.size(), 0.0);
            std::vector<long> leaves(routeDump.size(), 0), inner(routeDump.size(), 0), full(routeDump.size(), 0);
            for (size_t i = 0; i < routeDump.size(); ++i) {
                const RouteLevel& rl = routeLevels[i];
                const int fan = i == 0 ? rl.slots : rl.slots / routeLevels[i - 1].slots;
                for (int r = 0; r < rl.rows; ++r)
                    for (int sl = 0; sl < rl.slots; ++sl) {
                        const float* t = routeDump[i].data() + ((size_t)r * (size_t)rl.slots * 2 + (size_t)sl * 2) * 4;
                        if (t[2] > 1.5f) { ++leaves[i]; light[i] += t[3] * (double)((t[6] - t[4]) * (t[7] - t[5])); }
                        else if (t[2] > 0.5f) ++inner[i];
                        if (sl % fan == fan - 1 && t[2] > 0.5f) ++full[i];   // (the last place of its parent is taken: more may have been dropped)
                    }
                all += light[i];
            }
            std::printf("  ROUTESTAT (all solids, all colour steps; light = brightness x area of the box, a rough measure):\n");
            for (size_t i = 0; i < routeDump.size(); ++i)
                std::printf("    level %zu: %6ld leaves, %6ld go on inside, %5.1f%% of the light, %ld parents with every place taken\n", i + 1, leaves[i], inner[i],
                            all > 0 ? 100.0 * light[i] / all : 0.0, full[i]);
        }
        if (!nodeDump.empty())
            if (FILE* f = std::fopen(std::getenv("VDUMP"), "wb")) {
                std::fwrite(nodeDump.data(), sizeof(float), nodeDump.size(), f);
                std::fclose(f);
            }
        return write_png(path, W, H, frame.data());
    };
#ifdef GLASS_LIB
    // ---- live, for a host that shows the picture itself (glass_live.h): every frame is drawn straight into a buffer
    // the host can show without a copy. What those buffers are is the one thing that differs per OS (glass_door.h).
    // Nothing is waited for: only when the GPU has finished a frame is its buffer handed out. Three buffers; a frame is
    // drawn into one that is neither handed out nor still being drawn, or skipped.
    // The sun, the camera and the pace come from the host. Stopping is a pause: the thread and the engine stay.
    {
        struct Slot {
            std::atomic<int> busy{0};
            std::chrono::steady_clock::time_point begun;
        };
        static Slot slot[GLASS_DOOR_BUFFERS];
        if (!glass_door_open(engine, W, H, [](int i) {
                g_live.frameMs = std::chrono::duration<float, std::milli>(std::chrono::steady_clock::now() - slot[i].begun).count();
                g_live.front = i;
                g_live.frames.fetch_add(1);
                slot[i].busy = 0;
            }))
            return fail("pixel buffer");
        vBeauty->setRenderTarget(nullptr);   // the picture goes to the swap chain, i.e. into the pixel buffer
        const float focal = (float)(LH / (2.0 * std::tan(cam[6] * 0.5 * 3.14159265 / 180.0)));
        const float3 target{cam[3], cam[4], cam[5]};
        auto last = std::chrono::steady_clock::now();
        auto due = last;   // when the next frame is to start: one sixtieth of a second after the one before, whatever that one took
        float t = env_f("T", 0.f);
        for (;;) {
            auto begin = std::chrono::steady_clock::now();
            if (!g_live.run.load()) {   // paused
                last = due = begin;
                std::this_thread::sleep_for(std::chrono::milliseconds(30));
                continue;
            }
            t += std::chrono::duration<float>(begin - last).count() * g_live.speed.load();
            last = begin;
            int into = -1;
            for (int i = 0; i < GLASS_DOOR_BUFFERS && into < 0; ++i)
                if (!slot[i].busy.load() && g_live.front.load() != i) into = i;
            if (into >= 0) {
                const float sa = g_live.sunAz.load(), se = std::min(1.5f, std::max(0.05f, g_live.sunEl.load()));
                sunDir = float3{std::cos(se) * std::cos(sa), -std::sin(se), std::cos(se) * std::sin(sa)};
                const float ca = g_live.camAz.load(), ce = std::min(1.5f, std::max(0.03f, g_live.camEl.load())), cd = std::min(12.f, std::max(0.5f, g_live.camDist.load()));
                const float3 eye = target + float3{std::cos(ce) * std::sin(ca), std::sin(ce), std::cos(ce) * std::cos(ca)} * cd;
                cam[0] = eye.x; cam[1] = eye.y; cam[2] = eye.z;
                camera->lookAt(eye, target, float3{0, 1, 0});
                softenMi->setParameter("worldFromClip", mat4f(inverse(camera->getProjectionMatrix() * camera->getViewMatrix())));
                softenMi->setParameter("eye", eye);
                softenMi->setParameter("focal", focal);
                auto c0 = std::chrono::steady_clock::now();
                update(t);
                auto c1 = std::chrono::steady_clock::now();
                const bool go = renderer->beginFrame(glass_door_chain(into));
                auto c2 = std::chrono::steady_clock::now();
                if (go) {
                    slot[into].begun = begin;
                    slot[into].busy = 1;
                    draw_light();
                    if (!noBackdrop) renderer->render(vBackdrop);
                    renderer->render(vBeauty);
                    auto c3 = std::chrono::steady_clock::now();
                    renderer->endFrame();
                    auto c4 = std::chrono::steady_clock::now();
                    if (std::getenv("GLASS_TIMING")) {   // where a turn of this loop spends its time (CPU side)
                        static double acc[4] = {};
                        static int count = 0;
                        auto ms = [](auto a, auto b) { return std::chrono::duration<double, std::milli>(b - a).count(); };
                        acc[0] += ms(c0, c1); acc[1] += ms(c1, c2); acc[2] += ms(c2, c3); acc[3] += ms(c3, c4);
                        if (++count == 120) {
                            std::fprintf(stderr, "LIVE cpu per frame: update %.2f  beginFrame %.2f  views %.2f  endFrame %.2f ms\n", acc[0] / 120, acc[1] / 120, acc[2] / 120, acc[3] / 120);
                            count = 0;
                            acc[0] = acc[1] = acc[2] = acc[3] = 0;
                        }
                    }
                } else {
                    g_live.skipped.fetch_add(1);
                }
            } else {
                g_live.skipped.fetch_add(1);
            }
            due += std::chrono::microseconds(16667);
            if (due < begin) due = begin;   // fell behind: do not try to catch up
            std::this_thread::sleep_until(due);
        }
    }
#endif
    std::system("mkdir -p out");
    const float t0 = env_f("T", 0.f);
    update(t0);
    for (int i = 0; i < 3; ++i) {
        if (!open()) return fail("warm");
        draw_light();
        if (!noBackdrop) renderer->render(vBackdrop);
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
    std::printf("      %zu solid(s) with dents and holes, %zu shape(s): %s (%zu triangles on the GPU, most of them folded up)\n", exCount, exShapes.size(),
                exOnCpu ? "light and shadow from the CPU's copy (EXTRUDECPU)" : "light and shadow worked out on the GPU", exOnCpu ? (size_t)0 : exTriangles);

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
    // ---- BENCH=n: n frames one after another, each waited for to the end: what one frame really costs (CPU + GPU),
    // with no waiting for a slot and no frames queued behind one another in the number
    if (std::getenv("BENCH")) {
        const int n = (int)env_f("BENCH", 200.f);
        const float3 sun0 = sunDir;
        std::vector<double> ms;
        for (int i = 0; i < n + 20; ++i) {
            const float t = (float)i / 120.f * 6.2831853f;
            sunDir = normalize(float3{sun0.x * std::cos(0.3f * std::sin(t)), sun0.y, sun0.z + 0.4f * std::sin(t)});
            auto b0 = std::chrono::steady_clock::now();
            update(t);
            if (!open()) return fail("bench");
            static const bool xAll = std::getenv("XALLLIGHT");  // timing only
            if (!xAll) draw_light();
            if (!noBackdrop) renderer->render(vBackdrop);
            renderer->render(vBeauty);
            renderer->endFrame();
            engine->flushAndWait();
            if (i >= 20) ms.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - b0).count());
        }
        {
            // what the GPU itself was busy for, as Filament's frame timer has it (the last frames)
            std::vector<double> g;
            for (auto const& f : renderer->getFrameInfoHistory(32))
                if (f.gpuFrameDuration > 0) g.push_back(f.gpuFrameDuration * 1e-6);
            std::sort(g.begin(), g.end());
            if (!g.empty()) std::printf("BENCH GPU alone (%zu frames):  min %.2f  median %.2f  max %.2f ms\n", g.size(), g.front(), g[g.size() / 2], g.back());
        }
        std::sort(ms.begin(), ms.end());
        std::printf("BENCH %d frames, %zu solids: one frame takes  min %.2f  median %.2f  95%% %.2f  max %.2f ms\n", n, items.size(), ms.front(), ms[ms.size() / 2],
                    ms[(size_t)(ms.size() * 0.95)], ms.back());
        return 0;
    }
    if (std::getenv("PACE")) {
        const int n = (int)env_f("PACE_FRAMES", 600.f);
        const double hz = env_f("PACE_HZ", 60.f);
        const double period = 1000.0 / hz;
        const float3 sun0 = sunDir;
        std::vector<double> cpu, g, lat, upd, mainMs, backMs;
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
            if (!noBackdrop) renderer->render(vBackdrop);
            renderer->render(vBeauty);
            renderer->endFrame();
            ++drawn;
            if (drawn % 16 == 0 || i == n - 1) {
                engine->flush();
                auto hist = renderer->getFrameInfoHistory(16);
                for (auto const& f : hist) {
                    if (f.endFrame > 0 && f.beginFrame > 0) mainMs.push_back((f.endFrame - f.beginFrame) * 1e-6);
                    if (f.backendEndFrame > 0 && f.backendBeginFrame > 0) backMs.push_back((f.backendEndFrame - f.backendBeginFrame) * 1e-6);
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
        line("main thread, beginFrame -> endFrame", mainMs);
        line("backend thread, per frame", backMs);
        line("GPU time per frame", g);
        line("beginFrame -> GPU finished", lat);
        std::printf("     most light triangles in a frame: %zu from the CPU; balls drawn from the one shared shape: %zu, cones: %zu\n", maxTris, ballsAlike, conesDrawn);
        std::printf("     finished after their %.2f ms slot: %zu of %zu measured (%.1f%%)\n", period, late, lat.size(), lat.empty() ? 0.0 : 100.0 * late / lat.size());
        std::printf("     DROPPED (Filament asked to skip): %d of %d ticks (%.1f%%)  -> delivered %.1f fps of %.0f\n", skipped, n, 100.0 * skipped / n, drawn / (total / 1000.0), hz);
    }
    return 0;
}
