// drawn_made.h: part of drawn.h. Read only through drawn.h.
// What `picture` and `type` share: something of ThorVG's (made by `paints` on Filament's own thread, which says how
// many points wide and high it is drawn) on a canvas that draws into a texture of Filament's device, and the sheet
// that shows it. ground below 0: clear where nothing is drawn. shown = false: made but not put in the scene (for a matte).
// What ThorVG's own scene effects do to it, each as a share of its height (0: none): a blur; a shadow of a colour, so
// much of it (shadow.w), falling so far, that way (degrees, as ThorVG counts them) and so soft.
   // (copies: how many times it is put down: the document tells each where)
static Drawn* drawn(Stage& s, const std::function<tvg::Paint*(Drawn*, int*, int*)>& paints, const mat4f& place, float high, float3 ground, bool shown, const char* lay, const DrawnFx& fx = {}) {
    if (!door::device(s.engine)) return nullptr;   // (another backend)
    Drawn* d = new Drawn;
    WGPUTexture made = nullptr;
    tvg::Paint* loose = nullptr;
    bool owned = false;
    int w = 0, h = 0;
    float own = high;
    door::on_driver(s.engine, [&] {
        tvg::Initializer::init(0);
        tvg::Paint* p = paints(d, &w, &h);
        if (!p || w <= 0 || h <= 0) return;
        high *= d->tallBy, own = high;
        const float tall = (float)h;   // (what its effects' lengths are shares of: the thing as drawn, without room)
        if (fx.reach > 0.f) {
            const float room = std::ceil(fx.reach * (float)h);
            p->translate(room, room);
            high *= ((float)h + 2.f * room) / (float)h;   // (the thing itself stays as large)
            w += 2 * (int)room, h += 2 * (int)room;
        }
        if (fx.blur > 0.f || fx.shadow.w > 0.f || fx.tint > 0.f || fx.tritone > 0.f) {   // (in a scene of ThorVG's, which takes the effects; the picture larger all round by as far as they reach)
            const float blur = fx.blur * tall, fall = fx.fall * tall, soft = fx.soft * tall;
            const int room = (int)std::ceil(std::max({3.f * blur, fx.shadow.w > 0.f ? fall + 3.f * soft : 0.f, fx.room * tall}));
            tvg::Scene* scene = tvg::Scene::gen();   // (one scene takes them all, in order)
            scene->add(p);
            d->effects.push_back(scene);
            p = scene;
            p->translate((float)room, (float)room);
            effects_on(d->effects, fx, tall);
            d->tall = tall, d->fx = fx;   // (told again at each moment: those that move are drawn again)
            high *= (float)(h + 2 * room) / (float)h;   // (the thing itself stays as large)
            w += 2 * room, h += 2 * room;
        }
        d->pictureHigh = (float)h;
        if (const float most = (float)std::max(w, h); most > 4096.f) {   // (no larger than a picture the device takes, with room to spare: the whole drawn smaller, its sheet as large)
            tvg::Scene* smaller = tvg::Scene::gen();
            smaller->add(p);
            smaller->scale(4096.f / most);
            p = smaller;
            w = std::max(2, (int)std::floor((float)w * 4096.f / most)), h = std::max(2, (int)std::floor((float)h * 4096.f / most));
        }
        loose = d->film ? nullptr : p;   // (a paint that no film owns is let go here if the canvas never takes it)
        WGPUTextureDescriptor like{};
        like.size = {(uint32_t)w, (uint32_t)h, 1};
        like.format = WGPUTextureFormat_BGRA8Unorm;   // (what ThorVG's last step writes)
        like.usage = WGPUTextureUsage_RenderAttachment | WGPUTextureUsage_TextureBinding | WGPUTextureUsage_CopySrc | WGPUTextureUsage_CopyDst;   // (CopyDst: a frame kept on disk is put up in it)
        like.dimension = WGPUTextureDimension_2D;
        like.mipLevelCount = 1;
        like.sampleCount = 1;
        made = wgpuDeviceCreateTexture((WGPUDevice)door::device(s.engine), &like);
        d->made = made;
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
        if (d->film && d->toFrame > d->fromFrame && d->frames > 0.f) d->seconds *= (d->toFrame - d->fromFrame) / d->frames, d->frames = d->toFrame - d->fromFrame;   // (a named stretch of it: so long)
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
    if (fx.thick) sheet_deep(d->sheet, 0.1f * own), d->boxed = float3{d->sheet->pos[1].x, d->sheet->pos[2].y, 0.1f * own};   // (a fifth of its height through: the depth Raster_Extrude.js gives at Scale Z 1)
    if (d->frames > 1.f)   // (a film: it goes on by itself as the stage's time does, round and round)
        s.playing[d] = [&s, d](double now) { picture_at(s, d, std::fmod((float)now + d->starts, d->seconds)); };
    if (!shown) return d;
    d->look = add(s, d->sheet, flat_laid(lay).c_str(), std::vector<mat4f>((size_t)std::max(1, d->letters.empty() ? fx.copies : (int)d->letters.size()), place), 1, false, 1, 4, fx.together && d->letters.empty());   // (split into letters: a copy each)
    if (!d->look) return nullptr;
    d->look->setParameter("picture", d->texture, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
    if (fx.thick) d->look->setParameter("deep", float4{d->boxed, 0.f}), d->look->setParameter("deepTurn", float4{1.f, 0.f, 0.f, 1.f});
    return d;
}


// The picture in `file` put in the scene (an After Effects film, an SVG, a photo): a sheet `high` tall (as wide as
// the picture makes it) with its middle at `place`, facing along the place's z, drawn as finely as the stage is.
// The frame an After Effects file begins at (its own "ip", at the top of the file: ThorVG does not say it).
static float lottie_in(const char* file) {
    std::ifstream in(file);
    std::string all((std::istreambuf_iterator<char>(in)), std::istreambuf_iterator<char>());
    int depth = 0;
    bool quoted = false;
    for (size_t i = 0; i < all.size(); ++i) {   // (the key at the outermost depth only: the layers have theirs)
        const char c = all[i];
        if (quoted) { if (c == '\\') ++i; else if (c == '"') quoted = false; continue; }
        if (c == '"') {
            if (depth == 1 && all.compare(i, 4, "\"ip\"") == 0) {
                const size_t colon = all.find(':', i + 4);
                return colon == std::string::npos ? 0.f : std::strtof(all.c_str() + colon + 1, nullptr);
            }
            quoted = true;
        } else if (c == '{' || c == '[') ++depth;
        else if (c == '}' || c == ']') --depth;
    }
    return 0.f;
}

// marker: a named stretch of an After Effects file it plays, over and over (ThorVG's Lottie markers; none: all of it).
static Drawn* picture(Stage& s, const char* file, const mat4f& place, float high, float3 ground = float3{-1.f}, bool shown = true, const char* lay = nullptr, const DrawnFx& fx = {}, const char* marker = "") {
    if (!std::filesystem::is_regular_file(file)) return nullptr;   // (ThorVG ends the program on a folder)
    const std::string ext = std::filesystem::path(file).extension().string();
    const bool lottie = ext == ".json" || ext == ".lot";
    Drawn* made = drawn(s, [&](Drawn* d, int* w, int* h) -> tvg::Paint* {
        tvg::LottieAnimation* own = lottie ? tvg::LottieAnimation::gen() : nullptr;
        d->film = own ? own : tvg::Animation::gen();
        d->lottie = own;
        tvg::Picture* p = d->film->picture();
        float fw = 0.f, fh = 0.f;
        if (p->load(file) != tvg::Result::Success || p->size(&fw, &fh) != tvg::Result::Success || fw <= 0.f || fh <= 0.f) return nullptr;
        const float first = lottie ? lottie_in(file) : 0.f;
        std::vector<float> starts;   // (where each named stretch begins: one of no length runs to the next, or to the end)
        for (uint32_t i = 0; own && i < own->markersCnt(); ++i) {
            float from = 0.f, to = 0.f;
            if (own->marker(i, &from, &to)) starts.push_back(from);
        }
        for (uint32_t i = 0; own && i < own->markersCnt(); ++i) {   // (its stretches by name, and the one it plays)
            float from = 0.f, to = 0.f;
            const char* name = own->marker(i, &from, &to);
            if (!name || !*name) continue;
            std::string word(name);
            word.erase(std::remove_if(word.begin(), word.end(), [](char c) { return std::isspace((unsigned char)c); }), word.end());   // (a name is one word: "End\r\n" is End)
            d->markers.push_back(word);
            if (to <= from) {   // (none long: to the next, or to its end)
                to = 1e9f;
                for (float b : starts) if (b > from) to = std::min(to, b);
            }
            if (marker && *marker && word == marker)   // (a marker counts the file's own frames, which begin at its in point; ThorVG counts from 0)
                d->fromFrame = from - first, d->toFrame = std::min(to - first, d->film->totalFrame());
        }
        const float fit = (float)std::max(s.w, s.h) / std::max(fw, fh);
        *w = 2 * (int)std::lround(0.5f * fw * fit);
        *h = 2 * (int)std::lround(0.5f * fh * fit);
        p->size((float)*w, (float)*h);
        return p;
    }, place, high, ground, shown, lay, fx);
    if (made && made->frames > 1.f) {   // (its kept frames known on disk by its file as it is now: kept_file)
        std::error_code e;
        made->keyBase = std::filesystem::absolute(file, e).string() + "|" + std::to_string(std::filesystem::file_size(file, e)) + "|" +
                        std::to_string((long long)std::filesystem::last_write_time(file, e).time_since_epoch().count());
    }
    return made;
}

// Words put in the scene, set by ThorVG in the font of `fontFile` (a .ttf): a sheet `high` tall for each line, as
// wide as the words make it, its middle at `place`.
// How the colours between a ramp's stops are mixed, as CSS Color 4's color-interpolation-method: in sRGB, as ThorVG
// mixes them (0); in OKLab, as an eye mixes them (1: CSS Color 4's sample code for its conversions, Björn Ottosson's
// OKLab); round the hue the shorter way, saturation and value straight (2: CSS Color 4, hue interpolation, shorter);
// or not at all, each colour held till the next (3: Blender's ColorRamp, Constant). A mixed way is laid down as more
// stops between each two, which ThorVG then mixes plainly.
static float ramp_lin(float c) { return c <= 0.04045f ? c / 12.92f : std::pow((c + 0.055f) / 1.055f, 2.4f); }
static float ramp_gam(float c) { return c <= 0.0031308f ? 12.92f * c : 1.055f * std::pow(c, 1.f / 2.4f) - 0.055f; }
static float3 ramp_oklab(float3 c) {
    const float r = ramp_lin(c.x), g = ramp_lin(c.y), b = ramp_lin(c.z);
    const float l = std::cbrt(0.4122214708f * r + 0.5363325363f * g + 0.0514459929f * b), m = std::cbrt(0.2119034982f * r + 0.6806995451f * g + 0.1073969566f * b), s = std::cbrt(0.0883024619f * r + 0.2817188376f * g + 0.6299787005f * b);
    return {0.2104542553f * l + 0.7936177850f * m - 0.0040720468f * s, 1.9779984951f * l - 2.4285922050f * m + 0.4505937099f * s, 0.0259040371f * l + 0.7827717662f * m - 0.8086757660f * s};
}
static float3 ramp_srgb(float3 o) {
    const float l = o.x + 0.3963377774f * o.y + 0.2158037573f * o.z, m = o.x - 0.1055613458f * o.y - 0.0638541728f * o.z, s = o.x - 0.0894841775f * o.y - 1.2914855480f * o.z;
    const float L = l * l * l, M = m * m * m, S = s * s * s;
    return {std::clamp(ramp_gam(4.0767416621f * L - 3.3077115913f * M + 0.2309699292f * S), 0.f, 1.f), std::clamp(ramp_gam(-1.2684380046f * L + 2.6097574011f * M - 0.3413193965f * S), 0.f, 1.f), std::clamp(ramp_gam(-0.0041960863f * L - 0.7034186147f * M + 1.7076147010f * S), 0.f, 1.f)};
}
static float3 ramp_hsv(float3 c) {
    const float mx = std::max({c.x, c.y, c.z}), mn = std::min({c.x, c.y, c.z}), d = mx - mn;
    float h = 0.f;
    if (d > 0.f) h = mx == c.x ? std::fmod((c.y - c.z) / d, 6.f) : mx == c.y ? (c.z - c.x) / d + 2.f : (c.x - c.y) / d + 4.f;
    return {std::fmod(h * 60.f + 360.f, 360.f), mx > 0.f ? d / mx : 0.f, mx};
}
static float3 ramp_rgb(float3 h) {
    const float c = h.z * h.y, x = c * (1.f - std::fabs(std::fmod(h.x / 60.f, 2.f) - 1.f)), m = h.z - c;
    const int k = (int)(h.x / 60.f) % 6;
    const float3 p = k == 0 ? float3{c, x, 0} : k == 1 ? float3{x, c, 0} : k == 2 ? float3{0, c, x} : k == 3 ? float3{0, x, c} : k == 4 ? float3{x, 0, c} : float3{c, 0, x};
    return p + m;
}
static void ramp_blend(std::vector<float4>& ramp, std::vector<float>& at, int blend) {
    if (blend <= 0 || ramp.size() < 2) return;
    std::vector<float> pos = at.size() == ramp.size() ? at : std::vector<float>();
    if (pos.empty()) for (size_t i = 0; i < ramp.size(); ++i) pos.push_back((float)i / (float)(ramp.size() - 1));
    std::vector<float4> out;
    std::vector<float> outAt;
    constexpr int between = 8;
    for (size_t i = 0; i + 1 < ramp.size(); ++i) {
        const float4 a = ramp[i], b = ramp[i + 1];
        if (blend == 3) {   // (held till the next, which begins a hair after: ThorVG keeps one of two stops at one place)
            out.push_back(a), outAt.push_back(pos[i]), out.push_back(a), outAt.push_back(std::max(pos[i], pos[i + 1] - 1e-4f));
            continue;
        }
        for (int k = 0; k < between; ++k) {
            const float t = (float)k / between;
            float3 c;
            if (blend == 1) {
                c = ramp_srgb(ramp_oklab(a.xyz) * (1.f - t) + ramp_oklab(b.xyz) * t);
            } else {
                float3 p = ramp_hsv(a.xyz), q = ramp_hsv(b.xyz);
                float dh = q.x - p.x;
                if (dh > 180.f) dh -= 360.f; else if (dh < -180.f) dh += 360.f;   // (the shorter way round)
                c = ramp_rgb({std::fmod(p.x + dh * t + 360.f, 360.f), p.y + (q.y - p.y) * t, p.z + (q.z - p.z) * t});
            }
            out.push_back({c, a.w + (b.w - a.w) * t}), outAt.push_back(pos[i] + (pos[i + 1] - pos[i]) * t);
        }
    }
    out.push_back(ramp.back()), outAt.push_back(pos.back());
    ramp = out, at = outAt;
}

// The colours of a ramp as ThorVG's linear gradient, from the first at y0 to the last at y1 (evenly spaced, or where `at` says).
static tvg::Fill* ramp_of(const std::vector<float4>& given, float w, float h, bool radial = false, float angle = 0.f, float scale = 1.f, int spread = 0, const std::vector<float>& givenAt = {}, int blend = 0) {
    std::vector<float4> ramp = given;
    std::vector<float> at = givenAt;
    ramp_blend(ramp, at, blend);
    tvg::Fill* g = nullptr;
    const float cx = 0.5f * w, cy = 0.5f * h, k = std::max(scale, 0.01f);
    if (radial) {   // (from its middle out to its corners)
        tvg::RadialGradient* r = tvg::RadialGradient::gen();
        const float reach = 0.5f * std::hypot(w, h) * k;
        r->radial(cx, cy, reach, cx, cy, 0.f);
        g = r;
    } else {   // (across it the way it is turned: 0 from its top to its bottom, as far as its box reaches that way)
        tvg::LinearGradient* l = tvg::LinearGradient::gen();
        const float a = angle * (float)TAU / 360.f, dx = std::sin(a), dy = std::cos(a), reach = (0.5f * std::fabs(w * dx) + 0.5f * std::fabs(h * dy)) * k;
        l->linear(cx - dx * reach, cy - dy * reach, cx + dx * reach, cy + dy * reach);
        g = l;
    }
    g->spread((tvg::FillSpread)std::clamp(spread, 0, 2));
    std::vector<tvg::Fill::ColorStop> stops;
    for (size_t i = 0; i < ramp.size(); ++i)
        stops.push_back({at.size() == ramp.size() ? at[i] : (float)i / (float)(ramp.size() - 1), (uint8_t)std::lround(255.f * ramp[i].x), (uint8_t)std::lround(255.f * ramp[i].y), (uint8_t)std::lround(255.f * ramp[i].z), (uint8_t)std::lround(255.f * ramp[i].w)});   // (a colour's own see-through: eight digits)
    g->colorStops(stops.data(), (uint32_t)stops.size());
    return g;
}

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
    tvg::Text::unload(fontFile);   // (ThorVG counts each load: a font only asked about leaves memory, 543 MB over the shelf; one in use stays)
    return sets;
}
// edge: an outline round the letters, ThorVG's own (its colour; w: how wide, a share of the line's height; 0: none).
// Lines: the words may be broken by \n; align: how the lines lie one under another (ThorVG's own: 0 left, 1 right).
static Drawn* type(Stage& s, const char* words, const char* fontFile, const mat4f& place, float high, float3 colour = float3{0.f}, const char* lay = nullptr, float4 edge = float4{0.f}, float align = 0.f, const DrawnFx& fx = {}) {
    if (!std::filesystem::is_regular_file(fontFile)) return nullptr;
    return drawn(s, [&](Drawn* d, int* w, int* h) -> tvg::Paint* {
        if (tvg::Text::load(fontFile) != tvg::Result::Success) return nullptr;
        tvg::Text* t = tvg::Text::gen();
        float x = 0.f, y = 0.f, wide = 0.f, tall = 0.f;
        const float fine = 0.12f * (float)std::max(s.w, s.h);   // (letters this many points tall: a line as fine as an eighth of the stage)
        t->font(std::filesystem::path(fontFile).stem().string().c_str());   // (ThorVG names a font by its file)
        float sized = fine;   // (as large as it is set: ThorVG does not say it back)
        t->size(sized);
        t->text(words);
        t->spacing(fx.letters, fx.lines);
        if (fx.italic > 0.f) t->italic(fx.italic);
        if (fx.wrapWidth > 0.f) t->layout(fx.wrapWidth * fine, 0.f), t->wrap((tvg::TextWrap)std::clamp(fx.wrap, 0, 4)), t->align(align, 0.f);   // (in a box: broken into lines there, ThorVG's own)
        t->fill((uint8_t)std::lround(255.f * colour.x), (uint8_t)std::lround(255.f * colour.y), (uint8_t)std::lround(255.f * colour.z));
        bool measured = t->bounds(&x, &y, &wide, &tall) == tvg::Result::Success && wide > 0.f && tall > 0.f;
        if (measured && wide > 4096.f) {   // (a line too long for one picture is set smaller)
            t->size(sized = fine * 4096.f / wide);
            measured = t->bounds(&x, &y, &wide, &tall) == tvg::Result::Success;
        }
        if (!measured) return tvg::Paint::rel(t), nullptr;
        if (align > 0.f && fx.wrapWidth <= 0.f) {   // (the lines aligned in a box as wide as the widest of them: ThorVG's layout and align)
            t->layout(wide, 0.f);
            t->align(align, 0.f);
            measured = t->bounds(&x, &y, &wide, &tall) == tvg::Result::Success;
        }
        if (strchr(words, '\n') || fx.wrapWidth > 0.f) {   // (a size is a line's, as a font's: more lines, the sheet the taller; After Effects' and Alight Motion's Font Size)
            std::string one(words);
            one.erase(std::remove(one.begin(), one.end(), '\n'), one.end());
            tvg::Text* line = tvg::Text::gen();
            line->font(std::filesystem::path(fontFile).stem().string().c_str());
            line->size(sized);
            line->text(one.c_str());
            float lineX, lineY, lineWide, lineTall;
            if (line->bounds(&lineX, &lineY, &lineWide, &lineTall) == tvg::Result::Success && lineTall > 0.f) {
                tvg::TextMetrics font;
                if (fx.column && t->metrics(font) == tvg::Result::Success && font.advance > 0.f) t->spacing(fx.letters, fx.lines * lineTall / font.advance), t->bounds(&x, &y, &wide, &tall);
                d->tallBy = tall / lineTall;
            }
            tvg::Paint::rel(line);
        }
        const float outline = edge.w * tall;   // (half of it lies outside the letters: the picture is that much larger all round)
        if (outline > 0.f) t->outline(outline, (uint8_t)std::lround(255.f * edge.x), (uint8_t)std::lround(255.f * edge.y), (uint8_t)std::lround(255.f * edge.z));
        t->translate(-x + 0.5f * outline, -y + 0.5f * outline);
        *w = 2 * (int)std::ceil(0.5f * (wide + outline));
        *h = 2 * (int)std::ceil(0.5f * (tall + outline));
        if (!fx.split && fx.wrapWidth <= 0.f && align <= 0.f) d->text = t, d->words = words, d->textWide = (float)*w, d->textAt = -y + 0.5f * outline;   // (its words told again in place: words_at)
        std::string line(words);   // (a last line left empty, a Return after the words, is no line: its letters are the ones before)
        while (!line.empty() && (line.back() == '\n' || line.back() == '\r')) line.pop_back();
        if (fx.split && line.find('\n') == std::string::npos && fx.wrapWidth <= 0.f) {   // (one line split into its letters: where each ends, by how wide the words up to it are set)
            const auto across = [&](const std::string& some) {   // (how wide some letters are set, a share of the picture's width; and where their ink ends)
                tvg::Text* up = tvg::Text::gen();
                up->font(std::filesystem::path(fontFile).stem().string().c_str());
                up->size(sized);
                up->spacing(fx.letters, fx.lines);
                if (fx.italic > 0.f) up->italic(fx.italic);
                up->text(some.c_str());
                float ux = 0.f, uy = 0.f, uw = 0.f, uh = 0.f;
                const bool ok = up->bounds(&ux, &uy, &uw, &uh) == tvg::Result::Success;
                tvg::Paint::rel(up);
                return float2{ok ? (ux + uw - x + 0.5f * outline) / (float)*w : 0.f, ok ? (uw + outline) / (float)*w : 0.f};
            };
            std::vector<float2> inks;   // (each letter: where the words up to it end, and how wide it is alone)
            for (const char* at = line.c_str(); *at;) {
                const char* next = at + 1;
                while (*next && ((unsigned char)*next & 0xC0) == 0x80) ++next;   // (one letter, however many bytes)
                if (fx.splitWords) while (*next && *next != ' ') ++next;
                inks.push_back(float2{across(std::string(line.c_str(), next)).x, across(std::string(at, next)).y});
                at = next;
                if (fx.splitWords) while (*at == ' ') ++at;
            }
            float was = 0.f;   // (cut half way across the gap between a letter's ink and the next one's: no edge of a neighbour in either)
            for (size_t i = 0; i < inks.size(); ++i) {
                const float end = std::max(was, inks[i].x);
                const float cut = i + 1 < inks.size() ? std::max(end, 0.5f * (end + std::max(end, inks[i + 1].x - inks[i + 1].y))) : 1.f;
                d->letters.push_back(float2{was, cut});
                was = cut;
            }
        }
        if (fx.ramp.size() < 2) return t;
        // (a ramp: a rect as large as the words, filled with it, seen only where the letters are: ThorVG's own alpha
        //  mask; ThorVG counts a text's own gradient in the font's units, which are not to be had from outside)
        tvg::Shape* ramp = tvg::Shape::gen();
        ramp->appendRect(0.f, 0.f, (float)*w, (float)*h);
        ramp->fill(ramp_of(fx.ramp, (float)*w, (float)*h, fx.radial, fx.rampAngle, fx.rampScale, fx.spread, fx.rampAt, fx.rampBlend));
        ramp->mask(t, tvg::MaskMethod::Alpha);
        return ramp;
    }, place, high, float3{-1.f}, true, lay, fx);
}

// A flat shape's path, as `flat` draws it in a picture w by h points, set in by `in` (half its stroke): ThorVG's own
// rectangle, circle, polygon, star, line or wave; its corners rounded (a share of its shorter side), its lines and curves
// cut in pieces no longer than `fine` points (0: as they are), for what bends it as it is drawn. Kept as made in `d`.
bool sk_svg(const char* data, std::vector<tvg::PathCommand>& cmds, std::vector<tvg::Point>& pts);
int sk_ends(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, float from, float to, std::vector<float>& out);
static void flat_path(Drawn* d, tvg::Shape* shape, const char* form, int w, int h, float in, int points, float inner, float round, float fine) {
    const float cx = 0.5f * (float)w, cy = 0.5f * (float)h, rx = cx - in, ry = cy - in;
    const float thick = in > 0.f ? 2.f * in : 0.08f * (float)h;   // (a wave's line: its stroke, or a share of its height)
    if (form[0] == 'M' || form[0] == 'm') {   // (a path of its own, drawn by a pen: SVG's path data, in its height from its middle, down as SVG goes; Skia reads it)
        std::vector<tvg::PathCommand> c; std::vector<tvg::Point> p;
        if (sk_svg(form, c, p)) {
            for (tvg::Point& at : p) at = {cx + at.x * (float)h, cy + at.y * (float)h};
            shape->appendPath(c.data(), (uint32_t)c.size(), p.data(), (uint32_t)p.size());
        }
    }
    else if (!strcmp(form, "circle")) shape->appendCircle(cx, cy, rx, ry);
    else if (!strcmp(form, "wave")) {   // (a line waving across it: so many times along (points), so far up and down (inner, a share of its half height))
        const int cycles = std::clamp(points, 1, 100), n = 48 * cycles;
        const float amp = (cy - thick) * std::clamp(inner, 0.f, 1.f);
        for (int k = 0; k <= n; ++k) {
            const float x = thick + ((float)w - 2.f * thick) * (float)k / (float)n, y = cy + amp * std::sin(TAU * (float)cycles * (float)k / (float)n);
            k ? shape->lineTo(x, y) : shape->moveTo(x, y);
        }
    }
    else if (!strcmp(form, "spiral")) {   // (a line winding out from its middle, as Cavalry's spiral: so many turns (points), from `inner` of the way out)
        const float turns = (float)std::clamp(points, 1, 50), from = std::clamp(inner, 0.f, 0.95f);
        const int n = (int)(96.f * turns);
        for (int k = 0; k <= n; ++k) {
            const float u = (float)k / (float)n, a = -0.25f * TAU + TAU * turns * u, r = from + (1.f - from) * u;
            const float x = cx + (rx - thick) * r * std::cos(a), y = cy + (ry - thick) * r * std::sin(a);
            k ? shape->lineTo(x, y) : shape->moveTo(x, y);
        }
    }
    else if (!strcmp(form, "ring") || !strcmp(form, "arc")) {   // (a ring: its hole `inner` of the way out; an arc: so many degrees of it (points), `inner` thick, from straight up clockwise)
        const bool arc = !strcmp(form, "arc");
        const float hole = arc ? 1.f - std::clamp(inner, 0.02f, 1.f) : std::clamp(inner, 0.f, 0.98f), sweep = arc ? TAU * std::clamp((float)points, 1.f, 360.f) / 360.f : TAU;
        const int n = std::max(8, (int)(96.f * sweep / TAU));
        for (int k = 0; k <= n; ++k) {   // (round the outside, then back round the inside: the hole a contour of its own the other way)
            const float a = -0.25f * TAU + sweep * (float)k / (float)n;
            k ? shape->lineTo(cx + rx * std::cos(a), cy + ry * std::sin(a)) : shape->moveTo(cx + rx * std::cos(a), cy + ry * std::sin(a));
        }
        if (!arc) shape->close();
        if (hole > 0.f) {
            for (int k = n; k >= 0; --k) {
                const float a = -0.25f * TAU + sweep * (float)k / (float)n;
                (k == n && !arc) ? shape->moveTo(cx + rx * hole * std::cos(a), cy + ry * hole * std::sin(a)) : shape->lineTo(cx + rx * hole * std::cos(a), cy + ry * hole * std::sin(a));
            }
        }
        shape->close();
    }
    else if (!strcmp(form, "arrow")) {   // (pointing right: its shaft `inner` of its height thick, its head a third of its length)
        const float shaft = 0.5f * ry * std::clamp(inner, 0.05f, 1.f), head = (float)w - in - std::min(2.f * rx / 3.f, 1.2f * ry);
        const float pts[][2] = {{in, cy - shaft}, {head, cy - shaft}, {head, in}, {(float)w - in, cy}, {head, (float)h - in}, {head, cy + shaft}, {in, cy + shaft}};
        for (int k = 0; k < 7; ++k) k ? shape->lineTo(pts[k][0], pts[k][1]) : shape->moveTo(pts[k][0], pts[k][1]);
        shape->close();
    }
    else if (!strcmp(form, "chamfer")) {   // (a rect with its corners cut straight, as Cavalry's rectangle with chamfer: `inner` of its shorter half cut off each)
        const float c = std::clamp(inner, 0.f, 1.f) * std::min(rx, ry), x0 = in, y0 = in, x1 = (float)w - in, y1 = (float)h - in;
        const float pts[][2] = {{x0 + c, y0}, {x1 - c, y0}, {x1, y0 + c}, {x1, y1 - c}, {x1 - c, y1}, {x0 + c, y1}, {x0, y1 - c}, {x0, y0 + c}};
        for (int k = 0; k < 8; ++k) k ? shape->lineTo(pts[k][0], pts[k][1]) : shape->moveTo(pts[k][0], pts[k][1]);
        shape->close();
    }
    else if (!strcmp(form, "squircle")) {   // (a superellipse, |x|^p + |y|^p = 1: points its power, 2 an ellipse, higher squarer)
        const float p = std::clamp((float)points, 2.f, 40.f);
        for (int k = 0; k < 128; ++k) {
            const float a = TAU * (float)k / 128.f, c = std::cos(a), sn = std::sin(a);
            const float x = cx + rx * std::copysign(std::pow(std::fabs(c), 2.f / p), c), y = cy + ry * std::copysign(std::pow(std::fabs(sn), 2.f / p), sn);
            k ? shape->lineTo(x, y) : shape->moveTo(x, y);
        }
        shape->close();
    }
    else if (!strcmp(form, "cog")) {   // (a cogwheel: so many teeth (points), each `inner` of its radius deep, flat topped)
        const int teeth = std::clamp(points, 3, 100), n = 4 * teeth;
        const float deep = 1.f - std::clamp(inner, 0.02f, 0.9f);
        for (int k = 0; k < n; ++k) {   // (a tooth: rising, top, falling, bottom: four corners)
            const float a0 = -0.25f * TAU + TAU * (float)k / (float)n, r = (k % 4 == 1 || k % 4 == 2) ? 1.f : deep;
            k ? shape->lineTo(cx + rx * r * std::cos(a0), cy + ry * r * std::sin(a0)) : shape->moveTo(cx + rx * r * std::cos(a0), cy + ry * r * std::sin(a0));
        }
        shape->close();
    }
    else if (!strcmp(form, "polygon") || !strcmp(form, "star")) {
        const bool star = !strcmp(form, "star");
        const int n = std::clamp(points, 3, 100) * (star ? 2 : 1);
        for (int k = 0; k < n; ++k) {
            const float a = -0.5f * TAU / 2.f + TAU * (float)k / (float)n, r = star && (k & 1) ? std::clamp(inner, 0.01f, 1.f) : 1.f;   // (the first point straight up)
            k ? shape->lineTo(cx + rx * r * std::cos(a), cy + ry * r * std::sin(a)) : shape->moveTo(cx + rx * r * std::cos(a), cy + ry * r * std::sin(a));
        }
        shape->close();
    } else {
        const float side = std::min((float)w, (float)h) - 2.f * in, corner = !strcmp(form, "line") ? 0.5f * side : 0.f;   // (a rect's corners: round, bent as the others)
        shape->appendRect(in, in, (float)w - 2.f * in, (float)h - 2.f * in, corner, corner);
    }
    const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
    shape->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
    d->plain.assign(cmds, cmds + cmdsCnt), d->plainAt.assign(pts, pts + ptsCnt);
    d->plainHigh = (float)h, d->plainSide = std::min((float)w, (float)h) - 2.f * in;
    d->fine = fine;
    if ((round > 0.f && strcmp(form, "line")) || d->fine != 0.f)
        tvg_bend(shape, d->plain.data(), cmdsCnt, d->plainAt.data(), ptsCnt, strcmp(form, "line") ? std::clamp(round, 0.f, 0.5f) * d->plainSide : 0.f, 0.f, 0.f, 0, false, 0.f, d->fine), d->bent[0] = round;
}

// A flat shape in one colour, drawn by ThorVG's own Shape (a rectangle, a circle): `wide` times as wide as it is high.
// edge: ThorVG's stroke round it, inside its frame (its colour; w: how wide, a share of its height; 0: none).
// form: rect (round: its corners, a share of its shorter side), circle, polygon and star (points of it; a star's
// inner points at `inner` of the way out), line (a long rect, round at its ends), null (nothing to see: a thing to
// go with); all drawn by ThorVG's own Shape.
// How wide a picture's file is to its height, as ThorVG reads it (0: it cannot): on the driver thread, as drawn() makes one.
static float picture_aspect(Stage& s, const char* file) {
    float pw = 0.f, ph = 0.f;
    door::on_driver(s.engine, [&] {
        tvg::Initializer::init(0);
        tvg::Picture* pic = tvg::Picture::gen();
        if (pic->load(file) == tvg::Result::Success) pic->size(&pw, &ph);
        tvg::Paint::rel(pic);
    });
    s.engine->flushAndWait();
    return ph > 0.f ? pw / ph : 0.f;
}
static Drawn* flat(Stage& s, const char* form, float wide, const mat4f& place, float high, float3 colour, const char* lay = nullptr, float4 edge = float4{0.f}, const DrawnFx& fx = {},
                   int points = 6, float inner = 0.5f, float round = 0.f) {
    return drawn(s, [&](Drawn* d, int* w, int* h) -> tvg::Paint* {
        const float tall = std::min(0.5f * (float)std::max(s.w, s.h), 4096.f / std::max(wide, 1e-3f));   // (as fine as half the stage; a long thin bar less high, so that it is as long as it is said)
        *h = std::max(2, 2 * (int)std::ceil(0.5f * tall));
        *w = std::max(2, 2 * (int)std::ceil(0.5f * tall * wide));
        const float in = std::round(0.5f * std::min(edge.w * (float)*h, 0.5f * (float)std::min(*w, *h)));   // (half the stroke: the path is set in by that, so all of it is seen; a whole point, so its corners lie on whole points: ThorVG's round corners take a side as straight only when it is so exactly, tvgLottieModifier.cpp _colinear)
        const float thick = in > 0.f ? 2.f * in : 0.08f * (float)*h;   // (a wave's line: its stroke, or a share of its height)
        tvg::Shape* shape = d->shape = tvg::Shape::gen();
        flat_path(d, shape, form, *w, *h, in, points, inner, round, 0.f);
        shape->fill((uint8_t)std::lround(255.f * colour.x), (uint8_t)std::lround(255.f * colour.y), (uint8_t)std::lround(255.f * colour.z), strcmp(form, "null") && !fx.hollow ? 255 : 0);
        if (fx.ramp.size() > 1 && strcmp(form, "null")) shape->fill(ramp_of(fx.ramp, (float)*w, (float)*h, fx.radial, fx.rampAngle, fx.rampScale, fx.spread, fx.rampAt, fx.rampBlend));
        if (in > 0.f) shape->strokeWidth(2.f * in), shape->strokeFill((uint8_t)std::lround(255.f * edge.x), (uint8_t)std::lround(255.f * edge.y), (uint8_t)std::lround(255.f * edge.z));
        if (!strcmp(form, "wave") || !strcmp(form, "spiral")) {   // (only its line is seen: in its edge's colour, or its fill's; round at its ends)
            const float3 ink = in > 0.f ? float3{edge.x, edge.y, edge.z} : colour;
            shape->fill(0, 0, 0, 0);
            shape->strokeWidth(thick), shape->strokeFill((uint8_t)std::lround(255.f * ink.x), (uint8_t)std::lround(255.f * ink.y), (uint8_t)std::lround(255.f * ink.z));
            shape->strokeCap(tvg::StrokeCap::Round);
        }
        if (fx.cap >= 0) shape->strokeCap((tvg::StrokeCap)fx.cap);
        if (fx.join >= 0) shape->strokeJoin((tvg::StrokeJoin)fx.join);
        if (fx.dash > 0.f) { const float gap[2] = {fx.dash * (float)*h, fx.dash * (float)*h}; shape->strokeDash(gap, 2, fx.dashFrom * (float)*h); }   // (as long as the gaps; in shares of its height)
        if (!fx.media.empty() && strcmp(form, "null") && !fx.hollow) {   // (filled with a picture: it covers the shape, seen only where the shape is filled, ThorVG's own alpha mask as a ramp on letters is (type); its edge over it)
            tvg::Picture* pic = tvg::Picture::gen();
            float pw = 0.f, ph = 0.f;
            if (pic->load(fx.media.c_str()) == tvg::Result::Success && pic->size(&pw, &ph) == tvg::Result::Success && pw > 0.f && ph > 0.f) {
                const float k = std::max((float)*w / pw, (float)*h / ph);   // (covering it, the middle kept: cut, not stretched)
                pic->size(pw * k, ph * k);
                pic->translate(0.5f * ((float)*w - pw * k), 0.5f * ((float)*h - ph * k));
                tvg::Shape* cut = static_cast<tvg::Shape*>(shape->duplicate());
                cut->strokeWidth(0.f), cut->fill(255, 255, 255, 255);
                pic->mask(cut, tvg::MaskMethod::Alpha);
                shape->fill(0, 0, 0, 0);   // (its own fill none: the picture is its fill; its edge stays over it)
                tvg::Scene* both = tvg::Scene::gen();
                both->add(pic), both->add(shape);
                return both;
            }
            tvg::Paint::rel(pic);
        }
        return shape;
    }, place, high, float3{-1.f}, true, lay, fx);
}

void tvg_edge(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, uint32_t ptsCnt, float width, int cap, int join, float dash, float dashFrom,
              float from, float to, std::vector<float>& xy, std::vector<uint32_t>& idx);   // (bend.cpp)
