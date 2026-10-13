// doc.h: a document of the stage, and the editing of it as calls: what a shell on a screen would call, and nothing
// else. The document is a file of facts: a thing's value, by name, at a time (thing, name, at, value). A value with one
// fact stays as it is; with more it moves, and they are its keys. One call = one change = one step of undo (Lumit's
// bridge is so), and the stage shows it as the call returns.
// Borrowed whole: the file, saving (a change is in the file as its call returns) and undo are SQLite's (its session
// extension: what a call changed is kept, and undone by applying it reversed: the steps are in the file too);
// a moving value between its keys is GStreamer's (its controller, smooth and never past a key: no easing to set for
// each stretch); a thing's time is told in GES's words (start, inpoint, duration).
// The names a thing's facts go by (still to be settled):
//   kind   film | picture | glass          file   the film or After Effects file        shape  a glass's: ring | gem | cube
//   start, inpoint, duration, speed         when it comes on, from where in itself, for how long, how fast (below 0: backwards)
//   x, y, z, turn, high                     where its middle is, turned by how many degrees, how tall
//   key    1: its ground is keyed out (a film's green, a picture's white)               lay    add | multiply | screen
#include "drawn.h"
#include "video.h"
#include <gst/controller/gstinterpolationcontrolsource.h>
#include <sqlite3.h>

struct Doc {
    sqlite3* file = nullptr;
    Stage* stage = nullptr;
    std::map<std::pair<int, std::string>, GstControlSource*> moving;   // a value's keys, read from the facts when first asked for; forgotten at every change
    struct Shown { std::string of; Video* film = nullptr; Drawn* drawn = nullptr; Mesh* solid = nullptr; bool in = true; };
    std::map<int, Shown> shown;   // the things as they are on the stage, and what each was made of
};

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

// A value at a moment: where it has keys, between them; `otherwise` where the document does not say.
static double doc_number(Doc& d, int thing, const char* name, double at, double otherwise) {
    auto found = d.moving.find({thing, name});
    if (found == d.moving.end()) {
        GstControlSource* keys = nullptr;
        doc_rows(d, sqlite3_mprintf("select at, value from fact where thing = %d and name = %Q and typeof(value) <> 'text' order by at", thing, name), [&](sqlite3_stmt* row) {
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
    doc_rows(d, sqlite3_mprintf("select value from fact where thing = %d and name = %Q and typeof(value) = 'text'", thing, name), [&](sqlite3_stmt* row) { is = (const char*)sqlite3_column_text(row, 0); });
    return is;
}

// The stage made to match the document: a thing the document has and the stage has not is made (by the kit's own
// call for its kind), one the stage has and the document has not is taken off, one whose making is told otherwise now
// is made again. Where a thing is, and whether it is on yet, is told to it at every moment (doc_moment).
static void doc_moment(Doc& d, double now);
static bool doc_match(Doc& d) {
    Stage& s = *d.stage;
    std::map<int, std::string> told;   // every thing of the document, and all that its making goes by
    doc_rows(d, sqlite3_mprintf("select thing, group_concat(name || '=' || value, ' ') from (select * from fact where name in ('kind','file','shape','lay','key','inpoint','duration','speed') order by thing, name) group by thing"),
             [&](sqlite3_stmt* row) { told[sqlite3_column_int(row, 0)] = (const char*)sqlite3_column_text(row, 1); });
    for (auto it = d.shown.begin(); it != d.shown.end();) {
        const auto still = told.find(it->first);
        if (still != told.end() && still->second == it->second.of) { ++it; continue; }
        if (it->second.film) video_off(s, it->second.film);
        if (it->second.drawn) picture_off(s, it->second.drawn);
        if (it->second.solid) take_off(s, it->second.solid);
        it = d.shown.erase(it);
    }
    bool all = true;
    for (const auto& [thing, of] : told) {
        if (d.shown.count(thing)) continue;
        Doc::Shown made{of};
        const std::string kind = doc_word(d, thing, "kind"), file = doc_word(d, thing, "file"), lay = doc_word(d, thing, "lay"), shape = doc_word(d, thing, "shape");
        const bool keyed = doc_number(d, thing, "key", 0, 0) > 0.5;
        if (kind == "film") {
            const double from = doc_number(d, thing, "inpoint", 0, 0), length = doc_number(d, thing, "duration", 0, -1), speed = doc_number(d, thing, "speed", 0, 1);
            made.film = video(s, file.c_str(), mat4f{}, 1.f, (float)from, length < 0 ? -1.f : (float)(from + length * std::fabs(speed)), (float)speed, lay.empty() ? nullptr : lay.c_str());
            if (made.film) { s.playing.erase(made.film); if (keyed) key_out(made.film); }   // (the document tells it its moment)
        } else if (kind == "picture") {
            made.drawn = picture(s, file.c_str(), mat4f{}, 1.f, float3{1.f}, true, lay.empty() ? nullptr : lay.c_str());
            if (made.drawn) { s.playing.erase(made.drawn); if (keyed) key_out(made.drawn); }
        } else if (kind == "glass") {
            made.solid = shape == "gem" ? gem(0.5f) : shape == "cube" ? cube(0.1f) : torus(0.37f, 0.13f);
            if (!thick_glass(s, made.solid, {mat4f{}})) made.solid = nullptr;
        }
        if (!made.film && !made.drawn && !made.solid) { std::printf("thing %d could not be made (%s %s%s)\n", thing, kind.c_str(), file.c_str(), kind == "film" ? video_trouble : ""); all = false; continue; }
        d.shown[thing] = made;
    }
    doc_moment(d, s.now);
    return all;
}

// How long a thing is on, in seconds of the stage (0: for good), and the whole document (8 seconds if nothing in it ends).
static double doc_lasts(const Doc::Shown& t) { return t.film ? (double)t.film->seconds : t.drawn ? (double)t.drawn->seconds : 0.0; }
static double doc_seconds(Doc& d) {
    double all = 0.0;
    for (const auto& [thing, t] : d.shown) all = std::max(all, doc_lasts(t) > 0.0 ? doc_number(d, thing, "start", 0, 0) + doc_lasts(t) : 0.0);
    return all > 0.0 ? all : 8.0;
}

// Every thing told its moment: where it is, and what of itself it shows (a thing is on from its start for as long as it lasts).
static void doc_moment(Doc& d, double now) {
    Stage& s = *d.stage;
    for (auto& [thing, t] : d.shown) {
        const Mesh* m = t.film ? t.film->sheet : t.drawn ? t.drawn->sheet : t.solid;
        const double since = now - doc_number(d, thing, "start", 0, 0), lasts = doc_lasts(t);
        const bool in = since >= 0.0 && (lasts <= 0.0 || since < lasts);
        if (in != t.in) seen(s, m, t.in = in);
        if (!in) continue;
        if (t.film) video_at(s, t.film, (float)since);
        if (t.drawn && lasts > 0.0) picture_at(s, t.drawn, (float)since);
        const float3 at{(float)doc_number(d, thing, "x", now, 0), (float)doc_number(d, thing, "y", now, 0), (float)doc_number(d, thing, "z", now, 0)};
        const mat4f place = mat4f::translation(at) * mat4f::rotation((float)doc_number(d, thing, "turn", now, 0) * TAU / 360.f, float3{0, 0, 1}) * mat4f::scaling((float)doc_number(d, thing, "high", now, 1));
        move(s, m, [&](int, const mat4f&) { return place; });
    }
}

// The document in `path` (made if it is not there), on the stage.
static Doc* doc_open(Stage& s, const char* path) {
    Doc* d = new Doc;
    gst_init(nullptr, nullptr);
    if (sqlite3_open(path, &d->file) != SQLITE_OK) return nullptr;
    sqlite3_exec(d->file, "create table if not exists fact(thing integer, name text, at real, value, primary key(thing, name, at)) without rowid;"
                          "create table if not exists step(n integer primary key, change blob, undone integer default 0)", nullptr, nullptr, nullptr);
    d->stage = &s;
    s.playing[d] = [d](double now) { doc_moment(*d, now); };
    doc_match(*d);
    return d;
}

// One change to the document (in SQLite's own words), kept as one step of undo; the stage shows it at once.
static bool doc_change(Doc& d, char* change) {
    sqlite3_session* watch = nullptr;
    int size = 0;
    void* changed = nullptr;
    sqlite3_exec(d.file, "begin", nullptr, nullptr, nullptr);
    sqlite3session_create(d.file, "main", &watch);
    sqlite3session_attach(watch, "fact");
    const bool ok = sqlite3_exec(d.file, change, nullptr, nullptr, nullptr) == SQLITE_OK;
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
    sqlite3_exec(d.file, ok ? "commit" : "rollback", nullptr, nullptr, nullptr);
    for (auto& [value, keys] : d.moving) if (keys) gst_object_unref(keys);
    d.moving.clear();
    return ok && doc_match(d);
}

// The last step undone (back = true), or the last undone step done again. false: there is none.
static bool doc_step(Doc& d, bool back) {
    int n = 0;
    std::vector<uint8_t> change;
    doc_rows(d, sqlite3_mprintf(back ? "select n, change from step where not undone order by n desc limit 1" : "select n, change from step where undone order by n limit 1"), [&](sqlite3_stmt* row) {
        n = sqlite3_column_int(row, 0);
        change.assign((const uint8_t*)sqlite3_column_blob(row, 1), (const uint8_t*)sqlite3_column_blob(row, 1) + sqlite3_column_bytes(row, 1));
    });
    if (!n) return false;
    int size = (int)change.size();
    void* apply = change.data();
    if (back) sqlite3changeset_invert((int)change.size(), change.data(), &size, &apply);
    const bool ok = sqlite3changeset_apply(d.file, size, apply, nullptr, [](void*, int, sqlite3_changeset_iter*) { return (int)SQLITE_CHANGESET_ABORT; }, nullptr) == SQLITE_OK;
    if (back) sqlite3_free(apply);
    if (ok) doc_rows(d, sqlite3_mprintf("update step set undone = %d where n = %d", back ? 1 : 0, n));
    for (auto& [value, keys] : d.moving) if (keys) gst_object_unref(keys);
    d.moving.clear();
    return ok && doc_match(d);
}

// The calls. A new thing of a kind (film, picture, glass) from a file (a glass: its shape): its number.
static int doc_add(Doc& d, const char* kind, const char* file) {
    int thing = 1;
    doc_rows(d, sqlite3_mprintf("select coalesce(max(thing), 0) + 1 from fact"), [&](sqlite3_stmt* row) { thing = sqlite3_column_int(row, 0); });
    return doc_change(d, sqlite3_mprintf("insert into fact values(%d, 'kind', 0, %Q); insert into fact values(%d, %Q, 0, %Q)", thing, kind, thing, !strcmp(kind, "glass") ? "shape" : "file", file)) ? thing : 0;
}
static bool doc_remove(Doc& d, int thing) { return doc_change(d, sqlite3_mprintf("delete from fact where thing = %d", thing)); }
// A value that stays (its keys, if it had any, are gone); a word; a key at a moment; a key taken away.
static bool doc_set(Doc& d, int thing, const char* name, double is) { return doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = %Q; insert into fact values(%d, %Q, 0, %!.17g)", thing, name, thing, name, is)); }
static bool doc_say(Doc& d, int thing, const char* name, const char* word) { return doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = %Q; insert into fact values(%d, %Q, 0, %Q)", thing, name, thing, name, word)); }
static bool doc_key(Doc& d, int thing, const char* name, double at, double is) { return doc_change(d, sqlite3_mprintf("insert or replace into fact values(%d, %Q, %!.17g, %!.17g)", thing, name, at, is)); }
static bool doc_unkey(Doc& d, int thing, const char* name, double at) { return doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = %Q and at = %!.17g", thing, name, at)); }
static bool doc_undo(Doc& d) { return doc_step(d, true); }
static bool doc_redo(Doc& d) { return doc_step(d, false); }
