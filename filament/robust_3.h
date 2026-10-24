// robust.mm, part 3 (cut there by size, as it was): read only through robust.mm, in order.
#pragma once


int main() {
    auto spatial_pkg = load_file("spatial.filamat");
    auto gpu_pkg = load_file("robust_gpu.filamat");
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
    a.gpu = gpu_mat->createInstance();

    const float3 up{0, 1, 0};
    const float half = 0.075f;
    auto loop = closed_loop();
    auto open_frames = sample_uniform(loop, 16, true);
    Shut shut = close_arc(open_frames);
    Shut planar_shut = close_arc(sample_uniform(planar_rect(), 8, true));
    const float deg = 57.2957795f;
    float before = shut.before * deg;
    float join = shut.join * deg;
    float last_step = shut.last_step * deg;
    float planar_before = planar_shut.before * deg;
    float planar_join = planar_shut.join * deg;

    std::vector<uint8_t> c0, c1, c2, c3;
    const float3 loop_eye{1.55f, 1.25f, 1.55f};
    const float3 loop_at{0, 0, 0.1f};
    if (!shot(a, ribbon(open_frames, half, true), loop_eye, loop_at, up, c0)) return fail("seam before");
    if (!shot(a, ribbon(shut.frames, half, false), loop_eye, loop_at, up, c1)) return fail("seam after");
    auto seam = hstack(c0, c1);
    if (!write_png("out/robust-seam.png", W * 2, H, seam.data())) return fail("png seam");

    auto cusp = sample_uniform(cusp_curve(), 24, false);
    auto corners = sample_uniform(planar_rect(), 8, true);
    if (!shot(a, tube_colored(cusp, 0.045f, 16, false), {1.15f, 0.75f, 1.15f}, {0.4f, 0.12f, 0.04f}, up, c0)) return fail("cusp");
    if (!shot(a, tube_colored(corners, 0.045f, 12, false), {0.2f, 1.35f, 1.5f}, {0, 0, 0}, up, c1)) return fail("corners");
    auto cusp_pic = hstack(c0, c1);
    if (!write_png("out/robust-cusp.png", W * 2, H, cusp_pic.data())) return fail("png cusp");

    auto bend = local_bend();
    const float tol = 0.01f;
    const float adapt_r = 0.04f;
    std::vector<Sample> coarse;
    for (int s = 0; s < (int)bend.size(); ++s) push_sample(coarse, bend[s], 0, !coarse.empty(), coarse.empty() ? Sample{} : coarse.back());
    push_sample(coarse, bend.back(), 1, true, coarse.back());
    auto adapted = sample_adaptive(bend, adapt_r, tol);
    int coarse_bend = bend_rings(coarse, -0.02f, 0.12f, 0.12f);
    int adapt_bend = bend_rings(adapted, -0.02f, 0.12f, 0.12f);
    int coarse_straight = (int)coarse.size() - coarse_bend;
    int adapt_straight = (int)adapted.size() - adapt_bend;
    const float3 bend_eye{1.1f, 0.9f, 1.3f};
    const float3 bend_at{0.0f, 0.15f, 0};
    if (!shot(a, tube_colored(coarse, adapt_r, 12, false), bend_eye, bend_at, up, c0)) return fail("coarse");
    if (!shot(a, tube_colored(adapted, adapt_r, 12, false), bend_eye, bend_at, up, c1)) return fail("adapt");
    auto adapt_pic = hstack(c0, c1);
    if (!write_png("out/robust-adapt.png", W * 2, H, adapt_pic.data())) return fail("png adapt");

    auto local_s = sample_uniform(bend, 32, false);
    Hit local_hit = classify(local_s, 0.09f, 0.25f);
    auto global_s = sample_uniform(global_pass(), 12, false);
    Hit global_hit = classify(global_s, 0.09f, 0.5f);
    const float3 local_eye{0.85f, 0.7f, 1.05f};
    const float3 global_eye{1.6f, 1.5f, 2.2f};
    const float3 global_at{0, 0, 0};
    if (!shot(a, tube_colored(local_s, 0.09f, 16, false), local_eye, bend_at, up, c0)) return fail("local allow");
    if (!shot(a, tube_colored(local_s, local_hit.safe_r, 16, false), local_eye, bend_at, up, c1)) return fail("local resolve");
    if (!shot(a, tube_colored(global_s, 0.09f, 12, false), global_eye, global_at, up, c2)) return fail("global allow");
    if (!shot(a, tube_colored(global_s, global_hit.safe_r, 12, false), global_eye, global_at, up, c3)) return fail("global resolve");
    auto hits = grid2(c0, c1, c2, c3);
    if (!write_png("out/robust-hit.png", W * 2, H * 2, hits.data())) return fail("png hit");

    auto picture_curve = stress_curve(64);
    const int pic_samples = 8;
    const int pic_sides = 12;
    const float pic_r = 0.045f;
    const int tex_w = 256;
    auto picture_frames = sample_uniform(picture_curve, pic_samples, false);
    int tex_h = tex_rows((int)picture_curve.size() * 4, tex_w);
    int frame_h = tex_rows((int)picture_frames.size() * 2, tex_w);
    std::vector<float> texels, frame_texels;
    pack_controls(picture_curve, tex_w, tex_h, texels);
    pack_frames(picture_frames, tex_w, frame_h, frame_texels);
    Texture* controls = float_tex(a.engine, tex_w, tex_h, texels);
    Texture* frames_tex = float_tex(a.engine, tex_w, frame_h, frame_texels);
    if (!controls || !frames_tex) return fail("control texture");
    TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    a.gpu->setParameter("controls", controls, nearest);
    a.gpu->setParameter("frames", frames_tex, nearest);
    a.gpu->setParameter("samples", (float)pic_samples);
    a.gpu->setParameter("sides", (float)pic_sides);
    a.gpu->setParameter("spans", (float)picture_curve.size());
    a.gpu->setParameter("radius", pic_r);
    a.gpu->setParameter("texWidth", (float)tex_w);
    a.gpu->setParameter("texHeight", (float)tex_h);
    a.gpu->setParameter("frameHeight", (float)frame_h);
    Mesh topo = gpu_topology((int)picture_frames.size(), pic_sides);
    const float3 gpu_eye{1.7f, 1.35f, 1.9f};
    a.camera->lookAt(gpu_eye, {0, 0, 0}, up);
    utils::Entity ge = upload(a, topo, a.gpu);
    a.engine->flushAndWait();
    std::vector<uint8_t> gpu_pic, cpu_pic;
    if (!grab(a, gpu_pic) || !grab(a, gpu_pic)) return fail("gpu draw");
    a.scene->remove(ge);
    if (!shot(a, tube_colored(picture_frames, pic_r, pic_sides, false), gpu_eye, {0, 0, 0}, up, cpu_pic)) return fail("cpu parallel");
    int far_px = 0;
    double mae = mae_of(gpu_pic, cpu_pic, far_px);
    if (!write_png("out/robust-gpu.png", W * 2, H, hstack(gpu_pic, cpu_pic).data())) return fail("png gpu");

    auto vertical = vertical_curve();
    auto vertical_frames = sample_uniform(vertical, 16, false);
    int z_tangents = 0;
    for (const Sample& s : vertical_frames)
        if (std::fabs(s.T.z) > 0.98f) ++z_tangents;
    int z_h = tex_rows((int)vertical.size() * 4, tex_w);
    int z_fh = tex_rows((int)vertical_frames.size() * 2, tex_w);
    std::vector<float> z_controls, z_frames;
    pack_controls(vertical, tex_w, z_h, z_controls);
    pack_frames(vertical_frames, tex_w, z_fh, z_frames);
    Texture* z_ct = float_tex(a.engine, tex_w, z_h, z_controls);
    Texture* z_ft = float_tex(a.engine, tex_w, z_fh, z_frames);
    if (!z_ct || !z_ft) return fail("vertical texture");
    a.gpu->setParameter("controls", z_ct, nearest);
    a.gpu->setParameter("frames", z_ft, nearest);
    a.gpu->setParameter("samples", 16.f);
    a.gpu->setParameter("sides", 12.f);
    a.gpu->setParameter("spans", (float)vertical.size());
    a.gpu->setParameter("radius", 0.05f);
    a.gpu->setParameter("texHeight", (float)z_h);
    a.gpu->setParameter("frameHeight", (float)z_fh);
    const float3 z_eye{1.55f, 1.25f, 0.7f};
    const float3 z_at{0.2f, 0.1f, 0.75f};
    a.camera->lookAt(z_eye, z_at, up);
    Mesh z_topo = gpu_topology((int)vertical_frames.size(), 12);
    utils::Entity ze = upload(a, z_topo, a.gpu);
    a.engine->flushAndWait();
    std::vector<uint8_t> z_gpu, z_cpu;
    if (!grab(a, z_gpu) || !grab(a, z_gpu)) return fail("vertical gpu");
    a.scene->remove(ze);
    if (!shot(a, tube_colored(vertical_frames, 0.05f, 12, false), z_eye, z_at, up, z_cpu)) return fail("vertical cpu");
    int z_far = 0;
    double z_mae = mae_of(z_gpu, z_cpu, z_far);
    if (!write_png("out/robust-frame.png", W * 2, H, hstack(z_gpu, z_cpu).data())) return fail("png frame");

    FILE* log = std::fopen("out/robust.txt", "w");
    std::fprintf(log, "3D Path の堅牢性。成立済みの経路の上で、継ぎ目、尖点、分割、制御点の枚数、自己交差を測る。\n\n");
    std::fprintf(log, "継ぎ目\n");
    std::fprintf(log, "補正前の接合 %.2f 度。これは最後のサンプルを始点の接線まで運んだ角で、終点の輪はまだ無い。\n", before);
    std::fprintf(log, "弧長比例の回転を最後の区間まで載せ、終点の輪を始点と同じ位置に置いた。始点と終点の断面角 %.8f 度。縁のずれ %.8f。\n", join,
                 shut.edge);
    std::fprintf(log, "最後の区間に残るねじれは %.2f 度。これは分配の一段で、始点と終点の不一致ではない。\n", last_step);
    std::fprintf(log, "平面の矩形は補正前 %.8f 度、始点と終点 %.8f 度。角の接線は極限で回復した %d サンプル。\n", planar_before, planar_join,
                 count_recovered(corners));
    std::fprintf(log, "out/robust-seam.png 左が補正前、右が終点まで閉じた帯。\n\n");

    std::fprintf(log, "尖点\n");
    std::fprintf(log, "始点で制御点が重なる曲線。回復した接線 %d、枠の落下 %d / %d。帯の半幅の実長 %.4f。\n", count_recovered(cusp),
                 count_fail(cusp), (int)cusp.size(), edge_min(cusp, 0.045f));
    std::fprintf(log, "矩形の角はハンドルが端点にあり、一次導関数が 0。回復 %d、落下 %d / %d。\n", count_recovered(corners), count_fail(corners),
                 (int)corners.size());
    std::fprintf(log, "out/robust-cusp.png 左が尖点の管、右が矩形の角。白は枠が落ちた輪。\n\n");

    std::fprintf(log, "適応分割\n");
    std::fprintf(log, "許容は平坦さと半径×折れ角の両方で %.3f。半径 %.3f。\n", tol, adapt_r);
    std::fprintf(log, "スパンの端だけは輪 %d、曲がり付近 %d、それ以外 %d。\n", (int)coarse.size(), coarse_bend, coarse_straight);
    std::fprintf(log, "適応は輪 %d、曲がり付近 %d、それ以外 %d。\n", (int)adapted.size(), adapt_bend, adapt_straight);
    std::fprintf(log, "out/robust-adapt.png 左が等分割、右が適応。\n\n");

    std::fprintf(log, "自己交差\n");
    std::fprintf(log, "局所: 半径 0.090、最大曲率 %.3f、r*κ %.3f、%s。最近接（弧長 0.25 以上）%.3f。安全半径 %.4f。\n", local_hit.kappa_max,
                 0.09f * local_hit.kappa_max, local_hit.local ? "局所交差" : "局所は半径内", local_hit.separation, local_hit.safe_r);
    std::fprintf(log, "大域: 半径 0.090、最大曲率 %.3f、r*κ %.3f、%s。最近接（弧長 0.50 以上）%.3f、直径 0.180。安全半径 %.4f。\n",
                 global_hit.kappa_max, 0.09f * global_hit.kappa_max, global_hit.global ? "大域交差" : "大域は離れている", global_hit.separation,
                 global_hit.safe_r);
    std::fprintf(log, "許容は指定半径のまま描く。解消は安全半径への縮小で、太さを保った融合ではない。融合は別の形状演算として残す。\n");
    std::fprintf(log, "out/robust-hit.png 左上が局所の許容、右上が解消、左下が大域の許容、右下が解消。\n\n");

    std::fprintf(log, "GPU の断面\n");
    std::fprintf(log, "曲線の位置は頂点シェーダが制御点から計算する。断面の N と B は CPU の Parallel Transport を RGBA32F で渡す。\n");
    std::fprintf(log, "64 スパンの螺旋。GPU と CPU の画素差 mean abs %.2f、差の大きい画素 %d / %d。out/robust-gpu.png 左が GPU、右が CPU。\n", mae,
                 far_px, W * H);
    std::fprintf(log, "Z 方向の直線を含む曲線。|T.z|>0.98 は %d / %d。Parallel の落下 %d。GPU と CPU の画素差 %.2f、差の大きい画素 %d。\n",
                 z_tangents, (int)vertical_frames.size(), count_fail(vertical_frames), z_mae, z_far);
    std::fprintf(log, "out/robust-frame.png 左が GPU、右が CPU。どちらも同じ Parallel の枠。\n");
    std::fprintf(log, "時間はミリ秒。毎フレームは枠の再計算、制御点と枠のパック、テクスチャ転送、描画。トポロジは輪の数が変わらない限り一度。描画に readPixels は含まない。60fps の予算は 16.67 ms。\n");

    const double budget = 16.67;
    double worst = 0;
    for (int spans : {64, 256, 1024, 4096}) {
        auto curve = stress_curve(spans);
        int samples = 6;
        int sides = 8;
        auto t_frame = std::chrono::steady_clock::now();
        auto samp = sample_uniform(curve, samples, false);
        double frame_ms = ms_since(t_frame);
        int th = tex_rows((int)curve.size() * 4, tex_w);
        int fh = tex_rows((int)samp.size() * 2, tex_w);
        auto t_pack = std::chrono::steady_clock::now();
        pack_controls(curve, tex_w, th, texels);
        pack_frames(samp, tex_w, fh, frame_texels);
        double pack_ms = ms_since(t_pack);
        Texture* tex = float_tex(a.engine, tex_w, th, texels);
        Texture* ftex = float_tex(a.engine, tex_w, fh, frame_texels);
        a.engine->flushAndWait();
        auto t_up = std::chrono::steady_clock::now();
        tex->setImage(*a.engine, 0,
                      Texture::PixelBufferDescriptor(texels.data(), texels.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                     backend::PixelDataType::FLOAT, 1, 0, 0, (uint32_t)tex_w));
        ftex->setImage(*a.engine, 0,
                       Texture::PixelBufferDescriptor(frame_texels.data(), frame_texels.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                      backend::PixelDataType::FLOAT, 1, 0, 0, (uint32_t)tex_w));
        a.engine->flushAndWait();
        double upload_ms = ms_since(t_up);
        a.gpu->setParameter("controls", tex, nearest);
        a.gpu->setParameter("frames", ftex, nearest);
        a.gpu->setParameter("samples", (float)samples);
        a.gpu->setParameter("sides", (float)sides);
        a.gpu->setParameter("spans", (float)curve.size());
        a.gpu->setParameter("radius", pic_r);
        a.gpu->setParameter("texWidth", (float)tex_w);
        a.gpu->setParameter("texHeight", (float)th);
        a.gpu->setParameter("frameHeight", (float)fh);
        auto t_topo = std::chrono::steady_clock::now();
        Mesh top = gpu_topology((int)samp.size(), sides);
        double topo_ms = ms_since(t_topo);
        a.camera->lookAt(gpu_eye, {0, 0, 0}, up);
        utils::Entity e = upload(a, top, a.gpu);
        a.engine->flushAndWait();
        auto t_draw = std::chrono::steady_clock::now();
        int drew = 0;
        for (int i = 0; i < 3; ++i) {
            if (!a.renderer->beginFrame(a.swap)) continue;
            a.renderer->render(a.view);
            a.renderer->endFrame();
            ++drew;
        }
        a.engine->flushAndWait();
        double draw_ms = drew ? ms_since(t_draw) / (double)drew : -1;
        a.scene->remove(e);
        double per_frame = frame_ms + pack_ms + upload_ms + draw_ms;
        if (per_frame > worst) worst = per_frame;
        std::fprintf(log, "spans %d  rings %d  verts %d  枠 %.2f  パック %.2f  転送 %.2f  描画 %.2f  毎フレーム %.2f  トポロジ一度 %.2f  %s\n",
                     spans, (int)samp.size(), (int)(top.v.size() / 7), frame_ms, pack_ms, upload_ms, draw_ms, per_frame, topo_ms,
                     per_frame <= budget ? "予算内" : "予算超過");
    }
    std::fprintf(log, "毎フレームの最大は %.2f ms。トポロジの作り直しは含めていない。\n", worst);
    std::fclose(log);

    std::printf("join %.8f deg  edge %.8f  last_step %.2f  ztangents %d  zmae %.2f  gpu mae %.2f  worst %.2f\n", join, shut.edge, last_step,
                z_tangents, z_mae, mae, worst);
    std::printf("PASS robust\n");
    return 0;
}
