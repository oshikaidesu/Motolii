// glass_scene.mm, piece 5 (cut there by size, as it was): read only in its place in glass_scene.mm.
        if (it.vbody) {
            static VertexBuffer* gridVB = nullptr;
            static IndexBuffer* gridIB = nullptr;
            if (!gridVB) {
                const uint32_t count = (uint32_t)(gridSoup.size() / 5);
                gridVB = VertexBuffer::Builder().vertexCount(count).bufferCount(1)
                             .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                             .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*engine);
                gridVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(gridSoup.data(), gridSoup.size() * sizeof(float)));
                static std::vector<uint32_t> gridIdx;
                for (uint32_t q = 0; q + 3 < count; q += 4) gridIdx.insert(gridIdx.end(), {q, q + 1, q + 2, q, q + 2, q + 3});
                gridIB = IndexBuffer::Builder().indexCount((uint32_t)gridIdx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
                gridIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(gridIdx.data(), gridIdx.size() * sizeof(uint32_t)));
            }
            const TextureSampler ns(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
            MaterialInstance* c = it.cheap = glassVertMat->createInstance();
            c->setParameter("ior", nd);
            c->setParameter("abbe", 64.17f / disp);
            c->setParameter("planes", planes.data(), 40);
            c->setParameter("planeCount", count);
            c->setParameter("kind", (int32_t)0);
            c->setParameter("dims", float4{0, 0, 0, 0});
            c->setParameter("scene", sceneTex, ns);
            c->setParameter("shapes", exShapeTex, exNearest);
            c->setParameter("solidCount", (int32_t)0);
            c->setParameter("self", (int32_t)k);
            c->setParameter("nearCone", 0.f);
            c->setParameter("spread", env_f("SPREAD", 2.f));
            c->setParameter("floorY", 0.f);
            c->setParameter("absorption", float3{0.9f, 0.35f, 0.5f} * env_f("ABSORB", 0.15f));
            c->setParameter("backdrop", backdrop.color, lin);
            c->setParameter("nodes", nodesRT.color, ns);
            c->setParameter("faces", faceTex, ns);
            c->setParameter("tol", env_f("VTOL", 1.f));
            c->setParameter("levels", env_f("VLEVELS", 6.f));
            c->setParameter("cells", cellsRT.color, ns);
            c->setParameter("slop", math::float2{env_f("VSLOP", 0.f), env_f("VSLOPT", 0.f)});
            c->setParameter("showCells", env_f("VSHOW", 0.f));
            utils::Entity ce = em.create();
            RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}}).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, gridVB, gridIB)
                .material(0, c).culling(false).castShadows(false).receiveShadows(false).priority(2).layerMask(0xFF, 0x02).build(*engine, ce);
            if (!std::getenv("NOGLASS") && !std::getenv("VNOCHEAP")) scene->addEntity(ce);
        }
    }
    (void)shadowMat;
    (void)filamentShadow;
    if (vbodyOn) {
        faceTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(faceTable.data(), faceTable.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
        const TextureSampler ns(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
        nodesMi = glassNodesMat->createInstance();
        nodesMi->setParameter("pose", poseTex, ns);
        nodesMi->setParameter("faces", faceTex, ns);
        nodesMi->setParameter("scene", sceneTex, ns);
        nodesMi->setParameter("solidCount", (int32_t)items.size());
        nodesMi->setParameter("search", std::getenv("ALONE") ? 0.f : 1.f);
        nodesMi->setParameter("mode", env_f("VNODEMODE", 0.f));   // timing only (glass_nodes.mat)
        nodesMi->setParameter("ior", nd);
        nodesMi->setParameter("abbe", 64.17f / disp);
        nodesMi->setParameter("spread", env_f("SPREAD", 2.f));
        nodesMi->setParameter("floorY", 0.f);
        sNodes->addEntity(renderable(sqVB, sqIB, nodesMi, false, false));
        MaterialInstance* cellsMi = glassCellsMat->createInstance();
        cellsMi->setParameter("nodes", nodesRT.color, ns);
        cellsMi->setParameter("tol", env_f("VTOL", 1.f));
        sCells->addEntity(renderable(sqVB, sqIB, cellsMi, false, false));
    }

    utils::Entity sunE = em.create();
    LightManager::ShadowOptions shopt;
    shopt.mapSize = 2048;
    shopt.shadowNearHint = 0.5f;
    shopt.shadowFarHint = 8.0f;
    shopt.constantBias = 0.001f;
    shopt.normalBias = 0.5f;
    LightManager::Builder(LightManager::Type::SUN).color(sunColor).intensity(lux).direction(float3{0, -1, 0})
        .sunAngularRadius(sunRadiusDeg).sunHaloSize(1.2f).castShadows(true).shadowOptions(shopt).build(*engine, sunE);
    scene->addEntity(sunE);
    scene->setSkybox(Skybox::Builder().color({0.72f * roomLight, 0.74f * roomLight, 0.78f * roomLight, 1}).build(*engine));
    {
        // Studio environment (out/studio): gives the block something to mirror and keeps the shadow from being black.
        auto ib = load_file(GLASS_PATH("out/studio/_ibl.ktx").c_str());
        std::ifstream shf(GLASS_PATH("out/studio/sh.txt"));
        std::vector<float3> shv;
        std::string line;
        while (std::getline(shf, line)) {
            size_t l = line.find('('), r = line.find(')');
            if (l == std::string::npos || r == std::string::npos) continue;
            float x, y, z;
            if (std::sscanf(line.c_str() + l + 1, "%f,%f,%f", &x, &y, &z) == 3) shv.push_back({x, y, z});
        }
        if (ib.empty() || shv.size() < 9) return fail("studio environment (out/studio)");
        Texture* ibl = ktxreader::Ktx1Reader::createTexture(engine, new image::Ktx1Bundle(ib.data(), (uint32_t)ib.size()), false);
        const float iblLux = env_f("IBL", 900.f) * roomLight;
        scene->setIndirectLight(IndirectLight::Builder().reflections(ibl).irradiance(3, shv.data()).intensity(iblLux).build(*engine));
        TextureSampler cubeS(TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE);
        for (Item& it : items) {
            it.glass->setParameter("env", ibl, cubeS);
            it.glass->setParameter("envGain", iblLux * env_f("ENVGAIN", 0.f));
            it.glass->setParameter("sky", float3{0.95f, 0.97f, 1.0f} * (env_f("SKY", 1800.f) * roomLight));
            if (it.cheap) {
                it.cheap->setParameter("env", ibl, cubeS);
                it.cheap->setParameter("envGain", iblLux * env_f("ENVGAIN", 0.f));
                it.cheap->setParameter("sky", float3{0.95f, 0.97f, 1.0f} * (env_f("SKY", 1800.f) * roomLight));
            }
        }
    }

    // ---- camera and views
    Camera* camera = engine->createCamera(em.create());
    camera->setExposure(env_f("APERTURE", 4.0f), 1.f / 125.f, 100.f);
    float cam[7] = {1.55f, 1.45f, 2.15f, 0.10f, 0.0f, -0.05f, 32.f};
    if (wallCount > 0) {  // a little further back and higher, so that the walls are in the picture
        const float room[7] = {1.9f, 1.55f, 2.5f, -0.3f, 0.3f, -0.5f, 36.f};
        std::memcpy(cam, room, sizeof(cam));
    }
    if (std::getenv("SCATTER")) {   // strewn solids take more floor than the grid: stand back as far as it takes to see them all
        const float back = std::max(1.f, 0.85f * std::sqrt((float)items.size()) / 1.25f);
        for (int i = 0; i < 3; ++i) cam[i] = cam[3 + i] + (cam[i] - cam[3 + i]) * back;
    }
    if (const char* c = std::getenv("CAM")) std::sscanf(c, "%f %f %f %f %f %f %f", &cam[0], &cam[1], &cam[2], &cam[3], &cam[4], &cam[5], &cam[6]);
    camera->lookAt(float3{cam[0], cam[1], cam[2]}, float3{cam[3], cam[4], cam[5]}, float3{0, 1, 0});
    camera->setProjection((double)cam[6], double(W) / double(H), 0.05, 40.0, Camera::Fov::VERTICAL);

    View* vLight = engine->createView();
    vLight->setScene(sLight);
    vLight->setCamera(camera);
    vLight->setRenderTarget(light.rt);
    vLight->setViewport({0, 0, (uint32_t)LW, (uint32_t)LH});
    vLight->setDithering(View::Dithering::NONE);
    vLight->setPostProcessingEnabled(false);
    vLight->setShadowingEnabled(false);

    {
        const mat4 clipFromWorld = camera->getProjectionMatrix() * camera->getViewMatrix();
        softenMi->setParameter("worldFromClip", mat4f(inverse(clipFromWorld)));
        softenMi->setParameter("eye", float3{cam[0], cam[1], cam[2]});
        softenMi->setParameter("focal", (float)(LH / (2.0 * std::tan(cam[6] * 0.5 * 3.14159265 / 180.0))));
    }
    // ---- the shadow outlines, on the GPU (outline.mat): per shape a mesh of its surface, built once; the vertex shader
    // slides the sunlit part along the sun onto the floor. OUTLINECPU=1 makes them on the CPU instead, as before.
    const bool outlineOnCpu = std::getenv("OUTLINECPU") != nullptr;
    struct OutlineKind {
        std::string name;
        std::vector<int> solids;
        MaterialInstance* mi = nullptr;
    };
    static std::vector<OutlineKind> outlineKinds;
    for (size_t k = 0; k < items.size(); ++k) {
        if (items[k].local.kind == beam::Solid::EXTRUDE) continue;  // (solids with dents and holes draw their own shadow)
        int found = -1;
        for (size_t q = 0; q < outlineKinds.size(); ++q)
            if (outlineKinds[q].name == items[k].kind) found = (int)q;
        if (found < 0) {
            outlineKinds.push_back(OutlineKind{items[k].kind, {}, nullptr});
            found = (int)outlineKinds.size() - 1;
        }
        outlineKinds[(size_t)found].solids.push_back((int)k);
    }
    for (OutlineKind& ok : outlineKinds) {
        const beam::Solid& unit = items[(size_t)ok.solids[0]].local;
        auto* v = new std::vector<float>();   // position xyz, then four more numbers (the face's normal, for flat faces)
        auto* ix = new std::vector<uint32_t>();
        auto put = [&](float x, float y, float z, float nx = 0, float ny = 0, float nz = 0) {
            v->insert(v->end(), {x, y, z, nx, ny, nz, 0.f});
            return (uint32_t)(v->size() / 7 - 1);
        };
        const float pi = 3.14159265f;
        if (unit.kind == beam::Solid::FLAT) {
            for (size_t f = 0; f < unit.faces.size(); ++f) {
                const beam::Poly& P = unit.faces[f];
                if (P.n < 3) continue;
                const beam::V3 n = unit.planes[f].n;
                uint32_t first = 0;
                for (int i = 0; i < P.n; ++i) {
                    const uint32_t id = put((float)P.p[(size_t)i].x, (float)P.p[(size_t)i].y, (float)P.p[(size_t)i].z, (float)n.x, (float)n.y, (float)n.z);
                    if (i == 0) first = id;
                }
                for (int i = 1; i + 1 < P.n; ++i) ix->insert(ix->end(), {first, first + (uint32_t)i, first + (uint32_t)i + 1});
            }
        } else if (unit.kind == beam::Solid::BALL) {
            const int n = 56;
            const uint32_t mid = put(0, 0, 0);
            for (int j = 0; j < n; ++j) put(std::cos(2 * pi * j / n), std::sin(2 * pi * j / n), 0);
            for (int j = 0; j < n; ++j) ix->insert(ix->end(), {mid, mid + 1 + (uint32_t)j, mid + 1 + (uint32_t)((j + 1) % n)});
        } else if (unit.kind == beam::Solid::ROD) {
            const int n = 28, m = 32;
            for (int j = 0; j < n; ++j) {
                const float a = -1.f + 2.f * j / n, b = -1.f + 2.f * (j + 1) / n;
                const uint32_t q = put(0, a, 0);
                put(1, a, 0);
                put(1, b, 0);
                put(0, b, 0);
                ix->insert(ix->end(), {q, q + 1, q + 2, q, q + 2, q + 3});
            }
            for (int end = 1; end <= 2; ++end) {
                const uint32_t mid = put(0, 0, (float)end);
                for (int j = 0; j < m; ++j) put(std::cos(2 * pi * j / m), std::sin(2 * pi * j / m), (float)end);
                for (int j = 0; j < m; ++j) ix->insert(ix->end(), {mid, mid + 1 + (uint32_t)j, mid + 1 + (uint32_t)((j + 1) % m)});
            }
        } else {
            const int n = 48;
            for (int part = 0; part < 2; ++part)
                for (int j = 0; j < n; ++j) {
                    const uint32_t q = put(0, (float)j / n, (float)part);
                    put(1, (float)j / n, (float)part);
                    put(1, (float)(j + 1) / n, (float)part);
                    ix->insert(ix->end(), {q, q + 1, q + 2});
                }
        }
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(v->size() / 7)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                               .attribute(VertexAttribute::COLOR, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28).build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(v->data(), v->size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)ix->size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(ix->data(), ix->size() * sizeof(uint32_t)));
        ok.mi = outlineMat->createInstance();
        on_stage(ok.mi, false);
        ok.mi->setParameter("pose", poseTex, nearest);
        ok.mi->setParameter("kind", (float)unit.kind);
        ok.mi->setParameter("dims", float4{(float)unit.r, (float)unit.h, 0, 0});
        ok.mi->setParameter("floorY", 0.f);
        float4 which[16] = {};
        for (size_t i = 0; i < ok.solids.size() && i < 64; ++i) which[i / 4][i % 4] = (float)ok.solids[i];
        ok.mi->setParameter("solids", which, 16);
        utils::Entity e = em.create();
        RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .instances(ok.solids.size() * recvTotal)
            .material(0, ok.mi).culling(false).castShadows(false).receiveShadows(false).layerMask(0xFF, outlineOnCpu ? 0x00 : 0x01).build(*engine, e);
        sShade->addEntity(e);
    }

    // ---- EXPERIMENT, STREAK=1: the bright lines of the solids with an outline, drawn from the outline itself with no
    // table pass (edge_streak.mat): one four-cornered piece per straight bit of the outline, per solid and colour step.
    std::vector<MaterialInstance*> streakMis;
    // EXNEW=1: all the light and shadow of those solids drawn this way (window and shadow of the caps: edge_fill.mat),
    // the old table passes and their drawing switched off.
    const bool exNew = std::getenv("EXNEW") != nullptr;
    std::vector<MaterialInstance*> fillMis;
    if (std::getenv("STREAK") || exNew) {
        Material* streakMat = mat("edge_streak.filamat");
        Material* fillMat = mat("edge_fill.filamat");
        if (!streakMat || !fillMat) return fail("edge_streak.filamat / edge_fill.filamat");
        for (const ExShapeUse& sh : exShapes) {
            auto* v = new std::vector<float>();   // a.x, a.y, corner; b.x, b.y; the way out of the glass at a, at b
            auto* ix = new std::vector<uint32_t>();
            for (const std::vector<int>& loop : sh.ol->loops)
                for (size_t i = 0; i < loop.size(); ++i) {
                    const int ja = loop[i];
                    const beam::V2 a = sh.ol->pts[(size_t)ja], b = sh.ol->pts[(size_t)loop[(i + 1) % loop.size()]];
                    // straight: the same way out all along; on an arc: away from (or towards) the middle of its circle
                    beam::V2 na{b.y - a.y, a.x - b.x}, nb = na;
                    for (const beam::OutlineEdge& e : sh.ol->edges)
                        if (e.r != 0 && ja >= e.first && ja < e.first + e.chords) {
                            const double sg = e.r > 0 ? 1.0 : -1.0;
                            na = (a - e.c) * sg;
                            nb = (b - e.c) * sg;
                        }
                    na = na * (1.0 / beam::length(na));
                    nb = nb * (1.0 / beam::length(nb));
                    const uint32_t base = (uint32_t)(v->size() / 9);
                    for (int c = 0; c < 4; ++c) v->insert(v->end(), {(float)a.x, (float)a.y, (float)c, (float)b.x, (float)b.y, (float)na.x, (float)na.y, (float)nb.x, (float)nb.y});
                    ix->insert(ix->end(), {base, base + 1, base + 2, base, base + 2, base + 3});
                }
            VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(v->size() / 9)).bufferCount(1)
                                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 36)
                                   .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 36)
                                   .attribute(VertexAttribute::CUSTOM0, 0, VertexBuffer::AttributeType::FLOAT4, 20, 36).build(*engine);
            vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(v->data(), v->size() * sizeof(float)));
            IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)ix->size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
            ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(ix->data(), ix->size() * sizeof(uint32_t)));
            float4 which[16] = {};
            int n = 0;
            for (size_t k = 0; k < items.size() && n < 64; ++k)
                if (items[k].local.kind == beam::Solid::EXTRUDE && items[k].local.shape.get() == sh.ol) { which[n / 4][n % 4] = (float)k; ++n; }
            if (exNew) {   // the caps: window, shadow of the near cap, shadow of the far cap
                auto* cv = new std::vector<float>();
                for (const beam::V2& q : sh.ol->pts) cv->insert(cv->end(), {(float)q.x, (float)q.y, 0.f});
                auto* ci = new std::vector<uint32_t>(sh.ol->tris.begin(), sh.ol->tris.end());
                VertexBuffer* cvb = VertexBuffer::Builder().vertexCount((uint32_t)(cv->size() / 3)).bufferCount(1)
                                        .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12).build(*engine);
                cvb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(cv->data(), cv->size() * sizeof(float)));
                IndexBuffer* cib = IndexBuffer::Builder().indexCount((uint32_t)ci->size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
                cib->setBuffer(*engine, IndexBuffer::BufferDescriptor(ci->data(), ci->size() * sizeof(uint32_t)));
                for (int mode = 0; mode < 3; ++mode) {
                    MaterialInstance* mi = fillMat->createInstance();
                    mi->setParameter("pose", poseTex, nearest);
                    mi->setParameter("solids", which, 16);
                    mi->setParameter("halfThick", (float)sh.ol->h);
                    mi->setParameter("index", nd);
                    mi->setParameter("mode", (float)mode);
                    mi->setParameter("floorY", 0.f);
                    utils::Entity e = em.create();
                    RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
                        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, cvb, cib).instances((size_t)n)
                        .material(0, mi).culling(false).castShadows(false).receiveShadows(false).build(*engine, e);
                    (mode == 0 ? sLight : sShade)->addEntity(e);
                    fillMis.push_back(mi);
                }
            }
            for (int mode = 0; mode < (exNew ? 4 : 2); ++mode) {   // mirrored by the wall; out through the cut on the rim (the colours);
                                                                   // EXNEW: the rim the window lacks; the walls' shadow
                const float strength = mode >= 2 ? 1.f : env_f(mode == 0 ? "STREAK" : "STREAKRAINBOW", 1.f);
                if (strength <= 0.f) continue;
                MaterialInstance* mi = streakMat->createInstance();
                mi->setParameter("pose", poseTex, nearest);
                mi->setParameter("solids", which, 16);
                mi->setParameter("stepCount", (float)(mode >= 2 ? 1 : beam::EXT_STEPS));
                mi->setParameter("halfThick", (float)sh.ol->h);
                mi->setParameter("mode", (float)mode);
                mi->setParameter("chamfer", (float)sh.ol->chamfer);
                mi->setParameter("wide", env_f("STREAKWIDE", 0.f));   // extra width for every colour's band (edge_streak.mat); 0: as the cut gives it
                mi->setParameter("gain", strength);
                mi->setParameter("cap", env_f("STREAKCAP", 3.f));
                mi->setParameter("floorY", 0.f);
                utils::Entity e = em.create();
                RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
                    .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib).instances((size_t)n * (size_t)(mode >= 2 ? 1 : beam::EXT_STEPS))
                    .material(0, mi).culling(false).castShadows(false).receiveShadows(false).build(*engine, e);
                (mode == 3 ? sShade : sLight)->addEntity(e);
                streakMis.push_back(mi);
            }
        }
    }

    View* vFlatWalk = engine->createView();
    vFlatWalk->setScene(sFlatWalk);
    vFlatWalk->setCamera(camera);
    for (const RouteLevel& rl : routeLevels) rl.view->setCamera(camera);
    for (View* v : gbeamViews) v->setCamera(camera);
    vFlatWalk->setRenderTarget(flatWalk.rt);
    vFlatWalk->setViewport({0, 0, (uint32_t)(flatMostPieces * flatPer), (uint32_t)(items.size() * (size_t)flatLines)});
    vFlatWalk->setDithering(View::Dithering::NONE);
    vFlatWalk->setPostProcessingEnabled(false);
    vFlatWalk->setShadowingEnabled(false);
    View* vFlatWalk2 = engine->createView();
    if (flatClip) {
        vFlatWalk2->setScene(sFlatWalk2);
        vFlatWalk2->setCamera(camera);
        vFlatWalk2->setRenderTarget(flatWalk2.rt);
        vFlatWalk2->setViewport({0, 0, (uint32_t)(flatMostPieces * flatPer), (uint32_t)(items.size() * (size_t)flatLines)});
        vFlatWalk2->setDithering(View::Dithering::NONE);
        vFlatWalk2->setPostProcessingEnabled(false);
        vFlatWalk2->setShadowingEnabled(false);
    }
    View* vNodes = engine->createView();
    vNodes->setScene(sNodes);
    vNodes->setCamera(camera);
    vNodes->setRenderTarget(nodesRT.rt);
    vNodes->setViewport({0, 0, 289, (uint32_t)std::max<size_t>(1, items.size()) * 6});
    View* vCells = engine->createView();
    vCells->setScene(sCells);
    vCells->setCamera(camera);
    vCells->setRenderTarget(cellsRT.rt);
    vCells->setViewport({0, 0, 80, (uint32_t)std::max<size_t>(1, items.size()) * 6});
    vCells->setDithering(View::Dithering::NONE);
    vCells->setPostProcessingEnabled(false);
    vCells->setShadowingEnabled(false);
    vNodes->setDithering(View::Dithering::NONE);
    vNodes->setPostProcessingEnabled(false);
    vNodes->setShadowingEnabled(false);
    View* vConeWalk = engine->createView();
    vConeWalk->setScene(sConeWalk);
    vConeWalk->setCamera(camera);
    vConeWalk->setRenderTarget(coneWalk.rt);
    vConeWalk->setViewport({0, 0, (uint32_t)(coneV.size() / 3) * 2, (uint32_t)coneLines});
    vConeWalk->setDithering(View::Dithering::NONE);
    vConeWalk->setPostProcessingEnabled(false);
    vConeWalk->setShadowingEnabled(false);
    View* vExFind = engine->createView();
    View* vExWalk = engine->createView();
    for (View* v : {vExFind, vExWalk}) {
        const bool findPass = v == vExFind;
        v->setScene(findPass ? sExFind : sExWalk);
        v->setCamera(camera);
        v->setRenderTarget(findPass ? exFind.rt : exWalk.rt);
        v->setViewport({0, 0, (uint32_t)beam::EXT_COLUMNS, (uint32_t)(exRoom * (size_t)(findPass ? beam::EXT_FIND_ROWS : beam::EXT_ROWS))});
        v->setDithering(View::Dithering::NONE);
        v->setPostProcessingEnabled(false);
        v->setShadowingEnabled(false);
    }
    View* vShade = engine->createView();
    vShade->setScene(sShade);
    vShade->setCamera(camera);
    vShade->setRenderTarget(shade.rt);
    vShade->setViewport({0, 0, (uint32_t)LW, (uint32_t)LH});
    vShade->setDithering(View::Dithering::NONE);
    vShade->setPostProcessingEnabled(false);
    vShade->setShadowingEnabled(false);
    View* vSoft = engine->createView();
    vSoft->setScene(sSoft);
    vSoft->setCamera(camera);
    vSoft->setRenderTarget(soft.rt);
    vSoft->setViewport({0, 0, (uint32_t)SW, (uint32_t)SH});
    vSoft->setDithering(View::Dithering::NONE);
    vSoft->setPostProcessingEnabled(false);
    vSoft->setShadowingEnabled(false);

    // the stage: every material that drops light is told how to turn a distance flown into a spread in pixels (see
    // beam_soften.mat, `unit`)
    if (stageOn) {
        const float pxScale = std::tan(sunRadiusDeg * 3.14159265f / 180.f) * (float)(LH / (2.0 * std::tan(cam[6] * 0.5 * 3.14159265 / 180.0)));
        for (MaterialInstance* mi : stageLights) {
            set_stage(mi, true);
            if (blocksOn) mi->setDepthCulling(true);  // (the blocks are in the depth buffer of both pictures: what they hide is not drawn)
        }
        for (MaterialInstance* mi : landers) mi->setParameter("pxScale", pxScale);
    }

    const View::ShadowType shadowType = std::getenv("HARDSHADOW") ? View::ShadowType::PCF : View::ShadowType::PCSS;
    SoftShadowOptions softShadow;
    softShadow.penumbraScale = env_f("PENUMBRA", 1.f);
    View* vBackdrop = engine->createView();
    vBackdrop->setScene(scene);
    vBackdrop->setCamera(camera);
    vBackdrop->setRenderTarget(backdrop.rt);
    vBackdrop->setViewport({0, 0, (uint32_t)BW, (uint32_t)BH});
    vBackdrop->setVisibleLayers(0xFF, 0x01);
    vBackdrop->setDithering(View::Dithering::NONE);
    vBackdrop->setPostProcessingEnabled(false);
    vBackdrop->setShadowType(shadowType);
    vBackdrop->setSoftShadowOptions(softShadow);

    View* vBeauty = engine->createView();
    vBeauty->setVisibleLayers(0xFF, 0x03);
    vBeauty->setScene(scene);
    vBeauty->setCamera(camera);
    vBeauty->setRenderTarget(beauty.rt);
    vBeauty->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    vBeauty->setDithering(View::Dithering::NONE);
    static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
    vBeauty->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.04f).saturation(1.08f).build(*engine));
    vBeauty->setPostProcessingEnabled(true);
    vBeauty->setShadowType(shadowType);
    vBeauty->setSoftShadowOptions(softShadow);
    // Every shadow here is an outline drawn as missing light (and on a stage Filament's own shadow maps would take the
    // sun away twice). Nothing casts a Filament shadow, but while shadowing was left on the floor still worked out
    // a soft shadow at every pixel of both pictures, from an empty map: 2.5 ms of GPU a frame for nothing.
    vBeauty->setShadowingEnabled(false);
    vBackdrop->setShadowingEnabled(false);
    BloomOptions bloom{};
    bloom.enabled = !std::getenv("NOBLOOM");
    bloom.strength = env_f("BLOOM", 0.10f);
    bloom.levels = 6;
    bloom.resolution = 1080;
    bloom.threshold = true;
    bloom.highlight = 10.f;
    bloom.blendMode = BloomOptions::BlendMode::ADD;
    vBeauty->setBloomOptions(bloom);
    vBeauty->setAntiAliasing(std::getenv("NOFXAA") ? View::AntiAliasing::NONE : View::AntiAliasing::FXAA);

    Renderer::ClearOptions clearOpt;
    clearOpt.clear = true;
    clearOpt.discard = true;
    clearOpt.clearColor = {0, 0, 0, 0};
    renderer->setClearOptions(clearOpt);

    // ---- every frame: place the solids, trace their light (one thread each), hand the triangles to the GPU
    float3 sunDir = normalize(float3{0.92f, -0.42f, 0.22f});
    if (wallCount > 0) sunDir = normalize(float3{-0.85f, -0.40f, -0.35f});   // towards the walls, or nothing would land on them
