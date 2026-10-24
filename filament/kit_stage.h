// kit_stage.h: part of kit.h. Read only through kit.h.
#include <filament/Camera.h>
#include <filament/ColorGrading.h>
#include <filament/Engine.h>
#include <filament/IndexBuffer.h>
#include <filament/InstanceBuffer.h>
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
    VertexBuffer* vb = nullptr;   // (as last put in the scene: add_parts, reshape)
    IndexBuffer* ib = nullptr;
    uint32_t vbRoom = 0, ibRoom = 0;   // (how many points and indices they hold: written again in place while they fit)
};

// What a buffer is told, copied: Filament reads it later, on its own thread, and lets it go when it has (its callback),
// so the mesh may be made again meanwhile.
static backend::BufferDescriptor copied(const void* from, size_t bytes) {
    void* copy = malloc(bytes);
    memcpy(copy, from, bytes);
    return backend::BufferDescriptor(copy, bytes, [](void* buffer, size_t, void*) { free(buffer); });
}

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
    bool behindRead = false;   // whether anything reads `behind` (glass, the paper): until then it is not drawn
    View* view_through = nullptr; // the glass's far faces alone (layer 8), drawn second into `through`: which way they face (far_side.mat)
    Texture* through = nullptr;
    Texture* through_depth = nullptr;   // how far away those far faces are: thick_glass.mat's near faces measure the glass by it
    View* view_floor = nullptr;   // the floor from straight above (layer 4 only), drawn before all into `sunlight` (paper_and_sun)
    Texture* sunlight = nullptr;
    double now = 0.0;        // the stage's time, in seconds
    bool by_clock = false;   // true: played as the clock goes (nothing is waited for); false: picture by picture (each one exact)
    // (a live stage: what a moment asks of its films, kept to be done once the document is let go (motolii.mm), so
    //  that films put back to their start do not hold the shell's reads of it; empty: done at once)
    std::vector<std::function<void()>>* later = nullptr;
    // a thing's own sound: the cut of its file (seconds of the file) and how fast, how far into that it is as it comes on,
    // and when on the stage it comes on and for how long (below 0: to the end)
    struct Sound { std::string file; double from, to, speed, into, on, lasts; };
    std::map<const void*, std::function<Sound()>> sounding;   // what is heard, each under its own thing: asked when a film is written
    std::map<const void*, std::function<void(double)>> playing;   // what goes on by itself as the stage's time does (films, pictures), each under its own thing
    std::vector<std::function<void()>> said_after;      // a word of their own after each line of `clock` and `soak`
    struct Made { utils::Entity thing; const Mesh* of; int nth; mat4f place; bool matte = false;   // (matte: the shape a matte is cut by, in that matte's own scene)
                  InstanceBuffer* many = nullptr; int count = 1; };   // (all its places drawn in one go, Filament's own instancing: each place an instance of the one thing; `move` tells them all at once)
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
    // The desk: the editing window's whole stage pane (w, h pixels: 0, none), the picture there as on the stage but seen
    // further back, the stage's own lying at x, y (its top left) so many pixels a point; what lies off it is seen too.
    // Over it, the marks of the picked thing (its frame: middle, width, height, turn, the point it turns about, as
    // doc_frame gives them) and of the one under the hand; `point`: desk pixels a point of the screen the marks are sized in.
    struct { uint32_t w = 0, h = 0; double x = 0, y = 0, scale = 1, point = 2; math::mat4 was, wasCulled;
             bool picked = false, hovered = false, knob = true; double pick[8] = {}, hover[8] = {}; } desk;
    std::function<void()> over;   // drawn last, over all of them (a whole-screen effect: screen.h)
    std::vector<std::function<void()>> before;   // drawn before the stage's own view, each into a picture of its own (a thing with effects of its own: thing_fx.h)
    std::function<void()> lastly;   // and after everything, in the same frame (the desk: its stage's picture placed on it, live.h)
    Texture* drawnInto = nullptr;   // the picture the views draw into (live.h, screen.h); none: the screen
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
    s.view->setMultiSampleAntiAliasingOptions({.enabled = true, .sampleCount = 4});   // (the edges of flat shapes drawn as triangles, drawn.h flat_mesh: as Unity's and three.js's are)
    s.view->setScreenSpaceRefractionEnabled(true);
    s.view->setVisibleLayers(0x03, 0x03);
    Renderer::ClearOptions clear{};
    clear.clearColor = {0, 0, 0, 0};   // (clear, not black: the stage's own view covers all of it anyway, and a card's picture of one thing alone shows the card where it is not)
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

// The stage's pictures laid out on the desk (on: before they are drawn) or on the stage again (off: after): the views the
// stage's eye sees by, their viewport the desk's, the eye's projection moved and shrunk so the stage lies where the desk
// says. The floor's and the mattes' own eyes are left as they are.
static void desk_lay(Stage& s, bool on) {
    const Viewport at = on ? Viewport{0, 0, s.desk.w, s.desk.h} : Viewport{0, 0, (uint32_t)s.w, (uint32_t)s.h};
    for (View* v : {s.view, s.view_behind, s.view_through}) if (v) v->setViewport(at);
    const double near = s.camera->getNear(), far = s.camera->getCullingFar();
    if (!on) return s.camera->setCustomProjection(s.desk.was, s.desk.wasCulled, near, far);
    s.desk.was = s.camera->getProjectionMatrix(), s.desk.wasCulled = s.camera->getCullingProjectionMatrix();
    const double wide = s.w * s.desk.scale, high = s.h * s.desk.scale;
    math::mat4 further;   // (the stage's -1..1 to where it lies on the desk's)
    further[0][0] = wide / s.desk.w, further[1][1] = high / s.desk.h;
    further[3][0] = 2.0 * (s.desk.x + 0.5 * wide) / s.desk.w - 1.0, further[3][1] = 1.0 - 2.0 * (s.desk.y + 0.5 * high) / s.desk.h;
    s.camera->setCustomProjection(further * s.desk.was, further * s.desk.wasCulled, near, far);
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
        if (built[filamat]) built[filamat]->compile(Material::CompilerPriorityQueue::HIGH);   // (its programs made now, not at the first picture that needs them: Filament's own; the crowd's first minute left out 1-4 pictures so, 3-16 without)
    }
    return built[filamat];
}
// A mesh in the scene, once at each place, its triangles in parts (`ends`: where each part's indices end; none: one
// part), each part a look of its own of the same material (Filament's own: a renderable of so many primitives).
static std::vector<MaterialInstance*> add_parts(Stage& s, Mesh* m, const char* filamat, std::vector<mat4f> places, std::vector<uint32_t> ends,
                                                uint8_t layer = 1, bool shadows = false, int copies = 1, int order = 4, bool together = false) {
    Material* mat = material(s, filamat);
    if (!mat) return {};
    if (ends.empty()) ends = {(uint32_t)m->idx.size()};
    const size_t n = m->pos.size();
    auto* frames = geometry::SurfaceOrientation::Builder().vertexCount(n).normals(m->nrm.data()).build();
    m->quat.resize(n);
    frames->getQuats(m->quat.data(), n);
    delete frames;
    VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)n).bufferCount(3)
                           .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3)
                           .attribute(VertexAttribute::TANGENTS, 1, VertexBuffer::AttributeType::FLOAT4)
                           .attribute(VertexAttribute::UV0, 2, VertexBuffer::AttributeType::FLOAT2).build(*s.engine);
    vb->setBufferAt(*s.engine, 0, copied(m->pos.data(), n * sizeof(float3)));
    vb->setBufferAt(*s.engine, 1, copied(m->quat.data(), n * sizeof(quatf)));
    vb->setBufferAt(*s.engine, 2, copied(m->uv.data(), n * sizeof(float2)));
    IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)std::max<size_t>(1, m->idx.size())).bufferType(IndexBuffer::IndexType::UINT).build(*s.engine);
    if (!m->idx.empty()) ib->setBuffer(*s.engine, copied(m->idx.data(), m->idx.size() * sizeof(uint32_t)));
    m->vb = vb, m->ib = ib, m->vbRoom = (uint32_t)n, m->ibRoom = (uint32_t)std::max<size_t>(1, m->idx.size());
    std::vector<MaterialInstance*> looks;
    for (size_t k = 0; k < ends.size(); ++k) {
        MaterialInstance* mi = mat->createInstance();
        if (mat->hasParameter("opacity")) mi->setParameter("opacity", 1.f);   // (all there, until it is told otherwise)
        if (mat->hasParameter("matteUse")) {   // (a material that can be cut: by nothing, until `matted` says so)
            for (const char* name : {"matte0", "matte1", "matte2", "matte3"}) mi->setParameter(name, white_point(s), TextureSampler());
            for (const char* name : {"matteEye0", "matteEye1", "matteEye2", "matteEye3"}) mi->setParameter(name, mat4f{});
            if (mat->hasParameter("colours")) mi->setParameter("colours", white_point(s), TextureSampler());   // (a film's second plane: video.h)
            mi->setParameter("matteUse", float4{0.f});
            mi->setParameter("matteSoft", float4{0.f});
        }
        if (mat->hasParameter("backdrop")) mi->setParameter("backdrop", white_point(s), TextureSampler());   // (none read until doc.h says which)
        looks.push_back(mi);
    }
    float3 low = n ? m->pos[0] : float3{0.f}, high = low;   // (Filament wants a thing's rough size if it casts or takes shadows)
    for (const float3& p : m->pos) { low = min(low, p); high = max(high, p); }
    // (together: its places drawn in groups of as many as Filament draws in one go (64), each place an instance of one
    //  thing; a GPU pays for each draw, not for each triangle: a thousand copies are sixteen draws, not a thousand)
    //  One of two parts (a fill and its edge) is not: Filament draws a part for all its instances before the next part, so
    //  every edge would lie over every fill, where one copy's fill hid the edge of the one under it.)
    const size_t group = together && places.size() > 1 && ends.size() == 1 ? std::min<size_t>(places.size(), s.engine->getMaxAutomaticInstances()) : 1;
    for (const mat4f& place : places) {
        const size_t at = (size_t)(&place - places.data());
        if (at % group) continue;
        const size_t count = std::min(group, places.size() - at);
        InstanceBuffer* many = nullptr;
        if (group > 1) {
            many = InstanceBuffer::Builder(count).build(*s.engine);
            many->setLocalTransforms(places.data() + at, count);
        }
        utils::Entity e = utils::EntityManager::get().create();
        s.engine->getTransformManager().create(e, {}, many ? mat4f{} : place);
        RenderableManager::Builder b(ends.size());
        for (size_t k = 0; k < ends.size(); ++k) {
            const uint32_t from = k ? ends[k - 1] : 0;
            b.geometry(k, RenderableManager::PrimitiveType::TRIANGLES, vb, ib, from, ends[k] - from).material(k, looks[k]);
        }
        b.boundingBox({(low + high) * 0.5f, max((high - low) * 0.5f, float3{1e-3f})}).culling(false).castShadows(shadows).receiveShadows(shadows).layerMask(0xFF, layer);
        if (many) b.instances(count, many); else b.instances((size_t)copies);
        b.priority((uint8_t)order).build(*s.engine, e);
        s.scene->addEntity(e);
        s.made.push_back({e, m, (int)at, place, false, many, (int)count});
    }
    return looks;
}
static MaterialInstance* add(Stage& s, Mesh* m, const char* filamat, std::vector<mat4f> places = {mat4f{}}, uint8_t layer = 1,
                             bool shadows = false, int copies = 1, int order = 4, bool together = false) {
    const std::vector<MaterialInstance*> looks = add_parts(s, m, filamat, places, {}, layer, shadows, copies, order, together);
    return looks.empty() ? nullptr : looks[0];
}

// A mesh in the scene made anew (its points and triangles changed, as many or not): new buffers for every place it is
// at, its parts' ends as told; the old ones let go (Filament keeps them until no picture uses them).
// A mesh made again, put in the buffers it has while it fits them (as Rive and Live2D write theirs again, not make new
// ones), else in new ones a quarter larger than it needs (Rive's growth).
static void reshape(Stage& s, Mesh* m, const std::vector<uint32_t>& ends) {
    VertexBuffer* oldVb = nullptr;
    IndexBuffer* oldIb = nullptr;
    const size_t n = m->pos.size();
    if (!n || m->idx.empty()) return;
    m->quat.resize(n);
    if (std::all_of(m->nrm.begin(), m->nrm.end(), [&](const float3& v) { return v == m->nrm[0]; })) {   // (all facing one way, a flat shape: one frame for every point)
        auto* frames = geometry::SurfaceOrientation::Builder().vertexCount(1).normals(m->nrm.data()).build();
        frames->getQuats(m->quat.data(), 1);
        delete frames;
        std::fill(m->quat.begin() + 1, m->quat.end(), m->quat[0]);
    } else {
        auto* frames = geometry::SurfaceOrientation::Builder().vertexCount(n).normals(m->nrm.data()).build();
        frames->getQuats(m->quat.data(), n);
        delete frames;
    }
    if (!m->vb || n > m->vbRoom) {
        oldVb = m->vb, m->vbRoom = (uint32_t)(n + n / 4 + 16);
        m->vb = VertexBuffer::Builder().vertexCount(m->vbRoom).bufferCount(3)
                    .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3)
                    .attribute(VertexAttribute::TANGENTS, 1, VertexBuffer::AttributeType::FLOAT4)
                    .attribute(VertexAttribute::UV0, 2, VertexBuffer::AttributeType::FLOAT2).build(*s.engine);
    }
    if (!m->ib || m->idx.size() > m->ibRoom) {
        oldIb = m->ib, m->ibRoom = (uint32_t)(m->idx.size() + m->idx.size() / 4 + 48);
        m->ib = IndexBuffer::Builder().indexCount(m->ibRoom).bufferType(IndexBuffer::IndexType::UINT).build(*s.engine);
    }
    m->vb->setBufferAt(*s.engine, 0, copied(m->pos.data(), n * sizeof(float3)));
    m->vb->setBufferAt(*s.engine, 1, copied(m->quat.data(), n * sizeof(quatf)));
    m->vb->setBufferAt(*s.engine, 2, copied(m->uv.data(), n * sizeof(float2)));
    m->ib->setBuffer(*s.engine, copied(m->idx.data(), m->idx.size() * sizeof(uint32_t)));
    auto& faces = s.engine->getRenderableManager();
    for (const Stage::Made& made : s.made) {
        if (made.of != m) continue;
        const auto at = faces.getInstance(made.thing);
        for (size_t k = 0; k < ends.size() && k < faces.getPrimitiveCount(at); ++k) {
            const uint32_t from = k ? ends[k - 1] : 0;
            faces.setGeometryAt(at, k, RenderableManager::PrimitiveType::TRIANGLES, m->vb, m->ib, from, ends[k] - from);
        }
    }
    if (oldVb) s.engine->destroy(oldVb);
    if (oldIb) s.engine->destroy(oldIb);
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
    s.behindRead = true;
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
