// doc_values.h: part of doc.h (a value read at a moment, its keys and ways). Read only through doc.h.
static const DocShown doc_shown[] = {
    {"text", "words", "", "Text", 0, 0, nullptr},
    {"file", "file", "", "Font", 0, 0, nullptr},
    {"fill_media", "file", "", "Media", 0, 0, nullptr},   // (a shape filled with a picture: Alight Motion's Fill > Media; put from the Media shelf's Fill)
    {"size", "", "px", "Size", 0, 2, nullptr},
    {"align", "share", "%", "Align", 0, 1, nullptr},
    {"fill", "colours", "", "Fill", 0, 0, nullptr},
    {"stroke", "colour", "", "Stroke", 0, 0, nullptr},
    {"stroke_width", "", "px", "Stroke width", 0, 16, nullptr},
    {"split", "", "", "Split", 0, 0, nullptr},
    {"letter_spacing", "", "×", "Tracking", 0, 3, nullptr},
    {"line_spacing", "", "×", "Line gap", 0, 3, nullptr},
    {"italic", "share", "%", "Slant", 0, 0.5, nullptr},
    {"wrap", "", "", "Wrap", 0, 0, nullptr},
    {"wrap_at", "", "×", "Wrap at", 0, 40, nullptr},
    {"gradient", "", "", "Gradient", 0, 0, nullptr},
    {"gradient_angle", "angle", "°", "Angle", -180, 180, nullptr},
    {"points", "count", "", "Points", 3, 32, nullptr},
    {"inner", "share", "%", "Inner radius", 0.05, 1, nullptr},
    {"round", "share", "%", "Round", 0, 0.5, nullptr},
    {"trim", "share", "%", "Trim end", 0, 1, nullptr},
    {"trim_begin", "share", "%", "Trim start", 0, 1, nullptr},
    {"trim_offset", "share", "%", "Trim offset", -1, 1, nullptr},
    {"bloat", "", "", "Bloat", -1, 1, nullptr},
    {"offset", "", "", "Offset", -0.5, 0.5, nullptr},
    {"zigzag", "", "", "Zig zag", -0.3, 0.3, nullptr},
    {"zigzag_ridges", "count", "", "Ridges", 0, 20, "Zig zag"},
    {"zigzag_smooth", "onoff", "", "Smooth", 0, 1, "Zig zag"},
    {"warp", "", "", "Wave", -0.5, 0.5, nullptr},
    {"warp_width", "", "", "Wave length", 0.05, 2, "Wave"},
    {"warp_angle", "angle", "°", "Wave angle", -180, 180, "Wave"},
    {"warp_speed", "", "/s", "Wave speed", -5, 5, "Wave"},
    {"bend", "angle", "°", "Bend", -360, 360, nullptr},
    {"bend_direction", "", "", "Bend way", 0, 0, "Bend"},
    {"twist", "angle", "°", "Twist", -720, 720, nullptr},
    {"pinch", "", "", "Pinch", -1, 1, nullptr},
    {"skew", "angle", "°", "Skew", -80, 80, nullptr},
    {"skew_axis", "angle", "°", "Skew axis", -180, 180, "Skew"},
    {"squash", "", "", "Squash", -0.9, 2, nullptr},
    {"wiggle", "", "", "Wiggle", 0, 0.3, nullptr},
    {"wiggle_detail", "", "", "Detail", 0.5, 20, "Wiggle"},
    {"wiggle_speed", "", "/s", "Wiggle speed", 0, 10, "Wiggle"},
    {"stretch", "", "", "Stretch", 0, 0.5, nullptr},
    {"pin_tl_x", "", "", "Pin TL x", -0.5, 0.5, "Corner pin"}, {"pin_tl_y", "", "", "Pin TL y", -0.5, 0.5, "Corner pin"},
    {"pin_tr_x", "", "", "Pin TR x", -0.5, 0.5, "Corner pin"}, {"pin_tr_y", "", "", "Pin TR y", -0.5, 0.5, "Corner pin"},
    {"pin_bl_x", "", "", "Pin BL x", -0.5, 0.5, "Corner pin"}, {"pin_bl_y", "", "", "Pin BL y", -0.5, 0.5, "Corner pin"},
    {"pin_br_x", "", "", "Pin BR x", -0.5, 0.5, "Corner pin"}, {"pin_br_y", "", "", "Pin BR y", -0.5, 0.5, "Corner pin"},
    {"gradient_scale", "", "×", "Stretch", 0.1, 4, nullptr},
    {"gradient_spread", "", "", "Past end", 0, 0, nullptr},
    {"copies", "count", "", "How many", 1, 200, nullptr},
    {"copy_columns", "count", "", "Per row", 0, 40, nullptr},
    {"copy_x", "stage", "px", "Gap X", -2, 2, nullptr},
    {"copy_y", "down", "px", "Gap Y", -2, 2, nullptr},
    {"copy_turn", "angle", "°", "Turn all", 0, 720, nullptr},
    {"stagger", "", "s", "Delay", 0, 3, nullptr},
    {"copy_fill", "colours", "", "Colours", 0, 0, nullptr},
    {"copy_z", "stage", "px", "Gap deep", -2, 2, "Each copy"},
    {"copy_rotate_x", "angle", "°", "Tilt", -180, 180, "Each copy"},
    {"copy_rotate_y", "angle", "°", "Swing", -180, 180, "Each copy"},
    {"copy_rotate", "angle", "°", "Turn", -180, 180, "Each copy"},
    {"copy_size", "", "×", "Grow", 0.5, 1.5, "Each copy"},
    {"copy_opacity_end", "", "", "Last opacity", 0, 1, "Each copy"},
    {"copy_wide", "", "×", "Wider", 0.2, 5, "Each copy"},
    {"tall", "", "×", "Height", 0, 3, "Each copy"},
    {"copy_tall", "numbers", "", "Heights", 0, 0, "Each copy"},
    {"stagger_from", "", "", "From", 0, 0, nullptr},
    {"copy_fill_way", "", "", "Order", 0, 0, "Each copy"},
    {"wave_y", "stage", "px", "Up/down", -1, 1, nullptr},
    {"wave_x", "stage", "px", "Sideways", -1, 1, nullptr},
    {"wave_z", "stage", "px", "In/out", -1, 1, nullptr},
    {"wave_rotate", "angle", "°", "Swing", -90, 90, nullptr},
    {"wave_size", "share", "%", "Pulse", -1, 2, nullptr},
    {"wave_speed", "", "/s", "Speed", 0, 4, nullptr},
    {"wave_phase", "", "waves", "Ripple", -4, 4, nullptr},
    {"wave_noise", "share", "%", "Random", 0, 1, "Noise"},
    {"noise_scale", "stage", "px", "Blob", 0, 3, "Noise"},
    {"field_x", "stage", "px", "At X", -3, 3, nullptr},
    {"field_y", "down", "px", "At Y", -2, 2, nullptr},
    {"field_radius", "stage", "px", "Size", 0, 4, nullptr},
    {"strength", "share", "%", "Strength", 0, 1, nullptr},   // (a relation's: how much of it works)
    {"radius", "stage", "px", "Radius", 0, 2, nullptr},
    {"tilt", "angle", "°", "Tilt", -90, 90, nullptr},   // (an around's ring tipped back: 0 faces the camera, 90 lies flat)
    {"gap", "stage", "px", "Gap", -1, 1, nullptr},
    {"centre_x", "stage", "px", "Centre X", -2, 2, nullptr},
    {"centre_y", "down", "px", "Centre Y", -2, 2, nullptr},
    {"leader", "thing", "", "Leader", 0, 0, nullptr},
    {"leader_b", "thing", "", "Hand to", 0, 0, nullptr},
    {"hand", "share", "%", "Handed", 0, 1, nullptr},
    {"lag", "", "s", "Lag", 0, 1, nullptr},
    {"spread", "share", "%", "Spread", 0, 3, nullptr},
    {"scatter", "stage", "px", "Scatter", 0, 2, nullptr},
    {"field_size", "share", "%", "Grow", -1, 5, nullptr},
    {"field_push", "stage", "px", "Push", -2, 2, nullptr},
    {"field_band", "stage", "px", "Ring", 0, 2, nullptr},
    {"field_z", "stage", "px", "At Z", -2, 2, nullptr},
    {"field_rotate", "angle", "°", "Turn", -180, 180, nullptr},
    {"scatter_x", "stage", "px", "Spread X", 0, 4, nullptr},
    {"scatter_y", "stage", "px", "Spread Y", 0, 3, nullptr},
    {"scatter_z", "stage", "px", "Spread Z", 0, 3, nullptr},
    {"scatter_rotate", "angle", "°", "Turn", 0, 180, nullptr},
    {"scatter_size", "share", "%", "Size", 0, 1, nullptr},
    {"seed", "seed", "", "Seed", 0, 9999, nullptr},
    {"drift_x", "stage", "px/s", "Flow X", -2, 2, nullptr},
    {"drift_y", "down", "px/s", "Flow Y", -2, 2, nullptr},
    {"emissive", "", "×", "Light up", 0, 10, nullptr},
    {"marker", "", "", "Part", 0, 0, nullptr},
    {"animation", "", "", "Movement", 0, 0, nullptr},
    {"speed", "", "×", "Speed", -4, 4, nullptr},
    {"speed_hold", "", "", "Hold at", 0, 0, nullptr},
    {"retime", "", "", "Re-timing", 0, 0, nullptr}, {"retime_in", "", "s", "End of intro", 0, 10, nullptr}, {"retime_out", "", "s", "Start of outro", 0, 10, nullptr},   // (an element's, Alight Motion's)
    {"frame_rate", "", "fps", "Frame rate", 0, 60, nullptr},   // (0: as smooth as the stage)
    {"volume", "share", "%", "Volume", 0, 2, nullptr},   // (1 shown as 100%, as Alight Motion shows it)
    {"beats", "", "dB", "Beats", 0, 30, nullptr},
    {"clip_matte", "", "", "Cut by", 0, 0, nullptr},
    {"clip_invert", "onoff", "", "Invert", 0, 1, nullptr},
    {"clip_feather", "", "px", "Feather", 0, 100, nullptr},
    {"copy_on", "thing", "", "On", 0, 0, nullptr},
    {"copy_on_offset", "share", "%", "Slide", -1, 1, nullptr},
    {"connect", "thing", "", "Join", 0, 0, nullptr},
    {"x", "", "px", "Position X", -4, 4, nullptr}, {"y", "", "px", "Position Y", -3, 3, nullptr}, {"z", "", "px", "Position Z", -6, 6, nullptr},   // (in points of the picture from its top left: motolii_reading)
    {"scale_x", "share", "%", "Scale X", 0, 4, nullptr}, {"scale_y", "share", "%", "Scale Y", 0, 4, nullptr}, {"scale_z", "share", "%", "Scale Z", 0, 4, nullptr},
    {"rotate", "angle", "°", "Rotation", -180, 180, nullptr}, {"rotate_x", "angle", "°", "X Rotation", -180, 180, nullptr}, {"rotate_y", "angle", "°", "Y Rotation", -180, 180, nullptr},
    {"opacity", "share", "%", "Opacity", 0, 1, nullptr},
    {"blend", "", "", "Blend", 0, 0, nullptr}, {"clip", "onoff", "", "Clip to below", 0, 1, nullptr},
    {"orient", "onoff", "", "Auto-orient", 0, 1, nullptr}, {"note", "words", "", "Note", 0, 0, nullptr}, {"label", "", "", "Label colour", 0, 0, nullptr},
    {"distance", "", "px", "Distance", 0.05, 4, nullptr}, {"zoom", "share", "%", "Zoom", 0.1, 4, nullptr},
    {"anchor_x", "", "%", "Anchor X", -1, 1, "Anchor"}, {"anchor_y", "", "%", "Anchor Y", -1, 1, "Anchor"}, {"anchor_z", "", "%", "Anchor Z", -1, 1, "Anchor"},
    {"parent", "thing", "", "Parent", 0, 0, nullptr},
    {"parent_rotate", "share", "%", "Its turn", 0, 2, nullptr},
    {"parent_size", "share", "%", "Its size", 0, 2, nullptr},
    {"roll", "", "", "Roll", 0, 0, nullptr},
    {"scale_assist", "", "", "Assist", 0, 0, nullptr}, {"scale_assist_amount", "share", "%", "Scale", 0, 4, nullptr}, {"scale_assist_damp", "share", "%", "Damp", 1, 3, nullptr},
    {"roll_radius", "", "", "Roll radius", -5, 5, nullptr},   // (the thing it goes with: shown by its name)
    {"mosh_with", "thing", "", "Motion of", 0, 0, nullptr}, {"dissolve", "", "", "Dissolve", 0, 2, nullptr}, {"grain", "share", "%", "Grain", 0.1, 4, nullptr}, {"mosh_at", "", "s", "From", 0, 10, nullptr},
    {"connect_to", "thing", "", "With", 0, 0, nullptr},
    {"resample", "", "", "Points", 0, 64, nullptr},
    {"voxel", "share", "%", "Squares", 0, 0.2, nullptr},
    {"rough", "share", "%", "Rough", 0, 0.2, nullptr},
    {"rough_detail", "share", "%", "Detail", 0.005, 0.2, nullptr},
    {"trail", "thing", "", "Trail", 0, 0, nullptr},
    {"trail_length", "", "s", "Trail length", 0.05, 4, nullptr},
    {"fill_rule", "", "", "Fill rule", 0, 0, nullptr},
    {"morph", "share", "%", "Morph", 0, 1, nullptr},
    {"morph_to", "thing", "", "Into", 0, 0, nullptr},
    {"morph_steps", "", "", "Steps", 0, 32, nullptr},
    {"boolean", "", "", "Boolean", 0, 0, nullptr},
    {"boolean_with", "thing", "", "Shape", 0, 0, nullptr},
    {"follow", "thing", "", "Follow", 0, 0, nullptr},
    {"connect_reach", "stage", "px", "Reach", 0, 2, nullptr},
    {"cap", "", "", "Ends", 0, 0, nullptr},
    {"heads", "", "", "Heads", 0, 0, nullptr},
    {"head_size", "", "×", "Head", 0.5, 12, nullptr},
    {"head_shape", "", "", "Head shape", 0, 0, nullptr},
    {"head_start", "", "", "Start shape", 0, 0, nullptr},
    {"join", "", "", "Corners", 0, 0, nullptr},
    {"dash", "", "px", "Dash", 0, 100, nullptr},
    {"dash_offset", "", "px", "Offset", -100, 100, nullptr},
    {"metallic", "share", "%", "Metal", 0, 1, nullptr},
    {"roughness", "share", "%", "Rough", 0, 1, nullptr},
    {"reflectance", "share", "%", "Shine", 0, 1, nullptr},
    {"clear_coat", "share", "%", "Varnish", 0, 1, nullptr},
    {"clear_coat_roughness", "share", "%", "Haze", 0, 1, nullptr},
    {"sheen", "share", "%", "Sheen", 0, 1, nullptr},
    {"shading", "", "", "Model", 0, 0, nullptr},
    {"thickness", "share", "%", "Thick", 0, 1, nullptr},
    {"anisotropy", "", "", "Brushed", -1, 1, nullptr},
    {"ior", "", "", "Bend", 1, 3, nullptr},
    {"dispersion", "share", "%", "Rainbow", 0, 0.5, nullptr},
    {"swim", "share", "%", "Wobble", 0, 1, nullptr},
    {"dark", "", "", "Darken", 0, 20, nullptr},
    {"vivid", "", "×", "Colour", 0, 3, nullptr},
    {"intensity", "", "", "Bright", 0, 100, nullptr},
    {"falloff", "stage", "px", "Reach", 0, 20, nullptr},
    {"cone", "angle", "°", "Cone", 1, 89, nullptr},
    {"shadows", "onoff", "", "Shadows", 0, 1, nullptr},
    {"shadow_softness", "share", "%", "Soft", 0, 1, nullptr},
    {"sun_size", "angle", "°", "Disk", 0, 10, nullptr},
    {"sun_halo", "", "×", "Halo", 0, 100, nullptr},
    {"light_group", "", "", "Group", 0, 7, nullptr},
    {"contact_shadows", "onoff", "", "Contact", 0, 1, nullptr},
    {"projection", "", "", "Lens", 0, 0, nullptr},
    {"shift_x", "", "", "Shift X", -2, 2, nullptr},
    {"shift_y", "", "", "Shift Y", -2, 2, nullptr},
    {"tone_mapper", "", "", "Look", 0, 0, nullptr},
    {"exposure", "", "stops", "Bright", -5, 5, nullptr},
    {"contrast", "", "×", "Contrast", 0, 2, nullptr},
    {"saturation", "", "×", "Colour", 0, 2, nullptr},
    {"vibrance", "", "×", "Vibrance", 0, 2, nullptr},
    {"temperature", "", "", "Warmth", -1, 1, nullptr},
    {"bloom", "share", "%", "Bloom", 0, 1, nullptr},
    {"physics", "", "", "Physics", 0, 0, nullptr},
    {"bounce", "share", "%", "Bounce", 0, 1, nullptr},
    {"friction", "", "", "Friction", 0, 2, nullptr},
    {"gravity", "", "", "Gravity", 0, 30, nullptr},
    {"ambient_occlusion", "share", "%", "AO", 0, 1, nullptr},
    {"reflections", "share", "%", "Mirror", 0, 1, nullptr},
    {"vignette", "share", "%", "Vignette", 0, 1, nullptr},
    {"vignette_color", "colour", "", "Vignette", 0, 0, "Tints"},
    {"fog", "share", "%", "Fog", 0, 1, nullptr},
    {"fog_near", "", "", "Fog near", 0, 20, nullptr},
    {"fog_far", "", "", "Fog far", 0, 60, nullptr},
    {"crop_left", "share", "%", "Left", 0, 0.95, nullptr}, {"crop_top", "share", "%", "Top", 0, 0.95, nullptr}, {"crop_right", "share", "%", "Right", 0, 0.95, nullptr}, {"crop_bottom", "share", "%", "Bottom", 0, 0.95, nullptr},   // (where it is all but whole: Alight Motion's far; 0 not said, the fog's own thickness)
    {"fog_color", "colour", "", "Fog", 0, 0, "Tints"},
    {"lens_flare", "share", "%", "Flare", 0, 1, nullptr},
    {"anti_aliasing", "", "", "Smooth", 0, 0, nullptr},
    {"slope", "", "", "Slope", 0, 4, nullptr},
    {"offset", "", "", "Offset", -1, 1, nullptr},
    {"power", "", "", "Power", 0.1, 4, nullptr},
    {"shadows_color", "colour", "", "Shadows", 0, 0, nullptr},
    {"midtones_color", "colour", "", "Mids", 0, 0, nullptr},
    {"highlights_color", "colour", "", "Highs", 0, 0, nullptr},
    {"room", "", "", "Room", 0, 0, nullptr},
    {"environment", "", "×", "Bright", 0, 10, nullptr},
    {"environment_rotate", "angle", "°", "Turn", -180, 180, nullptr},
    {"behind", "", "", "Behind", 0, 0, nullptr},
    {"background", "colour", "", "Colour", 0, 0, nullptr},
};
// A thing's script (script.h): the knobs its file's head declares, as values of the thing named js_ and the knob's name,
// and its code, made once a file and again when the file changes (a script written and saved shows at once).
struct DocScript {
    long long stamp = -1;
    int seen = -1;   // (files_changed when the file was last asked whether it changed: asked again only after another)
    bool viewer = false;   // (Filament's own settings file: read and laid on the stage's view, not drawn: viewer_settings.h)
    std::string absent;    // (AbsentMotion's own motion effect, its id: the thing moved by AbsentMotion's C++, absent_motion.h)
    std::vector<ScriptKnob> knobs;
    std::vector<std::string> names;
    std::vector<DocValue> values;
    std::vector<DocShown> shown;
    std::unique_ptr<Script> run;
    bool clears = false;   // (its copies may each be as clear as it says: its text says opacity; drawn each of its own, not in one go)
};
// Each document its own: a script runs in a QuickJS runtime of its own, and "inside a given runtime, no
// multi-threading is supported" (QuickJS's documentation, 3.4.1 Runtime and contexts): two documents' hands (the
// window's and the recipes' previews) each run theirs, under their document's lock.
struct DocScripts { std::map<std::string, DocScript> all; int made = 0; };
static DocScripts& doc_scripts_kept(Doc& d) { if (!d.scripts) d.scripts = std::make_shared<DocScripts>(); return *d.scripts; }
static std::map<std::string, DocScript>& doc_scripts(Doc& d) { return doc_scripts_kept(d).all; }
static int& doc_scripts_made(Doc& d) { return doc_scripts_kept(d).made; }
static const DocShown* doc_shown_of(Doc& d, const char* name) {
    for (const DocShown& one : doc_shown) if (!strcmp(one.name, name)) return &one;
    for (const auto& [file, script] : doc_scripts(d))
        for (const DocShown& one : script.shown) if (!strcmp(one.name, name)) return &one;
    return nullptr;
}
// A whole-screen effect's knob in the document: its file's name before the knob's, as several lie on one stage and two
// may each have a knob of one name (Posterize's gamma and Boxinator's are not one value).
static std::string doc_screen_knob(const std::string& file, const std::string& knob) {   // (a value's name is one word: an ISF file's "Frosted Glass" says Frosted_Glass)
    std::string stem = std::filesystem::path(file).stem().string();
    for (char& c : stem) if (!std::isalnum((unsigned char)c) && c != '_') c = '_';
    return "js_" + stem + "_" + knob;
}
// ThorVG's own effects by the value that puts each on, as a thing's list of effects names them, and their letters
// (DrawnFx::order).
static const char* const doc_thor_names[] = {"tint", "tritone", "shadow", "blur"};   // (a glow is not ThorVG's: Filament's bloom on the thing's own picture, doc_moment)
static char doc_thor_letter(const std::string& w) {
    for (int i = 0; i < 4; ++i) if (w == doc_thor_names[i]) return "trsb"[i];
    return 0;
}
static DocScript* doc_script_of(Doc& d, const std::string& file) {
    if (file.empty() || file == "none" || std::filesystem::path(file).extension() == ".mid") return nullptr;   // (a MIDI file's notes are copies, not a script: midi.h)
    const std::string ext = std::filesystem::path(file).extension().string();
    const bool screen = ext == ".wgsl" || ext == ".fs" || ext == ".mjs" || three_is(file) || solid_is(file) || aviutl_is(file);   // (a three.js effect: as an ISF one, three_isf.cpp; a solid's shape: its knobs named by it too, not run as a script, solid_three.cpp; an AviUtl2 script: run by its own Lua, aviutl_fx.h)
    const bool absent = ext == ".am";   // (AbsentMotion's own motion: its knobs in its head, the thing moved by AbsentMotion's C++: absent_motion.h)
    const bool lottie = ext == ".json" || ext == ".lot";   // (an After Effects file: its slots its knobs, lottie_slots.h)   // (.mjs: a PlayCanvas splat effect, its knobs its attributes: splatfx.h)   // (a whole-screen effect's knobs: named by the effect too)
    DocScript& s = doc_scripts(d)[file];
    if (s.stamp >= 0 && s.seen == files_changed()) return &s;
    s.seen = files_changed();
    std::error_code wrong;
    const long long stamp = (long long)std::filesystem::last_write_time(aviutl_is(file) ? aviutl_path(file) : fx_source(file), wrong).time_since_epoch().count();
    if (wrong) return s.stamp >= 0 ? &s : nullptr;
    if (s.stamp != stamp) {
        s = DocScript{};
        s.viewer = lottie && viewer_is(file);
        s.stamp = stamp, s.seen = files_changed(), s.knobs = ext == ".mjs" ? SplatEffect(file).knobs() : s.viewer ? viewer_knobs(file) : lottie ? lottie_knobs(file) : script_knobs(screen ? isf_file(file) : file);
        if (screen) script_first(file, s.knobs);   // (the values it comes with, where its own show nothing: fx_first.sh)
        std::erase_if(s.knobs, [](const ScriptKnob& k) { return k.kind == "picture" && k.name == "backdrop"; });   // (what is behind it is given, not chosen on its card: screen.h)
        doc_scripts_made(d)++;   // (what was worked out from the old one is forgotten)
        for (const ScriptKnob& k : s.knobs) s.names.push_back(screen || lottie || absent ? doc_screen_knob(file, k.name) : "js_" + k.name);
        for (size_t i = 0; i < s.knobs.size(); ++i) {   // (the names are all made first: what points into them stays put)
            const ScriptKnob& k = s.knobs[i];
            const bool open = (ext == ".wgsl" || ext == ".fs") && k.kind == "track" && k.step < 1.0;   // (a number of an effect, none that counts: an author's range is what its knob shows, not all it can do)
            // (a motion's knobs a camera's too: it takes AbsentMotion's motions, golf ㊴; its strength was refused on it)
            s.values.push_back(DocValue{s.names[i].c_str(), absent ? "film picture text shape solid stage camera" : "film picture text shape solid stage", k.kind == "select" ? 0.0 : k.low, k.kind == "select" ? 0.0 : k.high, k.kind == "track", k.kind == "select" ? k.words.c_str() : nullptr, open});
            s.shown.push_back(DocShown{s.names[i].c_str(), k.kind == "check" ? "onoff" : k.kind == "picture" || k.kind == "sound" ? "thing" : "", "", k.label.c_str(), k.low, k.high, nullptr, open});
        }
        if (absent) {   // (read, not run: which of AbsentMotion's its head says)
            std::ifstream in(file);
            for (std::string line; std::getline(in, line);) if (line.rfind("// --absent:", 0) == 0) s.absent = line.substr(12);
            return &s;
        }
        if (lottie) return &s;   // (read, not run)
        if (!screen) s.run = std::make_unique<Script>(file);   // (a .wgsl or an ISF .fs is drawn, not run: screen.h)
        { std::ifstream in(screen ? isf_file(file) : file); std::stringstream text; text << in.rdbuf(); s.clears = text.str().find("opacity") != std::string::npos; }
    }
    return &s;
}
// A thing's scripts: its one (a path may have spaces) and its own effects (paths with spaces between), or the stage's
// effects, one over another.
static std::string doc_word(Doc& d, int thing, const char* name);
static double doc_number(Doc& d, int thing, const char* name, double at, double otherwise);
static std::vector<DocScript*> doc_scripts_on(Doc& d, int thing) {
    const std::string said = doc_word(d, thing, "script");
    std::vector<DocScript*> all;
    if (thing != 0) {
        if (DocScript* sc = doc_script_of(d, said)) all.push_back(sc);
        for (const std::string& file : script_files(doc_word(d, thing, "fx"))) if (!doc_thor_letter(file)) if (DocScript* sc = doc_script_of(d, file)) all.push_back(sc);
        if (const std::string file = doc_word(d, thing, "file"); doc_word(d, thing, "kind") == "picture")   // (an After Effects file's slots)
            if (const std::string ext = std::filesystem::path(file).extension().string(); ext == ".json" || ext == ".lot")
                if (DocScript* sc = doc_script_of(d, file); sc && !sc->knobs.empty()) all.push_back(sc);
        return all;
    }
    for (const std::string& file : script_files(said)) if (DocScript* sc = doc_script_of(d, file)) all.push_back(sc);
    return all;
}
static std::string doc_fx_title(const std::string& name);
static std::vector<std::string> doc_fx_list(Doc& d, int thing, bool off = false);
// AbsentMotion's own motion effects on a thing (.am files in its fx), in order, each its knobs' values now by the names
// AbsentMotion reads them by: a point's x and y and a colour's x to w one param of numbers in turn (effects/from_absent.py).
static std::vector<AbsentEffect> doc_absent_on(Doc& d, int thing, double now) {
    std::vector<AbsentEffect> all;
    for (const std::string& file : doc_fx_list(d, thing))   // (those turned off left out)
        if (std::filesystem::path(file).extension() == ".am")
            if (DocScript* sc = doc_script_of(d, file); sc && !sc->absent.empty()) {
                AbsentEffect e{sc->absent, {}};
                for (size_t i = 0; i < sc->knobs.size(); ++i) {
                    std::string name = sc->knobs[i].name;
                    const size_t cut = name.rfind('_');
                    const bool part = cut != std::string::npos && cut + 2 == name.size() && std::string("xyzw").find(name[cut + 1]) != std::string::npos;
                    if (part) name = name.substr(0, cut);
                    const float v = (float)doc_number(d, thing, sc->names[i].c_str(), now, sc->knobs[i].usual);
                    if (part && !e.params.empty() && e.params.back().first == name) e.params.back().second.push_back(v);
                    else e.params.push_back({name, {v}});
                }
                all.push_back(e);
            }
    return all;
}
static const DocCard doc_cards[] = {
    // (in the order a hand needs them: first what every thing has, as After Effects' Properties panel keeps Layer Transform at the top for all layer types; then, Alight Motion's Opacity & Blending, how it lies on what is under it; then its own, Text and Paragraph, Shape Properties)
    {"Transform", "space parent anchor_x anchor_y anchor_z x y z scale_x scale_y scale_z rotate_x rotate_y rotate opacity", "kind", "!camera"},   // (After Effects' Transform group: where it is, how large, how turned, how seen; each its own value)
    {"Camera", "parent x y z rotate_x rotate_y rotate distance zoom", "kind", "camera"},
    {"Fog", "fog fog_near fog_far fog_color", "kind", "camera"},   // (the camera's own, as Alight Motion's camera has: its colour, where it begins, where it is whole; the stage's when it says none)   // (After Effects' camera: what it looks at, where it is and how turned, how far its eye is, its zoom)
    {"Blending", "blend clip"},   // (After Effects' layer mode and its track matte, in its simplest: drawn over what lies below as it says, or only inside the one before it)
    {"Text", "text file size letter_spacing line_spacing align", "kind", "text"},   // (After Effects' Source Text and Character, Cavalry's Text Shape: what it says, in which font, how large, which way it lines up)
    {"Star", "points inner", "shape", "star"},   // (After Effects' Polystar: how many points, how far in between them)
    {"Polygon", "points", "shape", "polygon"},
    {"Fill", "fill fill_media stroke stroke_width"},   // (After Effects' Fill and Stroke, Cavalry's Fill and Stroke: a colour, colours down it, or the material glass; its edge and how wide)
    {"Type", "split italic wrap wrap_at", nullptr, nullptr, true},
    {"Gradient", "gradient gradient_angle gradient_scale gradient_spread", "fill", "~ ", true},
    {"Copies", "copies copy_columns copy_x copy_y copy_rotate copy_rotate_x copy_rotate_y copy_turn copy_size copy_opacity_end copy_wide copy_z tall copy_tall stagger stagger_from copy_fill copy_fill_way", nullptr, nullptr, true},
    {"Wave", "wave_x wave_y wave_z wave_rotate wave_size wave_speed wave_phase wave_noise noise_scale", nullptr, nullptr, true},
    {"Field", "field_x field_y field_z field_radius field_band field_size field_rotate field_push", nullptr, nullptr, true},
    {"Scatter", "scatter_x scatter_y scatter_z scatter_rotate scatter_size seed drift_x drift_y", nullptr, nullptr, true},
    {"Shine", "emissive", "kind", "!solid", true},   // (a flat thing lit from within: past 1, the stage's bloom takes it up where it lies in the stage's own view; laid over a solid thing, a picture of its own is clear round it and keeps no halo: ThorVG's glow does that)
    {"Motion", "orient", "kind", "!camera", true},   // (how it goes along its way: turned the way it is going, After Effects' Auto-Orient)
    {"Play", "marker animation speed speed_hold frame_rate", "kind", "!stage", true},   // (a thing's; the stage's own frame rate is said in the size box)
    {"Sound", "volume beats", "kind", "film"},   // (its beats: shown on the timeline's ruler where it strikes)
    {"Matte", "clip_matte clip_invert clip_feather", "clip", "1"},
    {"Parent", "parent_rotate parent_size roll roll_radius", "parent", "!0"},   // (Alight Motion's Parenting Helper: how much of its parent's turn and size it takes, and rolling as a wheel; while it has a parent)
    {"Element", "retime retime_in retime_out", "shape", "null"},   // (a group's, kept with it as an element: how it is played when put in longer or shorter, Alight Motion's Element settings' Re-timing and its marks)
    {"Scale Assist", "scale_assist scale_assist_amount scale_assist_damp", "scale_assist", "!"},   // (Alight Motion's Scale Assist: one way grown alone, or both, kept as bulky as it was; once put on from the Effects shelf, as its effect is: looks/Scale Assist.motolii)
    {"Connect", "connect connect_to connect_reach", nullptr, nullptr, true},
    {"Line", "cap join dash dash_offset heads head_shape head_start head_size", nullptr, nullptr, true},   // (and its open ends' shapes: Cavalry's heads, Alight Motion's line start and end)
    {"Path", "round bloat offset zigzag zigzag_ridges zigzag_smooth", "kind", "shape", true},   // (its path made anew: After Effects' shape operators, ThorVG's own)
    {"Edit Shape", "crop_left crop_top crop_right crop_bottom round", "kind", "picture"},   // (a photo's rect, Alight Motion's Edit Shape, AbsentMotion's Shape Options: cut in from its sides, what is left where it was, and its corners rounded; drawn as a rect its own file fills, doc_match)
    {"Deform", "warp bend twist pinch skew squash wiggle stretch bend_direction pin_tl_x pin_tl_y pin_tr_x pin_tr_y pin_bl_x pin_bl_y pin_br_x pin_br_y warp_width warp_angle warp_speed skew_axis wiggle_detail wiggle_speed", "kind", "shape", true},   // (its points moved as it is drawn: Cavalry's deformers)
    {"Surface", "shading metallic roughness reflectance clear_coat clear_coat_roughness sheen anisotropy thickness emissive", "fill", "!glass", true},
    {"Glass", "ior dispersion swim dark vivid", "fill", "glass"},
    {"Lit", "light_group", "kind", "solid", true},   // (which lights reach it)
    {"Light", "intensity falloff cone shadows shadow_softness contact_shadows light_group sun_size sun_halo", "kind", "light"},
    {"Projection", "projection shift_x shift_y"},
    {"Grade", "tone_mapper exposure contrast saturation vibrance temperature slope offset power shadows_color midtones_color highlights_color", nullptr, nullptr, true},
    {"Lens", "bloom ambient_occlusion reflections lens_flare vignette vignette_color fog fog_near fog_far fog_color anti_aliasing", "kind", "stage"},   // (the stage's; a camera's own fog is on its Fog card)
    {"Room", "room environment environment_rotate behind background"},
    {"Physics", "physics bounce friction gravity", nullptr, nullptr, true},
    {"Around", "strength radius offset tilt leader centre_x centre_y rotate rotate_y lag", "file", "around"},   // (a relation's card: its things round a middle, or round a leader; turned in turn, lag apart)
    {"Row", "strength gap centre_x centre_y rotate rotate_y lag", "file", "row"},
    {"Follow", "strength leader lag leader_b hand", "file", "follow"},
    {"Push", "strength leader radius", "file", "push"},
    {"Group", "strength lag x y spread scatter rotate rotate_y size opacity", "file", "group"},   // (things in an order and nothing else: away by its own move, turn, size and opacity, each in turn)
    {"Note", "label note", nullptr, nullptr, true},   // (what a maker wrote of it: After Effects' layer comment)
    {"Trim Paths", "trim trim_begin trim_offset", "trim", "+"},
    {"Datamosh", "mosh_with mosh_at", "mosh", "+"},
    {"Points", "dissolve grain", "file", "~.ply"},   // (a point cloud's own: points.h)   // (the film's own compressed pictures, mosh.h)   // (After Effects' shape operator: added from the shelf)
    {"Gaussian Blur", "blur blur_direction", "blur", "+"},   // (the effects (doc_effects), each its card while it is on, as After Effects' Effect Controls)
    {"Drop Shadow", "shadow shadow_distance shadow_angle shadow_blur shadow_opacity", "shadow", "+"},
    {"Glow", "glow_opacity", "glow", "+"},   // (how strong: Filament's bloom on its own picture, doc_moment)
    {"Duotone", "tint tint_black tint_white", "tint", "+"},
    {"Gradient Map", "tritone tritone_shadow tritone_midtone tritone_highlight", "tritone", "+"},
};
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
    d.phys_stale = true, d.scripted.clear();   // (the world made again only if what it is made of changed: doc_physics)
    d.moving.clear(), d.literals.clear(), d.words.clear(), d.keyed.clear(), d.halves.clear(), d.heardWords.clear(), d.talls.clear();
    d.named.clear(), d.tracks.clear(), d.drivers.clear(), d.connects.clear(), d.relationsOf.clear();
    d.laid.clear();
}
// The things in the order they lie, the one furthest back first: by `order` where it is said, else as they were made.
static const std::vector<int>& doc_laid(Doc& d) {
    if (d.laid.empty())
        doc_rows(d, sqlite3_mprintf("select f.thing from fact f where f.name = 'kind' order by coalesce((select o.value from fact o where o.thing = f.thing and o.name = 'order' and o.at = 0), f.thing), f.thing"),
                 [&](sqlite3_stmt* row) { d.laid.push_back(sqlite3_column_int(row, 0)); });
    return d.laid;
}

// A way's expression made ready for SQLite to work out (nullptr: SQLite does not take it; d.trouble says why).
static sqlite3_stmt* doc_goes(Doc& d, const std::string& expression) {
    auto ready = d.goes.find(expression);
    if (ready != d.goes.end()) return ready->second;
    sqlite3_stmt* made = nullptr;
    char* asked = sqlite3_mprintf("with v(u, t, d, a, b, p1, p2, p3, p4) as (select ?, ?, ?, ?, ?, ?, ?, ?, ?) select (%s) from v", expression.c_str());
    if (sqlite3_prepare_v2(d.file, asked, -1, &made, nullptr) != SQLITE_OK) d.trouble = sqlite3_errmsg(d.file), made = nullptr;
    sqlite3_free(asked);
    return made ? d.goes[expression] = made : nullptr;
}
static double doc_went(sqlite3_stmt* way, double u, double t, double length, double a, double b, const double* p) {
    const double told[] = {u, t, length, a, b, p[0], p[1], p[2], p[3]};
    for (int i = 0; i < 9; ++i) sqlite3_bind_double(way, i + 1, told[i]);
    const double is = sqlite3_step(way) == SQLITE_ROW ? sqlite3_column_double(way, 0) : a;
    sqlite3_reset(way);
    return is;
}
static sqlite3_stmt* doc_way_named(Doc& d, const std::string& name) {
    auto known = d.named.find(name);
    if (known != d.named.end()) return known->second;
    sqlite3_stmt* way = nullptr;
    doc_rows(d, sqlite3_mprintf("select goes from way where name = %Q", name.c_str()), [&](sqlite3_stmt* row) { way = doc_goes(d, (const char*)sqlite3_column_text(row, 0)); });
    if (!way)   // (not defined here: one of the ways by name, ways.h)
        for (const auto& one : named_ways) if (name == one.name) way = doc_goes(d, std::string("a + (b - a) * (") + one.goes + ")");
    return d.named[name] = way;
}
// A stretch that goes a way defined in the document (`own`: its name, then its numbers): the value at `u` of the way
// through, `t` seconds in, between a key of `a` and a key of `b`, `length` seconds apart (no such way: `a`).
static double doc_went_by(Doc& d, const std::string& own, double u, double t, double length, double a, double b) {
    std::istringstream said(own);
    std::string way;
    double p[4] = {0, 0, 0, 0};
    said >> way;
    for (double& one : p) said >> one;
    sqlite3_stmt* goes = doc_way_named(d, way);
    return goes ? doc_went(goes, u, t, length, a, b, p) : a;
}

// A value at a moment: `otherwise` where the document does not say; where it has keys, the value the stretch it is in
// gives (before the first key and after the last: theirs). A stretch goes the way said for it (not said: Linear).
static double doc_keys_at(Doc& d, const Doc::Keys& k, double at, double otherwise);
// Where a value's keys are kept (read from the facts once, as doc_number reads them): kept until the document changes.
static const Doc::Keys& doc_keys_of(Doc& d, int thing, const char* name);
static std::string doc_word(Doc& d, int thing, const char* name);
// The element put in that a thing is held in (a group above it put in from an element, doc_element_add): 0 none.
static int doc_element_above(Doc& d, int thing) {
    for (int p = thing, n = 0; p && n < 64; ++n) {
        const Doc::Keys& up = doc_keys_of(d, p, "parent");
        p = up.value.empty() ? 0 : (int)up.value.front();
        if (const Doc::Keys& was = doc_keys_of(d, p, "element_length"); p && !was.value.empty() && was.value.front() > 0.0) return p;
    }
    return 0;
}
// The moment of itself an element put in shows at `at`, as long as it is now against as long as it was: its intro and
// outro as they were, the middle held, stretched, looped or looped and stretched between, or nothing after its end
// (Alight Motion's Re-timing; AbsentMotion's NestedClock, NestedScenes.kt, its at(): Apache-2.0, Copyright 2026 The
// AbsentMotion authors; in seconds, not rounded to the middles of frames). -1e9: not seen then (Blank).
static double doc_element_clock(Doc& d, int el, double at) {
    const double origin = doc_keys_at(d, doc_keys_of(d, el, "start"), 0, 0), scene_end = doc_keys_at(d, doc_keys_of(d, el, "element_length"), 0, 0);
    const double lasts = doc_keys_at(d, doc_keys_of(d, el, "duration"), 0, -1), layer_end = lasts > 0.0 ? lasts : scene_end;
    const std::string method = doc_word(d, el, "retime");
    const double local = at - origin;
    if (method == "blank") return local < scene_end ? at : -1e9;
    const double intro = std::clamp(doc_keys_at(d, doc_keys_of(d, el, "retime_in"), 0, 0), 0.0, scene_end), out = doc_keys_at(d, doc_keys_of(d, el, "retime_out"), 0, 0);
    const double outro = out > 0.0 ? std::clamp(out, intro, scene_end) : scene_end, outro_start = layer_end - (scene_end - outro), room = outro_start - intro, span = outro - intro;
    double mapped = local;
    if (local >= outro_start) mapped = outro + local - outro_start;
    else if (local >= intro) {
        const double into = local - intro;
        if (method == "stretch" && room > 0.0) mapped = intro + into * span / room;
        else if (method == "loop") mapped = intro + (span > 0.0 ? std::fmod(into, span) : 0.0);
        else if (method == "loop_stretch" && span > 0.0 && room > 0.0) {
            const double each = room / std::max(1.0, std::round(room / span));
            mapped = intro + std::fmod(into, each) * span / each;
        } else mapped = intro + std::min(into, span);   // (Freeze, as an element is made)
    }
    return origin + mapped;
}
// The moment a thing is at, as the element put in that holds it shows it (none: the stage's own).
static double doc_element_now(Doc& d, int thing, double at) {
    const int el = thing ? doc_element_above(d, thing) : 0;
    return el ? doc_element_clock(d, el, at) : at;
}
// Time Quantization (Alight Motion's; After Effects' Posterize Time): a thing with a frame_rate is seen only at its own
// frames, counted from its start: a value that moves is read at the frame the moment is in. 0: as smooth as the stage.
// (Held in an element put in, first the moment the element shows: doc_element_now.)
static double doc_stepped(Doc& d, int thing, const Doc::Keys& k, double at) {
    if (k.at.size() < 2 || !thing) return at;
    at = doc_element_now(d, thing, at);
    const Doc::Keys& rate = doc_keys_of(d, thing, "frame_rate");
    if (rate.value.empty() || !(rate.value.front() > 0.0)) return at;
    const Doc::Keys& from = doc_keys_of(d, thing, "start");
    const double start = from.value.empty() ? 0.0 : from.value.front(), r = rate.value.front();
    return at <= start ? at : start + std::floor((at - start) * r + 1e-9) / r;
}
static double doc_number(Doc& d, int thing, const char* name, double at, double otherwise) {
    if (d.held.thing == thing)   // (what a hand is carrying shows in place of the document's own)
        if (const auto tried = d.held.tried.find(name); tried != d.held.tried.end()) return tried->second;
    const Doc::Keys& k = doc_keys_of(d, thing, name);
    return doc_keys_at(d, k, doc_stepped(d, thing, k, at), otherwise);
}
static const Doc::Keys& doc_keys_of(Doc& d, int thing, const char* name) {
    auto& of = d.moving[thing];
    auto found = of.find(std::string_view(name));
    if (found == of.end()) {
        Doc::Keys keys;
        doc_rows(d, sqlite3_mprintf("select at, value, coalesce(way, '') from fact where thing = %d and name = %Q order by at", thing, name), [&](sqlite3_stmt* row) {
            keys.at.push_back(sqlite3_column_double(row, 0));
            keys.value.push_back(sqlite3_column_double(row, 1));
            const std::string way = (const char*)sqlite3_column_text(row, 2);
            Interp kind;
            const bool of_the_nine = interp_said(way.empty() ? "Linear" : way, &kind) == 0;   // (not said: Linear, as a new key is in the past repository)
            keys.interp.push_back(kind);
            keys.own.push_back(of_the_nine ? "" : way);
        });
        if (keys.at.size() > 1) keys.loop = doc_word(d, thing, (std::string(name) + "_loop").c_str());   // (the keys gone through again past the last: doc_keys_at)
        found = d.moving[thing].emplace(std::string(name), std::move(keys)).first;
    }
    return found->second;
}
// A value from its keys at a moment (doc_number's own reckoning: before the first key and after the last, theirs).
static double doc_keys_at(Doc& d, const Doc::Keys& k, double at, double otherwise) {
    if (k.at.empty()) return otherwise;
    if (k.at.size() == 1 || at <= k.at.front()) return k.value.front();
    // (past the last key, gone through again: After Effects' loopOut, its cycle, pingpong and offset, on all its keys;
    //  whatever the value is, as the way of a stretch is. offset: each time on from where the last ended)
    double on = 0.0;
    if (at > k.at.back() && (k.loop == "cycle" || k.loop == "pingpong" || k.loop == "offset")) {
        const double span = k.at.back() - k.at.front(), rounds = std::floor((at - k.at.front()) / span), left = at - k.at.front() - rounds * span;
        at = k.loop == "pingpong" && std::fmod(rounds, 2.0) != 0.0 ? k.at.back() - left : k.at.front() + left;
        if (k.loop == "offset") on = rounds * (k.value.back() - k.value.front());
    }
    if (at <= k.at.front()) return k.value.front() + on;
    if (at >= k.at.back()) return k.value.back() + on;
    const size_t i = (size_t)(std::upper_bound(k.at.begin(), k.at.end(), at) - k.at.begin()) - 1;   // (the stretch it is in begins at key i)
    const double u = (at - k.at[i]) / (k.at[i + 1] - k.at[i]);
    if (k.own[i].empty()) return k.value[i] + (k.value[i + 1] - k.value[i]) * interp_ease(k.interp[i], u) + on;
    return doc_went_by(d, k.own[i], u, at - k.at[i], k.at[i + 1] - k.at[i], k.value[i], k.value[i + 1]) + on;
}
static std::string doc_word(Doc& d, int thing, const char* name) {
    if (thing == 0 && !strcmp(name, "kind")) return "stage";   // (thing 0 is the stage: its values are the whole picture's)
    if (d.held.thing == thing)   // (what a hand is carrying shows in place of the document's own)
        if (const auto said = d.held.said.find(name); said != d.held.said.end()) return said->second;
    const auto known = d.words.find({thing, name});   // (asked again and again between changes: the document asked once)
    if (known != d.words.end()) return known->second;
    std::string is;
    doc_rows(d, sqlite3_mprintf("select value from fact where thing = %d and name = %Q", thing, name), [&](sqlite3_stmt* row) { is = (const char*)sqlite3_column_text(row, 0); });
    d.words.emplace(std::make_pair(thing, std::string(name)), is);
    return is;
}

// Every look of a thing as it is on the stage (a sheet's one; a kit shape's, as many as it is drawn by; every face of a 3D file).
// A way of lying on what is under that reads what is under (the W3C's compositing, After Effects' and ThorVG's): its
// number in flat.mat's blendWay (0: one the GPU's own blending does: normal, add, multiply, screen).
static int doc_blend_way(const std::string& word) {
    static const char* const ways[] = {"darken", "lighten", "overlay", "soft_light", "hard_light", "color_dodge", "color_burn", "difference", "exclusion", "hue", "saturation", "color", "luminosity",
                                       "linear_burn", "linear_light", "vivid_light", "pin_light", "darker_color", "lighter_color", "subtract", "divide"};
    for (int i = 0; i < 21; ++i) if (word == ways[i]) return i + 1;
    return 0;
}
static void doc_looks(Stage& s, const Doc::Shown& t, const std::function<void(MaterialInstance*)>& each) {
    auto& faces = s.engine->getRenderableManager();
    if (t.film) each(t.film->look);
    if (t.drawn) each(t.drawn->look);
    for (const Stage::Made& made : s.made)
        if (t.shape && made.of == t.shape) each(faces.getMaterialInstanceAt(faces.getInstance(made.thing), 0));
    if (t.solid) for (const utils::Entity e : t.solid->parts) {
        const auto one = faces.getInstance(e);
        for (size_t part = 0; part < faces.getPrimitiveCount(one); ++part) each(faces.getMaterialInstanceAt(one, part));
    }
}
// Each drawn part of a thing (Filament's renderables), and the scene it is in (its run's: run 0 or not yet put, the stage's own).
static void doc_entities(Stage& s, const Doc::Shown& t, const std::function<void(utils::Entity)>& each) {
    const Mesh* m = t.film ? t.film->sheet : t.drawn ? t.drawn->sheet : t.shape;
    for (const Stage::Made& made : s.made)
        if (m && made.of == m && !made.matte) each(made.thing);
    if (t.solid) for (const utils::Entity e : t.solid->parts) each(e);
}
static void doc_parts(Stage& s, const Doc::Shown& t, const std::function<void(RenderableManager&, RenderableManager::Instance)>& each) {
    auto& faces = s.engine->getRenderableManager();
    doc_entities(s, t, [&](utils::Entity e) { each(faces, faces.getInstance(e)); });
}
static bool doc_covers(Doc& d, int thing, double now, double* box);
static bool doc_seen_box(Doc& d, int thing, double now, double* box);
static int doc_outline(Doc& d, int thing, double now, double* xy);
static bool doc_half(Doc& d, int thing, float2* half, float* deep = nullptr);
// Where a thing is at a moment, as a matrix: moved, turned about z, y and x, made its size, all about its anchor (a
// point of its own box: 0 its middle, 1 its edge on that side; the past repository's `anchor`, keyed as it was).
static math::mat4 doc_place(Doc& d, int thing, double now, int deeper = 0, int copy = 0, bool member = false);
static math::mat4 doc_shown_at(Doc& d, int thing, double now);
int sk_along(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, int n, float from, std::vector<float>& out);
static double doc_point(Doc& d, int thing, double now);
static math::double2 doc_seen(Doc& d, math::double3 p);
static math::mat4 doc_above(Doc& d, int thing, double now, int deeper);
static bool doc_match(Doc& d);
static double doc_seconds(Doc& d);
// A thing's copies given back its own look, their own looks done away with (no longer coloured each, or taken off).
// What a behaviour gives the nth of n copies at a moment (Cavalry's stagger, oscillator, noise and random, its names):
// from its minimum to its maximum.
static int doc_driven_by(Doc& d, int thing, const char* name);
