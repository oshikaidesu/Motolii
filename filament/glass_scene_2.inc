// glass_scene.mm, piece 2 (cut there by size, as it was): read only in its place in glass_scene.mm.
        for (int i = 0; i < 64; ++i) {   // fixed made-up patches: a spiral of slanted four-sided ones round the middle
            const float an = 0.7f * i, rr = 0.25f + 0.035f * i, tilt = 1.3f * i;
            const float cx = rr * std::cos(an), cz = rr * std::sin(an), hw = 0.10f + 0.004f * (i % 7), hh = 0.05f + 0.006f * (i % 5);
            const float e[8] = {cx, cz, hw * std::cos(tilt), hw * std::sin(tilt), -hh * std::sin(tilt + 0.3f), hh * std::cos(tilt + 0.3f), 0.004f + 0.002f * (i % 9), 0.25f};
            std::copy(e, e + 8, polys.begin() + i * 8);
        }
        Texture* polyTex = Texture::Builder().width(128).height(1).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
        polyTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(polys.data(), polys.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
        floorMi->setParameter("polys", polyTex, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST));
        floorMi->setParameter("polyCount", (int32_t)polyTest);
    }
    static std::vector<float> floor_v;
    static std::vector<uint32_t> floor_i = {0, 2, 1, 0, 3, 2};
    {
        const float F = env_f("FLOOR", 6.f);
        const float3 fp[4] = {{-F, 0, -F}, {F, 0, -F}, {F, 0, F}, {-F, 0, F}};
        for (int i = 0; i < 4; ++i) push_vert(floor_v, fp[i], {1, 0, 0}, {0, 1, 0});
    }
    auto [floorVB, floorIB] = lit_buffers(floor_v, floor_i);
    // on a stage the floor, the walls and the blocks are all the same kind of surface (recv_beam.mat)
    std::vector<MaterialInstance*> recvSurfaces;   // they are told where the sun is
    auto recv_surface = [&](float4 color, float rough, Texture* from) {
        MaterialInstance* mi = recvBeamMat->createInstance();
        mi->setParameter("color", color);
        mi->setParameter("roughness", rough);
        mi->setParameter("reflectance", env_f("FLOOR_REFL", 0.35f));
        mi->setParameter("gain", sunColor * (lux / 3.14159265f));
        mi->setParameter("light", from, lin);
        recvSurfaces.push_back(mi);
        return mi;
    };
    scene->addEntity(renderable(floorVB, floorIB, stageOn ? recv_surface(float4{0.86f, 0.85f, 0.84f, 1}, env_f("FLOOR_ROUGH", 0.42f), soft.color) : floorMi, false, true, 8.f));

    // ---- the solids
    // FILMBANDS: the colours (nanometres, from-to) that the films inside a solid mirror, in the order of beam::Solid::films
    static float4 filmNm[4] = {{380, 490, 0, 0}, {500, 590, 0, 0}, {600, 720, 0, 0}, {380, 490, 0, 0}};
    if (const char* fb = std::getenv("FILMBANDS"))
        std::sscanf(fb, "%f-%f,%f-%f,%f-%f,%f-%f", &filmNm[0].x, &filmNm[0].y, &filmNm[1].x, &filmNm[1].y, &filmNm[2].x, &filmNm[2].y, &filmNm[3].x, &filmNm[3].y);
    std::vector<Item> items;
    {
        using namespace beam;
        const double c = chamfer;  // flat width, in the solids' own units (they are about 1 across)
        auto add = [&](const char* kind, Solid s, V3 down, float size) {
            Item it{};
            it.kind = kind;
            it.local = std::move(s);
            it.down = down;
            it.size = size;
            items.push_back(std::move(it));
        };
        Solid tri3 = chamfered(prism(3, 0.55, 0.60), c), hex6 = chamfered(prism(6, 0.50, 0.38), c);
        add("cube", chamfered(box(0.5, 0.5, 0.5), c), {0, -1, 0}, 0.30f);
        add("slab", chamfered(box(0.80, 0.16, 0.55), c), {0, -1, 0}, 0.30f);
        add("prism", tri3, tri3.planes[0].n, 0.30f);                     // lying on a side: the classic prism
        add("pillar", chamfered(box(0.26, 0.75, 0.26), c), {0, -1, 0}, 0.30f);
        add("hexagon", hex6, {0, -1, 0}, 0.30f);
        add("bar", chamfered(box(0.95, 0.20, 0.20), c), {0, -1, 0}, 0.30f);
        add("pyramid", chamfered(pyramid(4, 0.62, 0.70), c), {0, -1, 0}, 0.30f);
        add("wedge", chamfered(prism(3, 0.50, 0.30), c), {0, -1, 0}, 0.30f);  // a standing triangular prism
        {   // glass with colour films inside (beam::Solid::films): the colour-splitting cube, and the same with its corners cut
            const double q = std::sqrt(0.5), t = std::sqrt(1.0 / 3.0);
            const std::vector<Plane> cross = {{{q, 0, q}, 0}, {{q, 0, -q}, 0}};
            Solid xc = chamfered(box(0.5, 0.5, 0.5), c);
            xc.films = cross;
            add("xcube", xc, {0, -1, 0}, 0.30f);
            std::vector<Plane> cut = box(0.5, 0.5, 0.5).planes;
            for (int i = 0; i < 8; ++i) cut.push_back({{(i & 1 ? t : -t), (i & 2 ? t : -t), (i & 4 ? t : -t)}, (double)env_f("XCUT", 0.62f)});
            Solid xk = from_planes(cut, 4.0);
            // four films, each aslant and none through the middle: much of the light then meets one film only and
            // comes out as white less one colour (yellow, pink, pale blue) instead of as red, green or blue
            xk.films = {{{q, 0, q}, 0.2}, {{q, 0, -q}, 0.2}, {{q, q, 0}, -0.2}, {{0, q, q}, 0.25}};
            if (const char* xf = std::getenv("XFILMS")) {   // XFILMS="nx,ny,nz,o nx,ny,nz,o ...": other films for this solid (up to 4)
                xk.films.clear();
                float a, b, d, o;
                int used = 0;
                while (xk.films.size() < 4 && std::sscanf(xf, " %f,%f,%f,%f%n", &a, &b, &d, &o, &used) == 4) {
                    xk.films.push_back({normalize(V3{a, b, d}), o});
                    xf += used;
                }
            }
            add("xcut", xk, {0, -1, 0}, 0.30f);
        }
        Solid oct = chamfered(octahedron(0.62), c);
        add("octahedron", oct, oct.planes[0].n, 0.30f);
        add("ball", ball(0.5), {0, -1, 0}, 0.30f);
        add("rod", rod(0.30, 0.70), {1, 0, 0}, 0.30f);       // a round bar lying on its side
        add("column", rod(0.34, 0.55), {0, -1, 0}, 0.30f);   // the same shape standing
        add("cone", cone(0.45, 0.95), {0, -1, 0}, 0.30f);
        add("conelying", cone(0.45, 0.95), {1, 0.45 / 0.95, 0}, 0.30f);  // the same shape on its side
        // with dents and holes (the outline lies in the solid's x, y; z is its thickness)
        const Solid starS = star(5, 0.5, 0.225, 0.12, c), washerS = washer(0.5, 0.25, 0.11, c);
        add("star", starS, {0, 0, -1}, 0.30f);           // lying flat
        add("washer", washerS, {0, 0, -1}, 0.30f);
        add("letterA", letter('A', 0.15, c), {0, -1, 0}, 0.30f);   // letters stand
        add("letterB", letter('B', 0.15, c), {0, -1, 0}, 0.30f);
        add("letterO", letter('O', 0.15, c), {0, -1, 0}, 0.30f);
        add("letterM", letter('M', 0.15, c), {0, -1, 0}, 0.30f);
        add("letterR", letter('R', 0.15, c), {0, -1, 0}, 0.30f);
        add("starup", starS, {0, -1, 0}, 0.30f);         // standing on two tips
        add("ring", washerS, {0, -1, 0}, 0.30f);         // the washer on its rim
        if (const char* only = std::getenv("ONLY")) {
            std::vector<Item> keep;
            // one kind, or several with commas between; "holes" = every solid with dents and holes
            const std::string list = std::string(",") + only + ",";
            for (Item& it : items)
                if (list.find(std::string(",") + it.kind + ",") != std::string::npos || (list.find(",holes,") != std::string::npos && it.local.kind == beam::Solid::EXTRUDE))
                    keep.push_back(std::move(it));
            items = std::move(keep);
        }
        const int want = (int)env_f("COUNT", (float)items.size());
        const size_t kinds = items.size();
        if (kinds == 0) return fail("no such kind");
        for (int i = (int)kinds; i < want; ++i) items.push_back(items[(size_t)i % kinds]);  // more of the same kinds
        if ((int)items.size() > want) items.resize((size_t)std::max(1, want));
        // a loose grid, each turned differently (fixed numbers: the same scene every run)
        // (PROPS: matte blocks take some of the places, spread through the grid; without them nothing changes here)
        const int propPlaces = std::getenv("PROPAT") ? 0 : propWant;
        const int n = (int)items.size() + propPlaces, cols = (int)std::ceil(std::sqrt((double)n * 1.4));
        auto place = [&](int at, float& x, float& z) {
            const int cx = at % cols, cz = at / cols;
            x = (cx - 0.5f * (cols - 1)) * 0.62f + ((cz & 1) ? 0.18f : 0.f);
            z = (cz - 0.5f * ((n - 1) / cols)) * 0.62f;
        };
        std::vector<int> propAt;
        for (int j = 0; j < propPlaces; ++j) {
            static const float half[4][3] = {{0.17f, 0.06f, 0.17f}, {0.12f, 0.20f, 0.12f}, {0.22f, 0.09f, 0.10f}, {0.13f, 0.13f, 0.13f}};
            static const float turn[4] = {0.3f, -0.5f, 0.9f, 0.2f};
            // spread over the rows, and never in the last place of a row: a solid then stands beside it on the side the
            // default sun of a stage comes from, and throws its light on it
            const int rows = (n + cols - 1) / cols;
            int cell = std::min(n - 2, ((2 * j + 1) * rows / (2 * propPlaces)) * cols + std::max(0, cols - 2 - j % 2));
            while (cell > 0 && std::find(propAt.begin(), propAt.end(), cell) != propAt.end()) --cell;
            propAt.push_back(cell);
            PropSpec ps{0, 0, half[j][0], half[j][1], half[j][2], turn[j]};
            place(propAt.back(), ps.x, ps.z);
            props.push_back(ps);
        }
        int at = 0;
        for (int i = 0; i < (int)items.size(); ++i, ++at) {
            Item& it = items[(size_t)i];
            while (std::find(propAt.begin(), propAt.end(), at) != propAt.end()) ++at;
            place(at, it.x, it.z);
            it.yaw = 0.35f + 0.83f * (float)i;
            it.spin = (i & 1) ? 1.0f : -0.7f;
            it.drift = 0.05f;
            it.phase = 1.7f * (float)i;
        }
        if (std::getenv("SCATTER")) {
            // strewn about like dice instead of the grid: anywhere in a patch of floor that grows with their number, not
            // touching, each of its own size and turn, flat-faced ones on any of their faces (the same every run; SEED: another throw)
            uint32_t seed = 0x9E3779B9u * (uint32_t)(1 + (int)env_f("SEED", 0.f));
            auto rnd = [&seed]() { seed = seed * 1664525u + 1013904223u; return (float)(seed >> 8) / 16777216.f; };
            const float side = 0.85f * std::sqrt((float)items.size());
            for (int i = 0; i < (int)items.size(); ++i) {
                Item& it = items[(size_t)i];
                it.size = 0.22f + 0.14f * rnd();
                it.yaw = 6.2832f * rnd();
                if (it.local.kind == beam::Solid::FLAT && !it.local.planes.empty()) {
                    const beam::V3 nrm = it.local.planes[(size_t)(rnd() * (float)it.local.planes.size()) % it.local.planes.size()].n;
                    it.down = nrm;
                }
                for (int tries = 0; tries < 200; ++tries) {
                    it.x = (rnd() - 0.5f) * side;
                    it.z = (rnd() - 0.5f) * side;
                    bool clear = true;
                    for (int j = 0; j < i && clear; ++j)
                        clear = std::hypot(it.x - items[(size_t)j].x, it.z - items[(size_t)j].z) > 1.25f * (it.size + items[(size_t)j].size);
                    if (clear) break;
                }
            }
        }
    }
    // ---- the stage (see the top): the room's planes and the blocks. They stand still for the whole run.
    beam::Stage stage;
    std::vector<beam::Block> blocks;
    std::vector<beam::Half> walls;
    const float wallTop = env_f("WALLTOP", 3.f);
    static float4 roomT[6], blocksT[16];
    if (stageOn) {
        float lowX = 1e9f, lowZ = 1e9f;
        for (const Item& it : items) lowX = std::min(lowX, it.x), lowZ = std::min(lowZ, it.z);
        for (const PropSpec& ps : props) {
            beam::Block B;
            B.c = {ps.x, ps.hy, ps.z};
            B.ax[0] = {std::cos(ps.yaw), 0, -std::sin(ps.yaw)};
            B.ax[1] = {0, 1, 0};
            B.ax[2] = {std::sin(ps.yaw), 0, std::cos(ps.yaw)};
            B.h[0] = ps.hx; B.h[1] = ps.hy; B.h[2] = ps.hz;
            blocks.push_back(B);
            lowX = std::min(lowX, ps.x), lowZ = std::min(lowZ, ps.z);
        }
        // the walls stand behind the solids as the default camera sees them: one across (z), one along (x), meeting in
        // a corner that WALLYAW turns about the vertical
        const float gap = env_f("WALLGAP", 0.40f), turn = env_f("WALLYAW", 0.f) * 3.14159265f / 180.f;
        const beam::V3 corner{env_f("WALLX", lowX - gap), 0, env_f("WALLZ", lowZ - gap)};
        const beam::V3 inward[2] = {{std::sin(turn), 0, std::cos(turn)}, {std::cos(turn), 0, -std::sin(turn)}};
        for (int w = 0; w < wallCount; ++w) walls.push_back(beam::Half{inward[w], -beam::dot(inward[w], corner)});
        stage = beam::make_stage(0.0, walls, wallTop, blocks);
        if (stage.room.size() > 6) return fail("too many walls");
        for (size_t k = 0; k < stage.room.size(); ++k)
            roomT[k] = float4{(float)stage.room[k].a.x, (float)stage.room[k].a.y, (float)stage.room[k].a.z, (float)stage.room[k].b};
        for (size_t b = 0; b < blocks.size(); ++b) {
            blocksT[4 * b] = float4{(float)blocks[b].c.x, (float)blocks[b].c.y, (float)blocks[b].c.z, 0};
            for (int k = 0; k < 3; ++k)
                blocksT[4 * b + 1 + (size_t)k] = float4{(float)blocks[b].ax[k].x, (float)blocks[b].ax[k].y, (float)blocks[b].ax[k].z, (float)blocks[b].h[k]};
        }
    }
    // what the materials that drop light on a stage need to know (recv_plane.glsl, recv_lands.glsl)
    std::vector<MaterialInstance*> landers;   // they are told every frame which way the sunlight travels
    auto set_stage = [&](MaterialInstance* mi, bool lands) {
        mi->setParameter("room", roomT, 6);
        mi->setParameter("roomCount", (float)stage.room.size());
        mi->setParameter("blocks", blocksT, 16);
        mi->setParameter("blockCount", (float)blocks.size());
        if (lands) landers.push_back(mi);
    };


    // ---- the cones' light (see above)
    static std::vector<float> coneV;  // row, column, patch per corner
    static std::vector<uint32_t> coneI, coneITip;  // for a rod, and for a cone with its tip (nothing to draw on its top)
    beam::cone_mesh(coneV, coneI, true);
    {
        std::vector<float> same;
        beam::cone_mesh(same, coneITip, false);
    }
    VertexBuffer* coneVB = VertexBuffer::Builder().vertexCount((uint32_t)(coneV.size() / 3)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12).build(*engine);
    coneVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(coneV.data(), coneV.size() * sizeof(float)));
    IndexBuffer* coneIB = IndexBuffer::Builder().indexCount((uint32_t)coneI.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    coneIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(coneI.data(), coneI.size() * sizeof(uint32_t)));
    IndexBuffer* coneIBTip = IndexBuffer::Builder().indexCount((uint32_t)coneITip.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    coneIBTip->setBuffer(*engine, IndexBuffer::BufferDescriptor(coneITip.data(), coneITip.size() * sizeof(uint32_t)));
    const int CONE_SETS = 2, CONE_STEPS_MOST = 32;  // two renderables: the cones, then the rods (a rod is drawn as a cone without its tip)
    size_t coneRoom = 0, conesWithTip = 0;  // cones and rods in this scene; how many of them are cones
    for (const Item& it : items) {
        coneRoom += it.local.kind == beam::Solid::CONE || it.local.kind == beam::Solid::ROD;
        conesWithTip += it.local.kind == beam::Solid::CONE;
    }
    const int coneStepCount = std::min(CONE_STEPS_MOST, std::max(1, (int)env_f("CONESTEPS", (float)beam::Params().coneSteps)));
    const int coneWays = std::min(beam::CONE_WAYS, std::max(1, (int)env_f("CONEWAYS", (float)beam::CONE_WAYS)));
    const int coneLines = (int)std::max<size_t>(1, coneRoom) * coneStepCount * coneWays;
    Texture* coneTex = Texture::Builder().width((uint32_t)std::max<size_t>(1, coneRoom)).height(4).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
    Target coneWalk = make_rt((int)(coneV.size() / 3) * 2, coneLines, Texture::InternalFormat::RGBA32F);
    const float4 coneRows{(float)beam::CONE_ROWS[0], (float)beam::CONE_ROWS[1], (float)beam::CONE_ROWS[2], (float)beam::CONE_ROWS[3]};
    const float4 coneCols{(float)beam::CONE_COLS[0], (float)beam::CONE_COLS[1], (float)beam::CONE_COLS[2], (float)beam::CONE_COLS[3]};
    MaterialInstance* coneWalkMi = coneWalkMat->createInstance();
    coneWalkMi->setParameter("cones", coneTex, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST));
    coneWalkMi->setParameter("stepCount", (float)coneStepCount);
    coneWalkMi->setParameter("wayCount", (float)coneWays);
    coneWalkMi->setParameter("rows", coneRows);
    coneWalkMi->setParameter("cols", coneCols);
    coneWalkMi->setParameter("floorY", 0.f);
    coneWalkMi->setParameter("tip", (float)beam::CONE_TIP);
    coneWalkMi->setParameter("inset", (float)beam::CONE_INSET);
    coneWalkMi->setParameter("edge", (float)beam::CONE_EDGE);
    coneWalkMi->setParameter("flatter", (float)beam::BALL_FLAT);
    coneWalkMi->setParameter("reach", (float)beam::BALL_REACH);
    Scene* sConeWalk = engine->createScene();
    sConeWalk->addEntity(renderable(sqVB, sqIB, coneWalkMi, false, false));
    MaterialInstance* coneMi[CONE_SETS];
    utils::Entity coneE[CONE_SETS];
    for (int b = 0; b < CONE_SETS; ++b) {
        coneMi[b] = coneMat->createInstance();
        on_stage(coneMi[b], true);
        coneMi[b]->setParameter("walk", coneWalk.color, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST));
        coneMi[b]->setParameter("floorY", 0.f);
        coneMi[b]->setParameter("sunTan", std::tan(sunRadiusDeg * 3.14159265f / 180.f));
        coneMi[b]->setParameter("cap", (float)beam::CONE_CAP);
        coneMi[b]->setParameter("stepCount", (float)coneStepCount);
        coneMi[b]->setParameter("wayCount", (float)coneWays);
        coneMi[b]->setParameter("rows", coneRows);
        coneMi[b]->setParameter("cols", coneCols);
        coneE[b] = em.create();
        const size_t count = b == 0 ? conesWithTip : coneRoom - conesWithTip;
        coneMi[b]->setParameter("first", (float)(b == 0 ? 0 : conesWithTip));
        RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, coneVB, b == 0 ? coneIBTip : coneIB)
            .instances(std::max<size_t>(1, count) * (size_t)(coneStepCount * coneWays) * recvKinds)
            .material(0, coneMi[b]).culling(false).castShadows(false).receiveShadows(false).layerMask(0xFF, 0x00).build(*engine, coneE[b]);
        sLight->addEntity(coneE[b]);
    }

    // ---- solids with dents and holes (beam_extrude.h). Built once: the table of shapes (their edges, the stretches
    // their walls are divided into, their triangles) and two meshes that serve every shape. Per frame the CPU only
    // writes where each of these solids is (three texels); two fragment passes (extrude_find.mat, extrude_walk.mat)
    // work out what becomes of the light along every wall, and the strips and outlines are drawn from that
    // (extrude_strip.mat, extrude_cap.mat), instanced per solid and colour step.
    struct ExShapeUse {
        const beam::Outline* ol;
        int line;        // its first line in the table of shapes
        int first = 0;   // the column of its first solid in the table of poses
        int count = 0;   // how many solids have this shape
    };
    std::vector<ExShapeUse> exShapes;
    size_t exCount = 0;
    for (Item& it : items) {
        if (it.local.kind != beam::Solid::EXTRUDE) continue;
        size_t at = 0;
        while (at < exShapes.size() && exShapes[at].ol != it.local.shape.get()) ++at;
        if (at == exShapes.size()) exShapes.push_back(ExShapeUse{it.local.shape.get(), (int)(3 * at)});
        ++exShapes[at].count;
        ++exCount;
    }
    {
        int col = 0;
        for (ExShapeUse& sh : exShapes) {
            sh.first = col;
            int k = 0;
            for (Item& it : items)
                if (it.local.kind == beam::Solid::EXTRUDE && it.local.shape.get() == sh.ol) it.exColumn = col + k++;
            col += sh.count;
        }
    }
    const bool exOnCpu = std::getenv("EXTRUDECPU") != nullptr;  // draw the CPU's copy of their light and shadow instead (to compare)
    const size_t exRoom = std::max<size_t>(1, exCount), exShapeRoom = std::max<size_t>(1, exShapes.size());
    static std::vector<float> exTable(exShapeRoom * 3 * beam::EXT_TABLE * 4, 0.f);
    for (size_t i = 0; i < exShapes.size(); ++i)
        if (!beam::ex_shape_table(*exShapes[i].ol, exTable.data() + i * 3 * beam::EXT_TABLE * 4)) return fail("an outline is too large for the table of shapes");
    Texture* exShapeTex = Texture::Builder().width(beam::EXT_TABLE).height((uint32_t)(3 * exShapeRoom)).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
    exShapeTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(exTable.data(), exTable.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
    Texture* exPoseTex = Texture::Builder().width((uint32_t)exRoom).height(3).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
    Target exFind = make_rt(beam::EXT_COLUMNS, (int)exRoom * beam::EXT_FIND_ROWS, Texture::InternalFormat::RGBA32F);
    Target exWalk = make_rt(beam::EXT_COLUMNS, (int)exRoom * beam::EXT_ROWS, Texture::InternalFormat::RGBA32F);
    const TextureSampler exNearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    std::vector<MaterialInstance*> exMis;  // every instance of the four materials: they all take the sun and the colour steps
    auto ex_instance = [&](Material* m) {
        MaterialInstance* mi = m->createInstance();
        mi->setParameter("shapes", exShapeTex, exNearest);
        mi->setParameter("poses", exPoseTex, exNearest);
        mi->setParameter("floorY", 0.f);
        exMis.push_back(mi);
        return mi;
    };
    MaterialInstance* exFindMi = ex_instance(exFindMat);
    MaterialInstance* exWalkMi = ex_instance(exWalkMat);
    exWalkMi->setParameter("find", exFind.color, exNearest);
    Scene* sExFind = engine->createScene();
    sExFind->addEntity(renderable(sqVB, sqIB, exFindMi, false, false));
    Scene* sExWalk = engine->createScene();
    sExWalk->addEntity(renderable(sqVB, sqIB, exWalkMi, false, false));
    static std::vector<float> exStripV, exCapV;
    static std::vector<uint32_t> exSlimI, exPairI, exFullI, exCapI;
    beam::ex_strip_mesh(exStripV, exSlimI, exPairI, exFullI);
    beam::ex_cap_mesh(exCapV, exCapI);
    auto corner_buffer = [&](std::vector<float>& v) {
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(v.size() / 3)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12).build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(v.data(), v.size() * sizeof(float)));
        return vb;
    };
    auto index_buffer = [&](std::vector<uint32_t>& i) {
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)i.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(i.data(), i.size() * sizeof(uint32_t)));
        return ib;
    };
    VertexBuffer* exStripVB = corner_buffer(exStripV);
    VertexBuffer* exCapVB = corner_buffer(exCapV);
    IndexBuffer* exSlimIB = index_buffer(exSlimI);
    IndexBuffer* exPairIB = index_buffer(exPairI);
    IndexBuffer* exFullIB = index_buffer(exFullI);
    IndexBuffer* exCapIB = index_buffer(exCapI);
    std::vector<utils::Entity> exEntities;
    std::vector<int> exEntityBit;  // which kind of light each one draws (for EXSHOW)
    int exBit = 0;
    const int exShow = (int)env_f("EXSHOW", 255.f);  // debug: a bit per kind of light (1 mirrored, 2 foot, 4 through walls, 8 glint, 16 chamfers, 32 window, 64 shadow)
    size_t exTriangles = 0;  // drawn per frame for these solids (most of them folded up to nothing)
    {
        const bool mirrors = true;  // (beam::Params::outerReflection)
        size_t exPer = 1;  // times as many instances on a stage (set by `cap` and `strip` below)
        auto draw = [&](Scene* into, VertexBuffer* vb, IndexBuffer* ib, size_t indices, MaterialInstance* mi, size_t instances) {
            utils::Entity e = em.create();
            RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
                .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib, 0, indices).instances(instances * exPer)
                .material(0, mi).culling(false).castShadows(false).receiveShadows(false).build(*engine, e);
            into->addEntity(e);
            exEntities.push_back(e);
            exEntityBit.push_back(exBit);
            exTriangles += indices / 3 * instances;
        };
        for (const ExShapeUse& sh : exShapes) {
            const size_t stretches = sh.ol->stretches.size(), tris = sh.ol->tris.size() / 3;
            auto cap = [&](Scene* into, int roleBase, int per) {
                MaterialInstance* mi = ex_instance(exCapMat);
                on_stage(mi, false);
                exPer = recvTotal;
                mi->setParameter("first", (float)sh.first);
                mi->setParameter("per", (float)per);
                mi->setParameter("roleBase", (float)roleBase);
                draw(into, exCapVB, exCapIB, tris * 3, mi, (size_t)sh.count * (size_t)per);
            };
            auto strip = [&](Scene* into, int family, int stepBase, int per) {
                MaterialInstance* mi = ex_instance(exStripMat);
                on_stage(mi, true);
                exPer = recvKinds;
                mi->setParameter("find", exFind.color, exNearest);
                mi->setParameter("walk", exWalk.color, exNearest);
                mi->setParameter("sunTan", std::tan(sunRadiusDeg * 3.14159265f / 180.f));
                mi->setParameter("first", (float)sh.first);
                mi->setParameter("per", (float)per);
                mi->setParameter("stepBase", (float)stepBase);
                mi->setParameter("family", (float)family);
                mi->setParameter("chamfer", (float)sh.ol->chamfer);
                // (slot 0 of the mirrored light is what the walls take out of the window: the window's own shader sees to that)
                const int slots = family == beam::detail::EX_MIRROR ? 2 : beam::detail::ex_slots(family);
                draw(into, exStripVB, slots == 3 ? exFullIB : (slots == 2 ? exPairIB : exSlimIB), stretches * 18 * (size_t)slots, mi, (size_t)sh.count * (size_t)per);
            };
            using namespace beam::detail;
            exBit = 64;
            cap(sShade, 0, 1);
            strip(sShade, EX_SHADOW, 0, 1);
            exBit = 32;
            cap(sLight, 1, mirrors ? 2 : 1);  // the window, and the sun mirrored off the cap
            exBit = 1;
            strip(sLight, EX_MIRROR, 0, beam::EXT_CAP_STEPS);
            exBit = 2;
            strip(sLight, EX_FOOT, 0, beam::EXT_STEPS);
            exBit = 4;
            strip(sLight, EX_WALL, 0, beam::EXT_STEPS);
            exBit = 8;
            if (mirrors) strip(sLight, EX_GLINT, beam::EXT_STEPS / 2, 1);
            exBit = 16;
            strip(sLight, EX_CUT, 0, 2 * beam::EXT_STEPS);
        }
    }

    // ---- the stage's own drawing. Every material that puts light or an outline on the floor draws it once per receiver
    // (recv_drop.glsl: the floor, each wall, three faces per block), see `on_stage` above. What the CPU still makes
    // as triangles (only with FLATCPU, OUTLINECPU, ...) and the blocks' own outlines go through recv_light.mat, which
    // does the same for a vertex buffer. All of it goes into the same two pictures as before (light, shade).
    utils::Entity stageLightE[2], stageShadeE[2];   // the CPU's triangles, light and outlines: [0] on the room's planes, [1] on the blocks' faces
    if (stageOn) {
        auto recv_tris = [&](VertexBuffer* vb, IndexBuffer* ib, float source, float cover, int first, int count) {
            MaterialInstance* mi = recvLightMat->createInstance();
            mi->setParameter("source", source);
            mi->setParameter("cover", cover);
            mi->setParameter("floorY", 0.f);
            mi->setParameter("recvFirst", (float)first);
            set_stage(mi, true);
            if (first == 0 && blocksOn) mi->setDepthCulling(true);
            utils::Entity e = em.create();
            RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}}).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
                .instances((size_t)count).material(0, mi).culling(false).castShadows(false).receiveShadows(false).build(*engine, e);
            return e;
        };
        const int faces = 3 * (int)blocks.size();   // (recv_light.mat: a block shows the sun three faces at most)
        stageLightE[0] = recv_tris(lightVB, lightIB, 0.f, 0.f, 0, roomPlanes);
        stageShadeE[0] = recv_tris(lightVB, lightIB, 1.f, 1.f, 0, roomPlanes);
        sLight->addEntity(stageLightE[0]);
        sShade->addEntity(stageShadeE[0]);
        if (blocksOn) {
            stageLightE[1] = recv_tris(lightVB, lightIB, 0.f, 0.f, roomPlanes, faces);
            stageShadeE[1] = recv_tris(lightVB, lightIB, 1.f, 1.f, roomPlanes, faces);
            sLight->addEntity(stageLightE[1]);
            sShade->addEntity(stageShadeE[1]);
        }
        // the softening pass: told the spread in pixels by whoever drew the light, and it hands light and shade on apart
        softenMi->setParameter("unit", 1.f);
        softenMi->setParameter("split", 1.f);
