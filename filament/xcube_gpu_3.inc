// xcube_gpu.mm, piece 3 (cut there by size, as it was): read only in its place in xcube_gpu.mm.
    auto env_of = [&](int kind, V3 d) {
        if (kind == 0) return float3{0.72f, 0.74f, 0.78f};
        if (kind == 1) {
            float3 c{0.025f, 0.027f, 0.030f};
            c = add_strip(c, d, {0.05f, 1.f, 0.02f}, 0.055f, {1, 1, 1}, 4.5f);
            c = add_strip(c, d, {1.f, 0.08f, 0.15f}, 0.040f, {1, 1, 1}, 3.5f);
            c = add_strip(c, d, {0.2f, 0.15f, 1.f}, 0.045f, {0.95f, 0.97f, 1.f}, 3.0f);
            return c;
        }
        float3 c{0.006f, 0.006f, 0.008f};
        // Spots sit on the reflection directions of the six film facets.
        // Neighbors are 8 degrees apart, so a 0.045 rad spot hits one facet.
        c = add_spot(c, d, {-0.689f, -0.571f, 0.446f}, 0.045f, {1, 1, 1}, 22.f);
        c = add_spot(c, d, {-0.644f, -0.499f, 0.581f}, 0.040f, {1.f, 0.18f, 0.10f}, 16.f);
        c = add_spot(c, d, {0.710f, -0.632f, -0.312f}, 0.040f, {0.12f, 0.82f, 1.f}, 16.f);
        c = add_spot(c, d, {0.689f, -0.571f, -0.446f}, 0.045f, {1, 1, 1}, 22.f);
        c = add_spot(c, d, {0.644f, -0.499f, -0.581f}, 0.040f, {1.f, 0.70f, 0.16f}, 14.f);
        c = add_spot(c, d, {-0.142f, -0.118f, -0.983f}, 0.050f, {1, 1, 1}, 18.f);
        c = add_spot(c, d, {-0.968f, 0.100f, -0.228f}, 0.045f, {1, 1, 1}, 12.f);
        return c;
    };
    static std::vector<float> iblFace[3][6];
    Texture* iblTex[3] = {};
    IndirectLight* ibl[3] = {};
    for (int kind = 0; kind < 3; ++kind) {
        Texture* tex = Texture::Builder().width((uint32_t)kFace).height((uint32_t)kFace).levels(1)
                           .sampler(Texture::Sampler::SAMPLER_CUBEMAP)
                           .format(Texture::InternalFormat::RGBA16F)
                           .usage(Texture::Usage::DEFAULT)
                           .build(*engine);
        for (int face = 0; face < 6; ++face) {
            auto& buf = iblFace[kind][face];
            buf.resize((size_t)kFace * kFace * 4);
            for (int y = 0; y < kFace; ++y) {
                float v = ((y + 0.5f) / kFace) * 2.f - 1.f;
                for (int x = 0; x < kFace; ++x) {
                    float u = ((x + 0.5f) / kFace) * 2.f - 1.f;
                    V3 dir = face_dir(face, u, v);
                    float3 c = env_of(kind, dir);
                    if (kind == 2) {
                        c = float3{0.50f, 0.52f, 0.56f};
                        float az = std::atan2(dir.x, dir.z);
                        int sector = (int)std::floor((az + 3.14159265f) * (10.f / 6.2831853f));
                        if (sector < 0) sector = 0;
                        if (sector > 9) sector = 9;
                        if (std::fabs(dir.y) < 0.55f) {
                            c = (sector % 2 == 0) ? float3{22.f, 22.f, 23.f} : float3{0.f, 0.f, 0.f};
                        } else if (dir.y > 0.75f) {
                            c = float3{16.f, 16.f, 17.f};
                        } else if (dir.y < -0.75f) {
                            c = float3{0.f, 0.f, 0.f};
                        }
                    }
                    size_t i = ((size_t)y * kFace + x) * 4;
                    buf[i] = c.x;
                    buf[i + 1] = c.y;
                    buf[i + 2] = c.z;
                    buf[i + 3] = 1.f;
                }
            }
            tex->setImage(*engine, 0, 0, 0, (uint32_t)face, (uint32_t)kFace, (uint32_t)kFace, 1,
                          Texture::PixelBufferDescriptor(buf.data(), buf.size() * sizeof(float), Texture::Format::RGBA, Texture::Type::FLOAT));
        }
        iblTex[kind] = tex;
        ibl[kind] = IndirectLight::Builder().reflections(tex).intensity(kIbl).build(*engine);
    }
    Skybox* skyIbl = Skybox::Builder().color({0.72f / kIbl, 0.74f / kIbl, 0.78f / kIbl, 1.f}).build(*engine);
    auto use_ibl = [&](int kind) { scene->setIndirectLight(ibl[kind]); scene->setSkybox(skyIbl); };
    auto show_lit = [&](bool on) {
        scene->remove(filmAl);
        scene->remove(filmBl);
        if (on) {
            scene->addEntity(filmAl);
            scene->addEntity(filmBl);
        }
    };
    use_ibl(0);
    if (!grab("out/xcube-ibl-a.png")) return fail("ibl-a");
    use_ibl(1);
    if (!grab("out/xcube-ibl-b.png")) return fail("ibl-b");
    use_ibl(2);
    if (!grab("out/xcube-ibl-c.png")) return fail("ibl-c");
    time_beauty("IBL C unlit");
    scene->remove(filmAf);
    scene->remove(filmBf);
    if (!grab("out/xcube-ibl-c-glass.png")) return fail("ibl-glass");
    time_beauty("IBL C glass");
    show_lit(true);
    if (!grab("out/xcube-ibl-c-film.png")) return fail("ibl-film");
    time_beauty("IBL C lit");
    scene->addEntity(echoE);
    echoMi->setParameter("bounces", 1.f);
    if (!grab("out/xcube-ibl-c-g1.png")) return fail("ibl-g1");
    time_beauty("IBL C G1");
    echoMi->setParameter("bounces", 2.f);
    if (!grab("out/xcube-ibl-c-g2.png")) return fail("ibl-g2");
    time_beauty("IBL C G2");
    scene->remove(echoE);
    V3 focus{0.f, 0.08f, 0.f};
    V3 eye0{0.55f, 0.78f, 0.70f};
    V3 rel = eye0 - focus;
    float camDist = std::sqrt(dot(rel, rel));
    float baseYaw = std::atan2(rel.x, rel.z);
    float basePitch = std::asin(std::clamp(rel.y / camDist, -1.f, 1.f));
    for (int f = 0; f < 24; ++f) {
        float u = (float)f / 23.f;
        float yaw = baseYaw + 0.262f * std::sin(6.283185f * u);
        float pitch = basePitch + 0.140f * std::sin(6.283185f * u);
        float cp = std::cos(pitch), sp = std::sin(pitch);
        V3 eye = focus + V3{camDist * cp * std::sin(yaw), camDist * sp, camDist * cp * std::cos(yaw)};
        camera->lookAt(float3{eye.x, eye.y, eye.z}, float3{focus.x, focus.y, focus.z}, float3{0, 1, 0});
        char path[64];
        std::snprintf(path, sizeof(path), "out/xcube-ibl-spin-%02d.png", f);
        if (!grab(path)) return fail("ibl-spin");
    }
    camera->lookAt(float3{0.55f, 0.78f, 0.70f}, float3{0, 0.04f, 0}, float3{0, 1, 0});
    show_lit(false);
    std::vector<float> gem_v;
    std::vector<uint32_t> gem_i;
    {
        const float s = 0.2f;
        V3 toCam = norm({0.50f, 0.35f, 0.62f});
        V3 side = norm(cross({0.f, 1.f, 0.f}, toCam));
        V3 up2 = cross(toCam, side);
        const V3 c0{0.f, 0.2f, 0.f};
        V3 v[6] = {c0 + toCam * s, c0 - toCam * s, c0 + side * s, c0 - side * s, c0 + up2 * s, c0 - up2 * s};
        int id[8][3] = {{0, 2, 4}, {0, 4, 3}, {0, 3, 5}, {0, 5, 2}, {1, 4, 2}, {1, 3, 4}, {1, 5, 3}, {1, 2, 5}};
        auto tri = [&](V3 a, V3 b, V3 c) {
            V3 n = cross(b - a, c - a);
            V3 mid = (a + b + c) * (1.f / 3.f);
            if (dot(n, mid - c0) < 0.f) {
                std::swap(b, c);
                n = n * -1.f;
            }
            V3 T = norm(b - a);
            uint32_t base = (uint32_t)(gem_v.size() / 7);
            push_vert(gem_v, a, T, n);
            push_vert(gem_v, b, T, n);
            push_vert(gem_v, c, T, n);
            gem_i.insert(gem_i.end(), {base, base + 1, base + 2});
        };
        for (int i = 0; i < 8; ++i) tri(v[id[i][0]], v[id[i][1]], v[id[i][2]]);
    }
    VertexBuffer* gemVB = VertexBuffer::Builder().vertexCount((uint32_t)(gem_v.size() / 7)).bufferCount(1)
                              .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                              .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                              .build(*engine);
    gemVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(gem_v.data(), gem_v.size() * sizeof(float)));
    IndexBuffer* gemIB = IndexBuffer::Builder().indexCount((uint32_t)gem_i.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    gemIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(gem_i.data(), gem_i.size() * sizeof(uint32_t)));
    const uint32_t ntri = (uint32_t)(gem_i.size() / 3);
    static std::vector<float> triBuf;
    triBuf.assign((size_t)ntri * 3 * 4, 0.f);
    for (uint32_t t = 0; t < ntri; ++t) {
        for (int k = 0; k < 3; ++k) {
            uint32_t vi = gem_i[t * 3 + k];
            size_t o = ((size_t)t * 3 + k) * 4;
            triBuf[o] = gem_v[vi * 7];
            triBuf[o + 1] = gem_v[vi * 7 + 1];
            triBuf[o + 2] = gem_v[vi * 7 + 2];
            triBuf[o + 3] = 1.f;
        }
    }
    Texture* triTex = Texture::Builder().width(ntri * 3).height(1).levels(1)
                          .sampler(Texture::Sampler::SAMPLER_2D)
                          .format(Texture::InternalFormat::RGBA16F)
                          .usage(Texture::Usage::DEFAULT)
                          .build(*engine);
    triTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(triBuf.data(), triBuf.size() * sizeof(float), Texture::Format::RGBA, Texture::Type::FLOAT));
    MaterialInstance* dreiMi = dreiMat->createInstance();
    TextureSampler dreiNearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    dreiMi->setParameter("tris", triTex, dreiNearest);
    dreiMi->setParameter("env", iblTex[2], TextureSampler(TextureSampler::MagFilter::LINEAR));
    MaterialInstance* octaMi = octaMat->createInstance();
    octaMi->setParameter("tris", triTex, dreiNearest);
    octaMi->setParameter("center", float3{0.f, 0.2f, 0.f});
    octaMi->setParameter("extent", extent);
    octaMi->setParameter("grid", (float)GRID);
    octaMi->setParameter("bands", (float)NW);
    octaMi->setParameter("ior", 2.4f);
    octaMi->setParameter("dispersion", dispersion);
    octaMi->setParameter("bounces", 3.f);
    octaMi->setParameter("triCount", (float)ntri);
    octaMi->setParameter("hitW", (float)hitW);
    octaMi->setParameter("hitH", (float)hitH);
    dreiMi->setParameter("eye", float3{0.55f, 0.78f, 0.70f});
    dreiMi->setParameter("triCount", (float)ntri);
    dreiMi->setParameter("bounces", 3.f);
    dreiMi->setParameter("ior", 2.4f);
    dreiMi->setParameter("dispersion", dispersion);
    dreiMi->setParameter("aberration", 0.045f);
    Skybox* skyGem = Skybox::Builder().color({0.012f, 0.016f, 0.028f, 1.f}).build(*engine);
    scene->setIndirectLight(nullptr);
    scene->setSkybox(skyGem);
    floorMi->setParameter("color", float4{0.03f, 0.034f, 0.05f, 1.f});
    dreiMi->setParameter("chroma", 1.f);
    dreiMi->setParameter("fresnel", 0.f);
    dreiMi->setParameter("grad", 0.f);
    dreiMi->setParameter("envScale", 1.f);
    utils::Entity cubeDrei = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, gemVB, gemIB)
        .material(0, dreiMi).culling(false).castShadows(true).receiveShadows(false)
        .build(*engine, cubeDrei);
    if (!grab("out/xcube-drei-a.png")) return fail("drei-a");
    time_beauty("drei A glass");
    scene->remove(cubeBevel);
    {
        const int RF = 128;
        Texture* roomCube = Texture::Builder().width((uint32_t)RF).height((uint32_t)RF).levels(1)
                                .sampler(Texture::Sampler::SAMPLER_CUBEMAP)
                                .format(Texture::InternalFormat::RGBA16F)
                                .usage(Texture::Usage::DEFAULT)
                                .build(*engine);
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
        static std::vector<float> roomFace[6];
        auto fill_env = [&](auto color_of) {
            for (int face = 0; face < 6; ++face) {
                roomFace[face].assign((size_t)RF * RF * 4, 0.f);
                for (int y = 0; y < RF; ++y) {
                    float v = ((y + 0.5f) / RF) * 2.f - 1.f;
                    for (int x = 0; x < RF; ++x) {
                        float u = ((x + 0.5f) / RF) * 2.f - 1.f;
                        float3 c = color_of(face_dir(face, u, v));
                        size_t i = ((size_t)y * RF + x) * 4;
                        roomFace[face][i] = c.x;
                        roomFace[face][i + 1] = c.y;
                        roomFace[face][i + 2] = c.z;
                        roomFace[face][i + 3] = 1.f;
                    }
                }
                roomCube->setImage(*engine, 0, 0, 0, (uint32_t)face, (uint32_t)RF, (uint32_t)RF, 1,
                                   Texture::PixelBufferDescriptor(roomFace[face].data(), roomFace[face].size() * sizeof(float), Texture::Format::RGBA, Texture::Type::FLOAT));
            }
        };
        fill_env([](V3 d) { return (d.x > 0.f) ? float3{36.f, 36.f, 38.f} : float3{0.f, 0.f, 0.f}; });
        dreiMi->setParameter("env", roomCube, TextureSampler(TextureSampler::MagFilter::LINEAR));
        scene->remove(floor);
        scene->addEntity(cubeDrei);
        dreiMi->setParameter("chroma", 0.f);
        dreiMi->setParameter("fresnel", 0.f);
        dreiMi->setParameter("bounces", 3.f);
        auto look_gem = [&](float yaw) {
            float cy = std::cos(yaw), sy = std::sin(yaw);
            V3 off{0.62f, 0.22f, 0.38f};
            V3 eye{cy * off.x + sy * off.z, 0.2f + off.y, -sy * off.x + cy * off.z};
            camera->lookAt(float3{eye.x, eye.y, eye.z}, float3{0.f, 0.2f, 0.f}, float3{0, 1, 0});
            dreiMi->setParameter("eye", float3{eye.x, eye.y, eye.z});
        };
        look_gem(0.f);
        if (!grab("out/xcube-octa.png")) return fail("octa");
        look_gem(0.45f);
        if (!grab("out/xcube-octa-l.png")) return fail("octa-l");
        look_gem(-0.45f);
        if (!grab("out/xcube-octa-r.png")) return fail("octa-r");
        look_gem(0.f);
        time_beauty("drei B bounce3");
        dreiMi->setParameter("bounces", 2.f);
        time_beauty("drei B bounce2");
        dreiMi->setParameter("bounces", 1.f);
        time_beauty("drei B bounce1");
        dreiMi->setParameter("bounces", 3.f);
        dreiMi->setParameter("fresnel", 1.f);
        time_beauty("drei fresnel same");
        dreiMi->setParameter("fresnel", 0.f);
        auto wrap_pi = [](float a) {
            const float pi = 3.14159265f;
            while (a > pi) a -= 2.f * pi;
            while (a < -pi) a += 2.f * pi;
            return a;
        };
        fill_env([&](V3 d) {
            float az = std::atan2(d.x, d.z);
            float3 c{0.22f, 0.24f, 0.28f};
            if (d.x > 0.1f && d.y > -0.35f) c = float3{1.4f, 1.5f, 1.7f};
            const float bars[5] = {-2.2f, -1.05f, -0.15f, 0.85f, 2.05f};
            for (float b : bars) {
                if (std::fabs(wrap_pi(az - b)) < 0.045f && d.y > -0.75f && d.y < 0.92f) c = float3{36.f, 38.f, 42.f};
            }
            return c;
        });
        dreiMi->setParameter("fresnel", 1.f);
        if (!grab("out/xcube-octa-studio.png")) return fail("octa-studio");
        time_beauty("drei studio fresnel");
        dreiMi->setParameter("fresnel", 0.f);
    }
    auto shoot_eye = [&](V3 eye, const char* path) {
        camera->lookAt(float3{eye.x, eye.y, eye.z}, float3{0.f, 0.04f, 0.f}, float3{0, 1, 0});
        dreiMi->setParameter("eye", float3{eye.x, eye.y, eye.z});
        return grab(path);
    };
    V3 eyeL = focus + V3{camDist * std::cos(basePitch) * std::sin(baseYaw + 0.26f), camDist * std::sin(basePitch), camDist * std::cos(basePitch) * std::cos(baseYaw + 0.26f)};
    V3 eyeR = focus + V3{camDist * std::cos(basePitch) * std::sin(baseYaw - 0.26f), camDist * std::sin(basePitch), camDist * std::cos(basePitch) * std::cos(baseYaw - 0.26f)};
    if (!shoot_eye(eyeL, "out/xcube-drei-b-l.png")) return fail("drei-bl");
    if (!shoot_eye(eyeR, "out/xcube-drei-b-r.png")) return fail("drei-br");
    V3 eyeFar = focus + (eye0 - focus) * 1.8f;
    V3 eyeNear = focus + (eye0 - focus) * 0.62f;
    camera->lookAt(float3{eyeFar.x, eyeFar.y, eyeFar.z}, float3{0.f, 0.04f, 0.f}, float3{0, 1, 0});
    dreiMi->setParameter("eye", float3{eyeFar.x, eyeFar.y, eyeFar.z});
    time_beauty("drei B far");
    camera->lookAt(float3{eyeNear.x, eyeNear.y, eyeNear.z}, float3{0.f, 0.04f, 0.f}, float3{0, 1, 0});
    dreiMi->setParameter("eye", float3{eyeNear.x, eyeNear.y, eyeNear.z});
    time_beauty("drei B near");
    camera->lookAt(float3{0.55f, 0.78f, 0.70f}, float3{0, 0.04f, 0}, float3{0, 1, 0});
    scene->remove(cubeDrei);
    scene->addEntity(floor);
    scene->addEntity(cubeBevel);
    std::printf("drei tris %u ior 2.4 bounces 3  no usampler BVH\n", ntri);
    scene->setIndirectLight(nullptr);
    scene->setSkybox(sky);
    floorMi->setParameter("color", float4{0.86f, 0.85f, 0.84f, 1});
    show_lit(false);
