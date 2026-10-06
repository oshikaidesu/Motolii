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
#include <fstream>
#include <vector>
#include <cstdio>
using namespace filament;

int main() {
    const uint32_t W = 256, H = 128;
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

    // the "decoded frame": an IOSurface-backed BGRA CVPixelBuffer, filled green (stand-in for a VideoToolbox frame)
    NSDictionary* pbAttrs = @{ (id)kCVPixelBufferMetalCompatibilityKey: @YES, (id)kCVPixelBufferIOSurfacePropertiesKey: @{} };
    CVPixelBufferRef pb = nullptr;
    CVPixelBufferCreate(kCFAllocatorDefault, 64, 64, kCVPixelFormatType_32BGRA, (CFDictionaryRef)pbAttrs, &pb);
    CVPixelBufferLockBaseAddress(pb, 0);
    uint8_t* px = (uint8_t*)CVPixelBufferGetBaseAddress(pb);
    size_t stride = CVPixelBufferGetBytesPerRow(pb);
    for (int y = 0; y < 64; ++y) for (int x = 0; x < 64; ++x) { uint8_t* q = px + y * stride + x * 4; q[0] = 0; q[1] = 255; q[2] = 0; q[3] = 255; }
    CVPixelBufferUnlockBaseAddress(pb, 0);
    printf("CVPixelBuffer has IOSurface: %s\n", CVPixelBufferGetIOSurface(pb) ? "yes" : "NO");

    auto* platform = static_cast<backend::PlatformMetal*>(engine->getPlatform());
    backend::Platform::ExternalImageHandle handle = platform->createExternalImage(pb);
    Texture* videoTex = Texture::Builder().sampler(Texture::Sampler::SAMPLER_EXTERNAL).format(Texture::InternalFormat::RGB8).external().build(*engine);
    videoTex->setExternalImage(*engine, handle);
    printf("setExternalImage done\n");

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
    for (int i = 0; i < 3; ++i) { if (renderer->beginFrame(sc)) { renderer->render(view); renderer->endFrame(); } engine->flushAndWait(); }

    IOSurfaceLock(outSurf, kIOSurfaceLockReadOnly, nullptr);
    const uint8_t* p = (const uint8_t*)IOSurfaceGetBaseAddress(outSurf);
    auto at = [&](int x, int y) { return p + (size_t)y * IOSurfaceGetBytesPerRow(outSurf) + x * 4; };
    printf("centre pixel (B,G,R,A): %u %u %u %u   corner pixel: %u %u %u %u\n", at(W/2,H/2)[0], at(W/2,H/2)[1], at(W/2,H/2)[2], at(W/2,H/2)[3], at(2,2)[0], at(2,2)[1], at(2,2)[2], at(2,2)[3]);
    printf("expect centre = video colour (green: 0 255 0), corner = skybox red (0 0 255)\n");
    IOSurfaceUnlock(outSurf, kIOSurfaceLockReadOnly, nullptr);
    return 0;
}
