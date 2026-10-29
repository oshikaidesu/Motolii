// doc_relations.h: part of doc.h (behaviours, falloffs, relations, physics). Read only through doc.h.
static double doc_behaviour_at(Doc& d, int by, int copy, int n, double t) {
    static int depth = 0;   // (behaviours given by behaviours, as Cavalry chains them: so deep at most, a ring of them stops)
    struct Deeper { Deeper() { ++depth; } ~Deeper() { --depth; } } deeper;
    const auto knob = [&](const char* name, double otherwise) {   // (one of its knobs: given by another behaviour, that one's for this copy)
        if (depth < 4) if (const int other = doc_driven_by(d, by, name); other && other != by) return doc_behaviour_at(d, other, copy, n, t);
        return doc_number(d, by, name, t, otherwise);
    };
    const std::string type = doc_word(d, by, "file");
    const double lo = knob("minimum", 0), hi = knob("maximum", 1), offset = knob("offset", 0);
    const double frequency = knob("frequency", 1), apart = knob("stagger", 0) / 360.0;
    const uint32_t seed = (uint32_t)doc_number(d, by, "seed", 0, 0);
    const auto hash = [&](int64_t k) {   // (0 to 1: the seed and k)
        uint32_t h = seed * 0x9E3779B9u ^ (uint32_t)k * 0x85EBCA6Bu ^ 0xC2B2AE35u;
        h ^= h >> 16, h *= 0x7FEB352Du, h ^= h >> 15, h *= 0x846CA68Bu, h ^= h >> 16;
        return (double)h / 4294967295.0;
    };
    double u = 0.0;
    if (type == "stagger") u = std::clamp((n > 1 ? (double)copy / (n - 1) : 0.0) + offset, 0.0, 1.0);   // (in their order)
    else if (type == "oscillator") u = 0.5 + 0.5 * std::sin((double)TAU * (frequency * t + offset + apart * copy));
    else if (type == "noise") {   // (a smooth noise along time, each copy so far along it)
        const double x = frequency * t + offset + apart * copy * 4.0, f = std::floor(x), e = (x - f) * (x - f) * (3.0 - 2.0 * (x - f));
        u = hash((int64_t)f) * (1.0 - e) + hash((int64_t)f + 1) * e;
    }
    else if (type == "random") u = hash(copy + 1);
    const double step = knob("step", 0), v = lo + (hi - lo) * u;
    return step > 0.0 ? std::round(v / step) * step : v;
}
// The behaviour a value of a thing is connected to (`<name>_by`), 0 none.
static bool doc_connects(Doc& d, int thing) {
    if (const auto known = d.connects.find(thing); known != d.connects.end()) return known->second;
    bool any = false;
    doc_rows(d, sqlite3_mprintf("select 1 from fact where thing = %d and name glob '*_by' and value <> 0 limit 1", thing), [&](sqlite3_stmt*) { any = true; });
    return d.connects.emplace(thing, any).first->second;
}
static int doc_driven_by(Doc& d, int thing, const char* name) {
    if (!doc_connects(d, thing)) return 0;
    const auto key = std::make_pair(thing, std::string(name));
    if (const auto known = d.drivers.find(key); known != d.drivers.end()) return known->second;
    int by = (int)doc_number(d, thing, (std::string(name) + "_by").c_str(), 0, 0);
    if (by && doc_word(d, by, "kind") != "behaviour") by = 0;
    return d.drivers.emplace(key, by).first->second;
}
bool sk_inside(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, float x, float y, float* far);
static math::mat4 doc_place(Doc& d, int thing, double now, int deeper, int copy, bool member);
// How much of a behaviour works at a place on the stage: its falloff's shape (where it lies now, as shape_cut lays its
// outline), all inside, none outside past falloff_soft; 1 with no falloff.
static double doc_falloff_at(Doc& d, int by, math::double3 at, double t) {
    const int f = (int)doc_number(d, by, "falloff", 0, 0);
    if (!f) return 1.0;
    const auto found = d.shown.find(f);
    const Drawn* b = found == d.shown.end() ? nullptr : found->second.drawn;
    if (!b || !b->cut || !b->outline) return 1.0;
    const math::double4 on = inverse(doc_place(d, f, t, 0, 0, false)) * math::double4{at, 1.0};
    const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
    b->outline->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
    float far = 0.f;
    const bool inside = sk_inside(cmds, cmdsCnt, pts, (float)(on.x / b->sheetHigh * b->plainHigh + 0.5 * b->cutW), (float)(0.5 * b->plainHigh - on.y / b->sheetHigh * b->plainHigh), &far);
    const double soft = doc_number(d, by, "falloff_soft", t, 0), away = far / b->plainHigh * b->sheetHigh;
    const double w = inside ? 1.0 : soft > 0.0 ? std::max(0.0, 1.0 - away / soft) : 0.0;
    return doc_number(d, by, "falloff_invert", 0, 0) > 0.5 ? 1.0 - w : w;
}
// A value given by a behaviour to a copy: what it gives, weighed by its falloff where the copy lies (`at`, if known)
// against the value's own.
static double doc_given(Doc& d, int by, int copy, int n, double t, double own, const math::double3* at) {
    const double v = doc_behaviour_at(d, by, copy, n, t);
    if (!at) return v;
    const double w = doc_falloff_at(d, by, *at, t);
    return own + (v - own) * w;
}
static void doc_copy_looks_off(Doc& d, const Mesh* m, MaterialInstance* main) {
    Stage& s = *d.stage;
    auto& faces = s.engine->getRenderableManager();
    for (const Stage::Made& made : s.made) {
        if (made.of != m) continue;
        const auto found = d.copyLooks.find(made.thing.getId());
        if (found == d.copyLooks.end()) continue;
        if (main) faces.setMaterialInstanceAt(faces.getInstance(made.thing), 0, main);
        s.engine->destroy(found->second);
        d.copyLooks.erase(found);
        if (const auto edge = d.copyEdges.find(made.thing.getId()); edge != d.copyEdges.end()) {   // (its edge's own too, back to the thing's edge look)
            const Doc::Shown* t = nullptr;
            for (const auto& [id, one] : d.shown) if (one.drawn && one.drawn->sheet == m) t = &one;
            if (t && t->drawn->edgeLook) faces.setMaterialInstanceAt(faces.getInstance(made.thing), 1, t->drawn->edgeLook);
            s.engine->destroy(edge->second);
            d.copyEdges.erase(edge);
        }
    }
}
// How many members a thing has in their order: a text split into letters, a letter each; else its copies.
// A value doc_place reads, by a name written in the program (a literal: its place in memory says which name, ever):
// where its keys are is found once and held until the document changes (as Beutl and Skottie hold a property), each
// moment reckoned from them; a value with no keys or one is its number at once. Not while a hand carries it.
static double doc_literal(Doc& d, int thing, const char* literal, double at, double otherwise);
// A relation's things (any of their values `<name>_by` = it, or = a relation that starts from it for them), in the order
// its members word says (0 where one was taken out, its place kept), else in the order they were made; with what each
// was in before and where it showed when it came in (under and from: as many as members, else none).
static const Doc::Relation& doc_relation_of(Doc& d, int by) {
    if (const auto known = d.relationsOf.find(by); known != d.relationsOf.end()) return known->second;
    static int depth = 0;   // (relations on relations: so deep at most, a ring of them stops)
    struct Deeper { Deeper() { ++depth; } ~Deeper() { --depth; } } deeper;
    Doc::Relation r;
    std::vector<int> connected;
    doc_rows(d, sqlite3_mprintf("select distinct thing from fact where name glob '*_by' and value = %d order by thing", by), [&](sqlite3_stmt* q) { connected.push_back(sqlite3_column_int(q, 0)); });
    const auto words = [&](const char* name) { std::vector<std::string> all; std::istringstream said(doc_word(d, by, name)); for (std::string one; said >> one;) all.push_back(one); return all; };
    const std::vector<std::string> members = words("members"), under = words("under"), from = words("from");
    const auto reaches = [&](int t) {   // (in it through a relation that starts from it)
        for (int cur = doc_driven_by(d, t, "x") ? doc_driven_by(d, t, "x") : doc_driven_by(d, t, "y"), n = 0; cur && n < 8 && depth < 6; ++n) {
            if (cur == by) return true;
            const Doc::Relation& above = doc_relation_of(d, cur);
            const auto at = std::find(above.members.begin(), above.members.end(), t);
            cur = at == above.members.end() || above.under.empty() ? 0 : above.under[(size_t)(at - above.members.begin())];
        }
        return false;
    };
    for (size_t k = 0; k < members.size(); ++k) {
        const int t = std::atoi(members[k].c_str());
        const bool in = t && std::find(r.members.begin(), r.members.end(), t) == r.members.end() && (std::find(connected.begin(), connected.end(), t) != connected.end() || reaches(t));
        r.members.push_back(in ? t : 0);
    }
    for (const int t : connected) if (std::find(r.members.begin(), r.members.end(), t) == r.members.end()) r.members.push_back(t);
    if (under.size() == members.size()) for (const std::string& u : under) r.under.push_back(std::atoi(u.c_str()));
    if (from.size() == 2 * members.size()) for (const std::string& f : from) r.from.push_back(std::atof(f.c_str()));
    r.under.resize(r.under.empty() ? 0 : r.members.size(), 0);
    r.from.resize(r.from.empty() ? 0 : 2 * r.members.size(), std::nan(""));
    return d.relationsOf.emplace(by, std::move(r)).first->second;
}
static const std::vector<int>& doc_members_of(Doc& d, int by) { return doc_relation_of(d, by).members; }
// What a relation gives a value of one of its things (`own`: the thing's own value there), weighed by its strength: it
// starts from what the relation the thing was in before gives (else its own value); an around or a row that knows where
// the thing showed when it came in adds the step from there to its place (so a hand that moves the thing moves it), a
// follow that knows adds how far its leader has gone since (else each puts it at its place, or at its leader's).
static double doc_related(Doc& d, int by, int thing, const char* name, double t, double own) {
    static int depth = 0;   // (a leader in a relation that leads back: so deep at most)
    struct Deeper { Deeper() { ++depth; } ~Deeper() { --depth; } } deeper;
    if (depth > 6) return own;
    const std::string type = doc_word(d, by, "file"), n = name;
    const Doc::Relation& rel = doc_relation_of(d, by);
    const auto found = std::find(rel.members.begin(), rel.members.end(), thing);
    if (found == rel.members.end()) return own;
    const size_t i = (size_t)(found - rel.members.begin());
    const int many = std::max(1, (int)rel.members.size()), was = rel.under.empty() ? 0 : rel.under[i];
    const auto start = [&](const char* what, double mine) { return was ? doc_related(d, was, thing, what, t, mine) : mine; };
    const double ti = type == "follow" ? t : t - doc_number(d, by, "lag", t, 0) * (double)i;   // (in turn: each so much later than the one before, as Cavalry's stagger; a follow's lag is its own)
    const double base = start(name, own), s = std::clamp(doc_number(d, by, "strength", ti, 1), -1.0, 2.0);   // (past 1 and below 0 as an ease overshoots: a Back or an Elastic between two keys shows; held within twice)
    if (doc_number(d, by, "visible", 0, 1) <= 0.5) return base;   // (switched off, its eye shut: what it is on top of shows, its strength kept)
    const double came = rel.from.empty() || (n != "x" && n != "y") ? std::nan("") : rel.from[2 * i + (n == "y" ? 1 : 0)];   // (where it showed when it came in: across and up only)
    const auto leader = [&](const char* which) { const int l = (int)doc_number(d, by, which, 0, 0); return l != thing ? l : 0; };
    if ((type == "around" || type == "row" || type == "group") && (n == "rotate" || n == "rotate_x" || n == "rotate_y"))   // (its own turn added to each, in turn)
        return base + doc_number(d, by, name, ti, 0) * s;
    if ((type == "around" || type == "row" || type == "group") && (n == "size" || n == "opacity"))   // (its own size and opacity as a share of each one's, in turn: 0 opacity out is gone)
        return base * (1.0 + (doc_number(d, by, name, ti, 1) - 1.0) * s);
    if (type == "group") {   // (where it was put, and away by its own move, its spread from their middle and its scatter, as much as its strength, in turn)
        if (n != "x" && n != "y") return base;
        const size_t axis = n == "x" ? 0 : 1;
        double middle = 0;
        int counted = 0;
        for (size_t k = 0; 2 * k + axis < rel.from.size(); ++k)
            if (!std::isnan(rel.from[2 * k + axis])) middle += rel.from[2 * k + axis], ++counted;   // (their middle where they were when they came in)
        if (!counted)   // (not told where they came in: their own places now)
            for (const int t : rel.members) if (t) middle += doc_literal(d, t, name, ti, 0), ++counted;
        middle = counted ? middle / counted : base;
        const double seed = std::sin((double)(i + 1) * (axis ? 78.233 : 12.9898)) * 43758.5453, scatter = (seed - std::floor(seed)) * 2 - 1;   // (fixed for each: the same scatter every time it is shown)
        const double away = doc_number(d, by, name, ti, 0) + (base - middle) * (doc_number(d, by, "spread", ti, 1) - 1) + doc_number(d, by, "scatter", ti, 0) * scatter;
        return base + away * s;
    }
    if (type == "around" || type == "row") {   // (around a middle, the first at the top, each as far round as its order; or in a row from the middle, gap apart)
        if (n != "x" && n != "y" && n != "z") return base;
        const int l = leader("leader");
        const double tilt = type == "around" ? doc_number(d, by, "tilt", t, 0) * TAU / 360.0 : 0.0;
        if (n == "z" && tilt == 0.0 && !l) return base;   // (a ring facing the camera with no leader leaves depth alone)
        const double cx = doc_number(d, by, "centre_x", t, 0) + (l ? doc_literal(d, l, "x", t, 0) : 0.0), cy = doc_number(d, by, "centre_y", t, 0) + (l ? doc_literal(d, l, "y", t, 0) : 0.0);
        double place;
        if (type == "around") {   // (round its middle, tipped back by its tilt about the across axis; round its leader in depth too)
            const double turn = TAU * (doc_number(d, by, "offset", t, 0) + (double)i / many), r = doc_number(d, by, "radius", t, 0.5);
            const double cz = l ? doc_literal(d, l, "z", t, 0) : 0.0, up = r * std::cos(turn);
            if (n == "z") return base + (cz + up * std::sin(tilt) - base) * s;
            place = n == "x" ? cx + r * std::sin(turn) : cy + up * std::cos(tilt);
        } else if (n == "z") return base;
        else place = n == "x" ? cx + doc_number(d, by, "gap", t, 0.2) * ((double)i - (many - 1) / 2.0) : cy;   // (a row about its middle: a wider gap spreads it both ways)
        return std::isnan(came) ? base + (place - base) * s : base + (place - came) * s;
    }
    if ((type == "follow" || type == "push") && d.settled) return base;
    if (type == "follow") {   // (where its leader was so many seconds ago, each later than the one before)
        const int a = leader("leader"), b = leader("leader_b");
        if (!a) return base;
        const double back = t - doc_number(d, by, "lag", t, 0) * (double)(i + 1), hand = b ? std::clamp(doc_number(d, by, "hand", t, 0), 0.0, 1.0) : 0.0;
        const double from = doc_literal(d, a, name, back, own), to = b ? doc_literal(d, b, name, back, own) : from, lead = from + (to - from) * hand;
        return std::isnan(came) ? base + lead * s : base + (lead - came) * s;
    }
    if (type == "push") {   // (out of its leader's way: within radius pressed towards its edge, the nearer still nearer; further off as it is)
        const int l = leader("leader");
        if (!l || (n != "x" && n != "y")) return base;
        const double bx = n == "x" ? base : start("x", doc_keys_at(d, doc_keys_of(d, thing, "x"), t, 0)), by_ = n == "y" ? base : start("y", doc_keys_at(d, doc_keys_of(d, thing, "y"), t, 0));
        const double lx = doc_literal(d, l, "x", t, 0), ly = doc_literal(d, l, "y", t, 0), r = doc_number(d, by, "radius", t, 0.5);
        double dx = bx - lx, dy = by_ - ly, far = std::hypot(dx, dy);
        if (far >= r || r <= 0.0) return base;
        if (far < 1e-6) dx = std::sin(TAU * (double)i / many), dy = std::cos(TAU * (double)i / many), far = 1e-6;
        const double out = r * (0.6 + 0.4 * far / r), v = n == "x" ? lx + dx / far * out : ly + dy / far * out;   // (into the outer part of the ring, in their order: a clear way, none piled on another)
        return base + (v - base) * s;
    }
    return base;
}
static double doc_literal(Doc& d, int thing, const char* literal, double at, double otherwise) {
    if (d.held.thing == thing) {   // (carried by a hand: what it carries; in a relation, still in it, the hand moving its own value)
        const double own = doc_number(d, thing, literal, at, otherwise);
        const int by = doc_driven_by(d, thing, literal);
        return by && doc_among(doc_relations, doc_word(d, by, "file")) ? doc_related(d, by, thing, literal, at, own) : own;
    }
    if (const int by = doc_driven_by(d, thing, literal)) {   // (given by a behaviour: the copy being placed's own, where it lies weighed by its falloff)
        const Doc::Keys& k = doc_keys_of(d, thing, literal);
        const double own = doc_keys_at(d, k, doc_stepped(d, thing, k, at), otherwise);
        if (doc_among(doc_relations, doc_word(d, by, "file"))) return doc_related(d, by, thing, literal, at, own);   // (a relation: by the thing's place among its things, not a copy's)
        return doc_given(d, by, d.ctxCopy, d.ctxN, at, own, d.ctxHasAt ? &d.ctxAt : nullptr);
    }
    const auto key = std::make_pair(thing, literal);
    auto kept = d.literals.find(key);
    if (kept == d.literals.end()) kept = d.literals.emplace(key, &doc_keys_of(d, thing, literal)).first;
    return doc_keys_at(d, *kept->second, doc_stepped(d, thing, *kept->second, at), otherwise);
}
static const std::string& doc_literal_word(Doc& d, int thing, const char* literal) {   // (a word so, kept the same way)
    const auto key = std::make_pair(thing, literal);
    if (const auto kept = d.heardWords.find(key); kept != d.heardWords.end()) return kept->second;
    return d.heardWords.emplace(key, doc_word(d, thing, literal)).first->second;
}
// A thing's copies as AbsentMotion's repeat effects on it lay them (absent_motion.h), at a moment: made again only when
// the moment, the document or a file has changed. None: no repeat effect on it.
static const std::vector<AbsentCopy>& doc_absent_copies(Doc& d, int thing, double now) {
    Doc::AbsentKept& k = d.absentKept[thing];
    const long changes = d.file ? (long)sqlite3_total_changes(d.file) : 0;
    if (k.at == now && k.changes == changes && k.seen == files_changed()) return k.copies;
    k.at = now, k.changes = changes, k.seen = files_changed();
    k.copies.clear();
    if (doc_word(d, thing, "fx").find(".am") == std::string::npos) return k.copies;
    const double begins = doc_literal(d, thing, "start", 0, 0), lasts = doc_literal(d, thing, "duration", 0, -1);
    const double high = doc_number(d, 0, "high", 0, 1080), k2 = high / std::max(1, d.stage->h);   // (a point of the picture in the document's pixels)
    const math::double2 at = doc_seen(d, (doc_above(d, thing, now, 0) * math::double4{doc_literal(d, thing, "x", now, 0), doc_literal(d, thing, "y", now, 0), doc_literal(d, thing, "z", now, 0), 1.0}).xyz) * k2;
    float2 half{0.5f};
    doc_half(d, thing, &half);
    const double reach = std::max(half.x, half.y) * doc_literal(d, thing, "size", now, 1) * doc_point(d, thing, now) * k2 * 1.415;   // (as CopiesOf: its box's farthest from where it stands, by 1.415)
    k.copies = absent_copies(doc_absent_on(d, thing, now), thing, 1000.0 * begins, 1000.0 * (begins + (lasts > 0 ? lasts : doc_seconds(d))), 1000.0 * now, (float)(d.stage->w * k2), (float)high, (float)at.x, (float)at.y, (float)reach);
    return k.copies;
}
// A thing's opacity as AbsentMotion's opacity effects on it leave it (a share, 1 with none: absent_motion.h).
static double doc_absent_opacity(Doc& d, int thing, double now) {
    if (doc_word(d, thing, "fx").find(".am") == std::string::npos) return 1.0;
    const double begins = doc_literal(d, thing, "start", 0, 0), lasts = doc_literal(d, thing, "duration", 0, -1);
    return absent_opacity(doc_absent_on(d, thing, now), thing, 1000.0 * begins, 1000.0 * (begins + (lasts > 0 ? lasts : doc_seconds(d))), 1000.0 * now);
}
static int doc_members(Doc& d, int thing) {
    const auto t = d.shown.find(thing);
    if (t != d.shown.end() && t->second.drawn && !t->second.drawn->letters.empty()) return (int)t->second.drawn->letters.size();
    if (const std::vector<AbsentCopy>& copies = doc_absent_copies(d, thing, d.stage->now); !copies.empty()) return (int)copies.size();   // (as many as AbsentMotion's repeat lays)
    return std::max(1, (int)doc_literal(d, thing, "copies", 0, 1));
}
// How much later a thing's nth copy moves than it is said to (the past repository's order law, GSAP's stagger: the
// stagger spread over them by their order: from the start, from the centre out, from the end, from the edges in).
static double doc_delay(Doc& d, int thing, int nth) {
    const double stagger = doc_literal(d, thing, "stagger", 0, 0);
    const int n = doc_members(d, thing);
    if (stagger <= 0.0 || n < 2) return 0.0;
    const double along = (double)nth / (double)(n - 1), centre = std::fabs(along - 0.5) * 2.0;
    const std::string& from = doc_literal_word(d, thing, "stagger_from");
    return stagger * (from == "centre" ? centre : from == "end" ? 1.0 - along : from == "edges" ? 1.0 - centre : along);
}
// Where the thing it goes with is (its parent's place; none, or one not there any more: as the stage is). A 2D thing
// going with the camera is as the stage is: it sticks to the camera already (doc_place), and would take its moves twice.
static math::mat4 doc_above(Doc& d, int thing, double now, int deeper = 0) {
    const int parent = (int)doc_literal(d, thing, "parent", 0, 0);
    const auto up = d.shown.find(parent), self = d.shown.find(thing);
    if (up != d.shown.end() && up->second.camera && self != d.shown.end() && self->second.space == 2) return math::mat4{};
    return parent && up != d.shown.end() && deeper < 64 ? doc_place(d, parent, now, deeper + 1) : math::mat4{};   // (64: a document written elsewhere with a ring of parents still ends)
}
// Where a split text's letter has its middle across the thing, in its own units (0: not a letter of one).
static double doc_letter_middle(Doc& d, int thing, int copy) {
    const auto sh = d.shown.find(thing);
    if (copy < 0 || sh == d.shown.end() || !sh->second.drawn || copy >= (int)sh->second.drawn->letters.size()) return 0.0;
    float2 half{0.5f};
    float deep = 0.f;
    doc_half(d, thing, &half, &deep);
    const float2 l = sh->second.drawn->letters[(size_t)copy];
    return ((double)(l.x + l.y) * 0.5 - 0.5) * 2.0 * half.x;
}
// The world of the things that fall and strike, made from the document when first asked: each one's shape (a box as
// large as it is, a ball for a ball, flat things held to their plane), what it bounces and rubs, where its keys put
// it. Where the world has one at a moment (false: the world does not have it then, its keys do).
static bool doc_physics(Doc& d, int thing, double now, PhysPose* pose, int copy = -1) {
    if (d.phys_stale) {   // (all the world is made of: every value and key of the things with physics, and the pull down)
        d.phys_stale = false;
        std::string made;
        doc_rows(d, sqlite3_mprintf("select group_concat(thing || ' ' || name || ' ' || at || ' ' || quote(value) || ' ' || coalesce(way, ''), ';') from (select * from fact where thing in "
                                    "(select thing from fact where name = 'physics' and value in ('moves', 'still')) or (thing = 0 and name = 'gravity') order by thing, name, at)"),
                 [&](sqlite3_stmt* row) { if (sqlite3_column_type(row, 0) == SQLITE_TEXT) made = (const char*)sqlite3_column_text(row, 0); });
        if (made != d.phys_made) d.phys.reset(), d.phys_of.clear(), d.phys_copy.clear(), d.phys_until.clear(), d.phys_made = made;
        d.phys_any = made.find(" physics ") != std::string::npos;
    }
    if (!d.phys_any) return false;   // (no thing has physics: nothing asked of the document at every place)
    const std::string mine = doc_word(d, thing, "physics");
    if (mine != "moves") return false;
    if (!d.phys) {
        std::vector<PhysThing> made;
        d.phys_of.clear(), d.phys_copy.clear(), d.phys_until.clear();
        for (int t : doc_laid(d)) {
            const std::string how = doc_word(d, t, "physics");
            if ((how != "moves" && how != "still") || doc_number(d, t, "parent", 0, 0) != 0.0) continue;
            float2 half{0.5f};
            float deep = 0.f;
            if (!doc_half(d, t, &half, &deep)) continue;
            const double size = doc_number(d, t, "size", 0, 1);
            PhysThing one;
            one.still = how == "still", one.flat = deep == 0.f;
            one.ball = doc_word(d, t, "shape") == "ball";
            one.half[0] = (float)(half.x * size * std::fabs(doc_number(d, t, "scale_x", 0, 1)));
            one.half[1] = (float)(half.y * size * std::fabs(doc_number(d, t, "scale_y", 0, 1)));
            one.half[2] = one.flat ? 0.05f : (float)(deep * size * std::fabs(doc_number(d, t, "scale_z", 0, 1)));
            one.bounce = (float)doc_number(d, t, "bounce", 0, 0.3), one.friction = (float)doc_number(d, t, "friction", 0, 0.5);
            for (const char* name : {"x", "y", "z", "rotate", "rotate_x", "rotate_y"}) {   // (its keys hold it till the last of them)
                doc_rows(d, sqlite3_mprintf("select max(at) from fact where thing = %d and name = %Q and " DOC_KEYED, t, name, t, name), [&](sqlite3_stmt* row) {
                    if (sqlite3_column_type(row, 0) != SQLITE_NULL) one.until = std::max(one.until, sqlite3_column_double(row, 0));
                });
            }
            const auto sh = d.shown.find(t);
            const std::vector<float2>* letters = sh != d.shown.end() && sh->second.drawn && !sh->second.drawn->letters.empty() ? &sh->second.drawn->letters : nullptr;
            const int copies = letters ? (int)letters->size() : 1;   // (a split text: each letter a body of its own, as wide as its ink, a little narrower so neighbours do not start inside each other)
            for (int c = 0; c < copies; ++c) {
                PhysThing each = one;
                if (letters) each.half[0] = std::max(0.01f, 0.92f * ((*letters)[(size_t)c].y - (*letters)[(size_t)c].x) * one.half[0]);
                made.push_back(each);
                d.phys_of.push_back(t), d.phys_copy.push_back(letters ? c : -1), d.phys_until.push_back(one.still ? 1e9 : one.until);
            }
        }
        const auto keyed = [&d](int index, double at) {   // (where its keys put its middle, and how they turn it)
            const int t = d.phys_of[(size_t)index], copy = d.phys_copy[(size_t)index];
            d.phys_keyed = true;
            const math::double3 middle = (doc_place(d, t, at, 0, std::max(copy, 0), copy >= 0) * math::double4{doc_letter_middle(d, t, copy), 0, 0, 1}).xyz;
            d.phys_keyed = false;
            const auto half = [](double degrees, math::double3 about) { const double a = 0.5 * degrees * (double)TAU / 360.0; return math::quat{std::cos(a), about.x * std::sin(a), about.y * std::sin(a), about.z * std::sin(a)}; };
            const math::quat q = half(doc_number(d, t, "rotate", at, 0), {0, 0, 1}) * half(doc_number(d, t, "rotate_y", at, 0), {0, 1, 0}) * half(doc_number(d, t, "rotate_x", at, 0), {1, 0, 0});
            PhysPose p;
            p.at[0] = (float)middle.x, p.at[1] = (float)middle.y, p.at[2] = (float)middle.z;
            p.turn[0] = (float)q.x, p.turn[1] = (float)q.y, p.turn[2] = (float)q.z, p.turn[3] = (float)q.w;
            return p;
        };
        d.phys = std::make_unique<PhysWorld>(made, (float)doc_number(d, 0, "gravity", 0, 9.81), keyed);
    }
    size_t index = d.phys_of.size();
    for (size_t i = 0; i < d.phys_of.size(); ++i) if (d.phys_of[i] == thing && d.phys_copy[i] == copy) { index = i; break; }   // (a thing's copies that are not letters keep to their keys)
    if (index == d.phys_of.size()) return false;
    if (now <= d.phys_until[index]) return false;   // (its keys have it still)
    *pose = d.phys->pose((int)index, (int)std::lround(now * 60.0));
    return true;
}
// Whether the stage places a thing as a member of its copies (each copy, the first too, as late as its order says, waving
// as its place in the wave says), or as itself: the stage and whoever asks how it moves (motolii_path) place it alike.
static bool doc_staggered(Doc& d, int thing, double now) {
    return doc_members(d, thing) > 1 || doc_number(d, thing, "stagger", 0, 0) > 0.0 || doc_number(d, thing, "wave_x", now, 0) != 0.0 || doc_number(d, thing, "wave_y", now, 0) != 0.0 || doc_number(d, thing, "wave_rotate", now, 0) != 0.0 || doc_number(d, thing, "wave_size", now, 0) != 0.0 || doc_number(d, thing, "field_radius", now, 0) > 0.0 || doc_number(d, thing, "scatter_x", now, 0) > 0.0 || doc_number(d, thing, "scatter_y", now, 0) > 0.0 || doc_number(d, thing, "scatter_rotate", now, 0) > 0.0 || doc_number(d, thing, "scatter_size", now, 0) > 0.0;
}
// How far a thing that turns the way it goes (orient) is turned by it at a moment, radians about z: the way its keys
// carry it then; standing still, the way it last went or will go. 0 for any other.
static std::vector<double> doc_key_moments(Doc& d, int thing, const char* name);
static double doc_heading(Doc& d, int thing, double now) {
    if (doc_literal(d, thing, "orient", 0, 0) <= 0.5) return 0.0;
    const auto going = [&](double from, double to, double* heading) {
        const double ax = doc_literal(d, thing, "x", from, 0), ay = doc_literal(d, thing, "y", from, 0), bx = doc_literal(d, thing, "x", to, 0), by = doc_literal(d, thing, "y", to, 0);
        return std::hypot(bx - ax, by - ay) > 1e-6 && (*heading = std::atan2(by - ay, bx - ax), true);
    };
    double heading = 0.0;
    for (const double step : {1.0 / 240.0, 1.0 / 30.0})
        if (going(now - step, now + step, &heading)) return heading;
    // (standing still: the way it went as it came to the last key before, else the way it will go from the first after)
    std::vector<double> keys = doc_key_moments(d, thing, "x"), more = doc_key_moments(d, thing, "y");
    keys.insert(keys.end(), more.begin(), more.end());
    std::sort(keys.begin(), keys.end());
    for (auto k = keys.rbegin(); k != keys.rend(); ++k)
        if (*k <= now && going(*k - 1.0 / 60.0, *k, &heading)) return heading;
    for (const double k : keys)
        if (k >= now && going(k, k + 1.0 / 60.0, &heading)) return heading;
    return 0.0;
}
