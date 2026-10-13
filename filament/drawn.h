// drawn.h: pictures ThorVG draws, in the scene. ThorVG draws with WebGPU on Filament's own device straight into a
// texture, and Filament reads that texture as any other: nothing goes through the CPU, nothing is copied, and it is
// the same on every system Filament's WebGPU backend runs on. Whatever ThorVG can draw comes this way: a Lottie film
// with its masks, mattes, shapes, type and effects, an SVG, shapes of one's own.
// Only on the WebGPU backend (BACKEND=webgpu, dawn_probe/kit_wgpu.sh), with ThorVG built against Dawn
// (dawn_probe/tvg_on_dawn.sh) and Filament patched to lend its device and take a texture (gpu_door.h).
// ThorVG's drawing is queued like Filament's own calls and runs on Filament's driver thread, so the picture is ready
// for the frame rendered after it; and all of ThorVG's work for a picture is done there, never on the caller's thread.
#include "dawn_probe/gpu_door.h"
#include <thorvg.h>
#include <webgpu/webgpu.h>
#include <functional>

struct Drawn {
    tvg::WgCanvas* canvas = nullptr;   // (touched on the driver thread only: inside `set` and `each`)
    Texture* texture = nullptr;        // what Filament reads
    int w = 0, h = 0;
};

// A picture of w x h for ThorVG to draw. `set` is run once, on the driver thread, with the canvas: put there what is to be drawn.
static Drawn* drawn_open(Stage& s, int w, int h, std::function<void(tvg::WgCanvas*)> set) {
    if (!door::device(s.engine)) return nullptr;   // (another backend)
    Drawn* d = new Drawn;
    d->w = w;
    d->h = h;
    WGPUTexture made = nullptr;
    door::on_driver(s.engine, [&] {
        WGPUTextureDescriptor like{};
        like.size = {(uint32_t)w, (uint32_t)h, 1};
        like.format = WGPUTextureFormat_BGRA8Unorm;   // (what ThorVG's last step writes)
        like.usage = WGPUTextureUsage_RenderAttachment | WGPUTextureUsage_TextureBinding | WGPUTextureUsage_CopySrc;
        like.dimension = WGPUTextureDimension_2D;
        like.mipLevelCount = 1;
        like.sampleCount = 1;
        made = wgpuDeviceCreateTexture((WGPUDevice)door::device(s.engine), &like);
        tvg::Initializer::init(0);
        d->canvas = tvg::WgCanvas::gen();
        if (d->canvas && d->canvas->target(door::device(s.engine), door::instance(s.engine), made, (uint32_t)w, (uint32_t)h, tvg::ColorSpace::ABGR8888S, 1) == tvg::Result::Success) set(d->canvas);
    });
    s.engine->flushAndWait();   // (once, at set-up: `made` is filled in on the other thread)
    if (!made || !d->canvas) return nullptr;
    d->texture = Texture::Builder().width((uint32_t)w).height((uint32_t)h).levels(1).sampler(Texture::Sampler::SAMPLER_2D).format(Texture::InternalFormat::RGBA8)
                     .usage(Texture::Usage::SAMPLEABLE).import((intptr_t)made).build(*s.engine);
    return d;
}

// The picture drawn anew for the frame that comes next. `each` (if any) is run first, on the driver thread: move
// what is on the canvas to its moment there (a film's frame).
static void drawn_now(Stage& s, Drawn* d, std::function<void()> each = {}) {
    door::on_driver(s.engine, [d, each] {
        if (each) each();
        d->canvas->update();
        d->canvas->draw(true);
        d->canvas->sync();
    });
}
