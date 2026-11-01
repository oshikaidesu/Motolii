// doc_sketch.h: part of doc.h. What a p5.js sketch draws in WEBGL, as p5 itself has it each frame (p5_stand.h's __mesh_at:
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
            if (raw.size() >= (3 + points * 6 + faces * 3 + edges * 2) * sizeof(float)) {
                const float* p = f + 3, *n = p + points * 3, *a = n + points * 3, *e = a + faces * 3;
                one.pos.assign((const float3*)p, (const float3*)n), one.nrm.assign((const float3*)n, (const float3*)a);
                for (size_t k = 0; k < faces * 3; ++k) one.idx.push_back((uint32_t)a[k]);
                for (size_t k = 0; k < edges * 2; ++k) one.edges.push_back((uint32_t)e[k]);
            }
        }
        shapes.push_back(std::move(one));
    }
    return &shapes[i];
}
static void doc_sketch_mesh(Doc& d, int thing, Doc::Shown& t, double now) {
    // (p5_stand.h's __mesh_at: how many of each, its unit, its size, what it clears with; then each shape it stands: which, its 16,
    //  fill, flat -1 none, its line, stroke, weight; each corner of a triangle: x y z, normal, colour, flat, line; each line: two
    //  ends, colour, weight, line. A sketch past MOST points shows as many as fit)
    constexpr size_t HEAD = 10, RET = 28, TRI = 12, SEG = 12, MOST = 3000000;
    Stage& s = *d.stage;
    DocScript* sc = doc_script_of(d, doc_literal_word(d, thing, "script"));
    if (!sc || !sc->run || !sc->run->ok()) return;
    const double gone = doc_literal(d, thing, "begin", 0, 0), begins = doc_literal(d, thing, "start", 0, 0) - gone;
    const int frame = std::max(0, (int)std::lround((now - begins) * 60.0));
    const math::double4 from = inverse(doc_shown_at(d, thing, now)) * math::double4{s.camera->getPosition(), 1.0};
    const float3 eye = float3{(float)(from.x / from.w), (float)(from.y / from.w), (float)(from.z / from.w)};
    std::map<int, float> strong;
    for (size_t i = 0; i < sc->knobs.size(); ++i)
        if (const std::string& k = sc->knobs[i].name; k.rfind("line_", 0) == 0) strong[std::atoi(k.c_str() + 5)] = (float)doc_number(d, thing, sc->names[i].c_str(), now, sc->knobs[i].usual);
    if (t.sketchFrame == frame && t.sketchMade == doc_scripts_made(d) && length(t.sketchEye - eye) < 1e-4f && t.sketchStrong == strong) return;
    if (t.sketchMade != doc_scripts_made(d)) t.sketchShapes.clear();
    t.sketchFrame = frame, t.sketchMade = doc_scripts_made(d), t.sketchEye = eye, t.sketchStrong = strong;
    const std::vector<uint8_t> raw = sc->run->bytes("__mesh_at(" + std::to_string(frame) + ")");
    if (raw.size() < HEAD * sizeof(float)) return;
    const float* f = (const float*)raw.data();
    const size_t draws = (size_t)f[0], tris = (size_t)f[1], segs = (size_t)f[2];
    const float U = f[3], wide = f[4], high = f[5];
    if (raw.size() < (HEAD + draws * RET + tris * TRI + segs * SEG) * sizeof(float)) return;
    const float* back = f + 6, *all = f + HEAD;
    Mesh& m = *t.shape;
    m.pos.clear(), m.nrm.clear(), m.uv.clear(), m.color.clear(), m.idx.clear();
    const auto power = [&](int line) { const auto k = strong.find(line); return k == strong.end() ? 1.f : k->second; };
    const auto point = [&](float3 p, float3 n, float4 c, float flatly) { m.pos.push_back(p), m.nrm.push_back(n), m.uv.push_back(float2{flatly, 0.f}), m.color.push_back(c); };
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
        m.idx.insert(m.idx.end(), {o, o + 1, o + 2, o, o + 2, o + 3});
    };
    if (back[3] > 0.f) {   // (what it clears with, a wall behind it as far back as the sketch is likely to go, the stage's size where it stands)
        const float x = 0.75f * U * wide, y = 0.75f * U * high, z = -3.f;
        const float4 c{back[0], back[1], back[2], back[3]};
        const uint32_t o = (uint32_t)m.pos.size();
        const float3 n{0.f, 0.f, 1.f};
        point(float3{-x, -y, z}, n, c, 1.f), point(float3{x, -y, z}, n, c, 1.f), point(float3{x, y, z}, n, c, 1.f), point(float3{-x, y, z}, n, c, 1.f);
        m.idx.insert(m.idx.end(), {o, o + 1, o + 2, o, o + 2, o + 3});
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
        const auto seen = [&](float3 p) { return (e * float4{p, 1.f}).xyz * flip; };
        if (r[21] >= 0.f && m.pos.size() + shape->pos.size() <= MOST) {
            const uint32_t o = (uint32_t)m.pos.size();
            const float4 c{r[17], r[18], r[19], r[20]};
            for (size_t v = 0; v < shape->pos.size(); ++v) {
                const float3 n = v < shape->nrm.size() ? turn * shape->nrm[v] : float3{0.f, 0.f, 1.f};
                point(seen(shape->pos[v]), length(n) > 1e-6f ? normalize(n * float3{1.f, -1.f, 1.f}) : float3{0.f, 0.f, 1.f}, c, r[21]);
            }
            for (size_t v = 0; v + 2 < shape->idx.size(); v += 3) m.idx.insert(m.idx.end(), {o + shape->idx[v], o + shape->idx[v + 2], o + shape->idx[v + 1]});
        }
        if (r[27] > 0.f)
            for (size_t v = 0; v + 1 < shape->edges.size(); v += 2) band(seen(shape->pos[shape->edges[v]]), seen(shape->pos[shape->edges[v + 1]]), float4{r[23], r[24], r[25], r[26]}, r[27] * U);
    }
    for (size_t i = 0; i + 2 < tris && m.pos.size() + 3 <= MOST; i += 3) {
        const uint32_t o = (uint32_t)m.pos.size();
        for (const size_t v : {i, i + 2, i + 1}) {
            const float* r = all + draws * RET + v * TRI;
            const float3 n = normalize(float3{r[3], -r[4], r[5]} + float3{0.f, 0.f, 1e-6f});
            point(float3{r[0], r[1], r[2]} * flip, n, float4{r[6], r[7], r[8], r[9] * power((int)r[11])}, r[10]);
        }
        m.idx.insert(m.idx.end(), {o, o + 1, o + 2});
    }
    for (size_t i = 0; i < segs; ++i) {
        const float* r = all + draws * RET + tris * TRI + i * SEG;
        band(float3{r[0], r[1], r[2]} * flip, float3{r[3], r[4], r[5]} * flip, float4{r[6], r[7], r[8], r[9] * power((int)r[11])}, r[10] * U);
    }
    if (m.idx.empty()) point(float3{0.f}, float3{0.f, 0.f, 1.f}, float4{0.f}, 1.f), point(float3{0.f}, float3{0.f, 0.f, 1.f}, float4{0.f}, 1.f), point(float3{0.f}, float3{0.f, 0.f, 1.f}, float4{0.f}, 1.f), m.idx = {0, 1, 2};
    reshape(s, &m, {(uint32_t)m.idx.size()});
}
