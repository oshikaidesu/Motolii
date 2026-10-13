// drawn.h: a picture ThorVG draws, put in the scene as a thing like any other. The scene and its things are
// Filament's (an entity with a place and a look); this only makes one whose look is ThorVG's picture: a flat sheet.
// The picture is a file: an After Effects film (Lottie) with all it holds, or an SVG. ThorVG draws it with WebGPU on
// Filament's own device straight into a texture Filament reads: nothing goes through the CPU, nothing is copied.
// Only on the WebGPU backend (BACKEND=webgpu, dawn_probe/kit_wgpu.sh), with ThorVG built against Dawn
// (dawn_probe/tvg_on_dawn.sh) and Filament patched to lend its device and take a texture (gpu_door.h).
// All of ThorVG's work is queued like Filament's own calls and runs on Filament's driver thread.
#include "dawn_probe/gpu_door.h"
#include <thorvg.h>
#include <webgpu/webgpu.h>

#include <filesystem>

struct Drawn {
    tvg::WgCanvas* canvas = nullptr;   // (touched on the driver thread only)
    tvg::Animation* film = nullptr;    // (the same)
    Texture* texture = nullptr;        // what Filament reads
    Mesh* sheet = nullptr;             // the thing in the scene: moved like any other (kit.h: move)
    MaterialInstance* look = nullptr;  // its look (none if it is not shown)
    float seconds = 0.f, frames = 0.f; // a film's length (a still: 0)
    float starts = 0.f;                // how far into itself it is when the stage's time is 0
};

// A font for the pictures put after it (an SVG names its fonts; ThorVG knows only the ones it is given).
static void font(Stage& s, const char* path) {
    door::on_driver(s.engine, [path] {
        tvg::Initializer::init(0);
        tvg::Text::load(path);
    });
}

// A film at its moment, `seconds` from its start (after its end it holds its last picture), for the frame that comes next.
static void picture_at(Stage& s, Drawn* d, float seconds) {
    door::on_driver(s.engine, [d, seconds] {
        d->film->frame(std::min(seconds / d->seconds * d->frames, d->frames - 1.f));
        d->canvas->update();
        d->canvas->draw(true);
        d->canvas->sync();
    });
}

// The picture in `file` put in the scene: a sheet `high` tall (as wide as the picture makes it) with its middle at
// `place`, facing along the place's z. It is drawn as finely as the stage is, and where the file draws nothing it is `ground`
// (a ground below 0: clear there). shown = false: the sheet and its picture are made but not put in the scene (for a
// matte: kit.h, matte(s, d->sheet, {place}, d->texture)).
static Drawn* picture(Stage& s, const char* file, const mat4f& place, float high, float3 ground = float3{-1.f}, bool shown = true, const char* lay = nullptr) {
    if (!door::device(s.engine)) return nullptr;   // (another backend)
    if (!std::filesystem::is_regular_file(file)) return nullptr;   // (ThorVG ends the program on a folder)
    Drawn* d = new Drawn;
    WGPUTexture made = nullptr;
    int w = 0, h = 0;
    door::on_driver(s.engine, [&] {
        tvg::Initializer::init(0);
        d->film = tvg::Animation::gen();
        tvg::Picture* p = d->film->picture();
        float fw = 0.f, fh = 0.f;
        if (p->load(file) != tvg::Result::Success || p->size(&fw, &fh) != tvg::Result::Success || fw <= 0.f || fh <= 0.f) return;
        const float fit = (float)std::max(s.w, s.h) / std::max(fw, fh);
        w = 2 * (int)std::lround(0.5f * fw * fit);
        h = 2 * (int)std::lround(0.5f * fh * fit);
        WGPUTextureDescriptor like{};
        like.size = {(uint32_t)w, (uint32_t)h, 1};
        like.format = WGPUTextureFormat_BGRA8Unorm;   // (what ThorVG's last step writes)
        like.usage = WGPUTextureUsage_RenderAttachment | WGPUTextureUsage_TextureBinding | WGPUTextureUsage_CopySrc;
        like.dimension = WGPUTextureDimension_2D;
        like.mipLevelCount = 1;
        like.sampleCount = 1;
        made = wgpuDeviceCreateTexture((WGPUDevice)door::device(s.engine), &like);
        d->canvas = tvg::WgCanvas::gen();
        if (!d->canvas || d->canvas->target(door::device(s.engine), door::instance(s.engine), made, (uint32_t)w, (uint32_t)h, tvg::ColorSpace::ABGR8888S, 1) != tvg::Result::Success) return;
        p->size((float)w, (float)h);
        tvg::Shape* under = tvg::Shape::gen();
        under->appendRect(0, 0, (float)w, (float)h);
        under->fill((uint8_t)std::lround(255.f * ground.x), (uint8_t)std::lround(255.f * ground.y), (uint8_t)std::lround(255.f * ground.z), 255);
        if (ground.x >= 0.f) d->canvas->add(under);
        d->canvas->add(p);
        d->frames = d->film->totalFrame();
        d->seconds = d->film->duration();
        d->canvas->update();
        d->canvas->draw(true);
        d->canvas->sync();
    });
    s.engine->flushAndWait();   // (once, at set-up: the lines above ran on the other thread)
    if (!made || !d->canvas || w <= 0) return nullptr;
    d->texture = Texture::Builder().width((uint32_t)w).height((uint32_t)h).levels(1).sampler(Texture::Sampler::SAMPLER_2D).format(Texture::InternalFormat::RGBA8)
                     .usage(Texture::Usage::SAMPLEABLE).import((intptr_t)made).build(*s.engine);
    d->sheet = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) {
        d->sheet->pos.push_back({0.5f * high * (float)w / (float)h * c.x, 0.5f * high * c.y, 0.f});
        d->sheet->nrm.push_back({0, 0, 1});
        d->sheet->uv.push_back(c * 0.5f + 0.5f);
    }
    d->sheet->idx = {0, 1, 2, 0, 2, 3};
    if (d->frames > 1.f)   // (a film: it goes on by itself as the stage's time does, round and round)
        s.playing[d] = [&s, d](double now) { picture_at(s, d, std::fmod((float)now + d->starts, d->seconds)); };
    if (!shown) return d;
    d->look = add(s, d->sheet, flat_laid(lay).c_str(), {place});
    if (!d->look) return nullptr;
    d->look->setParameter("picture", d->texture, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
    return d;
}


// A picture taken off the stage for good.
static void picture_off(Stage& s, Drawn* d) {
    s.playing.erase(d);
    take_off(s, d->sheet);
    door::on_driver(s.engine, [d] { delete d->canvas; delete d->film; });
    s.engine->flushAndWait();
    s.engine->destroy(d->texture);
    delete d;
}

