// doc.h: a document of the stage, and the editing of it as calls: what a shell on a screen would call, and nothing
// else. The document is a file of facts: a thing's value, by name, at a time (thing, name, at, value). A value with one
// fact stays as it is; with more it moves, and they are its keys. One call = one change = one step of undo (Lumit's
// bridge is so), and the stage shows it as the call returns. A change the stage cannot show is not made.
// Borrowed whole: the file, saving (a change is in the file as its call returns) and undo are SQLite's (its session
// extension: what a call changed is kept, and undone by applying it reversed: the steps are in the file too);
// a moving value between its keys is GStreamer's (its controller, smooth and never past a key: no easing to set for
// each stretch); a thing's time is told in GES's words (start, inpoint, duration).
// The names a thing's facts go by (still to be settled):
//   numbers  x, y, z, turn, high    where its middle is, turned by how many degrees, how tall
//            start, duration        when it comes on, and for how long (not said: for good, going round)
//            inpoint, speed         a film's: from where in itself, how fast (below 0: backwards)
//            keyed                  a film's: 1 if its green ground is keyed out
//   words    lay                    how it lies on what is behind: over | add | multiply | screen
//            file, shape            what it was made from (add says them): a file; a glass's shape: ring | gem | cube
//            words, colour          a type's: what it says (to the end of the line), and its colour as six digits (ff3366)
#include "drawn.h"
#include "solid.h"
#include "video.h"
#include <gst/controller/gstinterpolationcontrolsource.h>
#include <sqlite3.h>

struct Doc {
    sqlite3* file = nullptr;
    Stage* stage = nullptr;
    std::string trouble;   // why the last call that was not done was not
    std::map<std::pair<int, std::string>, GstControlSource*> moving;   // a value's keys, read from the facts when first asked for; forgotten at every change
    struct Shown { std::string of; Video* film = nullptr; Drawn* drawn = nullptr; Solid* solid = nullptr; Mesh* glass = nullptr; bool in = true; };
    std::map<int, Shown> shown;   // the things as they are on the stage, and what each was made of
};
// What a thing may be told, by name: which kinds of thing have it, a number between which two (or one of which
// words), and whether it can move (take keys).
static const char* const doc_kinds = "film picture type solid glass";   // (the kinds of thing)
struct DocValue { const char* name; const char* kinds; double low, high; bool moves; const char* words; };
static const DocValue doc_values[] = {
    {"x", doc_kinds, -1e4, 1e4, true, nullptr}, {"y", doc_kinds, -1e4, 1e4, true, nullptr}, {"z", doc_kinds, -1e4, 1e4, true, nullptr},
    {"turn", doc_kinds, -1e6, 1e6, true, nullptr}, {"high", doc_kinds, 1e-3, 1e4, true, nullptr},
    {"start", doc_kinds, 0, 1e6, false, nullptr}, {"duration", doc_kinds, 1e-3, 1e6, false, nullptr},
    {"inpoint", "film", 0, 1e6, false, nullptr}, {"speed", "film", -100, 100, false, nullptr}, {"keyed", "film", 0, 1, false, nullptr},
    {"lay", "film picture type", 0, 0, false, "over add multiply screen"}, {"file", "film picture solid type", 0, 0, false, ""}, {"shape", "glass", 0, 0, false, "ring gem cube"},
    {"words", "type", 0, 0, false, ""}, {"colour", "type", 0, 0, false, ""}};
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
    for (auto& [value, keys] : d.moving) if (keys) gst_object_unref(keys);
    d.moving.clear();
}

// A value at a moment: where it has keys, between them; `otherwise` where the document does not say.
static double doc_number(Doc& d, int thing, const char* name, double at, double otherwise) {
    auto found = d.moving.find({thing, name});
    if (found == d.moving.end()) {
        GstControlSource* keys = nullptr;
        doc_rows(d, sqlite3_mprintf("select at, value from fact where thing = %d and name = %Q order by at", thing, name), [&](sqlite3_stmt* row) {
            if (!keys) g_object_set(keys = gst_interpolation_control_source_new(), "mode", GST_INTERPOLATION_MODE_CUBIC_MONOTONIC, nullptr);
            gst_timed_value_control_source_set(GST_TIMED_VALUE_CONTROL_SOURCE(keys), (GstClockTime)(sqlite3_column_double(row, 0) * (double)GST_SECOND), sqlite3_column_double(row, 1));
        });
        found = d.moving.emplace(std::make_pair(thing, std::string(name)), keys).first;
    }
    gdouble is = otherwise;
    if (found->second) gst_control_source_get_value(found->second, (GstClockTime)(std::max(at, 0.0) * (double)GST_SECOND), &is);
    return is;
}
static std::string doc_word(Doc& d, int thing, const char* name) {
    std::string is;
    doc_rows(d, sqlite3_mprintf("select value from fact where thing = %d and name = %Q", thing, name), [&](sqlite3_stmt* row) { is = (const char*)sqlite3_column_text(row, 0); });
    return is;
}

// Every thing told its moment: whether it is on (from its start, for its duration if it has one), what of itself it
// shows (it goes round when it comes to its end), and where it is.
static void doc_moment(Doc& d, double now) {
    Stage& s = *d.stage;
    for (auto& [thing, t] : d.shown) {
        const double since = now - doc_number(d, thing, "start", 0, 0), lasts = doc_number(d, thing, "duration", 0, -1);
        const bool in = since >= 0.0 && (lasts < 0.0 || since < lasts);
        const Mesh* m = t.film ? t.film->sheet : t.drawn ? t.drawn->sheet : t.glass;
        if (in != t.in) { if (t.solid) solid_seen(s, t.solid, in); else seen(s, m, in); t.in = in; }
        if (!in) continue;
        if (t.film) video_at(s, t.film, (float)since);
        if (t.drawn && t.drawn->seconds > 0.f) picture_at(s, t.drawn, std::fmod((float)since, t.drawn->seconds));
        const float3 at{(float)doc_number(d, thing, "x", now, 0), (float)doc_number(d, thing, "y", now, 0), (float)doc_number(d, thing, "z", now, 0)};
        const mat4f place = mat4f::translation(at) * mat4f::rotation((float)doc_number(d, thing, "turn", now, 0) * TAU / 360.f, float3{0, 0, 1}) * mat4f::scaling((float)doc_number(d, thing, "high", now, 1));
        if (t.solid) solid_to(s, t.solid, place); else move(s, m, [&](int, const mat4f&) { return place; });
    }
}

// The stage made to match the document: a thing the document has and the stage has not is made (by the kit's own
// call for its kind), one the stage has and the document has not is taken off, one whose making is told otherwise now
// is made again. false: a thing could not be made (d.trouble says which, and why).
static bool doc_match(Doc& d) {
    Stage& s = *d.stage;
    std::map<int, std::string> told;   // every thing of the document, and all that its making goes by
    doc_rows(d, sqlite3_mprintf("select thing, group_concat(name || '=' || value, ' ') from (select * from fact where name in ('kind','file','shape','lay','keyed','inpoint','duration','speed','words','colour') order by thing, name) group by thing"),
             [&](sqlite3_stmt* row) { told[sqlite3_column_int(row, 0)] = (const char*)sqlite3_column_text(row, 1); });
    for (auto it = d.shown.begin(); it != d.shown.end();) {
        const auto still = told.find(it->first);
        if (still != told.end() && still->second == it->second.of) { ++it; continue; }
        if (it->second.film) video_off(s, it->second.film);
        if (it->second.drawn) picture_off(s, it->second.drawn);
        if (it->second.solid) solid_off(s, it->second.solid);
        if (it->second.glass) take_off(s, it->second.glass);
        it = d.shown.erase(it);
    }
    bool all = true;
    for (const auto& [thing, of] : told) {
        if (d.shown.count(thing)) continue;
        Doc::Shown made{of};
        const std::string kind = doc_word(d, thing, "kind"), file = doc_word(d, thing, "file"), laid = doc_word(d, thing, "lay"), shape = doc_word(d, thing, "shape");
        const char* lay = laid.empty() || laid == "over" ? nullptr : laid.c_str();
        std::string why = "it could not be read";
        if (kind != "glass" && !std::filesystem::is_regular_file(file)) why = "there is no such file";
        else if (kind == "film") {
            const double from = doc_number(d, thing, "inpoint", 0, 0), length = doc_number(d, thing, "duration", 0, -1), speed = doc_number(d, thing, "speed", 0, 1);
            made.film = video(s, file.c_str(), mat4f{}, 1.f, (float)from, length < 0 ? -1.f : (float)(from + length * std::fabs(speed)), (float)speed, lay);
            if (!made.film) why = video_trouble;
            else if (doc_number(d, thing, "keyed", 0, 0) > 0.5) key_out(made.film);
            if (made.film) s.playing.erase(made.film);   // (the document tells it its moment, and when its sound comes on)
            if (made.film && made.film->sound) s.sounding[made.film] = [&d, thing, v = made.film, file] { return Stage::Sound{file, v->from, v->to, v->speed, 0.0, doc_number(d, thing, "start", 0, 0), doc_number(d, thing, "duration", 0, -1)}; };
        } else if (kind == "picture") {
            if ((made.drawn = picture(s, file.c_str(), mat4f{}, 1.f, float3{-1.f}, true, lay))) s.playing.erase(made.drawn);
        } else if (kind == "type") {   // (words in a font: said by `say N words ...`; until then, the font's own name)
            const std::string words = doc_word(d, thing, "words"), colour = doc_word(d, thing, "colour");
            const long rgb = colour.empty() ? 0x111111 : colour.size() == 6 && colour.find_first_not_of("0123456789abcdefABCDEF") == std::string::npos ? std::strtol(colour.c_str(), nullptr, 16) : -1;
            if (rgb < 0) why = "a colour is six digits, as ff3366";
            else made.drawn = type(s, words.empty() ? std::filesystem::path(file).stem().string().c_str() : words.c_str(), file.c_str(), mat4f{}, 1.f, float3{(float)((rgb >> 16) & 255), (float)((rgb >> 8) & 255), (float)(rgb & 255)} / 255.f, lay);
        } else if (kind == "solid") {
            made.solid = solid(s, file.c_str(), mat4f{}, 1.f);
        } else if (kind == "glass") {
            made.glass = shape == "gem" ? gem(0.5f) : shape == "cube" ? cube(0.1f) : shape == "ring" ? torus(0.37f, 0.13f) : nullptr;
            if (!made.glass) why = "a glass's shape is ring, gem or cube";
            else if (!thick_glass(s, made.glass, {mat4f{}})) made.glass = nullptr;
        } else why = std::string("its kind is one of: ") + doc_kinds;
        if (!made.film && !made.drawn && !made.solid && !made.glass) { d.trouble = "the " + kind + " (" + file + shape + ") could not be made: " + why; all = false; continue; }
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

// The document in `path` (made if it is not there), on the stage. nullptr: the file is not a document.
static Doc* doc_open(Stage& s, const char* path) {
    Doc* d = new Doc;
    gst_init(nullptr, nullptr);
    if (sqlite3_open(path, &d->file) != SQLITE_OK) return nullptr;
    sqlite3_busy_timeout(d->file, 2000);   // (another program at the same document is waited for)
    if (sqlite3_exec(d->file, "create table if not exists fact(thing integer, name text, at real, value, primary key(thing, name, at)) without rowid;"
                              "create table if not exists step(n integer primary key, change blob, undone integer default 0);"
                              "create table if not exists thing(id integer primary key autoincrement);"   // (a thing's number is never given again: SQLite's own)
                              "insert into thing(id) select max(thing) from fact where (select count(*) from thing) = 0 having max(thing) is not null", nullptr, nullptr, nullptr) != SQLITE_OK) return nullptr;
    d->stage = &s;
    s.playing[d] = [d](double now) { doc_moment(*d, now); };
    if (!doc_match(*d)) std::printf("%s\n", d->trouble.c_str());
    return d;
}

// One change to the document (in SQLite's own words), kept as one step of undo; the stage shows it at once. If the
// stage cannot show it, it is not made: the document and the stage stay as they were.
static bool doc_change(Doc& d, char* change) {
    sqlite3_session* watch = nullptr;
    int size = 0;
    void* changed = nullptr;
    sqlite3_exec(d.file, "begin", nullptr, nullptr, nullptr);
    sqlite3session_create(d.file, "main", &watch);
    sqlite3session_attach(watch, "fact");
    bool ok = sqlite3_exec(d.file, change, nullptr, nullptr, nullptr) == SQLITE_OK;
    if (!ok) d.trouble = sqlite3_errmsg(d.file);
    sqlite3session_changeset(watch, &size, &changed);
    sqlite3session_delete(watch);
    sqlite3_free(change);
    if (ok && size) {   // (what was undone and not done again is forgotten: a new step follows the last one that stands)
        sqlite3_stmt* keep = nullptr;
        sqlite3_exec(d.file, "delete from step where undone", nullptr, nullptr, nullptr);
        sqlite3_prepare_v2(d.file, "insert into step(change) values(?)", -1, &keep, nullptr);
        sqlite3_bind_blob(keep, 1, changed, size, SQLITE_TRANSIENT);
        sqlite3_step(keep);
        sqlite3_finalize(keep);
    }
    sqlite3_free(changed);
    doc_forget(d);
    ok = ok && doc_match(d);
    sqlite3_exec(d.file, ok ? "commit" : "rollback", nullptr, nullptr, nullptr);
    if (!ok) { doc_forget(d); const std::string why = d.trouble; doc_match(d); d.trouble = why; }
    return ok;
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
    if (number && (!std::isfinite(*number) || *number < v->low || *number > v->high || (!strcmp(name, "keyed") && *number != 0.0 && *number != 1.0)))
        return std::snprintf(range, sizeof range, !strcmp(name, "keyed") ? " is 0 or 1" : " is from %g to %g", v->low, v->high), d.trouble = name + std::string(range), false;
    if (key && !v->moves) return d.trouble = std::string(name) + " does not move: keys go on " + moving, false;
    return true;
}

// The calls. A new thing of a kind (film, picture, type, solid, glass) from a file (a type: its font; a glass: its shape): its number (0: not
// made, and no number is used up).
static int doc_add(Doc& d, const char* kind, const char* file) {
    if (!doc_among(doc_kinds, kind)) return d.trouble = std::string("a thing is one of: ") + doc_kinds, 0;
    if (!doc_change(d, sqlite3_mprintf("insert into thing default values; insert into fact values(last_insert_rowid(), 'kind', 0.0, %Q); insert into fact values(last_insert_rowid(), %Q, 0.0, %Q)", kind, !strcmp(kind, "glass") ? "shape" : "file", file))) return 0;
    int thing = 0;   // (the newest number: SQLite's own count of things, which a change not made does not move)
    doc_rows(d, sqlite3_mprintf("select max(id) from thing"), [&](sqlite3_stmt* row) { thing = sqlite3_column_int(row, 0); });
    return thing;
}
static bool doc_remove(Doc& d, int thing) { return doc_knows(d, thing) && doc_change(d, sqlite3_mprintf("delete from fact where thing = %d", thing)); }
// A value that stays (its keys, if it had any, are gone); a word; a key at a moment; a key taken away.
static bool doc_set(Doc& d, int thing, const char* name, double is) {
    return doc_knows(d, thing, name, &is) && doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = %Q and at <> 0; insert into fact values(%d, %Q, 0.0, %!.17g) on conflict do update set value = excluded.value", thing, name, thing, name, is));
}
static bool doc_say(Doc& d, int thing, const char* name, const char* word) {
    return doc_knows(d, thing, name, nullptr, word) && doc_change(d, sqlite3_mprintf("insert into fact values(%d, %Q, 0.0, %Q) on conflict do update set value = excluded.value", thing, name, word));
}
static bool doc_key(Doc& d, int thing, const char* name, double at, double is) {
    if (!(at >= 0.0 && at <= 1e6)) return d.trouble = "a key is at a moment of the stage: from 0 to 1000000 seconds", false;
    return doc_knows(d, thing, name, &is, nullptr, true) && doc_change(d, sqlite3_mprintf("insert into fact values(%d, %Q, %!.17g, %!.17g) on conflict do update set value = excluded.value", thing, name, at, is));
}
static int doc_keys(Doc& d, int thing, const char* name, double at = -1.0) {   // (how many keys a value has; or whether it has one at a moment)
    int n = 0;
    doc_rows(d, at < 0.0 ? sqlite3_mprintf("select count(*) from fact where thing = %d and name = %Q and (select count(*) from fact where thing = %d and name = %Q) > 1", thing, name, thing, name)
                         : sqlite3_mprintf("select count(*) from fact where thing = %d and name = %Q and at = %!.17g", thing, name, at), [&](sqlite3_stmt* row) { n = sqlite3_column_int(row, 0); });
    return n;
}
static bool doc_unkey(Doc& d, int thing, const char* name, double at) {
    if (!doc_knows(d, thing, name)) return false;
    if (!doc_keys(d, thing, name, at)) return d.trouble = std::string(name) + " has no key at that moment", false;
    return doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = %Q and at = %!.17g", thing, name, at));
}
static bool doc_undo(Doc& d) { return doc_step(d, true); }
static bool doc_redo(Doc& d) { return doc_step(d, false); }
