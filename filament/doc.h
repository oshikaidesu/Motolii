// doc.h: a document of the stage, and the editing of it as calls: what a shell on a screen would call, and nothing
// else. The document is a file of facts: a thing's value, by name, at a time (thing, name, at, value). A value with one
// fact stays as it is; with more it moves, and they are its keys. One call = one change = one step of undo (Lumit's
// bridge is so), and the stage shows it as the call returns. A change the stage cannot show is not made.
// Keys, Motolii's own way: a key holds a value, and THE STRETCH FROM A KEY TO THE NEXT IS ITSELF DEFINED: how the value
// goes there. The kinds of stretch are Motolii's own nine, brought from the past repository as they are (interp.h:
// Hold, Linear, Bezier, Bounce, Elastic, Cyclic, Random, Steps, ElasticSteps, each with its numbers). A way of one's
// own can be added by an expression kept in the document (worked out by SQLite's own expressions: u is how far
// through the stretch, 0 to 1; t the seconds into it; d its length; a and b the two keys' values; p1 to p4 numbers
// said with it), taken only if it begins at the one key's value and ends at the other's.
// Borrowed whole: the file, saving (a change is in the file as its call returns) and undo are SQLite's (its session
// extension: what a call changed is kept, and undone by applying it reversed: the steps are in the file too). A
// thing's time is told in GES's words (start, duration).
// The names a thing's facts go by are ThorVG's own, where it has one (translate's x and y, rotate, size, blend and its
// words, text, fill, a segment's begin, add, remove):
//   numbers  x, y, z, rotate, size  where its middle is, turned by how many degrees, how tall
//            rotate_x, rotate_y     turned about its own x and y too (degrees): a thing leaning back, a thing turned to its side
//            opacity                how much of it is there: 1 all, 0 none (a film's, a picture's, a text's)
//            start, duration        when it comes on, and for how long (not said: for good, going round)
//            begin, speed           a film's: from where in itself, how fast (below 0: backwards)
//            order                  where it lies among the things: one with a greater number is nearer (not said: as they were made)
//            visible                0: it is put aside, not seen (ThorVG's word), though it is still in the document
//            clip                   1: it is seen only inside the frame of the thing before it (a 3D thing cut by a rectangle)
//            wide                   a shape's: how many times as wide as it is high
//            keyed                  a film's: 1 if its ground is keyed out (a green screen, of whatever colour: the colour its four corners share), 0 if not; not said: keyed if it has such a ground
//   words    blend                  how it lies on what is behind: normal | add | multiply | screen
//            file, shape            what it was made from (add says them): a file (a text's: its font); or, a solid's, one of the kit's shapes: ring | gem | cube;
//                                   or, a shape's (a flat one, drawn by ThorVG), rect | circle
//            text                   a text's: what it says (to the end of the line)
//            fill                   what its surface is: a text's colour as six digits (ff3366); a solid's: a colour, or the material glass
#include "drawn.h"
#include "interp.h"
#include "solid.h"
#include "video.h"
#include <sstream>
#include <sqlite3.h>

struct Doc {
    sqlite3* file = nullptr;
    Stage* stage = nullptr;
    std::string trouble;   // why the last call that was not done was not
    std::vector<int> laid;   // the things in the order they lie, the one furthest back first (read when first asked for, forgotten at every change)
    // One act of a hand: the changes made from its beginning to its end are one step of undo, and if one of them is
    // refused none of them is made. Acts may lie one within another: the outermost one is the step.
    bool keying = false;                // every value a hand changes gets a key at that moment (a shell's "animate all")
    double keying_from = 0.0;           // the stage's moment at which that was turned on: a value's first key then has the value it had, there, before it
    int act = 0;                        // how many acts are open, one within another
    bool act_ok = true;                 // no change of the open act has been refused
    bool writing = false;               // it is being written as a film: it is read, not changed, until that is done
    sqlite3_session* acting = nullptr;  // SQLite's own record of what the open act changes
    // A thing held by a hand (doc_hold): what of it is carried shows at once and is not yet the document's.
    struct Held { int thing = 0; std::map<std::string, double> from, tried; } held;
    struct Keys { std::vector<double> at, value; std::vector<Interp> interp; std::vector<std::string> own; };   // (own: a way defined in the document, where the stretch is not one of the nine kinds)
    std::map<std::pair<int, std::string>, Keys> moving;   // a value's keys, and the way of the stretch after each; read from the facts when first asked for, forgotten at every change
    std::map<std::string, sqlite3_stmt*> named, goes;     // a way by its name, and by its expression: made ready once (SQLite works it out)
    struct Shown { std::string of; Video* film = nullptr; Drawn* drawn = nullptr; Solid* solid = nullptr; Mesh* shape = nullptr; bool in = true, clipped = false; int run = -1, lies = -1; };
    std::map<int, Shown> shown;   // the things as they are on the stage, and what each was made of
    Mesh* clears[8] = {};         // for each run of things, the triangle that clears depth before it (clear_depth.mat), made when first needed
    unsigned cleared = 0;         // which of them are on the stage now
};
// What a thing may be told, by name: which kinds of thing have it, a number between which two (or one of which
// words), and whether it can move (take keys).
static const char* const doc_kinds = "film picture text solid shape";   // (the kinds of thing)
static const char* const doc_shapes = "ring gem cube";             // (the kit's own shapes a solid may be)
// (whether a value is keyed, in SQLite's words: see doc_key)
#define DOC_KEYED "(select count(*) > 1 or max(at) <> 0 or max(way) is not null from fact where thing = %d and name = %Q)"
struct DocValue { const char* name; const char* kinds; double low, high; bool moves; const char* words; };
static const DocValue doc_values[] = {
    {"x", doc_kinds, -1e4, 1e4, true, nullptr}, {"y", doc_kinds, -1e4, 1e4, true, nullptr}, {"z", doc_kinds, -1e4, 1e4, true, nullptr},
    {"rotate", doc_kinds, -1e6, 1e6, true, nullptr}, {"rotate_x", doc_kinds, -1e6, 1e6, true, nullptr}, {"rotate_y", doc_kinds, -1e6, 1e6, true, nullptr},
    {"size", doc_kinds, 1e-3, 1e4, true, nullptr}, {"anchor_x", doc_kinds, -100, 100, true, nullptr}, {"anchor_y", doc_kinds, -100, 100, true, nullptr}, {"anchor_z", doc_kinds, -100, 100, true, nullptr}, {"opacity", "film picture text shape", 0, 1, true, nullptr},
    {"clip", doc_kinds, 0, 1, false, nullptr}, {"visible", doc_kinds, 0, 1, false, nullptr}, {"order", doc_kinds, -1e9, 1e9, false, nullptr}, {"wide", "shape", 1e-3, 1e3, false, nullptr}, {"shape", "shape", 0, 0, false, "rect circle solid"},
    {"start", doc_kinds, 0, 1e6, false, nullptr}, {"duration", doc_kinds, 1e-3, 1e6, false, nullptr},
    {"begin", "film", 0, 1e6, false, nullptr}, {"speed", "film", -100, 100, false, nullptr}, {"keyed", "film", 0, 1, false, nullptr},
    {"blend", "film picture text shape", 0, 0, false, "normal add multiply screen"}, {"file", "film picture solid text", 0, 0, false, ""}, {"shape", "solid", 0, 0, false, doc_shapes},
    {"text", "text", 0, 0, false, ""}, {"fill", "text solid shape", 0, 0, false, ""},
    {"parent", doc_kinds, 0, 1e9, false, nullptr},   // (the thing it goes with: it is moved, turned and sized as that one is, then as it is itself; 0: none)
    {"align", "text", 0, 1, false, nullptr},
    // (ThorVG's own scene effects on a thing it draws: a blur, and a shadow of a colour, how far it falls, which way, how
    //  soft, how much of it; lengths in points of the picture at size 1)
    {"blur", "picture text shape", 0, 1000, false, nullptr}, {"shadow", "picture text shape", 0, 0, false, ""},
    {"shadow_distance", "picture text shape", 0, 1000, false, nullptr}, {"shadow_angle", "picture text shape", -1e6, 1e6, false, nullptr},
    {"shadow_blur", "picture text shape", 0, 1000, false, nullptr}, {"shadow_opacity", "picture text shape", 0, 1, false, nullptr},
    // (and its darks and lights made two colours, how much: ThorVG's tint; or its darks, middles and lights three: its tritone)
    {"tint", "picture text shape", 0, 1, false, nullptr}, {"tint_black", "picture text shape", 0, 0, false, ""}, {"tint_white", "picture text shape", 0, 0, false, ""},
    {"tritone", "picture text shape", 0, 1, false, nullptr}, {"tritone_shadow", "picture text shape", 0, 0, false, ""}, {"tritone_midtone", "picture text shape", 0, 0, false, ""}, {"tritone_highlight", "picture text shape", 0, 0, false, ""},   // (how its lines lie one under another, ThorVG's own: 0 at the left, 0.5 about the middle, 1 at the right)
    {"stroke", "text shape", 0, 0, false, ""}, {"stroke_width", "text shape", 0, 1000, false, nullptr}};   // (its edge: a colour, and how wide, in points of the picture at size 1)
static bool doc_among(const std::string& words, const std::string& one) { return (" " + words + " ").find(" " + one + " ") != std::string::npos; }

// Rows of the document: `each` is told every row of what is asked (in SQLite's own words).
static bool doc_rows(Doc& d, char* ask, const std::function<void(sqlite3_stmt*)>& each = nullptr) {
    sqlite3_stmt* rows = nullptr;
    bool ok = sqlite3_prepare_v2(d.file, ask, -1, &rows, nullptr) == SQLITE_OK;
    while (ok && sqlite3_step(rows) == SQLITE_ROW)
        if (each) each(rows);
    sqlite3_finalize(rows);
    sqlite3_free(ask);
    return ok;
}
static void doc_forget(Doc& d) {
    d.moving.clear();
    d.named.clear();
    d.laid.clear();
}
// The things in the order they lie, the one furthest back first: by `order` where it is said, else as they were made.
static const std::vector<int>& doc_laid(Doc& d) {
    if (d.laid.empty())
        doc_rows(d, sqlite3_mprintf("select f.thing from fact f where f.name = 'kind' order by coalesce((select o.value from fact o where o.thing = f.thing and o.name = 'order' and o.at = 0), f.thing), f.thing"),
                 [&](sqlite3_stmt* row) { d.laid.push_back(sqlite3_column_int(row, 0)); });
    return d.laid;
}

// A way's expression made ready for SQLite to work out (nullptr: SQLite does not take it; d.trouble says why).
static sqlite3_stmt* doc_goes(Doc& d, const std::string& expression) {
    auto ready = d.goes.find(expression);
    if (ready != d.goes.end()) return ready->second;
    sqlite3_stmt* made = nullptr;
    char* asked = sqlite3_mprintf("with v(u, t, d, a, b, p1, p2, p3, p4) as (select ?, ?, ?, ?, ?, ?, ?, ?, ?) select (%s) from v", expression.c_str());
    if (sqlite3_prepare_v2(d.file, asked, -1, &made, nullptr) != SQLITE_OK) d.trouble = sqlite3_errmsg(d.file), made = nullptr;
    sqlite3_free(asked);
    return made ? d.goes[expression] = made : nullptr;
}
static double doc_went(sqlite3_stmt* way, double u, double t, double length, double a, double b, const double* p) {
    const double told[] = {u, t, length, a, b, p[0], p[1], p[2], p[3]};
    for (int i = 0; i < 9; ++i) sqlite3_bind_double(way, i + 1, told[i]);
    const double is = sqlite3_step(way) == SQLITE_ROW ? sqlite3_column_double(way, 0) : a;
    sqlite3_reset(way);
    return is;
}
static sqlite3_stmt* doc_way_named(Doc& d, const std::string& name) {
    auto known = d.named.find(name);
    if (known != d.named.end()) return known->second;
    sqlite3_stmt* way = nullptr;
    doc_rows(d, sqlite3_mprintf("select goes from way where name = %Q", name.c_str()), [&](sqlite3_stmt* row) { way = doc_goes(d, (const char*)sqlite3_column_text(row, 0)); });
    return d.named[name] = way;
}
// A stretch that goes a way defined in the document (`own`: its name, then its numbers): the value at `u` of the way
// through, `t` seconds in, between a key of `a` and a key of `b`, `length` seconds apart (no such way: `a`).
static double doc_went_by(Doc& d, const std::string& own, double u, double t, double length, double a, double b) {
    std::istringstream said(own);
    std::string way;
    double p[4] = {0, 0, 0, 0};
    said >> way;
    for (double& one : p) said >> one;
    sqlite3_stmt* goes = doc_way_named(d, way);
    return goes ? doc_went(goes, u, t, length, a, b, p) : a;
}

// A value at a moment: `otherwise` where the document does not say; where it has keys, the value the stretch it is in
// gives (before the first key and after the last: theirs). A stretch goes the way said for it (not said: Linear).
static double doc_number(Doc& d, int thing, const char* name, double at, double otherwise) {
    if (d.held.thing == thing)   // (what a hand is carrying shows in place of the document's own)
        if (const auto tried = d.held.tried.find(name); tried != d.held.tried.end()) return tried->second;
    auto found = d.moving.find({thing, name});
    if (found == d.moving.end()) {
        Doc::Keys keys;
        doc_rows(d, sqlite3_mprintf("select at, value, coalesce(way, '') from fact where thing = %d and name = %Q order by at", thing, name), [&](sqlite3_stmt* row) {
            keys.at.push_back(sqlite3_column_double(row, 0));
            keys.value.push_back(sqlite3_column_double(row, 1));
            const std::string way = (const char*)sqlite3_column_text(row, 2);
            Interp kind;
            const bool of_the_nine = interp_said(way.empty() ? "Linear" : way, &kind) == 0;   // (not said: Linear, as a new key is in the past repository)
            keys.interp.push_back(kind);
            keys.own.push_back(of_the_nine ? "" : way);
        });
        found = d.moving.emplace(std::make_pair(thing, std::string(name)), keys).first;
    }
    const Doc::Keys& k = found->second;
    if (k.at.empty()) return otherwise;
    if (k.at.size() == 1 || at <= k.at.front()) return k.value.front();
    if (at >= k.at.back()) return k.value.back();
    const size_t i = (size_t)(std::upper_bound(k.at.begin(), k.at.end(), at) - k.at.begin()) - 1;   // (the stretch it is in begins at key i)
    const double u = (at - k.at[i]) / (k.at[i + 1] - k.at[i]);
    if (k.own[i].empty()) return k.value[i] + (k.value[i + 1] - k.value[i]) * interp_ease(k.interp[i], u);
    return doc_went_by(d, k.own[i], u, at - k.at[i], k.at[i + 1] - k.at[i], k.value[i], k.value[i + 1]);
}
static std::string doc_word(Doc& d, int thing, const char* name) {
    std::string is;
    doc_rows(d, sqlite3_mprintf("select value from fact where thing = %d and name = %Q", thing, name), [&](sqlite3_stmt* row) { is = (const char*)sqlite3_column_text(row, 0); });
    return is;
}

// Every look of a thing as it is on the stage (a sheet's one; a kit shape's, as many as it is drawn by; every face of a 3D file).
static void doc_looks(Stage& s, const Doc::Shown& t, const std::function<void(MaterialInstance*)>& each) {
    auto& faces = s.engine->getRenderableManager();
    if (t.film) each(t.film->look);
    if (t.drawn) each(t.drawn->look);
    for (const Stage::Made& made : s.made)
        if (t.shape && made.of == t.shape) each(faces.getMaterialInstanceAt(faces.getInstance(made.thing), 0));
    for (size_t i = 0; t.solid && i < t.solid->made->getRenderableEntityCount(); ++i) {
        const auto one = faces.getInstance(t.solid->made->getRenderableEntities()[i]);
        for (size_t part = 0; part < faces.getPrimitiveCount(one); ++part) each(faces.getMaterialInstanceAt(one, part));
    }
}
// Each drawn part of a thing (Filament's renderables).
static void doc_parts(Stage& s, const Doc::Shown& t, const std::function<void(RenderableManager&, RenderableManager::Instance)>& each) {
    auto& faces = s.engine->getRenderableManager();
    const Mesh* m = t.film ? t.film->sheet : t.drawn ? t.drawn->sheet : t.shape;
    for (const Stage::Made& made : s.made)
        if (m && made.of == m) each(faces, faces.getInstance(made.thing));
    for (size_t i = 0; t.solid && i < t.solid->made->getRenderableEntityCount(); ++i) each(faces, faces.getInstance(t.solid->made->getRenderableEntities()[i]));
}
static bool doc_covers(Doc& d, int thing, double now, double* box);
static bool doc_half(Doc& d, int thing, float2* half, float* deep = nullptr);
// Where a thing is at a moment, as a matrix: moved, turned about z, y and x, made its size, all about its anchor (a
// point of its own box: 0 its middle, 1 its edge on that side; the past repository's `anchor`, keyed as it was).
static math::mat4 doc_place(Doc& d, int thing, double now, int deeper = 0);
// Where the thing it goes with is (its parent's place; none, or one not there any more: as the stage is).
static math::mat4 doc_above(Doc& d, int thing, double now, int deeper = 0) {
    const int parent = (int)doc_number(d, thing, "parent", 0, 0);
    return parent && d.shown.count(parent) && deeper < 64 ? doc_place(d, parent, now, deeper + 1) : math::mat4{};   // (64: a document written elsewhere with a ring of parents still ends)
}
static math::mat4 doc_place(Doc& d, int thing, double now, int deeper) {
    const auto turned = [&](const char* name, math::double3 about) { return math::mat4::rotation(doc_number(d, thing, name, now, 0) * (double)TAU / 360.0, about); };
    float2 half{0.5f};
    float deep = 0.f;
    doc_half(d, thing, &half, &deep);
    const math::double3 anchor{doc_number(d, thing, "anchor_x", now, 0) * half.x, doc_number(d, thing, "anchor_y", now, 0) * half.y, doc_number(d, thing, "anchor_z", now, 0) * deep};
    return doc_above(d, thing, now, deeper) * math::mat4::translation(math::double3{doc_number(d, thing, "x", now, 0), doc_number(d, thing, "y", now, 0), doc_number(d, thing, "z", now, 0)}) *
           turned("rotate", {0, 0, 1}) * turned("rotate_y", {0, 1, 0}) * turned("rotate_x", {1, 0, 0}) * math::mat4::scaling(doc_number(d, thing, "size", now, 1)) * math::mat4::translation(-anchor);
}

// Every thing told its moment: whether it is on (from its start, for its duration if it has one), what of itself it
// shows (it goes round when it comes to its end), and where it is.
static void doc_moment(Doc& d, double now) {
    Stage& s = *d.stage;
    const std::vector<int> lie = doc_laid(d);
    // Each thing as it is now, in the order they lie: whether it is on, and whether it is a solid thing.
    struct Lies { int thing; Doc::Shown* shown; bool on, solid; double since; };
    std::vector<Lies> lies;
    for (int thing : lie) {
        const auto found = d.shown.find(thing);
        if (found == d.shown.end()) continue;
        const double since = now - doc_number(d, thing, "start", 0, 0), lasts = doc_number(d, thing, "duration", 0, -1);
        Lies one{thing, &found->second, since >= 0.0 && (lasts < 0.0 || since < lasts) && doc_number(d, thing, "visible", 0, 1) > 0.5, found->second.solid || found->second.shape, since};
        lies.push_back(one);
    }
    // Who is before whom (the past repository's law of 2026-09-12: a flat thing is not in the world). Flat things take
    // no part in depth and lie by the order of the things alone; solid things that follow one another settle depth
    // among themselves; a run of the one sort is drawn whole after the run before it. By Filament's own means: a run
    // is a channel (it has 8: past the eighth change of sort the rest are one run), the order within a run of flat
    // things is the blend order, and a flat thing's looks neither ask nor write depth.
    // A run of solid things that is not the first run has depth cleared before it (the triangle is the first drawn in
    // its channel), so it is over all that was drawn before, whatever depth that had.
    int run = 0, sort = -1;
    unsigned clear = 0;
    for (size_t i = 0; i < lies.size(); ++i) {
        if (!lies[i].on) continue;
        if (sort >= 0 && sort != (int)lies[i].solid && run < 7) ++run;
        sort = lies[i].solid;
        if (lies[i].solid && run > 0) clear |= 1u << run;
        Doc::Shown& t = *lies[i].shown;
        if (t.run == run && t.lies == (int)i) continue;
        if (t.run < 0 && !lies[i].solid) doc_looks(s, t, [](MaterialInstance* look) { look->setDepthCulling(false), look->setDepthWrite(false); });
        doc_parts(s, t, [&](RenderableManager& faces, RenderableManager::Instance one) {
            faces.setChannel(one, (uint8_t)run);
            for (size_t part = 0; !lies[i].solid && part < faces.getPrimitiveCount(one); ++part) faces.setBlendOrderAt(one, part, (uint16_t)i), faces.setGlobalBlendOrderEnabledAt(one, part, true);
        });
        t.run = run, t.lies = (int)i;
    }
    for (int k = 1; k < 8 && clear != d.cleared; ++k) {
        const bool want = clear >> k & 1u;
        if (want == (bool)(d.cleared >> k & 1u)) continue;
        if (!d.clears[k]) {
            d.clears[k] = new Mesh{{float3{-1, -1, 1}, float3{3, -1, 1}, float3{-1, 3, 1}}, std::vector<float3>(3, float3{0, 0, 1}), std::vector<float2>(3, float2{0.f}), {0, 1, 2}, {}};   // (Filament's own full-screen triangle)
            add(s, d.clears[k], "clear_depth.filamat", {mat4f{}}, 1, false, 1, 0);   // (order 0: before all else in its channel)
            for (const Stage::Made& made : s.made) if (made.of == d.clears[k]) s.engine->getRenderableManager().setChannel(s.engine->getRenderableManager().getInstance(made.thing), (uint8_t)k);
        }
        seen(s, d.clears[k], want);
    }
    d.cleared = clear;
    for (size_t i = 0; i < lies.size(); ++i) {
        const int thing = lies[i].thing;
        Doc::Shown& t = *lies[i].shown;
        const bool in = lies[i].on;
        const Mesh* m = t.film ? t.film->sheet : t.drawn ? t.drawn->sheet : t.shape;
        if (in != t.in) { if (t.solid) solid_seen(s, t.solid, in); else seen(s, m, in); t.in = in; }
        if (!in) continue;
        if (t.film) video_at(s, t.film, (float)lies[i].since);
        if (t.drawn && t.drawn->seconds > 0.f) picture_at(s, t.drawn, std::fmod((float)lies[i].since, t.drawn->seconds));
        const mat4f place = mat4f(doc_place(d, thing, now));
        if (t.solid) solid_to(s, t.solid, place); else move(s, m, [&](int, const mat4f&) { return place; });
        if (MaterialInstance* look = t.film ? t.film->look : t.drawn ? t.drawn->look : nullptr) look->setParameter("opacity", (float)doc_number(d, thing, "opacity", now, 1));
    }
    // A clipped thing is seen only inside the frame of the thing before it: Filament's own scissor on its looks, told
    // the rectangle of the picture that the thing before it covers now (turned or leaning: the upright box around it).
    int before = 0;
    for (int thing : lie) {
        const auto found = d.shown.find(thing);
        if (found == d.shown.end()) continue;
        Doc::Shown& t = found->second;
        double box[4] = {0, 0, 0, 0};
        const bool clipped = t.in && before && doc_number(d, thing, "clip", 0, 0) > 0.5 && doc_covers(d, before, now, box);
        if (clipped || t.clipped) {
            const uint32_t left = (uint32_t)std::clamp(box[0], 0.0, (double)s.w), bottom = (uint32_t)std::clamp((double)s.h - (box[1] + box[3]), 0.0, (double)s.h);
            doc_looks(s, t, [&](MaterialInstance* look) { if (clipped) look->setScissor(left, bottom, (uint32_t)std::lround(std::max(box[2], 0.0)), (uint32_t)std::lround(std::max(box[3], 0.0))); else look->unsetScissor(); });
            t.clipped = clipped;
        }
        if (t.in) before = thing;
    }
}

// The stage made to match the document: a thing the document has and the stage has not is made (by the kit's own
// call for its kind), one the stage has and the document has not is taken off, one whose making is told otherwise now
// is made again. false: a thing could not be made (d.trouble says which, and why).
static bool doc_match(Doc& d) {
    Stage& s = *d.stage;
    std::map<int, std::string> told;   // every thing of the document, and all that its making goes by
    doc_rows(d, sqlite3_mprintf("select thing, group_concat(name || '=' || value, ' ') from (select * from fact where thing > 0 and name in ('kind','file','shape','blend','keyed','begin','duration','speed','text','fill','wide','stroke','stroke_width','align','blur','shadow','shadow_distance','shadow_angle','shadow_blur','shadow_opacity','tint','tint_black','tint_white','tritone','tritone_shadow','tritone_midtone','tritone_highlight') order by thing, name) group by thing"),
             [&](sqlite3_stmt* row) { told[sqlite3_column_int(row, 0)] = (const char*)sqlite3_column_text(row, 1); });
    for (auto it = d.shown.begin(); it != d.shown.end();) {
        const auto still = told.find(it->first);
        if (still != told.end() && still->second == it->second.of) { ++it; continue; }
        if (it->second.film) video_off(s, it->second.film);
        if (it->second.drawn) picture_off(s, it->second.drawn);
        if (it->second.solid) solid_off(s, it->second.solid);
        if (it->second.shape) take_off(s, it->second.shape);
        it = d.shown.erase(it);
    }
    bool all = true;
    for (const auto& [thing, of] : told) {
        if (d.shown.count(thing)) continue;
        Doc::Shown made{of};
        const std::string kind = doc_word(d, thing, "kind"), file = doc_word(d, thing, "file"), laid = doc_word(d, thing, "blend"), shape = doc_word(d, thing, "shape");
        const char* lay = laid.empty() || laid == "normal" ? nullptr : laid.c_str();
        std::string why = "it could not be read";
        const std::string fill = doc_word(d, thing, "fill");
        const long rgb = fill.size() == 6 && fill.find_first_not_of("0123456789abcdefABCDEF") == std::string::npos ? std::strtol(fill.c_str(), nullptr, 16) : -1;
        const float3 colour = float3{(float)((rgb >> 16) & 255), (float)((rgb >> 8) & 255), (float)(rgb & 255)} / 255.f;
        // (an edge, ThorVG's stroke: its colour said as a fill is, and its width as a share of the thing's own height,
        //  from points of the picture at size 1 where things stand at first)
        const std::string stroke = doc_word(d, thing, "stroke");
        const long edgeRgb = stroke.size() == 6 && stroke.find_first_not_of("0123456789abcdefABCDEF") == std::string::npos ? std::strtol(stroke.c_str(), nullptr, 16) : -1;
        const math::double4 up = s.camera->getProjectionMatrix() * s.camera->getViewMatrix() * math::double4{0, 1, 0, 1};
        const float4 edge = edgeRgb < 0 ? float4{0.f} : float4{float3{(float)((edgeRgb >> 16) & 255), (float)((edgeRgb >> 8) & 255), (float)(edgeRgb & 255)} / 255.f, (float)(doc_number(d, thing, "stroke_width", 0, 0) / (0.5 * s.h * up.y / up.w))};
        if (!stroke.empty() && edgeRgb < 0) why = "a stroke is six digits, as ff3366";
        const double units = 0.5 * s.h * up.y / up.w;   // (points of the picture to one unit of the stage, where things stand at first)
        const std::string shade = doc_word(d, thing, "shadow");
        const long shadeRgb = shade.size() == 6 && shade.find_first_not_of("0123456789abcdefABCDEF") == std::string::npos ? std::strtol(shade.c_str(), nullptr, 16) : -1;
        DrawnFx fx;
        fx.blur = (float)(doc_number(d, thing, "blur", 0, 0) / units);
        if (shadeRgb >= 0) {   // (After Effects' own drop shadow, as it comes: half of it, 135 degrees, 10 points away, 10 soft)
            fx.shadow = float4{float3{(float)((shadeRgb >> 16) & 255), (float)((shadeRgb >> 8) & 255), (float)(shadeRgb & 255)} / 255.f, (float)doc_number(d, thing, "shadow_opacity", 0, 0.5)};
            fx.fall = (float)(doc_number(d, thing, "shadow_distance", 0, 10) / units), fx.way = (float)doc_number(d, thing, "shadow_angle", 0, 135), fx.soft = (float)(doc_number(d, thing, "shadow_blur", 0, 10) / units);
        } else if (!shade.empty()) why = "a shadow is six digits, as 000000";
        bool fxOk = shade.empty() || shadeRgb >= 0;
        const auto hue = [&](const char* name, const char* otherwise, float3* to) {   // (a colour said as six digits; not said: as it comes)
            std::string said = doc_word(d, thing, name);
            if (said.empty()) said = otherwise;
            const long rgb = said.size() == 6 && said.find_first_not_of("0123456789abcdefABCDEF") == std::string::npos ? std::strtol(said.c_str(), nullptr, 16) : -1;
            if (rgb < 0) return fxOk = false, why = std::string("a ") + name + " is six digits, as ff3366", void();
            *to = float3{(float)((rgb >> 16) & 255), (float)((rgb >> 8) & 255), (float)(rgb & 255)} / 255.f;
        };
        if ((fx.tint = (float)doc_number(d, thing, "tint", 0, 0)) > 0.f) hue("tint_black", "1b1f5c", &fx.tints[0]), hue("tint_white", "ff8a65", &fx.tints[1]);   // (a deep blue and a warm pink, as it comes)
        if ((fx.tritone = (float)doc_number(d, thing, "tritone", 0, 0)) > 0.f) hue("tritone_shadow", "1b1f5c", &fx.tones[0]), hue("tritone_midtone", "c44569", &fx.tones[1]), hue("tritone_highlight", "ffd166", &fx.tones[2]);
        if (shape.empty() && !std::filesystem::is_regular_file(file)) why = "there is no such file";
        else if (kind == "film") {
            const double from = doc_number(d, thing, "begin", 0, 0), length = doc_number(d, thing, "duration", 0, -1), speed = doc_number(d, thing, "speed", 0, 1);
            made.film = video(s, file.c_str(), mat4f{}, 1.f, (float)from, length < 0 ? -1.f : (float)(from + length * std::fabs(speed)), (float)speed, lay);
            if (!made.film) why = video_trouble;
            else if (const double keyed = doc_number(d, thing, "keyed", 0, -1); keyed != 0.0) key_own(made.film, keyed > 0.5);   // (not said: keyed if it has a ground of its own)
            if (made.film) s.playing.erase(made.film);   // (the document tells it its moment, and when its sound comes on)
            if (made.film && made.film->sound) s.sounding[made.film] = [&d, thing, v = made.film, file] { return Stage::Sound{file, v->from, v->to, v->speed, 0.0, doc_number(d, thing, "start", 0, 0), doc_number(d, thing, "duration", 0, -1)}; };
        } else if (kind == "picture") {
            if (fxOk && (made.drawn = picture(s, file.c_str(), mat4f{}, 1.f, float3{-1.f}, true, lay, fx))) s.playing.erase(made.drawn);
        } else if (kind == "text") {   // (words in a font: said by `say N text ...`; until then, the font's own name)
            const std::string words = doc_word(d, thing, "text");
            if (!fill.empty() && rgb < 0) why = "a text's fill is six digits, as ff3366";
            else if ((stroke.empty() || edgeRgb >= 0) && fxOk) made.drawn = type(s, words.empty() ? std::filesystem::path(file).stem().string().c_str() : words.c_str(), file.c_str(), mat4f{}, 1.f, fill.empty() ? float3{0x11 / 255.f} : colour, lay, edge, (float)doc_number(d, thing, "align", 0, 0), fx);
        } else if (kind == "shape") {   // (a flat shape in one colour: ThorVG's own rectangle or circle)
            if (!fill.empty() && rgb < 0) why = "a shape's fill is six digits, as ff3366";
            else if ((!stroke.empty() && edgeRgb < 0) || !fxOk) {}
            else if (shape == "solid") {   // (a flat colour as large as the stage, where it stands at first: as wide to its height, and as high in the stage's units)
                const float high = (float)(2.0 * up.w / up.y);
                made.drawn = flat(s, "rect", (float)s.w / (float)s.h, mat4f{}, high, fill.empty() ? float3{0xd9, 0xd2, 0xc3} / 255.f : colour, lay, float4{edge.xyz, edge.w / high}, DrawnFx{fx.blur / high, fx.shadow, fx.fall / high, fx.way, fx.soft / high, fx.tint, {fx.tints[0], fx.tints[1]}, fx.tritone, {fx.tones[0], fx.tones[1], fx.tones[2]}});
            } else made.drawn = flat(s, shape.c_str(), (float)doc_number(d, thing, "wide", 0, 1), mat4f{}, 1.f, fill.empty() ? float3{0xd9, 0xd2, 0xc3} / 255.f : colour, lay, edge, fx);
        } else if (kind == "solid") {   // (one of the kit's shapes, or a file; its fill is its surface: a colour, or the material glass)
            const bool glass = fill == "glass";
            if (!fill.empty() && !glass && rgb < 0) why = "a solid's fill is six digits, as ff3366, or glass";
            else if (!shape.empty()) {
                made.shape = shape == "gem" ? gem(0.5f) : shape == "cube" ? cube(0.1f) : torus(0.37f, 0.13f);
                MaterialInstance* look = glass ? thick_glass(s, made.shape, {mat4f{}}) : add(s, made.shape, "floor_lit.filamat");
                if (!look) made.shape = nullptr;
                else if (!glass) plain(look, fill.empty() ? float3{0xd9, 0xd2, 0xc3} / 255.f : colour);
            } else if (glass) {   // (a file's shape as glass: its triangles alone, for the kit's glass, which draws a shape three times)
                if ((made.shape = solid_mesh(file.c_str())) && !thick_glass(s, made.shape, {mat4f{}})) made.shape = nullptr;
            }
            else if ((made.solid = solid(s, file.c_str(), mat4f{}, 1.f)) && rgb >= 0) solid_fill(s, made.solid, colour);
        } else why = std::string("its kind is one of: ") + doc_kinds;
        if (!made.film && !made.drawn && !made.solid && !made.shape) { d.trouble = why + ": the " + kind + " " + std::filesystem::path(file + shape).filename().string() + " could not be made"; all = false; continue; }
        d.shown[thing] = made;
    }
    doc_moment(d, s.now);
    return all;
}

// How long the document is: to the end of what ends last (a thing with no duration: once through itself); 8 seconds if nothing in it ends.
static double doc_seconds(Doc& d) {
    double all = 0.0;
    for (const auto& [thing, t] : d.shown) {
        const double own = t.film ? (double)t.film->seconds : t.drawn ? (double)t.drawn->seconds : t.solid ? (double)t.solid->seconds : 0.0, lasts = doc_number(d, thing, "duration", 0, own);
        if (lasts > 0.0) all = std::max(all, doc_number(d, thing, "start", 0, 0) + lasts);
    }
    return all > 0.0 ? all : 8.0;
}

// A document's own stage: how many points wide and high it is (facts of thing 0, which is the stage itself; they are
// not steps: a stage is opened at a size, so a new size is the document's when it is next opened).
// doc_size: read before a stage is opened for the document (false: it does not say, or it is not there yet).
static bool doc_size(const char* path, int* wide, int* high) {
    sqlite3* file = nullptr;
    sqlite3_stmt* rows = nullptr;
    int w = 0, h = 0;
    if (sqlite3_open_v2(path, &file, SQLITE_OPEN_READONLY, nullptr) == SQLITE_OK && sqlite3_prepare_v2(file, "select name, value from fact where thing = 0 and name in ('wide', 'high')", -1, &rows, nullptr) == SQLITE_OK)
        while (sqlite3_step(rows) == SQLITE_ROW) (strcmp((const char*)sqlite3_column_text(rows, 0), "wide") ? h : w) = sqlite3_column_int(rows, 1);
    sqlite3_finalize(rows);
    sqlite3_close(file);
    return w > 0 && h > 0 ? (*wide = w, *high = h, true) : false;
}
// doc_size_words: the change that says it (nullptr: a stage cannot be that size). doc_size_is: a new document told
// the size it was first opened at (no step of undo: there is nothing before it).
static char* doc_size_words(int wide, int high) {
    if (wide < 16 || high < 16 || wide > 8192 || high > 8192 || wide % 2 || high % 2) return nullptr;
    return sqlite3_mprintf("insert into fact(thing, name, at, value) values(0, 'wide', 0.0, %d), (0, 'high', 0.0, %d) on conflict do update set value = excluded.value", wide, high);
}
static bool doc_size_is(sqlite3* file, int wide, int high) {
    char* say = doc_size_words(wide, high);
    const bool ok = say && sqlite3_exec(file, say, nullptr, nullptr, nullptr) == SQLITE_OK;
    sqlite3_free(say);
    return ok;
}

// The document in `path` (made if it is not there), on the stage. nullptr: the file is not a document.
static Doc* doc_open(Stage& s, const char* path) {
    Doc* d = new Doc;
    if (sqlite3_open(path, &d->file) != SQLITE_OK) return nullptr;
    sqlite3_busy_timeout(d->file, 2000);   // (another program at the same document is waited for)
    // (a key's `way` is how the stretch from it to the next key goes; `way` the table holds the ways defined in this document)
    if (sqlite3_exec(d->file, "create table if not exists fact(thing integer, name text, at real, value, way text, primary key(thing, name, at)) without rowid;"
                              "create table if not exists way(name text primary key, goes text) without rowid;"
                              "create table if not exists step(n integer primary key, change blob, undone integer default 0);"
                              "create table if not exists thing(id integer primary key autoincrement);"   // (a thing's number is never given again: SQLite's own)
                              "insert into thing(id) select max(thing) from fact where (select count(*) from thing) = 0 having max(thing) is not null", nullptr, nullptr, nullptr) != SQLITE_OK) return nullptr;
    if (sqlite3_exec(d->file, "select way from fact limit 0", nullptr, nullptr, nullptr) != SQLITE_OK)   // (a document from before stretches had ways)
        sqlite3_exec(d->file, "alter table fact add column way text", nullptr, nullptr, nullptr);
    d->stage = &s;
    s.playing[d] = [d](double now) { doc_moment(*d, now); };
    if (!doc_match(*d)) std::printf("%s\n", d->trouble.c_str());
    return d;
}

// One change to the document (in SQLite's own words), kept as one step of undo; the stage shows it at once. If the
// stage cannot show it, it is not made: the document and the stage stay as they were.
static void doc_act(Doc& d) {
    if (d.act++) return;
    sqlite3_exec(d.file, "begin", nullptr, nullptr, nullptr);
    sqlite3session_create(d.file, "main", &d.acting);
    sqlite3session_attach(d.acting, "fact");
    sqlite3session_attach(d.acting, "way");
    d.act_ok = true;
}
static bool doc_acted(Doc& d) {
    if (--d.act) return d.act_ok;
    int size = 0;
    void* changed = nullptr;
    sqlite3session_changeset(d.acting, &size, &changed);
    sqlite3session_delete(d.acting);
    d.acting = nullptr;
    if (d.act_ok && size) {   // (what was undone and not done again is forgotten: a new step follows the last one that stands)
        sqlite3_exec(d.file, "delete from step where undone", nullptr, nullptr, nullptr);
        sqlite3_stmt* keep = nullptr;
        sqlite3_prepare_v2(d.file, "insert into step(change) values(?)", -1, &keep, nullptr);
        sqlite3_bind_blob(keep, 1, changed, size, SQLITE_TRANSIENT);
        sqlite3_step(keep);
        sqlite3_finalize(keep);
    }
    sqlite3_free(changed);
    sqlite3_exec(d.file, d.act_ok ? "commit" : "rollback", nullptr, nullptr, nullptr);
    if (!d.act_ok) { doc_forget(d); const std::string why = d.trouble; doc_match(d); d.trouble = why; }   // (the stage as the document was before the act)
    return d.act_ok;
}
// One change to the document (in SQLite's own words), an act by itself unless it is made within one; the stage shows
// it at once. If the stage cannot show it, it is not made, and no other change of its act is.
static bool doc_change(Doc& d, char* change) {
    doc_act(d);
    bool ok = d.act_ok;   // (a change before it in this act was refused: d.trouble still says why)
    if (ok && d.writing) ok = false, d.trouble = "the document is being written as a film: it is not changed until that is done";
    if (ok && !(ok = sqlite3_exec(d.file, change, nullptr, nullptr, nullptr) == SQLITE_OK)) d.trouble = sqlite3_errmsg(d.file);
    sqlite3_free(change);
    if (ok) {
        doc_forget(d);
        ok = doc_match(d);
    }
    if (!ok) d.act_ok = false;
    return doc_acted(d) && ok;
}

// The last step undone (back = true), or the last undone step done again. false: there is none.
static bool doc_step(Doc& d, bool back) {
    int n = 0;
    std::vector<uint8_t> change;
    doc_rows(d, sqlite3_mprintf(back ? "select n, change from step where not undone order by n desc limit 1" : "select n, change from step where undone order by n limit 1"), [&](sqlite3_stmt* row) {
        n = sqlite3_column_int(row, 0);
        change.assign((const uint8_t*)sqlite3_column_blob(row, 1), (const uint8_t*)sqlite3_column_blob(row, 1) + sqlite3_column_bytes(row, 1));
    });
    if (!n) return d.trouble = back ? "nothing to undo" : "nothing to do again", false;
    int size = (int)change.size();
    void* apply = change.data();
    if (back) sqlite3changeset_invert((int)change.size(), change.data(), &size, &apply);
    const bool ok = sqlite3changeset_apply(d.file, size, apply, nullptr, [](void*, int, sqlite3_changeset_iter*) { return (int)SQLITE_CHANGESET_ABORT; }, nullptr) == SQLITE_OK;
    if (back) sqlite3_free(apply);
    if (!ok) return d.trouble = "the step does not fit the document as it is", false;
    doc_rows(d, sqlite3_mprintf("update step set undone = %d where n = %d", back ? 1 : 0, n));
    doc_forget(d);
    return doc_match(d);
}
// The way of the stretch that begins at a key, said back in one form: a kind's name as the table writes it and all
// its numbers, however it was told ("": there is no key of that value at that moment).
static std::string doc_way_of(Doc& d, int thing, const char* name, double at) {
    std::string said;
    doc_rows(d, sqlite3_mprintf("select coalesce(way, 'Linear') from fact where thing = %d and name = %Q and at = %!.17g and " DOC_KEYED, thing, name, at, thing, name),
             [&](sqlite3_stmt* row) { said = (const char*)sqlite3_column_text(row, 0); });
    Interp kind;
    return !said.empty() && interp_said(said, &kind) == 0 ? interp_words(kind) : said;
}

// Whether a call names a thing that is there and tells it something a thing of its kind can be told (if not,
// d.trouble says what it can be told).
static bool doc_knows(Doc& d, int thing, const char* name = nullptr, const double* number = nullptr, const char* word = nullptr, bool key = false) {
    const std::string kind = doc_word(d, thing, "kind");
    if (kind.empty()) return d.trouble = "there is no thing " + std::to_string(thing), false;
    if (!name) return true;
    const DocValue* v = nullptr;
    std::string has, moving;
    for (const DocValue& one : doc_values) {
        if (!doc_among(one.kinds, kind)) continue;
        has += std::string(has.empty() ? "" : ", ") + one.name;
        if (one.moves) moving += std::string(moving.empty() ? "" : ", ") + one.name;
        if (!strcmp(one.name, name)) v = &one;
    }
    char range[120];
    if (!v) return d.trouble = "a " + kind + " has no " + name + ": it has " + has, false;
    if (number && v->words) return d.trouble = std::string(name) + " is a word, told by say" + (*v->words ? std::string(": ") + v->words : ""), false;
    if (word && !v->words) return d.trouble = std::string(name) + " is a number, told by set", false;
    if (word && *v->words && !doc_among(v->words, word)) return d.trouble = std::string(name) + " is one of: " + v->words, false;
    if (number && (!std::isfinite(*number) || *number < v->low || *number > v->high || (doc_among("keyed clip visible", name) && *number != 0.0 && *number != 1.0)))
        return std::snprintf(range, sizeof range, doc_among("keyed clip visible", name) ? " is 0 or 1" : " is from %g to %g", v->low, v->high), d.trouble = name + std::string(range), false;
    if (key && !v->moves) return d.trouble = std::string(name) + " does not move: keys go on " + moving, false;
    if (number && !strcmp(name, "parent") && *number != 0.0) {   // (a parent is a thing that is there, and not the thing itself nor one that goes with it: the past repository's rule)
        const int to = (int)*number;
        if (to != *number || doc_word(d, to, "kind").empty()) return d.trouble = "a parent is a thing that is there", false;
        for (int up = to, n = 0; up && n < 1000; up = (int)doc_number(d, up, "parent", 0, 0), ++n)
            if (up == thing) return d.trouble = "a thing cannot go with itself, nor with one that goes with it", false;
    }
    return true;
}

// The calls. A new thing of a kind (film, picture, text, solid) from a file (a text: its font; a solid: a file, or ring, gem or cube): its number (0: not
// made, and no number is used up).
// What kind of thing a file makes, by its name's end ("": none): the one place that says so.
static const char* doc_kind_of(const char* file) {
    static const struct { const char* kind; const char* ends; } made[] = {
        {"film", "mp4 mov m4v webm mkv"}, {"picture", "png jpg jpeg webp svg json lot"}, {"solid", "glb gltf"}, {"text", "ttf otf ttc"}};
    std::string end = std::filesystem::path(file).extension().string();
    for (char& c : end) c = (char)std::tolower((unsigned char)c);
    if (doc_among(doc_shapes, file)) return "solid";
    if (doc_among("rect circle", file)) return "shape";
    for (const auto& one : made)
        if (end.size() > 1 && doc_among(one.ends, end.substr(1))) return one.kind;
    return "";
}
static int doc_add(Doc& d, const char* kind, const char* file) {
    if (!*kind) kind = doc_kind_of(file);   // (not said: what the file makes)
    if (!*kind) return d.trouble = std::string("no kind of thing is made from ") + file, 0;
    if (!doc_among(doc_kinds, kind)) return d.trouble = std::string("a thing is one of: ") + doc_kinds, 0;
    // (it comes on at the stage's present moment)
    if (!doc_change(d, sqlite3_mprintf("insert into thing default values; insert into fact(thing, name, at, value) values(last_insert_rowid(), 'kind', 0.0, %Q); insert into fact(thing, name, at, value) values(last_insert_rowid(), %Q, 0.0, %Q); insert into fact(thing, name, at, value) select id, 'start', 0.0, %!.17g from thing where %d order by id desc limit 1%s",
                                       kind, (!strcmp(kind, "solid") && doc_among(doc_shapes, file)) || !strcmp(kind, "shape") ? "shape" : "file", file, d.stage->now, d.stage->now > 0.0,
                                       // (a new text comes with a word to read, at a size that fits a stage)
                                       strcmp(kind, "text") ? "" : "; insert into fact(thing, name, at, value) select id, 'text', 0.0, 'Text' from thing order by id desc limit 1; insert into fact(thing, name, at, value) select id, 'size', 0.0, 0.5 from thing order by id desc limit 1"))) return 0;
    int thing = 0;   // (the newest number: SQLite's own count of things, which a change not made does not move)
    doc_rows(d, sqlite3_mprintf("select max(id) from thing"), [&](sqlite3_stmt* row) { thing = sqlite3_column_int(row, 0); });
    return thing;
}
static bool doc_remove(Doc& d, int thing) { return doc_knows(d, thing) && doc_change(d, sqlite3_mprintf("delete from fact where thing = %d", thing)); }
// A value that stays (its keys, if it had any, are gone); a word; a key at a moment; a key taken away.
static bool doc_parent(Doc& d, int thing, double to);
static bool doc_set(Doc& d, int thing, const char* name, double is) {
    if (!strcmp(name, "parent")) return doc_parent(d, thing, is);
    return doc_knows(d, thing, name, &is) && doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = %Q and at <> 0; insert into fact(thing, name, at, value) values(%d, %Q, 0.0, %!.17g) on conflict do update set value = excluded.value, way = null", thing, name, thing, name, is));
}
static bool doc_say(Doc& d, int thing, const char* name, const char* word) {
    return doc_knows(d, thing, name, nullptr, word) && doc_change(d, sqlite3_mprintf("insert into fact(thing, name, at, value) values(%d, %Q, 0.0, %Q) on conflict do update set value = excluded.value", thing, name, word));
}
// A value either stays (one fact, at 0, with no way: what `set` writes) or is keyed: a key is a key even alone, and
// a value's first key takes the place of the value that stayed (a key at 0 is marked by its way, Linear until told).
static int doc_keys(Doc& d, int thing, const char* name, double at);
static bool doc_key(Doc& d, int thing, const char* name, double at, double is) {
    if (!(at >= 0.0 && at <= 1e6)) return d.trouble = "a key is at a moment of the stage: from 0 to 1000000 seconds", false;
    if (!doc_knows(d, thing, name, &is, nullptr, true)) return false;
    const bool stayed = !doc_keys(d, thing, name, -1.0) && at != 0.0;   // (the value that stayed goes: the key holds the place now)
    return doc_change(d, sqlite3_mprintf("%z insert into fact(thing, name, at, value, way) values(%d, %Q, %!.17g, %!.17g, %s) on conflict do update set value = excluded.value, way = coalesce(way, excluded.way)",
                                         stayed ? sqlite3_mprintf("delete from fact where thing = %d and name = %Q;", thing, name) : sqlite3_mprintf(""), thing, name, at, is, at == 0.0 ? "'Linear'" : "null"));
}
static int doc_keys(Doc& d, int thing, const char* name, double at = -1.0) {   // (how many keys a value has; or whether it has one at a moment)
    int n = 0;
    doc_rows(d, at < 0.0 ? sqlite3_mprintf("select count(*) from fact where thing = %d and name = %Q and " DOC_KEYED, thing, name, thing, name)
                         : sqlite3_mprintf("select count(*) from fact where thing = %d and name = %Q and at = %!.17g", thing, name, at), [&](sqlite3_stmt* row) { n = sqlite3_column_int(row, 0); });
    return n;
}
// What a hand does to a value at a moment, as the document's own rules (a shell does not work them out):
// put: the value changed there. Every change is being keyed (d.keying): a key there, and if it is the value's first, what
//      it was stays as a key where the keying began. Not being keyed: a value with keys is moved whole (every key by as
//      much), a value without is the value that stays.
// key_here: a key at that moment holding what the value is there (nothing, if it has one there).
// raise: brought nearer or further back among the things.
// stay: its keys are taken away, and it stays what it is at that moment.
// keyed: 0 the value has no keys, 1 it has keys but none at that moment, 2 it has a key at that moment.
// moving: the names of the values of a thing that can take keys.
static double doc_usual(const char* name);
static bool doc_set(Doc& d, int thing, const char* name, double is);
static bool doc_put(Doc& d, int thing, const char* name, double at, double is) {   // (the past repository's own rule: motolii-edit, place_checked)
    if (!doc_knows(d, thing, name, &is)) return false;
    const bool keyed = doc_keys(d, thing, name) > 0;
    if (keyed && !d.keying) {   // (a movement that is not being keyed is moved whole: every key by as much)
        const double by = is - doc_number(d, thing, name, at, doc_usual(name));
        double ends[2] = {is, is};   // (its lowest and highest keys, carried: each must still be something the value can be)
        doc_rows(d, sqlite3_mprintf("select min(value) + %!.17g, max(value) + %!.17g from fact where thing = %d and name = %Q", by, by, thing, name),
                 [&](sqlite3_stmt* row) { ends[0] = sqlite3_column_double(row, 0), ends[1] = sqlite3_column_double(row, 1); });
        if (!doc_knows(d, thing, name, &ends[0]) || !doc_knows(d, thing, name, &ends[1])) return false;
        return by == 0.0 || doc_change(d, sqlite3_mprintf("update fact set value = value + %!.17g where thing = %d and name = %Q", by, thing, name));
    }
    if (keyed) return doc_key(d, thing, name, at, is);
    if (!d.keying) return doc_set(d, thing, name, is);
    // (its first key while every change is keyed: what it was stays, as a key where the keying began, so it travels from there)
    const double was = doc_number(d, thing, name, d.keying_from, doc_usual(name));
    doc_act(d);
    if (d.keying_from != at) doc_key(d, thing, name, d.keying_from, was);
    doc_key(d, thing, name, at, is);
    return doc_acted(d);
}
// A thing given another to go with (0: none) without moving on the stage, as the sample's Parent seat promises and
// After Effects does: where it is, how far it is turned about the eye's line and how large are said again as the new
// parent sees them (a value with keys is moved whole, by the rule of doc_put; a keyed size is left as it is). A parent
// that leans, turned about x or y, is not undone in the turn: only where the thing is takes it in.
static bool doc_parent(Doc& d, int thing, double to) {
    if (!doc_knows(d, thing, "parent", &to)) return false;
    const double now = d.stage->now;
    const auto chain = [&](int up, double* turn, double* size) {   // (how far the parents turn it, and how large they make it)
        *turn = 0.0, *size = 1.0;
        for (int n = 0; up && n < 64 && d.shown.count(up); up = (int)doc_number(d, up, "parent", 0, 0), ++n) *turn += doc_number(d, up, "rotate", now, 0), *size *= doc_number(d, up, "size", now, 1);
    };
    double turnWas, sizeWas, turnTo, sizeTo;
    chain((int)doc_number(d, thing, "parent", 0, 0), &turnWas, &sizeWas);
    chain((int)to, &turnTo, &sizeTo);
    const math::double4 world = doc_above(d, thing, now) * math::double4{doc_number(d, thing, "x", now, 0), doc_number(d, thing, "y", now, 0), doc_number(d, thing, "z", now, 0), 1.0};
    const double turn = doc_number(d, thing, "rotate", now, 0), size = doc_number(d, thing, "size", now, 1);
    doc_act(d);
    const bool keying = d.keying;
    d.keying = false;   // (said again, not keyed)
    bool ok = doc_change(d, sqlite3_mprintf("insert into fact(thing, name, at, value) values(%d, 'parent', 0.0, %!.17g) on conflict do update set value = excluded.value", thing, to));
    const math::double3 here = (inverse(doc_above(d, thing, now)) * world).xyz;
    ok = ok && doc_put(d, thing, "x", now, here.x) && doc_put(d, thing, "y", now, here.y) && doc_put(d, thing, "z", now, here.z) && doc_put(d, thing, "rotate", now, turn + turnWas - turnTo);
    if (ok && !doc_keys(d, thing, "size")) ok = doc_set(d, thing, "size", std::clamp(size * sizeWas / sizeTo, 1e-3, 1e4));
    d.keying = keying;
    return doc_acted(d) && ok;
}
static bool doc_key_here(Doc& d, int thing, const char* name, double at) {
    if (!doc_knows(d, thing, name, nullptr, nullptr, true)) return false;
    return doc_keys(d, thing, name, at) && doc_keys(d, thing, name) ? true : doc_key(d, thing, name, at, doc_number(d, thing, name, at, doc_usual(name)));
}
// raise: a thing brought `places` places nearer among the things (below 0: further back): each thing is told where it
// lies now, as one change.
static bool doc_raise(Doc& d, int thing, int places) {
    if (!doc_knows(d, thing)) return false;
    std::vector<int> lie = doc_laid(d);
    const long from = std::find(lie.begin(), lie.end(), thing) - lie.begin(), to = std::clamp(from + (long)places, 0L, (long)lie.size() - 1);
    if (to == from) return d.trouble = places > 0 ? "it is the nearest already" : "it is the furthest back already", false;
    lie.erase(lie.begin() + from);
    lie.insert(lie.begin() + to, thing);
    char* all = sqlite3_mprintf("");
    for (size_t i = 0; i < lie.size(); ++i)
        all = sqlite3_mprintf("%zinsert into fact(thing, name, at, value) values(%d, 'order', 0.0, %d) on conflict do update set value = excluded.value;", all, lie[i], (int)i + 1);
    return doc_change(d, all);
}
static bool doc_stay(Doc& d, int thing, const char* name, double at) {   // (its keys go: it stays what it is at that moment)
    return doc_knows(d, thing, name) && (!doc_keys(d, thing, name) || doc_set(d, thing, name, doc_number(d, thing, name, at, doc_usual(name))));
}
static int doc_keyed(Doc& d, int thing, const char* name, double at) { return !doc_keys(d, thing, name) ? 0 : doc_keys(d, thing, name, at) ? 2 : 1; }
// The words a value of a thing may be (a blend's, a solid's shape), with spaces between ("": any word, or not a word).
static std::string doc_words(Doc& d, int thing, const char* name) {
    const std::string kind = doc_word(d, thing, "kind");
    for (const DocValue& one : doc_values)
        if (!strcmp(one.name, name) && doc_among(one.kinds, kind)) return one.words ? one.words : "";
    return "";
}
// What a thing is called where things are listed: a text by its words, a shape by what shape it is, a file by its name.
static std::string doc_name(Doc& d, int thing) {
    std::string is = doc_word(d, thing, "text");
    if (is.empty()) is = doc_word(d, thing, "shape");
    if (is.empty()) return std::filesystem::path(doc_word(d, thing, "file")).filename().string();
    if (doc_word(d, thing, "kind") != "text") is[0] = (char)std::toupper((unsigned char)is[0]);
    std::replace(is.begin(), is.end(), '\n', ' ');   // (a text of lines is listed on one)
    return is;
}
static std::string doc_moving(Doc& d, int thing) {
    const std::string kind = doc_word(d, thing, "kind");
    std::string names;
    for (const DocValue& one : doc_values)
        if (one.moves && doc_among(one.kinds, kind)) names += std::string(names.empty() ? "" : " ") + one.name;
    return names;
}
static bool doc_unkey(Doc& d, int thing, const char* name, double at) {
    if (!doc_knows(d, thing, name)) return false;
    if (!doc_keys(d, thing, name) || !doc_keys(d, thing, name, at)) return d.trouble = std::string(name) + " has no key at that moment", false;
    if (doc_keys(d, thing, name) == 1) return doc_set(d, thing, name, doc_number(d, thing, name, at, 0));   // (its last key: it stays what the key held)
    return doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = %Q and at = %!.17g", thing, name, at));
}
// How the stretch from a key to the next goes: one of the nine kinds, alone or with all its numbers (Random 3 0.25
// 0.5 0.8 0.5), or a way defined in this document with up to four numbers.
static std::string doc_ways(Doc& d) {
    std::string all;
    for (const auto& kind : interp_kinds) all += std::string(all.empty() ? "" : ", ") + kind.name + (kind.numbers ? std::string(" (") + kind.names + ")" : "");
    doc_rows(d, sqlite3_mprintf("select name from way order by name"), [&](sqlite3_stmt* row) { all += std::string("; defined here: ") + (const char*)sqlite3_column_text(row, 0); });
    return all;
}
static bool doc_way(Doc& d, int thing, const char* name, double at, const char* way) {
    if (!doc_knows(d, thing, name, nullptr, nullptr, true)) return false;
    if (!doc_keys(d, thing, name, at)) return d.trouble = std::string(name) + " has no key at that moment: a stretch begins at a key", false;
    Interp kind;
    std::istringstream said(way);
    std::string named, more;
    double p = 0;
    int numbers = 0;
    const int of_the_nine = interp_said(way, &kind, &d.trouble);
    if (of_the_nine == 1) return false;
    said >> named;
    while (said >> p) ++numbers;
    said.clear();
    if (of_the_nine < 0 && !doc_way_named(d, named)) return d.trouble = "a way is one of: " + doc_ways(d), false;
    if (of_the_nine < 0 && (said >> more || numbers > 4)) return d.trouble = "a way defined here is its name and up to four numbers", false;
    return doc_change(d, sqlite3_mprintf("update fact set way = %Q where thing = %d and name = %Q and at = %!.17g", way, thing, name, at));
}
static bool doc_define(Doc& d, const char* name, const char* expression) {
    Interp kind;
    if (interp_said(name, &kind) >= 0) return d.trouble = std::string(name) + " is one of the nine kinds already", false;
    sqlite3_stmt* goes = doc_goes(d, expression);
    if (!goes) return false;
    const double p[4] = {1, 1, 1, 1}, tried[2][3] = {{0, 1, 1}, {2, -5, 3}};   // (two stretches to try it on: values a to b, over that many seconds)
    for (const auto& s : tried) {
        const double from = doc_went(goes, 0, 0, s[2], s[0], s[1], p), to = doc_went(goes, 1, s[2], s[2], s[0], s[1], p);
        char why[160];
        if (std::isfinite(from) && std::isfinite(to) && std::fabs(from - s[0]) < 1e-6 && std::fabs(to - s[1]) < 1e-6) continue;
        std::snprintf(why, sizeof why, "a way begins at the one key's value and ends at the other's: from %g to %g this one begins at %g and ends at %g", s[0], s[1], from, to);
        return d.trouble = why, false;
    }
    return doc_change(d, sqlite3_mprintf("insert into way values(%Q, %Q) on conflict do update set goes = excluded.goes", name, expression));
}
// What a shell reads to draw its lists: the things there are, how long each is on (its duration; below 0: for good,
// going round), and the moments of a value's keys (a value with one fact has none).
static std::vector<int> doc_things(Doc& d) { return doc_laid(d); }   // (in the order they lie: the one furthest back first)
static double doc_lasts(Doc& d, int thing) {
    const double lasts = doc_number(d, thing, "duration", 0, -1);
    return lasts > 0.0 ? lasts : -1.0;
}
static std::vector<double> doc_key_moments(Doc& d, int thing, const char* name) {
    std::vector<double> at;
    doc_rows(d, sqlite3_mprintf("select at from fact where thing = %d and name = %Q and " DOC_KEYED " order by at", thing, name, thing, name),
             [&](sqlite3_stmt* row) { at.push_back(sqlite3_column_double(row, 0)); });
    return at;
}

// Half of how wide and how high a thing is at size 1, before it is placed (a sheet: the sheet's; a solid: its
// bounds'): what a shell draws a thing's frame from.
static bool doc_half(Doc& d, int thing, float2* half, float* deep) {   // (deep: half of how far it reaches towards the eye and away; a flat thing: 0)
    const auto shown = d.shown.find(thing);
    if (shown == d.shown.end()) return false;
    const Doc::Shown& t = shown->second;
    if (t.solid) {
        const Aabb box = t.solid->made->getBoundingBox();
        const float tall = std::max(box.max.y - box.min.y, 1e-6f);
        if (deep) *deep = 0.5f * (box.max.z - box.min.z) / tall;
        return *half = float2{0.5f * (box.max.x - box.min.x) / tall, 0.5f}, true;
    }
    const Mesh* m = t.film ? t.film->sheet : t.drawn ? t.drawn->sheet : t.shape;
    if (!m || m->pos.empty()) return false;
    float3 most{0.f};
    for (const float3& p : m->pos) most = max(most, abs(p));
    if (deep) *deep = most.z;
    return *half = most.xy, true;
}

// Where a point of the stage is seen on the picture (points from the top left), by the stage's own eye.
static math::double2 doc_seen(Doc& d, math::double3 p) {
    Stage& s = *d.stage;
    const math::double4 c = s.camera->getProjectionMatrix() * s.camera->getViewMatrix() * math::double4{p, 1.0};
    return math::double2{(0.5 + 0.5 * c.x / c.w) * s.w, (0.5 - 0.5 * c.y / c.w) * s.h};
}
// How many points of the picture one unit of the stage is, where a thing stands (at its own depth).
static double doc_point(Doc& d, int thing, double now) {
    const math::double3 at = (doc_above(d, thing, now) * math::double4{doc_number(d, thing, "x", now, 0), doc_number(d, thing, "y", now, 0), doc_number(d, thing, "z", now, 0), 1.0}).xyz;
    return doc_seen(d, at + math::double3{1, 0, 0}).x - doc_seen(d, at).x;
}
// A thing's frame on the picture at a moment, as the stage's own eye sees it: its middle, how wide and how high (points
// of the picture, from the top left), and how far it is turned (degrees, clockwise as seen). false: it is not shown.
static bool doc_frame(Doc& d, int thing, double now, double* frame) {
    float2 half{0.f};
    if (!doc_half(d, thing, &half) || !d.shown[thing].in) return false;   // (a thing that is not on, or is put aside, has no frame: a hand cannot take hold of what is not seen)
    const double size = doc_number(d, thing, "size", now, 1), point = doc_point(d, thing, now);
    const math::double2 middle = doc_seen(d, (doc_place(d, thing, now) * math::double4{0, 0, 0, 1}).xyz);   // (its own middle, wherever its anchor is)
    frame[0] = middle.x, frame[1] = middle.y, frame[2] = 2.0 * half.x * size * point, frame[3] = 2.0 * half.y * size * point;
    frame[4] = -doc_number(d, thing, "rotate", now, 0);
    double box[4];   // (turned about another line than the eye's, its frame is the upright box of what it covers)
    if ((doc_number(d, thing, "rotate_x", now, 0) != 0.0 || doc_number(d, thing, "rotate_y", now, 0) != 0.0 || doc_number(d, thing, "parent", 0, 0) != 0.0) && doc_covers(d, thing, now, box))
        frame[2] = box[2], frame[3] = box[3], frame[4] = 0.0;   // (as wide and high as that box; its middle stays the thing's own)
    return true;
}
// The upright box of the picture a thing covers at a moment, however it is turned or leans: left, top, wide, high.
static bool doc_covers(Doc& d, int thing, double now, double* box) {
    float2 half{0.f};
    float deep = 0.f;
    if (!doc_half(d, thing, &half, &deep)) return false;
    const math::mat4 place = doc_place(d, thing, now);
    math::double2 low{1e12}, high{-1e12};
    for (int corner = 0; corner < 8; ++corner) {
        const math::double2 at = doc_seen(d, (place * math::double4{corner & 1 ? half.x : -half.x, corner & 2 ? half.y : -half.y, corner & 4 ? deep : -deep, 1.0}).xyz);
        low = min(low, at), high = max(high, at);
    }
    box[0] = low.x, box[1] = low.y, box[2] = high.x - low.x, box[3] = high.y - low.y;
    return true;
}

static double doc_usual(const char* name) { return doc_among("size speed opacity wide visible", name) ? 1.0 : !strcmp(name, "duration") ? -1.0 : 0.0; }
static bool doc_value(Doc& d, int thing, const char* name, double at, double* is) { return doc_knows(d, thing, name) && (*is = doc_number(d, thing, name, at, doc_usual(name)), true); }
// A thing held by a hand, carried, and let go: the one form of everything a hand drags (a thing on the stage, its bar
// on the timeline, a number). What is carried shows at once and is not the document's until it is let go, and then
// it is one step; let go without keeping, nothing happened. The reckoning is here, once: a shell says where the hand
// is, never what the document is to hold.
//   by "place"  the thing carried on the stage: a, b = how far the hand has gone since it took hold, in points of the
//               picture (to the right, down); the thing stays under the hand at its own depth
//   by "bar"    its bar slid on the timeline: a = how many seconds later (when it comes on)
//   by "head", "tail"  its bar cut at an end: a = how many seconds later that end is (a film cut at its head goes on
//               showing at each moment what it showed there)
//   by a value's name  that number carried: a = what it is now
static bool doc_hold(Doc& d, int thing) {
    if (!doc_knows(d, thing)) return false;
    if (d.writing) return d.trouble = "the document is being written as a film: nothing is held until that is done", false;
    if (d.held.thing != thing) d.held = Doc::Held{}, d.held.thing = thing;   // (held already: the same hold goes on)
    return true;
}
static bool doc_carry(Doc& d, const char* by, double a, double b) {
    Doc::Held& h = d.held;
    if (!h.thing) return d.trouble = "nothing is held", false;
    const double now = d.stage->now;
    const auto was = [&](const char* name) {   // (as it was when the hand first carried it)
        auto found = h.from.find(name);
        if (found == h.from.end()) found = h.from.emplace(name, doc_number(d, h.thing, name, now, doc_usual(name))).first;
        return found->second;
    };
    const auto tries = [&](const char* name, double is) { return doc_knows(d, h.thing, name, &is) && (h.tried[name] = is, true); };
    bool ok = true;
    if (!strcmp(by, "place")) {
        if (!h.from.count(" point")) h.from[" point"] = doc_point(d, h.thing, now);   // (remembered with the hold: one unit of the stage in points of the picture, where the thing stood)
        const double x = was("x"), y = was("y"), point = h.from[" point"];
        // (the hand's travel is the stage's; a thing that goes with another is moved in that one's own turned and sized way)
        const math::double3 by = inverse(doc_above(d, h.thing, now).upperLeft()) * math::double3{a / point, -b / point, 0.0};
        ok = tries("x", x + by.x) && tries("y", y + by.y);
    } else if (!strcmp(by, "bar")) {
        ok = tries("start", std::max(0.0, was("start") + a));
    } else if (!strcmp(by, "head")) {
        const double start = was("start"), lasts = was("duration"), to = std::clamp(start + a, 0.0, lasts > 0.0 ? start + lasts - 1e-3 : 1e6);
        ok = tries("start", to) && (lasts <= 0.0 || tries("duration", start + lasts - to));
        if (ok && doc_word(d, h.thing, "kind") == "film") ok = tries("begin", std::max(0.0, was("begin") + (to - start) * std::fabs(was("speed"))));
    } else if (!strcmp(by, "tail")) {
        const double start = was("start"), lasts = was("duration");
        ok = tries("duration", std::max(1e-3, (lasts > 0.0 ? lasts : doc_seconds(d) - start) + a));
    } else ok = tries(by, a);
    doc_moment(d, now);
    return ok;
}
static bool doc_let_go(Doc& d, bool keep) {
    const Doc::Held h = d.held;
    d.held = Doc::Held{};
    if (!h.thing) return d.trouble = "nothing is held", false;
    if (!keep || h.tried.empty()) return doc_moment(d, d.stage->now), true;
    doc_act(d);   // (all that was carried is one step: a value that takes keys by the rule of a value changed at a moment, any other as the value it is)
    for (const auto& [name, is] : h.tried) {
        bool moves = false;
        for (const DocValue& one : doc_values) moves = moves || (one.moves && name == one.name);
        if (!(moves ? doc_put(d, h.thing, name.c_str(), d.stage->now, is) : doc_set(d, h.thing, name.c_str(), is))) break;
    }
    return doc_acted(d);
}
static bool doc_undo(Doc& d) { return doc_step(d, true); }
static bool doc_redo(Doc& d) { return doc_step(d, false); }
