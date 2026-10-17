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
    void* marks[N] = {};               // (a desk's: ThorVG's canvas on each buffer, the marks drawn over the picture: desk_marks)
    Live* desk = nullptr;              // the editing window's whole stage pane (kit.h desk_lay): while it is on, the stage is drawn there, not here
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

// Its buffers done away with, once the GPU has finished with them (a desk made again at another size: the old one).
static void live_close(Stage& s, Live* l) {
    if (!l) return;
    s.engine->flushAndWait();
    door::on_driver(s.engine, [l] { for (void* c : l->marks) delete reinterpret_cast<tvg::WgCanvas*>(c); });
    for (int i = 0; i < Live::N; ++i) {
        if (l->target[i]) s.engine->destroy(l->target[i]);
        if (l->seen[i]) s.engine->destroy(l->seen[i]);
    }
    s.engine->flushAndWait();
    door::on_driver(s.engine, [l] {
        for (int i = 0; i < Live::N; ++i) {
            if (l->shared[i]) wgpuTextureRelease(l->shared[i]);
            if (l->lent[i]) wgpuSharedTextureMemoryRelease(l->lent[i]);
        }
    });
    s.engine->flushAndWait();
    for (CVPixelBufferRef b : l->buffer) if (b) CVPixelBufferRelease(b);   // (a shell holding one keeps it till it lets go)
    if (l->pending) s.engine->destroy(l->pending);
    delete l;
}

// The marks over the desk's picture (and the cover over what lies off the stage, under them), drawn by ThorVG into the same buffer after Filament's picture and not clearing
// it, on the same device and queue (drawn.h's way): the picked thing's frame, its four corners, the knob it turns by,
// the point it turns about; the frame of the one under the hand, thin. As the window's own marks were (stage_view.dart).
static void desk_marks(Stage& s, Live* l, int i) {
    const auto k = s.desk;

    void *device = door::device(s.engine), *instance = door::instance(s.engine);
    const int s_w = s.w, s_h = s.h;
    door::on_driver(s.engine, [l, i, k, device, instance, s_w, s_h] {
        auto& canvas = reinterpret_cast<tvg::WgCanvas*&>(l->marks[i]);
        if (!canvas) {
            tvg::Initializer::init(0);   // (as drawn.h: counted, once is enough)
            canvas = tvg::WgCanvas::gen();
            if (!canvas || canvas->target(device, instance, l->shared[i], k.w, k.h, tvg::ColorSpace::ABGR8888S, 2) != tvg::Result::Success) return;   // (2: drawn over: thorvg_over.patch)
        }
        canvas->remove();
        const float pt = (float)k.point;
        const auto at = [&](double x, double y) { return float2{(float)(x * k.scale + k.x), (float)(y * k.scale + k.y)}; };
        struct Frame { float2 c[4], top, mid, pivot; };
        const auto frame = [&](const double* f) {   // (its corners as seen: turned clockwise about its middle)
            const double turn = f[4] * 3.14159265358979 / 180.0, cs = std::cos(turn), sn = std::sin(turn);
            const auto local = [&](double u, double v) { const double x = u * 0.5 * f[2], y = v * 0.5 * f[3]; return at(f[0] + x * cs - y * sn, f[1] + x * sn + y * cs); };
            return Frame{{local(-1, -1), local(1, -1), local(1, 1), local(-1, 1)}, local(0, -1), at(f[0], f[1]), at(f[5], f[6])};
        };
        const auto line = [&](std::initializer_list<float2> pts, bool closed, float width, uint8_t r, uint8_t g, uint8_t b) {
            tvg::Shape* sh = tvg::Shape::gen();
            bool first = true;
            for (float2 p : pts) { if (first) sh->moveTo(p.x, p.y); else sh->lineTo(p.x, p.y); first = false; }
            if (closed) sh->close();
            sh->strokeWidth(width * pt), sh->strokeFill(r, g, b, 255);
            canvas->add(sh);
        };
        const auto legible = [&](std::initializer_list<float2> pts, bool closed, float width) { line(pts, closed, width + 2.f, 0, 0, 0), line(pts, closed, width, 0xB3, 0xC6, 0x6B); };
        const auto dot = [&](float2 c, float r, bool lit) { tvg::Shape* sh = tvg::Shape::gen(); sh->appendCircle(c.x, c.y, r * pt, r * pt); lit ? sh->fill(0xB3, 0xC6, 0x6B, 255) : sh->fill(0, 0, 0, 255); canvas->add(sh); };
        const auto square = [&](float2 c, float half, bool lit) { tvg::Shape* sh = tvg::Shape::gen(); sh->appendRect(c.x - half * pt, c.y - half * pt, 2.f * half * pt, 2.f * half * pt); lit ? sh->fill(0xB3, 0xC6, 0x6B, 255) : sh->fill(0, 0, 0, 255); canvas->add(sh); };
        tvg::Shape* off = tvg::Shape::gen();   // (what lies off the stage, covered half way: the stage's own frame reads as the picture)
        off->appendRect(0, 0, (float)k.w, (float)k.h);
        off->appendRect((float)k.x, (float)k.y, (float)(s_w * k.scale), (float)(s_h * k.scale), 0, 0, false);
        off->fill(0x19, 0x19, 0x19, 140);   // (the window's surround, Grey.g10, half way)
        canvas->add(off);
        if (k.hovered) {
            const Frame f = frame(k.hover);
            line({f.c[0], f.c[1], f.c[2], f.c[3]}, true, 3.f, 0, 0, 0), line({f.c[0], f.c[1], f.c[2], f.c[3]}, true, 1.f, 0xE6, 0xE6, 0xE6);
        }
        if (k.picked) {
            const Frame f = frame(k.pick);
            legible({f.c[0], f.c[1], f.c[2], f.c[3]}, true, 1.5f);
            if (k.knob) {   // (the knob it is turned by, a stem's length out from the middle of its top edge)
                float2 n = f.top - f.mid;
                const float len = length(n);
                n = len < 1.f ? float2{0.f, -1.f} : n / len;
                const float2 knob = f.top + n * 18.f * pt;
                legible({f.top, knob}, false, 1.f);
                dot(knob, 4.5f, false), dot(knob, 3.5f, true);
            }
            for (float2 c : f.c) square(c, 4.5f, false), square(c, 3.5f, true);
            for (const auto& [width, lit] : {std::pair{3.5f, false}, std::pair{1.5f, true}}) {   // (the point it turns about: a ring and four ticks)
                tvg::Shape* ring = tvg::Shape::gen();
                ring->appendCircle(f.pivot.x, f.pivot.y, 6.f * pt, 6.f * pt);
                ring->strokeWidth(width * pt), lit ? ring->strokeFill(0xB3, 0xC6, 0x6B, 255) : ring->strokeFill(0, 0, 0, 255);
                canvas->add(ring);
            }
            for (float2 d : {float2{1, 0}, float2{-1, 0}, float2{0, 1}, float2{0, -1}}) {
                line({f.pivot + d * 3.f * pt, f.pivot + d * 10.f * pt}, false, 3.f, 0, 0, 0);
                line({f.pivot + d * 3.f * pt, f.pivot + d * 10.f * pt}, false, 1.2f, 0xB3, 0xC6, 0x6B);
            }
        }
        canvas->update();
        canvas->draw(true);   // (its own picture cleared; the buffer drawn over, not replaced: target type 2, thorvg_over.patch)
        canvas->sync();
    });
}

// The stage's picture as it now is, drawn into the next buffer; it is the front one once the GPU has finished it.
static void live_draw(Stage& s, Live* l) {
    Live* desk = l->desk && s.desk.w ? l->desk : nullptr;
    s.out = desk ? desk->target[desk->next] : l->target[l->next];
    if (desk) desk_lay(s, true);
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
    if (desk) desk_marks(s, desk, desk->next);
    if (desk) {
        desk_lay(s, false);
        desk->pendingIn = desk->next;
        desk->next = (desk->next + 1) % Live::N;
    }
    l->pending = s.engine->createFence();   // (not waited for: the hand goes back to what it is asked; live_ready offers it once done)
    s.engine->flush();
    l->pendingIn = l->next;
    if (!s.over) {
        s.view->setRenderTarget(nullptr);
        for (View* v : s.layers) v->setRenderTarget(nullptr);
    }
    s.out = nullptr;
    if (desk) l->pendingIn = -1;   // (the stage's own buffer was not drawn: its last picture stays the front)
    else l->next = (l->next + 1) % Live::N;
}
// Whether the picture begun last is finished (then it is offered to the shell, the front): the hand asks this rather
// than waiting for the GPU, and begins no other picture till it is (Thermion's way: research_notes/flutter_native_kin.md).
static bool live_ready(Stage& s, Live* l) {
    if (!l->pending) return true;
    if (l->pending->wait(Fence::Mode::DONT_FLUSH, 0) != Fence::FenceStatus::CONDITION_SATISFIED) return false;
    s.engine->destroy(l->pending);
    l->pending = nullptr;
    if (l->pendingIn >= 0) l->front = l->pendingIn;
    ++l->frames;
    if (Live* desk = l->desk; desk && desk->pendingIn >= 0) desk->front = desk->pendingIn, desk->pendingIn = -1, ++desk->frames;
    if (Live* solo = l->solo; solo && solo->pendingIn >= 0) solo->front = solo->pendingIn, solo->pendingIn = -1, ++solo->frames;   // (finished with the stage's)
    return true;
}

// The last finished picture, held for the one who asks (a CVPixelBufferRef: the shell lets it go). Any thread.
static void* live_front(Live* l) {
    const int i = l->front;
    return i < 0 ? nullptr : (void*)CVPixelBufferRetain(l->buffer[i]);
}
