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
static void sketch_draw(Drawn* d, const std::vector<double>& o) {
    if (o.empty()) return;
    d->sketchScene->remove();
    bool wipe = o[0] != 0.0;
    const auto out_now = [&] {   // (a painting gone back to from its start comes in lengths: Metal's command buffers hold so much and no more, or it stops the program)
        d->canvas->update();
        d->canvas->draw(wipe), d->stale = true, wipe = false;
        d->canvas->sync();
        d->sketchScene->remove();
    };
    int held = 0;
    for (size_t i = 1; i + 12 <= o.size();) {
        const double* p = &o[i];
        const size_t n = (size_t)p[9], more = (size_t)p[10];
        const double* q = p + 12, *end = q + n;
        tvg::Paint* out;
        if (p[0] == 3.0) out = sketch_words(d, p, q, end);
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
            if (p[0] == 1.0) {
                if (more) one->fill(sketch_ramp(end, more)); else one->fill(r, g, b, a);
            } else {
                one->strokeWidth((float)p[6]), one->strokeCap((tvg::StrokeCap)(int)p[7]), one->strokeJoin((tvg::StrokeJoin)(int)p[8]);
                if (more) one->strokeFill(sketch_ramp(end, more)); else one->strokeFill(r, g, b, a);
            }
            out = one;
        }
        if (p[5] != 0.0) out->blend((tvg::BlendMethod)(int)p[5]);
        d->sketchScene->add(out);
        i += 12 + n + more;
        if (++held == 4000) out_now(), held = 0;
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
        d->sketchScene->scale(fit);
        d->sketchLines = std::make_shared<std::map<int, double>>();
        for (const int line : run->lines()) (*d->sketchLines)[line] = 1.0;
        d->sketch = [run, lines = d->sketchLines](int at, int had) {   // (each paint as strong as its line's knob is now, doc_moment)
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
            return o;
        };
        d->frames = 36000.f, d->seconds = 600.f;
        return d->sketchScene;
    }, mat4f{}, high, float3{-1.f}, true, lay, fx);
}
