// glass_scene.mm, piece 3 (cut there by size, as it was): read only in its place in glass_scene.mm.
        softenMi->setParameter("marks", blocksOn ? 1.f : 0.f);

        // the walls
        for (const beam::Half& w : walls) {
            auto* v = new std::vector<float>();
            auto* idx = new std::vector<uint32_t>{0, 2, 1, 0, 3, 2};   // (its front is the room's side)
            const float F = env_f("FLOOR", 6.f);
            const beam::V3 along = beam::normalize(beam::cross(w.a, beam::V3{0, 1, 0})), foot = w.a * -w.b;
            const float3 n{(float)w.a.x, (float)w.a.y, (float)w.a.z}, u{(float)along.x, (float)along.y, (float)along.z}, p0{(float)foot.x, 0.f, (float)foot.z};
            for (float3 p : {p0 - u * F, p0 + u * F, p0 + u * F + float3{0, wallTop, 0}, p0 - u * F + float3{0, wallTop, 0}}) push_vert(*v, p, u, n);
            auto [vb, ib] = lit_buffers(*v, *idx);
            scene->addEntity(renderable(vb, ib, recv_surface(float4{0.86f, 0.85f, 0.84f, 1}, env_f("WALL_ROUGH", 0.55f), soft.color), false, true, 12.f));
        }
        // the blocks: as surfaces in the picture, and flat in the picture of shade, each with its own mark
        // (recv_mask.mat), so that the softening pass can tell a block from what is seen behind it
        static const float4 tint[4] = {{0.90f, 0.86f, 0.80f, 1}, {0.78f, 0.83f, 0.88f, 1}, {0.82f, 0.87f, 0.80f, 1}, {0.89f, 0.81f, 0.81f, 1}};
        auto* outlineV = new std::vector<float>();
        for (size_t b = 0; b < blocks.size(); ++b) {
            const beam::Block& B = blocks[b];
            auto* v = new std::vector<float>();
            auto* idx = new std::vector<uint32_t>();
            for (int i = 0; i < 3; ++i)
                for (int sgn = -1; sgn <= 1; sgn += 2) {
                    const int j = (i + 1) % 3, k = (i + 2) % 3;
                    const beam::V3 n = B.ax[i] * (double)sgn, mid = B.c + n * B.h[i], a = B.ax[j] * B.h[j], c = B.ax[k] * B.h[k];
                    const beam::V3 q[4] = {mid - a - c, mid + a - c, mid + a + c, mid - a + c};
                    const uint32_t base = (uint32_t)(v->size() / 7);
                    for (beam::V3 p : q)
                        push_vert(*v, float3{(float)p.x, (float)p.y, (float)p.z}, float3{(float)B.ax[j].x, (float)B.ax[j].y, (float)B.ax[j].z}, float3{(float)n.x, (float)n.y, (float)n.z});
                    if (sgn > 0) idx->insert(idx->end(), {base, base + 1, base + 2, base, base + 2, base + 3});
                    else idx->insert(idx->end(), {base, base + 2, base + 1, base, base + 3, base + 2});   // (the front is the outside)
                    // and for the block's outline against the sun (see below): position; the face's normal, the block's number
                    for (int t : {0, 1, 2, 0, 2, 3}) outlineV->insert(outlineV->end(), {(float)q[t].x, (float)q[t].y, (float)q[t].z, (float)n.x, (float)n.y, (float)n.z, (float)b});
                }
            auto [vb, ib] = lit_buffers(*v, *idx);
            scene->addEntity(renderable(vb, ib, recv_surface(tint[b % 4], env_f("PROP_ROUGH", 0.7f), soft.color), false, true, 12.f));
            MaterialInstance* mark = recvMaskMat->createInstance();
            mark->setParameter("mark", float4{0, (float)(b + 1) / 8.f, 0, 0});
            sShade->addEntity(renderable(vb, ib, mark, false, false, 12.f));
            MaterialInstance* depthOnly = recvMaskMat->createInstance();   // (in the picture of light: nothing but depth)
            depthOnly->setParameter("mark", float4{0, 0, 0, 0});
            depthOnly->setColorWrite(false);
            sLight->addEntity(renderable(vb, ib, depthOnly, false, false, 12.f));
        }
        // the blocks' own outlines against the sun: their faces as they stand, made once; recv_light.mat slides the
        // ones that face the sun along it onto every receiver
        if (!blocks.empty()) {
            const uint32_t count = (uint32_t)(outlineV->size() / 7);
            VertexBuffer* vb = VertexBuffer::Builder().vertexCount(count).bufferCount(1)
                                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                                   .attribute(VertexAttribute::COLOR, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28).build(*engine);
            vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(outlineV->data(), outlineV->size() * sizeof(float)));
            IndexBuffer* ib = IndexBuffer::Builder().indexCount(count).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
            ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(light_i.data(), count * sizeof(uint32_t)));
            sShade->addEntity(recv_tris(vb, ib, 2.f, 1.f, 0, roomPlanes));
            sShade->addEntity(recv_tris(vb, ib, 2.f, 1.f, roomPlanes, faces));
        }
    }

    // the table of all faces of all solids, in world space, for glass seen through glass
    static const int SCENE_W = 48, SCENE_H = 64;
    if (items.size() > (size_t)SCENE_H) return fail("more than 64 solids");
    Texture* sceneTex = Texture::Builder().width(SCENE_W).height(SCENE_H).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);

    // ---- the light of the flat-faced solids, on the GPU (flat_walk.mat + flat_light.mat, "Flat faces on the GPU" in
    // beam_glass.h). Built once: per shape a table of its pieces and a mesh. Per frame the CPU only says where each
    // solid stands (the pose table). FLATCPU=1 works the same light out on the CPU instead, as before, to compare.
    const bool flatOnCpu = std::getenv("FLATCPU") != nullptr;
    const int piecesPerSide = (int)env_f("PIECES", 4.f);
    const int flatSteps = std::min(15, std::max(1, (int)env_f("FLATSTEPS", 8.f))) + 1;  // colour steps, and the white light mirrored off the outside
    const int rainbowParts = rainbowOnce ? std::min(7, std::max(1, (int)env_f("RAINBOW1", 1.f))) : 0;   // RAINBOW1=k: the spectrum in k stretches
    const int flatLines = rainbowOnce ? 2 * rainbowParts + 1 : flatSteps;   // lines per solid in the walk table (RAINBOW1: two ends per stretch, mirrored)
    const int flatDraws = rainbowOnce ? rainbowParts + 1 : flatSteps;       // times every piece is drawn (RAINBOW1: once per stretch, and once mirrored)
    struct FlatKind {
        std::string name;
        beam::FlatShape shape;
        std::vector<int> solids;  // the items of this shape
        MaterialInstance* mi = nullptr;
        MaterialInstance* mi2 = nullptr;   // EDGES: the pieces of the walk's second turn
        utils::Entity entity;
    };
    static std::vector<FlatKind> flatKinds;
    std::vector<int> flatKindOf(items.size(), -1);
    for (size_t k = 0; k < items.size(); ++k) {
        if (items[k].local.kind != beam::Solid::FLAT) continue;
        int found = -1;
        for (size_t q = 0; q < flatKinds.size(); ++q)
            if (flatKinds[q].name == items[k].kind) found = (int)q;
        if (found < 0) {
            flatKinds.push_back(FlatKind{items[k].kind, beam::flat_shape(items[k].local, piecesPerSide), {}, nullptr, {}});
            found = (int)flatKinds.size() - 1;
        }
        flatKinds[(size_t)found].solids.push_back((int)k);
        flatKindOf[k] = found;
    }
    int flatMostPieces = 1;
    for (const FlatKind& fk : flatKinds) flatMostPieces = std::max(flatMostPieces, fk.shape.pieces);
    Texture* flatPieces = Texture::Builder().width((uint32_t)flatMostPieces * 5).height((uint32_t)std::max<size_t>(1, flatKinds.size())).levels(1)
                              .format(Texture::InternalFormat::RGBA32F).build(*engine);
    {
        static std::vector<float> texels;
        texels.assign((size_t)flatMostPieces * 5 * std::max<size_t>(1, flatKinds.size()) * 4, 0.f);
        for (size_t q = 0; q < flatKinds.size(); ++q) std::copy(flatKinds[q].shape.table.begin(), flatKinds[q].shape.table.end(), texels.begin() + (long)(q * (size_t)flatMostPieces * 5 * 4));
        flatPieces->setImage(*engine, 0, Texture::PixelBufferDescriptor(texels.data(), texels.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
    }
    Texture* flatHull = Texture::Builder().width((uint32_t)flatMostPieces * 5).height((uint32_t)std::max<size_t>(1, flatKinds.size())).levels(1)
                            .format(Texture::InternalFormat::RGBA32F).build(*engine);
    {
        static std::vector<float> texels;
        texels.assign((size_t)flatMostPieces * 5 * std::max<size_t>(1, flatKinds.size()) * 4, 0.f);
        for (size_t q = 0; q < flatKinds.size(); ++q) std::copy(flatKinds[q].shape.hull.begin(), flatKinds[q].shape.hull.end(), texels.begin() + (long)(q * (size_t)flatMostPieces * 5 * 4));
        flatHull->setImage(*engine, 0, Texture::PixelBufferDescriptor(texels.data(), texels.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
    }
    Texture* poseTex = Texture::Builder().width(SCENE_H).height(4).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
    // EDGES=1: the pieces of light are cut where their light starts to go another way (flat_walk.mat, clip), so their
    // edges are true lines however coarse the pieces; the walk is then done twice (the second time for the far side
    // of such a line), into two tables of four texels per piece.
    // ROUTES=1 (experiment, route_light.mat): one shape per way the light goes. The pieces only serve to find the ways
    // (a few per face are enough: PIECES=4); nothing is cut along their grid, so EDGES has no part in it.
    // BEAMS=1: the light of the flat-faced solids by splitting beams exactly, on the CPU (beam_split.h). Every patch
    // comes out as its own outline; nothing is found by rays and nothing is checked per pixel. BEAMSTAT=1 counts.
    // GBEAMS=1: the same splitting done on the GPU, level by level in tables (gbeam.glsl); the CPU hands over poses only.
    const bool flatGBeams = env_f("GBEAMS", 0.f) > 0.5f && !rainbowOnce;
    static std::vector<View*> gbeamViews;   // in the order they are to be drawn
    // GBEAMS=2: the same with ONE fragment per beam: a level is a render target of several pictures, written at once by
    // a post-process material (mrt_probe.mm shows that Filament lets an application do this). Filament 1.77.2 on Metal
    // writes only the first FOUR colour attachments (MetalState.mm:62), 16 numbers, and an exact beam needs about 28
    // (up to 12 corners): so this version packs each corner into one number, on a grid of 1/4096 of its wall, and
    // keeps 11 corners. It is NOT exact; it is here to measure what one fragment per beam costs (gbm.glsl).
    const bool gbMrt = flatGBeams && env_f("GBEAMS", 0.f) > 1.5f;
    static std::vector<MaterialInstance*> gbmCommit;   // post-process material instances: to be committed by hand each frame
    // CBEAMS=1 (BACKEND=webgpu): the same splitting as BEAMS=1, as compute on the GPU (cbeam.wgsl); the CPU hands over
    // poses only and reads nothing back. CBEAMCHECK=1 (tests): the CPU's splitting is run beside it and the two
    // sets of triangles are compared, which reads the GPU's back.
#ifdef GLASS_COMPUTE_DOOR
    const bool cBeams = viaDawn && env_f("CBEAMS", 0.f) > 0.5f && !rainbowOnce && !flatGBeams;
#else
    const bool cBeams = false;
#endif
    const bool cCheck = cBeams && env_f("CBEAMCHECK", 0.f) > 0.5f;
    // PAIRS=1 (first proof; one kind of solid, no films): the light that goes in by one face and out by another, and
    // the light mirrored off the outside, drawn with no cutting of ours: pair_light.mat draws the face the light leaves
    // by and hands the GPU's own clipper the edges of the face it came in by. Nothing that bounced inside.
    // PAIRSTEPS: colour steps (64: every fine band on its own).
    const bool pairsOn = env_f("PAIRS", 0.f) > 0.5f && !rainbowOnce && !flatGBeams && !cBeams;
    const bool flatBeams = (env_f("BEAMS", 0.f) > 0.5f || cBeams || pairsOn) && !flatGBeams;
    static std::vector<std::array<beam::SplitShape, 16>> beamShapes;   // per shape, per set of films
    struct BeamPose {
        beam::V3 R[3], at;
        double s;
    };
    static std::vector<BeamPose> beamPose;
    const bool flatRoutes = env_f("ROUTES", 0.f) > 0.5f && !rainbowOnce && !flatBeams && !flatGBeams;
    const bool flatClip = env_f("EDGES", 0.f) > 0.5f && !rainbowOnce && !flatRoutes;
    const int flatPer = (flatClip || flatRoutes) ? 4 : 3;
    std::vector<MaterialInstance*> routeMis;
    // ROUTES=2: the ways are not found by rays but worked out from where the faces and films are, level by level
    // (route_tree.mat), and every leaf is drawn by route_draw.mat. ROUTESTAT=1 counts the leaves and their light.
    const bool routeTree = flatRoutes && env_f("ROUTES", 0.f) > 1.5f;
    struct RouteLevel {
        Target table;
        View* view;
        int slots, rows;
    };
    static std::vector<RouteLevel> routeLevels;
    if (flatMostPieces * flatPer > 16384) return fail("too many pieces for one line of the walk table (PIECES is too high)");
    Target flatWalk = make_rt(flatMostPieces * flatPer, (int)items.size() * flatLines, Texture::InternalFormat::RGBA32F);
    Target flatWalk2 = flatClip ? make_rt(flatMostPieces * flatPer, (int)items.size() * flatLines, Texture::InternalFormat::RGBA32F) : Target{nullptr, nullptr};
    const TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    MaterialInstance* flatWalkMi = flatWalkMat->createInstance();
    MaterialInstance* flatWalk2Mi = flatClip ? flatWalkMat->createInstance() : nullptr;
// what the walk is told goes to both of its turns
#define FLAT_WALK_SET(...) do { flatWalkMi->setParameter(__VA_ARGS__); if (flatWalk2Mi) flatWalk2Mi->setParameter(__VA_ARGS__); } while (0)
    if (!rainbowOnce) {
        flatWalkMi->setParameter("clip", flatRoutes ? 3.f : (flatClip ? 1.f : 0.f));
        flatWalkMi->setParameter("first", flatPieces, nearest);   // (not read on the first turn)
        if (flatWalk2Mi) {
            flatWalk2Mi->setParameter("clip", 2.f);
            flatWalk2Mi->setParameter("first", flatWalk.color, nearest);
        }
    }
    FLAT_WALK_SET("pose", poseTex, nearest);
    FLAT_WALK_SET("pieces", flatPieces, nearest);
    FLAT_WALK_SET("scene", sceneTex, nearest);
    FLAT_WALK_SET("solidCount", (int32_t)items.size());
    FLAT_WALK_SET("stepCount", (float)flatLines);
    FLAT_WALK_SET("floorY", 0.f);
    Scene* sFlatWalk = engine->createScene();
    sFlatWalk->addEntity(renderable(sqVB, sqIB, flatWalkMi, false, false));
    Scene* sFlatWalk2 = engine->createScene();
    if (flatWalk2Mi) sFlatWalk2->addEntity(renderable(sqVB, sqIB, flatWalk2Mi, false, false));
    for (FlatKind& fk : flatKinds) {
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(fk.shape.corners.size() / 5)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                               .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(fk.shape.corners.data(), fk.shape.corners.size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)fk.shape.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(fk.shape.idx.data(), fk.shape.idx.size() * sizeof(uint32_t)));
        float4 which[16] = {};
        for (size_t i = 0; i < fk.solids.size() && i < 64; ++i) which[i / 4][i % 4] = (float)fk.solids[i];
        for (int turn = 0; turn < (flatClip ? 2 : 1); ++turn) {   // (EDGES: once more, from the walk's second table)
            MaterialInstance* mi = flatMat->createInstance();
            on_stage(mi, true);
            mi->setParameter("walk", turn == 0 ? flatWalk.color : flatWalk2.color, nearest);
            mi->setParameter("stepCount", (float)flatLines);
            mi->setParameter("hull", flatHull, nearest);
            mi->setParameter("shape", (float)(&fk - flatKinds.data()));
            mi->setParameter("floorY", 0.f);
            if (!rainbowOnce) mi->setParameter("clip", flatClip ? 1.f : 0.f);
            mi->setParameter("solids", which, 16);
            const utils::Entity e = em.create();
            RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
                .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
                .instances(fk.solids.size() * (size_t)flatDraws * recvKinds)
                .material(0, mi).culling(false).castShadows(false).receiveShadows(false)
                .layerMask(0xFF, (flatOnCpu || flatRoutes || flatBeams || flatGBeams || std::getenv("NOLIGHT")) ? 0x00 : 0x01).build(*engine, e);
            sLight->addEntity(e);
            if (turn == 0) { fk.mi = mi; fk.entity = e; } else fk.mi2 = mi;
            if (turn == 1 && std::getenv("XDRAW2")) rm.setLayerMask(rm.getInstance(e), 0xFF, 0x00);   // (XDRAW2: timing only, its pieces not drawn)
            if (turn == 0 && std::getenv("XDRAW1")) rm.setLayerMask(rm.getInstance(e), 0xFF, 0x00);   // (XDRAW1: to look at the second turn alone)
        }
    }
    if (flatBeams) {
        for (FlatKind& fk : flatKinds) {
            beamShapes.emplace_back();
            for (int use = 0; use < 16; ++use) beamShapes.back()[(size_t)use] = beam::split_shape(items[(size_t)fk.solids[0]].local, use);
            std::printf("  beams: %s, all its films together cut it into %zu cells\n", fk.name.c_str(), beamShapes.back()[15].cells.size());
        }
        beamPose.resize(items.size());
    }
    MaterialInstance* pairMi = nullptr;
    int pairSteps = 0;
    if (pairsOn) {
        if (flatKinds.size() != 1 || stageOn) return fail("PAIRS: one kind of flat-faced solid, and no stage (first proof)");
        const FlatKind& fk = flatKinds[0];
        const beam::Solid& so = items[(size_t)fk.solids[0]].local;
        if (!so.films.empty()) return fail("PAIRS: a solid with films is not for this proof");
        if (fk.solids.size() > 64) return fail("PAIRS: 64 solids at most");
        std::vector<int> face;   // the faces that are there
        for (size_t f = 0; f < so.faces.size(); ++f)
            if (so.faces[f].n >= 3) face.push_back((int)f);
        int mostCorners = 0;
        auto* tab = new std::vector<float>(face.size() * 9 * 4, 0.f);
        for (size_t i = 0; i < face.size(); ++i) {
            const beam::Poly& F = so.faces[(size_t)face[i]];
            mostCorners = std::max(mostCorners, F.n);
            for (int c = 0; c < std::min(F.n, 8); ++c) {
                float* o = tab->data() + (i * 9 + (size_t)c) * 4;
                o[0] = (float)F.p[(size_t)c].x; o[1] = (float)F.p[(size_t)c].y; o[2] = (float)F.p[(size_t)c].z; o[3] = c == 0 ? (float)F.n : 0.f;
            }
            float* o = tab->data() + (i * 9 + 8) * 4;
            const beam::Plane& pl = so.planes[(size_t)face[i]];
            o[0] = (float)pl.n.x; o[1] = (float)pl.n.y; o[2] = (float)pl.n.z; o[3] = (float)pl.o;
        }
        if (mostCorners > 8) return fail("PAIRS: a face has more than 8 corners (the GPU clips by 8 planes a draw)");
        Texture* faceTex = Texture::Builder().width(9).height((uint32_t)face.size()).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
        faceTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(tab->data(), tab->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                     [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, tab));
        static std::vector<float> pv;        // (A, B, corner of B) per vertex
        static std::vector<uint32_t> pi;
        for (size_t a = 0; a < face.size(); ++a)
            for (size_t b = 0; b < face.size(); ++b) {
                const int nb = so.faces[(size_t)face[b]].n;
                for (int c = 1; c + 1 < nb; ++c)
                    for (int q : {0, c, c + 1}) {
                        pi.push_back((uint32_t)(pv.size() / 3));
                        pv.insert(pv.end(), {(float)a, (float)b, (float)q});
                    }
            }
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(pv.size() / 3)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12).build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(pv.data(), pv.size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)pi.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(pi.data(), pi.size() * sizeof(uint32_t)));
        Material* pairMat = mat("pair_light.filamat");
        if (!pairMat) return fail("pair_light.filamat");
        pairSteps = std::min(64, std::max(1, (int)env_f("PAIRSTEPS", 64.f)));
        pairMi = pairMat->createInstance();
        pairMi->setParameter("faces", faceTex, nearest);
        pairMi->setParameter("stepCount", (float)pairSteps);
        pairMi->setParameter("gain", env_f("SPECGAIN", 1.f));
        pairMi->setParameter("floorY", 0.f);
        pairMi->setParameter("probe", env_f("PAIRPROBE", 0.f));   // (timing only: pair_light.mat)
        const utils::Entity e = em.create();
        RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .instances(fk.solids.size() * (size_t)pairSteps)
            .material(0, pairMi).culling(false).castShadows(false).receiveShadows(false)
            .layerMask(0xFF, std::getenv("NOLIGHT") ? 0x00 : 0x01).build(*engine, e);
        sLight->addEntity(e);
        std::printf("  pairs: %s, %zu faces (most corners %d), %zu pairs of faces as %zu triangles, drawn %zu times a frame (%zu solids x %d colour steps): %zu triangles handed to the clipper\n",
                    fk.name.c_str(), face.size(), mostCorners, face.size() * face.size(), pi.size() / 3, fk.solids.size() * (size_t)pairSteps, fk.solids.size(), pairSteps,
                    pi.size() / 3 * fk.solids.size() * (size_t)pairSteps);
    }
#ifdef GLASS_COMPUTE_DOOR
    cbeam::Run* cRun = nullptr;
    uint32_t cSolids = 0, cStepsAll = 0;
    static std::vector<std::vector<float>> cCpuTris;   // CBEAMCHECK: what the CPU's splitting gave this frame, per solid
    if (cBeams) {
        if (stageOn) return fail("CBEAMS: not on a stage yet (the stage draws the light once per receiver)");
        cbeam::Table tb;
        if (!cbeam::table(beamShapes, tb)) return fail("CBEAMS: a face has more than 16 corners, or the table is too long");
        const uint32_t vertCap = (uint32_t)env_f("CBEAMVERTS", 393216.f) / 3 * 3, beamCap = (uint32_t)env_f("CBEAMCAP", 65536.f);
        if (beamCap & (beamCap - 1)) return fail("CBEAMCAP must be a power of two (cbeam.wgsl: place)");
        const std::vector<uint8_t> text = load_file(GLASS_PATH("cbeam.wgsl").c_str());
        if (text.empty()) return fail("cbeam.wgsl");
        BufferObject* bo = BufferObject::Builder().size(vertCap * 28).bindingType(BufferObject::BindingType::VERTEX).build(*engine);
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount(vertCap).bufferCount(1).enableBufferObjects()
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                               .attribute(VertexAttribute::COLOR, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28).build(*engine);
        vb->setBufferObjectAt(*engine, 0, bo);
        cRun = cbeam::open(engine, std::string(text.begin(), text.end()), tb, bo, vertCap, beamCap, (uint32_t)env_f("CBEAMLEVELS", 31.f), (uint32_t)items.size());
        if (!cRun) return fail("cbeam.wgsl: a pass is missing");
        const utils::Entity e = renderable(vb, lightIB, lightMi, false, false, 100.f);
        rm.setGeometryAt(rm.getInstance(e), 0, RenderableManager::PrimitiveType::TRIANGLES, vb, lightIB, 0, vertCap);
        if (std::getenv("NOLIGHT") || std::getenv("XCDRAW")) rm.setLayerMask(rm.getInstance(e), 0xFF, 0x00);   // (XCDRAW: timing only, computed but not drawn)
        sLight->addEntity(e);
        std::printf("  cbeams: compute on the GPU, table of %zu vec4s, %u levels of %u beams, room for %u triangles, most seeds to a shape %u\n", tb.t.size() / 4,
                    cRun->levels, beamCap, vertCap / 3, tb.seedMost);
    }
#endif
    if (gbMrt) {
        Material* nodeMat = mat("gbm_node.filamat");
        Material* indexMat = mat("gbm_index.filamat");
        Material* drawMat = mat("gbm_draw.filamat");
        if (!nodeMat || !indexMat || !drawMat) return fail("gbm_node / gbm_index / gbm_draw .filamat");
        const int levels = std::min(16, std::max(1, (int)env_f("GBEAMLEVELS", 8.f)));
        const int wide = std::min(1024, std::max(8, (int)env_f("GBEAMW", 96.f)));
        const int WALLS = 18, WALL_W = 11;
        auto table_view = [&](MaterialInstance* mi, RenderTarget* rt, int w, int h) {
            Scene* sc = engine->createScene();
            sc->addEntity(renderable(sqVB, sqIB, mi, false, false));
            View* v = engine->createView();
            v->setScene(sc);
            v->setRenderTarget(rt);
            v->setViewport({0, 0, (uint32_t)w, (uint32_t)h});
            v->setDithering(View::Dithering::NONE);
            v->setPostProcessingEnabled(false);
            v->setShadowingEnabled(false);
            gbeamViews.push_back(v);
        };
        for (FlatKind& fk : flatKinds) {
            using beam::V3;
            const beam::Solid& loc = items[(size_t)fk.solids[0]].local;
            auto* cellTexels = new std::vector<float>((size_t)WALLS * WALL_W * 256 * 4, 0.f);
            int mostWalls = 0, mostCorners = 0;
            for (int use = 0; use < 16; ++use) {
                const beam::SplitShape sh = beam::split_shape(loc, use);
                for (size_t c = 0; c < sh.cells.size() && c < 16; ++c) {
                    const beam::SplitCell& cell = sh.cells[c];
                    mostWalls = std::max(mostWalls, (int)cell.planes.size());
                    for (size_t w = 0; w < cell.planes.size() && w < (size_t)WALLS; ++w) {
                        const beam::Poly& F = cell.faces[w];
                        if (F.n < 3) continue;
                        float* o = cellTexels->data() + (((size_t)use * 16 + c) * (size_t)(WALLS * WALL_W) + w * (size_t)WALL_W) * 4;
                        const V3 nrm = cell.planes[w].n, org = nrm * cell.planes[w].o;
                        const V3 u = beam::normalize(F.p[1] - F.p[0]), v = beam::cross(nrm, u);
                        const int cn = std::min(12, F.n);
                        mostCorners = std::max(mostCorners, F.n);
                        o[0] = (float)nrm.x; o[1] = (float)nrm.y; o[2] = (float)nrm.z; o[3] = (float)cell.planes[w].o;
                        o[4] = cell.what[w] >= 0 ? (float)cell.what[w] : -1.f;
                        o[6] = (float)cn;
                        o[8] = (float)u.x; o[9] = (float)u.y; o[10] = (float)u.z;
                        o[12] = (float)v.x; o[13] = (float)v.y; o[14] = (float)v.z;
                        float lo[2] = {1e30f, 1e30f}, hi[2] = {-1e30f, -1e30f};
                        for (int k = 0; k < cn; ++k) {
                            const float a = (float)beam::dot(F.p[(size_t)k] - org, u), b = (float)beam::dot(F.p[(size_t)k] - org, v);
                            o[20 + k * 2] = a;
                            o[21 + k * 2] = b;
                            lo[0] = std::min(lo[0], a); lo[1] = std::min(lo[1], b);
                            hi[0] = std::max(hi[0], a); hi[1] = std::max(hi[1], b);
                        }
                        o[16] = lo[0]; o[17] = lo[1]; o[18] = hi[0]; o[19] = hi[1];
                    }
                }
            }
            if (mostWalls > WALLS || mostCorners > 12) return fail("GBEAMS=2: a cell has more than 18 walls, or a wall more than 12 corners");
            Texture* cellTex = Texture::Builder().width((uint32_t)(WALLS * WALL_W)).height(256).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
            cellTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(cellTexels->data(), cellTexels->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                         [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, cellTexels));
            float4 which[16] = {};
            for (size_t i = 0; i < fk.solids.size() && i < 64; ++i) which[i / 4][i % 4] = (float)fk.solids[i];
            const int rows = (int)fk.solids.size() * flatLines;
            const int OUTS = 4;
            static const char* prevName[4] = {"prev0", "prev1", "prev2", "prev3"};
            static const char* nodeName[4] = {"n0", "n1", "n2", "n3"};
            Texture* before[4];
            for (Texture*& b : before) b = cellTex;   // (a stand-in where a level has nothing before it)
            int beforeWide = 0;
            const auto use8 = Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC;
            for (int lv = 0; lv <= levels; ++lv) {
                const int places = lv == 0 ? 16 * WALLS * 3 : wide;
                Target index{nullptr, nullptr};
                if (lv > 0) {
                    index = make_rt(places, rows, Texture::InternalFormat::RGBA32F);
                    MaterialInstance* imi = indexMat->createInstance();
                    imi->setParameter("nodes", before[0], nearest);
                    imi->setParameter("walls", before[1], nearest);
                    imi->setParameter("nodesWide", (float)beforeWide);
                    table_view(imi, index.rt, places, rows);
                }
                Texture* now[4];
                RenderTarget::Builder rtb;
                for (int a = 0; a < OUTS; ++a) {
                    now[a] = Texture::Builder().width((uint32_t)places).height((uint32_t)rows).levels(1).usage(use8).format(Texture::InternalFormat::RGBA32F).build(*engine);
                    rtb.texture((RenderTarget::AttachmentPoint)a, now[a]);
                }
                RenderTarget* nodesRt = rtb.build(*engine);
                MaterialInstance* nmi = nodeMat->createInstance();
                nmi->setParameter("cells", cellTex, nearest);
                for (int a = 0; a < OUTS; ++a) nmi->setParameter(prevName[a], before[a], nearest);
                nmi->setParameter("index", lv > 0 ? index.color : cellTex, nearest);
                nmi->setParameter("pose", poseTex, nearest);
                nmi->setParameter("stepCount", (float)flatLines);
                nmi->setParameter("solids", which, 16);
                nmi->setParameter("solidsHere", (float)fk.solids.size());
                nmi->setParameter("level", (float)lv);
                nmi->setParameter("least", env_f("BEAMCUT", 2e-5f));
                nmi->setParameter("mostFaces", env_f("BEAMFACES", 8.f));
                routeMis.push_back(nmi);
                gbmCommit.push_back(nmi);
                table_view(nmi, nodesRt, places, rows);
                auto* v = new std::vector<float>();
                auto* ix = new std::vector<uint32_t>();
                for (int i = 0; i < places; ++i) {
                    const uint32_t base = (uint32_t)(v->size() / 3);
                    for (int k = 0; k < 12; ++k) v->insert(v->end(), {(float)i, (float)k, 0.f});
                    for (uint32_t k = 1; k + 1 < 12; ++k) ix->insert(ix->end(), {base, base + k, base + k + 1});
                }
                VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(v->size() / 3)).bufferCount(1)
                                       .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12).build(*engine);
                vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(v->data(), v->size() * sizeof(float)));
                IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)ix->size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
                ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(ix->data(), ix->size() * sizeof(uint32_t)));
                MaterialInstance* dmi = drawMat->createInstance();
                dmi->setParameter("cells", cellTex, nearest);
                for (int a = 0; a < OUTS; ++a) dmi->setParameter(nodeName[a], now[a], nearest);
                dmi->setParameter("pose", poseTex, nearest);
                dmi->setParameter("stepCount", (float)flatLines);
                dmi->setParameter("solids", which, 16);
                dmi->setParameter("floorY", 0.f);
                routeMis.push_back(dmi);
                const utils::Entity e = em.create();
                RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
                    .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
                    .instances((size_t)rows)
                    .material(0, dmi).culling(false).castShadows(false).receiveShadows(false)
                    .layerMask(0xFF, std::getenv("NOLIGHT") ? 0x00 : 0x01).build(*engine, e);
                sLight->addEntity(e);
                for (int a = 0; a < OUTS; ++a) before[a] = now[a];
                beforeWide = places;
            }
            std::printf("  gbeams, one fragment a beam: %s, %d levels of %d nodes after the first, %d lines; most walls to a cell %d, most corners to a wall %d\n", fk.name.c_str(),
                        levels, wide, rows, mostWalls, mostCorners);
        }
    } else if (flatGBeams) {
        Material* nodeMat = mat("gbeam_node.filamat");
        Material* indexMat = mat("gbeam_index.filamat");
        Material* drawMat = mat("gbeam_draw.filamat");
        if (!nodeMat || !indexMat || !drawMat) return fail("gbeam_node / gbeam_index / gbeam_draw .filamat");
