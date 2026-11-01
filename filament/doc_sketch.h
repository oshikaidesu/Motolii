// doc_sketch.h: part of doc.h. What a p5.js sketch draws in WEBGL, as p5 itself has it each frame (p5_mesh.h's __mesh_at:
// its own shapes, its own vertices, what its camera sees), stood in the world as one mesh, made again each frame as a
// sheet's is (kit_stage.h's reshape). Shapes come once and are laid out here (a thousand spheres are too many for a script
// to turn each frame), so a sketch has no limit of copies, and any shape it can draw is the shape it drew. A line is a thin
// band that faces the eye, as p5's line shader makes it. What it draws goes on from frame to frame until it is cleared.
static const SketchShape* doc_sketch_shape(std::vector<SketchShape>& shapes, Script& run, size_t i) {
    while (shapes.size() <= i) {
        const std::vector<uint8_t> raw = run.bytes("__mesh_shape(" + std::to_string(shapes.size()) + ")");
        SketchShape one;
        if (raw.size() >= 3 * sizeof(float)) {
            const float* f = (const float*)raw.data();
            const size_t points = (size_t)f[0], faces = (size_t)f[1], edges = (size_t)f[2];
            if (raw.size() >= (3 + points * 8 + faces * 3 + edges * 2) * sizeof(float)) {
                const float* p = f + 3, *n = p + points * 3, *u = n + points * 3, *a = u + points * 2, *e = a + faces * 3;
                one.pos.assign((const float3*)p, (const float3*)n), one.nrm.assign((const float3*)n, (const float3*)u), one.uv.assign((const float2*)u, (const float2*)a);
                for (size_t k = 0; k < faces * 3; ++k) one.idx.push_back((uint32_t)a[k]);
                for (size_t k = 0; k < edges * 2; ++k) one.edges.push_back((uint32_t)e[k]);
            }
        }
        shapes.push_back(std::move(one));
    }
    return &shapes[i];
}
// A shader of a sketch's as a Filament material: what p5_mesh.h's __shader_text made of it, compiled by Filament's own matc as the kit's
// are (dawn_probe/kit_wgpu.sh), a third of a second for a real one, kept by what it says. What cannot be made says why and paints nothing.
static SketchPaint doc_sketch_paint(Stage& s, const std::string& text, const std::string& names) {
    SketchPaint paint;
    std::istringstream lines(names);
    for (SketchTold one; lines >> one.name >> one.type >> one.count >> one.size; ) paint.told.push_back(one), paint.size += one.count * one.size;
    const std::filesystem::path made = std::filesystem::temp_directory_path() / "motolii_paints";
    std::filesystem::create_directories(made);
    const std::string stem = (made / std::to_string(std::hash<std::string>{}(text))).string();
    if (!std::filesystem::exists(stem + ".filamat")) {
        std::ofstream(stem + ".mat") << text;
        const std::string matc = (doc_folders("third_party/filament_src/out/wgpu_install/bin").front() / "matc").string();
        if (std::system(("'" + matc + "' -a webgpu -a metal -p desktop -o '" + stem + ".filamat' '" + stem + ".mat' > '" + stem + ".log' 2>&1").c_str()) != 0) {
            std::ifstream log(stem + ".log");
            std::fprintf(stderr, "a sketch's shader did not compile: %s\n", std::string(std::istreambuf_iterator<char>(log), {}).c_str());
            return paint;
        }
    }
    const std::vector<uint8_t> bytes = load_file((stem + ".filamat").c_str());
    if (!bytes.empty()) paint.material = Material::Builder().package(bytes.data(), bytes.size()).build(*s.engine);
    return paint;
}
static void doc_sketch_told(MaterialInstance* look, const SketchTold& one, const float* v) {
    const char* name = one.name.c_str();
    if (one.type == "int" || one.type == "bool") {
        std::vector<int32_t> whole(v, v + one.count);
        if (one.type == "bool") look->setParameter(name, (bool)whole[0]);
        else look->setParameter(name, whole.data(), (size_t)one.count);
    } else if (one.type == "float") look->setParameter(name, v, (size_t)one.count);
    else if (one.type == "vec2") look->setParameter(name, (const float2*)v, (size_t)one.count);
    else if (one.type == "vec3") look->setParameter(name, (const float3*)v, (size_t)one.count);
    else if (one.type == "vec4") look->setParameter(name, (const float4*)v, (size_t)one.count);
    else if (one.type == "mat3") look->setParameter(name, mat3f{float3{v[0], v[1], v[2]}, float3{v[3], v[4], v[5]}, float3{v[6], v[7], v[8]}});
    else if (one.type == "mat4") { mat4f m; std::memcpy(&m, v, sizeof m); look->setParameter(name, m); }
}
static void doc_sketch_mesh(Doc& d, int thing, Doc::Shown& t, double now) {
    // (p5_mesh.h's __mesh_at: how many of each, its unit, its size, what it clears with, how many pictures, how long the table of what
    //  it lights is; then each shape it stands: which, its 16, fill, flat -1 none, its line, stroke, weight, picture; each corner of a
    //  triangle: x y z, normal, colour, flat, line, where it reads its picture, which; each line: two ends, colour, weight, line; then
    //  sketch.mat's table; then what each way of being told a shader says. A sketch past MOST points shows as many as fit, past PICTURES
    //  pictures the rest as white, past WAYS ways the rest flat)
    constexpr size_t HEAD = 15, RET = 29, TRI = 15, SEG = 12, TOLD = 720, MOST = 3000000, PICTURES = 6, WAYS = 4096;
    Stage& s = *d.stage;
    DocScript* sc = doc_script_of(d, doc_literal_word(d, thing, "script"));
    if (!sc || !sc->run || !sc->run->ok()) return;
    const double gone = doc_literal(d, thing, "begin", 0, 0), begins = doc_literal(d, thing, "start", 0, 0) - gone;
    const int frame = std::max(0, (int)std::lround((now - begins) * 60.0));
    const auto shown = doc_shown_at(d, thing, now);
    const math::double4 from = inverse(shown) * math::double4{s.camera->getPosition(), 1.0};
    const float3 eye = float3{(float)(from.x / from.w), (float)(from.y / from.w), (float)(from.z / from.w)};
    t.look->setParameter("eye", eye), t.look->setParameter("fromSketch", math::mat3f(shown.upperLeft()));   // (what it lights itself it lights where it stands now)
    std::map<int, float> strong;
    for (size_t i = 0; i < sc->knobs.size(); ++i)
        if (const std::string& k = sc->knobs[i].name; k.rfind("line_", 0) == 0) strong[std::atoi(k.c_str() + 5)] = (float)doc_number(d, thing, sc->names[i].c_str(), now, sc->knobs[i].usual);
    if (t.sketchFrame == frame && t.sketchMade == doc_scripts_made(d) && length(t.sketchEye - eye) < 1e-4f && t.sketchStrong == strong) return;
    if (t.sketchMade != doc_scripts_made(d)) t.sketchShapes.clear(), t.sketchPics.clear(), t.sketchPaints.clear();
    t.sketchFrame = frame, t.sketchMade = doc_scripts_made(d), t.sketchEye = eye, t.sketchStrong = strong;
    const std::vector<uint8_t> raw = sc->run->bytes("__mesh_at(" + std::to_string(frame) + ")");
    if (raw.size() < HEAD * sizeof(float)) return;
    const float* f = (const float*)raw.data();
    const size_t draws = (size_t)f[0], tris = (size_t)f[1], segs = (size_t)f[2];
    const float U = f[3], wide = f[4], high = f[5];
    const size_t told = (size_t)f[11], ways = (size_t)f[12], said = (size_t)f[13], shaders = (size_t)f[14];
    if (raw.size() < (HEAD + draws * RET + tris * TRI + segs * SEG + told * 4 + said) * sizeof(float)) return;
    const float* back = f + 6, *all = f + HEAD, *table = all + draws * RET + tris * TRI + segs * SEG, *way = table + told * 4;
    if (told) t.look->setParameter("told", (const float4*)table, std::min(told, TOLD));
    for (size_t i = t.sketchPaints.size(); i < shaders; ++i) t.sketchPaints.push_back(doc_sketch_paint(s, sc->run->said("__mesh_shader(" + std::to_string(i) + ")"), sc->run->said("__mesh_shader_names(" + std::to_string(i) + ")")));
    std::vector<MaterialInstance*> worn(std::min(ways, WAYS), nullptr);
    std::map<size_t, size_t> used;
    for (size_t i = 0; i < ways; ++i) {   // (each way: which shader, how its place lands, what it is told)
        SketchPaint& paint = t.sketchPaints[(size_t)way[0]];
        if (i < WAYS && paint.material) {
            size_t& n = used[(size_t)way[0]];
            if (paint.looks.size() <= n) {
                paint.looks.push_back(paint.material->createInstance());
                for (const SketchTold& one : paint.told) if (one.type == "sampler2D") paint.looks.back()->setParameter(one.name.c_str(), white_point(s), TextureSampler());
            }
            worn[i] = paint.looks[n++];
            worn[i]->setParameter("s_scale", float4{way[1], way[2], way[3], way[4]});
            const float* v = way + 5;
            for (const SketchTold& one : paint.told) doc_sketch_told(worn[i], one, v), v += one.count * one.size;
        }
        way += 5 + paint.size;
    }
    for (size_t i = t.sketchPics.size(); i < std::min((size_t)f[10], PICTURES); ++i) {   // (each picture it draws with, its wraps as p5's textureWrap: clamp, repeat, mirror)
        const std::vector<uint8_t> said = sc->run->bytes("__mesh_pic(" + std::to_string(i) + ")");
        if (said.size() < 4 * sizeof(float)) break;
        const float* p = (const float*)said.data();
        Drawn* pic = sketch_texture(s, sc->run.get(), (doc_folders("third_party/filament_src/assets/fonts").front() / "Roboto-Medium.ttf").string(), (int)i, (int)p[0], (int)p[1]);
        t.sketchPics.push_back(pic);
        if (!pic) continue;
        constexpr TextureSampler::WrapMode wraps[] = {TextureSampler::WrapMode::CLAMP_TO_EDGE, TextureSampler::WrapMode::REPEAT, TextureSampler::WrapMode::MIRRORED_REPEAT};
        TextureSampler read(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
        read.setWrapModeS(wraps[(int)p[2]]), read.setWrapModeT(wraps[(int)p[3]]);
        t.look->setParameter(("picture" + std::to_string(i)).c_str(), pic->texture, read);
    }
    for (Drawn* pic : t.sketchPics) if (pic) picture_at(s, pic, (float)frame / 60.f);
    Mesh& m = *t.shape;
    m.pos.clear(), m.nrm.clear(), m.uv.clear(), m.uv1.clear(), m.color.clear(), m.idx.clear();
    std::vector<std::vector<uint32_t>> parts(worn.size() + 1);   // (0: sketch.mat's. Each way a part of its own, so that each wears what it was told)
    const auto part = [&](float flatly) -> std::vector<uint32_t>& {
        const size_t i = flatly >= 1000.f ? (size_t)flatly - 1000 : worn.size();
        return parts[i < worn.size() && worn[i] ? i + 1 : 0];
    };
    const auto power = [&](int line) { const auto k = strong.find(line); return k == strong.end() ? 1.f : k->second; };
    const auto point = [&](float3 p, float3 n, float4 c, float flatly, float2 at = {}, float pic = 0.f) { m.pos.push_back(p), m.nrm.push_back(n), m.uv.push_back(float2{flatly, pic}), m.uv1.push_back(at), m.color.push_back(c); };
    const auto band = [&](float3 a, float3 b, float4 c, float w) {   // (a line, two triangles, turned to the eye; a point a little square)
        if (m.pos.size() + 4 > MOST) return;
        const float3 way = b - a, look = eye - a;
        float3 side = cross(way, look);
        if (length(way) < 1e-6f) side = cross(float3{0.f, 1.f, 0.f}, look);
        side = side * (0.5f * w / std::max(length(side), 1e-6f));
        const float3 along = length(way) < 1e-6f ? normalize(cross(look, side)) * (0.5f * w) : float3{0.f};
        const uint32_t o = (uint32_t)m.pos.size();
        const float3 n = normalize(look);
        point(a - side - along, n, c, 1.f), point(a + side - along, n, c, 1.f), point(b + side + along, n, c, 1.f), point(b - side + along, n, c, 1.f);
        parts[0].insert(parts[0].end(), {o, o + 1, o + 2, o, o + 2, o + 3});
    };
    if (back[3] > 0.f) {   // (what it clears with: a wall near the end of what the world's eye sees, 6 back and 60 on from it (doc_moment.h), as wide from that eye as the sketch)
        const float z = -40.f, grow = (6.f - z) / 12.f, x = grow * U * wide, y = grow * U * high;
        const float4 c{back[0], back[1], back[2], back[3]};
        const uint32_t o = (uint32_t)m.pos.size();
        const float3 n{0.f, 0.f, 1.f};
        point(float3{-x, -y, z}, n, c, 1.f), point(float3{x, -y, z}, n, c, 1.f), point(float3{x, y, z}, n, c, 1.f), point(float3{-x, y, z}, n, c, 1.f);
        parts[0].insert(parts[0].end(), {o, o + 1, o + 2, o, o + 2, o + 3});
    }
    const float3 flip{U, -U, U};   // (p5's y goes down, the world's up: its triangles turned too, so a face stays the side it faces)
    for (size_t i = 0; i < draws; ++i) {
        const float* r = all + i * RET;
        const SketchShape* shape = doc_sketch_shape(t.sketchShapes, *sc->run, (size_t)r[0]);
        const float k = power((int)r[22]);
        if (k < 1e-4f) continue;
        mat4f e;
        std::memcpy(&e, r + 1, 16 * sizeof(float));
        e = e * mat4f::scaling(float3{k});
        const mat3f turn = transpose(inverse(e.upperLeft()));
        const auto seen = [&](float3 p) { const float4 q = e * float4{p, 1.f}; return q.xyz / std::max(q.w, 0.1f) * flip; };   // (as __w: where p5_mesh.h's __view carried it)
        if (r[21] >= 0.f && m.pos.size() + shape->pos.size() <= MOST) {
            const uint32_t o = (uint32_t)m.pos.size();
            std::vector<uint32_t>& into = part(r[21]);
            const float4 c{r[17], r[18], r[19], r[20]};
            for (size_t v = 0; v < shape->pos.size(); ++v) {
                const float3 n = v < shape->nrm.size() ? turn * shape->nrm[v] : float3{0.f, 0.f, 1.f};
                point(seen(shape->pos[v]), length(n) > 1e-6f ? normalize(n * float3{1.f, -1.f, 1.f}) : float3{0.f, 0.f, 1.f}, c, r[21] >= 1000.f ? 1.f : r[21], v < shape->uv.size() ? shape->uv[v] : float2{}, r[28]);
            }
            for (size_t v = 0; v + 2 < shape->idx.size(); v += 3) into.insert(into.end(), {o + shape->idx[v], o + shape->idx[v + 2], o + shape->idx[v + 1]});
        }
        if (r[27] > 0.f)
            for (size_t v = 0; v + 1 < shape->edges.size(); v += 2) band(seen(shape->pos[shape->edges[v]]), seen(shape->pos[shape->edges[v + 1]]), float4{r[23], r[24], r[25], r[26]}, r[27] * U);
    }
    for (size_t i = 0; i + 2 < tris && m.pos.size() + 3 <= MOST; i += 3) {
        const uint32_t o = (uint32_t)m.pos.size();
        for (const size_t v : {i, i + 2, i + 1}) {
            const float* r = all + draws * RET + v * TRI;
            const float3 n = normalize(float3{r[3], -r[4], r[5]} + float3{0.f, 0.f, 1e-6f});
            point(float3{r[0], r[1], r[2]} * flip, n, float4{r[6], r[7], r[8], r[9] * power((int)r[11])}, r[10] >= 1000.f ? 1.f : r[10], float2{r[12], r[13]}, r[14]);
        }
        std::vector<uint32_t>& into = part(all[draws * RET + i * TRI + 10]);
        into.insert(into.end(), {o, o + 1, o + 2});
    }
    for (size_t i = 0; i < segs; ++i) {
        const float* r = all + draws * RET + tris * TRI + i * SEG;
        band(float3{r[0], r[1], r[2]} * flip, float3{r[3], r[4], r[5]} * flip, float4{r[6], r[7], r[8], r[9] * power((int)r[11])}, r[10] * U);
    }
    if (m.pos.empty()) point(float3{0.f}, float3{0.f, 0.f, 1.f}, float4{0.f}, 1.f), point(float3{0.f}, float3{0.f, 0.f, 1.f}, float4{0.f}, 1.f), point(float3{0.f}, float3{0.f, 0.f, 1.f}, float4{0.f}, 1.f), parts[0] = {0, 1, 2};
    std::vector<uint32_t> ends;
    for (const std::vector<uint32_t>& one : parts) m.idx.insert(m.idx.end(), one.begin(), one.end()), ends.push_back((uint32_t)m.idx.size());
    if (ends.size() > t.sketchParts) more_parts(s, &m, t.sketchParts = ends.size() + ends.size() / 2, t.look);
    reshape(s, &m, ends);
    auto& faces = s.engine->getRenderableManager();
    for (const Stage::Made& made : s.made) {
        if (made.of != &m) continue;
        const auto at = faces.getInstance(made.thing);
        for (size_t i = 0; i < worn.size(); ++i) if (worn[i]) faces.setMaterialInstanceAt(at, i + 1, worn[i]);
        for (size_t k = ends.size(); k < t.sketchParts; ++k) faces.setGeometryAt(at, k, RenderableManager::PrimitiveType::TRIANGLES, m.vb, m.ib, 0, 0);
    }
}
