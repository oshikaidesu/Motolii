// Clip-distance probe: can a Filament material give the GPU's own clipper up to 8 half-planes, with the runtime as
// shipped (prebuilt 1.77.2, Metal)? One triangle covers the picture; clip_probe.mat hands over 8 distances that make
// an octagon and tests nothing itself. The picture is compared, pixel by pixel, with the octagon worked out here.
// matc and the runtime are the prebuilt ones, unchanged.
// ./build_one.sh clip_probe && ./out/clip_probe [planes]
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
    const int W = 512, H = 512;
    const int planes = argc > 1 ? atoi(argv[1]) : 8;
    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    Renderer* renderer = engine->createRenderer();
    SwapChain* swap = engine->createSwapChain(16u, 16u, 0);
    auto& em = utils::EntityManager::get();
    Texture* t = Texture::Builder().width(W).height(H).levels(1).usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                     .format(Texture::InternalFormat::RGBA32F).build(*engine);
    RenderTarget* rt = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR0, t).build(*engine);
    static float dummy[9] = {};
    VertexBuffer* vb = VertexBuffer::Builder().vertexCount(3).bufferCount(1).attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12).build(*engine);
    vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(dummy, sizeof(dummy)));
    static uint32_t idx[3] = {0, 1, 2};
    IndexBuffer* ib = IndexBuffer::Builder().indexCount(3).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(idx, sizeof(idx)));
    std::ifstream f("clip_probe.filamat", std::ios::binary);
    std::vector<char> b((std::istreambuf_iterator<char>(f)), std::istreambuf_iterator<char>());
    Material* m = Material::Builder().package(b.data(), b.size()).build(*engine);
    MaterialInstance* mi = m->createInstance();
    mi->setParameter("planes", (float)planes);
    Scene* sc = engine->createScene();
    utils::Entity e = em.create();
    RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {8, 8, 8}}).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
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
    // the octagon, worked out here for every pixel's middle; a pixel within one pixel of an edge may go either way.
    // (The picture may come back upside down: both ways are tried.)
    long lit = 0, should = 0, nearEdge = 0, wrong[2] = {0, 0};
    for (int y = 0; y < H; ++y)
        for (int x = 0; x < W; ++x) {
            const double px = (x + 0.5) / W * 2.0 - 1.0, py = (y + 0.5) / H * 2.0 - 1.0;
            double least = 1e9;
            for (int k = 0; k < planes; ++k) {
                const double a = 6.2831853 * (k + 0.5) / 8.0;
                least = std::min(least, 0.6 - (std::cos(a) * px + std::sin(a) * py));
            }
            const bool in = least >= 0;
            lit += got[(size_t)(y * W + x) * 4 + 3] > 0.5f; should += in;
            if (std::fabs(least) < 2.0 / W) { ++nearEdge; continue; }
            for (int flip = 0; flip < 2; ++flip) wrong[flip] += in != (got[(size_t)((flip ? H - 1 - y : y) * W + x) * 4 + 3] > 0.5f);
        }
    const long bad = std::min(wrong[0], wrong[1]);
    std::printf("clip planes handed to the GPU: %d. Pixels lit %ld, the shape has %ld; wrong away from the edges: %ld; pixels within one pixel of an edge (not judged): %ld\n", planes, lit, should, bad, nearEdge);
    std::printf("%s\n", bad == 0 && lit > 0 && std::labs(lit - should) <= nearEdge ? "PASS" : "FAIL");
    std::fflush(stdout);
    _exit(0);
}
