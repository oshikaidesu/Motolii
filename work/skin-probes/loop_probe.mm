// Throwaway probe 2. Can a CVPixelBuffer (what the video decoder hands out, IOSurface-backed) become a Filament texture
// with setExternalImage and be drawn into an IOSurface render target, with no CPU copy of the pixels?
#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#import <IOSurface/IOSurface.h>
#import <CoreVideo/CoreVideo.h>
#include <filament/Engine.h>
#include <filament/Renderer.h>
#include <filament/Scene.h>
#include <filament/View.h>
#include <filament/Camera.h>
#include <filament/RenderTarget.h>
#include <filament/Texture.h>
#include <filament/TextureSampler.h>
#include <filament/SwapChain.h>
#include <filament/Skybox.h>
#include <filament/Viewport.h>
#include <filament/Material.h>
#include <filament/MaterialInstance.h>
#include <filament/VertexBuffer.h>
#include <filament/IndexBuffer.h>
#include <filament/RenderableManager.h>
#include <backend/platforms/PlatformMetal.h>
#include <utils/EntityManager.h>
#import <AVFoundation/AVFoundation.h>
#include <fstream>
#include <chrono>
#include <algorithm>
#include <vector>
#include <cstdio>
using namespace filament;

int main() {
    const uint32_t W = 1920, H = 1080;
    Engine* engine = Engine::create(Engine::Backend::METAL);
    id<MTLDevice> dev = MTLCreateSystemDefaultDevice();

    // render target: IOSurface-backed MTLTexture we own
    NSDictionary* props = @{ (id)kIOSurfaceWidth: @(W), (id)kIOSurfaceHeight: @(H), (id)kIOSurfaceBytesPerElement: @4, (id)kIOSurfacePixelFormat: @((uint32_t)'BGRA') };
    IOSurfaceRef outSurf = IOSurfaceCreate((CFDictionaryRef)props);
    MTLTextureDescriptor* d = [MTLTextureDescriptor texture2DDescriptorWithPixelFormat:MTLPixelFormatBGRA8Unorm width:W height:H mipmapped:NO];
    d.usage = MTLTextureUsageRenderTarget | MTLTextureUsageShaderRead;
    id<MTLTexture> outTex = [dev newTextureWithDescriptor:d iosurface:outSurf plane:0];
    Texture* color = Texture::Builder().width(W).height(H).levels(1).format(Texture::InternalFormat::RGBA8)
        .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE).import((intptr_t)CFBridgingRetain(outTex)).build(*engine);
    Texture* depth = Texture::Builder().width(W).height(H).levels(1).format(Texture::InternalFormat::DEPTH24).usage(Texture::Usage::DEPTH_ATTACHMENT).build(*engine);
    RenderTarget* rt = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, color).texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);

    auto* platform = static_cast<backend::PlatformMetal*>(engine->getPlatform());
    Texture* videoTex = Texture::Builder().sampler(Texture::Sampler::SAMPLER_EXTERNAL).format(Texture::InternalFormat::RGB8).external().build(*engine);
    NSURL* url = [NSURL fileURLWithPath:[NSString stringWithUTF8String:getenv("CLIP")]];
    AVPlayerItem* item = [AVPlayerItem playerItemWithURL:url];
    NSDictionary* oa = @{ (id)kCVPixelBufferPixelFormatTypeKey: @(kCVPixelFormatType_32BGRA), (id)kCVPixelBufferMetalCompatibilityKey: @YES, (id)kCVPixelBufferIOSurfacePropertiesKey: @{} };
    AVPlayerItemVideoOutput* out = [[AVPlayerItemVideoOutput alloc] initWithPixelBufferAttributes:oa];
    [item addOutput:out];
    AVPlayer* player = [AVPlayer playerWithPlayerItem:item];
    while (item.status == AVPlayerItemStatusUnknown) [[NSRunLoop currentRunLoop] runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.01]];
    auto ms = [](auto a, auto b){ return std::chrono::duration<double, std::milli>(b-a).count(); };
    auto stats = [](std::vector<double> v, const char* name){ std::sort(v.begin(), v.end()); printf("%-34s n=%zu median %.2f  p90 %.2f  max %.2f ms\n", name, v.size(), v[v.size()/2], v[v.size()*9/10], v.back()); };
    std::vector<double> tAll;
std::ifstream f("ext.filamat", std::ios::binary); std::vector<char> pkg((std::istreambuf_iterator<char>(f)), {});
    Material* mat = Material::Builder().package(pkg.data(), pkg.size()).build(*engine);
    MaterialInstance* mi = mat->createInstance();
    mi->setParameter("tex", videoTex, TextureSampler(TextureSampler::MagFilter::LINEAR));

    struct V { float x, y, z, u, v; };
    static const V verts[4] = { {-1,-0.5f,-1, 0,0}, {1,-0.5f,-1, 1,0}, {1,0.5f,-1, 1,1}, {-1,0.5f,-1, 0,1} };
    static const uint16_t idx[6] = {0,1,2, 0,2,3};
    VertexBuffer* vb = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
        .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
        .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*engine);
    vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(verts, sizeof(verts)));
    IndexBuffer* ib = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::USHORT).build(*engine);
    ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(idx, sizeof(idx)));
    utils::Entity quad = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-1,-1,-2},{1,1,0}}).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
        .material(0, mi).culling(false).build(*engine, quad);

    SwapChain* sc = engine->createSwapChain(W, H, 0);
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    utils::Entity camEnt = utils::EntityManager::get().create();
    Camera* cam = engine->createCamera(camEnt);
    cam->setProjection(Camera::Projection::ORTHO, -1, 1, -0.5, 0.5, 0.1, 10);
    Skybox* sky = Skybox::Builder().color({1.0f, 0.0f, 0.0f, 1.0f}).build(*engine);
    scene->setSkybox(sky); scene->addEntity(quad);
    view->setScene(scene); view->setCamera(cam); view->setRenderTarget(rt); view->setViewport({0, 0, W, H}); view->setPostProcessingEnabled(false);
    
    CVPixelBufferRef last = nullptr; int readbacks = 0;
    auto frame = [&](CVPixelBufferRef pb) -> double {
        auto t0 = std::chrono::steady_clock::now();
        auto h = platform->createExternalImage(pb);
        videoTex->setExternalImage(*engine, h);
        if (renderer->beginFrame(sc)) { renderer->render(view); renderer->endFrame(); }
        engine->flushAndWait();
        return ms(t0, std::chrono::steady_clock::now());
    };
    // --- MODE SCRUB: deterministic exact seek, then decode->Filament, per frame
    {
        std::vector<double> seek, render, total; srand48(7);
        for (int i = 0; i < 120; ++i) {
            double t = drand48() * 50.0; t = round(t * 30.0) / 30.0;
            auto t0 = std::chrono::steady_clock::now();
            __block bool done = false;
            [player seekToTime:CMTimeMakeWithSeconds(t, 600) toleranceBefore:kCMTimeZero toleranceAfter:kCMTimeZero completionHandler:^(BOOL){ done = true; }];
            while (!done) [[NSRunLoop currentRunLoop] runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.0005]];
            CVPixelBufferRef pb = nullptr;
            while (!(pb = [out copyPixelBufferForItemTime:item.currentTime itemTimeForDisplay:nullptr])) [[NSRunLoop currentRunLoop] runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.0005]];
            auto t1 = std::chrono::steady_clock::now();
            double r = frame(pb); CVPixelBufferRelease(pb);
            if (i >= 5) { seek.push_back(ms(t0, t1)); render.push_back(r); total.push_back(ms(t0, std::chrono::steady_clock::now())); }
        }
        stats(seek, "scrub: seek+copyPixelBuffer"); stats(render, "scrub: Filament frame (flushAndWait)"); stats(total, "scrub: input->pixels total");
    }
    // --- MODE PLAY: rate 2 (stand-in for 60 fps clip rate), poll at ~240Hz, count distinct frames and per-frame GPU
    {
        [player seekToTime:CMTimeMakeWithSeconds(5, 600) toleranceBefore:kCMTimeZero toleranceAfter:kCMTimeZero]; usleep(300000);
        player.rate = 2.0;
        auto start = std::chrono::steady_clock::now(); int distinct = 0; CMTime lastT = kCMTimeInvalid; std::vector<double> rend, gaps; auto lastDraw = start;
        while (ms(start, std::chrono::steady_clock::now()) < 5000) {
            CMTime now = [out itemTimeForHostTime:CACurrentMediaTime()];
            if ([out hasNewPixelBufferForItemTime:now]) {
                CVPixelBufferRef pb = [out copyPixelBufferForItemTime:now itemTimeForDisplay:nullptr];
                if (pb) { double r = frame(pb); CVPixelBufferRelease(pb); rend.push_back(r); ++distinct; auto n = std::chrono::steady_clock::now(); gaps.push_back(ms(lastDraw, n)); lastDraw = n; }
            } else usleep(500);
        }
        printf("play rate 2.0: %d frames drawn in 5.0 s = %.1f fps\n", distinct, distinct / 5.0);
        stats(rend, "play: Filament frame (flushAndWait)"); stats(gaps, "play: draw-to-draw gap");
        player.rate = 0;
    }
    printf("GPU->CPU pixel readbacks: %d   per-frame processes: 0   per-frame file reads by us: 0\n", readbacks);
    return 0;
}
