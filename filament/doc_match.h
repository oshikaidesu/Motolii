// doc_match.h: part of doc.h (the stage made again as the document says; its file opened, changed, undone). Read only through doc.h.
// The values that move a flat shape's points as it is drawn (flat.mat's vertex stage): a shape with any of them cut
// fine all along, one with none only round its curves (a long line in four points, not two thousand).
static const char* const doc_bending = "'warp','bend','twist','pinch','skew','squash','wiggle','stretch','pin_tl_x','pin_tl_y','pin_tr_x','pin_tr_y','pin_bl_x','pin_bl_y','pin_br_x','pin_br_y'";
// A text's words for a column (After Effects' Vertical Type): the signs that lie along a line, in their forms that stand,
// Unicode's own <vertical> table. The long vowel has none there: a stroke as heavy as the letters (no source: a font's vert feature would draw its own, and ThorVG reads no such feature).
static std::string doc_column(std::string words) {
    static const char* const forms[][2] = {{"，", "︐"}, {"、", "︑"}, {"。", "︒"}, {"：", "︓"}, {"；", "︔"}, {"！", "︕"}, {"？", "︖"}, {"〖", "︗"}, {"〗", "︘"}, {"…", "︙"}, {"‥", "︰"}, {"—", "︱"}, {"ー", "丨"}, {"–", "︲"},
                                           {"（", "︵"}, {"）", "︶"}, {"｛", "︷"}, {"｝", "︸"}, {"〔", "︹"}, {"〕", "︺"}, {"【", "︻"}, {"】", "︼"}, {"《", "︽"}, {"》", "︾"}, {"〈", "︿"}, {"〉", "﹀"}, {"「", "﹁"}, {"」", "﹂"}, {"『", "﹃"}, {"』", "﹄"}, {"［", "﹇"}, {"］", "﹈"}};
    for (const auto& form : forms)
        for (size_t at = 0; (at = words.find(form[0], at)) != std::string::npos; at += strlen(form[1])) words.replace(at, strlen(form[0]), form[1]);
    return words;
}

static bool doc_bends(Doc& d, int thing) {
    bool any = false;
    doc_rows(d, sqlite3_mprintf("select 1 from fact where thing = %d and (name in (%s) or (name glob '*_by' and substr(name, 1, length(name) - 3) in (%s))) and (value <> 0 or at <> 0) limit 1", thing, doc_bending, doc_bending),
             [&](sqlite3_stmt*) { any = true; });   // (a bend given by a behaviour bends it too)
    if (d.held.thing == thing)
        for (const auto& [name, is] : d.held.tried) any = any || (is != 0.0 && strstr(doc_bending, ("'" + name + "'").c_str()));
    return any;
}
// A film thing's real datamosh (mosh.h): the film made again with the other's motion poured in, once made; "" none
// (not on, no other film, or not made yet: its own film meanwhile).
static std::string doc_mosh(Doc& d, int thing) {
    if (!doc_on(d, thing, "mosh") || doc_word(d, thing, "kind") != "film") return "";
    const int with = (int)doc_number(d, thing, "mosh_with", 0, 0);
    const std::string file = doc_word(d, thing, "file"), other = with > 0 && with != thing ? doc_word(d, with, "file") : "";
    if (other.empty() || doc_word(d, with, "kind") != "film") return "";
    return mosh_file(file, doc_number(d, thing, "mosh_at", 0, 1), other);
}
// A thing in the lamps' light: as an After Effects 3D layer once its composition has a light; with none, as it is drawn,
// unless it is said to be metal: a metal is what it mirrors, and it mirrors the stage's room (Element 3D's reflection map).
static std::vector<int> doc_things(Doc& d);
// A thing's bevel: which of its ways, 0 none (and none but on a box).
static int doc_bevel(Doc& d, int thing) {
    const std::string way = doc_word(d, thing, "bevel");
    const char* const ways[] = {"angular", "convex", "concave", "ogee", "step"};
    for (int k = 0; k < 5; k++) if (way == ways[k] && doc_on(d, thing, "scale_z")) return k + 1;
    return 0;
}
static bool doc_lit(Doc& d, int thing) {
    const std::string space = doc_word(d, thing, "space");
    if (!doc_among("film text picture shape", doc_word(d, thing, "kind").c_str()) || (space != "3d" && space != "2.5d" && !doc_on(d, thing, "scale_z"))) return false;
    if (doc_on(d, thing, "metallic")) return true;
    for (const int t : doc_things(d)) if (doc_word(d, t, "kind") == "light") return true;
    return false;
}
// A solid is made by its metal, rough and shine; a sheet is told them each moment (doc_moment.h), its picture left alone.
static bool doc_sheet_told(Doc& d, int thing, const std::string& name) {
    return doc_among("metallic roughness reflectance", name.c_str()) && doc_word(d, thing, "kind") != "solid";
}
static bool doc_match(Doc& d) {
    Stage& s = *d.stage;
    s.frames = doc_number(d, 0, "frame_rate", 0, 0);   // (the document's frame rate: its time seen at its frames from the next picture on)
    std::map<int, std::string> told;   // every thing of the document, and all that its making goes by
    doc_rows(d, sqlite3_mprintf("select thing, group_concat(name || '=' || value, ' ') from (select * from fact where thing > 0 and (name in (%s) or name glob '*_by') and not (name in ('metallic', 'roughness', 'reflectance') and thing in (select thing from fact where name = 'kind' and value != 'solid')) order by thing, name) group by thing", doc_making),
             [&](sqlite3_stmt* row) { told[sqlite3_column_int(row, 0)] = (const char*)sqlite3_column_text(row, 1); });
    for (auto& [thing, of] : told)   // (one way that reads what is under is made as any other: going from one to another tells the look, it makes nothing again)
        if (const size_t at = of.find("blend="); at != std::string::npos) {
            const size_t end = of.find(' ', at);
            if (doc_blend_way(of.substr(at + 6, end == std::string::npos ? std::string::npos : end - at - 6))) of.replace(at + 6, (end == std::string::npos ? of.size() : end) - at - 6, "*");
        }
    for (auto& [thing, of] : told) of += doc_bends(d, thing) ? " bent=1" : " bent=0";   // (bent or not: cut fine, or only round its curves)
    for (auto& [thing, of] : told) {   // (its effects: which are on and the room they reach, not their numbers (doc_fxing))
        const auto on = [&](const char* name) { return std::max(doc_most(d, thing, name, 0), doc_number(d, thing, name, 0, 0)) > 0.0; };
        if (doc_on(d, thing, "mosh")) of += " moshed=" + doc_mosh(d, thing);   // (made again once its datamosh is made)
        if (doc_on(d, thing, "scale_z")) of += " deep";   // (made once more as it is given depth: its sheet a box)
        if (doc_bevel(d, thing)) of += " bevel";
        if (doc_lit(d, thing)) of += " lit";   // (and as a lamp comes or goes)
        if (of.find("kind=text") != std::string::npos) of += " shown=" + doc_text_shown(d, thing, s.now);   // (its words as AbsentMotion's text effects show them now: made again as they change (doc_moment))
        of += " order=" + doc_fx_join(doc_fx_list(d, thing)) + " fx=" + std::to_string((int)on("blur")) + (doc_word(d, thing, "shadow").empty() ? "0" : "1") + (doc_word(d, thing, "glow").empty() ? "0" : "1") + std::to_string((int)on("tint")) + std::to_string((int)on("tritone"))
            + " room=" + std::to_string(doc_fx_room(d, thing));
    }
    if (const auto held = told.find(d.held.thing); held != told.end())   // (what a hand carries of its making shows too: it is made again as it would be)
        for (const auto& [name, is] : d.held.said) if (name[0] != ' ') held->second += " " + name + "~" + is;
    if (const auto held = told.find(d.held.thing); held != told.end())
        for (const auto& [name, is] : d.held.tried)
            if (strstr(doc_making, ("'" + name + "'").c_str()) && !doc_sheet_told(d, d.held.thing, name)) {   // (one that moves is drawn again each moment anyway (doc_moment): made again only as it comes on or goes off)
                bool moves = false;
                for (const DocValue& one : doc_values) moves = moves || (one.moves && name == one.name);
                const auto from = d.held.from.find(name);
                if (!moves) held->second += " " + name + "~" + std::to_string(is);
                else if (from != d.held.from.end() && (from->second != 0.0) != (is != 0.0)) held->second += " " + name + "~" + (is != 0.0 ? "on" : "off");
            }
    for (auto it = d.shown.begin(); it != d.shown.end();) {
        const auto still = told.find(it->first);
        if (still != told.end() && still->second == it->second.of && !(it->second.drawn && it->second.drawn->outgrown)) { ++it; continue; }   // (a text told words wider than its picture: made again however it reads)
        if (it->second.scene && it->second.scene != s.scene)   // (out of the scene of its run first: the kit takes things out of the stage's own only)
            doc_entities(s, it->second, [&](utils::Entity e) { it->second.scene->remove(e); });
        if (const Mesh* m = it->second.film ? it->second.film->sheet : it->second.drawn ? it->second.drawn->sheet : nullptr)   // (its copies' own looks first, while its copies are still known)
            doc_copy_looks_off(d, m, it->second.film ? it->second.film->look : it->second.drawn->look);
        if (const Mesh* m = it->second.film ? it->second.film->sheet : it->second.drawn ? it->second.drawn->sheet : it->second.shape)   // (and the shape it cuts a matte by: done away with first)
            doc_unshape(s, [m](const Stage::Made& made) { return made.of == m; });
        if (it->second.film) video_off(s, it->second.film);
        if (it->second.drawn) picture_off(s, it->second.drawn);
        for (const auto& deep : it->second.deep) picture_off(s, deep.sheet);
        if (it->second.solid) solid_off(s, it->second.solid);
        if (!it->second.light.isNull()) {   // (out of every scene, and done away with)
            s.scene->remove(it->second.light);
            s.engine->getLightManager().destroy(it->second.light), utils::EntityManager::get().destroy(it->second.light);
        }
        if (it->second.shape) s.playing.erase(it->second.shape), take_off(s, it->second.shape);   // (a cloud's sort goes with it)
        it = d.shown.erase(it);
    }
    bool all = true;
    for (const auto& [thing, of] : told) {
        if (d.shown.count(thing)) continue;
        Doc::Shown made{of};
        const std::string kind = doc_word(d, thing, "kind"), file = doc_word(d, thing, "file"), laid = doc_word(d, thing, "blend"), shape = doc_word(d, thing, "shape");
        const bool effected = doc_fx_files(doc_fx_steps(d, thing));   // (with effects of its own: drawn alone as normal, its way of lying on what is under taken once they are on: screen.h)
        const char* lay = effected || laid.empty() || laid == "normal" || doc_blend_way(laid) ? nullptr : laid.c_str();   // (one that reads what is under: drawn as normal, flat.mat mixes it)
        std::string why = "it could not be read";
        const std::string fill = doc_word(d, thing, "fill");
        // (a fill is a colour, six digits; or, of a text or a flat shape, two colours or more: a ramp down it, each of
        //  them six digits or eight, the last two how much of it is there: a colour fading to clear; each may say where
        //  it stands, @ and a share of the way, as CSS's colour stop positions: ff3366@0.35)
        std::vector<float4> ramp;
        std::vector<float> rampAt;
        long rgb = -1;
        {
            std::istringstream said(fill);
            for (std::string word; said >> word;) {
                const size_t mark = word.find('@');
                const std::string one = word.substr(0, mark);
                rampAt.push_back(mark == std::string::npos ? -1.f : std::clamp(std::strtof(word.c_str() + mark + 1, nullptr), 0.f, 1.f));
                const bool hex = (one.size() == 6 || one.size() == 8) && one.find_first_not_of("0123456789abcdefABCDEF") == std::string::npos;
                const long c = hex ? std::strtol(one.substr(0, 6).c_str(), nullptr, 16) : -1;
                if (c < 0) { ramp.clear(), rgb = -1; break; }
                if (ramp.empty()) rgb = c;
                const float there = one.size() == 8 ? (float)std::strtol(one.substr(6).c_str(), nullptr, 16) / 255.f : 1.f;
                ramp.push_back(float4{(float)((c >> 16) & 255) / 255.f, (float)((c >> 8) & 255) / 255.f, (float)(c & 255) / 255.f, there});
            }
            if (ramp.size() == 1 && ramp[0].w < 1.f) ramp.clear(), rgb = -1;   // (eight digits only down a ramp)
        }
        const float3 colour = float3{(float)((rgb >> 16) & 255), (float)((rgb >> 8) & 255), (float)(rgb & 255)} / 255.f;
        // (an edge, ThorVG's stroke: its colour said as a fill is, and its width as a share of the thing's own height,
        //  from points of the picture at size 1 where things stand at first)
        const std::string stroke = doc_word(d, thing, "stroke");
        const long edgeRgb = stroke.size() == 6 && stroke.find_first_not_of("0123456789abcdefABCDEF") == std::string::npos ? std::strtol(stroke.c_str(), nullptr, 16) : -1;
        const math::double4 up = s.camera->getProjectionMatrix() * s.camera->getViewMatrix() * math::double4{0, 1, 0, 1};
        const float4 edge = edgeRgb < 0 ? float4{0.f} : float4{float3{(float)((edgeRgb >> 16) & 255), (float)((edgeRgb >> 8) & 255), (float)(edgeRgb & 255)} / 255.f, (float)(doc_number(d, thing, "stroke_width", 0, 0) / (0.5 * s.h * up.y / up.w))};
        if (!stroke.empty() && edgeRgb < 0) why = "a stroke is six digits, as ff3366";
        const double units = 0.5 * s.h * up.y / up.w;   // (points of the picture to one unit of the stage, where things stand at first)
        DrawnFx fx;
        fx.thick = doc_on(d, thing, "scale_z"), fx.lit = doc_lit(d, thing), fx.bevel = doc_bevel(d, thing) > 0;
        if (ramp.size() > 1) fx.ramp = ramp;
        if (ramp.size() > 1 && std::any_of(rampAt.begin(), rampAt.end(), [](float a) { return a >= 0.f; })) {   // (placed: one not said stands evenly; one before the one ahead of it stands where that one does, CSS Images 3's colour stop fix-up)
            for (size_t i = 0; i < rampAt.size(); ++i) {
                if (rampAt[i] < 0.f) rampAt[i] = (float)i / (float)(rampAt.size() - 1);
                if (i > 0) rampAt[i] = std::max(rampAt[i], rampAt[i - 1]);
            }
            fx.rampAt = rampAt;
        }
        fx.radial = doc_word(d, thing, "gradient") == "radial", fx.rampAngle = (float)doc_number(d, thing, "gradient_angle", 0, 0), fx.rampScale = (float)doc_number(d, thing, "gradient_scale", 0, 1);
        fx.spread = doc_word(d, thing, "gradient_spread") == "reflect" ? 1 : doc_word(d, thing, "gradient_spread") == "repeat" ? 2 : 0;
        const std::vector<AbsentCopy>& repeated = doc_absent_copies(d, thing, d.stage->now);
        fx.copies = repeated.empty() ? (int)doc_number(d, thing, "copies", 0, 1) : (int)repeated.size();   // (as many as AbsentMotion's repeat lays, as doc_members says)
        {   // (its copies drawn in one go where none is cleared of its own (no script that says opacity) and their colours are a ramp flat.mat knows; a picture's own 3D layers neither)
            const std::string fills = doc_word(d, thing, "copy_fill"), script = doc_word(d, thing, "script");
            std::istringstream said(fills);
            int colours = 0;
            for (std::string one; said >> one && one != "none";) ++colours;
            const DocScript* sc = doc_script_of(d, script);
            bool connected = false;   // (a value given each copy by a behaviour: each copy its own look, not drawn in one go)
            doc_rows(d, sqlite3_mprintf("select 1 from fact where thing = %d and name glob '*_by' and value <> 0 limit 1", thing), [&](sqlite3_stmt*) { connected = true; });
            const bool faded = std::any_of(repeated.begin(), repeated.end(), [](const AbsentCopy& c) { return c.alpha < 0.999; });   // (AbsentMotion's copies fading each, a scatter field's far ones: each its own)
            fx.together = fx.copies > 1 && (kind == "shape" || kind == "text") && colours <= 8 && !(sc && sc->clears) && !connected && !faded;   // (eight colours at most: flat.mat's ramp)
        }
        fx.wrapWidth = (float)doc_number(d, thing, "wrap_at", 0, 0);
        const std::string split = doc_word(d, thing, "split"), script = doc_word(d, thing, "script");
        fx.splitWords = split == "words";
        fx.split = split == "letters" || fx.splitWords || (split.empty() && !script.empty() && script != "none");   // (a text given a script moves letter by letter, as Cavalry's are, unless said none)
        { const std::string w = doc_word(d, thing, "wrap"); fx.wrap = w == "character" || w == "vertical" ? 1 : w == "word" ? 2 : w == "smart" ? 3 : w == "ellipsis" ? 4 : fx.wrapWidth > 0.f ? 2 : 0; if (w == "vertical") fx.wrapWidth = 1.f, fx.column = true; }   // (ThorVG's TextWrap; a box said and no way: by words; vertical: a box a letter wide)
        fx.letters = (float)doc_number(d, thing, "letter_spacing", 0, 1), fx.lines = (float)doc_number(d, thing, "line_spacing", 0, 1), fx.italic = (float)doc_number(d, thing, "italic", 0, 0);
        const auto which = [&](const char* name, const char* words) { const std::string w = doc_word(d, thing, name); if (w.empty()) return -1; int n = 0; for (const char* at = words; *at; ++n) { const size_t len = strcspn(at, " "); if (w == std::string(at, len)) return n; at += len + (at[len] ? 1 : 0); } return -1; };
        fx.cap = which("cap", "butt round square"), fx.join = which("join", "miter round bevel");
        fx.rampBlend = std::max(0, which("gradient_blend", "rgb oklab hue steps"));   // (ThorVG's StrokeCap and StrokeJoin, in its order)
        fx.heads = std::max(0, which("heads", "none end start both")), fx.headSize = (float)doc_number(d, thing, "head_size", 0, 4), fx.headShape = std::max(0, which("head_shape", "arrow circle box chevron bar t_line square square_filled diamond diamond_filled arrow_hollow arrow_filled circle_hollow circle_filled teardrop_hollow teardrop_filled")), fx.headStart = which("head_start", "arrow circle box chevron bar t_line square square_filled diamond diamond_filled arrow_hollow arrow_filled circle_hollow circle_filled teardrop_hollow teardrop_filled");
        fx.dash = (float)(doc_number(d, thing, "dash", 0, 0) / units), fx.dashFrom = (float)(doc_number(d, thing, "dash_offset", 0, 0) / units);
        bool fxOk = doc_fx(d, thing, 0.0, true, units, fx, why);   // (made as large as its effects reach at their most; each moment tells them as they are then)
        if (!fxOk && doc_fx_files(doc_fx_steps(d, thing))) { d.trouble = why + ": the " + kind + " " + std::filesystem::path(file + shape).filename().string() + " could not be made"; all = false; continue; }   // (an effect file of any kind of thing, as a picture's own file is refused)
        made.units = units;
        if (doc_among("film picture text shape", kind)) made.space = doc_word(d, thing, "space") == "2.5d" ? 25 : doc_word(d, thing, "space") == "3d" ? 3 : 2;
        if (kind == "behaviour") made.behaviour = true;   // (nothing of it is seen: it says how values of others go)
        else if (kind == "camera") made.camera = true;   // (nothing of it is seen: it is how the rest is)
        else if (kind == "light") {   // (Filament's own light, made as the document says; placed and told how strong at each moment)
            const LightManager::Type type = file == "sun" ? LightManager::Type::SUN : file == "spot" ? LightManager::Type::FOCUSED_SPOT : LightManager::Type::POINT;
            if (!doc_among("sun point spot", file.c_str())) why = "a light is a sun, a point or a spot";
            else {
                made.light = utils::EntityManager::get().create();
                LightManager::ShadowOptions shade;   // (a finer map than as it comes, so the edge is not stepped; a lamp as large as its softness says)
                shade.mapSize = 2048;
                const float soft = (float)doc_number(d, thing, "shadow_softness", 0, 0);
                shade.screenSpaceContactShadows = doc_number(d, thing, "contact_shadows", 0, 0) > 0.5;
                if (soft > 0.f) shade.shadowBulbRadius = type == LightManager::Type::SUN ? 0.05f * soft : 0.5f * soft;
                LightManager::Builder(type).color(Color::toLinear<ACCURATE>(sRGBColor{fill.empty() ? float3{1.f} : colour})).falloff((float)doc_number(d, thing, "falloff", 0, 10))
                    .castShadows(doc_number(d, thing, "shadows", 0, 0) > 0.5).shadowOptions(shade).build(*s.engine, made.light);
                if (const int group = (int)doc_number(d, thing, "light_group", 0, 0); group > 0) {
                    auto& lights = s.engine->getLightManager();
                    lights.setLightChannel(lights.getInstance(made.light), 0, false), lights.setLightChannel(lights.getInstance(made.light), (unsigned)group, true);
                }
                made.in = false;   // (in the scenes when it is on)
            }
        }
        else if (shape.empty() && !std::filesystem::is_regular_file(file)) why = "there is no such file";
        else if (kind == "film") {
            const std::string moshed = doc_mosh(d, thing);   // (its datamosh, once made: played in its place, from its start)
            const double from = moshed.empty() ? doc_number(d, thing, "begin", 0, 0) : 0.0, length = doc_number(d, thing, "duration", 0, -1), speed = doc_number(d, thing, "speed", 0, 1);
            made.film = video(s, (moshed.empty() ? file : moshed).c_str(), mat4f{}, 1.f, (float)from, length < 0 ? -1.f : (float)(from + length * std::fabs(speed)), (float)speed, lay, doc_lit(d, thing));
            if (!made.film) why = video_trouble;
            else if (const double keyed = doc_number(d, thing, "keyed", 0, -1); keyed != 0.0) key_own(made.film, keyed > 0.5);   // (not said: keyed if it has a ground of its own)
            if (made.film) s.playing.erase(made.film);   // (the document tells it its moment, and when its sound comes on)
            if (made.film && made.film->sound) s.sounding[made.film] = [&d, thing, v = made.film, file] {   // (counted from where the film being written begins: a sound already going then is heard from that far into it)
                const double on = doc_number(d, thing, "start", 0, 0) - d.written_from, lasts = doc_number(d, thing, "duration", 0, -1), late = std::max(0.0, -on);
                Stage::Sound a{file, v->from, v->to, v->speed, late, std::max(0.0, on), lasts < 0.0 ? lasts : std::max(1e-3, lasts - late), doc_number(d, thing, "volume", 0, 1)};
                if (doc_keys_of(d, thing, "volume").at.size() > 1)   // (keyed: Alight Motion's Volume with its keys and eases, read each 1/30 s, as the document reads it)
                    for (double t = 0.0, end = a.lasts >= 0.0 ? a.lasts : doc_seconds(d) - d.written_from - a.on; t <= end + 1e-9 && a.loud.size() < 30 * 3600; t += 1.0 / 30.0)
                        a.loud.push_back((float)doc_number(d, thing, "volume", d.written_from + a.on + t, 1));
                return a;
            };
        } else if (kind == "picture") {
            std::string shown = file;
            float rollHigh = 1.f;   // (a MIDI file: its piano roll, at its speed's tempo: midi_roll.cpp)
            if (std::filesystem::path(file).extension() == ".mid") {
                const double speed = doc_number(d, thing, "speed", 0, 1);
                shown = midi_roll_file(file, speed == 1.0 ? 0.0 : midi_bpm(file.c_str()) * speed, why);
                double w = 0, h = 0;   // (a point of it a point of the document, as ALMaMPlayer's 1920 x 1080 screen shows it)
                if (!shown.empty() && midi_roll_box(shown, w, h)) rollHigh = (float)(h / doc_point(d, thing, 0));
            }
            const std::string ext = std::filesystem::path(file).extension().string();
            const double cl = doc_number(d, thing, "crop_left", 0, 0), ct = doc_number(d, thing, "crop_top", 0, 0), cr = doc_number(d, thing, "crop_right", 0, 0), cb = doc_number(d, thing, "crop_bottom", 0, 0);
            const bool cropped = cl + ct + cr + cb > 0.0;
            const float aspect = doc_among(".jpg .jpeg .png .webp .bmp .JPG .JPEG .PNG", ext) && (edge.w > 0.f || doc_number(d, thing, "round", 0, 0) > 0.0 || cropped) ? picture_aspect(s, file.c_str()) : 0.f;
            if (fxOk && aspect > 0.f) {   // (a photo with an edge, round corners or cut in: a rect as wide as it is, its own file its fill, as a shape filled with media is (Alight Motion's media layers take a border as its shapes do))
                DrawnFx own = fx;
                own.media = file;
                char cut[256];   // (cut in: the rect it shows a path of its own within the whole photo, in its height from its middle, so what is left stays where it was)
                std::snprintf(cut, sizeof cut, "M%.5f %.5f L%.5f %.5f L%.5f %.5f L%.5f %.5f Z", aspect * (cl - 0.5), ct - 0.5, aspect * (0.5 - std::min(cr, 0.95 - cl)), ct - 0.5,
                              aspect * (0.5 - std::min(cr, 0.95 - cl)), 0.5 - std::min(cb, 0.95 - ct), aspect * (cl - 0.5), 0.5 - std::min(cb, 0.95 - ct));
                made.drawn = flat(s, cropped ? cut : "rect", aspect, mat4f{}, 1.f, float3{1.f}, lay, edge, own.over(1.f), 4, 0.5f, (float)doc_number(d, thing, "round", 0, 0));
            } else if (fxOk && !shown.empty() && (made.drawn = picture(s, shown.c_str(), mat4f{}, rollHigh, float3{-1.f}, true, lay, fx, doc_word(d, thing, "marker").c_str()))) s.playing.erase(made.drawn);
            if (made.drawn && made.drawn->lottie && !picture_deep(s, made.drawn).empty()) {   // (its 3D layers stood up where they are, as After Effects shows them)
                made.motion = std::make_shared<Motion>();
                if (motion_open(*made.motion, file.c_str()))
                    for (uint32_t id : made.drawn->deep) {
                        float4 box;
                        if (Drawn* own = layer_picture(s, file.c_str(), id, 1.f / made.motion->h, &box)) made.deep.push_back({id, own, box});
                    }
            }
        } else if (kind == "text") {   // (words in a font: said by `say N text ...`; until then, the font's own name)
            const std::string words = doc_text_shown(d, thing, s.now, &made.wordsMove);   // (as typed, or as AbsentMotion's text effects show them now)
            made.words = words;
            if (!fill.empty() && fill != "behind" && rgb < 0) why = "a text's fill is six digits, as ff3366, or behind (what is behind it)";
            else if ((stroke.empty() || edgeRgb >= 0) && fxOk) made.drawn = type(s, words.empty() ? std::filesystem::path(file).stem().string().c_str() : (fx.column ? doc_column(words) : words).c_str(), file.c_str(), mat4f{}, 1.f, fill.empty() ? float3{0x11 / 255.f} : colour, lay, edge, fx.column ? .5f : (float)doc_number(d, thing, "align", 0, 0), fx);
        } else if (kind == "shape") {   // (a flat shape in one colour: ThorVG's own rectangle or circle)
            if (!fill.empty() && fill != "none" && fill != "behind" && rgb < 0) why = "a shape's fill is six digits, as ff3366, none, or behind (what is behind it)";
            else if ((!stroke.empty() && edgeRgb < 0) || !fxOk) {}
            else if (shape == "solid") {   // (a flat colour as large as the stage, where it stands at first: as wide to its height, and as high in the stage's units)
                const float high = (float)(2.0 * up.w / up.y);
                made.drawn = flat(s, "rect", (float)s.w / (float)s.h, mat4f{}, high, fill.empty() ? float3{0xd9, 0xd2, 0xc3} / 255.f : colour, lay, float4{edge.xyz, edge.w / high}, fx.over(high));
            } else {   // (a line is long and thin as it comes: 25 times as long as it is thick, a tenth of a unit thick)
                const bool line = shape == "line", wave = shape == "wave";   // (a wave: a line waving, ten times as long as it is high)
                fx.hollow = fill == "none";   // (filled with nothing: its edge alone, a frame)
                fx.media = doc_word(d, thing, "fill_media");   // (filled with a picture's file: drawn_made.h flat)
                fx.evenOdd = doc_word(d, thing, "fill_rule") == "evenodd";
                const float high = line ? 0.04f : wave ? 0.3f : 1.f;
                // (its outline alone, in one colour: cut into triangles and drawn by Filament as the scene's other things
                //  are, flat_mesh; with an edge, a ramp, a dash, effects, a trim or a bend made on the CPU: ThorVG's picture)
                const std::string drawn = doc_word(d, thing, "path"), form = shape != "path" ? shape : drawn.empty() ? "M0 0" : drawn;   // (a path shape: its own path is its form; none yet: nothing)
                const bool plain = !fx.thick && !wave && shape != "spiral" && shape != "null" && fx.media.empty() && fx.ramp.size() < 2 && fx.blur <= 0.f && fx.shadow.w <= 0.f && fx.tint <= 0.f && fx.tritone <= 0.f;
                if (plain)
                    made.drawn = flat_mesh(s, form.c_str(), (float)doc_number(d, thing, "wide", 0, line ? 25 : 1), mat4f{}, high, fill.empty() ? float3{0xd9, 0xd2, 0xc3} / 255.f : colour, lay,
                                           float4{edge.xyz, edge.w / high}, fx.over(high), (int)doc_number(d, thing, "points", 0, shape == "star" ? 5 : shape == "arc" ? 270 : shape == "squircle" ? 4 : shape == "cog" ? 12 : shape == "spiral" ? 3 : 6), (float)doc_number(d, thing, "inner", 0, shape == "cog" ? 0.2 : shape == "arc" ? 0.3 : shape == "spiral" ? 0.1 : shape == "chamfer" ? 0.3 : 0.5), (float)doc_number(d, thing, "round", 0, 0), doc_bends(d, thing));
                else made.drawn = flat(s, form.c_str(), (float)doc_number(d, thing, "wide", 0, line ? 25 : wave ? 10 : 1), mat4f{}, high, fill.empty() ? float3{0xd9, 0xd2, 0xc3} / 255.f : colour, lay,
                                  float4{edge.xyz, edge.w / high}, fx.over(high),
                                  (int)doc_number(d, thing, "points", 0, shape == "star" ? 5 : shape == "arc" ? 270 : shape == "squircle" ? 4 : shape == "cog" ? 12 : shape == "spiral" ? 3 : wave ? 4 : 6), (float)doc_number(d, thing, "inner", 0, shape == "cog" ? 0.2 : shape == "arc" ? 0.3 : shape == "spiral" ? 0.1 : shape == "chamfer" ? 0.3 : 0.5), (float)doc_number(d, thing, "round", 0, 0));
            }
        } else if (kind == "solid") {   // (one of the kit's shapes, or a file; its fill is its surface: a colour, or the material glass)
            const bool glass = fill == "glass";
            const auto glassy = [&](MaterialInstance* look) {   // (the kit's glass as the document says it; not said, as it comes)
                look->setParameter("ior", (float)doc_number(d, thing, "ior", 0, 1.5)), look->setParameter("split", (float)doc_number(d, thing, "dispersion", 0, 0.1));
                look->setParameter("swim", (float)doc_number(d, thing, "swim", 0, 0.5)), look->setParameter("dark", (float)doc_number(d, thing, "dark", 0, 4));
                look->setParameter("vivid", (float)doc_number(d, thing, "vivid", 0, 1.3));
            };
            const Surface surface{(float)doc_number(d, thing, "metallic", 0, 0.08), (float)doc_number(d, thing, "roughness", 0, 0.6), (float)doc_number(d, thing, "reflectance", 0, 0.5),
                                  (float)doc_number(d, thing, "clear_coat", 0, 0), (float)doc_number(d, thing, "clear_coat_roughness", 0, 0), (float)doc_number(d, thing, "emissive", 0, 0),
                                  (float)doc_number(d, thing, "sheen", 0, 0), (float)doc_number(d, thing, "anisotropy", 0, 0), (float)doc_number(d, thing, "thickness", 0, 0.5)};
            const std::string shading = doc_word(d, thing, "shading");
            const char* model = doc_number(d, thing, "clip", 0, 0) > 0.5 && (shading.empty() || shading == "lit") ? "floor_lit_matte.filamat" : surface_model(shading);   // (clipped: one a matte can cut)
            if (!fill.empty() && !glass && (rgb < 0 || ramp.size() > 1)) why = "a solid's fill is six digits, as ff3366, or glass";
            else if (!shape.empty()) {
                made.shape = shape == "gem" ? gem(0.5f) : shape == "cube" ? cube(0.1f) : shape == "ball" ? sphere(0.5f) : torus(0.37f, 0.13f);
                MaterialInstance* look = glass ? thick_glass(s, made.shape, {mat4f{}}) : add(s, made.shape, model, std::vector<mat4f>((size_t)std::max(1, (int)doc_number(d, thing, "copies", 0, 1)), mat4f{}));
                made.look = look;
                if (look && glass) glassy(look);
                if (!look) made.shape = nullptr;
                else if (!glass) plain(look, fill.empty() ? float3{0xd9, 0xd2, 0xc3} / 255.f : colour, surface);
            } else if (std::filesystem::path(file).extension() == ".ply") {   // (a point cloud: its grains, one buffer, points.h)
                for (const std::string& w : doc_fx_list(d, thing)) if (std::filesystem::path(w).extension() == ".mjs") made.splatFxFile = w;   // (a PlayCanvas splat effect on it: its look made with it)
                std::string effectWhy;
                Material* with = made.splatFxFile.empty() ? nullptr : points_effect(s, made.splatFxFile, made.splatFx, effectWhy);
                if (!effectWhy.empty()) std::fprintf(stderr, "%s\n", effectWhy.c_str());
                float3 box[2];
                made.shape = points(s, file.c_str(), why, &made.look, with, &made.splatPlain, box);
                if (made.splatFx)   // (a PlayCanvas effect's box, as it comes: the cloud's own)
                    for (int c = 0; c < 3; ++c) made.splatFx->usual(std::string("aabbMin_") + "xyz"[c], box[0][c]), made.splatFx->usual(std::string("aabbMax_") + "xyz"[c], box[1][c]);
                made.grains = made.shape != nullptr;
            } else if (glass) {   // (a file's shape as glass: its triangles alone, for the kit's glass, which draws a shape three times)
                MaterialInstance* look = (made.shape = solid_mesh(file.c_str())) ? thick_glass(s, made.shape, {mat4f{}}) : nullptr;
                made.look = look;
                if (!look) made.shape = nullptr; else glassy(look);
            }
            else if ((made.solid = solid(s, file.c_str(), mat4f{}, 1.f, (int)doc_number(d, thing, "copies", 0, 1))) && rgb >= 0) solid_fill(s, made.solid, colour, surface, model);
            if (made.solid) s.playing.erase(made.solid);   // (the document tells it its moment)
        } else why = std::string("its kind is one of: ") + doc_kinds;
        if (!made.film && !made.drawn && !made.solid && !made.shape && !made.camera && made.light.isNull() && kind != "behaviour") { d.trouble = why + ": the " + kind + " " + std::filesystem::path(file + shape).filename().string() + " could not be made"; all = false; continue; }
        d.shown[thing] = made;
        if ((made.solid || made.shape) && !made.grains) {   // (a solid thing casts a shadow from a light that throws them, and takes one; a cloud's grains are their own light)
            const int group = (int)doc_number(d, thing, "light_group", 0, 0);
            doc_parts(s, d.shown[thing], [group](RenderableManager& faces, RenderableManager::Instance one) {
                faces.setCastShadows(one, true), faces.setReceiveShadows(one, true), faces.setScreenSpaceContactShadows(one, true);   // (contact shadows: Filament draws them only where the light asks too)
                if (group > 0) faces.setLightChannel(one, (unsigned)group, true);
            });
        }
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

// The work area: the stretch of the document that is played round and written as a film (After Effects' own); not
// said, all of it. And the marks: moments of the document with a name (After Effects' composition markers). Both are
// facts of thing 0, the stage itself, and every change of them is a step.
static bool doc_change(Doc& d, char* change, bool later = false);   // (later: a key, whose act need not show it before its end)
static void doc_work(Doc& d, double* from, double* to) {
    const double all = doc_seconds(d);
    double in = 0.0, out = all;
    doc_rows(d, sqlite3_mprintf("select name, value from fact where thing = 0 and name in ('work_in', 'work_out')"),
             [&](sqlite3_stmt* row) { (strcmp((const char*)sqlite3_column_text(row, 0), "work_in") ? out : in) = sqlite3_column_double(row, 1); });
    *from = std::clamp(in, 0.0, std::max(0.0, all - 1e-3));
    *to = std::clamp(out, *from + 1e-3, all);
}
static bool doc_work_is(Doc& d, double from, double to) {
    if (!(from >= 0.0 && to > from + 1e-3 && to <= 1e6)) return d.trouble = "a work area is from one moment to a later one", false;
    return doc_change(d, sqlite3_mprintf("insert into fact(thing, name, at, value) values(0, 'work_in', 0.0, %!.17g), (0, 'work_out', 0.0, %!.17g) on conflict do update set value = excluded.value", from, to));
}
static bool doc_mark(Doc& d, double at, const char* word) {   // ("": the mark there is taken away, its colour with it)
    if (!(at >= 0.0 && at <= 1e6)) return d.trouble = "a mark is at a moment of the stage: from 0 to 1000000 seconds", false;
    if (strchr(word, '\n')) return d.trouble = "a mark's name is one line", false;
    return doc_change(d, *word ? sqlite3_mprintf("insert into fact(thing, name, at, value) values(0, 'mark', %!.17g, %Q) on conflict do update set value = excluded.value", at, word)
                               : sqlite3_mprintf("delete from fact where thing = 0 and name in ('mark', 'mark_colour') and at = %!.17g", at));
}
// A mark's colour, one of the sixteen label colours (1 to 16; 0: none), as Premiere Pro's, DaVinci Resolve's and
// CapCut's markers have one (golf ㉕, P02: a cut and a hit told apart): a fact beside its name at its moment.
static bool doc_mark_colour(Doc& d, double at, int colour) {
    if (colour < 0 || colour > 16) return d.trouble = "a mark's colour is one of 16 (0: none)", false;
    bool there = false;
    doc_rows(d, sqlite3_mprintf("select 1 from fact where thing = 0 and name = 'mark' and at = %!.17g", at), [&](sqlite3_stmt*) { there = true; });
    if (!there) return d.trouble = "there is no mark at that moment", false;
    return doc_change(d, colour ? sqlite3_mprintf("insert into fact(thing, name, at, value) values(0, 'mark_colour', %!.17g, %d) on conflict do update set value = excluded.value", at, colour)
                                : sqlite3_mprintf("delete from fact where thing = 0 and name = 'mark_colour' and at = %!.17g", at));
}
// The guides: lines across the picture at a place, in points of it (After Effects' composition guides): x, an upright
// line that far from the left; y, a level one that far from the top. Facts of the stage, a change of them a step.
static bool doc_guide(Doc& d, const char* axis, double at, bool on) {
    if (strcmp(axis, "x") && strcmp(axis, "y")) return d.trouble = "a guide is x (upright) or y (level)", false;
    if (!(std::fabs(at) <= 1e5)) return d.trouble = "a guide is within the picture's reach", false;
    return doc_change(d, on ? sqlite3_mprintf("insert into fact(thing, name, at, value) values(0, 'guide_%s', %!.17g, 1) on conflict do nothing", axis, at)
                            : sqlite3_mprintf("delete from fact where thing = 0 and name = 'guide_%s' and at = %!.17g", axis, at));
}
static std::string doc_guides(Doc& d) {   // (a line each: x or y, a space, where)
    std::string all;
    doc_rows(d, sqlite3_mprintf("select substr(name, 7), at from fact where thing = 0 and name in ('guide_x', 'guide_y') order by name, at"), [&](sqlite3_stmt* row) {
        char line[60];
        std::snprintf(line, sizeof line, "%s %.17g\n", (const char*)sqlite3_column_text(row, 0), sqlite3_column_double(row, 1));   // (all of it: a guide is found again by where it is)
        all += line;
    });
    return all;
}
static std::string doc_mark_colours(Doc& d) {   // (a line each, in time: the moment, a space, its colour)
    std::string all;
    doc_rows(d, sqlite3_mprintf("select at, value from fact where thing = 0 and name = 'mark_colour' order by at"), [&](sqlite3_stmt* row) {
        char line[60];
        std::snprintf(line, sizeof line, "%.17g %d\n", sqlite3_column_double(row, 0), sqlite3_column_int(row, 1));
        all += line;
    });
    return all;
}
// The window's own choices kept with the document, not steps of it (their table is not one the steps record): the
// inks and the fonts a new thing comes in, as a PowerPoint file keeps its theme's colours and fonts (golf (90),
// engine_needs.md E4). A word "" takes one away; doc_settings: a line each, the name, a space, the word.
static bool doc_setting(Doc& d, const char* name, const char* word) {
    if (!*name || strchr(name, ' ') || strchr(name, '\n') || strchr(word, '\n')) return d.trouble = "a setting is one word named, one line said", false;
    char* say = *word ? sqlite3_mprintf("insert into setting(name, value) values(%Q, %Q) on conflict do update set value = excluded.value", name, word)
                      : sqlite3_mprintf("delete from setting where name = %Q", name);
    const bool ok = sqlite3_exec(d.file, say, nullptr, nullptr, nullptr) == SQLITE_OK;
    sqlite3_free(say);
    return ok || (d.trouble = sqlite3_errmsg(d.file), false);
}
static std::string doc_settings(Doc& d) {
    std::string all;
    doc_rows(d, sqlite3_mprintf("select name, value from setting order by name"), [&](sqlite3_stmt* row) {
        all += std::string((const char*)sqlite3_column_text(row, 0)) + " " + (const char*)sqlite3_column_text(row, 1) + "\n";
    });
    return all;
}
static std::string doc_marks(Doc& d) {   // (a line each, in time: the moment, a space, the name)
    std::string all;
    doc_rows(d, sqlite3_mprintf("select at, value from fact where thing = 0 and name = 'mark' order by at"), [&](sqlite3_stmt* row) {
        char at[40];
        std::snprintf(at, sizeof at, "%.17g ", sqlite3_column_double(row, 0));   // (all of it: a mark is found again by its moment, to the last digit)
        all += at + std::string((const char*)sqlite3_column_text(row, 1)) + "\n";
    });
    return all;
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

static std::vector<std::filesystem::path> doc_folders(const char* folder);   // (doc_edit.h)
// The document in `path` (made if it is not there), on the stage. nullptr: the file is not a document.
static Doc* doc_open(Stage& s, const char* path) {
    Doc* d = new Doc;
    if (sqlite3_open(path, &d->file) != SQLITE_OK) return nullptr;
    // (a way defined in the document may ask how far a cubic bezier ease is at u, by its two handles, as the nine kinds'
    //  Bezier goes: bezier(u, x1, y1, x2, y2))
    sqlite3_create_function(d->file, "bezier", 5, SQLITE_UTF8 | SQLITE_DETERMINISTIC, nullptr, [](sqlite3_context* c, int, sqlite3_value** v) {
        sqlite3_result_double(c, interp_bezier(interp_clamp(sqlite3_value_double(v[1]), 0.0, 1.0), sqlite3_value_double(v[2]), interp_clamp(sqlite3_value_double(v[3]), 0.0, 1.0), sqlite3_value_double(v[4]), sqlite3_value_double(v[0])));
    }, nullptr, nullptr);
    // (and how far an ease of anime.js has gone at u, as anime.js says it: anime('outElastic(1, .3)', u), script.h)
    sqlite3_create_function(d->file, "anime", 2, SQLITE_UTF8 | SQLITE_DETERMINISTIC, nullptr, [](sqlite3_context* c, int, sqlite3_value** v) {
        static const std::string bundle = (doc_folders("third_party/anime").front() / "anime.umd.min.js").string();
        std::string trouble;
        const double went = anime_ease(bundle, (const char*)sqlite3_value_text(v[0]), sqlite3_value_double(v[1]), &trouble);
        if (trouble.empty()) sqlite3_result_double(c, went);
        else sqlite3_result_error(c, trouble.c_str(), -1);
    }, nullptr, nullptr);
    // (and how far another way has gone at u, as it is said: way('Cyclic', u), way('Bezier .2 0 0 1', u). So ways are laid
    //  over one another, one through another, times or plus it: GSAP's CustomWiggle's timingEase and amplitudeEase,
    //  Blender's F-Curve stack and its Noise; each begins at its first key and ends at its next, so what they make does too)
    sqlite3_create_function(d->file, "way", 2, SQLITE_UTF8 | SQLITE_DETERMINISTIC, d, [](sqlite3_context* c, int, sqlite3_value** v) {
        const char* text = (const char*)sqlite3_value_text(v[0]);
        const std::string said = text ? text : "";
        const double u = sqlite3_value_double(v[1]);
        Interp kind;
        sqlite3_result_double(c, u <= 0.0 || u >= 1.0 ? u : interp_said(said, &kind) == 0 ? interp_ease(kind, u) : doc_went_by(*(Doc*)sqlite3_user_data(c), said, u, u, 1.0, 0.0, 1.0));   // (past its ends as it was: an overshoot above it stays one)
    }, nullptr, nullptr);
    sqlite3_busy_timeout(d->file, 2000);   // (another program at the same document is waited for)
    // (a key's `way` is how the stretch from it to the next key goes; `way` the table holds the ways defined in this document)
    if (sqlite3_exec(d->file, "create table if not exists fact(thing integer, name text, at numeric, value, way text, primary key(thing, name, at)) without rowid;"
                              "create table if not exists way(name text primary key, goes text) without rowid;"
                              "create table if not exists step(n integer primary key, change blob, undone integer default 0);"
                              "create table if not exists thing(id integer primary key autoincrement);"   // (a thing's number is never given again: SQLite's own)
                              "create table if not exists setting(name text primary key, value text) without rowid;"   // (the window's choices kept with the document, not steps: doc_setting)
                              "insert into thing(id) select max(thing) from fact where (select count(*) from thing) = 0 having max(thing) is not null", nullptr, nullptr, nullptr) != SQLITE_OK) return nullptr;
    if (sqlite3_exec(d->file, "select way from fact limit 0", nullptr, nullptr, nullptr) != SQLITE_OK)   // (a document from before stretches had ways)
        sqlite3_exec(d->file, "alter table fact add column way text", nullptr, nullptr, nullptr);
    if (sqlite3_exec(d->file, "select made from step limit 0", nullptr, nullptr, nullptr) != SQLITE_OK)   // (a document from before its steps kept when they were made)
        sqlite3_exec(d->file, "alter table step add column made real", nullptr, nullptr, nullptr);
    // (a document from before its moments were numeric: SQLite's session keeps a row whose moment is a whole number
    //  made and then changed in one act as two, if the moment is real, and that step cannot be undone. Made again once)
    bool real = false;
    doc_rows(*d, sqlite3_mprintf("select 1 from pragma_table_info('fact') where name = 'at' and type = 'REAL'"), [&](sqlite3_stmt*) { real = true; });
    if (real)
        sqlite3_exec(d->file, "begin; create table fact_numeric(thing integer, name text, at numeric, value, way text, primary key(thing, name, at)) without rowid;"
                              "insert into fact_numeric select thing, name, at, value, way from fact; drop table fact; alter table fact_numeric rename to fact; commit", nullptr, nullptr, nullptr);
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
static bool doc_match(Doc& d);
static void doc_forget(Doc& d);
// What a step changed, as a shell is told it: each value (thing, name) its changeset touches, and whether a thing came
// or went (its kind written or taken away). The changeset is SQLite's own record of the step: nothing is missed.
static void doc_note(Doc& d, int size, void* changeset) {
    sqlite3_changeset_iter* it = nullptr;
    if (sqlite3changeset_start(&it, size, changeset) != SQLITE_OK) return;
    while (sqlite3changeset_next(it) == SQLITE_ROW) {
        const char* table = nullptr;
        int columns = 0, op = 0, indirect = 0;
        sqlite3changeset_op(it, &table, &columns, &op, &indirect);
        if (!table || strcmp(table, "fact")) { d.things_changed = true; continue; }   // (the ways defined: anything may go another way)
        sqlite3_value *thing = nullptr, *name = nullptr;
        (op == SQLITE_INSERT ? sqlite3changeset_new : sqlite3changeset_old)(it, 0, &thing);
        (op == SQLITE_INSERT ? sqlite3changeset_new : sqlite3changeset_old)(it, 1, &name);
        if (!thing || !name) continue;
        const std::string n = (const char*)sqlite3_value_text(name);
        d.changed.insert({sqlite3_value_int(thing), n});
        if (n == "kind" && op != SQLITE_UPDATE) d.things_changed = true;
    }
    sqlite3changeset_finalize(it);
}
static bool doc_acted(Doc& d) {
    if (d.act == 1 && d.act_unmatched) {   // (the act's changes shown once, all of them: if the stage cannot, none of them is made)
        d.act_unmatched = false;
        doc_forget(d);
        if (d.act_ok && !doc_match(d)) d.act_ok = false;
    }
    if (--d.act) return d.act_ok;
    int size = 0;
    void* changed = nullptr;
    sqlite3session_changeset(d.acting, &size, &changed);
    sqlite3session_delete(d.acting);
    d.acting = nullptr;
    if (size) doc_note(d, size, changed);   // (an act given up changed them too, and back: shown as they are again)
    if (d.act_ok && size) {   // (what was undone and not done again is forgotten: a new step follows the last one that stands)
        sqlite3_exec(d.file, "delete from step where undone", nullptr, nullptr, nullptr);
        sqlite3_stmt* keep = nullptr;
        sqlite3_prepare_v2(d.file, "insert into step(change, made) values(?, (julianday('now') - 2440587.5) * 86400.0)", -1, &keep, nullptr);   // (and when, in seconds: doc_edited)
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
static bool doc_change(Doc& d, char* change, bool later) {
    doc_act(d);
    bool ok = d.act_ok;   // (a change before it in this act was refused: d.trouble still says why)
    if (ok && d.writing) ok = false, d.trouble = "the document is being written as a film: it is not changed until that is done";
    if (ok && !(ok = sqlite3_exec(d.file, change, nullptr, nullptr, nullptr) == SQLITE_OK)) d.trouble = sqlite3_errmsg(d.file);
    sqlite3_free(change);
    if (ok && (later || d.act_together) && d.act > 1) d.act_unmatched = true;   // (a key within a larger act, one of a thousand, or any change within motolii_together's: matched once when the act ends)
    else if (ok) {
        doc_forget(d);
        ok = doc_match(d);
    }
    if (!ok) d.act_ok = false;
    return doc_acted(d) && ok;
}

// The last step undone (back = true), or the last undone step done again. false: there is none.
// How long the document has been worked on (Alight Motion's Total Edit Time), from when its steps were made: the time
// from one to the next counted while it is under a quarter of an hour, a longer one a break (WakaTime's way of
// counting time at work from its heartbeats, its keystroke timeout 15 minutes by default). Seconds.
static double doc_edited(Doc& d) {
    double all = 0.0, was = -1.0;
    doc_rows(d, sqlite3_mprintf("select made from step where made is not null order by n"), [&](sqlite3_stmt* row) {
        const double at = sqlite3_column_double(row, 0);
        if (was >= 0.0 && at > was && at - was < 900.0) all += at - was;
        was = at;
    });
    return all;
}
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
    if (ok) doc_note(d, size, apply);
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
static std::string doc_words(Doc& d, int thing, const char* name);
