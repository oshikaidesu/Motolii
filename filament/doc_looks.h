// doc_looks.h: part of doc.h (a thing's look kept and given to another). Read only through doc.h.
// A look is a document of its own with one thing (thing 1): its facts as the document keeps them (fact: name, at,
// value, way; and the ways they go by), all but what the thing is and holds (kind, file, words, a shape's figure and path) and where it lies
// (x, y, z) and what it hangs on (parent), and its size and turn unless they move (keyed). Kept with SQLite's own
// copying across two files (ATTACH, insert ... select), given back the same way inside one act: the changes SQLite's
// session records, one step of undo as any other. A file it names in the library's folders or the maker's own is kept
// as {motolii}/... or {own}/... (doc_folders), so a look made on one machine finds its effects on another (words only: a number stays a number).
// After Effects' animation presets (.ffx: effects and their keys), Alight Motion's Save as Preset and Paste Style.
static const char* const doc_look_kept_out = "'kind','file','text','shape','path','x','y','z','parent','element'";   // (what it is: a shape's own figure and drawn path too)
static const char* const doc_look_placed = "'size','scale_x','scale_y','rotate','rotate_x','rotate_y','orient'";   // (kept only when keyed: a pulse is a look, a size is not)
static const char* const doc_look_links = "'parent','mosh_with','copy_on','connect','connect_to','boolean_with','trail','morph_to'";   // (values that are another thing's number: doc_kinds.h)

// The library's folder and the maker's own, as a look's words for them say them (doc_folders: the same order).
static std::pair<std::string, std::string> doc_look_roots() {
    const auto at = doc_folders("looks");
    return {at[0].parent_path().string() + "/", at.size() > 1 ? at[1].parent_path().string() + "/" : std::string("\x01")};
}
// The thing's look written as a file (a new document; one there before is written over); whole: all of it but what it
// hangs on (a copy, Edit > Copy: doc_look_paste makes it a thing again).
// only: those values alone, their names with spaces between (a card's: its effect copied, as Alight Motion copies one).
static bool doc_look_save(Doc& d, int thing, const char* path, bool whole = false, const std::string& only = "") {
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
        " and not (name in (%s) and (select count(*) from main.fact where thing = f.thing and name = f.name) = 1)"
        " and (%Q = '' or instr(' ' || %Q || ' ', ' ' || name || ' ') > 0);"
        "insert into look.way select * from main.way where name in (select way from look.fact);", roots.first.c_str(), roots.second.c_str(), thing, whole ? "'parent'" : doc_look_kept_out, whole ? "''" : doc_look_placed, only.c_str(), only.c_str());
    ok = sqlite3_exec(d.file, copy, nullptr, nullptr, nullptr) == SQLITE_OK;
    sqlite3_free(copy);
    if (!ok) d.trouble = sqlite3_errmsg(d.file);
    // (the behaviours moving it, a wave, a noise, a random, a stagger, carried with it as things 2, 3, ...; a relation,
    //  which ties it to others, is not: its _by left out. Whole, what a group holds too (the things whose parent it is),
    //  and theirs, each telling another carried by its number here: Alight Motion copies a group with what is in it, as
    //  AbsentMotion's group is a scene of its own copied whole, its ElementActions.kt elementNodes)
    std::map<int, int> here{{thing, 1}};   // (a thing of the document: its number in the look)
    std::vector<int> walk{thing};
    int next = 2;
    const auto carry = [&](int was) {
        char* one = sqlite3_mprintf("insert into look.fact select %d, name, at, case typeof(value) when 'text' then replace(replace(value, %Q, '{motolii}/'), %Q, '{own}/') else value end, way from main.fact where thing = %d;"
                                    "insert into look.thing(id) values(%d);", next, roots.first.c_str(), roots.second.c_str(), was, next);
        ok = ok && sqlite3_exec(d.file, one, nullptr, nullptr, nullptr) == SQLITE_OK;
        sqlite3_free(one);
        here[was] = next++;
    };
    for (size_t i = 0; ok && i < walk.size(); ++i) {
        const int at = here[walk[i]];
        std::vector<int> by, held;
        doc_rows(d, sqlite3_mprintf("select distinct cast(value as integer) from look.fact where thing = %d and name like '%%\\_by' escape '\\'", at), [&](sqlite3_stmt* q) { by.push_back(sqlite3_column_int(q, 0)); });
        for (const int b : by) {
            const bool carried = b && doc_word(d, b, "kind") == "behaviour" && doc_among("stagger oscillator noise random", doc_word(d, b, "file"));
            if (carried && !here.count(b)) carry(b);
            char* gone = carried ? nullptr : sqlite3_mprintf("delete from look.fact where thing = %d and name like '%%\\_by' escape '\\' and value = %d;", at, b);
            ok = ok && (!gone || sqlite3_exec(d.file, gone, nullptr, nullptr, nullptr) == SQLITE_OK);
            sqlite3_free(gone);
        }
        if (whole) doc_rows(d, sqlite3_mprintf("select distinct thing from main.fact where name = 'parent' and cast(value as integer) = %d and thing != %d order by thing", walk[i], walk[i]), [&](sqlite3_stmt* q) { held.push_back(sqlite3_column_int(q, 0)); });
        for (const int h : held)
            if (!here.count(h)) carry(h), walk.push_back(h);
    }
    if (ok && here.size() > 1) {   // (each link from one carried to another carried told its number here; a link of a held thing to one not carried, gone)
        std::string cases, carried;
        for (const auto& [was, at] : here) cases += " when " + std::to_string(was) + " then " + std::to_string(at), carried += (carried.empty() ? "" : ",") + std::to_string(was);
        char* told = sqlite3_mprintf("delete from look.fact where thing != 1 and name in (%s) and cast(value as integer) not in (%s);"
                                     "update look.fact set value = case cast(value as integer)%s end where (name like '%%\\_by' escape '\\' or (thing != 1 and name in (%s))) and cast(value as integer) in (%s);",
                                     doc_look_links, carried.c_str(), cases.c_str(), doc_look_links, carried.c_str());
        ok = sqlite3_exec(d.file, told, nullptr, nullptr, nullptr) == SQLITE_OK;
        sqlite3_free(told);
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
// with it (as doc_keys_by moves a thing's), as Ableton Live pastes at the insert marker; the behaviours it carried and
// what a group held made anew after it, in their order, moved in time with it, each link among them told the new
// numbers. One step (with `also`, more said of them in it: SQL, the copy's things as pasted_as says them; `before`,
// said first). into: its first thing one already here, kept as it is, what the copy holds made anew in it.
static int doc_look_paste(Doc& d, const char* path, double now, const char* also = "", int into = 0, const char* before = "") {
    if (d.act) return d.trouble = "a copy is pasted by itself, not within another act", 0;   // (SQLite attaches outside a transaction)
    const auto roots = doc_look_roots();
    char* attach = sqlite3_mprintf("attach database 'file:%q?mode=ro' as look", path);
    const bool attached = sqlite3_exec(d.file, attach, nullptr, nullptr, nullptr) == SQLITE_OK;
    sqlite3_free(attach);
    if (!attached) return d.trouble = sqlite3_errmsg(d.file), 0;
    double was = 0;
    bool kind = false;
    sqlite3_stmt* q = nullptr;
    sqlite3_prepare_v2(d.file, "select name, value from look.fact where thing = 1 and name in ('start', 'kind') and at = 0", -1, &q, nullptr);
    while (q && sqlite3_step(q) == SQLITE_ROW) {
        if (std::string((const char*)sqlite3_column_text(q, 0)) == "start") was = sqlite3_column_double(q, 1);
        else kind = true;
    }
    sqlite3_finalize(q);
    std::string made_all;   // (each thing of the copy a thing anew: its number there, its number here)
    sqlite3_prepare_v2(d.file, "select distinct thing from look.fact order by thing", -1, &q, nullptr);
    while (q && sqlite3_step(q) == SQLITE_ROW) {
        const int k = sqlite3_column_int(q, 0);
        char* one = into && k == 1 ? sqlite3_mprintf("insert into pasted_as values(1, %d);", into) : sqlite3_mprintf("insert into thing default values; insert into pasted_as values(%d, last_insert_rowid());", k);
        made_all += one;
        sqlite3_free(one);
    }
    sqlite3_finalize(q);
    int made = 0;
    const double by = now - was;
    if (!kind) d.trouble = "that is no copy of a thing";
    else if (doc_change(d, sqlite3_mprintf(
                 "create temp table if not exists pasted_as(was integer, id integer); delete from pasted_as;%s%s"
                 "insert into fact select p.id, name, case when exists (select 1 from look.fact as o where o.thing = f.thing and o.name = f.name and (o.at <> 0 or o.way is not null)) then at + %!.17g else at end,"
                 " case when name = 'start' then max(0, value + %!.17g) when typeof(value) = 'text' then replace(replace(value, '{motolii}/', %Q), '{own}/', %Q) else value end, way from look.fact as f join pasted_as as p on p.was = f.thing where p.id != %d;"
                 "insert into fact(thing, name, at, value) select p.id, 'start', 0, max(0, %!.17g) from pasted_as as p where %!.17g > 0 and p.id != %d"   // (one that began at the first moment, said by nothing: begins later as much as the rest)
                 " and not exists (select 1 from look.fact where thing = p.was and name = 'start') and (select value from look.fact where thing = p.was and name = 'kind' and at = 0) <> 'behaviour';"
                 "update fact set value = (select id from pasted_as where was = cast(fact.value as integer)) where thing in (select id from pasted_as) and thing != %d"
                 " and (name like '%%\\_by' escape '\\' or (thing != (select id from pasted_as where was = 1) and name in (%s))) and cast(value as integer) in (select was from pasted_as);"
                 "insert or ignore into way select * from look.way;%s", before, made_all.c_str(), by, by, roots.first.c_str(), roots.second.c_str(), into, by, by, into, into, doc_look_links, also)))
        doc_rows(d, sqlite3_mprintf("select id from pasted_as where was = 1"), [&](sqlite3_stmt* row) { made = sqlite3_column_int(row, 0); });
    sqlite3_exec(d.file, "detach database look", nullptr, nullptr, nullptr);
    return made;
}
// The thing's look kept on the shelf by a name: in the maker's own looks folder (doc_folders), one of that name written
// over (Alight Motion's Save as Preset, After Effects' Save Animation Preset); only: these values alone, an effect's card
// (Alight Motion's preset of one effect), their keys with them.
static bool doc_look_keep(Doc& d, int thing, const std::string& name, const std::string& only = "") {
    if (name.empty() || name.find_first_of("/\\:") != std::string::npos) return d.trouble = "a look's name is a file's name", false;
    std::error_code wrong;
    const std::filesystem::path dir = doc_folders("looks").back();
    std::filesystem::create_directories(dir, wrong);
    std::string named = name;   // (an effect's: the first number its name has not yet, so each kept is another of its presets)
    for (int n = 1; !only.empty() && (named.empty() || named == name || std::filesystem::exists(dir / (named + ".motolii"))); ++n) named = name + " " + std::to_string(n);
    return doc_look_save(d, thing, (dir / (named + ".motolii")).string().c_str(), false, only);
}
// The document and the files it is made of gathered in one folder, to be taken elsewhere whole (Alight Motion's project
// package; After Effects' Collect Files): the document copied as SQLite copies a database (VACUUM INTO), each file a thing
// is made of (its `file`, not one of the library's own nor the maker's: those are found on any machine) copied beside
// it, and the copy told its name there (a word read from the folder it is opened in). How many files were copied (-1: not).
static int doc_pack(Doc& d, const std::string& folder, const std::string& name) {
    std::error_code wrong;
    std::filesystem::create_directories(folder, wrong);
    const std::filesystem::path to = std::filesystem::path(folder) / (name + ".motolii");
    std::filesystem::remove(to, wrong);
    char* copy = sqlite3_mprintf("vacuum into %Q", to.string().c_str());
    const bool copied = sqlite3_exec(d.file, copy, nullptr, nullptr, nullptr) == SQLITE_OK;
    sqlite3_free(copy);
    if (!copied) return d.trouble = sqlite3_errmsg(d.file), -1;
    std::vector<std::string> files;
    doc_rows(d, sqlite3_mprintf("select distinct value from fact where name = 'file' and typeof(value) = 'text'"), [&](sqlite3_stmt* q) { files.push_back((const char*)sqlite3_column_text(q, 0)); });
    const auto roots = doc_look_roots();
    sqlite3* pack = nullptr;
    if (sqlite3_open(to.string().c_str(), &pack) != SQLITE_OK) return sqlite3_close(pack), d.trouble = "the package could not be opened", -1;
    int n = 0;
    std::set<std::string> taken;
    for (const std::string& word : files) {
        const std::filesystem::path was = std::filesystem::absolute(word, wrong);
        const std::string whole = was.string();
        if (!std::filesystem::is_regular_file(was, wrong) || whole.rfind(roots.first, 0) == 0 || whole.rfind(roots.second, 0) == 0) continue;   // (a kind's own word, or the library's: nothing to carry)
        std::string named = was.filename().string();
        for (int k = 2; taken.count(named); ++k) named = was.stem().string() + " " + std::to_string(k) + was.extension().string();   // (two files of one name from two folders)
        taken.insert(named);
        if (!std::filesystem::copy_file(was, std::filesystem::path(folder) / named, std::filesystem::copy_options::overwrite_existing, wrong)) continue;
        char* told = sqlite3_mprintf("update fact set value = %Q where name = 'file' and value = %Q", named.c_str(), word.c_str());
        sqlite3_exec(pack, told, nullptr, nullptr, nullptr);
        sqlite3_free(told);
        ++n;
    }
    sqlite3_close(pack);
    return n;
}

// The looks on the shelf: looks/*.motolii in the library's folder and the maker's own (doc_folders), as the effects
// are ("name\tpath\tlabel\tpicture", one to a line): its name the file's, its picture the .png beside it. Then the
// elements, elements/*.motolii, labelled Element (Alight Motion's My Elements: doc_element_keep).
static std::string doc_look_names() {
    std::string all;
    std::error_code wrong;
    for (const char* kind : {"looks", "elements"})
        for (const auto& f : doc_folder_files(kind, ".motolii")) {
            const std::filesystem::path picture = std::filesystem::path(f).replace_extension(".png");
            all += (all.empty() ? "" : "\n") + f.stem().string() + "\t" + f.string() + (kind[0] == 'l' ? "\tLook\t" : "\tElement\t") + (std::filesystem::exists(picture, wrong) ? picture.string() : "");
        }
    return all;
}
// The thing kept as an element by its name, whole, with what it holds (Alight Motion's Save to My Elements; AbsentMotion's
// saveToMyElements, ElementActions.kt): in the maker's own elements folder, one of that name written over.
static bool doc_element_keep(Doc& d, int thing, const std::string& name) {
    if (name.empty() || name.find_first_of("/\\:") != std::string::npos) return d.trouble = "an element's name is a file's name", false;
    std::error_code wrong;
    const std::filesystem::path dir = doc_folders("elements").back();
    std::filesystem::create_directories(dir, wrong);
    return doc_look_save(d, thing, (dir / (name + ".motolii")).string().c_str(), true);
}
// When an element's file was last written (seconds), as an instance keeps it to know it has changed since.
static double doc_element_written(const std::string& path) {
    std::error_code wrong;
    const auto at = std::filesystem::last_write_time(path, wrong);
    return wrong ? -1.0 : std::chrono::duration<double>(at.time_since_epoch()).count();
}
// What each thing an element put in holds keeps of where it came from (said with the paste, as pasted_as says them):
// which of the element's things it is (element_part), a text's words as the element said them (element_text: words
// of its own since are its own, kept when the element changes: an override, as a Figma instance keeps one), and on
// the first, when the file was written (element_at), its Re-timing and how long it is (element_length).
static std::string doc_element_marks(double written) {
    char* marks = sqlite3_mprintf("insert or replace into fact(thing, name, at, value) select id, 'element_part', 0, was from pasted_as where was != 1;"
                                  "insert or replace into fact(thing, name, at, value) select p.id, 'element_text', 0, f.value from pasted_as as p join look.fact as f on f.thing = p.was and f.name = 'text' and f.at = 0;"
                                  "insert or replace into fact(thing, name, at, value) select id, 'element_at', 0, %!.17g from pasted_as where was = 1;"
                                  "delete from fact where thing = (select id from pasted_as where was = 1) and name in ('retime', 'retime_in', 'retime_out', 'element_length');"   // (and its Re-timing as the element says it, and how long the element is)
                                  "insert into fact(thing, name, at, value) select p.id, f.name, 0, f.value from pasted_as as p join look.fact as f on f.thing = 1 and f.at = 0 and f.name in ('retime', 'retime_in', 'retime_out') where p.was = 1;"
                                  "insert into fact(thing, name, at, value) select p.id, 'element_length', 0, f.value from pasted_as as p join look.fact as f on f.thing = 1 and f.at = 0 and f.name = 'duration' where p.was = 1;", written);
    std::string said = marks;
    sqlite3_free(marks);
    return said;
}
// An element put in at `now`, the newest thing, in the stage's middle, linked to its file (its `element`, the file as a
// look says it: AbsentMotion's addElement, ElementActions.kt, an embedScene at the scene's centre with its link). One step.
static int doc_element_add(Doc& d, const char* path, double now) {
    const auto roots = doc_look_roots();
    std::string word = path;
    for (const auto& [root, said] : {std::pair{roots.first, "{motolii}/"}, std::pair{roots.second, "{own}/"}})
        if (word.rfind(root, 0) == 0) word = said + word.substr(root.size());
    char* also = sqlite3_mprintf("delete from fact where thing = (select id from pasted_as where was = 1) and name in ('x', 'y', 'element');"
                                 "insert into fact(thing, name, at, value) select id, 'x', 0, 0 from pasted_as where was = 1 union all select id, 'y', 0, 0 from pasted_as where was = 1"
                                 " union all select id, 'element', 0, %Q from pasted_as where was = 1;%s", word.c_str(), doc_element_marks(doc_element_written(path)).c_str());
    const int made = doc_look_paste(d, path, now, also);
    sqlite3_free(also);
    return made;
}
// Each element put in whose file was written since follows it (AbsentMotion's ProjectStore.kt withLinkedElements: a
// linked embedScene's scene replaced by the element's as it is now): what it held made anew from the file, it kept
// as it is (where it lies, when it begins), each text whose words are its own since given them again. One step each.
// How many followed.
static int doc_elements_follow(Doc& d) {
    const auto roots = doc_look_roots();
    int n = 0;
    for (const int one : doc_things(d)) {
        std::string path = doc_word(d, one, "element");
        if (path.empty()) continue;
        if (path.rfind("{motolii}/", 0) == 0) path = roots.first + path.substr(10);
        else if (path.rfind("{own}/", 0) == 0) path = roots.second + path.substr(6);
        const double written = doc_element_written(path);
        if (written < 0 || written == doc_number(d, one, "element_at", 0, -1)) continue;
        std::vector<int> held{one};   // (it and all it holds, and theirs)
        std::string gone, kept;
        for (size_t i = 0; i < held.size(); ++i)
            for (const int t : doc_things(d))
                if (t != held[i] && (int)doc_number(d, t, "parent", 0, 0) == held[i] && std::find(held.begin(), held.end(), t) == held.end()) {
                    held.push_back(t);
                    char* g = sqlite3_mprintf("delete from fact where thing = %d;", t);
                    gone += g;
                    sqlite3_free(g);
                    const std::string words = doc_word(d, t, "text");
                    const int part = (int)doc_number(d, t, "element_part", 0, 0);
                    if (part && doc_word(d, t, "kind") == "text" && words != doc_word(d, t, "element_text")) {
                        char* k = sqlite3_mprintf("update fact set value = %Q where thing = (select id from pasted_as where was = %d) and name = 'text' and at = 0;", words.c_str(), part);
                        kept += k;
                        sqlite3_free(k);
                    }
                }
        const std::string also = doc_element_marks(written) + kept;
        n += doc_look_paste(d, path.c_str(), doc_number(d, one, "start", 0, 0), also.c_str(), one, gone.c_str()) == one;
    }
    return n;
}
