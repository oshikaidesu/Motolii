// MRT probe: can an application, with Filament v1.77.2 as shipped, have ONE fragment write several float4 values
// (several colour attachments of one RenderTarget), and read them all in the next pass?
// A post-process material may declare up to 8 colour outputs (filamat MaterialBuilder.cpp: surface materials "only
// support a maximum of 1 custom output"). Here one is put on a full-screen quad in an ordinary Scene / View whose
// RenderTarget has three RGBA32F attachments, drawn with Renderer::renderStandaloneView ("a poor man's compute API",
// Renderer.h), and the three textures are read back (to check only) and read by a second pass.
// ./build_one.sh mrt_probe && ./out/mrt_probe
#include <filament/Camera.h>
#include <filament/Engine.h>
#include <filament/IndexBuffer.h>
#include <filament/Material.h>
#include <filament/MaterialInstance.h>
#include <filament/RenderableManager.h>
#include <filament/Renderer.h>
#include <filament/RenderTarget.h>
#include <filament/Scene.h>
#include <filament/SwapChain.h>
#include <filament/Texture.h>
#include <filament/TextureSampler.h>
#include <filament/VertexBuffer.h>
#include <filament/View.h>
#include <filament/Viewport.h>
#include <math/vec4.h>
#include <utils/EntityManager.h>

#include <cmath>
#include <unistd.h>
#include <cstdio>
#include <fstream>
#include <vector>

using namespace filament;

static std::vector<char> load_file(const char* path) {
    std::ifstream f(path, std::ios::binary);
    return std::vector<char>((std::istreambuf_iterator<char>(f)), std::istreambuf_iterator<char>());
}

int main() {
    const int W = 8, H = 4;
    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return 1;
    Renderer* renderer = engine->createRenderer();
    SwapChain* swap = engine->createSwapChain(16u, 16u, 0);
    auto& em = utils::EntityManager::get();

    auto tex = [&]() {
        return Texture::Builder().width(W).height(H).levels(1).usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
            .format(Texture::InternalFormat::RGBA32F).build(*engine);
    };
    Texture* t[3] = {tex(), tex(), tex()};
    Texture* joined = tex();
    RenderTarget* three = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR0, t[0]).texture(RenderTarget::AttachmentPoint::COLOR1, t[1])
                              .texture(RenderTarget::AttachmentPoint::COLOR2, t[2]).build(*engine);
    RenderTarget* each[3];
    for (int i = 0; i < 3; ++i) each[i] = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR0, t[i]).build(*engine);
    RenderTarget* one = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR0, joined).build(*engine);
    std::printf("colour attachments a render target may have here: %d\n", (int)three->getSupportedColorAttachmentsCount());

    static float sq[] = {-1, -1, 0, 0, 0, 1, -1, 0, 1, 0, 1, 1, 0, 1, 1, -1, 1, 0, 0, 1};
    static uint32_t sqi[] = {0, 1, 2, 0, 2, 3};
    VertexBuffer* vb = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                           .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                           .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*engine);
    vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(sq, sizeof(sq)));
    IndexBuffer* ib = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(sqi, sizeof(sqi)));

    auto mat = [&](const char* path) -> Material* {
        auto b = load_file(path);
        return b.empty() ? nullptr : Material::Builder().package(b.data(), b.size()).build(*engine);
    };
    Material* writeMat = mat("mrt_probe.filamat");
    Material* readMat = mat("mrt_probe_read.filamat");
    if (!writeMat || !readMat) { std::printf("materials missing\n"); return 1; }
    MaterialInstance* writeMi = writeMat->createInstance();
    writeMi->setParameter("seed", 7.f);
    static const math::float4 table[4] = {{0, 0, 0, 0}, {0, 0, 0, 0}, {0, 1000, 0, 0}, {0, 0, 0, 0}};
    writeMi->setParameter("table", table, 4);
    static float src[8] = {0, 0, 0, 0, 0, 0, 20000, 0};   // texel (1, 0): z = 20000
    Texture* source = Texture::Builder().width(2).height(1).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
    source->setImage(*engine, 0, Texture::PixelBufferDescriptor(src, sizeof(src), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
    writeMi->setParameter("source", source, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST));
    MaterialInstance* readMi = readMat->createInstance();
    const TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    readMi->setParameter("first", t[0], nearest);
    readMi->setParameter("second", t[1], nearest);
    readMi->setParameter("third", t[2], nearest);

    Camera* camera = engine->createCamera(em.create());
    auto pass = [&](MaterialInstance* mi, RenderTarget* rt) {
        Scene* sc = engine->createScene();
        utils::Entity e = em.create();
        RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {8, 8, 8}}).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, mi).culling(false).castShadows(false).receiveShadows(false).build(*engine, e);
        sc->addEntity(e);
        View* v = engine->createView();
        v->setScene(sc);
        v->setCamera(camera);
        v->setRenderTarget(rt);
        v->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
        v->setPostProcessingEnabled(false);
        v->setShadowingEnabled(false);
        v->setDithering(View::Dithering::NONE);
        return v;
    };
    View* writeView = pass(writeMi, three);
    View* readView = pass(readMi, one);

    // outside any frame: "a poor man's compute API"
    // (MaterialInstance.h: "PostProcess and compute domain material instance must be commited manually")
    writeMi->commit(*engine);
    renderer->renderStandaloneView(writeView);
    renderer->renderStandaloneView(readView);

    std::vector<float> got[4];
    for (auto& g : got) g.assign((size_t)W * H * 4, -1.f);
    if (!renderer->beginFrame(swap)) { std::printf("no frame\n"); return 1; }
    for (int i = 0; i < 3; ++i)
        renderer->readPixels(each[i], 0, 0, W, H, backend::PixelBufferDescriptor(got[i].data(), got[i].size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
    renderer->readPixels(one, 0, 0, W, H, backend::PixelBufferDescriptor(got[3].data(), got[3].size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
    renderer->endFrame();
    engine->flushAndWait();

    int wrong = 0;
    for (int y = 0; y < H; ++y)
        for (int x = 0; x < W; ++x) {
            const float once = x * 10.f + y + 7.f + 1000.f + 20000.f;
            const float want[4][4] = {{once, 1, 2, 3}, {once + 0.25f, 10, 20, 30}, {once + 0.5f, 100, 200, 300}, {once, once + 0.25f, once + 0.5f, 300}};
            for (int i = 0; i < 4; ++i)
                for (int c = 0; c < 4; ++c)
                    if (std::fabs(got[i][(size_t)(y * W + x) * 4 + c] - want[i][c]) > 1e-3f) ++wrong;
        }
    const float* p = got[0].data() + (2 * W + 3) * 4;
    std::printf("texel (3, 2): first = %g %g %g %g", p[0], p[1], p[2], p[3]);
    p = got[1].data() + (2 * W + 3) * 4;
    std::printf("   second = %g %g %g %g", p[0], p[1], p[2], p[3]);
    p = got[2].data() + (2 * W + 3) * 4;
    std::printf("   third = %g %g %g %g\n", p[0], p[1], p[2], p[3]);
    p = got[3].data() + (2 * W + 3) * 4;
    std::printf("             read by the second pass = %g %g %g %g\n", p[0], p[1], p[2], p[3]);
    std::printf("%d of %d numbers wrong (rows may come out upside down: see below)\n", wrong, W * H * 16);
    if (wrong) {
        int flipped = 0;
        for (int y = 0; y < H; ++y)
            for (int x = 0; x < W; ++x)
                if (std::fabs(got[0][(size_t)((H - 1 - y) * W + x) * 4] - (x * 10.f + y + 7.f + 21000.f)) > 1e-6f) ++flipped;
        std::printf("with the rows turned over: %d of %d first numbers wrong\n", flipped, W * H);
    }
    std::printf("%s\n", wrong == 0 ? "PASS" : "CHECK");
    std::fflush(stdout);
    _exit(0);
}
