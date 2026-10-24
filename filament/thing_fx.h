// thing_fx.h: a thing's own effects (its fx: .wgsl and ISF files, ThorVG's own among them), as Filament's makers say
// to do one's own effect: the thing drawn alone into a texture offscreen (samples/rendertarget.cpp: a view of its own,
// its post-processing off), the effects run on that texture one after another (Friction's and After Effects' way: a
// layer's picture, then its effects), and the picture made laid where the thing lies (effected.mat; google/filament
// #1818, #7676, #3784). The functions as they were before the one view (git bf0d4c2, screen.h), less what laid the
// picture over the screen.
#pragma once
static char doc_thor_letter(const std::string& w);
static void thing_effects(Stage& s, ScreenStack& st, const std::string& files) {
    if (st.files == files) return;
    st.files = files;
    st.fx.clear();
    st.steps.clear();
    for (const std::string& file : script_files(files)) {
        if (file == "none") continue;
        if (const char k = doc_thor_letter(file)) { st.steps.push_back(k); continue; }   // (one of ThorVG's own: drawn by ThorVG, in its place)
        if (std::filesystem::path(file).extension() == ".mjs") continue;   // (a splat effect: in the cloud's own look, points.h)
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
    }
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
static void thing_effects_draw(Stage& s, ScreenStack& st, View* v, const ScreenNumbers& numbers, const std::function<DrawnFx()>& thorFx = nullptr, const ScreenPictures& pictures = nullptr) {
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
            return Texture::Builder().width(w).height(h).levels(1).format(Texture::InternalFormat::RGBA8).usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE).import((intptr_t)made).build(*s.engine);
        };
        st.shot = picture();
        st.depth = Texture::Builder().width(w).height(h).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT).format(Texture::InternalFormat::DEPTH24).build(*s.engine);
        st.target = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, st.shot).texture(RenderTarget::AttachmentPoint::DEPTH, st.depth).build(*s.engine);
        for (int k = 0; k < 2; ++k) st.between[k] = picture(), st.betweenTarget[k] = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, st.between[k]).build(*s.engine);
    }
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
    const float ppp = (float)vp.width / (float)std::max(1, s.w);   // (pixels of its picture a point of the stage: the desk's are fewer)
    const DrawnFx thor = thorFx && std::any_of(st.steps.begin(), st.steps.end(), [](char k) { return k != 0; }) ? thorFx() : DrawnFx{};
    for (size_t i = 0, f = 0; i < st.steps.size(); ++i) {   // (each into a picture between: the effects take the whole picture as theirs, and fill what is clear)
        Texture* from = i == 0 ? st.shot : st.between[(i - 1) % 2];
        if (st.steps[i]) { thing_thor(s, st, st.steps[i], st.device[i == 0 ? 0 : 1 + (i - 1) % 2], (int)(i % 2), w, h, thor, ppp); continue; }   // (one of ThorVG's own)
        screen_fresh(s, *st.fx[f]);
        ScreenFx& one = *st.fx[f++];
        one.othersNow.clear();
        for (const std::string& n : one.others) one.othersNow.push_back(pictures ? pictures(one.file, n) : nullptr);
        screen_draw(s, st, one, from, st.betweenTarget[i % 2], numbers, w, h);
    }
    st.seen = st.between[(st.steps.size() - 1) % 2];
}

// A thing with effects of its own as the stage keeps it: the view it is drawn alone in, its effects, and the sheet its
// picture with them on is laid on in the stage's own view.
struct ThingFx {
    ScreenStack st;
    View* view = nullptr;
    Mesh* sheet = nullptr;
    MaterialInstance* look = nullptr;
};

// The view a thing is drawn alone in: samples/rendertarget.cpp's offscreen view (its own scene, the stage's eye, its
// post-processing off: the stage's own view does that once, over everything).
static View* thing_fx_view(Stage& s) {
    View* v = s.engine->createView();
    v->setScene(s.engine->createScene());
    v->getScene()->setIndirectLight(s.scene->getIndirectLight());
    v->setCamera(s.camera);
    v->setViewport(s.view->getViewport());
    v->setPostProcessingEnabled(false);
    v->setVisibleLayers(0x03, 0x03);   // (both of the stage's layers seen, as the stage's own view sees them)
    v->setBlendMode(View::BlendMode::TRANSLUCENT);
    return v;
}

// The sheet: the whole picture, samples/materials/image.mat's way (its points are the screen's own corners).
static ThingFx& thing_fx_made(Stage& s, ThingFx& e) {
    if (e.view) return e;
    e.view = thing_fx_view(s);
    e.sheet = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) e.sheet->pos.push_back({c.x, c.y, 0.f}), e.sheet->nrm.push_back({0, 0, 1}), e.sheet->uv.push_back(c * 0.5f + 0.5f);
    e.sheet->idx = {0, 1, 2, 0, 2, 3};
    e.look = add(s, e.sheet, "effected.filamat");
    return e;
}

static void thing_fx_off(Stage& s, ThingFx& e) {
    if (e.sheet) take_off(s, e.sheet), delete e.sheet;
    if (e.view) s.engine->destroy(e.view->getScene()), s.engine->destroy(e.view);
    e = ThingFx{};
}
