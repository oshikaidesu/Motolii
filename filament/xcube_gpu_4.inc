// xcube_gpu.mm, piece 4 (cut there by size, as it was): read only in its place in xcube_gpu.mm.
    {
        const float s = 0.16f;
        const V3 c0{0.f, 0.22f, 0.f};
        auto rot = [](V3 p) {
            float cx = std::cos(0.55f), sx = std::sin(0.55f);
            V3 a{p.x, cx * p.y - sx * p.z, sx * p.y + cx * p.z};
            float cy = std::cos(0.4f), sy = std::sin(0.4f);
            return V3{cy * a.x + sy * a.z, a.y, -sy * a.x + cy * a.z};
        };
        V3 v[6] = {c0 + rot({0, s, 0}), c0 + rot({0, -s, 0}), c0 + rot({s, 0, 0}), c0 + rot({-s, 0, 0}), c0 + rot({0, 0, s}), c0 + rot({0, 0, -s})};
        int id[8][3] = {{0, 2, 4}, {0, 4, 3}, {0, 3, 5}, {0, 5, 2}, {1, 4, 2}, {1, 3, 4}, {1, 5, 3}, {1, 2, 5}};
        std::vector<float> cv;
        std::vector<uint32_t> ci;
        for (int i = 0; i < 8; ++i) {
            V3 a = v[id[i][0]], b = v[id[i][1]], c = v[id[i][2]];
            V3 n = cross(b - a, c - a);
            if (dot(n, (a + b + c) * (1.f / 3.f) - c0) < 0.f) {
                std::swap(b, c);
                n = n * -1.f;
            }
            V3 T = norm(b - a);
            uint32_t base = (uint32_t)(cv.size() / 7);
            push_vert(cv, a, T, n);
            push_vert(cv, b, T, n);
            push_vert(cv, c, T, n);
            ci.insert(ci.end(), {base, base + 1, base + 2});
        }
        VertexBuffer* crystalVB = VertexBuffer::Builder().vertexCount((uint32_t)(cv.size() / 7)).bufferCount(1)
                                      .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                                      .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                                      .build(*engine);
        crystalVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(cv.data(), cv.size() * sizeof(float)));
        IndexBuffer* crystalIB = IndexBuffer::Builder().indexCount((uint32_t)ci.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        crystalIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(ci.data(), ci.size() * sizeof(uint32_t)));
        utils::Entity crystal = utils::EntityManager::get().create();
        RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, crystalVB, crystalIB)
            .material(0, glass).culling(false).castShadows(true).receiveShadows(false)
            .build(*engine, crystal);
        glass->setParameter("thickness", 0.32f);
        glass->setParameter("dispersion", 48.f);
        scene->remove(cube);
        scene->remove(cubeBevel);
        scene->addEntity(crystal);
        view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
        view->setPostProcessingEnabled(true);
        view->setScreenSpaceRefractionEnabled(true);
        camera->setExposure(2.8f, 1.f / 180.f, 100.f);
        camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);
        camera->lookAt(float3{0.42f, 0.5f, 0.48f}, float3{0.f, 0.18f, 0.f}, float3{0, 1, 0});
        if (!grab("out/xcube-crystal.png")) return fail("crystal");
        scene->remove(crystal);
        glass->setParameter("thickness", half * 2.f);
        glass->setParameter("dispersion", dispersion);
        camera->lookAt(float3{0.55f, 0.78f, 0.70f}, float3{0, 0.04f, 0}, float3{0, 1, 0});
    }
    {
        V3 D = sun0;
        V3 up = std::fabs(D.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0};
        V3 U = norm(cross(up, D));
        V3 V = cross(D, U);
        octaMi->setParameter("dir", float3{D.x, D.y, D.z});
        octaMi->setParameter("tangent", float3{U.x, U.y, U.z});
        octaMi->setParameter("bitangent", float3{V.x, V.y, V.z});
        rm.setMaterialInstanceAt(rm.getInstance(hitE), 0, octaMi);
        if (!draw_hits() || !draw_paths() || !draw_atrous()) return fail("octa-caustic");
        rm.setMaterialInstanceAt(rm.getInstance(hitE), 0, hitMi);
        auto n_at = [](float nm, float disp) {
            float offset = 0.70795215f + (nm - 486.1f) / (656.3f - 486.1f) * (-0.29204785f - 0.70795215f);
            float P = disp / 20.f;
            return std::max(1.05f, 2.4f + P * (2.4f - 1.f) * offset);
        };
        const float dispAbbe = 20.f / 55.f;
        std::printf("n ior 2.4 dispersion 36     650 %.4f  550 %.4f  450 %.4f\n", n_at(650.f, 36.f), n_at(550.f, 36.f), n_at(450.f, 36.f));
        std::printf("n ior 2.4 dispersion 20/55  650 %.4f  550 %.4f  450 %.4f\n", n_at(650.f, dispAbbe), n_at(550.f, dispAbbe), n_at(450.f, dispAbbe));
        auto dump_hdr = [&](RenderTarget* rt, int w, int h, const char* path) -> bool {
            std::vector<float> px((size_t)w * h * 4, 0.f);
            if (!open()) return false;
            renderer->readPixels(rt, 0, 0, (uint32_t)w, (uint32_t)h,
                                 backend::PixelBufferDescriptor(px.data(), px.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
            renderer->endFrame();
            engine->flushAndWait();
            int zero = 0;
            float maxv = 0.f;
            for (int i = 0; i < w * h; ++i) {
                float m = std::max(px[(size_t)i * 4], std::max(px[(size_t)i * 4 + 1], px[(size_t)i * 4 + 2]));
                if (m > maxv) maxv = m;
                if (m < 1e-4f) zero++;
            }
            std::printf("%s  max %.4f  zero %d / %d\n", path, maxv, zero, w * h);
            std::vector<uint8_t> img((size_t)w * h * 4);
            float denom = std::log1p(std::max(maxv, 1e-6f));
            for (int i = 0; i < w * h; ++i) {
                float r = px[(size_t)i * 4], g = px[(size_t)i * 4 + 1], b = px[(size_t)i * 4 + 2];
                float m = std::max(r, std::max(g, b));
                uint8_t* o = img.data() + (size_t)i * 4;
                if (m < 1e-4f) {
                    o[0] = 180; o[1] = 0; o[2] = 180; o[3] = 255;
                } else {
                    float s = std::log1p(m) / denom;
                    float L = r + g + b;
                    o[0] = (uint8_t)std::clamp(r / L * s * 255.f, 0.f, 255.f);
                    o[1] = (uint8_t)std::clamp(g / L * s * 255.f, 0.f, 255.f);
                    o[2] = (uint8_t)std::clamp(b / L * s * 255.f, 0.f, 255.f);
                    o[3] = 255;
                }
            }
            return write_png(path, w, h, img.data());
        };
        if (!dump_hdr(mapRT, MAP, MAP, "out/xcube-caustic-map.png")) return fail("caustic-map");
        if (!dump_hdr(filtRT, MAP, MAP, "out/xcube-caustic-filt.png")) return fail("caustic-filt");
        auto face_dir = [](int face, float u, float v) {
            switch (face) {
            case 0: return norm({1.f, v, -u});
            case 1: return norm({-1.f, v, u});
            case 2: return norm({u, 1.f, -v});
            case 3: return norm({u, -1.f, v});
            case 4: return norm({u, v, 1.f});
            default: return norm({-u, v, -1.f});
            }
        };
        auto wrap_pi = [](float a) {
            const float pi = 3.14159265f;
            while (a > pi) a -= 2.f * pi;
            while (a < -pi) a += 2.f * pi;
            return a;
        };
        auto slit = [&](V3 d) {
            float az = std::atan2(d.x, d.z);
            float3 c{0.f, 0.f, 0.f};
            const float azs[12] = {-2.7f, -2.2f, -1.7f, -1.2f, -0.75f, -0.3f, 0.15f, 0.55f, 1.0f, 1.45f, 1.9f, 2.4f};
            for (int k = 0; k < 12; ++k) {
                float w = (k % 2 == 0) ? 0.028f : 0.04f;
                if (std::fabs(wrap_pi(az - azs[k])) < w && d.y > -0.85f && d.y < 0.92f) {
                    if (k % 4 == 1) c = float3{46.f, 16.f, 8.f};
                    else if (k % 4 == 3) c = float3{8.f, 34.f, 42.f};
                    else c = float3{42.f, 44.f, 48.f};
                }
            }
            return c;
        };
        static std::vector<std::vector<float>> envKeep;
        envKeep.clear();
        auto upload_env = [&](int RF, bool withMips) -> Texture* {
            int levels = 1;
            if (withMips) {
                for (int n = RF; n > 1; n /= 2) levels++;
            }
            Texture* tex = Texture::Builder().width((uint32_t)RF).height((uint32_t)RF).levels((uint8_t)levels)
                               .sampler(Texture::Sampler::SAMPLER_CUBEMAP)
                               .format(Texture::InternalFormat::RGBA16F)
                               .usage(Texture::Usage::DEFAULT)
                               .build(*engine);
            for (int face = 0; face < 6; ++face) {
                envKeep.emplace_back((size_t)RF * RF * 4);
                auto& buf = envKeep.back();
                for (int y = 0; y < RF; ++y) {
                    float fv = ((y + 0.5f) / RF) * 2.f - 1.f;
                    for (int x = 0; x < RF; ++x) {
                        float fu = ((x + 0.5f) / RF) * 2.f - 1.f;
                        float3 c = slit(face_dir(face, fu, fv));
                        size_t i = ((size_t)y * RF + x) * 4;
                        buf[i] = c.x; buf[i + 1] = c.y; buf[i + 2] = c.z; buf[i + 3] = 1.f;
                    }
                }
                tex->setImage(*engine, 0, 0, 0, (uint32_t)face, (uint32_t)RF, (uint32_t)RF, 1,
                              Texture::PixelBufferDescriptor(buf.data(), buf.size() * sizeof(float), Texture::Format::RGBA, Texture::Type::FLOAT));
                if (!withMips) continue;
                int n = RF;
                for (int level = 1; level < levels; ++level) {
                    int m = n / 2;
                    envKeep.emplace_back((size_t)m * m * 4);
                    auto& src = envKeep[envKeep.size() - 2];
                    auto& dst = envKeep.back();
                    for (int y = 0; y < m; ++y) {
                        for (int x = 0; x < m; ++x) {
                            for (int c = 0; c < 4; ++c) {
                                float s = src[((size_t)(y * 2) * n + x * 2) * 4 + c];
                                s += src[((size_t)(y * 2) * n + x * 2 + 1) * 4 + c];
                                s += src[((size_t)(y * 2 + 1) * n + x * 2) * 4 + c];
                                s += src[((size_t)(y * 2 + 1) * n + x * 2 + 1) * 4 + c];
                                dst[((size_t)y * m + x) * 4 + c] = s * 0.25f;
                            }
                        }
                    }
                    tex->setImage(*engine, (size_t)level, 0, 0, (uint32_t)face, (uint32_t)m, (uint32_t)m, 1,
                                  Texture::PixelBufferDescriptor(dst.data(), dst.size() * sizeof(float), Texture::Format::RGBA, Texture::Type::FLOAT));
                    n = m;
                }
            }
            engine->flushAndWait();
            return tex;
        };
        Texture* env128 = upload_env(128, false);
        Texture* env1024 = upload_env(1024, false);
        Texture* envMip = upload_env(1024, true);
        TextureSampler envLinear(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
        TextureSampler envMipSamp(TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR, TextureSampler::MagFilter::LINEAR);
        dreiMi->setParameter("env", env128, envLinear);
        dreiMi->setParameter("chroma", 1.f);
        dreiMi->setParameter("dispersion", dispersion);
        dreiMi->setParameter("fresnel", 0.f);
        dreiMi->setParameter("grad", 0.f);
        dreiMi->setParameter("bounces", 3.f);
        scene->remove(filter);
        scene->remove(splat);
        scene->remove(hitE);
        scene->remove(cube);
        scene->remove(cubeBevel);
        scene->addEntity(floor);
        scene->addEntity(cubeDrei);
        scene->addEntity(sun_e);
        scene->setIndirectLight(nullptr);
        scene->setSkybox(skyGem);
        floorMi->setParameter("color", float4{0.90f, 0.89f, 0.88f, 1});
        view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
        view->setPostProcessingEnabled(true);
        view->setScreenSpaceRefractionEnabled(false);
        view->setShadowType(View::ShadowType::PCF);
        view->setShadowingEnabled(true);
        BloomOptions lean = bloom;
        lean.strength = 0.08f;
        view->setBloomOptions(lean);
        camera->setExposure(2.8f, 1.f / 180.f, 100.f);
        camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);
        float3 eye{0.62f, 0.78f, 0.70f};
        camera->lookAt(eye, float3{0.f, 0.12f, 0.f}, float3{0, 1, 0});
        dreiMi->setParameter("eye", eye);
        if (!grab("out/xcube-gem.png")) return fail("gem");
        view->setShadowingEnabled(false);
        if (!grab("out/xcube-gem-noshadow.png")) return fail("gem-noshadow");
        view->setShadowingEnabled(true);
        dreiMi->setParameter("env", env1024, envLinear);
        if (!grab("out/xcube-gem-a.png")) return fail("gem-a");
        dreiMi->setParameter("env", env128, envLinear);
        dreiMi->setParameter("dispersion", dispAbbe);
        if (!grab("out/xcube-gem-b.png")) return fail("gem-b");
        dreiMi->setParameter("dispersion", dispersion);
        dreiMi->setParameter("env", envMip, envMipSamp);
        dreiMi->setParameter("grad", 1.f);
        if (!grab("out/xcube-gem-c.png")) return fail("gem-c");
        dreiMi->setParameter("grad", 0.f);
        dreiMi->setParameter("env", env128, envLinear);
        scene->remove(floor);
        time_beauty("gem only");
        scene->addEntity(floor);
        scene->remove(cubeDrei);
        scene->setSkybox(sky);
    }
    std::printf("ibl intensity %.0f  reflection cameras 0\n", kIbl);
    scene->remove(cubeBevel);
    scene->remove(filmAf);
    scene->remove(filmBf);
    scene->addEntity(cube);
    show_films(true);
    if (!bake(0.f, 0.f)) return fail("flat");

    float dist = std::sqrt(0.55f * 0.55f + 0.70f * 0.70f);
    for (int f = 0; f < 12; ++f) {
        float yaw = 6.283185f * (float)f / 12.f;
        camera->lookAt(float3{dist * std::sin(yaw), 0.78f, dist * std::cos(yaw)}, float3{0, 0.08f, 0}, float3{0, 1, 0});
        char path[64];
        std::snprintf(path, sizeof(path), "out/xcube-rw-orbit-%02d.png", f);
        if (!grab(path)) return fail("orbit");
    }
    std::printf("PASS xcube-gpu\n");
    return 0;
}
