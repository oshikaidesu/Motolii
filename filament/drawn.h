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

// What `picture` and `type` share: something of ThorVG's (made by `paints` on Filament's own thread, which says how
// many points wide and high it is drawn) on a canvas that draws into a texture of Filament's device, and the sheet
// that shows it. ground below 0: clear where nothing is drawn. shown = false: made but not put in the scene (for a matte).
static Drawn* drawn(Stage& s, const std::function<tvg::Paint*(Drawn*, int*, int*)>& paints, const mat4f& place, float high, float3 ground, bool shown, const char* lay) {
    if (!door::device(s.engine)) return nullptr;   // (another backend)
    Drawn* d = new Drawn;
    WGPUTexture made = nullptr;
    tvg::Paint* loose = nullptr;
    bool owned = false;
    int w = 0, h = 0;
    door::on_driver(s.engine, [&] {
        tvg::Initializer::init(0);
        tvg::Paint* p = paints(d, &w, &h);
        if (!p || w <= 0 || h <= 0) return;
        loose = d->film ? nullptr : p;   // (a paint that no film owns is let go here if the canvas never takes it)
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
        if (ground.x >= 0.f) {
            tvg::Shape* under = tvg::Shape::gen();
            under->appendRect(0, 0, (float)w, (float)h);
            under->fill((uint8_t)std::lround(255.f * ground.x), (uint8_t)std::lround(255.f * ground.y), (uint8_t)std::lround(255.f * ground.z), 255);
            d->canvas->add(under);
        }
        d->canvas->add(p);
        owned = true;
        if (d->film) d->frames = d->film->totalFrame(), d->seconds = d->film->duration();
        d->canvas->update();
        d->canvas->draw(true);
        d->canvas->sync();
    });
    s.engine->flushAndWait();   // (once, at set-up: the lines above ran on the other thread)
    if (!made || !owned) {   // (not made: what was begun is let go)
        door::on_driver(s.engine, [&] { if (loose) tvg::Paint::rel(loose); delete d->canvas; delete d->film; if (made) wgpuTextureRelease(made); });
        s.engine->flushAndWait();
        delete d;
        return nullptr;
    }
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


// The picture in `file` put in the scene (an After Effects film, an SVG, a photo): a sheet `high` tall (as wide as
// the picture makes it) with its middle at `place`, facing along the place's z, drawn as finely as the stage is.
static Drawn* picture(Stage& s, const char* file, const mat4f& place, float high, float3 ground = float3{-1.f}, bool shown = true, const char* lay = nullptr) {
    if (!std::filesystem::is_regular_file(file)) return nullptr;   // (ThorVG ends the program on a folder)
    return drawn(s, [&](Drawn* d, int* w, int* h) -> tvg::Paint* {
        d->film = tvg::Animation::gen();
        tvg::Picture* p = d->film->picture();
        float fw = 0.f, fh = 0.f;
        if (p->load(file) != tvg::Result::Success || p->size(&fw, &fh) != tvg::Result::Success || fw <= 0.f || fh <= 0.f) return nullptr;
        const float fit = (float)std::max(s.w, s.h) / std::max(fw, fh);
        *w = 2 * (int)std::lround(0.5f * fw * fit);
        *h = 2 * (int)std::lround(0.5f * fh * fit);
        p->size((float)*w, (float)*h);
        return p;
    }, place, high, ground, shown, lay);
}

// Words put in the scene, set by ThorVG in the font of `fontFile` (a .ttf): a sheet `high` tall for each line, as
// wide as the words make it, its middle at `place`.
// Whether ThorVG sets `words` in a font file: it reads the file and the words come out with a size (a font of other
// letters gives them none). ThorVG's own loader keeps its fonts under a lock, so any thread may ask, once ThorVG is up.
static bool type_sets(const char* fontFile, const char* words) {
    if (!std::filesystem::is_regular_file(fontFile) || tvg::Text::load(fontFile) != tvg::Result::Success) return false;
    tvg::Text* t = tvg::Text::gen();
    t->font(std::filesystem::path(fontFile).stem().string().c_str());
    t->size(100.f);
    t->text(words);
    float x = 0.f, y = 0.f, wide = 0.f, tall = 0.f;
    const bool sets = t->bounds(&x, &y, &wide, &tall) == tvg::Result::Success && wide > 0.f && tall > 0.f;
    tvg::Paint::rel(t);
    return sets;
}
// edge: an outline round the letters, ThorVG's own (its colour; w: how wide, a share of the line's height; 0: none).
// Lines: the words may be broken by \n; align: how the lines lie one under another (ThorVG's own: 0 left, 1 right).
static Drawn* type(Stage& s, const char* words, const char* fontFile, const mat4f& place, float high, float3 colour = float3{0.f}, const char* lay = nullptr, float4 edge = float4{0.f}, float align = 0.f) {
    if (!std::filesystem::is_regular_file(fontFile)) return nullptr;
    return drawn(s, [&](Drawn*, int* w, int* h) -> tvg::Paint* {
        if (tvg::Text::load(fontFile) != tvg::Result::Success) return nullptr;
        tvg::Text* t = tvg::Text::gen();
        float x = 0.f, y = 0.f, wide = 0.f, tall = 0.f;
        const float fine = 0.12f * (float)std::max(s.w, s.h);   // (letters this many points tall: a line as fine as an eighth of the stage)
        t->font(std::filesystem::path(fontFile).stem().string().c_str());   // (ThorVG names a font by its file)
        t->size(fine);
        t->text(words);
        t->fill((uint8_t)std::lround(255.f * colour.x), (uint8_t)std::lround(255.f * colour.y), (uint8_t)std::lround(255.f * colour.z));
        bool measured = t->bounds(&x, &y, &wide, &tall) == tvg::Result::Success && wide > 0.f && tall > 0.f;
        if (measured && wide > 4096.f) {   // (a line too long for one picture is set smaller)
            t->size(fine * 4096.f / wide);
            measured = t->bounds(&x, &y, &wide, &tall) == tvg::Result::Success;
        }
        if (!measured) return tvg::Paint::rel(t), nullptr;
        if (align > 0.f) {   // (the lines aligned in a box as wide as the widest of them: ThorVG's layout and align)
            t->layout(wide, 0.f);
            t->align(align, 0.f);
            measured = t->bounds(&x, &y, &wide, &tall) == tvg::Result::Success;
        }
        const float outline = edge.w * tall;   // (half of it lies outside the letters: the picture is that much larger all round)
        if (outline > 0.f) t->outline(outline, (uint8_t)std::lround(255.f * edge.x), (uint8_t)std::lround(255.f * edge.y), (uint8_t)std::lround(255.f * edge.z));
        t->translate(-x + 0.5f * outline, -y + 0.5f * outline);
        *w = 2 * (int)std::ceil(0.5f * (wide + outline));
        *h = 2 * (int)std::ceil(0.5f * (tall + outline));
        return t;
    }, place, high, float3{-1.f}, true, lay);
}

// A flat shape in one colour, drawn by ThorVG's own Shape (a rectangle, a circle): `wide` times as wide as it is high.
// edge: ThorVG's stroke round it, inside its frame (its colour; w: how wide, a share of its height; 0: none).
static Drawn* flat(Stage& s, const char* form, float wide, const mat4f& place, float high, float3 colour, const char* lay = nullptr, float4 edge = float4{0.f}) {
    return drawn(s, [&](Drawn*, int* w, int* h) -> tvg::Paint* {
        const float tall = std::min(0.5f * (float)std::max(s.w, s.h), 4096.f / std::max(wide, 1e-3f));   // (as fine as half the stage; a long thin bar less high, so that it is as long as it is said)
        *h = std::max(2, 2 * (int)std::ceil(0.5f * tall));
        *w = std::max(2, 2 * (int)std::ceil(0.5f * tall * wide));
        tvg::Shape* shape = tvg::Shape::gen();
        const float in = 0.5f * std::min(edge.w * (float)*h, 0.5f * (float)std::min(*w, *h));   // (half the stroke: the path is set in by that, so all of it is seen)
        if (!strcmp(form, "circle")) shape->appendCircle(0.5f * (float)*w, 0.5f * (float)*h, 0.5f * (float)*w - in, 0.5f * (float)*h - in);
        else shape->appendRect(in, in, (float)*w - 2.f * in, (float)*h - 2.f * in);
        shape->fill((uint8_t)std::lround(255.f * colour.x), (uint8_t)std::lround(255.f * colour.y), (uint8_t)std::lround(255.f * colour.z));
        if (in > 0.f) shape->strokeWidth(2.f * in), shape->strokeFill((uint8_t)std::lround(255.f * edge.x), (uint8_t)std::lround(255.f * edge.y), (uint8_t)std::lround(255.f * edge.z));
        return shape;
    }, place, high, float3{-1.f}, true, lay);
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

