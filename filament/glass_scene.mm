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
#include "beam_spectrum.h"   // SPEC=1
#ifdef GLASS_COMPUTE_DOOR
#include "cbeam.h"   // CBEAMS=1: the beams as compute (needs Filament's WebGPU backend; dawn_probe/glass_wgpu.sh builds with it)
#include <filament/BufferObject.h>
#endif

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
#include <unordered_map>
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
#ifdef __APPLE__
    if (std::getenv("WORKQOS")) pthread_set_qos_class_self_np(QOS_CLASS_USER_INTERACTIVE, 0);
#endif
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

    // BACKEND=webgpu (trial): Filament's WebGPU backend (Dawn). Needs a Filament built with it and materials compiled
    // for it (dawn_probe/build.sh, dawn_probe/glass_wgpu.sh); the prebuilt SDK has neither.
    const char* via = std::getenv("BACKEND");
    const bool viaDawn = via && std::string(via) == "webgpu";
#ifdef GLASS_LIB
    Engine* engine = glass_door_engine();
#else
    Engine* engine = Engine::Builder().backend(viaDawn ? Engine::Backend::WEBGPU : Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
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
#include "glass_scene_2.inc"
#include "glass_scene_3.inc"
#include "glass_scene_4.inc"
#include "glass_scene_5.inc"
#include "glass_scene_6.inc"
#include "glass_scene_7.inc"
#include "glass_scene_8.inc"
