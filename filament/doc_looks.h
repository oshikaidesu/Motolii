// doc_looks.h: part of doc.h (a thing's look kept and given to another). Read only through doc.h.
// A look is a document of its own with one thing (thing 1): its facts as the document keeps them (fact: name, at,
// value, way; and the ways they go by), all but what the thing is and holds (kind, file, words) and where it lies
// (x, y, z) and what it hangs on (parent), and its size and turn unless they move (keyed). Kept with SQLite's own
// copying across two files (ATTACH, insert ... select), given back the same way inside one act: the changes SQLite's
// session records, one step of undo as any other. A file it names in the library's folders or the maker's own is kept
// as {motolii}/... or {own}/... (doc_folders), so a look made on one machine finds its effects on another.
// After Effects' animation presets (.ffx: effects and their keys), Alight Motion's Save as Preset and Paste Style.
static const char* const doc_look_kept_out = "'kind','file','text','x','y','z','parent'";
static const char* const doc_look_placed = "'size','scale_x','scale_y','rotate','rotate_x','rotate_y','orient'";   // (kept only when keyed: a pulse is a look, a size is not)

// The library's folder and the maker's own, as a look's words for them say them (doc_folders: the same order).
static std::pair<std::string, std::string> doc_look_roots() {
    const auto at = doc_folders("looks");
    return {at[0].parent_path().string() + "/", at.size() > 1 ? at[1].parent_path().string() + "/" : std::string("\x01")};
}
// The thing's look written as a file (a new document; one there before is written over).
static bool doc_look_save(Doc& d, int thing, const char* path) {
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
        "insert into look.fact select 1, name, at, replace(replace(value, %Q, '{motolii}/'), %Q, '{own}/'), way from main.fact as f where thing = %d and name not in (%s)"
        " and not (name in (%s) and (select count(*) from main.fact where thing = f.thing and name = f.name) = 1);"
        "insert into look.way select * from main.way where name in (select way from look.fact);", roots.first.c_str(), roots.second.c_str(), thing, doc_look_kept_out, doc_look_placed);
    ok = sqlite3_exec(d.file, copy, nullptr, nullptr, nullptr) == SQLITE_OK;
    sqlite3_free(copy);
    if (!ok) d.trouble = sqlite3_errmsg(d.file);
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
    const bool ok = doc_change(d, sqlite3_mprintf(
        "delete from fact where thing = %d and name in (select name from look.fact) and name not in (%s);"
        "insert or replace into fact select %d, name, at, replace(replace(value, '{motolii}/', %Q), '{own}/', %Q), way from look.fact where name not in (%s);"
        "insert or ignore into way select * from look.way;", thing, doc_look_kept_out, thing, roots.first.c_str(), roots.second.c_str(), doc_look_kept_out));
    sqlite3_exec(d.file, "detach database look", nullptr, nullptr, nullptr);
    return ok;
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
