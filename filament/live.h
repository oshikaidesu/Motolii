// live.h: the stage's picture as it is drawn, handed to a shell on a screen without a copy. The main view draws into
// one of three pixel buffers that the GPU and the shell share (CVPixelBuffers on IOSurfaces: made as glass_door_mac.mm
// makes them, which did this on Filament's Metal backend). Here each buffer is shown to Dawn as a texture (the call
// video.h reads films by, used the other way round) and made a render target of Filament's through the door the
// patch already has (Texture::Builder::import). "Finished" is Filament's own flushAndWait coming back. macOS only.
#include <CoreVideo/CoreVideo.h>
#include <IOSurface/IOSurface.h>
#include <webgpu/webgpu.h>
#include <atomic>

struct Live {
    static constexpr int N = 3;
    CVPixelBufferRef buffer[N] = {};
    WGPUSharedTextureMemory lent[N] = {};
    WGPUTexture shared[N] = {};
    Texture* seen[N] = {};
    RenderTarget* target[N] = {};
    int next = 0;                      // the buffer the next picture is drawn into (they go round)
    std::atomic<int> front{-1};        // the last one finished
    std::atomic<uint64_t> frames{0};   // how many have been finished
};

static Live* live_open(Stage& s) {
    Live* l = new Live;
    NSDictionary* like = @{(id)kCVPixelBufferIOSurfacePropertiesKey : @{}, (id)kCVPixelBufferMetalCompatibilityKey : @YES};
    for (CVPixelBufferRef& b : l->buffer)
        if (CVPixelBufferCreate(kCFAllocatorDefault, (size_t)s.w, (size_t)s.h, kCVPixelFormatType_32BGRA, (__bridge CFDictionaryRef)like, &b) != kCVReturnSuccess) return nullptr;
    Engine* engine = s.engine;
    const uint32_t w = (uint32_t)s.w, h = (uint32_t)s.h;
    door::on_driver(engine, [l, engine, w, h] {
        for (int i = 0; i < Live::N; ++i) {
            WGPUSharedTextureMemoryIOSurfaceDescriptor from{};
            from.chain.sType = WGPUSType_SharedTextureMemoryIOSurfaceDescriptor;
            from.ioSurface = CVPixelBufferGetIOSurface(l->buffer[i]);
            WGPUSharedTextureMemoryDescriptor what{};
            what.nextInChain = &from.chain;
            l->lent[i] = wgpuDeviceImportSharedTextureMemory((WGPUDevice)door::device(engine), &what);
            WGPUTextureDescriptor as{};
            as.size = {w, h, 1};
            as.format = WGPUTextureFormat_BGRA8Unorm;
            as.usage = WGPUTextureUsage_RenderAttachment | WGPUTextureUsage_TextureBinding;
            as.dimension = WGPUTextureDimension_2D;
            as.mipLevelCount = 1;
            as.sampleCount = 1;
            l->shared[i] = wgpuSharedTextureMemoryCreateTexture(l->lent[i], &as);
            WGPUSharedTextureMemoryBeginAccessDescriptor lend{};   // (lent while the stage is live: on macOS nothing is locked by it)
            lend.initialized = false;
            wgpuSharedTextureMemoryBeginAccess(l->lent[i], l->shared[i], &lend);
        }
    });
    s.engine->flushAndWait();
    for (int i = 0; i < Live::N; ++i) {
        if (!l->shared[i]) return nullptr;
        l->seen[i] = Texture::Builder().width(w).height(h).levels(1).sampler(Texture::Sampler::SAMPLER_2D).format(Texture::InternalFormat::RGBA8)
                         .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE).import((intptr_t)l->shared[i]).build(*s.engine);
        l->target[i] = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, l->seen[i]).build(*s.engine);
    }
    return l;
}

// The stage's picture as it now is, drawn into the next buffer; it is the front one once the GPU has finished it.
static void live_draw(Stage& s, Live* l) {
    s.view->setRenderTarget(l->target[l->next]);
    draw(s, nullptr, true);
    s.view->setRenderTarget(nullptr);
    l->front = l->next;
    l->next = (l->next + 1) % Live::N;
    ++l->frames;
}

// The last finished picture, held for the one who asks (a CVPixelBufferRef: the shell lets it go). Any thread.
static void* live_front(Live* l) {
    const int i = l->front;
    return i < 0 ? nullptr : (void*)CVPixelBufferRetain(l->buffer[i]);
}
