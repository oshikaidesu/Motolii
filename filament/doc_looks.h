// doc_looks.h: part of doc.h (a thing's look kept and given to another). Read only through doc.h.
// A look is a document of its own with one thing (thing 1): its facts as the document keeps them (fact: name, at,
// value, way; and the ways they go by), all but what the thing is and holds (kind, file, words, a shape's figure and path) and where it lies
// (x, y, z) and what it hangs on (parent), and its size and turn unless they move (keyed). Kept with SQLite's own
// copying across two files (ATTACH, insert ... select), given back the same way inside one act: the changes SQLite's
// session records, one step of undo as any other. A file it names in the library's folders or the maker's own is kept
// as {motolii}/... or {own}/... (doc_folders), so a look made on one machine finds its effects on another (words only: a number stays a number).
// After Effects' animation presets (.ffx: effects and their keys), Alight Motion's Save as Preset and Paste Style.
static const char* const doc_look_kept_out = "'kind','file','text','shape','path','x','y','z','parent'";   // (what it is: a shape's own figure and drawn path too)
static const char* const doc_look_placed = "'size','scale_x','scale_y','rotate','rotate_x','rotate_y','orient'";   // (kept only when keyed: a pulse is a look, a size is not)

// The library's folder and the maker's own, as a look's words for them say them (doc_folders: the same order).
static std::pair<std::string, std::string> doc_look_roots() {
    const auto at = doc_folders("looks");
    return {at[0].parent_path().string() + "/", at.size() > 1 ? at[1].parent_path().string() + "/" : std::string("\x01")};
}
// The thing's look written as a file (a new document; one there before is written over); whole: all of it but what it
// hangs on (a copy, Edit > Copy: doc_look_paste makes it a thing again).
static bool doc_look_save(Doc& d, int thing, const char* path, bool whole = false) {
    std::error_code wrong;
    std::filesystem::remove(path, wrong);
    char* attach = sqlite3_mprintf("attach database %Q as look", path);
    bool ok = sqlite3_exec(d.file, attach, nullptr, nullptr, nullptr) == SQLITE_OK;
    sqlite3_free(attach);
    if (!ok) return d.trouble = sqlite3_errmsg(d.file), false;
    const auto roots = doc_look_roots();
    char* copy = sqlite3_mprintf(
        "create table look.fact(thing integer, name text, at numeric, value, way text, primary key(thing, name, at)) without rowid;"
        "create table look.way(name text primary key, goes text) without rowid;"
        "create table look.thing(id integer primary key autoincrement); insert into look.thing(id) values(1);"
        "insert into look.fact select 1, name, at, case typeof(value) when 'text' then replace(replace(value, %Q, '{motolii}/'), %Q, '{own}/') else value end, way from main.fact as f where thing = %d and name not in (%s)"
        " and not (name in (%s) and (select count(*) from main.fact where thing = f.thing and name = f.name) = 1);"
        "insert into look.way select * from main.way where name in (select way from look.fact);", roots.first.c_str(), roots.second.c_str(), thing, whole ? "'parent'" : doc_look_kept_out, whole ? "''" : doc_look_placed);
    ok = sqlite3_exec(d.file, copy, nullptr, nullptr, nullptr) == SQLITE_OK;
    sqlite3_free(copy);
    if (!ok) d.trouble = sqlite3_errmsg(d.file);
    // (the behaviours moving it, a wave, a noise, a random, a stagger, carried with it as things 2, 3, ...; a relation,
    //  which ties it to others, is not: its _by left out)
    std::vector<int> by;
    sqlite3_stmt* q = nullptr;
    sqlite3_prepare_v2(d.file, "select distinct cast(value as integer) from look.fact where thing = 1 and name like '%\\_by' escape '\\'", -1, &q, nullptr);
    while (q && sqlite3_step(q) == SQLITE_ROW) by.push_back(sqlite3_column_int(q, 0));
    sqlite3_finalize(q);
    int next = 2;
    for (const int b : by) {
        const bool carried = b && doc_word(d, b, "kind") == "behaviour" && doc_among("stagger oscillator noise random", doc_word(d, b, "file"));
        char* move = carried ? sqlite3_mprintf("insert into look.fact select %d, name, at, value, way from main.fact where thing = %d; insert into look.thing(id) values(%d);"
                                               "update look.fact set value = %d where thing = 1 and name like '%%\\_by' escape '\\' and value = %d;", next, b, next, next, b)
                             : sqlite3_mprintf("delete from look.fact where thing = 1 and name like '%%\\_by' escape '\\' and value = %d;", b);
        ok = ok && sqlite3_exec(d.file, move, nullptr, nullptr, nullptr) == SQLITE_OK;
        sqlite3_free(move);
        next += carried;
    }
    sqlite3_exec(d.file, "detach database look", nullptr, nullptr, nullptr);
    return ok;
}
// A look file given to the thing, as one step: each value the look says is the look's (its keys and ways with it), the
// thing's own of those names gone first; the rest of the thing stays as it was.
static bool doc_look_put(Doc& d, int thing, const char* path) {
    if (d.act) return d.trouble = "a look is given by itself, not within another act", false;   // (SQLite attaches outside a transaction)
    const auto roots = doc_look_roots();
    char* attach = sqlite3_mprintf("attach database 'file:%q?mode=ro' as look", path);
    const bool attached = sqlite3_exec(d.file, attach, nullptr, nullptr, nullptr) == SQLITE_OK;
    sqlite3_free(attach);
    if (!attached) return d.trouble = sqlite3_errmsg(d.file), false;
    std::string carried;   // (each behaviour the look carries made anew here, its own thing, and the thing's _by of it told the new one)
    sqlite3_stmt* q = nullptr;
    sqlite3_prepare_v2(d.file, "select distinct thing from look.fact where thing != 1", -1, &q, nullptr);
    while (q && sqlite3_step(q) == SQLITE_ROW) {
        const int k = sqlite3_column_int(q, 0);
        char* one = sqlite3_mprintf("insert into thing default values; insert into fact select last_insert_rowid(), name, at, value, way from look.fact where thing = %d;"
                                    "update fact set value = (select max(id) from thing) where thing = %d and name in (select name from look.fact where thing = 1 and name like '%%\\_by' escape '\\' and value = %d);", k, thing, k);
        carried += one;
        sqlite3_free(one);
    }
    sqlite3_finalize(q);
    const bool ok = doc_change(d, sqlite3_mprintf(
        "delete from fact where thing = %d and name in (select name from look.fact where thing = 1) and name not in (%s);"
        "insert or replace into fact select %d, name, at, case typeof(value) when 'text' then replace(replace(value, '{motolii}/', %Q), '{own}/', %Q) else value end, way from look.fact where thing = 1 and name not in (%s);"
        "insert or ignore into way select * from look.way;%s", thing, doc_look_kept_out, thing, roots.first.c_str(), roots.second.c_str(), doc_look_kept_out, carried.c_str()));
    sqlite3_exec(d.file, "detach database look", nullptr, nullptr, nullptr);
    return ok;
}
// A copy (doc_look_save whole) made a thing again, the newest number (Edit > Paste): it begins at `now`, its keys moved
// with it (as doc_keys_by moves a thing's), as Ableton Live pastes at the insert marker; the behaviours it carried made
// anew as doc_look_put makes them. One step.
static int doc_look_paste(Doc& d, const char* path, double now) {
    if (d.act) return d.trouble = "a copy is pasted by itself, not within another act", 0;   // (SQLite attaches outside a transaction)
    const auto roots = doc_look_roots();
    char* attach = sqlite3_mprintf("attach database 'file:%q?mode=ro' as look", path);
    const bool attached = sqlite3_exec(d.file, attach, nullptr, nullptr, nullptr) == SQLITE_OK;
    sqlite3_free(attach);
    if (!attached) return d.trouble = sqlite3_errmsg(d.file), 0;
    double was = 0;
    bool kind = false;
    std::string carried;   // (each behaviour it carried a thing anew, the copy's _by of it told the new one)
    sqlite3_stmt* q = nullptr;
    sqlite3_prepare_v2(d.file, "select name, value from look.fact where thing = 1 and name in ('start', 'kind') and at = 0", -1, &q, nullptr);
    while (q && sqlite3_step(q) == SQLITE_ROW) {
        if (std::string((const char*)sqlite3_column_text(q, 0)) == "start") was = sqlite3_column_double(q, 1);
        else kind = true;
    }
    sqlite3_finalize(q);
    sqlite3_prepare_v2(d.file, "select distinct thing from look.fact where thing != 1", -1, &q, nullptr);
    while (q && sqlite3_step(q) == SQLITE_ROW) {
        const int k = sqlite3_column_int(q, 0);
        char* one = sqlite3_mprintf("insert into thing default values; insert into fact select last_insert_rowid(), name, at, value, way from look.fact where thing = %d;"
                                    "update fact set value = (select max(id) from thing) where thing = (select id from pasted) and name like '%%\\_by' escape '\\' and value = %d;", k, k);
        carried += one;
        sqlite3_free(one);
    }
    sqlite3_finalize(q);
    int made = 0;
    const double by = now - was;
    if (!kind) d.trouble = "that is no copy of a thing";
    else if (doc_change(d, sqlite3_mprintf(
                 "insert into thing default values; create temp table if not exists pasted(id integer); delete from pasted; insert into pasted select max(id) from thing;"
                 "insert into fact select (select id from pasted), name, case when name in (select name from look.fact where thing = 1 group by name having count(*) > 1 or max(at) <> 0 or max(way) is not null) then at + %!.17g else at end,"
                 " case when name = 'start' then max(0, value + %!.17g) when typeof(value) = 'text' then replace(replace(value, '{motolii}/', %Q), '{own}/', %Q) else value end, way from look.fact where thing = 1;"
                 "insert or ignore into way select * from look.way;%s", by, by, roots.first.c_str(), roots.second.c_str(), carried.c_str())))
        doc_rows(d, sqlite3_mprintf("select id from pasted"), [&](sqlite3_stmt* row) { made = sqlite3_column_int(row, 0); });
    sqlite3_exec(d.file, "detach database look", nullptr, nullptr, nullptr);
    return made;
}
// The looks on the shelf: looks/*.motolii in the library's folder and the maker's own (doc_folders), as the effects
// are ("name\tpath\tlabel\tpicture", one to a line): its name the file's, its picture the .png beside it.
static std::string doc_look_names() {
    std::string all;
    std::error_code wrong;
    for (const auto& f : doc_folder_files("looks", ".motolii")) {
        const std::filesystem::path picture = std::filesystem::path(f).replace_extension(".png");
        all += (all.empty() ? "" : "\n") + f.stem().string() + "\t" + f.string() + "\tLook\t" + (std::filesystem::exists(picture, wrong) ? picture.string() : "");
    }
    return all;
}
