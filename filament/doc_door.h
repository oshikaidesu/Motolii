// doc_door.h: part of doc.h. Code taken from the net, read as what it is (door.h), kept in the maker's own folder as it
// came (so it is on the shelf too) and put on the stage. word: the code, or a file of it. A shader that reads a picture
// goes on the picked thing, as an effect; one that makes its own fills the stage; a sketch is a picture, in 3D a solid.
static int doc_door(Doc& d, const std::string& word, int picked) {
    std::string text = word;
    std::error_code wrong;
    if (word.size() < 1024 && std::filesystem::is_regular_file(word, wrong)) {
        std::ifstream in(word);
        std::stringstream all;
        all << in.rdbuf();
        text = all.str();
    }
    const std::filesystem::path folder = doc_folders("programs").back();
    const bool kept = std::filesystem::path(word).parent_path() == folder;   // (one of the Program shelf's own, as it is now: not kept again)
    text = door_page(text);
    const std::string came = text, form = door_form(text), told = door_js(text, form);
    const bool hydra = form == "hydra", shader = hydra || form == "shadertoy" || form == "sandbox";
    std::vector<HydraKnob> moving;
    if (hydra) {   // (the code as an ISF effect, kept as that: Hydra is a part apart, hydra_isf.cpp)
        std::string why, isf;
        if (!hydra_isf(doc_folders("extensions/hydra").back().string(), text, 30.0, isf, moving, why)) return d.trouble = why, 0;
        text = isf;
    }
    if (!shader && !script_is_sketch(told)) return d.trouble = "this is not code that Motolii can read", 0;
    uint64_t hash = 1469598103934665603ull;
    for (unsigned char c : text) hash = (hash ^ c) * 1099511628211ull;
    char tail[16];
    std::snprintf(tail, sizeof tail, "-%04llx", (unsigned long long)(hash & 0xffff));
    static const std::map<std::string, std::string> names = {{"", "p5.js"}, {"shadertoy", "Shadertoy"}, {"sandbox", "GLSL Sandbox"}, {"three", "three.js"}, {"hydra", "Hydra"}, {"dweet", "Dwitter"}, {"tixy", "tixy"}, {"turtle", "Turtletoy"}, {"two", "Two.js"}, {"paper", "Paper.js"}, {"canvas", "Canvas"}};
    const std::string name = names.at(form);
    std::filesystem::create_directories(folder, wrong);
    std::filesystem::path file = folder / (name + tail + (shader ? ".fs" : ".js"));
    if (kept && !hydra) file = word;
    else if (!std::filesystem::exists(file, wrong)) std::ofstream(file) << text;
    if (hydra) std::ofstream(file.parent_path() / (file.stem().string() + ".hydra")) << came;   // (what was written, kept beside what it became: the Program shelf shows this one)
    doc_act(d);
    int thing = 0;
    if (shader && (door_has(text, "iChannel") || (hydra && door_has(text, "inputImage"))) && picked) {
        if (doc_fx_file(d, picked, file.string().c_str(), d.stage->now)) thing = picked;
    } else if (shader) {
        if ((thing = doc_add(d, "shape", "rect")) && doc_say(d, thing, "name", name.c_str()) && doc_fx_file(d, thing, file.string().c_str(), d.stage->now)) doc_fit(d, thing, "stretch");
    } else if (form == "three") {   // (each pair of a shape and a material a solid of its own, under a group: stood in the world, not a picture of it)
        Script run(file.string(), (doc_folders("third_party/js/p5").front() / "p5.min.js").string());
        const int parts = std::atoi(run.said("__three_parts()").c_str());
        if (!run.ok()) return d.trouble = run.trouble(), d.act_ok = false, doc_acted(d), 0;
        if (!parts) return d.trouble = "this three.js scene drew no meshes", d.act_ok = false, doc_acted(d), 0;
        const bool together = d.act_together;
        d.act_together = true;   // (matched once at the end: each of a scene's parts is an act of its own otherwise, and each matches them all)
        if (parts > 1 && (thing = doc_add(d, "shape", "null"))) doc_say(d, thing, "name", name.c_str());
        for (int p = 0; p < parts; ++p) {
            const std::vector<uint8_t> shape = run.bytes("__three_glb(" + std::to_string(p) + ")");
            const std::filesystem::path glb = folder / (name + tail + "-" + std::to_string(p) + ".glb");
            std::ofstream out(glb, std::ios::binary);
            out.write((const char*)shape.data(), (std::streamsize)shape.size());
            out.close();
            std::istringstream says(run.said("__three_says(" + std::to_string(p) + ")"));
            int most = 1;
            double metal = 0, rough = 1;
            std::string colour;
            says >> most >> colour >> metal >> rough;
            const int solid = doc_add(d, "solid", glb.string().c_str());
            if (!solid) break;
            if (!thing) thing = solid;
            doc_say(d, solid, "name", (name + " " + std::to_string(p + 1)).c_str()), doc_say(d, solid, "fill", colour.c_str());
            doc_set(d, solid, "metallic", metal), doc_set(d, solid, "roughness", rough), doc_set(d, solid, "copies", std::min(most, 1000));
            doc_say(d, solid, "script", file.string().c_str()), doc_set(d, solid, "js_part", p);
            if (parts > 1) doc_set(d, solid, "parent", thing);
        }
        d.act_together = together;
    } else if (told.find("WEBGL") != std::string::npos) {
        if ((thing = doc_add(d, "solid", "cube"))) doc_say(d, thing, "name", name.c_str()), doc_say(d, thing, "script", file.string().c_str());
    } else if ((thing = doc_add(d, "picture", file.string().c_str()))) doc_say(d, thing, "name", name.c_str()), doc_fit(d, thing, "fit");
    if (thing && !moving.empty()) {   // (what moved in its code: keys, where it can be edited)
        doc_moment(d, d.stage->now);
        for (const HydraKnob& k : moving)
            for (size_t i = 0; i < k.keys.size(); ++i) doc_key(d, thing, ("js_" + k.name).c_str(), (double)i * HYDRA_STEP, k.keys[i]);
    }
    return doc_acted(d) ? thing : 0;
}
