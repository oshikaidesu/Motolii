// drawn_flat.h: part of drawn.h. Read only through drawn.h.
// A flat shape's triangles as its path is now (flat_mesh): its fill (none if hollow: cut by libtess2, made again only
// when its path changes, `refill`), then its edge (ThorVG's own stroker: trimmed and dashed as told, its triangles
// straight), in shares of its height from its middle (uv) and in the stage's units (pos), into its sheet; where each
// part's indices end (a part with none: one empty triangle).
static std::vector<uint32_t> shape_cut(Drawn* d, float trimFrom, float trimTo, bool refill) {
    const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
    d->outline->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
    Mesh& m = *d->sheet;
    const float w = d->cutW, h = d->plainHigh;
    const auto point = [&](float x, float y) {
        const float2 at{(x - 0.5f * w) / h, (0.5f * h - y) / h};
        m.pos.push_back({d->sheetHigh * at, 0.f}), m.nrm.push_back({0, 0, 1}), m.uv.push_back(at);
    };
    const auto none = [&] { if (m.pos.empty()) point(0.5f * w, 0.5f * h); for (int k = 0; k < 3; ++k) m.idx.push_back(0); };
    if (refill || m.idx.empty()) {
        m.pos.clear(), m.nrm.clear(), m.uv.clear(), m.idx.clear();
        if (!d->hollow && d->bent[9] > 0.f) {   // (made of squares: each is two triangles as it is, nothing to cut)
            for (uint32_t c = 0, p = 0; c < cmdsCnt; ++c)
                if (cmds[c] == tvg::PathCommand::MoveTo && c + 3 < cmdsCnt) {
                    const uint32_t k = (uint32_t)m.pos.size();
                    for (uint32_t i = 0; i < 4; ++i) point(pts[p + i].x, pts[p + i].y);
                    m.idx.insert(m.idx.end(), {k, k + 1, k + 2, k, k + 2, k + 3});
                    p += 4, c += 3;
                }
        } else if (!d->hollow) {   // (its outline: its pieces are short, so a curve is its end)
            TESStesselator* t = tessNewTess(nullptr);
            std::vector<float2> ring;
            const auto end = [&] { if (ring.size() > 2) tessAddContour(t, 2, ring.data(), sizeof(float2), (int)ring.size()); ring.clear(); };
            for (uint32_t c = 0, p = 0; c < cmdsCnt; ++c) switch (cmds[c]) {
                case tvg::PathCommand::MoveTo: end(); ring.push_back({pts[p].x, pts[p].y}); ++p; break;
                case tvg::PathCommand::LineTo: ring.push_back({pts[p].x, pts[p].y}); ++p; break;
                case tvg::PathCommand::CubicTo: ring.push_back({pts[p + 2].x, pts[p + 2].y}); p += 3; break;
                case tvg::PathCommand::Close: end(); break;
            }
            end();
            if (tessTesselate(t, d->fx.evenOdd ? TESS_WINDING_ODD : TESS_WINDING_NONZERO, TESS_POLYGONS, 3, 2, nullptr)) {
                const TESSreal* v = tessGetVertices(t);
                for (int i = 0; i < tessGetVertexCount(t); ++i) point(v[2 * i], v[2 * i + 1]);
                const TESSindex* e = tessGetElements(t);
                for (int i = 0; i < 3 * tessGetElementCount(t); ++i) m.idx.push_back((uint32_t)e[i]);
            }
            tessDeleteTess(t);
        }
        if (m.idx.empty()) none();
        d->fillPoints = (uint32_t)m.pos.size(), d->fillEnd = (uint32_t)m.idx.size();
    } else {   // (its fill as it was: its edge only made again)
        m.pos.resize(d->fillPoints), m.nrm.resize(d->fillPoints), m.uv.resize(d->fillPoints), m.idx.resize(d->fillEnd);
    }
    if (d->edgeWide <= 0.f) return {d->fillEnd};
    std::vector<float> xy;
    std::vector<uint32_t> idx;
    tvg_edge(cmds, cmdsCnt, pts, ptsCnt, d->edgeWide, d->fx.cap < 0 ? (int)tvg::StrokeCap::Square : d->fx.cap, d->fx.join < 0 ? (int)tvg::StrokeJoin::Bevel : d->fx.join,   // (ThorVG's own: square, bevel)
             d->fx.dash * h, d->fx.dashFrom * h, trimFrom, trimTo, xy, idx);
    const uint32_t base = (uint32_t)m.pos.size();
    for (size_t i = 0; i < xy.size(); i += 2) point(xy[i], xy[i + 1]);
    for (uint32_t i : idx) m.idx.push_back(base + i);
    if (d->fx.heads) {   // (an arrow head on each open outline's end, its start, or both, where its trim leaves them: a triangle in its edge's colour, pointing the way the line goes)
        static const char* const alight[] = {"t-line", "square", "square-filled", "diamond", "diamond-filled", "arrow-hollow", "arrow-filled", "circle-hollow", "circle-filled", "teardrop-hollow", "teardrop-filled"};
        const auto head = [&](tvg::Point at, tvg::Point back, int shape) {
            float dx = at.x - back.x, dy = at.y - back.y;
            const float len = std::hypot(dx, dy), l = d->fx.headSize * d->edgeWide;
            if (len < 1e-4f) return;
            dx /= len, dy /= len;
            if (shape >= 5) {   // (Alight Motion's own, as AbsentMotion draws them (absent_ends.cpp), as long as ours: its outline cut by libtess2 as the fill is, its curves in pieces)
                std::vector<tvg::PathCommand> c;
                std::vector<tvg::Point> p;
                if (!sk_svg(absent_line_end(at.x, at.y, dx, dy, 0.5f * l, d->edgeWide, d->fx.cap, d->fx.join, alight[std::min(shape - 5, 10)]).c_str(), c, p)) return;
                TESStesselator* t = tessNewTess(nullptr);
                std::vector<float2> ring;
                const auto close = [&] { if (ring.size() > 2) tessAddContour(t, 2, ring.data(), sizeof(float2), (int)ring.size()); ring.clear(); };
                for (size_t i = 0, q = 0; i < c.size(); ++i) switch (c[i]) {
                    case tvg::PathCommand::MoveTo: close(); ring.push_back({p[q].x, p[q].y}); ++q; break;
                    case tvg::PathCommand::LineTo: ring.push_back({p[q].x, p[q].y}); ++q; break;
                    case tvg::PathCommand::CubicTo: {
                        const float2 a = ring.empty() ? float2{p[q].x, p[q].y} : ring.back(), b{p[q].x, p[q].y}, e{p[q + 1].x, p[q + 1].y}, f{p[q + 2].x, p[q + 2].y};
                        for (int k = 1; k <= 8; ++k) { const float u = (float)k / 8.f, v = 1.f - u; ring.push_back(v * v * v * a + 3.f * v * v * u * b + 3.f * v * u * u * e + u * u * u * f); }
                        q += 3;
                        break;
                    }
                    case tvg::PathCommand::Close: close(); break;
                }
                close();
                if (tessTesselate(t, TESS_WINDING_NONZERO, TESS_POLYGONS, 3, 2, nullptr)) {
                    const uint32_t k = (uint32_t)m.pos.size();
                    const TESSreal* v = tessGetVertices(t);
                    for (int i = 0; i < tessGetVertexCount(t); ++i) point(v[2 * i], v[2 * i + 1]);
                    const TESSindex* e = tessGetElements(t);
                    for (int i = 0; i < 3 * tessGetElementCount(t); ++i) m.idx.push_back(k + (uint32_t)e[i]);
                }
                tessDeleteTess(t);
                return;
            }
            const uint32_t k = (uint32_t)m.pos.size();
            const auto put = [&](float along, float across) { point(at.x + along * l * dx - across * l * dy, at.y + along * l * dy + across * l * dx); };   // (in its own ways: along the line, across it; in its lengths)
            const auto quad = [&](uint32_t a) { m.idx.insert(m.idx.end(), {a, a + 1, a + 2, a, a + 2, a + 3}); };
            switch (shape) {
                case 1: {   // (a round on its end: as Cavalry's circleHead)
                    put(0.f, 0.f);
                    for (int i = 0; i <= 24; ++i) put(0.4f * std::cos(TAU * (float)i / 24.f), 0.4f * std::sin(TAU * (float)i / 24.f));
                    for (uint32_t i = 0; i < 24; ++i) m.idx.insert(m.idx.end(), {k, k + 1 + i, k + 2 + i});
                    break;
                }
                case 2: put(0.35f, -0.35f), put(0.35f, 0.35f), put(-0.35f, 0.35f), put(-0.35f, -0.35f), quad(k); break;   // (a square: boxHead)
                case 3: {   // (an open V, as thick as its edge: chevronHead)
                    const float w = 0.5f * d->edgeWide / l;
                    put(0.5f, w), put(0.5f, -w), put(-0.4f, -0.45f - w), put(-0.4f, -0.45f + w), quad(k);
                    put(0.5f, -w), put(0.5f, w), put(-0.4f, 0.45f + w), put(-0.4f, 0.45f - w), quad(k + 4);
                    break;
                }
                case 4: put(0.f + 0.5f * d->edgeWide / l, -0.45f), put(0.f + 0.5f * d->edgeWide / l, 0.45f), put(-0.5f * d->edgeWide / l, 0.45f), put(-0.5f * d->edgeWide / l, -0.45f), quad(k); break;   // (a bar across its end)
                default: put(0.5f, 0.f), put(-0.5f, 0.4f), put(-0.5f, -0.4f), m.idx.insert(m.idx.end(), {k, k + 1, k + 2});   // (a triangle: basicArrowHead)
            }
        };
        std::vector<float> ends;
        for (int k = 0, n = sk_ends(cmds, cmdsCnt, pts, trimFrom, trimTo, ends); k < n; ++k) {
            const float* e = &ends[8 * (size_t)k];
            if (d->fx.heads & 1) head({e[0], e[1]}, {e[0] - e[2], e[1] - e[3]}, d->fx.headShape);
            if (d->fx.heads & 2) head({e[4], e[5]}, {e[4] - e[6], e[5] - e[7]}, d->fx.headStart >= 0 ? d->fx.headStart : d->fx.headShape);
        }
    }
    if (idx.empty()) none();
    return {d->fillEnd, (uint32_t)m.idx.size()};
}

// A flat shape drawn by Filament itself, as the scene's other things are: its path made as `flat` makes it (ThorVG's),
// cut into triangles (libtess2, as Unity's vector shapes are) again only when its path changes (cut_at), shown with
// flat.mat washed in its colour on a white point (its edge a second part of it, in its edge's colour, ThorVG's own stroke); antialiased by
// the view's MSAA, its points moved in the look's vertex stage (bends). Nothing is drawn for it into a picture of its
// own: forty of them waving cost a quarter of the time as ThorVG pictures do (check/shape_mesh.sh). No ramp, no effects:
// those are `flat`'s. edge: its colour, and how wide (a share of its height; inside its frame, as `flat`'s).
static Drawn* flat_mesh(Stage& s, const char* form, float wide, const mat4f& place, float high, float3 colour, const char* lay = nullptr, float4 edge = float4{0.f},
                        const DrawnFx& fx = {}, int points = 6, float inner = 0.5f, float round = 0.f, bool bends = true) {
    const int h = 1024, w = std::max(2, (int)std::lround(1024.f * wide));   // (points to make its path in: its own, as fine as it is)
    Drawn* d = new Drawn;
    d->fx = fx, d->cutW = (float)w, d->sheetHigh = high, d->hollow = fx.hollow;
    const float in = std::round(0.5f * std::min(edge.w * (float)h, 0.5f * (float)std::min(w, h)));   // (half its edge: the path set in by that, as `flat` does, a whole point)
    d->edgeWide = 2.f * in;
    d->outline = tvg::Shape::gen();
    flat_path(d, d->outline, form, w, h, in, points, inner, round, (bends ? 1.f : -1.f) * (float)h / 64.f);   // (bent as it is drawn: cut fine all along; not: only its curves, its straight lines whole)
    d->sheet = new Mesh;
    const std::vector<uint32_t> ends = shape_cut(d, 0.f, 1.f, true);
    d->cut = true, d->wideBy = wide;
    d->wash = float4{Color::toLinear<ACCURATE>(sRGBColor{colour}), 1.f};
    const std::vector<MaterialInstance*> looks = add_parts(s, d->sheet, flat_laid(lay).c_str(), std::vector<mat4f>((size_t)std::max(1, fx.copies), place), ends, 1, false, 1, 4, fx.together);
    if (looks.empty()) return nullptr;
    d->look = looks[0];
    d->edgeLook = looks.size() > 1 ? looks[1] : nullptr;
    for (MaterialInstance* look : looks) look->setParameter("picture", white_point(s), TextureSampler());
    d->look->setParameter("wash", d->wash);
    if (d->edgeLook) d->edgeLook->setParameter("wash", float4{Color::toLinear<ACCURATE>(sRGBColor{edge.xyz}), 1.f});
    return d;
}

bool sk_rough(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, float apart, float far, uint32_t seed, std::vector<tvg::PathCommand>& outCmds, std::vector<tvg::Point>& outPts);
bool sk_resample(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, int n, std::vector<tvg::PathCommand>& outCmds, std::vector<tvg::Point>& outPts);
bool sk_voxel(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, float cell, std::vector<tvg::PathCommand>& outCmds, std::vector<tvg::Point>& outPts);
bool sk_morph(const tvg::PathCommand* ac, uint32_t acn, const tvg::Point* ap, const tvg::PathCommand* bc, uint32_t bcn, const tvg::Point* bp, float t,
              std::vector<tvg::PathCommand>& cmds, std::vector<tvg::Point>& pts);
bool sk_op(int op, const tvg::PathCommand* ac, uint32_t acn, const tvg::Point* ap, const tvg::PathCommand* bc, uint32_t bcn, const tvg::Point* bp,
           std::vector<tvg::PathCommand>& cmds, std::vector<tvg::Point>& pts);

// A flat shape in triangles cut anew for the frame that comes next, if its path is told otherwise: After Effects' trim
// paths on its edge (off its end, off its beginning, slid along), its operators (round, bloat, zig zag, offset: bend.cpp),
// then put together with another shape's path (`join`: Skia's path operation, skpath.cpp; -1 none; `by`/`at` that path
// in this one's points), as Cavalry's Boolean.
static std::atomic<size_t> cuts_held{0};
static constexpr size_t cuts_most = 256ull << 20;
Drawn::Cuts::~Cuts() { cuts_held -= bytes; }
static uint64_t cut_key(float3 trim, const std::array<float, 10>& bend, const Drawn::Other& join, const Drawn::Other& morph, const Drawn::Other& trace) {
    uint64_t h = 1469598103934665603ull;   // (FNV-1a over everything it is cut from)
    const auto mix = [&](const void* p, size_t n) { for (size_t i = 0; i < n; ++i) h = (h ^ ((const uint8_t*)p)[i]) * 1099511628211ull; };
    mix(&trim, sizeof trim), mix(bend.data(), sizeof bend);
    for (const Drawn::Other* o : {&join, &morph, &trace})
        mix(&o->how, sizeof o->how), mix(&o->much, sizeof o->much), mix(o->by.data(), o->by.size() * sizeof(tvg::PathCommand)), mix(o->at.data(), o->at.size() * sizeof(tvg::Point));
    return h;
}
static void cut_at(Stage& s, Drawn* d, float3 trim, const std::array<float, 10>& bend, Drawn::Other join = {}, Drawn::Other morph = {}, Drawn::Other trace = {}) {
    if (join.how < 0 || join.by.empty()) join = {};
    if (morph.much <= 0.f || morph.by.empty()) morph = {};
    if (trace.by.empty()) trace = {};
    const bool joins = !(join == d->joined) || !(morph == d->morphed) || !(trace == d->traced);
    if (!d->cut || !d->outline || (trim == d->trimmed && bend == d->bent && !joins)) return;
    const uint64_t key = cut_key(trim, bend, join, morph, trace);
    if (const auto kept = d->cuts.all.find(key); kept != d->cuts.all.end()) {   // (cut so before: as it was)
        const Drawn::Cut& c = kept->second;
        d->trimmed = trim, d->bent = bend, d->joined = std::move(join), d->morphed = std::move(morph), d->traced = std::move(trace);
        d->outline->reset(), d->outline->appendPath(c.by.data(), (uint32_t)c.by.size(), c.at.data(), (uint32_t)c.at.size());
        Mesh& m = *d->sheet;
        m.pos = c.pos, m.uv = c.uv, m.idx = c.idx, m.nrm.assign(m.pos.size(), float3{0, 0, 1});
        d->fillPoints = c.fillPoints, d->fillEnd = c.fillEnd;
        reshape(s, d->sheet, c.ends);
        return;
    }
    d->trimmed = trim;
    const bool refill = bend != d->bent || joins;
    if (refill) {
        d->bent = bend, d->joined = join, d->morphed = morph, d->traced = trace;
        const float high = d->plainHigh;
        const bool traces = !trace.by.empty();   // (tracing another: its track is its path, as Cavalry's Trails)
        std::vector<tvg::PathCommand> pathBy = traces ? trace.by : d->plain;
        std::vector<tvg::Point> pathAt = traces ? trace.at : d->plainAt;
        if (bend[8] >= 2.f) {   // (walked again at so many points an outline, straight between, before its operators: Cavalry's Resample Path)
            std::vector<tvg::PathCommand> c; std::vector<tvg::Point> p;
            if (sk_resample(pathBy.data(), (uint32_t)pathBy.size(), pathAt.data(), std::min((int)bend[8], 2048), c, p)) pathBy = std::move(c), pathAt = std::move(p);
        }
        tvg_bend(d->outline, pathBy.data(), (uint32_t)pathBy.size(), pathAt.data(), (uint32_t)pathAt.size(), std::clamp(bend[0], 0.f, 0.5f) * d->plainSide,
                 100.f * std::clamp(bend[1], -1.f, 1.f), bend[2] * high, (int)std::clamp(bend[3], 0.f, 100.f), bend[4] > 0.5f, bend[5] * high, d->fine);
        if (bend[6] > 0.f) {   // (roughened, as Illustrator's Roughen and After Effects' Roughen Edges: Skia's own discrete path effect, so far out, a point so often)
            const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
            d->outline->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
            std::vector<tvg::PathCommand> c; std::vector<tvg::Point> p;
            if (sk_rough(cmds, cmdsCnt, pts, std::max(bend[7], 0.005f) * high, std::min(bend[6], 0.5f) * high, 0, c, p))
                d->outline->reset(), d->outline->appendPath(c.data(), (uint32_t)c.size(), p.data(), (uint32_t)p.size());
        }
        if (morph.much > 0.f) {   // (turned into the other so far, as Cavalry's Morph: both walked at even steps, point to point)
            const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
            d->outline->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
            std::vector<tvg::PathCommand> c; std::vector<tvg::Point> p;
            if (morph.how > 1) {   // (a blend, as Illustrator's and Cavalry's: every step from itself to the other, evenly, in one path)
                std::vector<tvg::PathCommand> all; std::vector<tvg::Point> at;
                for (int k = 0; k < morph.how; ++k)
                    if (sk_morph(cmds, cmdsCnt, pts, morph.by.data(), (uint32_t)morph.by.size(), morph.at.data(), (float)k / (float)(morph.how - 1), c, p))
                        all.insert(all.end(), c.begin(), c.end()), at.insert(at.end(), p.begin(), p.end());
                if (!all.empty()) d->outline->reset(), d->outline->appendPath(all.data(), (uint32_t)all.size(), at.data(), (uint32_t)at.size());
            } else if (sk_morph(cmds, cmdsCnt, pts, morph.by.data(), (uint32_t)morph.by.size(), morph.at.data(), std::min(morph.much, 1.f), c, p))
                d->outline->reset(), d->outline->appendPath(c.data(), (uint32_t)c.size(), p.data(), (uint32_t)p.size());
        }
        if (join.how >= 0) {
            const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
            d->outline->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
            std::vector<tvg::PathCommand> c; std::vector<tvg::Point> p;
            sk_op(join.how, cmds, cmdsCnt, pts, join.by.data(), (uint32_t)join.by.size(), join.at.data(), c, p);
            d->outline->reset();
            if (!c.empty()) d->outline->appendPath(c.data(), (uint32_t)c.size(), p.data(), (uint32_t)p.size());
        }
        if (bend[9] > 0.f) {   // (in squares so large, last: Cavalry's Voxelize)
            const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
            d->outline->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
            std::vector<tvg::PathCommand> c; std::vector<tvg::Point> p;
            sk_voxel(cmds, cmdsCnt, pts, bend[9] * high, c, p);
            d->outline->reset();
            if (!c.empty()) d->outline->appendPath(c.data(), (uint32_t)c.size(), p.data(), (uint32_t)p.size());
        }
    }
    const float from = std::clamp(trim.y, 0.f, 1.f) + trim.z, to = 1.f - std::clamp(trim.x, 0.f, 1.f) + trim.z;
    const std::vector<uint32_t> ends = shape_cut(d, from, to, refill);
    reshape(s, d->sheet, ends);
    Drawn::Cut c;   // (kept, while there is room: the oldest of its own let go first)
    const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
    d->outline->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
    const Mesh& m = *d->sheet;
    c.bytes = cmdsCnt * sizeof *cmds + ptsCnt * sizeof *pts + m.pos.size() * (sizeof(float3) + sizeof(float2)) + m.idx.size() * sizeof(uint32_t) + 128;
    while (cuts_held + c.bytes > cuts_most && !d->cuts.order.empty()) {
        const auto old = d->cuts.all.find(d->cuts.order.front());
        d->cuts.order.pop_front();
        if (old == d->cuts.all.end()) continue;
        d->cuts.bytes -= old->second.bytes, cuts_held -= old->second.bytes;
        d->cuts.all.erase(old);
    }
    if (cuts_held + c.bytes > cuts_most) return;
    c.by.assign(cmds, cmds + cmdsCnt), c.at.assign(pts, pts + ptsCnt), c.pos = m.pos, c.uv = m.uv, c.idx = m.idx, c.ends = ends, c.fillPoints = d->fillPoints, c.fillEnd = d->fillEnd;
    d->cuts.bytes += c.bytes, cuts_held += c.bytes;
    d->cuts.order.push_back(key);
    d->cuts.all.emplace(key, std::move(c));
}

// One layer of an After Effects file drawn alone, in its own space: its place and turn left out (they are its sheet's,
// put in the scene by its pose: motion.h), on a sheet of its own `pixel` units a point of the composition. A 3D layer's
// picture where it is. `box`: where its sheet lies in the layer's own space (x, y, w, h, in points of the composition).
static Drawn* layer_picture(Stage& s, const char* file, uint32_t id, float pixel, float4* box) {
    if (!door::device(s.engine)) return nullptr;
    float4 found{0.f};
    door::on_driver(s.engine, [&] {   // (how large it is, at its first moment: ThorVG's own bounds of it alone)
        tvg::Initializer::init(0);
        tvg::Animation* look = tvg::LottieAnimation::gen();
        if (look->picture()->load(file) == tvg::Result::Success && (look->frame(1.f), look->frame(0.f), true))   // (built: ThorVG builds a moment when it is told one other than the one it has)
            if (const tvg::Paint* one = look->picture()->paint(id)) {
                tvg::Paint* copy = one->duplicate();
                copy->transform(tvg::Matrix{1, 0, 0, 0, 1, 0, 0, 0, 1});
                copy->bounds(&found.x, &found.y, &found.z, &found.w);
                tvg::Paint::rel(copy);
            }
        delete look;
    });
    s.engine->flushAndWait();
    if (found.z <= 0.f || found.w <= 0.f) return nullptr;
    *box = found;
    const float sharp = std::max(1.f, 512.f / found.w);   // (drawn at least 512 points high, however small it is in the composition)
    return drawn(s, [&](Drawn* d, int* w, int* h) -> tvg::Paint* {
        d->source = tvg::LottieAnimation::gen();
        if (d->source->picture()->load(file) != tvg::Result::Success) return nullptr;
        d->source->frame(1.f);   // (so that its first moment, asked for, is built)
        d->layerId = id;
        d->fit = tvg::Matrix{sharp, 0, -found.x * sharp, 0, sharp, -found.y * sharp, 0, 0, 1};
        *w = std::max(2, (int)std::ceil(found.z * sharp)), *h = std::max(2, (int)std::ceil(found.w * sharp));
        d->holder = tvg::Scene::gen();
        return d->holder;
    }, mat4f{}, found.w * pixel, float3{-1.f}, true, nullptr);
}

static Drawn* layer_picture(Stage& s, const char* file, const char* layer, float pixel, float4* box) { return layer_picture(s, file, tvg::Accessor::id(layer), pixel, box); }

// An After Effects picture's 3D layers (ThorVG says each layer's pose and whether it is 3D): from now on not drawn in
// its own picture, each to be a sheet of its own where it is (layer_picture). Their ids.
static std::vector<uint32_t> picture_deep(Stage& s, Drawn* d) {
    std::vector<uint32_t> ids;
    tvg::LottieAnimation* film = d ? d->lottie : nullptr;
    if (!film) return ids;
    door::on_driver(s.engine, [&] {
        auto* walk = tvg::Accessor::gen();
        walk->set(film->picture(), [&](const tvg::Paint* one, void*) {
            float n[19];
            uint32_t above = 0;
            int kind = 0;
            if (one->id && film->pose(one->id, n, &above, &kind) == tvg::Result::Success && kind == 1 && std::find(ids.begin(), ids.end(), one->id) == ids.end()) ids.push_back(one->id);
            return true;
        }, nullptr);
        delete walk;
    });
    s.engine->flushAndWait();
    d->deep = ids;
    return ids;
}

// That layer as it is at `frame` of its file, drawn again for the frame that comes next.
static void layer_at(Stage& s, Drawn* d, float frame) {
    if (!d || !d->source) return;
    door::on_driver(s.engine, [d, frame] {
        d->source->frame(frame);
        d->holder->remove();
        if (const tvg::Paint* one = d->source->picture()->paint(d->layerId)) {
            tvg::Paint* copy = one->duplicate();
            copy->transform(d->fit);
            d->holder->add(copy);
        }
        d->canvas->update();
        d->canvas->draw(true);
        d->canvas->sync();
    });
}

// A picture taken off the stage for good.
// Nothing waits for it (as Filament's own destroy): what ThorVG holds, and the picture itself, go on the driver thread
// after every frame sent before that may still draw it, since that thread runs what it is given in turn.
static void picture_off(Stage& s, Drawn* d) {
    s.playing.erase(d);
    take_off(s, d->sheet);
    d->look = nullptr;
    kept_drop(s, d);
    door::on_driver(s.engine, [d] { delete d->canvas; delete d->film; d->canvas = nullptr, d->film = nullptr; });   // (ThorVG let go of its texture before Filament destroys it)
    if (d->texture) s.engine->destroy(d->texture);
    door::on_driver(s.engine, [d] { delete d; });
}
