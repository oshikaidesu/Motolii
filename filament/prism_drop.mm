// Drop-in glass box demo. Everything per frame stays on the GPU:
//   caustic_tri.h      sun -> glass -> floor light (rainbow), drawn into a texture
//   backdrop view      the scene without the glass box, so the box can show what is behind it
//   mirror view        the scene seen by the camera mirrored in the floor (planar reflection)
//   final view         floor + glass box + tube, Filament's own tone mapping / bloom / shadows
// Nothing is traced on the CPU and nothing is read back, except one readPixels at the very end to save the PNG.
#include <filament/ColorGrading.h>
#include <filament/LightManager.h>
#include <filament/Skybox.h>
#include <filament/SwapChain.h>
#include <filament/ToneMapper.h>
#include <filament/TransformManager.h>

#include <math/mat3.h>
#include <math/mat4.h>
#include <math/quat.h>

#include <zlib.h>

#include <chrono>
#include <cstdlib>

#include "caustic_tri.h"

using namespace ctri;

static const int W = 1920, H = 1080;
static float env_f(const char* k, float d) {
    const char* v = std::getenv(k);
    return v ? (float)std::atof(v) : d;
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

// A unit cube (+-0.5), flat faces, counter-clockwise from outside.
static Mesh unit_cube() {
    Mesh m;
    for (int k = 0; k < 3; ++k)
        for (float s : {-1.f, 1.f}) {
            int i = (k + 1) % 3, j = (k + 2) % 3;
            float3 n{0.f};
            n[k] = s;
            const float uu[4] = {-1, 1, 1, -1}, vv[4] = {-1, -1, 1, 1};
            uint32_t base = (uint32_t)m.pos.size();
            for (int q = 0; q < 4; ++q) {
                float3 p{0.f};
                p[k] = s * 0.5f;
                p[i] = uu[q] * 0.5f;
                p[j] = vv[q] * 0.5f;
                m.pos.push_back(p);
                m.nrm.push_back(n);
            }
            for (uint32_t v : {0u, 1u, 2u, 0u, 2u, 3u}) m.idx.push_back(base + v);
        }
    orient_outward(m);
    return m;
}
static Mesh transformed(const Mesh& in, const mat4f& xf) {
    Mesh m = in;
    mat3f r = xf.upperLeft();
    for (size_t i = 0; i < m.pos.size(); ++i) {
        m.pos[i] = (xf * float4{in.pos[i], 1.f}).xyz;
        m.nrm[i] = normalize(r * in.nrm[i]);
    }
    return m;
}

struct Buffers {
    VertexBuffer* vb;
    IndexBuffer* ib;
};
// position + tangent frame (what Filament's lit materials want)
static Buffers lit_buffers(Engine* e, const Mesh& m) {
    size_t nv = m.pos.size();
    float* d = new float[nv * 7];
    for (size_t i = 0; i < nv; ++i) {
        float3 N = m.nrm[i];
        float3 helper = std::fabs(N.y) < 0.9f ? float3{0, 1, 0} : float3{1, 0, 0};
        float3 T = normalize(cross(helper, N));
        float3 B = cross(N, T);
        quatf q = mat3f::packTangentFrame(mat3f{T, B, N});
        float row[7] = {m.pos[i].x, m.pos[i].y, m.pos[i].z, q.x, q.y, q.z, q.w};
        std::memcpy(d + i * 7, row, sizeof(row));
    }
    VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)nv).bufferCount(1)
                           .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                           .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28).build(*e);
    vb->setBufferAt(*e, 0, VertexBuffer::BufferDescriptor(d, nv * 28, [](void* p, size_t, void*) { delete[] (float*)p; }));
    IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)m.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*e);
    uint32_t* id = new uint32_t[m.idx.size()];
    std::memcpy(id, m.idx.data(), m.idx.size() * 4);
    ib->setBuffer(*e, IndexBuffer::BufferDescriptor(id, m.idx.size() * 4, [](void* p, size_t, void*) { delete[] (uint32_t*)p; }));
    return {vb, ib};
}

int main() {
    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    Scene* scene = engine->createScene();
    auto& em = utils::EntityManager::get();
    auto& tm = engine->getTransformManager();

    Materials cm;
    if (!load_materials(engine, "caustic_tri", cm)) return fail("caustic materials (run caustic_tri/build.sh)");
    Material* glassMat = load_material(engine, "glassbox.filamat");
    Material* floorMat = load_material(engine, "stage_floor.filamat");
    Material* tubeMat = load_material(engine, "glass.filamat");
    Material* panelMat = load_material(engine, "emissive.filamat");
    if (!glassMat || !floorMat || !tubeMat || !panelMat) return fail("materials");

    // ---- the scene, in world units
    const float3 sun = normalize(float3{0.92f, -0.42f, 0.22f});
    const float lux = 22000.f;
    const float floorSize = 3.1f;
    const float half = 0.16f;
    const mat4f boxXf = mat4f::translation(float3{0.f, half + 0.001f, 0.f}) * mat4f::rotation(0.62f, float3{0, 1, 0}) *
                        mat4f::scaling(float3{2.f * half});
    const Mesh cube = unit_cube();

    const float src[6][3] = {{-0.78f, -0.22f, 0.12f}, {-0.22f, -0.48f, 0.38f}, {0.55f, -0.12f, 0.22f},
                             {0.42f, 0.46f, -0.22f},  {-0.18f, 0.38f, -0.48f}, {-0.68f, 0.08f, -0.12f}};
    std::vector<float3> ring;
    for (auto& q : src) ring.push_back({(q[0] + 0.095f) * 0.44f, 0.13f + q[2] * 0.16f, q[1] * 0.64f});
    const Mesh tubeMesh = tube(catmull_closed(ring, 24), 64, 0.02f);

    // ---- sun -> glass -> floor light, all on the GPU
    Params cp;
    cp.ior = 1.52f;
    cp.dispersion = env_f("LIGHT_DISPERSION", 0.04f);
    cp.intensity = 1.f;
    cp.footprint = false;
    cp.rsize = floorSize;
    cp.cap = (int)env_f("LIGHT_CAPTURE", 256);
    cp.map = (int)env_f("LIGHT_MAP", 1024);
    CausticPass light(engine, cm, {transformed(cube, boxXf), tubeMesh}, sun, cp);

    // ---- targets
    RT backdrop = make_rt(engine, W, H, Texture::InternalFormat::RGBA16F, true);
    RT mirror = make_rt(engine, W / 2, H / 2, Texture::InternalFormat::RGBA16F, true);
    RT final8 = make_rt(engine, W, H, Texture::InternalFormat::RGBA8, true);
    TextureSampler lin(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE);

    // ---- floor
    MaterialInstance* floorMi = floorMat->createInstance();
    floorMi->setParameter("color", float4{0.86f, 0.85f, 0.84f, 1});
    floorMi->setParameter("roughness", 0.42f);
    floorMi->setParameter("lightScale", lux * std::fabs(sun.y) * env_f("LIGHT_GAIN", 1.f));
    floorMi->setParameter("lightBlur", env_f("LIGHT_BLUR", 1.6f) / (float)cp.map);
    floorMi->setParameter("lightSize", floorSize);
    floorMi->setParameter("mirrorStrength", env_f("MIRROR", 0.f));
    floorMi->setParameter("light", light.caustic(), lin);
    floorMi->setParameter("mirror", mirror.color, lin);
    {
        Mesh f;
        const float hs = floorSize * 0.5f;
        f.pos = {{-hs, 0, -hs}, {hs, 0, -hs}, {hs, 0, hs}, {-hs, 0, hs}};
        f.nrm.assign(4, float3{0, 1, 0});
        f.idx = {0, 2, 1, 0, 3, 2};
        Buffers b = lit_buffers(engine, f);
        utils::Entity ent = em.create();
        RenderableManager::Builder(1).boundingBox({{-4, -1, -4}, {4, 1, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, b.vb, b.ib)
            .material(0, floorMi).culling(false).castShadows(false).receiveShadows(true)
            .layerMask(0xFF, 0x04)  // layer 2: hidden in the mirror view
            .build(*engine, ent);
        scene->addEntity(ent);
    }
    // ---- a bright wall panel behind, as in the original prism picture
    {
        Mesh p;
        const float3 c{0.f, 0.55f, -1.15f};
        p.pos = {c + float3{-1.6f, -0.7f, 0}, c + float3{1.6f, -0.7f, 0}, c + float3{1.6f, 0.7f, 0}, c + float3{-1.6f, 0.7f, 0}};
        p.nrm.assign(4, float3{0, 0, 1});
        p.idx = {0, 1, 2, 0, 2, 3};
        Buffers b = lit_buffers(engine, p);
        MaterialInstance* mi = panelMat->createInstance();
        mi->setParameter("color", float4{2200.f, 2300.f, 2600.f, 1});
        utils::Entity ent = em.create();
        RenderableManager::Builder(1).boundingBox({{-4, -4, -4}, {4, 4, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, b.vb, b.ib)
            .material(0, mi).culling(false).castShadows(false).receiveShadows(false).build(*engine, ent);
        scene->addEntity(ent);
    }
    // ---- the glass tube (Filament's own glass)
    {
        Buffers b = lit_buffers(engine, tubeMesh);
        MaterialInstance* mi = tubeMat->createInstance();
        mi->setParameter("ior", 1.55f);
        mi->setParameter("transmission", 1.0f);
        mi->setParameter("thickness", 0.10f);
        mi->setParameter("dispersion", 0.6f);
        utils::Entity ent = em.create();
        RenderableManager::Builder(1).boundingBox({{-4, -4, -4}, {4, 4, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, b.vb, b.ib)
            .material(0, mi).culling(false).castShadows(true).receiveShadows(false).build(*engine, ent);
        scene->addEntity(ent);
    }
    // ---- the drop-in glass box: a unit cube + a transform, nothing else
    MaterialInstance* glassMi = glassMat->createInstance();
    glassMi->setParameter("backdrop", backdrop.color, lin);
    glassMi->setParameter("ior", env_f("GLASS_IOR", 1.52f));
    glassMi->setParameter("abbe", env_f("GLASS_ABBE", 22.f));
    glassMi->setParameter("reach", env_f("GLASS_REACH", 0.5f));
    glassMi->setParameter("envGain", env_f("GLASS_ENV", 1.f));
    glassMi->setParameter("sceneMix", env_f("GLASS_SCENE", 1.f));
    glassMi->setParameter("absorption", float3{0.10f, 0.04f, 0.06f});
    {
        Buffers b = lit_buffers(engine, cube);
        utils::Entity ent = em.create();
        RenderableManager::Builder(1).boundingBox({{-0.5f, -0.5f, -0.5f}, {0.5f, 0.5f, 0.5f}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, b.vb, b.ib)
            .material(0, glassMi).culling(false).castShadows(true).receiveShadows(false)
            .layerMask(0xFF, 0x02)  // layer 1: hidden in the backdrop view
            .build(*engine, ent);
        tm.create(ent);
        tm.setTransform(tm.getInstance(ent), boxXf);
        scene->addEntity(ent);
    }

    // ---- sun
    utils::Entity sunE = em.create();
    LightManager::ShadowOptions shopt;
    shopt.mapSize = 2048;
    shopt.vsm.blurWidth = 8.f;
    shopt.vsm.elvsm = true;
    LightManager::Builder(LightManager::Type::SUN).color(float3{1.f, 0.98f, 0.96f}).intensity(lux).direction(sun)
        .sunAngularRadius(0.35f).sunHaloSize(1.2f).castShadows(true).shadowOptions(shopt).build(*engine, sunE);
    scene->addEntity(sunE);
    scene->setSkybox(Skybox::Builder().color({0.72f, 0.74f, 0.78f, 1}).build(*engine));

    // ---- cameras and views
    const float3 eye{0.55f, 0.78f, 0.70f}, target{0.f, 0.04f, 0.f};
    Camera* camera = engine->createCamera(em.create());
    camera->setExposure(2.8f, 1.0f / 125.0f, 100.0f);
    camera->lookAt(eye, target, {0, 1, 0});
    camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);

    auto scene_view = [&](RenderTarget* rt, int w, int h, uint8_t layers, bool post) {
        View* v = engine->createView();
        v->setScene(scene);
        v->setCamera(camera);
        v->setRenderTarget(rt);
        v->setViewport({0, 0, (uint32_t)w, (uint32_t)h});
        v->setVisibleLayers(0xFF, layers);
        v->setDithering(View::Dithering::NONE);
        v->setPostProcessingEnabled(post);
        v->setShadowType(View::ShadowType::VSM);
        VsmShadowOptions vsm;
        vsm.highPrecision = true;
        v->setVsmShadowOptions(vsm);
        return v;
    };
    View* vBackdrop = scene_view(backdrop.rt, W, H, 0x01 | 0x04, false);          // no glass box
    View* vFinal = scene_view(final8.rt, W, H, 0x01 | 0x02 | 0x04, true);
    static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
    vFinal->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.04f).saturation(1.08f).build(*engine));
    vFinal->setAntiAliasing(View::AntiAliasing::FXAA);
    vFinal->setScreenSpaceRefractionEnabled(true);
    vBackdrop->setScreenSpaceRefractionEnabled(true);
    BloomOptions bloom{};
    bloom.enabled = true;
    bloom.strength = 0.18f;
    bloom.levels = 6;
    bloom.resolution = 1080;
    bloom.threshold = true;
    bloom.highlight = 10.f;
    bloom.blendMode = BloomOptions::BlendMode::ADD;
    vFinal->setBloomOptions(bloom);

    auto open_frame = [&]() {
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) return true;
        return false;
    };
    const bool skipLight = env_f("SKIP_LIGHT", 0) > 0.5f, skipBackdrop = env_f("SKIP_BACKDROP", 0) > 0.5f,
               skipFinal = env_f("SKIP_FINAL", 0) > 0.5f;
    auto draw = [&]() {
        if (!skipLight) light.draw(renderer);
        renderer->setClearOptions({});
        if (!skipBackdrop) renderer->render(vBackdrop);
        if (!skipFinal) renderer->render(vFinal);
    };

    // ---- timing: whole frame, no readback
    std::vector<double> t;
    const int frames = (int)env_f("FRAMES", 60);
    for (int i = 0; i < frames; ++i) {
        auto a = std::chrono::steady_clock::now();
        if (!open_frame()) return fail("beginFrame");
        draw();
        renderer->endFrame();
        engine->flushAndWait();
        t.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - a).count());
    }
    std::sort(t.begin(), t.end());
    if (!t.empty())
        std::printf("TIME whole frame at %dx%d (light + backdrop + final, no readback): median %.2f ms, worst %.2f ms over %d frames (60fps = 16.67 ms)\n",
                    W, H, t[t.size() / 2], t.back(), (int)t.size());

    // ---- one picture (the only readback in this program)
    std::vector<uint8_t> pic((size_t)W * H * 4);
    if (!open_frame()) return fail("beginFrame");
    draw();
    renderer->readPixels(final8.rt, 0, 0, W, H,
                         backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
    renderer->endFrame();
    engine->flushAndWait();
    std::system("mkdir -p out");
    if (!write_png("out/prism-drop.png", W, H, pic.data())) return fail("png");
    std::printf("wrote out/prism-drop.png\n");
    return 0;
}
