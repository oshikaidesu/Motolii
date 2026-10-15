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

#include <mach/mach.h>

#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <fstream>
#include <csignal>
#include <map>
#include <functional>
#include <string>
#include <thread>
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
    Camera* camera;      // the eye that looks at the stage: every picture of it is taken by this
    Camera* film;        // the film's own camera: the same, unless another eye looks (god_view); mattes in the film's picture are taken by it
    View* view_behind;   // the same scene without the glass (layer 2), drawn first into `behind`: what a block shows
    Texture* behind;
    View* view_through = nullptr; // the glass's far faces alone (layer 8), drawn second into `through`: which way they face (far_side.mat)
    Texture* through = nullptr;
    Texture* through_depth = nullptr;   // how far away those far faces are: thick_glass.mat's near faces measure the glass by it
    View* view_floor = nullptr;   // the floor from straight above (layer 4 only), drawn before all into `sunlight` (paper_and_sun)
    Texture* sunlight = nullptr;
    double now = 0.0;        // the stage's time, in seconds
    bool by_clock = false;   // true: played as the clock goes (nothing is waited for); false: picture by picture (each one exact)
    // a thing's own sound: the cut of its file (seconds of the file) and how fast, how far into that it is as it comes on,
    // and when on the stage it comes on and for how long (below 0: to the end)
    struct Sound { std::string file; double from, to, speed, into, on, lasts; };
    std::map<const void*, std::function<Sound()>> sounding;   // what is heard, each under its own thing: asked when a film is written
    std::map<const void*, std::function<void(double)>> playing;   // what goes on by itself as the stage's time does (films, pictures), each under its own thing
    std::vector<std::function<void()>> said_after;      // a word of their own after each line of `clock` and `soak`
    struct Made { utils::Entity thing; const Mesh* of; int nth; mat4f place; bool matte = false; };   // (matte: the shape a matte is cut by, in that matte's own scene)
    utils::Entity sun;                     // the sun, where there is one (paper_and_sun), and all that goes by its direction: `sun_to`
    std::vector<MaterialInstance*> by_sun;
    struct Matte { Scene* scene; View* view; Texture* picture; bool soft; Camera* eye; utils::Entity on; mat4f onPlace; };
    std::vector<Matte> mattes;   // the mattes: each a picture taken by its own eye, drawn before all else (matte); soft: some thing reads it blurred
    struct Cut { MaterialInstance* look; int slot, matte; float use, soft; };
    std::vector<Cut> cuts;       // which look reads which matte (matted): told where the matte's eye is before every picture
    Texture* white = nullptr;    // a white point: the matte of a thing that nothing cuts
    // (what is made once for this stage's engine and kept: a stage opened after another in the same program has its own)
    std::map<std::string, Material*> materials;    // each material file, read and built once
    void* solids = nullptr; void* solidPictures = nullptr;   // Filament's reader of 3D files and its reader of their pictures (solid.h)
    std::vector<Made> made;   // every thing put on the stage: of which mesh, at which of that mesh's places (a film moves them: `move`)
    std::vector<View*> layers;   // further pictures laid over the stage's own view, in order (layer_view); the first `layered` of them are drawn
    size_t layered = 0;
    std::function<void()> over;   // drawn last, over all of them (a whole-screen effect: screen.h)
    RenderTarget* out = nullptr;   // where the picture goes at last (none: the screen; the window's live picture: live.h)
};

static Stage open_stage(int w, int h) {
    Stage s{w, h};
    // (BACKEND=webgpu: Filament's WebGPU backend, Dawn, which is the way to Windows; it needs the Filament built from source, see dawn_probe/kit_wgpu.sh)
    const char* backend = getenv("BACKEND");
    s.engine = Engine::Builder().backend(backend && !strcmp(backend, "webgpu") ? Engine::Backend::WEBGPU : Engine::Backend::METAL).build();
    // (a picture that does nothing to its colours gets them through exactly: Filament may then take its one fine row of
    // numbers instead of a coarse block of them; with the block a white of 255 comes out 249)
    s.engine->setFeatureFlag("engine.color_grading.use_1d_lut", true);
    s.renderer = s.engine->createRenderer();
    s.scene = s.engine->createScene();
    s.view = s.engine->createView();
    s.swap = s.engine->createSwapChain((uint32_t)w, (uint32_t)h, 0);
    s.camera = s.engine->createCamera(utils::EntityManager::get().create());
    s.camera->setExposure(16.0f, 1.0f / 125.0f, 100.0f);
    s.film = s.camera;
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

// A further picture laid over the stage's own (and over the ones before it): Filament's own way of laying one picture
// over another, a view of its own scene, blended on (its UI is laid so: ImGuiHelper). flat: a picture of flat things,
// which takes nothing of depth (no depth of field, no bloom, no shade where things meet); its colours still go the way
// the stage's own do (with no post-processing at all they would not be made ready for the screen: darker). Made when
// first wanted, lit by the stage's own room, seen by the stage's eye; the k-th is layers[k].
static View* layer_view(Stage& s, size_t k, bool flat) {
    while (s.layers.size() <= k) {
        View* v = s.engine->createView();
        v->setScene(s.engine->createScene());
        v->getScene()->setIndirectLight(s.scene->getIndirectLight());
        v->setCamera(s.camera);
        v->setViewport(s.view->getViewport());
        v->setDithering(View::Dithering::NONE);
        v->setAntiAliasing(s.view->getAntiAliasing());
        v->setAmbientOcclusionOptions(s.view->getAmbientOcclusionOptions());
        v->setColorGrading(const_cast<ColorGrading*>(s.view->getColorGrading()));   // (the same one: Filament hands it out read-only)
        v->setScreenSpaceRefractionEnabled(false);
        v->setVisibleLayers(0x03, 0x03);
        v->setBlendMode(View::BlendMode::TRANSLUCENT);
        if (s.view->getRenderTarget()) v->setRenderTarget(s.view->getRenderTarget());
        s.layers.push_back(v);
        Renderer::ClearOptions clear = s.renderer->getClearOptions();   // (a picture laid over another begins clear, not black: the stage's own view covers all of it anyway)
        clear.clearColor = {0, 0, 0, 0};
        s.renderer->setClearOptions(clear);
    }
    View::AmbientOcclusionOptions shade = s.view->getAmbientOcclusionOptions();
    shade.enabled = shade.enabled && !flat;
    s.layers[k]->setAmbientOcclusionOptions(shade);
    return s.layers[k];
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

static Texture* white_point(Stage& s) {
    static const uint8_t one[4] = {255, 255, 255, 255};
    if (!s.white) {
        s.white = Texture::Builder().width(1).height(1).levels(1).format(Texture::InternalFormat::RGBA8).build(*s.engine);
        s.white->setImage(*s.engine, 0, backend::PixelBufferDescriptor(one, 4, backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
    }
    return s.white;
}

// The flat sheet's material for a way of lying on what is behind it: nullptr (over it, as usual), or "add", "multiply",
// "screen": Filament's own blending, flat.mat built once for each.
static std::string flat_laid(const char* lay) { return lay ? std::string("flat_") + lay + ".filamat" : "flat.filamat"; }

// A mesh on the stage, once per place. layer 2: left out of the picture of what is behind (all the glass);
// layer 4: only in the floor's picture of sunlight; layer 8: only in the picture of the glass's far faces. order: among things of a kind, lower is drawn first.
// copies: how many times each is drawn in one go (a material that asks for its instance index tells them apart).
// A material by its file: read and built once (every thing of it has its own look). nullptr: there is no such file.
static Material* material(Stage& s, const char* filamat) {
    auto& built = s.materials;
    if (!built.count(filamat)) {
        const std::vector<uint8_t> pkg = load_file(filamat);
        built[filamat] = pkg.empty() ? nullptr : Material::Builder().package(pkg.data(), pkg.size()).build(*s.engine);
    }
    return built[filamat];
}
static MaterialInstance* add(Stage& s, Mesh* m, const char* filamat, std::vector<mat4f> places = {mat4f{}}, uint8_t layer = 1,
                             bool shadows = false, int copies = 1, int order = 4) {
    Material* mat = material(s, filamat);
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
    if (mat->hasParameter("opacity")) mi->setParameter("opacity", 1.f);   // (all there, until it is told otherwise)
    if (mat->hasParameter("matteUse")) {   // (a material that can be cut: by nothing, until `matted` says so)
        for (const char* name : {"matte0", "matte1", "matte2", "matte3"}) mi->setParameter(name, white_point(s), TextureSampler());
        for (const char* name : {"matteEye0", "matteEye1", "matteEye2", "matteEye3"}) mi->setParameter(name, mat4f{});
        if (mat->hasParameter("colours")) mi->setParameter("colours", white_point(s), TextureSampler());   // (a film's second plane: video.h)
        mi->setParameter("matteUse", float4{0.f});
        mi->setParameter("matteSoft", float4{0.f});
    }
    float3 low = m->pos[0], high = m->pos[0];   // (Filament wants a thing's rough size if it casts or takes shadows)
    for (const float3& p : m->pos) { low = min(low, p); high = max(high, p); }
    for (const mat4f& place : places) {
        utils::Entity e = utils::EntityManager::get().create();
        s.engine->getTransformManager().create(e, {}, place);
        RenderableManager::Builder(1).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib).material(0, mi)
            .boundingBox({(low + high) * 0.5f, max((high - low) * 0.5f, float3{1e-3f})}).culling(false).castShadows(shadows).receiveShadows(shadows).layerMask(0xFF, layer).instances((size_t)copies).priority((uint8_t)order).build(*s.engine, e);
        s.scene->addEntity(e);
        s.made.push_back({e, m, (int)(&place - places.data()), place});
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

// A ball of radius r, round from pole to pole.
static Mesh* sphere(float r, int around = 48, int about = 24) {
    Mesh* m = new Mesh;
    for (int i = 0; i <= around; ++i)
        for (int k = 0; k <= about; ++k) {
            const float a = TAU * (float)i / around, b = 0.5f * TAU * (float)k / about - 0.25f * TAU;
            const float3 out{std::cos(a) * std::cos(b), std::sin(b), std::sin(a) * std::cos(b)};
            m->pos.push_back(out * r);
            m->nrm.push_back(out);
            m->uv.push_back({(float)i / around, (float)k / about});
            if (i < around && k < about) {
                const uint32_t p = (uint32_t)(i * (about + 1) + k), q = p + 1, s = p + (uint32_t)(about + 1), t = s + 1;
                m->idx.insert(m->idx.end(), {p, q, s, s, q, t});
            }
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
    float grain = 0.12f;                 // (the paper's tooth)
    float sky = 7000.f;                  // (the studio's light, which fills the shadows: low, so they stay dark)
    const char* studio = "out/lightroom/lightroom_ibl.ktx";   // (the room: it lights the shadows and is what Filament's glass mirrors)
    float sunWide = 0.022f;              // (the sun's width: how far shade spreads for every unit it flew; far wider than the true sun, for the softness)
};
static bool paper_and_sun(Stage& s, const Sunlit& lit) {
    if (!room(s, lit.studio, lit.sky)) return false;
    s.sun = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::SUN).color({1.0f, 0.98f, 0.95f}).intensity(lit.sunLux).direction(lit.sunWay)
        .castShadows(false).build(*s.engine, s.sun);
    s.scene->addEntity(s.sun);

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
    s.by_sun.push_back(bare);
    bare->setParameter("value", 1.f);
    bare->setParameter("floorY", 0.f);
    Mesh* sheet = new Mesh(*floor);   // (the paper itself reaches further than its picture of sunlight: beyond it the sun is bare)
    for (float3& p : sheet->pos) p = p * 4.f;
    // The paper is drawn once, into the picture of the scene without the glass (layer 16: that picture only); on the
    // screen it is a copy out of that picture (layer 2: the screen only).
    MaterialInstance* paper = add(s, sheet, "paper.filamat", {mat4f{}}, 16);
    MaterialInstance* copy = add(s, sheet, "paper_copy.filamat", {mat4f{}}, 2);
    if (!paper || !copy) return false;
    s.view_behind->setVisibleLayers(0x10, 0x10);
    copy->setParameter("behind", s.behind, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST, TextureSampler::WrapMode::CLAMP_TO_EDGE));
    paper->setParameter("color", lit.paper);
    paper->setParameter("grain", lit.grain);
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
    s.by_sun.push_back(mi);
    mi->setParameter("value", 0.f);
    mi->setParameter("floorY", 0.f);
    return true;
}

// The light a glass thing throws on the paper (glasslight.mat): the thing's own mesh at its places, once a colour.
// Exaggerated apart from the look of the glass: the colours' indices far apart, the light a little stronger than true.
// uneven: how far its faces lean from their own facing (0 for a mesh that is uneven itself), over `unevenSize` of the mesh's units.
static MaterialInstance* glass_light(Stage& s, const Sunlit& lit, Mesh* m, std::vector<mat4f> places, float uneven = 0.f, float unevenSize = 1.f) {
    const int colours = 20;
    MaterialInstance* mi = add(s, m, "glasslight.filamat", places, 4, false, colours);
    if (!mi) return nullptr;
    mi->setParameter("sunWay", lit.sunWay);
    s.by_sun.push_back(mi);
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
    return mi;
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
    s.by_sun.push_back(mi);
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

// How deep a solid is as a rule: four times its volume over its surface (a ball: two thirds of its width; a bar: its
// width; a sheet: twice its thickness). What the glasses below take their depth from, so that a scene gives none.
static float usual_depth(const Mesh* m) {
    double volume = 0, surface = 0;
    for (size_t i = 0; i + 2 < m->idx.size(); i += 3) {
        const float3 a = m->pos[m->idx[i]], b = m->pos[m->idx[i + 1]], c = m->pos[m->idx[i + 2]];
        volume += dot(a, cross(b, c)) / 6.0;
        surface += length(cross(b - a, c - a)) / 2.0;
    }
    return (float)(4.0 * std::fabs(volume) / surface);
}

// Glass by Filament's own means (own_glass.mat: its refraction, absorption, dispersion and mirroring of the room),
// put down as it comes: its depth is the mesh's own. colour: what white light is left as after a usual depth of
// this glass ({1, 1, 1}: clear). crushed: how far its faces lean this way and that (0: smooth).
// Thin, bent solids (blades, ribbons, sheets) read well in it; it does not show its own far side as thick_glass does.
static MaterialInstance* own_glass(Stage& s, Mesh* m, std::vector<mat4f> places, float3 colour = float3{1.f}, float crushed = 0.f) {
    MaterialInstance* mi = add(s, m, "own_glass.filamat", places, 2);
    if (!mi) return nullptr;
    const float deep = 8.f * usual_depth(m) * length(places[0][0].xyz);   // (overdone eight times: what is behind swims in it)
    mi->setParameter("thickness", deep);
    mi->setParameter("absorb", float3{-std::log(std::max(colour.x, 1e-3f)), -std::log(std::max(colour.y, 1e-3f)), -std::log(std::max(colour.z, 1e-3f))} / deep);
    mi->setParameter("lean", crushed);
    mi->setParameter("film", 0.f);
    mi->setParameter("split", 0.5f);
    return mi;
}

// Glass seen through both of its sides (thick_glass.mat): the mesh at its places, drawn three times. Its far faces go
// into a picture of their own, which way they face and how far away they are (far_side.mat; nothing else is in that
// picture), and as they look from within over the picture of the scene without the glass (far_look.mat);
// its near faces, on the screen, bend the eye's way in, look the far face up there, and bend it out again to the scene.
// The first call makes the picture. Nothing is asked of the caller: the glass is put down as it comes.
static MaterialInstance* thick_glass(Stage& s, Mesh* m, std::vector<mat4f> places) {
    if (!s.view_through) {
        s.through = Texture::Builder().width(s.w).height(s.h).levels(1).usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE)
                        .format(Texture::InternalFormat::RGBA16F).build(*s.engine);
        s.through_depth = Texture::Builder().width(s.w).height(s.h).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT | Texture::Usage::SAMPLEABLE)
                              .format(Texture::InternalFormat::DEPTH32F).build(*s.engine);
        s.view_through = s.engine->createView();
        s.view_through->setScene(s.scene);
        s.view_through->setCamera(s.camera);
        s.view_through->setViewport({0, 0, (uint32_t)s.w, (uint32_t)s.h});
        s.view_through->setRenderTarget(RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, s.through).texture(RenderTarget::AttachmentPoint::DEPTH, s.through_depth).build(*s.engine));
        s.view_through->setPostProcessingEnabled(false);
        s.view_through->setVisibleLayers(0xFF, 0x08);
    }
    MaterialInstance* back = add(s, m, "far_side.filamat", places, 8);
    MaterialInstance* look = add(s, m, "far_look.filamat", places, 16);   // (layer 16: only in the picture of the scene without the glass)
    MaterialInstance* front = add(s, m, "thick_glass.filamat", places, 2);
    if (!back || !look || !front) return nullptr;
    back->setCullingMode(backend::CullingMode::FRONT);
    look->setCullingMode(backend::CullingMode::FRONT);
    s.view_behind->setVisibleLayers(0x10, 0x10);
    const TextureSampler smooth(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE);
    const TextureSampler exact(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST, TextureSampler::WrapMode::CLAMP_TO_EDGE);
    front->setParameter("behind", s.behind, smooth);
    front->setParameter("farSide", s.through, smooth);
    front->setParameter("farDepth", s.through_depth, exact);
    // How deep the solid is as a rule: four times its volume over its surface (a ball: two thirds of its width; a bar:
    // its width; a sheet: twice its thickness). Worked out once, here; a way through the glass much longer than this
    // is the glass seen along itself, and goes dark.
    const float usual = usual_depth(m) * length(places[0][0].xyz);
    // A bar or a sheet has places where it is seen along itself; a lump (a ball, a gem) has none: its longest way is
    // straight through its middle, about one and a half times `usual`. So the nearer to a lump the solid is (how deep
    // against how far it reaches), the longer a way must be before it counts as long: the middle of a gem stays clear.
    float3 mid{0.f};
    for (float3 p : m->pos) mid += p;
    mid = mid / (float)m->pos.size();
    float reach = 0.f;
    for (float3 p : m->pos) reach = std::max(reach, length(p - mid));
    const float lump = usual / (2.f * reach * length(places[0][0].xyz));   // (a ball: 0.67; a cube: 0.38; a ring of bar: about 0.1)
    const float longFrom = 1.f + 0.7f * std::min(std::max((lump - 0.3f) / 0.25f, 0.f), 1.f);
    front->setParameter("ior", 1.5f);
    front->setParameter("thickness", 0.8f * usual);
    front->setParameter("swim", 0.5f);       // (what is behind the glass swims in it, overdone; at 1 letters melt)
    front->setParameter("usual", usual);
    front->setParameter("longFrom", longFrom);
    front->setParameter("dark", 4.f);        // (how fast: at 7 a ring seen edge-on carries two navy bruises; at 3 the darks are pale lilac)
    front->setParameter("edgeFire", 13.f);   // (14: a yellow haze far beside every dark; 20: an orange stain along every edge)
    front->setParameter("split", 0.10f);     // (0.2: the colours turn to blocks)
    front->setParameter("samples", 6.f);     // (each one colour of the rainbow; 8: 0.6 ms dearer)
    front->setParameter("vivid", 1.3f);      // (laid on after the dark, so the rainbow at its edge is the most vivid thing; 1.35 and up: neon)
    front->setParameter("roomGain", 1.f);    // (the glass's own room, studio.glsl: the same wherever the glass is put)
    return front;
}

// A colour of a picture made clear (a green screen): for a sheet that shows a picture (flat.mat). near: how near to
// the colour counts as the screen itself (in the two colour numbers of video, where pure green is 0.6 from grey: the
// screen's own noise must fit in it); soft: over how much more the picture comes in (a point half screen, half grey
// is 0.3 from the screen); spill: how far from the screen its colour is still taken out of what stays.
static void key_out(MaterialInstance* look, float3 colour = float3{0.f, 1.f, 0.f}, float near = 0.12f, float soft = 0.3f, float spill = 0.9f) {
    look->setParameter("key", float4{colour, near});
    look->setParameter("keySoft", float2{soft, spill});
}

// A matte, as After Effects has them: a shape that is not seen itself and cuts other things. Any mesh is one: a solid,
// or a sheet with a picture on it (ThorVG's), which cuts by how clear the picture is, or by its light (After Effects'
// luma matte). It is a thing like any other: `move` moves it.
// on: where the matte is. nullptr: in the film's picture, a window on the screen: it cuts by where its shape is seen
// from the film's camera, whatever lies between, and stays in the picture while things move behind it. A mesh on the
// stage: on that thing, a sticker: it rides on the thing (Filament's own parent and child) and cuts by where its
// shape lies on it, seen along the thing's own z.
// strength: how much it covers at most (a half-clear matte). into: a matte made before, which this shape joins.
// What comes back names the matte. Each matte is a picture of its own, taken before all else (`draw`) by its own eye,
// the film's camera or one riding on the thing, with nothing but its shapes in it; the things it cuts look themselves
// up in it by that eye (matte.glsl), so they are cut the same whichever eye looks at the stage.
static int matte(Stage& s, Mesh* m, std::vector<mat4f> places, Texture* picture = nullptr, bool byLight = false, float strength = 1.f, int into = -1, const Mesh* on = nullptr) {
    auto& tm = s.engine->getTransformManager();
    if (into < 0) {
        Stage::Matte made{};
        uint32_t w = 2 * (uint32_t)s.w, h = 2 * (uint32_t)s.h;   // (in the film's picture: twice as fine as the screen)
        made.eye = s.film;
        made.onPlace = mat4f{};
        for (const Stage::Made& d : s.made) {
            if (!on || d.of != on) continue;
            float3 low = on->pos[0], high = on->pos[0];
            for (const float3& p : on->pos) { low = min(low, p); high = max(high, p); }
            // an eye before the thing, looking along its z at all of it, and going where it goes
            utils::Entity e = utils::EntityManager::get().create();
            tm.create(e, tm.getInstance(d.thing), mat4f::translation(float3{0.f, 0.f, high.z + 1.f}));
            made.eye = s.engine->createCamera(e);
            made.eye->setProjection(Camera::Projection::ORTHO, low.x, high.x, low.y, high.y, 0.1, double(high.z - low.z) + 2.0);
            const float fine = (float)std::max(s.w, s.h) / std::max(high.x - low.x, high.y - low.y);   // (its longer side as fine as the screen's)
            w = 2 * (uint32_t)std::lround(0.5f * fine * (high.x - low.x));
            h = 2 * (uint32_t)std::lround(0.5f * fine * (high.y - low.y));
            made.on = d.thing;
            made.onPlace = d.place;
            break;
        }
        uint8_t levels = 1;
        while ((std::max(w, h) >> levels) > 0) ++levels;
        made.picture = Texture::Builder().width(w).height(h).levels(levels).format(Texture::InternalFormat::R8)
                           .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::GEN_MIPMAPPABLE).build(*s.engine);
        made.scene = s.engine->createScene();
        made.view = s.engine->createView();
        made.view->setScene(made.scene);
        made.view->setCamera(made.eye);
        made.view->setViewport({0, 0, w, h});
        made.view->setRenderTarget(RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, made.picture).build(*s.engine));
        made.view->setPostProcessingEnabled(false);
        s.mattes.push_back(made);
        into = (int)s.mattes.size() - 1;
    }
    Stage::Matte& to = s.mattes[(size_t)into];
    MaterialInstance* shape = add(s, m, "matte.filamat", places);
    if (!shape) return -1;
    for (size_t i = s.made.size() - places.size(); i < s.made.size(); ++i) {   // (its shape is in the matte's own scene, and in no other picture)
        s.scene->remove(s.made[i].thing);
        to.scene->addEntity(s.made[i].thing);
        s.made[i].matte = true;
        if (to.eye == s.film) continue;
        s.made[i].place = inverse(to.onPlace) * s.made[i].place;   // (on a thing: its place is told from the thing from now on)
        tm.setParent(tm.getInstance(s.made[i].thing), tm.getInstance(to.on));
        tm.setTransform(tm.getInstance(s.made[i].thing), s.made[i].place);
    }
    shape->setParameter("picture", picture ? picture : white_point(s), TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
    shape->setParameter("byLight", byLight ? 1.f : 0.f);
    shape->setParameter("strength", strength);
    if (!picture) shape->setCullingMode(backend::CullingMode::BACK);   // (a solid: its near faces only, so that it covers once)
    return into;
}

// The thing with this look is cut by that matte: seen only inside it (or only outside it). soft: how far the edge is
// blurred, in pixels of the screen (of the matte's own picture, if it is on a thing; 0: as sharp as the matte is). Up to four mattes cut one thing (call again): what
// all of them leave is seen. false: the look's material cannot be cut (it does not read mattes: matte.glsl), or four cut it already.
static bool matted(Stage& s, MaterialInstance* look, int matte, bool inside = true, float soft = 0.f) {
    static const char* const names[] = {"matte0", "matte1", "matte2", "matte3"};
    float4 use{0.f}, softs{0.f};   // (what cuts this look already: the stage's own list says)
    int n = 0;
    for (const Stage::Cut& c : s.cuts)
        if (c.look == look) use[c.slot] = c.use, softs[c.slot] = c.soft, ++n;
    if (n == 4 || !look->getMaterial()->hasParameter("matteUse")) return false;
    look->setParameter(names[n], s.mattes[(size_t)matte].picture,
                       TextureSampler(TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE));
    use[n] = inside ? 1.f : -1.f;
    softs[n] = std::log2(1.f + 2.f * std::max(soft, 0.f));   // (a level of the matte's picture: each is twice as coarse as the one before)
    look->setParameter("matteUse", use);
    look->setParameter("matteSoft", softs);
    if (soft > 0.f) s.mattes[(size_t)matte].soft = true;
    s.cuts.push_back({look, n, matte, use[n], softs[n]});
    return true;
}

// Another eye on the stage than the film's camera: the stage is looked at through it from now on (`camera`), as one
// stands beside a set and watches; the film's own camera (`film`) goes on taking the mattes that are in the film's picture.
static Camera* god_view(Stage& s) {
    s.camera = s.engine->createCamera(utils::EntityManager::get().create());
    s.camera->setExposure(16.0f, 1.0f / 125.0f, 100.0f);
    for (View* v : {s.view, s.view_behind, s.view_through})
        if (v) v->setCamera(s.camera);
    return s.camera;
}

// Three frames, the last one read back and written as a .png.
// One picture of the stage drawn (the helper pictures first), and read back into `pic` if one is given.
// The stage's time set: everything that goes on by itself is at that moment for the picture drawn next.
static void time_is(Stage& s, double seconds) {
    s.now = seconds;
    for (const auto& [who, goes] : s.playing) goes(seconds);
}

// A frame begins (Filament's beginFrame: false if this one is to be left out). What the looks are told is sent on
// as a frame begins, so each matte's eye is told just before: where it is now, a pose handed over, nothing more.
static bool begins(Stage& s) {
    static const char* const eyes[] = {"matteEye0", "matteEye1", "matteEye2", "matteEye3"};
    for (const Stage::Cut& c : s.cuts) {
        const Camera* eye = s.mattes[(size_t)c.matte].eye;
        c.look->setParameter(eyes[c.slot], mat4f(eye->getProjectionMatrix() * eye->getViewMatrix()));
    }
    return s.renderer->beginFrame(s.swap);
}

// All the pictures of one frame, the helper pictures first (between Filament's beginFrame and endFrame). Every way
// of drawing the stage goes through here.
static void pictures_of(Stage& s) {
    for (Stage::Matte& m : s.mattes) {
        s.renderer->render(m.view);
        if (m.soft) m.picture->generateMipmaps(*s.engine);
    }
    if (s.view_floor) s.renderer->render(s.view_floor);
    s.renderer->render(s.view_behind);
    if (s.view_through) s.renderer->render(s.view_through);
    s.renderer->render(s.view);
    for (size_t k = 0; k < s.layered && k < s.layers.size(); ++k) s.renderer->render(s.layers[k]);
    if (s.over) s.over();
}

static void draw(Stage& s, std::vector<uint8_t>* pic = nullptr, bool wait = true) {
    while (!begins(s)) {}
    pictures_of(s);
    if (pic)
        s.renderer->readPixels(0, 0, (uint32_t)s.w, (uint32_t)s.h,
                               backend::PixelBufferDescriptor(pic->data(), pic->size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
    s.renderer->endFrame();
    if (wait) s.engine->flushAndWait();
}

// Whether the things made of mesh `m` are in the picture (they stay made, and are moved and played as before).
static void seen(Stage& s, const Mesh* m, bool in) {
    for (const Stage::Made& d : s.made)
        if (d.of == m) { if (in) s.scene->addEntity(d.thing); else s.scene->remove(d.thing); }
}

// The things made of mesh `m` taken off the stage for good.
static void take_off(Stage& s, const Mesh* m) {
    auto& looks = s.engine->getRenderableManager();
    MaterialInstance* look = nullptr;
    for (size_t i = s.made.size(); i-- > 0;) {
        if (s.made[i].of != m) continue;
        look = looks.getMaterialInstanceAt(looks.getInstance(s.made[i].thing), 0);
        s.scene->remove(s.made[i].thing);
        s.engine->destroy(s.made[i].thing);
        s.made.erase(s.made.begin() + (long)i);
    }
    if (!look) return;
    std::erase_if(s.cuts, [look](const Stage::Cut& c) { return c.look == look; });   // (no matte's eye is told to it any more)
    s.engine->destroy(look);   // (its look goes with it)
}

// A film moves things between its pictures: every thing made of mesh `m` goes where `to` says (told which of the
// mesh's places it is, and that place). Poses only: nothing is worked out here that a picture is made of.
static void move(Stage& s, const Mesh* m, const std::function<mat4f(int, const mat4f&)>& to) {
    auto& tm = s.engine->getTransformManager();
    for (const Stage::Made& d : s.made)
        if (d.of == m) tm.setTransform(tm.getInstance(d.thing), to(d.nth, d.place));
}

// A film of the stage, 60 pictures a second, written by ffmpeg. Before each picture `pose` is told how far through
// the film it is (0 up to 1): where the end meets the start, the film loops. What is heard is the things' own sound
// (Stage::sounding), cut, sped, gone round and mixed by ffmpeg as each thing says.
static bool film(Stage& s, const char* path, int pictures, const std::function<void(float)>& pose) {
    const auto quoted = [](std::string name) {   // (a file's name as a shell takes it)
        for (size_t at = 0; (at = name.find('\'', at)) != std::string::npos; at += 4) name.replace(at, 1, "'\\''");
        return "'" + name + "'";
    };
    std::string files, each, all;
    int n = 0;
    for (const auto& [who, tells] : s.sounding) {
        const Stage::Sound a = tells();
        const double fast = std::fabs(a.speed);
        char how[700];
        std::string halved;   // (ffmpeg's atempo goes no slower than half at a time)
        double left = fast;
        for (; left < 0.5; left /= 0.5) halved += "atempo=0.5,";
        std::snprintf(how, sizeof how, "[%d:a]aresample=48000,atrim=start=%g:end=%g,asetpts=PTS-STARTPTS,%s%satempo=%g,aloop=loop=-1:size=%ld,atrim=start=%g%s%s,asetpts=PTS-STARTPTS,adelay=delays=%ld:all=1[a%d];",
                      n + 1, a.from, a.to, a.speed < 0.0 ? "areverse," : "", halved.c_str(), left, std::lround((a.to - a.from) / fast * 48000.0), a.into,
                      a.lasts < 0.0 ? "" : ":duration=", a.lasts < 0.0 ? "" : std::to_string(a.lasts).c_str(), std::lround(a.on * 1000.0), n + 1);
        files += " -i " + quoted(a.file);
        each += how;
        all += "[a" + std::to_string(++n) + "]";
    }
    const std::string sound = n ? files + " -filter_complex \"" + each + all + "amix=inputs=" + std::to_string(n) + ":normalize=0:duration=longest,alimiter[a]\" -map 0:v -map '[a]' -c:a aac -b:a 160k -t " + std::to_string(pictures / 60.0) : "";
    const std::string cmd = "ffmpeg -loglevel error -y -f rawvideo -pix_fmt rgba -s " + std::to_string(s.w) + "x" + std::to_string(s.h) + " -r 60 -i -" + sound + " -c:v libx264 -crf 17 -g 60 -pix_fmt yuv420p -movflags +faststart " + quoted(path);
    std::signal(SIGPIPE, SIG_IGN);   // (if ffmpeg gives up, writing to it fails and that is told, instead of this program ending with it)
    FILE* to = popen(cmd.c_str(), "w");
    if (!to) return false;
    std::vector<uint8_t> pic((size_t)s.w * s.h * 4);
    s.by_clock = false;
    for (int i = 0; i < pictures; ++i) {
        time_is(s, (double)i / 60.0);
        pose((float)i / (float)pictures);
        draw(s, &pic);
        if (std::fwrite(pic.data(), 1, pic.size(), to) != pic.size()) break;
    }
    return pclose(to) == 0;
}

// The same film played without being kept, to see what a picture costs. Two measures.
// Flat out, the GPU waited for after every fourth picture: pictures a second. Steady to a few hundredths on Metal, so
// good for telling two versions apart there; NOT for comparing backends: how often one waits moves it by a
// millisecond or two, and differently on each (waiting after every picture or every second one, WebGPU and Metal
// read alike; after every fourth, WebGPU reads a millisecond or two dearer; seldom or never, both are held back).
// (And at 60 a second the GPU slows its clock, so a picture's own time cannot be read off a paced run either.)
static void clock(Stage& s, int pictures, const std::function<void(float)>& pose) {
    s.by_clock = true;
    const auto from = std::chrono::steady_clock::now();
    for (int i = 0; i < pictures; ++i) {
        time_is(s, (double)i / 60.0);
        pose((float)i / (float)pictures);
        draw(s, nullptr, i % 4 == 3);
    }
    s.engine->flushAndWait();
    const double took = std::chrono::duration<double>(std::chrono::steady_clock::now() - from).count();
    std::printf("%.0f pictures a second, %.2f ms each (%.1f times 60)\n", pictures / took, 1000.0 * took / pictures, pictures / took / 60.0);
    for (const auto& word : s.said_after) word();
    // And by the clock, as a screen asks for them: on every tick Filament is asked once whether it will draw (it says
    // no when it is behind), nothing is waited for. Pictures drawn and pictures left out, at 60, 120 and 240 a second.
    for (int rate : {60, 120, 240}) {
        int drawn = 0, left_out = 0;
        const auto start = std::chrono::steady_clock::now();
        for (int i = 0; i < 2 * rate; ++i) {   // two seconds
            std::this_thread::sleep_until(start + std::chrono::microseconds((long long)i * 1000000 / rate));
            time_is(s, (double)i / (double)rate);
            pose((float)(i % (pictures * rate / 60)) / (float)(pictures * rate / 60));
            if (!begins(s)) { ++left_out; continue; }
            pictures_of(s);
            s.renderer->endFrame();
            ++drawn;
        }
        const auto asked = std::chrono::steady_clock::now();
        s.engine->flushAndWait();
        std::printf("at %d a second: %d drawn, %d left out; the last was done %.1f ms after it was asked for\n", rate, drawn, left_out,
                    std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - asked).count());
        for (const auto& word : s.said_after) word();
    }
}

// The stage played on and on by the clock at 60 a second, with no jump in time (the film goes round as often as it
// takes): how it holds up over minutes. A line a minute: pictures drawn and left out (Filament says no when it is
// behind), the longest this thread took over one picture, and how much memory the program holds.
static void soak(Stage& s, double seconds, int pictures, const std::function<void(float)>& pose) {
    s.by_clock = true;
    const auto start = std::chrono::steady_clock::now();
    const long ticks = (long)(seconds * 60.0);
    int drawn = 0, left_out = 0, all_drawn = 0, all_left = 0;
    double longest = 0.0;
    for (long i = 0; i < ticks; ++i) {
        std::this_thread::sleep_until(start + std::chrono::microseconds(i * 1000000 / 60));
        const auto began = std::chrono::steady_clock::now();
        time_is(s, (double)i / 60.0);
        pose((float)((double)i / (double)pictures));   // (on past 1: time does not go back)
        if (begins(s)) {
            pictures_of(s);
            s.renderer->endFrame();
            ++drawn;
        } else ++left_out;
        longest = std::max(longest, std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - began).count());
        if ((i + 1) % 3600 == 0 || i + 1 == ticks) {
            mach_task_basic_info_data_t info;
            mach_msg_type_number_t count = MACH_TASK_BASIC_INFO_COUNT;
            task_info(mach_task_self(), MACH_TASK_BASIC_INFO, (task_info_t)&info, &count);
            std::printf("minute %2ld: %d drawn, %d left out; longest %.1f ms; memory %.0f MB\n", (i + 1 + 3599) / 3600, drawn, left_out, longest, (double)info.resident_size / 1048576.0);
            std::fflush(stdout);
            for (const auto& word : s.said_after) word();
            all_drawn += drawn; all_left += left_out;
            drawn = left_out = 0;
            longest = 0.0;
        }
    }
    s.engine->flushAndWait();
    std::printf("in all: %d drawn, %d left out, over %.0f seconds\n", all_drawn, all_left, seconds);
}

static bool shoot(Stage& s, const char* path) {
    static std::vector<uint8_t> pic;
    pic.resize((size_t)s.w * s.h * 4);
    for (int i = 0; i < 3; ++i) draw(s, i == 2 ? &pic : nullptr);
    uint8_t* plane = pic.data();
    NSBitmapImageRep* rep = [[NSBitmapImageRep alloc] initWithBitmapDataPlanes:&plane pixelsWide:s.w pixelsHigh:s.h bitsPerSample:8 samplesPerPixel:4
                                                                        hasAlpha:YES isPlanar:NO colorSpaceName:NSDeviceRGBColorSpace bytesPerRow:s.w * 4 bitsPerPixel:32];
    return [[rep representationUsingType:NSBitmapImageFileTypePNG properties:@{}] writeToFile:@(path) atomically:YES];
}

// The sun turned to shine another way (a film's pose): the shade and the light the glass throws go with it.
static void sun_to(Stage& s, float3 way) {
    auto& lights = s.engine->getLightManager();
    lights.setDirection(lights.getInstance(s.sun), way);
    for (MaterialInstance* mi : s.by_sun) mi->setParameter("sunWay", way);
}

// What a picture's program does once its stage is set: the still (out/<name>.png); or, asked for `film`, the film
// (out/<name>.mp4); or, asked for `time`, the film played for its cost.
static int show(Stage& s, int argc, char** argv, const char* name, int pictures, const std::function<void(float)>& pose) {
    const std::string still = std::string("out/") + name + ".png", moving = std::string("out/") + name + ".mp4";
    if (argc > 1 && !strcmp(argv[1], "time")) return clock(s, pictures, pose), 0;
    if (argc > 1 && !strcmp(argv[1], "soak")) return soak(s, argc > 2 ? atof(argv[2]) : 60.0, pictures, pose), 0;   // (soak 600: ten minutes)
    const bool as_film = argc > 1 && !strcmp(argv[1], "film");
    if (!as_film) {   // (a still: the stage three tenths of the way through its film)
        s.by_clock = false;
        time_is(s, 0.3 * (double)pictures / 60.0);
        pose(0.3f);
    }
    if (!(as_film ? film(s, moving.c_str(), pictures, pose) : shoot(s, still.c_str()))) return fail(as_film ? "film" : "png");
    std::printf("wrote %s\n", (as_film ? moving : still).c_str());
    return 0;
}
