// spatial.mm, part 2 (cut there by size, as it was): read only through spatial.mm, in order.
#pragma once


static float thorvg_xy_delta(const std::vector<Cubic>& spans) {
    auto* shape = tvg::Shape::gen();
    if (!write_xy(shape, spans)) return -1;
    const tvg::PathCommand* cmds = nullptr;
    const tvg::Point* pts = nullptr;
    if (shape->path(&cmds, nullptr, &pts, nullptr) != tvg::Result::Success) return -1;
    float m = 0;
    int pi = 0;
    int span = -1;
    for (int i = 0; cmds[i] != tvg::PathCommand::Close; ++i) {
        if (cmds[i] == tvg::PathCommand::MoveTo) {
            ++pi;
            continue;
        }
        if (cmds[i] != tvg::PathCommand::CubicTo) continue;
        ++span;
        V3 a{pts[pi - 1].x, pts[pi - 1].y, 0};
        V3 b{pts[pi].x, pts[pi].y, 0};
        V3 c{pts[pi + 1].x, pts[pi + 1].y, 0};
        V3 d{pts[pi + 2].x, pts[pi + 2].y, 0};
        pi += 3;
        Cubic got{a, b, c, d};
        for (int k = 0; k <= 8; ++k) {
            float t = (float)k / 8.f;
            V3 u = bezier(spans[span], t);
            V3 v = bezier(got, t);
            m = std::max(m, std::hypot(u.x - v.x, u.y - v.y));
        }
    }
    return m;
}

static bool raster_shape(tvg::Shape* stroke, std::vector<uint8_t>& rgba) {
    std::vector<uint32_t> buf((size_t)W * H, 0);
    auto* canvas = tvg::SwCanvas::gen();
    canvas->target(buf.data(), W, W, H, tvg::ColorSpace::ABGR8888S);
    auto* bg = tvg::Shape::gen();
    bg->appendRect(0, 0, (float)W, (float)H);
    bg->fill(18, 20, 24, 255);
    stroke->strokeWidth(3.f);
    stroke->strokeJoin(tvg::StrokeJoin::Round);
    stroke->strokeFill(236, 224, 196, 255);
    canvas->add(bg);
    canvas->add(stroke);
    canvas->draw(true);
    canvas->sync();
    rgba.resize((size_t)W * H * 4);
    for (int i = 0; i < W * H; ++i) {
        uint32_t p = buf[i];
        rgba[(size_t)i * 4 + 0] = (uint8_t)(p & 255);
        rgba[(size_t)i * 4 + 1] = (uint8_t)((p >> 8) & 255);
        rgba[(size_t)i * 4 + 2] = (uint8_t)((p >> 16) & 255);
        rgba[(size_t)i * 4 + 3] = 255;
    }
    return true;
}

static bool raster_xy(const std::vector<Cubic>& spans, std::vector<uint8_t>& rgba) {
    std::vector<Cubic> screen = spans;
    for (Cubic& c : screen)
        for (V3& p : c.p) {
            p.x = p.x * 220.f + W * 0.5f;
            p.y = -p.y * 220.f + H * 0.5f;
        }
    auto* stroke = tvg::Shape::gen();
    if (!write_xy(stroke, screen)) return false;
    return raster_shape(stroke, rgba);
}

struct App {
    Engine* engine = nullptr;
    Renderer* renderer = nullptr;
    Scene* scene = nullptr;
    View* view = nullptr;
    SwapChain* swap = nullptr;
    Camera* camera = nullptr;
    MaterialInstance* mi = nullptr;
    MaterialInstance* gpu_mi = nullptr;
    RenderTarget* rt = nullptr;
};

static bool grab(App& a, std::vector<uint8_t>& picture) {
    picture.resize((size_t)W * H * 4);
    bool drew = false;
    for (int attempt = 0; attempt < 4 && !drew; ++attempt) {
        if (!a.renderer->beginFrame(a.swap)) continue;
        a.renderer->render(a.view);
        a.renderer->readPixels(a.rt, 0, 0, W, H,
                               backend::PixelBufferDescriptor(picture.data(), picture.size(), backend::PixelDataFormat::RGBA,
                                                              backend::PixelDataType::UBYTE));
        a.renderer->endFrame();
        drew = true;
    }
    a.engine->flushAndWait();
    return drew;
}

static utils::Entity upload(App& a, const Mesh& mesh, MaterialInstance* mi) {
    const uint32_t n = (uint32_t)(mesh.v.size() / 7);
    auto* vb = VertexBuffer::Builder()
                   .vertexCount(n)
                   .bufferCount(1)
                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                   .attribute(VertexAttribute::COLOR, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                   .build(*a.engine);
    vb->setBufferAt(*a.engine, 0, VertexBuffer::BufferDescriptor(mesh.v.data(), mesh.v.size() * sizeof(float)));
    auto* ib = IndexBuffer::Builder().indexCount((uint32_t)mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*a.engine);
    ib->setBuffer(*a.engine, IndexBuffer::BufferDescriptor(mesh.idx.data(), mesh.idx.size() * sizeof(uint32_t)));
    utils::Entity e = utils::EntityManager::get().create();
    RenderableManager::Builder(1)
        .boundingBox({{-4, -4, -4}, {4, 4, 4}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
        .material(0, mi)
        .culling(false)
        .castShadows(false)
        .receiveShadows(false)
        .build(*a.engine, e);
    a.scene->addEntity(e);
    return e;
}

static void look(App& a, float3 eye, float3 at, float3 up) {
    a.camera->lookAt(eye, at, up);
}

static bool shot(App& a, const Mesh& mesh, float3 eye, float3 at, float3 up, std::vector<uint8_t>& rgba) {
    look(a, eye, at, up);
    utils::Entity e = upload(a, mesh, a.mi);
    a.engine->flushAndWait();
    bool ok = grab(a, rgba) && grab(a, rgba);
    a.scene->remove(e);
    return ok;
}

static std::vector<uint8_t> hstack(const std::vector<std::vector<uint8_t>>& cells) {
    int n = (int)cells.size();
    std::vector<uint8_t> out((size_t)W * n * H * 4);
    for (int i = 0; i < n; ++i)
        for (int y = 0; y < H; ++y)
            std::memcpy(out.data() + ((size_t)y * W * n + (size_t)i * W) * 4, cells[i].data() + (size_t)y * W * 4, (size_t)W * 4);
    return out;
}

static std::vector<uint8_t> grid2(const std::vector<uint8_t>& a, const std::vector<uint8_t>& b, const std::vector<uint8_t>& c,
                                  const std::vector<uint8_t>& d) {
    std::vector<uint8_t> out((size_t)W * 2 * H * 2 * 4);
    const std::vector<uint8_t>* cells[4] = {&a, &b, &c, &d};
    for (int i = 0; i < 4; ++i) {
        int ox = (i % 2) * W;
        int oy = (i / 2) * H;
        for (int y = 0; y < H; ++y)
            std::memcpy(out.data() + ((size_t)(oy + y) * W * 2 + ox) * 4, cells[i]->data() + (size_t)y * W * 4, (size_t)W * 4);
    }
    return out;
}

static bool project_guide(App& a, const std::vector<Sample>& samples, std::vector<uint8_t>& rgba) {
    mat4 view = a.camera->getViewMatrix();
    mat4 proj = a.camera->getProjectionMatrix();
    auto* stroke = tvg::Shape::gen();
    bool moved = false;
    for (const Sample& s : samples) {
        float4 clip = proj * (view * float4{s.p.x, s.p.y, s.p.z, 1});
        if (clip.w <= 0.05f) continue;
        float x = (clip.x / clip.w * 0.5f + 0.5f) * W;
        float y = (0.5f - clip.y / clip.w * 0.5f) * H;
        if (!moved) {
            stroke->moveTo(x, y);
            moved = true;
        } else {
            stroke->lineTo(x, y);
        }
    }
    if (!moved) return false;
    return raster_shape(stroke, rgba);
}

static double ms_since(std::chrono::steady_clock::time_point t0) {
    return std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
}

static std::vector<Cubic> stress_curve(int spans) {
    std::vector<V3> p;
    for (int i = 0; i <= spans; ++i) {
        float a = (float)i / (float)spans * 6.2831853f * 3.f;
        float r = 0.55f + 0.18f * std::sin(a * 2.f);
        p.push_back({r * std::cos(a), r * std::sin(a) * 0.7f, 0.35f * std::sin(a * 3.f)});
    }
    return catmull(p, false);
}

int main() {
    if (tvg::Initializer::init(0) != tvg::Result::Success) return fail("thorvg");
    auto spatial_pkg = load_file("spatial.filamat");
    auto gpu_pkg = load_file("spatial_gpu.filamat");
    if (spatial_pkg.empty() || gpu_pkg.empty()) return fail("filamat");

    App a;
    a.engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    a.renderer = a.engine->createRenderer();
    a.scene = a.engine->createScene();
    a.view = a.engine->createView();
    a.swap = a.engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    a.camera = a.engine->createCamera(utils::EntityManager::get().create());
    a.camera->setExposure(16.f, 1.f / 125.f, 100.f);
    a.camera->setProjection(38.0, double(W) / double(H), 0.05, 40.0, Camera::Fov::VERTICAL);
    Texture* color = Texture::Builder()
                         .width(W)
                         .height(H)
                         .levels(1)
                         .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                         .format(Texture::InternalFormat::RGBA8)
                         .build(*a.engine);
    Texture* depth = Texture::Builder()
                         .width(W)
                         .height(H)
                         .levels(1)
                         .usage(Texture::Usage::DEPTH_ATTACHMENT)
                         .format(Texture::InternalFormat::DEPTH24)
                         .build(*a.engine);
    a.rt = RenderTarget::Builder()
                           .texture(RenderTarget::AttachmentPoint::COLOR, color)
                           .texture(RenderTarget::AttachmentPoint::DEPTH, depth)
                           .build(*a.engine);
    static LinearToneMapper tone;
    a.view->setColorGrading(ColorGrading::Builder().toneMapper(&tone).build(*a.engine));
    a.scene->setSkybox(Skybox::Builder().color({0.04f, 0.045f, 0.055f, 1}).build(*a.engine));
    a.view->setScene(a.scene);
    a.view->setCamera(a.camera);
    a.view->setRenderTarget(a.rt);
    a.view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    a.view->setPostProcessingEnabled(true);
    a.view->setScreenSpaceRefractionEnabled(false);
    a.view->setAntiAliasing(View::AntiAliasing::FXAA);
    a.view->setDithering(View::Dithering::NONE);
    Renderer::ClearOptions clear{};
    clear.clearColor = {0.04f, 0.045f, 0.055f, 1};
    clear.clear = true;
    a.renderer->setClearOptions(clear);
    Material* spatial_mat = Material::Builder().package(spatial_pkg.data(), spatial_pkg.size()).build(*a.engine);
    Material* gpu_mat = Material::Builder().package(gpu_pkg.data(), gpu_pkg.size()).build(*a.engine);
    if (!spatial_mat || !gpu_mat) return fail("material");
    a.mi = spatial_mat->createInstance();
    a.gpu_mi = gpu_mat->createInstance();

    const float3 up{0, 1, 0};
    const float3 eye_a{1.7f, 1.15f, 1.9f};
    const float3 eye_b{-1.8f, 0.7f, 1.35f};
    const float3 eye_c{0.15f, 2.3f, 0.2f};
    const float3 eye_top_up{0, 0, -1};
    auto samples_of = [](const std::vector<Cubic>& c, FrameKind f, bool closed) { return sample_curve(c, f, closed, SAMPLES); };
    auto mesh_of = [&](const std::vector<Cubic>& c, FrameKind f, ShapeKind s, bool closed) {
        return build_shape(samples_of(c, f, closed), s, closed);
    };

    std::vector<uint8_t> c0, c1, c2, c3;
    auto bend = catmull(edit_points(), false);
    auto tube_bend = mesh_of(bend, FrameKind::Parallel, ShapeKind::Tube, false);
    uint64_t hash_a = mesh_hash(tube_bend);
    if (!shot(a, tube_bend, eye_a, {0.1f, 0, 0.3f}, up, c0)) return fail("orbit a");
    uint64_t hash_b = mesh_hash(tube_bend);
    if (!shot(a, tube_bend, eye_b, {0.1f, 0, 0.3f}, up, c1)) return fail("orbit b");
    if (!shot(a, tube_bend, eye_c, {0.1f, 0, 0.3f}, eye_top_up, c2)) return fail("orbit c");
    auto orbit = hstack({c0, c1, c2});
    if (!write_png("out/spatial-orbit.png", W * 3, H, orbit.data())) return fail("png orbit");

    auto moved_pts = edit_points();
    moved_pts[2].y += 0.2f;
    moved_pts[2].z += 0.55f;
    auto moved = catmull(moved_pts, false);
    if (!shot(a, tube_bend, eye_a, {0.1f, 0, 0.3f}, up, c0)) return fail("edit before");
    if (!shot(a, mesh_of(moved, FrameKind::Parallel, ShapeKind::Tube, false), eye_a, {0.1f, 0, 0.3f}, up, c1)) return fail("edit after");
    auto edit = hstack({c0, c1});
    if (!write_png("out/spatial-edit.png", W * 2, H, edit.data())) return fail("png edit");

    auto diag = diag_curve();
    if (!shot(a, mesh_of(diag, FrameKind::Parallel, ShapeKind::Tube, false), {1.6f, 1.4f, 2.2f}, {0.2f, 0.15f, 0.6f}, up, c0))
        return fail("shape tube");
    if (!shot(a, mesh_of(diag, FrameKind::Parallel, ShapeKind::Ribbon, false), {1.6f, 1.4f, 2.2f}, {0.2f, 0.15f, 0.6f}, up, c1))
        return fail("shape ribbon");
    if (!shot(a, mesh_of(diag, FrameKind::Parallel, ShapeKind::Sweep, false), {1.6f, 1.4f, 2.2f}, {0.2f, 0.15f, 0.6f}, up, c2))
        return fail("shape sweep");
    auto shapes = hstack({c0, c1, c2});
    if (!write_png("out/spatial-shapes.png", W * 3, H, shapes.data())) return fail("png shapes");

    const float3 diag_eye{1.8f, 1.5f, 2.0f};
    const float3 diag_at{0.2f, 0.1f, 0.6f};
    if (!shot(a, mesh_of(diag, FrameKind::ZUp, ShapeKind::Ribbon, false), diag_eye, diag_at, up, c0)) return fail("frame z");
    if (!shot(a, mesh_of(diag, FrameKind::Frenet, ShapeKind::Ribbon, false), diag_eye, diag_at, up, c1)) return fail("frame f");
    if (!shot(a, mesh_of(diag, FrameKind::Parallel, ShapeKind::Ribbon, false), diag_eye, diag_at, up, c2)) return fail("frame p");
    auto frames = hstack({c0, c1, c2});
    if (!write_png("out/spatial-frames.png", W * 3, H, frames.data())) return fail("png frames");

    auto flat = flat_rect();
    auto lifted = lifted_rect();
    float xy_delta = thorvg_xy_delta(flat);
    std::vector<Cubic> lifted_back = lifted;
    for (Cubic& c : lifted_back)
        for (V3& p : c.p) p.z = 0;
    float back_delta = 0;
    for (size_t i = 0; i < flat.size(); ++i)
        for (int k = 0; k < 4; ++k) back_delta = std::max(back_delta, len(flat[i].p[k] - lifted_back[i].p[k]));
    if (!raster_xy(flat, c0)) return fail("raster flat");
    if (!shot(a, mesh_of(flat, FrameKind::Parallel, ShapeKind::Tube, true), {0.2f, 1.5f, 1.8f}, {0, 0, 0}, up, c1)) return fail("flat 3d");
    if (!shot(a, mesh_of(lifted, FrameKind::Parallel, ShapeKind::Tube, true), {0.9f, 1.2f, 1.8f}, {0, 0, 0.2f}, up, c2))
        return fail("lifted 3d");
    look(a, {0.9f, 1.2f, 1.8f}, {0, 0, 0.2f}, up);
    if (!project_guide(a, samples_of(lifted, FrameKind::Parallel, true), c3)) return fail("guide");
    auto roundtrip = grid2(c0, c1, c2, c3);
    if (!write_png("out/spatial-roundtrip.png", W * 2, H * 2, roundtrip.data())) return fail("png round");

    look(a, eye_a, {0.1f, 0, 0.3f}, up);
    std::vector<uint8_t> g0, g1;
    if (!project_guide(a, samples_of(bend, FrameKind::Parallel, false), g0)) return fail("guide a");
    look(a, eye_b, {0.1f, 0, 0.3f}, up);
    if (!project_guide(a, samples_of(bend, FrameKind::Parallel, false), g1)) return fail("guide b");
    float guide_delta = 0;
    int guide_n = std::min((int)g0.size(), (int)g1.size());
    int guide_far = 0;
    for (int i = 0; i < guide_n; i += 4) {
        int d = std::abs((int)g0[i] - (int)g1[i]) + std::abs((int)g0[i + 1] - (int)g1[i + 1]) + std::abs((int)g0[i + 2] - (int)g1[i + 2]);
        if (d > 40) ++guide_far;
    }
    (void)guide_delta;
    auto guides = hstack({g0, g1});
    if (!write_png("out/spatial-guides.png", W * 2, H, guides.data())) return fail("png guides");

    auto cusp = cusp_curve();
    auto cross = cross_curve();
    auto loop = closed_curve();
    if (!shot(a, mesh_of(cusp, FrameKind::Parallel, ShapeKind::Tube, false), {1.2f, 0.8f, 1.4f}, {0.4f, 0.1f, 0.05f}, up, c0))
        return fail("cusp");
    if (!shot(a, mesh_of(cross, FrameKind::Parallel, ShapeKind::Tube, false), {1.3f, 1.0f, 1.6f}, {0, 0, 0}, up, c1))
        return fail("cross");
    if (!shot(a, mesh_of(loop, FrameKind::Parallel, ShapeKind::Ribbon, true), {1.5f, 1.2f, 1.6f}, {0, 0, 0.1f}, up, c2))
        return fail("loop");
    auto fails = hstack({c0, c1, c2});
    if (!write_png("out/spatial-fail.png", W * 3, H, fails.data())) return fail("png fail");

    auto sharp = sharp_corner();
    auto zup_s = samples_of(diag, FrameKind::ZUp, false);
    auto fr_s = samples_of(diag, FrameKind::Frenet, false);
    auto pa_s = samples_of(diag, FrameKind::Parallel, false);
    auto cusp_s = samples_of(cusp, FrameKind::Parallel, false);
    auto cross_s = samples_of(cross, FrameKind::Parallel, false);
    float cross_clear = min_center_clearance(cross_s);

    float4 controls[16] = {};
    for (int i = 0; i < 4; ++i)
        for (int k = 0; k < 4; ++k) controls[i * 4 + k] = float4{bend[i].p[k].x, bend[i].p[k].y, bend[i].p[k].z, 1};
    a.gpu_mi->setParameter("controls", controls, 16);
    a.gpu_mi->setParameter("samples", (float)SAMPLES);
    a.gpu_mi->setParameter("sides", (float)SIDES);
    a.gpu_mi->setParameter("radius", TUBE_R);
    Mesh gpu_topo;
    int grings = 4 * SAMPLES;
    gpu_topo.v.assign((size_t)grings * SIDES * 7, 0.f);
    for (int i = 0; i < grings; ++i) {
        int i1 = i + 1;
        if (i1 >= grings) break;
        for (int k = 0; k < SIDES; ++k) {
            uint32_t a0 = (uint32_t)(i * SIDES + k);
            uint32_t b0 = (uint32_t)(i * SIDES + (k + 1) % SIDES);
            uint32_t c0i = (uint32_t)(i1 * SIDES + k);
            uint32_t d0 = (uint32_t)(i1 * SIDES + (k + 1) % SIDES);
            gpu_topo.idx.insert(gpu_topo.idx.end(), {a0, b0, c0i, b0, d0, c0i});
        }
    }
    for (size_t i = 3; i < gpu_topo.v.size(); i += 7) gpu_topo.v[i + 3] = 1.f;
    look(a, eye_a, {0.1f, 0, 0.3f}, up);
    utils::Entity ge = upload(a, gpu_topo, a.gpu_mi);
    a.engine->flushAndWait();
    std::vector<uint8_t> gpu_pic, cpu_pic;
    if (!grab(a, gpu_pic) || !grab(a, gpu_pic)) return fail("gpu draw");
    a.scene->remove(ge);
    auto cpu_z = mesh_of(bend, FrameKind::ZUp, ShapeKind::Tube, false);
    if (!shot(a, cpu_z, eye_a, {0.1f, 0, 0.3f}, up, cpu_pic)) return fail("cpu zup");
    double mae = 0;
    int far_px = 0;
    for (int i = 0; i < W * H; ++i) {
        int dsum = 0;
        for (int ch = 0; ch < 3; ++ch) {
            int d = std::abs((int)gpu_pic[(size_t)i * 4 + ch] - (int)cpu_pic[(size_t)i * 4 + ch]);
            mae += d;
            dsum += d;
        }
        if (dsum > 80) ++far_px;
    }
    mae /= (double)W * H * 3;
    if (!write_png("out/spatial-gpu.png", W, H, gpu_pic.data())) return fail("png gpu");
    if (!write_png("out/spatial-gpu-cpu.png", W, H, cpu_pic.data())) return fail("png gpu cpu");
    bool gpu_follows = mae < 18.0 && far_px < W * H / 5;

    FILE* log = std::fopen("out/spatial.txt", "w");
    std::fprintf(log, "3D Path。曲線は三次ベジェ。2D は ThorVG、描画は Filament。3D の制御点はこの検証の中にある。\n\n");
    std::fprintf(log, "課題1 空間曲線\n");
    std::fprintf(log, "編集用の開曲線は 5 点からの Catmull-Rom、4 スパン。点 3 を y+0.20 z+0.55 動かした。\n");
    std::fprintf(log, "移動前 C0 %.6f  C1 %.2f 度。移動後 C0 %.6f  C1 %.2f 度。\n", c0_gap(bend), c1_deg(bend), c0_gap(moved), c1_deg(moved));
    std::fprintf(log, "同じメッシュのハッシュ %llx と %llx。カメラ 3 つで out/spatial-orbit.png。\n", (unsigned long long)hash_a,
                 (unsigned long long)hash_b);
    std::fprintf(log, "Z=0 の矩形を ThorVG に書いて読み戻した XY 差 %.6f。\n", xy_delta);
    std::fprintf(log, "out/spatial-edit.png 左が移動前、右が移動後。\n\n");

    std::fprintf(log, "課題2 曲線から立体\n");
    std::fprintf(log, "診断曲線は直線、XZ の曲がり、Z 方向の直線、その先の曲がり。C0 %.6f C1 %.2f 度。\n", c0_gap(diag), c1_deg(diag));
    std::fprintf(log, "中心線の誤差 Z-up %.6f Frenet %.6f Parallel %.6f。\n", spine_error(diag, zup_s, SAMPLES), spine_error(diag, fr_s, SAMPLES),
                 spine_error(diag, pa_s, SAMPLES));
    std::fprintf(log, "枠が落ちたサンプル Z-up %d / %d、Frenet %d、Parallel %d。\n", count_fail(zup_s), (int)zup_s.size(), count_fail(fr_s),
                 count_fail(pa_s));
    std::fprintf(log, "法線の反転 Z-up %d、Frenet %d、Parallel %d。白は枠か接線が落ちた輪。\n", count_flips(zup_s), count_flips(fr_s),
                 count_flips(pa_s));
    std::fprintf(log, "out/spatial-frames.png は帯。左 Z-up、中 Frenet、右 Parallel。out/spatial-shapes.png は Parallel の管、帯、四角断面。\n");
    std::fprintf(log, "ハンドルを折った角の C1 は %.2f 度。折る前の診断曲線は %.2f 度。\n", c1_deg(sharp), c1_deg(diag));
    std::fprintf(log, "尖点の接線ゼロは %d サンプル。交差する 2 本の中心距離 %.3f、管の直径 %.3f。\n",
                 (int)std::count_if(cusp_s.begin(), cusp_s.end(), [](const Sample& s) { return s.zero_tangent; }), cross_clear, TUBE_R * 2);
    std::fprintf(log, "閉曲線の Parallel ホロノミー %.2f 度。Frenet %.2f 度。out/spatial-fail.png 左が尖点、中が交差、右が閉曲線の帯。\n\n",
                 holonomy_deg(loop, FrameKind::Parallel, SAMPLES), holonomy_deg(loop, FrameKind::Frenet, SAMPLES));

    std::fprintf(log, "課題3 2D と 3D\n");
    std::fprintf(log, "矩形の Z を戻した制御点差 %.6f。ThorVG との XY 差 %.6f。\n", back_delta, xy_delta);
    std::fprintf(log, "out/spatial-roundtrip.png は左上 2D、右上 Z=0 の管、左下 Z を持ち上げた管、右下はそのカメラへの投影ガイド。\n");
    std::fprintf(log, "投影ガイドは 3D 曲線そのものではない。カメラを変えるとガイドの画素差は %d。メッシュハッシュは %s。\n", guide_far,
                 hash_a == hash_b ? "一致" : "不一致");
    std::fprintf(log, "out/spatial-guides.png は同じ 3D 曲線を別カメラで投影した 2 枚。\n\n");

    std::fprintf(log, "課題4 負荷\n");
    std::fprintf(log, "GPU 評価は 4 スパン、断面 %d、輪 %d。制御点は float4[16]。頂点位置は原点で、曲線は materialVertex が足す。\n", SIDES,
                 SAMPLES);
    std::fprintf(log, "GPU と CPU Z-up の画素差 mean abs %.2f、差の大きい画素 %d / %d。%s\n", mae, far_px, W * H,
                 gpu_follows ? "シルエットは追従している。" : "追従したとは言えない。out/spatial-gpu.png を正としない。");

    std::fprintf(log, "CPU は Parallel の管。samples %d sides %d。時間はミリ秒。\n", SAMPLES, SIDES);
    for (int spans : {8, 64, 256, 1024, 4096}) {
        auto curve = stress_curve(spans);
        auto t0 = std::chrono::steady_clock::now();
        auto samp = sample_curve(curve, FrameKind::Parallel, false, SAMPLES);
        double frame_ms = ms_since(t0);
        auto t1 = std::chrono::steady_clock::now();
        Mesh mesh = build_shape(samp, ShapeKind::Tube, false);
        double mesh_ms = ms_since(t1);
        auto t2 = std::chrono::steady_clock::now();
        utils::Entity e = upload(a, mesh, a.mi);
        a.engine->flushAndWait();
        double upload_ms = ms_since(t2);
        look(a, {0.2f, 1.4f, 2.4f}, {0, 0, 0}, up);
        auto t3 = std::chrono::steady_clock::now();
        for (int i = 0; i < 3; ++i) {
            if (!a.renderer->beginFrame(a.swap)) continue;
            a.renderer->render(a.view);
            a.renderer->endFrame();
        }
        a.engine->flushAndWait();
        double draw_ms = ms_since(t3) / 3.0;
        a.scene->remove(e);
        std::fprintf(log, "spans %d  verts %d  枠 %.2f  メッシュ %.2f  転送 %.2f  描画 %.2f\n", spans, (int)(mesh.v.size() / 7), frame_ms,
                     mesh_ms, upload_ms, draw_ms);
    }
    std::fprintf(log, "\n採用\n");
    std::fprintf(log, "2D のパスと投影ガイドは ThorVG。3D の制御点は Z を持つ三次で、Z=0 のとき 2D と一致する。\n");
    std::fprintf(log, "断面の向きは Parallel。Z-up は接線が Z に沿うと落ち、Frenet は曲率ゼロで落ちる。尖点は接線が消える。閉曲線はホロノミーで帯の色が食い違う。\n");
    std::fprintf(log, "管、帯、四角の掃引は同じサンプル列から出る。Filament はできたメッシュを描く。新しいシーン系は要らない。\n");
    if (gpu_follows)
        std::fprintf(log, "4 スパンまでは、制御点を引数配列で渡し、頂点シェーダが曲線と Z-up 断面を足せた。枠の輸送は CPU の方が安定する。\n");
    else
        std::fprintf(log, "頂点シェーダでの曲線評価は、この描画では CPU の管と一致しなかった。成立しているのは CPU で断面まで作る経路。\n");
    std::fclose(log);

    std::printf("xy %.6f  back %.6f  hash %s  guide_far %d  gpu mae %.2f far %d\n", xy_delta, back_delta, hash_a == hash_b ? "same" : "diff",
                guide_far, mae, far_px);
    std::printf("PASS spatial\n");
    return 0;
}
