// poster.mm: a band and a heap of blocks of clear glass in front of a black poster with white letters, seen straight on.
// Everything that draws is Filament as it is: screen-space refraction with dispersion, its small studio as the
// room the glass mirrors, a sun, tone mapping. One render, no switches.
// ./poster.sh  ->  out/poster.png
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
#include <filament/Viewport.h>

#include <geometry/SurfaceOrientation.h>
#include <image/Ktx1Bundle.h>
#include <ktxreader/Ktx1Reader.h>
#include <math/mat4.h>
#include <utils/EntityManager.h>

#import <Cocoa/Cocoa.h>

#include <cmath>
#include <cstdint>
#include <cstdio>
#include <fstream>
#include <vector>

using namespace filament;
using namespace filament::math;

static const int W = 1080;
static const int H = 1920;
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

// The band: a rounded slab swept round a loop that leaves the view on the right. It leans, twists a little and
// swells and thins in slow waves (whole counts along the loop, so it closes).
static float3 band_dir(float u) {   // the way the loop runs
    const float a = u * TAU;
    return normalize(float3{-2.f * std::sin(a), 1.84f * std::cos(a), 0.3f * std::cos(2.f * a + 0.6f)});
}
static float3 band_point(float u, float ang) {
    auto centre = [](float a) { return float3{1.5f + 2.f * std::cos(a), 1.4f + 1.84f * std::sin(a), 0.15f * std::sin(2.f * a + 0.6f)}; };
    const float a = u * TAU, wide = 0.42f, thick = 0.315f, round = 2.4f;
    float3 T = band_dir(u);
    float3 N = normalize(cross(float3{0, 0, 1}, T)), B = cross(T, N);
    const float twist = 0.5f + 0.2f * std::sin(4.f * a);
    float3 n = N * std::cos(twist) + B * std::sin(twist), b = cross(T, n);
    const float wave = 1.f + 0.09f * (0.55f * std::sin(2.f * ang + TAU * 9.f * u) + 0.35f * std::sin(3.f * ang - TAU * 14.f * u + 1.3f)
                                      + 0.25f * std::sin(ang + TAU * 23.f * u + 4.0f) + 0.5f * std::sin(TAU * 5.f * u + 0.7f));
    const float c = std::cos(ang), s = std::sin(ang);
    return centre(a) + n * (std::copysign(std::pow(std::fabs(c), 2.f / round), c) * thick * wave)
                     + b * (std::copysign(std::pow(std::fabs(s), 2.f / round), s) * wide * wave);
}

static Mesh* band() {
    const int rings = 384, sides = 64;
    Mesh* m = new Mesh;
    for (int i = 0; i < rings; ++i)
        for (int k = 0; k < sides; ++k) {
            m->pos.push_back(band_point((float)i / rings, (float)k / sides * TAU));
            m->uv.push_back({(float)i / rings, (float)k / sides});
        }
    auto at = [&](int i, int k) { return m->pos[(size_t)(((i + rings) % rings) * sides + (k + sides) % sides)]; };
    for (int i = 0; i < rings; ++i)
        for (int k = 0; k < sides; ++k) {
            // the waves along the band are left out of the normal: the lamps then mirror as long lines, not as dots
            m->nrm.push_back(normalize(cross(at(i, k + 1) - at(i, k - 1), band_dir((float)i / rings))));
            const uint32_t i1 = (uint32_t)((i + 1) % rings), k1 = (uint32_t)((k + 1) % sides);
            const uint32_t p = (uint32_t)(i * sides + k), q = (uint32_t)(i * sides) + k1, r = i1 * sides + (uint32_t)k, t = i1 * sides + k1;
            m->idx.insert(m->idx.end(), {p, q, r, q, t, r});
        }
    return m;
}

// One glass block, edge 1, as real ones are: the edges a little round (a tenth of half the edge), the faces flat.
// glassbox.mat draws it: the sight line goes in, through the block's real far faces, mirrors inside where it cannot
// leave, and shows what lies behind where it comes out.
static const float ROUND = 0.1f;
static Mesh* cube() {
    Mesh* m = new Mesh;
    std::vector<float> line;   // (where a face is cut: finely across the rounding, once across the flat)
    for (int k = 0; k <= 4; ++k) line.push_back(-1.f + ROUND * (float)k / 4.f);
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
// The heap, placed by hand: on the letters (over plain black a block shows little), clear of one another as seen
// (a block does not show another block through itself), one large with a corner towards the eye.
// {middle}, half the edge, {the axis it is turned about}, how far
static std::vector<mat4f> heap() {
    struct Placed { float3 c; float h; float3 axis; float turn; };
    const Placed all[] = {{{0.22f, 2.30f, 0.5f}, 0.15f, {0.3f, 1.f, 0.2f}, 0.7f},  {{0.62f, 1.70f, 0.6f}, 0.20f, {1.f, -0.6f, 0.3f}, 0.85f},
                          {{0.10f, 1.02f, 0.2f}, 0.10f, {0.5f, 1.f, -0.4f}, 1.1f}, {{0.60f, 0.66f, 0.7f}, 0.15f, {-0.4f, 1.f, 0.6f}, 0.6f},
                          {{0.84f, 1.21f, 0.3f}, 0.06f, {1.f, 0.3f, 0.5f}, 0.9f}};
    std::vector<mat4f> out;
    for (const Placed& p : all) out.push_back(mat4f::translation(p.c) * mat4f::rotation(p.turn, normalize(p.axis)) * mat4f::scaling(float3{2.f * p.h}));
    return out;
}

// White letters on black, drawn by the system's own text drawing.
static std::vector<uint8_t> poster_pixels() {
    std::vector<uint8_t> px((size_t)W * H * 4, 0);
    CGColorSpaceRef cs = CGColorSpaceCreateDeviceRGB();
    CGContextRef cg = CGBitmapContextCreate(px.data(), W, H, 8, W * 4, cs, kCGImageAlphaPremultipliedLast);
    CGContextSetRGBFillColor(cg, 0, 0, 0, 1);
    CGContextFillRect(cg, CGRectMake(0, 0, W, H));
    [NSGraphicsContext setCurrentContext:[NSGraphicsContext graphicsContextWithCGContext:cg flipped:NO]];
    auto text = [&](NSString* t, CGFloat size, CGFloat x, CGFloat top) {
        [t drawAtPoint:NSMakePoint(x, H - top - size * 1.2) withAttributes:@{NSFontAttributeName : [NSFont fontWithName:@"Helvetica-Bold" size:size], NSForegroundColorAttributeName : NSColor.whiteColor}];
    };
    auto dashes = [&](CGFloat top) {
        CGContextSetRGBFillColor(cg, 1, 1, 1, 1);
        for (int x = 90; x < W - 90; x += 22) CGContextFillRect(cg, CGRectMake(x, H - top, 12, 6));
    };
    text(@"MOTOLII", 26, 90, 80);
    text(@"2026", 40, 90, 110);
    text(@"FILAMENT", 26, 520, 80);
    text(@"1.77.2", 40, 520, 110);
    text(@"CLEAR", 268, 78, 300);
    text(@"GLASS", 268, 78, 560);
    dashes(940);
    text(@"MOVE", 268, 78, 990);
    text(@"MORE", 268, 78, 1250);
    dashes(1640);
    text(@"REFRACTION", 34, 90, 1670);
    text(@"DISPERSION", 34, 90, 1712);
    text(@"ONE PASS", 34, 620, 1670);
    text(@"AS IT IS", 34, 620, 1712);
    [NSGraphicsContext.currentContext flushGraphics];
    [NSGraphicsContext setCurrentContext:nil];
    CGContextRelease(cg);
    CGColorSpaceRelease(cs);
    return px;
}

int main() {
    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    Camera* camera = engine->createCamera(utils::EntityManager::get().create());
    const float3 eye{0.f, 1.4f, 6.f};
    const float fov = 30.f;
    camera->setExposure(16.0f, 1.0f / 125.0f, 100.0f);
    camera->lookAt(eye, {0.f, 1.4f, 0.f}, {0, 1, 0});
    camera->setProjection(fov, double(W) / double(H), 0.05, 60.0, Camera::Fov::VERTICAL);

    static FilmicToneMapper filmic;
    view->setScene(scene);
    view->setCamera(camera);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setColorGrading(ColorGrading::Builder().toneMapper(&filmic).saturation(0.75f).build(*engine));
    view->setDithering(View::Dithering::NONE);
    view->setAntiAliasing(View::AntiAliasing::FXAA);
    view->setScreenSpaceRefractionEnabled(true);
    Renderer::ClearOptions clear{};
    clear.clearColor = {0, 0, 0, 1};
    clear.clear = true;
    renderer->setClearOptions(clear);

    // the room: the small photo studio that comes with Filament (poster.sh makes the .ktx with cmgen)
    static std::vector<uint8_t> room_bytes = load_file("out/small_studio/small_studio_ibl.ktx");
    if (room_bytes.empty()) return fail("out/small_studio/small_studio_ibl.ktx");
    auto* bundle = new image::Ktx1Bundle(room_bytes.data(), (uint32_t)room_bytes.size());
    float3 sh[9];
    if (!bundle->getSphericalHarmonics(sh)) return fail("room: no light numbers in the .ktx");
    Texture* room = ktxreader::Ktx1Reader::createTexture(engine, bundle, false);
    scene->setIndirectLight(IndirectLight::Builder().reflections(room).irradiance(3, sh).intensity(60000.f).build(*engine));
    utils::Entity sun = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::SUN).color({1.0f, 0.98f, 0.95f}).intensity(57000.f).direction(normalize(float3{0.35f, -0.6f, -0.7f}))
        .castShadows(false).build(*engine, sun);
    scene->addEntity(sun);

    // layer 2: left out of the picture of what is behind (the glass blocks themselves)
    auto add = [&](Mesh* m, const char* filamat, std::vector<mat4f> places = {mat4f{}}, uint8_t layer = 1) -> MaterialInstance* {
        static std::vector<std::vector<uint8_t>> pkgs;
        pkgs.push_back(load_file(filamat));
        Material* mat = pkgs.back().empty() ? nullptr : Material::Builder().package(pkgs.back().data(), pkgs.back().size()).build(*engine);
        if (!mat) return nullptr;
        const size_t n = m->pos.size();
        auto* frames = geometry::SurfaceOrientation::Builder().vertexCount(n).normals(m->nrm.data()).build();
        m->quat.resize(n);
        frames->getQuats(m->quat.data(), n);
        delete frames;
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)n).bufferCount(3)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3)
                               .attribute(VertexAttribute::TANGENTS, 1, VertexBuffer::AttributeType::FLOAT4)
                               .attribute(VertexAttribute::UV0, 2, VertexBuffer::AttributeType::FLOAT2).build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(m->pos.data(), n * sizeof(float3)));
        vb->setBufferAt(*engine, 1, VertexBuffer::BufferDescriptor(m->quat.data(), n * sizeof(quatf)));
        vb->setBufferAt(*engine, 2, VertexBuffer::BufferDescriptor(m->uv.data(), n * sizeof(float2)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)m->idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(m->idx.data(), m->idx.size() * sizeof(uint32_t)));
        MaterialInstance* mi = mat->createInstance();
        for (const mat4f& place : places) {
            utils::Entity e = utils::EntityManager::get().create();
            engine->getTransformManager().create(e, {}, place);
            RenderableManager::Builder(1).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib).material(0, mi)
                .culling(false).castShadows(false).receiveShadows(false).layerMask(0xFF, layer).build(*engine, e);
            scene->addEntity(e);
        }
        return mi;
    };

    // the poster fills the view, 0.6 behind the band's middle (further back and the letters tear instead of bending)
    const float dist = 6.6f, hh = dist * std::tan(0.5f * fov * TAU / 360.f), hw = hh * W / H;
    Mesh* sheet = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) {
        sheet->pos.push_back({eye.x + hw * c.x, eye.y + hh * c.y, eye.z - dist});
        sheet->nrm.push_back({0, 0, 1});
        sheet->uv.push_back(c * 0.5f + 0.5f);
    }
    sheet->idx = {0, 1, 2, 0, 2, 3};
    MaterialInstance* sheet_mi = add(sheet, "poster.filamat");
    if (!sheet_mi) return fail("poster.filamat");
    static std::vector<uint8_t> px = poster_pixels();
    Texture* tex = Texture::Builder().width(W).height(H).levels(1).format(Texture::InternalFormat::SRGB8_A8).build(*engine);
    tex->setImage(*engine, 0, Texture::PixelBufferDescriptor(px.data(), px.size(), Texture::Format::RGBA, Texture::Type::UBYTE));
    sheet_mi->setParameter("color", tex, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
    MaterialInstance* band_mi = add(band(), "poster_glass.filamat");
    if (!band_mi) return fail("poster_glass.filamat");
    band_mi->setParameter("thickness", 0.6f);
    band_mi->setParameter("lean", 0.f);

    // what is behind the blocks: the same scene without them, drawn first into a picture of its own
    Texture* behind = Texture::Builder().width(W).height(H).levels(1).usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE)
                          .format(Texture::InternalFormat::RGBA16F).build(*engine);
    Texture* behind_depth = Texture::Builder().width(W).height(H).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT).format(Texture::InternalFormat::DEPTH24).build(*engine);
    View* view_behind = engine->createView();
    view_behind->setScene(scene);
    view_behind->setCamera(camera);
    view_behind->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view_behind->setRenderTarget(RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, behind).texture(RenderTarget::AttachmentPoint::DEPTH, behind_depth).build(*engine));
    view_behind->setPostProcessingEnabled(false);
    view_behind->setScreenSpaceRefractionEnabled(true);
    view->setVisibleLayers(0x03, 0x03);
    MaterialInstance* blocks_mi = add(cube(), "glassbox.filamat", heap(), 2);
    if (!blocks_mi) return fail("glassbox.filamat");
    blocks_mi->setParameter("backdrop", behind, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE));
    blocks_mi->setParameter("ior", 1.5f);
    blocks_mi->setParameter("abbe", 2.5f);       // (the colours split wide: below 2 the eight steps show as stripes)
    blocks_mi->setParameter("reach", 0.5f);
    blocks_mi->setParameter("envGain", 0.05f);   // (glassbox's own studio, nearly off: at 1 the blocks go milky, at 0 they sink into black)
    blocks_mi->setParameter("sceneMix", 1.f);
    blocks_mi->setParameter("absorption", float3{0.10f, 0.04f, 0.06f});
    blocks_mi->setParameter("round", ROUND);
    blocks_mi->setParameter("lean", 0.08f);      // (the letters inside run like water; at 0.15 they break up)

    static std::vector<uint8_t> pic((size_t)W * H * 4);
    for (int i = 0; i < 3; ++i) {
        while (!renderer->beginFrame(swap)) {}
        renderer->render(view_behind);
        renderer->render(view);
        if (i == 2)
            renderer->readPixels(0, 0, (uint32_t)W, (uint32_t)H,
                                 backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
        renderer->endFrame();
        engine->flushAndWait();
    }
    uint8_t* plane = pic.data();
    NSBitmapImageRep* rep = [[NSBitmapImageRep alloc] initWithBitmapDataPlanes:&plane pixelsWide:W pixelsHigh:H bitsPerSample:8 samplesPerPixel:4
                                                                        hasAlpha:YES isPlanar:NO colorSpaceName:NSDeviceRGBColorSpace bytesPerRow:W * 4 bitsPerPixel:32];
    if (![[rep representationUsingType:NSBitmapImageFileTypePNG properties:@{}] writeToFile:@"out/poster.png" atomically:YES]) return fail("png");
    std::printf("wrote out/poster.png\n");
    return 0;
}
