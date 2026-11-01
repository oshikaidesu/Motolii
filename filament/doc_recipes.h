// doc_recipes.h: part of doc.h (recipes). Read only through doc.h.
static std::vector<DocRecipe> doc_recipes_read() {
    std::vector<DocRecipe> all;
    for (const auto& f : doc_folder_files("recipes", ".js")) {
        DocRecipe r;
        r.path = f.string();
        std::ifstream in(f);
        std::stringstream text;
        text << in.rdbuf();
        const std::string form = door_form(text.str()), told = door_js(text.str(), form);   // (a form from the net told as a sketch: door.h)
        if (text.str().find("\n// make ") == std::string::npos && script_is_sketch(told)) {   // (a sketch of p5.js as it comes, no head of ours: a solid given it)
            r.name = f.stem().string(), r.what = form.empty() ? "A p5.js sketch" : "A sketch from the net", r.code = told.find("WEBGL") != std::string::npos;
            r.lines = {r.code ? "make solid cube" : "make picture " + r.path};   // (in 3D its solids stand in the world; in 2D what it paints is a picture)
            if (!r.code) r.lines.push_back("fit fit");   // (as large as the picture: doc_fit, After Effects' Fit to Comp)
            all.push_back(r);
            continue;
        }
        in.clear(), in.seekg(0);
        for (std::string line; std::getline(in, line);) {
            if (line.rfind("//", 0) != 0) { if (line.find_first_not_of(" \t\r") != std::string::npos) { r.code = true; break; } continue; }
            line.erase(0, line.find_first_not_of("/ "));
            if (line.rfind("--", 0) == 0) continue;   // (a knob)
            if (r.name.empty()) { const size_t colon = line.find(':'); r.name = line.substr(0, colon); r.what = colon == std::string::npos ? "" : line.substr(line.find_first_not_of(' ', colon + 1)); }
            else if (!line.empty()) r.lines.push_back(line);
        }
        if (!r.name.empty() && !r.lines.empty() && r.lines[0].rfind("make ", 0) == 0) all.push_back(r);
    }
    return all;
}
static std::string doc_recipe_names() {
    std::string all;
    for (const DocRecipe& r : doc_recipes()) all += (all.empty() ? "" : "\n") + r.name + "\t" + r.what;
    return all;
}
// Whether a file the document's pictures are made from (the stage's effects, the things' scripts) has been written
// over since it was read: a stage that is not drawing is woken by it (a file saved shows at once).
static bool doc_files_moved(Doc& d) {
    bool moved = screen_moved(d.screen);
    for (int t : doc_laid(d)) doc_scripts_on(d, t);   // (each looks at its file's time and reads it again if it changed)
    if (DocScripts& kept = doc_scripts_kept(d); kept.made != kept.told) kept.told = kept.made, moved = true, d.scripted.clear();   // (read again here, or by a window asking for its cards first)
    return moved;
}
// The files of a kind in its folders (doc_folders) ("name\tpath\tlabel\tpicture", one to a line): the whole-screen effects (effects/*.wgsl),
// the motions a thing may be given (animators/*.js); a file's name as its first line says it, before the colon; its label
// (what it is for, the shelf's kind) as a head line "// --label:Distort" says it (AviUtl's --label:, its script's menu); its
// picture for the shelf the .png beside it, if there is one ("" none).
// An effect's tile on the shelf, "\tpicture\thome\tsame": its picture (the one drawn for it in
// ~/Library/Caches/Motolii/tiles, with its moving frames beside it, home_0..7.png, as Alight Motion's tiles move; until
// then a .png beside it, as fx_tiles.sh writes the library's; "" none yet), where one drawn for it goes, named by the
// file and when it and the first.tsv beside it were last written, and how the window draws them, as midi_roll_file names
// its rolls (an @ file's script by "file#name"): any of them changed, it is drawn again; and "same" when none of its frames is seen to differ from the sample (home.same; the window
// draws and measures them: skin/book/lib/workspace/effect_tiles.dart).
static std::string doc_tile(const std::string& ref, const std::filesystem::path& beside) {
    std::error_code wrong;
    const std::string file = ref.substr(0, ref.find('#'));
    std::string key = ref;
    if (auto at = std::filesystem::last_write_time(file, wrong); !wrong) key += "|" + std::to_string((long long)at.time_since_epoch().count());
    if (auto at = std::filesystem::last_write_time(std::filesystem::path(file).parent_path() / "first.tsv", wrong); !wrong) key += "|first " + std::to_string((long long)at.time_since_epoch().count());   // (the values it comes with: script_first)
    wrong.clear();
    key += "|tiles 2";   // (how the window draws them, effect_tiles.dart: made again when that changes)
    uint64_t hash = 1469598103934665603ull;
    for (unsigned char c : key) hash = (hash ^ c) * 1099511628211ull;
    const char* home = std::getenv("HOME");
    char name[24];
    std::snprintf(name, sizeof name, "%016llx.png", (unsigned long long)hash);
    const std::string drawn = (std::filesystem::path(home ? home : "/tmp") / "Library/Caches/Motolii/tiles" / name).string();
    const std::string picture = std::filesystem::exists(drawn, wrong) ? drawn : !beside.empty() && std::filesystem::exists(beside, wrong) ? beside.string() : "";
    return "\t" + picture + "\t" + drawn + "\t" + (std::filesystem::exists(drawn + ".same", wrong) ? "same" : "");
}
// What a WGSL effect is, read from it as isf_tags reads an ISF file (the user's Mercari: read, not asked; Shadertoy's
// filters, golf (92)): from its head, Motolii's own lines (screen.h): its passes (--pass@; more than one: multipass;
// one kept from frame to frame, its first flag 1, or --history@: feedback), another picture (--picture@: image); from
// its body, the stage's clock read (.time, .frame: time).
static std::string doc_wgsl_tags(const std::filesystem::path& file) {
    std::ifstream in(file);
    int passes = 0;
    bool feedback = false, image = false, clock = false;
    static const std::regex ticks(R"(\.\s*(time|frame)\b)");   // (read, knobs.time: a field declared and not read is not the clock)
    for (std::string line; std::getline(in, line);) {
        if (line.rfind("//", 0) == 0) {
            if (const size_t at = line.find("--pass@"); at != std::string::npos) {
                ++passes;
                if (const size_t colon = line.find(':', at); colon != std::string::npos && line.compare(colon + 1, 1, "1") == 0) feedback = true;
            }
            feedback = feedback || line.find("--history@") != std::string::npos;
            image = image || line.find("--picture@") != std::string::npos;
        } else if (!clock && std::regex_search(line, ticks)) clock = true;
    }
    std::string tags;
    for (const auto& [on, word] : {std::pair{passes > 1, "multipass"}, {feedback, "feedback"}, {clock, "time"}, {image, "image"}}) if (on) tags += (tags.empty() ? "" : " ") + std::string(word);
    return tags;
}
static std::string doc_folder_names(const char* folder, const char* extension) {
    struct Seen { std::string all; int seen = -1; };
    static std::map<std::string, Seen> seen;
    Seen& one = seen[folder];
    std::string& all = one.all;
    if (!doc_folder_moved(one.seen)) return all;
    all.clear();
    for (const auto& f : doc_folder_files(folder, extension)) {
        const std::string read = isf_file(f.string());   // (an ISF effect: its head as made WGSL; one that cannot be is not shown)
        if (read.empty()) continue;
        std::ifstream in(read);
        std::string first, name = f.stem().string(), label, about;
        if (std::getline(in, first) && first.rfind("// ", 0) == 0 && first.find(':') != std::string::npos) name = first.substr(3, first.find(':') - 3);
        for (std::string line; std::getline(in, line) && line.rfind("//", 0) == 0;) {
            if (const size_t at = line.find("--label:"); at != std::string::npos && label.empty()) label = line.substr(at + 8);
            if (const size_t at = line.find("--about:"); at != std::string::npos && about.empty()) about = line.substr(at + 8);   // (what it does, in its maker's words)
        }
        if (f.extension() == ".mjs") label = "Points";   // (a PlayCanvas splat effect: for a point cloud)
        std::replace(about.begin(), about.end(), '\t', ' ');
        all += (all.empty() ? "" : "\n") + name + "\t" + f.string() + "\t" + label + doc_tile(f.string(), std::filesystem::path(f).replace_extension(".png")) + "\t" + about + "\t" + (f.extension() == ".fs" ? isf_tags(f.string()) : f.extension() == ".wgsl" ? doc_wgsl_tags(f) : "");   // (and what it is, read from it: isf_tags, doc_wgsl_tags)
    }
    return all;
}
// AviUtl2's scripts as their makers give them, a package a folder (aviutl_scripts/<maker>__<package>/, unchanged: a
// script's require reads beside it), in the library's aviutl_scripts/ and the maker's own (doc_folders), to any depth:
// each .anm2 and .obj2 a line as doc_folder_names's, each script of an @ file ("file#name") its own, its label its
// --label:. A file is read again only when it was written over.
static std::string doc_aviutl_names() {
    static std::string all;
    static int seen = -1;
    static std::map<std::string, std::pair<std::filesystem::file_time_type, std::string>> read;   // (each file's lines as it was)
    if (!doc_folder_moved(seen)) return all;
    all.clear();
    std::error_code wrong;
    for (const auto& dir : doc_folders("aviutl_scripts")) {
        std::vector<std::filesystem::path> files;
        for (auto e = std::filesystem::recursive_directory_iterator(dir, wrong); !wrong && e != std::filesystem::recursive_directory_iterator(); e.increment(wrong))
            if (e->is_regular_file(wrong) && aviutl_is(e->path().string())) files.push_back(e->path());
        wrong.clear();
        std::sort(files.begin(), files.end());
        for (const auto& f : files) {
            const auto stamp = std::filesystem::last_write_time(f, wrong);
            auto& [when, lines] = read[f.string()];
            if (lines.empty() || when != stamp) {
                when = stamp, lines.clear();
                std::ifstream in(f);
                const bool many = f.filename().string()[0] == '@';
                std::string name = many ? "" : f.stem().string(), label;
                // (and what each is, read from it as isf_tags reads an ISF file: AviUtl's own kinds by the extension, a custom
                //  object .obj2 a generator and a scene change .scn2 a transition; obj.getaudio: sound; obj.time, obj.frame: time)
                bool sound = false, clock = false;
                const std::string ext = f.extension().string();
                const auto put = [&] {
                    std::string tags;
                    for (const auto& [on, word] : {std::pair{ext == ".scn2", "transition"}, {ext == ".obj2", "generator"}, {sound, "sound"}, {clock, "time"}}) if (on) tags += (tags.empty() ? "" : " ") + std::string(word);
                    if (!name.empty()) lines += name + "\t" + f.string() + (many ? "#" + name : "") + "\t" + label + doc_tile(f.string() + (many ? "#" + name : ""), many ? std::filesystem::path() : std::filesystem::path(f).replace_extension(".png")) + "\t\t" + tags + "\n";
                    sound = clock = false;
                };
                for (std::string line; std::getline(in, line);) {
                    if (line.rfind("\xEF\xBB\xBF", 0) == 0) line.erase(0, 3);
                    line.erase(line.find_last_not_of(" \t\r") + 1);
                    if (many && line.rfind("@", 0) == 0) { put(), name = line.substr(1), label.clear(); continue; }
                    if (label.empty() && line.rfind("--label:", 0) == 0) label = line.substr(8);
                    sound = sound || line.find("obj.getaudio") != std::string::npos;
                    clock = clock || line.find("obj.time") != std::string::npos || line.find("obj.frame") != std::string::npos;
                }
                put();
            }
            all += lines;
        }
    }
    if (!all.empty()) all.pop_back();
    return all;
}
// The ways a maker keeps of their own, as a maker's scripts are theirs: a file each in ways/ of their own folder
// (doc_folders), and the few the library comes with (one put in either is a tile of the Ease desk at once). Its first line says its name
// before the colon; the rest is how far it is, as a way defined in a document says it (doc_define: a, b, u, p1..p4,
// bezier(u, x1, y1, x2, y2), way('Cyclic', u)). Put on a key, it is written into the document, so the document goes as it did wherever it
// is opened. The maker's own of a name goes before the library's.
// One for the whole program, read by every document's hand and every window's look: held by a lock, given as a copy.
static std::mutex& doc_own_ways_lock() { static std::mutex one; return one; }
static std::atomic<int>& doc_own_ways_made() { static std::atomic<int> made{0}; return made; }   // (how many times what was read was not what was before: the window's picture made again, motolii_hand.h)
static std::map<std::string, std::string> doc_own_ways() {
    std::lock_guard<std::mutex> hold(doc_own_ways_lock());
    static std::map<std::string, std::string> kept;
    static int seen = -1;
    if (seen == files_changed()) return kept;   // (looked at again when the window says a file changed: the OS's own watch, GIO's GFileMonitor, tells only while GLib's default main context runs, and none runs here; dart:io's runs in the window)
    seen = files_changed();
    std::map<std::string, std::string> all;
    std::error_code wrong;
    for (const auto& dir : doc_folders("ways"))
        for (const auto& e : std::filesystem::directory_iterator(dir, wrong)) {
            if (e.path().extension() != ".way") continue;
            std::ifstream in(e.path());
            std::string line, name, goes;
            while (std::getline(in, line)) {
                if (line.rfind("//", 0) != 0) { goes += line + " "; continue; }
                if (name.empty() && line.find(':') != std::string::npos) name = line.substr(2, line.find(':') - 2);
            }
            name.erase(0, name.find_first_not_of(' '));
            if (name.empty()) name = e.path().stem().string();
            if (goes.find_first_not_of(' ') != std::string::npos) all[name] = goes;   // (the maker's folder is read last: theirs of a name stands)
        }
    if (all != kept) kept = all, ++doc_own_ways_made();
    return kept;
}
// The recipe of that name made at the stage's present moment, one step (0: not made, d.trouble says why).
static bool doc_fit(Doc& d, int thing, const std::string& how);
static int doc_make(Doc& d, const char* name) {
    std::vector<DocRecipe> all = doc_recipes();
    const DocRecipe* r = nullptr;
    for (const DocRecipe& one : all) if (one.name == name) r = &one;
    if (!r) return d.trouble = std::string("there is no recipe ") + name, 0;
    doc_act(d);
    int now = 0, first = 0, before = 0;   // (the thing the lines tell: "make" the first, "then" another, each named as the recipe)
    double from = 0.0;   // (where its keys count from: "here" says the playhead)
    for (const std::string& line : r->lines) {
        std::istringstream w(line);
        std::string call, value;
        w >> call >> value;
        std::string rest;
        std::getline(w, rest);
        rest.erase(0, rest.find_first_not_of(' '));
        if (call == "make" || call == "then") {
            before = now;
            if (!(now = doc_add(d, value.c_str(), rest.c_str()))) break;
            if (!first) first = now;
            doc_say(d, now, "name", r->name.c_str()), doc_set(d, now, "start", 0);   // (on from the document's beginning, wherever the stage stands: a picture that goes round with it)
            if (now == first && r->code) doc_say(d, now, "script", r->path.c_str());
            continue;
        }
        const int thing = now;
        if (!thing) break;
        if (call == "set") doc_set(d, thing, value.c_str(), rest == "first" ? first : rest == "before" ? before : rest == "behind" ? first - 0.5 : std::atof(rest.c_str()));   // (a thing it goes with: the recipe's first, or the one made before this one; an order just behind the first)
        else if (call == "say") doc_say(d, thing, value.c_str(), value == "script" && rest == "this" ? r->path.c_str() : rest.c_str());   // (another of its things run by the recipe's own code: "this")
        else if (call == "here") from = d.stage->now, doc_set(d, thing, "start", from);   // (it begins where the playhead is, as a clip is placed: not a picture that goes round)
        else if (call == "fit") doc_fit(d, thing, value);
        else if (call == "key") { std::istringstream k(rest); double at = 0, is = 0; k >> at >> is; doc_key(d, thing, value.c_str(), from + at, is); }   // (at that moment of the document, wherever the stage stands: its film loops as made)
    }
    return doc_acted(d) && now ? first : 0;
}
static bool doc_stay(Doc& d, int thing, const char* name, double at) {   // (its keys go: it stays what it is at that moment)
    return doc_knows(d, thing, name) && (!doc_keys(d, thing, name) || doc_set(d, thing, name, doc_number(d, thing, name, at, doc_usual(d, name))));
}
static int doc_keyed(Doc& d, int thing, const char* name, double at) { return !doc_keys(d, thing, name) ? 0 : doc_keys(d, thing, name, at) ? 2 : 1; }
// The words a value of a thing may be (a blend's, a solid's shape), with spaces between ("": any word, or not a word).
static const DocValue* doc_script_value(Doc& d, int thing, const char* name) {   // (a knob of the thing's script)
    if (strncmp(name, "js_", 3)) return nullptr;
    for (const DocScript* sc : doc_scripts_on(d, thing))
        for (const DocValue& one : sc->values) if (!strcmp(one.name, name)) return &one;
    return nullptr;
}
static std::string doc_words(Doc& d, int thing, const char* name) {
    if (const DocValue* v = doc_script_value(d, thing, name)) return v->words ? v->words : "";
    const std::string kind = doc_word(d, thing, "kind");
    if (!strcmp(name, "animation") && kind == "solid") {   // (its file's own movements, as gltfio reads them)
        std::string all;
        if (const auto t = d.shown.find(thing); t != d.shown.end() && t->second.solid)
            for (const std::string& m : t->second.solid->moves) all += (all.empty() ? "" : " ") + m;
        return all;
    }
    if (!strcmp(name, "marker") && kind == "picture") {   // (the named stretches of its file, as ThorVG reads them; any: all of it)
        std::string all = "all";
        if (const auto t = d.shown.find(thing); t != d.shown.end() && t->second.drawn)
            for (const std::string& m : t->second.drawn->markers) all += " " + m;
        return all;
    }
    for (const DocValue& one : doc_values)
        if (!strcmp(one.name, name) && doc_among(one.kinds, kind)) return one.words ? one.words : "";
    return "";
}
// What a thing is called where things are listed: a text by its words, a shape by what shape it is, a file by its name.
static std::string doc_name(Doc& d, int thing) {
    if (const std::string said = doc_word(d, thing, "name"); !said.empty()) return said;
    if (doc_word(d, thing, "kind") == "camera") return "Camera";
    std::string is = doc_word(d, thing, "text");
    if (is.empty()) is = doc_word(d, thing, "shape");
    if (is.empty() && doc_word(d, thing, "kind") == "behaviour") {   // (a behaviour by its kind, a second of a kind told apart by its number, as Blender's .001)
        const std::string file = doc_word(d, thing, "file");
        int nth = 0, all = 0;
        for (const int t : doc_things(d))
            if (doc_word(d, t, "kind") == "behaviour" && doc_word(d, t, "file") == file && doc_word(d, t, "name").empty()) { ++all; if (t <= thing) ++nth; }
        return all > 1 && nth > 1 ? file + " " + std::to_string(nth) : file;
    }
    if (is.empty() && doc_word(d, thing, "kind") == "light") is = doc_word(d, thing, "file");   // (Sun, as the tile says it)
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
    if (doc_among("film picture text shape solid stage camera", kind))   // (and its scripts' knobs that take keys, an effect's tracks, as doc_has lists them: golf_noise.md 44)
        for (const DocScript* sc : doc_scripts_on(d, thing))
            for (const DocValue& one : sc->values) if (one.moves) names += std::string(names.empty() ? "" : " ") + one.name;
    return names;
}
// The names of all the values a thing has (from the values table: those of its kind), and a number's range.
static std::string doc_has(Doc& d, int thing) {
    const std::string kind = doc_word(d, thing, "kind");
    std::string names;
    for (const DocValue& one : doc_values)
        if (doc_among(one.kinds, kind)) names += std::string(names.empty() ? "" : " ") + one.name;
    if (doc_among("film picture text shape solid stage camera", kind))
        for (const DocScript* sc : doc_scripts_on(d, thing))
            for (const DocValue& one : sc->values) names += std::string(names.empty() ? "" : " ") + one.name;
    return names;
}
// The cards of a thing: "title: name name; title: name" (those of its values its kind has; a card with none, left out).
static bool doc_on(Doc& d, int thing, const char* name);
// An effect of a thing's list as its card is called: ThorVG's own by its card's title (Gaussian Blur...), a file by its
// name made a title (halftone -> Halftone).
static std::string doc_fx_title(const std::string& name) {
    if (const std::string source = fx_source(name); source != name) {   // (one put on again: its file's name and its number, "Glow 2", a card of its own)
        const std::string stem = std::filesystem::path(name).stem().string();
        return doc_fx_title(source) + " " + stem.substr(stem.rfind('~') + 1);
    }
    if (doc_thor_letter(name))
        for (const DocCard& card : doc_cards) if (card.when && card.is[0] == '+' && name == card.when) return card.title;
    static std::map<std::string, std::pair<int, std::string>> named;   // (as the shelf names it: its head's own name, its maker's (doc_folder_names); read again once a file has changed)
    if (auto& one = named[name]; one.first == files_changed() && !one.second.empty()) return one.second;
    else if (std::ifstream in(isf_file(name)); in) {
        std::string first;
        if (std::getline(in, first) && first.rfind("// ", 0) == 0 && first.find(':') != std::string::npos) return (one = {files_changed(), first.substr(3, first.find(':') - 3)}).second;
    }
    std::string t = std::filesystem::path(name).stem().string();
    for (char& c : t) if (c == '_') c = ' ';
    if (!t.empty()) t[0] = (char)std::toupper((unsigned char)t[0]);
    return t;
}
// An effect of the thing (by its card's title) put on it again right after it, its values as they are, keys and all:
// Alight Motion's effect card ⋮ Duplicate (AbsentMotion's EffectsPanel: a copy of the effect with its params). Its
// word the file's with ~n before its ending ("Glow~2.fs": read as the file, fx_source), so its values are its own
// (a value's name is its word's: doc_screen_knob). An ISF or a .wgsl file's alone. The new word ("" none).
static std::string doc_fx_duplicate(Doc& d, int thing, const std::string& title) {
    std::vector<std::string> list = script_files(doc_word(d, thing, "fx"));
    const auto at = std::find_if(list.begin(), list.end(), [&](const std::string& w) { return doc_fx_title(w) == title; });
    if (at == list.end()) return d.trouble = "no effect of that name is on it", "";
    const std::string word = *at;
    const std::filesystem::path source(fx_source(word));
    if (source.extension() != ".fs" && source.extension() != ".wgsl") return d.trouble = "only an ISF or a .wgsl effect is put on again so far", "";
    std::string made;
    for (int n = 2; made.empty() || std::find(list.begin(), list.end(), made) != list.end(); ++n) made = (source.parent_path() / (source.stem().string() + "~" + std::to_string(n) + source.extension().string())).string();
    list.insert(at + 1, made);
    std::string fx;
    for (const std::string& w : list) fx += (fx.empty() ? "" : " ") + w;
    const DocScript* script = doc_script_of(d, word);
    doc_act(d);
    bool ok = doc_say(d, thing, "fx", fx.c_str());
    if (script)
        for (const ScriptKnob& k : script->knobs)
            ok = ok && doc_change(d, sqlite3_mprintf("insert or replace into fact select thing, %Q, at, value, way from fact where thing = %d and name = %Q", doc_screen_knob(made, k.name).c_str(), thing, doc_screen_knob(word, k.name).c_str()));
    return doc_acted(d) && ok ? made : "";
}
// The titles of a thing's effect cards in the order its effects are laid ("Halftone;Gaussian Blur").
static std::string doc_fx_titles(Doc& d, int thing) {
    std::string all;
    for (const std::string& w : doc_fx_list(d, thing, true)) all += (all.empty() ? "" : ";") + doc_fx_title(w);   // (those turned off too: their cards are there)
    return all;
}
// (more: those of the "added" rows not put on: what a maker may ask for, not shown unasked)
static std::string doc_cards_of(Doc& d, int thing, std::string* more = nullptr) {
    const std::string has = " " + doc_has(d, thing) + " ";
    // (its effects' cards come under its own (Text, Fill, Transform, Blending...), before the cards of what it may do
    //  besides, Copies and on, as Premiere Pro's Effect Controls lays the effects put on under Motion and Opacity: they
    //  were under ten unused cards, four screens down; golf (58))
    std::string all, besides;
    std::map<std::string, std::string> effects;   // (ThorVG's own effects' cards by the value that puts each on: laid out as its list lays them)
    const bool behaviour = doc_word(d, thing, "kind") == "behaviour";
    bool past = false;
    for (const DocCard& card : doc_cards) {
        past = past || !strcmp(card.title, "Copies");
        if (behaviour && !(card.when && !strcmp(card.when, "file"))) continue;   // (a behaviour: only its own card, not those of things that happen to share a name with its knobs)
        if (card.when && card.is[0] == '+') {   // (an effect's card while it is on: doc_on)
            if (!doc_on(d, thing, card.when)) continue;
        } else if (card.when && card.is[0] == '~') {   // (a ramp's card while the fill has more colours than one)
            if (doc_word(d, thing, card.when).find(card.is + 1) == std::string::npos) continue;
        } else if (card.when) {   // (a card for one fill and not another, or while it is clipped)
            const std::string w = doc_word(d, thing, card.when), want = card.is[0] == '!' ? card.is + 1 : card.is;
            char* end = nullptr;
            const double n = std::strtod(w.c_str(), &end);
            const bool is = w == want || (w.empty() && want == "0") || (!w.empty() && !*end && n == std::atof(want.c_str()) && !want.empty() && isdigit((unsigned char)want[0]));   // (a number not said is 0: a parent not given)
            if (is == (card.is[0] == '!')) continue;
        }
        std::string names;
        // (not there unasked: what a maker sees of a thing is Alight Motion's tiles for its kind (AbsentMotion's LayerTools.kt:
        //  Color & Fill, Blending, Transform, its Edit, Effects; a video's Speed), the rest asked for by More. The stage's own
        //  are asked for as a whole, by Stage settings)
        const std::string kindNow = doc_word(d, thing, "kind");
        const std::string spaceNow = doc_word(d, thing, "space");
        if (!strcmp(card.title, "Surface") && kindNow != "solid" && spaceNow != "3d" && spaceNow != "2.5d" && !doc_on(d, thing, "scale_z")) continue;   // (a flat thing's one value of it is its Shine; in depth it has the rest, After Effects' Material Options on a 3D layer)
        bool said = !card.added || thing == 0 || (!strcmp(card.title, "Play") && kindNow == "film");
        for (const char* at = card.names; *at;) {
            const size_t len = strcspn(at, " ");
            const std::string n(at, len);
            said = said || !doc_word(d, thing, n.c_str()).empty() || doc_keys(d, thing, n.c_str());
            const bool still = (n == "animation" || n == "speed") && doc_word(d, thing, "kind") == "solid" && doc_words(d, thing, "animation").empty();   // (a solid with no movement of its own has none to play)
            if (has.find(" " + n + " ") != std::string::npos && !still) names += (names.empty() ? "" : " ") + n;
            at += len + (at[len] ? 1 : 0);
        }
        if (!names.empty() && !said && more) *more += (more->empty() ? "" : "; ") + std::string(card.title) + ": " + names;
        if (names.empty() || !said) continue;
        if (card.when && card.is[0] == '+' && doc_thor_letter(card.when)) effects[card.when] = std::string(card.title) + ": " + names;
        else (past ? besides : all) += ((past ? besides : all).empty() ? "" : "; ") + std::string(card.title) + ": " + names;
    }
    if (thing != 0)   // (its effects, a card each, in the order they are laid: ThorVG's own and the files' alike)
        for (const std::string& w : doc_fx_list(d, thing, true)) {   // (those turned off too: their values are kept)
            std::string card;
            if (doc_thor_letter(w)) { if (const auto at = effects.find(w); at != effects.end()) card = at->second; }
            else if (const DocScript* sc = doc_script_of(d, w)) {
                for (const DocValue& one : sc->values) card += (card.empty() ? "" : " ") + std::string(one.name);
                if (!card.empty()) card = doc_fx_title(w) + ": " + card;
            }
            if (!card.empty()) all += (all.empty() ? "" : "; ") + card;
        }
    if (!besides.empty()) all += (all.empty() ? "" : "; ") + besides;
    if (doc_among("picture text shape solid stage", doc_word(d, thing, "kind")))   // (its script's own card: the knobs its head declares)
        if (const std::vector<DocScript*> scs = doc_scripts_on(d, thing); !scs.empty()) {
            std::string names = thing == 0 ? "script follow" : "script";
            for (const DocScript* sc : scs)
                if (thing == 0 || sc == doc_script_of(d, doc_word(d, thing, "script")) || (std::filesystem::path(doc_word(d, thing, "file")).extension() == ".js" && sc == doc_script_of(d, doc_word(d, thing, "file"), false)))   // (a thing's effect files have cards of their own, above; a sketch's lines that draw are its picture's)
                for (const DocValue& one : sc->values)
                    if ((" " + names + " ").find(" " + std::string(one.name) + " ") == std::string::npos) names += std::string(" ") + one.name;   // (a knob two effects share is one)
            if (names != "script") all += (all.empty() ? "" : "; ") + std::string("Script: ") + names;   // (none of them with knobs: no card)
        }
    return all;
}
static bool doc_range(Doc& d, int thing, const char* name, double* low, double* high) {
    const std::string kind = doc_word(d, thing, "kind");
    for (const DocValue& one : doc_values)
        if (!strcmp(one.name, name) && doc_among(one.kinds, kind) && !one.words) return *low = one.low, *high = one.high, true;
    if (const DocValue* v = doc_script_value(d, thing, name); v && !v->words) return *low = v->low, *high = v->high, true;
    return false;
}
// Whether a value is on: keyed, or said and not nought (a colour said is on, black too).
static bool doc_on(Doc& d, int thing, const char* name) {
    if (doc_keys(d, thing, name)) return true;
    const std::string w = doc_word(d, thing, name);
    double low = 0, high = 0;
    return !w.empty() && (!doc_range(d, thing, name, &low, &high) || std::atof(w.c_str()) != 0);
}
// The effects a thing may be given (the Effects shelf; their cards are doc_cards' "+" rows): a name, the value that puts
// it on (ThorVG's own), and what it is told then: a number, a word, or keys from that moment ("keys 0 1 1 0": 1 now,
// 0 a second on: Trim Paths drawn on, as After Effects' is mostly used). [with]: the card's other values said with it, as
// it is drawn when they are not said (doc_moment's own), so its card shows what the stage does ("name=number name:word").
struct DocEffect { const char* name; const char* value; const char* on; bool word; const char* label; const char* with = ""; const char* about = ""; };   // (label: what it is for, as an effect file's --label: says; about: what it does, in words the shelf's search reads)
static const DocEffect doc_effects[] = {
    {"Gaussian Blur", "blur", "12", false, "Light"},
    {"Drop Shadow", "shadow", "000000", true, "Light", "shadow_distance=10 shadow_angle=135 shadow_blur=10 shadow_opacity=0.5"},
    {"Glow", "glow", "on", true, "", "glow_opacity=1"},   // (a document that has it; the shelf's Glow is the file am_lightglow.wgsl: golf (79))
    {"Trim Paths", "trim", "keys 0 1 1 0", false, "Path"},
    {"Move Along Path", "copy_on_offset", "keys 0 0 1 1", false, "Path", "stagger=0.01", "Goes along the outline of the layer under it, end to end in a second. Alight Motion's Move Along Path"},   // (one copy of itself, as copies 1 and a stagger lay one on an outline: doc_place)
    {"Datamosh", "mosh", "1", false, "Glitch", "mosh_at=1"},
    {"Duotone", "tint", "1", false, "Color", "tint_black:1b1f5c tint_white:ff8a65"},
    {"Gradient Map", "tritone", "1", false, "Color", "tritone_shadow:1b1f5c tritone_midtone:c44569 tritone_highlight:ffd166"},
    {"Background Copy", "fill", "behind", true, "Utility", "", "Its fill is what is behind it: the effects on it work on the background in its shape. After Effects' Adjustment Layer"},   // (its fill what is behind it: the effects on it then work on the background in its shape, and it still lies on what is under as its blend says; After Effects' adjustment layer as one more effect)
};
// The shelf's effects, "name\tvalue\tlabel\tpicture\t\t\tabout" one to a line (picture: thing_<value>.png in the library's effects/, "" none;
// about: what it does, as a file's description is the 7th of motolii_effects').
static std::string doc_effect_names() {
    std::string all;
    std::error_code wrong;
    for (const DocEffect& e : doc_effects) {
        if (!*e.label) continue;
        const std::filesystem::path picture = doc_folders("effects").front() / ("thing_" + std::string(e.value) + ".png");
        all += (all.empty() ? "" : "\n") + std::string(e.name) + "\t" + e.value + "\t" + e.label + "\t" + (std::filesystem::exists(picture, wrong) ? picture.string() : "") + "\t\t\t" + e.about;
    }
    return all;
}
// An effect put on a thing at a moment, or taken off it if it is on (unsaid, its keys too), one step.
// A thing's list of effects said as it now is (none: unsaid).
static bool doc_fx_say(Doc& d, int thing, const std::vector<std::string>& list) {
    // (a camera has no picture to lay an effect on: it takes AbsentMotion's motions, which move its place, as
    //  AbsentMotion's camera is placed by its own motions (camera.cpp LayerView, styles.cpp OwnAt: ApplyMotion); golf ㊴)
    if (doc_word(d, thing, "kind") == "camera")
        for (const std::string& w : list)
            if (std::filesystem::path(w).extension() != ".am") return d.trouble = "a camera takes motions only (it has no picture of its own): " + doc_fx_title(w) + " is an effect", false;
    if (!list.empty()) return doc_say(d, thing, "fx", doc_fx_join(list).c_str());
    return !doc_knows(d, thing, "fx") || doc_word(d, thing, "fx").empty() || doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = 'fx'", thing));
}
// An effect file put at the end of a thing's effects, or taken out if it is among them (one step). Taken out, its knobs'
// values and keys go with it, as After Effects' and Alight Motion's effect deleted takes its keyframes (golf P15,
// golf_noise.md (67): put on again, the old keys came back); its eye (fx_off) keeps them, as Alight Motion's does.
// A transition put on at a moment (`now`; NAN: none said) runs there: its progress keyed 0 then and 1 a second after, so
// the thing goes and what is under it shows, as After Effects' transition effects take a layer away (Transition
// Completion 0 to 100%) and its transition presets lay their keys from the current time; a second, the editors' own
// length of a transition (Premiere Pro's default, Final Cut Pro's 30 frames), pulled longer or shorter by its keys.
// A transition is ISF's (ISF_Spec): its progress, and the endImage it goes to (golf (65): two keys a value by hand).
static bool doc_fx_file(Doc& d, int thing, const char* file, double now = NAN) {
    std::vector<std::string> list = doc_fx_list(d, thing, true);
    const auto at = std::find(list.begin(), list.end(), file);
    const bool out = at != list.end();
    if (out) list.erase(at);
    else list.push_back(file);
    doc_act(d);
    if (!doc_fx_say(d, thing, list)) d.act_ok = false;
    if (const DocScript* sc = out ? doc_script_of(d, file) : nullptr)
        for (const std::string& n : sc->names)
            if (!doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = %Q", thing, n.c_str()))) d.act_ok = false;
    // (put on at the thing's own start, it comes in instead: 1 to 0, what is under it giving way to it, as Premiere Pro's
    //  transition on a clip's head edge brings the clip in and one on its tail takes it out: P16, the next scene wiped in)
    const bool in = !std::isnan(now) && std::fabs(now - doc_number(d, thing, "start", 0, 0)) < 0.5 / 60.0;   // (within half a frame of the stage's 60, motolii_hand.h RATE)
    if (const DocScript* sc = !out && !std::isnan(now) ? doc_script_of(d, file) : nullptr)
        if (std::any_of(sc->knobs.begin(), sc->knobs.end(), [](const ScriptKnob& k) { return k.name == "endImage"; }))
            for (size_t i = 0; i < sc->knobs.size(); ++i)
                if (sc->knobs[i].name == "progress" && !(doc_key(d, thing, sc->names[i].c_str(), now, in ? 1.0 : 0.0) && doc_key(d, thing, sc->names[i].c_str(), now + 1.0, in ? 0.0 : 1.0))) d.act_ok = false;
    return doc_acted(d);
}
// One of a thing's effects (a file, or ThorVG's own by its value's name) moved `by` places in its list, After Effects'
// dragging an effect up or down its stack (one step).
static bool doc_fx_move(Doc& d, int thing, const char* name, int by) {
    std::vector<std::string> list = doc_fx_list(d, thing, true);
    auto at = std::find(list.begin(), list.end(), name);
    if (at == list.end()) at = std::find_if(list.begin(), list.end(), [&](const std::string& w) { return doc_fx_title(w) == name; });   // (or by its card's title)
    if (at == list.end()) return d.trouble = std::string(name) + " is not among its effects", false;
    const int from = (int)(at - list.begin()), to = std::clamp(from + by, 0, (int)list.size() - 1);
    if (from == to) return true;
    const std::string one = list[(size_t)from];
    list.erase(list.begin() + from);
    list.insert(list.begin() + to, one);
    doc_act(d);
    if (!doc_fx_say(d, thing, list)) d.act_ok = false;
    return doc_acted(d);
}
static bool doc_effect(Doc& d, int thing, const char* name, double at) {
    for (const DocEffect& e : doc_effects) {
        if (strcmp(e.name, name)) continue;
        const bool on = !strcmp(e.value, "fill") ? doc_word(d, thing, "fill") == e.on : doc_on(d, thing, e.value);   // (a fill: on when it is this one)
        std::vector<std::string> list = doc_fx_list(d, thing, true);   // (ThorVG's own in the thing's list: put on at its end, as After Effects does, or out)
        doc_act(d);
        bool ok;
        if (on) ok = doc_knows(d, thing, e.value) && doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = %Q", thing, e.value));   // (unsaid: as it was before it was put on)
        else if (!strncmp(e.on, "keys ", 5)) {
            double a0 = 0, v0 = 0, a1 = 0, v1 = 0;
            ok = std::sscanf(e.on + 5, "%lf %lf %lf %lf", &a0, &v0, &a1, &v1) == 4 && doc_key(d, thing, e.value, at + a0, v0) && doc_key(d, thing, e.value, at + a1, v1);
        } else ok = e.word ? doc_say(d, thing, e.value, e.on) : doc_set(d, thing, e.value, std::atof(e.on));
        std::istringstream with(e.with);
        for (std::string one; !on && ok && with >> one;) {   // (its other values, as they are drawn unsaid)
            const size_t at_ = one.find_first_of("=:");
            if (at_ == std::string::npos) continue;
            const std::string n = one.substr(0, at_), v = one.substr(at_ + 1);
            const bool said = one[at_] == '=' ? !std::isnan(doc_number(d, thing, n.c_str(), at, NAN)) : !doc_word(d, thing, n.c_str()).empty();
            if (!said) ok = one[at_] == '=' ? doc_set(d, thing, n.c_str(), std::atof(v.c_str())) : doc_say(d, thing, n.c_str(), v.c_str());   // (one said already, by a hand or before, is left as it is)
        }
        if (ok && !strcmp(e.value, "copy_on_offset")) {   // (on the layer immediately beneath it, Alight Motion's; off, as it was)
            const std::vector<int> lie = doc_laid(d);
            const auto at_ = std::find(lie.begin(), lie.end(), thing);
            if (on) ok = doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name in ('copy_on', 'stagger')", thing));
            else if (at_ != lie.begin() && at_ != lie.end()) ok = doc_set(d, thing, "copy_on", *(at_ - 1));
        }
        if (ok && doc_thor_letter(e.value)) {
            if (const auto had = std::find(list.begin(), list.end(), e.value); had != list.end()) list.erase(had);
            if (!on) list.push_back(e.value);
            ok = doc_fx_say(d, thing, list);
        }
        if (!ok) d.act_ok = false;
        return doc_acted(d);
    }
    d.trouble = std::string("no effect is called ") + name;
    return false;
}
// One of a thing's effects (by its card's title, or its own name) taken off, as the shelf takes it off when pressed
// again: an effect file out of its list (doc_fx_file), one of ThorVG's own unsaid (doc_effect). Alight Motion's
// Delete on an effect's card (one step).
// One of its effects turned off, or on again (Alight Motion's eye on an effect): by its card's title, its values kept.
static bool doc_fx_eye(Doc& d, int thing, const char* title) {
    const std::vector<std::string> list = doc_fx_list(d, thing, true);
    if (std::none_of(list.begin(), list.end(), [&](const std::string& w) { return doc_fx_title(w) == title; })) return d.trouble = std::string(title) + " is not among its effects", false;
    const bool was = (";" + doc_word(d, thing, "fx_off") + ";").find(";" + std::string(title) + ";") != std::string::npos;
    std::vector<std::string> off;
    std::istringstream said(doc_word(d, thing, "fx_off"));
    for (std::string one; std::getline(said, one, ';');) if (!one.empty() && one != title) off.push_back(one);
    if (!was) off.push_back(title);
    std::string all;
    for (const std::string& one : off) all += (all.empty() ? "" : ";") + one;
    return doc_say(d, thing, "fx_off", all.c_str());
}
static bool doc_fx_out(Doc& d, int thing, const char* name) {
    const std::vector<std::string> list = doc_fx_list(d, thing, true);
    auto at = std::find(list.begin(), list.end(), name);
    if (at == list.end()) at = std::find_if(list.begin(), list.end(), [&](const std::string& w) { return doc_fx_title(w) == name; });
    if (at == list.end()) return d.trouble = std::string(name) + " is not among its effects", false;
    if (!doc_thor_letter(*at)) return doc_fx_file(d, thing, at->c_str());
    for (const DocEffect& e : doc_effects) if (*at == e.value) return doc_effect(d, thing, e.name, 0.0);
    return d.trouble = *at + " is put on by no effect", false;
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
    all += "; by name:";
    for (const auto& one : named_ways) all += std::string(" ") + one.name;
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
    const auto known = d.keyed.find({thing, name});   // (asked again and again between changes: the document asked once)
    if (known != d.keyed.end()) return known->second;
    std::vector<double>& at = d.keyed[{thing, name}];
    doc_rows(d, sqlite3_mprintf("select at from fact where thing = %d and name = %Q and " DOC_KEYED " order by at", thing, name, thing, name),
             [&](sqlite3_stmt* row) { at.push_back(sqlite3_column_double(row, 0)); });
    return at;
}

// Half of how wide and how high a thing is at size 1, before it is placed (a sheet: the sheet's; a solid: its
// bounds'): what a shell draws a thing's frame from.
static bool doc_half_now(Doc& d, int thing, float2* half, float* deep);
static bool doc_half(Doc& d, int thing, float2* half, float* deep) {   // (deep: half of how far it reaches towards the eye and away; a flat thing: 0)
    if (const auto kept = d.halves.find(thing); kept != d.halves.end()) return *half = kept->second.first, deep ? *deep = kept->second.second : 0.f, true;
    float far = 0.f;
    if (!doc_half_now(d, thing, half, &far)) return false;
    d.halves[thing] = {*half, far};
    if (deep) *deep = far;
    return true;
}
static bool doc_half_now(Doc& d, int thing, float2* half, float* deep) {
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
    const double w = std::max(c.w, 1e-6);   // (a point at or behind the eye, a floor's far corners: far off that way, not a number that is none)
    return math::double2{(0.5 + 0.5 * c.x / w) * s.w, (0.5 - 0.5 * c.y / w) * s.h};
}
// A camera as the working eye sees it, the picture's own points, spaced: its frame's four corners, the point it looks at,
// where it is. AbsentMotion's CameraFrame: the picture's rectangle where it looks, as large as the lens takes in there;
// After Effects' point of interest and camera. A point behind the eye: "x". Not by the working eye: nothing.
static std::string doc_gizmo(Doc& d, int thing, double now) {
    if (!d.working || doc_word(d, thing, "kind") != "camera") return "";
    Stage& s = *d.stage;
    const DocEye e = doc_camera_eye(d, thing, now);
    const double high = 6.0 * e.far * std::tan(0.5 * e.fov), wide = high * double(s.w) / double(s.h);
    const math::double3 x = (e.round * math::double4{wide, 0, 0, 0}).xyz, y = (e.round * math::double4{0, high, 0, 0}).xyz;
    std::string said;
    for (const math::double3 p : {e.look - x + y, e.look + x + y, e.look + x - y, e.look - x - y, e.look, e.eye}) {
        const math::double4 c = s.camera->getProjectionMatrix() * s.camera->getViewMatrix() * math::double4{p, 1.0};
        const math::double2 at = doc_seen(d, p);
        said += c.w <= 0.0 ? "x x " : std::to_string(at.x) + " " + std::to_string(at.y) + " ";
    }
    return said;
}
// How many points of the picture one unit of the stage is, where a thing stands (at its own depth).
static double doc_point(Doc& d, int thing, double now) {
    const math::double3 at = (doc_above(d, thing, now) * math::double4{doc_number(d, thing, "x", now, 0), doc_number(d, thing, "y", now, 0), doc_number(d, thing, "z", now, 0), 1.0}).xyz;
    return doc_seen(d, at + math::double3{1, 0, 0}).x - doc_seen(d, at).x;
}
// Where a thing is seen at a moment: as the stage places it (a thing alone that waves or is late, as the member it is
// drawn as); one with copies, where they are laid about.
static math::mat4 doc_shown_at(Doc& d, int thing, double now) { return doc_place(d, thing, now, 0, 0, doc_members(d, thing) == 1 && doc_staggered(d, thing, now)); }
// A thing's frame on the picture at a moment, as the stage's own eye sees it: its middle, how wide and how high (points
// of the picture, from the top left), and how far it is turned (degrees, clockwise as seen). false: it is not shown.
static bool doc_frame(Doc& d, int thing, double now, double* frame) {
    float2 half{0.f};
    float deep = 0.f;
    if (!doc_half(d, thing, &half, &deep) || !d.shown[thing].in) return false;   // (a thing that is not on, or is put aside, has no frame: a hand cannot take hold of what is not seen)
    const double size = doc_number(d, thing, "size", now, 1), point = doc_point(d, thing, now);
    const math::double2 middle = doc_seen(d, (doc_shown_at(d, thing, now) * math::double4{0, 0, 0, 1}).xyz);   // (its own middle, wherever its anchor is)
    const double sx = doc_number(d, thing, "scale_x", now, 1), sy = doc_number(d, thing, "scale_y", now, 1), sz = doc_number(d, thing, "scale_z", now, 1);
    frame[0] = middle.x, frame[1] = middle.y, frame[2] = 2.0 * half.x * size * std::fabs(sx) * point, frame[3] = 2.0 * half.y * size * std::fabs(sy) * point;
    frame[4] = -doc_number(d, thing, "rotate", now, 0) - doc_heading(d, thing, now) * 360.0 / (double)TAU;
    const math::double3 anchor{doc_number(d, thing, "anchor_x", now, 0) * half.x, doc_number(d, thing, "anchor_y", now, 0) * half.y, 0.0};
    const math::double2 pivot = doc_seen(d, (doc_shown_at(d, thing, now) * math::double4{anchor, 1.0}).xyz);   // (the point it turns about)
    frame[5] = pivot.x, frame[6] = pivot.y, frame[7] = 2.0 * deep * size * std::fabs(sz) * point;   // (and how deep it is: a flat thing, 0)
    double box[4];   // (turned about another line than the eye's, its frame is the upright box of what it covers)
    if (doc_number(d, thing, "rotate_x", now, 0) != 0.0 || doc_number(d, thing, "rotate_y", now, 0) != 0.0 || doc_number(d, thing, "parent", 0, 0) != 0.0) {
        double around[128];   // (as wide and high as what it covers: its own points where they are at hand, doc_outline)
        const int n = doc_outline(d, thing, now, around);
        if (n > 2) {
            double x0 = around[0], x1 = x0, y0 = around[1], y1 = y0;
            for (int i = 1; i < n; ++i) x0 = std::min(x0, around[2 * i]), x1 = std::max(x1, around[2 * i]), y0 = std::min(y0, around[2 * i + 1]), y1 = std::max(y1, around[2 * i + 1]);
            frame[2] = 2.0 * std::max(frame[0] - x0, x1 - frame[0]), frame[3] = 2.0 * std::max(frame[1] - y0, y1 - frame[1]), frame[4] = 0.0;   // (its middle stays the thing's own: as far each way as what it covers goes on its farther side)
        } else if (doc_covers(d, thing, now, box)) frame[2] = box[2], frame[3] = box[3], frame[4] = 0.0;
    }
    return true;
}
