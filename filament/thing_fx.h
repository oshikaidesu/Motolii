// thing_fx.h: a thing's own effects (its fx: .wgsl and ISF files, ThorVG's own among them), as Filament's makers say
// to do one's own effect: the thing drawn alone into a texture offscreen (samples/rendertarget.cpp: a view of its own,
// its post-processing off), the effects run on that texture one after another (Friction's and After Effects' way: a
// layer's picture, then its effects), and the picture made laid where the thing lies (effected.mat; google/filament
// #1818, #7676, #3784). The functions as they were before the one view (git bf0d4c2, screen.h), less what laid the
// picture over the screen.
#pragma once
#include "solid_three.h"
#include "aviutl_fx.h"
static char doc_thor_letter(const std::string& w);
static void thing_effects(Stage& s, ScreenStack& st, const std::string& files) {
    if (st.files == files) return;
    st.files = files;
    st.fx.clear();
    st.steps.clear();
    st.readsBehind = false;
    std::vector<std::shared_ptr<AviutlRun>> scripts;
    for (const std::string& file : script_files(files)) {
        if (file == "none") continue;
        if (aviutl_is(file)) {   // (an AviUtl2 script: run each frame on its picture, the one already made kept)
            const auto kept = std::find_if(st.aviutl.begin(), st.aviutl.end(), [&](const auto& r) { return r->file == file; });
            scripts.push_back(kept != st.aviutl.end() ? *kept : aviutl_made(s, file)), st.steps.push_back('a');
            continue;
        }
        if (const char k = doc_thor_letter(file)) { st.steps.push_back(k); continue; }   // (one of ThorVG's own: drawn by ThorVG, in its place)
        if (std::filesystem::path(file).extension() == ".mjs") continue;   // (a splat effect: in the cloud's own look, points.h)
        if (const std::string ext = std::filesystem::path(file).extension().string(); ext == ".am" || ext == ".json") continue;   // (AbsentMotion's own motion, the thing moved: absent_motion.h; Filament's own settings, the stage's: viewer_settings.h)
        if (solid_is(file)) continue;   // (a solid's shape: its picture laid on it, not an effect on its picture: thing_fx_solid)
        ScreenFx*& made = st.made[file];
        if (!made) {
            made = new ScreenFx;
            made->file = file;
            std::string code;
            if (!screen_read(*made, code)) { std::fprintf(stderr, "%s\n", made->trouble.c_str()); continue; }
            made->door = door::screen(s.engine, file.c_str(), code.c_str(), made->bytes, &made->trouble, made->pictures, made->layered);
            if (!made->door) std::fprintf(stderr, "the effect %s is not taken: %s\n", file.c_str(), made->trouble.c_str());
            made->stamp = screen_stamp(file);
        }
        if (made->door) st.fx.push_back(made), st.steps.push_back(0);
        if (made->door && std::find(made->others.begin(), made->others.end(), "backdrop") != made->others.end()) st.readsBehind = true;   // (AbsentMotion's backdrop: the stage's behind, screen.h)
    }
    st.aviutl = scripts;
}
// One of ThorVG's own effects as a step (k: its letter), on the picture `from` (the device's) drawn into picture
// between `into` (0 or 1): a picture of the one before put in a scene of ThorVG's, the effect on the scene, drawn on the
// driver thread in line with Filament's pictures. Its lengths are points of the stage, `ppp` pixels each.
static void thing_thor(Stage& s, ScreenStack& st, char k, void* from, int into, uint32_t w, uint32_t h, const DrawnFx& fx, float ppp) {
    door::submit(s.engine);   // (what Filament drew into `from` on the GPU's queue first: ThorVG hands its own drawing over at once)
    door::on_driver(s.engine, [&s, &st, k, from, into, w, h, fx, ppp] {
        tvg::WgCanvas*& c = st.thor[into];
        if (!c) {
            tvg::Initializer::init(0);
            c = tvg::WgCanvas::gen();
            if (c->target(door::device(s.engine), door::instance(s.engine), st.device[into + 1], w, h, tvg::ColorSpace::ABGR8888S, 1) != tvg::Result::Success) return;
        }
        c->remove();
        tvg::Picture* picture = tvg::Picture::gen();
        if (picture->load(from, w, h) != tvg::Result::Success) return tvg::Paint::rel(picture);
        tvg::Scene* scene = tvg::Scene::gen();
        scene->add(picture);
        scene->add(tvg::SceneEffect::Clear);
        effect_on(scene, k, fx, ppp);
        c->add(scene);
        c->update();
        c->draw(true);
        c->sync();
    });
}
// Its pictures, as large as the view draws (made again when that moves): made before the frame, as the look that shows
// one is told it then (thing_fx_seen).
static void thing_fx_pictures(Stage& s, ScreenStack& st, View* v) {
    const Viewport vp = v->getViewport();
    const uint32_t w = (uint32_t)std::max(1, vp.left + (int)vp.width), h = (uint32_t)std::max(1, vp.bottom + (int)vp.height);
    if (!st.shot || st.shot->getWidth() != w || st.shot->getHeight() != h) {   // (its pictures as large as the view draws; made again when that moves)
        for (Texture*& t : {std::ref(st.shot), std::ref(st.depth), std::ref(st.between[0]), std::ref(st.between[1])}) if (t) s.engine->destroy(t), t = nullptr;
        for (RenderTarget*& r : {std::ref(st.target), std::ref(st.betweenTarget[0]), std::ref(st.betweenTarget[1])}) if (r) s.engine->destroy(r), r = nullptr;
        door::on_driver(s.engine, [made = std::vector<void*>(st.device, st.device + 3), thor = std::vector<tvg::WgCanvas*>(st.thor, st.thor + 2)] {   // (after the pictures drawn with them)
            for (tvg::WgCanvas* c : thor) delete c;
            for (void* t : made) if (t) wgpuTextureRelease((WGPUTexture)t);
        });
        st.thor[0] = st.thor[1] = nullptr;
        int n = 0;
        const auto picture = [&] {   // (the device's own, so ThorVG reads and draws it too; Filament's by import)
            WGPUTextureDescriptor like{};
            like.size = {w, h, 1};
            like.format = WGPUTextureFormat_BGRA8Unorm;   // (what ThorVG's last step writes, as a picture of its own is: drawn.h)
            like.usage = WGPUTextureUsage_RenderAttachment | WGPUTextureUsage_TextureBinding | WGPUTextureUsage_CopySrc | WGPUTextureUsage_CopyDst;
            like.dimension = WGPUTextureDimension_2D;
            like.mipLevelCount = 1, like.sampleCount = 1;
            static const char* const names[3] = {"thing shot", "thing between 0", "thing between 1"};
            like.label = {names[n], WGPU_STRLEN};
            void* made = st.device[n++] = wgpuDeviceCreateTexture((WGPUDevice)door::device(s.engine), &like);
            return Texture::Builder().width(w).height(h).levels(1).format(Texture::InternalFormat::RGBA8).usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC).import((intptr_t)made).build(*s.engine);
        };
        st.shot = picture();
        st.depth = Texture::Builder().width(w).height(h).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT).format(Texture::InternalFormat::DEPTH24).build(*s.engine);
        st.target = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, st.shot).texture(RenderTarget::AttachmentPoint::DEPTH, st.depth).build(*s.engine);
        for (int k = 0; k < 2; ++k) st.between[k] = picture(), st.betweenTarget[k] = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, st.between[k]).build(*s.engine);
    }
}
// The picture its effects leave (the last one drawn into; its own, with none): what its sheet or solid shows.
static Texture* thing_fx_seen(const ScreenStack& st) { return st.steps.empty() ? st.shot : st.between[st.steps.size() % 2]; }   // (the last made back in linear light: thing_effects_draw)
static void thing_effects_draw(Stage& s, ScreenStack& st, View* v, const ScreenNumbers& numbers, const std::function<DrawnFx()>& thorFx = nullptr, const ScreenPictures& pictures = nullptr) {
    thing_fx_pictures(s, st, v);   // (made already when it was aimed: a thing's; the stage's own others made here)
    const Viewport vp = v->getViewport();
    const uint32_t w = (uint32_t)std::max(1, vp.left + (int)vp.width), h = (uint32_t)std::max(1, vp.bottom + (int)vp.height);
    RenderTarget* const was = v->getRenderTarget();
    const Renderer::ClearOptions clearWas = s.renderer->getClearOptions();
    Renderer::ClearOptions clear = clearWas;
    clear.clearColor = {0, 0, 0, 0}, clear.clear = true, clear.discard = true;   // (the thing alone, on nothing)
    s.renderer->setClearOptions(clear);
    v->setRenderTarget(st.target);
    s.renderer->render(v);
    v->setRenderTarget(was);
    s.renderer->setClearOptions(clearWas);
    st.seen = st.shot;
    if (st.steps.empty()) return;   // (quiet with no effects: only its picture, for another's)
    const float ppp = (float)s.view->getViewport().width / (float)std::max(1, s.w);   // (pixels of its picture a point of the stage, as the stage's view draws: the desk's are fewer)
    const DrawnFx thor = thorFx && std::any_of(st.steps.begin(), st.steps.end(), [](char k) { return k != 0 && k != 'a'; }) ? thorFx() : DrawnFx{};
    screen_tone(s, st.shot, st.betweenTarget[1], w, h, true);   // (its picture as its colours are said, sRGB, for the effects: screen.h screen_tone)
    for (size_t i = 0, f = 0, a = 0; i < st.steps.size(); ++i) {   // (each into a picture between: the effects take the whole picture as theirs, and fill what is clear)
        Texture* from = i == 0 ? st.between[1] : st.between[(i - 1) % 2];
        if (st.steps[i] == 'a') { aviutl_draw_step(s, *st.aviutl[a++], from, st.betweenTarget[i % 2], numbers, w, h); continue; }   // (an AviUtl2 script)
        if (st.steps[i]) { thing_thor(s, st, st.steps[i], st.device[i == 0 ? 2 : 1 + (i - 1) % 2], (int)(i % 2), w, h, thor, ppp); continue; }   // (one of ThorVG's own)
        screen_fresh(s, *st.fx[f]);
        ScreenFx& one = *st.fx[f++];
        one.othersNow.clear();
        for (const std::string& n : one.others) one.othersNow.push_back(pictures ? pictures(one.file, n) : nullptr);
        screen_draw(s, st, one, from, st.betweenTarget[i % 2], numbers, w, h);
    }
    screen_tone(s, st.between[(st.steps.size() - 1) % 2], st.betweenTarget[st.steps.size() % 2], w, h, false);   // (back in linear light, as the stage takes it)
    st.seen = thing_fx_seen(st);
}

// A thing with effects of its own as the stage keeps it: the view it is drawn alone in, its effects, and the sheet its
// picture with them on is laid on in the stage's own view.
struct ThingFx {
    ScreenStack st;
    View* view = nullptr;
    Mesh* sheet = nullptr;
    MaterialInstance* look = nullptr;
    std::string lay;                       // (how its sheet lies on what is behind: Filament's own blending, effected.mat's ways; none: over it)
    utils::Entity eye;                     // (its own camera's: told the stage's each picture, as samples/rendertarget.cpp's offscreen camera is)
    Camera* camera = nullptr;
    double box[4] = {0, 0, 0, 0};          // (where it is seen, in points of the stage: left, top, right, bottom)
    // (its picture on a solid's faces, when its fx names a shape (solids/*.js): the shape's file, what the solid was last
    //  made of, the solid and its look; for one made of its outline, its coverage as last read back, and how many times)
    std::string solidFile, solidSaid, solidAs;   // (solidAs: its look's material)
    std::map<std::string, double> knobs;   // (its shape's knobs as said now, its words, and the thing's size: its own units)
    std::map<std::string, std::string> words;
    float2 size{1.f};
    Mesh* solid = nullptr;
    MaterialInstance* solidLook = nullptr;
    bool solidRepeat = false;
    struct Cover { std::vector<uint8_t> a; int w = 0, h = 0, times = 0; };
    std::shared_ptr<Cover> cover = std::make_shared<Cover>();
};

// The view a thing is drawn alone in: samples/rendertarget.cpp's offscreen view (its own scene, the stage's eye, its
// post-processing off: the stage's own view does that once, over everything).
static View* thing_fx_view(Stage& s, ThingFx& e) {
    View* v = s.engine->createView();
    v->setScene(s.engine->createScene());
    v->getScene()->setIndirectLight(s.scene->getIndirectLight());
    e.eye = utils::EntityManager::get().create();
    e.camera = s.engine->createCamera(e.eye);
    v->setCamera(e.camera);
    v->setViewport(s.view->getViewport());
    v->setPostProcessingEnabled(false);
    v->setVisibleLayers(0x03, 0x03);   // (both of the stage's layers seen, as the stage's own view sees them)
    v->setBlendMode(View::BlendMode::TRANSLUCENT);
    return v;
}

// The sheet: the whole picture, samples/materials/image.mat's way (its points are the screen's own corners).
static ThingFx& thing_fx_made(Stage& s, ThingFx& e, const std::string& lay = "") {
    if (e.view && e.lay == lay) return e;
    if (!e.view) e.view = thing_fx_view(s, e);
    if (e.sheet) take_off(s, e.sheet), delete e.sheet;   // (another way of lying: its sheet made again in it)
    e.lay = lay;
    e.sheet = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) e.sheet->pos.push_back({c.x, c.y, 0.f}), e.sheet->nrm.push_back({0, 0, 1}), e.sheet->uv.push_back(c * 0.5f + 0.5f);
    e.sheet->idx = {0, 1, 2, 0, 2, 3};
    e.look = add(s, e.sheet, lay.empty() ? "effected.filamat" : ("effected_" + lay + ".filamat").c_str());
    e.look->setParameter("image", white_point(s), TextureSampler());
    e.look->setParameter("place", math::mat4f{});   // (the whole stage till it is aimed)   // (a picture from the start, as image.mat's sample sets it at once: Filament readies every instance before any view, the thing's own drawn first; never seen, its picture set before the stage's view)
    return e;
}

static void thing_fx_off(Stage& s, ThingFx& e) {
    if (e.sheet) take_off(s, e.sheet), delete e.sheet;
    if (e.solid) take_off(s, e.solid), delete e.solid;
    if (e.view) s.engine->destroy(e.view->getScene()), s.engine->destroy(e.view);
    if (e.camera) s.engine->destroyCameraComponent(e.eye), utils::EntityManager::get().destroy(e.eye);
    e = ThingFx{};
}

// Aimed at the box the thing is seen in (left, top, right, bottom, points of the stage): its own camera told the stage's
// projection and place, as samples/rendertarget.cpp tells its offscreen camera each frame (setCustomProjection,
// setModelMatrix), the projection narrowed to the box so the box is the whole of its picture (After Effects' layer: its
// effects work on the layer's own picture, an ISF effect's centre its middle); its picture as many pixels as the box
// covers on the stage's view; its sheet laid on the box (effected.mat place).
static void thing_fx_aim(Stage& s, ThingFx& e, const double* box) {
    std::copy(box, box + 4, e.box);
    const double ppp = (double)s.view->getViewport().width / std::max(1, s.w);
    const double x0 = 2 * box[0] / s.w - 1, x1 = 2 * box[2] / s.w - 1, y0 = 1 - 2 * box[3] / s.h, y1 = 1 - 2 * box[1] / s.h;   // (the box in the stage's clip space, y up)
    math::mat4 crop;
    crop[0][0] = 2 / (x1 - x0), crop[1][1] = 2 / (y1 - y0), crop[3][0] = -(x0 + x1) / (x1 - x0), crop[3][1] = -(y0 + y1) / (y1 - y0);
    const Camera& c = *s.camera;
    e.camera->setCustomProjection(crop * c.getProjectionMatrix(), crop * c.getCullingProjectionMatrix(), c.getNear(), c.getCullingFar());
    e.camera->setModelMatrix(c.getModelMatrix());
    e.camera->setExposure(c.getAperture(), c.getShutterSpeed(), c.getSensitivity());
    e.view->setViewport({0, 0, (uint32_t)std::max(1L, std::lround((box[2] - box[0]) * ppp)), (uint32_t)std::max(1L, std::lround((box[3] - box[1]) * ppp))});
    math::mat4f place;
    place[0][0] = (float)((x1 - x0) / 2), place[1][1] = (float)((y1 - y0) / 2), place[3][0] = (float)((x0 + x1) / 2), place[3][1] = (float)((y0 + y1) / 2);
    e.look->setParameter("place", place);
}

// Aimed square at the thing's own face, for a solid's faces (its picture the same however it is turned): an orthographic
// camera on its own z line, as wide and high as it is (Filament's Camera::setProjection ORTHO; told each frame, as
// samples/rendertarget.cpp tells its offscreen camera), its picture 1024 pixels along its longer side.
static void thing_fx_face(Stage& s, ThingFx& e, const math::mat4& place, float2 half) {
    math::double3 x = place[0].xyz, y = place[1].xyz;
    const double sx = length(x), sy = length(y), wide = half.x * sx, high = half.y * sy, far = 4.0 * std::max(wide, high) + 1.0;
    if (wide < 1e-9 || high < 1e-9) return;
    x /= sx, y = normalize(y - dot(y, x) * x);
    const math::double3 z = cross(x, y);
    e.camera->setModelMatrix(math::mat4(math::double4{x, 0}, math::double4{y, 0}, math::double4{z, 0}, math::double4{place[3].xyz + 0.5 * far * z, 1}));
    e.camera->setProjection(Camera::Projection::ORTHO, -wide, wide, -high, high, 0.0, far);
    const Camera& c = *s.camera;
    e.camera->setExposure(c.getAperture(), c.getShutterSpeed(), c.getSensitivity());
    const uint32_t w = (uint32_t)std::max(1L, std::lround(1024 * wide / std::max(wide, high))), h = (uint32_t)std::max(1L, std::lround(1024 * high / std::max(wide, high)));
    e.view->setViewport({0, 0, w, h});
}

// Its solid made again when what it is made of has changed (its shape's file, knobs and size; its coverage): put in the
// stage's own scene, casting and taking shadows, unlit when its Lit is off; one already there given its new points in
// place (reshape). Within a frame only that: a look made then is not ready till the next (Filament readies every look
// as a frame begins, FEngine::prepare), so a new one waits for the next moment.
static void thing_fx_solid(Stage& s, ThingFx& e, bool inFrame) {
    std::string said = e.solidFile + " " + std::to_string(e.size.x) + " " + std::to_string(e.size.y) + " " + std::to_string(e.cover->times);
    for (const auto& [n, v] : e.knobs) said += " " + n + "=" + std::to_string(v);
    for (const auto& [n, w] : e.words) said += " " + n + ":" + w;
    const auto lit = e.knobs.find("lit");
    const bool flat = lit != e.knobs.end() && lit->second == 0;
    const char* const look = flat ? "solid_image_flat.filamat" : "solid_image.filamat";
    const bool anew = !e.solid || e.solidAs != look;
    if (said == e.solidSaid || (anew && inFrame)) return;
    SolidMade made;
    std::string why;
    if (!solid_three(e.solidFile, e.knobs, e.words, e.size.x, e.size.y, e.cover->a, e.cover->w, e.cover->h, made, why)) std::fprintf(stderr, "the solid %s is not made: %s\n", e.solidFile.c_str(), why.substr(0, 600).c_str());
    if (made.idx.empty() || made.pos.size() != made.nrm.size() || made.uv.size() * 3 != made.pos.size() * 2) made = SolidMade{{0, 0, 0, 0, 0, 0, 0, 0, 0}, {0, 0, 1, 0, 0, 1, 0, 0, 1}, {0, 0, 0, 0, 0, 0}, {0, 1, 2}};   // (none yet, Raster Extrude's before its picture is read: a point, its look ready)
    e.solidSaid = said;
    if (anew && e.solid) take_off(s, e.solid), delete e.solid, e.solid = nullptr;
    Mesh* m = e.solid ? e.solid : new Mesh;
    m->pos.resize(made.pos.size() / 3), m->nrm.resize(made.pos.size() / 3), m->uv.resize(made.uv.size() / 2);
    std::memcpy(m->pos.data(), made.pos.data(), made.pos.size() * sizeof(float)), std::memcpy(m->nrm.data(), made.nrm.data(), made.nrm.size() * sizeof(float));
    std::memcpy(m->uv.data(), made.uv.data(), made.uv.size() * sizeof(float));
    m->idx = std::move(made.idx);
    e.solidRepeat = made.repeat;
    if (!anew) return reshape(s, m, {(uint32_t)m->idx.size()});
    e.solid = m, e.solidAs = look;
    e.solidLook = add(s, m, look, {mat4f{}}, 1, true);
    if (!e.solidLook) return;
    e.solidLook->setParameter("image", white_point(s), TextureSampler());   // (a picture from the start, as the sheet's)
    if (!flat) e.solidLook->setParameter("roughness", 0.5f), e.solidLook->setParameter("metallic", 0.f);
}

// Its picture's coverage read back for a shape made of its outline (Filament's Renderer::readPixels, its callback on a
// later frame: the shape made of it then), every so many pixels, at most `cells` along the longer side.
static void thing_fx_cover(Stage& s, ThingFx& e, int cells) {
    RenderTarget* const from = e.st.steps.empty() ? e.st.target : e.st.betweenTarget[(e.st.steps.size() - 1) % 2];
    const Viewport vp = e.view->getViewport();
    if (!from || !vp.width || !vp.height) return;
    auto* pixels = new std::vector<uint8_t>((size_t)vp.width * vp.height * 4);
    s.renderer->readPixels(from, 0, 0, vp.width, vp.height, backend::PixelBufferDescriptor::make(pixels->data(), pixels->size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE,
        [pixels, cover = e.cover, w = (int)vp.width, h = (int)vp.height, cells](void const*, size_t) {
            const double every = (double)std::max(w, h) / std::max(2, cells);
            const int cw = std::max(2, (int)(w / every)), ch = std::max(2, (int)(h / every));
            std::vector<uint8_t> a((size_t)cw * ch);
            for (int y = 0; y < ch; ++y) for (int x = 0; x < cw; ++x) a[(size_t)(y * cw + x)] = (*pixels)[4 * ((size_t)std::min(h - 1, (int)(y * every)) * w + std::min(w - 1, (int)(x * every))) + 3];
            if (a != cover->a || cw != cover->w) cover->a = std::move(a), cover->w = cw, cover->h = ch, cover->times++;
            delete pixels;
        }));
}
