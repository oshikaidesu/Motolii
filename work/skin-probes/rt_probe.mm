// Throwaway probe. Question: can Filament (Metal) render into an IOSurface-backed MTLTexture that WE created and import()ed?
// If yes, the frame goes renderer -> IOSurface with no readback, and the same MTLTexture can be handed to wgpu / Flutter.
#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#import <IOSurface/IOSurface.h>
#include <filament/Engine.h>
#include <filament/Renderer.h>
#include <filament/Scene.h>
#include <filament/View.h>
#include <filament/Camera.h>
#include <filament/RenderTarget.h>
#include <filament/Texture.h>
#include <filament/SwapChain.h>
#include <filament/Skybox.h>
#include <filament/Viewport.h>
#include <utils/EntityManager.h>
#include <cstdio>
using namespace filament;

int main() {
    const uint32_t W = 256, H = 128;
    Engine* engine = Engine::create(Engine::Backend::METAL);
    if (!engine) { printf("no engine\n"); return 1; }

    id<MTLDevice> dev = MTLCreateSystemDefaultDevice();
    NSDictionary* props = @{
        (id)kIOSurfaceWidth: @(W), (id)kIOSurfaceHeight: @(H),
        (id)kIOSurfaceBytesPerElement: @4,
        (id)kIOSurfacePixelFormat: @((uint32_t)'BGRA')
    };
    IOSurfaceRef surf = IOSurfaceCreate((CFDictionaryRef)props);
    MTLTextureDescriptor* d = [MTLTextureDescriptor texture2DDescriptorWithPixelFormat:MTLPixelFormatBGRA8Unorm width:W height:H mipmapped:NO];
    d.usage = MTLTextureUsageRenderTarget | MTLTextureUsageShaderRead;
    id<MTLTexture> mtl = [dev newTextureWithDescriptor:d iosurface:surf plane:0];
    printf("IOSurface-backed MTLTexture: %s\n", mtl ? "created" : "FAILED");

    Texture* color = Texture::Builder().width(W).height(H).levels(1)
        .format(Texture::InternalFormat::RGBA8)
        .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE)
        .import((intptr_t)CFBridgingRetain(mtl)).build(*engine);
    printf("imported as Filament Texture: %s\n", color ? "ok" : "FAILED");
    Texture* depth = Texture::Builder().width(W).height(H).levels(1)
        .format(Texture::InternalFormat::DEPTH24).usage(Texture::Usage::DEPTH_ATTACHMENT).build(*engine);
    RenderTarget* rt = RenderTarget::Builder()
        .texture(RenderTarget::AttachmentPoint::COLOR, color)
        .texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
    printf("RenderTarget: %s\n", rt ? "ok" : "FAILED");

    SwapChain* sc = engine->createSwapChain(W, H, 0);
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    utils::Entity camEnt = utils::EntityManager::get().create();
    Camera* cam = engine->createCamera(camEnt);
    Skybox* sky = Skybox::Builder().color({1.0f, 0.0f, 0.0f, 1.0f}).build(*engine);
    scene->setSkybox(sky);
    view->setScene(scene); view->setCamera(cam); view->setRenderTarget(rt);
    view->setViewport({0, 0, W, H});
    view->setPostProcessingEnabled(false);

    for (int i = 0; i < 3; ++i) {
        if (renderer->beginFrame(sc)) { renderer->render(view); renderer->endFrame(); }
        engine->flushAndWait();
    }
    IOSurfaceLock(surf, kIOSurfaceLockReadOnly, nullptr);
    const uint8_t* p = (const uint8_t*)IOSurfaceGetBaseAddress(surf);
    printf("pixel(0,0) in the IOSurface (B,G,R,A bytes): %u %u %u %u   (skybox colour was pure red: expect B=0 G=0 R=255)\n", p[0], p[1], p[2], p[3]);
    IOSurfaceUnlock(surf, kIOSurfaceLockReadOnly, nullptr);
    return 0;
}
