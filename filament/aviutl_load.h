// aviutl_load.h: part of aviutl_fx.h. obj.load as lua.txt says it, for a figure, a picture file (and obj.copybuffer's "image:") and words (the AviUtl2 script
// skill's references/api.md, gasdyueer, MIT: "figure", name[, color, size, line, round]; "image", file, true or false).
// Drawn by ThorVG's own CPU canvas (SwCanvas, in Motolii's ThorVG), its premultiplied pixels laid in the object as
// they are, kept by what was asked (the same asked again is not drawn again).
// The figures as sashimono-edit draws them (MIT, engine/sources.py _shape_path, _polygon_path, _star_path; objapi.py
// FIGURE_NAMES, MAX_FIGURE_SIZE 4096, FILLED_LINE 4000): a square picture `size` wide (背景 the screen), polygons and
// the star with a corner up, inside a circle of it (sashimono's measure of AviUtl2: a triangle of 400 at x ±173,
// y -200..+99); a line drawn inward, its outer edge the figure's own (_inside_outline: the stroke cut by the figure);
// a line as thick as the figure or 4000, filled.

#ifdef __APPLE__
#include <CoreText/CoreText.h>   // (a font found by its name: aviutl_font_file)
#endif

static const std::map<std::string, int> aviutl_figures ={{"円", 0}, {"四角形", 4}, {"三角形", 3}, {"五角形", 5}, {"六角形", 6}, {"星形", -5}, {"背景", 1}};

// Its pixels, as a picture of the run's own, kept by `key`; made by `paint` into a canvas w x h (none: false).
static const AviutlBuf* aviutl_drawn(AviutlRun& r, const std::string& key, uint32_t w, uint32_t h, const std::function<bool(tvg::SwCanvas*)>& paint) {
    if (const auto kept = r.loaded.find(key); kept != r.loaded.end()) return &kept->second;
    if (!w || !h || w > 4096 || h > 4096) return nullptr;
    auto* pixels = new std::vector<uint32_t>((size_t)w * h, 0);
    tvg::SwCanvas* canvas = tvg::SwCanvas::gen();
    const bool drew = canvas->target(pixels->data(), w, w, h, tvg::ColorSpace::ABGR8888) == tvg::Result::Success && paint(canvas) && canvas->draw(true) == tvg::Result::Success && canvas->sync() == tvg::Result::Success;
    delete canvas;
    if (!drew) return delete pixels, nullptr;
    AviutlBuf& b = r.loaded[key];
    b.w = w, b.h = h;
    b.t = Texture::Builder().width(w).height(h).levels(1).format(Texture::InternalFormat::RGBA8).build(*r.s->engine);
    b.t->setImage(*r.s->engine, 0, backend::PixelBufferDescriptor(pixels->data(), pixels->size() * 4, backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE,
        [](void*, size_t, void* u) { delete (std::vector<uint32_t>*)u; }, pixels));
    return &b;
}

static tvg::Shape* aviutl_figure(int corners, float w, float h) {
    tvg::Shape* s = tvg::Shape::gen();
    if (corners == 0) return s->appendCircle(w / 2, h / 2, w / 2, h / 2), s;
    if (corners == 4 || corners == 1) return s->appendRect(0, 0, w, h), s;
    const int n = corners < 0 ? -2 * corners : corners;
    for (int i = 0; i < n; ++i) {
        const double a = (corners < 0 ? M_PI * i / -corners : 2 * M_PI * i / corners) - M_PI / 2, k = corners < 0 && i % 2 ? 0.382 : 1.0;
        const float x = (float)(w / 2 + std::cos(a) * w / 2 * k), y = (float)(h / 2 + std::sin(a) * h / 2 * k);
        i ? s->lineTo(x, y) : s->moveTo(x, y);
    }
    return s->close(), s;
}

// A picture file, read by ThorVG and kept (none chosen, not there or not read: none, false to the script, as lua.txt says).
static const AviutlBuf* aviutl_image(AviutlRun& r, std::string file) {
    if (!file.empty() && file[0] != '/') file = (std::filesystem::path(r.file).parent_path() / file).string();   // (a name of its own beside the script)
    std::error_code wrong;
    if (file.empty() || !std::filesystem::is_regular_file(file, wrong)) return nullptr;
    float w = 0, h = 0;
    tvg::Picture* p = tvg::Picture::gen();
    const bool read = p->load(file.c_str()) == tvg::Result::Success && p->size(&w, &h) == tvg::Result::Success && w >= 1 && h >= 1;
    delete p;
    if (!read) return nullptr;
    const auto at = std::filesystem::last_write_time(file, wrong);
    return aviutl_drawn(r, "image|" + file + "|" + std::to_string((long long)at.time_since_epoch().count()), (uint32_t)std::lround(w), (uint32_t)std::lround(h), [&](tvg::SwCanvas* c) {
        tvg::Picture* q = tvg::Picture::gen();
        return q->load(file.c_str()) == tvg::Result::Success && c->add(q) == tvg::Result::Success;
    });
}

// The file of a font asked by its name, as the Mac finds it (CoreText: the family, a face's own name or its shown
// name, its bold and italic face if it has them); not found, or a file ThorVG does not read (a collection, .ttc): the
// window's own for new words (ws_2.dart's wsTextFont). `slant`: italic asked of a face that is not, to be slanted.
static std::string aviutl_font_file(const std::string& name, bool bold, bool italic, bool* slant) {
    std::string file;
    bool sloped = false;
#ifdef __APPLE__
    CFStringRef said = name.empty() ? nullptr : CFStringCreateWithCString(nullptr, name.c_str(), kCFStringEncodingUTF8);
    for (CFStringRef by : {kCTFontFamilyNameAttribute, kCTFontNameAttribute, kCTFontDisplayNameAttribute}) {
        if (!said || !file.empty()) break;
        const void* keys[] = {by};
        const void* values[] = {said};
        CFDictionaryRef asked = CFDictionaryCreate(nullptr, keys, values, 1, &kCFTypeDictionaryKeyCallBacks, &kCFTypeDictionaryValueCallBacks);
        CFSetRef must = CFSetCreate(nullptr, keys, 1, &kCFTypeSetCallBacks);
        CTFontDescriptorRef wanted = CTFontDescriptorCreateWithAttributes(asked);
        if (CTFontDescriptorRef found = CTFontDescriptorCreateMatchingFontDescriptor(wanted, must)) {
            CTFontRef font = CTFontCreateWithFontDescriptor(found, 12, nullptr);
            const CTFontSymbolicTraits traits = (bold ? kCTFontBoldTrait : 0) | (italic ? kCTFontItalicTrait : 0);
            if (CTFontRef faced = traits ? CTFontCreateCopyWithSymbolicTraits(font, 12, nullptr, traits, traits) : nullptr) CFRelease(font), font = faced;
            sloped = italic && !(CTFontGetSymbolicTraits(font) & kCTFontItalicTrait);
            if (CFURLRef url = (CFURLRef)CTFontCopyAttribute(font, kCTFontURLAttribute)) {
                char path[4096];
                if (CFURLGetFileSystemRepresentation(url, true, (UInt8*)path, sizeof path)) file = path;
                CFRelease(url);
            }
            CFRelease(font), CFRelease(found);
        }
        CFRelease(wanted), CFRelease(must), CFRelease(asked);
    }
    if (said) CFRelease(said);
#endif
    const std::string ext = std::filesystem::path(file).extension().string();
    if (file.empty() || (ext != ".ttf" && ext != ".otf" && ext != ".TTF" && ext != ".OTF") || tvg::Text::load(file.c_str()) != tvg::Result::Success) {
#ifdef _WIN32
        file = "C:/Windows/Fonts/arial.ttf";
#else
        file = "/System/Library/Fonts/Supplemental/Arial Unicode.ttf";
#endif
        sloped = italic;
    }
    return *slant = sloped, file;
}

// obj.load "text": the words in obj.setfont's font, size, colour and decoration, a picture as large as they are set
// (YMM4-LuaScript, routersys, MIT: TextRenderer.Render, the layout's own box; nothing said, one clear pixel). Set by
// ThorVG's text as Motolii's words are (drawn_made.h type: its size in points, AviUtl's in pixels, so 72 / 96 of it).
// The decorations as sashimono-edit measured AviUtl2 v2.1.6a (MIT, compat/decoration.py DECORATIONS, by obj.setfont's
// number in its order: mapping.py _DECORATION_BY_INDEX): an edge of 8, 4, 12 and 4 hundredths of the size outside the
// letters (縁取り, 細, 太, 角), a shadow 5 hundredths right and down (影付き; 薄, half as strong), in the edge colour.
static const AviutlBuf* aviutl_text(AviutlRun& r, const std::string& words) {
    const auto& f = r.font;
    const double size = std::clamp(f.size, 1.0, 4096.0);
    const int deco = (int)f.deco, colour = (int)f.colour, edge = (int)f.edge;
    static const double edges[] = {0, 0, 0, 0.08, 0.04, 0.12, 0.04};
    const float out = (float)(size * (deco >= 0 && deco <= 6 ? edges[deco] : 0)), down = deco == 1 || deco == 2 ? (float)(0.05 * size) : 0.f;
    const std::string key = "text|" + f.name + "|" + std::to_string(size) + "|" + std::to_string(deco) + "|" + std::to_string(colour) + "|" + std::to_string(edge) + "|" + std::to_string(f.bold) + std::to_string(f.italic) + "|" + words;
    r.worded.insert(key);
    if (const auto kept = r.loaded.find(key); kept != r.loaded.end()) return &kept->second;
    bool slant = false;
    const std::string file = aviutl_font_file(f.name, f.bold, f.italic, &slant);
    if (tvg::Text::load(file.c_str()) != tvg::Result::Success) return nullptr;
    const auto set = [&](int c, uint8_t a) {
        tvg::Text* t = tvg::Text::gen();
        t->font(std::filesystem::path(file).stem().string().c_str());   // (ThorVG names a font by its file)
        t->size((float)(size * 72.0 / 96.0));
        t->text(words.c_str());
        if (slant) t->italic();
        t->fill((c >> 16) & 255, (c >> 8) & 255, c & 255);
        t->opacity(a);
        return t;
    };
    tvg::Scene* all = tvg::Scene::gen();
    if (down > 0.f) {
        tvg::Text* shadow = set(edge, deco == 2 ? 128 : 255);
        shadow->translate(down, down);
        all->add(shadow);
    }
    if (out > 0.f) {
        tvg::Text* rim = set(edge, 255);
        rim->outline(2.f * out, (edge >> 16) & 255, (edge >> 8) & 255, edge & 255);   // (twice as wide: its half within the letters under them)
        all->add(rim);
    }
    all->add(set(colour, 255));
    float x = 0, y = 0, w = 0, h = 0;
    if (all->bounds(&x, &y, &w, &h) != tvg::Result::Success || w <= 0.f || h <= 0.f) return tvg::Paint::rel(all), nullptr;
    all->translate(-x, -y);
    bool given = false;
    const AviutlBuf* made = aviutl_drawn(r, key, (uint32_t)std::ceil(w), (uint32_t)std::ceil(h), [&](tvg::SwCanvas* c) { return given = true, c->add(all) == tvg::Result::Success; });
    if (!given) tvg::Paint::rel(all);
    return made;
}

static int aviutl_load(lua_State* L) {
    AviutlRun& r = aviutl_of(L);
    const std::string kind = aviutl_str(L, 1);
    if (kind == "tempbuffer" && lua_gettop(L) == 1 && r.bufs.count(kind)) return aviutl_copy(r, "object", kind), lua_pushboolean(L, 1), 1;
    const auto lay = [&](const AviutlBuf* from) {
        if (!from) return lua_pushboolean(L, 0), 1;
        const AviutlBuf& to = aviutl_buf(r, "object", from->w, from->h);
        aviutl_lay(r, *from, to.rt, to.w, to.h, 0, 0, 1, 1, 0, 1, 0, 0, -1, 1);
        return lua_pushboolean(L, 1), 1;
    };
    if (kind == "figure") {
        const std::string name = aviutl_str(L, 2);
        const auto f = aviutl_figures.find(name);
        if (f == aviutl_figures.end()) return aviutl_no(L, "obj.load figure " + name);
        const int colour = (int)aviutl_num(L, 3, 0xffffff), size = (int)std::clamp(aviutl_num(L, 4, 100), 1.0, 4096.0);
        const uint32_t w = f->second == 1 ? (uint32_t)r.s->w : (uint32_t)size, h = f->second == 1 ? (uint32_t)r.s->h : (uint32_t)size;
        double line = aviutl_num(L, 5, 0);
        if (line >= std::min(4000.0, (double)std::max(w, h))) line = 0;
        if (lua_gettop(L) >= 6 && aviutl_num(L, 6) != 0) return aviutl_no(L, "obj.load figure with round corners");
        if (f->second == 4 && line <= 0) {   // (a square filled: the buffer cleared to it)
            aviutl_clear(r, aviutl_buf(r, "object", w, h), colour, true);
            return lua_pushboolean(L, 1), 1;
        }
        const uint8_t cr = (colour >> 16) & 255, cg = (colour >> 8) & 255, cb = colour & 255;
        return lay(aviutl_drawn(r, "figure|" + name + "|" + std::to_string(colour) + "|" + std::to_string(w) + "x" + std::to_string(h) + "|" + std::to_string(line), w, h, [&](tvg::SwCanvas* c) {
            tvg::Shape* s = aviutl_figure(f->second, (float)w, (float)h);
            if (line <= 0) return s->fill(cr, cg, cb), c->add(s) == tvg::Result::Success;
            s->fill(0, 0, 0, 0), s->strokeWidth((float)(2 * line)), s->strokeFill(cr, cg, cb);   // (twice as wide, the half outside cut away by the figure)
            s->clip(aviutl_figure(f->second, (float)w, (float)h));
            return c->add(s) == tvg::Result::Success;
        }));
    }
    if (kind == "image") return lay(aviutl_image(r, aviutl_str(L, 2)));
    if (kind == "text") {
        const std::string words = lua_isstring(L, 2) ? lua_tostring(L, 2) : "";
        if (words.empty()) return aviutl_clear(r, aviutl_buf(r, "object", 1, 1), 0, false), lua_pushboolean(L, 1), 1;
        return lay(aviutl_text(r, words));
    }
    return aviutl_no(L, "obj.load " + kind + (kind == "figure" ? " " + aviutl_str(L, 2) : ""));
}
