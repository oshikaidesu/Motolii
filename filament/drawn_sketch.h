// drawn_sketch.h: part of drawn.h. Read only through drawn.h.
// A p5.js sketch that paints in 2D, run with no page (p5_stand.h): each frame what it paints is ThorVG's shapes on the
// picture's canvas, which keeps what it held (Canvas::draw(false): WgCompositor loads what is there), as a sketch's canvas
// does, so a sketch that never wipes builds up its painting and a frame costs what it adds.
#include "script.h"
static tvg::Fill* sketch_ramp(const double* g, size_t many) {   // (a canvas gradient's numbers, p5_stand.h's ramp)
    std::vector<tvg::Fill::ColorStop> stops;
    for (size_t i = 7; i + 5 <= many; i += 5)
        stops.push_back({(float)g[i], (uint8_t)std::lround(g[i + 1]), (uint8_t)std::lround(g[i + 2]), (uint8_t)std::lround(g[i + 3]), (uint8_t)std::lround(g[i + 4])});
    tvg::Fill* out;
    if (g[0] == 1.0) { auto* l = tvg::LinearGradient::gen(); l->linear((float)g[1], (float)g[2], (float)g[4], (float)g[5]); out = l; }
    else { auto* r = tvg::RadialGradient::gen(); r->radial((float)g[4], (float)g[5], (float)g[6], (float)g[1], (float)g[2], (float)g[3]); out = r; }
    out->colorStops(stops.data(), (uint32_t)stops.size());
    return out;
}
static tvg::Paint* sketch_words(Drawn* d, const double* p, const double* q, const double* end) {
    std::string words;   // (its letters, as UTF-8)
    for (const double* c = q + 6; c < end; ++c) {
        const uint32_t u = (uint32_t)*c;
        if (u < 0x80) words += (char)u;
        else if (u < 0x800) words += (char)(0xC0 | u >> 6), words += (char)(0x80 | (u & 63));
        else if (u < 0x10000) words += (char)(0xE0 | u >> 12), words += (char)(0x80 | (u >> 6 & 63)), words += (char)(0x80 | (u & 63));
        else words += (char)(0xF0 | u >> 18), words += (char)(0x80 | (u >> 12 & 63)), words += (char)(0x80 | (u >> 6 & 63)), words += (char)(0x80 | (u & 63));
    }
    tvg::Text* t = tvg::Text::gen();
    t->font(d->sketchFont.c_str());
    t->size((float)p[6]);
    t->text(words.c_str());
    t->fill((uint8_t)std::lround(p[1]), (uint8_t)std::lround(p[2]), (uint8_t)std::lround(p[3]));
    t->opacity((uint8_t)std::lround(p[4]));
    tvg::TextMetrics m{};
    t->metrics(m);
    float x = 0.f, y = 0.f, w = 0.f, h = 0.f;
    t->bounds(&x, &y, &w, &h);
    const float across = p[7] == 1.0 ? -0.5f * w : p[7] == 2.0 ? -w : 0.f;   // (textAlign)
    const float down = p[8] == 1.0 ? 0.f : p[8] == 2.0 ? -0.5f * (m.ascent - m.descent) : p[8] == 3.0 ? -(m.ascent - m.descent) : -m.ascent;   // (textBaseline; ThorVG's words hang from their top)
    t->transform(tvg::Matrix{(float)q[0], (float)q[2], (float)(q[4] + q[0] * across + q[2] * down), (float)q[1], (float)q[3], (float)(q[5] + q[1] * across + q[3] * down), 0.f, 0.f, 1.f});
    return t;
}
// A whole picture, p5_stand.h's kind 4: its matrix, its number in __bitmaps, its size, the part of it shown, whether it is smoothed.
static tvg::Paint* sketch_bitmap(Drawn* d, const double* p, const double* q, std::vector<std::shared_ptr<std::vector<uint8_t>>>& held) {
    std::lock_guard<std::mutex> hold(d->sketchPicsLock);
    const auto pic = d->sketchPics.find((int)q[6]);
    const uint32_t w = (uint32_t)q[7], h = (uint32_t)q[8];
    if (pic == d->sketchPics.end() || pic->second->size() != 4ull * w * h) return nullptr;
    held.push_back(pic->second);
    tvg::Picture* one = tvg::Picture::gen();
    if (q[13] != 0.0) one->filter(tvg::FilterMethod::Nearest);
    one->load((const uint32_t*)pic->second->data(), w, h, tvg::ColorSpace::ABGR8888S, false);
    const tvg::Matrix at{(float)q[0], (float)q[2], (float)q[4], (float)q[1], (float)q[3], (float)q[5], 0.f, 0.f, 1.f};
    one->transform(at), one->opacity((uint8_t)std::lround(p[4]));
    if (q[9] > 0.0 || q[10] > 0.0 || q[11] < q[7] || q[12] < q[8]) {
        tvg::Shape* part = tvg::Shape::gen();
        part->appendRect((float)q[9], (float)q[10], (float)q[11], (float)q[12]), part->fill(255, 255, 255), part->transform(at), one->clip(part);
    }
    return one;
}
// A conic gradient, which ThorVG has not: by the canvas spec's words, each point's colour by its angle from the first ray,
// a picture of the shape's bounds, the shape its clip. A table of 1024 steps, not the stops each time. (p5_stand.h's ramp.)
static tvg::Paint* sketch_conic(tvg::Shape* shape, const double* g, size_t many, std::vector<std::shared_ptr<std::vector<uint8_t>>>& held) {
    float bx = 0.f, by = 0.f, bw = 0.f, bh = 0.f;
    shape->bounds(&bx, &by, &bw, &bh);
    const int W = std::clamp((int)std::ceil(bw), 1, 1024), H = std::clamp((int)std::ceil(bh), 1, 1024);
    const double det = g[1] * g[4] - g[2] * g[3];
    if (det == 0.0) return shape;
    uint32_t ramp[1024];
    for (int i = 0; i < 1024; ++i) {
        const double t = (i + 0.5) / 1024.0;
        size_t b = 7;
        while (b + 5 < many && g[b + 5] <= t) b += 5;
        const size_t c = b + 5 < many ? b + 5 : b;
        const double f = c == b || t <= g[b] ? 0.0 : (t - g[b]) / (g[c] - g[b]);
        uint32_t out = 0;
        for (int k = 3; k >= 0; --k) out = out << 8 | (uint32_t)std::lround(g[b + 1 + k] + f * (g[c + 1 + k] - g[b + 1 + k]));
        ramp[i] = out;
    }
    auto pic = std::make_shared<std::vector<uint8_t>>(4ull * W * H);
    uint32_t* to = (uint32_t*)pic->data();
    for (int y = 0; y < H; ++y) for (int x = 0; x < W; ++x) {
        const double dx = bx + (x + 0.5) * bw / W - g[5], dy = by + (y + 0.5) * bh / H - g[6];
        double t = std::atan2((g[1] * dy - g[2] * dx) / det, (g[4] * dx - g[3] * dy) / det) / (2.0 * M_PI);
        if (t < 0.0) t += 1.0;
        *to++ = ramp[std::min(1023, (int)(t * 1024.0))];
    }
    held.push_back(pic);
    tvg::Picture* out = tvg::Picture::gen();
    out->load((const uint32_t*)pic->data(), W, H, tvg::ColorSpace::ABGR8888S, false);
    out->transform(tvg::Matrix{bw / W, 0.f, bx, 0.f, bh / H, by, 0.f, 0.f, 1.f});
    out->clip(shape);
    return out;
}
static void sketch_bitmaps(Drawn* d, Script* run, const std::vector<double>& o) {   // (the pictures a frame paints, while its script may be asked)
    for (size_t i = 1; i + 12 <= o.size(); i += 12 + (size_t)o[i + 9] + (size_t)o[i + 10]) {
        if (o[i] != 4.0) continue;
        const int id = (int)o[i + 18];
        std::lock_guard<std::mutex> hold(d->sketchPicsLock);
        if (d->sketchPics.count(id)) continue;
        auto pic = std::make_shared<std::vector<uint8_t>>(run->bytes("__bitmap(" + std::to_string(id) + ")"));
        if (d->sketchPics.size() >= 64) d->sketchPics.erase(d->sketchPics.begin());   // (a film's, one a frame: the oldest goes)
        d->sketchPics[id] = std::move(pic);
    }
}
static void sketch_draw(Drawn* d, const std::vector<double>& o) {
    if (o.empty()) return;
    d->sketchScene->remove();
    bool wipe = o[0] != 0.0;
    std::vector<std::shared_ptr<std::vector<uint8_t>>> pics;   // (ThorVG reads them where they are, until it has drawn)
    const auto out_now = [&] {   // (a painting gone back to from its start comes in lengths: Metal's command buffers hold so much and no more, or it stops the program)
        d->canvas->update();
        d->canvas->draw(wipe), d->stale = true, wipe = false;
        d->canvas->sync();
        d->sketchScene->remove();
        pics.clear();
    };
    int held = 0;
    for (size_t i = 1; i + 12 <= o.size();) {
        const double* p = &o[i];
        const size_t n = (size_t)p[9], more = (size_t)p[10];
        const double* q = p + 12, *end = q + n;
        tvg::Paint* out;
        if (p[0] == 3.0) out = sketch_words(d, p, q, end);
        else if (p[0] == 4.0) out = sketch_bitmap(d, p, q, pics);
        else {
            tvg::Shape* one = tvg::Shape::gen();
            while (q < end)
                switch ((int)*q) {
                    case 0: one->moveTo((float)q[1], (float)q[2]), q += 3; break;
                    case 1: one->lineTo((float)q[1], (float)q[2]), q += 3; break;
                    case 2: one->cubicTo((float)q[1], (float)q[2], (float)q[3], (float)q[4], (float)q[5], (float)q[6]), q += 7; break;
                    default: one->close(), q += 1;
                }
            const uint8_t r = (uint8_t)std::lround(p[1]), g = (uint8_t)std::lround(p[2]), b = (uint8_t)std::lround(p[3]), a = (uint8_t)std::lround(p[4]);
            const bool conic = more && end[0] == 3.0;
            if (p[0] == 1.0) {
                if (more && !conic) one->fill(sketch_ramp(end, more)); else one->fill(r, g, b, a);
            } else {
                one->strokeWidth((float)p[6]), one->strokeCap((tvg::StrokeCap)(int)p[7]), one->strokeJoin((tvg::StrokeJoin)(int)p[8]);
                if (more && !conic) one->strokeFill(sketch_ramp(end, more)); else one->strokeFill(r, g, b, a);
            }
            out = conic && p[0] == 1.0 ? sketch_conic(one, end, more, pics) : one;
        }
        if (out && p[5] != 0.0) {
            if (out->type() == tvg::Type::Picture) {   // (WgCompositor::renderImage reads no blend on a picture with a clip, and WgSolidBatch takes one with none: a scene's it does read)
                tvg::Scene* alone = tvg::Scene::gen();
                alone->add(out), out = alone;
            }
            out->blend((tvg::BlendMethod)(int)p[5]);
        }
        if (out) d->sketchScene->add(out);
        i += 12 + n + more;
        if (++held == 4000 || pics.size() >= 256) out_now(), held = 0;
    }
    out_now();
}
// As a film with no end in sight: its own 60 frames a second, as p5_stand.h's clock, ten minutes of them.
static Drawn* sketch_picture(Stage& s, std::shared_ptr<Script> run, const std::string& fontFile, float high, const char* lay, const DrawnFx& fx) {
    const int fw = run->width(), fh = run->height();
    if (fw <= 0 || fh <= 0) return nullptr;
    return drawn(s, [&](Drawn* d, int* w, int* h) -> tvg::Paint* {
        const float fit = (float)std::max(s.w, s.h) / (float)std::max(fw, fh);   // (as finely as the stage is, as picture() draws an SVG)
        *w = 2 * (int)std::lround(0.5f * (float)fw * fit), *h = 2 * (int)std::lround(0.5f * (float)fh * fit);
        if (tvg::Text::load(fontFile.c_str()) == tvg::Result::Success) d->sketchFont = std::filesystem::path(fontFile).stem().string();   // (a sketch's font, any: the one Filament brings)
        d->sketchScene = tvg::Scene::gen();
        d->sketchScene->scale(fit / run->density());   // (its paints come in its canvas's own pixels: pixelDensity(2) has twice as many as its width says)
        d->sketchLines = std::make_shared<std::map<int, double>>();
        for (const int line : run->lines()) (*d->sketchLines)[line] = 1.0;
        d->sketch = [run, d, lines = d->sketchLines](int at, int had) {   // (each paint as strong as its line's knob is now, doc_moment)
            std::vector<double> o = run->paints(at, had);
            for (size_t i = 1; i + 12 <= o.size();) {
                double* p = &o[i];
                const size_t n = (size_t)p[9], more = (size_t)p[10];
                if (const auto k = lines->find((int)p[11]); k != lines->end() && k->second < 1.0) {
                    p[4] *= k->second;
                    for (size_t stop = 7; stop + 5 <= more; stop += 5) p[12 + n + stop + 4] *= k->second;
                }
                i += 12 + n + more;
            }
            sketch_bitmaps(d, run.get(), o);
            return o;
        };
        d->frames = 36000.f, d->seconds = 600.f;
        return d->sketchScene;
    }, mat4f{}, high, float3{-1.f}, true, lay, fx);
}
// A picture a WEBGL sketch draws with (a createGraphics): the same canvas at its own size, shown only as sketch.mat reads it (doc_sketch.h).
static Drawn* sketch_texture(Stage& s, Script* run, const std::string& fontFile, int pic, int wide, int high) {
    Drawn* made = drawn(s, [&](Drawn* d, int* w, int* h) -> tvg::Paint* {
        *w = wide, *h = high;
        if (tvg::Text::load(fontFile.c_str()) == tvg::Result::Success) d->sketchFont = std::filesystem::path(fontFile).stem().string();
        d->sketchScene = tvg::Scene::gen();
        d->sketch = [run, d, pic](int at, int had) { std::vector<double> o = run->paints(at, had, pic); sketch_bitmaps(d, run, o); return o; };
        d->frames = 36000.f, d->seconds = 600.f;
        return d->sketchScene;
    }, mat4f{}, 1.f, float3{-1.f}, false, nullptr);
    if (made) s.playing.erase(made);
    return made;
}
