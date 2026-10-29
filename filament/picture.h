// The window's two pictures of the document (skin/BRIDGE.md): what it shows is read from these, never from the
// document while it builds. Made by the hand, while it holds the document, as JSON (rapidjson, the copy Filament's assimp
// brings); set down whole, and the window told by its one listener which picture is new.
//   the document's (1): made again when the document changes; every thing's words, numbers at rest, ranges, keys...
//   the moment's (2):   made with each picture; where each thing is and what moves, at the moment drawn
// (motolii.mm: read at its end, after every call it asks)
#pragma once
#include "third_party/filament_src/third_party/libassimp/contrib/rapidjson/include/rapidjson/stringbuffer.h"
#include "third_party/filament_src/third_party/libassimp/contrib/rapidjson/include/rapidjson/writer.h"

using PictureWriter = rapidjson::Writer<rapidjson::StringBuffer>;

static std::vector<std::string> picture_words(const char* said) {   // (a list said with spaces between)
    std::vector<std::string> all;
    std::istringstream in(said ? said : "");
    for (std::string w; in >> w;) all.push_back(w);
    return all;
}

// Every name a thing's numbers and words go by: those it has, those of its cards, and every fact it has.
static std::vector<std::string> picture_names(Motolii* m, int thing) {
    std::set<std::string> all;
    for (const std::string& n : picture_words(motolii_values(m, thing))) all.insert(n);
    for (std::string n : picture_words(motolii_cards(m, thing))) {
        if (!n.empty() && n.back() == ';') n.pop_back();
        if (n.empty() || n.back() == ':' || !std::islower((unsigned char)n[0])) continue;   // (a card's own name ends in ':')
        all.insert(n);
    }
    for (const char* n : {"kind", "name", "file", "fill", "blend", "text", "shape", "path", "parent", "space", "stroke", "fx", "script", "mosh"}) all.insert(n);
    doc_rows(*m->d, sqlite3_mprintf("select distinct name from fact where thing = %d", thing), [&](sqlite3_stmt* row) { all.insert((const char*)sqlite3_column_text(row, 0)); });   // (and every fact it has: a relation's own, x_by)
    return {all.begin(), all.end()};
}

static void picture_thing(Motolii* m, int thing, PictureWriter& w) {
    w.StartObject();
    w.Key("lasts"), w.Double(motolii_lasts(m, thing));
    w.Key("values"), w.String(motolii_values(m, thing));
    w.Key("moving"), w.String(motolii_moving(m, thing));
    w.Key("cards"), w.String(motolii_cards(m, thing));
    w.Key("fx"), w.String(motolii_fx_titles(m, thing));
    w.Key("stretches"), w.String(motolii_thing_stretches(m, thing));
    {
        double at[512];
        const int n = std::min(motolii_thing_key_moments(m, thing, at, 512), 512);
        w.Key("keys"), w.StartArray();
        for (int i = 0; i < n; ++i) w.Double(at[i]);
        w.EndArray();
    }
    const std::vector<std::string> names = picture_names(m, thing);
    w.Key("word"), w.StartObject();
    for (const std::string& n : names) if (const std::string s = motolii_word(m, thing, n.c_str()); !s.empty()) w.Key(n.c_str()), w.String(s.c_str());
    w.EndObject();
    w.Key("value"), w.StartObject();   // (at rest: what it is at its first moment; one not here is as it comes, the document's "usual", if its kind knows it)
    for (const std::string& n : names) if (double v = 0; doc_value(*m->d, thing, n.c_str(), 0, &v)) w.Key(n.c_str()), w.Double(v);
    w.EndObject();
    w.Key("range"), w.StartObject();
    for (const std::string& n : names) if (double lo = 0, hi = 0; motolii_range(m, thing, n.c_str(), &lo, &hi)) w.Key(n.c_str()), w.StartArray(), w.Double(lo), w.Double(hi), w.EndArray();
    w.EndObject();
    w.Key("words"), w.StartObject();
    for (const std::string& n : names) if (const std::string s = motolii_words(m, thing, n.c_str()); !s.empty()) w.Key(n.c_str()), w.String(s.c_str());
    w.EndObject();
    w.Key("reading"), w.StartObject();
    for (const std::string& n : names) w.Key(n.c_str()), w.String(motolii_reading(m, thing, n.c_str()));
    w.EndObject();
    w.Key("shown"), w.StartObject();   // (how each is shown to a hand: the maker's table, and a script's own)
    for (const std::string& n : names) if (const std::string s = motolii_shown(m, n.c_str()); !s.empty()) w.Key(n.c_str()), w.String(s.c_str());
    w.EndObject();
    w.Key("keyMoments"), w.StartObject();   // (each value that has keys: when they are)
    for (const std::string& n : names) {
        double at[512];
        const int k = std::min(motolii_key_moments(m, thing, n.c_str(), at, 512), 512);
        if (k <= 0) continue;
        w.Key(n.c_str()), w.StartArray();
        for (int i = 0; i < k; ++i) w.Double(at[i]);
        w.EndArray();
    }
    w.EndObject();
    w.Key("keyValues"), w.StartObject();   // (what each key holds)
    for (const std::string& n : names) {
        double at[512];
        const int k = std::min(motolii_key_moments(m, thing, n.c_str(), at, 512), 512);
        if (k <= 0) continue;
        w.Key(n.c_str()), w.StartArray();
        for (int i = 0; i < k; ++i) if (double v = 0; doc_value(*m->d, thing, n.c_str(), at[i], &v)) w.Double(v); else w.Null();
        w.EndArray();
    }
    w.EndObject();
    w.Key("keyWays"), w.StartObject();   // (the way of the stretch each key begins, as it is said: "" none)
    for (const std::string& n : names) {
        double at[512];
        const int k = std::min(motolii_key_moments(m, thing, n.c_str(), at, 512), 512);
        if (k <= 0) continue;
        w.Key(n.c_str()), w.StartArray();
        for (int i = 0; i < k; ++i) w.String(motolii_way_of(m, thing, n.c_str(), at[i]));
        w.EndArray();
    }
    w.EndObject();
    w.EndObject();
}

// The pictures are the stage's own reads, never the asker's: why its last call was refused is left as it was, whatever
// these reads ask on the way (SQLite's "error-code preserving interfaces": sqlite.org/c3ref/errcode.html), kept and put
// back however they end (C++'s scope guard, RAII).
struct PictureKeepsWhy {
    Doc& d;
    std::string why = d.trouble;
    ~PictureKeepsWhy() { d.trouble = why; }
};
// The document's picture made again where it changed (on the hand, the document held): the things whose facts changed
// (all of them when a thing came or went, or one was named again: a name is a choice of others' cards), and the
// document's own. Each thing's part keeps the version it was made at, so a window asks for what is newer than it has.
static void picture_update(Motolii* m) {
    Doc& d = *m->d;
    const PictureKeepsWhy keep{d};
    std::set<int> again;
    bool all = d.things_changed || m->pictured.empty();
    std::string changes = d.things_changed ? "things\n" : "";
    for (const auto& [thing, name] : d.changed) {
        changes += std::to_string(thing) + " " + name + "\n";
        if (name == "name") all = true;
        again.insert(thing);
    }
    d.changed.clear(), d.things_changed = false;
    m->picturedRows = sqlite3_total_changes(d.file);
    int things[1024];
    const int n = std::min(motolii_things(m, things, 1024), 1024);
    std::map<int, std::string> made;
    const auto make = [&](int thing) {
        rapidjson::StringBuffer out;
        PictureWriter w(out);
        picture_thing(m, thing, w);
        made[thing] = {out.GetString(), out.GetSize()};
    };
    if (all || again.count(0)) make(0);   // (the stage's own: its cards)
    for (int i = 0; i < n; ++i) if (all || again.count(things[i])) make(things[i]);
    rapidjson::StringBuffer out;
    PictureWriter w(out);
    w.StartObject();
    w.Key("seconds"), w.Double(motolii_seconds(m));
    w.Key("rate"), w.Int(motolii_rate(m));
    { int wide = 0, high = 0; motolii_size_of(m, &wide, &high); w.Key("size"), w.StartArray(), w.Int(wide), w.Int(high), w.EndArray(); }
    w.Key("fits"), w.Bool(motolii_fits(m) != 0);
    { double a = 0, b = 0; motolii_work_of(m, &a, &b); w.Key("work"), w.StartArray(), w.Double(a), w.Double(b), w.EndArray(); }
    w.Key("point"), w.Double(motolii_point(m));
    w.Key("edited"), w.Double(doc_edited(*m->d));   // (how long it has been worked on, seconds: Alight Motion's Total Edit Time)
    w.Key("marks"), w.String(motolii_marks(m));
    w.Key("guides"), w.String(motolii_guides(m));
    w.Key("ways"), w.String(motolii_ways(m));
    w.Key("wayNames"), w.String(motolii_way_names(m));
    w.Key("effectNames"), w.String(motolii_effect_names(m));
    w.Key("wayCurves"), w.StartObject();   // (each way's shape: how far at 65 steps of u, 0 to 1, a curve drawn from these)
    for (const std::string& way : picture_words(motolii_way_names(m))) {
        w.Key(way.c_str()), w.StartArray();
        for (int i = 0; i <= 64; ++i) w.Double(motolii_way_at(m, way.c_str(), i / 64.0));
        w.EndArray();
    }
    w.EndObject();
    w.Key("changes"), w.String(changes.c_str());
    w.Key("usual"), w.StartObject();   // (each value as it comes, said nowhere)
    for (const DocValue& v : doc_values) if (!v.words) w.Key(v.name), w.Double(doc_usual(*m->d, v.name)), w.Key((std::string(v.name) + "_by").c_str()), w.Double(0);
    w.EndObject();
    w.Key("knows"), w.StartObject();   // (the values each kind of thing has, said or not: as doc_knows answers)
    for (const char* kind : {"film", "picture", "text", "solid", "shape", "camera", "light", "behaviour", "stage"}) {
        std::string all;
        for (const DocValue& v : doc_values)
            if (doc_among(v.kinds, kind)) all += std::string(all.empty() ? "" : " ") + v.name + (v.words || !strcmp(kind, "stage") ? "" : std::string(" ") + v.name + "_by");   // (a value connected to a behaviour: not the stage's)
        w.Key(kind), w.String(all.c_str());
    }
    w.EndObject();
    w.Key("things"), w.StartArray();
    for (int i = 0; i < n; ++i) w.Int(things[i]);
    w.EndArray();
    w.EndObject();
    std::lock_guard<std::mutex> hold(m->pictureLock);
    std::string own = {out.GetString(), out.GetSize()};
    if (made.empty() && own == m->pictureAll) return;   // (nothing the window reads changed: no new picture)
    const long v = ++m->pictureVersion;
    for (auto& [thing, part] : made) m->pictured[thing] = {v, std::move(part)};
    std::set<int> are(things, things + n);
    are.insert(0);
    for (auto it = m->pictured.begin(); it != m->pictured.end();) it = are.count(it->first) ? std::next(it) : m->pictured.erase(it);   // (a thing gone: its part too)
    m->pictureAll = std::move(own);
    m->pictureTold |= 1;
}

// The moment's picture (on the hand, the document held, as the moment is told): where each thing is and how it is
// turned, as the stage shows it; and the picked thing's moving values, which of its values have keys there, its points.
static void picture_moment(Motolii* m) {
    Doc& d = *m->d;
    const PictureKeepsWhy keep{d};
    const double at = m->now;
    rapidjson::StringBuffer out;
    PictureWriter w(out);
    w.StartObject();
    w.Key("now"), w.Double(at);
    int things[1024];
    const int n = std::min(motolii_things(m, things, 1024), 1024);
    w.Key("thing"), w.StartObject();
    for (int i = 0; i < n; ++i) {
        const int t = things[i];
        w.Key(std::to_string(t).c_str()), w.StartObject();
        double frame[8], box[4];
        if (doc_frame(d, t, at, frame)) { w.Key("frame"), w.StartArray(); for (double f : frame) w.Double(f); w.EndArray(); }
        if (doc_seen_box(d, t, at, box)) { w.Key("seen"), w.StartArray(); for (double b : box) w.Double(b); w.EndArray(); }
        w.Key("shows"), w.StartObject();   // (as the stage shows it: [value, driven by a relation or behaviour])
        for (const char* name : {"x", "y", "z", "rotate", "scale_x", "scale_y", "opacity"})
            if (double v = 0; const int got = motolii_shows(m, t, name, at, &v)) w.Key(name), w.StartArray(), w.Double(v), w.Bool(got == 2), w.EndArray();
        w.EndObject();
        if (t == m->pick || m->watched.count(t)) {   // (its values that move, at the moment: the picked thing's and those the window watches)
            w.Key("value"), w.StartObject();
            for (const std::string& name : picture_words(motolii_moving(m, t)))
                if (double v = 0; doc_value(d, t, name.c_str(), at, &v)) w.Key(name.c_str()), w.Double(v);
            w.EndObject();
        }
        if (t == m->pick) {
            double xy[512], of[256];
            const int p = std::min(doc_points(d, t, at, xy, 256), 256);
            w.Key("points"), w.StartArray(); for (int k = 0; k < 2 * p; ++k) w.Double(xy[k]); w.EndArray();
            const int h = std::min(doc_points(d, t, at, xy, 256, true, of), 256);
            w.Key("handles"), w.StartArray(); for (int k = 0; k < 2 * h; ++k) w.Double(xy[k]); w.EndArray();
            w.Key("of"), w.StartArray(); for (int k = 0; k < h; ++k) w.Double(of[k]); w.EndArray();
        }
        w.EndObject();
    }
    w.EndObject();
    w.EndObject();
    std::lock_guard<std::mutex> hold(m->pictureLock);
    ++m->momentVersion;
    m->momentPicture = {out.GetString(), out.GetSize()};
    m->pictureTold |= 2;
}

// Told after the document is let go (Lumit's rule): which pictures are new, 1 the document's, 2 the moment's, 4 what
// of the stage's pictures is kept on disk.
static void picture_tell(Motolii* m) {
    int told;
    { std::lock_guard<std::mutex> hold(m->pictureLock); told = m->pictureTold, m->pictureTold = 0; }
    if (const long k = m->kept.told.load(); k != m->keptTold) told |= 4, m->keptTold = k;   // (the stage's pictures kept on disk changed)
    if (told && m->told) m->told(told);
}
