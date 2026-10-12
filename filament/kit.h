// kit.h: what the glass pictures share (poster.mm, blocks.mm): a stage (Filament's engine, one view, a headless
// screen), a way to put a mesh on it, the round-edged glass block, and the picture of what is behind the blocks.
// Everything that draws is Filament as it is; glassbox.mat is the one material of ours that follows a sight line.
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
#include <filament/SwapChain.h>
#include <filament/Texture.h>
#include <filament/TextureSampler.h>
#include <filament/ToneMapper.h>
#include <filament/TransformManager.h>
#include <filament/VertexBuffer.h>
#include <filament/View.h>
#include <filament/Box.h>
#include <filament/Viewport.h>

#include <geometry/SurfaceOrientation.h>
#include <image/Ktx1Bundle.h>
#include <ktxreader/Ktx1Reader.h>
#include <math/mat4.h>
#include <utils/EntityManager.h>

#import <Cocoa/Cocoa.h>
#import <SceneKit/SceneKit.h>
#import <ModelIO/ModelIO.h>
#import <SceneKit/ModelIO.h>

#include <cmath>
#include <cstdint>
#include <cstdio>
#include <fstream>
#include <vector>

using namespace filament;
using namespace filament::math;

static const float TAU = 6.2831853f;

struct Mesh {
    std::vector<float3> pos, nrm;
    std::vector<float2> uv;
    std::vector<uint32_t> idx;
    std::vector<quatf> quat;
};

static std::vector<uint8_t> load_file(const char* path) {
    std::ifstream in(path, std::ios::binary);
    return std::vector<uint8_t>((std::istreambuf_iterator<char>(in)), {});
}

static int fail(const char* msg) {
    std::printf("FAIL %s\n", msg);
    return 1;
}

struct Stage {
    int w, h;
    Engine* engine;
    Renderer* renderer;
    Scene* scene;
    View* view;
    SwapChain* swap;
    Camera* camera;
    View* view_behind;   // the same scene without the glass (layer 2), drawn first into `behind`: what a block shows
    Texture* behind;
    View* view_floor = nullptr;   // the floor from straight above (layer 4 only), drawn before all into `sunlight` (paper_and_sun)
    Texture* sunlight = nullptr;
};

static Stage open_stage(int w, int h) {
    Stage s{w, h};
    s.engine = Engine::Builder().backend(Engine::Backend::METAL).build();
    s.renderer = s.engine->createRenderer();
    s.scene = s.engine->createScene();
    s.view = s.engine->createView();
    s.swap = s.engine->createSwapChain((uint32_t)w, (uint32_t)h, 0);
    s.camera = s.engine->createCamera(utils::EntityManager::get().create());
    s.camera->setExposure(16.0f, 1.0f / 125.0f, 100.0f);
    s.view->setScene(s.scene);
    s.view->setCamera(s.camera);
    s.view->setViewport({0, 0, (uint32_t)w, (uint32_t)h});
    s.view->setDithering(View::Dithering::NONE);
    s.view->setAntiAliasing(View::AntiAliasing::FXAA);
    s.view->setScreenSpaceRefractionEnabled(true);
    s.view->setVisibleLayers(0x03, 0x03);
    Renderer::ClearOptions clear{};
    clear.clearColor = {0, 0, 0, 1};
    clear.clear = true;
    s.renderer->setClearOptions(clear);
    s.behind = Texture::Builder().width(w).height(h).levels(1).usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE)
                   .format(Texture::InternalFormat::RGBA16F).build(*s.engine);
    Texture* depth = Texture::Builder().width(w).height(h).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT).format(Texture::InternalFormat::DEPTH24).build(*s.engine);
    s.view_behind = s.engine->createView();
    s.view_behind->setScene(s.scene);
    s.view_behind->setCamera(s.camera);
    s.view_behind->setViewport({0, 0, (uint32_t)w, (uint32_t)h});
    s.view_behind->setRenderTarget(RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, s.behind).texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*s.engine));
    s.view_behind->setPostProcessingEnabled(false);
    s.view_behind->setScreenSpaceRefractionEnabled(false);
    return s;
}

// The room the glass mirrors and is lit by: one of the studios that come with Filament, made into a .ktx by cmgen.
static bool room(Stage& s, const char* ktx, float intensity) {
    static std::vector<std::vector<uint8_t>> kept;
    kept.push_back(load_file(ktx));
    if (kept.back().empty()) return false;
    auto* bundle = new image::Ktx1Bundle(kept.back().data(), (uint32_t)kept.back().size());
    float3 sh[9];
    if (!bundle->getSphericalHarmonics(sh)) return false;
    Texture* tex = ktxreader::Ktx1Reader::createTexture(s.engine, bundle, false);
    s.scene->setIndirectLight(IndirectLight::Builder().reflections(tex).irradiance(3, sh).intensity(intensity).build(*s.engine));
    return true;
}

// A mesh on the stage, once per place. layer 2: left out of the picture of what is behind (all the glass);
// layer 4: only in the floor's picture of sunlight. order: among things of a kind, lower is drawn first.
// copies: how many times each is drawn in one go (a material that asks for its instance index tells them apart).
static MaterialInstance* add(Stage& s, Mesh* m, const char* filamat, std::vector<mat4f> places = {mat4f{}}, uint8_t layer = 1,
                             bool shadows = false, int copies = 1, int order = 4) {
    static std::vector<std::vector<uint8_t>> pkgs;
    pkgs.push_back(load_file(filamat));
    Material* mat = pkgs.back().empty() ? nullptr : Material::Builder().package(pkgs.back().data(), pkgs.back().size()).build(*s.engine);
    if (!mat) return nullptr;
    const size_t n = m->pos.size();
    auto* frames = geometry::SurfaceOrientation::Builder().vertexCount(n).normals(m->nrm.data()).build();
    m->quat.resize(n);
    frames->getQuats(m->quat.data(), n);
    delete frames;
    VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)n).bufferCount(3)
                           .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3)
                           .attribute(VertexAttribute::TANGENTS, 1, VertexBuffer::AttributeType::FLOAT4)
                           .attribute(VertexAttribute::UV0, 2, VertexBuffer::AttributeType::FLOAT2).build(*s.engine);
    vb->setBufferAt(*s.engine, 0, VertexBuffer::BufferDescriptor(m->pos.data(), n * sizeof(float3)));
    vb->setBufferAt(*s.engine, 1, VertexBuffer::BufferDescriptor(m->quat.data(), n * sizeof(quatf)));
    vb->setBufferAt(*s.engine, 2, VertexBuffer::BufferDescriptor(m->uv.data(), n * sizeof(float2)));
    IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)m->idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*s.engine);
    ib->setBuffer(*s.engine, IndexBuffer::BufferDescriptor(m->idx.data(), m->idx.size() * sizeof(uint32_t)));
    MaterialInstance* mi = mat->createInstance();
    float3 low = m->pos[0], high = m->pos[0];   // (Filament wants a thing's rough size if it casts or takes shadows)
    for (const float3& p : m->pos) { low = min(low, p); high = max(high, p); }
    for (const mat4f& place : places) {
        utils::Entity e = utils::EntityManager::get().create();
        s.engine->getTransformManager().create(e, {}, place);
        RenderableManager::Builder(1).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib).material(0, mi)
            .boundingBox({(low + high) * 0.5f, max((high - low) * 0.5f, float3{1e-3f})}).culling(false).castShadows(shadows).receiveShadows(shadows).layerMask(0xFF, layer).instances((size_t)copies).priority((uint8_t)order).build(*s.engine, e);
        s.scene->addEntity(e);
    }
    return mi;
}

// One glass block, edge 1, as real ones are: the edges a little round (a tenth of half the edge), the faces flat.
// glassbox.mat draws it: the sight line goes in, through the block's real far faces, mirrors inside where it cannot
// leave, and shows what lies behind where it comes out.
static Mesh* cube(float ROUND, int flat = 1) {   // flat: how many times a flat is cut across (boxlight.mat wants it fine)
    Mesh* m = new Mesh;
    std::vector<float> line;   // (where a face is cut: finely across the rounding, once across the flat)
    for (int k = 0; k <= 4; ++k) line.push_back(-1.f + ROUND * (float)k / 4.f);
    for (int k = 1; k < flat; ++k) line.push_back(-(1.f - ROUND) + 2.f * (1.f - ROUND) * (float)k / (float)flat);
    for (int k = 4; k >= 0; --k) line.push_back(1.f - ROUND * (float)k / 4.f);
    const int n = (int)line.size();
    const float3 ax[3] = {{1, 0, 0}, {0, 1, 0}, {0, 0, 1}};
    for (int f = 0; f < 6; ++f) {
        const float sgn = f < 3 ? 1.f : -1.f;
        const float3 nf = ax[f % 3] * sgn, a = ax[(f + 1) % 3], b = ax[(f + 2) % 3] * sgn;
        const uint32_t base = (uint32_t)m->pos.size();
        for (int j = 0; j < n; ++j)
            for (int i = 0; i < n; ++i) {
                const float3 q = nf + a * line[i] + b * line[j];
                const float3 core = clamp(q, float3{-(1.f - ROUND)}, float3{1.f - ROUND}), out = normalize(q - core);
                m->pos.push_back((core + out * ROUND) * 0.5f);
                m->nrm.push_back(out);
                m->uv.push_back({0, 0});
            }
        for (int j = 0; j + 1 < n; ++j)
            for (int i = 0; i + 1 < n; ++i) {
                const uint32_t p0 = base + (uint32_t)(j * n + i), p1 = p0 + 1, p2 = p0 + (uint32_t)n + 1, p3 = p0 + (uint32_t)n;
                m->idx.insert(m->idx.end(), {p0, p1, p2, p0, p2, p3});
            }
    }
    return m;
}

// The glass blocks: the round-edged block at each place, drawn by glassbox.mat over the picture of what is behind.
// What differs from picture to picture (the studio's brightness, the colour split, the lean of the faces) is the
// caller's to set on what this returns.
static MaterialInstance* glass_blocks(Stage& s, std::vector<mat4f> places, float round, bool shadows = false) {
    MaterialInstance* mi = add(s, cube(round), "glassbox.filamat", places, 2, shadows);
    if (!mi) return nullptr;
    mi->setParameter("backdrop", s.behind, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE));
    mi->setParameter("ior", 1.5f);
    mi->setParameter("reach", 0.5f);
    mi->setParameter("sceneMix", 1.f);
    mi->setParameter("absorption", float3{0.10f, 0.04f, 0.06f});
    mi->setParameter("round", round);
    mi->setParameter("bySize", 1.f);
    return mi;
}

// A ring of round section, lying in the plane of x and y.
static Mesh* torus(float R, float r, int around = 128, int about = 40) {
    Mesh* m = new Mesh;
    for (int i = 0; i < around; ++i)
        for (int k = 0; k < about; ++k) {
            const float a = TAU * (float)i / around, b = TAU * (float)k / about;
            const float3 out{std::cos(a) * std::cos(b), std::sin(a) * std::cos(b), std::sin(b)};
            m->pos.push_back(float3{std::cos(a), std::sin(a), 0.f} * R + out * r);
            m->nrm.push_back(out);
            m->uv.push_back({(float)i / around, (float)k / about});
            const uint32_t i1 = (uint32_t)((i + 1) % around), k1 = (uint32_t)((k + 1) % about);
            const uint32_t p = (uint32_t)(i * about + k), q = (uint32_t)(i * about) + k1, s = i1 * about + (uint32_t)k, t = i1 * about + k1;
            m->idx.insert(m->idx.end(), {p, s, q, q, s, t});
        }
    return m;
}

// A solid of twenty flat faces (an icosahedron), each face its own three corners: hard edges.
static Mesh* gem(float size) {
    const float f = (1.f + std::sqrt(5.f)) * 0.5f;
    const float3 v[12] = {{-1, f, 0}, {1, f, 0}, {-1, -f, 0}, {1, -f, 0}, {0, -1, f}, {0, 1, f}, {0, -1, -f}, {0, 1, -f}, {f, 0, -1}, {f, 0, 1}, {-f, 0, -1}, {-f, 0, 1}};
    const int faces[20][3] = {{0, 11, 5}, {0, 5, 1}, {0, 1, 7}, {0, 7, 10}, {0, 10, 11}, {1, 5, 9}, {5, 11, 4}, {11, 10, 2}, {10, 7, 6}, {7, 1, 8},
                              {3, 9, 4}, {3, 4, 2}, {3, 2, 6}, {3, 6, 8}, {3, 8, 9}, {4, 9, 5}, {2, 4, 11}, {6, 2, 10}, {8, 6, 7}, {9, 8, 1}};
    Mesh* m = new Mesh;
    for (const auto& t : faces) {
        const float3 n = normalize(cross(v[t[1]] - v[t[0]], v[t[2]] - v[t[0]]));
        for (int k = 0; k < 3; ++k) {
            m->idx.push_back((uint32_t)m->pos.size());
            m->pos.push_back(normalize(v[t[k]]) * size);
            m->nrm.push_back(n);
            m->uv.push_back({0, 0});
        }
    }
    return m;
}

// Letters as a solid, by the system's own solid text (SceneKit makes it, ModelIO hands over the mesh): `deep`
// through, the edges cut off by `chamfer`; a capital is about 0.7 high. The letters' lower left is at the origin,
// they run along x and stand up along y, the middle of their depth is at z = 0.
static Mesh* letters(const char* text, float deep, float chamfer) {
    const float size = 10.f;   // (SceneKit cuts curves by a fixed fineness: asked large, then made small)
    SCNText* solid = [SCNText textWithString:@(text) extrusionDepth:deep * size];
    solid.font = [NSFont fontWithName:@"Helvetica-Bold" size:size];
    solid.chamferRadius = chamfer * size;
    solid.flatness = 0.02;
    MDLMesh* made = [MDLMesh meshWithSCNGeometry:solid];
    MDLVertexAttributeData* ps = [made vertexAttributeDataForAttributeNamed:MDLVertexAttributePosition asFormat:MDLVertexFormatFloat3];
    MDLVertexAttributeData* ns = [made vertexAttributeDataForAttributeNamed:MDLVertexAttributeNormal asFormat:MDLVertexFormatFloat3];
    if (!made.vertexCount || !ps || !ns) return nullptr;
    Mesh* m = new Mesh;
    float3 low{1e9f};
    for (NSUInteger i = 0; i < made.vertexCount; ++i) {
        const float* p = (const float*)((const uint8_t*)ps.dataStart + i * ps.stride);
        const float* n = (const float*)((const uint8_t*)ns.dataStart + i * ns.stride);
        m->pos.push_back(float3{p[0], p[1], p[2]} / size);
        m->nrm.push_back(normalize(float3{n[0], n[1], n[2]}));
        m->uv.push_back({0, 0});
        low = min(low, m->pos.back());
    }
    for (float3& p : m->pos) p = p - float3{low.x, low.y, 0.f};
    for (MDLSubmesh* part in made.submeshes) {
        if (part.geometryType != MDLGeometryTypeTriangles) continue;
        const void* b = [part.indexBuffer map].bytes;
        for (NSUInteger i = 0; i < part.indexCount; ++i)
            m->idx.push_back(part.indexType == MDLIndexBitDepthUInt32 ? ((const uint32_t*)b)[i] : part.indexType == MDLIndexBitDepthUInt16 ? (uint32_t)((const uint16_t*)b)[i] : (uint32_t)((const uint8_t*)b)[i]);
    }
    return m->idx.empty() ? nullptr : m;
}

// Paper under a hard sun, with one of Filament's studios as the sky that fills the shadows (kept low, so the shadows
// stay dark). What the glass things stand on in blocks.mm and shapes.mm.
// How much sun reaches the paper is one picture, `sunlight`: the floor from straight above, 1 where the sun is bare,
// 0 in a thing's shade (shadow_of), more where glass throws its light (glass_light, box_light). paper.mat softens it
// and turns it into light, so shade and thrown light are equally soft and of one piece. (Filament's own sun shadow is
// not used: it is shade or no shade, and knows nothing of thrown light.)
struct Sunlit {
    float3 paper{0.52f, 0.60f, 0.65f};   // (a middle blue-grey: the light the glass throws must be able to be brighter than the paper)
    float3 sunWay = normalize(float3{-0.55f, -0.50f, 0.67f});   // (the sun low, the shadows long)
    float sunLux = 100000.f;
    float sky = 7000.f;                  // (the studio's light, which fills the shadows: low, so they stay dark)
    const char* studio = "out/lightroom/lightroom_ibl.ktx";   // (the room: it lights the shadows and is what Filament's glass mirrors)
    float sunWide = 0.022f;              // (the sun's width: how far shade spreads for every unit it flew; far wider than the true sun, for the softness)
};
static bool paper_and_sun(Stage& s, const Sunlit& lit) {
    if (!room(s, lit.studio, lit.sky)) return false;
    utils::Entity sun = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::SUN).color({1.0f, 0.98f, 0.95f}).intensity(lit.sunLux).direction(lit.sunWay)
        .castShadows(false).build(*s.engine, sun);
    s.scene->addEntity(sun);

    // the picture of sunlight: 8 units of floor about the middle, seen from above
    const float half = 4.f;
    const uint32_t size = 2048;
    s.sunlight = Texture::Builder().width(size).height(size).levels(1).usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE)
                     .format(Texture::InternalFormat::RGBA16F).build(*s.engine);
    Texture* depth = Texture::Builder().width(size).height(size).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT).format(Texture::InternalFormat::DEPTH24).build(*s.engine);
    Camera* above = s.engine->createCamera(utils::EntityManager::get().create());
    above->setProjection(Camera::Projection::ORTHO, -half, half, -half, half, 0.1, 40.0);
    above->lookAt({0, 20, 0}, {0, 0, 0}, {0, 0, -1});
    s.view_floor = s.engine->createView();
    s.view_floor->setScene(s.scene);
    s.view_floor->setCamera(above);
    s.view_floor->setViewport({0, 0, size, size});
    s.view_floor->setRenderTarget(RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, s.sunlight).texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*s.engine));
    s.view_floor->setPostProcessingEnabled(false);
    s.view_floor->setShadowingEnabled(false);
    s.view_floor->setVisibleLayers(0xFF, 0x04);

    Mesh* floor = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) {
        floor->pos.push_back({2.f * half * c.x, 0.f, -2.f * half * c.y});
        floor->nrm.push_back({0, 1, 0});
        floor->uv.push_back(c * 0.5f + 0.5f);
    }
    floor->idx = {0, 1, 2, 0, 2, 3};
    MaterialInstance* bare = add(s, floor, "floor_shade.filamat", {mat4f{}}, 4, false, 1, 0);   // the bare sun, everywhere, first
    if (!bare) return false;
    bare->setParameter("sunWay", lit.sunWay);
    bare->setParameter("value", 1.f);
    bare->setParameter("floorY", 0.f);
    Mesh* sheet = new Mesh(*floor);   // (the paper itself reaches further than its picture of sunlight: beyond it the sun is bare)
    for (float3& p : sheet->pos) p = p * 4.f;
    MaterialInstance* paper = add(s, sheet, "paper.filamat");
    if (!paper) return false;
    paper->setParameter("color", lit.paper);
    paper->setParameter("grain", 0.12f);
    paper->setParameter("sunlight", s.sunlight, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE));
    paper->setParameter("pictureFromScene", mat4f(above->getProjectionMatrix() * above->getViewMatrix()));
    paper->setParameter("sunWide", lit.sunWide);   // (a sun some 4.5 degrees wide: far wider than the true one, for the softness)
    paper->setParameter("perUnit", 1.f / (2.f * half));
    paper->setParameter("furthest", 2.5f);   // (softened over 0.02 of the floor's units)
    return true;
}

// A glass thing's shade on the paper: its mesh laid flat along the sun's way, in 0, into the picture of sunlight.
static bool shadow_of(Stage& s, const Sunlit& lit, Mesh* m, std::vector<mat4f> places) {
    MaterialInstance* mi = add(s, m, "floor_shade.filamat", places, 4, false, 1, 1);
    if (!mi) return false;
    mi->setParameter("sunWay", lit.sunWay);
    mi->setParameter("value", 0.f);
    mi->setParameter("floorY", 0.f);
    return true;
}

// The light a glass thing throws on the paper (glasslight.mat): the thing's own mesh at its places, once a colour.
// Exaggerated apart from the look of the glass: the colours' indices far apart, the light a little stronger than true.
// uneven: how far its faces lean from their own facing (0 for a mesh that is uneven itself), over `unevenSize` of the mesh's units.
static bool glass_light(Stage& s, const Sunlit& lit, Mesh* m, std::vector<mat4f> places, float uneven = 0.f, float unevenSize = 1.f) {
    const int colours = 20;
    MaterialInstance* mi = add(s, m, "glasslight.filamat", places, 4, false, colours);
    if (!mi) return false;
    mi->setParameter("sunWay", lit.sunWay);
    mi->setParameter("colours", (float)colours);
    float size = 0.f;
    for (const float3& p : m->pos) size = std::max(size, std::max(std::fabs(p.x), std::max(std::fabs(p.y), std::fabs(p.z))));
    mi->setParameter("size", size);
    mi->setParameter("ior", 1.5f);
    mi->setParameter("abbe", 6.f);
    mi->setParameter("lean", uneven);
    mi->setParameter("leanSize", unevenSize);
    mi->setParameter("facing", float2{0.6f, 0.74f});
    mi->setParameter("most", 6.f);
    mi->setParameter("gain", 0.9f);
    mi->setParameter("floorY", 0.f);
    return true;
}

// The light a glass BOX throws on the paper (boxlight.mat): as glass_light, but the light is taken through the box
// itself (in by one face, mirrored inside where it cannot leave, out by another), which a box allows by plain
// formulas. That is what gives a glass box its true shadow: dark nearly all over, since most of its light leaves
// through its underside, with a few small windows of light in it. places: the unit cube's places (a box each).
static bool box_light(Stage& s, const Sunlit& lit, std::vector<mat4f> places, float round) {
    const int colours = 20;
    if (places.size() > 64) return false;
    static float4 poseX[64], poseY[64], poseZ[64], poseAt[64];
    for (size_t i = 0; i < places.size(); ++i) {
        const mat4f& p = places[i];
        poseX[i] = {normalize(p[0].xyz), length(p[0].xyz) * 0.5f};
        poseY[i] = {normalize(p[1].xyz), length(p[1].xyz) * 0.5f};
        poseZ[i] = {normalize(p[2].xyz), length(p[2].xyz) * 0.5f};
        poseAt[i] = {p[3].xyz, 0.f};
    }
    MaterialInstance* mi = add(s, cube(round, 16), "boxlight.filamat", {mat4f{}}, 4, false, (int)places.size() * colours);
    if (!mi) return false;
    mi->setParameter("poseX", poseX, 64);
    mi->setParameter("poseY", poseY, 64);
    mi->setParameter("poseZ", poseZ, 64);
    mi->setParameter("poseAt", poseAt, 64);
    mi->setParameter("sunWay", lit.sunWay);
    mi->setParameter("colours", (float)colours);
    mi->setParameter("ior", 1.5f);
    mi->setParameter("abbe", 6.f);
    mi->setParameter("round", round);
    mi->setParameter("lean", 0.f);
    mi->setParameter("most", 6.f);
    mi->setParameter("gain", 1.f);
    mi->setParameter("floorY", 0.f);
    return true;
}

// A ball of radius 1.
static Mesh* ball(int around = 20, int up = 12) {
    Mesh* m = new Mesh;
    for (int j = 0; j <= up; ++j)
        for (int i = 0; i <= around; ++i) {
            const float a = TAU * (float)i / around, b = 0.5f * TAU * ((float)j / up - 0.5f);
            const float3 out{std::cos(a) * std::cos(b), std::sin(b), std::sin(a) * std::cos(b)};
            m->pos.push_back(out);
            m->nrm.push_back(out);
            m->uv.push_back({(float)i / around, (float)j / up});
        }
    for (int j = 0; j < up; ++j)
        for (int i = 0; i < around; ++i) {
            const uint32_t p = (uint32_t)(j * (around + 1) + i), q = p + 1, r = p + (uint32_t)around + 1, t = r + 1;
            m->idx.insert(m->idx.end(), {p, r, q, q, r, t});
        }
    return m;
}

// Bubbles of air caught in the glass blocks, as cast blocks have: small balls strewn inside each (most tiny, a few
// larger, some in a rising string), drawn into the picture of what is behind, so a block shows them the way it shows
// everything else: bent, and mirrored inside. each: how many a block. The same every run.
static bool bubbles(Stage& s, std::vector<mat4f> places, int each) {
    uint32_t seed = 11u;
    auto rnd = [&]() { seed = seed * 1664525u + 1013904223u; return (float)(seed >> 8) / 16777216.f; };
    std::vector<mat4f> all;
    for (const mat4f& block : places) {
        float3 at{0.f};
        for (int i = 0; i < each; ++i) {
            if (i % 6 == 0) at = float3{rnd() - 0.5f, rnd() - 0.5f, rnd() - 0.5f} * 0.8f;   // a new string starts
            else at = clamp(at + float3{(rnd() - 0.5f) * 0.06f, 0.03f + rnd() * 0.08f, (rnd() - 0.5f) * 0.06f}, float3{-0.44f}, float3{0.44f});
            const float size = rnd() < 0.15f ? 0.012f + 0.014f * rnd() : 0.003f + 0.006f * rnd();   // (as a part of the block's edge)
            all.push_back(block * mat4f::translation(at) * mat4f::scaling(float3{size} / float3{length(block[0].xyz), length(block[1].xyz), length(block[2].xyz)} * length(block[0].xyz)));
        }
    }
    MaterialInstance* mi = add(s, ball(), "bubble.filamat", all);
    if (!mi) return false;
    mi->setParameter("strength", 1.f);
    return true;
}

// Three frames, the last one read back and written as a .png.
static bool shoot(Stage& s, const char* path) {
    static std::vector<uint8_t> pic;
    pic.resize((size_t)s.w * s.h * 4);
    for (int i = 0; i < 3; ++i) {
        while (!s.renderer->beginFrame(s.swap)) {}
        if (s.view_floor) s.renderer->render(s.view_floor);
        s.renderer->render(s.view_behind);
        s.renderer->render(s.view);
        if (i == 2)
            s.renderer->readPixels(0, 0, (uint32_t)s.w, (uint32_t)s.h,
                                   backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
        s.renderer->endFrame();
        s.engine->flushAndWait();
    }
    uint8_t* plane = pic.data();
    NSBitmapImageRep* rep = [[NSBitmapImageRep alloc] initWithBitmapDataPlanes:&plane pixelsWide:s.w pixelsHigh:s.h bitsPerSample:8 samplesPerPixel:4
                                                                        hasAlpha:YES isPlanar:NO colorSpaceName:NSDeviceRGBColorSpace bytesPerRow:s.w * 4 bitsPerPixel:32];
    return [[rep representationUsingType:NSBitmapImageFileTypePNG properties:@{}] writeToFile:@(path) atomically:YES];
}
