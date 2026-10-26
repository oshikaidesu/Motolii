// motolii_calls.h: part of motolii.mm. Read only through motolii.mm.
static int way_is(Motolii* m, int thing, const char* name, double at, const char* way) {   // (on the hand)
    Doc& d = doc(m);
    bool here = false;
    doc_rows(d, sqlite3_mprintf("select 1 from way where name = %Q", way), [&](sqlite3_stmt*) { here = true; });
    const auto ways = doc_own_ways();
    const auto own = ways.find(way);
    if (here || own == ways.end()) return changes(m, (int)(doc_way(d, thing, name, at, way)));
    doc_act(d);   // (the maker's own way: written into the document and put on the key, one step)
    if (!doc_define(d, way, own->second.c_str()) || !doc_way(d, thing, name, at, way)) d.act_ok = false;
    return changes(m, (int)doc_acted(d));
}
int motolii_way(Motolii* m, int thing, const char* name, double at, const char* way) { return by_hand(m, [=] { return way_is(m, thing, name, at, way); }); }
const char* motolii_effect_names(Motolii* m) { return by_look(m, [m] { return (m->listed = doc_effect_names()).c_str(); }); }
int motolii_effect(Motolii* m, int thing, const char* name, double at) { return by_hand(m, [=] { return changes(m, (int)(doc_effect(doc(m), thing, name, at))); }); }
int motolii_define(Motolii* m, const char* way, const char* expression) { return by_hand(m, [=] { return changes(m, (int)(doc_define(doc(m), way, expression))); }); }
const char* motolii_ways(Motolii* m) { return by_look(m, [m] { return (m->listed = doc_ways(*m->d)).c_str(); }); }
int motolii_value(Motolii* m, int thing, const char* name, double at, double* is) {
    return by_look(m, [=] { return (int)(doc_value(doc(m), thing, name, at, is)); });
}
// A value as the stage shows it: what what drives it (a relation, a behaviour) makes of it, 2; its own, 1; none, 0.
int motolii_shows(Motolii* m, int thing, const char* name, double at, double* is) {
    return by_look(m, [=] {
        if (!doc_value(*m->d, thing, name, at, is)) return 0;
        if (!doc_driven_by(*m->d, thing, name)) return 1;
        *is = doc_literal(*m->d, thing, doc_lasting(name), at, *is);
        return 2;
    });
}
int motolii_things(Motolii* m, int* things, int room) {
    return by_look(m, [=] { const std::vector<int> all = doc_things(*m->d); for (int i = 0; i < room && i < (int)all.size(); ++i) things[i] = all[(size_t)i]; return (int)all.size(); });
}
const char* motolii_word(Motolii* m, int thing, const char* name) { return by_look(m, [=] { return (said_word() = !strcmp(name, "name") ? doc_name(*m->d, thing) : doc_word(*m->d, thing, name)).c_str(); }); }
double motolii_lasts(Motolii* m, int thing) { return by_look(m, [=] { return doc_lasts(*m->d, thing); }); }
int motolii_key_moments(Motolii* m, int thing, const char* name, double* at, int room) {
    return by_look(m, [=] { const std::vector<double> all = doc_key_moments(*m->d, thing, name); for (int i = 0; i < room && i < (int)all.size(); ++i) at[i] = all[(size_t)i]; return (int)all.size(); });
}
// A thing's frame on the picture at the stage's moment, as the stage's own eye sees it: its middle, how wide and how
// high (points of the picture, from the top left), and how far it is turned (degrees, clockwise as seen).
int motolii_frame(Motolii* m, int thing, double* frame) {
    return by_look(m, [=] { return (int)(doc_knows(doc(m), thing) && doc_frame(*m->d, thing, m->now, frame)); });
}
// What of a thing is seen at the stage's moment: the upright box of its outline on the picture (left, top, right, bottom).
int motolii_seen(Motolii* m, int thing, double* box) { return by_look(m, [=] { return (int)(doc_knows(doc(m), thing) && doc_seen_box(*m->d, thing, m->now, box)); }); }
int motolii_path(Motolii* m, int thing, int copy, double from, double to, int n, double* path) {
    return by_look(m, [=] {
        if (!doc_knows(doc(m), thing) || n < 1) return 0;
        for (int i = 0; i < n; ++i) {   // (the stage's own placing, at moments of the preview's own: the stage's moment stays)
            const double at = from + (to - from) * i / n;
            const math::mat4 p = doc_place(*m->d, thing, at, 0, copy, copy > 0 || doc_staggered(*m->d, thing, at));   // (as a copy, or as itself, as the stage places it)
            path[4 * i] = p[3][0], path[4 * i + 1] = p[3][1];
            path[4 * i + 2] = std::atan2(p[0][1], p[0][0]) * 360.0 / (double)TAU, path[4 * i + 3] = length(p[0].xyz);
        }
        return n;
    });
}
// The same, in the round: path[4i..4i+3] = where it is (x, y, z, units of the stage) and how large; for a preview that
// draws it as seen from any side.
int motolii_path3(Motolii* m, int thing, int copy, double from, double to, int n, double* path) {
    return by_look(m, [=] {
        if (!doc_knows(doc(m), thing) || n < 1) return 0;
        for (int i = 0; i < n; ++i) {
            const double at = from + (to - from) * i / n;
            const math::mat4 p = doc_place(*m->d, thing, at, 0, copy, copy > 0 || doc_staggered(*m->d, thing, at));
            path[4 * i] = p[3][0], path[4 * i + 1] = p[3][1], path[4 * i + 2] = p[3][2], path[4 * i + 3] = length(p[0].xyz);
        }
        return n;
    });
}
int motolii_keying(Motolii* m, int on) {   // (on below 0: only says whether every change is keyed: a read, answered at once, not queued behind the hand's picture)
    if (on < 0) return (int)m->keying.load();
    m->keying = on != 0;   // (said at once; the document told in the hand's turn, without waiting)
    return posted(m, "keying", [m, on] { m->d->keying = on != 0, m->d->keying_from = m->now; m->keying = m->d->keying; }), on != 0;
}
int motolii_hold(Motolii* m, int thing) { return by_hand(m, [=] { return (int)doc_hold(doc(m), thing); }); }
int motolii_carry(Motolii* m, const char* by, double a, double b) { return posted(m, std::string("carry ") + by, [m, by = std::string(by), a, b] { doc_carry(doc(m), by.c_str(), a, b); }); }   // (refused: said when it is let go)
int motolii_let_go(Motolii* m, int keep) { return by_hand(m, [=] { return changes(m, (int)doc_let_go(doc(m), keep != 0)); }); }
int motolii_put(Motolii* m, int thing, const char* name, double at, double number) { return by_hand(m, [=] { return changes(m, (int)doc_put(doc(m), thing, name, at, number)); }); }
int motolii_key_here(Motolii* m, int thing, const char* name, double at) { return by_hand(m, [=] { return changes(m, (int)doc_key_here(doc(m), thing, name, at)); }); }
int motolii_size(Motolii* m, int wide, int high) {
    return by_hand(m, [=] {
        char* say = doc_size_words(wide, high);
        return say ? changes(m, (int)doc_change(*m->d, say)) : (m->d->trouble = "a stage is from 16 to 8192 points each way, an even number of them", 0);
    });
}
int motolii_work(Motolii* m, double from, double to) { return by_hand(m, [=] { return changes(m, (int)doc_work_is(doc(m), from, to)); }); }
int motolii_work_of(Motolii* m, double* from, double* to) { return by_look(m, [=] { return doc_work(*m->d, from, to), 1; }); }
// A film's sound: how loud it is, 100 times a second of its own (read once a file, as fast as it decodes), and the
// moments of the document it strikes (grows `rise` dB louder at once), where its film shows them.
static const std::vector<float>* loudness_of(const std::string& file, bool read_it = true) {   // (not read_it: only if read already)
    static std::map<std::string, std::vector<float>> read;
    static std::mutex one;
    std::lock_guard<std::mutex> hold(one);
    auto found = read.find(file);
    if (found == read.end()) {
        if (!read_it) return nullptr;
        found = read.emplace(file, sound_loudness(file.c_str(), 100.0)).first;
    }
    return &found->second;
}
int motolii_loudness(Motolii* m, int thing, float* db, int room) {
    const std::string file = by_look(m, [=] { return doc_knows(doc(m), thing) && doc_word(*m->d, thing, "kind") == "film" ? doc_word(*m->d, thing, "file") : std::string(); });
    if (file.empty()) return 0;
    const std::vector<float>& all = *loudness_of(file);
    for (int i = 0; i < room && i < (int)all.size(); ++i) db[i] = all[(size_t)i];
    return (int)all.size();
}
// How loud a film's sound is at n moments of the document from `from` to before `to`, where its film shows them (dB;
// -100 where it is silent or not shown). -1: its sound is not read yet (motolii_loudness reads it), 0: no sound.
int motolii_loudness_over(Motolii* m, int thing, double from, double to, int n, float* db) {
    const std::string file = by_look(m, [=] { return doc_knows(doc(m), thing) && doc_word(*m->d, thing, "kind") == "film" ? doc_word(*m->d, thing, "file") : std::string(); });
    if (file.empty()) return 0;
    const std::vector<float>* all = loudness_of(file, false);
    if (!all) return -1;
    if (all->empty()) return 0;
    return by_look(m, [&] {   // (the document's moment to the film's own: from its start, from where it begins, at its speed)
        Doc& d = *m->d;
        const double start = doc_number(d, thing, "start", 0, 0), begin = doc_number(d, thing, "begin", 0, 0), speed = doc_number(d, thing, "speed", 0, 1), lasts = doc_number(d, thing, "duration", 0, -1);
        for (int i = 0; i < n; ++i) {
            const double s = from + (to - from) * i / n, own = begin + (s - start) * speed;
            const long k = std::lround(own * 100.0);
            db[i] = s < start || (lasts > 0.0 && s >= start + lasts) || k < 0 || k >= (long)all->size() ? -100.f : (*all)[(size_t)k];
        }
        return n;
    });
}
int motolii_strikes(Motolii* m, int thing, double rise, int own, double* at, int room) {
    const std::string file = by_look(m, [=] { return doc_knows(doc(m), thing) && doc_word(*m->d, thing, "kind") == "film" ? doc_word(*m->d, thing, "file") : std::string(); });
    if (file.empty()) return 0;
    const std::vector<double> struck = sound_strikes(*loudness_of(file), 100.0, rise);
    if (own) {   // (in the film's own seconds, as its loudness is counted)
        for (int i = 0; i < room && i < (int)struck.size(); ++i) at[i] = struck[(size_t)i];
        return (int)struck.size();
    }
    return by_look(m, [&] {   // (the film's own seconds to the document's: from its start, from where it begins, at its speed, while it lasts)
        Doc& d = *m->d;
        const double start = doc_number(d, thing, "start", 0, 0), begin = doc_number(d, thing, "begin", 0, 0), speed = doc_number(d, thing, "speed", 0, 1), lasts = doc_number(d, thing, "duration", 0, -1);
        int n = 0;
        for (const double t : struck) {
            const double when = speed == 0.0 ? -1.0 : start + (t - begin) / speed;
            if (when < start || (lasts > 0.0 && when >= start + lasts)) continue;
            if (n < room) at[n] = when;
            ++n;
        }
        return n;
    });
}
int motolii_mark(Motolii* m, double at, const char* word) { return by_hand(m, [=] { return changes(m, (int)doc_mark(doc(m), at, word)); }); }
int motolii_guide(Motolii* m, const char* axis, double at, int on) { return by_hand(m, [=] { return changes(m, (int)doc_guide(doc(m), axis, at, on != 0)); }); }
const char* motolii_guides(Motolii* m) { return by_look(m, [=] { return (said_word() = doc_guides(*m->d)).c_str(); }); }
const char* motolii_marks(Motolii* m) { return by_look(m, [=] { return (said_word() = doc_marks(*m->d)).c_str(); }); }
int motolii_fits(Motolii* m) {
    return by_look(m, [=] {   // (a read: the document's size against the stage's)
        int wide = m->s->w, high = m->s->h;
        doc_rows(*m->d, sqlite3_mprintf("select name, value from fact where thing = 0 and name in ('wide', 'high')"),
                 [&](sqlite3_stmt* row) { (strcmp((const char*)sqlite3_column_text(row, 0), "wide") ? high : wide) = sqlite3_column_int(row, 1); });
        return (int)(wide == m->s->w && high == m->s->h);
    });
}
int motolii_size_of(Motolii* m, int* wide, int* high) { return by_look(m, [=] { return *wide = m->s->w, *high = m->s->h, 1; }); }
int motolii_raise(Motolii* m, int thing, int places) { return by_hand(m, [=] { return changes(m, (int)doc_raise(doc(m), thing, places)); }); }
int motolii_stay(Motolii* m, int thing, const char* name, double at) { return by_hand(m, [=] { return changes(m, (int)doc_stay(doc(m), thing, name, at)); }); }
int motolii_keyed(Motolii* m, int thing, const char* name, double at) {
    return by_look(m, [=] { return doc_keyed(*m->d, thing, name, at); });
}
const char* motolii_words(Motolii* m, int thing, const char* name) { return by_look(m, [=] { return (said_word() = doc_words(*m->d, thing, name)).c_str(); }); }
// Every moment a key of any of a thing's values that take keys stands at, each once and in time: a timeline's bar
// marks in one call (asked a value at a time, a document of fifteen recipes was hundreds of calls each reading).
int motolii_thing_key_moments(Motolii* m, int thing, double* at, int room) {
    return by_look(m, [=] {   // (one question to SQLite: the moments of the values of it that have keys, as DOC_KEYED says one has)
        int i = 0;
        doc_rows(*m->d, sqlite3_mprintf("select distinct at from fact where thing = %d and name in (select name from fact where thing = %d group by name having count(*) > 1 or max(at) <> 0 or max(way) is not null) order by at", thing, thing), [&](sqlite3_stmt* row) {
            if (i < room) at[i] = sqlite3_column_double(row, 0);
            ++i;
        });
        return i;
    });
}
const char* motolii_moving(Motolii* m, int thing) { return by_look(m, [=] { return (said_word() = doc_moving(*m->d, thing)).c_str(); }); }
const char* motolii_values(Motolii* m, int thing) { return by_look(m, [=] { return (said_word() = doc_has(*m->d, thing)).c_str(); }); }
int motolii_range(Motolii* m, int thing, const char* name, double* low, double* high) { return by_look(m, [=] { return (int)doc_range(*m->d, thing, name, low, high); }); }
const char* motolii_recipes(Motolii* m) { return by_look(m, [=] { return (said_word() = doc_recipe_names()).c_str(); }); }
const char* motolii_effects(Motolii* m) {   // (the effects folder's, then AviUtl2's scripts)
    return by_look(m, [=] {
        const std::string files = doc_folder_names("effects", ".wgsl .fs .mjs .js"), aviutl = doc_aviutl_names();
        return (said_word() = files + (files.empty() || aviutl.empty() ? "" : "\n") + aviutl).c_str();
    });
}
int motolii_look_save(Motolii* m, int thing, const char* path) { return by_look(m, [=] { return (int)doc_look_save(*m->d, thing, path); }); }
int motolii_look_put(Motolii* m, int thing, const char* path) { return by_hand(m, [=] { return changes(m, doc_look_put(doc(m), thing, path)); }); }
const char* motolii_looks(Motolii* m) { return by_look(m, [=] { return (said_word() = doc_look_names()).c_str(); }); }
const char* motolii_animators(Motolii* m) { return by_look(m, [=] { return (said_word() = doc_folder_names("animators", ".js")).c_str(); }); }
int motolii_make(Motolii* m, const char* recipe) { return by_hand(m, [=] { return changes(m, doc_make(doc(m), recipe)); }); }
const char* motolii_shown(Motolii* m, const char* name) {
    return by_look(m, [=] {
        const DocShown* s = doc_shown_of(*m->d, name);
        char line[512] = "";
        if (s) std::snprintf(line, sizeof line, "%s\t%s\t%s\t%g\t%g\t%s", s->look, s->unit, s->label, s->low, s->high, s->fold ? s->fold : "");
        return (said_word() = line).c_str();
    });
}
const char* motolii_cards(Motolii* m, int thing) { return by_look(m, [=] { return (said_word() = doc_cards_of(*m->d, thing)).c_str(); }); }
const char* motolii_fx_titles(Motolii* m, int thing) { return by_look(m, [=] { return (said_word() = doc_fx_titles(*m->d, thing)).c_str(); }); }
const char* motolii_kind_of(const char* file) { return doc_kind_of(file); }
int motolii_sets(const char* font, const char* words) { return type_sets(font, words) ? 1 : 0; }
// The names of the ways a stretch may go (the nine kinds, then the ways defined in this document), with spaces between;
// and how one of the nine goes: how far it is, from 0 at its beginning to 1 at its end, at `u` of the way through.
// A kind's numbers as a hand takes them: each one's name, how low and how high ("seed 1 999 grain 0 0.5 ..."); none: "".
const char* motolii_way_numbers(Motolii*, const char* kind) {   // (the kinds are the maker's, not the document's: nothing waited for)
    static thread_local std::string said;
    said.clear();
    for (const auto& k : interp_kinds) {
        if (strcasecmp(k.name, kind)) continue;
        std::istringstream names(k.names);
        std::string name;
        char one[96];
        for (int j = 0; j < k.numbers && names >> name; ++j) std::snprintf(one, sizeof one, "%s%s %.9g %.9g", said.empty() ? "" : " ", name.c_str(), k.lo[j], k.hi[j]), said += one;
    }
    return said.c_str();
}
const char* motolii_way_names(Motolii* m) {
    return by_look(m, [=] {
        said_word().clear();
        for (const auto& kind : interp_kinds) said_word() += std::string(said_word().empty() ? "" : " ") + kind.name;
        std::set<std::string> here;
        doc_rows(*m->d, sqlite3_mprintf("select name from way order by name"), [&](sqlite3_stmt* row) { here.insert((const char*)sqlite3_column_text(row, 0)); said_word() += std::string(" ") + (const char*)sqlite3_column_text(row, 0); });
        for (const auto& [name, goes] : doc_own_ways()) if (!here.count(name)) said_word() += " " + name;   // (then the maker's own, not yet in this document)
        return said_word().c_str();
    });
}
double motolii_way_at(Motolii* m, const char* way, double u) {
    Interp kind;
    if (interp_said(way, &kind) == 0) return interp_ease(kind, u);
    return by_look(m, [=] {
        bool here = false;
        doc_rows(*m->d, sqlite3_mprintf("select 1 from way where name = %Q", way), [&](sqlite3_stmt*) { here = true; });
        const auto ways = doc_own_ways();
        const auto own = ways.find(way);
        if (!here && own != ways.end()) {   // (the maker's own, not yet in this document: as its file says it)
            const double p[4] = {1, 1, 1, 1};
            sqlite3_stmt* goes = doc_goes(*m->d, own->second);
            return goes ? doc_went(goes, u, u, 1.0, 0.0, 1.0, p) : u;
        }
        return doc_went_by(*m->d, way, u, u, 1.0, 0.0, 1.0);   // (one defined in the document: as between a key of 0 and a key of 1, a second apart)
    });
}
// A film's first picture as a small .png: GStreamer's own line for it (decode, scale, into a .png), held at its first
// picture, which is taken and written. It asks nothing of a stage, so a shell may call it from another thread.
// (fraction: where in the film, 0 to 1; a picture there, not the first: a hand moving over a tile looks along the film)
static int thumbnail_at(const char* film, const char* png, int wide, double fraction) {
    gst_init(nullptr, nullptr);
    gchar* uri = gst_filename_to_uri(film, nullptr);
    if (!uri) return 0;
    gchar* line = g_strdup_printf("uridecodebin uri=\"%s\" ! videoconvert ! videoscale ! video/x-raw,width=%d,pixel-aspect-ratio=1/1 ! pngenc ! appsink name=out", uri, wide);
    GstElement* made = gst_parse_launch(line, nullptr);
    g_free(line), g_free(uri);
    if (!made) return 0;
    GstElement* out = gst_bin_get_by_name(GST_BIN(made), "out");
    gst_element_set_state(made, GST_STATE_PAUSED);
    GstSample* first = gst_app_sink_try_pull_preroll(GST_APP_SINK(out), 4 * GST_SECOND);
    if (first && fraction > 0.0) {
        gint64 length = 0;
        if (gst_element_query_duration(made, GST_FORMAT_TIME, &length) && length > 0) {
            gst_sample_unref(first);
            gst_element_seek_simple(made, GST_FORMAT_TIME, (GstSeekFlags)(GST_SEEK_FLAG_FLUSH | GST_SEEK_FLAG_ACCURATE), (gint64)(std::min(fraction, 0.98) * (double)length));
            first = gst_app_sink_try_pull_preroll(GST_APP_SINK(out), 4 * GST_SECOND);
        }
    }
    bool done = false;
    GstMapInfo bytes;
    if (first && gst_buffer_map(gst_sample_get_buffer(first), &bytes, GST_MAP_READ)) {
        done = g_file_set_contents(png, (const gchar*)bytes.data, (gssize)bytes.size, nullptr);
        gst_buffer_unmap(gst_sample_get_buffer(first), &bytes);
    }
    if (first) gst_sample_unref(first);
    gst_element_set_state(made, GST_STATE_NULL);
    gst_object_unref(out);
    gst_object_unref(made);
    return done;
}
int motolii_thumbnail(const char* film, const char* png, int wide) { return thumbnail_at(film, png, wide, 0.0); }
int motolii_thumbnail_at(const char* film, const char* png, int wide, double fraction) { return thumbnail_at(film, png, wide, fraction); }
// A font file's kinds, as CoreText reads them (its own: the file's family class and traits): serif, sans, script,
// mono, bold, light, italic, symbols, ornaments, space separated; "" for none.
const char* motolii_font_tags(const char* path) {
    static thread_local std::string out;
    out.clear();
    CFURLRef url = CFURLCreateFromFileSystemRepresentation(nullptr, (const UInt8*)path, (CFIndex)std::strlen(path), false);
    CFArrayRef all = url ? CTFontManagerCreateFontDescriptorsFromURL(url) : nullptr;
    if (all && CFArrayGetCount(all) > 0) {
        CTFontDescriptorRef one = (CTFontDescriptorRef)CFArrayGetValueAtIndex(all, 0);
        CFDictionaryRef traits = (CFDictionaryRef)CTFontDescriptorCopyAttribute(one, kCTFontTraitsAttribute);
        if (traits) {
            SInt32 symbolic = 0;
            float weight = 0.f;
            if (CFNumberRef n = (CFNumberRef)CFDictionaryGetValue(traits, kCTFontSymbolicTrait)) CFNumberGetValue(n, kCFNumberSInt32Type, &symbolic);
            if (CFNumberRef n = (CFNumberRef)CFDictionaryGetValue(traits, kCTFontWeightTrait)) CFNumberGetValue(n, kCFNumberFloatType, &weight);
            const uint32_t kind = ((uint32_t)symbolic & kCTFontClassMaskTrait) >> kCTFontClassMaskShift;
            if (kind >= 1 && kind <= 7) out += "serif ";
            else if (kind == 8) out += "sans ";
            else if (kind == 9) out += "ornaments ";
            else if (kind == 10) out += "script ";
            else if (kind == 12) out += "symbols ";
            if ((uint32_t)symbolic & kCTFontMonoSpaceTrait) out += "mono ";
            if ((uint32_t)symbolic & kCTFontBoldTrait || weight > 0.3f) out += "bold ";
            if (weight < -0.3f) out += "light ";
            if ((uint32_t)symbolic & kCTFontItalicTrait) out += "italic ";
            CFRelease(traits);
        }
    }
    if (all) CFRelease(all);
    if (url) CFRelease(url);
    if (!out.empty()) out.pop_back();
    return out.c_str();
}
int motolii_together(Motolii* m, int on) {
    return by_hand(m, [=] {
        if (on) return doc_act(*m->d), m->d->act_together = true, 1;
        const int done = m->d->act ? (int)doc_acted(*m->d) : 0;
        if (!m->d->act) m->d->act_together = false;
        picture_update(m), picture_moment(m);   // (the pictures put off while the act was open, made once)
        return done;
    });
}
// Every stretch of a thing's keyed values, a line each: from, to, its way ("from\tto\tway\n"): a timeline draws how each
// goes between its two keys (the stretch is the inside's own: the window only draws what it says).
const char* motolii_thing_stretches(Motolii* m, int thing) {
    return by_look(m, [=] {
        std::string all;
        doc_rows(*m->d, sqlite3_mprintf("select at, next, way from (select at, lead(at) over (partition by name order by at) as next, coalesce(way, 'Linear') as way from fact where thing = %d and name in (select name from fact where thing = %d group by name having count(*) > 1 or max(at) <> 0 or max(way) is not null)) where next is not null order by at", thing, thing), [&](sqlite3_stmt* row) {
            std::string way = (const char*)sqlite3_column_text(row, 2);
            Interp kind;
            if (interp_said(way, &kind) == 0) way = interp_words(kind);
            char line[64];
            std::snprintf(line, sizeof line, "%.17g\t%.17g\t", sqlite3_column_double(row, 0), sqlite3_column_double(row, 1));
            all += line + way + "\n";
        });
        return (said_word() = all).c_str();
    });
}
const char* motolii_way_of(Motolii* m, int thing, const char* name, double at) { return by_look(m, [=] { return (said_word() = doc_way_of(*m->d, thing, name, at)).c_str(); }); }
// How many points of the picture one unit of the stage is, where things stand when their z is 0.
static double point_of(Motolii* m) {   // (points of the picture to one unit of the stage, at its middle, as the stage's own eye sees it:
    // 6 units off with 30 degrees up and down (motolii_open): the document's scale, whatever camera the stage looks through now)
    return m->s->h / (2.0 * 6.0 * std::tan(15.0 * TAU / 360.0));
}
double motolii_point(Motolii* m) { return by_look(m, [=] { return point_of(m); }); }
// How a hand reads a thing's value: as the value times `times`, and `from` more ("times from"). Places are counted in
// points of the picture from its top left, as After Effects counts them; scale and opacity in hundredths; the turn about
// z clockwise (a camera's turns as they are, its pitch looking down); the anchor in hundredths of the box from its top
// left (front -50 to back 50); a camera's distance in points of the picture (the stage's own eye 6 units off); any other
// value by its look (stage: points, down: points down the picture, share: hundredths), else as it is.
const char* motolii_reading(Motolii* m, int thing, const char* name) {
    return by_look(m, [=] {
        const double p = point_of(m), w = m->s->w, h = m->s->h;
        const bool camera = doc_word(*m->d, thing, "kind") == "camera";
        const std::string n = name;
        double times = 1, from = 0;
        if (n == "x") times = p, from = w / 2;
        else if (n == "y") times = -p, from = h / 2;
        else if (n == "z") times = p;
        else if (n == "scale_x" || n == "scale_y" || n == "scale_z" || n == "opacity" || n == "zoom") times = 100;
        else if (n == "rotate") times = camera ? 1 : -1;
        else if (n == "rotate_x") times = camera ? -1 : 1;
        else if (n == "anchor_x") times = 50, from = 50;
        else if (n == "anchor_y") times = -50, from = 50;
        else if (n == "anchor_z") times = -50;
        else if (n == "distance") times = 6 * p;
        else if (n == "size") times = p;
        else if (const DocShown* s = doc_shown_of(*m->d, name)) {
            const std::string look = s->look;
            times = look == "stage" ? p : look == "down" ? -p : look == "share" ? 100 : 1;
        }
        char line[64];
        std::snprintf(line, sizeof line, "%.17g %.17g", times, from);
        return (said_word() = line).c_str();
    });
}
// A change asked without waiting (a shell's hand that must not stand still while the hand works it): queued in order
// behind every other, a ticket said at once; motolii_answer says whether it is done (-1 not yet, 0 refused: then
// motolii_answer_why says why, 1 done; for add and make, the new thing). The calls: say, set, let_go (number: keep),
// undo, redo, together (number: begin 1, end 0: the end's answer is the step's), add (name: kind, word: file), make
// (word: recipe), duplicate (the answer is the new thing), remove, mark (number: at, word), guide (name: axis, number: at, word "off" takes it away), work
// (number: from, word: to), raise (number: places), sequence (number: overlap), key_here, unkey, stay (number: at), way
// (number: at, word: way), effect (number: at), hold (a hand takes hold of it). A thing -1: the one the last add or make made.
int motolii_later(Motolii* m, const char* call, int thing, const char* name, const char* word, double number) {
    const std::string c = call ? call : "", n = name ? name : "", w = word ? word : "";
    int ticket = 0;
    {
        std::lock_guard<std::mutex> hold(m->asked_lock);
        ticket = ++m->tickets;
        m->lastPosted.clear();
        m->asked.push_back([m, c, n, w, thing = thing, number, ticket]() mutable {
            if (thing == -1) thing = m->made;   // (the thing the last add or make asked later made)
            const long rows = sqlite3_total_changes(m->d->file);
            int ok = 0;
            try {   // (a call that throws is refused with its words, not the end of the hand: the window waits on its ticket)
            if (c == "say") ok = changes(m, (int)doc_say(doc(m), thing, n.c_str(), w.c_str()));
            else if (c == "set") ok = changes(m, (int)doc_set(doc(m), thing, n.c_str(), w == "made" ? m->made : number));   // (word "made": the number is the thing the last add made, as a value that names a thing: a behaviour connected, a leader)
            else if (c == "let_go") ok = changes(m, (int)doc_let_go(doc(m), number != 0.0));
            else if (c == "undo") ok = (int)doc_undo(doc(m));
            else if (c == "redo") ok = (int)doc_redo(doc(m));
            else if (c == "together") { ok = number != 0.0 ? (doc_act(*m->d), m->d->act_together = true, 1) : (m->d->act ? (int)doc_acted(*m->d) : 0); if (!m->d->act) m->d->act_together = false; }
            else if (c == "add") ok = m->made = changes(m, doc_add(doc(m), n.c_str(), w.c_str()));   // (name: the kind, word: the file; the answer is the new thing)
            else if (c == "make") ok = m->made = changes(m, doc_make(doc(m), w.c_str()));
            else if (c == "duplicate") ok = m->made = changes(m, doc_duplicate(doc(m), thing));   // (the answer is the new thing)
            else if (c == "remove") ok = changes(m, (int)doc_remove(doc(m), thing));
            else if (c == "relate") ok = changes(m, (int)doc_relate(doc(m), thing, w.c_str(), m->now));
            else if (c == "fx_file") ok = changes(m, (int)doc_fx_file(doc(m), thing, w.c_str()));   // (an effect file put at the end of the thing's effects, or taken out)
            else if (c == "fx_move") ok = changes(m, (int)doc_fx_move(doc(m), thing, w.c_str(), (int)std::lround(number)));   // (one of its effects moved up or down its list)   // (the things in word put in the relation `thing` as they show now)
            else if (c == "lead") ok = changes(m, (int)doc_lead(doc(m), thing, (int)number, m->now));   // (the relation `thing` led by the thing `number`, nothing moving)
            else if (c == "free") ok = changes(m, (int)doc_free(doc(m), thing, (int)number, m->now, false));   // (a thing out of the relation `number`: back to what is under it, as the relation deleted does; only bake keeps where it shows)
            else if (c == "release") ok = changes(m, (int)doc_release(doc(m), thing, m->now));   // (the relation `thing` let go, each of its things where it shows now)
            else if (c == "mark") ok = changes(m, (int)doc_mark(doc(m), number, w.c_str()));
            else if (c == "guide") ok = changes(m, (int)doc_guide(doc(m), n.c_str(), number, w != "off"));
            else if (c == "work") ok = changes(m, (int)doc_work_is(doc(m), number, std::atof(w.c_str())));
            else if (c == "size") {   // (word: "wide high", the stage's size the document says: opened at it again by the window)
                int wide = 0, high = 0;
                std::sscanf(w.c_str(), "%d %d", &wide, &high);
                char* say = doc_size_words(wide, high);
                ok = say ? changes(m, (int)doc_change(*m->d, say)) : (m->d->trouble = "a stage is from 16 to 8192 points each way, an even number of them", 0);
            }
            else if (c == "raise") ok = changes(m, (int)doc_raise(doc(m), thing, (int)number));
            else if (c == "sequence") ok = changes(m, (int)doc_sequence(doc(m), number, RATE));
            else if (c == "key_here") ok = changes(m, (int)doc_key_here(doc(m), thing, n.c_str(), number));
            else if (c == "key") ok = changes(m, (int)doc_key(doc(m), thing, n.c_str(), std::atof(w.c_str()), number));   // (word: the moment; number: what the key holds)
            else if (c == "unkey") ok = changes(m, (int)doc_unkey(doc(m), thing, n.c_str(), number));
            else if (c == "stay") ok = changes(m, (int)doc_stay(doc(m), thing, n.c_str(), number));
            else if (c == "way") ok = way_is(m, thing, n.c_str(), number, w.c_str());
            else if (c == "effect") ok = changes(m, (int)doc_effect(doc(m), thing, n.c_str(), number));
            else if (c == "pen") {   // (word: where the hand is and its handle, "x y hx hy", points of the picture; the answer is the path shape)
                double x = 0.0, y = 0.0, hx = 0.0, hy = 0.0;
                std::sscanf(w.c_str(), "%lf %lf %lf %lf", &x, &y, &hx, &hy);
                ok = m->made = changes(m, doc_pen(doc(m), thing, x, y, hx, hy));
            }
            else if (c == "live") ok = live_is(m);   // (the stage's live picture opened: answer 1 once it is)
            else if (c == "pick") ok = pick_is(m, thing, (int)number);   // (the thing picked in the window, number: the one under its hand)
            else if (c == "watch") {   // (word: the things the window shows the moving values of, spaced; their values in the moment's picture from the next)
                m->watched.clear();
                std::istringstream in(w);
                for (int t; in >> t;) m->watched.insert(t);
                picture_moment(m);   // (their values at once, not with the next picture)
                ok = 1, m->changed = true;
            }
            else if (c == "desk") {   // (word: "w h x y scale point", as motolii_desk; answer 1 once the desk's picture exists)
                int dw = 0, dh = 0;
                double x = 0.0, y = 0.0, sc = 0.0, pt = 0.0;
                std::sscanf(w.c_str(), "%d %d %lf %lf %lf %lf", &dw, &dh, &x, &y, &sc, &pt);
                ok = desk_is(m, dw, dh, x, y, sc, pt);
            }
            else if (c == "hold") ok = (int)doc_hold(doc(m), thing);   // (a hand takes hold: what it carries next is this thing's)
            else m->trouble = "a change asked later is one of: say, set, hold, let_go, undo, redo, together, add, make, pen, duplicate, remove, mark, guide, work, raise, sequence, key_here, unkey, stay, way, effect, fx_file, fx_move, size, watch, pick, desk, live, relate, lead, free, release";
            } catch (const std::exception& e) {
                ok = 0;
                std::fprintf(stderr, "motolii: %s threw: %s\n", c.c_str(), e.what());
                m->trouble = std::string(c) + " threw: " + e.what();
            }
            if (sqlite3_total_changes(m->d->file) != rows) m->revision.fetch_add(1, std::memory_order_release), picture_update(m), picture_moment(m);   // (the pictures new before the answer: the window reads what is so now)
            m->changed = true;
            { std::lock_guard<std::mutex> hold(m->asked_lock); m->answers[ticket] = {ok, ok ? std::string() : std::string(motolii_trouble(m))}; }
            { std::lock_guard<std::mutex> hold(m->pictureLock); m->pictureTold |= 8; }   // (and the window told an answer is there: motolii_tell, as Thermion's registry is told a request is done)
        });
    }
    m->asked_now.notify_one();
    return ticket;
}
int motolii_answer(Motolii* m, int ticket) {
    std::lock_guard<std::mutex> hold(m->asked_lock);
    const auto found = m->answers.find(ticket);
    if (found == m->answers.end()) return -1;
    const int ok = found->second.first;
    m->answered_why = found->second.second;
    m->answers.erase(found);
    return ok;
}
const char* motolii_answer_why(Motolii* m) { return m->answered_why.c_str(); }
int motolii_undo(Motolii* m) { return by_hand(m, [=] { return (int)(doc_undo(doc(m))); }); }
int motolii_redo(Motolii* m) { return by_hand(m, [=] { return (int)(doc_redo(doc(m))); }); }

double motolii_seconds(Motolii* m) { return by_look(m, [m] { return doc_seconds(*m->d); }); }
int motolii_at(Motolii* m, double seconds) {
    // (sent, not waited for: the window reads what was drawn, at the moment it was drawn (the moment's picture, picture.h),
    //  so it need not wait for the hand to tell this one; a later one still waiting replaces it. Past the document's
    //  ends it is refused there, and said in the trouble)
    return posted(m, "at", [m, seconds] {
        if (!(seconds >= 0.0 && seconds <= doc_seconds(*m->d))) { char runs[80]; std::snprintf(runs, sizeof runs, "the document runs from 0 to %.2f seconds", doc_seconds(*m->d)); m->d->trouble = runs; return; }
        m->now = seconds;   // (told to the stage in the hand's round, the films put there after the document is let go: not here, under its lock)
    });
}
static bool ends(const char* name, const char* with) { return strlen(name) > strlen(with) && !strcmp(name + strlen(name) - strlen(with), with); }
static bool placed(const char* name) { return std::filesystem::is_directory(std::filesystem::absolute(name).parent_path()); }
int motolii_still(Motolii* m, const char* png) {
    return by_hand(m, [m, png] {
        doc(m);
        if (!told(m, ends(png, ".png"), "a still is a .png") || !told(m, placed(png), "there is no such folder")) return 0;
        time_is(*m->s, m->now);
        return told(m, shoot(*m->s, png), "the file could not be written");
    });
}
int motolii_rate(Motolii*) { return RATE; }
// The window's thread watched (Chromium's ThreadRestrictions, in the manner of its DisallowBlocking): while on, any
// change it waits for and any read that waits on the document over 2 ms is kept, named and timed; motolii_broken says
// them all ("" none), one to a line, the last 200. Watched from the opening; turned on again, what was kept is forgotten.
void motolii_watch(Motolii* m, int on) { std::lock_guard<std::mutex> h(m->asked_lock); if (on) m->broke.clear(), m->broken = 0; m->watching = on != 0; }
long motolii_broken_count(Motolii* m) { return m->broken.load(); }   // (no lock: a window may ask it every picture)
const char* motolii_broken(Motolii* m) {
    std::lock_guard<std::mutex> h(m->asked_lock);
    m->listed.clear();
    for (const std::string& one : m->broke) m->listed += (m->listed.empty() ? "" : "\n") + one;
    return m->listed.c_str();
}
int motolii_sequence(Motolii* m, double overlap) { return by_hand(m, [=] { return changes(m, (int)doc_sequence(doc(m), overlap, RATE)); }); }
