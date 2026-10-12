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

// A mesh on the stage, once per place. layer 2: left out of the picture of what is behind (all the glass).
// copies: how many times each is drawn in one go (a material that asks for its instance index tells them apart).
static MaterialInstance* add(Stage& s, Mesh* m, const char* filamat, std::vector<mat4f> places = {mat4f{}}, uint8_t layer = 1,
                             bool shadows = false, int copies = 1) {
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
            .boundingBox({(low + high) * 0.5f, max((high - low) * 0.5f, float3{1e-3f})}).culling(false).castShadows(shadows).receiveShadows(shadows).layerMask(0xFF, layer).instances((size_t)copies).build(*s.engine, e);
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

// Pale paper under a hard sun, with one of Filament's studios as the sky that fills the shadows (kept low, so the
// shadows stay dark). What the glass things stand on in blocks.mm and shapes.mm.
struct Sunlit {
    float3 paper, sunWay;
    float sunLux;
};
static bool paper_and_sun(Stage& s, Sunlit& lit) {
    lit = {{0.70f, 0.77f, 0.82f}, normalize(float3{-0.34f, -0.75f, 0.67f}), 100000.f};
    if (!room(s, "out/lightroom/lightroom_ibl.ktx", 7000.f)) return false;
    LightManager::ShadowOptions shadow;
    shadow.mapSize = 4096;
    utils::Entity sun = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::SUN).color({1.0f, 0.98f, 0.95f}).intensity(lit.sunLux).direction(lit.sunWay)
        .castShadows(true).shadowOptions(shadow).build(*s.engine, sun);
    s.scene->addEntity(sun);
    Mesh* floor = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) {
        floor->pos.push_back({30.f * c.x, 0.f, -30.f * c.y});
        floor->nrm.push_back({0, 1, 0});
        floor->uv.push_back(c * 0.5f + 0.5f);
    }
    floor->idx = {0, 1, 2, 0, 2, 3};
    MaterialInstance* floor_mi = add(s, floor, "floor_lit.filamat", {mat4f{}}, 1, true);
    if (!floor_mi) return false;
    floor_mi->setParameter("color", float4{lit.paper, 1.f});
    floor_mi->setParameter("roughness", 0.9f);
    return true;
}

// The light a glass thing throws on the paper (glasslight.mat): the thing's own mesh at its places, once a colour.
// Exaggerated apart from the look of the glass: the colours' indices far apart, the light a little stronger than true.
// uneven: how far its faces lean from their own facing (0 for a mesh that is uneven itself), over `unevenSize` of the mesh's units.
static bool glass_light(Stage& s, const Sunlit& lit, Mesh* m, std::vector<mat4f> places, float uneven = 0.f, float unevenSize = 1.f) {
    const int colours = 20;
    MaterialInstance* mi = add(s, m, "glasslight.filamat", places, 1, false, colours);
    if (!mi) return false;
    mi->setParameter("sunWay", lit.sunWay);
    mi->setParameter("sunOnFloor", lit.paper * (lit.sunLux / 3.14159265f));
    mi->setParameter("colours", (float)colours);
    float size = 0.f;
    for (const float3& p : m->pos) size = std::max(size, std::max(std::fabs(p.x), std::max(std::fabs(p.y), std::fabs(p.z))));
    mi->setParameter("size", size);
    mi->setParameter("ior", 1.5f);
    mi->setParameter("abbe", 2.f);
    mi->setParameter("lean", uneven);
    mi->setParameter("leanSize", unevenSize);
    mi->setParameter("facing", float2{0.6f, 0.74f});
    mi->setParameter("most", 6.f);
    mi->setParameter("gain", 1.4f);
    mi->setParameter("floorY", 0.f);
    return true;
}

// Three frames, the last one read back and written as a .png.
static bool shoot(Stage& s, const char* path) {
    static std::vector<uint8_t> pic;
    pic.resize((size_t)s.w * s.h * 4);
    for (int i = 0; i < 3; ++i) {
        while (!s.renderer->beginFrame(s.swap)) {}
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
