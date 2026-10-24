// glass_scene.mm, piece 4 (cut there by size, as it was): read only in its place in glass_scene.mm.
        const int levels = std::min(16, std::max(1, (int)env_f("GBEAMLEVELS", 8.f)));
        const int wide = std::min(1024, std::max(8, (int)env_f("GBEAMW", 96.f)));
        const int NODE_W = 12, WALLS = 18, WALL_W = 14;
        auto table_view = [&](MaterialInstance* mi, const Target& t, int w, int h) {
            Scene* sc = engine->createScene();
            sc->addEntity(renderable(sqVB, sqIB, mi, false, false));
            View* v = engine->createView();
            v->setScene(sc);
            v->setRenderTarget(t.rt);
            v->setViewport({0, 0, (uint32_t)w, (uint32_t)h});
            v->setDithering(View::Dithering::NONE);
            v->setPostProcessingEnabled(false);
            v->setShadowingEnabled(false);
            gbeamViews.push_back(v);
        };
        for (FlatKind& fk : flatKinds) {
            const beam::Solid& loc = items[(size_t)fk.solids[0]].local;
            // the cells and their walls, per set of films in use (gbeam.glsl)
            auto* cellTexels = new std::vector<float>((size_t)WALLS * WALL_W * 256 * 4, 0.f);
            int mostWalls = 0, mostCorners = 0;
            for (int use = 0; use < 16; ++use) {
                const beam::SplitShape sh = beam::split_shape(loc, use);
                for (size_t c = 0; c < sh.cells.size() && c < 16; ++c) {
                    const beam::SplitCell& cell = sh.cells[c];
                    mostWalls = std::max(mostWalls, (int)cell.planes.size());
                    for (size_t w = 0; w < cell.planes.size() && w < (size_t)WALLS; ++w) {
                        if (cell.faces[w].n < 3) continue;
                        float* o = cellTexels->data() + (((size_t)use * 16 + c) * (size_t)(WALLS * WALL_W) + w * (size_t)WALL_W) * 4;
                        o[0] = (float)cell.planes[w].n.x; o[1] = (float)cell.planes[w].n.y; o[2] = (float)cell.planes[w].n.z; o[3] = (float)cell.planes[w].o;
                        const int cn = std::min(12, cell.faces[w].n);
                        mostCorners = std::max(mostCorners, cell.faces[w].n);
                        o[4] = cell.what[w] >= 0 ? (float)cell.what[w] : -1.f;
                        o[6] = (float)cn;
                        for (int k = 0; k < cn; ++k) {
                            o[8 + k * 4] = (float)cell.faces[w].p[(size_t)k].x; o[9 + k * 4] = (float)cell.faces[w].p[(size_t)k].y; o[10 + k * 4] = (float)cell.faces[w].p[(size_t)k].z;
                        }
                    }
                }
            }
            if (mostWalls > WALLS) return fail("GBEAMS: a cell has more than 18 walls");
            Texture* cellTex = Texture::Builder().width((uint32_t)(WALLS * WALL_W)).height(256).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
            cellTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(cellTexels->data(), cellTexels->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                         [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, cellTexels));
            float4 which[16] = {};
            for (size_t i = 0; i < fk.solids.size() && i < 64; ++i) which[i / 4][i % 4] = (float)fk.solids[i];
            const int rows = (int)fk.solids.size() * flatLines;
            Texture* before = cellTex;   // (a stand-in where a level has nothing before it)
            int beforeWide = 0;
            for (int lv = 0; lv <= levels; ++lv) {
                const int places = lv == 0 ? 16 * WALLS * 3 : wide;
                Target index{nullptr, nullptr};
                if (lv > 0) {
                    index = make_rt(places, rows, Texture::InternalFormat::RGBA32F);
                    MaterialInstance* imi = indexMat->createInstance();
                    imi->setParameter("nodes", before, nearest);
                    imi->setParameter("nodesWide", (float)beforeWide);
                    table_view(imi, index, places, rows);
                }
                const Target nodes = make_rt(places * NODE_W, rows, Texture::InternalFormat::RGBA32F);
                MaterialInstance* nmi = nodeMat->createInstance();
                nmi->setParameter("cells", cellTex, nearest);
                nmi->setParameter("prev", before, nearest);
                nmi->setParameter("index", lv > 0 ? index.color : cellTex, nearest);
                nmi->setParameter("pose", poseTex, nearest);
                nmi->setParameter("stepCount", (float)flatLines);
                nmi->setParameter("solids", which, 16);
                nmi->setParameter("solidsHere", (float)fk.solids.size());
                nmi->setParameter("level", (float)lv);
                nmi->setParameter("least", env_f("BEAMCUT", 2e-5f));
                nmi->setParameter("mostFaces", env_f("BEAMFACES", 8.f));
                nmi->setParameter("floorY", 0.f);
                routeMis.push_back(nmi);
                table_view(nmi, nodes, places * NODE_W, rows);
                // the patches of this level that have left the glass
                auto* v = new std::vector<float>();
                auto* ix = new std::vector<uint32_t>();
                for (int i = 0; i < places; ++i) {
                    const uint32_t base = (uint32_t)(v->size() / 3);
                    for (int k = 0; k < 10; ++k) v->insert(v->end(), {(float)i, (float)k, 0.f});
                    for (uint32_t k = 1; k + 1 < 10; ++k) ix->insert(ix->end(), {base, base + k, base + k + 1});
                }
                VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(v->size() / 3)).bufferCount(1)
                                       .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12).build(*engine);
                vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(v->data(), v->size() * sizeof(float)));
                IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)ix->size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
                ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(ix->data(), ix->size() * sizeof(uint32_t)));
                MaterialInstance* dmi = drawMat->createInstance();
                dmi->setParameter("nodes", nodes.color, nearest);
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
                before = nodes.color;
                beforeWide = places;
            }
            std::printf("  gbeams: %s, %d levels of %d nodes after the first, %d lines; most walls to a cell %d, most corners to a wall %d\n", fk.name.c_str(), levels, wide,
                        rows, mostWalls, mostCorners);
        }
    }
    if (routeTree) {
        Material* treeMat = mat("route_tree.filamat");
        Material* drawMat = mat("route_draw.filamat");
        if (!treeMat || !drawMat) return fail("route_tree.filamat / route_draw.filamat");
        static const int fans[] = {6, 4, 3, 2, 2};   // how many things an entry of the level before may go on to meet
        const int levels = std::min(5, std::max(1, (int)env_f("ROUTEDEPTH", 5.f)));
        for (FlatKind& fk : flatKinds) {
            const beam::Solid& loc = items[(size_t)fk.solids[0]].local;
            float4 planes[40] = {}, films[4] = {};
            int planeCount = 0;
            auto* cornerTexels = new std::vector<float>((size_t)8 * 40 * 4, 0.f);
            for (size_t f = 0; f < loc.planes.size() && planeCount < 40; ++f) {   // (the order of the table of faces)
                if (loc.faces[f].n < 3) continue;
                planes[planeCount] = float4{(float)loc.planes[f].n.x, (float)loc.planes[f].n.y, (float)loc.planes[f].n.z, (float)loc.planes[f].o};
                const int cn = std::min(8, loc.faces[f].n);
                for (int k = 0; k < cn; ++k) {
                    float* o = cornerTexels->data() + ((size_t)planeCount * 8 + (size_t)k) * 4;
                    o[0] = (float)loc.faces[f].p[(size_t)k].x; o[1] = (float)loc.faces[f].p[(size_t)k].y; o[2] = (float)loc.faces[f].p[(size_t)k].z;
                    o[3] = (float)cn;
                }
                ++planeCount;
            }
            const int filmCount = (int)std::min<size_t>(4, loc.films.size());
            for (int f = 0; f < filmCount; ++f) films[f] = float4{(float)loc.films[(size_t)f].n.x, (float)loc.films[(size_t)f].n.y, (float)loc.films[(size_t)f].n.z, (float)loc.films[(size_t)f].o};
            Texture* cornerTex = Texture::Builder().width(8).height(40).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
            cornerTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(cornerTexels->data(), cornerTexels->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                           [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, cornerTexels));
            float4 which[16] = {};
            for (size_t i = 0; i < fk.solids.size() && i < 64; ++i) which[i / 4][i % 4] = (float)fk.solids[i];
            const int rows = (int)fk.solids.size() * flatLines * planeCount;
            if (rows > 16384) return fail("ROUTES=2: too many solids of one shape for one table");
            auto common = [&](MaterialInstance* mi) {
                mi->setParameter("pose", poseTex, nearest);
                mi->setParameter("corners", cornerTex, nearest);
                mi->setParameter("planes", planes, 40);
                mi->setParameter("planeCount", (int32_t)planeCount);
                mi->setParameter("films", films, 4);
                mi->setParameter("filmCount", (int32_t)filmCount);
                mi->setParameter("stepCount", (float)flatLines);
                mi->setParameter("solids", which, 16);
                routeMis.push_back(mi);
            };
            int slots = 1;
            Texture* before = cornerTex;
            for (int lv = 1; lv <= levels; ++lv) {
                slots *= fans[lv - 1];
                RouteLevel rl{make_rt(slots * 2, rows, Texture::InternalFormat::RGBA32F), engine->createView(), slots, rows};
                MaterialInstance* tmi = treeMat->createInstance();
                common(tmi);
                tmi->setParameter("prev", before, nearest);
                tmi->setParameter("solidsHere", (float)fk.solids.size());
                tmi->setParameter("level", (float)lv);
                tmi->setParameter("fan", (float)fans[lv - 1]);
                tmi->setParameter("last", lv == levels ? 1.f : 0.f);
                Scene* sc = engine->createScene();
                sc->addEntity(renderable(sqVB, sqIB, tmi, false, false));
                rl.view->setScene(sc);
                rl.view->setRenderTarget(rl.table.rt);
                rl.view->setViewport({0, 0, (uint32_t)(slots * 2), (uint32_t)rows});
                rl.view->setDithering(View::Dithering::NONE);
                rl.view->setPostProcessingEnabled(false);
                rl.view->setShadowingEnabled(false);
                routeLevels.push_back(rl);
                before = rl.table.color;
                // its leaves: four corners per entry
                auto* v = new std::vector<float>();
                auto* ix = new std::vector<uint32_t>();
                for (int i = 0; i < slots; ++i) {
                    const uint32_t base = (uint32_t)(v->size() / 3);
                    for (int k = 0; k < 4; ++k) v->insert(v->end(), {(float)i, (float)(k & 1), (float)(k >> 1)});
                    ix->insert(ix->end(), {base, base + 1, base + 3, base, base + 3, base + 2});
                }
                VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(v->size() / 3)).bufferCount(1)
                                       .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12).build(*engine);
                vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(v->data(), v->size() * sizeof(float)));
                IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)ix->size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
                ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(ix->data(), ix->size() * sizeof(uint32_t)));
                MaterialInstance* dmi = drawMat->createInstance();
                common(dmi);
                dmi->setParameter("tree", rl.table.color, nearest);
                dmi->setParameter("floorY", 0.f);
                const utils::Entity e = em.create();
                RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
                    .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
                    .instances((size_t)rows)
                    .material(0, dmi).culling(false).castShadows(false).receiveShadows(false)
                    .layerMask(0xFF, std::getenv("NOLIGHT") ? 0x00 : 0x01).build(*engine, e);
                sLight->addEntity(e);
            }
            std::printf("  routes: %s, %d faces, %d films, %d levels, %d entries in the last, %d lines\n", fk.name.c_str(), planeCount, filmCount, levels, slots, rows);
        }
    } else if (flatRoutes) {
        Material* routeMat = mat("route_light.filamat");
        if (!routeMat) return fail("route_light.filamat");
        for (FlatKind& fk : flatKinds) {
            using beam::V3;
            const beam::Solid& loc = items[(size_t)fk.solids[0]].local;
            float4 planes[40] = {}, films[4] = {};
            int planeCount = 0;
            for (size_t f = 0; f < loc.planes.size() && planeCount < 40; ++f)   // (the order of the table of faces)
                if (loc.faces[f].n >= 3) planes[planeCount++] = float4{(float)loc.planes[f].n.x, (float)loc.planes[f].n.y, (float)loc.planes[f].n.z, (float)loc.planes[f].o};
            const int filmCount = (int)std::min<size_t>(4, loc.films.size());
            for (int f = 0; f < filmCount; ++f) films[f] = float4{(float)loc.films[(size_t)f].n.x, (float)loc.films[(size_t)f].n.y, (float)loc.films[(size_t)f].n.z, (float)loc.films[(size_t)f].o};
            // the corners of every face, for route_light.mat
            std::vector<int> tableOf(loc.planes.size(), -1);
            auto* cornerTexels = new std::vector<float>((size_t)8 * 40 * 4, 0.f);
            for (size_t f = 0, row = 0; f < loc.planes.size() && row < 40; ++f) {
                if (loc.faces[f].n < 3) continue;
                tableOf[f] = (int)row;
                const int cn = std::min(8, loc.faces[f].n);
                for (int k = 0; k < cn; ++k) {
                    float* o = cornerTexels->data() + (row * 8 + (size_t)k) * 4;
                    o[0] = (float)loc.faces[f].p[(size_t)k].x; o[1] = (float)loc.faces[f].p[(size_t)k].y; o[2] = (float)loc.faces[f].p[(size_t)k].z;
                    o[3] = (float)cn;
                }
                ++row;
            }
            Texture* cornerTex = Texture::Builder().width(8).height(40).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
            cornerTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(cornerTexels->data(), cornerTexels->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                           [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, cornerTexels));
            // per piece four corners (the material puts them round the light of the way that piece finds)
            auto* v = new std::vector<float>();
            auto* ix = new std::vector<uint32_t>();
            int firstOfFace = 0, lastFace = -2;
            for (int i = 0; i < fk.shape.pieces; ++i) {
                const float* tb = fk.shape.table.data() + (size_t)i * 20;
                const V3 nf{tb[4], tb[5], tb[6]}, c0{tb[8], tb[9], tb[10]};
                int face = -1;
                for (size_t f = 0; f < loc.planes.size(); ++f)
                    if (loc.faces[f].n >= 3 && beam::dot(loc.planes[f].n, nf) > 0.99999 && std::fabs(beam::dot(loc.planes[f].n, c0) - loc.planes[f].o) < 1e-5) face = (int)f;
                if (face != lastFace) { firstOfFace = i; lastFace = face; }
                if (face < 0) continue;
                const uint32_t base = (uint32_t)(v->size() / 5);
                for (int k = 0; k < 4; ++k) v->insert(v->end(), {(float)i, (float)(k & 1), (float)(k >> 1), (float)firstOfFace, (float)tableOf[(size_t)face]});
                ix->insert(ix->end(), {base, base + 1, base + 3, base, base + 3, base + 2});
            }
            VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(v->size() / 5)).bufferCount(1)
                                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                                   .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*engine);
            vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(v->data(), v->size() * sizeof(float)));
            IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)ix->size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
            ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(ix->data(), ix->size() * sizeof(uint32_t)));
            MaterialInstance* mi = routeMat->createInstance();
            mi->setParameter("walk", flatWalk.color, nearest);
            mi->setParameter("pieces", flatPieces, nearest);
            mi->setParameter("pose", poseTex, nearest);
            mi->setParameter("corners", cornerTex, nearest);
            mi->setParameter("loose", env_f("ROUTELOOSE", 0.f));
            mi->setParameter("shape", (float)(&fk - flatKinds.data()));
            mi->setParameter("stepCount", (float)flatLines);
            mi->setParameter("planes", planes, 40);
            mi->setParameter("planeCount", (int32_t)planeCount);
            mi->setParameter("films", films, 4);
            mi->setParameter("filmCount", (int32_t)filmCount);
            mi->setParameter("floorY", 0.f);
            float4 which[16] = {};
            for (size_t i = 0; i < fk.solids.size() && i < 64; ++i) which[i / 4][i % 4] = (float)fk.solids[i];
            mi->setParameter("solids", which, 16);
            const utils::Entity e = em.create();
            RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {100.f, 100.f, 100.f}})
                .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
                .instances(fk.solids.size() * (size_t)flatLines)
                .material(0, mi).culling(false).castShadows(false).receiveShadows(false)
                .layerMask(0xFF, std::getenv("NOLIGHT") ? 0x00 : 0x01).build(*engine, e);
            sLight->addEntity(e);
            routeMis.push_back(mi);
            std::printf("  routes: %s, %d pieces to find the ways, %zu corners in the mesh\n", fk.name.c_str(), fk.shape.pieces, v->size() / 5);
        }
    }
    static std::vector<std::vector<float>> mesh_v(items.size());
    static std::vector<std::vector<uint32_t>> mesh_i(items.size());
    // ---- VBODY=1 (experiment): the four-sided faces of flat-faced solids are not followed pixel by pixel. A table pass
    // (glass_nodes.mat) works out, at 17 x 17 points per face, where the sight line through the glass ends up; the faces are
    // drawn as a net of cells from that (glass_vert.mat), and only the cells the table cannot vouch for are followed per pixel.
    static const bool vbodyOn = std::getenv("VBODY") != nullptr && !stageOn;
    Material* glassVertMat = vbodyOn ? mat("glass_vert.filamat") : nullptr;
    Material* glassNodesMat = vbodyOn ? mat("glass_nodes.filamat") : nullptr;
    Material* glassCellsMat = vbodyOn ? mat("glass_cells.filamat") : nullptr;
    if (vbodyOn && (!glassVertMat || !glassNodesMat || !glassCellsMat)) return fail("VBODY materials");
    static std::vector<float> faceTable((size_t)24 * SCENE_H * 4, 0.f);   // per solid (line), per face: a corner (+1: in use), its two neighbours, the normal
    Texture* faceTex = Texture::Builder().width(24).height(SCENE_H).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
    Target nodesRT = make_rt(289, std::max(1, (int)items.size()) * 6, Texture::InternalFormat::RGBA32F);
    Target cellsRT = make_rt(80, std::max(1, (int)items.size()) * 6, Texture::InternalFormat::RGBA16F);   // which cells pass (glass_cells.mat)
    Scene* sCells = engine->createScene();
    MaterialInstance* nodesMi = nullptr;
    Scene* sNodes = engine->createScene();
    static std::vector<float> gridSoup;   // the net, the same for every solid: (face, level, cell), which corner
    if (vbodyOn) {
        for (int face = 0; face < 6; ++face)
            for (int level = 0; level < 7; ++level) {
                const int per = level == 0 ? 4 : 8;
                for (int cell = 0; cell < per * per; ++cell)
                    for (const int* c : {(const int[]){0, 0}, (const int[]){1, 0}, (const int[]){1, 1}, (const int[]){0, 1}})   // four corners a cell
                        gridSoup.insert(gridSoup.end(), {(float)face, (float)level, (float)cell, (float)c[0], (float)c[1]});
            }
    }
    for (size_t k = 0; k < items.size(); ++k) {
        Item& it = items[k];
        const beam::Solid& unit = it.local;
        float reach = 0;
        if (unit.kind != beam::Solid::FLAT && unit.kind != beam::Solid::EXTRUDE) {
            // a round solid: a fine enough mesh of its surface (the material works out the true shape per pixel)
            const float r = (float)unit.r, h = (float)unit.h, pi = 3.14159265f;
            auto quad = [&](float3 a, float3 b, float3 c, float3 d, float3 N) {  // a, b, c, d counter-clockwise seen from outside
                const uint32_t base = (uint32_t)(mesh_v[k].size() / 7);
                float3 T = normalize(b - a + float3{1e-6f, 2e-6f, 3e-6f});
                for (float3 p : {a, b, c, d}) push_vert(mesh_v[k], p, T, N);
                mesh_i[k].insert(mesh_i[k].end(), {base, base + 1, base + 2, base, base + 2, base + 3});
            };
            const int ns = 64;
            if (unit.kind == beam::Solid::BALL) {
                const int nr = 32;
                auto at = [&](int i, int j) {
                    const float th = pi * i / nr, ph = 2 * pi * j / ns;
                    return float3{r * std::sin(th) * std::cos(ph), r * std::cos(th), r * std::sin(th) * std::sin(ph)};
                };
                for (int i = 0; i < nr; ++i)
                    for (int j = 0; j < ns; ++j) quad(at(i, j), at(i, j + 1), at(i + 1, j + 1), at(i + 1, j), normalize(at(i, j) + at(i + 1, j + 1)));
                reach = r;
            } else if (unit.kind == beam::Solid::CONE) {
                auto ring = [&](int j, float y, float rad) {
                    const float ph = 2 * pi * j / ns;
                    return float3{rad * std::cos(ph), y, rad * std::sin(ph)};
                };
                const float k = r / h;
                for (int j = 0; j < ns; ++j) {
                    quad(float3{0, 0.5f * h, 0}, float3{0, 0.5f * h, 0}, ring(j + 1, -0.5f * h, r), ring(j, -0.5f * h, r), normalize(ring(j, k, 1) + ring(j + 1, k, 1)));
                    quad(float3{0, -0.5f * h, 0}, ring(j, -0.5f * h, r), ring(j + 1, -0.5f * h, r), float3{0, -0.5f * h, 0}, float3{0, -1, 0});
                }
                reach = std::sqrt(r * r + 0.25f * h * h);
            } else {
                auto ring = [&](int j, float y, float rad) {
                    const float ph = 2 * pi * j / ns;
                    return float3{rad * std::cos(ph), y, rad * std::sin(ph)};
                };
                for (int j = 0; j < ns; ++j) {
                    quad(ring(j, h, r), ring(j + 1, h, r), ring(j + 1, -h, r), ring(j, -h, r), normalize(ring(j, 0, 1) + ring(j + 1, 0, 1)));
                    quad(float3{0, h, 0}, ring(j + 1, h, r), ring(j, h, r), float3{0, h, 0}, float3{0, 1, 0});
                    quad(float3{0, -h, 0}, ring(j, -h, r), ring(j + 1, -h, r), float3{0, -h, 0}, float3{0, -1, 0});
                }
                reach = std::sqrt(r * r + h * h);
            }
        }
        if (unit.kind == beam::Solid::EXTRUDE) {
            // caps, walls and chamfers as flat triangles (the material puts the walls of arcs on their true curve)
            std::vector<beam::V3> pos, nrm;
            beam::extrude_mesh(*unit.shape, pos, nrm);
            for (size_t i = 0; i + 2 < pos.size(); i += 3) {
                const float3 a{(float)pos[i].x, (float)pos[i].y, (float)pos[i].z}, b{(float)pos[i + 1].x, (float)pos[i + 1].y, (float)pos[i + 1].z},
                    c{(float)pos[i + 2].x, (float)pos[i + 2].y, (float)pos[i + 2].z}, N{(float)nrm[i].x, (float)nrm[i].y, (float)nrm[i].z};
                const uint32_t base = (uint32_t)(mesh_v[k].size() / 7);
                const float3 T = normalize(b - a);
                for (float3 p : {a, b, c}) {
                    push_vert(mesh_v[k], p, T, N);
                    reach = std::max(reach, length(p));
                }
                mesh_i[k].insert(mesh_i[k].end(), {base, base + 1, base + 2});
            }
        }
        for (size_t f = 0; f < unit.faces.size(); ++f) {
            const beam::Poly& P = unit.faces[f];
            if (P.n < 3) continue;
            float3 N{(float)unit.planes[f].n.x, (float)unit.planes[f].n.y, (float)unit.planes[f].n.z};
            float3 T = normalize(float3{(float)(P.p[1].x - P.p[0].x), (float)(P.p[1].y - P.p[0].y), (float)(P.p[1].z - P.p[0].z)});
            // outward winding for back-face culling (the glass is single sided)
            const bool flip = beam::dot(beam::cross(P.p[1] - P.p[0], P.p[2] - P.p[0]), unit.planes[f].n) < 0;
            uint32_t base = (uint32_t)(mesh_v[k].size() / 7);
            for (int i = 0; i < P.n; ++i) {
                const float3 p{(float)P.p[(size_t)i].x, (float)P.p[(size_t)i].y, (float)P.p[(size_t)i].z};
                reach = std::max(reach, length(p));
                push_vert(mesh_v[k], p, T, N);
            }
            for (int i = 1; i + 1 < P.n; ++i) {
                if (flip) mesh_i[k].insert(mesh_i[k].end(), {base, base + (uint32_t)i + 1, base + (uint32_t)i});
                else mesh_i[k].insert(mesh_i[k].end(), {base, base + (uint32_t)i, base + (uint32_t)i + 1});
            }
        }
        auto [vb, ib] = lit_buffers(mesh_v[k], mesh_i[k]);
        it.glass = glassMat->createInstance();
        if (vbodyOn && unit.kind == beam::Solid::FLAT) {
            // its four-sided faces (up to six of them) go into the table the cheap layer is drawn from
            int used = 0;
            for (size_t f = 0; f < unit.faces.size(); ++f) {
                const beam::Poly& P = unit.faces[f];
                if (P.n != 4 || used >= 6) continue;
                if (beam::length(P.p[0] + P.p[2] - P.p[1] - P.p[3]) > 1e-6 * (beam::length(P.p[2] - P.p[0]) + 1e-30)) continue;
                const beam::V3 nf = unit.planes[f].n;
                beam::V3 c0 = P.p[0], c1 = P.p[1], c2 = P.p[3];
                if (beam::dot(beam::cross(c1 - c0, c2 - c0), nf) < 0) std::swap(c1, c2);   // counter-clockwise seen from outside
                float* o = faceTable.data() + (k * 24 + (size_t)used * 4) * 4;
                const float e[16] = {(float)c0.x, (float)c0.y, (float)c0.z, 1.f, (float)c1.x, (float)c1.y, (float)c1.z, 0.f,
                                     (float)c2.x, (float)c2.y, (float)c2.z, 0.f, (float)nf.x, (float)nf.y, (float)nf.z, 0.f};
                std::copy(e, e + 16, o);
                ++used;
            }
            it.vbody = used > 0;
        }
        // STDGLASS=1, measuring only: draw the body with Filament's own screen-space refraction (std_glass.mat) instead
        static Material* stdGlassMat = std::getenv("STDGLASS") ? mat("std_glass.filamat") : nullptr;
        MaterialInstance* drawn = it.glass;
        if (it.vbody && (std::getenv("VSHOW") || std::getenv("VREDOLD"))) {   // (VREDOLD: timing only, the old way behind costs nothing)
            static Material* redMat = mat("flat_red.filamat");
            if (redMat) drawn = redMat->createInstance();
        }
        if (stdGlassMat) {
            drawn = stdGlassMat->createInstance();
            drawn->setParameter("ior", nd);
            drawn->setParameter("thickness", (float)it.size * env_f("STDTHICK", 1.f));
            drawn->setParameter("dispersion", env_f("STDDISP", 20.f / 64.17f * disp));
        }
        it.glass->setParameter("ior", nd);
        it.glass->setParameter("abbe", 64.17f / disp);
        std::vector<float4> planes;
        for (size_t f = 0; f < unit.planes.size(); ++f)
            if (unit.faces[f].n >= 3) planes.push_back(float4{(float)unit.planes[f].n.x, (float)unit.planes[f].n.y, (float)unit.planes[f].n.z, (float)unit.planes[f].o});
        if (planes.size() > 40) return fail("a solid has more than 40 faces");
        const int32_t count = (int32_t)planes.size();
        planes.resize(40, float4{0, 1, 0, 1e6f});
        it.glass->setParameter("planes", planes.data(), 40);
        it.glass->setParameter("planeCount", count);
        {
            float4 films[4] = {};
            const int nf = it.local.kind == beam::Solid::FLAT ? (int)std::min<size_t>(4, it.local.films.size()) : 0;
            for (int f = 0; f < nf; ++f) films[f] = float4{(float)it.local.films[(size_t)f].n.x, (float)it.local.films[(size_t)f].n.y, (float)it.local.films[(size_t)f].n.z, (float)it.local.films[(size_t)f].o};
            it.glass->setParameter("films", films, 4);
            it.glass->setParameter("filmNm", filmNm, 4);
            it.glass->setParameter("filmCount", (int32_t)nf);
        }
        it.glass->setParameter("kind", (int32_t)unit.kind);
        it.glass->setParameter("dims", float4{(float)unit.r, (float)unit.h, 0, 0});
        it.glass->setParameter("scene", sceneTex, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST));
        it.glass->setParameter("shapes", exShapeTex, exNearest);
        it.glass->setParameter("solidCount", (int32_t)(std::getenv("ALONE") ? 0 : items.size()));
        it.glass->setParameter("self", (int32_t)k);
        it.glass->setParameter("nearCone", env_f("NEARCONE", 0.8f));  // which solids are asked about being seen through this one (glass_solid.mat, `near`; 0: all)
        it.glass->setParameter("spread", env_f("SPREAD", 2.f));  // colours part only at the last face; this makes up for the others
        it.glass->setParameter("floorY", 0.f);
        it.glass->setParameter("absorption", float3{0.9f, 0.35f, 0.5f} * env_f("ABSORB", 0.15f));
        it.glass->setParameter("backdrop", backdrop.color, lin);
        if (stageOn) {  // what else a sight line through the glass can end on
            it.glass->setParameter("room", roomT, 6);
            it.glass->setParameter("roomCount", (float)stage.room.size());
            it.glass->setParameter("blocks", blocksT, 16);
            it.glass->setParameter("blockCount", (float)blocks.size());
        }
        // drawn by its own material and left out of the backdrop picture (layer 1)
        it.entity = em.create();
        RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {reach, reach, reach}}).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, drawn).culling(false).castShadows(false).receiveShadows(false).layerMask(0xFF, 0x02).build(*engine, it.entity);
        tm.create(it.entity);
        if (!std::getenv("NOGLASS")) scene->addEntity(it.entity);
