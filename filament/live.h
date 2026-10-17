// live.h: the stage's picture as it is drawn, handed to a shell on a screen without a copy. The main view draws into
// one of three pixel buffers that the GPU and the shell share (CVPixelBuffers on IOSurfaces: made as glass_door_mac.mm
// makes them, which did this on Filament's Metal backend). Here each buffer is shown to Dawn as a texture (the call
// video.h reads films by, used the other way round) and made a render target of Filament's through the door the
// patch already has (Texture::Builder::import). "Finished" is a Filament fence put after it signalled. macOS only.
#include <CoreVideo/CoreVideo.h>
#include <IOSurface/IOSurface.h>
#include <webgpu/webgpu.h>
#include <atomic>
#include <filament/Fence.h>

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
    double waited = 0;   // seconds spent waiting for the GPU to finish them (for measuring)
    Fence* pending = nullptr;          // the picture begun and not yet finished by the GPU, and which buffer it is in
    int pendingIn = -1;
    Live* solo = nullptr;              // a card's small picture of one thing alone (kit.h solo_view), drawn in the same frame, finished with it
};

static Live* live_open(Stage& s, uint32_t w = 0, uint32_t h = 0) {   // (0: the stage's own size)
    Live* l = new Live;
    if (!w) w = (uint32_t)s.w, h = (uint32_t)s.h;
    NSDictionary* like = @{(id)kCVPixelBufferIOSurfacePropertiesKey : @{}, (id)kCVPixelBufferMetalCompatibilityKey : @YES};
    for (CVPixelBufferRef& b : l->buffer)
        if (CVPixelBufferCreate(kCFAllocatorDefault, (size_t)w, (size_t)h, kCVPixelFormatType_32BGRA, (__bridge CFDictionaryRef)like, &b) != kCVReturnSuccess) return nullptr;
    Engine* engine = s.engine;
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
    s.out = l->target[l->next];
    if (!s.over) {   // (with a whole-screen effect the stage draws to its own picture and the effect to this one: screen.h)
        s.view->setRenderTarget(s.out);
        for (View* v : s.layers) v->setRenderTarget(s.out);   // (the pictures laid over it go to the same place)
    }
    Live* solo = l->solo && s.solo && !s.soloIn.empty() ? l->solo : nullptr;   // (one thing alone too, while a card shows it)
    if (solo) s.solo->setRenderTarget(solo->target[solo->next]);
    draw(s, nullptr, false);
    if (solo) {
        s.solo->setRenderTarget(nullptr);
        solo->pendingIn = solo->next;
        solo->next = (solo->next + 1) % Live::N;
    }
    l->pending = s.engine->createFence();   // (not waited for: the hand goes back to what it is asked; live_ready offers it once done)
    s.engine->flush();
    l->pendingIn = l->next;
    if (!s.over) {
        s.view->setRenderTarget(nullptr);
        for (View* v : s.layers) v->setRenderTarget(nullptr);
    }
    s.out = nullptr;
    l->next = (l->next + 1) % Live::N;
}
// Whether the picture begun last is finished (then it is offered to the shell, the front): the hand asks this rather
// than waiting for the GPU, and begins no other picture till it is (Thermion's way: research_notes/flutter_native_kin.md).
static bool live_ready(Stage& s, Live* l) {
    if (!l->pending) return true;
    if (l->pending->wait(Fence::Mode::DONT_FLUSH, 0) != Fence::FenceStatus::CONDITION_SATISFIED) return false;
    s.engine->destroy(l->pending);
    l->pending = nullptr;
    l->front = l->pendingIn;
    ++l->frames;
    if (Live* solo = l->solo; solo && solo->pendingIn >= 0) solo->front = solo->pendingIn, solo->pendingIn = -1, ++solo->frames;   // (finished with the stage's)
    return true;
}

// The last finished picture, held for the one who asks (a CVPixelBufferRef: the shell lets it go). Any thread.
static void* live_front(Live* l) {
    const int i = l->front;
    return i < 0 ? nullptr : (void*)CVPixelBufferRetain(l->buffer[i]);
}
