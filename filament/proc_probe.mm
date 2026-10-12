// Procedural-vertex probe: can a renderable be drawn whose vertices carry NO data, their places being worked out in
// the vertex stage from getVertexIndex() and getInstanceIndex() alone? (Filament 1.77.2 as shipped.)
// ./build_one.sh proc_probe && ./out/proc_probe [buffers]     buffers: 0 (default) or 1 (a dummy attribute)
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
#include <filament/VertexBuffer.h>
#include <filament/View.h>
#include <filament/Viewport.h>
#include <math/vec4.h>
#include <utils/EntityManager.h>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <unistd.h>
#include <vector>
using namespace filament;
int main(int argc, char** argv) {
    const int W = 64, H = 64, CELLS = 4, INST = CELLS * CELLS;
    const int buffers = argc > 1 ? atoi(argv[1]) : 0;
    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    Renderer* renderer = engine->createRenderer();
    SwapChain* swap = engine->createSwapChain(16u, 16u, 0);
    auto& em = utils::EntityManager::get();
    Texture* t = Texture::Builder().width(W).height(H).levels(1).usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                     .format(Texture::InternalFormat::RGBA32F).build(*engine);
    RenderTarget* rt = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR0, t).build(*engine);
    VertexBuffer::Builder vbb;
    vbb.vertexCount(3).bufferCount((uint8_t)buffers);
    static float dummy[9] = {};
    if (buffers > 0) vbb.attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12);
    VertexBuffer* vb = vbb.build(*engine);
    if (buffers > 0) vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(dummy, sizeof(dummy)));
    static uint32_t idx[3] = {0, 1, 2};
    IndexBuffer* ib = IndexBuffer::Builder().indexCount(3).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(idx, sizeof(idx)));
    std::ifstream f("proc_probe.filamat", std::ios::binary);
    std::vector<char> b((std::istreambuf_iterator<char>(f)), std::istreambuf_iterator<char>());
    Material* m = Material::Builder().package(b.data(), b.size()).build(*engine);
    MaterialInstance* mi = m->createInstance();
    static math::float4 table[64];
    for (int i = 0; i < 64; ++i) table[i] = {1.f, 0, 0, 0};
    mi->setParameter("table", table, 64);
    mi->setParameter("cells", (float)CELLS);
    Scene* sc = engine->createScene();
    utils::Entity e = em.create();
    RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {8, 8, 8}}).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib).instances(INST)
        .material(0, mi).culling(false).castShadows(false).receiveShadows(false).build(*engine, e);
    sc->addEntity(e);
    View* v = engine->createView();
    v->setScene(sc);
    Camera* cam = engine->createCamera(em.create());
    cam->setProjection(Camera::Projection::ORTHO, -1.0, 1.0, -1.0, 1.0, 0.1, 10.0);
    cam->lookAt({0, 0, 1}, {0, 0, 0}, {0, 1, 0});
    v->setCamera(cam);
    renderer->setClearOptions({.clearColor = {0, 0, 0, 0}, .clear = true});
    v->setRenderTarget(rt);
    v->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    v->setPostProcessingEnabled(false);
    renderer->renderStandaloneView(v);
    std::vector<float> got((size_t)W * H * 4, -1.f);
    renderer->beginFrame(swap);
    renderer->readPixels(rt, 0, 0, W, H, backend::PixelBufferDescriptor(got.data(), got.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
    renderer->endFrame();
    engine->flushAndWait();
    // the triangle of instance i covers, in its cell, the points with 0.1 < x, 0.1 < y, x + y < 0.9 (roughly): look
    // at one point well inside it and one well outside
    int good = 0, flippedGood = 0;
    for (int i = 0; i < INST; ++i) {
        const int cx = i % CELLS, cy = i / CELLS, cw = W / CELLS;
        auto at = [&](float u, float w, bool flip) {
            const int x = (int)((cx + u) * cw), y0 = (int)((cy + w) * cw), y = flip ? H - 1 - y0 : y0;
            return got.data() + (size_t)(y * W + x) * 4;
        };
        for (int flip = 0; flip < 2; ++flip) {
            const float* in = at(0.25f, 0.25f, flip);
            const float* out = at(0.85f, 0.85f, flip);
            const bool ok = std::fabs(in[2] - i) < 1e-4f && std::fabs(in[0] - (i + 1)) < 1e-4f && std::fabs(in[3] - 1.f) < 1e-4f && out[3] < 0.5f;
            (flip ? flippedGood : good) += ok;
        }
    }
    const float* p = got.data() + (size_t)((int)(1.25f * 16) * W + (int)(2.25f * 16)) * 4;
    std::printf("vertex buffers: %d. A point inside the triangle of instance 6: sum = %g, vertex = %g, instance = %g, alpha = %g\n", buffers, p[0], p[1], p[2], p[3]);
    std::printf("instances whose triangle is where it should be: %d of %d (with the picture upside down: %d)\n", good, INST, flippedGood);
    std::printf("%s\n", (good == INST || flippedGood == INST) ? "PASS" : "CHECK");
    std::fflush(stdout);
    _exit(0);
}
