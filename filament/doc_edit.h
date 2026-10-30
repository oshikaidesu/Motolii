// doc_edit.h: part of doc.h (things added, freed, related, led; keys and values put). Read only through doc.h.
static bool doc_knows(Doc& d, int thing, const char* name = nullptr, const double* number = nullptr, const char* word = nullptr, bool key = false) {
    const std::string kind = doc_word(d, thing, "kind");
    if (kind.empty()) return d.trouble = "there is no thing " + std::to_string(thing), false;
    if (!name) return true;
    const DocValue* v = nullptr;
    for (const DocValue& one : doc_values)
        if (!strcmp(one.name, name) && doc_among(one.kinds, kind)) { v = &one; break; }
    static const DocValue by{"_by", "film picture text solid shape camera light behaviour", 0, 1e9, false, nullptr};   // (a value connected to a behaviour: `<name>_by` = that behaviour, 0 none)
    if (!v && strlen(name) > 3 && !strcmp(name + strlen(name) - 3, "_by")) {
        const std::string base(name, strlen(name) - 3);
        for (const DocValue& one : doc_values)
            if (base == one.name && doc_among(one.kinds, kind) && !one.words) v = &by;
        if (v && number && *number != 0.0 && (*number != std::floor(*number) || doc_word(d, (int)*number, "kind") != "behaviour")) return d.trouble = std::string(name) + " is a behaviour that is there", false;
    }
    if (!v && !strncmp(name, "js_", 3) && doc_among("film picture text shape solid stage camera", kind))   // (a knob of the thing's script; a camera's motion's, golf ㊴)
        for (const DocScript* sc : doc_scripts_on(d, thing))
            for (const DocValue& one : sc->values)
                if (!strcmp(one.name, name)) v = &one;
    const auto said = [&](bool moves) {   // (the names it has, or those that move: said only when a call is refused)
        std::string all;
        for (const DocValue& one : doc_values)
            if (doc_among(one.kinds, kind) && (!moves || one.moves)) all += std::string(all.empty() ? "" : ", ") + one.name;
        if (doc_among("film picture text shape solid stage camera", kind))
            for (const DocScript* sc : doc_scripts_on(d, thing))
                for (const DocValue& one : sc->values)
                    if (!moves || one.moves) all += std::string(all.empty() ? "" : ", ") + one.name;
        return all;
    };
    char range[120];
    if (!v) return d.trouble = "a " + kind + " has no " + name + ": it has " + said(false), false;
    if (number && v->words) return d.trouble = std::string(name) + " is a word, told by say" + (*v->words ? std::string(": ") + v->words : ""), false;
    if (word && !v->words) return d.trouble = std::string(name) + " is a number, told by set", false;
    if (word && *v->words && !doc_among(v->words, word)) return d.trouble = std::string(name) + " is one of: " + v->words, false;
    if (word && *word && (!strcmp(name, "marker") || !strcmp(name, "animation"))) {   // (a name its file has)
        const std::string has = doc_words(d, thing, name);
        if (has.empty()) return d.trouble = std::string("its file has no ") + (!strcmp(name, "marker") ? "markers" : "movements of its own"), false;
        if (!doc_among(has.c_str(), word)) return d.trouble = std::string(name) + " is one of: " + has, false;
    }
    if (word && *word && !strcmp(name, "copy_tall") && strcmp(word, "none")) {   // (numbers, one for each copy in their order)
        std::istringstream all(word);
        for (std::string one; all >> one;) { char* end = nullptr; const double is = std::strtod(one.c_str(), &end); if (*end || !std::isfinite(is) || is < 0.0 || is > 1e3) return d.trouble = "copy_tall is numbers from 0 to 1000, one for each copy, as 1 2.5 0.8", false; }
    }
    if (word && *word && !strcmp(name, "copy_fill") && strcmp(word, "none")) {   // (colours, six digits each; none: not coloured each)
        std::istringstream all(word);
        int n = 0;
        for (std::string one; all >> one; ++n)
            if (one.size() != 6 || one.find_first_not_of("0123456789abcdefABCDEF") != std::string::npos) return d.trouble = "copy_fill is colours of six digits, as ff3366 1b2fd8", false;
        if (n < 1) return d.trouble = "copy_fill is colours of six digits, as ff3366 1b2fd8, or none", false;
    }
    if (word && *word && (doc_among("vignette_color fog_color background", name) || (!strcmp(name, "fill") && kind == "light")))   // (one colour: six digits)
        if (strlen(word) != 6 || strspn(word, "0123456789abcdefABCDEF") != 6) return d.trouble = std::string(name) + " is six digits, as ff3366", false;
    if (number && (!std::isfinite(*number) || *number < v->low || *number > v->high || (doc_among("keyed clip visible solo lock shadows clip_invert", name) && *number != 0.0 && *number != 1.0)))
        return std::snprintf(range, sizeof range, doc_among("keyed clip visible solo lock shadows clip_invert", name) ? " is 0 or 1" : " is from %g to %g", v->low, v->high), d.trouble = name + std::string(range), false;
    if (key && !v->moves) { const std::string moving = said(true); return d.trouble = std::string(name) + " does not move: " + (moving.empty() ? std::string("nothing of it does") : "keys go on " + moving), false; }
    if ((number || word || key) && strcmp(name, "lock") && doc_number(d, thing, "lock", 0, 0) > 0.5) return d.trouble = "it is locked", false;
    if (number && (!strcmp(name, "connect") || !strcmp(name, "connect_to")) && *number != 0.0 && (*number != std::floor(*number) || (int)*number == thing || doc_word(d, (int)*number, "kind").empty()))
        return d.trouble = std::string(name) + " is another thing that is there", false;   // (as a parent is)
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
// What kind of thing a file makes, by its name's end ("": none): the one place that says so. A sound is a film heard and
// not seen (doc_add).
static const char* doc_kind_of(const char* file) {
    static const struct { const char* kind; const char* ends; } made[] = {
        {"film", "mp4 mov m4v webm mkv"}, {"sound", "mp3 wav m4a aac flac ogg opus"}, {"picture", "png jpg jpeg webp svg json lot mid"}, {"solid", "glb gltf ply"}, {"text", "ttf otf ttc"}};
    std::string end = std::filesystem::path(file).extension().string();
    for (char& c : end) c = (char)std::tolower((unsigned char)c);
    if (doc_among(doc_shapes, file)) return "solid";
    if (doc_among("rect circle polygon star line null ring arc arrow squircle cog path spiral chamfer", file)) return "shape";
    for (const auto& one : made)
        if (end.size() > 1 && doc_among(one.ends, end.substr(1))) return one.kind;
    return "";
}
static int doc_add(Doc& d, const char* kind, const char* file) {
    if (!*kind) kind = doc_kind_of(file);   // (not said: what the file makes)
    if (!*kind) return d.trouble = std::string("no kind of thing is made from ") + file, 0;
    // (a sound comes in a film that is heard, hidden: Alight Motion's audio layer has no picture, only its wave on the
    //  timeline (reports/alight_ui.md 177), as Extract Sound's copy (doc_extract_sound) is a hidden film heard; its eye
    //  shows its wave, video.h)
    const bool sound = !strcmp(kind, "sound");
    if (sound) kind = "film";
    if (!doc_among(doc_kinds, kind) && strcmp(kind, "behaviour")) return d.trouble = std::string("a thing is one of: ") + doc_kinds + " behaviour", 0;
    if (!strcmp(kind, "behaviour") && !doc_among("stagger oscillator noise random around row follow push group", file)) return d.trouble = "a behaviour is a stagger, an oscillator, a noise, a random, or a relation: around, row, follow, push", 0;
    if (!strcmp(kind, "light") && !doc_among("sun point spot", file)) return d.trouble = "a light is a sun, a point or a spot", 0;
    // (it comes on at the stage's present moment, the playhead: AbsentMotion's addAudio and addText, startTime frameStart(frame))
    if (!doc_change(d, sqlite3_mprintf("insert into thing default values; insert into fact(thing, name, at, value) values(last_insert_rowid(), 'kind', 0.0, %Q); insert into fact(thing, name, at, value) values(last_insert_rowid(), %Q, 0.0, %Q); insert into fact(thing, name, at, value) select id, 'start', 0.0, %!.17g from thing where %d order by id desc limit 1%s",
                                       kind, (!strcmp(kind, "solid") && doc_among(doc_shapes, file)) || !strcmp(kind, "shape") ? "shape" : "file", file, d.stage->now, d.stage->now > 0.0,
                                       // (a new text comes with a word to read, at a size that fits a stage, in the grey
                                       //  AbsentMotion's addText fills one with: TextActions.kt withFill 0xFF7F7F7F)
                                       (std::string(strcmp(kind, "text") ? "" : "; insert into fact(thing, name, at, value) select id, 'text', 0.0, 'Text' from thing order by id desc limit 1; insert into fact(thing, name, at, value) select id, 'size', 0.0, 0.5 from thing order by id desc limit 1; insert into fact(thing, name, at, value) select id, 'fill', 0.0, '7f7f7f' from thing order by id desc limit 1") +
                                        // (and lasts as long as the document says a thing put in lasts, if it says: AbsentMotion's New Layer Length,
                                        //  its StructureActions.kt and TextActions.kt endTimeFor; a film its own length, a group's Null as long as what it holds)
                                        (doc_among("picture text solid shape", kind) && strcmp(file, "null") ? "; insert into fact(thing, name, at, value) select (select max(id) from thing), 'duration', 0.0, value from fact where thing = 0 and name = 'layer_length' and at = 0 and value > 0" : "") +
                                        (sound ? "; insert into fact(thing, name, at, value) select max(id), 'visible', 0.0, 0 from thing" : "")).c_str()))) return 0;
    int thing = 0;   // (the newest number: SQLite's own count of things, which a change not made does not move)
    doc_rows(d, sqlite3_mprintf("select max(id) from thing"), [&](sqlite3_stmt* row) { thing = sqlite3_column_int(row, 0); });
    return thing;
}
// A thing made again: a new number with every fact of it (its keys and ways too), lying where it lies (After Effects'
// Duplicate Layer; the new one is the newest number, as for add).
static int doc_duplicate(Doc& d, int thing) {
    if (!doc_knows(d, thing)) return d.trouble = "that is no thing", 0;
    if (!doc_change(d, sqlite3_mprintf("insert into thing default values; insert into fact(thing, name, at, value, way) select last_insert_rowid(), name, at, value, way from fact where thing = %d;", thing))) return 0;
    int made = 0;
    doc_rows(d, sqlite3_mprintf("select max(id) from thing"), [&](sqlite3_stmt* row) { made = sqlite3_column_int(row, 0); });
    return made;
}
static bool doc_release(Doc& d, int by, double now);
static bool doc_free(Doc& d, int thing, int by, double now, bool stay);
static std::vector<int> doc_things(Doc& d);
static bool doc_remove(Doc& d, int thing) {
    if (doc_knows(d, thing) && doc_number(d, thing, "lock", 0, 0) > 0.5) return d.trouble = "it is locked", false;
    if (doc_knows(d, thing) && doc_word(d, thing, "kind") != "behaviour") {   // (a leader taken away: what it led lets its things go where they show, as After Effects keeps a child where it is when its parent goes)
        std::vector<int> led;
        for (const int r : doc_things(d))
            if (doc_word(d, r, "kind") == "behaviour" && doc_among(doc_relations, doc_word(d, r, "file")) && ((int)doc_number(d, r, "leader", 0, 0) == thing || (int)doc_number(d, r, "leader_b", 0, 0) == thing)) led.push_back(r);
        if (!led.empty()) {
            doc_act(d);
            bool ok = true;
            for (const int r : led) ok = ok && doc_release(d, r, d.stage ? d.stage->now : 0.0);
            ok = ok && doc_change(d, sqlite3_mprintf("delete from fact where thing = %d", thing));
            return doc_acted(d) && ok;
        }
    }
    if (doc_knows(d, thing) && doc_among(doc_relations, doc_word(d, thing, "file")) && doc_word(d, thing, "kind") == "behaviour") {   // (a relation taken away: its things back to what is under it, as a constraint deleted in Blender; release keeps them where they show)
        const std::vector<int> all = doc_members_of(d, thing);
        doc_act(d);
        bool ok = true;
        for (const int t : all) if (ok && t) ok = doc_free(d, t, thing, d.stage ? d.stage->now : 0.0, false);
        ok = ok && doc_change(d, sqlite3_mprintf("delete from fact where thing = %d", thing));
        return doc_acted(d) && ok;
    }
    return doc_knows(d, thing) && doc_change(d, sqlite3_mprintf("delete from fact where thing = %d", thing)); }
// A value that stays (its keys, if it had any, are gone); a word; a key at a moment; a key taken away.
static bool doc_parent(Doc& d, int thing, double to);
static bool doc_lead(Doc& d, int by, int leader, double now);
static bool doc_leading = false;   // (doc_lead itself writing the leader)
static bool doc_set(Doc& d, int thing, const char* name, double is) {
    if (!strcmp(name, "parent")) return doc_parent(d, thing, is);
    if (!strcmp(name, "leader") && !doc_leading && is >= 1 && doc_word(d, thing, "kind") == "behaviour" && doc_among(doc_relations, doc_word(d, thing, "file")) && !doc_word(d, thing, "members").empty())   // (a relation's leader changed by any hand: as its crown changes it, nothing jumps)
        return doc_lead(d, thing, (int)is, d.stage ? d.stage->now : 0.0);
    return doc_knows(d, thing, name, &is) && doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = %Q and at <> 0; insert into fact(thing, name, at, value) values(%d, %Q, 0.0, %!.17g) on conflict do update set value = excluded.value, way = null", thing, name, thing, name, is));
}
static bool doc_say(Doc& d, int thing, const char* name, const char* word) {
    return doc_knows(d, thing, name, nullptr, word) && doc_change(d, sqlite3_mprintf("insert into fact(thing, name, at, value) values(%d, %Q, 0.0, %Q) on conflict do update set value = excluded.value", thing, name, word));
}
// A thing taken out of a relation where it shows at that moment: no longer in it (what its values were connected to
// is what this one started from for it, or none; a relation that started from this one for it now starts from that),
// its place among the relation's members kept empty so no other moves, then each of its connected values moved (all
// its keys alike) by as much as it would otherwise jump. One step; nothing jumps.
static void doc_act(Doc& d);
static bool doc_acted(Doc& d);
static const char* doc_lasting(const std::string& name) {   // (a name made at run time, kept where doc_literal can know it again by its place: one place for each name, ever)
    static std::set<std::string> kept;
    return kept.insert(name).first->c_str();
}
static bool doc_free(Doc& d, int thing, int by, double now, bool stay) {
    const Doc::Relation rel = doc_relation_of(d, by);   // (a copy: what follows forgets it)
    const auto at = std::find(rel.members.begin(), rel.members.end(), thing);
    if (at == rel.members.end()) return d.trouble = "it is not in that relation", false;
    const size_t i = (size_t)(at - rel.members.begin());
    const int was = rel.under.empty() ? 0 : rel.under[i];
    std::vector<std::pair<std::string, double>> names;   // (each connected value and where it shows now)
    doc_rows(d, sqlite3_mprintf("select name from fact where thing = %d and name glob '*_by' and value <> 0", thing), [&](sqlite3_stmt* q) { names.push_back({(const char*)sqlite3_column_text(q, 0), 0.0}); });
    for (auto& [n, shown] : names) shown = doc_literal(d, thing, doc_lasting(n.substr(0, n.size() - 3)), now, 0.0);
    const auto word = [&](const char* name, int r, const std::function<std::string(size_t, const std::string&)>& each) {
        std::string all;
        std::istringstream said(doc_word(d, r, name));
        size_t k = 0;
        for (std::string one; said >> one; ++k) all += (all.empty() ? "" : " ") + each(k, one);
        return sqlite3_mprintf("update fact set value = %Q where thing = %d and name = %Q;", all.c_str(), r, name);
    };
    doc_act(d);
    char* change = word("members", by, [&](size_t k, const std::string& one) { return k == i ? std::string("_") : one; });
    for (const auto& [n, _] : names)   // (connected to this one: to what it started from)
        if ((int)doc_number(d, thing, n.c_str(), 0, 0) == by)
            change = was ? sqlite3_mprintf("%z update fact set value = %d where thing = %d and name = %Q;", change, was, thing, n.c_str()) : sqlite3_mprintf("%z delete from fact where thing = %d and name = %Q;", change, thing, n.c_str());
    for (const auto& [r, other] : d.relationsOf)   // (a relation that started from this one for it: from that one)
        if (r != by) {
            const auto there = std::find(other.members.begin(), other.members.end(), thing);
            if (there != other.members.end() && !other.under.empty() && other.under[(size_t)(there - other.members.begin())] == by)
                change = sqlite3_mprintf("%z %z", change, word("under", r, [&](size_t k, const std::string& one) { return k == (size_t)(there - other.members.begin()) ? std::to_string(was) : one; }));
        }
    bool ok = doc_change(d, change);
    doc_forget(d);
    for (const auto& [n, shown] : names) {   // (each value moved by what it would jump, when baked; else as the relation at strength 0 for good: back to the one under it, or its own value, so a morph by strength and a removal agree)
        if (!ok || !stay) break;
        const std::string base = n.substr(0, n.size() - 3);
        const double off = shown - doc_literal(d, thing, doc_lasting(base), now, 0.0);
        if (std::abs(off) < 1e-12) continue;
        ok = doc_change(d, sqlite3_mprintf("update fact set value = value + %!.17g where thing = %d and name = %Q; insert into fact(thing, name, at, value) select %d, %Q, 0.0, %!.17g where not exists (select 1 from fact where thing = %d and name = %Q);",
                                           off, thing, base.c_str(), thing, base.c_str(), off, thing, base.c_str()));
        doc_forget(d);
    }
    return doc_acted(d) && ok;
}
// Things put in a relation as they show now: in its order (round its middle, or from the left), each with the relation
// it was in before (this one starts from what that gives) and where it showed (this one adds only the step from there:
// nothing jumps when it comes in, and a hand still moves it); an around's or a row's middle and size where they stand, a
// follow's or a push's leader (the leftmost, the largest) if none is said. Then connected. One step.
static bool doc_relate(Doc& d, int by, const char* ids, double now) {
    const std::string type = doc_word(d, by, "file");
    if (doc_word(d, by, "kind") != "behaviour" || !doc_among(doc_relations, type)) return d.trouble = "that is no relation", false;
    std::vector<int> things;
    std::istringstream said(ids);
    for (std::string one; said >> one;) if (const int t = std::atoi(one.c_str()); t && t != by && doc_knows(d, t) && std::find(things.begin(), things.end(), t) == things.end()) things.push_back(t);
    if (things.empty()) return d.trouble = "no things to put in it", false;
    std::map<int, std::pair<double, double>> at;   // (where each settles: what follows and pushes now is not kept)
    d.settled = true;
    doc_forget(d);
    for (const int t : things) at[t] = {doc_literal(d, t, "x", now, 0.0), doc_literal(d, t, "y", now, 0.0)};
    d.settled = false;
    doc_forget(d);
    const auto has = [&](const char* name) { bool any = false; doc_rows(d, sqlite3_mprintf("select 1 from fact where thing = %d and name = %Q limit 1", by, name), [&](sqlite3_stmt*) { any = true; }); return any; };
    doc_act(d);
    bool ok = true;
    int leader = (int)doc_number(d, by, "leader", 0, 0);
    if (!leader && (type == "follow" || type == "push") && things.size() > 1) {
        leader = *std::min_element(things.begin(), things.end(), [&](int a, int b) { return type == "follow" ? at[a].first < at[b].first : doc_number(d, a, "size", now, 1) > doc_number(d, b, "size", now, 1); });
        ok = ok && doc_set(d, by, "leader", leader);
    }
    if (type == "follow" && !has("lag")) ok = ok && doc_set(d, by, "lag", 0.1);   // (said, so its card shows what it does: each a tenth of a second later)
    if (type == "push" && !has("radius")) ok = ok && doc_set(d, by, "radius", 0.5);
    std::vector<int> members;
    for (const int t : things) if (t != leader) members.push_back(t);
    double cx = 0, cy = 0;
    for (const int t : members) cx += at[t].first / members.size(), cy += at[t].second / members.size();
    if (type == "group")   // (in reading order: the top line first, each line from the left)
        std::sort(members.begin(), members.end(), [&](int a, int b) { return std::round(at[a].second * 10) != std::round(at[b].second * 10) ? at[a].second > at[b].second : at[a].first < at[b].first; });
    if (type == "around" || type == "row") {
        if (has("centre_x")) cx = doc_number(d, by, "centre_x", now, 0), cy = doc_number(d, by, "centre_y", now, 0);
        else ok = ok && doc_set(d, by, "centre_x", cx) && doc_set(d, by, "centre_y", cy);   // (its middle where they stand)
        std::sort(members.begin(), members.end(), [&](int a, int b) { return type == "around" ? std::atan2(at[a].first - cx, at[a].second - cy) < std::atan2(at[b].first - cx, at[b].second - cy) : at[a].first < at[b].first; });
        if (type == "around" && !has("radius")) {
            double r = 0;
            for (const int t : members) r += std::hypot(at[t].first - cx, at[t].second - cy) / members.size();
            ok = ok && doc_set(d, by, "radius", std::max(r, 0.2));
        }
        if (type == "row" && !has("gap") && members.size() > 1) {   // (as far apart as they stand, but never closer than the largest of them is wide)
            double wide = 0;
            for (const int t : members) wide = std::max(wide, doc_number(d, t, "size", now, 1) * std::max(1.0, doc_number(d, t, "wide", now, 1)));
            ok = ok && doc_set(d, by, "gap", std::max((at[members.back()].first - at[members.front()].first) / (members.size() - 1), wide * 1.6));
        }
    }
    std::string order, under, from;
    for (const int t : members) {
        const int before = doc_driven_by(d, t, "x");
        const bool related = before && doc_among(doc_relations, doc_word(d, before, "file"));
        order += (order.empty() ? "" : " ") + std::to_string(t);
        under += (under.empty() ? "" : " ") + std::to_string(related ? before : 0);
        const std::pair<double, double> came = type == "follow" ? std::make_pair(doc_literal(d, leader, "x", now, 0.0), doc_literal(d, leader, "y", now, 0.0)) : at[t];
        char buf[64];
        std::snprintf(buf, sizeof buf, "%.17g %.17g", came.first, came.second);
        from += (from.empty() ? "" : " ") + std::string(buf);
    }
    ok = ok && doc_say(d, by, "members", order.c_str()) && doc_say(d, by, "under", under.c_str()) && doc_say(d, by, "from", from.c_str());
    for (const int t : members) {
        ok = ok && doc_set(d, t, "x_by", by) && doc_set(d, t, "y_by", by) && (!doc_knows(d, t, "z") || doc_set(d, t, "z_by", by));   // (its depth too, where it has one: a ring tipped back, a leader in depth)
        if (type == "around" || type == "row" || type == "group")   // (its turns, size and opacity too: the relation's own, given in turn)
            for (const char* n : {"rotate", "rotate_x", "rotate_y", "size", "opacity"}) ok = ok && (!doc_knows(d, t, n) || doc_set(d, t, (std::string(n) + "_by").c_str(), by));
    }
    return doc_acted(d) && ok;
}
// A relation led by another: the new leader out of its things if it was one, the old one in among them where it shows,
// a follow's start the new leader's settled place; then every thing it touches moved by as much as it would otherwise
// jump. One step; nothing jumps.
static bool doc_lead(Doc& d, int by, int leader, double now) {
    struct Leading { Leading() { doc_leading = true; } ~Leading() { doc_leading = false; } } leading;
    const std::string type = doc_word(d, by, "file");
    if (doc_word(d, by, "kind") != "behaviour" || !doc_among(doc_relations, type)) return d.trouble = "that is no relation", false;
    if (!doc_knows(d, leader) || leader == by) return d.trouble = "a leader is a thing that is there", false;
    const int was = (int)doc_number(d, by, "leader", 0, 0);
    if (was == leader) return true;
    std::vector<int> touched;
    for (const int t : doc_members_of(d, by)) if (t) touched.push_back(t);
    for (const int t : {was, leader}) if (t && std::find(touched.begin(), touched.end(), t) == touched.end()) touched.push_back(t);
    std::map<int, std::pair<double, double>> shown;
    for (const int t : touched) shown[t] = {doc_literal(d, t, "x", now, 0.0), doc_literal(d, t, "y", now, 0.0)};
    d.settled = true;
    doc_forget(d);
    const std::pair<double, double> settles = {doc_literal(d, leader, "x", now, 0.0), doc_literal(d, leader, "y", now, 0.0)};
    const std::pair<double, double> wasAt = was ? std::make_pair(doc_literal(d, was, "x", now, 0.0), doc_literal(d, was, "y", now, 0.0)) : settles;
    d.settled = false;
    doc_forget(d);
    doc_act(d);
    const std::vector<int>& in = doc_members_of(d, by);
    bool ok = std::find(in.begin(), in.end(), leader) == in.end() || doc_free(d, leader, by, now, true);   // (the leader is not one it moves)
    if ((type == "around" || type == "row") && ok)   // (its middle is its leader: the ring or the row goes round it, as planets round a sun)
        ok = doc_set(d, by, "centre_x", 0) && doc_set(d, by, "centre_y", 0);
    ok = ok && doc_set(d, by, "leader", leader);
    if (ok && was && type != "around" && type != "row") {   // (the old leader in among its things, where it shows; a ring's or a row's old middle stays out)
        const int under = doc_driven_by(d, was, "x");
        char add[96];
        std::snprintf(add, sizeof add, " %.17g %.17g", type == "follow" ? settles.first : wasAt.first, type == "follow" ? settles.second : wasAt.second);
        ok = doc_say(d, by, "members", (doc_word(d, by, "members") + " " + std::to_string(was)).c_str())
             && doc_say(d, by, "under", (doc_word(d, by, "under") + " " + std::to_string(under && doc_among(doc_relations, doc_word(d, under, "file")) ? under : 0)).c_str())
             && doc_say(d, by, "from", (doc_word(d, by, "from") + add).c_str()) && doc_set(d, was, "x_by", by) && doc_set(d, was, "y_by", by) && (!doc_knows(d, was, "z") || doc_set(d, was, "z_by", by));
    }
    if (ok && type == "follow") {   // (each starts from where the new leader settles)
        std::string from;
        char one[64];
        for (size_t k = 0, n = doc_members_of(d, by).size(); k < n; ++k) std::snprintf(one, sizeof one, "%s%.17g %.17g", k ? " " : "", settles.first, settles.second), from += one;
        ok = doc_say(d, by, "from", from.c_str());
    }
    doc_forget(d);
    for (const auto& [t, at] : shown) {   // (each moved by what it would jump; a ring or a row goes to its new middle)
        if (!ok || type == "around" || type == "row") break;
        const double ox = at.first - doc_literal(d, t, "x", now, 0.0), oy = at.second - doc_literal(d, t, "y", now, 0.0);
        for (const auto& [name, off] : {std::make_pair("x", ox), std::make_pair("y", oy)})
            if (std::abs(off) > 1e-12)
                ok = ok && doc_change(d, sqlite3_mprintf("update fact set value = value + %!.17g where thing = %d and name = %Q; insert into fact(thing, name, at, value) select %d, %Q, 0.0, %!.17g where not exists (select 1 from fact where thing = %d and name = %Q);", off, t, name, t, name, off, t, name));
        doc_forget(d);
    }
    return doc_acted(d) && ok;
}
// A relation let go: each of its things kept where it shows at that moment, the relation gone.
static bool doc_remove(Doc& d, int thing);
static bool doc_release(Doc& d, int by, double now) {
    if (doc_word(d, by, "kind") != "behaviour") return d.trouble = "that is no relation", false;
    const std::vector<int> all = doc_members_of(d, by);   // (a copy: freeing one forgets the list)
    doc_act(d);   // (one step: all of them freed and the relation gone, or none of it)
    bool ok = true;
    for (const int t : all) if (ok && t) ok = doc_free(d, t, by, now, true);
    ok = ok && doc_remove(d, by);
    return doc_acted(d) && ok;
}
// A value either stays (one fact, at 0, with no way: what `set` writes) or is keyed: a key is a key even alone, and
// a value's first key takes the place of the value that stayed (a key at 0 is marked by its way, Linear until told).
static int doc_keys(Doc& d, int thing, const char* name, double at);
static bool doc_key(Doc& d, int thing, const char* name, double at, double is) {
    if (!(at >= 0.0 && at <= 1e6)) return d.trouble = "a key is at a moment of the stage: from 0 to 1000000 seconds", false;
    if (!doc_knows(d, thing, name, &is, nullptr, true)) return false;
    const bool stayed = !doc_keys(d, thing, name, -1.0) && at != 0.0;   // (the value that stayed goes: the key holds the place now)
    return doc_change(d, sqlite3_mprintf("%z insert into fact(thing, name, at, value, way) values(%d, %Q, %!.17g, %!.17g, %s) on conflict do update set value = excluded.value, way = coalesce(way, excluded.way)",
                                         stayed ? sqlite3_mprintf("delete from fact where thing = %d and name = %Q;", thing, name) : sqlite3_mprintf(""), thing, name, at, is, at == 0.0 ? "'Linear'" : "null"), true);
}
// Many keys of one value at once (doc_key's rule, asked once: the value that stayed goes when keys come), one statement
// made ready and said again for each; one act, the stage made again at its end.
static bool doc_key_many(Doc& d, int thing, const char* name, int n, const double* at, const double* is, const char* way = nullptr) {   // (way: each key's, as a MIDI file's are Hold)
    for (int i = 0; i < n; ++i) {
        double v = is[i];
        if (!(at[i] >= 0.0 && at[i] <= 1e6)) return d.trouble = "a key is at a moment of the stage: from 0 to 1000000 seconds", false;
        if (!doc_knows(d, thing, name, &v, nullptr, true)) return false;
    }
    if (n <= 0) return true;
    doc_act(d);
    const bool stayed = !doc_keys(d, thing, name, -1.0) && std::any_of(at, at + n, [](double a) { return a != 0.0; });
    if (stayed) doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = %Q", thing, name), true);
    sqlite3_stmt* put = nullptr;
    bool ok = d.act_ok && sqlite3_prepare_v2(d.file, "insert into fact(thing, name, at, value, way) values(?, ?, ?, ?, ?) on conflict do update set value = excluded.value, way = coalesce(way, excluded.way)", -1, &put, nullptr) == SQLITE_OK;
    for (int i = 0; ok && i < n; ++i) {
        sqlite3_reset(put);
        sqlite3_bind_int(put, 1, thing), sqlite3_bind_text(put, 2, name, -1, SQLITE_STATIC), sqlite3_bind_double(put, 3, at[i]), sqlite3_bind_double(put, 4, is[i]);
        if (way) sqlite3_bind_text(put, 5, way, -1, SQLITE_STATIC);
        else if (at[i] == 0.0) sqlite3_bind_text(put, 5, "Linear", -1, SQLITE_STATIC);
        else sqlite3_bind_null(put, 5);
        ok = sqlite3_step(put) == SQLITE_DONE;
    }
    if (!ok) d.trouble = sqlite3_errmsg(d.file), d.act_ok = false;
    sqlite3_finalize(put);
    d.act_unmatched = true;   // (matched once, when the act ends)
    return doc_acted(d) && ok;
}
static int doc_keys(Doc& d, int thing, const char* name, double at = -1.0) {   // (how many keys a value has; or whether it has one at a moment)
    int n = 0;
    doc_rows(d, at < 0.0 ? sqlite3_mprintf("select count(*) from fact where thing = %d and name = %Q and " DOC_KEYED, thing, name, thing, name)
                         : sqlite3_mprintf("select count(*) from fact where thing = %d and name = %Q and at = %!.17g", thing, name, at), [&](sqlite3_stmt* row) { n = sqlite3_column_int(row, 0); });
    return n;
}
// What a hand does to a value at a moment, as the document's own rules (a shell does not work them out):
// put: the value changed there. Every change is being keyed (d.keying): a key there, its first as any (After Effects'
//      Auto-keyframe). Not being keyed: a value with keys gets a key there (as After Effects keys a property that has
//      keyframes, wherever it is changed), a value without is the value that stays.
// key_here: a key at that moment holding what the value is there (nothing, if it has one there).
// raise: brought nearer or further back among the things.
// stay: its keys are taken away, and it stays what it is at that moment.
// keyed: 0 the value has no keys, 1 it has keys but none at that moment, 2 it has a key at that moment.
// moving: the names of the values of a thing that can take keys.
static double doc_usual(Doc& d, const char* name);
static bool doc_set(Doc& d, int thing, const char* name, double is);
static bool doc_put(Doc& d, int thing, const char* name, double at, double is) {   // (After Effects' rule)
    if (!doc_knows(d, thing, name, &is)) return false;
    // (a value with keys, or any while every change is keyed: a key there and only there, as After Effects' Auto-keyframe
    //  keys a property at the current time and Alight Motion's diamond keys it at the playhead. What it was is not kept
    //  as a key where the keying began, as the past repository's place_checked did: a change made before that moment
    //  left a key after it that brought the value back, golf (62))
    if (d.keying || doc_keys(d, thing, name) > 0) return doc_key(d, thing, name, at, is);
    return doc_set(d, thing, name, is);
}
// A thing given another to go with (0: none) without moving on the stage, as the sample's Parent seat promises and
// After Effects does: where it is, how far it is turned about the eye's line and how large are said again as the new
// parent sees them (a value with keys is moved whole, by the rule of doc_put; a keyed size is left as it is). A parent
// that leans, turned about x or y, is not undone in the turn: only where the thing is takes it in.
static std::vector<double> doc_key_moments(Doc& d, int thing, const char* name);
static bool doc_parent(Doc& d, int thing, double to) {
    if (!doc_knows(d, thing, "parent", &to)) return false;
    for (int up = (int)to, n = 0; up && n < 64; up = (int)doc_number(d, up, "parent", 0, 0), ++n)   // (a ring of things each going with the next goes nowhere)
        if (up == thing) return d.trouble = "it would go with itself (that one goes with it already)", false;
    const double now = d.stage->now;
    const auto chain = [&](int up, double* turn, double* size) {   // (how far the parents turn it, and how large they make it)
        *turn = 0.0, *size = 1.0;
        for (int n = 0; up && n < 64 && d.shown.count(up); up = (int)doc_number(d, up, "parent", 0, 0), ++n) *turn += doc_number(d, up, "rotate", now, 0), *size *= doc_number(d, up, "size", now, 1);
    };
    double turnWas, sizeWas, turnTo, sizeTo;
    chain((int)doc_number(d, thing, "parent", 0, 0), &turnWas, &sizeWas);
    chain((int)to, &turnTo, &sizeTo);
    // (where it stands at each of its place's keys, and now: every one of them taken over as the hand sees it now, as
    //  After Effects moves a layer's keys whole when it is parented; none is added)
    const char* const xyz[] = {"x", "y", "z"};
    std::map<double, math::double4> stood;
    stood[now] = {doc_number(d, thing, "x", now, 0), doc_number(d, thing, "y", now, 0), doc_number(d, thing, "z", now, 0), 1.0};
    for (const char* name : xyz)
        for (const double at : doc_key_moments(d, thing, name)) stood[at] = {doc_number(d, thing, "x", at, 0), doc_number(d, thing, "y", at, 0), doc_number(d, thing, "z", at, 0), 1.0};
    const math::mat4 was = doc_above(d, thing, now);
    const double size = doc_number(d, thing, "size", now, 1);
    doc_act(d);
    bool ok = doc_change(d, sqlite3_mprintf("insert into fact(thing, name, at, value) values(%d, 'parent', 0.0, %!.17g) on conflict do update set value = excluded.value", thing, to));
    doc_forget(d);   // (the parent read again as now written, not as kept from the last picture: as doc_unrelate reads after its change)
    const math::mat4 into = inverse(doc_above(d, thing, now)) * was;
    for (int k = 0; ok && k < 3; ++k) {
        const std::vector<double> keys = doc_key_moments(d, thing, xyz[k]);
        if (keys.empty()) ok = doc_set(d, thing, xyz[k], (into * stood[now])[k]);
        for (const double at : keys) ok = ok && doc_change(d, sqlite3_mprintf("update fact set value = %!.17g where thing = %d and name = %Q and at = %!.17g", (into * stood[at])[k], thing, xyz[k], at));
    }
    const double turnBy = turnWas - turnTo;
    if (ok && turnBy != 0.0) ok = doc_keys(d, thing, "rotate") ? doc_change(d, sqlite3_mprintf("update fact set value = value + %!.17g where thing = %d and name = 'rotate'", turnBy, thing))
                                                               : doc_set(d, thing, "rotate", doc_number(d, thing, "rotate", now, 0) + turnBy);
    // (a thing with no size, a camera, keeps none: its parent is its Target's layer in the inspector, and was refused for a
    //  size it has not, "a camera has no size": golf P08, golf_noise.md (68); as doc_relations.h skips what a thing has not)
    if (ok && doc_knows(d, thing, "size") && !doc_keys(d, thing, "size")) ok = doc_set(d, thing, "size", std::clamp(size * sizeWas / sizeTo, 1e-3, 1e4));
    return doc_acted(d) && ok;
}
static bool doc_key_here(Doc& d, int thing, const char* name, double at) {
    if (!doc_knows(d, thing, name, nullptr, nullptr, true)) return false;
    return doc_keys(d, thing, name, at) && doc_keys(d, thing, name) ? true : doc_key(d, thing, name, at, doc_number(d, thing, name, at, doc_usual(d, name)));
}
// raise: a thing brought `places` places nearer among the things (below 0: further back): each thing is told where it
// lies now, as one change.
static bool doc_raise(Doc& d, int thing, int places) {
    if (!doc_knows(d, thing)) return false;
    doc_forget(d);   // (the order as the document has it now, a thing just added in the same act too: as doc_relate forgets before it reads)
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
// Pictures in one call (what Cavalry's tutorials build from a shape, a duplicator and a behaviour or two, each named):
// a file each in recipes/ (doc_folders: the library's, the maker's own; a file put there is on the shelf at once). Its head:
// "// Name: what it makes", "// make kind file", then a thing's values, one to a line: "set name number",
// "say name words", "key name seconds number", "then kind file" for another thing; its knobs as script.h says.
// Code below the head is the first thing's script.
struct DocRecipe { std::string name, what, path; std::vector<std::string> lines; bool code = false; };
static std::vector<DocRecipe> doc_recipes_read();
// The folders a kind of file is read from: the library's own (beside the out/ this library is in), then the maker's
// own, as a maker's scripts and presets are theirs (MOTOLII_OWN, or %APPDATA%/Motolii on Windows, or
// ~/Library/Application Support/Motolii on a Mac), each with a folder of that kind in it. The maker's of a name stands.
static std::vector<std::filesystem::path> doc_folders(const char* folder) {
    Dl_info lib{};
    dladdr((const void*)&doc_folders, &lib);
    std::error_code wrong;
    std::vector<std::filesystem::path> at{std::filesystem::weakly_canonical(lib.dli_fname ? lib.dli_fname : ".", wrong).parent_path().parent_path() / folder};
    if (const char* own = std::getenv("MOTOLII_OWN")) at.push_back(std::filesystem::path(own) / folder);
    else if (const char* app = std::getenv("APPDATA")) at.push_back(std::filesystem::path(app) / "Motolii" / folder);
    else if (const char* home = std::getenv("HOME")) at.push_back(std::filesystem::path(home) / "Library" / "Application Support" / "Motolii" / folder);
    return at;
}
// The files of that kind in those folders, by their name (the maker's after the library's: theirs of a name stands).
static std::vector<std::filesystem::path> doc_folder_files(const char* folder, const char* extension) {
    std::map<std::string, std::filesystem::path> by;
    std::error_code wrong;
    for (const auto& dir : doc_folders(folder))
        for (const auto& e : std::filesystem::directory_iterator(dir, wrong))   // (extensions: one, or a few with spaces between)
            if ((" " + std::string(extension) + " ").find(" " + e.path().extension().string() + " ") != std::string::npos && e.path().has_extension()) by[e.path().filename().string()] = e.path();
    std::vector<std::filesystem::path> files;
    for (const auto& [_, f] : by) files.push_back(f);
    return files;
}
// Whether a kind of file is to be read again: once at first, then after the window says a file was put in, taken out or
// written over (files_changed). A window asks at every build.
static bool doc_folder_moved(int& seen) {
    if (seen == files_changed()) return false;
    seen = files_changed();
    return true;
}
static std::vector<DocRecipe> doc_recipes() {
    static std::vector<DocRecipe> all;
    static int seen = -1;
    if (doc_folder_moved(seen)) all = doc_recipes_read();
    return all;
}
